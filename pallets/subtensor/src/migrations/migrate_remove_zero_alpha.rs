use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

/// The migration name used for the `HasMigrationRun` guard.
const MIGRATION_NAME: &[u8] = b"migrate_remove_zero_alpha_v2";

/// Called from `on_runtime_upgrade`. Schedules the cleanup by setting phase = 1
/// if the migration hasn't run yet. This is O(1) — no iteration.
pub fn migrate_remove_zero_alpha<T: Config>() -> Weight {
    let migration_name = MIGRATION_NAME.to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{}' already completed. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    // Schedule the cleanup to run in on_idle by setting phase to 1
    ZeroAlphaCleanupPhase::<T>::put(1u8);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{}' scheduled. Will clean up zero entries via on_idle.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}

/// Called from `on_idle` each block. Uses `remaining_weight` to dynamically
/// bound how many entries to process. Stays on the same phase until all entries
/// in that map are cleaned, then advances to the next phase.
///
/// Phases:
///   0 = inactive/complete
///   1 = cleaning Alpha
///   2 = cleaning AlphaV2
///   3 = cleaning TotalHotkeyShares
///   4 = cleaning TotalHotkeySharesV2
///   5 = cleaning TotalHotkeyAlphaLastEpoch
///   6 = cleaning AlphaDividendsPerSubnet
pub fn on_idle_remove_zero_alpha<T: Config>(remaining_weight: Weight) -> Weight {
    let phase = ZeroAlphaCleanupPhase::<T>::get();

    // Phase 0 means not active or already completed
    if phase == 0 {
        return Weight::zero();
    }

    // Minimum weight needed: 1 read (phase) + at least one iteration (read + write)
    let min_weight = T::DbWeight::get().reads_writes(2, 1);
    if remaining_weight.ref_time() < min_weight.ref_time() {
        return Weight::zero();
    }

    let mut weight = T::DbWeight::get().reads(1); // reading phase

    // Budget for batch work = remaining_weight minus overhead (phase read + phase write)
    let overhead = T::DbWeight::get().reads_writes(1, 1);
    let budget = remaining_weight.saturating_sub(overhead);

    match phase {
        1 => {
            let (consumed, removed, done) = clean_alpha_batch::<T>(budget);
            weight = weight.saturating_add(consumed);
            log::info!(
                "Zero-alpha cleanup phase 1 (Alpha): removed {removed} zero entries this batch. Done: {done}"
            );
            if done {
                ZeroAlphaCleanupPhase::<T>::put(2u8);
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }
        2 => {
            let (consumed, removed, done) = clean_alpha_v2_batch::<T>(budget);
            weight = weight.saturating_add(consumed);
            log::info!(
                "Zero-alpha cleanup phase 2 (AlphaV2): removed {removed} zero entries this batch. Done: {done}"
            );
            if done {
                ZeroAlphaCleanupPhase::<T>::put(3u8);
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }
        3 => {
            let (consumed, removed, done) = clean_total_hotkey_shares_batch::<T>(budget);
            weight = weight.saturating_add(consumed);
            log::info!(
                "Zero-alpha cleanup phase 3 (TotalHotkeyShares): removed {removed} zero entries this batch. Done: {done}"
            );
            if done {
                ZeroAlphaCleanupPhase::<T>::put(4u8);
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }
        4 => {
            let (consumed, removed, done) = clean_total_hotkey_shares_v2_batch::<T>(budget);
            weight = weight.saturating_add(consumed);
            log::info!(
                "Zero-alpha cleanup phase 4 (TotalHotkeySharesV2): removed {removed} zero entries this batch. Done: {done}"
            );
            if done {
                ZeroAlphaCleanupPhase::<T>::put(5u8);
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }
        5 => {
            let (consumed, removed, done) = clean_total_hotkey_alpha_last_epoch_batch::<T>(budget);
            weight = weight.saturating_add(consumed);
            log::info!(
                "Zero-alpha cleanup phase 5 (TotalHotkeyAlphaLastEpoch): removed {removed} zero entries this batch. Done: {done}"
            );
            if done {
                ZeroAlphaCleanupPhase::<T>::put(6u8);
                weight = weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }
        6 => {
            let (consumed, removed, done) = clean_alpha_dividends_per_subnet_batch::<T>(budget);
            weight = weight.saturating_add(consumed);
            log::info!(
                "Zero-alpha cleanup phase 6 (AlphaDividendsPerSubnet): removed {removed} zero entries this batch. Done: {done}"
            );
            if done {
                // All phases complete — mark migration as done
                HasMigrationRun::<T>::insert(MIGRATION_NAME.to_vec(), true);
                ZeroAlphaCleanupPhase::<T>::put(0u8);
                weight = weight.saturating_add(T::DbWeight::get().writes(2));
                log::info!("Zero-alpha cleanup: All phases complete. Migration marked as done.");
            }
        }
        _ => {
            // Unknown phase, reset
            ZeroAlphaCleanupPhase::<T>::put(0u8);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
        }
    }

    weight
}

/// Remove zero-valued entries from Alpha, bounded by weight budget.
/// Returns (weight_consumed, entries_removed, is_done).
fn clean_alpha_batch<T: Config>(budget: Weight) -> (Weight, u64, bool) {
    let read_cost = T::DbWeight::get().reads(1);
    let write_cost = T::DbWeight::get().writes(1);
    let per_entry_max = read_cost.saturating_add(write_cost);
    let mut weight = Weight::zero();
    let mut removed = 0u64;

    for ((hotkey, coldkey, netuid), value) in Alpha::<T>::iter() {
        if weight.saturating_add(per_entry_max).any_gt(budget) {
            return (weight, removed, false);
        }
        weight = weight.saturating_add(read_cost);
        if value == 0 {
            Alpha::<T>::remove((hotkey, coldkey, netuid));
            weight = weight.saturating_add(write_cost);
            removed = removed.saturating_add(1);
        }
    }

    (weight, removed, true)
}

/// Remove zero-valued entries from AlphaV2, bounded by weight budget.
fn clean_alpha_v2_batch<T: Config>(budget: Weight) -> (Weight, u64, bool) {
    let read_cost = T::DbWeight::get().reads(1);
    let write_cost = T::DbWeight::get().writes(1);
    let per_entry_max = read_cost.saturating_add(write_cost);
    let mut weight = Weight::zero();
    let mut removed = 0u64;

    for ((hotkey, coldkey, netuid), value) in AlphaV2::<T>::iter() {
        if weight.saturating_add(per_entry_max).any_gt(budget) {
            return (weight, removed, false);
        }
        weight = weight.saturating_add(read_cost);
        if value.is_zero() {
            AlphaV2::<T>::remove((hotkey, coldkey, netuid));
            weight = weight.saturating_add(write_cost);
            removed = removed.saturating_add(1);
        }
    }

    (weight, removed, true)
}

/// Remove zero-valued entries from TotalHotkeyShares, bounded by weight budget.
fn clean_total_hotkey_shares_batch<T: Config>(budget: Weight) -> (Weight, u64, bool) {
    let read_cost = T::DbWeight::get().reads(1);
    let write_cost = T::DbWeight::get().writes(1);
    let per_entry_max = read_cost.saturating_add(write_cost);
    let mut weight = Weight::zero();
    let mut removed = 0u64;

    for (hotkey, netuid, value) in TotalHotkeyShares::<T>::iter() {
        if weight.saturating_add(per_entry_max).any_gt(budget) {
            return (weight, removed, false);
        }
        weight = weight.saturating_add(read_cost);
        if value == 0 {
            TotalHotkeyShares::<T>::remove(hotkey, netuid);
            weight = weight.saturating_add(write_cost);
            removed = removed.saturating_add(1);
        }
    }

    (weight, removed, true)
}

/// Remove zero-valued entries from TotalHotkeySharesV2, bounded by weight budget.
fn clean_total_hotkey_shares_v2_batch<T: Config>(budget: Weight) -> (Weight, u64, bool) {
    let read_cost = T::DbWeight::get().reads(1);
    let write_cost = T::DbWeight::get().writes(1);
    let per_entry_max = read_cost.saturating_add(write_cost);
    let mut weight = Weight::zero();
    let mut removed = 0u64;

    for (hotkey, netuid, value) in TotalHotkeySharesV2::<T>::iter() {
        if weight.saturating_add(per_entry_max).any_gt(budget) {
            return (weight, removed, false);
        }
        weight = weight.saturating_add(read_cost);
        if value.is_zero() {
            TotalHotkeySharesV2::<T>::remove(hotkey, netuid);
            weight = weight.saturating_add(write_cost);
            removed = removed.saturating_add(1);
        }
    }

    (weight, removed, true)
}

/// Remove zero-valued entries from TotalHotkeyAlphaLastEpoch, bounded by weight budget.
fn clean_total_hotkey_alpha_last_epoch_batch<T: Config>(budget: Weight) -> (Weight, u64, bool) {
    let read_cost = T::DbWeight::get().reads(1);
    let write_cost = T::DbWeight::get().writes(1);
    let per_entry_max = read_cost.saturating_add(write_cost);
    let mut weight = Weight::zero();
    let mut removed = 0u64;

    for (hotkey, netuid, value) in TotalHotkeyAlphaLastEpoch::<T>::iter() {
        if weight.saturating_add(per_entry_max).any_gt(budget) {
            return (weight, removed, false);
        }
        weight = weight.saturating_add(read_cost);
        if value.is_zero() {
            TotalHotkeyAlphaLastEpoch::<T>::remove(hotkey, netuid);
            weight = weight.saturating_add(write_cost);
            removed = removed.saturating_add(1);
        }
    }

    (weight, removed, true)
}

/// Remove zero-valued entries from AlphaDividendsPerSubnet, bounded by weight budget.
fn clean_alpha_dividends_per_subnet_batch<T: Config>(budget: Weight) -> (Weight, u64, bool) {
    let read_cost = T::DbWeight::get().reads(1);
    let write_cost = T::DbWeight::get().writes(1);
    let per_entry_max = read_cost.saturating_add(write_cost);
    let mut weight = Weight::zero();
    let mut removed = 0u64;

    for (netuid, hotkey, value) in AlphaDividendsPerSubnet::<T>::iter() {
        if weight.saturating_add(per_entry_max).any_gt(budget) {
            return (weight, removed, false);
        }
        weight = weight.saturating_add(read_cost);
        if value.is_zero() {
            AlphaDividendsPerSubnet::<T>::remove(netuid, hotkey);
            weight = weight.saturating_add(write_cost);
            removed = removed.saturating_add(1);
        }
    }

    (weight, removed, true)
}
