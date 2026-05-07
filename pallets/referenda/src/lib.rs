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
//! [`OnPollCreated::on_poll_created`].
//!
//! Tally updates arrive through [`Polls::on_tally_updated`]. The hook is
//! intentionally side-effect-light: it stores the new tally and arms an
//! alarm at `now + 1`. All decision logic runs from the alarm via
//! `advance_referendum`, which keeps the tally hook free of re-entrancy.
//!
//! `advance_referendum` is the single state-machine entry point. For an
//! `Ongoing` referendum it dispatches into the appropriate threshold or
//! timing logic; on terminal statuses it is a no-op.
//!
//! ## Dispatch wrapping
//!
//! Approval (Execute) and Adjustable submission both schedule a wrapper
//! call `Pallet::enact(index, call)` rather than the governed call
//! directly. The scheduler invokes the wrapper with `RawOrigin::Root` at
//! the configured time; `enact` dispatches the inner call and marks the
//! referendum `Enacted` in the same call. Dispatch and `Enacted` are
//! atomic; the pallet never has to infer dispatch from scheduler-internal
//! state. `enact` no-ops on terminal-no-dispatch statuses, so a stale
//! wrapper task that fires after a failed scheduler cancel (e.g. inside
//! `kill` or `do_cancel`) cannot dispatch. The submit-time preimage is
//! dropped at scheduling time since the wrapper is the sole reference to
//! the inner call from then on.
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
//!                       │      ├─ approve_threshold + Execute  ─► Approved ─► enact ─► Enacted
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
//!                              │ schedule enact(index) at submitted + initial_delay
//!                              ▼
//!     vote re-arms alarm   ┌───────┐   kill
//!     (now + 1)         ┌─►│Ongoing│───────────────────────────► Killed    (terminal)
//!                       │  └───┬───┘
//!                       │      │
//!                       │      ├─ enact fires (natural)        ─► Enacted     (terminal)
//!                       │      │ alarm fires:
//!                       │      ├─ fast_track_threshold        ─► FastTracked ─► enact ─► Enacted
//!                       │      ├─ cancel_threshold            ─► Cancelled   (terminal)
//!                       │      └─ otherwise: do_adjust_delay  ─► move enact task earlier,
//!                       └──────┘                                 stay Ongoing
//! ```
//!
//! ## Status taxonomy
//!
//! * `Ongoing`: voting in progress.
//! * `Approved`: vote crossed `approve_threshold` on a `PassOrFail` track
//!   with `ApprovalAction::Execute`. The `enact(index)` wrapper is
//!   scheduled on this index and will mark `Enacted` when it dispatches.
//! * `Delegated`: vote crossed `approve_threshold` on a `PassOrFail` track
//!   with `ApprovalAction::Review`. The call now lives on a fresh
//!   referendum on the configured review track; this index is a terminal
//!   audit trail.
//! * `Rejected`: vote crossed `reject_threshold` on a `PassOrFail` track.
//! * `Expired`: `PassOrFail` decision period elapsed without crossing
//!   either threshold.
//! * `FastTracked`: vote crossed `fast_track_threshold` on an `Adjustable`
//!   track. Wrapper rescheduled to next block; marks `Enacted` on dispatch.
//! * `Cancelled`: vote crossed `cancel_threshold` on an `Adjustable`
//!   track. Wrapper cancelled and `PendingDispatch` cleared.
//! * `Enacted`: the dispatch attempt completed. The `Enacted` event
//!   carries the inner call's result via an `Option<DispatchError>`.
//! * `Killed`: privileged termination via `KillOrigin`.
//!
//! ## Alarm and task discipline
//!
//! Each referendum has at most one alarm (`alarm_name(index)`) and at
//! most one enactment task (`task_name(index)`). [`set_alarm`] is
//! idempotent: it cancels any prior alarm with the same name before
//! scheduling a new one. `conclude` cancels the alarm so terminal-state
//! referenda do not waste scheduler dispatches.
//!
//! `Adjustable` enactment tasks can move earlier (fast-track, linear
//! interpolation) but never later than `submitted + initial_delay`.
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
    dispatch::{DispatchResult, GetDispatchInfo},
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
use subtensor_runtime_common::{OnPollCompleted, OnPollCreated, Polls, SetLike, VoteTally};

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod types;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Pinned at 0 to satisfy try-runtime CLI's pre/post-upgrade checks. The
/// project tracks migrations via a per-pallet `HasMigrationRun` map (see
/// `pallet-crowdloan`), so this value is not bumped on schema changes.
pub const STORAGE_VERSION: frame_support::traits::StorageVersion =
    frame_support::traits::StorageVersion::new(0);

