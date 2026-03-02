use super::*;
use crate::liquidation::types::{LiquidationPhase, LiquidationState};
use crate::pallet::{
    LiquidatingSubnets, MechanismCountCurrent, NetuidCooldown, NetworksAdded, SubnetTAO,
    SubnetworkN, Tempo, TotalStake,
};
use crate::{Config, Error, Event, Pallet};
use frame_support::pallet_prelude::*;
use frame_support::weights::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Saturating;
use subtensor_runtime_common::{Currency, NetUid};

impl<T: Config> Pallet<T> {
    /// Start the liquidation process for a subnet.
    /// Validates state, records counts, snapshots TAO pot, calculates weight budget.
    pub fn start_liquidation(netuid: NetUid) -> DispatchResult {
        // 1. Ensure subnet exists and is not already liquidating
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        ensure!(
            !LiquidatingSubnets::<T>::contains_key(netuid),
            Error::<T>::SubnetAlreadyLiquidating
        );
        ensure!(netuid != NetUid::ROOT, Error::<T>::CannotLiquidateRoot);

        // Only one liquidation at a time
        ensure!(
            LiquidatingSubnets::<T>::iter().next().is_none(),
            Error::<T>::LiquidationInProgress
        );

        // Ensure netuid is not in cooldown
        ensure!(
            !NetuidCooldown::<T>::contains_key(netuid),
            Error::<T>::NetuidInCooldown
        );

        // 2. Record counts for work estimation
        let neuron_count = SubnetworkN::<T>::get(netuid);
        let mechanism_count: u8 = MechanismCountCurrent::<T>::get(netuid).into();

        let staker_count = (neuron_count as u32)
            .saturating_mul(STAKERS_PER_NEURON_ESTIMATE)
            .max(100);

        // 3. Snapshot TAO pot at freeze time and decrement TotalStake.
        // The TAO pot represents staked TAO that will be distributed as free balance
        // (via Currency::deposit) during the DistributeAlpha phase, so it must be
        // removed from TotalStake now. This mirrors the atomic path in
        // destroy_alpha_in_out_stakes (remove_stake.rs).
        let pot_tao = SubnetTAO::<T>::get(netuid);
        let tao_pot: u64 = pot_tao.into();
        if tao_pot > 0 {
            TotalStake::<T>::mutate(|total| *total = total.saturating_sub(pot_tao));
        }

        // 4. Calculate weight budget
        let total_weight =
            Self::estimate_liquidation_weight(staker_count, neuron_count, mechanism_count);
        let tempo = Tempo::<T>::get(netuid) as u64;
        let target_blocks = tempo.max(MIN_LIQUIDATION_BLOCKS);
        let weight_per_block = Weight::from_parts(
            total_weight.ref_time().checked_div(target_blocks).unwrap_or(0),
            total_weight.proof_size().checked_div(target_blocks).unwrap_or(0),
        );

        // 5-6. Create liquidation state
        let started_at = <frame_system::Pallet<T>>::block_number();
        let max_liquidation_blocks_u32 = u32::try_from(MAX_LIQUIDATION_BLOCKS).unwrap_or(u32::MAX);
        let max_completion =
            started_at.saturating_add(BlockNumberFor::<T>::from(max_liquidation_blocks_u32));

        let state = LiquidationState {
            started_at,
            max_completion_block: max_completion,
            phase: LiquidationPhase::Freeze,
            weight_per_block,
            total_stakers: staker_count,
            total_neurons: neuron_count,
            mechanism_count,
            tao_pot,
            total_alpha_value: 0,
            snapshot_count: 0,
            tao_distributed: 0,
        };

        LiquidatingSubnets::<T>::insert(netuid, state);

        // 7. Emit event
        Self::deposit_event(Event::LiquidationStarted {
            netuid,
            started_at,
            estimated_blocks: target_blocks,
            staker_count,
        });

        Ok(())
    }

    /// Check if a subnet is currently in liquidation.
    pub fn is_subnet_liquidating(netuid: NetUid) -> bool {
        LiquidatingSubnets::<T>::contains_key(netuid)
    }

    /// Ensure a subnet is not in liquidation, returning an error if it is.
    pub fn ensure_not_liquidating(netuid: NetUid) -> DispatchResult {
        ensure!(
            !Self::is_subnet_liquidating(netuid),
            Error::<T>::SubnetLiquidating
        );
        Ok(())
    }

    /// Estimate total weight for the entire liquidation.
    ///
    /// Arithmetic is done in `u128` to avoid silent saturation, then clamped to
    /// `u64::MAX`. Each term is named so the dominant cost is easy to spot.
    pub fn estimate_liquidation_weight(
        staker_count: u32,
        neuron_count: u16,
        mechanism_count: u8,
    ) -> Weight {
        let s = u128::from(staker_count);
        let n = u128::from(neuron_count);
        let m = u128::from(mechanism_count);

        // SnapshotStakers iterates ALL Alpha entries across all subnets
        let total_subnets = NetworksAdded::<T>::iter()
            .filter(|(_, added)| *added)
            .count() as u128;
        let alpha_entries = s.saturating_mul(total_subnets.max(1));

        let snapshot_weight = alpha_entries.saturating_mul(u128::from(WEIGHT_PER_SNAPSHOT));
        let distribution_weight = s.saturating_mul(u128::from(WEIGHT_PER_DISTRIBUTION));
        let matrix_weight = n
            .saturating_mul(n)
            .saturating_mul(m)
            .saturating_mul(u128::from(WEIGHT_PER_MATRIX_ENTRY));
        let neuron_clear_weight = n
            .saturating_mul(u128::from(TWO_KEY_MAP_COUNT))
            .saturating_mul(u128::from(WEIGHT_PER_NEURON_CLEAR));
        let hyperparam_weight =
            u128::from(HYPERPARAM_COUNT).saturating_mul(u128::from(WEIGHT_PER_HYPERPARAM));

        let total = u128::from(FIXED_OVERHEAD)
            .saturating_add(snapshot_weight)
            .saturating_add(distribution_weight)
            .saturating_add(matrix_weight)
            .saturating_add(neuron_clear_weight)
            .saturating_add(hyperparam_weight);

        let ref_time = u64::try_from(total).unwrap_or(u64::MAX);
        Weight::from_parts(ref_time, 0)
    }
}
