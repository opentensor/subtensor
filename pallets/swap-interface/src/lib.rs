#![cfg_attr(not(feature = "std"), no_std)]
use core::ops::Neg;

use frame_support::pallet_prelude::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, CurrencyReserve, NetUid, TaoCurrency};

pub use order::*;

mod order;

pub trait SwapHandler<AccountId> {
    fn swap<PaidIn, PaidOut, ReserveIn, ReserveOut, OrderT>(
        netuid: NetUid,
        order: OrderT,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<PaidIn, PaidOut>, DispatchError>
    where
        PaidIn: Currency,
        PaidOut: Currency,
        ReserveIn: CurrencyReserve<PaidIn>,
        ReserveOut: CurrencyReserve<PaidOut>,
        OrderT: Order<PaidIn, PaidOut>,
        Self: SwapEngine<PaidIn, PaidOut, ReserveIn, ReserveOut, OrderT>;
    fn sim_swap<PaidIn, PaidOut, ReserveIn, ReserveOut, OrderT>(
        netuid: NetUid,
        order: OrderT,
        amount: PaidIn,
    ) -> Result<SwapResult<PaidIn, PaidOut>, DispatchError>
    where
        PaidIn: Currency,
        PaidOut: Currency,
        ReserveIn: CurrencyReserve<PaidIn>,
        ReserveOut: CurrencyReserve<PaidOut>,
        OrderT: Order<PaidIn, PaidOut>,
        Self: DefaultPriceLimit<PaidIn, PaidOut>
            + SwapEngine<PaidIn, PaidOut, ReserveIn, ReserveOut, OrderT>;
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

pub trait DefaultPriceLimit<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    fn default_price_limit<C: Currency>() -> C;
}

pub trait SwapEngine<PaidIn, PaidOut, ReserveIn, ReserveOut, OrderT>
where
    PaidIn: Currency,
    PaidOut: Currency,
    ReserveIn: CurrencyReserve<PaidIn>,
    ReserveOut: CurrencyReserve<PaidOut>,
    OrderT: Order<PaidIn, PaidOut>,
{
    fn swap(
        netuid: NetUid,
        order: OrderT,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<PaidIn, PaidOut>, DispatchError>;
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
}

impl<PaidIn, PaidOut> SwapResult<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    pub fn paid_in_reserve_delta(&self) -> i128 {
        self.amount_paid_in.to_u64() as i128
    }

    pub fn paid_out_reserve_delta(&self) -> i128 {
        (self.amount_paid_out.to_u64() as i128).neg()
    }
}
