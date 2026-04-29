#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use sp_runtime::DispatchError;
use subtensor_runtime_common::Polls;

const PROPOSER: u128 = 1;
const PROPOSER_B: u128 = 2;
const VOTER_A: u128 = 101;
const VOTER_B: u128 = 102;
const VOTER_C: u128 = 103;

const TRACK_PASS_OR_FAIL: u8 = 0;
const TRACK_ADJUSTABLE: u8 = 1;
const TRACK_DELEGATING: u8 = 2;
const TRACK_NO_PROPOSER_SET: u8 = 3;

const DECISION_PERIOD: u64 = 20;
const INITIAL_DELAY: u64 = 100;

fn make_call() -> RuntimeCall {
    RuntimeCall::System(frame_system::Call::<Test>::remark { remark: vec![] })
}

fn submit_on(track: u8, proposer: U256) -> ReferendumIndex {
    let index = ReferendumCount::<Test>::get();
    assert_ok!(Referenda::submit(
        RuntimeOrigin::signed(proposer),
        track,
        Box::new(make_call()),
    ));
    index
}

fn vote(voter: u128, index: ReferendumIndex, aye: bool) {
    assert_ok!(SignedVoting::vote(
        RuntimeOrigin::signed(U256::from(voter)),
        index,
        aye,
    ));
}

fn status_of(index: ReferendumIndex) -> ReferendumStatusOf<Test> {
    ReferendumStatusFor::<Test>::get(index).expect("referendum should exist")
}

fn current_block() -> u64 {
    System::block_number()
}

fn scheduler_alarm_block(index: ReferendumIndex) -> Option<u64> {
    use frame_support::traits::schedule::v3::Named;
    <Scheduler as Named<u64, RuntimeCall, OriginCaller>>::next_dispatch_time(alarm_name(index)).ok()
}

fn signed_tally_exists(index: ReferendumIndex) -> bool {
    pallet_signed_voting::TallyOf::<Test>::get(index).is_some()
}

fn has_event(matcher: impl Fn(&Event<Test>) -> bool) -> bool {
    referenda_events().iter().any(matcher)
}

/// Assert the standard "concluded and cleaned up" invariants for a terminal
/// referendum: not Ongoing, no tally, no pending alarm, and the slot has
/// been released from `ActiveCount`.
fn assert_concluded(index: ReferendumIndex, expected_active_after: u32) {
    assert!(!Referenda::is_ongoing(index));
    assert!(!signed_tally_exists(index));
    assert_eq!(ActiveCount::<Test>::get(), expected_active_after);
    // Conclude cancels the alarm; only Approved/FastTracked re-arm a new
    // one for the Enacted transition.
    if !matches!(
        ReferendumStatusFor::<Test>::get(index),
        Some(ReferendumStatus::Approved(_)) | Some(ReferendumStatus::FastTracked(_))
    ) {
        assert!(scheduler_alarm_block(index).is_none());
    }
}

/// Drive the referendum forward up to `max_blocks` or until it leaves
/// `Ongoing`.
fn drive_to_terminal(index: ReferendumIndex, max_blocks: u64) {
    let stop = current_block() + max_blocks;
    while current_block() < stop && Referenda::is_ongoing(index) {
        run_to_block(current_block() + 1);
    }
}

#[test]
fn environment_is_initialized() {
    TestState::default().build_and_execute(|| {
        assert!(MemberSet::Single(CollectiveId::Proposers).contains(&U256::from(PROPOSER)));
        assert_eq!(MemberSet::Single(CollectiveId::Triumvirate).len(), 3);
    });
}

#[test]
fn submit_pass_or_fail_records_state_and_schedules_deadline_alarm() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        let now = current_block();

        assert_eq!(ReferendumCount::<Test>::get(), 1);
        assert_eq!(ActiveCount::<Test>::get(), 1);
        assert!(signed_tally_exists(index));
        assert_eq!(scheduler_alarm_block(index), Some(now + DECISION_PERIOD));
        assert!(Pallet::<Test>::next_task_dispatch_time(index).is_none());

        match status_of(index) {
            ReferendumStatus::Ongoing(info) => {
                assert_eq!(info.track, TRACK_PASS_OR_FAIL);
                assert_eq!(info.proposer, U256::from(PROPOSER));
                assert_eq!(info.submitted, now);
                assert!(matches!(info.proposal, Proposal::Action(_)));
            }
            _ => panic!("expected Ongoing"),
        }

        assert!(has_event(|e| matches!(
            e,
            Event::Submitted { index: i, track, proposer }
                if *i == index
                    && *track == TRACK_PASS_OR_FAIL
                    && *proposer == U256::from(PROPOSER)
        )));
    });
}

