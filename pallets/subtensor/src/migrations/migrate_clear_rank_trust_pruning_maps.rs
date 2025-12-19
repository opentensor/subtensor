use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

/// Remove all keys from Rank, Trust, and PruningScores.
pub fn migrate_clear_rank_trust_pruning_maps<T: Config>() -> Weight {
    let mig_name: Vec<u8> = b"clear_rank_trust_pruning_maps".to_vec();
    let mig_name_str = String::from_utf8_lossy(&mig_name);

    // 1 read for the HasMigrationRun flag
    let mut total_weight = T::DbWeight::get().reads(1);

    // Run-once guard
    if HasMigrationRun::<T>::get(&mig_name) {
        log::info!("Migration '{mig_name_str}' already executed - skipping");
        return total_weight;
    }

    log::info!("Running migration '{mig_name_str}'");

    let mut total_reads: u64 = 0;
    let mut total_writes: u64 = 0;
    let limit: u32 = u32::MAX;

    // ------------------------------
    // 1) Rank: clear in one go
    // ------------------------------
    let rank_res = Rank::<T>::clear(limit, None);
    let rank_reads = rank_res.loops as u64;
    let rank_writes = rank_res.backend as u64;
    total_reads = total_reads.saturating_add(rank_reads);
    total_writes = total_writes.saturating_add(rank_writes);

    log::info!(
        "Rank wipe: backend={}, loops={}, cursor_is_none={}",
        rank_res.backend,
        rank_res.loops,
        rank_res.maybe_cursor.is_none(),
    );

    // ------------------------------
    // 2) Trust: clear in one go
    // ------------------------------
    let trust_res = Trust::<T>::clear(limit, None);
    let trust_reads = trust_res.loops as u64;
    let trust_writes = trust_res.backend as u64;
    total_reads = total_reads.saturating_add(trust_reads);
    total_writes = total_writes.saturating_add(trust_writes);

    log::info!(
        "Trust wipe: backend={}, loops={}, cursor_is_none={}",
        trust_res.backend,
        trust_res.loops,
        trust_res.maybe_cursor.is_none(),
    );

    // ------------------------------
    // 3) PruningScores: clear in one go
    // ------------------------------
    let ps_res = PruningScores::<T>::clear(limit, None);
    let ps_reads = ps_res.loops as u64;
    let ps_writes = ps_res.backend as u64;
    total_reads = total_reads.saturating_add(ps_reads);
    total_writes = total_writes.saturating_add(ps_writes);

    log::info!(
        "PruningScores wipe: backend={}, loops={}, cursor_is_none={}",
        ps_res.backend,
        ps_res.loops,
        ps_res.maybe_cursor.is_none(),
    );

    // Accumulate reads/writes from Rank/Trust/PruningScores into the total weight
    total_weight =
        total_weight.saturating_add(T::DbWeight::get().reads_writes(total_reads, total_writes));

    // Mark migration as done
    HasMigrationRun::<T>::insert(&mig_name, true);
    total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!("Migration '{mig_name_str}' completed");

    total_weight
}
