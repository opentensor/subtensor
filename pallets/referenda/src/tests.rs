#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use sp_runtime::Perbill;
use subtensor_runtime_common::{Polls, VoteTally};

/// Test that the mock environment is correctly set up with collectives.
#[test]
fn environment_works() {
    TestState::default().build_and_execute(|| {
        // Proposers collective has 2 members
        assert!(MemberSet::Single(CollectiveId::Proposers).contains(&U256::from(1)));
        assert!(MemberSet::Single(CollectiveId::Proposers).contains(&U256::from(2)));
        assert!(!MemberSet::Single(CollectiveId::Proposers).contains(&U256::from(99)));

        // Triumvirate has 3 members
        assert_eq!(MemberSet::Single(CollectiveId::Triumvirate).len(), 3);
        assert!(MemberSet::Single(CollectiveId::Triumvirate).contains(&U256::from(101)));
        assert!(MemberSet::Single(CollectiveId::Triumvirate).contains(&U256::from(102)));
        assert!(MemberSet::Single(CollectiveId::Triumvirate).contains(&U256::from(103)));
    });
}

/// Test: non-proposer cannot submit.
#[test]
fn submit_fails_for_non_proposer() {
    TestState::default().build_and_execute(|| {
        let non_proposer = U256::from(999);
        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        let proposal = Proposal::Action(bounded);

        assert_noop!(
            Referenda::submit(RuntimeOrigin::signed(non_proposer), 0u8, proposal),
            Error::<Test>::NotProposer
        );
    });
}

/// Test: submit on invalid track fails.
#[test]
fn submit_fails_for_bad_track() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        let proposal = Proposal::Action(bounded);

        assert_noop!(
            Referenda::submit(RuntimeOrigin::signed(proposer), 99u8, proposal),
            Error::<Test>::BadTrack
        );
    });
}

/// Full cycle integration test: submit Action, triumvirate votes 2/3 aye, approved.
#[test]
fn full_proposal_cycle_action_approved() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let alice = U256::from(101); // triumvirate member
        let bob = U256::from(102); // triumvirate member

        // 1. Submit an Action proposal on track 0 (triumvirate, PassOrFail)
        let call = RuntimeCall::System(frame_system::Call::<Test>::remark {
            remark: vec![1, 2, 3],
        });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        let proposal = Proposal::Action(bounded);

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            proposal,
        ));

        // Verify referendum was created
        assert_eq!(ReferendumCount::<Test>::get(), 1);
        assert!(Referenda::is_ongoing(0));

        // Verify signed-voting initialized the tally
        assert!(pallet_signed_voting::TallyOf::<Test>::get(0u32).is_some());
        let tally = pallet_signed_voting::TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 0);

        // 2. Alice votes aye
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(alice), 0u32, true,));

        // After 1/3 approval: 33% < 67% threshold, still ongoing
        assert!(Referenda::is_ongoing(0));

        // 3. Bob votes aye
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(bob), 0u32, true,));

        // After 2/3 approval: 67% >= 67% threshold, should be approved
        assert!(!Referenda::is_ongoing(0));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(0),
            Some(ReferendumStatus::Approved(_))
        ));

        // Verify signed-voting cleaned up
        assert!(pallet_signed_voting::TallyOf::<Test>::get(0u32).is_none());

        // 4. Advance blocks to let the scheduled call execute
        run_to_block(5);
    });
}

/// Test: PassOrFail referendum expires when no threshold is reached.
#[test]
fn passorfail_expires_on_timeout() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);

        // Submit a proposal
        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        assert!(Referenda::is_ongoing(0));

        // No one votes. Advance past the decision_period (20 blocks).
        // The scheduler should fire nudge_referendum which marks it as Expired.
        run_to_block(25);

        assert!(matches!(
            ReferendumStatusFor::<Test>::get(0),
            Some(ReferendumStatus::Expired(_))
        ));

        // Verify cleanup
        assert!(pallet_signed_voting::TallyOf::<Test>::get(0u32).is_none());
    });
}

/// Test: cancel a referendum.
#[test]
fn cancel_ongoing_referendum() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);

        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        assert!(Referenda::is_ongoing(0));

        // Cancel requires root (CancelOrigin = EnsureRoot)
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), 0));

        assert!(matches!(
            ReferendumStatusFor::<Test>::get(0),
            Some(ReferendumStatus::Cancelled(_))
        ));

        // Verify cleanup
        assert!(pallet_signed_voting::TallyOf::<Test>::get(0u32).is_none());
    });
}

/// Test: cancel fails for non-root.
#[test]
fn cancel_fails_for_non_root() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);

        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        assert_noop!(
            Referenda::cancel(RuntimeOrigin::signed(U256::from(999)), 0),
            DispatchError::BadOrigin
        );
    });
}

