use super::*;
use crate::HasMigrationRun;
use frame_support::{traits::Get, weights::Weight};
use scale_info::prelude::string::String;
use sp_io::{storage::clear_prefix, KillStorageResult};

pub fn migrate_commit_reveal_2<T: Config>() -> Weight {
    let migration_name = b"migrate_commit_reveal_2".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            migration_name
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    // ------------------------------
    // Step 1: Remove WeightCommitRevealInterval entries
    // ------------------------------

    const WEIGHT_COMMIT_REVEAL_INTERVAL_PREFIX: &[u8] =
        b"pallet_subtensor::WeightCommitRevealInterval";
    let removal_results = clear_prefix(WEIGHT_COMMIT_REVEAL_INTERVAL_PREFIX, Some(u32::MAX));

    let removed_entries_count = match removal_results {
        KillStorageResult::AllRemoved(removed) => removed as u64,
        KillStorageResult::SomeRemaining(removed) => {
            log::info!("Failed To Remove Some Items During migrate_commit_reveal_v2",);
            removed as u64
        }
    };

    weight = weight.saturating_add(T::DbWeight::get().writes(removed_entries_count));

    log::info!(
        "Removed {:?} entries from WeightCommitRevealInterval.",
        removed_entries_count
    );

    // ------------------------------
    // Step 2: Remove WeightCommits entries
    // ------------------------------

    const WEIGHT_COMMITS_PREFIX: &[u8] = b"pallet_subtensor::WeightCommits";
    let removal_results_commits = clear_prefix(WEIGHT_COMMITS_PREFIX, Some(u32::MAX));

    let removed_commits_entries = match removal_results_commits {
        KillStorageResult::AllRemoved(removed) => removed as u64,
        KillStorageResult::SomeRemaining(removed) => {
            log::info!("Failed To Remove Some Items During migrate_commit_reveal_v2",);
            removed as u64
        }
    };

    weight = weight.saturating_add(T::DbWeight::get().writes(removed_commits_entries));

    log::info!(
        "Removed {} entries from WeightCommits.",
        removed_commits_entries
    );

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
