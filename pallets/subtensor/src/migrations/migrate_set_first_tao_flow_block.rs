use super::*;
use crate::HasMigrationRun;
use frame_support::{traits::Get, weights::Weight};
use scale_info::prelude::string::String;

pub fn migrate_set_first_tao_flow_block<T: Config>() -> Weight {
    let migration_name = b"migrate_set_first_tao_flow_block".to_vec();

    let mut weight = T::DbWeight::get().reads(1);
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        "Running migration '{:?}'",
        String::from_utf8_lossy(&migration_name)
    );

    // Actual migration
    let current_block = Pallet::<T>::get_current_block_as_u64();
    FlowFirstBlock::<T>::set(Some(current_block));
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

    // Mark Migration as Completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().reads(2));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