/// Test: PassOrFail rejection when nays reach threshold.
#[test]
fn passorfail_rejected_on_nay_threshold() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let alice = U256::from(101);
        let bob = U256::from(102);

        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        // Alice votes nay
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            false,
        ));

        // 33% rejection, still ongoing
        assert!(Referenda::is_ongoing(0));

        // Bob votes nay
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(bob), 0u32, false,));

        // 67% rejection >= 67% threshold: rejected
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(0),
            Some(ReferendumStatus::Rejected(_))
        ));
    });
}

/// Test: member rotation removes votes.
#[test]
fn member_rotation_removes_votes() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let alice = U256::from(101);

        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        // Alice votes aye
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(alice), 0u32, true,));

        // Verify tally: 1 aye
        let tally = pallet_signed_voting::TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 1);
        assert_eq!(tally.nays, 0);

        // Remove Alice from triumvirate (root origin)
        assert_ok!(pallet_multi_collective::Pallet::<Test>::remove_member(
            RuntimeOrigin::root(),
            CollectiveId::Triumvirate,
            alice,
        ));

        // Alice's vote should be removed via OnMembersChanged -> VoteCleanup
        let tally = pallet_signed_voting::TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 0);

        // Referendum should still be ongoing
        assert!(Referenda::is_ongoing(0));
    });
}

/// Test: vote change during active referendum.
#[test]
fn vote_change_updates_tally() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let alice = U256::from(101);

        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        // Alice votes aye
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(alice), 0u32, true,));
        let tally = pallet_signed_voting::TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 1);
        assert_eq!(tally.nays, 0);

        // Alice changes vote to nay
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(alice),
            0u32,
            false,
        ));
        let tally = pallet_signed_voting::TallyOf::<Test>::get(0u32).unwrap();
        assert_eq!(tally.ayes, 0);
        assert_eq!(tally.nays, 1);
    });
}

/// Helper: pre-schedule a named task (the target of a Review referendum).
fn schedule_named_task(name: [u8; 32], when: u64) {
    let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![9] });
    assert_ok!(pallet_scheduler::Pallet::<Test>::schedule_named(
        RuntimeOrigin::root(),
        name,
        when,
        None,
        128,
        Box::new(call),
    ));
}

fn task_scheduled_at(name: [u8; 32]) -> Option<u64> {
    pallet_scheduler::Lookup::<Test>::get(name).map(|(block, _)| block)
}

/// Test: Submitting a Review proposal that references a task not in the
/// scheduler fails with `ReviewTaskNotFound`, with no state mutation.
#[test]
fn submit_fails_for_review_of_nonexistent_task() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let ghost_task: [u8; 32] = [0u8; 32];

        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(proposer),
                1u8,
                Proposal::Review(ghost_task),
            ),
            Error::<Test>::ReviewTaskNotFound
        );
    });
}

/// Test: Adjustable delay interpolates linearly between `initial_delay` (at
/// approval = 0) and 0 (at approval = fast_track_threshold), anchored at the
/// submission block.
#[test]
fn adjustable_interpolates_delay_anchored_at_submission() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let alice = U256::from(101);
        let task_name: [u8; 32] = *b"review_task_1aaaaaaaaaaaaaaaaaaa";

        System::set_block_number(10);
        schedule_named_task(task_name, 5000);

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            1u8,
            Proposal::Review(task_name),
        ));

        // No votes yet → original schedule untouched.
        assert_eq!(task_scheduled_at(task_name), Some(5000));

        // One aye out of three: approval = 1/3, with fast_track = 75% and
        // initial_delay = 100, delay ≈ ((75% − 33%) / 75%) × 100 ≈ 55-56 blocks.
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(alice), 0u32, true));

        let approval = Perbill::from_rational(1u32, 3u32);
        let fast_track = Perbill::from_percent(75);
        let gap = fast_track.saturating_sub(approval);
        let fraction = Perbill::from_rational(gap.deconstruct(), fast_track.deconstruct());
        let expected_delay: u64 = fraction * 100u64;
        let submitted = 10u64;
        assert_eq!(
            task_scheduled_at(task_name),
            Some(submitted + expected_delay)
        );

        // Sanity: delay is strictly between 0 and initial_delay.
        assert!(expected_delay > 0);
        assert!(expected_delay < 100);
    });
}

/// Test: Delay depends only on approval; nay votes leave the target untouched.
/// Target is anchored at `submitted`, so advancing `now` between votes does
/// not push the dispatch block forward.
#[test]
fn adjustable_target_stable_across_nay_votes_and_time() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let alice = U256::from(101);
        let bob = U256::from(102);
        let task_name: [u8; 32] = *b"review_task_2aaaaaaaaaaaaaaaaaaa";

        System::set_block_number(10);
        schedule_named_task(task_name, 5000);

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            1u8,
            Proposal::Review(task_name),
        ));

        // Alice aye at block 10: approval = 1/3 → target = submitted + delay.
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(alice), 0u32, true));
        let target_after_aye = task_scheduled_at(task_name).expect("rescheduled");
        assert!(target_after_aye > 10);

        // Bob nay at block 30: approval unchanged, rejection = 1/3 (below 51%).
        // Target must be identical — not 30 + delay, since anchor is `submitted`.
        System::set_block_number(30);
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(bob), 0u32, false));
        assert_eq!(task_scheduled_at(task_name), Some(target_after_aye));
        assert!(Referenda::is_ongoing(0));
    });
}

