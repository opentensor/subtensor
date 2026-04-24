#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    dispatch::{DispatchResult, RawOrigin},
    pallet_prelude::*,
    sp_runtime::{
        Perbill, Saturating,
        traits::{BlockNumberProvider, Dispatchable, Zero},
    },
    traits::{
        Bounded, QueryPreimage, StorePreimage,
        schedule::{
            DispatchTime,
            v3::{Anon as ScheduleAnon, Named as ScheduleNamed},
        },
    },
};
use frame_system::pallet_prelude::*;
use subtensor_runtime_common::{PollHooks, Polls, SetLike, VoteTally};

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const MAX_TRACK_NAME_LEN: usize = 32;
pub type TrackName = [u8; MAX_TRACK_NAME_LEN];

pub type PalletsOriginOf<T> =
    <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
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

pub type ReferendumStatusOf<T> = ReferendumStatus<
    AccountIdOf<T>,
    TrackIdOf<T>,
    BoundedCallOf<T>,
    BlockNumberFor<T>,
    ScheduleAddressOf<T>,
>;

pub type ReferendumInfoOf<T> = ReferendumInfo<
    AccountIdOf<T>,
    TrackIdOf<T>,
    BoundedCallOf<T>,
    BlockNumberFor<T>,
    ScheduleAddressOf<T>,
>;

pub type ReferendumIndex = u32;
pub type ProposalTaskName = [u8; 32];

// --- Proposal enum ---

#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
pub enum Proposal<Call> {
    /// A call to execute if approved.
    Action(Call),
    /// A reference to an existing scheduled task — votes adjust its timing.
    Review(ProposalTaskName),
}

// --- Decision strategy ---

#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
pub enum DecisionStrategy<BlockNumber> {
    /// Binary decision: the referendum passes or fails before a deadline.
    PassOrFail {
        decision_period: BlockNumber,
        approve_threshold: Perbill,
        reject_threshold: Perbill,
    },
    /// Timing adjustment for an already-scheduled task.
    Adjustable {
        initial_delay: BlockNumber,
        fast_track_threshold: Perbill,
        reject_threshold: Perbill,
    },
}

// --- Track types ---

#[derive(Clone, Debug)]
pub struct TrackInfo<Name, BlockNumber, ProposerSet, VoterSet, VotingScheme> {
    pub name: Name,
    pub proposer_set: ProposerSet,
    pub voting_scheme: VotingScheme,
    pub voter_set: VoterSet,
    pub decision_strategy: DecisionStrategy<BlockNumber>,
}

#[derive(Clone, Debug)]
pub struct Track<Id, Name, BlockNumber, ProposerSet, VoterSet, VotingScheme> {
    pub id: Id,
    pub info: TrackInfo<Name, BlockNumber, ProposerSet, VoterSet, VotingScheme>,
}

pub trait TracksInfo<Name, AccountId, Call, BlockNumber> {
    type Id: Parameter + MaxEncodedLen + Copy + Ord + PartialOrd + Send + Sync + 'static;
    type ProposerSet: SetLike<AccountId>;
    type VotingScheme: PartialEq;
    type VoterSet: SetLike<AccountId>;

    fn tracks() -> impl Iterator<
        Item = Track<
            Self::Id,
            Name,
            BlockNumber,
            Self::ProposerSet,
            Self::VoterSet,
            Self::VotingScheme,
        >,
    >;

    fn track_ids() -> impl Iterator<Item = Self::Id> {
        Self::tracks().map(|x| x.id)
    }

    fn info(
        id: Self::Id,
    ) -> Option<TrackInfo<Name, BlockNumber, Self::ProposerSet, Self::VoterSet, Self::VotingScheme>>
    {
        Self::tracks().find(|t| t.id == id).map(|t| t.info)
    }

    fn authorize_proposal(id: Self::Id, proposal: &Call) -> bool;
}

// --- Referendum types ---

