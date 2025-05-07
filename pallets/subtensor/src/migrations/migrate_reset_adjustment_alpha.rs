use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

pub fn migrate_reset_adjustment_alpha<T: Config>() -> Weight {
    let migration_name = b"migrate_reset_adjustment_alpha".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    // ------------------------------
    // Step 0: Check if already run
    // ------------------------------
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

    // ------------------------------
    // Step 1: Reset all subnet's adjustment alpha to 0.5 if the value exceeds 0.5
    // ------------------------------

    let mut reset_entries_count = 0u64;

    for netuid in AdjustmentAlpha::<T>::iter_keys() {
        AdjustmentAlpha::<T>::mutate(netuid, |adjustment| {
            *adjustment = (*adjustment).min(32768); // 0.5
        });
        reset_entries_count = reset_entries_count.saturating_add(1);
    }

    weight = weight
        .saturating_add(T::DbWeight::get().reads_writes(reset_entries_count, reset_entries_count));

    log::info!(
        "Reset {} subnets from AdjustmentAlpha.",
        reset_entries_count
    );

    // ------------------------------
    // Step 2: Mark Migration as Completed
    // ------------------------------
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
