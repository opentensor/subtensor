use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;

pub fn migrate_init_tao_flows<T: Config>() -> Weight {
    let mig_name: Vec<u8> = b"migrate_init_tao_flows".to_vec();
    let mut total_weight = T::DbWeight::get().reads(1);

    // run once
    if HasMigrationRun::<T>::get(&mig_name) {
        log::info!(
            "Migration '{}' already executed - skipping",
            String::from_utf8_lossy(&mig_name)
        );
        return total_weight;
    }
    log::info!("Running migration '{}'", String::from_utf8_lossy(&mig_name));

    let mut curr = SubnetEmaTaoFlow::<T>::clear(u32::MAX, None);
    total_weight = total_weight
        .saturating_add(T::DbWeight::get().reads_writes(curr.loops as u64, curr.unique as u64));
    while curr.maybe_cursor.is_some() {
        // Clear until empty
        curr = SubnetEmaTaoFlow::<T>::clear(u32::MAX, curr.maybe_cursor.as_deref());
        total_weight = total_weight
            .saturating_add(T::DbWeight::get().reads_writes(curr.loops as u64, curr.unique as u64));
    }

    // mark as done
    HasMigrationRun::<T>::insert(&mig_name, true);
    total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{}' completed",
        String::from_utf8_lossy(&mig_name)
    );
    total_weight
}