#[test]
fn submit_adjustable_records_state_and_schedules_task_with_reaper() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let now = current_block();

        assert!(matches!(
            status_of(index),
            ReferendumStatus::Ongoing(ReferendumInfo {
                proposal: Proposal::Review,
                ..
            })
        ));
        assert_eq!(
            Pallet::<Test>::next_task_dispatch_time(index),
            Some(now + INITIAL_DELAY)
        );
        assert_eq!(scheduler_alarm_block(index), Some(now + INITIAL_DELAY + 1));
    });
}

#[test]
fn submit_assigns_monotonic_indices() {
    TestState::default().build_and_execute(|| {
        let i0 = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        let i1 = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let i2 = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER_B));
        assert_eq!((i0, i1, i2), (0, 1, 2));
        assert_eq!(ReferendumCount::<Test>::get(), 3);
        assert_eq!(ActiveCount::<Test>::get(), 3);
    });
}

#[test]
fn submit_rejects_invalid_origins_and_tracks() {
    TestState::default().build_and_execute(|| {
        // Bad track id.
        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(U256::from(PROPOSER)),
                99u8,
                Box::new(make_call()),
            ),
            Error::<Test>::BadTrack
        );
        // Root and unsigned both fail; submit takes a signed origin only.
        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::root(),
                TRACK_PASS_OR_FAIL,
                Box::new(make_call())
            ),
            DispatchError::BadOrigin
        );
        // Caller is not in the proposer set.
        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(U256::from(999)),
                TRACK_PASS_OR_FAIL,
                Box::new(make_call()),
            ),
            Error::<Test>::NotProposer
        );
        // Track has no proposer set.
        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(U256::from(PROPOSER)),
                TRACK_NO_PROPOSER_SET,
                Box::new(make_call()),
            ),
            Error::<Test>::TrackNotSubmittable
        );
    });
}

#[test]
fn submit_rejects_call_when_authorize_proposal_returns_false() {
    TestState::default().build_and_execute(|| {
        set_authorize_proposal(false);
        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(U256::from(PROPOSER)),
                TRACK_PASS_OR_FAIL,
                Box::new(make_call()),
            ),
            Error::<Test>::ProposalNotAuthorized
        );
    });
}

#[test]
fn submit_caps_at_max_queued_and_recycles_after_kill() {
    TestState::default().build_and_execute(|| {
        // Fill exactly to MaxQueued = 10.
        for _ in 0..10 {
            assert_ok!(Referenda::submit(
                RuntimeOrigin::signed(U256::from(PROPOSER)),
                TRACK_PASS_OR_FAIL,
                Box::new(make_call()),
            ));
        }
        assert_eq!(ActiveCount::<Test>::get(), 10);

        // 11th submission rejected.
        assert_noop!(
            Referenda::submit(
                RuntimeOrigin::signed(U256::from(PROPOSER)),
                TRACK_PASS_OR_FAIL,
                Box::new(make_call()),
            ),
            Error::<Test>::QueueFull
        );

        // Killing one frees the slot for reuse.
        assert_ok!(Referenda::kill(RuntimeOrigin::root(), 5));
        assert_ok!(Referenda::submit(
            RuntimeOrigin::signed(U256::from(PROPOSER)),
            TRACK_PASS_OR_FAIL,
            Box::new(make_call()),
        ));
        assert_eq!(ActiveCount::<Test>::get(), 10);
    });
}

#[test]
fn kill_concludes_with_killed_status_and_full_cleanup() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        run_to_block(current_block() + 5);
        let killed_at = current_block();

        assert_ok!(Referenda::kill(RuntimeOrigin::root(), index));

        assert!(matches!(status_of(index), ReferendumStatus::Killed(b) if b == killed_at));
        assert_concluded(index, 0);
        assert!(Pallet::<Test>::next_task_dispatch_time(index).is_none());
        assert!(has_event(
            |e| matches!(e, Event::Killed { index: i } if *i == index)
        ));
    });
}

#[test]
fn kill_rejects_non_kill_origin_and_unknown_index() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        assert_noop!(
            Referenda::kill(RuntimeOrigin::signed(U256::from(PROPOSER)), index),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Referenda::kill(RuntimeOrigin::root(), 999),
            Error::<Test>::ReferendumNotFound
        );
    });
}

