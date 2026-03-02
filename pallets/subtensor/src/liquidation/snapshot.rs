use super::*;
use crate::liquidation::types::{ChunkResult, CursorBytes, LiquidationPhase, LiquidationState};
use crate::pallet::{Alpha, LiquidationStakerSnapshot};
use crate::{Config, Pallet};
use frame_support::weights::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use subtensor_runtime_common::NetUid;

impl<T: Config> Pallet<T> {
    /// Build the staker snapshot incrementally over multiple blocks.
    /// Uses `Alpha::<T>::iter_keys_from(cursor)` with netuid filter since
    /// Alpha is keyed (hot, cold, netuid) and we cannot iter_prefix by netuid.
    pub fn snapshot_stakers_chunk(
        netuid: NetUid,
        cursor: Option<CursorBytes>,
        state: &mut LiquidationState<BlockNumberFor<T>>,
        weight_budget: Weight,
    ) -> ChunkResult {
        let items_per_budget = weight_budget
            .ref_time()
            .checked_div(WEIGHT_PER_SNAPSHOT)
            .unwrap_or(0);
        if items_per_budget == 0 {
            return ChunkResult::Incomplete {
                weight_used: Weight::zero(),
                phase: LiquidationPhase::SnapshotStakers { cursor },
            };
        }

        // Start or resume iteration
        let iter: sp_std::boxed::Box<dyn Iterator<Item = (T::AccountId, T::AccountId, NetUid)>> =
            match &cursor {
                Some(raw_key) => {
                    sp_std::boxed::Box::new(Alpha::<T>::iter_keys_from(raw_key.to_vec()))
                }
                None => sp_std::boxed::Box::new(Alpha::<T>::iter_keys()),
            };

        let mut count = 0u64;
        let mut last_raw_key: Option<sp_std::vec::Vec<u8>>;

        for (hot, cold, this_netuid) in iter {
            // Update cursor for every entry we visit
            last_raw_key = Some(Alpha::<T>::hashed_key_for((&hot, &cold, this_netuid)));

            // Filter: only process entries for our target netuid
            if this_netuid != netuid {
                count = count.saturating_add(1);
                if count >= items_per_budget {
                    return Self::save_snapshot_cursor(last_raw_key, count);
                }
                continue;
            }

            // Convert shares to actual alpha value
            let pool = Self::get_alpha_share_pool(hot.clone(), netuid);
            let actual_val = pool.try_get_value(&cold).unwrap_or(0);
            let share_u64f64 = Alpha::<T>::get((&hot, &cold, netuid));
            let alpha_value: u64 = if actual_val == 0 {
                share_u64f64.saturating_to_num::<u64>()
            } else {
                actual_val
            };

            // Skip dust positions
            if alpha_value < MIN_SNAPSHOT_ALPHA {
                count = count.saturating_add(1);
                if count >= items_per_budget {
                    return Self::save_snapshot_cursor(last_raw_key, count);
                }
                continue;
            }

            let alpha_u128 = u128::from(alpha_value);

            // Write to indexed StorageDoubleMap
            let idx = state.snapshot_count;
            LiquidationStakerSnapshot::<T>::insert(netuid, idx, (hot, cold, alpha_u128));
            state.snapshot_count = state.snapshot_count.saturating_add(1);

            // Accumulate total_alpha_value
            state.total_alpha_value = state.total_alpha_value.saturating_add(alpha_u128);

            count = count.saturating_add(1);
            if count >= items_per_budget {
                return Self::save_snapshot_cursor(last_raw_key, count);
            }
        }

        // Iterator exhausted — snapshot complete
        ChunkResult::Complete(Weight::from_parts(
            count.saturating_mul(WEIGHT_PER_SNAPSHOT),
            0,
        ))
    }

    /// Build an incomplete `ChunkResult` from a raw cursor key.
    fn save_snapshot_cursor(last_raw_key: Option<sp_std::vec::Vec<u8>>, count: u64) -> ChunkResult {
        let cursor = last_raw_key.and_then(|k| {
            k.try_into()
                .map_err(|_| {
                    log::warn!("Snapshot cursor exceeds BoundedVec capacity, restarting iteration");
                })
                .ok()
        });
        ChunkResult::Incomplete {
            weight_used: Weight::from_parts(count.saturating_mul(WEIGHT_PER_SNAPSHOT), 0),
            phase: LiquidationPhase::SnapshotStakers { cursor },
        }
    }
}
