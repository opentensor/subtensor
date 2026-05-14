use alloc::vec::Vec;

use frame_support::pallet_prelude::*;
use pallet_multi_collective::{
    Collective, CollectiveInfo, CollectiveInspect, CollectivesInfo, OnNewTerm,
};
use pallet_subtensor::root_registered::{OnRootRegistrationChange, RootRegisteredInspector};
use runtime_common::prod_or_fast;
use substrate_fixed::types::I96F32;
use subtensor_runtime_common::{TaoBalance, pad_name, time::DAYS};

use crate::{AccountId, BlockNumber, Runtime};

/// Minimum subnet age for a subnet owner to be eligible for the Building collective.
pub const MIN_SUBNET_AGE: BlockNumber = prod_or_fast!(180 * DAYS, 100);

/// Target size of the Economic ranked collective.
pub const ECONOMIC_SIZE: u32 = 16;

/// Target size of the Building ranked collective.
pub const BUILDING_SIZE: u32 = 16;

/// Cap on the EconomicEligible collective. Equal to the root subnet's
/// maximum UID count: membership mirrors the set of coldkeys with at
/// least one root-registered hotkey, so the worst case is one distinct
/// coldkey per root UID.
pub const ECONOMIC_ELIGIBLE_SIZE: u32 = 64;

/// Time before a collective rotation is triggered.
const TERM_DURATION: BlockNumber = prod_or_fast!(60 * DAYS, 100);

/// Identifier of a collective managed by `pallet-multi-collective`.
#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum CollectiveId {
    /// Accounts authorized to submit proposals on the triumvirate track.
    #[codec(index = 0)]
    Proposers,
    /// Three-member approval body for track 0.
    #[codec(index = 1)]
    Triumvirate,
    /// Top validators: one half of the collective oversight voter set.
    #[codec(index = 2)]
    Economic,
    /// Top subnet owners: one half of the collective oversight voter set.
    #[codec(index = 3)]
    Building,
    /// Staging set for the Economic collective. Membership is driven by
    /// `do_root_register` in `pallet-subtensor`; each rotation projects
    /// the top-`ECONOMIC_SIZE` from here into `Economic`.
    #[codec(index = 4)]
    EconomicEligible,
}

pub struct Collectives;
impl CollectivesInfo<BlockNumber, [u8; 32]> for Collectives {
    type Id = CollectiveId;

    fn collectives() -> impl Iterator<Item = Collective<Self::Id, BlockNumber, [u8; 32]>> {
        [
            Collective {
                id: CollectiveId::Proposers,
                info: CollectiveInfo {
                    name: pad_name(b"proposers"),
                    min_members: 1,
                    max_members: Some(20),
                    term_duration: None,
                },
            },
            Collective {
                id: CollectiveId::Triumvirate,
                info: CollectiveInfo {
                    name: pad_name(b"triumvirate"),
                    min_members: 3,
                    max_members: Some(3),
                    term_duration: None,
                },
            },
            Collective {
                id: CollectiveId::Economic,
                info: CollectiveInfo {
                    name: pad_name(b"economic"),
                    min_members: 1,
                    max_members: Some(ECONOMIC_SIZE),
                    term_duration: Some(TERM_DURATION),
                },
            },
            Collective {
                id: CollectiveId::Building,
                info: CollectiveInfo {
                    name: pad_name(b"building"),
                    min_members: 1,
                    max_members: Some(BUILDING_SIZE),
                    term_duration: Some(TERM_DURATION),
                },
            },
            Collective {
                id: CollectiveId::EconomicEligible,
                info: CollectiveInfo {
                    name: pad_name(b"economic_eligible"),
                    min_members: 0,
                    max_members: Some(ECONOMIC_ELIGIBLE_SIZE),
                    term_duration: None,
                },
            },
        ]
        .into_iter()
    }
}

/// `OnNewTerm` for `pallet-multi-collective`: dispatches by collective id
/// to a ranking pass over on-chain state.
pub struct TermManagement;
impl OnNewTerm<CollectiveId> for TermManagement {
    fn weight() -> Weight {
        // Worst-case bound used to pre-charge `force_rotate`. `on_initialize`
        // separately accumulates the actual weight returned by `on_new_term`,
        // so this bound is only consulted at extrinsic dispatch.
        //
        // TODO(weights): tighten once `StakingHotkeys` has an explicit size
        // bound or once the ranking helpers move to a bounded iterator.
        const RANKING_ITERATIONS_BOUND: u64 = 5_000;
        const READS_PER_ITERATION: u64 = 8;
        let db = <Runtime as frame_system::Config>::DbWeight::get();
        let ranking = db.reads(RANKING_ITERATIONS_BOUND.saturating_mul(READS_PER_ITERATION));
        let apply = db.reads_writes(1, 1);
        ranking.saturating_add(apply)
    }

