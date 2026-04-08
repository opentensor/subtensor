use super::*;
use frame_support::traits::fungible::Mutate;
use frame_support::traits::tokens::Preservation;
use frame_support::transactional;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};
use subtensor_swap_interface::{OrderSwapInterface, SwapHandler};

impl<T: Config> OrderSwapInterface<T::AccountId> for Pallet<T> {
    #[transactional]
    fn buy_alpha(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        tao_amount: TaoBalance,
        limit_price: TaoBalance,
        validate: bool,
    ) -> Result<AlphaBalance, DispatchError> {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        Self::ensure_subtoken_enabled(netuid)?;
        if validate {
            ensure!(
                Self::coldkey_owns_hotkey(coldkey, hotkey),
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
        // `limit_price` arrives in the same units as `current_alpha_price()` (a raw ratio
        // where 1.0 ≈ 1 unit/alpha).  The AMM encodes its price_limit as `price × 10⁹`
        // (matching the rao-per-TAO precision convention), so we scale up here before
        // handing off to `stake_into_subnet`.  saturating_mul handles the no-ceiling case
        // (limit_price = u64::MAX) by saturating to u64::MAX, which the AMM interprets as
        // an astronomically high ceiling that current prices never reach.
        let amm_limit = TaoBalance::from(limit_price.to_u64().saturating_mul(1_000_000_000));
        let alpha_out =
            Self::stake_into_subnet(hotkey, coldkey, netuid, actual_tao, amm_limit, false, false)?;
        if validate {
            Self::set_stake_operation_limit(hotkey, coldkey, netuid);
        }
        Ok(alpha_out)
    }

    #[transactional]
    fn sell_alpha(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        alpha_amount: AlphaBalance,
        limit_price: TaoBalance,
        validate: bool,
    ) -> Result<TaoBalance, DispatchError> {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        Self::ensure_subtoken_enabled(netuid)?;
        if validate {
            ensure!(
                Self::coldkey_owns_hotkey(coldkey, hotkey),
                Error::<T>::HotKeyAccountNotExists
            );

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
            ensure!(
                available >= alpha_amount,
                Error::<T>::NotEnoughStakeToWithdraw
            );
            Self::ensure_stake_operation_limit_not_exceeded(hotkey, coldkey, netuid)?;
        }
        // Same ×10⁹ scaling as in buy_alpha: limit_price is in current_alpha_price() units;
        // the AMM expects price × 10⁹.  For the no-floor case (limit_price = 0) the result
        // is 0, which the AMM treats as "no lower bound".
        let amm_limit = TaoBalance::from(limit_price.to_u64().saturating_mul(1_000_000_000));
        let tao_out =
            Self::unstake_from_subnet(hotkey, coldkey, netuid, alpha_amount, amm_limit, false)?;
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
        validate_receiver: bool,
    ) -> DispatchResult {
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);
        Self::ensure_subtoken_enabled(netuid)?;
        if validate_sender {
            ensure!(
                Self::coldkey_owns_hotkey(from_coldkey, from_hotkey),
                Error::<T>::HotKeyAccountNotExists
            );
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
        if validate_receiver {
            ensure!(
                Self::coldkey_owns_hotkey(to_coldkey, to_hotkey),
                Error::<T>::HotKeyAccountNotExists
            );
            Self::set_stake_operation_limit(to_hotkey, to_coldkey, netuid);
        }
        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn set_up_netuid_for_benchmark(netuid: NetUid) {
        if !Self::if_subnet_exist(netuid) {
            Self::init_new_network(netuid, 100);
        }
        SubtokenEnabled::<T>::insert(netuid, true);
        // Seed pool reserves so the AMM price is well-defined and swaps return non-zero.
        SubnetTAO::<T>::insert(netuid, TaoBalance::from(1_000_000_000_000_u64));
        SubnetAlphaIn::<T>::insert(netuid, AlphaBalance::from(1_000_000_000_000_u64));
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn set_up_acc_for_benchmark(hotkey: &T::AccountId, coldkey: &T::AccountId) {
        Self::create_account_if_non_existent(coldkey, hotkey);
        Self::add_balance_to_coldkey_account(coldkey, TaoBalance::from(1_000_000_000_000_u64));
    }
}
