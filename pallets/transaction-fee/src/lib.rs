#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use frame_support::{
    traits::{
        Imbalance, OnUnbalanced,
        fungible::{Balanced, Credit, Debt, Inspect},
        tokens::{Precision, WithdrawConsequence},
    },
    weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
};

use sp_runtime::{
    Perbill, Saturating,
    traits::{DispatchInfoOf, PostDispatchInfoOf},
};
// use substrate_fixed::types::U96F32;
// use subtensor_runtime_common::{AlphaCurrency, NetUid};
use pallet_transaction_payment::Config as PTPConfig;

use smallvec::smallvec;
use subtensor_runtime_common::Balance;

pub use pallet_transaction_payment::OnChargeTransaction;

pub struct LinearWeightToFee;

impl WeightToFeePolynomial for LinearWeightToFee {
    type Balance = Balance;

    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        let coefficient = WeightToFeeCoefficient {
            coeff_integer: 0,
            coeff_frac: Perbill::from_parts(500_000), // 0.5 unit per weight
            negative: false,
            degree: 1,
        };

        smallvec![coefficient]
    }
}

/// Custom FungibleAdapter based on standard FungibleAdapter from transsaction_payment 
/// FRAME pallet
/// 
pub struct FungibleAdapter<F, OU>(PhantomData<(F, OU)>);

impl<T, F, OU> OnChargeTransaction<T> for FungibleAdapter<F, OU>
where
    T: PTPConfig,
    F: Balanced<T::AccountId>,
    OU: OnUnbalanced<Credit<T::AccountId, F>>,
{
    type LiquidityInfo = Option<Credit<T::AccountId, F>>;
    type Balance = <F as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    fn withdraw_fee(
        who: &<T>::AccountId,
        call: &<T>::RuntimeCall,
        _dispatch_info: &DispatchInfoOf<<T>::RuntimeCall>,
        fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        log::error!("====================== withdraw_fee. Call = {:?}", call);

        if fee.is_zero() {
            return Ok(None);
        }

        match F::withdraw(
            who,
            fee,
            Precision::Exact,
            frame_support::traits::tokens::Preservation::Preserve,
            frame_support::traits::tokens::Fortitude::Polite,
        ) {
            Ok(imbalance) => Ok(Some(imbalance)),
            Err(_) => Err(InvalidTransaction::Payment.into()),
        }
    }

    fn can_withdraw_fee(
        who: &T::AccountId,
        _call: &T::RuntimeCall,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        if fee.is_zero() {
            return Ok(());
        }

        match F::can_withdraw(who, fee) {
            WithdrawConsequence::Success => Ok(()),
            _ => Err(InvalidTransaction::Payment.into()),
        }
    }

    fn correct_and_deposit_fee(
        who: &<T>::AccountId,
        _dispatch_info: &DispatchInfoOf<<T>::RuntimeCall>,
        _post_info: &PostDispatchInfoOf<<T>::RuntimeCall>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        if let Some(paid) = already_withdrawn {
            // Calculate how much refund we should return
            let refund_amount = paid.peek().saturating_sub(corrected_fee);
            // refund to the the account that paid the fees if it exists. otherwise, don't refind
            // anything.
            let refund_imbalance = if F::total_balance(who) > F::Balance::zero() {
                F::deposit(who, refund_amount, Precision::BestEffort)
                    .unwrap_or_else(|_| Debt::<T::AccountId, F>::zero())
            } else {
                Debt::<T::AccountId, F>::zero()
            };
            // merge the imbalance caused by paying the fees and refunding parts of it again.
            let adjusted_paid: Credit<T::AccountId, F> = paid
                .offset(refund_imbalance)
                .same()
                .map_err(|_| TransactionValidityError::Invalid(InvalidTransaction::Payment))?;
            // Call someone else to handle the imbalance (fee and tip separately)
            let (tip, fee) = adjusted_paid.split(tip);
            OU::on_unbalanceds(Some(fee).into_iter().chain(Some(tip)));
        }

        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn endow_account(who: &T::AccountId, amount: Self::Balance) {
        let _ = F::deposit(who, amount, Precision::BestEffort);
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn minimum_balance() -> Self::Balance {
        F::minimum_balance()
    }
}