#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
#[subtensor_macros::freeze_struct("722bd128d396b3fa")]
pub struct ReferendumInfo<AccountId, TrackId, Call, BlockNumber, ScheduleId> {
    pub track: TrackId,
    pub proposal: Proposal<Call>,
    pub submitter: AccountId,
    pub submitted: BlockNumber,
    pub scheduled_task: Option<(BlockNumber, ScheduleId)>,
}

#[derive(
    Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, PartialEq, Eq, TypeInfo, Debug,
)]
pub enum ReferendumStatus<AccountId, Id, Call, BlockNumber, ScheduleId> {
    Ongoing(ReferendumInfo<AccountId, Id, Call, BlockNumber, ScheduleId>),
    Approved(BlockNumber),
    Rejected(BlockNumber),
    Cancelled(BlockNumber),
    Expired(BlockNumber),
}

// --- Pallet ---

#[frame_support::pallet(dev_mode)]
#[allow(clippy::expect_used)]
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

        type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

        /// Lifecycle hooks for voting pallets.
        type PollHooks: PollHooks<ReferendumIndex>;
    }

    #[pallet::storage]
    pub type ReferendumCount<T: Config> = StorageValue<_, ReferendumIndex, ValueQuery>;

    /// Number of currently-ongoing referenda. Bounded by `MaxQueued`.
    /// Distinct from `ReferendumCount`, which is a monotonic ID generator.
    #[pallet::storage]
    pub type ActiveCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    pub type ReferendumStatusFor<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, ReferendumStatusOf<T>, OptionQuery>;

    /// Cached tally per referendum. Updated on each on_tally_updated call.
    #[pallet::storage]
    pub type ReferendumTallyOf<T: Config> =
        StorageMap<_, Blake2_128Concat, ReferendumIndex, VoteTally, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new referendum was submitted.
        Submitted {
            index: ReferendumIndex,
            track: TrackIdOf<T>,
            proposer: T::AccountId,
        },
        /// A referendum was approved.
        Approved { index: ReferendumIndex },
        /// A referendum was rejected.
        Rejected { index: ReferendumIndex },
        /// A referendum was cancelled.
        Cancelled { index: ReferendumIndex },
        /// A referendum expired without reaching any threshold.
        Expired { index: ReferendumIndex },
        /// A Review referendum adjusted the delay of a scheduled task.
        DelayAdjusted {
            index: ReferendumIndex,
            new_when: BlockNumberFor<T>,
        },
        /// A scheduler operation failed for a referendum.
        SchedulerOperationFailed { index: ReferendumIndex },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The specified track does not exist.
        BadTrack,
        /// The caller is not in the track's proposer set.
        NotProposer,
        /// The referendum is not active.
        ReferendumFinalized,
        /// The proposal is not authorized for this track.
        ProposalNotAuthorized,
        /// Too many active referenda.
        QueueFull,
        /// An operation on the scheduler failed.
        SchedulerError,
        /// The specified referendum does not exist.
        ReferendumNotFound,
        /// The proposal type is not compatible with the track's decision strategy.
        InvalidConfiguration,
        /// The named task referenced by a Review proposal is not scheduled.
        ReviewTaskNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new referendum.
        #[pallet::call_index(0)]
        pub fn submit(
            origin: OriginFor<T>,
            track: TrackIdOf<T>,
            proposal: Proposal<BoundedCallOf<T>>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;

            // 1. Validate track
            let track_info = T::Tracks::info(track).ok_or(Error::<T>::BadTrack)?;

            // 2. Validate proposal-strategy compatibility
            ensure!(
                Self::is_valid_configuration(&proposal, &track_info.decision_strategy),
                Error::<T>::InvalidConfiguration
            );

            // 2b. For Review proposals, verify the named task is actually scheduled.
            if let Proposal::Review(task_name) = &proposal {
                ensure!(
                    <T::Scheduler as ScheduleNamed<
                        BlockNumberFor<T>,
                        CallOf<T>,
                        PalletsOriginOf<T>,
                    >>::next_dispatch_time(*task_name)
                    .is_ok(),
                    Error::<T>::ReviewTaskNotFound
                );
            }

            // 3. Validate proposer
            ensure!(
                track_info.proposer_set.contains(&submitter),
                Error::<T>::NotProposer
            );

            // 4. Check capacity against active referenda (not total submissions)
            let active = ActiveCount::<T>::get();
            ensure!(active < T::MaxQueued::get(), Error::<T>::QueueFull);

            let index = ReferendumCount::<T>::get();
            ReferendumCount::<T>::put(index.saturating_add(1));
            ActiveCount::<T>::put(active.saturating_add(1));

            // 4. Schedule finalization for PassOrFail deadline
            let now = T::BlockNumberProvider::current_block_number();
            let scheduled_task = if let DecisionStrategy::PassOrFail {
                decision_period, ..
            } = &track_info.decision_strategy
            {
                let when = now.saturating_add(*decision_period);
                let call: CallOf<T> = Call::<T>::finalize_referendum { index }.into();
                let bounded = T::Preimages::bound(call).map_err(|_| Error::<T>::SchedulerError)?;
                let address = T::Scheduler::schedule(
                    DispatchTime::At(when),
                    None,
                    128u8,
                    RawOrigin::Root.into(),
                    bounded,
                )
                .map_err(|_| Error::<T>::SchedulerError)?;
                Some((when, address))
            } else {
                None
            };

            // 5. Store referendum
            let info = ReferendumInfo {
                track,
                proposal,
                submitter: submitter.clone(),
                submitted: now,
                scheduled_task,
            };
            ReferendumStatusFor::<T>::insert(index, ReferendumStatus::Ongoing(info));

            // 6. Notify voting pallets
            T::PollHooks::on_poll_created(index);

            // 7. Emit event
            Self::deposit_event(Event::<T>::Submitted {
                index,
                track,
                proposer: submitter,
            });

            Ok(())
        }

        /// Cancel an ongoing referendum.
        #[pallet::call_index(1)]
        pub fn cancel(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            T::CancelOrigin::ensure_origin(origin)?;

            let status =
                ReferendumStatusFor::<T>::get(index).ok_or(Error::<T>::ReferendumNotFound)?;

            let ReferendumStatus::Ongoing(info) = status else {
                return Err(Error::<T>::ReferendumFinalized.into());
            };

            // Cancel any scheduled task
            if let Some((_when, address)) = info.scheduled_task
                && let Err(err) = T::Scheduler::cancel(address)
            {
                Self::handle_scheduler_error(index, "cancel", err);
            }

            Self::conclude(
                index,
                ReferendumStatusOf::<T>::Cancelled,
                Event::<T>::Cancelled { index },
            );
            Ok(())
        }

        /// Called by the scheduler when a PassOrFail referendum's decision_period expires.
        #[pallet::call_index(2)]
        pub fn finalize_referendum(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
            ensure_root(origin)?;

            let status =
                ReferendumStatusFor::<T>::get(index).ok_or(Error::<T>::ReferendumNotFound)?;

            let ReferendumStatus::Ongoing(info) = status else {
                return Err(Error::<T>::ReferendumFinalized.into());
            };

            let track_info = T::Tracks::info(info.track).ok_or(Error::<T>::BadTrack)?;

            let DecisionStrategy::PassOrFail {
                approve_threshold,
                reject_threshold,
                ..
            } = track_info.decision_strategy
            else {
                return Err(Error::<T>::InvalidConfiguration.into());
            };

            let tally = ReferendumTallyOf::<T>::get(index).unwrap_or_default();

            if tally.approval >= approve_threshold {
                Self::do_approve(index, &info);
            } else if tally.rejection >= reject_threshold {
                Self::do_reject(index);
            } else {
                Self::do_expire(index);
            }

            Ok(())
        }
    }
}

