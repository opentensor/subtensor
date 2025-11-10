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

    // ------------------------------
    // 1) Rank: collect keys, then remove
    // ------------------------------
    let rank_keys: Vec<NetUid> = Rank::<T>::iter_keys().collect();
    let rank_removed = rank_keys.len() as u64;
    total_reads = total_reads.saturating_add(rank_removed);
    for k in rank_keys {
        Rank::<T>::remove(k);
        total_writes = total_writes.saturating_add(1);
    }

    // ------------------------------
    // 2) Trust: collect keys, then remove
    // ------------------------------
    let trust_keys: Vec<NetUid> = Trust::<T>::iter_keys().collect();
    let trust_removed = trust_keys.len() as u64;
    total_reads = total_reads.saturating_add(trust_removed);
    for k in trust_keys {
        Trust::<T>::remove(k);
        total_writes = total_writes.saturating_add(1);
    }

    // ------------------------------
    // 3) PruningScores: collect keys, then remove
    // ------------------------------
    let ps_keys: Vec<NetUid> = PruningScores::<T>::iter_keys().collect();
    let ps_removed = ps_keys.len() as u64;
    total_reads = total_reads.saturating_add(ps_removed);
    for k in ps_keys {
        PruningScores::<T>::remove(k);
        total_writes = total_writes.saturating_add(1);
    }

    // Accumulate reads/writes into the total weight
    total_weight =
        total_weight.saturating_add(T::DbWeight::get().reads_writes(total_reads, total_writes));

    log::info!("Rank wipe: removed={rank_removed}");
    log::info!("Trust wipe: removed={trust_removed}");
    log::info!("PruningScores wipe: removed={ps_removed}");

    // Mark migration as done
    HasMigrationRun::<T>::insert(&mig_name, true);
    total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!("Migration '{mig_name_str}' completed");

    total_weight
}
