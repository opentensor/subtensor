use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

pub fn migrate_network_lock_reduction_interval<T: Config>() -> Weight {
    const NEW_VALUE: u64 = 28_800;

    let migration_name = b"migrate_network_lock_reduction_interval".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    // Skip if already executed
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{}' already run - skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    // ── 1) Set new values ─────────────────────────────────────────────────
    NetworkLockReductionInterval::<T>::put(NEW_VALUE);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    NetworkRateLimit::<T>::put(NEW_VALUE);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    Pallet::<T>::set_network_last_lock(TaoCurrency::from(1_000_000_000_000));
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    Pallet::<T>::set_network_last_lock_block(Pallet::<T>::get_current_block_as_u64());
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // ── 2) Mark migration done ───────────────────────────────────────────
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed - NetworkLockReductionInterval => {}.",
        String::from_utf8_lossy(&migration_name),
        NEW_VALUE
    );

    weight
}
