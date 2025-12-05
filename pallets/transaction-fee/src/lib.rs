#![cfg_attr(not(feature = "std"), no_std)]

// FRAME
use frame_support::{
    pallet_prelude::*,
    traits::{
        Imbalance, IsSubType, OnUnbalanced,
        fungible::{
            Balanced, Credit, Debt, DecreaseIssuance, Imbalance as FungibleImbalance,
            IncreaseIssuance, Inspect,
        },
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
use subtensor_swap_interface::SwapHandler;

// Misc
use core::marker::PhantomData;
use smallvec::smallvec;
use sp_std::vec::Vec;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{Balance, Currency, NetUid};

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
            coeff_frac: Perbill::from_parts(50_000), // 0.05 unit per weight
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
        alpha_vec: &[(AccountIdOf<T>, NetUid)],
        tao_amount: u64,
    ) -> bool;
    fn withdraw_in_alpha(
        coldkey: &AccountIdOf<T>,
        alpha_vec: &[(AccountIdOf<T>, NetUid)],
        tao_amount: u64,
    );
    fn get_all_netuids_for_coldkey_and_hotkey(
        coldkey: &AccountIdOf<T>,
        hotkey: &AccountIdOf<T>,
    ) -> Vec<NetUid>;
}

/// Deduct the transaction fee from the Subtensor Pallet TotalIssuance when charging the transaction
/// fee.
pub struct TransactionFeeHandler<T>(core::marker::PhantomData<T>);
impl<T> Default for TransactionFeeHandler<T> {
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T>
    OnUnbalanced<
        FungibleImbalance<
            u64,
            DecreaseIssuance<AccountIdOf<T>, pallet_balances::Pallet<T>>,
            IncreaseIssuance<AccountIdOf<T>, pallet_balances::Pallet<T>>,
        >,
    > for TransactionFeeHandler<T>
where
    T: frame_system::Config,
    T: pallet_subtensor::Config,
    T: pallet_balances::Config<Balance = u64>,
{
    fn on_nonzero_unbalanced(
        imbalance: FungibleImbalance<
            u64,
            DecreaseIssuance<AccountIdOf<T>, pallet_balances::Pallet<T>>,
            IncreaseIssuance<AccountIdOf<T>, pallet_balances::Pallet<T>>,
        >,
    ) {
        let ti_before = pallet_subtensor::TotalIssuance::<T>::get();
        pallet_subtensor::TotalIssuance::<T>::put(
            ti_before.saturating_sub(imbalance.peek().into()),
        );
        drop(imbalance);
    }
}

/// Handle Alpha fees
impl<T> AlphaFeeHandler<T> for TransactionFeeHandler<T>
where
    T: frame_system::Config,
    T: pallet_subtensor::Config,
    T: pallet_subtensor_swap::Config,
{
    /// This function checks if tao_amount fee can be withdraw in Alpha currency
    /// by converting Alpha to TAO at the current price and ignoring slippage.
    ///
    /// If this function returns true, the transaction will be included in the block
    /// and Alpha will be withdraw from the account, no matter whether transaction
    /// is successful or not.
    ///
    /// If this function returns true, but at the time of execution the Alpha price
    /// changes and it becomes impossible to pay tx fee with the Alpha balance,
    /// the transaction still executes and all Alpha is withdrawn from the account.
    fn can_withdraw_in_alpha(
        coldkey: &AccountIdOf<T>,
        alpha_vec: &[(AccountIdOf<T>, NetUid)],
        tao_amount: u64,
    ) -> bool {
        if alpha_vec.is_empty() {
            // Alpha vector is empty, nothing to withdraw
            return false;
        }

        // Divide tao_amount among all alpha entries
        let tao_per_entry = tao_amount.checked_div(alpha_vec.len() as u64).unwrap_or(0);

        // The rule here is that we should be able to withdraw at least from one entry.
        // This is not ideal because it may not pay all fees, but UX is the priority
        // and this approach still provides spam protection.
        alpha_vec.iter().any(|(hotkey, netuid)| {
            let alpha_balance = U96F32::saturating_from_num(
                pallet_subtensor::Pallet::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    hotkey, coldkey, *netuid,
                ),
            );
            let alpha_price = pallet_subtensor_swap::Pallet::<T>::current_alpha_price(*netuid);
            alpha_price.saturating_mul(alpha_balance) >= tao_per_entry
        })
    }

