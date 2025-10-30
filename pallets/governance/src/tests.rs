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
                RuntimeEvent::Governance(Event::<Test>::AllowedProposersSet {
                    incoming: vec![
                        U256::from(1),
                        U256::from(2),
                        U256::from(3),
                        U256::from(4),
                        U256::from(5)
                    ],
                    outgoing: vec![],
                    removed_proposals: vec![],
                })
            );
        });
}

#[test]
fn set_allowed_proposers_removes_proposals_of_outgoing_proposers() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash1, _proposal_index1) = create_custom_proposal(
            U256::from(1),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 1i32.to_be_bytes().to_vec())],
            },
        );
        let (proposal_hash2, _proposal_index2) = create_custom_proposal(
            U256::from(1),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 2i32.to_be_bytes().to_vec())],
            },
        );
        let (proposal_hash3, _proposal_index3) = create_custom_proposal(
            U256::from(3),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 3i32.to_be_bytes().to_vec())],
            },
        );
        assert_eq!(
            AllowedProposers::<Test>::get(),
            vec![U256::from(1), U256::from(2), U256::from(3)]
        );

        let allowed_proposers =
            BoundedVec::truncate_from(vec![U256::from(2), U256::from(3), U256::from(4)]);
        assert_ok!(Pallet::<Test>::set_allowed_proposers(
            RuntimeOrigin::root(),
            allowed_proposers.clone()
        ));

        assert_eq!(AllowedProposers::<Test>::get(), allowed_proposers);
        assert_eq!(
            Proposals::<Test>::get(),
            vec![(U256::from(3), proposal_hash3)]
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::AllowedProposersSet {
                incoming: vec![U256::from(4)],
                outgoing: vec![U256::from(1)],
                removed_proposals: vec![
                    (U256::from(1), proposal_hash1),
                    (U256::from(1), proposal_hash2)
                ],
            })
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
                RuntimeEvent::Governance(Event::<Test>::TriumvirateSet {
                    incoming: vec![U256::from(1001), U256::from(1002), U256::from(1003)],
                    outgoing: vec![],
                })
            );
        });
}