/// Test: When `now` exceeds the interpolated target, the next tally update
/// fast-tracks the task and concludes the referendum as Approved.
#[test]
fn adjustable_fast_tracks_when_elapsed_catches_up() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let alice = U256::from(101);
        let task_name: [u8; 32] = *b"review_task_3aaaaaaaaaaaaaaaaaaa";

        System::set_block_number(10);
        schedule_named_task(task_name, 5000);

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            1u8,
            Proposal::Review(task_name),
        ));

        // Advance past the would-be target (10 + 55 = 65).
        System::set_block_number(200);

        // Alice votes aye. Computed target = 65, but now = 200 → fast-track.
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(alice), 0u32, true));

        assert!(matches!(
            ReferendumStatusFor::<Test>::get(0),
            Some(ReferendumStatus::Approved(_))
        ));

        // do_fast_track reschedules to DispatchTime::After(0), i.e. now + 1.
        assert_eq!(task_scheduled_at(task_name), Some(201));
    });
}

// ============================================================================
// Section 1: submit extrinsic edge cases
// ============================================================================

/// Review proposals are only valid on Adjustable tracks. Submitting one on a
/// PassOrFail track must fail with InvalidConfiguration and leave no state.
#[test]
fn submit_fails_for_review_on_passorfail_track() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let task_name: [u8; 32] = *b"some_taskaaaaaaaaaaaaaaaaaaaaaaa";

        System::set_block_number(10);
        schedule_named_task(task_name, 5000);

        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(proposer),
                0u8, // track 0 is PassOrFail
                Proposal::Review(task_name),
            ),
            Error::<Test>::InvalidConfiguration
        );

        assert_eq!(ReferendumCount::<Test>::get(), 0);
        assert_eq!(ActiveCount::<Test>::get(), 0);
    });
}

/// Action proposals are only valid on PassOrFail tracks. Submitting one on an
/// Adjustable track must fail with InvalidConfiguration and leave no state.
#[test]
fn submit_fails_for_action_on_adjustable_track() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();

        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(proposer),
                1u8, // track 1 is Adjustable
                Proposal::Action(bounded),
            ),
            Error::<Test>::InvalidConfiguration
        );

        assert_eq!(ReferendumCount::<Test>::get(), 0);
        assert_eq!(ActiveCount::<Test>::get(), 0);
    });
}

/// Pinned to surface the dead `authorize_proposal` gap: the trait method is
/// defined on `TracksInfo` but `submit` never invokes it, so rejections from
/// the runtime-side hook are ignored.
#[test]
#[ignore = "known gap: submit does not call TracksInfo::authorize_proposal"]
fn submit_rejects_when_authorize_proposal_returns_false() {
    TestState::default().build_and_execute(|| {
        set_authorize_proposal(false);

        let proposer = U256::from(1);
        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();

        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(proposer),
                0u8,
                Proposal::Action(bounded),
            ),
            Error::<Test>::ProposalNotAuthorized
        );
    });
}

/// A successful submit emits exactly one `Submitted` event with the expected
/// index, track, and proposer.
#[test]
fn submit_emits_submitted_event_with_correct_fields() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        let submitted_events: Vec<_> = referenda_events()
            .into_iter()
            .filter(|e| matches!(e, Event::Submitted { .. }))
            .collect();
        assert_eq!(submitted_events.len(), 1);
        assert_eq!(
            submitted_events[0],
            Event::Submitted {
                index: 0,
                track: 0u8,
                proposer,
            }
        );
    });
}

/// Submit on a PassOrFail track produces an `Ongoing` status with:
/// - the submitter recorded
/// - `submitted` equal to the current block
/// - `scheduled_task = Some((decision_period_end, address))`
#[test]
fn submit_populates_referendum_status_as_ongoing() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        System::set_block_number(42);

        let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
        let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            0u8,
            Proposal::Action(bounded),
        ));

        let status = ReferendumStatusFor::<Test>::get(0).expect("status exists");
        let ReferendumStatus::Ongoing(info) = status else {
            panic!("expected Ongoing status, got {:?}", status);
        };

        assert_eq!(info.track, 0u8);
        assert_eq!(info.submitter, proposer);
        assert_eq!(info.submitted, 42);

        // PassOrFail: decision_period = 20, so scheduled task fires at 42 + 20 = 62.
        let (when, _address) = info.scheduled_task.expect("PassOrFail schedules timeout");
        assert_eq!(when, 62);
    });
}

