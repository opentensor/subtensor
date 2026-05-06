use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;

/// One-shot migration for the dynamic-tempo / owner-triggered-epochs feature.
///
/// 1. Back-fills `LastEpochBlock[netuid]` for every existing subnet so the first
///    post-upgrade epoch lands on the same block as the legacy modulo formula
///    `(block + netuid + 1) % (tempo + 1) == 0`. The new scheduler period is
///    `tempo + 1` (next firing at `LastEpochBlock + tempo + 1`).
/// 2. Defensively clamps `Tempo` values in `(0, MIN_TEMPO) ∪ (MAX_TEMPO, u16::MAX]`
///    into `[MIN_TEMPO, MAX_TEMPO]`. Subnets with `Tempo == 0` are left as-is — the
///    legacy short-circuit keeps them dormant and matches their pre-upgrade behaviour.
/// 3. Converts each subnet's existing `ActivityCutoff[netuid]` (absolute block count)
///    into `ActivityCutoffFactorMilli[netuid]` (per-mille of `tempo`) so that
///    `factor * tempo / 1000 ≈ old_cutoff` post-upgrade. Production defaults
///    (`tempo=360`, `cutoff=5000`) round-trip to 4999 blocks (1-block delta from
///    integer division, ≈0.02%). Out-of-range factors are clamped to
///    `[MIN_ACTIVITY_CUTOFF_FACTOR_MILLI, MAX_ACTIVITY_CUTOFF_FACTOR_MILLI]` —
///    extreme historical cutoffs may shift to the nearest representable factor.
pub fn migrate_dynamic_tempo<T: Config>() -> Weight {
    let mig_name: Vec<u8> = b"dynamic_tempo_v1".to_vec();
    let mig_name_str = String::from_utf8_lossy(&mig_name);

    let mut total_weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&mig_name) {
        log::info!("Migration '{mig_name_str}' already executed - skipping");
        return total_weight;
    }

    log::info!("Running migration '{mig_name_str}'");

    let current_block = Pallet::<T>::get_current_block_as_u64();
    let mut visited: u64 = 0;
    let mut tempo_clamped: u64 = 0;
    let mut last_epoch_seeded: u64 = 0;
    let mut activity_factor_seeded: u64 = 0;
    let mut activity_factor_clamped: u64 = 0;
    let mut reads: u64 = 0;
    let mut writes: u64 = 0;

    let netuids: Vec<NetUid> = Tempo::<T>::iter_keys().collect();
    reads = reads.saturating_add(netuids.len() as u64);

    for netuid in netuids.into_iter() {
        visited = visited.saturating_add(1);
        let mut tempo = Tempo::<T>::get(netuid);
        reads = reads.saturating_add(1);

        if tempo == 0 {
            // Legacy `tempo == 0` short-circuit preserved; do not seed `LastEpochBlock`.
            continue;
        }

        // Defensive bounds clamp.
        let clamped = tempo.clamp(MIN_TEMPO, MAX_TEMPO);
        if clamped != tempo {
            tempo = clamped;
            Tempo::<T>::insert(netuid, tempo);
            tempo_clamped = tempo_clamped.saturating_add(1);
            writes = writes.saturating_add(1);
        }

        // Compute next-epoch block under the *legacy* modulo formula and back-fill
        // `LastEpochBlock` so the *new* formula yields the same next-epoch block.
        // Legacy `blocks_until_next_epoch`:
        //   adjusted = current_block + netuid + 1
        //   remainder = adjusted % (tempo + 1)
        //   blocks_until_next = tempo - remainder
        // New formula: next firing at `LastEpochBlock + tempo + 1`. Solve for `LastEpochBlock`:
        //   LastEpochBlock = current_block + blocks_until_next - tempo - 1
        //                  = current_block - (tempo + 1 - blocks_until_next)
        let netuid_plus_one = (u16::from(netuid) as u64).saturating_add(1);
        let tempo_plus_one = (tempo as u64).saturating_add(1);
        let adjusted = current_block.wrapping_add(netuid_plus_one);
        let remainder = adjusted.checked_rem(tempo_plus_one).unwrap_or(0);
        let blocks_until_next = (tempo as u64).saturating_sub(remainder);
        let offset = tempo_plus_one.saturating_sub(blocks_until_next);
        let last_epoch = current_block.saturating_sub(offset);

        LastEpochBlock::<T>::insert(netuid, last_epoch);
        last_epoch_seeded = last_epoch_seeded.saturating_add(1);
        writes = writes.saturating_add(1);

        // Convert legacy absolute `ActivityCutoff` into per-mille `ActivityCutoffFactorMilli`
        let old_cutoff = ActivityCutoff::<T>::get(netuid) as u64;
        reads = reads.saturating_add(1);
        let tempo_u64 = tempo as u64;
        let raw_factor = old_cutoff
            .saturating_mul(1_000)
            .saturating_add(tempo_u64.saturating_sub(1))
            .checked_div(tempo_u64)
            .unwrap_or(INITIAL_ACTIVITY_CUTOFF_FACTOR_MILLI as u64);
        let clamped = raw_factor
            .max(MIN_ACTIVITY_CUTOFF_FACTOR_MILLI as u64)
            .min(MAX_ACTIVITY_CUTOFF_FACTOR_MILLI as u64) as u32;
        if clamped as u64 != raw_factor {
            activity_factor_clamped = activity_factor_clamped.saturating_add(1);
        }
        ActivityCutoffFactorMilli::<T>::insert(netuid, clamped);
        activity_factor_seeded = activity_factor_seeded.saturating_add(1);
        writes = writes.saturating_add(1);
    }

    total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(reads, writes));

    log::info!(
        "Dynamic tempo migration: visited={visited}, tempo_clamped={tempo_clamped}, last_epoch_seeded={last_epoch_seeded}, activity_factor_seeded={activity_factor_seeded}, activity_factor_clamped={activity_factor_clamped}"
    );

    HasMigrationRun::<T>::insert(&mig_name, true);
    total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!("Migration '{mig_name_str}' completed");

    total_weight
}
