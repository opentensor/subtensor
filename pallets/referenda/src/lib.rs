#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    sp_runtime::{
        Perbill, Saturating,
        traits::{BlockNumberProvider, Dispatchable},
    },
    traits::{
        Bounded, EnsureOriginWithArg, LockIdentifier, QueryPreimage, StorePreimage,
        schedule::{
            DispatchTime, Priority,
            v3::{Anon as ScheduleAnon, Named as ScheduleNamed, TaskName},
        },
    },
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

pub use pallet::*;

pub const MAX_TRACK_NAME_LEN: usize = 32;
type TrackName = [u8; MAX_TRACK_NAME_LEN];

pub const ASSEMBLY_ID: LockIdentifier = *b"assembly";

pub type PalletsOriginOf<T> =
    <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

type CallOf<T> = <T as Config>::RuntimeCall;
type BoundedCallOf<T> = Bounded<CallOf<T>, <T as frame_system::Config>::Hashing>;

type ScheduleAddressOf<T> = <<T as Config>::Scheduler as ScheduleAnon<
    BlockNumberFor<T>,
    CallOf<T>,
    PalletsOriginOf<T>,
>>::Address;

pub type BlockNumberFor<T> =
    <<T as Config>::BlockNumberProvider as BlockNumberProvider>::BlockNumber;

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

#[frame_support::pallet(dev_mode)]
pub mod pallet {
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

        type MaxQueued: Get<u32>;

        type SubmitOrigin: EnsureOriginWithArg<
                Self::RuntimeOrigin,
                TrackIdOf<Self>,
                Success = Option<Self::AccountId>,
            >;

        type KillOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, ReferendumIndex>;

        type Tracks: TracksInfo<TrackName, Self::AccountId, CallOf<Self>, BlockNumberFor<Self>>;

        type PollHooks: PollHooks<ReferendumIndex>;

        type BlockNumberProvider: BlockNumberProvider;
    }

    #[pallet::storage]
    pub type ReferendumCount<T: Config> = StorageValue<_, ReferendumIndex, ValueQuery>;

    #[pallet::storage]
    pub type ReferendumStatusFor<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, ReferendumStatusOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Submitted {
            submitter: Option<AccountIdOf<T>>,
            index: ReferendumIndex,
            track: TrackIdOf<T>,
            proposal: Proposal<BoundedCallOf<T>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        TrackNotFound,
        PreimageStoredWithDifferentLength,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn submit(
            origin: OriginFor<T>,
            track_id: TrackIdOf<T>,
            call: Box<<T as Config>::RuntimeCall>,
        ) -> DispatchResult {
            let who = T::SubmitOrigin::ensure_origin(origin, &track_id)?;

            let bounded_call = T::Preimages::bound(*call)?;

            let now = T::BlockNumberProvider::current_block_number();
            let index = ReferendumCount::<T>::get();
            ReferendumCount::<T>::put(index.saturating_add(1));

            let track = T::Tracks::info(track_id).ok_or(Error::<T>::TrackNotFound)?;

            let (proposal, alarm) = match track.decision_strategy {
                DecisionStrategy::PassOrFail {
                    decision_period, ..
                } => {
                    let when = now.saturating_add(decision_period);
                    // The alarm will trigger when the decision period ends
                    // to mark the referendum as expired if it has not been decided.
                    let alarm = Self::set_alarm(index, when)?;
                    (Proposal::Action(bounded_call), alarm)
                }
                DecisionStrategy::Adjustable { initial_delay, .. } => {
                    let when = now.saturating_add(initial_delay);
                    Self::schedule_adjustable(index, when, bounded_call)?;
                    // The alarm will trigger just after the scheduled proposal
                    // to check if it has been executed and update the status accordingly,
                    // it will be updated every time the schedule is adjusted.
                    let alarm = Self::set_alarm(index, when.saturating_add(One::one()))?;
                    (Proposal::Review, alarm)
                }
            };

            let info = ReferendumInfo {
                track: track_id,
                proposal: proposal.clone(),
                submitted: now,
                tally: VoteTally::new(),
                alarm,
            };
            ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));

            T::PollHooks::on_poll_created(index);

            Self::deposit_event(Event::<T>::Submitted {
                submitter: who,
                track: track_id,
                index,
                proposal,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn kill(_origin: OriginFor<T>, _index: ReferendumIndex) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(2)]
        pub fn cleanup(_origin: OriginFor<T>, _index: ReferendumIndex) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(3)]
        pub fn nudge_referendum(_origin: OriginFor<T>, _index: ReferendumIndex) -> DispatchResult {
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn referendum_task_name(index: ReferendumIndex) -> TaskName {
        (ASSEMBLY_ID, "adjustable", index).using_encoded(sp_io::hashing::blake2_256)
    }

    fn set_alarm(
        index: ReferendumIndex,
        when: BlockNumberFor<T>,
    ) -> Result<(BlockNumberFor<T>, ScheduleAddressOf<T>), DispatchError> {
        let call = T::Preimages::bound(CallOf::<T>::from(Call::nudge_referendum { index }))?;
        T::Scheduler::schedule(
            DispatchTime::At(when),
            None,
            Priority::MAX,
            frame_system::RawOrigin::Root.into(),
            call,
        )
        .map(|address| (when, address))
    }

    fn schedule_adjustable(
        index: ReferendumIndex,
        when: BlockNumberFor<T>,
        call: BoundedCallOf<T>,
    ) -> DispatchResult {
        T::Scheduler::schedule_named(
            Self::referendum_task_name(index),
            DispatchTime::At(when),
            None,
            Priority::MAX,
            frame_system::RawOrigin::Root.into(),
            call,
        )?;
        Ok(())
    }
}