#[frame_support::pallet]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The aggregate runtime call type. Submitted calls and the
        /// pallet's own `advance_referendum` are dispatched through this.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo
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

        /// Subscriber notified when a new referendum is created. The hook
        /// returns its actual weight; the pallet pre-charges
        /// `OnPollCreated::weight()` and refunds the unused portion.
        type OnPollCreated: OnPollCreated<ReferendumIndex>;

        /// Subscriber notified when a referendum reaches a terminal status.
        /// Same weight contract as [`OnPollCreated`].
        type OnPollCompleted: OnPollCompleted<ReferendumIndex>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Helper for setting up cross-pallet state needed by benchmarks.
        /// The runtime provides track ids of each strategy variant plus a
        /// proposer guaranteed to be in those tracks' proposer sets.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelper<TrackIdOf<Self>, Self::AccountId, CallOf<Self>>;
    }

    /// Benchmark setup helper. The runtime wires this with track ids and a
    /// proposer that match its track table; the mock provides defaults
    /// matching `pallet-referenda::mock::TestTracks`.
    ///
    /// Note: only a `PassOrFail` track is needed for the approve benchmark
    /// because the `Review` outcome is the worst case and bounds `Execute`
    /// from above (see [`weights::WeightInfo`]).
    #[cfg(feature = "runtime-benchmarks")]
    pub trait BenchmarkHelper<TrackId, AccountId, Call> {
        /// Track id of a `PassOrFail` track. The benchmark drives both the
        /// approve and reject paths through it.
        fn track_passorfail() -> TrackId;
        /// Track id of an `Adjustable` track.
        fn track_adjustable() -> TrackId;
        /// Account in the proposer set of both tracks returned above.
        fn proposer() -> AccountId;
        /// A call that `T::Tracks::authorize_proposal` accepts. Should be
        /// cheap to bound (e.g. `frame_system::remark`).
        fn call() -> Call;
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
        /// Approved on an `Execute` track; the call is scheduled for
        /// dispatch. Review tracks emit `Delegated` or
        /// `ReviewSchedulingFailed` instead.
        Approved { index: ReferendumIndex },
        /// Approved on a `Review` track; the call has been handed off to
        /// the child review referendum at `review`.
        Delegated {
            index: ReferendumIndex,
            review: ReferendumIndex,
            track: TrackIdOf<T>,
        },
        /// Review handoff failed; the parent stays `Ongoing` and retries
        /// on the next vote or expires at the deadline.
        ReviewSchedulingFailed {
            index: ReferendumIndex,
            track: TrackIdOf<T>,
        },
        /// Rejection threshold reached.
        Rejected { index: ReferendumIndex },
        /// Cancel threshold reached; the scheduled call has been cancelled.
        Cancelled { index: ReferendumIndex },
        /// Privileged termination via `KillOrigin`.
        Killed { index: ReferendumIndex },
        /// Decision period elapsed without crossing approve or reject.
        Expired { index: ReferendumIndex },
        /// Fast-track threshold reached; the call now runs next block.
        FastTracked { index: ReferendumIndex },
        /// The dispatch attempt completed at block `when`. `error` is
        /// `None` if the inner call returned `Ok`, otherwise it carries
        /// the failure.
        Enacted {
            index: ReferendumIndex,
            when: BlockNumberFor<T>,
            error: Option<DispatchError>,
        },
        /// A scheduler operation failed; surfaced for observability. The
        /// pallet does not roll back the surrounding state change.
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
        /// The track's voter set is empty at submit time. With no eligible
        /// voters the tally would freeze at zero and the threshold logic
        /// would drive the referendum to a pre-determined outcome (lapse
        /// to enacted on `Adjustable`, expire on `PassOrFail`).
        EmptyVoterSet,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Validate the runtime track table once at startup. Delegates to
        /// [`TracksInfo::check_integrity`]; a misconfiguration panics with
        /// the trait's diagnostic.
        fn integrity_test() {
            T::Tracks::check_integrity().expect("pallet-referenda: invalid track configuration");
        }

        #[cfg(feature = "try-runtime")]
        fn try_state(
            _n: BlockNumberFor<T>,
        ) -> Result<(), frame_support::sp_runtime::TryRuntimeError> {
            Pallet::<T>::do_try_state()
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new referendum on `track` carrying `call`. The proposal
        /// type is derived from the track's strategy: `Action(call)` for
        /// `PassOrFail`, `Review` for `Adjustable` (with the call scheduled
        /// for dispatch after `initial_delay`).
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::submit().saturating_add(T::OnPollCreated::weight())
        )]
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
            // Refuse a poll whose voter set is currently empty. With no
            // eligible voters the threshold checks resolve to a fixed
            // outcome regardless of the call's merits; on `Adjustable`
            // tracks that outcome is enactment at `initial_delay`.
            ensure!(!track_info.voter_set.is_empty(), Error::<T>::EmptyVoterSet);
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

            T::OnPollCreated::on_poll_created(index);

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
        #[pallet::weight(
            T::WeightInfo::kill().saturating_add(T::OnPollCompleted::weight())
        )]
        pub fn kill(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            T::KillOrigin::ensure_origin(origin)?;

            Self::ensure_ongoing(index)?;

            // Best-effort cleanup. The task entry may be absent (`PassOrFail`
            // has no enactment task before approval); a missing task is
            // expected and not reported. If `cancel_named` fails and the
            // wrapper task still fires, `enact` no-ops on the terminal
            // status.
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
        #[pallet::weight(
            // Worst-case bound: the approve-with-`Review` branch fires both hooks.
            T::WeightInfo::advance_referendum()
                .saturating_add(T::OnPollCreated::weight())
                .saturating_add(T::OnPollCompleted::weight())
        )]
        pub fn advance_referendum(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            ensure_root(origin)?;

            let status =
                ReferendumStatusFor::<T>::get(index).ok_or(Error::<T>::ReferendumNotFound)?;

            if let ReferendumStatus::Ongoing(info) = status {
                Self::advance_ongoing(index, info)?;
            }

            Ok(())
        }

        /// Dispatch `call` and mark the referendum `Enacted`. Invoked by
        /// the scheduler with `RawOrigin::Root` at the configured dispatch
        /// time; root may also call this directly to retry a stuck
        /// referendum if the scheduler dropped its task.
        ///
        /// No-op when the referendum is in a terminal-no-dispatch state
        /// (`Cancelled`, `Killed`, `Rejected`, `Expired`, `Delegated`,
        /// `Enacted`), so a stale wrapper task that fires after a failed
        /// scheduler cancel cannot dispatch.
        #[pallet::call_index(3)]
        #[pallet::weight(
            T::WeightInfo::advance_referendum()
                .saturating_add(call.get_dispatch_info().call_weight)
        )]
        pub fn enact(
            origin: OriginFor<T>,
            index: ReferendumIndex,
            call: Box<CallOf<T>>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let Some(status) = ReferendumStatusFor::<T>::get(index) else {
                return Ok(());
            };
            match status {
                ReferendumStatus::Ongoing(_)
                | ReferendumStatus::Approved(_)
                | ReferendumStatus::FastTracked(_) => {}
                _ => return Ok(()),
            }

            let error = call
                .dispatch(frame_system::RawOrigin::Root.into())
                .err()
                .map(|post| post.error);

            let now = T::BlockNumberProvider::current_block_number();
            Self::conclude(
                index,
                ReferendumStatus::Enacted(now),
                Event::<T>::Enacted {
                    index,
                    when: now,
                    error,
                },
            );

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// An empty voter set silently breaks delegation: `schedule_for_review`
    /// would create a review child no one can vote on, and the Adjustable
    /// state machine would lapse it to `Enacted` after `initial_delay`.
    /// Genesis can legitimately observe empty voter sets before the
    /// stake-ranking warmup populates collectives; that is a separate
    /// concern and not enforced here.
    #[cfg(any(feature = "try-runtime", test))]
    pub fn do_try_state() -> Result<(), frame_support::sp_runtime::TryRuntimeError> {
        for track in T::Tracks::tracks() {
            ensure!(
                !track.info.voter_set.is_empty(),
                "pallet-referenda: track has empty voter set"
            );
        }
        Ok(())
    }

    /// Used by `PassOrFail` paths that leave the referendum `Ongoing`
    /// without a vote-driven decision.
    fn expire_or_rearm_deadline(
        index: ReferendumIndex,
        submitted: BlockNumberFor<T>,
        decision_period: BlockNumberFor<T>,
    ) {
        let deadline = submitted.saturating_add(decision_period);
        let now = T::BlockNumberProvider::current_block_number();
        if now >= deadline {
            Self::do_expire(index);
        } else if let Err(err) = Self::set_alarm(index, deadline) {
            Self::report_scheduler_error(index, "set_alarm", err);
        }
    }

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
                    Self::do_approve(index, &info, on_approval, *decision_period);
                } else if tally.rejection >= *reject_threshold {
                    Self::do_reject(index);
                } else {
                    Self::expire_or_rearm_deadline(index, info.submitted, *decision_period);
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

    /// Move a referendum to a terminal status: cancel any pending alarm,
    /// store the new status, decrement `ActiveCount`, notify subscribers
    /// via `OnPollCompleted`, and emit `event`.
    fn conclude(index: ReferendumIndex, status: ReferendumStatusOf<T>, event: Event<T>) {
        if let Err(err) = T::Scheduler::cancel_named(alarm_name(index)) {
            Self::report_scheduler_error(index, "cancel_alarm", err);
        }
        ReferendumStatusFor::<T>::insert(index, status);
        ActiveCount::<T>::mutate(|c| *c = c.saturating_sub(1));
        T::OnPollCompleted::on_poll_completed(index);
        Self::deposit_event(event);
    }

    /// Apply the configured `on_approval` action. Both `Execute` and
    /// `Review` fail closed on scheduler error: the parent stays
    /// `Ongoing` with the deadline alarm re-armed so the approved call
    /// cannot dispatch without going through the configured path.
    fn do_approve(
        index: ReferendumIndex,
        info: &ReferendumInfoOf<T>,
        on_approval: &ApprovalAction<TrackIdOf<T>>,
        decision_period: BlockNumberFor<T>,
    ) {
        let Proposal::Action(bounded_call) = &info.proposal else {
            // Reachable only on a configuration mismatch (track strategy
            // changed under live referenda). Bail without action.
            return;
        };

        // Proposal needs to be delegated to the review track.
        if let ApprovalAction::Review { track } = on_approval {
            let Some(review) =
                Self::schedule_for_review(bounded_call.clone(), info.proposer.clone(), *track)
            else {
                Self::deposit_event(Event::<T>::ReviewSchedulingFailed {
                    index,
                    track: *track,
                });
                Self::expire_or_rearm_deadline(index, info.submitted, decision_period);
                return;
            };

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

        // Normal proposal execution path.
        if let Err(err) = Self::schedule_enactment(
            index,
            DispatchTime::After(Zero::zero()),
            bounded_call.clone(),
        ) {
            Self::report_scheduler_error(index, "schedule_enactment", err);
            Self::expire_or_rearm_deadline(index, info.submitted, decision_period);
            return;
        }

        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::Approved(now),
            Event::<T>::Approved { index },
        );
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
        if track_info.voter_set.is_empty() {
            return None;
        }

        let now = T::BlockNumberProvider::current_block_number();
        let when = now.saturating_add(initial_delay);
        let new_index = ReferendumCount::<T>::get();

        if let Err(err) = Self::schedule_enactment(new_index, DispatchTime::At(when), bounded_call)
        {
            Self::report_scheduler_error(new_index, "schedule_enactment", err);
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

        T::OnPollCreated::on_poll_created(new_index);

        Some(new_index)
    }

    fn do_reject(index: ReferendumIndex) {
        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::Rejected(now),
            Event::<T>::Rejected { index },
        );
    }

    fn do_expire(index: ReferendumIndex) {
        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::Expired(now),
            Event::<T>::Expired { index },
        );
    }

    fn do_fast_track(index: ReferendumIndex) {
        if let Err(err) =
            T::Scheduler::reschedule_named(task_name(index), DispatchTime::After(Zero::zero()))
        {
            Self::report_scheduler_error(index, "reschedule_task", err);
            return;
        }

        let now = T::BlockNumberProvider::current_block_number();
        Self::conclude(
            index,
            ReferendumStatus::FastTracked(now),
            Event::<T>::FastTracked { index },
        );
    }

    /// The scheduler emits its own `Canceled` event for the underlying task.
    /// If `cancel_named` fails and the wrapper still fires, `enact` no-ops
    /// on the `Cancelled` status.
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
    /// Schedule `Pallet::enact(index, call)` to fire at `desired`. The
    /// wrapper carries the inner call and dispatches it on fire, making
    /// the `Ongoing/Approved/FastTracked -> Enacted` transition atomic
    /// with dispatch. The submit-time preimage is dropped here since the
    /// wrapper is now the sole reference to the inner call.
    fn schedule_enactment(
        index: ReferendumIndex,
        desired: DispatchTime<BlockNumberFor<T>>,
        bounded_call: BoundedCallOf<T>,
    ) -> DispatchResult {
        let (inner, _) = T::Preimages::realize(&bounded_call)?;
        let wrapper = T::Preimages::bound(CallOf::<T>::from(Call::enact {
            index,
            call: Box::new(inner),
        }))?;
        T::Scheduler::schedule_named(
            task_name(index),
            desired,
            None,
            0, // highest priority
            frame_system::RawOrigin::Root.into(),
            wrapper,
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

    fn on_tally_updated_weight() -> Weight {
        T::WeightInfo::on_tally_updated()
    }
}