/// Adjustable tracks have no deadline — submit must not schedule a timeout.
#[test]
fn submit_skips_scheduler_for_adjustable_track() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let task_name: [u8; 32] = *b"review_task_skipaaaaaaaaaaaaaaaa";

        System::set_block_number(10);
        schedule_named_task(task_name, 5000);

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            1u8,
            Proposal::Review(task_name),
        ));

        let ReferendumStatus::Ongoing(info) =
            ReferendumStatusFor::<Test>::get(0).expect("status exists")
        else {
            panic!("expected Ongoing status");
        };

        assert!(
            info.scheduled_task.is_none(),
            "Adjustable submit must not schedule a timeout"
        );
    });
}

/// Concurrent submits on the same block produce monotonically-increasing
/// indexes with no gaps and no recycling. `ActiveCount` reflects the live set.
#[test]
fn submit_assigns_monotonic_ids_across_concurrent_submits() {
    TestState::default().build_and_execute(|| {
        let proposer_a = U256::from(1);
        let proposer_b = U256::from(2);

        let submit_as = |who: U256| {
            let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
            let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
            Referenda::submit(RuntimeOrigin::signed(who), 0u8, Proposal::Action(bounded))
        };

        assert_ok!(submit_as(proposer_a));
        assert_ok!(submit_as(proposer_b));
        assert_ok!(submit_as(proposer_a));

        assert_eq!(ReferendumCount::<Test>::get(), 3);
        assert_eq!(ActiveCount::<Test>::get(), 3);

        for (idx, expected_submitter) in [proposer_a, proposer_b, proposer_a].iter().enumerate() {
            let ReferendumStatus::Ongoing(info) =
                ReferendumStatusFor::<Test>::get(idx as u32).expect("exists")
            else {
                panic!("expected Ongoing for index {}", idx);
            };
            assert_eq!(info.submitter, *expected_submitter);
        }
    });
}

// ============================================================================
// Section 2: cancel extrinsic edge cases
// ============================================================================

/// Cancel on a never-submitted index must fail with `ReferendumNotFound`.
#[test]
fn cancel_nonexistent_returns_referendum_not_found() {
    TestState::default().build_and_execute(|| {
        assert_noop!(
            Referenda::cancel(RuntimeOrigin::root(), 999),
            Error::<Test>::ReferendumNotFound
        );
    });
}

/// Helper: submit a PassOrFail Action proposal on track 0 and return its index.
fn submit_action_on_track_0(proposer: U256) -> ReferendumIndex {
    let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
    let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
    let index = ReferendumCount::<Test>::get();
    assert_ok!(Referenda::submit(
        RuntimeOrigin::signed(proposer),
        0u8,
        Proposal::Action(bounded),
    ));
    index
}

/// Cancelling a referendum already approved (via 2/3 ayes) must fail with
/// `ReferendumFinalized` and leave the stored Approved status untouched.
#[test]
fn cancel_approved_referendum_returns_referendum_finalized() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Approved(_))
        ));

        assert_noop!(
            Referenda::cancel(RuntimeOrigin::root(), index),
            Error::<Test>::ReferendumFinalized
        );
    });
}

/// Cancelling a referendum already rejected (via 2/3 nays) must fail with
/// `ReferendumFinalized`.
#[test]
fn cancel_rejected_referendum_returns_referendum_finalized() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            false
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Rejected(_))
        ));

        assert_noop!(
            Referenda::cancel(RuntimeOrigin::root(), index),
            Error::<Test>::ReferendumFinalized
        );
    });
}

/// Cancelling a referendum that expired on timeout must fail with
/// `ReferendumFinalized`.
#[test]
fn cancel_expired_referendum_returns_referendum_finalized() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));

        // decision_period for track 0 = 20; submitted at block 1, alarm at 21.
        run_to_block(25);
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Expired(_))
        ));

        assert_noop!(
            Referenda::cancel(RuntimeOrigin::root(), index),
            Error::<Test>::ReferendumFinalized
        );
    });
}

/// Cancelling a referendum twice: second call must fail with
/// `ReferendumFinalized`.
#[test]
fn cancel_already_cancelled_returns_referendum_finalized() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));

        assert_noop!(
            Referenda::cancel(RuntimeOrigin::root(), index),
            Error::<Test>::ReferendumFinalized
        );
    });
}

/// A successful cancel emits exactly one `Cancelled` event for the correct index.
#[test]
fn cancel_emits_cancelled_event() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));

        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));

        let cancelled_events: Vec<_> = referenda_events()
            .into_iter()
            .filter(|e| matches!(e, Event::Cancelled { .. }))
            .collect();
        assert_eq!(cancelled_events.len(), 1);
        assert_eq!(cancelled_events[0], Event::Cancelled { index });
    });
}

