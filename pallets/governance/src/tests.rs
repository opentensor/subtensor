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
                ayes: BoundedVec::new(),
                nays: BoundedVec::new(),
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

#[test]
fn collective_member_aye_vote_on_scheduled_proposal_works() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();

        // Add an aye vote from an economic collective member.
        let economic_member = U256::from(2001);
        assert_ok!(Pallet::<Test>::vote_on_scheduled(
            RuntimeOrigin::signed(economic_member),
            proposal_hash,
            proposal_index,
            true
        ));
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            CollectiveVoting::<Test>::get(proposal_hash),
            Some(CollectiveVotes {
                index: proposal_index,
                ayes: BoundedVec::truncate_from(vec![economic_member]),
                nays: BoundedVec::new(),
                initial_dispatch_time: now + MotionDuration::get(),
                delay: Zero::zero(),
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: economic_member,
                proposal_hash,
                voted: true,
                yes: 1,
                no: 0,
            })
        );

        // Add a second aye vote from a building collective member.
        let building_member = U256::from(3001);
        assert_ok!(Pallet::<Test>::vote_on_scheduled(
            RuntimeOrigin::signed(building_member),
            proposal_hash,
            proposal_index,
            true
        ));

        assert_eq!(
            CollectiveVoting::<Test>::get(proposal_hash),
            Some(CollectiveVotes {
                index: proposal_index,
                ayes: BoundedVec::truncate_from(vec![economic_member, building_member]),
                nays: BoundedVec::new(),
                initial_dispatch_time: now + MotionDuration::get(),
                delay: Zero::zero(),
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: building_member,
                proposal_hash,
                voted: true,
                yes: 2,
                no: 0,
            })
        );
    });
}

#[test]
fn collective_member_votes_succession_on_scheduled_proposal_adjust_delay_and_can_fast_track() {
    TestState::default().build_and_execute(|| {
        let now = frame_system::Pallet::<Test>::block_number();
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();
        let voting = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(voting.delay, 0);

        // Adding a nay vote increases the delay
        vote_nay_on_scheduled!(U256::from(2001), proposal_hash, proposal_index);
        let initial_delay = InitialSchedulingDelay::get() as f64;
        let initial_dispatch_time = now + MotionDuration::get();
        let delay = (initial_delay * 1.5_f64.powi(1)).ceil() as u64;
        assert_eq!(
            CollectiveVoting::<Test>::get(proposal_hash),
            Some(CollectiveVotes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::truncate_from(vec![U256::from(2001)]),
                initial_dispatch_time,
                delay,
            })
        );
        assert_eq!(
            get_scheduler_proposal_task(proposal_hash).unwrap().0,
            initial_dispatch_time + delay
        );
        assert_eq!(
            nth_last_event(3),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: U256::from(2001),
                proposal_hash,
                voted: false,
                yes: 0,
                no: 1,
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ScheduledProposalDelayAdjusted {
                proposal_hash,
                dispatch_time: DispatchTime::At(initial_dispatch_time + delay),
            })
        );

        // Adding a second nay vote increases the delay
        vote_nay_on_scheduled!(U256::from(2002), proposal_hash, proposal_index);
        let delay = (initial_delay * 1.5_f64.powi(2)).ceil() as u64;
        assert_eq!(
            CollectiveVoting::<Test>::get(proposal_hash),
            Some(CollectiveVotes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::truncate_from(vec![U256::from(2001), U256::from(2002)]),
                initial_dispatch_time,
                delay,
            })
        );
        assert_eq!(
            get_scheduler_proposal_task(proposal_hash).unwrap().0,
            initial_dispatch_time + delay
        );
        assert_eq!(
            nth_last_event(3),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: U256::from(2002),
                proposal_hash,
                voted: false,
                yes: 0,
                no: 2,
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ScheduledProposalDelayAdjusted {
                proposal_hash,
                dispatch_time: DispatchTime::At(initial_dispatch_time + delay),
            })
        );

        // Adding a third nay vote increases the delay
        vote_nay_on_scheduled!(U256::from(2003), proposal_hash, proposal_index);
        let delay = (initial_delay * 1.5_f64.powi(3)) as u64;
        assert_eq!(
            CollectiveVoting::<Test>::get(proposal_hash),
            Some(CollectiveVotes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::truncate_from(vec![
                    U256::from(2001),
                    U256::from(2002),
                    U256::from(2003)
                ]),
                initial_dispatch_time,
                delay,
            })
        );
        assert_eq!(
            get_scheduler_proposal_task(proposal_hash).unwrap().0,
            initial_dispatch_time + delay
        );
        assert_eq!(
            nth_last_event(3),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: U256::from(2003),
                proposal_hash,
                voted: false,
                yes: 0,
                no: 3,
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ScheduledProposalDelayAdjusted {
                proposal_hash,
                dispatch_time: DispatchTime::At(initial_dispatch_time + delay),
            })
        );

        // Adding a aye vote decreases the delay because net score become lower
        vote_aye_on_scheduled!(U256::from(2004), proposal_hash, proposal_index);
        let delay = (initial_delay * 1.5_f64.powi(2)).ceil() as u64;
        assert_eq!(
            CollectiveVoting::<Test>::get(proposal_hash),
            Some(CollectiveVotes {
                index: proposal_index,
                ayes: BoundedVec::truncate_from(vec![U256::from(2004)]),
                nays: BoundedVec::truncate_from(vec![
                    U256::from(2001),
                    U256::from(2002),
                    U256::from(2003)
                ]),
                initial_dispatch_time,
                delay,
            })
        );
        assert_eq!(
            get_scheduler_proposal_task(proposal_hash).unwrap().0,
            initial_dispatch_time + delay
        );
        assert_eq!(
            nth_last_event(3),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: U256::from(2004),
                proposal_hash,
                voted: true,
                yes: 1,
                no: 3,
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ScheduledProposalDelayAdjusted {
                proposal_hash,
                dispatch_time: DispatchTime::At(initial_dispatch_time + delay),
            })
        );

        // Now let's run some blocks until before the sheduled time
        run_to_block(initial_dispatch_time + delay - 5);
        // Task hasn't been executed yet
        assert!(get_scheduler_proposal_task(proposal_hash).is_some());

        // Adding a new aye vote should fast track the proposal because the delay will
        // fall below the elapsed time
        vote_aye_on_scheduled!(U256::from(2005), proposal_hash, proposal_index);
        assert!(CollectiveVoting::<Test>::get(proposal_hash).is_none());
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            get_scheduler_proposal_task(proposal_hash).unwrap().0,
            // Fast track here means next block scheduling
            now + 1
        );
        // The proposal is still scheduled, even if next block, we keep track of it
        assert_eq!(Scheduled::<Test>::get(), vec![proposal_hash]);
        assert_eq!(
            nth_last_event(3),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: U256::from(2005),
                proposal_hash,
                voted: true,
                yes: 2,
                no: 3,
            })
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ScheduledProposalFastTracked { proposal_hash })
        );

        // Now let run one block to see the proposal executed
        assert_eq!(sp_io::storage::get(b"Foobar"), None); // Not executed yet
        run_to_block(now + delay + 1);
        assert!(get_scheduler_proposal_task(proposal_hash).is_none());
        let stored_value = 42u32.to_be_bytes().to_vec().into();
        assert_eq!(sp_io::storage::get(b"Foobar"), Some(stored_value)); // Executed
    });
}

