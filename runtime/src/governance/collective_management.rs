//! Concrete `OnNewTerm` implementation that backs the Economic /
//! Building collectives by ranking on-chain `pallet-subtensor` data.
//!
//! Lives in the runtime (rather than `pallet-governance-policy`) so the
//! collective-population logic can read `pallet-subtensor` storage
//! directly without making the policy pallet runtime-specific. The
//! trigger is generic in `pallet-multi-collective` (its `on_initialize`
//! modulo check + `force_rotate` extrinsic both fire `OnNewTerm`); the
//! *meaning* of "a new term started for collective X" is what this
//! module supplies.

use alloc::vec::Vec;

use frame_support::pallet_prelude::*;
use pallet_multi_collective::CanRotate;
use substrate_fixed::types::I96F32;
use subtensor_runtime_common::TaoBalance;

use crate::{
    AccountId, BlockNumber, GovernanceCollectiveId, GovernanceMinSubnetAge,
    GovernanceRankedCollectiveSize, Runtime,
};

/// Concrete `OnNewTerm` impl wired into `pallet-multi-collective`.
/// Dispatches by collective id to a ranking pass over on-chain state.
pub struct CollectiveManagement;

impl pallet_multi_collective::OnNewTerm<GovernanceCollectiveId> for CollectiveManagement {
    fn weight() -> Weight {
        // Worst-case bound used to pre-charge `force_rotate`.
        // `on_initialize` separately accumulates the *actual* weight
        // returned by `on_new_term`, so this bound is only consulted
        // at extrinsic dispatch.
        //
        // The dominant cost is the ranking pass (`top_validators` or
        // `top_subnet_owners`) which iterates an unbounded storage map
        // and, today, charges 8 reads per staking hotkey or 3 per
        // subnet. We size the bound generously: 5_000 iterations × 8
        // reads, plus the `apply_rotation` storage cost (1 read + 1
        // write for the membership update, plus per-outgoing-member
        // cleanup work counted separately by `OnMembersChanged::weight`).
        //
        // TODO(weights): tighten once `StakingHotkeys` has an explicit
        // size bound or once the ranking helpers move to a bounded
        // iterator.
        const RANKING_ITERATIONS_BOUND: u64 = 5_000;
        const READS_PER_ITERATION: u64 = 8;
        let db = <Runtime as frame_system::Config>::DbWeight::get();
        let ranking = db.reads(RANKING_ITERATIONS_BOUND.saturating_mul(READS_PER_ITERATION));
        let apply = db.reads_writes(1, 1);
        ranking.saturating_add(apply)
    }

    fn on_new_term(collective_id: GovernanceCollectiveId) -> Weight {
        // Gate via the inherent `GovernanceCollectiveId::can_rotate()`.
        // The pallet is policy-agnostic — `force_rotate` will route any
        // existing id through this hook, so we silently no-op here for
        // curated collectives (Proposers / Triumvirate) rather than
        // attempt a ranking pass against data we don't have.
        if !collective_id.can_rotate() {
            log::debug!(
                target: "runtime::collective-management",
                "on_new_term({:?}) — non-rotating collective; no-op.",
                collective_id,
            );
            return Weight::zero();
        }

        match collective_id {
            GovernanceCollectiveId::Economic => Self::rotate_economic(),
            GovernanceCollectiveId::Building => Self::rotate_building(),
            // Unreachable: `can_rotate()` returns false for these.
            GovernanceCollectiveId::Proposers | GovernanceCollectiveId::Triumvirate => {
                Weight::zero()
            }
        }
    }
}

impl CollectiveManagement {
    fn rotate_economic() -> Weight {
        let (members, query_weight) = Self::top_validators(GovernanceRankedCollectiveSize::get());
        Self::apply_rotation(GovernanceCollectiveId::Economic, members, query_weight)
    }

    fn rotate_building() -> Weight {
        let (members, query_weight) = Self::top_subnet_owners(
            GovernanceRankedCollectiveSize::get(),
            GovernanceMinSubnetAge::get(),
        );
        Self::apply_rotation(GovernanceCollectiveId::Building, members, query_weight)
    }