// --- Helper methods ---

impl<T: Config> Pallet<T> {
    /// Extract the ReferendumInfo from an Ongoing status.
    fn ongoing_referendum_info(index: ReferendumIndex) -> Option<ReferendumInfoOf<T>> {
        if let Some(ReferendumStatus::Ongoing(info)) = ReferendumStatusFor::<T>::get(index) {
            Some(info)
        } else {
            None
        }
    }

    /// Log and emit an event when a scheduler operation fails.
    fn handle_scheduler_error(index: ReferendumIndex, operation: &str, err: DispatchError) {
        log::error!(
            target: "runtime::referenda",
            "Scheduler {} failed for referendum {}: {:?}",
            operation,
            index,
            err,
        );
        Self::deposit_event(Event::<T>::SchedulerOperationFailed { index });
    }

    /// Record the final status, remove the tally, notify voting pallets, and emit the event.
    fn conclude(
        index: ReferendumIndex,
        status: fn(BlockNumberFor<T>) -> ReferendumStatusOf<T>,
        event: Event<T>,
    ) {
        let now = T::BlockNumberProvider::current_block_number();
        ReferendumStatusFor::<T>::insert(index, status(now));
        ReferendumTallyOf::<T>::remove(index);
        ActiveCount::<T>::mutate(|c| *c = c.saturating_sub(1));
        T::PollHooks::on_poll_completed(index);
        Self::deposit_event(event);
    }

