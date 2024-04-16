#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod types;
pub mod weights;

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::traits::Currency;
use sp_runtime::traits::Zero;
use sp_std::boxed::Box;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    // Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Currency type that will be used to place deposits on neurons
        type Currency: ReservableCurrency<Self::AccountId> + Send + Sync;

        // Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        // Interface to allow other pallets to control who can register identities
        type CanRegister: crate::CanRegisterIdentity<Self::AccountId>;

        // Configuration fields
        /// Maximum user-configured additional fields
        #[pallet::constant]
        type MaxAdditionalFields: Get<u32>;

        /// The amount held on deposit for a registered identity
        #[pallet::constant]
        type InitialDeposit: Get<BalanceOf<Self>>;

        /// The amount held on deposit per additional field for a registered identity.
        #[pallet::constant]
        type FieldDeposit: Get<BalanceOf<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        IdentitySet { who: T::AccountId }, // Emitted when a user registers an identity
        IdentityDissolved { who: T::AccountId }, // Emitted when a user dissolves an identity
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account attempted to register an identity but doesn't meet the requirements.
        CannotRegister,
        /// Account passed too many additional fields to their identity
        TooManyFields,
        /// Account doesn't have a registered identity
        NotRegistered,
    }

    /// Identity data by account
    #[pallet::storage]
    #[pallet::getter(fn identity_of)]
    pub(super) type IdentityOf<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        Registration<BalanceOf<T>, T::MaxAdditionalFields>,
        OptionQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight((
			T::WeightInfo::set_identity(),
			DispatchClass::Operational
		))]
        pub fn set_identity(
            origin: OriginFor<T>,
            identified: T::AccountId,
            info: Box<IdentityInfo<T::MaxAdditionalFields>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                T::CanRegister::can_register(&who, &identified),
                Error::<T>::CannotRegister
            );

            let extra_fields = info.additional.len() as u32;
            ensure!(
                extra_fields <= T::MaxAdditionalFields::get(),
                Error::<T>::TooManyFields
            );

            let fd = <BalanceOf<T>>::from(extra_fields) * T::FieldDeposit::get();
            let mut id = match <IdentityOf<T>>::get(&identified) {
                Some(mut id) => {
                    id.info = *info;
                    id
                }
                None => Registration {
                    info: *info,
                    deposit: Zero::zero(),
                },
            };

            let old_deposit = id.deposit;
            id.deposit = T::InitialDeposit::get() + fd;
            if id.deposit > old_deposit {
                T::Currency::reserve(&who, id.deposit - old_deposit)?;
            }
            if old_deposit > id.deposit {
                let err_amount = T::Currency::unreserve(&who, old_deposit - id.deposit);
                debug_assert!(err_amount.is_zero());
            }

            <IdentityOf<T>>::insert(&identified, id);
            Self::deposit_event(Event::IdentitySet { who: identified });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::clear_identity())]
        pub fn clear_identity(
            origin: OriginFor<T>,
            identified: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                T::CanRegister::can_register(&who, &identified),
                Error::<T>::CannotRegister
            );

            let id = <IdentityOf<T>>::take(&identified).ok_or(Error::<T>::NotRegistered)?;
            let deposit = id.total_deposit();

            let err_amount = T::Currency::unreserve(&who, deposit);
            debug_assert!(err_amount.is_zero());

            Self::deposit_event(Event::IdentityDissolved { who: identified });

            Ok(().into())
        }
    }
}
// Interfaces to interact with other pallets
pub trait CanRegisterIdentity<AccountId> {
    fn can_register(who: &AccountId, identified: &AccountId) -> bool;
}

impl<A> CanRegisterIdentity<A> for () {
    fn can_register(_: &A, _: &A) -> bool {
        false
    }
}