    fn on_new_term(collective_id: CollectiveId) -> Weight {
        // The pallet is policy-agnostic; `force_rotate` will route any
        // existing id through this hook even for curated collectives
        // (Proposers / Triumvirate), so we silently no-op for those rather
        // than attempt a ranking pass against data we don't have.
        match collective_id {
            CollectiveId::Economic => Self::rotate_economic(),
            CollectiveId::Building => Self::rotate_building(),
            _ => Weight::zero(),
        }
    }
}

impl TermManagement {
    fn rotate_economic() -> Weight {
        let (members, query_weight) = Self::top_validators(ECONOMIC_SIZE);
        Self::apply_rotation(CollectiveId::Economic, members, query_weight)
    }

    fn rotate_building() -> Weight {
        let (members, query_weight) = Self::top_subnet_owners(BUILDING_SIZE, MIN_SUBNET_AGE);
        Self::apply_rotation(CollectiveId::Building, members, query_weight)
    }

    /// Rank coldkeys by total TAO stake (TAO equivalent across all subnets,
    /// including delegated stake). Iterates `pallet_subtensor::StakingHotkeys`
    /// to enumerate participating coldkeys, then `get_total_stake_for_coldkey`
    /// for each. Returns the top `n` distinct coldkeys, descending by stake.
    pub fn top_validators(n: u32) -> (Vec<AccountId>, Weight) {
        let mut weight = Weight::zero();
        let mut entries: Vec<(AccountId, TaoBalance)> = Vec::new();

        for (coldkey, _) in pallet_subtensor::StakingHotkeys::<Runtime>::iter() {
            // Conservative per-coldkey read estimate; actual cost depends on
            // hotkeys * subnets, which we can't know here without iterating again.
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
    /// subnets registered at least `min_age` blocks ago. Multiple subnets
    /// owned by the same coldkey are deduplicated to that coldkey's
    /// *highest* moving price; owning more subnets shouldn't multiply your
    /// governance weight beyond a single seat in the Building collective.
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

    /// Push a new membership list into multi-collective storage. Goes through
    /// `set_members` (rather than direct storage writes) so size validation,
    /// the `OnMembersChanged` hook, and the canonical `MembersSet` event all
    /// fire on every rotation.
    fn apply_rotation(
        collective_id: CollectiveId,
        members: Vec<AccountId>,
        query_weight: Weight,
    ) -> Weight {
        let len = members.len() as u64;
        // TODO: bypass the extrinsic and emit a rotation-failure event.
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

        query_weight.saturating_add(
            <Runtime as frame_system::Config>::DbWeight::get()
                .reads_writes(1, 1)
                .saturating_add(
                    <Runtime as frame_system::Config>::DbWeight::get().reads_writes(len, len),
                ),
        )
    }
}

/// Syncs `EconomicEligible` membership to the root-registered coldkey set.
/// Fired by `pallet-subtensor` whenever a coldkey crosses the 0↔1 boundary
/// in `RootRegisteredHotkeyCount`. `do_add_member` / `do_remove_member`
/// are idempotent and skip origin checks, so the sync is best-effort:
/// failures are logged but do not block the underlying root-registration
/// or hotkey-swap call.
pub struct EconomicEligibleSync;

impl OnRootRegistrationChange<AccountId> for EconomicEligibleSync {
    fn on_added(coldkey: &AccountId) {
        if let Err(err) = pallet_multi_collective::Pallet::<Runtime>::do_add_member(
            CollectiveId::EconomicEligible,
            coldkey.clone(),
        ) {
            log::error!(
                target: "runtime::economic-eligible-sync",
                "do_add_member failed for {:?}: {:?}",
                coldkey,
                err,
            );
        }
    }

    fn on_removed(coldkey: &AccountId) {
        if let Err(err) = pallet_multi_collective::Pallet::<Runtime>::do_remove_member(
            CollectiveId::EconomicEligible,
            coldkey.clone(),
        ) {
            log::error!(
                target: "runtime::economic-eligible-sync",
                "do_remove_member failed for {:?}: {:?}",
                coldkey,
                err,
            );
        }
    }
}

/// Read-side accessor for `pallet-subtensor`'s try_state invariant. Reads
/// the `EconomicEligible` membership directly so the runtime can assert
/// it stays in sync with `RootRegisteredHotkeyCount`.
pub struct EconomicEligibleInspector;

impl RootRegisteredInspector<AccountId> for EconomicEligibleInspector {
    fn members() -> Option<Vec<AccountId>> {
        Some(
            <pallet_multi_collective::Pallet<Runtime> as CollectiveInspect<
                AccountId,
                CollectiveId,
            >>::members_of(CollectiveId::EconomicEligible),
        )
    }
}
