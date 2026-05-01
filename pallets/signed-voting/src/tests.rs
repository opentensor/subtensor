#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use frame_support::{assert_noop, assert_ok, sp_runtime::Perbill};
use sp_core::U256;
use sp_runtime::DispatchError;
use subtensor_runtime_common::VoteTally;

use crate::{
    Error, Event as SignedVotingEvent, Pallet as SignedVotingPallet, SignedVoteTally, TallyOf,
    VotingFor, mock::*,
};

// -------- Section 1: Environment --------

#[test]
fn environment_works() {
    TestState::build_and_execute(|| {
        // No polls registered at start.
        let voters = vec![U256::from(1), U256::from(2), U256::from(3)];
        start_poll(0, VotingScheme::Signed, voters.clone());

        // on_poll_created populated TallyOf with total = voter_set.len().
        let tally = TallyOf::<Test>::get(0u32).expect("tally inserted");
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 0);
        assert_eq!(tally.total, 3);

        // No votes, no events, no tally updates yet.
        assert!(signed_voting_events().is_empty());
        assert!(take_tally_updates().is_empty());
    });
}

// -------- Section 2: vote — success paths --------

#[test]
fn vote_records_aye() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(
            0,
            VotingScheme::Signed,
            vec![alice, U256::from(2), U256::from(3)],
        );

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 1);
        assert_eq!(tally.nays, 0);
        assert_eq!(tally.total, 3);
        assert_eq!(VotingFor::<Test>::get(0u32, alice), Some(true));

        assert_eq!(
            signed_voting_events().last(),
            Some(&SignedVotingEvent::Voted {
                who: alice,
                poll_index: 0,
                approve: true,
                tally,
            })
        );
    });
}

#[test]
fn vote_records_nay() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(
            0,
            VotingScheme::Signed,
            vec![alice, U256::from(2), U256::from(3)],
        );

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            false,
        ));

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 1);
        assert_eq!(VotingFor::<Test>::get(0u32, alice), Some(false));
    });
}

#[test]
fn vote_change_flips_direction() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(
            0,
            VotingScheme::Signed,
            vec![alice, U256::from(2), U256::from(3)],
        );

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));
        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!((tally.ayes, tally.nays), (1, 0));

        // aye → nay
        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            false,
        ));
        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!((tally.ayes, tally.nays), (0, 1));

        // nay → aye (exercises the other branch of try_vote)
        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));
        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!((tally.ayes, tally.nays), (1, 0));
    });
}

#[test]
fn vote_aggregates_across_voters() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        let charlie = U256::from(3);
        start_poll(0, VotingScheme::Signed, vec![alice, bob, charlie]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));
        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(bob),
            0u32,
            false,
        ));
        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(charlie),
            0u32,
            true,
        ));

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 2);
        assert_eq!(tally.nays, 1);
        assert_eq!(tally.total, 3);
    });
}

#[test]
fn vote_pushes_tally_to_polls() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(
            0,
            VotingScheme::Signed,
            vec![alice, U256::from(2), U256::from(3)],
        );

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        let updates = take_tally_updates();
        assert_eq!(updates.len(), 1);
        let (idx, tally) = &updates[0];
        assert_eq!(*idx, 0);
        // approval = 1/3; rejection = 0; abstention = 2/3.
        assert_eq!(tally.approval, Perbill::from_rational(1u32, 3u32));
        assert_eq!(tally.rejection, Perbill::zero());
        assert_eq!(tally.abstention, Perbill::from_rational(2u32, 3u32));
    });
}

// -------- Section 3: vote — error paths --------

#[test]
fn vote_requires_signed_origin() {
    TestState::build_and_execute(|| {
        start_poll(0, VotingScheme::Signed, vec![U256::from(1)]);

        assert_noop!(
            SignedVotingPallet::<Test>::vote(RuntimeOrigin::root(), 0u32, true),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn vote_rejects_inactive_poll() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice]);
        complete_poll(0);

        assert_noop!(
            SignedVotingPallet::<Test>::vote(RuntimeOrigin::signed(alice), 0u32, true),
            Error::<Test>::PollNotOngoing
        );
    });
}

#[test]
fn vote_rejects_wrong_voting_scheme() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Anonymous, vec![alice]);

        assert_noop!(
            SignedVotingPallet::<Test>::vote(RuntimeOrigin::signed(alice), 0u32, true),
            Error::<Test>::InvalidVotingScheme
        );
    });
}

