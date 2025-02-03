use super::*;
use alloc::string::String;
use frame_support::IterableStorageMap;
use frame_support::{traits::Get, weights::Weight};
use log;
use sp_runtime::format;
use substrate_fixed::types::U64F64;

fn do_migration_<T: Config>(coldkey: T::AccountId) -> Weight {
    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

	// Get all the


}

pub fn migrate_add_staking_coldkeys<T: Config>() -> Weight {
    let migration_name = b"migrate_add_staking_coldkeys".to_vec();

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

	weight = weight.saturating_add(do_migration_<T>(coldkey));

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
