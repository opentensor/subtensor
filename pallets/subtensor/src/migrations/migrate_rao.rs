use super::*;
use alloc::string::String;
use frame_support::IterableStorageMap;
use frame_support::{traits::Get, weights::Weight};
use log;
use crate::subnets::symbols::get_symbol_for_subnet;

pub fn migrate_rao<T: Config>() -> Weight {
    let migration_name = b"migrate_rao".to_vec();

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

    let netuids: Vec<u16> = <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter()
        .map(|(netuid, _)| netuid)
        .collect();
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(netuids.len() as u64, 0));

    // Migrate all TAO to root.
    Stake::<T>::iter().for_each(|(hotkey, coldkey, stake)| {
        // Increase SubnetTAO on root.
        SubnetTAO::<T>::mutate(0, |total| {
            *total = total.saturating_add(stake);
        });
        // Increase SubnetAlphaOut on root.
        SubnetAlphaOut::<T>::mutate(0, |total| {
            *total = total.saturating_add(stake);
        });
        // Set all the stake on root 0 subnet.
        Alpha::<T>::mutate((hotkey.clone(), coldkey.clone(), 0), |total| {
            *total = total.saturating_add(stake)
        });
        TotalHotkeyShares::<T>::mutate(hotkey.clone(), 0, |total| {
            *total = total.saturating_add(stake)
        });
        // Set the total stake on the hotkey
        TotalHotkeyAlpha::<T>::mutate(hotkey.clone(), 0, |total| {
            *total = total.saturating_add(stake)
        });
        // 6 reads and 6 writes.
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(6, 6));
    });

    // Convert subnets and give them lock.
    for netuid in netuids.iter().clone() {
        if *netuid == 0 {
            // Give root a single RAO in pool to avoid any catestrophic division by zero.
            SubnetAlphaIn::<T>::insert(netuid, 1);
            SubnetMechanism::<T>::insert(netuid, 0); // Set to zero mechanism.
            TokenSymbol::<T>::insert(netuid, get_symbol_for_subnet(*netuid));
            continue;
        }
        let owner: T::AccountId = SubnetOwner::<T>::get(netuid);
        let lock: u64 = SubnetLocked::<T>::get(netuid);
        let initial_liquidity: u64 = 100_000_000_000; // 100 TAO.
        let remaining_lock: u64 = lock.saturating_sub( initial_liquidity );
        Pallet::<T>::add_balance_to_coldkey_account(&owner, remaining_lock);
        SubnetTAO::<T>::insert(netuid, initial_liquidity); // Set TAO to the lock.
        SubnetAlphaIn::<T>::insert(netuid, initial_liquidity); // Set AlphaIn to the initial alpha distribution.
        SubnetAlphaOut::<T>::insert(netuid, 0); // Set zero subnet alpha out.
        SubnetMechanism::<T>::insert(netuid, 1); // Convert to dynamic immediately with initialization.
        Tempo::<T>::insert(netuid, DefaultTempo::<T>::get());
        // Set global weight to 1.8% from the start
        TaoWeight::<T>::insert(netuid, 320_413_933_267_719_290);
        // Set the token symbol for this subnet using Self instead of Pallet::<T>
        TokenSymbol::<T>::insert(netuid, get_symbol_for_subnet(*netuid));

        // HotkeyEmissionTempo::<T>::put(30); // same as subnet tempo // (DEPRECATED)
        // Set the target stakes per interval to 10.
        // TargetStakesPerInterval::<T>::put(10); (DEPRECATED)
    }

    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed. Storage version set to 7.",
        String::from_utf8_lossy(&migration_name)
    );

    // Return the migration weight.
    weight
}
