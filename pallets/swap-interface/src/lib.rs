#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
}

pub trait SwapHandler<AccountId> {
    fn swap(
        netuid: u16,
        order_t: OrderType,
        amount: u64,
        price_limit: u64,
        should_rollback: bool,
    ) -> Result<SwapResult, DispatchError>;
    fn approx_fee_amount(netuid: u16, amount: u64) -> u64;
    fn max_price() -> u64;
    fn min_price() -> u64;
}

#[derive(Debug, PartialEq)]
pub struct SwapResult {
    pub amount_paid_out: u64,
    pub fee_paid: u64,
    pub refund: u64,
    // calculated new tao/alpha reserves
    pub new_tao_reserve: u64,
    pub new_alpha_reserve: u64,
}

pub trait LiquidityDataProvider<AccountId> {
    fn tao_reserve(netuid: u16) -> u64;
    fn alpha_reserve(netuid: u16) -> u64;
    fn tao_balance(account_id: &AccountId) -> u64;
    fn alpha_balance(netuid: u16, coldkey_account_id: &AccountId, hotkey_account_id: &AccountId) -> u64;
}
