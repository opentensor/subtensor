#![cfg_attr(not(feature = "std"), no_std)]

// FRAME
use frame_support::{
    pallet_prelude::*,
    traits::{
        Imbalance, IsSubType, OnUnbalanced,
        fungible::{Balanced, Credit, Debt, Inspect},
        tokens::{Precision, WithdrawConsequence},
    },
    weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
};

// Runtime
use sp_runtime::{
    Perbill, Saturating,
    traits::{DispatchInfoOf, PostDispatchInfoOf},
};

// Pallets
use pallet_subtensor::Call as SubtensorCall;
use pallet_transaction_payment::Config as PTPConfig;
use pallet_transaction_payment::OnChargeTransaction;

// Misc
use core::marker::PhantomData;
use smallvec::smallvec;
use sp_std::vec::Vec;
use subtensor_runtime_common::{Balance, NetUid};

// Tests
#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type CallOf<T> = <T as frame_system::Config>::RuntimeCall;

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

/// Trait that allows working with Alpha
pub trait AlphaFeeHandler<T: frame_system::Config> {
    fn can_withdraw_in_alpha(
        coldkey: &AccountIdOf<T>,
        alpha_vec: &Vec<(AccountIdOf<T>, NetUid)>,
        tao_amount: u64,
    ) -> bool;
    fn withdraw_in_alpha(
        coldkey: &AccountIdOf<T>,
        alpha_vec: &Vec<(AccountIdOf<T>, NetUid)>,
        tao_amount: u64,
    );
}

/// Enum that describes either a withdrawn amount of transaction fee in TAO or the
/// fact that fee was charged in Alpha (without an amount because it is not needed)
pub enum WithdrawnFee<T: frame_system::Config, F: Balanced<AccountIdOf<T>>> {
    Tao(Credit<AccountIdOf<T>, F>),
    Alpha,
}

/// Custom OnChargeTransaction implementation based on standard FungibleAdapter from transaction_payment
/// FRAME pallet
///
pub struct SubtensorTxFeeHandler<F, OU>(PhantomData<(F, OU)>);

/// This implementation contains the list of calls that require paying transaction
/// fees in Alpha
impl<F, OU> SubtensorTxFeeHandler<F, OU> {
    /// Returns Vec<(hotkey, netuid)> if the given call should pay fees in Alpha instead of TAO.
    /// The vector represents all subnets where this hotkey has any alpha stake. Fees will be
    /// distributed evenly between subnets in case of multiple subnets.
    pub fn fees_in_alpha<T>(call: &CallOf<T>) -> Vec<(AccountIdOf<T>, NetUid)>
    where
        T: frame_system::Config + pallet_subtensor::Config,
        CallOf<T>: IsSubType<pallet_subtensor::Call<T>>,
    {
        let mut alpha_vec: Vec<(AccountIdOf<T>, NetUid)> = Vec::new();

        // Otherwise, switch to Alpha for the extrinsics that assume converting Alpha
        // to TAO
        // TODO: Populate the list
        match call.is_sub_type() {
            Some(SubtensorCall::remove_stake { hotkey, netuid, .. }) => {
                log::info!("fees_in_alpha: matched remove_stake => use Alpha");
                alpha_vec.push((hotkey.clone(), *netuid))
            }
            _ => {}
        }

        alpha_vec
    }
}

impl<T, F, OU> OnChargeTransaction<T> for SubtensorTxFeeHandler<F, OU>
where
    T: PTPConfig + pallet_subtensor::Config,
    CallOf<T>: IsSubType<pallet_subtensor::Call<T>>,
    F: Balanced<T::AccountId>,
    OU: OnUnbalanced<Credit<T::AccountId, F>> + AlphaFeeHandler<T>,
    <F as Inspect<AccountIdOf<T>>>::Balance: Into<u64>,
{
    type LiquidityInfo = Option<WithdrawnFee<T, F>>;
    type Balance = <F as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

    fn withdraw_fee(
        who: &AccountIdOf<T>,
        call: &CallOf<T>,
        _dispatch_info: &DispatchInfoOf<CallOf<T>>,
        fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        if fee.is_zero() {
            return Ok(None);
        }

        // Traditional fees in TAO
        match F::withdraw(
            who,
            fee,
            Precision::Exact,
            frame_support::traits::tokens::Preservation::Preserve,
            frame_support::traits::tokens::Fortitude::Polite,
        ) {
            Ok(imbalance) => Ok(Some(WithdrawnFee::Tao(imbalance))),
            Err(_) => {
                let alpha_vec = Self::fees_in_alpha::<T>(call);
                if !alpha_vec.is_empty() {
                    let fee_u64: u64 = fee.into();
                    OU::withdraw_in_alpha(who, &alpha_vec, fee_u64);
                    return Ok(Some(WithdrawnFee::Alpha));
                }
                Err(InvalidTransaction::Payment.into())
            }
        }
    }

    fn can_withdraw_fee(
        who: &AccountIdOf<T>,
        call: &CallOf<T>,
        _dispatch_info: &DispatchInfoOf<CallOf<T>>,
        fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        if fee.is_zero() {
            return Ok(());
        }

        // Prefer traditional fees in TAO
        match F::can_withdraw(who, fee) {
            WithdrawConsequence::Success => Ok(()),
            _ => {
                // Fallback to fees in Alpha if possible
                let alpha_vec = Self::fees_in_alpha::<T>(call);
                if !alpha_vec.is_empty() {
                    let fee_u64: u64 = fee.into();
                    if OU::can_withdraw_in_alpha(who, &alpha_vec, fee_u64) {
                        return Ok(());
                    }
                }
                Err(InvalidTransaction::Payment.into())
            }
        }
    }

    fn correct_and_deposit_fee(
        who: &AccountIdOf<T>,
        _dispatch_info: &DispatchInfoOf<CallOf<T>>,
        _post_info: &PostDispatchInfoOf<CallOf<T>>,
        corrected_fee: Self::Balance,
        tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        if let Some(withdrawn) = already_withdrawn {
            // Fee may be paid in TAO or in Alpha. Only refund and update total issuance for
            // TAO fees because Alpha fees are charged precisely and do not need any adjustments
            match withdrawn {
                WithdrawnFee::Tao(paid) => {
                    // Calculate how much refund we should return
                    let refund_amount = paid.peek().saturating_sub(corrected_fee);
                    // refund to the account that paid the fees if it exists. otherwise, don't refund
                    // anything.
                    let refund_imbalance = if F::total_balance(who) > F::Balance::zero() {
                        F::deposit(who, refund_amount, Precision::BestEffort)
                            .unwrap_or_else(|_| Debt::<T::AccountId, F>::zero())
                    } else {
                        Debt::<T::AccountId, F>::zero()
                    };
                    // merge the imbalance caused by paying the fees and refunding parts of it again.
                    let adjusted_paid: Credit<T::AccountId, F> =
                        paid.offset(refund_imbalance).same().map_err(|_| {
                            TransactionValidityError::Invalid(InvalidTransaction::Payment)
                        })?;
                    // Call someone else to handle the imbalance (fee and tip separately)
                    let (tip, fee) = adjusted_paid.split(tip);
                    OU::on_unbalanceds(Some(fee).into_iter().chain(Some(tip)));
                }
                WithdrawnFee::Alpha => {
                    // We do not refund Alpha, charges are final
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn endow_account(who: &AccountIdOf<T>, amount: Self::Balance) {
        let _ = F::deposit(who, amount, Precision::BestEffort);
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn minimum_balance() -> Self::Balance {
        F::minimum_balance()
    }
}
