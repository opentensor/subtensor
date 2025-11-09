use super::*;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

use subtensor_swap_interface::SwapHandler;

pub fn get_unactive_sn_netuids<T: Config>(
    pool_initial_alpha: AlphaCurrency,
) -> (Vec<NetUid>, Weight) {
    // Loop over all subnets, if the AlphaIssuance is > pool_initial_alpha
    // but FirstEmissionBlockNumber is None
    // then this subnet should be reset
    let mut weight = T::DbWeight::get().reads(1);
    let unactive_netuids = Pallet::<T>::get_all_subnet_netuids()
        .iter()
        .filter(|&netuid| !netuid.is_root())
        .filter(|&netuid| {
            let alpha_issuance = Pallet::<T>::get_alpha_issuance(*netuid);
            let first_emission_block_number = FirstEmissionBlockNumber::<T>::get(*netuid);
            alpha_issuance != pool_initial_alpha && first_emission_block_number.is_none()
        })
        .copied()
        .collect::<Vec<_>>();
    weight = weight
        .saturating_add(T::DbWeight::get().reads(unactive_netuids.len().saturating_mul(3) as u64));

    (unactive_netuids, weight)
}

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

    let (unactive_netuids, w) = get_unactive_sn_netuids::<T>(pool_initial_alpha);
    weight = weight.saturating_add(w);

    // Collect the hotkeys to remove for each subnet
    let mut to_remove_alpha_hotkeys: BTreeMap<NetUid, Vec<T::AccountId>> = BTreeMap::new();
    let mut to_remove_alpha_coldkeys: BTreeMap<NetUid, Vec<(T::AccountId, T::AccountId)>> =
        BTreeMap::new();
    let mut all_hotkeys_set = BTreeSet::new();
    for (hotkey, netuid_i, _) in TotalHotkeyAlpha::<T>::iter() {
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        if unactive_netuids.contains(&netuid_i) {
            // Only for unactive subnets
            to_remove_alpha_hotkeys
                .entry(netuid_i)
                .or_insert(Vec::new())
                .push(hotkey.clone());
            all_hotkeys_set.insert(hotkey);
        }
    }

    // Collect the coldkeys to remove for each subnet
    for hotkey in all_hotkeys_set.iter() {
        for ((coldkey, netuid_i), _) in Alpha::<T>::iter_prefix((&hotkey,)) {
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
            if unactive_netuids.contains(&netuid_i) {
                // Only for unactive subnets
                to_remove_alpha_coldkeys
                    .entry(netuid_i)
                    .or_insert(Vec::new())
                    .push((hotkey.clone(), coldkey));
            }
        }
    }

    for netuid in unactive_netuids.iter() {
        // Reset the subnet as it shouldn't have any emissions
        PendingServerEmission::<T>::remove(*netuid);
        PendingValidatorEmission::<T>::remove(*netuid);
        PendingRootAlphaDivs::<T>::remove(*netuid);
        PendingOwnerCut::<T>::remove(*netuid);
        SubnetTaoInEmission::<T>::remove(*netuid);
        SubnetAlphaInEmission::<T>::remove(*netuid);
        SubnetAlphaOutEmission::<T>::remove(*netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(7));

        // Reset pool
        let actual_tao_lock_amount = SubnetLocked::<T>::get(*netuid);
        let actual_tao_lock_amount_less_pool_tao =
            actual_tao_lock_amount.saturating_sub(pool_initial_tao);
        weight = weight.saturating_add(T::DbWeight::get().reads(1));

        // Reset v3 pool
        let burned_tao = match T::SwapInterface::clear_protocol_liquidity(*netuid) {
            Ok((_tao, fee_tao, _alpha, _fee_alpha)) => fee_tao,
            Err(e) => {
                log::error!("Failed to clear protocol liquidity for netuid {netuid:?}: {e:?}");
                TaoCurrency::ZERO
            }
        };
        Pallet::<T>::recycle_tao(burned_tao);
        // might be based on ticks but this is a rough estimate
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(6, 14));

        // Recycle already emitted TAO
        let subnet_tao = SubnetTAO::<T>::get(*netuid);
        if subnet_tao > pool_initial_tao {
            let tao_to_recycle = subnet_tao.saturating_sub(pool_initial_tao);
            Pallet::<T>::recycle_tao(tao_to_recycle);
            TotalStake::<T>::mutate(|total| {
                *total = total.saturating_sub(tao_to_recycle);
            });
            SubnetTAO::<T>::mutate(*netuid, |amount| {
                *amount = amount.saturating_sub(tao_to_recycle);
            });
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(3, 3));
        }

        // Reset pool alpha
        SubnetAlphaIn::<T>::insert(*netuid, pool_initial_alpha);
        SubnetAlphaOut::<T>::insert(*netuid, AlphaCurrency::ZERO);
        // Reset volume
        SubnetVolume::<T>::insert(*netuid, 0u128);
        // Reset recycled (from init_new_network)
        RAORecycledForRegistration::<T>::insert(*netuid, actual_tao_lock_amount_less_pool_tao);
        weight = weight.saturating_add(T::DbWeight::get().writes(4));

        // Reset Alpha stake entries for this subnet
        let to_reset: Vec<T::AccountId> = match to_remove_alpha_hotkeys.get(netuid) {
            Some(hotkeys) => hotkeys.clone(),
            None => Vec::new(),
        };

        for hotkey in to_reset {
            TotalHotkeyAlpha::<T>::remove(&hotkey, *netuid);
            TotalHotkeyShares::<T>::remove(&hotkey, *netuid);
            TotalHotkeyAlphaLastEpoch::<T>::remove(&hotkey, *netuid);
            weight = weight.saturating_add(T::DbWeight::get().writes(3));

            // Reset root claimable and claimed
            RootClaimable::<T>::mutate(&hotkey, |claimable| {
                claimable.remove(netuid);
            });
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            let removal_result = RootClaimed::<T>::clear_prefix((*netuid, &hotkey), u32::MAX, None);
            weight = weight.saturating_add(
                T::DbWeight::get()
                    .reads_writes(removal_result.loops as u64, removal_result.backend as u64),
            );

            let to_reset_alpha: Vec<(T::AccountId, T::AccountId)> =
                match to_remove_alpha_coldkeys.get(netuid) {
                    Some(coldkeys) => coldkeys.clone(),
                    None => Vec::new(),
                };
            for (hotkey, coldkey) in to_reset_alpha {
                Alpha::<T>::remove((hotkey, coldkey, netuid));
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
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
