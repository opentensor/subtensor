#![cfg(test)]
#![allow(clippy::iter_skip_next, clippy::unwrap_used, clippy::indexing_slicing)]
use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;

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
    let allowed_proposers = (0..=max_allowed_proposers).map(U256::from).collect();
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
    let triumvirate = (1..=4).map(U256::from).collect();
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
            assert!(AllowedProposers::<Test>::get().is_empty());

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
        let (proposal_hash1, _proposal_index1) = create_custom_proposal!(
            U256::from(1),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 1i32.to_be_bytes().to_vec())],
            }
        );
        let (proposal_hash2, _proposal_index2) = create_custom_proposal!(
            U256::from(1),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 2i32.to_be_bytes().to_vec())],
            }
        );
        let (proposal_hash3, _proposal_index3) = create_custom_proposal!(
            U256::from(3),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 3i32.to_be_bytes().to_vec())],
            }
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
                BoundedVec::truncate_from((1..=5).map(U256::from).collect::<Vec<_>>());

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
            let allowed_proposers = BoundedVec::truncate_from(
                std::iter::repeat_n(U256::from(1), 2).collect::<Vec<_>>(),
            );

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
                BoundedVec::truncate_from((3..=8).map(U256::from).collect::<Vec<_>>());

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
            assert!(Triumvirate::<Test>::get().is_empty());

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
        let (proposal_hash1, proposal_index1) = create_custom_proposal!(
            U256::from(1),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 1i32.to_be_bytes().to_vec())],
            }
        );
        let (proposal_hash2, proposal_index2) = create_custom_proposal!(
            U256::from(2),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 2i32.to_be_bytes().to_vec())],
            }
        );
        let (proposal_hash3, proposal_index3) = create_custom_proposal!(
            U256::from(3),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 3i32.to_be_bytes().to_vec())],
            }
        );
        assert_eq!(
            Triumvirate::<Test>::get(),
            vec![U256::from(1001), U256::from(1002), U256::from(1003)]
        );

        vote_aye_on_proposed!(U256::from(1001), proposal_hash1, proposal_index1);

        vote_nay_on_proposed!(U256::from(1002), proposal_hash2, proposal_index2);
        vote_aye_on_proposed!(U256::from(1003), proposal_hash2, proposal_index2);

        vote_nay_on_proposed!(U256::from(1001), proposal_hash3, proposal_index3);
        vote_aye_on_proposed!(U256::from(1002), proposal_hash3, proposal_index3);

        let triumvirate =
            BoundedVec::truncate_from(vec![U256::from(1001), U256::from(1003), U256::from(1004)]);
        assert_ok!(Pallet::<Test>::set_triumvirate(
            RuntimeOrigin::root(),
            triumvirate.clone()
        ));
        assert_eq!(Triumvirate::<Test>::get(), triumvirate);
        let voting1 = TriumvirateVoting::<Test>::get(proposal_hash1).unwrap();
        assert_eq!(voting1.ayes.to_vec(), vec![U256::from(1001)]);
        assert!(voting1.nays.to_vec().is_empty());
        let voting2 = TriumvirateVoting::<Test>::get(proposal_hash2).unwrap();
        assert_eq!(voting2.ayes.to_vec(), vec![U256::from(1003)]);
        assert!(voting2.nays.to_vec().is_empty());
        let voting3 = TriumvirateVoting::<Test>::get(proposal_hash3).unwrap();
        assert!(voting3.ayes.to_vec().is_empty());
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
            let triumvirate = BoundedVec::truncate_from(
                std::iter::repeat_n(U256::from(1001), 2).collect::<Vec<_>>(),
            );

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
                BoundedVec::truncate_from((3..=8).map(U256::from).collect::<Vec<_>>());

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
        assert_eq!(proposal_index, 0);
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
            TriumvirateVoting::<Test>::get(proposal_hash),
            Some(TriumvirateVotes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::new(),
                end: now + MotionDuration::get(),
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ProposalSubmitted {
                account: U256::from(1),
                proposal_index: 0,
                proposal_hash,
                voting_end: now + MotionDuration::get(),
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
                items: std::iter::repeat_n(key_value, 50).collect::<Vec<_>>(),
            },
        ));
        let length_bound = proposal.encoded_size() as u32;

        let proposal_index = ProposalCount::<Test>::get();
        assert_eq!(proposal_index, 0);
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
        assert!(<Test as pallet::Config>::Preimages::have(bounded_proposal));
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            TriumvirateVoting::<Test>::get(proposal_hash),
            Some(TriumvirateVotes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::new(),
                end: now + MotionDuration::get(),
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ProposalSubmitted {
                account: U256::from(1),
                proposal_index: 0,
                proposal_hash,
                voting_end: now + MotionDuration::get(),
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
        let (proposal_hash, proposal_index) = create_proposal!();

        vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        vote_aye_on_proposed!(U256::from(1002), proposal_hash, proposal_index);

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
                            format!("Foobar{i}").as_bytes().to_vec(),
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
fn triumirate_vote_aye_as_first_voter_works() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        let approve = true;
        assert_ok!(Pallet::<Test>::vote_on_proposed(
            RuntimeOrigin::signed(U256::from(1001)),
            proposal_hash,
            proposal_index,
            approve
        ));

        let votes = TriumvirateVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![U256::from(1001)]);
        assert!(votes.nays.to_vec().is_empty());
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnProposal {
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
fn triumvirate_vote_nay_as_first_voter_works() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        let approve = false;
        assert_ok!(Pallet::<Test>::vote_on_proposed(
            RuntimeOrigin::signed(U256::from(1001)),
            proposal_hash,
            proposal_index,
            approve
        ));

        let votes = TriumvirateVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.nays.to_vec(), vec![U256::from(1001)]);
        assert!(votes.ayes.to_vec().is_empty());
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnProposal {
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
fn triumvirate_vote_can_be_updated() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        // Vote aye initially
        vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        let votes = TriumvirateVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![U256::from(1001)]);
        assert!(votes.nays.to_vec().is_empty());
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnProposal {
                account: U256::from(1001),
                proposal_hash,
                voted: true,
                yes: 1,
                no: 0,
            })
        );

        // Then vote nay, replacing the aye vote
        vote_nay_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        let votes = TriumvirateVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.nays.to_vec(), vec![U256::from(1001)]);
        assert!(votes.ayes.to_vec().is_empty());
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnProposal {
                account: U256::from(1001),
                proposal_hash,
                voted: false,
                yes: 0,
                no: 1,
            })
        );

        // Then vote aye again, replacing the nay vote
        vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        let votes = TriumvirateVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![U256::from(1001)]);
        assert!(votes.nays.to_vec().is_empty());
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnProposal {
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
fn two_triumvirate_aye_votes_schedule_proposal() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        vote_nay_on_proposed!(U256::from(1002), proposal_hash, proposal_index);
        vote_aye_on_proposed!(U256::from(1003), proposal_hash, proposal_index);

        assert!(Proposals::<Test>::get().is_empty());
        assert!(!TriumvirateVoting::<Test>::contains_key(proposal_hash));
        assert_eq!(Scheduled::<Test>::get(), vec![proposal_hash]);
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            CollectiveVoting::<Test>::get(proposal_hash),
            Some(CollectiveVotes {
                index: proposal_index,
                initial_dispatch_time: now + MotionDuration::get(),
                delay: Zero::zero(),
            })
        );
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            get_scheduler_proposal_task(proposal_hash).unwrap().0,
            now + MotionDuration::get()
        );
        assert_eq!(
            nth_last_event(2),
            RuntimeEvent::Governance(Event::<Test>::VotedOnProposal {
                account: U256::from(1003),
                proposal_hash,
                voted: true,
                yes: 2,
                no: 1,
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ProposalScheduled { proposal_hash })
        );
    });
}