pub type ReferendumIndex = u32;

pub struct TrackInfo<Id, Name, Moment, ProposerSet, VoterSet, VotingScheme> {
    pub name: Name,
    pub proposer_set: ProposerSet,
    pub voting_scheme: VotingScheme,
    pub voter_set: VoterSet,
    pub decision_strategy: DecisionStrategy<Id, Moment>,
}

pub struct Track<Id, Name, Moment, ProposerSet, VoterSet, VotingScheme> {
    pub id: Id,
    pub info: TrackInfo<Id, Name, Moment, ProposerSet, VoterSet, VotingScheme>,
}

pub trait TracksInfo<Name, AccountId, Call, Moment> {
    type Id: Parameter + MaxEncodedLen + Copy + Ord + PartialOrd + Send + Sync + 'static;

    type ProposerSet: SetLike<AccountId>;

    type VotingScheme: PartialEq;
    type VoterSet: SetLike<AccountId>;

    fn tracks() -> impl Iterator<
        Item = Track<Self::Id, Name, Moment, Self::ProposerSet, Self::VoterSet, Self::VotingScheme>,
    >;

    fn track_ids() -> impl Iterator<Item = Self::Id> {
        Self::tracks().map(|x| x.id)
    }

    fn info(
        id: Self::Id,
    ) -> Option<
        TrackInfo<Self::Id, Name, Moment, Self::ProposerSet, Self::VoterSet, Self::VotingScheme>,
    > {
        Self::tracks().find(|t| t.id == id).map(|t| t.info)
    }

    fn authorize_proposal(id: Self::Id, proposal: &Call) -> bool;
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

impl<T: Config> Polls<T::AccountId> for Pallet<T> {
    type Index = ReferendumIndex;
    type VotingScheme = VotingSchemeOf<T>;
    type VoterSet = VoterSetOf<T>;

    fn is_ongoing(_index: Self::Index) -> bool {
        false
    }

    fn voting_scheme_of(_index: Self::Index) -> Option<Self::VotingScheme> {
        None
    }

    fn voter_set_of(_index: Self::Index) -> Option<Self::VoterSet> {
        None
    }

    fn on_tally_updated(_index: Self::Index, _tally: &VoteTally) {}
}