#[test]
fn set_triumvirate_removes_votes_of_outgoing_triumvirate_members() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash1, proposal_index1) = create_custom_proposal(
            U256::from(1),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 1i32.to_be_bytes().to_vec())],
            },
        );
        let (proposal_hash2, proposal_index2) = create_custom_proposal(
            U256::from(2),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 2i32.to_be_bytes().to_vec())],
            },
        );
        let (proposal_hash3, proposal_index3) = create_custom_proposal(
            U256::from(3),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 3i32.to_be_bytes().to_vec())],
            },
        );
        assert_eq!(
            Triumvirate::<Test>::get(),
            vec![U256::from(1001), U256::from(1002), U256::from(1003)]
        );

        vote_aye(U256::from(1001), proposal_hash1, proposal_index1);

        vote_nay(U256::from(1002), proposal_hash2, proposal_index2);
        vote_aye(U256::from(1003), proposal_hash2, proposal_index2);

        vote_nay(U256::from(1001), proposal_hash3, proposal_index3);
        vote_aye(U256::from(1002), proposal_hash3, proposal_index3);

        let triumvirate =
            BoundedVec::truncate_from(vec![U256::from(1001), U256::from(1003), U256::from(1004)]);
        assert_ok!(Pallet::<Test>::set_triumvirate(
            RuntimeOrigin::root(),
            triumvirate.clone()
        ));
        assert_eq!(Triumvirate::<Test>::get(), triumvirate);
        let voting1 = Voting::<Test>::get(proposal_hash1).unwrap();
        assert_eq!(voting1.ayes.to_vec(), vec![U256::from(1001)]);
        assert_eq!(voting1.nays.to_vec(), vec![]);
        let voting2 = Voting::<Test>::get(proposal_hash2).unwrap();
        assert_eq!(voting2.ayes.to_vec(), vec![U256::from(1003)]);
        assert_eq!(voting2.nays.to_vec(), vec![]);
        let voting3 = Voting::<Test>::get(proposal_hash3).unwrap();
        assert_eq!(voting3.ayes.to_vec(), vec![]);
        assert_eq!(voting3.nays.to_vec(), vec![U256::from(1001)]);
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::TriumvirateSet {
                incoming: vec![U256::from(1004)],
                outgoing: vec![U256::from(1002)],
            })
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

        let proposal_index = ProposalCount::<Test>::get();
        assert_ok!(Pallet::<Test>::propose(
            RuntimeOrigin::signed(U256::from(1)),
            proposal.clone(),
            length_bound
        ));

        let proposal_hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);
        let bounded_proposal = <Test as pallet::Config>::Preimages::bound(*proposal).unwrap();
        assert_eq!(
            Proposals::<Test>::get(),
            vec![(U256::from(1), proposal_hash)]
        );
        assert_eq!(ProposalCount::<Test>::get(), 1);
        assert_eq!(
            ProposalOf::<Test>::get(proposal_hash),
            Some(bounded_proposal)
        );
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            Voting::<Test>::get(proposal_hash),
            Some(Votes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::new(),
                end: now + MotionDuration::get(),
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Proposed {
                account: U256::from(1),
                proposal_index: 0,
                proposal_hash,
                end: now + MotionDuration::get(),
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

        let proposal_index = ProposalCount::<Test>::get();
        assert_ok!(Pallet::<Test>::propose(
            RuntimeOrigin::signed(U256::from(1)),
            proposal.clone(),
            length_bound
        ));

        let proposal_hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);
        assert_eq!(
            Proposals::<Test>::get(),
            vec![(U256::from(1), proposal_hash)]
        );
        assert_eq!(ProposalCount::<Test>::get(), 1);
        let stored_proposals = ProposalOf::<Test>::iter().collect::<Vec<_>>();
        assert_eq!(stored_proposals.len(), 1);
        let (stored_hash, bounded_proposal) = &stored_proposals[0];
        assert_eq!(stored_hash, &proposal_hash);
        assert!(<Test as pallet::Config>::Preimages::have(&bounded_proposal));
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            Voting::<Test>::get(proposal_hash),
            Some(Votes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::new(),
                end: now + MotionDuration::get(),
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Proposed {
                account: U256::from(1),
                proposal_index: 0,
                proposal_hash,
                end: now + MotionDuration::get(),
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
fn propose_with_already_scheduled_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        vote_aye(U256::from(1001), proposal_hash, proposal_index);
        vote_aye(U256::from(1002), proposal_hash, proposal_index);

        let proposal = Box::new(RuntimeCall::System(
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
            },
        ));
        let length_bound = proposal.encoded_size() as u32;
        assert_noop!(
            Pallet::<Test>::propose(
                RuntimeOrigin::signed(U256::from(1)),
                proposal.clone(),
                length_bound
            ),
            Error::<Test>::AlreadyScheduled
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

#[test]
fn vote_aye_as_first_voter_works() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        let approve = true;
        assert_ok!(Pallet::<Test>::vote(
            RuntimeOrigin::signed(U256::from(1001)),
            proposal_hash,
            proposal_index,
            approve
        ));

        let votes = Voting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![U256::from(1001)]);
        assert_eq!(votes.nays.to_vec(), vec![]);
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Voted {
                account: U256::from(1001),
                proposal_hash,
                voted: true,
                yes: 1,
                no: 0,
            })
        );
    });
}

#[test]
fn vote_nay_as_first_voter_works() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        let approve = false;
        assert_ok!(Pallet::<Test>::vote(
            RuntimeOrigin::signed(U256::from(1001)),
            proposal_hash,
            proposal_index,
            approve
        ));

        let votes = Voting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.nays.to_vec(), vec![U256::from(1001)]);
        assert_eq!(votes.ayes.to_vec(), vec![]);
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Voted {
                account: U256::from(1001),
                proposal_hash,
                voted: false,
                yes: 0,
                no: 1,
            })
        );
    });
}

