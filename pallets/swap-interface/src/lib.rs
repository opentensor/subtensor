#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, CurrencyReserve, NetUid, TaoCurrency};

pub mod order;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
}

pub trait SwapHandler<AccountId> {
    fn swap<PaidIn, PaidOut, ReserveIn, ReserveOut>(
        netuid: NetUid,
        order_t: OrderType,
        amount: PaidIn,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<PaidIn, PaidOut>, DispatchError>
    where
        PaidIn: Currency,
        PaidOut: Currency,
        ReserveIn: CurrencyReserve<PaidIn>,
        ReserveOut: CurrencyReserve<PaidOut>;
    fn sim_swap<PaidIn, PaidOut, ReserveIn, ReserveOut>(
        netuid: NetUid,
        order_t: OrderType,
        amount: PaidIn,
    ) -> Result<SwapResult<PaidIn, PaidOut>, DispatchError>
    where
        PaidIn: Currency,
        PaidOut: Currency,
        ReserveIn: CurrencyReserve<PaidIn>,
        ReserveOut: CurrencyReserve<PaidOut>;
    fn approx_fee_amount<T: Currency>(netuid: NetUid, amount: T) -> T;
    fn current_alpha_price(netuid: NetUid) -> U96F32;
    fn max_price<C: Currency>() -> C;
    fn min_price<C: Currency>() -> C;
    fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    );
    fn is_user_liquidity_enabled(netuid: NetUid) -> bool;
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SwapResult<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    pub amount_paid_in: PaidIn,
    pub amount_paid_out: PaidOut,
    pub fee_paid: PaidIn,
    // For calculation of new tao/alpha reserves
    pub tao_reserve_delta: i128,
    pub alpha_reserve_delta: i128,
}
