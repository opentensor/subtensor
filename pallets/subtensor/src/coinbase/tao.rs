/// This file contains all critical operations with TAO and Alpha:
/// 
///   - Minting, burning, recycling, and transferring
///   - Reading colkey TAO balances
///   - Access to subnet TAO reserves
/// 

use frame_support::traits::{
    Imbalance, fungible::Mutate,
    tokens::{
        Fortitude, Precision, Preservation,
        fungible::{Balanced as _, Inspect as _},
    },
};
use subtensor_runtime_common::{NetUid, TaoBalance};

use super::*;

pub type BalanceOf<T> =
    <<T as Config>::Currency as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

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

    pub fn burn_tao(
        coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
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

    pub fn add_balance_to_coldkey_account(
        coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) {
        // infallible
        let _ = <T as Config>::Currency::deposit(coldkey, amount, Precision::BestEffort);
    }

    #[must_use = "Balance must be used to preserve total issuance of token"]
    pub fn remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<TaoBalance, DispatchError> {
        if amount.is_zero() {
            return Ok(TaoBalance::ZERO);
        }

        let credit = <T as Config>::Currency::withdraw(
            coldkey,
            amount,
            Precision::BestEffort,
            Preservation::Preserve,
            Fortitude::Polite,
        )
        .map_err(|_| Error::<T>::BalanceWithdrawalError)?
        .peek();

        if credit.is_zero() {
            return Err(Error::<T>::ZeroBalanceAfterWithdrawn.into());
        }

        Ok(credit.into())
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