/// Cancel must remove the finalize-referendum alarm from the scheduler.
/// After cancel, the slot at `submitted + decision_period` holds no live task.
#[test]
fn cancel_removes_scheduled_finalize_task() {
    TestState::default().build_and_execute(|| {
        // Submitted at block 1, decision_period = 20 → alarm at block 21.
        let index = submit_action_on_track_0(U256::from(1));
        let alarm_block = 1u64 + 20u64;

        let live_before = pallet_scheduler::Agenda::<Test>::get(alarm_block)
            .iter()
            .filter(|x| x.is_some())
            .count();
        assert_eq!(live_before, 1, "alarm present before cancel");

        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));

        let live_after = pallet_scheduler::Agenda::<Test>::get(alarm_block)
            .iter()
            .filter(|x| x.is_some())
            .count();
        assert_eq!(live_after, 0, "alarm cleared after cancel");
    });
}

/// Cancelling a Review referendum is a no-op on the scheduler side (no alarm,
/// and the named task it references is intentionally left scheduled — cancel
/// is administrative and does not kill the target task).
#[test]
fn cancel_of_review_referendum_concludes_without_touching_named_task() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let task_name: [u8; 32] = *b"review_task_cancaaaaaaaaaaaaaaaa";

        System::set_block_number(10);
        schedule_named_task(task_name, 5000);

        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            1u8,
            Proposal::Review(task_name),
        ));

        assert_eq!(task_scheduled_at(task_name), Some(5000));

        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), 0));

        assert!(matches!(
            ReferendumStatusFor::<Test>::get(0),
            Some(ReferendumStatus::Cancelled(_))
        ));
        // The named task is unaffected — cancel() does not call cancel_named.
        assert_eq!(task_scheduled_at(task_name), Some(5000));
    });
}

/// Test: MaxQueued bounds active referenda, not total submissions.
/// Finalized referenda (cancelled, rejected, approved, expired) free up capacity.
#[test]
fn max_queued_bounds_active_referenda() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let max = <Test as crate::Config>::MaxQueued::get();

        let submit_one = || {
            let call = RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] });
            let bounded = <Test as crate::Config>::Preimages::bound(call).unwrap();
            Referenda::submit(
                RuntimeOrigin::signed(proposer),
                0u8,
                Proposal::Action(bounded),
            )
        };

        for _ in 0..max {
            assert_ok!(submit_one());
        }
        assert_eq!(ActiveCount::<Test>::get(), max);

        assert_noop!(submit_one(), Error::<Test>::QueueFull);

        // Cancelling a referendum frees one slot.
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), 0));
        assert_eq!(ActiveCount::<Test>::get(), max - 1);

        assert_ok!(submit_one());
        assert_eq!(ActiveCount::<Test>::get(), max);

        // IDs remain monotonic — no recycling.
        assert_eq!(ReferendumCount::<Test>::get(), max + 1);
    });
}

// ============================================================================
// Section 3: finalize_referendum direct tests
// ============================================================================

/// finalize_referendum requires root origin.
#[test]
fn finalize_non_root_fails() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_noop!(
            Referenda::finalize_referendum(RuntimeOrigin::signed(U256::from(1)), index),
            DispatchError::BadOrigin
        );
    });
}

/// finalize_referendum on an index that was never submitted fails with
/// `ReferendumNotFound`.
#[test]
fn finalize_nonexistent_fails() {
    TestState::default().build_and_execute(|| {
        assert_noop!(
            Referenda::finalize_referendum(RuntimeOrigin::root(), 999),
            Error::<Test>::ReferendumNotFound
        );
    });
}

/// finalize_referendum on an already-concluded referendum fails with
/// `ReferendumFinalized`.
#[test]
fn finalize_already_concluded_fails() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));
        assert_noop!(
            Referenda::finalize_referendum(RuntimeOrigin::root(), index),
            Error::<Test>::ReferendumFinalized
        );
    });
}

/// When the cached tally is at/above `approve_threshold`, finalize approves.
/// Tally is injected directly to exercise the branch — normal voting
/// auto-approves before finalize fires.
#[test]
fn finalize_with_approval_threshold_approves() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        ReferendumTallyOf::<Test>::insert(
            index,
            VoteTally {
                approval: Perbill::from_percent(80),
                rejection: Perbill::zero(),
                abstention: Perbill::from_percent(20),
            },
        );

        assert_ok!(Referenda::finalize_referendum(RuntimeOrigin::root(), index));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Approved(_))
        ));
    });
}

/// When the cached tally is at/above `reject_threshold`, finalize rejects.
#[test]
fn finalize_with_rejection_threshold_rejects() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        ReferendumTallyOf::<Test>::insert(
            index,
            VoteTally {
                approval: Perbill::zero(),
                rejection: Perbill::from_percent(80),
                abstention: Perbill::from_percent(20),
            },
        );

        assert_ok!(Referenda::finalize_referendum(RuntimeOrigin::root(), index));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Rejected(_))
        ));
    });
}

/// When neither threshold is reached (default/missing tally), finalize expires.
#[test]
fn finalize_with_neither_threshold_expires() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        // No cached tally → default zeros → neither threshold met.
        assert_ok!(Referenda::finalize_referendum(RuntimeOrigin::root(), index));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Expired(_))
        ));
    });
}