#[test]
fn collective_member_vote_on_scheduled_proposal_can_be_updated() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();
        let economic_member = U256::from(2001);

        // Vote aye initially as an economic collective member
        vote_aye_on_scheduled!(economic_member, proposal_hash, proposal_index);
        let votes = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![economic_member]);
        assert!(votes.nays.to_vec().is_empty());
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: economic_member,
                proposal_hash,
                voted: true,
                yes: 1,
                no: 0,
            })
        );

        // Then vote nay, replacing the aye vote
        vote_nay_on_scheduled!(economic_member, proposal_hash, proposal_index);
        let votes = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
        assert!(votes.ayes.to_vec().is_empty());
        assert_eq!(votes.nays.to_vec(), vec![economic_member]);
        assert_eq!(
            System::events().into_iter().rev().nth(3).unwrap().event,
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: economic_member,
                proposal_hash,
                voted: false,
                yes: 0,
                no: 1,
            })
        );

        // Then vote aye again, replacing the nay vote
        vote_aye_on_scheduled!(economic_member, proposal_hash, proposal_index);
        let votes = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
        assert_eq!(votes.ayes.to_vec(), vec![economic_member]);
        assert!(votes.nays.to_vec().is_empty());
        assert_eq!(
            System::events().into_iter().rev().nth(3).unwrap().event,
            RuntimeEvent::Governance(Event::<Test>::VotedOnScheduled {
                account: economic_member,
                proposal_hash,
                voted: true,
                yes: 1,
                no: 0,
            })
        );
    });
}

#[test]
fn collective_member_aye_votes_above_threshold_on_scheduled_proposal_fast_tracks() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();
        let threshold = FastTrackThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE);
        let combined_collective = EconomicCollective::<Test>::get()
            .into_iter()
            .chain(BuildingCollective::<Test>::get().into_iter());

        for member in combined_collective.into_iter().take(threshold as usize) {
            vote_aye_on_scheduled!(member, proposal_hash, proposal_index);
        }

        assert!(CollectiveVoting::<Test>::get(proposal_hash).is_none());
        let now = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            get_scheduler_proposal_task(proposal_hash).unwrap().0,
            now + 1
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ScheduledProposalFastTracked { proposal_hash })
        );

        // Now let run one block to see the proposal executed
        assert_eq!(sp_io::storage::get(b"Foobar"), None); // Not executed yet
        run_to_block(now + 1);
        assert!(get_scheduler_proposal_task(proposal_hash).is_none());
        let stored_value = 42u32.to_be_bytes().to_vec().into();
        assert_eq!(sp_io::storage::get(b"Foobar"), Some(stored_value)); // Executed
    });
}