#[test]
fn kill_rejects_already_finalized_referendum_for_every_terminal_status() {
    // Drive each conclusion path, then attempt to kill: must always fail
    // with `ReferendumFinalized`.
    TestState::default().build_and_execute(|| {
        // Killed.
        let i = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        assert_ok!(Referenda::kill(RuntimeOrigin::root(), i));
        assert_noop!(
            Referenda::kill(RuntimeOrigin::root(), i),
            Error::<Test>::ReferendumFinalized
        );

        // Approved.
        let i = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        vote(VOTER_A, i, true);
        vote(VOTER_B, i, true);
        run_to_block(current_block() + 2);
        assert!(matches!(status_of(i), ReferendumStatus::Approved(_)));
        assert_noop!(
            Referenda::kill(RuntimeOrigin::root(), i),
            Error::<Test>::ReferendumFinalized
        );

        // Rejected.
        let i = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        vote(VOTER_A, i, false);
        vote(VOTER_B, i, false);
        run_to_block(current_block() + 2);
        assert!(matches!(status_of(i), ReferendumStatus::Rejected(_)));
        assert_noop!(
            Referenda::kill(RuntimeOrigin::root(), i),
            Error::<Test>::ReferendumFinalized
        );

        // Expired.
        let i = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        run_to_block(current_block() + DECISION_PERIOD + 1);
        assert!(matches!(status_of(i), ReferendumStatus::Expired(_)));
        assert_noop!(
            Referenda::kill(RuntimeOrigin::root(), i),
            Error::<Test>::ReferendumFinalized
        );

        // Cancelled.
        let i = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        vote(VOTER_A, i, false);
        vote(VOTER_B, i, false);
        run_to_block(current_block() + 2);
        assert!(matches!(status_of(i), ReferendumStatus::Cancelled(_)));
        assert_noop!(
            Referenda::kill(RuntimeOrigin::root(), i),
            Error::<Test>::ReferendumFinalized
        );
    });
}

#[test]
fn pass_or_fail_below_threshold_stays_ongoing() {
    TestState::default().build_and_execute(|| {
        let aye_only = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        vote(VOTER_A, aye_only, true);
        run_to_block(current_block() + 2);
        assert!(Referenda::is_ongoing(aye_only));

        let nay_only = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        vote(VOTER_A, nay_only, false);
        run_to_block(current_block() + 2);
        assert!(Referenda::is_ongoing(nay_only));
    });
}

#[test]
fn pass_or_fail_approves_at_threshold_and_reaches_enacted() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));

        vote(VOTER_A, index, true);
        vote(VOTER_B, index, true);
        run_to_block(current_block() + 2);

        // Intermediate state: Approved with follow-up alarm.
        assert!(matches!(status_of(index), ReferendumStatus::Approved(_)));
        assert_concluded(index, 0);
        assert!(scheduler_alarm_block(index).is_some());
        assert!(has_event(
            |e| matches!(e, Event::Approved { index: i } if *i == index)
        ));

        // Run forward: Enacted is reached after the task dispatches.
        run_to_block(current_block() + 5);
        assert!(matches!(status_of(index), ReferendumStatus::Enacted(_)));
        assert!(has_event(
            |e| matches!(e, Event::Enacted { index: i, .. } if *i == index)
        ));
    });
}

#[test]
fn pass_or_fail_unanimous_aye_also_approves() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        vote(VOTER_A, index, true);
        vote(VOTER_B, index, true);
        vote(VOTER_C, index, true);
        run_to_block(current_block() + 2);
        assert!(matches!(status_of(index), ReferendumStatus::Approved(_)));
    });
}

#[test]
fn pass_or_fail_rejects_at_threshold_with_full_cleanup() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));

        vote(VOTER_A, index, false);
        vote(VOTER_B, index, false);
        run_to_block(current_block() + 2);

        assert!(matches!(status_of(index), ReferendumStatus::Rejected(_)));
        assert_concluded(index, 0);
        assert!(has_event(
            |e| matches!(e, Event::Rejected { index: i } if *i == index)
        ));
    });
}

