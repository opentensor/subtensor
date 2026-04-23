#![cfg(test)]
#![allow(clippy::unwrap_used)]

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use sp_runtime::Perbill;

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
        assert_ok!(SignedVoting::vote(RuntimeOrigin::signed(alice), 0u32, false,));

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
        assert_eq!(task_scheduled_at(task_name), Some(submitted + expected_delay));

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
            Referenda::submit(RuntimeOrigin::signed(proposer), 0u8, Proposal::Action(bounded))
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
