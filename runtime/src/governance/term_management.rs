use alloc::vec::Vec;

use frame_support::pallet_prelude::*;
use pallet_multi_collective::{
    CollectiveInspect, OnNewTerm, weights::WeightInfo as MultiCollectiveWeightInfo,
};
use substrate_fixed::types::{I96F32, U64F64};

use crate::{AccountId, BlockNumber, Runtime};

use super::collectives::{
    BUILDING_SIZE, CollectiveId, ECONOMIC_ELIGIBLE_SIZE, ECONOMIC_SIZE, MIN_SUBNET_AGE,
};

/// `OnNewTerm` for `pallet-multi-collective`: dispatches by collective id
/// to a ranking pass over on-chain state.
pub struct TermManagement;

impl OnNewTerm<CollectiveId> for TermManagement {
    fn weight() -> Weight {
        // Worst-case bound used to pre-charge `force_rotate`. `on_initialize`
        // separately accumulates the actual weight returned by `on_new_term`,
        // so this bound is only consulted at extrinsic dispatch. Picks the
        // larger of the two rotation paths (Economic / Building).
        //
        // Economic ranking: one read for the EconomicEligible roster plus
        // one EMA lookup per member, bounded by ECONOMIC_ELIGIBLE_SIZE.
        // Building ranking: three reads per subnet, bounded by SUBNET_BOUND
        // (chosen above the `SubnetLimit` default with headroom).
        //
        // TODO(weights): both ranking bounds are hand-rolled from storage
        // caps. Replace with a runtime-level benchmark of
        // `rotate_economic` / `rotate_building` once the runtime crate
        // grows a benchmark harness.
        const SUBNET_BOUND: u64 = 256;
        let db = <Runtime as frame_system::Config>::DbWeight::get();
        let economic = db.reads(u64::from(ECONOMIC_ELIGIBLE_SIZE).saturating_add(1));
        let building = db.reads(SUBNET_BOUND.saturating_mul(3));
        let ranking = if economic.ref_time() >= building.ref_time() {
            economic
        } else {
            building
        };
        let apply = <Runtime as pallet_multi_collective::Config>::WeightInfo::set_members();
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
        let (members, query_weight) = Self::top_economic_eligible(ECONOMIC_SIZE);
        Self::apply_rotation(CollectiveId::Economic, members, query_weight)
    }

    fn rotate_building() -> Weight {
        let (members, query_weight) = Self::top_subnet_owners(BUILDING_SIZE, MIN_SUBNET_AGE);
        Self::apply_rotation(CollectiveId::Building, members, query_weight)
    }

    /// Project the top `n` coldkeys from `EconomicEligible` by their
    /// root-registered stake EMA. The EMA is maintained by the subtensor
    /// pallet's round-robin sampler ([`crate::governance::stake_ema`]),
    /// so the ranking is intentionally smoothed: a coldkey can't leapfrog
    /// established members by stacking stake right before a rotation.
    pub fn top_economic_eligible(n: u32) -> (Vec<AccountId>, Weight) {
        let db = <Runtime as frame_system::Config>::DbWeight::get();
        let eligible = <pallet_multi_collective::Pallet<Runtime> as CollectiveInspect<
            AccountId,
            CollectiveId,
        >>::members_of(CollectiveId::EconomicEligible);
        let mut weight = db.reads(1);

        let entries: Vec<(AccountId, U64F64)> = eligible
            .into_iter()
            .map(|coldkey| {
                let state = pallet_subtensor::RootRegisteredEma::<Runtime>::get(&coldkey);
                (coldkey, state.ema)
            })
            .collect();
        weight = weight.saturating_add(db.reads(entries.len() as u64));

        (rank_top_n(entries, n), weight)
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
            merge_owner_by_highest_price(&mut entries, owner, price);
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

        query_weight
            .saturating_add(<Runtime as pallet_multi_collective::Config>::WeightInfo::set_members())
    }
}

