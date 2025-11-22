#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    dispatch::{GetDispatchInfo, RawOrigin},
    pallet_prelude::*,
    sp_runtime::traits::Dispatchable,
    traits::{
        Bounded, ChangeMembers, IsSubType, QueryPreimage, StorePreimage, fungible,
        schedule::{
            DispatchTime, Priority,
            v3::{Named as ScheduleNamed, TaskName},
        },
    },
};
use frame_system::pallet_prelude::*;
use sp_runtime::{Percent, Saturating, traits::Hash};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
use subtensor_macros::freeze_struct;

mod mock;
mod tests;
pub use pallet::*;

/// WARNING: Any changes to these 3 constants require a migration to update the `BoundedVec` in storage
/// for `Triumvirate`, `EconomicCollective`, or `BuildingCollective`.
pub const TRIUMVIRATE_SIZE: u32 = 3;
pub const ECONOMIC_COLLECTIVE_SIZE: u32 = 16;
pub const BUILDING_COLLECTIVE_SIZE: u32 = 16;

pub const TOTAL_COLLECTIVES_SIZE: u32 = ECONOMIC_COLLECTIVE_SIZE + BUILDING_COLLECTIVE_SIZE;

pub type CurrencyOf<T> = <T as Config>::Currency;

pub type BalanceOf<T> =
    <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type LocalCallOf<T> = <T as Config>::RuntimeCall;

pub type BoundedCallOf<T> = Bounded<LocalCallOf<T>, <T as frame_system::Config>::Hashing>;

pub type PalletsOriginOf<T> =
    <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

pub type ScheduleAddressOf<T> =
    <T as ScheduleNamed<BlockNumberFor<T>, LocalCallOf<T>, PalletsOriginOf<T>>>::Address;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[freeze_struct("7b322ade3ccaaba")]
pub struct TriumvirateVotes<AccountId, BlockNumber> {
    /// The proposal's unique index.
    index: ProposalIndex,
    /// The set of triumvirate members that approved it.
    ayes: BoundedVec<AccountId, ConstU32<TRIUMVIRATE_SIZE>>,
    /// The set of triumvirate members that rejected it.
    nays: BoundedVec<AccountId, ConstU32<TRIUMVIRATE_SIZE>>,
    /// The hard end time of this vote.
    end: BlockNumber,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[freeze_struct("68b000ed325d45c4")]
pub struct CollectiveVotes<AccountId, BlockNumber> {
    /// The proposal's unique index.
    index: ProposalIndex,
    /// The set of collective members that approved it.
    ayes: BoundedVec<AccountId, ConstU32<TOTAL_COLLECTIVES_SIZE>>,
    /// The set of collective members that rejected it.
    nays: BoundedVec<AccountId, ConstU32<TOTAL_COLLECTIVES_SIZE>>,
    /// The initial dispatch time of the proposal.
    initial_dispatch_time: BlockNumber,
    /// The additional delay applied to the proposal on top of the initial delay.
    delay: BlockNumber,
}

#[derive(
    PartialEq,
    Eq,
    Clone,
    Encode,
    Decode,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    DecodeWithMemTracking,
)]
pub enum CollectiveMember<AccountId> {
    Economic(AccountId),
    Building(AccountId),
}

pub trait CollectiveMembersProvider<T: Config> {
    fn get_economic_collective() -> BoundedVec<T::AccountId, ConstU32<ECONOMIC_COLLECTIVE_SIZE>>;
    fn get_building_collective() -> BoundedVec<T::AccountId, ConstU32<BUILDING_COLLECTIVE_SIZE>>;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        // /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// The currency mechanism.
        type Currency: fungible::Balanced<Self::AccountId, Balance = u64>
            + fungible::Mutate<Self::AccountId>;

        /// The preimage provider which will be used to store the call to dispatch.
        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

        /// The scheduler which will be used to schedule the proposal for execution.
        type Scheduler: ScheduleNamed<
                BlockNumberFor<Self>,
                LocalCallOf<Self>,
                PalletsOriginOf<Self>,
                Hasher = Self::Hashing,
            >;

