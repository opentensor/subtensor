use core::marker::PhantomData;

use safe_math::*;
use substrate_fixed::types::{I64F64, U64F64};
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};

use super::pallet::*;
use crate::{
    SqrtPrice,
    tick::{ActiveTickIndexManager, TickIndex},
};

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

    // Computed values
    current_liquidity: U64F64,
    possible_delta_in: PaidIn,

    // Ticks and prices (current, limit, edge, target)
    target_sqrt_price: SqrtPrice,
    limit_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    edge_sqrt_price: SqrtPrice,
    edge_tick: TickIndex,

    // Result values
    action: SwapStepAction,
    delta_in: PaidIn,
    final_price: SqrtPrice,
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
        limit_sqrt_price: SqrtPrice,
        drop_fees: bool,
    ) -> Self {
        // Calculate prices and ticks
        let current_tick = CurrentTick::<T>::get(netuid);
        let current_sqrt_price = AlphaSqrtPrice::<T>::get(netuid);
        let edge_tick = Self::tick_edge(netuid, current_tick);
        let edge_sqrt_price = edge_tick.as_sqrt_price_bounded();

        let fee = Pallet::<T>::calculate_fee_amount(netuid, amount_remaining, drop_fees);
        let possible_delta_in = amount_remaining.saturating_sub(fee);

        // Target price and quantities
        let current_liquidity = U64F64::saturating_from_num(CurrentLiquidity128::<T>::get(netuid));
        let target_sqrt_price =
            Self::sqrt_price_target(current_liquidity, current_sqrt_price, possible_delta_in);

        Self {
            netuid,
            drop_fees,
            target_sqrt_price,
            limit_sqrt_price,
            current_sqrt_price,
            edge_sqrt_price,
            edge_tick,
            possible_delta_in,
            current_liquidity,
            action: SwapStepAction::Stop,
            delta_in: PaidIn::ZERO,
            final_price: target_sqrt_price,
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
        // exchange the full amount, or reach the edge price.
        if Self::price_is_closer(&self.target_sqrt_price, &self.limit_sqrt_price)
            && Self::price_is_closer(&self.target_sqrt_price, &self.edge_sqrt_price)
        {
            // Case 1. target_quantity is the lowest
            // The trade completely happens within one tick, no tick crossing happens.
            self.action = SwapStepAction::Stop;
            self.final_price = self.target_sqrt_price;
            self.delta_in = self.possible_delta_in;
        } else if Self::price_is_closer(&self.limit_sqrt_price, &self.target_sqrt_price)
            && Self::price_is_closer(&self.limit_sqrt_price, &self.edge_sqrt_price)
        {
            // Case 2. lim_quantity is the lowest
            // The trade also completely happens within one tick, no tick crossing happens.
            self.action = SwapStepAction::Stop;
            self.final_price = self.limit_sqrt_price;
            self.delta_in = Self::delta_in(
                self.current_liquidity,
                self.current_sqrt_price,
                self.limit_sqrt_price,
            );
            recalculate_fee = true;
        } else {
            // Case 3. edge_quantity is the lowest
            // Tick crossing is likely
            self.action = SwapStepAction::Crossing;
            self.delta_in = Self::delta_in(
                self.current_liquidity,
                self.current_sqrt_price,
                self.edge_sqrt_price,
            );
            self.final_price = self.edge_sqrt_price;
            recalculate_fee = true;
        }

        log::trace!("\tAction           : {:?}", self.action);
        log::trace!(
            "\tCurrent Price    : {}",
            self.current_sqrt_price
                .saturating_mul(self.current_sqrt_price)
        );
        log::trace!(
            "\tTarget Price     : {}",
            self.target_sqrt_price
                .saturating_mul(self.target_sqrt_price)
        );
        log::trace!(
            "\tLimit Price      : {}",
            self.limit_sqrt_price.saturating_mul(self.limit_sqrt_price)
        );
        log::trace!(
            "\tEdge Price       : {}",
            self.edge_sqrt_price.saturating_mul(self.edge_sqrt_price)
        );
        log::trace!("\tDelta In         : {}", self.delta_in);

        // Because on step creation we calculate fee off the total amount, we might need to
        // recalculate it in case if we hit the limit price or the edge price.
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

        // Now correct the action if we stopped exactly at the edge no matter what was the case
        // above. Because order type buy moves the price up and tick semi-open interval doesn't
        // include its right point, we cross on buys and stop on sells.
        let natural_reason_stop_price =
            if Self::price_is_closer(&self.limit_sqrt_price, &self.target_sqrt_price) {
                self.limit_sqrt_price
            } else {
                self.target_sqrt_price
            };
        if natural_reason_stop_price == self.edge_sqrt_price {
            self.action = Self::action_on_edge_sqrt_price();
        }
    }

    /// Process a single step of a swap
    fn process_swap(&self) -> Result<SwapStepResult<PaidIn, PaidOut>, Error<T>> {
        // Hold the fees
        let current_liquidity = CurrentLiquidity128::<T>::get(self.netuid);
        Self::add_fees(self.netuid, current_liquidity, self.fee);
        let delta_out = Self::convert_deltas(self.netuid, self.delta_in);
        // log::trace!("\tDelta Out        : {delta_out:?}");

        if self.action == SwapStepAction::Crossing {
            let mut tick = Ticks128::<T>::get(self.netuid, self.edge_tick).unwrap_or_default();
            tick.fees_out_tao = I64F64::saturating_from_num(FeeGlobalTao::<T>::get(self.netuid))
                .saturating_sub(tick.fees_out_tao);
            tick.fees_out_alpha =
                I64F64::saturating_from_num(FeeGlobalAlpha::<T>::get(self.netuid))
                    .saturating_sub(tick.fees_out_alpha);
            Self::update_liquidity_at_crossing(self.netuid)?;
            Ticks128::<T>::insert(self.netuid, self.edge_tick, tick);
        }

        // Update current price
        AlphaSqrtPrice::<T>::set(self.netuid, self.final_price);

        // Update current tick
        let new_current_tick = TickIndex::from_sqrt_price_bounded(self.final_price);
        CurrentTick::<T>::set(self.netuid, new_current_tick);

        Ok(SwapStepResult {
            amount_to_take: self.delta_in.saturating_add(self.fee),
            fee_paid: self.fee,
            delta_in: self.delta_in,
            delta_out,
        })
    }

    pub(crate) fn action(&self) -> SwapStepAction {
        self.action
    }
}

