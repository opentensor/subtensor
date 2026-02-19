use core::marker::PhantomData;

use frame_support::ensure;
use safe_math::*;
use sp_core::Get;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, Currency, CurrencyReserve, NetUid, TaoCurrency};

use super::pallet::*;

/// A struct representing a single swap step with all its parameters and state
pub(crate) struct BasicSwapStep<T, PaidIn, PaidOut>
where
    T: Config,
    PaidIn: Currency,
    PaidOut: Currency,
{
    // Input parameters
    netuid: NetUid,
    drop_fees: bool,
    requested_delta_in: PaidIn,
    limit_price: U64F64,

    // Intermediate calculations
    target_price: U64F64,
    current_price: U64F64,

    // Result values
    delta_in: PaidIn,
    final_price: U64F64,
    fee: PaidIn,

    _phantom: PhantomData<(T, PaidIn, PaidOut)>,
}

impl<T, PaidIn, PaidOut> BasicSwapStep<T, PaidIn, PaidOut>
where
    T: Config,
    PaidIn: Currency,
    PaidOut: Currency,
    Self: SwapStep<T, PaidIn, PaidOut>,
{
    /// Creates and initializes a new swap step
    pub(crate) fn new(
        netuid: NetUid,
        amount_remaining: PaidIn,
        limit_price: U64F64,
        drop_fees: bool,
    ) -> Self {
        let fee = Pallet::<T>::calculate_fee_amount(netuid, amount_remaining, drop_fees);
        let requested_delta_in = amount_remaining.saturating_sub(fee);

        // Target and current prices
        let target_price = Self::price_target(netuid, requested_delta_in);
        let current_price = Pallet::<T>::current_price(netuid);

        Self {
            netuid,
            drop_fees,
            requested_delta_in,
            limit_price,
            target_price,
            current_price,
            delta_in: PaidIn::ZERO,
            final_price: target_price,
            fee,
            _phantom: PhantomData,
        }
    }

    /// Execute the swap step and return the result
    pub(crate) fn execute(&mut self) -> Result<SwapStepResult<PaidIn, PaidOut>, Error<T>> {
        self.determine_action();
        self.process_swap()
    }

    /// Determine the appropriate action for this swap step
    fn determine_action(&mut self) {
        let mut recalculate_fee = false;

        // Calculate the stopping price: The price at which we either reach the limit price,
        // or exchange the full amount.
        if Self::price_is_closer(&self.target_price, &self.limit_price) {
            // Case 1. target_quantity is the lowest, execute in full
            self.final_price = self.target_price;
            self.delta_in = self.requested_delta_in;
        } else {
            // Case 2. lim_quantity is the lowest
            self.final_price = self.limit_price;
            self.delta_in = Self::delta_in(self.netuid, self.current_price, self.limit_price);
            recalculate_fee = true;
        }

        log::trace!("\tCurrent Price    : {}", self.current_price);
        log::trace!("\tTarget Price     : {}", self.target_price);
        log::trace!("\tLimit Price      : {}", self.limit_price);
        log::trace!("\tDelta In         : {}", self.delta_in);

        // Because on step creation we calculate fee off the total amount, we might need to
        // recalculate it in case if we hit the limit price.
        if recalculate_fee {
            let u16_max = U64F64::saturating_from_num(u16::MAX);
            let fee_rate = if self.drop_fees {
                U64F64::saturating_from_num(0)
            } else {
                U64F64::saturating_from_num(FeeRate::<T>::get(self.netuid))
            };
            let delta_fixed = U64F64::saturating_from_num(self.delta_in);
            self.fee = delta_fixed
                .saturating_mul(fee_rate.safe_div(u16_max.saturating_sub(fee_rate)))
                .saturating_to_num::<u64>()
                .into();
        }
    }

    /// Process a single step of a swap
    fn process_swap(&self) -> Result<SwapStepResult<PaidIn, PaidOut>, Error<T>> {
        // Convert amounts, actual swap happens here
        let delta_out = Self::convert_deltas(self.netuid, self.delta_in);
        log::trace!("\tDelta Out        : {delta_out}");
        let mut fee_to_block_author = 0.into();
        if self.delta_in > 0.into() {
            ensure!(delta_out > 0.into(), Error::<T>::ReservesTooLow);

            // Split fees according to DefaultFeeSplit between liquidity pool and
            // validators. In case we want just to forward 100% of fees to the block
            // author, it can be done this way:
            // ```
            //     fee_to_block_author = self.fee;
            // ```
            let fee_split = DefaultFeeSplit::get();
            let lp_fee = fee_split.mul_floor(self.fee.to_u64()).into();
            Self::add_fees(self.netuid, lp_fee);
            fee_to_block_author = self.fee.saturating_sub(lp_fee);
        }

        Ok(SwapStepResult {
            fee_paid: self.fee,
            delta_in: self.delta_in,
            delta_out,
            fee_to_block_author,
        })
    }
}

