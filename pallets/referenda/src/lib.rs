#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use codec::Encode;
use frame_support::{
    Blake2_128Concat, Parameter,
    dispatch::DispatchResult,
    pallet_prelude::{Get, IsType, OptionQuery, StorageMap, StorageValue, ValueQuery},
    sp_runtime::{
        DispatchError, Saturating,
        traits::{BlockNumberProvider, Dispatchable, One},
    },
    traits::{
        EnsureOriginWithArg, LockIdentifier, QueryPreimage, StorePreimage,
        schedule::{
            DispatchTime, Priority,
            v3::{Anon as ScheduleAnon, Named as ScheduleNamed, TaskName},
        },
    },
};
use frame_system::pallet_prelude::{OriginFor, ensure_root};
use subtensor_runtime_common::{PollHooks, Polls, VoteTally};

pub use pallet::*;
pub use types::*;

mod types;

pub const ASSEMBLY_ID: LockIdentifier = *b"assembly";

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
        Expired {
            index: ReferendumIndex,
        },
        Executed {
            when: BlockNumberFor<T>,
            index: ReferendumIndex,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        TrackNotFound,
        NotOngoing,
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
                    // Triggers after decision period ends to mark the referendum as expired if
                    // it has not been decided yet.
                    let alarm = Self::set_alarm(index, when)?;
                    (Proposal::Action(bounded_call), alarm)
                }
                DecisionStrategy::Adjustable { initial_delay, .. } => {
                    let when = now.saturating_add(initial_delay);
                    Self::schedule_adjustable(index, when, bounded_call)?;
                    // Triggers after initial delay to check if the referendum has been executed
                    // and update the status accordingly if it has.
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
        pub fn nudge_referendum(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            ensure_root(origin)?;
            let info = Self::ensure_ongoing(index)?;
            let now = T::BlockNumberProvider::current_block_number();

            let new_status = match info.proposal {
                Proposal::Action(call) => {
                    T::Preimages::drop(&call);
                    Self::deposit_event(Event::<T>::Expired { index });
                    ReferendumStatus::Expired(now)
                }
                Proposal::Review => {
                    // Should never happen that the referendum is still scheduled while being nudged
                    // given the alarm is one block after the scheduled time but we check it anyway.
                    if Self::task_next_dispatch_time(task_name(index)).is_ok() {
                        return Ok(());
                    }
                    let when = now - One::one();
                    Self::deposit_event(Event::<T>::Executed { index, when });
                    ReferendumStatus::Executed(when)
                }
            };
            ReferendumStatusFor::<T>::insert(index, new_status);

            T::PollHooks::on_poll_completed(index);

            Ok(())
        }

        #[pallet::call_index(3)]
        pub fn cleanup(_origin: OriginFor<T>, _index: ReferendumIndex) -> DispatchResult {
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn task_next_dispatch_time(task_name: TaskName) -> Result<BlockNumberFor<T>, DispatchError> {
        <T::Scheduler as ScheduleNamed<
            BlockNumberFor<T>,
            CallOf<T>,
            PalletsOriginOf<T>,
        >>::next_dispatch_time(task_name)
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
            task_name(index),
            DispatchTime::At(when),
            None,
            Priority::MAX,
            frame_system::RawOrigin::Root.into(),
            call,
        )?;
        Ok(())
    }

    fn ensure_ongoing(index: ReferendumIndex) -> Result<ReferendumInfoOf<T>, DispatchError> {
        match ReferendumStatusFor::<T>::get(index) {
            Some(ReferendumStatus::Ongoing(info)) => Ok(info),
            _ => Err(Error::<T>::NotOngoing.into()),
        }
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
        let info = Self::ensure_ongoing(index).ok()?;
        let track = T::Tracks::info(info.track)?;
        Some(track.voting_scheme)
    }

    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet> {
        let info = Self::ensure_ongoing(index).ok()?;
        let track = T::Tracks::info(info.track)?;
        Some(track.voter_set)
    }

    fn on_tally_updated(_index: Self::Index, _tally: &VoteTally) {}
}

fn task_name(index: ReferendumIndex) -> TaskName {
    (ASSEMBLY_ID, "adjustable", index).using_encoded(sp_io::hashing::blake2_256)
}
