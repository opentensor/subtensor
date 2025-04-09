//! # Crowdloan Pallet
//!
//! A pallet allowing users to create generic crowdloans and contribute to them,
//! the raised funds are then transferred to a target address and an extrinsic
//! is dispatched, making it reusable for any crowdloan type.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use codec::{Decode, Encode};
use frame_support::{
    PalletId,
    dispatch::GetDispatchInfo,
    pallet_prelude::*,
    sp_runtime::{
        RuntimeDebug,
        traits::{AccountIdConversion, CheckedAdd, Dispatchable, Zero},
    },
    traits::{
        Bounded, Currency, Get, IsSubType, QueryPreimage, ReservableCurrency, StorePreimage,
        tokens::ExistenceRequirement,
    },
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedSub, Saturating};
use weights::WeightInfo;

pub use pallet::*;
use subtensor_macros::freeze_struct;

type CrowdloanId = u32;

mod benchmarking;
mod mock;
mod tests;
pub mod weights;

pub(crate) type CurrencyOf<T> = <T as Config>::Currency;

pub(crate) type BalanceOf<T> =
    <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type BoundedCallOf<T> =
    Bounded<<T as Config>::RuntimeCall, <T as frame_system::Config>::Hashing>;

/// A struct containing the information about a crowdloan.
#[freeze_struct("de8793ad88ba2969")]
#[derive(Encode, Decode, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CrowdloanInfo<AccountId, Balance, BlockNumber, Call> {
    /// The creator of the crowdloan.
    pub creator: AccountId,
    /// The initial deposit of the crowdloan from the creator.
    pub deposit: Balance,
    /// The end block of the crowdloan.
    pub end: BlockNumber,
    /// The cap to raise.
    pub cap: Balance,
    /// The amount raised so far.
    pub raised: Balance,
    /// The target address to transfer the raised funds to.
    pub target_address: AccountId,
    /// The call to dispatch when the crowdloan is finalized.
    pub call: Call,
    /// Whether the crowdloan has been finalized.
    pub finalized: bool,
}

type CrowdloanInfoOf<T> = CrowdloanInfo<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    BlockNumberFor<T>,
    BoundedCallOf<T>,
