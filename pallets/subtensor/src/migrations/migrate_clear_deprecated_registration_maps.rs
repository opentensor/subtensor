use super::*;
use crate::AccountIdOf;
use frame_support::{
    IterableStorageMap,
    pallet_prelude::{Blake2_128Concat, OptionQuery},
    storage_alias,
    traits::Get,
    weights::Weight,
};
use scale_info::prelude::string::String;

/// Clears deprecated registration-related storage items after moving to the new
/// BurnHalfLife/BurnIncreaseMult model.
///
/// This migration is **idempotent** via `HasMigrationRun`.
///
/// Why it “takes into account root_register”:
/// - `root_register()` still relies on `RegistrationsThisInterval`/`RegistrationsThisBlock`.
/// - We **do not reset** those counters here.
/// - We set `BurnLastHalvingBlock(NetUid::ROOT)` to “now” so your new per-interval reset logic
///   does **not** retroactively wipe root’s interval counter (which could temporarily allow extra
///   root registrations).
/// - We migrate the old `AdjustmentInterval` → `BurnHalfLife` for **all** networks (including ROOT),
///   preserving the prior interval length semantics.
///
/// Deprecated maps cleared:
/// - PoW path: `UsedWork`, `Difficulty`, `MinDifficulty`, `MaxDifficulty`, `NetworkPowRegistrationAllowed`
/// - Old reg accounting: `POWRegistrationsThisInterval`, `BurnRegistrationsThisInterval`
/// - Old adjustment system: `AdjustmentAlpha`, `AdjustmentInterval`, `LastAdjustmentBlock`
pub fn migrate_clear_deprecated_registration_maps<T: Config>() -> Weight {
    const RAO_PER_TAO: u64 = 1_000_000_000;
    const ONE_TAO_RAO: u64 = 1 * RAO_PER_TAO;
    const DEFAULT_BURN_INCREASE_MULT: u64 = 2;

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

    // Use the current block, but ensure it’s non-zero
    let current_block = Pallet::<T>::get_current_block_as_u64();
    let block_to_set = if current_block == 0 { 1 } else { current_block };

    // --- 1) Initialize new pricing params for *all* networks (including ROOT)
    // - BurnHalfLife replaces AdjustmentInterval; migrate old value.
    // - BurnIncreaseMult defaults to 2.
    // - BurnLastHalvingBlock set to "now" to prevent retroactive halving/interval resets.
    //
    // We do NOT touch RegistrationsThisInterval/RegistrationsThisBlock here.
    let mut networks_seen: u64 = 0;

    for (netuid, added) in NetworksAdded::<T>::iter() {
        if !added {
            continue;
        }
        networks_seen = networks_seen.saturating_add(1);

        // 1.a) Migrate old AdjustmentInterval -> BurnHalfLife (guard against 0).
        let old_interval: u16 = AdjustmentInterval::<T>::get(netuid);
        weight = weight.saturating_add(T::DbWeight::get().reads(1));

        let new_half_life: u16 = old_interval.max(1);
        BurnHalfLife::<T>::insert(netuid, new_half_life);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));

        // 1.b) Set BurnIncreaseMult default.
        BurnIncreaseMult::<T>::insert(netuid, DEFAULT_BURN_INCREASE_MULT.max(1));
        weight = weight.saturating_add(T::DbWeight::get().writes(1));

        // 1.c) Start halving schedule "now".
        BurnLastHalvingBlock::<T>::insert(netuid, block_to_set);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));

        // 1.d) Ensure burn is non-zero on non-root nets so multiplier logic works.
        if netuid != NetUid::ROOT {
            let burn_u64: u64 = Pallet::<T>::get_burn(netuid).into();
            weight = weight.saturating_add(T::DbWeight::get().reads(1));

            if burn_u64 == 0 {
                Pallet::<T>::set_burn(netuid, TaoCurrency::from(ONE_TAO_RAO));
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }
    }

    // Account for the NetworksAdded iteration itself.
    weight = weight.saturating_add(T::DbWeight::get().reads(networks_seen));

    // --- 2) Clear deprecated/unused maps

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
    clear_map_and_log!(UsedWork, "UsedWork");
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

    // Burn bounds (deprecated, NOT part of new spec)
    clear_map_and_log!(MinBurn, "MinBurn");
    clear_map_and_log!(MaxBurn, "MaxBurn");

    // --- 3) Mark migration done
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed at block {}. Initialized BurnHalfLife/BurnIncreaseMult/BurnLastHalvingBlock for {} networks and cleared deprecated maps (root_register preserved).",
        String::from_utf8_lossy(&migration_name),
        block_to_set,
        networks_seen
    );

    weight
}