    /// Evaluate the tally against the track's decision strategy and act accordingly.
    fn update_tally(index: ReferendumIndex, tally: &VoteTally) {
        ReferendumTallyOf::<T>::insert(index, tally);

        let Some(info) = Self::ongoing_referendum_info(index) else {
            return;
        };
        let Some(track_info) = T::Tracks::info(info.track) else {
            return;
        };

        match &info.proposal {
            Proposal::Action(_) => {
                let DecisionStrategy::PassOrFail {
                    approve_threshold,
                    reject_threshold,
                    ..
                } = &track_info.decision_strategy
                else {
                    // Unreachable: valid configuration enforced in is_valid_configuration
                    return;
                };

                if tally.approval >= *approve_threshold {
                    Self::do_approve(index, &info);
                } else if tally.rejection >= *reject_threshold {
                    Self::do_reject(index);
                }
            }
            Proposal::Review(task_name) => {
                let DecisionStrategy::Adjustable {
                    fast_track_threshold,
                    reject_threshold,
                    initial_delay,
                } = &track_info.decision_strategy
                else {
                    // Unreachable: valid configuration enforced in is_valid_configuration
                    return;
                };

                if tally.approval >= *fast_track_threshold {
                    Self::do_fast_track(index, task_name);
                } else if tally.rejection >= *reject_threshold {
                    Self::do_reject(index);
                } else {
                    Self::do_adjust_delay(
                        index,
                        task_name,
                        tally,
                        info.submitted,
                        *initial_delay,
                        *fast_track_threshold,
                    );
                }
            }
        }
    }

    /// Check that the proposal type is compatible with the track's decision strategy.
    fn is_valid_configuration(
        proposal: &Proposal<BoundedCallOf<T>>,
        strategy: &DecisionStrategy<BlockNumberFor<T>>,
    ) -> bool {
        matches!(
            (proposal, strategy),
            (Proposal::Action(_), DecisionStrategy::PassOrFail { .. })
                | (Proposal::Review(_), DecisionStrategy::Adjustable { .. })
        )
    }