#[test]
fn vote_can_be_updated() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        // Vote aye initially
        vote_aye(U256::from(1001), proposal_hash, proposal_index);
        let votes = Voting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![U256::from(1001)]);
        assert_eq!(votes.nays.to_vec(), vec![]);
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Voted {
                account: U256::from(1001),
                proposal_hash,
                voted: true,
                yes: 1,
                no: 0,
            })
        );

        // Then vote nay, replacing the aye vote
        vote_nay(U256::from(1001), proposal_hash, proposal_index);
        let votes = Voting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.nays.to_vec(), vec![U256::from(1001)]);
        assert_eq!(votes.ayes.to_vec(), vec![]);
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Voted {
                account: U256::from(1001),
                proposal_hash,
                voted: false,
                yes: 0,
                no: 1,
            })
        );

        // Then vote aye again, replacing the nay vote
        vote_aye(U256::from(1001), proposal_hash, proposal_index);
        let votes = Voting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![U256::from(1001)]);
        assert_eq!(votes.nays.to_vec(), vec![]);
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::Voted {
                account: U256::from(1001),
                proposal_hash,
                voted: true,
                yes: 1,
                no: 0,
            })
        );
    });
}

#[test]
fn two_aye_votes_schedule_proposal() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        vote_aye(U256::from(1001), proposal_hash, proposal_index);
        vote_nay(U256::from(1002), proposal_hash, proposal_index);
        vote_aye(U256::from(1003), proposal_hash, proposal_index);

        assert_eq!(Proposals::<Test>::get(), vec![]);
        assert!(!Voting::<Test>::contains_key(proposal_hash));
        assert_eq!(Scheduled::<Test>::get(), vec![proposal_hash]);
        let task_name: [u8; 32] = proposal_hash.as_ref().try_into().unwrap();
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            pallet_scheduler::Lookup::<Test>::get(task_name).unwrap().0,
            now + MotionDuration::get()
        );
        let events = last_n_events(3);
        assert_eq!(
            events[0],
            RuntimeEvent::Governance(Event::<Test>::Voted {
                account: U256::from(1003),
                proposal_hash,
                voted: true,
                yes: 2,
                no: 1,
            })
        );
        assert_eq!(
            events[2],
            RuntimeEvent::Governance(Event::<Test>::Scheduled { proposal_hash })
        );
    });
}

#[test]
fn two_nay_votes_cancel_proposal() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        vote_nay(U256::from(1001), proposal_hash, proposal_index);
        vote_aye(U256::from(1002), proposal_hash, proposal_index);
        vote_nay(U256::from(1003), proposal_hash, proposal_index);

        assert_eq!(Proposals::<Test>::get(), vec![]);
        assert!(!Voting::<Test>::contains_key(proposal_hash));
        assert_eq!(Scheduled::<Test>::get(), vec![]);
        assert_eq!(ProposalOf::<Test>::get(proposal_hash), None);
        let events = last_n_events(2);
        assert_eq!(
            events[0],
            RuntimeEvent::Governance(Event::<Test>::Voted {
                account: U256::from(1003),
                proposal_hash,
                voted: false,
                yes: 1,
                no: 2,
            })
        );
        assert_eq!(
            events[1],
            RuntimeEvent::Governance(Event::<Test>::Cancelled { proposal_hash })
        );
    });
}

#[test]
fn vote_as_bad_origin_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        assert_noop!(
            Pallet::<Test>::vote(RuntimeOrigin::root(), proposal_hash, proposal_index, true),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Pallet::<Test>::vote(RuntimeOrigin::none(), proposal_hash, proposal_index, true),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn vote_as_non_triumvirate_member_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        assert_noop!(
            Pallet::<Test>::vote(
                RuntimeOrigin::signed(U256::from(42)),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::NotTriumvirateMember
        );
    });
}

#[test]
fn vote_on_missing_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let invalid_proposal_hash =
            <Test as frame_system::Config>::Hashing::hash(b"Invalid proposal");
        assert_noop!(
            Pallet::<Test>::vote(
                RuntimeOrigin::signed(U256::from(1001)),
                invalid_proposal_hash,
                0,
                true
            ),
            Error::<Test>::ProposalMissing
        );
    });
}