#[test]
fn pass_or_fail_expires_at_deadline_with_full_cleanup() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        let submitted = current_block();

        run_to_block(submitted + DECISION_PERIOD - 1);
        assert!(Referenda::is_ongoing(index));

        run_to_block(submitted + DECISION_PERIOD);
        assert!(matches!(status_of(index), ReferendumStatus::Expired(_)));
        assert_concluded(index, 0);
        assert!(has_event(
            |e| matches!(e, Event::Expired { index: i } if *i == index)
        ));
    });
}

#[test]
fn pass_or_fail_non_decisive_vote_does_not_prematurely_expire() {
    // Regression: a single non-decisive vote used to schedule a next-block
    // alarm that then expired the referendum despite the deadline being
    // far away. The fix restores the deadline alarm in the no-decision
    // branch.
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        let submitted = current_block();

        vote(VOTER_A, index, true);
        run_to_block(current_block() + 5);

        assert!(Referenda::is_ongoing(index));
        assert_eq!(
            scheduler_alarm_block(index),
            Some(submitted + DECISION_PERIOD),
            "deadline alarm should be restored"
        );

        // Without further votes, the deadline alarm still fires the expiry.
        run_to_block(submitted + DECISION_PERIOD + 1);
        assert!(matches!(status_of(index), ReferendumStatus::Expired(_)));
    });
}

#[test]
fn pass_or_fail_decisive_vote_at_last_block_of_deadline_approves() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        let submitted = current_block();

        run_to_block(submitted + DECISION_PERIOD - 1);
        vote(VOTER_A, index, true);
        vote(VOTER_B, index, true);
        run_to_block(current_block() + 2);

        assert!(matches!(status_of(index), ReferendumStatus::Approved(_)));
    });
}

#[test]
fn pass_or_fail_vote_change_can_flip_outcome_before_alarm_fires() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));

        vote(VOTER_A, index, true);
        vote(VOTER_B, index, true);
        // Voter B changes mind before the alarm fires; tally drops below
        // approval threshold.
        vote(VOTER_B, index, false);

        run_to_block(current_block() + 2);
        assert!(Referenda::is_ongoing(index));
    });
}

#[test]
fn delegation_creates_child_review_and_keeps_active_count_net_zero() {
    TestState::default().build_and_execute(|| {
        let parent = submit_on(TRACK_DELEGATING, U256::from(PROPOSER));
        assert_eq!(ActiveCount::<Test>::get(), 1);

        vote(VOTER_A, parent, true);
        vote(VOTER_B, parent, true);
        run_to_block(current_block() + 2);

        let child = parent + 1;

        assert!(matches!(status_of(parent), ReferendumStatus::Delegated(_)));
        match status_of(child) {
            ReferendumStatus::Ongoing(info) => {
                assert_eq!(info.track, TRACK_ADJUSTABLE);
                assert!(matches!(info.proposal, Proposal::Review));
                assert_eq!(info.proposer, U256::from(PROPOSER));
            }
            _ => panic!("child should be Ongoing"),
        }

        // ActiveCount: parent -1, child +1, net unchanged.
        assert_eq!(ActiveCount::<Test>::get(), 1);

        let events = referenda_events();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::Delegated { index, review, track }
                if *index == parent && *review == child && *track == TRACK_ADJUSTABLE
        )));
        // No Submitted for the child, no Approved for the parent.
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Submitted { .. }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::Approved { .. }))
                .count(),
            0
        );
    });
}

#[test]
fn delegated_parent_is_terminal_and_child_progresses_independently() {
    TestState::default().build_and_execute(|| {
        let parent = submit_on(TRACK_DELEGATING, U256::from(PROPOSER));
        vote(VOTER_A, parent, true);
        vote(VOTER_B, parent, true);
        run_to_block(current_block() + 2);
        let child = parent + 1;

        // Manual advance does not promote Delegated.
        let snapshot = status_of(parent);
        assert_ok!(Referenda::advance_referendum(RuntimeOrigin::root(), parent));
        assert_eq!(status_of(parent), snapshot);

        // Child reaches Enacted via natural execution. Parent unchanged.
        run_to_block(current_block() + INITIAL_DELAY + 5);
        assert!(matches!(status_of(child), ReferendumStatus::Enacted(_)));
        assert!(matches!(status_of(parent), ReferendumStatus::Delegated(_)));
    });
}

