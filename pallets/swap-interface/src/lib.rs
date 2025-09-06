#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

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
        drop_fees: bool,
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
    fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    );
    fn is_user_liquidity_enabled(netuid: NetUid) -> bool;
    fn dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult;
    fn try_initialize_v3(netuid: NetUid) -> DispatchResult;
}

#[derive(Debug, PartialEq)]
pub struct SwapResult {
    pub amount_paid_in: u64,
    pub amount_paid_out: u64,
    pub fee_paid: u64,
    // For calculation of new tao/alpha reserves
    pub tao_reserve_delta: i64,
    pub alpha_reserve_delta: i64,
}
