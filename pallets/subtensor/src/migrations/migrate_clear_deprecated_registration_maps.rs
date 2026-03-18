use super::*;
use frame_support::{
    traits::Get,
    weights::Weight,
};
use scale_info::prelude::string::String;

pub fn migrate_clear_deprecated_registration_maps<T: Config>() -> Weight {
    let migration_name = b"migrate_clear_deprecated_registration_maps_v1".to_vec();
    let mut weight: Weight = T::DbWeight::get().reads(1);

    // --- 0) Skip if already executed
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{}' already run - skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    // --- 1) Clear deprecated.
    macro_rules! clear_map_and_log {
        ($map:ident, $label:expr) => {{
            let res = $map::<T>::clear(u32::MAX, None);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
            if res.maybe_cursor.is_some() {
                log::warn!(
                    target: "runtime",
                    "Migration '{}' - '{}' not fully cleared (cursor present).",
                    String::from_utf8_lossy(&migration_name),
                    $label
                );
            } else {
                log::info!(
                    target: "runtime",
                    "Migration '{}' - cleared '{}'.",
                    String::from_utf8_lossy(&migration_name),
                    $label
                );
            }
        }};
    }

    // PoW path (deprecated)
    clear_map_and_log!(Difficulty, "Difficulty");
    clear_map_and_log!(MinDifficulty, "MinDifficulty");
    clear_map_and_log!(MaxDifficulty, "MaxDifficulty");
    clear_map_and_log!(
        NetworkPowRegistrationAllowed,
        "NetworkPowRegistrationAllowed"
    );

    // Old per-interval tracking (deprecated)
    clear_map_and_log!(POWRegistrationsThisInterval, "POWRegistrationsThisInterval");
    clear_map_and_log!(
        BurnRegistrationsThisInterval,
        "BurnRegistrationsThisInterval"
    );

    // Old adjustment mechanism (deprecated)
    clear_map_and_log!(AdjustmentAlpha, "AdjustmentAlpha");
    clear_map_and_log!(AdjustmentInterval, "AdjustmentInterval");
    clear_map_and_log!(LastAdjustmentBlock, "LastAdjustmentBlock");

    // Burn bounds (deprecated, not part of the new model)
    clear_map_and_log!(MinBurn, "MinBurn");
    clear_map_and_log!(MaxBurn, "MaxBurn");

    // --- 2) Mark migration done
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed. Cleared deprecated registration maps only; new-model storage left untouched.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}