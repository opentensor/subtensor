//! Benchmarks for Governance Pallet
#![cfg(feature = "runtime-benchmarks")]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]
use crate::pallet::*;
use crate::{ProposalIndex, TriumvirateVotes};
use codec::Encode;
use frame_benchmarking::{account, v2::*};
use frame_support::{
    assert_ok,
    traits::{QueryPreimage, StorePreimage},
};
use frame_system::RawOrigin;
use sp_runtime::{
    BoundedVec, Vec,
    traits::{Get, Hash},
};
use sp_std::vec;

extern crate alloc;

const SEED: u32 = 0;

use alloc::boxed::Box;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_allowed_proposers(p: Linear<1, { T::MaxProposals::get() }>) {
        let max_proposers = T::MaxAllowedProposers::get();

        for i in 0..max_proposers {
            allowed_proposer::<T>(i);
        }

        for i in 0..p {
            let proposer = AllowedProposers::<T>::get()[(i % max_proposers) as usize].clone();
            create_dummy_proposal::<T>(proposer, Some(i), vec![], vec![]);
        }

        // Generate some allowed proposers all different from the old ones to force worst case clean up.
        let mut new_allowed_proposers = (0..max_proposers)
            .map(|i| account("allowed_proposer", 1000 + i, SEED))
            .collect::<Vec<_>>();

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            BoundedVec::truncate_from(new_allowed_proposers.clone()),
        );

        new_allowed_proposers.sort();
        assert_eq!(AllowedProposers::<T>::get().to_vec(), new_allowed_proposers);
        assert_eq!(Proposals::<T>::get().len(), 0);
        assert_eq!(ProposalOf::<T>::iter().count(), 0);
        assert_eq!(TriumvirateVoting::<T>::iter().count(), 0);
    }

    #[benchmark]
    fn set_triumvirate(p: Linear<1, { T::MaxProposals::get() }>) {
        let proposer = allowed_proposer::<T>(0);
        let triumvirate = triumvirate::<T>();

        // Set up some proposals with triumvirate votes
        let proposals = (0..p)
            .map(|i| {
                let ayes = vec![triumvirate[0].clone()];
                let nays = vec![triumvirate[2].clone()];
                create_dummy_proposal::<T>(proposer.clone(), Some(i), ayes, nays)
            })
            .collect::<Vec<_>>();

        // Setup some triumvirate totally different from the old one to force worst case clean up.
        let mut new_triumvirate = vec![
            account("triumvirate", 1000, SEED),
            account("triumvirate", 1001, SEED),
            account("triumvirate", 1002, SEED),
        ];

        #[extrinsic_call]
        _(
            RawOrigin::Root,
            BoundedVec::truncate_from(new_triumvirate.clone()),
        );

        new_triumvirate.sort();
        assert_eq!(Triumvirate::<T>::get().to_vec(), new_triumvirate);
        for (hash, _) in proposals {
            let voting = TriumvirateVoting::<T>::get(hash).unwrap();
            assert!(voting.ayes.to_vec().is_empty());
            assert!(voting.nays.to_vec().is_empty());
        }
    }

    #[benchmark]
    fn propose() {
        let proposer = allowed_proposer::<T>(0);

        // Create a large enough proposal to avoid inlining
        let key_value = (b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec());
        let proposal: Box<<T as Config>::RuntimeCall> = Box::new(
            frame_system::Call::<T>::set_storage {
                items: sp_std::iter::repeat_n(key_value, 50).collect::<Vec<_>>(),
            }
            .into(),
        );
        let proposal_hash = T::Hashing::hash_of(&proposal);
        let length_bound = proposal.encoded_size() as u32;

        #[extrinsic_call]
        _(
            RawOrigin::Signed(proposer.clone()),
            proposal.clone(),
            length_bound,
        );

        assert_eq!(
            Proposals::<T>::get().to_vec(),
            vec![(proposer.clone(), proposal_hash)]
        );
        assert!(ProposalOf::<T>::contains_key(proposal_hash));
        let stored_proposals = ProposalOf::<T>::iter().collect::<Vec<_>>();
        assert_eq!(stored_proposals.len(), 1);
        let (_stored_hash, bounded_proposal) = &stored_proposals[0];
        assert!(<T as Config>::Preimages::have(bounded_proposal));
    }

    #[benchmark]
    fn vote_on_proposed() {
        let proposer = allowed_proposer::<T>(0);
        let triumvirate = triumvirate::<T>();

        // Set up some proposal with two votes, fast tracking is the worst case.
        let ayes = vec![triumvirate[0].clone()];
        let nays = vec![triumvirate[1].clone()];
        let (hash, index) = create_dummy_proposal::<T>(proposer, Some(0), ayes, nays);

        #[extrinsic_call]
        _(RawOrigin::Signed(triumvirate[2].clone()), hash, index, true);

        assert!(Proposals::<T>::get().is_empty());
        assert_eq!(ProposalOf::<T>::iter().count(), 0);
        assert_eq!(TriumvirateVoting::<T>::iter().count(), 0);
        assert_eq!(Scheduled::<T>::get().to_vec(), vec![hash]);
    }

    #[benchmark]
    fn vote_on_scheduled() {
        let proposer = allowed_proposer::<T>(0);
        let triumvirate = triumvirate::<T>();

        let member: T::AccountId = account("collective_member", 4242, SEED);
        EconomicCollective::<T>::try_append(member.clone()).unwrap();

        // Set up some scheduled proposal
        let ayes = vec![triumvirate[0].clone()];
        let nays = vec![triumvirate[1].clone()];
        let (hash, index) = create_dummy_proposal::<T>(proposer, Some(0), ayes, nays);
        assert_ok!(Pallet::<T>::vote_on_proposed(
            RawOrigin::Signed(triumvirate[2].clone()).into(),
            hash,
            index,
            true,
        ));
        let delay = CollectiveVoting::<T>::get(hash).unwrap().delay;

        #[extrinsic_call]
        _(RawOrigin::Signed(member.clone()), hash, index, false);

        assert_eq!(CollectiveVoting::<T>::iter().count(), 1);
        let voting = CollectiveVoting::<T>::get(hash).unwrap();
        assert!(voting.ayes.to_vec().is_empty());
        assert_eq!(voting.nays.to_vec(), vec![member]);
        assert!(voting.delay > delay);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

fn allowed_proposer<T: Config>(index: u32) -> T::AccountId {
    let proposer: T::AccountId = account("allowed_proposer", index, SEED);
    AllowedProposers::<T>::try_append(proposer.clone()).unwrap();
    proposer
}

fn triumvirate<T: Config>() -> Vec<T::AccountId> {
    let triumvirate = vec![
        account("triumvirate", 0, SEED),
        account("triumvirate", 1, SEED),
        account("triumvirate", 2, SEED),
    ];
    Triumvirate::<T>::put(BoundedVec::truncate_from(triumvirate.clone()));
    triumvirate
}

fn dummy_proposal<T: Config>(n: u32) -> Box<<T as Config>::RuntimeCall> {
    Box::new(
        frame_system::Call::<T>::set_storage {
            items: vec![(b"Foobar".to_vec(), n.to_be_bytes().to_vec())],
        }
        .into(),
    )
}

fn create_dummy_proposal<T: Config>(
    proposer: T::AccountId,
    index: Option<ProposalIndex>,
    ayes: Vec<T::AccountId>,
    nays: Vec<T::AccountId>,
) -> (T::Hash, ProposalIndex) {
    let proposal_index = index.unwrap_or(0);
    let proposal = dummy_proposal::<T>(proposal_index);
    let proposal_hash = T::Hashing::hash_of(&proposal);
    let bounded_proposal = T::Preimages::bound(*proposal).unwrap();

    Proposals::<T>::try_append((proposer.clone(), proposal_hash)).unwrap();
    ProposalOf::<T>::insert(proposal_hash, bounded_proposal);
    TriumvirateVoting::<T>::insert(
        proposal_hash,
        TriumvirateVotes {
            index: proposal_index,
            ayes: BoundedVec::truncate_from(ayes),
            nays: BoundedVec::truncate_from(nays),
            end: frame_system::Pallet::<T>::block_number() + T::MotionDuration::get(),
        },
    );

    (proposal_hash, proposal_index)
}
