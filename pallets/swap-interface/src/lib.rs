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
    fn sim_swap(netuid: u16, order_t: OrderType, amount: u64) -> Result<SwapResult, DispatchError>;
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

pub trait SubnetInfo<AccountId> {
    fn tao_reserve(netuid: u16) -> u64;
    fn alpha_reserve(netuid: u16) -> u64;
    fn exists(netuid: u16) -> bool;
    fn mechanism(netuid: u16) -> u16;
    fn is_owner(account_id: &AccountId, netuid: u16) -> bool;
}

pub trait BalanceOps<AccountId> {
    fn tao_balance(account_id: &AccountId) -> u64;
    fn alpha_balance(netuid: u16, coldkey: &AccountId, hotkey: &AccountId) -> u64;
    fn increase_balance(coldkey: &AccountId, tao: u64);
    fn decrease_balance(coldkey: &AccountId, tao: u64) -> Result<u64, DispatchError>;
    fn increase_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: u16,
        alpha: u64,
    ) -> Result<(), DispatchError>;
    fn decrease_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: u16,
        alpha: u64,
    ) -> Result<u64, DispatchError>;
}
