#![cfg(test)]
use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use std::iter::repeat;

#[test]
fn environment_works() {
    TestState::default().build_and_execute(|| {
        assert_eq!(
            AllowedProposers::<Test>::get(),
            vec![U256::from(1), U256::from(2), U256::from(3)]
        );
        assert_eq!(
            Triumvirate::<Test>::get(),
            vec![U256::from(1001), U256::from(1002), U256::from(1003)]
        );
    });
}

#[test]
fn environment_members_are_sorted() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(2), U256::from(3), U256::from(1)])
        .with_triumvirate(vec![U256::from(1002), U256::from(1001), U256::from(1003)])
        .build_and_execute(|| {
            assert_eq!(
                AllowedProposers::<Test>::get(),
                vec![U256::from(1), U256::from(2), U256::from(3)]
            );
            assert_eq!(
                Triumvirate::<Test>::get(),
                vec![U256::from(1001), U256::from(1002), U256::from(1003)]
            );
        });
}

#[test]
#[should_panic(expected = "Allowed proposers cannot contain duplicate accounts.")]
fn environment_with_duplicate_allowed_proposers_panics() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(2)])
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Allowed proposers length cannot exceed MaxAllowedProposers.")]
fn environment_with_too_many_allowed_proposers_panics() {
    let max_allowed_proposers = <Test as pallet::Config>::MaxAllowedProposers::get() as usize;
    let allowed_proposers = (0..=max_allowed_proposers).map(|i| U256::from(i)).collect();
    TestState::default()
        .with_allowed_proposers(allowed_proposers)
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Triumvirate cannot contain duplicate accounts.")]
fn environment_with_duplicate_triumvirate_panics() {
    TestState::default()
        .with_triumvirate(vec![U256::from(1001), U256::from(1002), U256::from(1002)])
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Triumvirate length cannot exceed 3.")]
fn environment_with_too_many_triumvirate_panics() {
    let triumvirate = (1..=4).map(|i| U256::from(i)).collect();
    TestState::default()
        .with_triumvirate(triumvirate)
        .build_and_execute(|| {});
}

#[test]
#[should_panic(expected = "Allowed proposers and triumvirate must be disjoint.")]
fn environment_with_overlapping_allowed_proposers_and_triumvirate_panics() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(3)])
        .with_triumvirate(vec![U256::from(1001), U256::from(1002), U256::from(1)])
        .build_and_execute(|| {});
}

#[test]
fn set_allowed_proposers_works() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .build_and_execute(|| {
            let allowed_proposers = BoundedVec::truncate_from(vec![
                U256::from(5),
                U256::from(1),
                U256::from(4),
                U256::from(3),
                U256::from(2),
            ]);
            assert_eq!(AllowedProposers::<Test>::get(), vec![]);

            assert_ok!(Pallet::<Test>::set_allowed_proposers(
                // SetAllowedProposersOrigin is EnsureRoot<Self::AccountId>
                RuntimeOrigin::root(),
                allowed_proposers.clone()
            ));

            assert_eq!(
                AllowedProposers::<Test>::get().to_vec(),
                // Sorted allowed proposers
                vec![
                    U256::from(1),
                    U256::from(2),
                    U256::from(3),
                    U256::from(4),
                    U256::from(5)
                ]
            );
            assert_eq!(
                last_event(),
                RuntimeEvent::Governance(Event::<Test>::AllowedProposersSet)
            );
        });
}

#[test]
fn set_allowed_proposers_with_bad_origin_fails() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .build_and_execute(|| {
            let allowed_proposers =
                BoundedVec::truncate_from((1..=5).map(|i| U256::from(i)).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(
                    RuntimeOrigin::signed(U256::from(42)),
                    allowed_proposers.clone()
                ),
                DispatchError::BadOrigin
            );

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(RuntimeOrigin::none(), allowed_proposers),
                DispatchError::BadOrigin
            );
        });
}

#[test]
fn set_allowed_proposers_with_duplicate_accounts_fails() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .build_and_execute(|| {
            let allowed_proposers =
                BoundedVec::truncate_from(repeat(U256::from(1)).take(2).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(RuntimeOrigin::root(), allowed_proposers),
                Error::<Test>::DuplicateAccounts
            );
        });
}