#[test]
fn vote_on_scheduled_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        vote_aye(U256::from(1001), proposal_hash, proposal_index);
        vote_aye(U256::from(1002), proposal_hash, proposal_index);

        assert_eq!(Proposals::<Test>::get(), vec![]);
        assert_eq!(Scheduled::<Test>::get(), vec![proposal_hash]);

        assert_noop!(
            Pallet::<Test>::vote(
                RuntimeOrigin::signed(U256::from(1003)),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::ProposalMissing
        );
    })
}

#[test]
fn vote_on_proposal_with_wrong_index_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        assert_noop!(
            Pallet::<Test>::vote(
                RuntimeOrigin::signed(U256::from(1001)),
                proposal_hash,
                proposal_index + 1,
                true
            ),
            Error::<Test>::WrongProposalIndex
        );
    });
}

#[test]
fn duplicate_vote_on_proposal_already_voted_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal();

        let aye_voter = RuntimeOrigin::signed(U256::from(1001));
        let approve = true;
        assert_ok!(Pallet::<Test>::vote(
            aye_voter.clone(),
            proposal_hash,
            proposal_index,
            approve
        ));
        assert_noop!(
            Pallet::<Test>::vote(aye_voter, proposal_hash, proposal_index, approve),
            Error::<Test>::DuplicateVote
        );

        let nay_voter = RuntimeOrigin::signed(U256::from(1002));
        let approve = false;
        assert_ok!(Pallet::<Test>::vote(
            nay_voter.clone(),
            proposal_hash,
            proposal_index,
            approve
        ));
        assert_noop!(
            Pallet::<Test>::vote(nay_voter, proposal_hash, proposal_index, approve),
            Error::<Test>::DuplicateVote
        );
    });
}

#[test]
fn aye_vote_on_proposal_with_too_many_scheduled_fails() {
    TestState::default().build_and_execute(|| {
        // We fill the scheduled proposals up to the maximum.
        for i in 0..MaxScheduled::get() {
            let (proposal_hash, proposal_index) = create_custom_proposal(
                U256::from(1),
                frame_system::Call::<Test>::set_storage {
                    items: vec![(b"Foobar".to_vec(), i.to_be_bytes().to_vec())],
                },
            );
            vote_aye(U256::from(1001), proposal_hash, proposal_index);
            vote_aye(U256::from(1002), proposal_hash, proposal_index);
        }

        let (proposal_hash, proposal_index) = create_proposal();

        vote_aye(U256::from(1001), proposal_hash, proposal_index);
        assert_noop!(
            Pallet::<Test>::vote(
                RuntimeOrigin::signed(U256::from(1002)),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::TooManyScheduled
        );
    });
}

fn create_custom_proposal(
    proposer: U256,
    call: impl Into<LocalCallOf<Test>>,
) -> (<mock::Test as frame_system::Config>::Hash, u32) {
    let proposal = Box::new(call.into());
    let length_bound = proposal.encoded_size() as u32;
    let proposal_hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);
    let proposal_index = ProposalCount::<Test>::get();

    assert_ok!(Pallet::<Test>::propose(
        RuntimeOrigin::signed(proposer),
        proposal.clone(),
        length_bound
    ));

    (proposal_hash, proposal_index)
}

fn create_proposal() -> (<mock::Test as frame_system::Config>::Hash, u32) {
    create_custom_proposal(
        U256::from(1),
        frame_system::Call::<Test>::set_storage {
            items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
        },
    )
}

fn vote_aye(
    voter: U256,
    proposal_hash: <mock::Test as frame_system::Config>::Hash,
    proposal_index: u32,
) {
    assert_ok!(Pallet::<Test>::vote(
        RuntimeOrigin::signed(voter),
        proposal_hash,
        proposal_index,
        true
    ));
}

fn vote_nay(
    voter: U256,
    proposal_hash: <mock::Test as frame_system::Config>::Hash,
    proposal_index: u32,
) {
    assert_ok!(Pallet::<Test>::vote(
        RuntimeOrigin::signed(voter),
        proposal_hash,
        proposal_index,
        false
    ));
}