        /// Origin allowed to set allowed proposers.
        type SetAllowedProposersOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Origin allowed to set triumvirate.
        type SetTriumvirateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The collective members provider.
        type CollectiveMembersProvider: CollectiveMembersProvider<Self>;

        /// How many accounts allowed to submit proposals.
        #[pallet::constant]
        type MaxAllowedProposers: Get<u32>;

        /// Maximum weight for a proposal.
        #[pallet::constant]
        type MaxProposalWeight: Get<Weight>;

        /// Maximum number of proposals allowed to be active in parallel.
        #[pallet::constant]
        type MaxProposals: Get<u32>;

        /// Maximum number of proposals that can be scheduled for execution in parallel.
        #[pallet::constant]
        type MaxScheduled: Get<u32>;

        /// The duration of a motion.
        #[pallet::constant]
        type MotionDuration: Get<BlockNumberFor<Self>>;

        /// Initial scheduling delay for proposal execution.
        #[pallet::constant]
        type InitialSchedulingDelay: Get<BlockNumberFor<Self>>;

        /// Period of time between collective rotations.
        #[pallet::constant]
        type CollectiveRotationPeriod: Get<BlockNumberFor<Self>>;

        /// Period of time between cleanup of proposals and scheduled proposals.
        #[pallet::constant]
        type CleanupPeriod: Get<BlockNumberFor<Self>>;

        /// Percent threshold for a proposal to be cancelled by a collective vote.
        #[pallet::constant]
        type CancellationThreshold: Get<Percent>;

