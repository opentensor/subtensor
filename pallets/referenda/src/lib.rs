#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    sp_runtime::{
        Perbill,
        traits::{BlockNumberProvider, Dispatchable},
    },
    traits::{
        Bounded, QueryPreimage, StorePreimage,
        schedule::v3::{Anon as ScheduleAnon, Named as ScheduleNamed},
    },
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{Polls, SetLike, VoteTally};

pub use pallet::*;

pub const MAX_TRACK_NAME_LEN: usize = 32;
type TrackName = [u8; MAX_TRACK_NAME_LEN];

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

pub type ReferendumStatusOf<T> =
    ReferendumStatus<TrackIdOf<T>, CallOf<T>, BlockNumberFor<T>, ScheduleAddressOf<T>>;

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

        type CancelOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type Tracks: TracksInfo<TrackName, Self::AccountId, CallOf<Self>, BlockNumberFor<Self>>;

        type BlockNumberProvider: BlockNumberProvider;
    }

    #[pallet::storage]
    pub type ReferendumCount<T: Config> = StorageValue<_, ReferendumIndex, ValueQuery>;

    #[pallet::storage]
    pub type ReferendumStatusFor<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, ReferendumStatusOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn submit(
            _origin: OriginFor<T>,
            _track: TrackIdOf<T>,
            _proposal: (),
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn cancel(_origin: OriginFor<T>, _index: ReferendumIndex) -> DispatchResult {
            Ok(())
        }
    }
}

pub type ReferendumIndex = u32;

pub struct TrackInfo<Name, Moment, ProposerSet, VoterSet, VotingScheme> {
    pub name: Name,
    pub proposer_set: ProposerSet,
    pub voting_scheme: VotingScheme,
    pub voter_set: VoterSet,
    pub decision_strategy: DecisionStrategy<Moment>,
}

pub struct Track<Id, Name, Moment, ProposerSet, VoterSet, VotingScheme> {
    pub id: Id,
    pub info: TrackInfo<Name, Moment, ProposerSet, VoterSet, VotingScheme>,
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
    ) -> Option<TrackInfo<Name, Moment, Self::ProposerSet, Self::VoterSet, Self::VotingScheme>>
    {
        Self::tracks().find(|t| t.id == id).map(|t| t.info)
    }

    fn authorize_proposal(id: Self::Id, proposal: &Call) -> bool;
}

pub struct ReferendumInfo<TrackId, Call, Moment, ScheduleAddress> {
    pub track: TrackId,
    pub proposal: Call,
    pub submitter: AccountId,
    pub submitted: Moment,
    pub tally: VoteTally,
    pub alarm: Option<(Moment, ScheduleAddress)>,
}

pub enum ReferendumStatus<Id, Call, Moment, ScheduleAddress> {
    Ongoing(ReferendumInfo<Id, Call, Moment, ScheduleAddress>),
    Approved(Moment),
    Rejected(Moment),
    Cancelled(Moment),
    Expired(Moment),
}

/// The decision strategy for a track.
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
        /// The delay from the current block to the initial scheduled execution.
        initial_delay: Moment,
        /// Approval above this threshold reschedules the task to execute immediately.
        fast_track_threshold: Perbill,
        /// Rejection above this threshold cancels the scheduled task entirely.
        reject_threshold: Perbill,
    },
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