#[test]
fn killing_child_does_not_change_parent_delegated_status() {
    TestState::default().build_and_execute(|| {
        let parent = submit_on(TRACK_DELEGATING, U256::from(PROPOSER));
        vote(VOTER_A, parent, true);
        vote(VOTER_B, parent, true);
        run_to_block(current_block() + 2);
        let child = parent + 1;

        assert_ok!(Referenda::kill(RuntimeOrigin::root(), child));
        assert!(matches!(status_of(parent), ReferendumStatus::Delegated(_)));
        assert!(matches!(status_of(child), ReferendumStatus::Killed(_)));
    });
}

#[test]
fn schedule_for_review_returns_none_for_invalid_targets() {
    TestState::default().build_and_execute(|| {
        let bounded = <Test as Config>::Preimages::bound(make_call()).unwrap();

        // Unknown track id.
        assert!(
            Pallet::<Test>::schedule_for_review(bounded.clone(), U256::from(PROPOSER), 99u8)
                .is_none()
        );

        // PassOrFail track (Review handoff requires Adjustable).
        assert!(
            Pallet::<Test>::schedule_for_review(bounded, U256::from(PROPOSER), TRACK_PASS_OR_FAIL,)
                .is_none()
        );
    });
}

#[test]
fn adjustable_lapses_to_enacted_when_no_decisive_votes() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let submitted = current_block();

        run_to_block(submitted + INITIAL_DELAY + 5);

        assert!(matches!(status_of(index), ReferendumStatus::Enacted(_)));
        assert_concluded(index, 0);

        let events = referenda_events();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Enacted { index: i, .. } if *i == index))
        );
        // Lapse skips the Approved/FastTracked intermediate state.
        for kind in ["Approved", "FastTracked"] {
            let count = events
                .iter()
                .filter(|e| match e {
                    Event::Approved { .. } => kind == "Approved",
                    Event::FastTracked { .. } => kind == "FastTracked",
                    _ => false,
                })
                .count();
            assert_eq!(count, 0, "lapse should not emit {}", kind);
        }
    });
}

#[test]
fn adjustable_fast_tracks_at_threshold_and_reaches_enacted() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));

        vote(VOTER_A, index, true);
        vote(VOTER_B, index, true);
        vote(VOTER_C, index, true);
        run_to_block(current_block() + 5);

        assert!(matches!(status_of(index), ReferendumStatus::Enacted(_)));
        let events = referenda_events();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::FastTracked { index: i } if *i == index))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Enacted { index: i, .. } if *i == index))
        );
    });
}

#[test]
fn adjustable_cancels_at_threshold_and_cleans_up_task() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));

        vote(VOTER_A, index, false);
        vote(VOTER_B, index, false);
        run_to_block(current_block() + 2);

        assert!(matches!(status_of(index), ReferendumStatus::Cancelled(_)));
        assert_concluded(index, 0);
        assert!(Pallet::<Test>::next_task_dispatch_time(index).is_none());
        assert!(has_event(
            |e| matches!(e, Event::Cancelled { index: i } if *i == index)
        ));
    });
}

#[test]
fn adjustable_zero_approval_keeps_full_initial_delay() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let submitted = current_block();
        assert_eq!(
            Pallet::<Test>::next_task_dispatch_time(index),
            Some(submitted + INITIAL_DELAY)
        );
    });
}

#[test]
fn adjustable_partial_approval_pulls_target_earlier() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let submitted = current_block();

        vote(VOTER_A, index, true);
        run_to_block(current_block() + 2);

        let new_target = Pallet::<Test>::next_task_dispatch_time(index).unwrap();
        assert!(new_target < submitted + INITIAL_DELAY);
        assert!(
            new_target >= submitted,
            "target cannot move earlier than submission block"
        );
    });
}

#[test]
fn adjustable_target_is_stable_across_elapsed_blocks() {
    // The interpolation is anchored at `submitted`, so sitting through
    // blocks without new votes does not drift the target forward.
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));

        vote(VOTER_A, index, true);
        run_to_block(current_block() + 2);
        let target_after_vote = Pallet::<Test>::next_task_dispatch_time(index).unwrap();

        run_to_block(current_block() + 10);
        let target_later = Pallet::<Test>::next_task_dispatch_time(index).unwrap();
        assert_eq!(target_after_vote, target_later);
    });
}

#[test]
fn adjustable_late_vote_when_target_is_in_the_past_fast_tracks() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let submitted = current_block();

        // Run forward past where the partial-approval target would land.
        run_to_block(submitted + INITIAL_DELAY / 2 + 10);

        vote(VOTER_A, index, true);
        run_to_block(current_block() + 5);

        assert!(matches!(status_of(index), ReferendumStatus::Enacted(_)));
        assert!(has_event(
            |e| matches!(e, Event::FastTracked { index: i } if *i == index)
        ));
    });
}

