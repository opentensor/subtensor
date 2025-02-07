use alloc::string::String;

use frame_support::IterableStorageMap;
use frame_support::{traits::Get, weights::Weight};

use super::*;

pub fn migrate_set_min_lock_cost<T: Config>() -> Weight {
    let migration_name = b"migrate_set_min_lock_cost".to_vec();

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

    // Set min lock cost to 1 TAO
    NetworkMinLockCost::<T>::put(1_000_000_000);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

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