/// Defensive: finalize_referendum invoked on an Adjustable-track referendum
/// (an unreachable path in normal flow — Adjustable doesn't schedule finalize)
/// returns `InvalidConfiguration`.
#[test]
fn finalize_on_adjustable_returns_invalid_configuration() {
    TestState::default().build_and_execute(|| {
        let proposer = U256::from(1);
        let task_name: [u8; 32] = *b"review_adjustaaaaaaaaaaaaaaaaaaa";

        System::set_block_number(10);
        schedule_named_task(task_name, 5000);
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(proposer),
            1u8,
            Proposal::Review(task_name),
        ));

        assert_noop!(
            Referenda::finalize_referendum(RuntimeOrigin::root(), 0),
            Error::<Test>::InvalidConfiguration
        );
    });
}

// ============================================================================
// Section 4: PassOrFail state transitions
// ============================================================================

/// Approval exactly at the threshold approves (>= semantics).
/// Track 0 threshold = 2/3. 2 ayes of 3 triumvirate members = 66.67%.
#[test]
fn approval_at_exact_threshold_approves() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert!(Referenda::is_ongoing(index));

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Approved(_))
        ));
    });
}

/// Rejection exactly at the threshold rejects (>= semantics).
#[test]
fn rejection_at_exact_threshold_rejects() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));
        assert!(Referenda::is_ongoing(index));

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            false
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Rejected(_))
        ));
    });
}

/// On approval, the decision-period timeout alarm is cancelled and an
/// execution task is scheduled for the next block.
#[test]
fn approval_cancels_timeout_alarm_and_schedules_execution() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        let alarm_block = 1u64 + 20u64;

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));

        let alarm_slots = pallet_scheduler::Agenda::<Test>::get(alarm_block)
            .iter()
            .filter(|x| x.is_some())
            .count();
        assert_eq!(alarm_slots, 0, "timeout alarm cancelled on approval");

        // Approved Action is scheduled at DispatchTime::After(0) → next block.
        let exec_slots = pallet_scheduler::Agenda::<Test>::get(2)
            .iter()
            .filter(|x| x.is_some())
            .count();
        assert_eq!(exec_slots, 1, "approved call scheduled for execution");
    });
}

/// On rejection, the decision-period timeout alarm is cancelled.
#[test]
fn rejection_cancels_timeout_alarm() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        let alarm_block = 1u64 + 20u64;

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            false
        ));

        let alarm_slots = pallet_scheduler::Agenda::<Test>::get(alarm_block)
            .iter()
            .filter(|x| x.is_some())
            .count();
        assert_eq!(alarm_slots, 0, "timeout alarm cancelled on rejection");
    });
}

// ============================================================================
// Section 5: Adjustable state transitions
// ============================================================================

/// Helper: schedule `task_name` and submit a Review on track 1.
fn submit_review_on_track_1(proposer: U256, task_name: [u8; 32], when: u64) -> ReferendumIndex {
    schedule_named_task(task_name, when);
    let index = ReferendumCount::<Test>::get();
    assert_ok!(Referenda::submit(
        RuntimeOrigin::signed(proposer),
        1u8,
        Proposal::Review(task_name),
    ));
    index
}

/// Approval at/above the fast_track_threshold fast-tracks the named task to
/// the next block and concludes as Approved.
#[test]
fn adjustable_fast_tracks_above_approval_threshold() {
    TestState::default().build_and_execute(|| {
        let task_name: [u8; 32] = *b"adj_fast_trackaaaaaaaaaaaaaaaaaa";
        System::set_block_number(10);
        let index = submit_review_on_track_1(U256::from(1), task_name, 5000);

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert!(Referenda::is_ongoing(index));

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));
        // 66.67% < 75%, still ongoing.
        assert!(Referenda::is_ongoing(index));

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(103)),
            index,
            true
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Approved(_))
        ));
        // do_fast_track reschedules to After(0) = next block = 11.
        assert_eq!(task_scheduled_at(task_name), Some(11));
    });
}

/// Rejection at/above reject_threshold (51%) cancels the named task and
/// concludes as Rejected.
#[test]
fn adjustable_rejection_cancels_named_task() {
    TestState::default().build_and_execute(|| {
        let task_name: [u8; 32] = *b"adj_rejectaaaaaaaaaaaaaaaaaaaaaa";
        System::set_block_number(10);
        let index = submit_review_on_track_1(U256::from(1), task_name, 5000);

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));
        assert!(Referenda::is_ongoing(index));

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            false
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Rejected(_))
        ));
        assert_eq!(task_scheduled_at(task_name), None);
    });
}

/// With zero approval and 1/3 nay (sub-reject), the interpolated delay
/// equals the full `initial_delay`: target = submitted + initial_delay.
#[test]
fn adjustable_zero_approval_uses_full_initial_delay() {
    TestState::default().build_and_execute(|| {
        let task_name: [u8; 32] = *b"adj_zero_appaaaaaaaaaaaaaaaaaaaa";
        System::set_block_number(10);
        let index = submit_review_on_track_1(U256::from(1), task_name, 5000);

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));

        // submitted(10) + initial_delay(100) = 110.
        assert_eq!(task_scheduled_at(task_name), Some(110));
        assert!(Referenda::is_ongoing(index));
    });
}