        /// Percent threshold for a proposal to be fast-tracked by a collective vote.
        #[pallet::constant]
        type FastTrackThreshold: Get<Percent>;
    }

    /// Accounts allowed to submit proposals.
    #[pallet::storage]
    pub type AllowedProposers<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxAllowedProposers>, ValueQuery>;

    /// Active members of the triumvirate.
    #[pallet::storage]
    pub type Triumvirate<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, ConstU32<TRIUMVIRATE_SIZE>>, ValueQuery>;

    #[pallet::storage]
    pub type ProposalCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Tuples of account proposer and hash of the active proposals being voted on.
    #[pallet::storage]
    pub type Proposals<T: Config> =
        StorageValue<_, BoundedVec<(T::AccountId, T::Hash), T::MaxProposals>, ValueQuery>;

    /// Actual proposal for a given hash.
    #[pallet::storage]
    pub type ProposalOf<T: Config> =
        StorageMap<_, Identity, T::Hash, BoundedCallOf<T>, OptionQuery>;

    /// Triumvirate votes for a given proposal, if it is ongoing.
    #[pallet::storage]
    pub type TriumvirateVoting<T: Config> = StorageMap<
        _,
        Identity,
        T::Hash,
        TriumvirateVotes<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// The hashes of the proposals that have been scheduled for execution.
    #[pallet::storage]
    pub type Scheduled<T: Config> =
        StorageValue<_, BoundedVec<T::Hash, T::MaxScheduled>, ValueQuery>;

    /// The economic collective members (top 20 validators by total stake).
    #[pallet::storage]
    pub type EconomicCollective<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, ConstU32<ECONOMIC_COLLECTIVE_SIZE>>, ValueQuery>;

    /// The building collective members (top 20 subnet owners by moving average price).
    #[pallet::storage]
    pub type BuildingCollective<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, ConstU32<BUILDING_COLLECTIVE_SIZE>>, ValueQuery>;

    /// Collectives votes for a given proposal, if it is scheduled.
    #[pallet::storage]
    pub type CollectiveVoting<T: Config> = StorageMap<
        _,
        Identity,
        T::Hash,
        CollectiveVotes<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub allowed_proposers: Vec<T::AccountId>,
        pub triumvirate: Vec<T::AccountId>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let allowed_proposers_set = Pallet::<T>::check_for_duplicates(&self.allowed_proposers)
                .expect("Allowed proposers cannot contain duplicate accounts.");
            assert!(
                self.allowed_proposers.len() <= T::MaxAllowedProposers::get() as usize,
                "Allowed proposers length cannot exceed MaxAllowedProposers."
            );

            let triumvirate_set = Pallet::<T>::check_for_duplicates(&self.triumvirate)
                .expect("Triumvirate cannot contain duplicate accounts.");
            assert!(
                self.triumvirate.len() <= TRIUMVIRATE_SIZE as usize,
                "Triumvirate length cannot exceed {TRIUMVIRATE_SIZE}."
            );

            assert!(
                allowed_proposers_set.is_disjoint(&triumvirate_set),
                "Allowed proposers and triumvirate must be disjoint."
            );

            Pallet::<T>::initialize_allowed_proposers(&self.allowed_proposers);
            Pallet::<T>::initialize_triumvirate(&self.triumvirate);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The allowed proposers have been set.
        AllowedProposersSet {
            incoming: Vec<T::AccountId>,
            outgoing: Vec<T::AccountId>,
            removed_proposals: Vec<(T::AccountId, T::Hash)>,
        },
        /// The triumvirate has been set.
        TriumvirateSet {
            incoming: Vec<T::AccountId>,
            outgoing: Vec<T::AccountId>,
        },
        /// A proposal has been submitted.
        ProposalSubmitted {
            account: T::AccountId,
            proposal_index: u32,
            proposal_hash: T::Hash,
            voting_end: BlockNumberFor<T>,
        },
        /// A triumvirate member has voted on a proposal.
        TriumvirateMemberVoted {
            account: T::AccountId,
            proposal_hash: T::Hash,
            voted: bool,
            yes: u32,
            no: u32,
        },
        /// A collective member has voted on a proposal.
        CollectiveMemberVoted {
            account: T::AccountId,
            proposal_hash: T::Hash,
            voted: bool,
            yes: u32,
            no: u32,
        },
        /// A proposal has been scheduled for execution by triumvirate.
        ProposalScheduled { proposal_hash: T::Hash },
        /// A proposal has been cancelled by triumvirate.
        ProposalCancelled { proposal_hash: T::Hash },
        /// A scheduled proposal has been fast-tracked by collectives.
        ScheduledProposalFastTracked { proposal_hash: T::Hash },
        /// A scheduled proposal has been cancelled by collectives.
        ScheduledProposalCancelled { proposal_hash: T::Hash },
        /// A scheduled proposal schedule time has been delayed by collectives.
        ScheduledProposalDelayAdjusted {
            proposal_hash: T::Hash,
            dispatch_time: DispatchTime<BlockNumberFor<T>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Duplicate accounts not allowed.
        DuplicateAccounts,
        /// There can only be a maximum of `MaxAllowedProposers` allowed proposers.
        TooManyAllowedProposers,
        /// Triumvirate length cannot exceed 3.
        InvalidTriumvirateLength,
        /// Allowed proposers and triumvirate must be disjoint.
        AllowedProposersAndTriumvirateMustBeDisjoint,
        /// Origin is not an allowed proposer.
        NotAllowedProposer,
        /// The given weight bound for the proposal was too low.
        WrongProposalLength,
        /// The given weight bound for the proposal was too low.
        WrongProposalWeight,
        /// Duplicate proposals not allowed.
        DuplicateProposal,
        /// There can only be a maximum of `MaxProposals` active proposals in parallel.
        TooManyProposals,
        /// Origin is not a triumvirate member.
        NotTriumvirateMember,
        /// Proposal must exist.
        ProposalMissing,
        /// Mismatched index.
        WrongProposalIndex,
        /// Duplicate vote not allowed.
        DuplicateVote,
        /// There can only be a maximum of `MaxScheduled` proposals scheduled for execution.
        TooManyScheduled,
        /// There can only be a maximum of 3 votes for a proposal.
        TooManyVotes,
        /// Call is not available in the preimage storage.
        CallUnavailable,
        /// Proposal hash is not 32 bytes.
        InvalidProposalHashLength,
        /// Proposal is already scheduled.
        AlreadyScheduled,
        /// Origin is not a collective member.
        NotCollectiveMember,
        /// Proposal is not scheduled.
        ProposalNotScheduled,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();

            let economic_collective = EconomicCollective::<T>::get();
            let building_collective = BuildingCollective::<T>::get();
            let is_first_run = economic_collective.is_empty() || building_collective.is_empty();
            let should_rotate = now % T::CollectiveRotationPeriod::get() == Zero::zero();
            let should_cleanup = now % T::CleanupPeriod::get() == Zero::zero();

            if is_first_run || should_rotate {
                weight.saturating_accrue(Self::do_rotate_collectives());
            }

            if should_cleanup {
                weight.saturating_accrue(Self::do_cleanup_proposals(now));
                weight.saturating_accrue(Self::do_cleanup_scheduled());
            }

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the allowed proposers.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::zero())]
        pub fn set_allowed_proposers(
            origin: OriginFor<T>,
            mut new_allowed_proposers: BoundedVec<T::AccountId, T::MaxAllowedProposers>,
        ) -> DispatchResult {
            T::SetAllowedProposersOrigin::ensure_origin(origin)?;

            let new_allowed_proposers_set =
                Pallet::<T>::check_for_duplicates(&new_allowed_proposers)
                    .ok_or(Error::<T>::DuplicateAccounts)?;

            let triumvirate = Triumvirate::<T>::get();
            let triumvirate_set: BTreeSet<_> = triumvirate.iter().collect();
            ensure!(
                triumvirate_set.is_disjoint(&new_allowed_proposers_set),
                Error::<T>::AllowedProposersAndTriumvirateMustBeDisjoint
            );

            let mut allowed_proposers = AllowedProposers::<T>::get().to_vec();
            allowed_proposers.sort();
            new_allowed_proposers.sort();
            let (incoming, outgoing) =
                <() as ChangeMembers<T::AccountId>>::compute_members_diff_sorted(
                    &new_allowed_proposers.to_vec(),
                    &allowed_proposers,
                );

            // Remove proposals from the outgoing allowed proposers.
            let mut removed_proposals = vec![];
            for (proposer, proposal_hash) in Proposals::<T>::get() {
                if outgoing.contains(&proposer) {
                    Self::clear_proposal(proposal_hash);
                    removed_proposals.push((proposer, proposal_hash));
                }
            }

            AllowedProposers::<T>::put(new_allowed_proposers);

            Self::deposit_event(Event::<T>::AllowedProposersSet {
                incoming,
                outgoing,
                removed_proposals,
            });
            Ok(())
        }

        /// Set the triumvirate.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::zero())]
        pub fn set_triumvirate(
            origin: OriginFor<T>,
            mut new_triumvirate: BoundedVec<T::AccountId, ConstU32<TRIUMVIRATE_SIZE>>,
        ) -> DispatchResult {
            T::SetTriumvirateOrigin::ensure_origin(origin)?;

            let new_triumvirate_set = Pallet::<T>::check_for_duplicates(&new_triumvirate)
                .ok_or(Error::<T>::DuplicateAccounts)?;
            ensure!(
                new_triumvirate.len() == TRIUMVIRATE_SIZE as usize,
                Error::<T>::InvalidTriumvirateLength
            );

            let allowed_proposers = AllowedProposers::<T>::get();
            let allowed_proposers_set: BTreeSet<_> = allowed_proposers.iter().collect();
            ensure!(
                allowed_proposers_set.is_disjoint(&new_triumvirate_set),
                Error::<T>::AllowedProposersAndTriumvirateMustBeDisjoint
            );

            let mut triumvirate = Triumvirate::<T>::get().to_vec();
            triumvirate.sort();
            new_triumvirate.sort();
            let (incoming, outgoing) =
                <() as ChangeMembers<T::AccountId>>::compute_members_diff_sorted(
                    &new_triumvirate.to_vec(),
                    &triumvirate,
                );

            // Remove votes from the outgoing triumvirate members.
            for (_proposer, proposal_hash) in Proposals::<T>::get() {
                TriumvirateVoting::<T>::mutate(proposal_hash, |voting| {
                    if let Some(voting) = voting.as_mut() {
                        voting.ayes.retain(|a| !outgoing.contains(a));
                        voting.nays.retain(|a| !outgoing.contains(a));
                    }
                });
            }

            Triumvirate::<T>::put(new_triumvirate);

            Self::deposit_event(Event::<T>::TriumvirateSet { incoming, outgoing });
            Ok(())
        }

        /// Propose a new proposal.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::zero())]
        pub fn propose(
            origin: OriginFor<T>,
            proposal: Box<<T as Config>::RuntimeCall>,
            #[pallet::compact] length_bound: u32,
        ) -> DispatchResult {
            let who = Self::ensure_allowed_proposer(origin)?;

            let proposal_len = proposal.encoded_size();
            ensure!(
                proposal_len <= length_bound as usize,
                Error::<T>::WrongProposalLength
            );
            let proposal_weight = proposal.get_dispatch_info().call_weight;
            ensure!(
                proposal_weight.all_lte(T::MaxProposalWeight::get()),
                Error::<T>::WrongProposalWeight
            );

            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(
                !ProposalOf::<T>::contains_key(proposal_hash),
                Error::<T>::DuplicateProposal
            );
            let scheduled = Scheduled::<T>::get();
            ensure!(
                !scheduled.contains(&proposal_hash),
                Error::<T>::AlreadyScheduled
            );

            Proposals::<T>::try_append((who.clone(), proposal_hash))
                .map_err(|_| Error::<T>::TooManyProposals)?;

            let proposal_index = ProposalCount::<T>::get();
            ProposalCount::<T>::mutate(|i| i.saturating_inc());

            let bounded_proposal = T::Preimages::bound(*proposal)?;
            ProposalOf::<T>::insert(proposal_hash, bounded_proposal);

            let now = frame_system::Pallet::<T>::block_number();
            let end = now + T::MotionDuration::get();
            TriumvirateVoting::<T>::insert(
                proposal_hash,
                TriumvirateVotes {
                    index: proposal_index,
                    ayes: BoundedVec::new(),
                    nays: BoundedVec::new(),
                    end,
                },
            );

            Self::deposit_event(Event::<T>::ProposalSubmitted {
                account: who,
                proposal_index,
                proposal_hash,
                voting_end: end,
            });
            Ok(())
        }

        /// Vote on a proposal as a triumvirate member.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::zero())]
        pub fn vote_on_proposed(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            #[pallet::compact] proposal_index: ProposalIndex,
            approve: bool,
        ) -> DispatchResult {
            let who = Self::ensure_triumvirate_member(origin)?;

            let proposals = Proposals::<T>::get();
            ensure!(
                proposals.iter().any(|(_, h)| h == &proposal_hash),
                Error::<T>::ProposalMissing
            );

            let voting = Self::do_vote_on_proposed(&who, proposal_hash, proposal_index, approve)?;

            let yes_votes = voting.ayes.len() as u32;
            let no_votes = voting.nays.len() as u32;

            Self::deposit_event(Event::<T>::TriumvirateMemberVoted {
                account: who,
                proposal_hash,
                voted: approve,
                yes: yes_votes,
                no: no_votes,
            });

            if yes_votes >= 2 {
                Self::do_schedule(proposal_hash, proposal_index)?;
            } else if no_votes >= 2 {
                Self::do_cancel(proposal_hash)?;
            }

            Ok(())
        }

        /// Vote on a proposal as a collective member.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::zero())]
        pub fn vote_on_scheduled(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            #[pallet::compact] proposal_index: ProposalIndex,
            approve: bool,
        ) -> DispatchResult {
            let who = Self::ensure_collective_member(origin)?;

            let scheduled = Scheduled::<T>::get();
            ensure!(
                scheduled.contains(&proposal_hash),
                Error::<T>::ProposalNotScheduled
            );

            let voting = Self::do_vote_on_scheduled(&who, proposal_hash, proposal_index, approve)?;

            let yes_votes = voting.ayes.len() as u32;
            let no_votes = voting.nays.len() as u32;

            Self::deposit_event(Event::<T>::CollectiveMemberVoted {
                account: who,
                proposal_hash,
                voted: approve,
                yes: yes_votes,
                no: no_votes,
            });

            let should_fast_track =
                yes_votes >= T::FastTrackThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE) as u32;
            let should_cancel =
                no_votes >= T::CancellationThreshold::get().mul_ceil(TOTAL_COLLECTIVES_SIZE) as u32;

            if should_fast_track {
                Self::do_fast_track(proposal_hash)?;
            } else if should_cancel {
                Self::do_cancel_scheduled(proposal_hash)?;
            } else {
                Self::do_adjust_delay(proposal_hash, voting)?;
            }

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn initialize_allowed_proposers(allowed_proposers: &[T::AccountId]) {
        if !allowed_proposers.is_empty() {
            assert!(
                AllowedProposers::<T>::get().is_empty(),
                "Allowed proposers are already initialized!"
            );
            let mut allowed_proposers = BoundedVec::truncate_from(allowed_proposers.to_vec());
            allowed_proposers.sort();
            AllowedProposers::<T>::put(allowed_proposers);
        }
    }

    fn initialize_triumvirate(triumvirate: &[T::AccountId]) {
        assert!(
            Triumvirate::<T>::get().is_empty(),
            "Triumvirate is already initialized!"
        );
        let mut triumvirate = BoundedVec::truncate_from(triumvirate.to_vec());
        triumvirate.sort();
        Triumvirate::<T>::put(triumvirate);
    }

    fn check_for_duplicates(accounts: &[T::AccountId]) -> Option<BTreeSet<&T::AccountId>> {
        let accounts_set: BTreeSet<_> = accounts.iter().collect();
        if accounts_set.len() == accounts.len() {
            Some(accounts_set)
        } else {
            None
        }
    }

    fn do_vote_on_proposed(
        who: &T::AccountId,
        proposal_hash: T::Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> Result<TriumvirateVotes<T::AccountId, BlockNumberFor<T>>, DispatchError> {
        TriumvirateVoting::<T>::try_mutate(proposal_hash, |voting| {
            let voting = voting.as_mut().ok_or(Error::<T>::ProposalMissing)?;
            ensure!(voting.index == index, Error::<T>::WrongProposalIndex);
            Self::do_vote_inner(&who, approve, &mut voting.ayes, &mut voting.nays)?;
            Ok(voting.clone())
        })
    }

    fn do_vote_on_scheduled(
        who: &T::AccountId,
        proposal_hash: T::Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> Result<CollectiveVotes<T::AccountId, BlockNumberFor<T>>, DispatchError> {
        CollectiveVoting::<T>::try_mutate(proposal_hash, |voting| {
            let voting = voting.as_mut().ok_or(Error::<T>::ProposalNotScheduled)?;
            ensure!(voting.index == index, Error::<T>::WrongProposalIndex);
            Self::do_vote_inner(&who, approve, &mut voting.ayes, &mut voting.nays)?;
            Ok(voting.clone())
        })
    }

    fn do_vote_inner<N: Get<u32>>(
        who: &T::AccountId,
        approve: bool,
        ayes: &mut BoundedVec<T::AccountId, N>,
        nays: &mut BoundedVec<T::AccountId, N>,
    ) -> DispatchResult {
        let has_yes_vote = ayes.iter().any(|a| a == who);
        let has_no_vote = nays.iter().any(|a| a == who);

        if approve {
            if !has_yes_vote {
                ayes.try_push(who.clone())
                    // Unreachable because nobody can double vote.
                    .map_err(|_| Error::<T>::TooManyVotes)?;
            } else {
                return Err(Error::<T>::DuplicateVote.into());
            }
            if has_no_vote {
                nays.retain(|a| a != who);
            }
        } else {
            if !has_no_vote {
                nays.try_push(who.clone())
                    // Unreachable because nobody can double vote.
                    .map_err(|_| Error::<T>::TooManyVotes)?;
            } else {
                return Err(Error::<T>::DuplicateVote.into());
            }
            if has_yes_vote {
                ayes.retain(|a| a != who);
            }
        }

        Ok(())
    }

    fn do_schedule(proposal_hash: T::Hash, proposal_index: ProposalIndex) -> DispatchResult {
        Scheduled::<T>::try_append(proposal_hash).map_err(|_| Error::<T>::TooManyScheduled)?;

        let bounded = ProposalOf::<T>::get(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
        ensure!(T::Preimages::have(&bounded), Error::<T>::CallUnavailable);

        let now = frame_system::Pallet::<T>::block_number();
        let name = Self::task_name_from_hash(proposal_hash)?;
        let dispatch_time = now + T::InitialSchedulingDelay::get();
        T::Scheduler::schedule_named(
            name,
            DispatchTime::At(dispatch_time),
            None,
            Priority::default(),
            RawOrigin::Root.into(),
            bounded,
        )?;
        Self::clear_proposal(proposal_hash);

        CollectiveVoting::<T>::insert(
            proposal_hash,
            CollectiveVotes {
                index: proposal_index,
                ayes: BoundedVec::new(),
                nays: BoundedVec::new(),
                initial_dispatch_time: dispatch_time,
                delay: Zero::zero(),
            },
        );

        Self::deposit_event(Event::<T>::ProposalScheduled { proposal_hash });
        Ok(())
    }

    fn do_cancel(proposal_hash: T::Hash) -> DispatchResult {
        Self::clear_proposal(proposal_hash);
        Self::deposit_event(Event::<T>::ProposalCancelled { proposal_hash });
        Ok(())
    }

    fn do_fast_track(proposal_hash: T::Hash) -> DispatchResult {
        let name = Self::task_name_from_hash(proposal_hash)?;
        T::Scheduler::reschedule_named(
            name,
            // It will be scheduled on the next block because scheduler already ran for this block.
            DispatchTime::After(Zero::zero()),
        )?;
        CollectiveVoting::<T>::remove(&proposal_hash);
        Self::deposit_event(Event::<T>::ScheduledProposalFastTracked { proposal_hash });
        Ok(())
    }

    fn do_cancel_scheduled(proposal_hash: T::Hash) -> DispatchResult {
        let name = Self::task_name_from_hash(proposal_hash)?;
        T::Scheduler::cancel_named(name)?;
        Self::clear_scheduled_proposal(proposal_hash);
        Self::deposit_event(Event::<T>::ScheduledProposalCancelled { proposal_hash });
        Ok(())
    }

    fn do_adjust_delay(
        proposal_hash: T::Hash,
        mut voting: CollectiveVotes<T::AccountId, BlockNumberFor<T>>,
    ) -> DispatchResult {
        let net_score = voting.nays.len() as i32 - voting.ayes.len() as i32;

        let additional_delay = Self::compute_additional_delay(net_score);

        // No change, no need to reschedule
        if voting.delay == additional_delay {
            return Ok(());
        }

        let now = frame_system::Pallet::<T>::block_number();
        let elapsed_time = now.saturating_sub(voting.initial_dispatch_time);

        // We are past new delay, fast track
        if elapsed_time > additional_delay {
            return Self::do_fast_track(proposal_hash);
        }

        let name = Self::task_name_from_hash(proposal_hash)?;
        let dispatch_time = DispatchTime::At(voting.initial_dispatch_time + additional_delay);
        T::Scheduler::reschedule_named(name, dispatch_time)?;

        voting.delay = additional_delay;
        CollectiveVoting::<T>::insert(proposal_hash, voting);

        Self::deposit_event(Event::<T>::ScheduledProposalDelayAdjusted {
            proposal_hash,
            dispatch_time,
        });
        Ok(())
    }

    fn clear_proposal(proposal_hash: T::Hash) {
        Proposals::<T>::mutate(|proposals| {
            proposals.retain(|(_, h)| h != &proposal_hash);
        });
        ProposalOf::<T>::remove(&proposal_hash);
        TriumvirateVoting::<T>::remove(&proposal_hash);
    }

    fn clear_scheduled_proposal(proposal_hash: T::Hash) {
        Scheduled::<T>::mutate(|scheduled| {
            scheduled.retain(|h| h != &proposal_hash);
        });
        CollectiveVoting::<T>::remove(&proposal_hash);
    }

    fn do_rotate_collectives() -> Weight {
        let mut weight = Weight::zero();

        let economic_collective_members = T::CollectiveMembersProvider::get_economic_collective();
        let building_collective_members = T::CollectiveMembersProvider::get_building_collective();
        // TODO: handle weights

        EconomicCollective::<T>::put(economic_collective_members);
        BuildingCollective::<T>::put(building_collective_members);
        weight.saturating_accrue(T::DbWeight::get().writes(2));

        weight
    }

    fn do_cleanup_proposals(now: BlockNumberFor<T>) -> Weight {
        let mut weight = Weight::zero();

        let mut proposals = Proposals::<T>::get();
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        proposals.retain(|(_, proposal_hash)| {
            let voting = TriumvirateVoting::<T>::get(proposal_hash);
            weight.saturating_accrue(T::DbWeight::get().reads(1));

            match voting {
                Some(voting) if voting.end > now => true,
                _ => {
                    ProposalOf::<T>::remove(proposal_hash);
                    TriumvirateVoting::<T>::remove(proposal_hash);
                    weight.saturating_accrue(T::DbWeight::get().writes(2));
                    false
                }
            }
        });

        Proposals::<T>::put(proposals);
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    }

    fn do_cleanup_scheduled() -> Weight {
        let mut weight = Weight::zero();

        let mut scheduled = Scheduled::<T>::get();
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        scheduled.retain(
            |proposal_hash| match Self::task_name_from_hash(*proposal_hash) {
                Ok(name) => {
                    let dispatch_time = T::Scheduler::next_dispatch_time(name);
                    CollectiveVoting::<T>::remove(proposal_hash);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
                    dispatch_time.is_ok()
                }
                // Unreachable because proposal hash is always 32 bytes.
                Err(_) => false,
            },
        );

        Scheduled::<T>::put(scheduled);
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    }

    fn ensure_allowed_proposer(origin: OriginFor<T>) -> Result<T::AccountId, DispatchError> {
        let who = ensure_signed(origin)?;
        let allowed_proposers = AllowedProposers::<T>::get();
        ensure!(
            allowed_proposers.contains(&who),
            Error::<T>::NotAllowedProposer
        );
        Ok(who)
    }

    fn ensure_triumvirate_member(origin: OriginFor<T>) -> Result<T::AccountId, DispatchError> {
        let who = ensure_signed(origin)?;
        let triumvirate = Triumvirate::<T>::get();
        ensure!(triumvirate.contains(&who), Error::<T>::NotTriumvirateMember);
        Ok(who)
    }

    fn ensure_collective_member(origin: OriginFor<T>) -> Result<T::AccountId, DispatchError> {
        let who = ensure_signed(origin)?;
        let economic_collective = EconomicCollective::<T>::get();
        let building_collective = BuildingCollective::<T>::get();

        if economic_collective.contains(&who) || building_collective.contains(&who) {
            Ok(who)
        } else {
            Err(Error::<T>::NotCollectiveMember.into())
        }
    }

    fn task_name_from_hash(proposal_hash: T::Hash) -> Result<TaskName, DispatchError> {
        Ok(proposal_hash
            .as_ref()
            .try_into()
            .map_err(|_| Error::<T>::InvalidProposalHashLength)?)
    }

    fn compute_additional_delay(net_score: i32) -> BlockNumberFor<T> {
        if net_score > 0 {
            let initial_delay = T::InitialSchedulingDelay::get().into().as_u64() as f64;
            let multiplier = 1.5_f64.powi(net_score.abs());
            ((initial_delay * multiplier).ceil() as u32).into()
        } else {
            Zero::zero()
        }
    }
}