/// Sort `entries` by descending score and return the first `n` keys.
/// `sort_by` is stable, so ties preserve the input order (mostly relevant
/// when `EconomicEligible` rows share identical EMA values during warmup).
fn rank_top_n<K>(mut entries: Vec<(K, U64F64)>, n: u32) -> Vec<K> {
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(n as usize);
    entries.into_iter().map(|(k, _)| k).collect()
}

/// Insert `(owner, price)` into `entries`, keeping only the owner's
/// highest price across multiple subnets. Mutates in place; doesn't
/// allocate when the owner already has an entry.
fn merge_owner_by_highest_price<A: PartialEq>(
    entries: &mut Vec<(A, I96F32)>,
    owner: A,
    price: I96F32,
) {
    if let Some(existing) = entries.iter_mut().find(|(o, _)| *o == owner) {
        if price > existing.1 {
            existing.1 = price;
        }
    } else {
        entries.push((owner, price));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rank_entry(key: u32, score: u64) -> (u32, U64F64) {
        (key, U64F64::saturating_from_num(score))
    }

    fn price(value: i64) -> I96F32 {
        I96F32::saturating_from_num(value)
    }

    #[test]
    fn rank_top_n_truncates_to_n() {
        let result = rank_top_n(
            vec![
                rank_entry(1, 10),
                rank_entry(2, 30),
                rank_entry(3, 20),
                rank_entry(4, 40),
            ],
            2,
        );
        assert_eq!(result, vec![4, 2]);
    }

    #[test]
    fn rank_top_n_zero_returns_empty() {
        let result = rank_top_n(vec![rank_entry(1, 10), rank_entry(2, 30)], 0);
        assert!(result.is_empty());
    }

    #[test]
    fn rank_top_n_larger_than_input_returns_all_sorted() {
        let result = rank_top_n(vec![rank_entry(1, 10), rank_entry(2, 30)], 100);
        assert_eq!(result, vec![2, 1]);
    }

    #[test]
    fn rank_top_n_empty_input_returns_empty() {
        let result = rank_top_n::<u32>(vec![], 5);
        assert!(result.is_empty());
    }

    #[test]
    fn rank_top_n_ties_preserve_insertion_order() {
        let result = rank_top_n(
            vec![rank_entry(1, 10), rank_entry(2, 10), rank_entry(3, 10)],
            2,
        );
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn merge_inserts_first_observation() {
        let mut entries: Vec<(u32, I96F32)> = Vec::new();
        merge_owner_by_highest_price(&mut entries, 7, price(100));
        assert_eq!(entries, vec![(7, price(100))]);
    }

    #[test]
    fn merge_upgrades_to_higher_price_for_same_owner() {
        let mut entries = vec![(7, price(100))];
        merge_owner_by_highest_price(&mut entries, 7, price(250));
        assert_eq!(entries, vec![(7, price(250))]);
    }

    #[test]
    fn merge_keeps_existing_when_new_price_lower() {
        let mut entries = vec![(7, price(250))];
        merge_owner_by_highest_price(&mut entries, 7, price(100));
        assert_eq!(entries, vec![(7, price(250))]);
    }

    #[test]
    fn merge_dedups_owner_across_multiple_subnets() {
        // Owner 7 holds two subnets, owner 8 holds one. After merging the
        // three observations, owner 7 has a single entry at its highest
        // price (300), not two — exactly the property that prevents
        // multi-subnet ownership from inflating a coldkey's governance
        // weight.
        let mut entries: Vec<(u32, I96F32)> = Vec::new();
        merge_owner_by_highest_price(&mut entries, 7, price(100));
        merge_owner_by_highest_price(&mut entries, 8, price(200));
        merge_owner_by_highest_price(&mut entries, 7, price(300));
        assert_eq!(entries, vec![(7, price(300)), (8, price(200))]);
    }
}