    fn withdraw_in_alpha(
        coldkey: &AccountIdOf<T>,
        alpha_vec: &[(AccountIdOf<T>, NetUid)],
        tao_amount: u64,
    ) {
        if alpha_vec.is_empty() {
            return;
        }

        let tao_per_entry = tao_amount.checked_div(alpha_vec.len() as u64).unwrap_or(0);

        alpha_vec.iter().for_each(|(hotkey, netuid)| {
            // Divide tao_amount evenly among all alpha entries
            let alpha_balance = U96F32::saturating_from_num(
                pallet_subtensor::Pallet::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    hotkey, coldkey, *netuid,
                ),
            );
            let alpha_price = pallet_subtensor_swap::Pallet::<T>::current_alpha_price(*netuid);
            let alpha_fee = U96F32::saturating_from_num(tao_per_entry)
                .checked_div(alpha_price)
                .unwrap_or(alpha_balance)
                .min(alpha_balance)
                .saturating_to_num::<u64>();

            pallet_subtensor::Pallet::<T>::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                coldkey,
                *netuid,
                alpha_fee.into(),
            );
        });
    }

    fn get_all_netuids_for_coldkey_and_hotkey(
        coldkey: &AccountIdOf<T>,
        hotkey: &AccountIdOf<T>,
    ) -> Vec<NetUid> {
        pallet_subtensor::Pallet::<T>::get_all_subnet_netuids()
            .into_iter()
            .filter(|netuid| pallet_subtensor::SubtokenEnabled::<T>::get(netuid))
            .filter(|netuid| {
                pallet_subtensor::Pallet::<T>::get_stake_for_hotkey_and_coldkey_on_subnet(
                    hotkey, coldkey, *netuid,
                ) != 0.into()
            })
            .collect()
    }
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
    pub fn fees_in_alpha<T>(who: &AccountIdOf<T>, call: &CallOf<T>) -> Vec<(AccountIdOf<T>, NetUid)>
    where
        T: frame_system::Config + pallet_subtensor::Config,
        CallOf<T>: IsSubType<pallet_subtensor::Call<T>>,
        OU: AlphaFeeHandler<T>,
    {
        let mut alpha_vec: Vec<(AccountIdOf<T>, NetUid)> = Vec::new();

        // Otherwise, switch to Alpha for the extrinsics that assume converting Alpha
        // to TAO
        // TODO: Populate the list
        match call.is_sub_type() {
            Some(SubtensorCall::remove_stake { hotkey, netuid, .. }) => {
                alpha_vec.push((hotkey.clone(), *netuid))
            }
            Some(SubtensorCall::remove_stake_limit { hotkey, netuid, .. }) => {
                alpha_vec.push((hotkey.clone(), *netuid))
            }
            Some(SubtensorCall::remove_stake_full_limit { hotkey, netuid, .. }) => {
                alpha_vec.push((hotkey.clone(), *netuid))
            }
            Some(SubtensorCall::unstake_all { hotkey, .. }) => {
                let netuids = OU::get_all_netuids_for_coldkey_and_hotkey(who, hotkey);
                netuids
                    .into_iter()
                    .for_each(|netuid| alpha_vec.push((hotkey.clone(), netuid)));
            }
            Some(SubtensorCall::unstake_all_alpha { hotkey, .. }) => {
                let netuids = OU::get_all_netuids_for_coldkey_and_hotkey(who, hotkey);
                netuids
                    .into_iter()
                    .for_each(|netuid| alpha_vec.push((hotkey.clone(), netuid)));
            }
            Some(SubtensorCall::move_stake {
                origin_hotkey,
                destination_hotkey: _,
                origin_netuid,
                ..
            }) => alpha_vec.push((origin_hotkey.clone(), *origin_netuid)),
            Some(SubtensorCall::transfer_stake {
                destination_coldkey: _,
                hotkey,
                origin_netuid,
                ..
            }) => alpha_vec.push((hotkey.clone(), *origin_netuid)),
            Some(SubtensorCall::swap_stake {
                hotkey,
                origin_netuid,
                ..
            }) => alpha_vec.push((hotkey.clone(), *origin_netuid)),
            Some(SubtensorCall::swap_stake_limit {
                hotkey,
                origin_netuid,
                ..
            }) => alpha_vec.push((hotkey.clone(), *origin_netuid)),
            Some(SubtensorCall::recycle_alpha {
                hotkey,
                amount: _,
                netuid,
            }) => alpha_vec.push((hotkey.clone(), *netuid)),
            Some(SubtensorCall::burn_alpha {
                hotkey,
                amount: _,
                netuid,
            }) => alpha_vec.push((hotkey.clone(), *netuid)),
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
        _call: &CallOf<T>,
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
                // let alpha_vec = Self::fees_in_alpha::<T>(who, call);
                // if !alpha_vec.is_empty() {
                //     let fee_u64: u64 = fee.into();
                //     OU::withdraw_in_alpha(who, &alpha_vec, fee_u64);
                //     return Ok(Some(WithdrawnFee::Alpha));
                // }
                Err(InvalidTransaction::Payment.into())
            }
        }
    }

    fn can_withdraw_fee(
        who: &AccountIdOf<T>,
        _call: &CallOf<T>,
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
                // // Fallback to fees in Alpha if possible
                // let alpha_vec = Self::fees_in_alpha::<T>(who, call);
                // if !alpha_vec.is_empty() {
                //     let fee_u64: u64 = fee.into();
                //     if OU::can_withdraw_in_alpha(who, &alpha_vec, fee_u64) {
                //         return Ok(());
                //     }
                // }
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
                    // Subtensor does not refund Alpha fees, charges are final
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
