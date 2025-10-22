#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{
    dispatch::GetDispatchInfo,
    pallet_prelude::*,
    sp_runtime::traits::Dispatchable,
    traits::{IsSubType, fungible},
};

mod mock;
mod tests;
pub use pallet::*;

pub type CurrencyOf<T> = <T as Config>::Currency;

pub type BalanceOf<T> =
    <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

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

        /// How many accounts allowed to submit proposals.
        #[pallet::constant]
        type MaxAllowedProposers: Get<u32>;
    }

    const TRIUMVIRATE_SIZE: u32 = 3;

    /// Accounts allowed to submit proposals.
    #[pallet::storage]
    pub type AllowedProposers<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxAllowedProposers>, ValueQuery>;

    // Active members of the triumvirate.
    #[pallet::storage]
    pub type Triumvirate<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, ConstU32<TRIUMVIRATE_SIZE>>, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub allowed_proposers: Vec<T::AccountId>,
        pub triumvirate: Vec<T::AccountId>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            use alloc::collections::btree_set::BTreeSet;

            let allowed_proposers_set: BTreeSet<_> = self.allowed_proposers.iter().collect();
            assert_eq!(
                allowed_proposers_set.len(),
                self.allowed_proposers.len(),
                "Allowed proposers cannot contain duplicate accounts."
            );
            assert!(
                self.allowed_proposers.len() <= T::MaxAllowedProposers::get() as usize,
                "Allowed proposers length cannot exceed MaxAllowedProposers."
            );

            let triumvirate_set: BTreeSet<_> = self.triumvirate.iter().collect();
            assert_eq!(
                triumvirate_set.len(),
                self.triumvirate.len(),
                "Triumvirate cannot contain duplicate accounts."
            );
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

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
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
}
