#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    sp_runtime::{
        Perbill,
        traits::{BlockNumberProvider, Dispatchable, Hash as HashT},
    },
    traits::{
        Bounded, OriginTrait, QueryPreimage, StorePreimage,
        schedule::{
            DispatchTime,
            v3::{Anon as ScheduleAnon, Named as ScheduleNamed, TaskName},
        },
    },
};
use frame_system::{RawOrigin, ensure_signed_or_root, pallet_prelude::*};
use scale_info::TypeInfo;
use sp_runtime::Saturating;
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

pub use pallet::*;

pub const MAX_TRACK_NAME_LEN: usize = 32;
pub type TrackName = [u8; MAX_TRACK_NAME_LEN];

pub type ReferendumIndex = u32;

pub type PalletsOriginOf<T> =
    <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type CallOf<T> = <T as Config>::RuntimeCall;
pub type BoundedCallOf<T> = Bounded<CallOf<T>, <T as frame_system::Config>::Hashing>;

pub type TracksOf<T> = <T as Config>::Tracks;
pub type TrackIdOf<T> =
    <TracksOf<T> as TracksInfo<TrackName, AccountIdOf<T>, CallOf<T>, BlockNumberFor<T>>>::Id;
pub type VotingSchemeOf<T> = <TracksOf<T> as TracksInfo<
    TrackName,
    AccountIdOf<T>,
    CallOf<T>,
    BlockNumberFor<T>,
>>::VotingScheme;
pub type VoterSetOf<T> =
    <TracksOf<T> as TracksInfo<TrackName, AccountIdOf<T>, CallOf<T>, BlockNumberFor<T>>>::VoterSet;
pub type ProposerSetOf<T> = <TracksOf<T> as TracksInfo<
    TrackName,
    AccountIdOf<T>,
    CallOf<T>,
    BlockNumberFor<T>,
>>::ProposerSet;

pub type ScheduleAddressOf<T> = <<T as Config>::Scheduler as ScheduleAnon<
    BlockNumberFor<T>,
    CallOf<T>,
    PalletsOriginOf<T>,
>>::Address;

pub type ReferendumStatusOf<T> =
    ReferendumStatus<AccountIdOf<T>, TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>>;

pub type ReferendumInfoOf<T> =
    ReferendumInfo<AccountIdOf<T>, TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>>;

#[frame_support::pallet]
pub mod pallet {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
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

        type Scheduler: ScheduleAnon<
                BlockNumberFor<Self>,
                CallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            > + ScheduleNamed<
                BlockNumberFor<Self>,
                CallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            >;

        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

        /// Origin allowed to cancel a referendum.
        type CancelOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type Tracks: TracksInfo<TrackName, Self::AccountId, CallOf<Self>, BlockNumberFor<Self>>;

        type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

        /// Lifecycle hooks for voting pallets.
        type PollHooks: PollHooks<ReferendumIndex>;