/// A tally update that moves the target emits a DelayAdjusted event with the
/// newly-computed dispatch block.
#[test]
fn adjustable_vote_emits_delay_adjusted_event() {
    TestState::default().build_and_execute(|| {
        let task_name: [u8; 32] = *b"adj_event_emitaaaaaaaaaaaaaaaaaa";
        System::set_block_number(10);
        let index = submit_review_on_track_1(U256::from(1), task_name, 5000);

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));

        let new_when = task_scheduled_at(task_name).expect("rescheduled");
        let adjusted_events: Vec<_> = referenda_events()
            .into_iter()
            .filter_map(|e| match e {
                Event::DelayAdjusted {
                    index: i,
                    new_when: w,
                } => Some((i, w)),
                _ => None,
            })
            .collect();
        assert_eq!(adjusted_events, vec![(index, new_when)]);
    });
}

// ============================================================================
// Section 6: Polls trait conformance
// ============================================================================

/// is_ongoing returns false for an index that was never submitted.
#[test]
fn polls_is_ongoing_false_for_nonexistent() {
    TestState::default().build_and_execute(|| {
        assert!(!<Referenda as Polls<U256>>::is_ongoing(999));
    });
}

/// is_ongoing returns false after each finalized state variant.
#[test]
fn polls_is_ongoing_false_for_cancelled() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));
        assert!(!<Referenda as Polls<U256>>::is_ongoing(index));
    });
}

#[test]
fn polls_is_ongoing_false_for_approved() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));
        assert!(!<Referenda as Polls<U256>>::is_ongoing(index));
    });
}

#[test]
fn polls_is_ongoing_false_for_rejected() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            false
        ));
        assert!(!<Referenda as Polls<U256>>::is_ongoing(index));
    });
}

#[test]
fn polls_is_ongoing_false_for_expired() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        run_to_block(25);
        assert!(!<Referenda as Polls<U256>>::is_ongoing(index));
    });
}

/// voting_scheme_of returns Some for an ongoing referendum and None once
/// concluded.
#[test]
fn polls_voting_scheme_of_returns_none_after_conclusion() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert!(<Referenda as Polls<U256>>::voting_scheme_of(index).is_some());
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));
        assert!(<Referenda as Polls<U256>>::voting_scheme_of(index).is_none());
    });
}

/// voter_set_of returns Some for an ongoing referendum and None once concluded.
#[test]
fn polls_voter_set_of_returns_none_after_conclusion() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert!(<Referenda as Polls<U256>>::voter_set_of(index).is_some());
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));
        assert!(<Referenda as Polls<U256>>::voter_set_of(index).is_none());
    });
}

/// on_tally_updated caches the pushed tally in `ReferendumTallyOf` so that
/// `finalize_referendum` can evaluate it at timeout.
#[test]
fn polls_on_tally_updated_caches_tally() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        let tally = VoteTally {
            approval: Perbill::from_percent(10),
            rejection: Perbill::from_percent(20),
            abstention: Perbill::from_percent(70),
        };
        <Referenda as Polls<U256>>::on_tally_updated(index, &tally);
        assert_eq!(ReferendumTallyOf::<Test>::get(index), Some(tally));
    });
}

/// on_tally_updated on a concluded referendum must not change its status
/// and must not emit a new transition event.
#[test]
fn polls_on_tally_updated_noop_when_concluded() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));

        let events_before = referenda_events().len();
        let tally = VoteTally {
            approval: Perbill::from_percent(99),
            rejection: Perbill::zero(),
            abstention: Perbill::from_percent(1),
        };
        <Referenda as Polls<U256>>::on_tally_updated(index, &tally);

        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Cancelled(_))
        ));
        assert_eq!(referenda_events().len(), events_before);
    });
}

// ============================================================================
// Section 7: PollHooks lifecycle contract
// ============================================================================
//
// The hook is wired to SignedVoting; we observe the hook firing through
// SignedVoting's internal `TallyOf` storage: present after on_poll_created,
// absent after on_poll_completed.

#[test]
fn pollhooks_on_poll_created_initializes_signed_voting_tally() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert!(pallet_signed_voting::TallyOf::<Test>::get(index).is_some());
    });
}

#[test]
fn pollhooks_on_poll_completed_clears_tally_on_approve() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));
        assert!(pallet_signed_voting::TallyOf::<Test>::get(index).is_none());
    });
}

#[test]
fn pollhooks_on_poll_completed_clears_tally_on_reject() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            false
        ));
        assert!(pallet_signed_voting::TallyOf::<Test>::get(index).is_none());
    });
}

#[test]
fn pollhooks_on_poll_completed_clears_tally_on_cancel() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));
        assert!(pallet_signed_voting::TallyOf::<Test>::get(index).is_none());
    });
}