#[test]
fn vote_rejects_non_member() {
    TestState::build_and_execute(|| {
        let mallory = U256::from(999);
        start_poll(0, VotingScheme::Signed, vec![U256::from(1), U256::from(2)]);

        assert_noop!(
            SignedVotingPallet::<Test>::vote(RuntimeOrigin::signed(mallory), 0u32, true),
            Error::<Test>::NotInVoterSet
        );
    });
}

#[test]
fn vote_rejects_duplicate_same_direction() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        assert_noop!(
            SignedVotingPallet::<Test>::vote(RuntimeOrigin::signed(alice), 0u32, true),
            Error::<Test>::DuplicateVote
        );

        // Tally unchanged by the failing duplicate.
        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!((tally.ayes, tally.nays), (1, 0));
    });
}

// -------- Section 4: remove_vote --------

#[test]
fn remove_vote_happy_path_aye() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice, U256::from(2)]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));
        assert_ok!(SignedVotingPallet::<Test>::remove_vote(
            RuntimeOrigin::signed(alice),
            0u32,
        ));

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 0);
        assert_eq!(tally.total, 2);
        assert_eq!(VotingFor::<Test>::get(0u32, alice), None);

        assert_eq!(
            signed_voting_events().last(),
            Some(&SignedVotingEvent::VoteRemoved {
                who: alice,
                poll_index: 0,
                tally,
            })
        );
    });
}

#[test]
fn remove_vote_happy_path_nay() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice, U256::from(2)]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            false,
        ));
        assert_ok!(SignedVotingPallet::<Test>::remove_vote(
            RuntimeOrigin::signed(alice),
            0u32,
        ));

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.nays, 0);
        assert_eq!(VotingFor::<Test>::get(0u32, alice), None);
    });
}

#[test]
fn remove_vote_requires_signed_origin() {
    TestState::build_and_execute(|| {
        start_poll(0, VotingScheme::Signed, vec![U256::from(1)]);

        assert_noop!(
            SignedVotingPallet::<Test>::remove_vote(RuntimeOrigin::root(), 0u32),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn remove_vote_rejects_inactive_poll() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice]);
        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));
        complete_poll(0);

        assert_noop!(
            SignedVotingPallet::<Test>::remove_vote(RuntimeOrigin::signed(alice), 0u32),
            Error::<Test>::PollNotOngoing
        );
    });
}

#[test]
fn remove_vote_rejects_wrong_scheme() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Anonymous, vec![alice]);

        assert_noop!(
            SignedVotingPallet::<Test>::remove_vote(RuntimeOrigin::signed(alice), 0u32),
            Error::<Test>::InvalidVotingScheme
        );
    });
}

#[test]
fn remove_vote_rejects_non_member() {
    TestState::build_and_execute(|| {
        let mallory = U256::from(999);
        start_poll(0, VotingScheme::Signed, vec![U256::from(1)]);

        assert_noop!(
            SignedVotingPallet::<Test>::remove_vote(RuntimeOrigin::signed(mallory), 0u32),
            Error::<Test>::NotInVoterSet
        );
    });
}

#[test]
fn remove_vote_rejects_never_voted() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice]);

        assert_noop!(
            SignedVotingPallet::<Test>::remove_vote(RuntimeOrigin::signed(alice), 0u32),
            Error::<Test>::VoteNotFound
        );
    });
}

/// Documents quirk 5a: a voter who was in the voter set when casting a vote,
/// then got rotated out, cannot remove their own stale vote. Current
/// `ensure_part_of_voter_set` check fires before the removal logic. A defensive
/// UX fix would allow self-removal regardless of current membership.
#[test]
#[ignore = "5a quirk: remove_vote rejects rotated-out voters via NotInVoterSet; test asserts ideal behavior"]
fn remove_vote_allows_self_removal_post_rotation() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice, U256::from(2)]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        // Rotate alice out (without invoking remove_votes_for).
        remove_voter(0, alice);

        // IDEAL: alice can still remove her own vote.
        // ACTUAL: returns NotInVoterSet — this assertion fails today.
        assert_ok!(SignedVotingPallet::<Test>::remove_vote(
            RuntimeOrigin::signed(alice),
            0u32,
        ));
        assert_eq!(VotingFor::<Test>::get(0u32, alice), None);
    });
}

// -------- Section 5: PollHooks::on_poll_created --------