>;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configuration trait.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The overarching call type.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// The currency mechanism.
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The weight information for the pallet.
        type WeightInfo: WeightInfo;

        /// The preimage provider which will be used to store the call to dispatch.
        type Preimages: QueryPreimage<H = Self::Hashing> + StorePreimage;

        /// The pallet id that will be used to derive crowdloan account ids.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The minimum deposit required to create a crowdloan.
        #[pallet::constant]
        type MinimumDeposit: Get<BalanceOf<Self>>;

        /// The minimum contribution required to contribute to a crowdloan.
        #[pallet::constant]
        type MinimumContribution: Get<BalanceOf<Self>>;

        /// The minimum block duration for a crowdloan.
        #[pallet::constant]
        type MinimumBlockDuration: Get<BlockNumberFor<Self>>;

        /// The maximum block duration for a crowdloan.
        #[pallet::constant]
        type MaximumBlockDuration: Get<BlockNumberFor<Self>>;

        /// The maximum number of contributors that can be refunded in a single refund.
        #[pallet::constant]
        type RefundContributorsLimit: Get<u32>;
    }

    /// A map of crowdloan ids to their information.
    #[pallet::storage]
    pub type Crowdloans<T: Config> =
        StorageMap<_, Identity, CrowdloanId, CrowdloanInfoOf<T>, OptionQuery>;

    /// The next incrementing crowdloan id.
    #[pallet::storage]
    pub type NextCrowdloanId<T> = StorageValue<_, CrowdloanId, ValueQuery, ConstU32<0>>;

    /// A map of crowdloan ids to their contributors and their contributions.
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

    /// The current crowdloan id that will be set during the finalize call, making it
    /// temporarily accessible to the dispatched call.
    #[pallet::storage]
    pub type CurrentCrowdloanId<T: Config> = StorageValue<_, CrowdloanId, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A crowdloan was created.
        Created {
            crowdloan_id: CrowdloanId,
            creator: T::AccountId,
            end: BlockNumberFor<T>,
            cap: BalanceOf<T>,
        },
        /// A contribution was made to an active crowdloan.
        Contributed {
            crowdloan_id: CrowdloanId,
            contributor: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// A contribution was withdrawn from a failed crowdloan.
        Withdrew {
            crowdloan_id: CrowdloanId,
            contributor: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// A refund was partially processed for a failed crowdloan.
        PartiallyRefunded { crowdloan_id: CrowdloanId },
        /// A refund was fully processed for a failed crowdloan.
        AllRefunded { crowdloan_id: CrowdloanId },
        /// A crowdloan was finalized, funds were transferred and the call was dispatched.
        Finalized { crowdloan_id: CrowdloanId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The crowdloan initial deposit is too low.
        DepositTooLow,
        /// The crowdloan cap is too low.
        CapTooLow,
        /// The crowdloan cannot end in the past.
        CannotEndInPast,
        /// The crowdloan block duration is too short.
        BlockDurationTooShort,
        /// The block duration is too long.
        BlockDurationTooLong,
        /// The account does not have enough balance to pay for the initial deposit/contribution.
        InsufficientBalance,
        /// An overflow occurred.
        Overflow,
        /// The crowdloan id is invalid.
        InvalidCrowdloanId,
        /// The crowdloan cap has been fully raised.
        CapRaised,
        /// The contribution period has ended.
        ContributionPeriodEnded,
        /// The contribution is too low.
        ContributionTooLow,
        /// The origin is not from the creator of the crowdloan.
        ExpectedCreatorOrigin,
        /// The crowdloan has already been finalized.
        AlreadyFinalized,
        /// The crowdloan contribution period has not ended yet.
        ContributionPeriodNotEnded,
        /// The contributor has no contribution for this crowdloan.
        NoContribution,
        /// The crowdloan cap has not been raised.
        CapNotRaised,
        /// An underflow occurred.
        Underflow,
        /// Call to dispatch was not found in the preimage storage.
        CallUnavailable,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a crowdloan that will raise funds up to a maximum cap and if successful,
        /// will transfer funds to the target address and dispatch a call (using creator origin).
        ///
        /// The initial deposit will be transfered to the crowdloan account and will be refunded
        /// in case the crowdloan fails to raise the cap. Additionally, the creator will pay for
        /// the execution of the call
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// Parameters:
        /// - `deposit`: The initial deposit from the creator.
        /// - `cap`: The maximum amount of funds that can be raised.
        /// - `end`: The block number at which the crowdloan will end.
        /// - `target_address`: The address to transfer the raised funds to.
        /// - `call`: The call to dispatch when the crowdloan is finalized.
        #[pallet::call_index(0)]
        #[pallet::weight({
			let di = call.get_dispatch_info();
			let inner_call_weight = match di.pays_fee {
				Pays::Yes => di.weight,
				Pays::No => Weight::zero(),
			};
			let base_weight = T::WeightInfo::create();
			(base_weight.saturating_add(inner_call_weight), Pays::Yes)
		})]
        pub fn create(
            origin: OriginFor<T>,
            #[pallet::compact] deposit: BalanceOf<T>,
            #[pallet::compact] cap: BalanceOf<T>,
            #[pallet::compact] end: BlockNumberFor<T>,
            target_address: T::AccountId,
            call: Box<<T as Config>::RuntimeCall>,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();

            // Ensure the deposit is at least the minimum deposit and cap is greater
            ensure!(
                deposit >= T::MinimumDeposit::get(),
                Error::<T>::DepositTooLow
            );
            ensure!(cap > deposit, Error::<T>::CapTooLow);

            // Ensure the end block is after the current block and the duration is
            // between the minimum and maximum block duration
            ensure!(now < end, Error::<T>::CannotEndInPast);
            let block_duration = end.checked_sub(&now).ok_or(Error::<T>::Underflow)?;
            ensure!(
                block_duration >= T::MinimumBlockDuration::get(),
                Error::<T>::BlockDurationTooShort
            );
            ensure!(
                block_duration <= T::MaximumBlockDuration::get(),
                Error::<T>::BlockDurationTooLong
            );

            // Ensure the creator has enough balance to pay the initial deposit
            ensure!(
                CurrencyOf::<T>::free_balance(&creator) >= deposit,
                Error::<T>::InsufficientBalance
            );

            let crowdloan_id = NextCrowdloanId::<T>::get();
            let next_crowdloan_id = crowdloan_id.checked_add(1).ok_or(Error::<T>::Overflow)?;

            Crowdloans::<T>::insert(
                crowdloan_id,
                CrowdloanInfo {
                    creator: creator.clone(),
                    deposit,
                    end,
                    cap,
                    raised: deposit,
                    target_address,
                    call: T::Preimages::bound(*call)?,
                    finalized: false,
                },
            );

            NextCrowdloanId::<T>::put(next_crowdloan_id);

            // Track the crowdloan account and transfer the deposit to the crowdloan account
            frame_system::Pallet::<T>::inc_providers(&Self::crowdloan_account_id(crowdloan_id));
            CurrencyOf::<T>::transfer(
                &creator,
                &Self::crowdloan_account_id(crowdloan_id),
                deposit,
                ExistenceRequirement::AllowDeath,
            )?;

            Contributions::<T>::insert(crowdloan_id, &creator, deposit);

            Self::deposit_event(Event::<T>::Created {
                crowdloan_id,
                creator,
                end,
                cap,
            });

            Ok(())
        }

        /// Contribute to an active crowdloan.
        ///
        /// The contribution will be transfered to the crowdloan account and will be refunded
        /// if the crowdloan fails to raise the cap. If the contribution would raise the amount above the cap,
        /// the contribution will be set to the amount that is left to be raised.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// Parameters:
        /// - `crowdloan_id`: The id of the crowdloan to contribute to.
        /// - `amount`: The amount to contribute.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::contribute())]
        pub fn contribute(
            origin: OriginFor<T>,
            #[pallet::compact] crowdloan_id: CrowdloanId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let contributor = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();

            let mut crowdloan = Self::ensure_crowdloan_exists(crowdloan_id)?;

            // Ensure crowdloan has not ended and has not raised cap
            ensure!(now < crowdloan.end, Error::<T>::ContributionPeriodEnded);
            ensure!(crowdloan.raised < crowdloan.cap, Error::<T>::CapRaised);

            // Ensure contribution is at least the minimum contribution
            ensure!(
                amount >= T::MinimumContribution::get(),
                Error::<T>::ContributionTooLow
            );

            // Ensure contribution does not overflow the actual raised amount
            // and it does not exceed the cap
            let left_to_raise = crowdloan
                .cap
                .checked_sub(&crowdloan.raised)
                .ok_or(Error::<T>::Underflow)?;

            // If the contribution would raise the amount above the cap,
            // set the contribution to the amount that is left to be raised
            let amount = amount.min(left_to_raise);

            // Ensure contribution does not overflow the actual raised amount
            crowdloan.raised = crowdloan
                .raised
                .checked_add(&amount)
                .ok_or(Error::<T>::Overflow)?;

            // Ensure contribution does not overflow the contributor's total contributions
            let contribution = Contributions::<T>::get(crowdloan_id, &contributor)
                .unwrap_or(Zero::zero())
                .checked_add(&amount)
                .ok_or(Error::<T>::Overflow)?;

            // Ensure contributor has enough balance to pay
            ensure!(
                CurrencyOf::<T>::free_balance(&contributor) >= amount,
                Error::<T>::InsufficientBalance
            );

            CurrencyOf::<T>::transfer(
                &contributor,
                &Self::crowdloan_account_id(crowdloan_id),
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            Contributions::<T>::insert(crowdloan_id, &contributor, contribution);
            Crowdloans::<T>::insert(crowdloan_id, &crowdloan);

            Self::deposit_event(Event::<T>::Contributed {
                contributor,
                crowdloan_id,
                amount,
            });

            Ok(())
        }

        /// Withdraw a contribution from a failed crowdloan.
        ///
        /// The origin doesn't needs to be the contributor, it can be any account,
        /// making it possible for someone to trigger a refund for a contributor.
        ///
        /// The dispatch origin for this call must be _Signed_.
        ///
        /// Parameters:
        /// - `contributor`: The contributor to withdraw from.
        /// - `crowdloan_id`: The id of the crowdloan to withdraw from.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::withdraw())]
        pub fn withdraw(
            origin: OriginFor<T>,
            contributor: T::AccountId,
            #[pallet::compact] crowdloan_id: CrowdloanId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut crowdloan = Self::ensure_crowdloan_exists(crowdloan_id)?;
            Self::ensure_crowdloan_failed(&crowdloan)?;

            // Ensure contributor has balance left in the crowdloan account
            let amount =
                Contributions::<T>::get(crowdloan_id, &contributor).unwrap_or_else(Zero::zero);
            ensure!(amount > Zero::zero(), Error::<T>::NoContribution);

            CurrencyOf::<T>::transfer(
                &Self::crowdloan_account_id(crowdloan_id),
                &contributor,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Remove the contribution from the contributions map and update
            // refunds so far
            Contributions::<T>::remove(crowdloan_id, &contributor);
            crowdloan.raised = crowdloan.raised.saturating_sub(amount);

            Crowdloans::<T>::insert(crowdloan_id, &crowdloan);

            Self::deposit_event(Event::<T>::Withdrew {
                contributor,
                crowdloan_id,
                amount,
            });

            Ok(())
        }

        /// Refund a failed crowdloan.
        ///
        /// The call will try to refund all contributors up to the limit defined by the `RefundContributorsLimit`.
        /// If the limit is reached, the call will stop and the crowdloan will be marked as partially refunded.
        ///
        /// The dispatch origin for this call must be _Signed_ and doesn't need to be the creator of the crowdloan.
        ///
        /// Parameters:
        /// - `crowdloan_id`: The id of the crowdloan to refund.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::refund(T::RefundContributorsLimit::get()))]
        pub fn refund(
            origin: OriginFor<T>,
            #[pallet::compact] crowdloan_id: CrowdloanId,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let mut crowdloan = Self::ensure_crowdloan_exists(crowdloan_id)?;
            let crowdloan_account = Self::crowdloan_account_id(crowdloan_id);
            Self::ensure_crowdloan_failed(&crowdloan)?;

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
                refund_count = refund_count.checked_add(1).ok_or(Error::<T>::Overflow)?;
            }

            Crowdloans::<T>::insert(crowdloan_id, &crowdloan);

            // Clear refunded contributors
            for contributor in refunded_contributors {
                Contributions::<T>::remove(crowdloan_id, &contributor);
            }

            if all_refunded {
                Self::deposit_event(Event::<T>::AllRefunded { crowdloan_id });
                // The loop didn't run fully, we refund the unused weights.
                Ok(Some(T::WeightInfo::refund(refund_count)).into())
            } else {
                Self::deposit_event(Event::<T>::PartiallyRefunded { crowdloan_id });
                // The loop ran fully, we don't refund anything.
                Ok(().into())
            }
        }

        /// Finalize a successful crowdloan.
        ///
        /// The call will transfer the raised amount to the target address and dispatch the call that
        /// was provided when the crowdloan was created. The CurrentCrowdloanId will be set to the
        /// crowdloan id being finalized so the dispatched call can access it temporarily by accessing
        /// the `CurrentCrowdloanId` storage item.
        ///
        /// The dispatch origin for this call must be _Signed_ and must be the creator of the crowdloan.
        ///
        /// Parameters:
        /// - `crowdloan_id`: The id of the crowdloan to finalize.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::finalize())]
        pub fn finalize(
            origin: OriginFor<T>,
            #[pallet::compact] crowdloan_id: CrowdloanId,
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;

            let mut crowdloan = Self::ensure_crowdloan_exists(crowdloan_id)?;
            Self::ensure_crowdloan_succeeded(&crowdloan)?;

            ensure!(
                creator == crowdloan.creator,
                Error::<T>::ExpectedCreatorOrigin
            );

            // Transfer the raised amount to the target address
            CurrencyOf::<T>::transfer(
                &Self::crowdloan_account_id(crowdloan_id),
                &crowdloan.target_address,
                crowdloan.raised,
                ExistenceRequirement::AllowDeath,
            )?;

            // Set the current crowdloan id so the dispatched call
            // can access it temporarily
            CurrentCrowdloanId::<T>::put(crowdloan_id);

            // Retrieve the call from the preimage storage
            let call = match T::Preimages::peek(&crowdloan.call) {
                Ok((call, _)) => call,
                Err(_) => {
                    // If the call is not found, we drop it from the preimage storage
                    // because it's not needed anymore
                    T::Preimages::drop(&crowdloan.call);
                    return Err(Error::<T>::CallUnavailable)?;
                }
            };

            // Dispatch the call with creator origin
            call.dispatch(frame_system::RawOrigin::Signed(creator).into())
                .map(|_| ())
                .map_err(|e| e.error)?;

            // Clear the current crowdloan id
            CurrentCrowdloanId::<T>::kill();

            // Mark the crowdloan as finalized
            crowdloan.finalized = true;
            Crowdloans::<T>::insert(crowdloan_id, &crowdloan);

            Self::deposit_event(Event::<T>::Finalized { crowdloan_id });

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

    // A crowdloan is considered to have failed if it has ended, has not raised the cap and
    // has not been finalized.
    fn ensure_crowdloan_failed(crowdloan: &CrowdloanInfoOf<T>) -> Result<(), Error<T>> {
        let now = frame_system::Pallet::<T>::block_number();
        ensure!(now >= crowdloan.end, Error::<T>::ContributionPeriodNotEnded);
        ensure!(crowdloan.raised < crowdloan.cap, Error::<T>::CapRaised);
        ensure!(!crowdloan.finalized, Error::<T>::AlreadyFinalized);
        Ok(())
    }

    // A crowdloan is considered to have succeeded if it has ended, has raised the cap and
    // has not been finalized.
    fn ensure_crowdloan_succeeded(crowdloan: &CrowdloanInfoOf<T>) -> Result<(), Error<T>> {
        let now = frame_system::Pallet::<T>::block_number();
        ensure!(now >= crowdloan.end, Error::<T>::ContributionPeriodNotEnded);
        ensure!(crowdloan.raised == crowdloan.cap, Error::<T>::CapNotRaised);
        ensure!(!crowdloan.finalized, Error::<T>::AlreadyFinalized);
        Ok(())
    }
}