#[test]
fn two_triumvirate_nay_votes_cancel_proposal() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        vote_nay_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        vote_aye_on_proposed!(U256::from(1002), proposal_hash, proposal_index);
        vote_nay_on_proposed!(U256::from(1003), proposal_hash, proposal_index);

        assert!(Proposals::<Test>::get().is_empty());
        assert!(!TriumvirateVoting::<Test>::contains_key(proposal_hash));
        assert!(Scheduled::<Test>::get().is_empty());
        assert!(ProposalOf::<Test>::get(proposal_hash).is_none());
        assert_eq!(
            nth_last_event(1),
            RuntimeEvent::Governance(Event::<Test>::VotedOnProposal {
                account: U256::from(1003),
                proposal_hash,
                voted: false,
                yes: 1,
                no: 2,
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ProposalCancelled { proposal_hash })
        );
    });
}

#[test]
fn triumvirate_vote_as_bad_origin_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
                RuntimeOrigin::root(),
                proposal_hash,
                proposal_index,
                true
            ),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
                RuntimeOrigin::none(),
                proposal_hash,
                proposal_index,
                true
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn triumvirate_vote_as_non_triumvirate_member_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
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
fn triumvirate_vote_on_missing_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let invalid_proposal_hash =
            <Test as frame_system::Config>::Hashing::hash(b"Invalid proposal");
        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
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
fn triumvirate_vote_on_scheduled_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        vote_aye_on_proposed!(U256::from(1002), proposal_hash, proposal_index);

        assert!(Proposals::<Test>::get().is_empty());
        assert_eq!(Scheduled::<Test>::get(), vec![proposal_hash]);

        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
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
fn triumvirate_vote_on_proposal_with_wrong_index_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
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
fn triumvirate_vote_after_voting_period_ended_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        let now = frame_system::Pallet::<Test>::block_number();
        run_to_block(now + MotionDuration::get() + 1);

        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
                RuntimeOrigin::signed(U256::from(1001)),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::VotingPeriodEnded
        );
    });
}