    /// Approve a referendum: dispatch its Action call for execution.
    fn do_approve(index: ReferendumIndex, info: &ReferendumInfoOf<T>) {
        if let Some((_when, ref address)) = info.scheduled_task
            && let Err(err) = T::Scheduler::cancel(address.clone())
        {
            Self::handle_scheduler_error(index, "cancel", err);
        }

        if let Proposal::Action(ref bounded_call) = info.proposal
            && let Err(err) = T::Scheduler::schedule(
                DispatchTime::After(Zero::zero()),
                None,
                128u8,
                RawOrigin::Root.into(),
                bounded_call.clone(),
            )
        {
            Self::handle_scheduler_error(index, "schedule", err);
        }

        Self::conclude(
            index,
            ReferendumStatusOf::<T>::Approved,
            Event::<T>::Approved { index },
        );
    }

    /// Reject a referendum, cancelling any associated scheduled task.
    fn do_reject(index: ReferendumIndex) {
        if let Some(info) = Self::ongoing_referendum_info(index) {
            if let Some((_when, address)) = info.scheduled_task
                && let Err(err) = T::Scheduler::cancel(address)
            {
                Self::handle_scheduler_error(index, "cancel", err);
            }
            if let Proposal::Review(task_name) = info.proposal
                && let Err(err) = T::Scheduler::cancel_named(task_name)
            {
                Self::handle_scheduler_error(index, "cancel_named", err);
            }
        }

        Self::conclude(
            index,
            ReferendumStatusOf::<T>::Rejected,
            Event::<T>::Rejected { index },
        );
    }

    /// Expire a referendum that reached its deadline without meeting any threshold.
    fn do_expire(index: ReferendumIndex) {
        Self::conclude(
            index,
            ReferendumStatusOf::<T>::Expired,
            Event::<T>::Expired { index },
        );
    }

    /// Fast-track a Review referendum: reschedule its task to execute immediately.
    fn do_fast_track(index: ReferendumIndex, task_name: &ProposalTaskName) {
        if let Err(err) =
            T::Scheduler::reschedule_named(*task_name, DispatchTime::After(Zero::zero()))
        {
            Self::handle_scheduler_error(index, "reschedule_named", err);
        }

        Self::conclude(
            index,
            ReferendumStatusOf::<T>::Approved,
            Event::<T>::Approved { index },
        );
    }

    /// Adjust the delay of a scheduled task based on the tally.
    ///
    /// Linear interpolation: delay scales from `initial_delay` at approval = 0
    /// down to 0 as approval approaches `fast_track_threshold`. The dispatch
    /// target is anchored at `submitted` so that repeated vote updates don't
    /// drift the schedule forward. If elapsed time has already caught up to the
    /// interpolated target, fast-track immediately (matches V1's
    /// `elapsed > additional_delay` short-circuit).
    fn do_adjust_delay(
        index: ReferendumIndex,
        task_name: &ProposalTaskName,
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
            Self::do_fast_track(index, task_name);
            return;
        }

        // Skip the reschedule if the target didn't actually move —
        // the scheduler rejects no-op reschedules with RescheduleNoChange.
        if let Ok(current) = <T::Scheduler as ScheduleNamed<
            BlockNumberFor<T>,
            CallOf<T>,
            PalletsOriginOf<T>,
        >>::next_dispatch_time(*task_name)
            && current == target
        {
            return;
        }

        if let Err(err) = T::Scheduler::reschedule_named(*task_name, DispatchTime::At(target)) {
            Self::handle_scheduler_error(index, "reschedule_named", err);
            return;
        }

        Self::deposit_event(Event::<T>::DelayAdjusted {
            index,
            new_when: target,
        });
    }
}

// --- Polls trait implementation ---

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
        Self::ongoing_referendum_info(index)
            .and_then(|info| T::Tracks::info(info.track).map(|t| t.voting_scheme))
    }

    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet> {
        Self::ongoing_referendum_info(index)
            .and_then(|info| T::Tracks::info(info.track).map(|t| t.voter_set))
    }

    fn on_tally_updated(index: Self::Index, tally: &VoteTally) {
        Self::update_tally(index, tally);
    }
}