#[test]
fn on_poll_created_initializes_tally_with_voter_set_size() {
    TestState::build_and_execute(|| {
        let voters: Vec<U256> = (1..=5u32).map(U256::from).collect();
        start_poll(0, VotingScheme::Signed, voters);

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(
            tally,
            SignedVoteTally {
                ayes: 0,
                nays: 0,
                total: 5
            }
        );
    });
}

/// Active-poll tracking is implicit: every started poll has a `TallyOf`
/// entry until `on_poll_completed` removes it. There is no separate
/// `ActivePolls` cap to mismatch against the producer's queue limit.
#[test]
fn on_poll_created_tracks_polls_in_tally() {
    TestState::build_and_execute(|| {
        start_poll(0, VotingScheme::Signed, vec![U256::from(1)]);
        start_poll(1, VotingScheme::Signed, vec![U256::from(2)]);
        start_poll(2, VotingScheme::Signed, vec![U256::from(3)]);

        let mut keys: Vec<u32> = TallyOf::<Test>::iter_keys().collect();
        keys.sort();
        assert_eq!(keys, vec![0u32, 1, 2]);
    });
}

// -------- Section 6: PollHooks::on_poll_completed --------

#[test]
fn on_poll_completed_clears_votes_and_tally() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let bob = U256::from(2);
        start_poll(0, VotingScheme::Signed, vec![alice, bob]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));
        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(bob),
            0u32,
            false,
        ));
        assert!(TallyOf::<Test>::get(0u32).is_some());
        assert!(VotingFor::<Test>::get(0u32, alice).is_some());

        complete_poll(0);

        assert!(TallyOf::<Test>::get(0u32).is_none());
        assert_eq!(VotingFor::<Test>::get(0u32, alice), None);
        assert_eq!(VotingFor::<Test>::get(0u32, bob), None);
        // No active polls left — `TallyOf` is the implicit index and
        // `on_poll_completed` removes the entry.
        assert_eq!(TallyOf::<Test>::iter_keys().count(), 0);
    });
}

/// `on_poll_completed` clears every `VotingFor` entry for the poll via an
/// unbounded `clear_prefix(u32::MAX, None)`. Exercised with 200 voters to
/// catch any regression to a bounded / cursor-discarding version.
#[test]
fn on_poll_completed_clears_all_votes() {
    TestState::build_and_execute(|| {
        let voters: Vec<U256> = (1..=200u32).map(U256::from).collect();
        start_poll(0, VotingScheme::Signed, voters.clone());

        for v in &voters {
            assert_ok!(SignedVotingPallet::<Test>::vote(
                RuntimeOrigin::signed(*v),
                0u32,
                true,
            ));
        }

        complete_poll(0);

        for v in &voters {
            assert_eq!(VotingFor::<Test>::get(0u32, *v), None);
        }
        assert!(TallyOf::<Test>::get(0u32).is_none());
    });
}

// -------- Section 7: remove_votes_for --------

#[test]
fn remove_votes_for_clears_aye_vote() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(
            0,
            VotingScheme::Signed,
            vec![alice, U256::from(2), U256::from(3)],
        );

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        SignedVotingPallet::<Test>::remove_votes_for(&alice);

        // ayes decrement; total is *not* updated (B1 stale-total bug, covered
        // in an #[ignore] test below).
        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 0);
        assert_eq!(VotingFor::<Test>::get(0u32, alice), None);
    });
}

#[test]
fn remove_votes_for_clears_nay_vote() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice, U256::from(2)]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            false,
        ));

        SignedVotingPallet::<Test>::remove_votes_for(&alice);

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.nays, 0);
        assert_eq!(VotingFor::<Test>::get(0u32, alice), None);
    });
}

#[test]
fn remove_votes_for_iterates_active_polls() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice]);
        start_poll(1, VotingScheme::Signed, vec![alice, U256::from(2)]);
        start_poll(2, VotingScheme::Signed, vec![alice, U256::from(3)]);

        for idx in 0u32..3 {
            assert_ok!(SignedVotingPallet::<Test>::vote(
                RuntimeOrigin::signed(alice),
                idx,
                true,
            ));
        }

        SignedVotingPallet::<Test>::remove_votes_for(&alice);

        for idx in 0u32..3 {
            assert_eq!(VotingFor::<Test>::get(idx, alice), None);
        }
    });
}

