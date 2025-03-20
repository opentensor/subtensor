use super::*;
use frame_support::weights::Weight;
use log;
use scale_info::prelude::string::String;

pub fn migrate_reset_bonds<T: Config>() -> Weight {
    use frame_support::traits::Get;
    let migration_name = b"migrate_reset_bonds".to_vec();

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

    // ===== Migration Body =====
    // Clear all bonds
    let mut curr = Bonds::<T>::clear(u32::MAX, None);
    weight = weight
        .saturating_add(T::DbWeight::get().reads_writes(curr.loops as u64, curr.unique as u64));
    while curr.maybe_cursor.is_some() {
        curr = Bonds::<T>::clear(u32::MAX, curr.maybe_cursor.as_deref());
        weight = weight
            .saturating_add(T::DbWeight::get().reads_writes(curr.loops as u64, curr.unique as u64));
    }

    // ===== Migration End =====
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
