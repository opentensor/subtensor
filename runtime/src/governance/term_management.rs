use alloc::vec::Vec;

use frame_support::pallet_prelude::*;
use pallet_multi_collective::{
    CollectiveInspect, OnNewTerm, Pallet as MultiCollective,
    weights::WeightInfo as MultiCollectiveWeightInfo,
};
use pallet_subtensor::{Pallet as Subtensor, *};
use substrate_fixed::types::{I96F32, U64F64};

use crate::{AccountId, BlockNumber, Runtime};

use super::collectives::{BUILDING_SIZE, CollectiveId, ECONOMIC_SIZE, MIN_SUBNET_AGE};
use super::weights::{SubstrateWeight as GovernanceWeight, WeightInfo as GovernanceWeightInfo};

/// Minimum root-registered EMA samples before Economic eligibility.
/// With the current sampler cadence, 210 is roughly 30 days.
pub const ECONOMIC_ELIGIBILITY_THRESHOLD: u32 = 210;

/// Runtime rotation policy for rotating collectives.
pub struct TermManagement;

impl OnNewTerm<CollectiveId> for TermManagement {
    fn weight() -> Weight {
        [
            GovernanceWeight::<Runtime>::rotate_economic(),
            GovernanceWeight::<Runtime>::rotate_building(),
        ]
        .into_iter()
        .max_by_key(Weight::ref_time)
        .unwrap_or_default()
    }

    fn on_new_term(collective_id: CollectiveId) -> Weight {
        // Curated collectives are managed outside this rotation policy.
        match collective_id {
            CollectiveId::Economic => Self::rotate_economic(),
            CollectiveId::Building => Self::rotate_building(),
            _ => Weight::zero(),
        }
    }
}

impl TermManagement {
    pub(crate) fn rotate_economic() -> Weight {
        let (members, query_weight) = Self::top_validators(ECONOMIC_SIZE);
        Self::apply_rotation(CollectiveId::Economic, members, query_weight)
    }

    pub(crate) fn rotate_building() -> Weight {
        let (members, query_weight) = Self::top_subnet_owners(BUILDING_SIZE, MIN_SUBNET_AGE);
        Self::apply_rotation(CollectiveId::Building, members, query_weight)
    }

    /// Top validator coldkeys by smoothed root-registered value.
    pub fn top_validators(n: u32) -> (Vec<AccountId>, Weight) {
        let db = <Runtime as frame_system::Config>::DbWeight::get();
        let eligible =
            <MultiCollective<Runtime> as CollectiveInspect<AccountId, CollectiveId>>::members_of(
                CollectiveId::EconomicEligible,
            );
        let mut weight = db.reads(1);

        let entries: Vec<(AccountId, U64F64)> = eligible
            .into_iter()
            .filter_map(|coldkey| {
                weight.saturating_accrue(db.reads(1));
                let state = RootRegisteredEma::<Runtime>::get(&coldkey);
                (state.samples >= ECONOMIC_ELIGIBILITY_THRESHOLD).then_some((coldkey, state.ema))
            })
            .collect();

        (rank_top_n(entries, n), weight)
    }

    /// Top subnet-owner coldkeys by their best mature subnet price.
    pub fn top_subnet_owners(n: u32, min_age: BlockNumber) -> (Vec<AccountId>, Weight) {
        let mut weight = Weight::zero();
        let now: u64 = <frame_system::Pallet<Runtime>>::block_number().into();
        let min_age_u64: u64 = min_age.into();

        let mut entries: Vec<(AccountId, I96F32)> = Vec::new();
        for netuid in Subtensor::<Runtime>::get_all_subnet_netuids() {
            weight.saturating_accrue(<Runtime as frame_system::Config>::DbWeight::get().reads(3));
            let registered_at: u64 = NetworkRegisteredAt::<Runtime>::get(netuid);
            if now.saturating_sub(registered_at) < min_age_u64 {
                continue;
            }
            let price = SubnetMovingPrice::<Runtime>::get(netuid);
            let owner = SubnetOwner::<Runtime>::get(netuid);
            merge_owner_by_highest_price(&mut entries, owner, price);
        }

        (rank_top_n(entries, n), weight)
    }

    /// Apply a rotated membership through the collective pallet.
    fn apply_rotation(
        collective_id: CollectiveId,
        members: Vec<AccountId>,
        query_weight: Weight,
    ) -> Weight {
        // TODO: bypass the extrinsic and emit a rotation-failure event.
        let result = MultiCollective::<Runtime>::set_members(
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

/// Sort by descending score and return the first `n` keys.
fn rank_top_n<K, S: Ord>(mut entries: Vec<(K, S)>, n: u32) -> Vec<K> {
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(n as usize);
    entries.into_iter().map(|(k, _)| k).collect()
}

/// Keep only an owner's highest observed subnet price.
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
