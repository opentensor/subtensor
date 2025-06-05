#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::NetUid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
}

pub trait SwapHandler<AccountId> {
    fn swap(
        netuid: NetUid,
        order_t: OrderType,
        amount: u64,
        price_limit: u64,
        should_rollback: bool,
    ) -> Result<SwapResult, DispatchError>;
    fn sim_swap(
        netuid: NetUid,
        order_t: OrderType,
        amount: u64,
    ) -> Result<SwapResult, DispatchError>;
    fn approx_fee_amount(netuid: NetUid, amount: u64) -> u64;
    fn current_alpha_price(netuid: NetUid) -> U96F32;
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