#[test]
fn collective_member_nay_votes_above_threshold_on_scheduled_proposal_cancels() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();
        let threshold = CancellationThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE);
        let combined_collective = EconomicCollective::<Test>::get()
            .into_iter()
            .chain(BuildingCollective::<Test>::get().into_iter());

        for member in combined_collective.into_iter().take(threshold as usize) {
            vote_nay_on_scheduled!(member, proposal_hash, proposal_index);
        }

        assert!(Scheduled::<Test>::get().is_empty());
        assert!(CollectiveVoting::<Test>::get(proposal_hash).is_none());
        assert!(get_scheduler_proposal_task(proposal_hash).is_none());
        assert_eq!(
            last_event(),
            RuntimeEvent::Governance(Event::<Test>::ScheduledProposalCancelled { proposal_hash })
        );
    });
}

#[test]
fn collective_member_aye_vote_triggering_fast_track_on_next_block_scheduled_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();
        let threshold = FastTrackThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE);
        let combined_collective = EconomicCollective::<Test>::get()
            .into_iter()
            .chain(BuildingCollective::<Test>::get().into_iter());

        let below_threshold = (threshold - 1) as usize;
        for member in combined_collective.clone().take(below_threshold) {
            vote_aye_on_scheduled!(member, proposal_hash, proposal_index);
        }

        let voting = CollectiveVoting::<Test>::get(proposal_hash).unwrap();
        run_to_block(voting.initial_dispatch_time - 1);

        let voter = combined_collective.skip(below_threshold).next().unwrap();
        assert_noop!(
            Pallet::<Test>::vote_on_scheduled(
                RuntimeOrigin::signed(voter),
                proposal_hash,
                proposal_index,
                true
            ),
            pallet_scheduler::Error::<Test>::RescheduleNoChange
        );
    });
}

#[test]
fn collective_member_vote_on_scheduled_proposal_from_non_collective_member_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();

        assert_noop!(
            Pallet::<Test>::vote_on_scheduled(
                RuntimeOrigin::signed(U256::from(42)),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::NotCollectiveMember
        );
    });
}

#[test]
fn collective_member_vote_on_non_scheduled_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_proposal!();

        assert_noop!(
            Pallet::<Test>::vote_on_scheduled(
                RuntimeOrigin::signed(U256::from(2001)),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::ProposalNotScheduled
        );
    });
}

#[test]
fn collective_member_vote_on_fast_tracked_scheduled_proposal_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();
        let threshold = FastTrackThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE);
        let combined_collective = EconomicCollective::<Test>::get()
            .into_iter()
            .chain(BuildingCollective::<Test>::get().into_iter());

        for member in combined_collective.clone().take(threshold as usize) {
            vote_aye_on_scheduled!(member, proposal_hash, proposal_index);
        }

        let voter = combined_collective.skip(threshold as usize).next().unwrap();
        assert_noop!(
            Pallet::<Test>::vote_on_scheduled(
                RuntimeOrigin::signed(voter),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::VotingPeriodEnded
        );
    });
}

#[test]
fn collective_member_vote_on_scheduled_proposal_with_wrong_index_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, _proposal_index) = create_scheduled_proposal!();

        assert_noop!(
            Pallet::<Test>::vote_on_scheduled(
                RuntimeOrigin::signed(U256::from(2001)),
                proposal_hash,
                42,
                true
            ),
            Error::<Test>::WrongProposalIndex
        );
    });
}

#[test]
fn duplicate_collective_member_vote_on_scheduled_proposal_already_voted_fails() {
    TestState::default().build_and_execute(|| {
        let (proposal_hash, proposal_index) = create_scheduled_proposal!();

        let aye_voter = U256::from(2001);
        vote_aye_on_scheduled!(aye_voter, proposal_hash, proposal_index);
        assert_noop!(
            Pallet::<Test>::vote_on_scheduled(
                RuntimeOrigin::signed(aye_voter),
                proposal_hash,
                proposal_index,
                true
            ),
            Error::<Test>::DuplicateVote
        );

        let nay_voter = U256::from(2002);
        vote_nay_on_scheduled!(nay_voter, proposal_hash, proposal_index);
        assert_noop!(
            Pallet::<Test>::vote_on_scheduled(
                RuntimeOrigin::signed(nay_voter),
                proposal_hash,
                proposal_index,
                false
            ),
            Error::<Test>::DuplicateVote
        );
    });
}

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

#[macro_export]
macro_rules! vote_aye_on_scheduled {
    ($voter:expr, $proposal_hash:expr, $proposal_index:expr) => {{
        assert_ok!(Pallet::<Test>::vote_on_scheduled(
            RuntimeOrigin::signed($voter),
            $proposal_hash,
            $proposal_index,
            true
        ));
    }};
}

#[macro_export]
macro_rules! vote_nay_on_scheduled {
    ($voter:expr, $proposal_hash:expr, $proposal_index:expr) => {{
        assert_ok!(Pallet::<Test>::vote_on_scheduled(
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