impl<T: Config> SwapStep<T, TaoCurrency, AlphaCurrency>
    for BasicSwapStep<T, TaoCurrency, AlphaCurrency>
{
    fn delta_in(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        sqrt_price_target: SqrtPrice,
    ) -> TaoCurrency {
        liquidity_curr
            .saturating_mul(sqrt_price_target.saturating_sub(sqrt_price_curr))
            .saturating_to_num::<u64>()
            .into()
    }

    fn tick_edge(netuid: NetUid, current_tick: TickIndex) -> TickIndex {
        ActiveTickIndexManager::<T>::find_closest_higher(
            netuid,
            current_tick.next().unwrap_or(TickIndex::MAX),
        )
        .unwrap_or(TickIndex::MAX)
    }

    fn sqrt_price_target(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        delta_in: TaoCurrency,
    ) -> SqrtPrice {
        let delta_fixed = U64F64::saturating_from_num(delta_in);

        // No liquidity means that price should go to the limit
        if liquidity_curr == 0 {
            return SqrtPrice::saturating_from_num(
                Pallet::<T>::max_price_inner::<TaoCurrency>().to_u64(),
            );
        }

        delta_fixed
            .safe_div(liquidity_curr)
            .saturating_add(sqrt_price_curr)
    }

    fn price_is_closer(sq_price1: &SqrtPrice, sq_price2: &SqrtPrice) -> bool {
        sq_price1 <= sq_price2
    }

    fn action_on_edge_sqrt_price() -> SwapStepAction {
        SwapStepAction::Crossing
    }

    fn add_fees(netuid: NetUid, current_liquidity: u128, fee: TaoCurrency) {
        if current_liquidity == 0 {
            return;
        }

        if current_liquidity <= u64::MAX as u128 {
            let fee_fixed = U64F64::saturating_from_num(fee.to_u64());
            let current_liquidity_fixed = U64F64::saturating_from_num(current_liquidity);
            FeeGlobalTao::<T>::mutate(netuid, |value| {
                *value = value.saturating_add(fee_fixed.safe_div(current_liquidity_fixed))
            });
        } else {
            // Decompose current_liquidity = hi * 2^64 + lo
            const TWO_POW_64: u128 = 1u128 << 64;
            let hi: u64 = (current_liquidity.safe_div(TWO_POW_64)) as u64;
            let lo: u64 = (current_liquidity
                .checked_rem(TWO_POW_64)
                .unwrap_or_default()) as u64;

            // Build liquidity_factor = 2^-64 / (hi + lo/2^64) in U64F64
            let s: U64F64 = U64F64::saturating_from_num(hi).saturating_add(
                U64F64::saturating_from_num(lo).safe_div(U64F64::saturating_from_num(TWO_POW_64)),
            );
            // 2^-64 = from_bits(1)
            let liquidity_factor: U64F64 = U64F64::from_bits(1).safe_div(s);

            // result = fee * inv
            let fee_delta = U64F64::saturating_from_num(fee).saturating_mul(liquidity_factor);
            FeeGlobalTao::<T>::mutate(netuid, |value| *value = value.saturating_add(fee_delta));
        }
    }

    fn convert_deltas(netuid: NetUid, delta_in: TaoCurrency) -> AlphaCurrency {
        // Skip conversion if delta_in is zero
        if delta_in.is_zero() {
            return AlphaCurrency::ZERO;
        }

        let liquidity_curr = SqrtPrice::saturating_from_num(CurrentLiquidity128::<T>::get(netuid));
        let sqrt_price_curr = AlphaSqrtPrice::<T>::get(netuid);
        let delta_fixed = SqrtPrice::saturating_from_num(delta_in.to_u64());

        // Calculate result based on order type with proper fixed-point math
        // Using safe math operations throughout to prevent overflows
        let result = {
            // (liquidity_curr * sqrt_price_curr + delta_fixed) * sqrt_price_curr;
            let a = liquidity_curr
                .saturating_mul(sqrt_price_curr)
                .saturating_add(delta_fixed)
                .saturating_mul(sqrt_price_curr);
            // liquidity_curr / a;
            let b = liquidity_curr.safe_div(a);
            // b * delta_fixed;
            b.saturating_mul(delta_fixed)
        };

        result.saturating_to_num::<u64>().into()
    }

    fn update_liquidity_at_crossing(netuid: NetUid) -> Result<(), Error<T>> {
        let mut liquidity_curr = CurrentLiquidity128::<T>::get(netuid);
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);

        // Find the appropriate tick based on order type
        let tick = {
            // Self::find_closest_higher_active_tick(netuid, current_tick_index),
            let upper_tick = ActiveTickIndexManager::<T>::find_closest_higher(
                netuid,
                current_tick_index.next().unwrap_or(TickIndex::MAX),
            )
            .unwrap_or(TickIndex::MAX);
            Ticks128::<T>::get(netuid, upper_tick)
        }
        .ok_or(Error::<T>::InsufficientLiquidity)?;

        let liquidity_update_abs_u128 = tick.liquidity_net_as_u128();

        // Update liquidity based on the sign of liquidity_net and the order type
        liquidity_curr = if tick.liquidity_net >= 0 {
            liquidity_curr.saturating_add(liquidity_update_abs_u128)
        } else {
            liquidity_curr.saturating_sub(liquidity_update_abs_u128)
        };

        CurrentLiquidity128::<T>::set(netuid, liquidity_curr);

        Ok(())
    }
}

