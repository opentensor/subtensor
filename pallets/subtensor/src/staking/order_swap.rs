use super::*;
use frame_support::traits::fungible::Mutate;
use frame_support::traits::tokens::Preservation;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};
use subtensor_swap_interface::{OrderSwapInterface, SwapHandler};

impl<T: Config> OrderSwapInterface<T::AccountId> for Pallet<T> {
    fn buy_alpha(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        tao_amount: TaoBalance,
        limit_price: TaoBalance,
        apply_limits: bool,
    ) -> Result<AlphaBalance, DispatchError> {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        Self::ensure_subtoken_enabled(netuid)?;
        if apply_limits {
            ensure!(
                Self::hotkey_account_exists(hotkey),
                Error::<T>::HotKeyAccountNotExists
            );
            ensure!(
                tao_amount >= DefaultMinStake::<T>::get(),
                Error::<T>::AmountTooLow
            );
            ensure!(
                Self::can_remove_balance_from_coldkey_account(coldkey, tao_amount),
                Error::<T>::NotEnoughBalanceToStake
            );
        }
        // Debit TAO from the buyer before the pool swap so the pallet's
        // intermediary account (and individual buyers in execute_orders) cannot
        // stake more TAO than they actually hold.
        let actual_tao = Self::remove_balance_from_coldkey_account(coldkey, tao_amount)?;
        let alpha_out = Self::stake_into_subnet(
            hotkey,
            coldkey,
            netuid,
            actual_tao,
            limit_price,
            false,
            false,
        )?;
        if apply_limits {
            Self::set_stake_operation_limit(hotkey, coldkey, netuid);
        }
        Ok(alpha_out)
    }

    fn sell_alpha(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        alpha_amount: AlphaBalance,
        limit_price: TaoBalance,
        apply_limits: bool,
    ) -> Result<TaoBalance, DispatchError> {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        Self::ensure_subtoken_enabled(netuid)?;
        if apply_limits {
            ensure!(!alpha_amount.is_zero(), Error::<T>::AmountTooLow);
            let tao_equiv = T::SwapInterface::current_alpha_price(netuid)
                .saturating_mul(U96F32::saturating_from_num(alpha_amount.to_u64()))
                .saturating_to_num::<u64>();
            ensure!(
                TaoBalance::from(tao_equiv) >= DefaultMinStake::<T>::get(),
                Error::<T>::AmountTooLow
            );
            let available =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid);
            ensure!(available >= alpha_amount, Error::<T>::NotEnoughStakeToWithdraw);
            Self::ensure_stake_operation_limit_not_exceeded(hotkey, coldkey, netuid)?;
        }
        let tao_out =
            Self::unstake_from_subnet(hotkey, coldkey, netuid, alpha_amount, limit_price, false)?;
        // Credit TAO proceeds to the seller so the pallet's intermediary account
        // (and individual sellers in execute_orders) have real balance to
        // distribute or forward to the fee collector.
        Self::add_balance_to_coldkey_account(coldkey, tao_out);
        Ok(tao_out)
    }

    fn current_alpha_price(netuid: NetUid) -> U96F32 {
        T::SwapInterface::current_alpha_price(netuid)
    }

    fn transfer_tao(from: &T::AccountId, to: &T::AccountId, amount: TaoBalance) -> DispatchResult {
        <T as Config>::Currency::transfer(from, to, amount, Preservation::Expendable)?;
        Ok(())
    }

    fn transfer_staked_alpha(
        from_coldkey: &T::AccountId,
        from_hotkey: &T::AccountId,
        to_coldkey: &T::AccountId,
        to_hotkey: &T::AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
        validate_sender: bool,
        set_receiver_limit: bool,
    ) -> DispatchResult {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        Self::ensure_subtoken_enabled(netuid)?;
        if validate_sender {
            ensure!(!amount.is_zero(), Error::<T>::AmountTooLow);
            let tao_equiv = T::SwapInterface::current_alpha_price(netuid)
                .saturating_mul(U96F32::saturating_from_num(amount.to_u64()))
                .saturating_to_num::<u64>();
            ensure!(
                TaoBalance::from(tao_equiv) >= DefaultMinStake::<T>::get(),
                Error::<T>::AmountTooLow
            );
            Self::ensure_stake_operation_limit_not_exceeded(from_hotkey, from_coldkey, netuid)?;
        }

        let available =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(from_hotkey, from_coldkey, netuid);
        ensure!(available >= amount, Error::<T>::NotEnoughStakeToWithdraw);
        Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            from_hotkey,
            from_coldkey,
            netuid,
            amount,
        );
        Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
            to_hotkey, to_coldkey, netuid, amount,
        );
        LastColdkeyHotkeyStakeBlock::<T>::insert(
            to_coldkey,
            to_hotkey,
            Self::get_current_block_as_u64(),
        );
        if set_receiver_limit {
            Self::set_stake_operation_limit(to_hotkey, to_coldkey, netuid);
        }
        Ok(())
    }
}
