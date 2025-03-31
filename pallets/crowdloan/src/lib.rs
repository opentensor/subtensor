#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    PalletId,
    dispatch::GetDispatchInfo,
    sp_runtime::RuntimeDebug,
    traits::{Currency, Get, IsSubType, ReservableCurrency, tokens::ExistenceRequirement},
};
use scale_info::TypeInfo;

pub use pallet::*;
use sp_runtime::traits::AccountIdConversion;

type CrowdloanId = u32;

mod tests;

type CurrencyOf<T> = <T as Config>::Currency;
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct CrowdloanInfo<AccountId, Balance, BlockNumber> {
    pub depositor: AccountId,
    pub deposit: Balance,
    pub minimum_contribution: Balance,
    pub end: BlockNumber,
    pub cap: Balance,
    pub raised: Balance,
}

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, sp_runtime::traits::Dispatchable};
    use frame_system::pallet_prelude::{BlockNumberFor, *};
    use sp_runtime::traits::CheckedSub;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        #[pallet::constant]
        type MinimumDeposit: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type AbsoluteMinimumContribution: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MinimumBlockDuration: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type MaximumBlockDuration: Get<BlockNumberFor<Self>>;
    }

    #[pallet::storage]
    pub type Crowdloans<T: Config> = StorageMap<
        _,
        Identity,
        CrowdloanId,
        CrowdloanInfo<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type NextCrowdloanId<T> = StorageValue<_, CrowdloanId, ValueQuery, ConstU32<0>>;

    #[pallet::storage]
    pub type Contributions<T: Config> = StorageDoubleMap<
        _,
        Identity,
        CrowdloanId,
        Identity,
        T::AccountId,
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created {
            crowdloan_id: CrowdloanId,
            depositor: T::AccountId,
            end: BlockNumberFor<T>,
            cap: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        DepositTooLow,
        CapTooLow,
        MinimumContributionTooLow,
        CannotEndInPast,
        BlockDurationTooShort,
        BlockDurationTooLong,
        InsufficientBalance,
        Overflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a crowdloan
        #[pallet::call_index(0)]
        pub fn create(
            origin: OriginFor<T>,
            deposit: BalanceOf<T>,
            minimum_contribution: BalanceOf<T>,
            cap: BalanceOf<T>,
            end: BlockNumberFor<T>,
        ) -> DispatchResult {
            let depositor = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();

            // Ensure the deposit is at least the minimum deposit and cap is greater
            // than the deposit
            ensure!(
                deposit >= T::MinimumDeposit::get(),
                Error::<T>::DepositTooLow
            );
            ensure!(cap > deposit, Error::<T>::CapTooLow);

            // Ensure the minimum contribution is at least the absolute minimum contribution
            ensure!(
                minimum_contribution >= T::AbsoluteMinimumContribution::get(),
                Error::<T>::MinimumContributionTooLow
            );

            // Ensure the end block is after the current block and the duration is
            // between the minimum and maximum block duration
            ensure!(end > now, Error::<T>::CannotEndInPast);
            let block_duration = end.checked_sub(&now).expect("checked end after now; qed");
            ensure!(
                block_duration >= T::MinimumBlockDuration::get(),
                Error::<T>::BlockDurationTooShort
            );
            ensure!(
                block_duration <= T::MaximumBlockDuration::get(),
                Error::<T>::BlockDurationTooLong
            );

            // Ensure the depositor has enough balance to pay the deposit.
            ensure!(
                T::Currency::free_balance(&depositor) >= deposit,
                Error::<T>::InsufficientBalance
            );

            let crowdloan_id = NextCrowdloanId::<T>::get();
            let next_crowdloan_id = crowdloan_id.checked_add(1).ok_or(Error::<T>::Overflow)?;

            Crowdloans::<T>::insert(
                &crowdloan_id,
                CrowdloanInfo {
                    depositor: depositor.clone(),
                    deposit,
                    minimum_contribution,
                    end,
                    cap,
                    raised: deposit,
                },
            );

            NextCrowdloanId::<T>::put(next_crowdloan_id);

            // Transfer the deposit to the crowdloan account
            T::Currency::transfer(
                &depositor,
                &Self::crowdloan_account_id(crowdloan_id),
                deposit,
                ExistenceRequirement::AllowDeath,
            )?;

            // Add initial deposit to contributions
            Contributions::<T>::insert(&crowdloan_id, &depositor, deposit);

            Self::deposit_event(Event::<T>::Created {
                crowdloan_id,
                depositor,
                end,
                cap,
            });

            Ok(())
        }

        /// Contribute to a crowdloan
        #[pallet::call_index(1)]
        pub fn contribute(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }

        /// Withdraw all contributior balance from a crowdloan
        #[pallet::call_index(2)]
        pub fn withdraw(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }

        /// Refund every contributor's balance from a crowdloan
        #[pallet::call_index(3)]
        pub fn refund(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }

        /// Remove fund after end and refunds
        #[pallet::call_index(4)]
        pub fn dissolve(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }

        /// Edit configuration
        #[pallet::call_index(5)]
        pub fn edit(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn crowdloan_account_id(id: CrowdloanId) -> T::AccountId {
        T::PalletId::get().into_sub_account_truncating(id)
    }
}