impl<T: Config> SwapStep<T, AlphaCurrency, TaoCurrency>
    for BasicSwapStep<T, AlphaCurrency, TaoCurrency>
{
    fn delta_in(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        sqrt_price_target: SqrtPrice,
    ) -> AlphaCurrency {
        let one = U64F64::saturating_from_num(1);

        liquidity_curr
            .saturating_mul(
                one.safe_div(sqrt_price_target.into())
                    .saturating_sub(one.safe_div(sqrt_price_curr)),
            )
            .saturating_to_num::<u64>()
            .into()
    }

    fn tick_edge(netuid: NetUid, current_tick: TickIndex) -> TickIndex {
        let current_price: SqrtPrice = AlphaSqrtPrice::<T>::get(netuid);
        let current_tick_price = current_tick.as_sqrt_price_bounded();
        let is_active = ActiveTickIndexManager::<T>::tick_is_active(netuid, current_tick);

        if is_active && current_price > current_tick_price {
            return ActiveTickIndexManager::<T>::find_closest_lower(netuid, current_tick)
                .unwrap_or(TickIndex::MIN);
        }

        ActiveTickIndexManager::<T>::find_closest_lower(
            netuid,
            current_tick.prev().unwrap_or(TickIndex::MIN),
        )
        .unwrap_or(TickIndex::MIN)
    }

    fn sqrt_price_target(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        delta_in: AlphaCurrency,
    ) -> SqrtPrice {
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let one = U64F64::saturating_from_num(1);

        // No liquidity means that price should go to the limit
        if liquidity_curr == 0 {
            return SqrtPrice::saturating_from_num(
                Pallet::<T>::min_price_inner::<AlphaCurrency>().to_u64(),
            );
        }

        one.safe_div(
            delta_fixed
                .safe_div(liquidity_curr)
                .saturating_add(one.safe_div(sqrt_price_curr)),
        )
    }

    fn price_is_closer(sq_price1: &SqrtPrice, sq_price2: &SqrtPrice) -> bool {
        sq_price1 >= sq_price2
    }

    fn action_on_edge_sqrt_price() -> SwapStepAction {
        SwapStepAction::Stop
    }

    fn add_fees(netuid: NetUid, current_liquidity: u128, fee: AlphaCurrency) {
        if current_liquidity == 0 {
            return;
        }

        if current_liquidity <= u64::MAX as u128 {
            let fee_fixed = U64F64::saturating_from_num(fee.to_u64());
            let current_liquidity_fixed = U64F64::saturating_from_num(current_liquidity);
            FeeGlobalAlpha::<T>::mutate(netuid, |value| {
                *value = value.saturating_add(fee_fixed.safe_div(current_liquidity_fixed))
            });
        } else {
            // Decompose current_liquidity = hi * 2^64 + lo
            const TWO_POW_64: u128 = 1u128 << 64;
            let hi: u64 = (current_liquidity.safe_div(TWO_POW_64)) as u64;
            let lo: u64 = (current_liquidity
                .checked_rem(TWO_POW_64)
                .unwrap_or_default()) as u64;

            // Build liquidity_factor = 2^-64 / (hi + lo/2^64) in U64F64
            let s: U64F64 = U64F64::saturating_from_num(hi).saturating_add(
                U64F64::saturating_from_num(lo).safe_div(U64F64::saturating_from_num(TWO_POW_64)),
            );
            // 2^-64 = from_bits(1)
            let liquidity_factor: U64F64 = U64F64::from_bits(1).safe_div(s);

            // result = fee * inv
            let fee_delta = U64F64::saturating_from_num(fee).saturating_mul(liquidity_factor);
            FeeGlobalAlpha::<T>::mutate(netuid, |value| *value = value.saturating_add(fee_delta));
        }
    }

    fn convert_deltas(netuid: NetUid, delta_in: AlphaCurrency) -> TaoCurrency {
        // Skip conversion if delta_in is zero
        if delta_in.is_zero() {
            return TaoCurrency::ZERO;
        }

        let liquidity_curr = SqrtPrice::saturating_from_num(CurrentLiquidity128::<T>::get(netuid));
        let sqrt_price_curr = AlphaSqrtPrice::<T>::get(netuid);
        let delta_fixed = SqrtPrice::saturating_from_num(delta_in.to_u64());

        // Calculate result based on order type with proper fixed-point math
        // Using safe math operations throughout to prevent overflows
        let result = {
            // liquidity_curr / (liquidity_curr / sqrt_price_curr + delta_fixed);
            let denom = liquidity_curr
                .safe_div(sqrt_price_curr)
                .saturating_add(delta_fixed);
            let a = liquidity_curr.safe_div(denom);
            // a * sqrt_price_curr;
            let b = a.saturating_mul(sqrt_price_curr);

            // delta_fixed * b;
            delta_fixed.saturating_mul(b)
        };

        result.saturating_to_num::<u64>().into()
    }

    fn update_liquidity_at_crossing(netuid: NetUid) -> Result<(), Error<T>> {
        let mut liquidity_curr = CurrentLiquidity128::<T>::get(netuid);
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);

        // Find the appropriate tick based on order type
        let tick = {
            // Self::find_closest_lower_active_tick(netuid, current_tick_index)
            let current_price = AlphaSqrtPrice::<T>::get(netuid);
            let current_tick_price = current_tick_index.as_sqrt_price_bounded();
            let is_active = ActiveTickIndexManager::<T>::tick_is_active(netuid, current_tick_index);

            let lower_tick = if is_active && current_price > current_tick_price {
                ActiveTickIndexManager::<T>::find_closest_lower(netuid, current_tick_index)
                    .unwrap_or(TickIndex::MIN)
            } else {
                ActiveTickIndexManager::<T>::find_closest_lower(
                    netuid,
                    current_tick_index.prev().unwrap_or(TickIndex::MIN),
                )
                .unwrap_or(TickIndex::MIN)
            };
            Ticks128::<T>::get(netuid, lower_tick)
        }
        .ok_or(Error::<T>::InsufficientLiquidity)?;

        let liquidity_update_abs_u128 = tick.liquidity_net_as_u128();

        // Update liquidity based on the sign of liquidity_net and the order type
        liquidity_curr = if tick.liquidity_net >= 0 {
            liquidity_curr.saturating_sub(liquidity_update_abs_u128)
        } else {
            liquidity_curr.saturating_add(liquidity_update_abs_u128)
        };

        CurrentLiquidity128::<T>::set(netuid, liquidity_curr);

        Ok(())
    }
}

