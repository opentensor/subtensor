#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    dispatch::{GetDispatchInfo, RawOrigin},
    pallet_prelude::*,
    sp_runtime::traits::Dispatchable,
    traits::{
        Bounded, ChangeMembers, IsSubType, QueryPreimage, StorePreimage, fungible,
        schedule::{DispatchTime, Priority, v3::Named as ScheduleNamed},
    },
};
use frame_system::pallet_prelude::*;
use sp_runtime::{Saturating, traits::Hash};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};

mod mock;
mod tests;
pub use pallet::*;

const TRIUMVIRATE_SIZE: u32 = 3;

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
pub struct Votes<AccountId, BlockNumber> {
    /// The proposal's unique index.
    index: ProposalIndex,
    /// The set of triumvirate members that approved it.
    ayes: BoundedVec<AccountId, ConstU32<TRIUMVIRATE_SIZE>>,
    /// The set of triumvirate members that rejected it.
    nays: BoundedVec<AccountId, ConstU32<TRIUMVIRATE_SIZE>>,
    /// The hard end time of this vote.
    end: BlockNumber,
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

    /// The hashes of the active proposals being voted on.
    #[pallet::storage]
    pub type Proposals<T: Config> =
        StorageValue<_, BoundedVec<T::Hash, T::MaxProposals>, ValueQuery>;

    /// The hashes of the proposals that have been scheduled for execution.
    #[pallet::storage]
    pub type Scheduled<T: Config> =
        StorageValue<_, BoundedVec<T::Hash, T::MaxScheduled>, ValueQuery>;

    /// Actual proposal for a given hash.
    #[pallet::storage]
    pub type ProposalOf<T: Config> =
        StorageMap<_, Identity, T::Hash, BoundedCallOf<T>, OptionQuery>;

    /// Votes for a given proposal, if it is ongoing.
    #[pallet::storage]
    pub type Voting<T: Config> =
        StorageMap<_, Identity, T::Hash, Votes<T::AccountId, BlockNumberFor<T>>, OptionQuery>;

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
        TriumvirateSet,
        AllowedProposersSet,
        Proposed {
            account: T::AccountId,
            proposal_index: u32,
            proposal_hash: T::Hash,
            end: BlockNumberFor<T>,
        },
        Voted {
            account: T::AccountId,
            proposal_hash: T::Hash,
            voted: bool,
            yes: u32,
            no: u32,
        },
        Scheduled {
            proposal_hash: T::Hash,
        },
        Cancelled {
            proposal_hash: T::Hash,
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
            let (_incoming, _outgoing) =
                <() as ChangeMembers<T::AccountId>>::compute_members_diff_sorted(
                    &allowed_proposers,
                    &new_allowed_proposers.to_vec(),
                );

            // TODO: Cleanup proposals/votes from the allowed proposers.

            AllowedProposers::<T>::put(new_allowed_proposers);

            Self::deposit_event(Event::<T>::AllowedProposersSet);
            Ok(())
        }

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
            let (_incoming, _outgoing) =
                <() as ChangeMembers<T::AccountId>>::compute_members_diff_sorted(
                    &triumvirate,
                    &new_triumvirate.to_vec(),
                );

            // TODO: Cleanup proposals/votes from the triumvirate.

            Triumvirate::<T>::put(new_triumvirate);

