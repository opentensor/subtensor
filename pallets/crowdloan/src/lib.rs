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
use sp_runtime::traits::{AccountIdConversion, CheckedAdd, Zero};

type CrowdloanIndex = u32;

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
        CrowdloanIndex,
        CrowdloanInfo<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type NextCrowdloanIndex<T> = StorageValue<_, CrowdloanIndex, ValueQuery, ConstU32<0>>;

    #[pallet::storage]
    pub type Contributions<T: Config> = StorageDoubleMap<
        _,
        Identity,
        CrowdloanIndex,
        Identity,
        T::AccountId,
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created {
            crowdloan_id: CrowdloanIndex,
            depositor: T::AccountId,
            end: BlockNumberFor<T>,
            cap: BalanceOf<T>,
        },
        Contributed {
            crowdloan_id: CrowdloanIndex,
            contributor: T::AccountId,
            amount: BalanceOf<T>,
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
        InvalidCrowdloanIndex,
        CapRaised,
        CapExceeded,
        ContributionPeriodEnded,
        ContributionTooLow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a crowdloan
        #[pallet::call_index(0)]
        pub fn create(
            origin: OriginFor<T>,
            #[pallet::compact] deposit: BalanceOf<T>,
            #[pallet::compact] minimum_contribution: BalanceOf<T>,
            #[pallet::compact] cap: BalanceOf<T>,
            #[pallet::compact] end: BlockNumberFor<T>,
        ) -> DispatchResult {
            let depositor = ensure_signed(origin)?;

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
            let now = frame_system::Pallet::<T>::block_number();
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

            let crowdloan_id = NextCrowdloanIndex::<T>::get();
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

            NextCrowdloanIndex::<T>::put(next_crowdloan_id);

            // Transfer the deposit to the crowdloan account
            frame_system::Pallet::<T>::inc_providers(&Self::crowdloan_account_id(crowdloan_id));
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
        pub fn contribute(
            origin: OriginFor<T>,
            #[pallet::compact] crowdloan_id: CrowdloanIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let contributor = ensure_signed(origin)?;

            // Ensure the crowdloan exists
            let mut crowdloan =
                Crowdloans::<T>::get(crowdloan_id).ok_or(Error::<T>::InvalidCrowdloanIndex)?;

            // Ensure the crowdloan has not ended
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(crowdloan.end > now, Error::<T>::ContributionPeriodEnded);

            // Ensure the cap has not been fully raised
            ensure!(crowdloan.raised < crowdloan.cap, Error::<T>::CapRaised);

            // Ensure the contribution is at least the minimum contribution
            ensure!(
                amount >= crowdloan.minimum_contribution,
                Error::<T>::ContributionTooLow
            );

            // Ensure the contribution does not overflow the actual raised amount
            // and it does not exceed the cap
            crowdloan.raised = crowdloan
                .raised
                .checked_add(&amount)
                .ok_or(Error::<T>::Overflow)?;
            ensure!(crowdloan.raised <= crowdloan.cap, Error::<T>::CapExceeded);

            // Ensure the contribution does not overflow the contributor's balance and update
            // the contribution
            let contribution = Contributions::<T>::get(&crowdloan_id, &contributor)
                .unwrap_or(Zero::zero())
                .checked_add(&amount)
                .ok_or(Error::<T>::Overflow)?;
            Contributions::<T>::insert(&crowdloan_id, &contributor, contribution);

            Crowdloans::<T>::insert(crowdloan_id, &crowdloan);

            Self::deposit_event(Event::<T>::Contributed {
                contributor,
                crowdloan_id,
                amount,
            });
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
    pub fn crowdloan_account_id(id: CrowdloanIndex) -> T::AccountId {
        T::PalletId::get().into_sub_account_truncating(id)
    }
}
