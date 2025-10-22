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
    }

    /// Accounts allowed to submit proposals.
    #[pallet::storage]
    pub type AllowedProposers<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, ConstU32<10>>, ValueQuery>;

    // Active members of the triumvirate.
    #[pallet::storage]
    pub type Triumvirate<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, ConstU32<3>>, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}
