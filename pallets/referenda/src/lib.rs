#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use codec::Encode;
use frame_support::{
    Parameter, Twox64Concat,
    dispatch::DispatchResult,
    pallet_prelude::{IsType, OptionQuery, StorageMap, StorageValue, ValueQuery},
    sp_runtime::{
        DispatchError, Saturating,
        traits::{BlockNumberProvider, Dispatchable, One},
    },
    traits::{
        EnsureOriginWithArg, LockIdentifier, QueryPreimage, StorePreimage,
        schedule::{
            DispatchTime, Priority,
            v3::{Named as ScheduleNamed, TaskName},
        },
    },
};
use frame_system::pallet_prelude::{OriginFor, ensure_root};
use subtensor_runtime_common::{PollHooks, Polls, VoteTally};

pub use pallet::*;
pub use types::*;

mod types;

pub const REFERENDA_ID: LockIdentifier = *b"referend";

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

        type Scheduler: ScheduleNamed<
                BlockNumberFor<Self>,
                CallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            >;

        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

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
        StorageMap<_, Twox64Concat, ReferendumIndex, ReferendumStatusOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Submitted {
            submitter: Option<AccountIdOf<T>>,
            index: ReferendumIndex,
            track: TrackIdOf<T>,
            proposal: Proposal<BoundedCallOf<T>>,
        },
        Approved {
            index: ReferendumIndex,
        },
        Rejected {
            index: ReferendumIndex,
        },
        Expired {
            index: ReferendumIndex,
        },
        FastTracked {
            index: ReferendumIndex,
        },
        Cancelled {
            index: ReferendumIndex,
        },
        Enacted {
            index: ReferendumIndex,
            when: BlockNumberFor<T>,
        },
        Killed {
            index: ReferendumIndex,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        TrackNotFound,
        NotFound,
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

            let proposal = match track.decision_strategy {
                DecisionStrategy::PassOrFail {
                    decision_period, ..
                } => {
                    // Triggers after decision period ends to mark the referendum as expired if
                    // it has not been decided yet.
                    Self::set_alarm(index, now.saturating_add(decision_period))?;
                    Proposal::Action(bounded_call)
                }
                DecisionStrategy::Adjustable { initial_delay, .. } => {
                    let when = now.saturating_add(initial_delay);
                    Self::schedule_enactment(index, when, bounded_call)?;
                    // Triggers after initial delay to check if the referendum has been enacted
                    // and update the status accordingly if it has.
                    Self::set_alarm(index, when.saturating_add(One::one()))?;
                    Proposal::Review
                }
            };

            let info = ReferendumInfo {
                track: track_id,
                proposal: proposal.clone(),
                submitted: now,
                tally: VoteTally::new(),
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
        pub fn advance_referendum(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            ensure_root(origin)?;
            let now = T::BlockNumberProvider::current_block_number();
            let status = ReferendumStatusFor::<T>::get(index).ok_or(Error::<T>::NotFound)?;

            match status {
                ReferendumStatus::Ongoing(info) => {
                    let track = T::Tracks::info(info.track).ok_or(Error::<T>::TrackNotFound)?;

                    match (info.proposal, track.decision_strategy) {
                        (
                            Proposal::Action(call),
                            DecisionStrategy::PassOrFail {
                                decision_period,
                                approve_threshold,
                                reject_threshold,
                                on_approval,
                            },
                        ) => {
                            if info.tally.approval >= approve_threshold {
                                match on_approval {
                                    ApprovalAction::Execute => call.dispatch(),
                                    ApprovalAction::ScheduleAndReview { review_track } => {
                                        // Move to new track
                                    }
                                };
                            } else if info.tally.rejection >= reject_threshold {
                            }
                        }
                        (
                            Proposal::Review,
                            DecisionStrategy::Adjustable {
                                initial_delay,
                                fast_track_threshold,
                                cancel_threshold,
                            },
                        ) => {
                            if info.tally.approval >= fast_track_threshold {
                                Self::schedule_enactment(
                                    index,
                                    now.saturating_add(One::one()),
                                    call,
                                )?;
                                let new_status = ReferendumStatus::FastTracked(now);
                                ReferendumStatusFor::<T>::insert(index, new_status);
                                Self::deposit_event(Event::<T>::FastTracked { index });
                                // Schedule for next block
                            } else if info.tally.rejection >= cancel_threshold {
                                let new_status = ReferendumStatus::Cancelled(now);
                                ReferendumStatusFor::<T>::insert(index, status);
                                Self::deposit_event(Event::<T>::Cancelled { index });
                            } else {
                                // Adjust the delay
                            }
                        }
                    }
                }
                ReferendumStatus::Approved(_) | ReferendumStatus::FastTracked(_) => {
                    // The referendum has been enacted in the last block so we update the
                    // status accordingly.
                    let when = now.saturating_sub(One::one());
                    ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Enacted(when));
                    Self::deposit_event(Event::<T>::Enacted { index, when });
                }
                _ => {}
            };
            // match (info.proposal, track.decision_strategy) {
            //     (
            //         Proposal::Action(call),
            //         DecisionStrategy::PassOrFail {
            //             decision_period,
            //             approve_threshold,
            //             reject_threshold,
            //             on_approval,
            //         },
            //     ) => {
            //         if tally.approval >= *approve_threshold {
            //             match on_approval {
            //                 ApprovalAction::Execute => {
            //                     T::Scheduler::cancel_named(alarm_name(index))?;
            //                     call.dispatch()
            //                 }
            //                 ApprovalAction::ScheduleAndReview { review_track } => {
            //                     // Move to new track
            //                 }
            //             };
            //             Self::deposit_event(Event::<T>::Approved { index });
            //         } else if tally.rejection >= *reject_threshold {
            //         }
            //     }
            //     (
            //         Proposal::Review,
            //         DecisionStrategy::Adjustable {
            //             initial_delay,
            //             fast_track_threshold,
            //             cancel_threshold,
            //         },
            //     ) => {
            //         if tally.approval >= fast_track_threshold {
            //             // Schedule for next block
            //         } else if tally.rejection >= cancel_threshold {

            //         } else {
            //             // Adjust the delay
            //         }
            //         None
            //     }
            //     // Unreachable, track decision strategy defines proposal type
            //     _ => None,
            // };

            Ok(())
        }

        #[pallet::call_index(3)]
        pub fn cleanup(_origin: OriginFor<T>, _index: ReferendumIndex) -> DispatchResult {
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn set_alarm(index: ReferendumIndex, when: BlockNumberFor<T>) -> DispatchResult {
        let call = T::Preimages::bound(CallOf::<T>::from(Call::nudge_referendum { index }))?;
        T::Scheduler::schedule_named(
            alarm_name(index),
            DispatchTime::At(when),
            None,
            Priority::MAX,
            frame_system::RawOrigin::Root.into(),
            call,
        )?;
        Ok(())
    }

    fn schedule_enactment(
        index: ReferendumIndex,
        desired: DispatchTime<BlockNumberFor<T>>,
        call: BoundedCallOf<T>,
    ) -> DispatchResult {
        T::Scheduler::schedule_named(
            task_name(index),
            desired,
            None,
            0,
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

    fn on_tally_updated(index: Self::Index, tally: &VoteTally) {
        let Some(mut info) = Self::ensure_ongoing(index).ok() else {
            return;
        };
        let Some(track) = T::Tracks::info(info.track) else {
            return;
        };
        let now = T::BlockNumberProvider::current_block_number();

        info.tally = *tally;
        ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));

        let _ = T::Scheduler::cancel_named(alarm_name(index));
        let _ = Self::set_alarm(index, now.saturating_add(One::one()));
    }
}

fn task_name(index: ReferendumIndex) -> TaskName {
    (REFERENDA_ID, "enactment", index).using_encoded(sp_io::hashing::blake2_256)
}

fn alarm_name(index: ReferendumIndex) -> TaskName {
    (REFERENDA_ID, "alarm", index).using_encoded(sp_io::hashing::blake2_256)
}