        /// Maximum simultaneously `Ongoing` referenda per track. New submissions beyond this
        /// fail with `TrackFull`. Spam guard; historical (finalized) referenda do not count.
        #[pallet::constant]
        type MaxQueued: Get<u32>;
    }

    #[pallet::storage]
    pub type ReferendumCount<T: Config> = StorageValue<_, ReferendumIndex, ValueQuery>;

    #[pallet::storage]
    pub type ReferendumStatusFor<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, ReferendumStatusOf<T>, OptionQuery>;

    #[pallet::storage]
    pub type ReferendumTally<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, VoteTally, OptionQuery>;

    /// Count of `Ongoing` referenda per track. Incremented on `submit`, decremented in
    /// `finalize`. Bounded above by `T::MaxQueued`.
    #[pallet::storage]
    pub type OngoingPerTrack<T: Config> =
        StorageMap<_, Blake2_128Concat, TrackIdOf<T>, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Submitted {
            index: ReferendumIndex,
            track: TrackIdOf<T>,
            /// `None` when initiated by Root (e.g. an approved governance batch).
            submitter: Option<T::AccountId>,
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
        TaskRescheduled {
            index: ReferendumIndex,
            at: BlockNumberFor<T>,
        },
        TaskCancelled {
            index: ReferendumIndex,
        },
        /// The Review referendum's underlying task is no longer in the scheduler.
        Stale {
            index: ReferendumIndex,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The track does not exist.
        UnknownTrack,
        /// The proposer is not in the track's proposer set.
        NotAllowedProposer,
        /// The proposal is not authorized for this track.
        UnauthorizedProposal,
        /// The referendum does not exist.
        UnknownReferendum,
        /// The referendum is not in an Ongoing state.
        NotOngoing,
        /// The proposal kind does not match the track's decision strategy.
        IncompatibleProposalKind,
        /// Preimage storage failure.
        PreimageError,
        /// Scheduler operation failed.
        SchedulerError,
        /// The task referenced by a `Review` proposal is not scheduled.
        TaskNotScheduled,
        /// The track already has `MaxQueued` ongoing referenda.
        TrackFull,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new referendum.
        ///
        /// Accepts Signed or Root origin. For Signed origins the account must be in the
        /// track's `proposer_set`. Root origin bypasses the proposer check — this is the path
        /// taken when a `batch_all` dispatched after a PassOrFail approval submits a follow-up
        /// `Review` referendum on another track.
        ///
        /// For `Action(call)` proposals, the call is bounded via preimage storage.
        /// For `Review(task_name)` proposals, the task must already exist in the scheduler.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::zero())] // TODO: add benchmarks
        pub fn submit(
            origin: OriginFor<T>,
            track: TrackIdOf<T>,
            proposal: Proposal<Box<CallOf<T>>>,
        ) -> DispatchResult {
            let who = ensure_signed_or_root(origin)?;
            Self::do_submit(who, track, proposal)
        }

        /// Cancel an ongoing referendum. Origin must satisfy `CancelOrigin`.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::zero())] // TODO: add benchmarks
        pub fn cancel(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            T::CancelOrigin::ensure_origin(origin)?;
            Self::do_cancel(index)
        }

        /// Re-evaluate an ongoing referendum. Called by the scheduler at `decision_period`
        /// expiry for `PassOrFail` tracks. Origin must be Root.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::zero())] // TODO: add benchmarks
        pub fn nudge_referendum(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            ensure_root(origin)?;
            Self::do_nudge(index)
        }
    }
}

/// Proposal payload — generic over the call representation.
///
/// - As an **extrinsic argument** (`submit`): `Proposal<Box<CallOf<T>>>` — the caller passes
///   an inline, unbounded call.
/// - As **stored on-chain**: `Proposal<BoundedCallOf<T>>` — the call has been preimage-bounded
///   into a hash+length, giving a fixed-size encoding.
///
/// Conversion from the input form to the stored form happens in `do_submit`.
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum Proposal<C> {
    Action(C),
    Review(TaskName),
}

#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct ReferendumInfo<AccountId, TrackId, BoundedCall, BlockNumber> {
    pub track: TrackId,
    pub proposal: Proposal<BoundedCall>,
    /// `None` means the referendum was initiated by Root (e.g. an approved governance batch).
    pub submitter: Option<AccountId>,
    pub submitted: BlockNumber,
    /// For `PassOrFail`: the deadline at which the scheduler alarm will fire.
    /// For `Adjustable`: unused.
    pub alarm: Option<BlockNumber>,
    /// For `Adjustable` `Review` referenda: the originally-scheduled dispatch block of the
    /// referenced task, captured at submit time. Used as the baseline for linear delay
    /// interpolation so the baseline doesn't drift as the task is rescheduled by votes.
    pub initial_dispatch_time: Option<BlockNumber>,
}

impl<A, T, B, N> MaxEncodedLen for ReferendumInfo<A, T, B, N>
where
    A: MaxEncodedLen,
    T: MaxEncodedLen,
    B: MaxEncodedLen,
    N: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        // submitter: Option<A> = 1 + A
        1_usize
            .saturating_add(A::max_encoded_len())
            .saturating_add(T::max_encoded_len())
            .saturating_add(Proposal::<B>::max_encoded_len())
            .saturating_add(N::max_encoded_len()) // submitted
            // alarm: Option<N> = 1 + N
            .saturating_add(1_usize.saturating_add(N::max_encoded_len()))
            // initial_dispatch_time: Option<N> = 1 + N
            .saturating_add(1_usize.saturating_add(N::max_encoded_len()))
    }
}

#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub enum ReferendumStatus<AccountId, TrackId, BoundedCall, BlockNumber> {
    Ongoing(ReferendumInfo<AccountId, TrackId, BoundedCall, BlockNumber>),
    Approved(BlockNumber),
    Rejected(BlockNumber),
    Cancelled(BlockNumber),
    Expired(BlockNumber),
    /// `Adjustable` `Review` referendum whose underlying scheduler task no longer exists
    /// (executed externally or cancelled outside governance). Vote outcome is moot.
    Stale(BlockNumber),
}

