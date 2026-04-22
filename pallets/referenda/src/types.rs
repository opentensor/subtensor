use super::Config;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{
    pallet_prelude::*,
    sp_runtime::Perbill,
    traits::{Bounded, OriginTrait},
};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use subtensor_runtime_common::{SetLike, VoteTally};

pub const MAX_TRACK_NAME_LEN: usize = 32;
pub type TrackName = [u8; MAX_TRACK_NAME_LEN];

pub type ReferendumIndex = u32;

pub type PalletsOriginOf<T> =
    <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
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

pub type ReferendumStatusOf<T> =
    ReferendumStatus<TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>>;
pub type ReferendumInfoOf<T> = ReferendumInfo<TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>>;

/// Proposal payload — marker or real call depending on track strategy.
///
/// - `Action(call)` is used on `PassOrFail` tracks: the call is stored bounded and
///   executed/scheduled based on vote outcome.
/// - `Review` is used on `Adjustable` tracks: the call was scheduled by the pallet at
///   submit time (the scheduler task is owned by referenda). The poll's role is to shift
///   the execution time, not decide whether to execute.
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum Proposal<Call> {
    Action(Call),
    Review,
}

/// What happens when a `PassOrFail` referendum reaches the approve threshold.
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum ApprovalAction<TrackId> {
    /// Schedule the approved call for enactment on the next block and finalize the poll.
    /// One-phase governance flow.
    Execute,
    /// Schedule the approved call for enactment on `review_track` (using its
    /// `initial_delay`) and automatically create a `Review` referendum there so a second
    /// collective can adjust/cancel the timing. Two-phase governance flow.
    ///
    /// `review_track` MUST be an `Adjustable` track in the runtime's `TracksInfo`.
    ScheduleAndReview { review_track: TrackId },
}

/// The decision strategy for a track.
#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum DecisionStrategy<TrackId, Moment> {
    /// Binary approve/reject with a time-bounded decision window.
    PassOrFail {
        /// How long voters have to reach a decision before the referendum expires.
        decision_period: Moment,
        /// Minimum approval fraction (ayes / total eligible) to approve.
        approve_threshold: Perbill,
        /// Minimum rejection fraction (nays / total eligible) to reject.
        reject_threshold: Perbill,
        /// What to do on approval — direct execution or hand off to a review track.
        on_approval: ApprovalAction<TrackId>,
    },
    /// Timing adjustment: the call is scheduled at submit time at `now + initial_delay`;
    /// votes shift the execution block. No deadline — lives until enactment or cancel.
    Adjustable {
        /// Baseline delay from `submit` block. At 0% approval the task sits at this offset.
        /// Linearly shrinks to 0 as approval approaches `fast_track_threshold`.
        initial_delay: Moment,
        /// Approval above this threshold reschedules enactment to the next block.
        fast_track_threshold: Perbill,
        /// Rejection above this threshold cancels the scheduled call entirely.
        reject_threshold: Perbill,
    },
}

#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub struct ReferendumInfo<TrackId, Call, Moment> {
    pub track: TrackId,
    pub proposal: Proposal<Call>,
    pub submitted: Moment,
    /// Current tally. Updated in place by `on_tally_updated`. Single source of truth.
    pub tally: VoteTally,
    /// `PassOrFail`: when the timeout alarm fires. `Adjustable`: unused.
    pub alarm: Option<Moment>,
    /// `Adjustable` Review referenda: originally-scheduled enactment block, captured at
    /// submit time and used as the baseline for linear delay interpolation (so the
    /// baseline doesn't drift while votes reschedule the task).
    pub initial_dispatch_time: Option<Moment>,
}

#[derive(
    Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, Debug,
)]
pub enum ReferendumStatus<TrackId, Call, Moment> {
    Ongoing(ReferendumInfo<TrackId, Call, Moment>),
    /// PassOrFail approved; call has been scheduled (or auto-spawned a review poll).
    Approved(Moment),
    /// PassOrFail rejected — approve threshold not met, reject threshold met.
    Rejected(Moment),
    /// Origin-initiated cancellation via `CancelOrigin`.
    Cancelled(Moment),
    /// PassOrFail timed out without reaching either threshold.
    Expired(Moment),
    /// Adjustable poll reached `fast_track_threshold`; task rescheduled to next block.
    FastTracked(Moment),
    /// Adjustable poll whose underlying scheduler task vanished externally (executed or
    /// cancelled outside governance). Vote outcome is moot.
    Stale(Moment),
}

#[derive(Clone)]
pub struct TrackInfo<Id, Name, Moment, VoterSet, VotingScheme> {
    pub name: Name,
    pub voting_scheme: VotingScheme,
    pub voter_set: VoterSet,
    pub decision_strategy: DecisionStrategy<Id, Moment>,
}

#[derive(Clone)]
pub struct Track<Id, Name, Moment, VoterSet, VotingScheme> {
    pub id: Id,
    pub info: TrackInfo<Id, Name, Moment, VoterSet, VotingScheme>,
}

pub trait TracksInfo<Name, AccountId, Call, Moment> {
    type Id: Parameter + MaxEncodedLen + Copy + Ord + PartialOrd + Send + Sync + 'static;
    type VotingScheme: PartialEq + Clone;
    type VoterSet: SetLike<AccountId>;

    fn tracks()
    -> impl Iterator<Item = Track<Self::Id, Name, Moment, Self::VoterSet, Self::VotingScheme>>;

    fn track_ids() -> impl Iterator<Item = Self::Id> {
        Self::tracks().map(|t| t.id)
    }

    fn info(
        id: Self::Id,
    ) -> Option<TrackInfo<Self::Id, Name, Moment, Self::VoterSet, Self::VotingScheme>> {
        Self::tracks().find(|t| t.id == id).map(|t| t.info)
    }

    /// Optional per-track authorization of a proposed call. Default allows all.
    fn authorize_proposal(_id: Self::Id, _call: &Call) -> bool {
        true
    }
}
