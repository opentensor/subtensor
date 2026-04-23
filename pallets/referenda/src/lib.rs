#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use codec::Encode;
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    sp_runtime::{
        Perbill, Saturating,
        traits::{BlockNumberProvider, Dispatchable},
    },
    traits::{
        EnsureOriginWithArg, LockIdentifier, QueryPreimage, StorePreimage,
        schedule::{
            DispatchTime,
            v3::{Named as ScheduleNamed, TaskName},
        },
    },
};
use frame_system::{RawOrigin, pallet_prelude::*};
use subtensor_runtime_common::{PollHooks, Polls, VoteTally};

pub use pallet::*;
pub use types::*;

mod types;

pub const REFERENDA_ID: LockIdentifier = *b"referend";

#[frame_support::pallet]
pub mod pallet {
    #![allow(clippy::expect_used)]
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + From<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>
            + From<frame_system::Call<Self>>;

        type Scheduler: ScheduleNamed<
                BlockNumberFor<Self>,
                CallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            >;

        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

        /// Per-track submit authorization. `Success = Option<AccountId>` so the same type
        /// can gate Signed (→ `Some(who)`) and Root (→ `None`) origins, giving the runtime
        /// full flexibility (e.g. signed-in-Proposers on the proposal track, Root-only
        /// for review tracks).
        type SubmitOrigin: EnsureOriginWithArg<
                Self::RuntimeOrigin,
                TrackIdOf<Self>,
                Success = Option<Self::AccountId>,
            >;

        /// Origin allowed to cancel an ongoing referendum.
        type CancelOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type Tracks: TracksInfo<TrackName, Self::AccountId, CallOf<Self>, BlockNumberFor<Self>>;

        type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

        /// Lifecycle hooks for voting pallets.
        type PollHooks: PollHooks<ReferendumIndex>;

        /// Maximum simultaneously `Ongoing` referenda per track. Spam guard.
        #[pallet::constant]
        type MaxQueued: Get<u32>;
    }

    #[pallet::storage]
    pub type ReferendumCount<T: Config> = StorageValue<_, ReferendumIndex, ValueQuery>;

    #[pallet::storage]
    pub type ReferendumStatusFor<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, ReferendumStatusOf<T>, OptionQuery>;

    /// Count of `Ongoing` referenda per track. Incremented on submit, decremented on
    /// finalization. Bounded above by `T::MaxQueued`.
    #[pallet::storage]
    pub type OngoingPerTrack<T: Config> =
        StorageMap<_, Blake2_128Concat, TrackIdOf<T>, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Submitted {
            index: ReferendumIndex,
            track: TrackIdOf<T>,
            /// `None` when initiated by Root (direct submit or pallet-internal via
            /// `ScheduleAndReview`).
            submitter: Option<T::AccountId>,
            proposal: Proposal<BoundedCallOf<T>>,
        },
        Approved {
            index: ReferendumIndex,
        },
        Rejected {
            index: ReferendumIndex,
        },
        Cancelled {
            index: ReferendumIndex,
        },
        Expired {
            index: ReferendumIndex,
        },
        FastTracked {
            index: ReferendumIndex,
        },
        /// Adjustable Review poll whose underlying scheduled task vanished externally.
        Stale {
            index: ReferendumIndex,
        },
        /// Emitted on every vote-driven reschedule (fast_track, cancel, or interpolation).
        TaskRescheduled {
            index: ReferendumIndex,
            at: BlockNumberFor<T>,
        },
        /// Emitted on fast_track/cancel when the task is cancelled in scheduler.
        TaskCancelled {
            index: ReferendumIndex,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The track does not exist.
        UnknownTrack,
        /// The proposal call is not authorized for this track.
        UnauthorizedProposal,
        /// The referendum does not exist.
        UnknownReferendum,
        /// The referendum is not in an Ongoing state.
        NotOngoing,
        /// Preimage storage failure.
        PreimageError,
        /// Scheduler operation failed.
        SchedulerError,
        /// The track already has `MaxQueued` ongoing referenda.
        TrackFull,
        /// `on_approval: ScheduleAndReview` points at a track that is not `Adjustable`.
        InvalidReviewTrack,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new referendum.
        ///
        /// Origin must satisfy `T::SubmitOrigin` for the given track. Success may yield
        /// `Some(AccountId)` (Signed) or `None` (Root), stored for provenance.
        ///
        /// Behavior is determined by the track's `decision_strategy`:
        /// - `PassOrFail`: stores the call bounded; sets a decision-period alarm; does
        ///   **not** schedule any enactment yet (happens on approval).
        /// - `Adjustable`: schedules the call at `now + initial_delay` immediately so the
        ///   collective can adjust timing; stores `Proposal::Review` as a marker.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::zero())] // TODO: benchmarks
        pub fn submit(
            origin: OriginFor<T>,
            track: TrackIdOf<T>,
            call: Box<CallOf<T>>,
        ) -> DispatchResult {
            let who = T::SubmitOrigin::ensure_origin(origin, &track)?;
            Self::do_submit(track, *call, who)
        }

        /// Cancel an ongoing referendum. Also cancels any scheduled enactment task.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::zero())] // TODO: benchmarks
        pub fn cancel(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            T::CancelOrigin::ensure_origin(origin)?;
            Self::do_cancel(index)
        }

        /// Scheduler alarm callback — called with Root origin at the PassOrFail
        /// `decision_period` deadline to mark the referendum as `Expired` (or to apply a
        /// last-minute threshold check).
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::zero())] // TODO: benchmarks
        pub fn nudge_referendum(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            ensure_root(origin)?;
            Self::do_nudge(index)
        }
    }
}

