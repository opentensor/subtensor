#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;

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
    fn add_liquidity(
        netuid: u16,
        coldkey_account_id: &AccountId,
        hotkey_account_id: &AccountId,
        tick_low: i32,
        tick_high: i32,
        liquidity: u64,
    ) -> Result<(u128, u64, u64), DispatchError>;
    fn remove_liquidity(
        netuid: u16,
        coldkey_account_id: &AccountId,
        position_id: u128,
    ) -> Result<UpdateLiquidityResult, DispatchError>;
    fn modify_position(
        netuid: u16,
        coldkey_account_id: &AccountId,
        hotkey_account_id: &AccountId,
        position_id: u128,
        liquidity_delta: i64,
    ) -> Result<UpdateLiquidityResult, DispatchError>;
    fn approx_fee_amount(netuid: u16, amount: u64) -> u64;
    fn current_alpha_price(netuid: u16) -> U96F32;
    fn max_price() -> u64;
    fn min_price() -> u64;
}

#[derive(Debug, PartialEq)]
pub struct SwapResult {
    pub amount_paid_in: u64,
    pub amount_paid_out: u64,
    pub fee_paid: u64,
    // calculated new tao/alpha reserves
    pub new_tao_reserve: u64,
    pub new_alpha_reserve: u64,
}

#[derive(Debug, PartialEq)]
pub struct UpdateLiquidityResult {
    pub tao: u64,
    pub alpha: u64,
    pub fee_tao: u64,
    pub fee_alpha: u64,
}

pub trait LiquidityDataProvider<AccountId> {
    fn tao_reserve(netuid: u16) -> u64;
    fn alpha_reserve(netuid: u16) -> u64;
    fn tao_balance(account_id: &AccountId) -> u64;
    fn alpha_balance(
        netuid: u16,
        coldkey_account_id: &AccountId,
        hotkey_account_id: &AccountId,
    ) -> u64;
    fn subnet_mechanism(netuid: u16) -> u16;
}
