use super::*;
use frame_support::IterableStorageMap;
use frame_support::weights::Weight;
use log;
use scale_info::prelude::{string::String, vec::Vec};

pub fn migrate_tao_reserves_at_last_block<T: Config>() -> Weight {
    use frame_support::traits::Get;
    let migration_name = b"migrate_tao_reserves_at_last_block".to_vec();

    // Start counting weight
    let mut weight = T::DbWeight::get().reads(1);

    // Check if we already ran this migration
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        target: "runtime",
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );
    // -------------------------------------
    // 1) Migrate TAO reserves at last block
    // -------------------------------------
    let current_reserves_vec: Vec<u64> = <SubnetTAO<T> as IterableStorageMap<NetUid, u64>>::iter()
        .filter(|(netuid, _)| !netuid.is_root())
        .map(|(_, reserves)| reserves)
        .collect();
    let current_reserves: u64 = current_reserves_vec.iter().sum();
    let current_reserves_len = current_reserves_vec.len() as u64;

    weight = weight
        .saturating_add(T::DbWeight::get().reads_writes(current_reserves_len.saturating_add(1), 0));

    TaoReservesAtLastBlock::<T>::set(current_reserves);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // -----------------------------
    // Mark the migration as done
    // -----------------------------
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