// =============================================================================
// Core logic
// =============================================================================

enum FinalState {
    Approved,
    Rejected,
    Cancelled,
    Expired,
    FastTracked,
    Stale,
}

impl<T: Config> Pallet<T> {
    fn now() -> BlockNumberFor<T> {
        T::BlockNumberProvider::current_block_number()
    }

    fn do_submit(
        track_id: TrackIdOf<T>,
        call: CallOf<T>,
        who: Option<T::AccountId>,
    ) -> DispatchResult {
        let track = T::Tracks::info(track_id).ok_or(Error::<T>::UnknownTrack)?;

        // Spam guard.
        ensure!(
            OngoingPerTrack::<T>::get(track_id) < T::MaxQueued::get(),
            Error::<T>::TrackFull
        );

        // Per-track call authorization.
        ensure!(
            T::Tracks::authorize_proposal(track_id, &call),
            Error::<T>::UnauthorizedProposal
        );

        let bounded_call = T::Preimages::bound(call).map_err(|_| Error::<T>::PreimageError)?;

        let index = ReferendumCount::<T>::get();
        ReferendumCount::<T>::put(index.saturating_add(1));

        let now = Self::now();

        let (proposal, alarm, initial_dispatch_time) = match &track.decision_strategy {
            DecisionStrategy::PassOrFail {
                decision_period, ..
            } => {
                let deadline = now.saturating_add(*decision_period);
                // Best-effort: referendum still ongoing even if alarm fails to schedule.
                let alarm = if Self::schedule_alarm(index, deadline).is_ok() {
                    Some(deadline)
                } else {
                    None
                };
                (Proposal::Action(bounded_call), alarm, None)
            }
            DecisionStrategy::Adjustable { initial_delay, .. } => {
                let when = now.saturating_add(*initial_delay);
                Self::schedule_enactment(index, DispatchTime::At(when), bounded_call)
                    .map_err(|_| Error::<T>::SchedulerError)?;
                (Proposal::Review, None, Some(when))
            }
        };

        let info = ReferendumInfo {
            track: track_id,
            proposal: proposal.clone(),
            submitted: now,
            tally: VoteTally::new(),
            alarm,
            initial_dispatch_time,
        };
        ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));
        OngoingPerTrack::<T>::mutate(track_id, |c| *c = c.saturating_add(1));

        T::PollHooks::on_poll_created(index);

        Self::deposit_event(Event::<T>::Submitted {
            index,
            track: track_id,
            submitter: who,
            proposal,
        });
        Ok(())
    }

    /// Internal review-poll creation invoked by `ScheduleAndReview` on PassOrFail
    /// approval. Bypasses `SubmitOrigin` (pallet itself is the submitter) but still
    /// enforces `MaxQueued`.
    fn create_review_referendum(
        review_track: TrackIdOf<T>,
        bounded_call: BoundedCallOf<T>,
    ) -> DispatchResult {
        let track = T::Tracks::info(review_track).ok_or(Error::<T>::UnknownTrack)?;
        let initial_delay = match &track.decision_strategy {
            DecisionStrategy::Adjustable { initial_delay, .. } => *initial_delay,
            _ => return Err(Error::<T>::InvalidReviewTrack.into()),
        };
        ensure!(
            OngoingPerTrack::<T>::get(review_track) < T::MaxQueued::get(),
            Error::<T>::TrackFull
        );

        let index = ReferendumCount::<T>::get();
        ReferendumCount::<T>::put(index.saturating_add(1));

        let now = Self::now();
        let when = now.saturating_add(initial_delay);
        Self::schedule_enactment(index, DispatchTime::At(when), bounded_call)
            .map_err(|_| Error::<T>::SchedulerError)?;

        let info = ReferendumInfo {
            track: review_track,
            proposal: Proposal::Review,
            submitted: now,
            tally: VoteTally::new(),
            alarm: None,
            initial_dispatch_time: Some(when),
        };
        ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));
        OngoingPerTrack::<T>::mutate(review_track, |c| *c = c.saturating_add(1));

        T::PollHooks::on_poll_created(index);

        Self::deposit_event(Event::<T>::Submitted {
            index,
            track: review_track,
            submitter: None,
            proposal: Proposal::Review,
        });
        Ok(())
    }

    fn do_cancel(index: ReferendumIndex) -> DispatchResult {
        let info = Self::ensure_ongoing(index)?;

        if info.alarm.is_some() {
            let _ = Self::cancel_alarm(index);
        }
        // If a task was scheduled (Adjustable Review), cancel it too.
        if matches!(info.proposal, Proposal::Review) {
            let _ = T::Scheduler::cancel_named(enactment_name(index));
            Self::deposit_event(Event::<T>::TaskCancelled { index });
        }
        Self::finalize(index, FinalState::Cancelled);
        Ok(())
    }

    fn do_nudge(index: ReferendumIndex) -> DispatchResult {
        let info = match Self::ensure_ongoing(index) {
            Ok(i) => i,
            Err(_) => return Ok(()), // already finalized — no-op
        };
        let track = T::Tracks::info(info.track).ok_or(Error::<T>::UnknownTrack)?;

        match &track.decision_strategy {
            DecisionStrategy::PassOrFail {
                approve_threshold,
                reject_threshold,
                on_approval,
                ..
            } => {
                if info.tally.approval >= *approve_threshold {
                    Self::try_approve_action(index, info.clone(), on_approval.clone());
                } else if info.tally.rejection >= *reject_threshold {
                    Self::finalize(index, FinalState::Rejected);
                } else {
                    Self::finalize(index, FinalState::Expired);
                }
            }
            DecisionStrategy::Adjustable { .. } => {
                // Adjustable has no alarm-driven timeout.
            }
        }
        Ok(())
    }

    fn try_approve_action(
        index: ReferendumIndex,
        info: ReferendumInfoOf<T>,
        on_approval: ApprovalAction<TrackIdOf<T>>,
    ) {
        let Proposal::Action(bounded) = info.proposal else {
            // Approved path doesn't apply to Review — those finalize via Adjustable
            // state machine inside on_tally_updated.
            return;
        };
        match on_approval {
            ApprovalAction::Execute => {
                let now = Self::now();
                let when = now.saturating_add(1u32.into());
                let _ = Self::schedule_enactment(index, DispatchTime::At(when), bounded);
            }
            ApprovalAction::ScheduleAndReview { review_track } => {
                // Auto-spawn a Review referendum on the configured review track.
                if Self::create_review_referendum(review_track, bounded).is_err() {
                    // If review creation fails, we still finalize Approved — the outer
                    // decision stands, there's just no oversight window. Better than
                    // leaving the original poll Ongoing forever.
                }
            }
        }
        Self::finalize(index, FinalState::Approved);
    }

    /// Central state transition for all terminal states. Decrements the per-track
    /// counter, overwrites storage, calls voting-pallet cleanup hook, removes stored
    /// info tally from Ongoing variant, and emits the matching event.
    fn finalize(index: ReferendumIndex, state: FinalState) {
        if let Some(ReferendumStatus::Ongoing(info)) = ReferendumStatusFor::<T>::get(index) {
            OngoingPerTrack::<T>::mutate(info.track, |c| *c = c.saturating_sub(1));
        }

        let now = Self::now();
        let status = match state {
            FinalState::Approved => ReferendumStatus::Approved(now),
            FinalState::Rejected => ReferendumStatus::Rejected(now),
            FinalState::Cancelled => ReferendumStatus::Cancelled(now),
            FinalState::Expired => ReferendumStatus::Expired(now),
            FinalState::FastTracked => ReferendumStatus::FastTracked(now),
            FinalState::Stale => ReferendumStatus::Stale(now),
        };
        ReferendumStatusFor::<T>::insert(index, status);
        T::PollHooks::on_poll_completed(index);

        let event = match state {
            FinalState::Approved => Event::<T>::Approved { index },
            FinalState::Rejected => Event::<T>::Rejected { index },
            FinalState::Cancelled => Event::<T>::Cancelled { index },
            FinalState::Expired => Event::<T>::Expired { index },
            FinalState::FastTracked => Event::<T>::FastTracked { index },
            FinalState::Stale => Event::<T>::Stale { index },
        };
        Self::deposit_event(event);
    }

    fn ensure_ongoing(index: ReferendumIndex) -> Result<ReferendumInfoOf<T>, DispatchError> {
        match ReferendumStatusFor::<T>::get(index) {
            Some(ReferendumStatus::Ongoing(info)) => Ok(info),
            Some(_) => Err(Error::<T>::NotOngoing.into()),
            None => Err(Error::<T>::UnknownReferendum.into()),
        }
    }

    /// Schedule a `nudge_referendum(index)` call at `when` with Root origin.
    fn schedule_alarm(index: ReferendumIndex, when: BlockNumberFor<T>) -> Result<(), ()> {
        let call: CallOf<T> = Call::<T>::nudge_referendum { index }.into();
        let bounded = T::Preimages::bound(call).map_err(|_| ())?;
        T::Scheduler::schedule_named(
            alarm_name(index),
            DispatchTime::At(when),
            None,
            0u8,
            RawOrigin::Root.into(),
            bounded,
        )
        .map(|_| ())
        .map_err(|_| ())
    }

    fn cancel_alarm(index: ReferendumIndex) -> Result<(), ()> {
        T::Scheduler::cancel_named(alarm_name(index)).map_err(|_| ())
    }

    /// Place a call into the scheduler as a named enactment task owned by referenda.
    fn schedule_enactment(
        index: ReferendumIndex,
        when: DispatchTime<BlockNumberFor<T>>,
        bounded: BoundedCallOf<T>,
    ) -> Result<(), ()> {
        T::Scheduler::schedule_named(
            enactment_name(index),
            when,
            None,
            0u8,
            RawOrigin::Root.into(),
            bounded,
        )
        .map(|_| ())
        .map_err(|_| ())
    }

    /// Linear interpolation of additional delay for `Adjustable`:
    /// `initial_delay * (1 - approval / fast_track_threshold)`.
    fn interpolate_delay(
        approval: Perbill,
        fast_track_threshold: Perbill,
        initial_delay: BlockNumberFor<T>,
    ) -> BlockNumberFor<T> {
        use sp_runtime::traits::SaturatedConversion;
        let approval_parts = approval.deconstruct() as u64;
        let threshold_parts = (fast_track_threshold.deconstruct() as u64).max(1);
        let clamped = approval_parts.min(threshold_parts);
        let numerator = threshold_parts.saturating_sub(clamped);
        let initial_u64: u64 = initial_delay.saturated_into();
        let additional = initial_u64
            .saturating_mul(numerator)
            .checked_div(threshold_parts)
            .unwrap_or(0);
        additional.saturated_into()
    }
}