#[test]
fn set_allowed_proposers_with_triumvirate_intersection_fails() {
    TestState::default()
        .with_allowed_proposers(vec![])
        .with_triumvirate(vec![U256::from(1), U256::from(2), U256::from(3)])
        .build_and_execute(|| {
            let allowed_proposers =
                BoundedVec::truncate_from((3..=8).map(|i| U256::from(i)).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_allowed_proposers(RuntimeOrigin::root(), allowed_proposers),
                Error::<Test>::AllowedProposersAndTriumvirateMustBeDisjoint
            );
        });
}

#[test]
fn set_triumvirate_works() {
    TestState::default()
        .with_triumvirate(vec![])
        .build_and_execute(|| {
            let triumvirate = BoundedVec::truncate_from(vec![
                U256::from(1003),
                U256::from(1001),
                U256::from(1002),
            ]);
            assert_eq!(Triumvirate::<Test>::get(), vec![]);

            assert_ok!(Pallet::<Test>::set_triumvirate(
                // SetTriumvirateOrigin is EnsureRoot<Self::AccountId>
                RuntimeOrigin::root(),
                triumvirate.clone()
            ));

            assert_eq!(
                Triumvirate::<Test>::get(),
                // Sorted triumvirate
                vec![U256::from(1001), U256::from(1002), U256::from(1003)]
            );
            assert_eq!(
                last_event(),
                RuntimeEvent::Governance(Event::<Test>::TriumvirateSet)
            );
        });
}

#[test]
fn set_triumvirate_with_bad_origin_fails() {
    TestState::default()
        .with_triumvirate(vec![])
        .build_and_execute(|| {
            let triumvirate = BoundedVec::truncate_from(
                (1..=3).map(|i| U256::from(1000 + i)).collect::<Vec<_>>(),
            );

            assert_noop!(
                Pallet::<Test>::set_triumvirate(
                    RuntimeOrigin::signed(U256::from(42)),
                    triumvirate.clone()
                ),
                DispatchError::BadOrigin
            );

            assert_noop!(
                Pallet::<Test>::set_triumvirate(RuntimeOrigin::none(), triumvirate),
                DispatchError::BadOrigin
            );
        });
}

#[test]
fn set_triumvirate_with_duplicate_accounts_fails() {
    TestState::default()
        .with_triumvirate(vec![])
        .build_and_execute(|| {
            let triumvirate =
                BoundedVec::truncate_from(repeat(U256::from(1001)).take(2).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_triumvirate(RuntimeOrigin::root(), triumvirate),
                Error::<Test>::DuplicateAccounts
            );
        });
}

#[test]
fn set_triumvirate_with_allowed_proposers_intersection_fails() {
    TestState::default()
        .with_allowed_proposers(vec![U256::from(1), U256::from(2), U256::from(3)])
        .build_and_execute(|| {
            let triumvirate =
                BoundedVec::truncate_from((3..=8).map(|i| U256::from(i)).collect::<Vec<_>>());

            assert_noop!(
                Pallet::<Test>::set_triumvirate(RuntimeOrigin::root(), triumvirate),
                Error::<Test>::AllowedProposersAndTriumvirateMustBeDisjoint
            );
        });
}

#[test]
fn propose_works_with_inline_preimage() {
    TestState::default().build_and_execute(|| {
        let key_value = (b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec());
        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                items: vec![key_value],
            },
        ));
        let length_bound = proposal.encoded_size() as u32;

        assert_ok!(Pallet::<Test>::propose(
            RuntimeOrigin::signed(U256::from(1)),
            proposal.clone(),
            length_bound
        ));

        let proposal_hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);
        let bounded_proposal = <Test as pallet::Config>::Preimages::bound(*proposal).unwrap();
        assert_eq!(Proposals::<Test>::get(), vec![proposal_hash]);
        assert_eq!(ProposalCount::<Test>::get(), 1);
        assert_eq!(
            ProposalOf::<Test>::get(proposal_hash),
            Some(bounded_proposal)
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Proposed {
                account: U256::from(1),
                proposal_index: 0,
                proposal_hash,
            })
        );
    });
}

