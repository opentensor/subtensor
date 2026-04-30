#![cfg_attr(not(feature = "std"), no_std)]

//! # Referenda
//!
//! Track-based on-chain referenda with two decision strategies.
//!
//! ## Tracks
//!
//! Each referendum is filed against a `Track` defined by the runtime via the
//! [`TracksInfo`] trait. A track carries the proposer set, the voter set, the
//! voting scheme, and the decision strategy. Two strategies are supported:
//!
//! * `PassOrFail`: a binary decision before a deadline. Submitters provide a
//!   call. On approval the call is dispatched (either directly, or handed off
//!   to an `Adjustable` review track via `ApprovalAction::Review`).
//! * `Adjustable`: a timing decision over an already-scheduled call. The call
//!   runs after `initial_delay` by default. Voters can fast-track it sooner,
//!   cancel it entirely, or shift the dispatch time via linear interpolation.
//!
//! ## Lifecycle
//!
//! `submit` records a referendum, schedules the relevant scheduler entries
//! (an alarm for `PassOrFail`; an enactment task plus a reaper alarm for
//! `Adjustable`), and notifies subscribers via
//! [`PollHooks::on_poll_created`].
//!
//! Tally updates arrive through [`Polls::on_tally_updated`]. The hook is
//! intentionally side-effect-light: it stores the new tally and arms an
//! alarm at `now + 1`. All decision logic runs from the alarm via
//! `advance_referendum`, which keeps the tally hook free of re-entrancy.
//!
//! `advance_referendum` is the single state-machine entry point. For an
//! `Ongoing` referendum it dispatches into the appropriate threshold or
//! timing logic; for a referendum already in `Approved` or `FastTracked`
//! it transitions to `Enacted` once the underlying scheduled task has
//! actually run (deferring if it has not).
//!
//! ## State machine
//!
//! `PassOrFail` track:
//!
//! ```text
//!                            submit
//!                              │
//!                              ▼
//!     vote re-arms alarm   ┌───────┐   kill
//!     (now + 1)         ┌─►│Ongoing│───────────────────────────► Killed    (terminal)
//!                       │  └───┬───┘
//!                       │      │
//!                       │      │ alarm fires:
//!                       │      ├─ approve_threshold + Execute  ─► Approved ─► Enacted
//!                       │      ├─ approve_threshold + Review   ─► Delegated (terminal)
//!                       │      ├─ reject_threshold             ─► Rejected  (terminal)
//!                       │      ├─ deadline reached             ─► Expired   (terminal)
//!                       │      └─ no decision, before deadline ─► re-arm at deadline,
//!                       └──────┘                                  stay Ongoing
//! ```
//!
//! `Adjustable` track:
//!
//! ```text
//!                            submit
//!                              │
//!                              │ schedule task at  submitted + initial_delay
//!                              │ schedule reaper at submitted + initial_delay + 1
//!                              ▼
//!     vote re-arms alarm   ┌───────┐   kill
//!     (now + 1)         ┌─►│Ongoing│───────────────────────────► Killed    (terminal)
//!                       │  └───┬───┘
//!                       │      │
//!                       │      │ alarm fires:
//!                       │      ├─ task already ran (lapse)    ─► Enacted     (terminal)
//!                       │      ├─ fast_track_threshold        ─► FastTracked ─► Enacted
//!                       │      ├─ cancel_threshold            ─► Cancelled   (terminal)
//!                       │      └─ otherwise: do_adjust_delay  ─► move task earlier,
//!                       └──────┘                                 restore reaper alarm
//! ```
//!
//! ## Status taxonomy
//!
//! * `Ongoing`: voting in progress.
//! * `Approved`: vote crossed `approve_threshold` on a `PassOrFail` track
//!   with `ApprovalAction::Execute`. Call scheduled on this index;
//!   transitions to `Enacted` once it has dispatched.
//! * `Delegated`: vote crossed `approve_threshold` on a `PassOrFail` track
//!   with `ApprovalAction::Review`. The call now lives on a fresh
//!   referendum on the configured review track; this index is a terminal
//!   audit trail.
//! * `Rejected`: vote crossed `reject_threshold` on a `PassOrFail` track.
//! * `Expired`: `PassOrFail` decision period elapsed without crossing
//!   either threshold.
//! * `FastTracked`: vote crossed `fast_track_threshold` on an `Adjustable`
//!   track. Scheduled task moved to next block; transitions to `Enacted`.
//! * `Cancelled`: vote crossed `cancel_threshold` on an `Adjustable`
//!   track. Scheduled task cancelled.
//! * `Enacted`: the referendum's call has dispatched. Reached either
//!   from `Approved` / `FastTracked` after dispatch, or directly when an
//!   `Adjustable` task ran on its own schedule with no vote-driven
//!   decision (the lapse path).
//! * `Killed`: privileged termination via `KillOrigin`.
//!
//! ## Alarm and task discipline
//!
//! Each referendum has at most one alarm (`alarm_name(index)`) and at
//! most one enactment task (`task_name(index)`). [`set_alarm`] is
//! idempotent: it cancels any prior alarm with the same name before
//! scheduling a new one. `conclude` cancels the alarm so terminal-state
//! referenda do not waste scheduler dispatches. Callers that need a
//! follow-up alarm (the `Approved -> Enacted` and
//! `FastTracked -> Enacted` transitions) call `set_alarm` after
//! `conclude`.
//!
//! `Adjustable` enactment tasks can move earlier (fast-track, linear
//! interpolation) but never later than `submitted + initial_delay`. The
//! reaper alarm is anchored at `submitted + initial_delay + 1` so it
//! always fires after the natural execution time, catching any path that
//! reaches the deadline without a vote-driven decision.
//!
//! ## Runtime configuration check
//!
//! [`Pallet::integrity_test`] runs at startup and asserts that the track
//! table is well-formed: track ids are unique, and every
//! `ApprovalAction::Review { track }` references a track that exists and
//! uses the `Adjustable` strategy. A misconfigured runtime panics at boot
//! with a precise cause.

