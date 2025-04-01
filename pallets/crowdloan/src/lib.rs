#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_support::{
    PalletId,
    dispatch::GetDispatchInfo,
    sp_runtime::RuntimeDebug,
    traits::{Currency, Get, IsSubType, ReservableCurrency, tokens::ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

pub use pallet::*;
use sp_runtime::traits::{AccountIdConversion, CheckedAdd, Zero};

type CrowdloanId = u32;

mod tests;

type CurrencyOf<T> = <T as Config>::Currency;
type BalanceOf<T> = <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct CrowdloanInfo<AccountId, Balance, BlockNumber> {
    pub depositor: AccountId,
    pub deposit: Balance,
    pub end: BlockNumber,
    pub cap: Balance,
    pub raised: Balance,
}

type CrowdloanInfoOf<T> =
    CrowdloanInfo<<T as frame_system::Config>::AccountId, BalanceOf<T>, BlockNumberFor<T>>;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;
    use frame_support::sp_runtime::traits::Dispatchable;
    use sp_runtime::traits::{CheckedSub, Saturating};

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
        type MinimumContribution: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MinimumBlockDuration: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type MaximumBlockDuration: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type RefundContributorsLimit: Get<u32>;
    }

    #[pallet::storage]
    pub type Crowdloans<T: Config> =
        StorageMap<_, Identity, CrowdloanId, CrowdloanInfoOf<T>, OptionQuery>;

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
        Contributed {
            crowdloan_id: CrowdloanId,
            contributor: T::AccountId,
            amount: BalanceOf<T>,
        },
        Withdrew {
            crowdloan_id: CrowdloanId,
            contributor: T::AccountId,
            amount: BalanceOf<T>,
        },
        PartiallyRefunded {
            crowdloan_id: CrowdloanId,
        },
        Refunded {
            crowdloan_id: CrowdloanId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        DepositTooLow,
        CapTooLow,
                CannotEndInPast,
        BlockDurationTooShort,
        BlockDurationTooLong,
        InsufficientBalance,
        Overflow,
        InvalidCrowdloanId,
        CapRaised,
        CapExceeded,
        ContributionPeriodEnded,
        ContributionTooLow,
        InvalidOrigin,
        AlreadyFinalized,
        ContributionPeriodNotEnded,
        NoContribution,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a crowdloan
        #[pallet::call_index(0)]
        pub fn create(
            origin: OriginFor<T>,
            #[pallet::compact] deposit: BalanceOf<T>,
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
                CurrencyOf::<T>::free_balance(&depositor) >= deposit,
                Error::<T>::InsufficientBalance
            );

            let crowdloan_id = NextCrowdloanId::<T>::get();
            let next_crowdloan_id = crowdloan_id.checked_add(1).ok_or(Error::<T>::Overflow)?;

            Crowdloans::<T>::insert(
                &crowdloan_id,
                CrowdloanInfo {
                    depositor: depositor.clone(),
                    deposit,
                                        end,
                    cap,
                    raised: deposit,
                },
            );

            NextCrowdloanId::<T>::put(next_crowdloan_id);

            // Transfer the deposit to the crowdloan account
            frame_system::Pallet::<T>::inc_providers(&Self::crowdloan_account_id(crowdloan_id));
            CurrencyOf::<T>::transfer(
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
            #[pallet::compact] crowdloan_id: CrowdloanId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let contributor = ensure_signed(origin)?;

            let mut crowdloan = Self::ensure_crowdloan_exists(crowdloan_id)?;

            // Ensure the crowdloan has not ended
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(crowdloan.end > now, Error::<T>::ContributionPeriodEnded);

            Self::ensure_crowdloan_has_not_fully_raised(&crowdloan)?;

            // Ensure the contribution is at least the minimum contribution
            ensure!(
                amount >= T::MinimumContribution::get(),
                Error::<T>::ContributionTooLow
            );

            // Ensure the contribution does not overflow the actual raised amount
            // and it does not exceed the cap
            crowdloan.raised = crowdloan
                .raised
                .checked_add(&amount)
                .ok_or(Error::<T>::Overflow)?;
            ensure!(crowdloan.raised <= crowdloan.cap, Error::<T>::CapExceeded);

            CurrencyOf::<T>::transfer(
                &contributor,
                &Self::crowdloan_account_id(crowdloan_id),
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

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

                #[pallet::call_index(3)]
        pub fn withdraw(
            origin: OriginFor<T>,
            contributor: T::AccountId,
            #[pallet::compact] crowdloan_id: CrowdloanId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut crowdloan = Self::ensure_crowdloan_exists(crowdloan_id)?;
Self::ensure_crowdloan_ended(&crowdloan)?;
            Self::ensure_crowdloan_has_not_fully_raised(&crowdloan)?;

            // Ensure the contributor has a contribution
            let amount = Contributions::<T>::get(&crowdloan_id, &contributor)
                .unwrap_or_else(|| Zero::zero());
            ensure!(amount > Zero::zero(), Error::<T>::NoContribution);

            CurrencyOf::<T>::transfer(
                &Self::crowdloan_account_id(crowdloan_id),
                &contributor,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Remove the contribution from the contributions map and update
            // tracked refunds so far
            Contributions::<T>::remove(&crowdloan_id, &contributor);
            crowdloan.raised = crowdloan.raised.saturating_sub(amount);

            Crowdloans::<T>::insert(crowdloan_id, &crowdloan);

            Self::deposit_event(Event::<T>::Withdrew {
                contributor,
                crowdloan_id,
                amount,
            });

            Ok(())
        }

                #[pallet::call_index(4)]
        pub fn refund(
origin: OriginFor<T>,
            #[pallet::compact] crowdloan_id: CrowdloanId,
) -> DispatchResult {
ensure_signed(origin)?;

            let mut crowdloan = Self::ensure_crowdloan_exists(crowdloan_id)?;
            let crowdloan_account = Self::crowdloan_account_id(crowdloan_id);
            Self::ensure_crowdloan_ended(&crowdloan)?;
            Self::ensure_crowdloan_has_not_fully_raised(&crowdloan)?;

            let mut refunded_contributors: Vec<T::AccountId> = vec![];
            let mut refund_count = 0;
            // Assume everyone can be refunded
            let mut all_refunded = true;
            let contributions = Contributions::<T>::iter_prefix(crowdloan_id);
            for (contributor, amount) in contributions {
                if refund_count >= T::RefundContributorsLimit::get() {
                    // Not everyone can be refunded
                    all_refunded = false;
                    break;
                }

                CurrencyOf::<T>::transfer(
                    &crowdloan_account,
                    &contributor,
                    amount,
                    ExistenceRequirement::AllowDeath,
                )?;
                refunded_contributors.push(contributor);
                crowdloan.raised = crowdloan.raised.saturating_sub(amount);
                refund_count += 1;
            }

            Crowdloans::<T>::insert(crowdloan_id, &crowdloan);

            // Clear refunded contributors
            for contributor in refunded_contributors {
                Contributions::<T>::remove(&crowdloan_id, &contributor);
            }

            if all_refunded {
                Self::deposit_event(Event::<T>::Refunded { crowdloan_id });
            } else {
                Self::deposit_event(Event::<T>::PartiallyRefunded { crowdloan_id });
            }

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
    fn crowdloan_account_id(id: CrowdloanId) -> T::AccountId {
        T::PalletId::get().into_sub_account_truncating(id)
    }

    fn ensure_crowdloan_exists(crowdloan_id: CrowdloanId) -> Result<CrowdloanInfoOf<T>, Error<T>> {
        Crowdloans::<T>::get(crowdloan_id).ok_or(Error::<T>::InvalidCrowdloanId)
    }

    fn ensure_crowdloan_ended(crowdloan: &CrowdloanInfoOf<T>) -> Result<(), Error<T>> {
        let now = frame_system::Pallet::<T>::block_number();
        ensure!(now >= crowdloan.end, Error::<T>::ContributionPeriodNotEnded);
        Ok(())
    }

    fn ensure_crowdloan_has_not_fully_raised(
        crowdloan: &CrowdloanInfoOf<T>,
    ) -> Result<(), Error<T>> {
        ensure!(crowdloan.raised < crowdloan.cap, Error::<T>::CapRaised);
        Ok(())
    }
}