#[test]
fn adjustable_reaper_alarm_restored_after_non_decisive_vote() {
    // Regression: a non-decisive vote on an Adjustable referendum used to
    // leave the alarm at `now + 1`. After that alarm fired, no further
    // alarm was scheduled and the referendum could sit Ongoing past the
    // natural execution time. The fix restores the reaper alarm in
    // `do_adjust_delay`.
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let submitted = current_block();

        vote(VOTER_A, index, true);
        run_to_block(current_block() + 3);
        assert!(Referenda::is_ongoing(index));
        assert_eq!(
            scheduler_alarm_block(index),
            Some(submitted + INITIAL_DELAY + 1),
            "reaper alarm must be restored"
        );

        // No further votes; should still reach Enacted.
        run_to_block(submitted + INITIAL_DELAY + 5);
        assert!(matches!(status_of(index), ReferendumStatus::Enacted(_)));
    });
}

fn drive_to_status<F: Fn() -> ReferendumIndex>(
    submit: F,
    drive: impl Fn(ReferendumIndex),
) -> ReferendumIndex {
    let i = submit();
    drive(i);
    i
}

#[test]
fn polls_returns_some_for_ongoing_and_none_for_every_terminal_status() {
    TestState::default().build_and_execute(|| {
        // Ongoing: the trait returns Some.
        let ongoing = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        assert!(Referenda::is_ongoing(ongoing));
        assert_eq!(
            Referenda::voting_scheme_of(ongoing),
            Some(VotingScheme::Signed)
        );
        assert!(Referenda::voter_set_of(ongoing).is_some());

        // Helper closures that drive a fresh referendum to each terminal state.
        let killed = drive_to_status(
            || submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER)),
            |i| {
                assert_ok!(Referenda::kill(RuntimeOrigin::root(), i));
            },
        );

        let approved_or_enacted = drive_to_status(
            || submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER)),
            |i| {
                vote(VOTER_A, i, true);
                vote(VOTER_B, i, true);
                drive_to_terminal(i, 50);
            },
        );

        let rejected = drive_to_status(
            || submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER)),
            |i| {
                vote(VOTER_A, i, false);
                vote(VOTER_B, i, false);
                drive_to_terminal(i, 50);
            },
        );

        let expired = drive_to_status(
            || submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER)),
            |i| {
                run_to_block(current_block() + DECISION_PERIOD + 1);
                let _ = i;
            },
        );

        let cancelled = drive_to_status(
            || submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER)),
            |i| {
                vote(VOTER_A, i, false);
                vote(VOTER_B, i, false);
                drive_to_terminal(i, 50);
            },
        );

        let lapsed = drive_to_status(
            || submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER)),
            |i| {
                run_to_block(current_block() + INITIAL_DELAY + 5);
                let _ = i;
            },
        );

        let delegated = drive_to_status(
            || submit_on(TRACK_DELEGATING, U256::from(PROPOSER)),
            |i| {
                vote(VOTER_A, i, true);
                vote(VOTER_B, i, true);
                run_to_block(current_block() + 2);
            },
        );

        for terminal in [
            killed,
            approved_or_enacted,
            rejected,
            expired,
            cancelled,
            lapsed,
            delegated,
        ] {
            assert!(!Referenda::is_ongoing(terminal));
            assert!(Referenda::voting_scheme_of(terminal).is_none());
            assert!(Referenda::voter_set_of(terminal).is_none());
        }
    });
}

#[test]
fn polls_returns_none_for_unknown_index() {
    TestState::default().build_and_execute(|| {
        assert!(!Referenda::is_ongoing(999));
        assert!(Referenda::voting_scheme_of(999).is_none());
        assert!(Referenda::voter_set_of(999).is_none());
    });
}

#[test]
fn advance_referendum_origin_and_index_validation() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        assert_noop!(
            Referenda::advance_referendum(RuntimeOrigin::signed(U256::from(PROPOSER)), index),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Referenda::advance_referendum(RuntimeOrigin::root(), 999),
            Error::<Test>::ReferendumNotFound
        );
    });
}

