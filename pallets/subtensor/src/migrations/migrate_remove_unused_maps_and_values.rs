use super::*;
use crate::HasMigrationRun;
use frame_support::{traits::Get, weights::Weight};
use scale_info::prelude::string::String;
use sp_io::{
    KillStorageResult,
    hashing::twox_128,
    storage::{clear, clear_prefix},
};

fn remove_prefix<T: Config>(old_map: &str, weight: &mut Weight) {
    let mut prefix = Vec::new();
    prefix.extend_from_slice(&twox_128("SubtensorModule".as_bytes()));
    prefix.extend_from_slice(&twox_128(old_map.as_bytes()));

    let removal_results = clear_prefix(&prefix, Some(u32::MAX));

    let removed_entries_count = match removal_results {
        KillStorageResult::AllRemoved(removed) => removed as u64,
        KillStorageResult::SomeRemaining(removed) => {
            log::info!("Failed To Remove Some Items During migration");
            removed as u64
        }
    };

    log::info!("Removed {removed_entries_count:?} entries from {old_map:?} map.");

    *weight = (*weight).saturating_add(T::DbWeight::get().writes(removed_entries_count));
}

pub fn migrate_remove_unused_maps_and_values<T: Config>() -> Weight {
    let migration_name = b"migrate_remove_unused_maps_and_values".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

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

    // Remove EmissionValues entries
    remove_prefix::<T>("EmissionValues", &mut weight);

    // Remove NetworkMaxStake
    remove_prefix::<T>("NetworkMaxStake", &mut weight);

    // Remove SubnetLimit
    clear(b"SubtensorModule::SubnetLimit");

    // Mark Migration as Completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