extern crate alloc;

use alloc::boxed::Box;
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    sp_runtime::{
        Perbill, Saturating,
        traits::{BlockNumberProvider, Dispatchable, One, Zero},
    },
    traits::{
        QueryPreimage, StorePreimage,
        schedule::{DispatchTime, v3::Named as ScheduleNamed},
    },
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

pub use pallet::*;
pub use types::*;

mod types;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet(dev_mode)]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The aggregate runtime call type. Submitted calls and the
        /// pallet's own `advance_referendum` are dispatched through this.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + From<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>
            + From<frame_system::Call<Self>>;

        /// Named scheduler used to queue enactment tasks and alarms. Each
        /// referendum has at most one task and one alarm, identified by
        /// the names produced by [`task_name`] and [`alarm_name`].
        type Scheduler: ScheduleNamed<
                BlockNumberFor<Self>,
                CallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            >;

        /// Preimage provider used to bound submitted calls into a
        /// content-addressed reference and to bound the pallet's own
        /// `advance_referendum` call when scheduling alarms.
        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

        /// Maximum number of simultaneously-active referenda. Submission is
        /// rejected with [`Error::QueueFull`] when this is reached.
        type MaxQueued: Get<u32>;

        /// Origin authorized to terminate an ongoing referendum via `kill`.
        type KillOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Track configuration. Defines the proposer set, voter set, voting
        /// scheme, and decision strategy for each track id.
        type Tracks: TracksInfo<TrackName, Self::AccountId, CallOf<Self>, BlockNumberFor<Self>>;

        /// Source of "now" used for scheduling decisions. Typically
        /// `frame_system::Pallet<T>`; configurable for runtimes that
        /// expose a different block-number authority.
        type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

        /// Lifecycle hooks invoked when a referendum is created or
        /// completed. Notifies any subscriber that needs to react to those
        /// events.
        type PollHooks: PollHooks<ReferendumIndex>;
    }

    /// Monotonic referendum id generator. Incremented by `submit`; never
    /// decremented. Existing referenda continue to be identified by their
    /// assigned id even after the count moves on.
    #[pallet::storage]
    pub type ReferendumCount<T: Config> = StorageValue<_, ReferendumIndex, ValueQuery>;

    /// Number of currently-ongoing referenda. Bounded by [`Config::MaxQueued`]
    /// and used as the capacity check at submit time. Distinct from
    /// [`ReferendumCount`], which only ever grows.
    #[pallet::storage]
    pub type ActiveCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Status of every referendum that has been submitted, keyed by index.
    /// Entries persist after the referendum reaches a terminal state so the
    /// outcome remains queryable for audit.
    #[pallet::storage]
    pub type ReferendumStatusFor<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, ReferendumStatusOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new referendum was submitted.
        Submitted {
            index: ReferendumIndex,
            track: TrackIdOf<T>,
            proposer: T::AccountId,
        },
        /// Approval threshold reached. The call has been scheduled for
        /// dispatch on this referendum's index.
        Approved { index: ReferendumIndex },
        /// Approved with `ApprovalAction::Review`. The call has been handed
        /// off to a fresh referendum at `review` on `track`. No `Submitted`
        /// event is emitted for the child.
        Delegated {
            index: ReferendumIndex,
            review: ReferendumIndex,
            track: TrackIdOf<T>,
        },
        /// Rejection threshold reached.
        Rejected { index: ReferendumIndex },
        /// Cancel threshold reached. The scheduled task has been cancelled.
        Cancelled { index: ReferendumIndex },
        /// Privileged termination via `KillOrigin`.
        Killed { index: ReferendumIndex },
        /// Decision period elapsed without crossing approve or reject
        /// thresholds.
        Expired { index: ReferendumIndex },
        /// Fast-track threshold reached. The scheduled task has been moved
        /// to run next block.
        FastTracked { index: ReferendumIndex },
        /// The referendum's call has been dispatched at block `when`.
        Enacted {
            index: ReferendumIndex,
            when: BlockNumberFor<T>,
        },
        /// A scheduler operation failed for this referendum. Surfaced for
        /// off-chain observability; the pallet does not roll back the
        /// surrounding state change.
        SchedulerOperationFailed { index: ReferendumIndex },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The specified track does not exist.
        BadTrack,
        /// The track has no proposer set configured.
        TrackNotSubmittable,
        /// The caller is not in the track's proposer set.
        NotProposer,
        /// The referendum has already concluded.
        ReferendumFinalized,
        /// The proposal is not authorized for this track.
        ProposalNotAuthorized,
        /// Active-referenda cap (`MaxQueued`) reached.
        QueueFull,
        /// A scheduler operation failed at submit time.
        SchedulerError,
        /// The specified referendum does not exist.
        ReferendumNotFound,
        /// Reached a state combination that should be prevented by submit-time
        /// invariants. Indicates a configuration mismatch (typically a
        /// track's strategy changed under live referenda via runtime upgrade).
        Unreachable,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Validate the runtime track table once at startup. Delegates to
        /// [`TracksInfo::check_integrity`]; a misconfiguration panics with
        /// the trait's diagnostic.
        fn integrity_test() {
            T::Tracks::check_integrity().expect("pallet-referenda: invalid track configuration");
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new referendum on `track` carrying `call`. The proposal
        /// type is derived from the track's strategy: `Action(call)` for
        /// `PassOrFail`, `Review` for `Adjustable` (with the call scheduled
        /// for dispatch after `initial_delay`).
        #[pallet::call_index(0)]
        pub fn submit(
            origin: OriginFor<T>,
            track: TrackIdOf<T>,
            call: Box<CallOf<T>>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let track_info = T::Tracks::info(track).ok_or(Error::<T>::BadTrack)?;

            // All validation runs before any state mutation. The capacity
            // check is bounded on currently-active referenda, not on
            // lifetime submissions.
            let Some(ref proposer_set) = track_info.proposer_set else {
                return Err(Error::<T>::TrackNotSubmittable.into());
            };
            ensure!(proposer_set.contains(&proposer), Error::<T>::NotProposer);
            ensure!(
                T::Tracks::authorize_proposal(&track_info, &call),
                Error::<T>::ProposalNotAuthorized
            );
            let active = ActiveCount::<T>::get();
            ensure!(active < T::MaxQueued::get(), Error::<T>::QueueFull);

            let now = T::BlockNumberProvider::current_block_number();
            let bounded_call = T::Preimages::bound(*call)?;
            let index = ReferendumCount::<T>::get();
            ReferendumCount::<T>::put(index.saturating_add(1));
            ActiveCount::<T>::put(active.saturating_add(1));

            let proposal = match track_info.decision_strategy {
                DecisionStrategy::PassOrFail {
                    decision_period, ..
                } => {
                    // Deadline alarm: fires at the decision period's end to
                    // expire the referendum if no decision has been reached.
                    Self::set_alarm(index, now.saturating_add(decision_period))?;
                    Proposal::Action(bounded_call)
                }
                DecisionStrategy::Adjustable { initial_delay, .. } => {
                    let when = now.saturating_add(initial_delay);
                    Self::schedule_enactment(index, DispatchTime::At(when), bounded_call)?;
                    // Reaper alarm: fires one block after the natural
                    // execution time so that even with no votes, the
                    // referendum reaches a terminal state and releases its
                    // active slot.
                    Self::set_alarm(index, when.saturating_add(One::one()))?;
                    Proposal::Review
                }
            };

            let info = ReferendumInfo {
                track,
                proposal,
                proposer: proposer.clone(),
                submitted: now,
                tally: VoteTally::default(),
            };
            ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));

            T::PollHooks::on_poll_created(index);

            Self::deposit_event(Event::<T>::Submitted {
                index,
                track,
                proposer,
            });

            Ok(())
        }

        /// Privileged termination of an ongoing referendum. Cancels any
        /// pending scheduler entries and concludes as `Killed`.
        #[pallet::call_index(1)]
        pub fn kill(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            T::KillOrigin::ensure_origin(origin)?;

            Self::ensure_ongoing(index)?;

            // Best-effort cleanup. The task entry may be absent (`PassOrFail`
            // has no enactment task before approval); a missing task is
            // expected and not reported.
            let _ = T::Scheduler::cancel_named(task_name(index));
            if let Err(err) = T::Scheduler::cancel_named(alarm_name(index)) {
                Self::report_scheduler_error(index, "cancel_alarm", err);
            }

            let now = T::BlockNumberProvider::current_block_number();
            Self::conclude(
                index,
                ReferendumStatus::Killed(now),
                Event::<T>::Killed { index },
            );
            Ok(())
        }

        /// Drive the state machine for `index`. Invoked by the alarm and
        /// available as a privileged extrinsic for manual recovery.
        #[pallet::call_index(2)]
        pub fn advance_referendum(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            ensure_root(origin)?;

            let now = T::BlockNumberProvider::current_block_number();
            let status =
                ReferendumStatusFor::<T>::get(index).ok_or(Error::<T>::ReferendumNotFound)?;

            match status {
                ReferendumStatus::Ongoing(info) => Self::advance_ongoing(index, info)?,
                ReferendumStatus::Approved(_) | ReferendumStatus::FastTracked(_) => {
                    Self::transition_to_enacted(index, now);
                }
                _ => {
                    // Terminal state: nothing further to do. Reached when an
                    // alarm fires after a manual kill or a delegated handoff.
                }
            };

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Log a scheduler failure and emit `SchedulerOperationFailed` for
    /// off-chain observability. Used in scheduled-call contexts where
    /// `Err` cannot be propagated to a caller.
    fn report_scheduler_error(index: ReferendumIndex, operation: &str, err: DispatchError) {
        log::error!(
            target: "runtime::referenda",
            "Scheduler {} failed for referendum {}: {:?}",
            operation,
            index,
            err,
        );
        Self::deposit_event(Event::<T>::SchedulerOperationFailed { index });
    }

    /// Evaluate the state of an `Ongoing` referendum and dispatch to the
    /// appropriate action helper. Branches on the proposal kind: PassOrFail
    /// runs threshold checks against the deadline; Adjustable also handles
    /// the natural-execution case (task already ran).
    fn advance_ongoing(index: ReferendumIndex, info: ReferendumInfoOf<T>) -> DispatchResult {
        let track_info = T::Tracks::info(info.track).ok_or(Error::<T>::BadTrack)?;
        let tally = info.tally;

        match &info.proposal {
            Proposal::Action(_) => {
                let DecisionStrategy::PassOrFail {
                    decision_period,
                    approve_threshold,
                    reject_threshold,
                    on_approval,
                } = &track_info.decision_strategy
                else {
                    return Err(Error::<T>::Unreachable.into());
                };

                if tally.approval >= *approve_threshold {
                    Self::do_approve(index, &info, on_approval);
                } else if tally.rejection >= *reject_threshold {
                    Self::do_reject(index);
                } else {
                    // No decision yet. Expire only if the deadline has
                    // passed; otherwise restore the deadline alarm so the
                    // expiry will eventually fire if no further votes
                    // arrive.
                    let deadline = info.submitted.saturating_add(*decision_period);
                    let now = T::BlockNumberProvider::current_block_number();
                    if now >= deadline {
                        Self::do_expire(index);
                    } else if let Err(err) = Self::set_alarm(index, deadline) {
                        Self::report_scheduler_error(index, "set_alarm", err);
                    }
                }
            }
            Proposal::Review => {
                let DecisionStrategy::Adjustable {
                    initial_delay,
                    fast_track_threshold,
                    cancel_threshold,
                } = &track_info.decision_strategy
                else {
                    return Err(Error::<T>::Unreachable.into());
                };

                // The task ran on its own schedule with no decisive votes.
                // Lapse directly to `Enacted` rather than running threshold
                // logic (which would falsely conclude as fast-tracked).
                if Self::next_task_dispatch_time(index).is_none() {
                    Self::do_lapse_to_enacted(index);
                    return Ok(());
                }

                // Reaper position reached but the task is still queued —
                // it was postponed by the scheduler under weight pressure.
                // Don't run threshold logic here (with no votes,
                // `do_adjust_delay` would fall through to `do_fast_track`
                // and conclude as `FastTracked` even though no member
                // fast-tracked); re-arm and wait for the task to dispatch.
                let reaper_at = info
                    .submitted
                    .saturating_add(*initial_delay)
                    .saturating_add(One::one());
                let now = T::BlockNumberProvider::current_block_number();
                if now >= reaper_at {
                    if let Err(err) = Self::set_alarm(index, now.saturating_add(One::one())) {
                        Self::report_scheduler_error(index, "set_alarm", err);
                    }
                    return Ok(());
                }

                if tally.approval >= *fast_track_threshold {
                    Self::do_fast_track(index);
                } else if tally.rejection >= *cancel_threshold {
                    Self::do_cancel(index);
                } else {
                    Self::do_adjust_delay(
                        index,
                        &tally,
                        info.submitted,
                        *initial_delay,
                        *fast_track_threshold,
                    );
                }
            }
        }

        Ok(())
    }

    /// Promote an `Approved` or `FastTracked` referendum to `Enacted` once
    /// its scheduled task has run. If the task is still queued (the alarm
    /// fired before the task could be dispatched, typically under block
    /// weight pressure), re-arm the alarm and leave the status unchanged.
    fn transition_to_enacted(index: ReferendumIndex, now: BlockNumberFor<T>) {
        if Self::next_task_dispatch_time(index).is_some() {
            let next = now.saturating_add(One::one());
            if let Err(err) = Self::set_alarm(index, next) {
                Self::report_scheduler_error(index, "set_alarm", err);
            }
            return;
        }

        let when = now.saturating_sub(One::one());
        ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Enacted(when));
        Self::deposit_event(Event::<T>::Enacted { index, when });
    }

    /// Move a referendum to a terminal status: cancel any pending alarm,
    /// store the new status, decrement `ActiveCount`, notify voting pallets,
    /// and emit `event`. Callers that need a follow-up alarm (the
    /// `Approved -> Enacted` and `FastTracked -> Enacted` transitions) must
    /// call `set_alarm` AFTER this function, since `conclude` cancels
    /// whatever alarm is currently scheduled.
    fn conclude(index: ReferendumIndex, status: ReferendumStatusOf<T>, event: Event<T>) {
        if let Err(err) = T::Scheduler::cancel_named(alarm_name(index)) {
            Self::report_scheduler_error(index, "cancel_alarm", err);
        }
        ReferendumStatusFor::<T>::insert(index, status);
        ActiveCount::<T>::mutate(|c| *c = c.saturating_sub(1));
        T::PollHooks::on_poll_completed(index);
        Self::deposit_event(event);
    }

    /// Apply the configured `on_approval` action.
    ///
    /// `Execute` schedules the call on this index for next-block dispatch
    /// and arms a follow-up alarm so the status promotes to `Enacted` once
    /// the task has run.
    ///
    /// `Review` hands the call off to a fresh Adjustable referendum on the
    /// configured track. The parent concludes as `Delegated`. If the review
    /// track is missing or not Adjustable, falls through to `Execute` so the
    /// approved call is not lost.
    fn do_approve(
        index: ReferendumIndex,
        info: &ReferendumInfoOf<T>,
        on_approval: &ApprovalAction<TrackIdOf<T>>,
    ) {
        let Proposal::Action(bounded_call) = &info.proposal else {
            // Reachable only on a configuration mismatch (track strategy
            // changed under live referenda). Bail without action.
            return;
        };

        if let ApprovalAction::Review { track } = on_approval
            && let Some(review) =
                Self::schedule_for_review(bounded_call.clone(), info.proposer.clone(), *track)
        {
            let now = T::BlockNumberProvider::current_block_number();
            Self::conclude(
                index,
                ReferendumStatus::Delegated(now),
                Event::<T>::Delegated {
                    index,
                    review,
                    track: *track,
                },
            );
            return;
        }

        // Execute path (also the Review fallback when the review track is
        // unusable: better to dispatch than to drop the approved call).
        if let Err(err) = Self::schedule_enactment(
            index,
            DispatchTime::After(Zero::zero()),
            bounded_call.clone(),
        ) {
            Self::report_scheduler_error(index, "schedule_enactment", err);
        }
        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::Approved(now),
            Event::<T>::Approved { index },
        );
        // Follow-up alarm fires at `now + 2`: the task is at `now + 1`, so
        // by `now + 2` the scheduler has had a chance to dispatch it. Set
        // after `conclude` because `conclude` cancels any pending alarm.
        let alarm_at = now.saturating_add(One::one()).saturating_add(One::one());
        if let Err(err) = Self::set_alarm(index, alarm_at) {
            Self::report_scheduler_error(index, "set_alarm", err);
        }
    }

    /// Create a fresh Adjustable referendum on `track` carrying the approved
    /// call. The new referendum's slot is claimed against `ActiveCount`; the
    /// caller's `conclude` on the parent releases its slot, so the net change
    /// to `ActiveCount` is zero. No `Submitted` event is emitted (the child
    /// is created by approval, not user submission).
    ///
    /// Returns the new index on success. Returns `None` if the track is
    /// missing or not Adjustable, or if any scheduler operation fails. On
    /// failure no storage is committed so the caller can fall back cleanly.
    fn schedule_for_review(
        bounded_call: BoundedCallOf<T>,
        proposer: T::AccountId,
        track: TrackIdOf<T>,
    ) -> Option<ReferendumIndex> {
        let track_info = T::Tracks::info(track)?;
        let DecisionStrategy::Adjustable { initial_delay, .. } = track_info.decision_strategy
        else {
            return None;
        };

        let now = T::BlockNumberProvider::current_block_number();
        let when = now.saturating_add(initial_delay);
        let new_index = ReferendumCount::<T>::get();

        // Run the failable scheduler operations first. Commit storage only
        // after both succeed so a partial failure cannot leave a child
        // referendum stuck `Ongoing`.
        if let Err(err) = Self::schedule_enactment(new_index, DispatchTime::At(when), bounded_call)
        {
            Self::report_scheduler_error(new_index, "schedule_enactment", err);
            return None;
        }
        if let Err(err) = Self::set_alarm(new_index, when.saturating_add(One::one())) {
            Self::report_scheduler_error(new_index, "set_alarm", err);
            let _ = T::Scheduler::cancel_named(task_name(new_index));
            return None;
        }

        ReferendumCount::<T>::put(new_index.saturating_add(1));
        ActiveCount::<T>::mutate(|c| *c = c.saturating_add(1));

        let new_info = ReferendumInfo {
            track,
            proposal: Proposal::Review,
            proposer,
            submitted: now,
            tally: VoteTally::default(),
        };
        ReferendumStatusFor::<T>::insert(new_index, ReferendumStatus::Ongoing(new_info));

        T::PollHooks::on_poll_created(new_index);

        Some(new_index)
    }

    /// Record `Enacted` directly without an intermediate decided state. Used
    /// when an Adjustable referendum's task ran on its own schedule with no
    /// vote-driven decision. The recorded block is `now - 1`, matching the
    /// reaper alarm's position one block after the natural execution time.
    fn do_lapse_to_enacted(index: ReferendumIndex) {
        let now = T::BlockNumberProvider::current_block_number();
        let when = now.saturating_sub(One::one());
        Self::conclude(
            index,
            ReferendumStatus::Enacted(when),
            Event::<T>::Enacted { index, when },
        );
    }

    /// Conclude as `Rejected`. Reached when rejection crosses
    /// `reject_threshold` on a `PassOrFail` track.
    fn do_reject(index: ReferendumIndex) {
        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::Rejected(now),
            Event::<T>::Rejected { index },
        );
    }

    /// Conclude as `Expired`. Reached when the decision period ends without
    /// crossing approve or reject thresholds.
    fn do_expire(index: ReferendumIndex) {
        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::Expired(now),
            Event::<T>::Expired { index },
        );
    }

    /// Reschedule the task to run next block and arm the follow-up alarm
    /// for the `FastTracked -> Enacted` transition.
    fn do_fast_track(index: ReferendumIndex) {
        if let Err(err) =
            T::Scheduler::reschedule_named(task_name(index), DispatchTime::After(Zero::zero()))
        {
            Self::report_scheduler_error(index, "reschedule_task", err);
        }

        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::FastTracked(now),
            Event::<T>::FastTracked { index },
        );

        // Task at `now + 1`; alarm at `now + 2` catches the post-dispatch
        // state. Set after `conclude` since `conclude` cancels any pending
        // alarm.
        let alarm_at = now.saturating_add(One::one()).saturating_add(One::one());
        if let Err(err) = Self::set_alarm(index, alarm_at) {
            Self::report_scheduler_error(index, "set_alarm", err);
        }
    }

    /// Cancel the scheduled task and conclude as `Cancelled`. Reached when
    /// rejection crosses `cancel_threshold` on an `Adjustable` track. The
    /// scheduler emits its own `Canceled` event for the underlying task.
    fn do_cancel(index: ReferendumIndex) {
        if let Err(err) = T::Scheduler::cancel_named(task_name(index)) {
            Self::report_scheduler_error(index, "cancel_task", err);
        }

        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::Cancelled(now),
            Event::<T>::Cancelled { index },
        );
    }

    /// Move the scheduled task earlier based on the current tally.
    ///
    /// Computes a linear interpolation: at `approval = 0`, the delay equals
    /// `initial_delay`; as approval approaches `fast_track_threshold`, the
    /// delay shrinks toward zero. The dispatch target is anchored at
    /// `submitted` so repeated reschedules cannot drift the call forward.
    /// If elapsed time has already caught up to the interpolated target,
    /// fast-track immediately. Otherwise restores the natural-execution
    /// alarm at `submitted + initial_delay + 1` so the referendum cannot
    /// end up without a pending alarm after voting stops.
    fn do_adjust_delay(
        index: ReferendumIndex,
        tally: &VoteTally,
        submitted: BlockNumberFor<T>,
        initial_delay: BlockNumberFor<T>,
        fast_track_threshold: Perbill,
    ) {
        let gap = fast_track_threshold.saturating_sub(tally.approval);
        let fraction =
            Perbill::from_rational(gap.deconstruct(), fast_track_threshold.deconstruct());
        let computed_delay: BlockNumberFor<T> = fraction.mul_floor(initial_delay);
        let target = submitted.saturating_add(computed_delay);

        let now = T::BlockNumberProvider::current_block_number();
        if target <= now {
            Self::do_fast_track(index);
            return;
        }

        // Skip the scheduler call when the target did not move. The scheduler
        // rejects no-op reschedules with `RescheduleNoChange`.
        if Self::next_task_dispatch_time(index) != Some(target)
            && let Err(err) =
                T::Scheduler::reschedule_named(task_name(index), DispatchTime::At(target))
        {
            Self::report_scheduler_error(index, "reschedule_task", err);
        }

        let natural_alarm = submitted
            .saturating_add(initial_delay)
            .saturating_add(One::one());
        if let Err(err) = Self::set_alarm(index, natural_alarm) {
            Self::report_scheduler_error(index, "set_alarm", err);
        }
    }

    /// Schedule (or replace) the alarm for `index` to fire at `when`.
    /// Cancels any prior alarm with the same name first so callers do not
    /// need to track whether one is currently pending.
    fn set_alarm(index: ReferendumIndex, when: BlockNumberFor<T>) -> Result<(), DispatchError> {
        let _ = T::Scheduler::cancel_named(alarm_name(index));
        let call = T::Preimages::bound(CallOf::<T>::from(Call::advance_referendum { index }))?;
        T::Scheduler::schedule_named(
            alarm_name(index),
            DispatchTime::At(when),
            None,
            0, // highest priority
            frame_system::RawOrigin::Root.into(),
            call,
        )?;
        Ok(())
    }

    /// Schedule the enactment task for `index`. Called once per index in the
    /// referendum lifecycle.
    fn schedule_enactment(
        index: ReferendumIndex,
        desired: DispatchTime<BlockNumberFor<T>>,
        call: BoundedCallOf<T>,
    ) -> DispatchResult {
        T::Scheduler::schedule_named(
            task_name(index),
            desired,
            None,
            0, // highest priority
            frame_system::RawOrigin::Root.into(),
            call,
        )?;
        Ok(())
    }

    /// Return the `Ongoing` info for `index`, or an error if the referendum
    /// is finalized or absent.
    fn ensure_ongoing(index: ReferendumIndex) -> Result<ReferendumInfoOf<T>, DispatchError> {
        match ReferendumStatusFor::<T>::get(index) {
            Some(ReferendumStatus::Ongoing(info)) => Ok(info),
            Some(_) => Err(Error::<T>::ReferendumFinalized.into()),
            None => Err(Error::<T>::ReferendumNotFound.into()),
        }
    }

    /// Next scheduled dispatch time of the enactment task, or `None` if no
    /// task with that name is currently queued.
    fn next_task_dispatch_time(index: ReferendumIndex) -> Option<BlockNumberFor<T>> {
        <T::Scheduler as ScheduleNamed<
            BlockNumberFor<T>,
            CallOf<T>,
            PalletsOriginOf<T>,
        >>::next_dispatch_time(task_name(index))
        .ok()
    }
}