#[test]
fn advance_referendum_on_ongoing_runs_the_decision_logic() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        vote(VOTER_A, index, true);
        vote(VOTER_B, index, true);
        // Manual advance instead of waiting for the alarm.
        assert_ok!(Referenda::advance_referendum(RuntimeOrigin::root(), index));
        assert!(matches!(status_of(index), ReferendumStatus::Approved(_)));
    });
}

#[test]
fn advance_referendum_is_a_noop_for_every_terminal_status() {
    TestState::default().build_and_execute(|| {
        // Killed.
        let i = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        assert_ok!(Referenda::kill(RuntimeOrigin::root(), i));
        let snapshot = status_of(i);
        assert_ok!(Referenda::advance_referendum(RuntimeOrigin::root(), i));
        assert_eq!(status_of(i), snapshot);

        // Rejected.
        let i = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        vote(VOTER_A, i, false);
        vote(VOTER_B, i, false);
        run_to_block(current_block() + 2);
        let snapshot = status_of(i);
        assert_ok!(Referenda::advance_referendum(RuntimeOrigin::root(), i));
        assert_eq!(status_of(i), snapshot);

        // Enacted.
        let i = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        run_to_block(current_block() + INITIAL_DELAY + 5);
        let snapshot = status_of(i);
        assert_ok!(Referenda::advance_referendum(RuntimeOrigin::root(), i));
        assert_eq!(status_of(i), snapshot);

        // Delegated.
        let i = submit_on(TRACK_DELEGATING, U256::from(PROPOSER));
        vote(VOTER_A, i, true);
        vote(VOTER_B, i, true);
        run_to_block(current_block() + 2);
        let snapshot = status_of(i);
        assert_ok!(Referenda::advance_referendum(RuntimeOrigin::root(), i));
        assert_eq!(status_of(i), snapshot);
    });
}

#[test]
fn set_alarm_replaces_existing_or_arms_fresh() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        let submitted = current_block();
        assert_eq!(
            scheduler_alarm_block(index),
            Some(submitted + DECISION_PERIOD)
        );

        // Replace.
        assert_ok!(Pallet::<Test>::set_alarm(index, current_block() + 5));
        assert_eq!(scheduler_alarm_block(index), Some(current_block() + 5));

        // Cancel manually, then arm again.
        use frame_support::traits::schedule::v3::Named;
        let _ =
            <Scheduler as Named<u64, RuntimeCall, OriginCaller>>::cancel_named(alarm_name(index));
        assert!(scheduler_alarm_block(index).is_none());

        assert_ok!(Pallet::<Test>::set_alarm(index, current_block() + 10));
        assert_eq!(scheduler_alarm_block(index), Some(current_block() + 10));
    });
}

#[test]
fn parallel_referenda_have_independent_lifecycles() {
    TestState::default().build_and_execute(|| {
        let pf = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        let adj = submit_on(TRACK_ADJUSTABLE, U256::from(PROPOSER));
        let submitted = current_block();
        assert_eq!(ActiveCount::<Test>::get(), 2);

        // Approve pf; adj must keep its scheduling untouched.
        vote(VOTER_A, pf, true);
        vote(VOTER_B, pf, true);
        run_to_block(current_block() + 5);

        assert!(matches!(status_of(pf), ReferendumStatus::Enacted(_)));
        assert!(Referenda::is_ongoing(adj));
        assert_eq!(
            Pallet::<Test>::next_task_dispatch_time(adj),
            Some(submitted + INITIAL_DELAY)
        );
    });
}

#[test]
fn integrity_test_passes_for_valid_track_table() {
    // The mock's track table satisfies both invariants: ids are unique and
    // the only `ApprovalAction::Review { track: 1 }` points at track 1
    // which uses the Adjustable strategy.
    TestState::default().build_and_execute(|| {
        use frame_support::traits::Hooks;
        Pallet::<Test>::integrity_test();
    });
}

#[test]
fn vote_after_termination_does_not_mutate_referenda_state() {
    TestState::default().build_and_execute(|| {
        let index = submit_on(TRACK_PASS_OR_FAIL, U256::from(PROPOSER));
        assert_ok!(Referenda::kill(RuntimeOrigin::root(), index));

        let active_before = ActiveCount::<Test>::get();
        let status_before = status_of(index);
        let _ = SignedVoting::vote(RuntimeOrigin::signed(U256::from(VOTER_A)), index, true);

        assert_eq!(ActiveCount::<Test>::get(), active_before);
        assert_eq!(status_of(index), status_before);
        assert!(scheduler_alarm_block(index).is_none());
    });
}
