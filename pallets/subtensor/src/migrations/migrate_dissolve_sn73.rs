use alloc::string::String;

use frame_support::{traits::Get, weights::Weight};
use substrate_fixed::types::U64F64;

use super::*;

pub fn migrate_dissolve_sn73<T: Config>() -> Weight {
    let migration_name = b"migrate_dissolve_sn73".to_vec();
    let this_netuid = 73;

    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    // Check if the migration has already run
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            migration_name
        );
        return weight;
    }
    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    if NetworksAdded::<T>::get(this_netuid) {
        // Subnet exists, skip
        log::info!("Subnet was already added, skipping");
    } else {
        // ======== Migration Logic ========

        // Get the subnet TAO
        let subnet_tao = U64F64::from_num(SubnetTAO::<T>::get(this_netuid));
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        log::debug!("Subnet TAO: {}", subnet_tao);

        // Adjust total stake and total issuance
        TotalStake::<T>::mutate(|total| {
            *total = total.saturating_sub(subnet_tao.saturating_to_num::<u64>());
        });
        TotalIssuance::<T>::mutate(|total| {
            *total = total.saturating_sub(subnet_tao.saturating_to_num::<u64>());
        });
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));

        // Record for total issuance tracking
        let mut total_swapped: u64 = 0;

        let mut total_alpha: U64F64 = U64F64::from_num(0);
        // Iterate over every hotkey and sum up the total alpha
        let mut hotkeys_to_remove: Vec<T::AccountId> = Vec::new();
        for (hotkey, netuid_i, total_hotkey_alpha) in TotalHotkeyAlpha::<T>::iter() {
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
            if netuid_i != this_netuid {
                continue;
            }

            hotkeys_to_remove.push(hotkey);
            total_alpha = total_alpha.saturating_add(U64F64::from_num(total_hotkey_alpha));
        }
        log::debug!("Total alpha: {}", total_alpha);

        // Iterate over every hotkey and distribute the TAO from the pool
        // using previous total alpha as the denominator
        for hotkey in hotkeys_to_remove.iter() {
            log::debug!("Hotkey: {:?}", hotkey.clone());

            let total_hotkey_alpha_i = TotalHotkeyAlpha::<T>::get(hotkey.clone(), this_netuid);
            let total_hotkey_alpha = U64F64::from_num(total_hotkey_alpha_i);
            weight = weight.saturating_add(T::DbWeight::get().reads(1));

            // Get the total hotkey shares
            let total_hotkey_shares =
                U64F64::from_num(TotalHotkeyShares::<T>::get(hotkey.clone(), this_netuid));
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
            log::debug!("Total hotkey shares: {}", total_hotkey_shares);

            // Get the equivalent amount of TAO
            let hotkey_tao: U64F64 = total_hotkey_alpha
                .saturating_div(total_alpha)
                .saturating_mul(subnet_tao);
            log::debug!("Total hotkey alpha: {}", total_hotkey_alpha);
            log::debug!("Hotkey TAO: {}", hotkey_tao);

            let mut coldkeys_to_remove: Vec<T::AccountId> = Vec::new();
            // Distribute the TAO to each of the stakers to the hotkey
            for ((coldkey, netuid_i), alpha_i) in Alpha::<T>::iter_prefix((&hotkey,)) {
                weight = weight.saturating_add(T::DbWeight::get().reads(1));
                if netuid_i != this_netuid {
                    continue;
                }

                coldkeys_to_remove.push(coldkey.clone());

                let alpha_shares = U64F64::from_num(alpha_i);
                let coldkey_share: U64F64 = alpha_shares.saturating_div(total_hotkey_shares);

                let coldkey_tao = coldkey_share.saturating_mul(hotkey_tao);
                let coldkey_alpha = coldkey_share.saturating_mul(total_hotkey_alpha);
                log::debug!("Alpha shares: {}", alpha_shares);
                log::debug!("Coldkey share: {}", coldkey_share);
                log::debug!("Coldkey TAO: {}", coldkey_tao);

                // Distribute the TAO to the coldkey
                let as_tao: u64 = coldkey_tao.saturating_to_num::<u64>();
                let as_alpha: u64 = coldkey_alpha.saturating_to_num::<u64>();

                if as_tao > 0 {
                    Pallet::<T>::add_balance_to_coldkey_account(&coldkey, as_tao);
                    total_swapped = total_swapped.saturating_add(as_tao);
                    weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

                    // Emit event
                    Pallet::<T>::deposit_event(Event::StakeRemoved(
                        coldkey.clone(),
                        hotkey.clone(),
                        as_tao,
                        as_alpha,
                        this_netuid,
                    ));
                }
            }
            // Clear coldkeys
            for coldkey in coldkeys_to_remove {
                Alpha::<T>::remove((&hotkey, coldkey, this_netuid));
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }

        // Update total issuance
        TotalIssuance::<T>::mutate(|v| *v = v.saturating_add(total_swapped));
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        // Verify total issuance change is correct
        if subnet_tao
            .saturating_to_num::<u64>()
            .abs_diff(total_swapped)
            >= 100_000
        {
            log::info!(
                "Total issuance change is incorrect: {} != {}",
                subnet_tao.saturating_to_num::<u64>(),
                total_swapped
            );
            if cfg!(feature = "try-runtime") {
                assert!(
                    subnet_tao
                        .saturating_to_num::<u64>()
                        .abs_diff(total_swapped)
                        <= 100_000
                );
            }
        }

        // === Clear storage entries ===
        // Clear subnet owner and hotkey
        SubnetOwner::<T>::remove(this_netuid);
        SubnetOwnerHotkey::<T>::remove(this_netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(2));

        // Clear hotkeys
        for hotkey in hotkeys_to_remove {
            TotalHotkeyAlpha::<T>::remove(hotkey.clone(), this_netuid);
            TotalHotkeyShares::<T>::remove(hotkey.clone(), this_netuid);
            weight = weight.saturating_add(T::DbWeight::get().writes(2));
        }

        // Clear pool
        SubnetTAO::<T>::remove(this_netuid);
        SubnetAlphaIn::<T>::remove(this_netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(2));

        // Clear AlphaOut
        SubnetAlphaOut::<T>::remove(this_netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));

        // Clear pending emissions
        SubnetTaoInEmission::<T>::remove(this_netuid);
        SubnetAlphaInEmission::<T>::remove(this_netuid);
        SubnetAlphaOutEmission::<T>::remove(this_netuid);
        PendingEmission::<T>::remove(this_netuid);
        PendingRootDivs::<T>::remove(this_netuid);
        PendingAlphaSwapped::<T>::remove(this_netuid);
        PendingOwnerCut::<T>::remove(this_netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(7));

        // Clear trackers
        let clear_results_0 =
            AlphaDividendsPerSubnet::<T>::clear_prefix(this_netuid, u32::MAX, None);
        weight = weight.saturating_add(T::DbWeight::get().writes(clear_results_0.unique.into()));
        let clear_results_1 = TaoDividendsPerSubnet::<T>::clear_prefix(this_netuid, u32::MAX, None);
        weight = weight.saturating_add(T::DbWeight::get().writes(clear_results_1.unique.into()));

        // Clear subnet volume
        SubnetVolume::<T>::remove(this_netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));

        // Clear child keys
        let clear_results_2 = PendingChildKeys::<T>::clear_prefix(this_netuid, u32::MAX, None);
        weight = weight.saturating_add(T::DbWeight::get().writes(clear_results_2.unique.into()));

        let mut childkeys_to_remove: Vec<T::AccountId> = Vec::new();
        for (childkey, netuid_i, _parents) in ParentKeys::<T>::iter() {
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
            if netuid_i != this_netuid {
                continue;
            }

            childkeys_to_remove.push(childkey);
        }

        let mut parent_keys_to_remove: Vec<T::AccountId> = Vec::new();
        for (parent_key, netuid_i, _children) in ChildKeys::<T>::iter() {
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
            if netuid_i != this_netuid {
                continue;
            }

            parent_keys_to_remove.push(parent_key);
        }

        for child_key in childkeys_to_remove {
            ParentKeys::<T>::remove(child_key, this_netuid);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
        }

        for parent_key in parent_keys_to_remove {
            ChildKeys::<T>::remove(parent_key, this_netuid);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
        }

        // Clear reg allowed maps
        NetworkRegistrationAllowed::<T>::remove(this_netuid);
        NetworkPowRegistrationAllowed::<T>::remove(this_netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(2));
        // ======== End Migration Logic ========
    }

    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed.",
        String::from_utf8_lossy(&migration_name)
    );

    // Return the migration weight.
    weight
}