impl<T: Config> SwapStep<T, TaoCurrency, AlphaCurrency>
    for BasicSwapStep<T, TaoCurrency, AlphaCurrency>
{
    fn delta_in(netuid: NetUid, price_curr: U64F64, price_target: U64F64) -> TaoCurrency {
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let balancer = SwapBalancer::<T>::get(netuid);
        TaoCurrency::from(balancer.calculate_quote_delta_in(
            price_curr,
            price_target,
            tao_reserve.into(),
        ))
    }

    fn price_target(netuid: NetUid, delta_in: TaoCurrency) -> U64F64 {
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());
        let balancer = SwapBalancer::<T>::get(netuid);
        let dy = delta_in;
        let dx = Self::convert_deltas(netuid, dy);
        balancer.calculate_price(
            u64::from(alpha_reserve.saturating_sub(dx)),
            u64::from(tao_reserve.saturating_add(dy)),
        )
    }

    fn price_is_closer(price1: &U64F64, price2: &U64F64) -> bool {
        price1 <= price2
    }

    fn add_fees(netuid: NetUid, fee: TaoCurrency) {
        FeesTao::<T>::mutate(netuid, |total| *total = total.saturating_add(fee))
    }

    fn convert_deltas(netuid: NetUid, delta_in: TaoCurrency) -> AlphaCurrency {
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let balancer = SwapBalancer::<T>::get(netuid);
        let e = balancer.exp_quote_base(tao_reserve.into(), delta_in.into());
        let one = U64F64::from_num(1);
        let alpha_reserve_fixed = U64F64::from_num(alpha_reserve);
        AlphaCurrency::from(
            alpha_reserve_fixed
                .saturating_mul(one.saturating_sub(e))
                .saturating_to_num::<u64>(),
        )
    }
}

impl<T: Config> SwapStep<T, AlphaCurrency, TaoCurrency>
    for BasicSwapStep<T, AlphaCurrency, TaoCurrency>
{
    fn delta_in(netuid: NetUid, price_curr: U64F64, price_target: U64F64) -> AlphaCurrency {
        let alpha_reserve = T::AlphaReserve::reserve(netuid);
        let balancer = SwapBalancer::<T>::get(netuid);
        AlphaCurrency::from(balancer.calculate_base_delta_in(
            price_curr,
            price_target,
            alpha_reserve.into(),
        ))
    }

    fn price_target(netuid: NetUid, delta_in: AlphaCurrency) -> U64F64 {
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());
        let balancer = SwapBalancer::<T>::get(netuid);
        let dx = delta_in;
        let dy = Self::convert_deltas(netuid, dx);
        balancer.calculate_price(
            u64::from(alpha_reserve.saturating_add(dx)),
            u64::from(tao_reserve.saturating_sub(dy)),
        )
    }

    fn price_is_closer(price1: &U64F64, price2: &U64F64) -> bool {
        price1 >= price2
    }

    fn add_fees(netuid: NetUid, fee: AlphaCurrency) {
        FeesAlpha::<T>::mutate(netuid, |total| *total = total.saturating_add(fee))
    }

    fn convert_deltas(netuid: NetUid, delta_in: AlphaCurrency) -> TaoCurrency {
        let alpha_reserve = T::AlphaReserve::reserve(netuid.into());
        let tao_reserve = T::TaoReserve::reserve(netuid.into());
        let balancer = SwapBalancer::<T>::get(netuid);
        let e = balancer.exp_base_quote(alpha_reserve.into(), delta_in.into());
        let one = U64F64::from_num(1);
        let tao_reserve_fixed = U64F64::from_num(u64::from(tao_reserve));
        TaoCurrency::from(
            tao_reserve_fixed
                .saturating_mul(one.saturating_sub(e))
                .saturating_to_num::<u64>(),
        )
    }
}

pub(crate) trait SwapStep<T, PaidIn, PaidOut>
where
    T: Config,
    PaidIn: Currency,
    PaidOut: Currency,
{
    /// Get the input amount needed to reach the target price
    fn delta_in(netuid: NetUid, price_curr: U64F64, price_target: U64F64) -> PaidIn;

    /// Get the target price based on the input amount
    fn price_target(netuid: NetUid, delta_in: PaidIn) -> U64F64;

    /// Returns True if price1 is closer to the current price than price2
    /// in terms of order direction.
    ///    For buying:  price1 <= price2
    ///    For selling: price1 >= price2
    fn price_is_closer(price1: &U64F64, price2: &U64F64) -> bool;

    /// Add fees to the global fee counters
    fn add_fees(netuid: NetUid, fee: PaidIn);

    /// Convert input amount (delta_in) to output amount (delta_out)
    ///
    /// This is the core method of the swap that tells how much output token is given for an
    /// amount of input token
    fn convert_deltas(netuid: NetUid, delta_in: PaidIn) -> PaidOut;
}

#[derive(Debug, PartialEq)]
pub(crate) struct SwapStepResult<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    pub(crate) fee_paid: PaidIn,
    pub(crate) delta_in: PaidIn,
    pub(crate) delta_out: PaidOut,
    pub(crate) fee_to_block_author: PaidIn,
}