impl<T: Config> Polls<T::AccountId> for Pallet<T> {
    type Index = ReferendumIndex;
    type VotingScheme = VotingSchemeOf<T>;
    type VoterSet = VoterSetOf<T>;

    fn is_ongoing(index: Self::Index) -> bool {
        matches!(
            ReferendumStatusFor::<T>::get(index),
            Some(ReferendumStatus::Ongoing(_))
        )
    }

    fn voting_scheme_of(index: Self::Index) -> Option<Self::VotingScheme> {
        let info = Self::ensure_ongoing(index).ok()?;
        T::Tracks::info(info.track).map(|t| t.voting_scheme)
    }

    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet> {
        let info = Self::ensure_ongoing(index).ok()?;
        T::Tracks::info(info.track).map(|t| t.voter_set)
    }

    fn on_tally_updated(index: Self::Index, tally: &VoteTally) {
        let Ok(mut info) = Self::ensure_ongoing(index) else {
            return;
        };
        let Some(track) = T::Tracks::info(info.track) else {
            return;
        };

        // Persist the latest tally first — finalization paths may overwrite this anyway,
        // but for intermediate branches we need it saved.
        info.tally = *tally;

        match &track.decision_strategy {
            DecisionStrategy::PassOrFail {
                approve_threshold,
                reject_threshold,
                on_approval,
                ..
            } => {
                if tally.approval >= *approve_threshold {
                    if info.alarm.is_some() {
                        let _ = Self::cancel_alarm(index);
                    }
                    Self::try_approve_action(index, info, on_approval.clone());
                } else if tally.rejection >= *reject_threshold {
                    if info.alarm.is_some() {
                        let _ = Self::cancel_alarm(index);
                    }
                    Self::finalize(index, FinalState::Rejected);
                } else {
                    ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));
                }
            }
            DecisionStrategy::Adjustable {
                initial_delay,
                fast_track_threshold,
                reject_threshold,
            } => {
                // Stale detection: task must still exist in scheduler.
                if T::Scheduler::next_dispatch_time(enactment_name(index)).is_err() {
                    Self::finalize(index, FinalState::Stale);
                    return;
                }

                if tally.approval >= *fast_track_threshold {
                    let when = Self::now().saturating_add(1u32.into());
                    if T::Scheduler::reschedule_named(enactment_name(index), DispatchTime::At(when))
                        .is_err()
                    {
                        Self::finalize(index, FinalState::Stale);
                        return;
                    }
                    Self::deposit_event(Event::<T>::TaskRescheduled { index, at: when });
                    Self::finalize(index, FinalState::FastTracked);
                } else if tally.rejection >= *reject_threshold {
                    if T::Scheduler::cancel_named(enactment_name(index)).is_err() {
                        Self::finalize(index, FinalState::Stale);
                        return;
                    }
                    Self::deposit_event(Event::<T>::TaskCancelled { index });
                    Self::finalize(index, FinalState::Cancelled);
                } else if let Some(baseline) = info.initial_dispatch_time {
                    // Linear interpolation path.
                    let additional = Self::interpolate_delay(
                        tally.approval,
                        *fast_track_threshold,
                        *initial_delay,
                    );
                    let earliest = Self::now().saturating_add(1u32.into());
                    let target = baseline.saturating_add(additional);
                    let when = if target < earliest { earliest } else { target };
                    if T::Scheduler::reschedule_named(enactment_name(index), DispatchTime::At(when))
                        .is_err()
                    {
                        Self::finalize(index, FinalState::Stale);
                        return;
                    }
                    Self::deposit_event(Event::<T>::TaskRescheduled { index, at: when });
                    ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));
                } else {
                    ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));
                }
            }
        }
    }
}

/// Deterministic 32-byte task name for scheduler alarms (PassOrFail timeout).
fn alarm_name(index: ReferendumIndex) -> TaskName {
    (REFERENDA_ID, "alarm", index).using_encoded(sp_io::hashing::blake2_256)
}

/// Deterministic 32-byte task name for scheduler enactment tasks (Adjustable + Execute).
fn enactment_name(index: ReferendumIndex) -> TaskName {
    (REFERENDA_ID, "enactment", index).using_encoded(sp_io::hashing::blake2_256)
}