#[test]
fn pollhooks_on_poll_completed_clears_tally_on_expire() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        run_to_block(25);
        assert!(pallet_signed_voting::TallyOf::<Test>::get(index).is_none());
    });
}

// ============================================================================
// Section 8: Storage invariants
// ============================================================================

#[test]
fn active_count_decrements_on_approve() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_eq!(ActiveCount::<Test>::get(), 1);
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));
        assert_eq!(ActiveCount::<Test>::get(), 0);
    });
}

#[test]
fn active_count_decrements_on_reject() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_eq!(ActiveCount::<Test>::get(), 1);
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            false
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            false
        ));
        assert_eq!(ActiveCount::<Test>::get(), 0);
    });
}

#[test]
fn active_count_decrements_on_expire() {
    TestState::default().build_and_execute(|| {
        submit_action_on_track_0(U256::from(1));
        assert_eq!(ActiveCount::<Test>::get(), 1);
        run_to_block(25);
        assert_eq!(ActiveCount::<Test>::get(), 0);
    });
}

/// Finalized entries are NOT removed from `ReferendumStatusFor`; the pallet
/// keeps them as history across every conclusion path.
#[test]
fn referendum_status_preserved_post_conclusion() {
    TestState::default().build_and_execute(|| {
        // Approved
        let i1 = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            i1,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            i1,
            true
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(i1),
            Some(ReferendumStatus::Approved(_))
        ));

        // Rejected
        let i2 = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            i2,
            false
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            i2,
            false
        ));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(i2),
            Some(ReferendumStatus::Rejected(_))
        ));

        // Cancelled
        let i3 = submit_action_on_track_0(U256::from(1));
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), i3));
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(i3),
            Some(ReferendumStatus::Cancelled(_))
        ));

        // Expired
        let i4 = submit_action_on_track_0(U256::from(1));
        run_to_block(50);
        assert!(matches!(
            ReferendumStatusFor::<Test>::get(i4),
            Some(ReferendumStatus::Expired(_))
        ));
    });
}

/// `ReferendumTallyOf` is cleared on each conclusion path.
#[test]
fn referendum_tally_cleared_on_approve() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));
        assert!(ReferendumTallyOf::<Test>::get(index).is_none());
    });
}

#[test]
fn referendum_tally_cleared_on_cancel() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));
        assert!(ReferendumTallyOf::<Test>::get(index).is_none());
    });
}

#[test]
fn referendum_tally_cleared_on_expire() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        run_to_block(25);
        assert!(ReferendumTallyOf::<Test>::get(index).is_none());
    });
}

// ============================================================================
// Section 9: Scheduler error handling
// ============================================================================
//
// Scheduler-side errors in the post-submit flow (cancel, approve, reject,
// fast-track, adjust-delay) must not unwind the caller — they are logged and
// surfaced via `SchedulerOperationFailed`. We force the error by clearing
// the Agenda slot holding the referendum's alarm, so `Scheduler::cancel`
// returns NotFound on the next attempt.

fn clear_agenda_slot(block: u64) {
    pallet_scheduler::Agenda::<Test>::mutate(block, |agenda| {
        for slot in agenda.iter_mut() {
            *slot = None;
        }
    });
}

/// Cancel still concludes the referendum when the scheduler cancel of the
/// alarm fails; a `SchedulerOperationFailed` event is emitted.
#[test]
fn cancel_with_failed_scheduler_emits_operation_failed_event() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        clear_agenda_slot(1u64 + 20u64);

        assert_ok!(Referenda::cancel(RuntimeOrigin::root(), index));

        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Cancelled(_))
        ));

        let failed: Vec<_> = referenda_events()
            .into_iter()
            .filter(|e| matches!(e, Event::SchedulerOperationFailed { .. }))
            .collect();
        assert_eq!(failed, vec![Event::SchedulerOperationFailed { index }]);
    });
}

/// Approval still concludes the referendum and emits Approved, even when
/// do_approve's attempt to cancel the alarm fails. A SchedulerOperationFailed
/// is additionally emitted.
#[test]
fn approve_with_failed_alarm_cancel_still_concludes() {
    TestState::default().build_and_execute(|| {
        let index = submit_action_on_track_0(U256::from(1));
        clear_agenda_slot(1u64 + 20u64);

        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(101)),
            index,
            true
        ));
        assert_ok!(SignedVoting::vote(
            RuntimeOrigin::signed(U256::from(102)),
            index,
            true
        ));

        assert!(matches!(
            ReferendumStatusFor::<Test>::get(index),
            Some(ReferendumStatus::Approved(_))
        ));

        let failed_count = referenda_events()
            .into_iter()
            .filter(|e| matches!(e, Event::SchedulerOperationFailed { index: i } if *i == index))
            .count();
        assert!(
            failed_count >= 1,
            "expected at least one SchedulerOperationFailed"
        );
    });
}
