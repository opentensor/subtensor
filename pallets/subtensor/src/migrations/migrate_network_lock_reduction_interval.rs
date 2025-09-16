use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

pub fn migrate_network_lock_reduction_interval<T: Config>() -> Weight {
    const NEW_VALUE: u64 = 28_800;
    const ONE_WEEK_BLOCKS: u64 = 50_400;

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

    let current_block = Pallet::<T>::get_current_block_as_u64();

    // ── 1) Set new values ─────────────────────────────────────────────────
    NetworkLockReductionInterval::<T>::put(NEW_VALUE);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    NetworkRateLimit::<T>::put(NEW_VALUE);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    Pallet::<T>::set_network_last_lock(TaoCurrency::from(1_000_000_000_000));
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Hold price at 2000 TAO until day 7, then begin linear decay
    Pallet::<T>::set_network_last_lock_block(current_block.saturating_add(ONE_WEEK_BLOCKS));
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Allow registrations starting at day 7
    NetworkRegistrationStartBlock::<T>::put(current_block.saturating_add(ONE_WEEK_BLOCKS));
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // ── 2) Mark migration done ───────────────────────────────────────────
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed - NetworkLockReductionInterval & NetworkRateLimit => {}. \
         last_lock set to 1_000_000_000_000 rao; last_lock_block/start_block => {}.",
        String::from_utf8_lossy(&migration_name),
        NEW_VALUE,
        current_block.saturating_add(ONE_WEEK_BLOCKS),
    );

    weight
}
