#![cfg_attr(not(feature = "std"), no_std)]
use core::ops::Neg;

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, CurrencyReserve, NetUid, TaoCurrency};

pub use order::*;

mod order;

pub trait SwapHandler<AccountId> {
    fn swap<OrderT, ReserveIn, ReserveOut>(
        netuid: NetUid,
        order: OrderT,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<OrderT>, DispatchError>
    where
        OrderT: Order,
        ReserveIn: CurrencyReserve<OrderT::PaidIn>,
        ReserveOut: CurrencyReserve<OrderT::PaidOut>,
        Self: SwapEngine<OrderT, ReserveIn, ReserveOut>;
    fn sim_swap<OrderT, ReserveIn, ReserveOut>(
        netuid: NetUid,
        order: OrderT,
    ) -> Result<SwapResult<OrderT>, DispatchError>
    where
        OrderT: Order,
        ReserveIn: CurrencyReserve<OrderT::PaidIn>,
        ReserveOut: CurrencyReserve<OrderT::PaidOut>,
        Self: DefaultPriceLimit<OrderT> + SwapEngine<OrderT, ReserveIn, ReserveOut>;
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

pub trait DefaultPriceLimit<OrderT: Order> {
    fn default_price_limit<C: Currency>() -> C;
}

pub trait SwapEngine<OrderT, ReserveIn, ReserveOut>
where
    OrderT: Order,
    ReserveIn: CurrencyReserve<OrderT::PaidIn>,
    ReserveOut: CurrencyReserve<OrderT::PaidOut>,
{
    fn swap(
        netuid: NetUid,
        order: OrderT,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<OrderT>, DispatchError>;
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct SwapResult<OrderT: Order> {
    pub amount_paid_in: OrderT::PaidIn,
    pub amount_paid_out: OrderT::PaidOut,
    pub fee_paid: OrderT::PaidIn,
}

impl<OrderT: Order> SwapResult<OrderT> {
    pub fn paid_in_reserve_delta(&self) -> i128 {
        self.amount_paid_in.to_u64() as i128
    }

    pub fn paid_out_reserve_delta(&self) -> i128 {
        (self.amount_paid_out.to_u64() as i128).neg()
    }
}
