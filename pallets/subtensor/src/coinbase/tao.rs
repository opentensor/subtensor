/// This file contains all critical operations with TAO and Alpha:
/// 
///   - Minting, burning, recycling, and transferring
///   - Reading colkey TAO balances
///   - Access to subnet TAO reserves
/// 

use frame_support::traits::{
    dispatch::{DispatchError, DispatchResult},
    Imbalance, fungible::Mutate,
    tokens::{
        Fortitude, Precision, Preservation,
        fungible::{Balanced, Credit, Inspect},
    },
};
use sp_runtime::traits::AccountIdConversion;
use subtensor_runtime_common::{NetUid, TaoBalance};

use super::*;

pub type BalanceOf<T> =
    <<T as Config>::Currency as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type CreditOf<T> = Credit<
    <T as frame_system::Config>::AccountId,
    <T as Config>::Currency,
>;

impl<T: Config> Pallet<T> {

    pub fn get_subnet_tao(netuid: NetUid) -> TaoBalance {
        let maybe_subnet_account = Self::get_subnet_account_id(netuid);
        if let Some(subnet_account) = maybe_subnet_account {
            Self::get_coldkey_balance(&subnet_account)
        } else {
            0.into()
        }
    }

    pub fn transfer_tao(
        origin_coldkey: &T::AccountId,
        destination_coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        <T as pallet::Config>::Currency::transfer(origin_coldkey, destination_coldkey, amount, Preservation::Expendable)?;
        Ok(())
    }

    /// Permanently remove TAO amount from existence by moving to the burn 
    /// address. Does not effect issuance rate
    pub fn burn_tao(
        coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let burn_address: T::AccountId = T::BurnAccountId::get().into_account_truncating();
        Self::transfer_tao(coldkey, &burn_address, amount)?;
        Ok(())
    }

    /// Remove TAO from existence and reduce total issuance.
    /// Effects issuance rate by reducing TI.
    pub fn recycle_tao(
        coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_sub(amount));

        let _ = <T as Config>::Currency::withdraw(
            coldkey,
            amount,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Force,
        )
        .map_err(|_| Error::<T>::BalanceWithdrawalError)?
        .peek();

        Ok(())        
    }    

    pub fn can_remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> bool {
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }

        // This bit is currently untested. @todo

        <T as Config>::Currency::can_withdraw(coldkey, amount)
            .into_result(false)
            .is_ok()
    }

    pub fn get_coldkey_balance(
        coldkey: &T::AccountId,
    ) -> BalanceOf<T>
    {
        <T as Config>::Currency::reducible_balance(
            coldkey,
            Preservation::Expendable,
            Fortitude::Polite,
        )
    }

    pub fn kill_coldkey_account(
        coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<TaoBalance, DispatchError> {
        if amount.is_zero() {
            return Ok(0.into());
        }

        let credit = <T as Config>::Currency::withdraw(
            coldkey,
            amount,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Force,
        )
        .map_err(|_| Error::<T>::BalanceWithdrawalError)?
        .peek();

        if credit.is_zero() {
            return Err(Error::<T>::ZeroBalanceAfterWithdrawn.into());
        }

        Ok(credit)
    }

    /// Create TAO and return the imbalance.
    /// 
    /// The mint workflow is following:
    ///   1. mint_tao in block_emission
    ///   2. spend_tao in run_coinbase (distribute to subnets)
    ///   3. None should be left, so burn the remainder using burn_credit for records
    pub fn mint_tao(amount: BalanceOf<T>) -> CreditOf<T> {
        <T as Config>::Currency::issue(amount)
    }

    /// Spend part of the imbalance
    /// The part parameter is the balance itself that will be credited to the coldkey
    /// Return the remaining credit or error
    pub fn spend_tao(
        coldkey: &T::AccountId,
        credit: CreditOf<T>,
        part: BalanceOf<T>,
    ) -> Result<CreditOf<T>, DispatchError> {
        let (to_spend, remainder) = credit.split(part);

        T::Currency::resolve(who, to_spend)
            .map_err(|_credit| DispatchError::Other("Could not resolve partial credit"))?;

        Ok(remainder)
    }

    /// Finalizes the unused part of the minted TAO. Normally, there should be none, this function 
    /// is only needed for guarding / logging
    pub fn burn_credit(credit: CreditOf<T>) -> DispatchResult {
        let amount = credit.peek();
        if amount.is_zero() {
            // Normal behavior
            return Ok(());
        }

        // Some credit is remaining. This is error and it should be corrected. Record the situation with 
        // burned amount in logs and in burn_address.
        let burn_address: T::AccountId = T::BurnAccountId::get().into_account_truncating();
        log::error!(
            "burn_credit received non-zero credit ({:?}); sending it to burn account {:?}, which will burn it",
            amount,
            burn_address,
        );

        T::Currency::resolve(&burn_address, credit).map_err(|unresolved_credit| {
            log::error!(
                "burn_credit failed: could not resolve credit {:?} into burn account {:?}",
                unresolved_credit.peek(),
                burn_address,
            );
            DispatchError::Other("Could not resolve burn credit")
        })
    }

    // pub fn drain_tao_imbalance_into_subnet_reserve(imbalance: NegativeImbalance, netuid: NetUid) {
    // }

    // pub fn mint_tao_for_subnet_reserve(tao: TaoBalance, netuid: NetUid) -> DispatchResult {
    //     let maybe_subnet_account = SubtensorModule::get_subnet_account_id(netuid);
    //     if let Some(subnet_account) = maybe_subnet_account {
    //         let _ = <T as Config>::Currency::deposit(subnet_account, tao, Precision::BestEffort);
    //         Ok(())
    //     } else {
    //         Err(Error::<T>::SubnetNotExists)
    //     }
    // }


}