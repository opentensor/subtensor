//! Benchmarks for Pallet-Multi-Collective.
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use super::*;
#[allow(unused)]
use crate::Pallet as MultiCollective;
use frame_benchmarking::{account, v2::*};
use frame_support::traits::EnsureOriginWithArg;
use sp_std::vec::Vec;

const SEED: u32 = 0;

/// Pick the first collective id declared by the runtime's `CollectivesInfo`.
fn first_collective_id<T: Config>() -> T::CollectiveId {
    T::Collectives::collective_ids()
        .next()
        .expect("runtime must declare at least one collective for benchmarks")
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn add_member() {
        let collective_id = first_collective_id::<T>();
        let who: T::AccountId = account("new_member", 0, SEED);
        let origin = T::AddOrigin::try_successful_origin(&collective_id)
            .expect("AddOrigin must supply a successful origin for benchmarks");

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, collective_id, who.clone());
    }

    #[benchmark]
    fn remove_member() {
        let collective_id = first_collective_id::<T>();
        let who: T::AccountId = account("member_to_remove", 0, SEED);

        // Seed: make `who` a member first via AddOrigin.
        let add_origin = T::AddOrigin::try_successful_origin(&collective_id).unwrap();
        Pallet::<T>::add_member(add_origin, collective_id, who.clone()).unwrap();

        let remove_origin = T::RemoveOrigin::try_successful_origin(&collective_id).unwrap();

        #[extrinsic_call]
        _(remove_origin as T::RuntimeOrigin, collective_id, who.clone());
    }

    #[benchmark]
    fn swap_member() {
        let collective_id = first_collective_id::<T>();
        let old: T::AccountId = account("old_member", 0, SEED);
        let new: T::AccountId = account("new_member", 1, SEED);

        let add_origin = T::AddOrigin::try_successful_origin(&collective_id).unwrap();
        Pallet::<T>::add_member(add_origin, collective_id, old.clone()).unwrap();

        let swap_origin = T::SwapOrigin::try_successful_origin(&collective_id).unwrap();

        #[extrinsic_call]
        _(swap_origin as T::RuntimeOrigin, collective_id, old.clone(), new.clone());
    }

    #[benchmark]
    fn reset_members(m: Linear<1, { T::MaxMembers::get() }>) {
        let collective_id = first_collective_id::<T>();
        let new_members: Vec<T::AccountId> = (0..m)
            .map(|i| account("reset_member", i, SEED))
            .collect();

        let origin = T::ResetOrigin::try_successful_origin(&collective_id).unwrap();

        #[extrinsic_call]
        _(origin as T::RuntimeOrigin, collective_id, new_members.clone());
    }
}
