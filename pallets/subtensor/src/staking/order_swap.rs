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
    ) -> Result<AlphaBalance, DispatchError> {
        Self::stake_into_subnet(
            hotkey,
            coldkey,
            netuid,
            tao_amount,
            limit_price,
            false,
            false,
        )
    }

    fn sell_alpha(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        alpha_amount: AlphaBalance,
        limit_price: TaoBalance,
    ) -> Result<TaoBalance, DispatchError> {
        Self::unstake_from_subnet(hotkey, coldkey, netuid, alpha_amount, limit_price, false)
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
        intermediate_account: Option<T::AccountId>,
    ) -> DispatchResult {
        // Why not `transfer_stake_within_subnet`?
        //
        // 1. Silent no-op on insufficient balance — `decrease_stake_for_hotkey_and_coldkey_on_subnet`
        //    returns `()` without error when the coldkey has less stake than requested. Without the
        //    explicit `ensure!` below, the decrease would silently fail while the increase still
        //    runs, creating alpha out of thin air on the destination.
        //
        // 2. `AmountTooLow` minimum-stake check — `transfer_stake_within_subnet` rejects transfers
        //    whose TAO equivalent is below `DefaultMinStake`. Small pro-rata shares distributed to
        //    buyers in `distribute_alpha_pro_rata` are legitimate but can fall below that threshold,
        //    which would abort the entire batch.
        //
        // 3. Rate-limit (`StakingOperationRateLimitExceeded`) — `validate_stake_transition` (called
        //    via `do_transfer_stake`) checks `StakingOperationRateLimiter` on the origin account.
        //    The pallet intermediary account would be rate-limited after the first transfer per block.
        //
        // `LastColdkeyHotkeyStakeBlock` is updated for the destination after the transfer,
        // consistent with `transfer_stake_within_subnet`. It is a write-only observability item
        // (never read on-chain) but keeping it up-to-date is cheap and keeps off-chain indexers
        // accurate.

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
        Ok(())
    }
}