#[test]
fn duplicate_triumvirate_vote_on_proposal_already_voted_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        let aye_voter = RuntimeOrigin::signed(U256::from(1001));
        let approve = true;
        assert_ok!(Pallet::<Test>::vote_on_proposed(
            aye_voter.clone(),
            proposal_hash,
            proposal_index,
            approve
        ));
        assert_noop!(
            Pallet::<Test>::vote_on_proposed(aye_voter, proposal_hash, proposal_index, approve),
            Error::<Test>::DuplicateVote
        );

        let nay_voter = RuntimeOrigin::signed(U256::from(1002));
        let approve = false;
        assert_ok!(Pallet::<Test>::vote_on_proposed(
            nay_voter.clone(),
            proposal_hash,
            proposal_index,
            approve
        ));
        assert_noop!(
            Pallet::<Test>::vote_on_proposed(nay_voter, proposal_hash, proposal_index, approve),
            Error::<Test>::DuplicateVote
        );
    });
}

#[test]
fn triumvirate_aye_vote_on_proposal_with_too_many_scheduled_fails() {
    TestState::default().build_and_execute(|| {
        // We fill the scheduled proposals up to the maximum.
        for i in 0..MaxScheduled::get() {
            let (proposal_hash, proposal_index) = create_custom_proposal!(
                U256::from(1),
                frame_system::Call::<Test>::set_storage {
                    items: vec![(b"Foobar".to_vec(), i.to_be_bytes().to_vec())],
                }
            );
            vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
            vote_aye_on_proposed!(U256::from(1002), proposal_hash, proposal_index);
        }

        let (proposal_hash, proposal_index) = create_proposal!();

        vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        assert_noop!(
            Pallet::<Test>::vote_on_proposed(
                RuntimeOrigin::signed(U256::from(1002)),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::TooManyScheduled
        );
    });
}

// Named collective voting tests removed — all collective voting is now anonymous via bLSAG ring signatures.
// See the `anonymous_voting` module at the bottom of this file for threshold/delay/cancellation tests.

#[test]
fn collective_rotation_run_correctly_at_rotation_period() {
    TestState::default().build_and_execute(|| {
        let next_economic_collective = (1..=ECONOMIC_COLLECTIVE_SIZE)
            .map(|i| U256::from(4000 + i))
            .collect::<Vec<_>>();
        let next_building_collective = (1..=BUILDING_COLLECTIVE_SIZE)
            .map(|i| U256::from(5000 + i))
            .collect::<Vec<_>>();

        assert_eq!(
            EconomicCollective::<Test>::get().len(),
            ECONOMIC_COLLECTIVE_SIZE as usize,
        );
        assert_ne!(
            EconomicCollective::<Test>::get().to_vec(),
            next_economic_collective
        );
        assert_eq!(
            BuildingCollective::<Test>::get().len(),
            BUILDING_COLLECTIVE_SIZE as usize,
        );
        assert_ne!(
            BuildingCollective::<Test>::get().to_vec(),
            next_building_collective
        );

        set_next_economic_collective!(next_economic_collective.clone());
        set_next_building_collective!(next_building_collective.clone());

        run_to_block(CollectiveRotationPeriod::get());

        assert_eq!(
            EconomicCollective::<Test>::get().to_vec(),
            next_economic_collective
        );
        assert_eq!(
            BuildingCollective::<Test>::get().to_vec(),
            next_building_collective
        );
    });
}

#[macro_export]
macro_rules! create_custom_proposal {
    ($proposer:expr, $call:expr) => {{
        let proposal: Box<<Test as frame_system::Config>::RuntimeCall> = Box::new($call.into());
        let length_bound = proposal.encoded_size() as u32;
        let proposal_hash = <Test as frame_system::Config>::Hashing::hash_of(&proposal);
        let proposal_index = ProposalCount::<Test>::get();

        assert_ok!(Pallet::<Test>::propose(
            RuntimeOrigin::signed($proposer),
            proposal.clone(),
            length_bound
        ));

        (proposal_hash, proposal_index)
    }};
}

#[macro_export]
macro_rules! create_proposal {
    () => {{
        create_custom_proposal!(
            U256::from(1),
            frame_system::Call::<Test>::set_storage {
                items: vec![(b"Foobar".to_vec(), 42u32.to_be_bytes().to_vec())],
            }
        )
    }};
}

#[macro_export]
macro_rules! create_scheduled_proposal {
    () => {{
        let (proposal_hash, proposal_index) = create_proposal!();
        vote_aye_on_proposed!(U256::from(1001), proposal_hash, proposal_index);
        vote_aye_on_proposed!(U256::from(1002), proposal_hash, proposal_index);
        (proposal_hash, proposal_index)
    }};
}

#[macro_export]
macro_rules! vote_aye_on_proposed {
    ($voter:expr, $proposal_hash:expr, $proposal_index:expr) => {{
        assert_ok!(Pallet::<Test>::vote_on_proposed(
            RuntimeOrigin::signed($voter),
            $proposal_hash,
            $proposal_index,
            true
        ));
    }};
}

#[macro_export]
macro_rules! vote_nay_on_proposed {
    ($voter:expr, $proposal_hash:expr, $proposal_index:expr) => {{
        assert_ok!(Pallet::<Test>::vote_on_proposed(
            RuntimeOrigin::signed($voter),
            $proposal_hash,
            $proposal_index,
            false
        ));
    }};
}

pub(crate) fn get_scheduler_proposal_task(
    proposal_hash: <Test as frame_system::Config>::Hash,
) -> Option<pallet_scheduler::TaskAddress<BlockNumberFor<Test>>> {
    let task_name: [u8; 32] = proposal_hash.as_ref().try_into().unwrap();
    pallet_scheduler::Lookup::<Test>::get(task_name)
}

// ==========================================================================
// Anonymous voting tests
// ==========================================================================

mod anonymous_voting {
    use super::*;
    use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, scalar::Scalar};
    use rand::rngs::OsRng;
    use rand_core::{CryptoRng, RngCore};

    fn random_keypair(rng: &mut (impl CryptoRng + RngCore)) -> ([u8; 32], [u8; 32]) {
        let k = Scalar::random(rng);
        let p = (k * RISTRETTO_BASEPOINT_POINT).compress().to_bytes();
        (k.to_bytes(), p)
    }

    /// Convert a Ristretto public key (32 bytes) to U256 AccountId.
    /// U256 encodes as little-endian 32 bytes via SCALE, so this round-trips.
    fn pk_to_account(pk: &[u8; 32]) -> U256 {
        U256::from_little_endian(pk)
    }

    /// Generate `n` Ristretto keypairs and set them as economic collective members.
    /// Remaining economic slots and all building slots are filled with non-Ristretto U256 values
    /// (they won't be in the ring since they're not valid Ristretto points).
    fn setup_ristretto_collective(n: usize) -> (Vec<[u8; 32]>, Vec<[u8; 32]>) {
        let mut rng = OsRng;
        let mut sks = Vec::new();
        let mut pks = Vec::new();
        let mut economic = Vec::new();

        for _ in 0..n.min(ECONOMIC_COLLECTIVE_SIZE as usize) {
            let (sk, pk) = random_keypair(&mut rng);
            sks.push(sk);
            pks.push(pk);
            economic.push(pk_to_account(&pk));
        }
        // Fill remaining economic slots with values that are NOT valid Ristretto points.
        // U256::MAX - i encodes as bytes with high bits set, which cannot be valid
        // compressed Ristretto points.
        for i in economic.len()..ECONOMIC_COLLECTIVE_SIZE as usize {
            economic.push(U256::MAX - U256::from(i));
        }

        let mut building = Vec::new();
        for _i in n.min(ECONOMIC_COLLECTIVE_SIZE as usize)..n {
            let (sk, pk) = random_keypair(&mut rng);
            sks.push(sk);
            pks.push(pk);
            building.push(pk_to_account(&pk));
        }
        for i in building.len()..BUILDING_COLLECTIVE_SIZE as usize {
            building.push(U256::MAX - U256::from(100 + i));
        }

        set_next_economic_collective!(economic);
        set_next_building_collective!(building);
        // Trigger rotation to apply the new collectives
        Pallet::<Test>::rotate_collectives();

        (sks, pks)
    }

    /// Mine a PoW nonce for a given vote payload. Difficulty is 1 in tests.
    fn mine_pow(
        proposal_hash: <Test as frame_system::Config>::Hash,
        approve: bool,
        signature: &stp_crypto::BlsagSignature,
    ) -> u64 {
        for nonce in 0u64.. {
            if Pallet::<Test>::verify_pow(proposal_hash, approve, signature, nonce).is_ok() {
                return nonce;
            }
        }
        unreachable!()
    }

    /// Cast an anonymous vote and return the signature (for key image tracking).
    fn cast_anonymous_vote(
        proposal_hash: <Test as frame_system::Config>::Hash,
        proposal_index: ProposalIndex,
        sk: &[u8; 32],
        ring: &[[u8; 32]],
        approve: bool,
    ) -> stp_crypto::BlsagSignature {
        let mut rng = OsRng;
        let sig = stp_crypto::sign(sk, ring, proposal_hash.as_ref(), &mut rng).unwrap();
        let nonce = mine_pow(proposal_hash, approve, &sig);
        assert_ok!(Pallet::<Test>::anonymous_vote_on_scheduled(
            RuntimeOrigin::none(),
            proposal_hash,
            proposal_index,
            approve,
            sig.clone(),
            nonce,
        ));
        sig
    }

    /// Set up collectives with `n` valid Ristretto members, create a scheduled proposal,
    /// and return everything needed for anonymous voting.
    fn setup_anonymous_vote(
        n: usize,
    ) -> (
        <Test as frame_system::Config>::Hash,
        ProposalIndex,
        Vec<[u8; 32]>,
        Vec<[u8; 32]>,
    ) {
        let (sks, _pks) = setup_ristretto_collective(n);
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();
        let ring = ProposalRing::<Test>::get(proposal_hash)
            .expect("ring should be frozen")
            .to_vec();
        assert_eq!(ring.len(), n);
        (proposal_hash, proposal_index, sks, ring)
    }

    #[test]
    fn ring_uses_account_id_bytes_directly() {
        TestState::default().build_and_execute(|| {
            let (_sks, pks) = setup_ristretto_collective(3);
            let (proposal_hash, _) = create_scheduled_proposal!();

            let ring = ProposalRing::<Test>::get(proposal_hash).unwrap();
            assert_eq!(ring.len(), 3);

            // Ring members are the raw public key bytes of the collective members
            for pk in &pks {
                assert!(ring.contains(pk));
            }
        });
    }

    #[test]
    fn ring_frozen_at_schedule_time() {
        TestState::default().build_and_execute(|| {
            let (_sks, pks) = setup_ristretto_collective(3);
            let (proposal_hash, _) = create_scheduled_proposal!();
            let ring = ProposalRing::<Test>::get(proposal_hash).unwrap();
            assert_eq!(ring.len(), 3);

            // Rotate collectives to different members AFTER scheduling
            let mut rng = OsRng;
            let mut new_economic = Vec::new();
            for _ in 0..ECONOMIC_COLLECTIVE_SIZE as usize {
                let (_, pk) = random_keypair(&mut rng);
                new_economic.push(pk_to_account(&pk));
            }
            set_next_economic_collective!(new_economic);
            Pallet::<Test>::rotate_collectives();

            // Ring should still be the original 3
            let ring_after = ProposalRing::<Test>::get(proposal_hash).unwrap();
            assert_eq!(ring_after.len(), 3);
            for pk in &pks {
                assert!(ring_after.contains(pk));
            }
        });
    }

    #[test]
    fn no_ring_when_fewer_than_2_valid_ristretto_members() {
        TestState::default().build_and_execute(|| {
            // Only 1 valid Ristretto member, rest are invalid U256 values
            let (_sks, _pks) = setup_ristretto_collective(1);
            let (proposal_hash, _) = create_scheduled_proposal!();
            // Ring should NOT be stored (need >= 2 valid Ristretto points)
            assert!(ProposalRing::<Test>::get(proposal_hash).is_none());
        });
    }

    #[test]
    fn anonymous_vote_works() {
        TestState::default().build_and_execute(|| {
            let (proposal_hash, proposal_index, sks, ring) = setup_anonymous_vote(3);

            let sig = cast_anonymous_vote(proposal_hash, proposal_index, &sks[0], &ring, true);

            assert_eq!(AnonymousAyeCount::<Test>::get(proposal_hash), 1);
            assert_eq!(AnonymousNayCount::<Test>::get(proposal_hash), 0);
            assert_eq!(
                AnonymousVotes::<Test>::get(proposal_hash, sig.key_image),
                Some(true)
            );
            assert!(matches!(
                last_event(),
                RuntimeEvent::Governance(Event::AnonymousVoteCast {
                    approve: true,
                    yes: 1,
                    no: 0,
                    ..
                })
            ));
        });
    }

    #[test]
    fn anonymous_vote_can_change_direction() {
        TestState::default().build_and_execute(|| {
            let mut rng = OsRng;
            let (proposal_hash, proposal_index, sks, ring) = setup_anonymous_vote(3);

            // Vote aye first
            let sig1 = stp_crypto::sign(&sks[0], &ring, proposal_hash.as_ref(), &mut rng).unwrap();
            let nonce1 = mine_pow(proposal_hash, true, &sig1);
            assert_ok!(Pallet::<Test>::anonymous_vote_on_scheduled(
                RuntimeOrigin::none(),
                proposal_hash,
                proposal_index,
                true,
                sig1.clone(),
                nonce1,
            ));
            assert_eq!(AnonymousAyeCount::<Test>::get(proposal_hash), 1);

            // Change to nay (same key image)
            let sig2 = stp_crypto::sign(&sks[0], &ring, proposal_hash.as_ref(), &mut rng).unwrap();
            assert_eq!(sig1.key_image, sig2.key_image);
            let nonce2 = mine_pow(proposal_hash, false, &sig2);
            assert_ok!(Pallet::<Test>::anonymous_vote_on_scheduled(
                RuntimeOrigin::none(),
                proposal_hash,
                proposal_index,
                false,
                sig2,
                nonce2,
            ));

            assert_eq!(AnonymousAyeCount::<Test>::get(proposal_hash), 0);
            assert_eq!(AnonymousNayCount::<Test>::get(proposal_hash), 1);

            let events: Vec<_> = System::events().into_iter().map(|e| e.event).collect();
            assert!(events.iter().any(|e| matches!(
                e,
                RuntimeEvent::Governance(Event::AnonymousVoteUpdated {
                    approve: false,
                    yes: 0,
                    no: 1,
                    ..
                })
            )));
        });
    }

    #[test]
    fn anonymous_vote_with_invalid_signature_fails() {
        TestState::default().build_and_execute(|| {
            let mut rng = OsRng;
            let (proposal_hash, proposal_index, sks, ring) = setup_anonymous_vote(3);

            let mut signature =
                stp_crypto::sign(&sks[0], &ring, proposal_hash.as_ref(), &mut rng).unwrap();
            signature.challenge[0] ^= 0xff;
            let pow_nonce = mine_pow(proposal_hash, true, &signature);

            assert_noop!(
                Pallet::<Test>::anonymous_vote_on_scheduled(
                    RuntimeOrigin::none(),
                    proposal_hash,
                    proposal_index,
                    true,
                    signature,
                    pow_nonce,
                ),
                Error::<Test>::RingSignatureVerificationFailed
            );
        });
    }

    #[test]
    fn anonymous_vote_with_invalid_pow_fails() {
        TestState::default().build_and_execute(|| {
            let mut rng = OsRng;
            let (proposal_hash, proposal_index, sks, ring) = setup_anonymous_vote(3);

            let signature =
                stp_crypto::sign(&sks[0], &ring, proposal_hash.as_ref(), &mut rng).unwrap();
            // Mine PoW for approve=false, but submit with approve=true
            let wrong_nonce = mine_pow(proposal_hash, false, &signature);
            assert_noop!(
                Pallet::<Test>::anonymous_vote_on_scheduled(
                    RuntimeOrigin::none(),
                    proposal_hash,
                    proposal_index,
                    true,
                    signature,
                    wrong_nonce,
                ),
                Error::<Test>::InvalidPowProof
            );
        });
    }

    #[test]
    fn anonymous_vote_on_non_scheduled_proposal_fails() {
        TestState::default().build_and_execute(|| {
            let mut rng = OsRng;
            let (sks, pks) = setup_ristretto_collective(3);
            let (proposal_hash, proposal_index) = create_proposal!();

            let signature =
                stp_crypto::sign(&sks[0], &pks, proposal_hash.as_ref(), &mut rng).unwrap();
            let pow_nonce = mine_pow(proposal_hash, true, &signature);

            assert_noop!(
                Pallet::<Test>::anonymous_vote_on_scheduled(
                    RuntimeOrigin::none(),
                    proposal_hash,
                    proposal_index,
                    true,
                    signature,
                    pow_nonce,
                ),
                Error::<Test>::ProposalNotScheduled
            );
        });
    }

    #[test]
    fn anonymous_vote_cleanup_on_fast_track() {
        TestState::default().build_and_execute(|| {
            // Use all 32 members as valid Ristretto keys so we can reach thresholds
            let (proposal_hash, proposal_index, sks, ring) = setup_anonymous_vote(32);

            // Cast one aye vote
            cast_anonymous_vote(proposal_hash, proposal_index, &sks[0], &ring, true);
            assert_eq!(AnonymousAyeCount::<Test>::get(proposal_hash), 1);

            // Cast enough aye votes to reach fast-track threshold (67% of 32 = 22)
            let threshold = FastTrackThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE) as usize;
            for i in 1..threshold {
                cast_anonymous_vote(proposal_hash, proposal_index, &sks[i], &ring, true);
            }

            // Proposal should have been fast-tracked, storage cleaned up
            assert!(CollectiveVoting::<Test>::get(proposal_hash).is_none());
            assert!(ProposalRing::<Test>::get(proposal_hash).is_none());
            assert_eq!(AnonymousAyeCount::<Test>::get(proposal_hash), 0);
            assert_eq!(AnonymousNayCount::<Test>::get(proposal_hash), 0);
            assert_eq!(
                last_event(),
                RuntimeEvent::Governance(Event::<Test>::ScheduledProposalFastTracked {
                    proposal_hash
                })
            );
        });
    }

    #[test]
    fn anonymous_nay_votes_above_threshold_cancels() {
        TestState::default().build_and_execute(|| {
            let (proposal_hash, proposal_index, sks, ring) = setup_anonymous_vote(32);

            let threshold = CancellationThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE) as usize;
            for i in 0..threshold {
                cast_anonymous_vote(proposal_hash, proposal_index, &sks[i], &ring, false);
            }

            assert!(Scheduled::<Test>::get().is_empty());
            assert!(CollectiveVoting::<Test>::get(proposal_hash).is_none());
            assert!(get_scheduler_proposal_task(proposal_hash).is_none());
            assert_eq!(
                last_event(),
                RuntimeEvent::Governance(Event::<Test>::ScheduledProposalCancelled {
                    proposal_hash
                })
            );
        });
    }

    #[test]
    fn anonymous_nay_votes_adjust_delay() {
        TestState::default().build_and_execute(|| {
            let now = frame_system::Pallet::<Test>::block_number();
            let (proposal_hash, proposal_index, sks, ring) = setup_anonymous_vote(32);
            let voting = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
            assert_eq!(voting.delay, 0);

            // One nay vote should increase the delay
            cast_anonymous_vote(proposal_hash, proposal_index, &sks[0], &ring, false);
            let initial_delay = InitialSchedulingDelay::get() as f64;
            let initial_dispatch_time = now + MotionDuration::get();
            let expected_delay = (initial_delay * 1.5_f64.powi(1)).ceil() as u64;

            let voting = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
            assert_eq!(voting.delay, expected_delay);
            assert_eq!(
                get_scheduler_proposal_task(proposal_hash).unwrap().0,
                initial_dispatch_time + expected_delay
            );

            // Adding an aye vote should reduce the delay (net score goes to 0)
            cast_anonymous_vote(proposal_hash, proposal_index, &sks[1], &ring, true);
            let voting = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
            assert_eq!(voting.delay, 0);
        });
    }
}