impl<A, T, B, N> MaxEncodedLen for ReferendumStatus<A, T, B, N>
where
    A: MaxEncodedLen,
    T: MaxEncodedLen,
    B: MaxEncodedLen,
    N: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        1_usize.saturating_add(
            ReferendumInfo::<A, T, B, N>::max_encoded_len().max(N::max_encoded_len()),
        )
    }
}

#[derive(Clone)]
pub struct TrackInfo<Name, Moment, ProposerSet, VoterSet, VotingScheme> {
    pub name: Name,
    pub proposer_set: ProposerSet,
    pub voting_scheme: VotingScheme,
    pub voter_set: VoterSet,
    pub decision_strategy: DecisionStrategy<Moment>,
}

#[derive(Clone)]
pub struct Track<Id, Name, Moment, ProposerSet, VoterSet, VotingScheme> {
    pub id: Id,
    pub info: TrackInfo<Name, Moment, ProposerSet, VoterSet, VotingScheme>,
}

pub trait TracksInfo<Name, AccountId, Call, Moment> {
    type Id: Parameter + MaxEncodedLen + Copy + Ord + PartialOrd + Send + Sync + 'static;

    type ProposerSet: SetLike<AccountId>;
    type VotingScheme: PartialEq + Clone;
    type VoterSet: SetLike<AccountId>;

    fn tracks() -> impl Iterator<
        Item = Track<Self::Id, Name, Moment, Self::ProposerSet, Self::VoterSet, Self::VotingScheme>,
    >;

    fn track_ids() -> impl Iterator<Item = Self::Id> {
        Self::tracks().map(|t| t.id)
    }

    fn info(
        id: Self::Id,
    ) -> Option<TrackInfo<Name, Moment, Self::ProposerSet, Self::VoterSet, Self::VotingScheme>>
    {
        Self::tracks().find(|t| t.id == id).map(|t| t.info)
    }

    /// Default allows all proposals.
    fn authorize_proposal(_id: Self::Id, _proposal: &Call) -> bool {
        true
    }
}

/// The decision strategy for a track.
#[derive(Clone)]
pub enum DecisionStrategy<Moment> {
    /// Binary decision: the referendum passes or fails.
    ///
    /// Voters have until `decision_period` to reach a threshold.
    /// If `approve_threshold` is reached, the call is scheduled for execution.
    /// If `reject_threshold` is reached, the referendum is cancelled.
    /// If neither threshold is reached before the deadline, the referendum expires.
    PassOrFail {
        /// How long voters have to reach a decision.
        decision_period: Moment,
        /// Minimum approval (ayes / total eligible) to execute the call.
        approve_threshold: Perbill,
        /// Minimum rejection (nays / total eligible) to cancel the referendum.
        reject_threshold: Perbill,
    },
    /// Timing adjustment: the referendum controls when an already-scheduled task executes.
    ///
    /// The task is scheduled externally (e.g., via a batch call). Votes shift its
    /// execution time: strong approval brings it forward, strong rejection cancels it.
    /// There is no deadline — the referendum lives until the task executes or is cancelled.
    Adjustable {
        /// Maximum additional delay applied at 0% approval. Delay shrinks linearly to 0 as
        /// approval approaches `fast_track_threshold`. Added on top of the originally
        /// scheduled dispatch time captured when the `Review` referendum was submitted.
        initial_delay: Moment,
        /// Approval above this threshold reschedules the task to execute immediately.
        fast_track_threshold: Perbill,
        /// Rejection above this threshold cancels the scheduled task entirely.
        reject_threshold: Perbill,
    },
}

impl<T: Config> Pallet<T> {
    fn now() -> BlockNumberFor<T> {
        T::BlockNumberProvider::current_block_number()
    }