            Self::deposit_event(Event::<T>::TriumvirateSet);
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::zero())]
        pub fn propose(
            origin: OriginFor<T>,
            proposal: Box<<T as Config>::RuntimeCall>,
            #[pallet::compact] length_bound: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let allowed_proposers = AllowedProposers::<T>::get();
            ensure!(
                allowed_proposers.contains(&who),
                Error::<T>::NotAllowedProposer
            );

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

            Proposals::<T>::try_append(proposal_hash).map_err(|_| Error::<T>::TooManyProposals)?;

            let proposal_index = ProposalCount::<T>::get();
            ProposalCount::<T>::mutate(|i| i.saturating_inc());

            let bounded_proposal = T::Preimages::bound(*proposal)?;
            ProposalOf::<T>::insert(proposal_hash, bounded_proposal);

            let now = frame_system::Pallet::<T>::block_number();
            let end = now + T::MotionDuration::get();
            Voting::<T>::insert(
                proposal_hash,
                Votes {
                    index: proposal_index,
                    ayes: BoundedVec::new(),
                    nays: BoundedVec::new(),
                    end,
                },
            );

            Self::deposit_event(Event::<T>::Proposed {
                account: who,
                proposal_index,
                proposal_hash,
                end,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::zero())]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            #[pallet::compact] proposal_index: ProposalIndex,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let triumvirate = Triumvirate::<T>::get();
            ensure!(triumvirate.contains(&who), Error::<T>::NotTriumvirateMember);

            let proposals = Proposals::<T>::get();
            ensure!(
                proposals.contains(&proposal_hash),
                Error::<T>::ProposalMissing
            );

            Self::do_vote(&who, proposal_hash, proposal_index, approve)?;

            let voting = Voting::<T>::get(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
            let yes_votes = voting.ayes.len() as u32;
            let no_votes = voting.nays.len() as u32;

            Self::deposit_event(Event::<T>::Voted {
                account: who,
                proposal_hash,
                voted: approve,
                yes: yes_votes,
                no: no_votes,
            });

            if yes_votes >= 2 {
                Self::do_schedule(proposal_hash)?;
            } else if no_votes >= 2 {
                Self::do_cancel(proposal_hash)?;
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

    fn do_vote(
        who: &T::AccountId,
        proposal_hash: T::Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> DispatchResult {
        Voting::<T>::try_mutate(proposal_hash, |voting| -> DispatchResult {
            let voting = voting.as_mut().ok_or(Error::<T>::ProposalMissing)?;
            ensure!(voting.index == index, Error::<T>::WrongProposalIndex);

            let has_yes_vote = voting.ayes.iter().any(|a| a == who);
            let has_no_vote = voting.nays.iter().any(|a| a == who);

            if approve {
                if !has_yes_vote {
                    voting
                        .ayes
                        .try_push(who.clone())
                        // Unreachable because nobody can double vote.
                        .map_err(|_| Error::<T>::TooManyVotes)?;
                } else {
                    return Err(Error::<T>::DuplicateVote.into());
                }
                if has_no_vote {
                    voting.nays.retain(|a| a != who);
                }
            } else {
                if !has_no_vote {
                    voting
                        .nays
                        .try_push(who.clone())
                        // Unreachable because nobody can double vote.
                        .map_err(|_| Error::<T>::TooManyVotes)?;
                } else {
                    return Err(Error::<T>::DuplicateVote.into());
                }
                if has_yes_vote {
                    voting.ayes.retain(|a| a != who);
                }
            }

            Ok(())
        })
    }

    fn do_schedule(proposal_hash: T::Hash) -> DispatchResult {
        Proposals::<T>::mutate(|proposals| {
            proposals.retain(|h| h != &proposal_hash);
        });
        Scheduled::<T>::try_append(proposal_hash).map_err(|_| Error::<T>::TooManyScheduled)?;

        let bounded = ProposalOf::<T>::get(proposal_hash).ok_or(Error::<T>::ProposalMissing)?;
        ensure!(T::Preimages::have(&bounded), Error::<T>::CallUnavailable);

        let now = frame_system::Pallet::<T>::block_number();
        T::Scheduler::schedule_named(
            proposal_hash
                .as_ref()
                .try_into()
                // Unreachable because we expect the hash to be 32 bytes.
                .map_err(|_| Error::<T>::InvalidProposalHashLength)?,
            DispatchTime::At(now + T::MotionDuration::get()),
            None,
            Priority::default(),
            RawOrigin::Root.into(),
            bounded,
        )?;

        Self::deposit_event(Event::<T>::Scheduled { proposal_hash });
        Ok(())
    }

    fn do_cancel(proposal_hash: T::Hash) -> DispatchResult {
        Proposals::<T>::mutate(|proposals| {
            proposals.retain(|h| h != &proposal_hash);
        });
        ProposalOf::<T>::remove(&proposal_hash);
        Voting::<T>::remove(&proposal_hash);

        Self::deposit_event(Event::<T>::Cancelled { proposal_hash });
        Ok(())
    }
}