#[test]
fn propose_works_with_lookup_preimage() {
    TestState::default().build_and_execute(|| {
        let key_value = (b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec());
        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                // We deliberately create a large proposal to avoid inlining.
                items: repeat(key_value).take(50).collect::<Vec<_>>(),
            },
        ));
        let length_bound = proposal.encoded_size() as u32;
        println!("length_bound: {}", length_bound);

        assert_ok!(Pallet::<Test>::propose(
            RuntimeOrigin::signed(U256::from(1)),
            proposal.clone(),
            length_bound
        ));

        let proposal_hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);
        assert_eq!(Proposals::<Test>::get(), vec![proposal_hash]);
        assert_eq!(ProposalCount::<Test>::get(), 1);
        let stored_proposals = ProposalOf::<Test>::iter().collect::<Vec<_>>();
        assert_eq!(stored_proposals.len(), 1);
        let (stored_hash, bounded_proposal) = &stored_proposals[0];
        assert_eq!(stored_hash, &proposal_hash);
        assert!(<Test as pallet::Config>::Preimages::have(&bounded_proposal));
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Proposed {
                account: U256::from(1),
                proposal_index: 0,
                proposal_hash,
            })
        );
    });
}

#[test]
fn propose_with_bad_origin_fails() {
    TestState::default().build_and_execute(|| {
        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
            },
        ));
        let length_bound = proposal.encoded_size() as u32;

        assert_noop!(
            Pallet::<Test>::propose(RuntimeOrigin::root(), proposal.clone(), length_bound),
            DispatchError::BadOrigin
        );

        assert_noop!(
            Pallet::<Test>::propose(RuntimeOrigin::none(), proposal.clone(), length_bound),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn propose_with_non_allowed_proposer_fails() {
    TestState::default().build_and_execute(|| {
        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
            },
        ));
        let length_bound = proposal.encoded_size() as u32;

        assert_noop!(
            Pallet::<Test>::propose(
                RuntimeOrigin::signed(U256::from(42)),
                proposal.clone(),
                length_bound
            ),
            Error::<Test>::NotAllowedProposer
        );
    });
}

#[test]
fn propose_with_incorrect_length_bound_fails() {
    TestState::default().build_and_execute(|| {
        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
            },
        ));
        // We deliberately set the length bound to be one less than the proposal length.
        let length_bound = proposal.encoded_size() as u32 - 1;

        assert_noop!(
            Pallet::<Test>::propose(
                RuntimeOrigin::signed(U256::from(1)),
                proposal.clone(),
                length_bound
            ),
            Error::<Test>::WrongProposalLength
        );
    });
}

#[test]
fn propose_with_incorrect_weight_bound_fails() {
    TestState::default().build_and_execute(|| {
        let proposal = Box::new(RuntimeCall::TestPallet(
            pallet_test::Call::<Test>::expensive_call {},
        ));
        let length_bound = proposal.encoded_size() as u32;

        assert_noop!(
            Pallet::<Test>::propose(
                RuntimeOrigin::signed(U256::from(1)),
                proposal.clone(),
                length_bound
            ),
            Error::<Test>::WrongProposalWeight
        );
    });
}

#[test]
fn propose_with_duplicate_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
            },
        ));
        let length_bound = proposal.encoded_size() as u32;

        assert_ok!(Pallet::<Test>::propose(
            RuntimeOrigin::signed(U256::from(1)),
            proposal.clone(),
            length_bound
        ));

        assert_noop!(
            Pallet::<Test>::propose(
                RuntimeOrigin::signed(U256::from(1)),
                proposal.clone(),
                length_bound
            ),
            Error::<Test>::DuplicateProposal
        );
    });
}

#[test]
fn propose_with_too_many_proposals_fails() {
    TestState::default().build_and_execute(|| {
        // Create the maximum number of proposals.
        let proposals = (1..=MaxProposals::get())
            .map(|i| {
                let proposal = Box::new(RuntimeCall::System(
                    frame_system::Call::<Test>::set_storage {
                        items: vec![(
                            format!("Foobar{}", i).as_bytes().to_vec(),
                            42u32.to_be_bytes().to_vec(),
                        )],
                    },
                ));
                let length_bound = proposal.encoded_size() as u32;
                (proposal, length_bound)
            })
            .collect::<Vec<_>>();

        for (proposal, length_bound) in proposals {
            assert_ok!(Pallet::<Test>::propose(
                RuntimeOrigin::signed(U256::from(1)),
                proposal,
                length_bound
            ));
        }

        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
            },
        ));
        let length_bound = proposal.encoded_size() as u32;
        assert_noop!(
            Pallet::<Test>::propose(RuntimeOrigin::signed(U256::from(1)), proposal, length_bound),
            Error::<Test>::TooManyProposals
        );
    });
}