    fn do_submit(
        who: Option<T::AccountId>,
        track_id: TrackIdOf<T>,
        proposal: Proposal<Box<CallOf<T>>>,
    ) -> DispatchResult {
        let track = T::Tracks::info(track_id).ok_or(Error::<T>::UnknownTrack)?;
        // Root bypasses the proposer-set check.
        if let Some(ref account) = who {
            ensure!(
                track.proposer_set.contains(account),
                Error::<T>::NotAllowedProposer
            );
        }
        // Spam guard: per-track cap on concurrently Ongoing referenda.
        ensure!(
            OngoingPerTrack::<T>::get(track_id) < T::MaxQueued::get(),
            Error::<T>::TrackFull
        );

        // Validate proposal kind against decision strategy and, for Review, snapshot the
        // originally-scheduled dispatch block so the interpolation baseline is stable.
        let mut initial_dispatch_time: Option<BlockNumberFor<T>> = None;
        match (&proposal, &track.decision_strategy) {
            (Proposal::Action(call), DecisionStrategy::PassOrFail { .. }) => {
                ensure!(
                    T::Tracks::authorize_proposal(track_id, call),
                    Error::<T>::UnauthorizedProposal
                );
            }
            (Proposal::Review(task_name), DecisionStrategy::Adjustable { .. }) => {
                let when = <<T as Config>::Scheduler as ScheduleNamed<
                    BlockNumberFor<T>,
                    CallOf<T>,
                    PalletsOriginOf<T>,
                >>::next_dispatch_time(*task_name)
                .map_err(|_| Error::<T>::TaskNotScheduled)?;
                initial_dispatch_time = Some(when);
            }
            _ => return Err(Error::<T>::IncompatibleProposalKind.into()),
        }

        // Convert user input into stored form.
        let stored_proposal: Proposal<BoundedCallOf<T>> = match proposal {
            Proposal::Action(call) => {
                let bounded = T::Preimages::bound(*call).map_err(|_| Error::<T>::PreimageError)?;
                Proposal::Action(bounded)
            }
            Proposal::Review(name) => Proposal::Review(name),
        };

        let index = ReferendumCount::<T>::get();
        ReferendumCount::<T>::put(index.saturating_add(1));

        let now = Self::now();

        // Set alarm for PassOrFail timeout.
        let alarm = if let DecisionStrategy::PassOrFail {
            decision_period, ..
        } = track.decision_strategy
        {
            let when = now.saturating_add(decision_period);
            if let Err(()) = Self::schedule_alarm(index, when) {
                // Scheduling failed — skip alarm; referendum still ongoing but with no timeout.
                // Callers can still use cancel() to end it.
                None
            } else {
                Some(when)
            }
        } else {
            None
        };

        let info = ReferendumInfo {
            track: track_id,
            proposal: stored_proposal,
            submitter: who.clone(),
            submitted: now,
            alarm,
            initial_dispatch_time,
        };
        // `submitter` carried as Option: None indicates system/root-initiated.
        ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));
        ReferendumTally::<T>::insert(
            index,
            VoteTally {
                approval: Perbill::zero(),
                rejection: Perbill::zero(),
                abstention: Perbill::zero(),
            },
        );
        OngoingPerTrack::<T>::mutate(track_id, |c| *c = c.saturating_add(1));

        T::PollHooks::on_poll_created(index);

        Self::deposit_event(Event::<T>::Submitted {
            index,
            track: track_id,
            submitter: who,
        });
        Ok(())
    }

    fn do_cancel(index: ReferendumIndex) -> DispatchResult {
        let status = ReferendumStatusFor::<T>::get(index).ok_or(Error::<T>::UnknownReferendum)?;
        let ongoing = match status {
            ReferendumStatus::Ongoing(info) => info,
            _ => return Err(Error::<T>::NotOngoing.into()),
        };

        // Cancel any alarm.
        if ongoing.alarm.is_some() {
            let _ = Self::cancel_alarm(index);
        }

        // If it's an Adjustable Review proposal, cancel the referenced task.
        if let Proposal::Review(task_name) = &ongoing.proposal {
            let _ = T::Scheduler::cancel_named(*task_name);
            Self::deposit_event(Event::<T>::TaskCancelled { index });
        }

        Self::finalize(index, FinalState::Cancelled);
        Ok(())
    }

    fn do_nudge(index: ReferendumIndex) -> DispatchResult {
        let status = ReferendumStatusFor::<T>::get(index).ok_or(Error::<T>::UnknownReferendum)?;
        let info = match status {
            ReferendumStatus::Ongoing(i) => i,
            _ => return Ok(()), // already finalized
        };
        let tally = ReferendumTally::<T>::get(index).unwrap_or(VoteTally {
            approval: Perbill::zero(),
            rejection: Perbill::zero(),
            abstention: Perbill::zero(),
        });
        let track = T::Tracks::info(info.track).ok_or(Error::<T>::UnknownTrack)?;

        match &track.decision_strategy {
            DecisionStrategy::PassOrFail {
                approve_threshold,
                reject_threshold,
                ..
            } => {
                if tally.approval >= *approve_threshold {
                    Self::try_approve(index, &info);
                } else if tally.rejection >= *reject_threshold {
                    Self::finalize(index, FinalState::Rejected);
                } else {
                    Self::finalize(index, FinalState::Expired);
                }
            }
            DecisionStrategy::Adjustable { .. } => {
                // Adjustable has no timeout, ignore.
            }
        }
        Ok(())
    }

    fn try_approve(index: ReferendumIndex, info: &ReferendumInfoOf<T>) {
        match &info.proposal {
            Proposal::Action(bounded) => {
                // Best-effort schedule at next block; any failure leaves as Approved with no task.
                let _ = T::Scheduler::schedule(
                    DispatchTime::After(Zero::zero()),
                    None,
                    0u8,
                    RawOrigin::Root.into(),
                    bounded.clone(),
                );
            }
            Proposal::Review(_) => {
                // Approval of Adjustable means fast-track was triggered; already handled.
            }
        }
        Self::finalize(index, FinalState::Approved);
    }

    fn finalize(index: ReferendumIndex, state: FinalState) {
        // Decrement per-track counter before overwriting the status.
        if let Some(ReferendumStatus::Ongoing(info)) = ReferendumStatusFor::<T>::get(index) {
            OngoingPerTrack::<T>::mutate(info.track, |c| *c = c.saturating_sub(1));
        }

        let now = Self::now();
        let status = match state {
            FinalState::Approved => ReferendumStatus::Approved(now),
            FinalState::Rejected => ReferendumStatus::Rejected(now),
            FinalState::Cancelled => ReferendumStatus::Cancelled(now),
            FinalState::Expired => ReferendumStatus::Expired(now),
            FinalState::Stale => ReferendumStatus::Stale(now),
        };
        ReferendumStatusFor::<T>::insert(index, status);
        T::PollHooks::on_poll_completed(index);
        ReferendumTally::<T>::remove(index);

        let event = match state {
            FinalState::Approved => Event::<T>::Approved { index },
            FinalState::Rejected => Event::<T>::Rejected { index },
            FinalState::Cancelled => Event::<T>::Cancelled { index },
            FinalState::Expired => Event::<T>::Expired { index },
            FinalState::Stale => Event::<T>::Stale { index },
        };
        Self::deposit_event(event);
    }

    fn schedule_alarm(index: ReferendumIndex, when: BlockNumberFor<T>) -> Result<(), ()> {
        let name = Self::alarm_task_name(index);
        let call: <T as Config>::RuntimeCall = Call::<T>::nudge_referendum { index }.into();
        let bounded = T::Preimages::bound(call).map_err(|_| ())?;
        T::Scheduler::schedule_named(
            name,
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
        let name = Self::alarm_task_name(index);
        T::Scheduler::cancel_named(name).map_err(|_| ())
    }

    fn alarm_task_name(index: ReferendumIndex) -> TaskName {
        // `tag.len()` (25) + `u32` LE bytes (4) = 29 ≤ 32; bounds verified at compile time below.
        const TAG: &[u8] = b"subtensor/referenda/alarm";
        const _: () = assert!(TAG.len() + core::mem::size_of::<ReferendumIndex>() <= 32);

        let mut bytes = [0u8; 32];
        if let Some(slot) = bytes.get_mut(..TAG.len()) {
            slot.copy_from_slice(TAG);
        }
        let idx_bytes = index.to_le_bytes();
        let end = TAG.len().saturating_add(idx_bytes.len());
        if let Some(slot) = bytes.get_mut(TAG.len()..end) {
            slot.copy_from_slice(&idx_bytes);
        }
        // Hash to get a fully distinct, well-distributed 32-byte name.
        <T::Hashing as HashT>::hash(&bytes)
            .as_ref()
            .try_into()
            .unwrap_or(bytes)
    }

    /// Linear interpolation of the additional delay for an `Adjustable` referendum:
    /// returns `initial_delay * (1 - approval / fast_track_threshold)`, clamped so that
    /// `approval >= fast_track_threshold` yields zero.
    fn interpolate_delay(
        approval: Perbill,
        fast_track_threshold: Perbill,
        initial_delay: BlockNumberFor<T>,
    ) -> BlockNumberFor<T> {
        use sp_runtime::traits::SaturatedConversion;

        let approval_parts = approval.deconstruct() as u64;
        // `max(1)` prevents divide-by-zero in the (nonsensical) 0% threshold case.
        let threshold_parts = (fast_track_threshold.deconstruct() as u64).max(1);
        let clamped = approval_parts.min(threshold_parts);
        let numerator = threshold_parts.saturating_sub(clamped);
        let initial_u64: u64 = initial_delay.saturated_into();
        let additional_u64 = initial_u64
            .saturating_mul(numerator)
            .checked_div(threshold_parts)
            .unwrap_or(0);
        additional_u64.saturated_into()
    }
}