impl<T: Config> Polls<T::AccountId> for Pallet<T> {
    type Index = ReferendumIndex;
    type VotingScheme = VotingSchemeOf<T>;
    type VoterSet = VoterSetOf<T>;

    fn is_ongoing(index: Self::Index) -> bool {
        Self::ensure_ongoing(index).is_ok()
    }

    fn voting_scheme_of(index: Self::Index) -> Option<Self::VotingScheme> {
        Self::ensure_ongoing(index)
            .ok()
            .and_then(|info| T::Tracks::info(info.track).map(|t| t.voting_scheme))
    }

    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet> {
        Self::ensure_ongoing(index)
            .ok()
            .and_then(|info| T::Tracks::info(info.track).map(|t| t.voter_set))
    }

    fn on_tally_updated(index: Self::Index, tally: &VoteTally) {
        let Some(mut info) = Self::ensure_ongoing(index).ok() else {
            return;
        };
        let now = T::BlockNumberProvider::current_block_number();

        info.tally = *tally;
        ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));

        // Defer evaluation by one block. The hook stores the new tally; the
        // alarm fires next block and runs `advance_referendum` from a clean
        // dispatch context, avoiding re-entrancy with the voting pallet.
        if let Err(err) = Self::set_alarm(index, now.saturating_add(One::one())) {
            Self::report_scheduler_error(index, "set_alarm", err);
        }
    }
}
