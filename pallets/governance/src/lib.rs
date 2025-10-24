#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    dispatch::GetDispatchInfo,
    pallet_prelude::*,
    sp_runtime::traits::Dispatchable,
    traits::{Bounded, ChangeMembers, IsSubType, QueryPreimage, StorePreimage, fungible},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{Saturating, traits::Hash};
use sp_std::collections::btree_set::BTreeSet;

mod mock;
mod tests;
pub use pallet::*;

pub type CurrencyOf<T> = <T as Config>::Currency;

pub type BalanceOf<T> =
    <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type BoundedCallOf<T> =
    Bounded<<T as Config>::RuntimeCall, <T as frame_system::Config>::Hashing>;

#[frame_support::pallet]
#[allow(clippy::expect_used)]
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
    }

    const TRIUMVIRATE_SIZE: u32 = 3;

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

    /// The hashes of the active proposals.
    #[pallet::storage]
    pub type Proposals<T: Config> =
        StorageValue<_, BoundedVec<T::Hash, T::MaxProposals>, ValueQuery>;

    /// Actual proposal for a given hash.
    #[pallet::storage]
    pub type ProposalOf<T: Config> =
        StorageMap<_, Identity, T::Hash, BoundedCallOf<T>, OptionQuery>;

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

            Self::deposit_event(Event::<T>::Proposed {
                account: who,
                proposal_index,
                proposal_hash,
            });
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
}