enum FinalState {
    Approved,
    Rejected,
    Cancelled,
    Expired,
    Stale,
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
        let status = ReferendumStatusFor::<T>::get(index)?;
        let info = match status {
            ReferendumStatus::Ongoing(i) => i,
            _ => return None,
        };
        T::Tracks::info(info.track).map(|t| t.voting_scheme)
    }

    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet> {
        let status = ReferendumStatusFor::<T>::get(index)?;
        let info = match status {
            ReferendumStatus::Ongoing(i) => i,
            _ => return None,
        };
        T::Tracks::info(info.track).map(|t| t.voter_set)
    }

    fn on_tally_updated(index: Self::Index, tally: &VoteTally) {
        ReferendumTally::<T>::insert(index, tally.clone());

        let status = match ReferendumStatusFor::<T>::get(index) {
            Some(ReferendumStatus::Ongoing(info)) => info,
            _ => return,
        };
        let track = match T::Tracks::info(status.track) {
            Some(t) => t,
            None => return,
        };

        match &track.decision_strategy {
            DecisionStrategy::PassOrFail {
                approve_threshold,
                reject_threshold,
                ..
            } => {
                if tally.approval >= *approve_threshold {
                    if status.alarm.is_some() {
                        let _ = Self::cancel_alarm(index);
                    }
                    Self::try_approve(index, &status);
                } else if tally.rejection >= *reject_threshold {
                    if status.alarm.is_some() {
                        let _ = Self::cancel_alarm(index);
                    }
                    Self::finalize(index, FinalState::Rejected);
                }
            }
            DecisionStrategy::Adjustable {
                initial_delay,
                fast_track_threshold,
                reject_threshold,
            } => {
                let task_name = match &status.proposal {
                    Proposal::Review(n) => *n,
                    Proposal::Action(_) => return,
                };

                // If the underlying task is gone (executed or cancelled outside governance),
                // the Review referendum has nothing to act on — finalize as `Stale` so it
                // releases its track slot and stops accepting votes.
                let task_alive = <<T as Config>::Scheduler as ScheduleNamed<
                    BlockNumberFor<T>,
                    CallOf<T>,
                    PalletsOriginOf<T>,
                >>::next_dispatch_time(task_name)
                .is_ok();
                if !task_alive {
                    Self::finalize(index, FinalState::Stale);
                    return;
                }

                if tally.approval >= *fast_track_threshold {
                    let now = Self::now();
                    let when = now.saturating_add(1u32.into());
                    if T::Scheduler::reschedule_named(task_name, DispatchTime::At(when)).is_err() {
                        Self::finalize(index, FinalState::Stale);
                        return;
                    }
                    Self::deposit_event(Event::<T>::TaskRescheduled { index, at: when });
                    Self::finalize(index, FinalState::Approved);
                } else if tally.rejection >= *reject_threshold {
                    if T::Scheduler::cancel_named(task_name).is_err() {
                        Self::finalize(index, FinalState::Stale);
                        return;
                    }
                    Self::deposit_event(Event::<T>::TaskCancelled { index });
                    Self::finalize(index, FinalState::Cancelled);
                } else if let Some(baseline) = status.initial_dispatch_time {
                    // Linear interpolation between 0% approval (full `initial_delay`)
                    // and `fast_track_threshold` (zero extra delay).
                    let additional = Self::interpolate_delay(
                        tally.approval,
                        *fast_track_threshold,
                        *initial_delay,
                    );
                    let now = Self::now();
                    let target = baseline.saturating_add(additional);
                    let earliest = now.saturating_add(1u32.into());
                    let when = if target < earliest { earliest } else { target };
                    if T::Scheduler::reschedule_named(task_name, DispatchTime::At(when)).is_err() {
                        Self::finalize(index, FinalState::Stale);
                        return;
                    }
                    Self::deposit_event(Event::<T>::TaskRescheduled { index, at: when });
                }
            }
        }
    }
}
