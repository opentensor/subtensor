//! Benchmarks for Pallet-Signed-Voting.
//!
//! Each benchmark seeds a fake `ReferendumStatus::Ongoing` entry directly into
//! `pallet-referenda` storage so that `T::Polls::is_ongoing` returns true and the
//! full extrinsic path (signature check, scheme check, membership check, tally
//! mutation, `on_tally_updated` callback) is timed end-to-end.
//!
//! The fake referendum uses a **two-voter snapshot** so a single `aye` lands at 50%
//! approval — below the typical 2/3 PassOrFail threshold — so the post-vote
//! `on_tally_updated` doesn't auto-finalize and tear down our storage mid-benchmark.
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use super::*;
#[allow(unused)]
use crate::Pallet as SignedVoting;
use frame_benchmarking::{account, v2::*};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use pallet_referenda::{
    Proposal, ReferendumInfo, ReferendumStatus, ReferendumStatusFor, TracksInfo as _,
};

const SEED: u32 = 0;

#[benchmarks(where T: pallet_referenda::Config)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn vote() {
        let who: T::AccountId = account("voter", 0, SEED);
        let other: T::AccountId = account("voter", 1, SEED);
        let poll_index: PollIndexOf<T> = Default::default();
        let ref_index: pallet_referenda::ReferendumIndex = 0;

        let track_id = <T as pallet_referenda::Config>::Tracks::track_ids()
            .next()
            .expect("runtime must declare at least one track");
        let ref_info = ReferendumInfo {
            track: track_id,
            proposal: Proposal::Review,
            submitted: frame_system::Pallet::<T>::block_number(),
            tally: subtensor_runtime_common::VoteTally::new(),
            alarm: None,
            initial_dispatch_time: None,
        };
        ReferendumStatusFor::<T>::insert(ref_index, ReferendumStatus::Ongoing(ref_info));

        let snapshot: BoundedVec<T::AccountId, T::MaxSnapshotMembers> =
            BoundedVec::try_from(alloc::vec![who.clone(), other]).unwrap();
        VoterSnapshot::<T>::insert(poll_index, snapshot);
        TallyOf::<T>::insert(
            poll_index,
            SignedVoteTally {
                ayes: 0,
                nays: 0,
                total: 2,
            },
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(who.clone()), poll_index, true);
    }

    #[benchmark]
    fn remove_vote() {
        let who: T::AccountId = account("voter", 0, SEED);
        let other: T::AccountId = account("voter", 1, SEED);
        let poll_index: PollIndexOf<T> = Default::default();
        let ref_index: pallet_referenda::ReferendumIndex = 0;

        let track_id = <T as pallet_referenda::Config>::Tracks::track_ids()
            .next()
            .expect("runtime must declare at least one track");
        let ref_info = ReferendumInfo {
            track: track_id,
            proposal: Proposal::Review,
            submitted: frame_system::Pallet::<T>::block_number(),
            tally: subtensor_runtime_common::VoteTally::new(),
            alarm: None,
            initial_dispatch_time: None,
        };
        ReferendumStatusFor::<T>::insert(ref_index, ReferendumStatus::Ongoing(ref_info));

        let snapshot: BoundedVec<T::AccountId, T::MaxSnapshotMembers> =
            BoundedVec::try_from(alloc::vec![who.clone(), other]).unwrap();
        VoterSnapshot::<T>::insert(poll_index, snapshot);
        TallyOf::<T>::insert(
            poll_index,
            SignedVoteTally {
                ayes: 0,
                nays: 0,
                total: 2,
            },
        );
        // Place an existing aye so `remove_vote` has something to revert.
        Pallet::<T>::try_vote(poll_index, &who, true).unwrap();

        #[extrinsic_call]
        _(RawOrigin::Signed(who.clone()), poll_index);
    }
}
