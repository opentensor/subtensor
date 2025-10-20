use super::*;
use frame_support::{storage::IterableStorageMap, traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

pub fn migrate_kappa_map_to_default<T: Config>() -> Weight {
    let mig_name: Vec<u8> = b"kappa_map_to_default".to_vec();

    // 1 read for the HasMigrationRun flag
    let mut total_weight = T::DbWeight::get().reads(1);

    // Run once guard
    if HasMigrationRun::<T>::get(&mig_name) {
        log::info!(
            "Migration '{}' already executed - skipping",
            String::from_utf8_lossy(&mig_name)
        );
        return total_weight;
    }

    log::info!("Running migration '{}'", String::from_utf8_lossy(&mig_name));

    let target: u16 = DefaultKappa::<T>::get();

    let mut reads: u64 = 0;
    let mut writes: u64 = 0;
    let mut visited: u64 = 0;
    let mut updated: u64 = 0;
    let mut unchanged: u64 = 0;

    for (netuid, current) in Kappa::<T>::iter() {
        visited += 1;
        reads += 1;

        if current != target {
            Kappa::<T>::insert(netuid, target);
            writes += 1;
            updated += 1;
        } else {
            unchanged += 1;
        }
    }

    total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(reads, writes));

    log::info!(
        "Kappa migration summary: visited={visited}, updated={updated}, unchanged={unchanged}, target_default={}",
        target
    );

    HasMigrationRun::<T>::insert(&mig_name, true);
    total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{}' completed",
        String::from_utf8_lossy(&mig_name)
    );

    total_weight
}
