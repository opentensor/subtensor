use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;
use share_pool::{SafeFloat, SafeFloatSerializable};

pub fn migrate_share_pool_high_precision<T: Config>() -> Weight {
    let migration_name = b"migrate_share_pool_high_precision".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    // ------------------------------
    // Step 0: Check if already run
    // ------------------------------
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    // ------------------------------
    // Step 1: Migrate TotalHotkeyShares -> TotalHotkeySharesV2
    // ------------------------------

    let mut migrated_ths_entries_count = 0u64;

    for (hotkey, netuid, shares) in TotalHotkeyShares::<T>::iter() {
        TotalHotkeyShares::<T>::remove(&hotkey, netuid);

        if shares != 0 {
            let ths_safe_float: SafeFloat = shares.into();
            let ths_safe_float_serializable: SafeFloatSerializable = (&ths_safe_float).into();
            TotalHotkeySharesV2::<T>::insert(hotkey, netuid, ths_safe_float_serializable);

            migrated_ths_entries_count = migrated_ths_entries_count.saturating_add(1);
        }
    }

    weight = weight.saturating_add(T::DbWeight::get().reads(migrated_ths_entries_count));
    weight = weight.saturating_add(T::DbWeight::get().writes(migrated_ths_entries_count));

    log::info!("Migrated {migrated_ths_entries_count} entries from TotalHotkeyShares.");

    // ------------------------------
    // Step 2: Migrate Alpha -> AlphaV2
    // ------------------------------

    let mut migrated_alpha_entries_count = 0u64;

    for ((hotkey, coldkey, netuid), alpha) in Alpha::<T>::iter() {
        Alpha::<T>::remove((&hotkey, &coldkey, netuid));

        if alpha != 0 {
            let alpha_safe_float: SafeFloat = alpha.into();
            let alpha_safe_float_serializable: SafeFloatSerializable = (&alpha_safe_float).into();
            AlphaV2::<T>::insert((hotkey, coldkey, netuid), alpha_safe_float_serializable);

            migrated_alpha_entries_count = migrated_alpha_entries_count.saturating_add(1);
        }
    }

    weight = weight.saturating_add(T::DbWeight::get().reads(migrated_alpha_entries_count));
    weight = weight.saturating_add(T::DbWeight::get().writes(migrated_alpha_entries_count));

    log::info!("Migrated {migrated_alpha_entries_count} entries from Alpha.");

    // ------------------------------
    // Step 3: Mark Migration as Completed
    // ------------------------------
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
