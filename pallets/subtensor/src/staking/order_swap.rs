use super::*;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};
use subtensor_swap_interface::{OrderSwapInterface, SwapHandler};
use substrate_fixed::types::U96F32;

impl<T: Config> OrderSwapInterface<T::AccountId> for Pallet<T> {
    fn buy_alpha(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        tao_amount: TaoBalance,
        limit_price: TaoBalance,
    ) -> Result<AlphaBalance, DispatchError> {
        Self::stake_into_subnet(hotkey, coldkey, netuid, tao_amount, limit_price, false, false)
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
}
