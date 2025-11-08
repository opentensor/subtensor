use super::*;

pub fn migrate_reset_unactive_sn<T: Config>() -> Weight {
    let migration_name = b"migrate_reset_unactive_sn".to_vec();
    let mut weight: Weight = T::DbWeight::get().reads(1);

    // Skip if already executed
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{}' already run - skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }
    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    // From init_new_network
    let pool_initial_tao: TaoCurrency = Pallet::<T>::get_network_min_lock();
    let pool_initial_alpha: AlphaCurrency = pool_initial_tao.to_u64().into();

    // Loop over all subnets, if the AlphaIssuance is > pool_initial_alpha
    // but FirstEmissionBlockNumber is None
    // then we reset the subnet
    for netuid in Pallet::<T>::get_all_subnet_netuids()
        .iter()
        .filter(|&netuid| !netuid.is_root())
    {
        let alpha_issuance = Pallet::<T>::get_alpha_issuance(*netuid);
        let first_emission_block_number = FirstEmissionBlockNumber::<T>::get(*netuid);
        weight = weight.saturating_add(T::DbWeight::get().reads(3));
        if alpha_issuance != pool_initial_alpha && first_emission_block_number.is_none() {
            // Reset the subnet as it shouldn't have any emissions
            PendingServerEmission::<T>::remove(*netuid);
            PendingValidatorEmission::<T>::remove(*netuid);
            PendingRootAlphaDivs::<T>::remove(*netuid);
            PendingOwnerCut::<T>::remove(*netuid);

            // Reset pool

            // Recycle already emitted TAO
            let subnet_tao = SubnetTAO::<T>::get(*netuid);
            if subnet_tao > pool_initial_tao {
                Pallet::<T>::recycle_tao(subnet_tao.saturating_sub(pool_initial_tao));
            }
            SubnetTAO::<T>::insert(*netuid, pool_initial_tao);

            // Reset pool alpha
            SubnetAlphaIn::<T>::insert(*netuid, pool_initial_alpha);
            SubnetAlphaOut::<T>::insert(*netuid, AlphaCurrency::ZERO);
            // Reset volume
            SubnetVolume::<T>::insert(*netuid, 0u128);

            // Reset Alpha stake entries for this subnet
            let mut to_reset = Vec::new();
            for (hotkey, _, alpha) in
                TotalHotkeyAlpha::<T>::iter().filter(|(_, netuid_, _)| *netuid_ == *netuid)
            {
                if alpha > AlphaCurrency::from(0) {
                    to_reset.push((hotkey, netuid, alpha));
                }
            }
            for (hotkey, netuid_, _) in to_reset {
                TotalHotkeyAlpha::<T>::remove(&hotkey, netuid_);
                TotalHotkeyShares::<T>::remove(&hotkey, netuid_);
                TotalHotkeyAlphaLastEpoch::<T>::remove(&hotkey, netuid_);

                // Reset root claimable and claimed
                RootClaimable::<T>::mutate(&hotkey, |claimable| {
                    claimable.remove(netuid_);
                });
                let _ = RootClaimed::<T>::clear_prefix((netuid_, &hotkey), u32::MAX, None);

                let mut to_reset_alpha: Vec<(&T::AccountId, T::AccountId, NetUid)> = Vec::new();
                for ((coldkey, _), _) in Alpha::<T>::iter_prefix((&hotkey,))
                    .filter(|((_, netuid_), _)| *netuid_ == *netuid)
                {
                    to_reset_alpha.push((&hotkey, coldkey, *netuid_));
                }
                for (hotkey, coldkey, netuid_) in to_reset_alpha {
                    Alpha::<T>::remove((hotkey, coldkey, netuid_));
                }
            }
        }
    }

    // Mark Migration as Completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
