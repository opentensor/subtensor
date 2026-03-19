use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

pub fn migrate_remove_zero_alpha<T: Config>() -> Weight {
    let migration_name = b"migrate_remove_zero_alpha".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    // ------------------------------
    // Step 0: Check if already run
    // ------------------------------
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    // ------------------------------
    // Step 1: Remove zero entries in Alpha (StorageNMap, value type: U64F64)
    // ------------------------------
    let mut removed_alpha = 0u64;

    for ((hotkey, coldkey, netuid), value) in Alpha::<T>::iter() {
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        if value == 0 {
            Alpha::<T>::remove((&hotkey, &coldkey, netuid));
            removed_alpha = removed_alpha.saturating_add(1);
        }
    }
    weight = weight.saturating_add(T::DbWeight::get().writes(removed_alpha));
    log::info!("Removed {removed_alpha} zero entries from Alpha.");

    // ------------------------------
    // Step 2: Remove zero entries in TotalHotkeyShares (value type: U64F64)
    // ------------------------------
    let mut removed_shares = 0u64;

    for (hotkey, netuid, value) in TotalHotkeyShares::<T>::iter() {
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        if value == 0 {
            TotalHotkeyShares::<T>::remove(&hotkey, netuid);
            removed_shares = removed_shares.saturating_add(1);
        }
    }
    weight = weight.saturating_add(T::DbWeight::get().writes(removed_shares));
    log::info!("Removed {removed_shares} zero entries from TotalHotkeyShares.");

    // ------------------------------
    // Step 3: Remove zero entries in TotalHotkeyAlphaLastEpoch (value type: AlphaBalance)
    // ------------------------------
    let mut removed_last_epoch = 0u64;

    for (hotkey, netuid, value) in TotalHotkeyAlphaLastEpoch::<T>::iter() {
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        if value.is_zero() {
            TotalHotkeyAlphaLastEpoch::<T>::remove(&hotkey, netuid);
            removed_last_epoch = removed_last_epoch.saturating_add(1);
        }
    }
    weight = weight.saturating_add(T::DbWeight::get().writes(removed_last_epoch));
    log::info!("Removed {removed_last_epoch} zero entries from TotalHotkeyAlphaLastEpoch.");

    // ------------------------------
    // Step 4: Remove zero entries in AlphaDividendsPerSubnet (value type: AlphaBalance)
    // ------------------------------
    let mut removed_dividends = 0u64;

    for (netuid, hotkey, value) in AlphaDividendsPerSubnet::<T>::iter() {
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        if value.is_zero() {
            AlphaDividendsPerSubnet::<T>::remove(netuid, &hotkey);
            removed_dividends = removed_dividends.saturating_add(1);
        }
    }
    weight = weight.saturating_add(T::DbWeight::get().writes(removed_dividends));
    log::info!("Removed {removed_dividends} zero entries from AlphaDividendsPerSubnet.");

    // ------------------------------
    // Step 5: Mark Migration as Completed
    // ------------------------------
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{}' completed. Removed: Alpha={removed_alpha}, TotalHotkeyShares={removed_shares}, TotalHotkeyAlphaLastEpoch={removed_last_epoch}, AlphaDividendsPerSubnet={removed_dividends}",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