#[test]
fn remove_votes_for_noop_for_non_voter() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        let mallory = U256::from(999);
        start_poll(0, VotingScheme::Signed, vec![alice]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        // mallory never voted. remove_votes_for should be a no-op for them.
        let tally_before = TallyOf::<Test>::get(0u32).unwrap();
        SignedVotingPallet::<Test>::remove_votes_for(&mallory);
        let tally_after = TallyOf::<Test>::get(0u32).unwrap();

        assert_eq!(tally_before, tally_after);
        assert_eq!(VotingFor::<Test>::get(0u32, alice), Some(true));
    });
}

#[test]
fn remove_votes_for_emits_invalidated_event() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(0, VotingScheme::Signed, vec![alice, U256::from(2)]);

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        SignedVotingPallet::<Test>::remove_votes_for(&alice);

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(
            signed_voting_events().last(),
            Some(&SignedVotingEvent::VoteInvalidated {
                who: alice,
                poll_index: 0,
                tally,
            })
        );
    });
}

/// `remove_votes_for` preserves `total`: the runtime rotates voters via
/// `swap_member` / `set_members`, which keep the voter-set size constant
/// and fill the slot a departing voter leaves. Decrementing `total` here
/// would break the denominator on swap (incoming member present but uncounted).
#[test]
fn remove_votes_for_preserves_total() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        start_poll(
            0,
            VotingScheme::Signed,
            vec![alice, U256::from(2), U256::from(3)],
        );

        assert_ok!(SignedVotingPallet::<Test>::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            true,
        ));

        SignedVotingPallet::<Test>::remove_votes_for(&alice);

        let tally = TallyOf::<Test>::get(0u32).unwrap();
        // Alice's vote is cleared; `total` stays at its creation-time value
        // of 3 — a replacement via swap_member fills her slot.
        assert_eq!(tally.total, 3);
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 0);
    });
}

/// `remove_votes_for` walks `TallyOf` directly, so it scales with the
/// number of *actually live* polls — there's no separate cap that could
/// silently drop entries from the cleanup set.
#[test]
fn remove_votes_for_clears_all_live_polls_regardless_of_count() {
    TestState::build_and_execute(|| {
        let alice = U256::from(1);
        // Far more polls than the old `MaxActivePolls = 3` cap allowed.
        for idx in 0u32..6 {
            start_poll(
                idx,
                VotingScheme::Signed,
                vec![alice, U256::from(100 + idx as u64)],
            );
        }

        for idx in 0u32..6 {
            assert_ok!(SignedVotingPallet::<Test>::vote(
                RuntimeOrigin::signed(alice),
                idx,
                true,
            ));
        }

        SignedVotingPallet::<Test>::remove_votes_for(&alice);

        for idx in 0u32..6 {
            assert_eq!(VotingFor::<Test>::get(idx, alice), None);
        }
    });
}

// -------- Section 8: SignedVoteTally → VoteTally conversion --------

#[test]
fn conversion_computes_ratios_correctly() {
    let tally = SignedVoteTally {
        ayes: 1,
        nays: 2,
        total: 10,
    };
    let vote_tally: VoteTally = tally.into();

    assert_eq!(vote_tally.approval, Perbill::from_rational(1u32, 10u32));
    assert_eq!(vote_tally.rejection, Perbill::from_rational(2u32, 10u32));
    assert_eq!(vote_tally.abstention, Perbill::from_rational(7u32, 10u32));
}

#[test]
fn conversion_ayes_only_saturates_approval() {
    let tally = SignedVoteTally {
        ayes: 3,
        nays: 0,
        total: 3,
    };
    let vote_tally: VoteTally = tally.into();

    assert_eq!(vote_tally.approval, Perbill::one());
    assert_eq!(vote_tally.rejection, Perbill::zero());
    assert_eq!(vote_tally.abstention, Perbill::zero());
}

/// Zero-total tally converts to `VoteTally::default()` — everyone implicitly
/// abstains rather than claiming simultaneous 100% approval/rejection/abstention
/// (which substrate's `Perbill::from_rational(_, 0) = one()` convention would
/// otherwise produce).
#[test]
fn conversion_zero_total_returns_default() {
    let tally = SignedVoteTally {
        ayes: 0,
        nays: 0,
        total: 0,
    };
    let vote_tally: VoteTally = tally.into();

    assert_eq!(vote_tally, VoteTally::default());
    assert_eq!(vote_tally.approval, Perbill::zero());
    assert_eq!(vote_tally.rejection, Perbill::zero());
    assert_eq!(vote_tally.abstention, Perbill::one());
}