pub(crate) trait SwapStep<T, PaidIn, PaidOut>
where
    T: Config,
    PaidIn: Currency,
    PaidOut: Currency,
{
    /// Get the input amount needed to reach the target price
    fn delta_in(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        sqrt_price_target: SqrtPrice,
    ) -> PaidIn;

    /// Get the tick at the current tick edge.
    ///
    /// If anything is wrong with tick math and it returns Err, we just abort the deal, i.e. return
    /// the edge that is impossible to execute
    fn tick_edge(netuid: NetUid, current_tick: TickIndex) -> TickIndex;

    /// Get the target square root price based on the input amount
    ///
    /// This is the price that would be reached if
    ///    - There are no liquidity positions other than protocol liquidity
    ///    - Full delta_in amount is executed
    fn sqrt_price_target(
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        delta_in: PaidIn,
    ) -> SqrtPrice;

    /// Returns True if sq_price1 is closer to the current price than sq_price2
    /// in terms of order direction.
    ///    For buying:  sq_price1 <= sq_price2
    ///    For selling: sq_price1 >= sq_price2
    fn price_is_closer(sq_price1: &SqrtPrice, sq_price2: &SqrtPrice) -> bool;

    /// Get swap step action on the edge sqrt price.
    fn action_on_edge_sqrt_price() -> SwapStepAction;

    /// Add fees to the global fee counters
    fn add_fees(netuid: NetUid, current_liquidity: u128, fee: PaidIn);

    /// Convert input amount (delta_in) to output amount (delta_out)
    ///
    /// This is the core method of uniswap V3 that tells how much output token is given for an
    /// amount of input token within one price tick.
    fn convert_deltas(netuid: NetUid, delta_in: PaidIn) -> PaidOut;

    /// Update liquidity when crossing a tick
    fn update_liquidity_at_crossing(netuid: NetUid) -> Result<(), Error<T>>;
}

#[derive(Debug, PartialEq)]
pub(crate) struct SwapStepResult<PaidIn, PaidOut>
where
    PaidIn: Currency,
    PaidOut: Currency,
{
    pub(crate) amount_to_take: PaidIn,
    pub(crate) fee_paid: PaidIn,
    pub(crate) delta_in: PaidIn,
    pub(crate) delta_out: PaidOut,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwapStepAction {
    Crossing,
    Stop,
}