    /// Rank coldkeys by total TAO stake (TAO equivalent across all
    /// subnets, including delegated stake). Iterates
    /// `pallet_subtensor::StakingHotkeys` to enumerate participating
    /// coldkeys, then `get_total_stake_for_coldkey` for each. Returns
    /// the top `n` distinct coldkeys, descending by stake.
    pub fn top_validators(n: u32) -> (Vec<AccountId>, Weight) {
        let mut weight = Weight::zero();
        let mut entries: Vec<(AccountId, TaoBalance)> = Vec::new();

        for (coldkey, _) in pallet_subtensor::StakingHotkeys::<Runtime>::iter() {
            // Conservative per-coldkey read estimate — actual cost
            // depends on hotkeys × subnets, which we can't know here
            // without iterating again.
            weight =
                weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().reads(8));
            let stake = pallet_subtensor::Pallet::<Runtime>::get_total_stake_for_coldkey(&coldkey);
            entries.push((coldkey, stake));
        }

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n as usize);
        let members = entries.into_iter().map(|(c, _)| c).collect::<Vec<_>>();
        (members, weight)
    }

    /// Rank subnet-owner coldkeys by `SubnetMovingPrice`, restricted to
    /// subnets registered at least `min_age` blocks ago.
    ///
    /// Multiple subnets owned by the same coldkey are deduplicated to
    /// that coldkey's *highest* moving price — owning more subnets
    /// shouldn't multiply your governance weight beyond a single seat
    /// in the Building collective.
    pub fn top_subnet_owners(n: u32, min_age: BlockNumber) -> (Vec<AccountId>, Weight) {
        let mut weight = Weight::zero();
        let now: u64 = <frame_system::Pallet<Runtime>>::block_number().into();
        let min_age_u64: u64 = min_age.into();

        let mut entries: Vec<(AccountId, I96F32)> = Vec::new();
        for netuid in pallet_subtensor::Pallet::<Runtime>::get_all_subnet_netuids() {
            // 3 reads: NetworkRegisteredAt + SubnetMovingPrice + SubnetOwner.
            weight =
                weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().reads(3));
            let registered_at: u64 = pallet_subtensor::NetworkRegisteredAt::<Runtime>::get(netuid);
            if now.saturating_sub(registered_at) < min_age_u64 {
                continue;
            }
            let price = pallet_subtensor::SubnetMovingPrice::<Runtime>::get(netuid);
            let owner = pallet_subtensor::SubnetOwner::<Runtime>::get(netuid);

            // Dedupe: keep the highest-priced subnet per owner.
            if let Some(existing) = entries.iter_mut().find(|(o, _)| *o == owner) {
                if price > existing.1 {
                    existing.1 = price;
                }
            } else {
                entries.push((owner, price));
            }
        }

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n as usize);
        let members = entries.into_iter().map(|(c, _)| c).collect::<Vec<_>>();
        (members, weight)
    }

    /// Push a new membership list into multi-collective storage.
    /// Goes through `set_members` (rather than direct storage writes)
    /// so size validation, the `OnMembersChanged` hook, and the canonical
    /// `MembersSet` event all fire on every rotation.
    fn apply_rotation(
        collective_id: GovernanceCollectiveId,
        members: Vec<AccountId>,
        query_weight: Weight,
    ) -> Weight {
        let len = members.len() as u64;
        let result = pallet_multi_collective::Pallet::<Runtime>::set_members(
            frame_system::RawOrigin::Root.into(),
            collective_id,
            members,
        );

        if let Err(err) = result {
            log::error!(
                target: "runtime::collective-management",
                "set_members failed for {:?}: {:?}",
                collective_id,
                err,
            );
        }

        // 1 read for old members + 1 write for new + O(len) cleanup work
        // in `OnMembersChanged`. Conservative — the actual cost of
        // signed-voting cleanup is per-active-poll.
        query_weight.saturating_add(
            <Runtime as frame_system::Config>::DbWeight::get()
                .reads_writes(1, 1)
                .saturating_add(
                    <Runtime as frame_system::Config>::DbWeight::get().reads_writes(len, len),
                ),
        )
    }
}
