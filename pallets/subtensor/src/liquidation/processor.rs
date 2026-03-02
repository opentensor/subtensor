use super::*;
use crate::liquidation::types::{ChunkResult, LiquidationState};
use crate::pallet::*;
use crate::{Config, Event, Pallet};
use frame_support::weights::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::SaturatedConversion;
use subtensor_runtime_common::NetUid;

impl<T: Config> Pallet<T> {
    /// Main state machine dispatch. Processes one or more phases within the given weight budget.
    /// Returns (weight_used, updated_state, is_liquidation_complete).
    pub fn process_liquidation_step(
        netuid: NetUid,
        mut state: LiquidationState<BlockNumberFor<T>>,
        weight_budget: Weight,
    ) -> (Weight, LiquidationState<BlockNumberFor<T>>, bool) {
        use crate::liquidation::types::LiquidationPhase;

        let mut remaining = weight_budget;
        let mut total_consumed = Weight::zero();

        loop {
            let phase_tag = state.phase.tag();
            let phase = core::mem::replace(&mut state.phase, LiquidationPhase::Freeze);

            let result = match phase {
                LiquidationPhase::Freeze => ChunkResult::Complete(Weight::zero()),
                LiquidationPhase::SnapshotStakers { cursor } => {
                    Self::snapshot_stakers_chunk(netuid, cursor, &mut state, remaining)
                }
                LiquidationPhase::ClearHyperparams => {
                    ChunkResult::Complete(Self::clear_hyperparams(netuid))
                }
                LiquidationPhase::ClearNeuronData { map_idx, cursor } => {
                    Self::clear_neuron_data_chunk(netuid, map_idx, cursor, remaining)
                }
                LiquidationPhase::ClearRootWeights { uid_cursor } => {
                    Self::clear_root_weights_chunk(netuid, uid_cursor, remaining)
                }
                LiquidationPhase::FinalizeRootDividends { .. } => {
                    ChunkResult::Complete(Self::finalize_root_dividends(netuid))
                }
                LiquidationPhase::DistributeAlpha { cursor_idx } => {
                    Self::distribute_alpha_chunk(netuid, cursor_idx, &mut state, remaining)
                }
                LiquidationPhase::DissolveUserLPs { .. } => {
                    ChunkResult::Complete(Self::dissolve_user_lps(netuid))
                }
                LiquidationPhase::ClearProtocolLPs => {
                    ChunkResult::Complete(Self::clear_protocol_lps(netuid))
                }
                LiquidationPhase::ClearMatrices {
                    mechanism_idx,
                    map_idx,
                    cursor,
                } => Self::clear_matrices_chunk(netuid, mechanism_idx, map_idx, cursor, remaining),
                LiquidationPhase::ClearTwoKeyMaps { map_idx, .. } => {
                    Self::clear_two_key_maps_chunk(netuid, map_idx, remaining)
                }
                LiquidationPhase::FinalCleanup => {
                    Self::final_cleanup(netuid);
                    ChunkResult::Complete(Weight::from_parts(FIXED_OVERHEAD, 0))
                }
            };

            let weight_used = result.weight_used();
            remaining = remaining.saturating_sub(weight_used);
            total_consumed = total_consumed.saturating_add(weight_used);

            match result {
                ChunkResult::Complete(_) => {
                    Self::deposit_event(Event::LiquidationPhaseCompleted {
                        netuid,
                        phase: phase_tag,
                    });

                    match phase_tag.next_phase() {
                        Some(next) => state.phase = next,
                        None => return (total_consumed, state, true),
                    }
                }
                ChunkResult::Incomplete { phase, .. } => {
                    state.phase = phase;
                    return (total_consumed, state, false);
                }
            }

            if remaining.ref_time() < MIN_PHASE_WEIGHT {
                return (total_consumed, state, false);
            }
        }
    }

    /// Process all liquidations in `on_idle`. Called from hooks.rs.
    pub fn process_liquidations(remaining_weight: Weight) -> Weight {
        use frame_support::weights::WeightMeter;

        let mut meter = WeightMeter::with_limit(remaining_weight);

        // Collect liquidating subnets to avoid borrow issues
        let liquidating: sp_std::vec::Vec<(NetUid, LiquidationState<BlockNumberFor<T>>)> =
            LiquidatingSubnets::<T>::iter().collect();

        for (netuid, state) in liquidating {
            if !meter.can_consume(Weight::from_parts(MIN_LIQUIDATION_WEIGHT, 0)) {
                break;
            }

            let current_block = Self::get_current_block_as_u64();
            let max_block: u64 = state.max_completion_block.saturated_into();

            // Check timeout
            if current_block > max_block {
                Self::emergency_finalize(netuid, &state);
                Self::deposit_event(Event::LiquidationTimeout {
                    netuid,
                    phase_at_timeout: state.phase.tag(),
                });
                LiquidatingSubnets::<T>::remove(netuid);
                let _ = meter.try_consume(Weight::from_parts(FIXED_OVERHEAD, 0));
                continue;
            }

            // Calculate budget with safety net
            let started_at: u64 = state.started_at.saturated_into();
            let blocks_elapsed = current_block.saturating_sub(started_at);
            let tempo = Tempo::<T>::get(netuid) as u64;
            let adjusted_budget = if blocks_elapsed
                > tempo
                    .saturating_mul(BUDGET_DOUBLE_THRESHOLD_NUMER)
                    .saturating_div(BUDGET_DOUBLE_THRESHOLD_DENOM)
            {
                state.weight_per_block.saturating_mul(2)
            } else {
                state.weight_per_block
            };

            let budget = meter.remaining().min(adjusted_budget);

            let (weight_used, final_state, is_complete) =
                Self::process_liquidation_step(netuid, state, budget);

            let _ = meter.try_consume(weight_used);

            if !is_complete {
                LiquidatingSubnets::<T>::insert(netuid, final_state);
            } else {
                LiquidatingSubnets::<T>::remove(netuid);
                Self::clear_staker_snapshot(netuid, final_state.snapshot_count);

                let actual_blocks_elapsed =
                    Self::get_current_block_as_u64().saturating_sub(started_at);

                Self::deposit_event(Event::LiquidationCompleted {
                    netuid,
                    total_blocks: actual_blocks_elapsed,
                    tao_distributed: final_state.tao_distributed,
                    stakers_paid: final_state.snapshot_count,
                });
                Self::deposit_event(Event::NetworkRemoved(netuid));

                Self::finalize_liquidation_slot(netuid);
            }
        }

        meter.consumed()
    }
}
