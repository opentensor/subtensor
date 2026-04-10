use super::*;
use frame_support::{
    pallet_prelude::*,
    sp_runtime::{Perbill, traits::BlockNumberProvider},
    traits::{Bounded, schedule::v3::Anon as ScheduleAnon},
};
use subtensor_runtime_common::{SetLike, VoteTally};

pub const MAX_TRACK_NAME_LEN: usize = 32;
pub type TrackName = [u8; MAX_TRACK_NAME_LEN];

pub type BlockNumberFor<T> =
    <<T as Config>::BlockNumberProvider as BlockNumberProvider>::BlockNumber;
pub type PalletsOriginOf<T> =
    <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type CallOf<T> = <T as Config>::RuntimeCall;
pub type BoundedCallOf<T> = Bounded<CallOf<T>, <T as frame_system::Config>::Hashing>;

pub type ScheduleAddressOf<T> = <<T as Config>::Scheduler as ScheduleAnon<
    BlockNumberFor<T>,
    CallOf<T>,
    PalletsOriginOf<T>,
>>::Address;

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
    ReferendumStatus<TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>, ScheduleAddressOf<T>>;
pub type ReferendumInfoOf<T> =
    ReferendumInfo<TrackIdOf<T>, BoundedCallOf<T>, BlockNumberFor<T>, ScheduleAddressOf<T>>;

pub type ReferendumIndex = u32;

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct ReferendumInfo<TrackId, Call, Moment, ScheduleAddress> {
    pub track: TrackId,
    pub proposal: Proposal<Call>,
    pub submitted: Moment,
    pub tally: VoteTally,
    pub alarm: (Moment, ScheduleAddress),
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum ReferendumStatus<TrackId, Call, Moment, ScheduleAddress> {
    Ongoing(ReferendumInfo<TrackId, Call, Moment, ScheduleAddress>),
    Approved(Moment),
    Rejected(Moment),
    Expired(Moment),
    FastTracked(Moment),
    Cancelled(Moment),
    Executed(Moment),
    Killed(Moment),
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum DecisionStrategy<TrackId, Moment> {
    PassOrFail {
        decision_period: Moment,
        approve_threshold: Perbill,
        reject_threshold: Perbill,
        on_approval: ApprovalAction<TrackId>,
    },
    Adjustable {
        initial_delay: Moment,
        fast_track_threshold: Perbill,
        cancel_threshold: Perbill,
    },
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum ApprovalAction<TrackId> {
    Execute,
    ScheduleAndReview { review_track: TrackId },
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum Proposal<Call> {
    Action(Call),
    Review,
}

pub struct TrackInfo<Id, Name, Moment, VoterSet, VotingScheme> {
    pub name: Name,
    pub voting_scheme: VotingScheme,
    pub voter_set: VoterSet,
    pub decision_strategy: DecisionStrategy<Id, Moment>,
}

pub struct Track<Id, Name, Moment, VoterSet, VotingScheme> {
    pub id: Id,
    pub info: TrackInfo<Id, Name, Moment, VoterSet, VotingScheme>,
}

pub trait TracksInfo<Name, AccountId, Call, Moment> {
    type Id: Parameter + MaxEncodedLen + Copy + Ord + PartialOrd + Send + Sync + 'static;
    type VotingScheme: PartialEq;
    type VoterSet: SetLike<AccountId>;

    fn tracks()
    -> impl Iterator<Item = Track<Self::Id, Name, Moment, Self::VoterSet, Self::VotingScheme>>;

    fn track_ids() -> impl Iterator<Item = Self::Id> {
        Self::tracks().map(|x| x.id)
    }

    fn info(
        id: Self::Id,
    ) -> Option<TrackInfo<Self::Id, Name, Moment, Self::VoterSet, Self::VotingScheme>> {
        Self::tracks().find(|t| t.id == id).map(|t| t.info)
    }

    fn authorize_proposal(id: Self::Id, proposal: &Call) -> bool;
}
