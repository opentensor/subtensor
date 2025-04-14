use super::*;
use crate::HasMigrationRun;
use frame_support::{traits::Get, weights::Weight};
use sp_io::{KillStorageResult, hashing::twox_128, storage::clear_prefix};

pub fn migrate_remove_total_hotkey_coldkey_stakes_this_interval<T: Config>() -> Weight {
    let migration_name = "migrate_remove_total_hotkey_coldkey_stakes_this_interval";
    let migration_name_bytes = migration_name.as_bytes().to_vec();

    let mut weight = T::DbWeight::get().reads(1);
    if HasMigrationRun::<T>::get(&migration_name_bytes) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            migration_name
        );
        return weight;
    }

    log::info!("Running migration '{}'", migration_name);

    let pallet_name = twox_128(b"SubtensorModule");
    let storage_name = twox_128(b"TotalHotkeyColdkeyStakesThisInterval");
    let prefix = [pallet_name, storage_name].concat();

    // Try to remove all entries from the storage, if some entries are remaining,
    // the migration will re-run again on next blocks until all entries are removed.
    let removed_entries_count = match clear_prefix(&prefix, Some(u32::MAX)) {
        KillStorageResult::AllRemoved(removed) => {
            // Mark migration as completed
            HasMigrationRun::<T>::insert(&migration_name_bytes, true);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));

            log::info!(
                "Migration '{:?}' completed successfully. {:?} entries removed.",
                migration_name,
                removed
            );
            removed
        }
        KillStorageResult::SomeRemaining(removed) => {
            log::info!(
                "Migration '{:?}' completed partially. {:?} entries removed.",
                migration_name,
                removed
            );
            removed
        }
    };

    weight.saturating_add(T::DbWeight::get().writes(removed_entries_count as u64))
}
