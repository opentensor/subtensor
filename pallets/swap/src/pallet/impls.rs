use core::marker::PhantomData;
use core::ops::Neg;

use frame_support::storage::{TransactionOutcome, transactional};
use frame_support::{ensure, pallet_prelude::DispatchError, traits::Get};
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use sp_runtime::{DispatchResult, Vec, traits::AccountIdConversion};
use substrate_fixed::types::{I64F64, U64F64, U96F32};
use subtensor_runtime_common::{
    AlphaCurrency, BalanceOps, Currency, NetUid, SubnetInfo, TaoCurrency,
};
use subtensor_swap_interface::{SwapHandler, SwapResult};

use super::pallet::*;
use crate::{
    OrderType, SqrtPrice,
    position::{Position, PositionId},
    tick::{ActiveTickIndexManager, Tick, TickIndex},
};

const MAX_SWAP_ITERATIONS: u16 = 1000;

#[derive(Debug, PartialEq)]
pub struct UpdateLiquidityResult {
    pub tao: TaoCurrency,
    pub alpha: AlphaCurrency,
    pub fee_tao: TaoCurrency,
    pub fee_alpha: AlphaCurrency,
    pub removed: bool,
    pub tick_low: TickIndex,
    pub tick_high: TickIndex,
}

#[derive(Debug, PartialEq)]
pub struct RemoveLiquidityResult {
    pub tao: TaoCurrency,
    pub alpha: AlphaCurrency,
    pub fee_tao: TaoCurrency,
    pub fee_alpha: AlphaCurrency,
    pub tick_low: TickIndex,
    pub tick_high: TickIndex,
    pub liquidity: u64,
}
/// A struct representing a single swap step with all its parameters and state
struct SwapStep<T: frame_system::Config> {
    // Input parameters
    netuid: NetUid,
    order_type: OrderType,
    drop_fees: bool,

    // Computed values
    current_liquidity: U64F64,
    possible_delta_in: u64,

    // Ticks and prices (current, limit, edge, target)
    target_sqrt_price: SqrtPrice,
    limit_sqrt_price: SqrtPrice,
    current_sqrt_price: SqrtPrice,
    edge_sqrt_price: SqrtPrice,
    edge_tick: TickIndex,

    // Result values
    action: SwapStepAction,
    delta_in: u64,
    final_price: SqrtPrice,
    fee: u64,

    // Phantom data to use T
    _phantom: PhantomData<T>,
}

impl<T: Config> SwapStep<T> {
    /// Creates and initializes a new swap step
    fn new(
        netuid: NetUid,
        order_type: OrderType,
        amount_remaining: u64,
        limit_sqrt_price: SqrtPrice,
        drop_fees: bool,
    ) -> Self {
        // Calculate prices and ticks
        let current_tick = CurrentTick::<T>::get(netuid);
        let current_sqrt_price = Pallet::<T>::current_price_sqrt(netuid);
        let edge_tick = Pallet::<T>::tick_edge(netuid, current_tick, order_type);
        let edge_sqrt_price = edge_tick.as_sqrt_price_bounded();

        let fee = Pallet::<T>::calculate_fee_amount(netuid, amount_remaining, drop_fees);
        let possible_delta_in = amount_remaining.saturating_sub(fee);

        // Target price and quantities
        let current_liquidity = U64F64::saturating_from_num(CurrentLiquidity::<T>::get(netuid));
        let target_sqrt_price = Pallet::<T>::sqrt_price_target(
            order_type,
            current_liquidity,
            current_sqrt_price,
            possible_delta_in,
        );

        Self {
            netuid,
            order_type,
            drop_fees,
            target_sqrt_price,
            limit_sqrt_price,
            current_sqrt_price,
            edge_sqrt_price,
            edge_tick,
            possible_delta_in,
            current_liquidity,
            action: SwapStepAction::Stop,
            delta_in: 0,
            final_price: target_sqrt_price,
            fee,
            _phantom: PhantomData,
        }
    }

    /// Execute the swap step and return the result
    fn execute(&mut self) -> Result<SwapStepResult, Error<T>> {
        self.determine_action();
        self.process_swap()
    }

    /// Returns True if sq_price1 is closer to the current price than sq_price2
    /// in terms of order direction.
    ///    For buying:  sq_price1 <= sq_price2
    ///    For selling: sq_price1 >= sq_price2
    ///
    fn price_is_closer(&self, sq_price1: &SqrtPrice, sq_price2: &SqrtPrice) -> bool {
        match self.order_type {
            OrderType::Buy => sq_price1 <= sq_price2,
            OrderType::Sell => sq_price1 >= sq_price2,
        }
    }

    /// Determine the appropriate action for this swap step
    fn determine_action(&mut self) {
        let mut recalculate_fee = false;

        // Calculate the stopping price: The price at which we either reach the limit price,
        // exchange the full amount, or reach the edge price.
        if self.price_is_closer(&self.target_sqrt_price, &self.limit_sqrt_price)
            && self.price_is_closer(&self.target_sqrt_price, &self.edge_sqrt_price)
        {
            // Case 1. target_quantity is the lowest
            // The trade completely happens within one tick, no tick crossing happens.
            self.action = SwapStepAction::Stop;
            self.final_price = self.target_sqrt_price;
            self.delta_in = self.possible_delta_in;
        } else if self.price_is_closer(&self.limit_sqrt_price, &self.target_sqrt_price)
            && self.price_is_closer(&self.limit_sqrt_price, &self.edge_sqrt_price)
        {
            // Case 2. lim_quantity is the lowest
            // The trade also completely happens within one tick, no tick crossing happens.
            self.action = SwapStepAction::Stop;
            self.final_price = self.limit_sqrt_price;
            self.delta_in = Self::delta_in(
                self.order_type,
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
                self.order_type,
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

        // Because on step creation we calculate fee off the total amount, we might need to recalculate it
        // in case if we hit the limit price or the edge price.
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
                .saturating_to_num::<u64>();
        }

        // Now correct the action if we stopped exactly at the edge no matter what was the case above
        // Because order type buy moves the price up and tick semi-open interval doesn't include its right
        // point, we cross on buys and stop on sells.
        let natural_reason_stop_price =
            if self.price_is_closer(&self.limit_sqrt_price, &self.target_sqrt_price) {
                self.limit_sqrt_price
            } else {
                self.target_sqrt_price
            };
        if natural_reason_stop_price == self.edge_sqrt_price {
            self.action = match self.order_type {
                OrderType::Buy => SwapStepAction::Crossing,
                OrderType::Sell => SwapStepAction::Stop,
            };
        }
    }

    /// Process a single step of a swap
    fn process_swap(&self) -> Result<SwapStepResult, Error<T>> {
        // Hold the fees
        Pallet::<T>::add_fees(self.netuid, self.order_type, self.fee);
        let delta_out = Pallet::<T>::convert_deltas(self.netuid, self.order_type, self.delta_in);
        log::trace!("\tDelta Out        : {delta_out:?}");

        if self.action == SwapStepAction::Crossing {
            let mut tick = Ticks::<T>::get(self.netuid, self.edge_tick).unwrap_or_default();
            tick.fees_out_tao = I64F64::saturating_from_num(FeeGlobalTao::<T>::get(self.netuid))
                .saturating_sub(tick.fees_out_tao);
            tick.fees_out_alpha =
                I64F64::saturating_from_num(FeeGlobalAlpha::<T>::get(self.netuid))
                    .saturating_sub(tick.fees_out_alpha);
            Pallet::<T>::update_liquidity_at_crossing(self.netuid, self.order_type)?;
            Ticks::<T>::insert(self.netuid, self.edge_tick, tick);
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

    /// Get the input amount needed to reach the target price
    fn delta_in(
        order_type: OrderType,
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        sqrt_price_target: SqrtPrice,
    ) -> u64 {
        let one = U64F64::saturating_from_num(1);

        (match order_type {
            OrderType::Sell => liquidity_curr.saturating_mul(
                one.safe_div(sqrt_price_target.into())
                    .saturating_sub(one.safe_div(sqrt_price_curr)),
            ),
            OrderType::Buy => {
                liquidity_curr.saturating_mul(sqrt_price_target.saturating_sub(sqrt_price_curr))
            }
        })
        .saturating_to_num::<u64>()
    }
}

impl<T: Config> Pallet<T> {
    pub fn current_price(netuid: NetUid) -> U96F32 {
        match T::SubnetInfo::mechanism(netuid.into()) {
            1 => {
                if SwapV3Initialized::<T>::get(netuid) {
                    let sqrt_price = AlphaSqrtPrice::<T>::get(netuid);
                    U96F32::saturating_from_num(sqrt_price.saturating_mul(sqrt_price))
                } else {
                    let tao_reserve = T::SubnetInfo::tao_reserve(netuid.into());
                    let alpha_reserve = T::SubnetInfo::alpha_reserve(netuid.into());
                    if !alpha_reserve.is_zero() {
                        U96F32::saturating_from_num(tao_reserve)
                            .saturating_div(U96F32::saturating_from_num(alpha_reserve))
                    } else {
                        U96F32::saturating_from_num(0)
                    }
                }
            }
            _ => U96F32::saturating_from_num(1),
        }
    }

    pub fn current_price_sqrt(netuid: NetUid) -> SqrtPrice {
        AlphaSqrtPrice::<T>::get(netuid)
    }

    // initializes V3 swap for a subnet if needed
    pub(super) fn maybe_initialize_v3(netuid: NetUid) -> Result<(), Error<T>> {
        if SwapV3Initialized::<T>::get(netuid) {
            return Ok(());
        }

        // Initialize the v3:
        // Reserves are re-purposed, nothing to set, just query values for liquidity and price calculation
        let tao_reserve = <T as Config>::SubnetInfo::tao_reserve(netuid.into());
        let alpha_reserve = <T as Config>::SubnetInfo::alpha_reserve(netuid.into());

        // Set price
        let price = U64F64::saturating_from_num(tao_reserve)
            .safe_div(U64F64::saturating_from_num(alpha_reserve));

        let epsilon = U64F64::saturating_from_num(0.000000000001);

        let current_sqrt_price = price.checked_sqrt(epsilon).unwrap_or(U64F64::from_num(0));
        AlphaSqrtPrice::<T>::set(netuid, current_sqrt_price);

        // Set current tick
        let current_tick = TickIndex::from_sqrt_price_bounded(current_sqrt_price);
        CurrentTick::<T>::set(netuid, current_tick);

        // Set initial (protocol owned) liquidity and positions
        // Protocol liquidity makes one position from TickIndex::MIN to TickIndex::MAX
        // We are using the sp_arithmetic sqrt here, which works for u128
        let liquidity = helpers_128bit::sqrt(
            (tao_reserve.to_u64() as u128).saturating_mul(alpha_reserve.to_u64() as u128),
        ) as u64;
        let protocol_account_id = Self::protocol_account_id();

        let (position, _, _) = Self::add_liquidity_not_insert(
            netuid,
            &protocol_account_id,
            TickIndex::MIN,
            TickIndex::MAX,
            liquidity,
        )?;

        Positions::<T>::insert(&(netuid, protocol_account_id, position.id), position);

        Ok(())
    }

    /// Adjusts protocol liquidity with new values of TAO and Alpha reserve
    pub(super) fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    ) {
        // Update protocol position with new liquidity
        let protocol_account_id = Self::protocol_account_id();
        let mut positions =
            Positions::<T>::iter_prefix_values((netuid, protocol_account_id.clone()))
                .collect::<sp_std::vec::Vec<_>>();

        if let Some(position) = positions.get_mut(0) {
            // Claim protocol fees and add them to liquidity
            let (tao_fees, alpha_fees) = position.collect_fees();

            // Adjust liquidity
            let current_sqrt_price = Pallet::<T>::current_price_sqrt(netuid);
            let maybe_token_amounts = position.to_token_amounts(current_sqrt_price);
            if let Ok((tao, alpha)) = maybe_token_amounts {
                // Get updated reserves, calculate liquidity
                let new_tao_reserve = tao
                    .saturating_add(tao_delta.to_u64())
                    .saturating_add(tao_fees);
                let new_alpha_reserve = alpha
                    .saturating_add(alpha_delta.to_u64())
                    .saturating_add(alpha_fees);
                let new_liquidity = helpers_128bit::sqrt(
                    (new_tao_reserve as u128).saturating_mul(new_alpha_reserve as u128),
                ) as u64;
                let liquidity_delta = new_liquidity.saturating_sub(position.liquidity);

                // Update current liquidity
                CurrentLiquidity::<T>::mutate(netuid, |current_liquidity| {
                    *current_liquidity = current_liquidity.saturating_add(liquidity_delta);
                });

                // Update protocol position
                position.liquidity = new_liquidity;
                Positions::<T>::insert(
                    (netuid, protocol_account_id, position.id),
                    position.clone(),
                );

                // Update position ticks
                Self::add_liquidity_at_index(netuid, position.tick_low, liquidity_delta, false);
                Self::add_liquidity_at_index(netuid, position.tick_high, liquidity_delta, true);
            }
        }
    }

    /// Executes a token swap on the specified subnet.
    ///
    /// # Parameters
    /// - `netuid`: The identifier of the subnet on which the swap is performed.
    /// - `order_type`: The type of the swap (e.g., Buy or Sell).
    /// - `amount`: The amount of tokens to swap.
    /// - `limit_sqrt_price`: A price limit (expressed as a square root) to bound the swap.
    /// - `simulate`: If `true`, the function runs in simulation mode and does not persist any changes.
    ///
    /// # Returns
    /// Returns a [`Result`] with a [`SwapResult`] on success, or a [`DispatchError`] on failure.
    ///
    /// The [`SwapResult`] contains:
    /// - `amount_paid_out`: The amount of tokens received from the swap.
    /// - `refund`: Any unswapped portion of the input amount, refunded to the caller.
    ///
    /// # Simulation Mode
    /// When `simulate` is set to `true`, the function:
    /// 1. Executes all logic without persisting any state changes (i.e., performs a dry run).
    /// 2. Skips reserve checks — it may return an `amount_paid_out` greater than the available reserve.
    ///
    /// Use simulation mode to preview the outcome of a swap without modifying the blockchain state.
    pub fn do_swap(
        netuid: NetUid,
        order_type: OrderType,
        amount: u64,
        limit_sqrt_price: SqrtPrice,
        drop_fees: bool,
        simulate: bool,
    ) -> Result<SwapResult, DispatchError> {
        transactional::with_transaction(|| {
            // Read alpha and tao reserves before transaction
            let tao_reserve = T::SubnetInfo::tao_reserve(netuid.into());
            let alpha_reserve = T::SubnetInfo::alpha_reserve(netuid.into());

            let mut result =
                Self::swap_inner(netuid, order_type, amount, limit_sqrt_price, drop_fees)
                    .map_err(Into::into);

            if simulate || result.is_err() {
                // Simulation only
                TransactionOutcome::Rollback(result)
            } else {
                // Should persist changes

                // Check if reserves are overused
                if let Ok(ref swap_result) = result {
                    let checked_reserve = match order_type {
                        OrderType::Buy => alpha_reserve.to_u64(),
                        OrderType::Sell => tao_reserve.to_u64(),
                    };

                    if checked_reserve < swap_result.amount_paid_out {
                        result = Err(Error::<T>::InsufficientLiquidity.into());
                    }
                }

                TransactionOutcome::Commit(result)
            }
        })
    }

    fn swap_inner(
        netuid: NetUid,
        order_type: OrderType,
        amount: u64,
        limit_sqrt_price: SqrtPrice,
        drop_fees: bool,
    ) -> Result<SwapResult, Error<T>> {
        match order_type {
            OrderType::Buy => ensure!(
                T::SubnetInfo::alpha_reserve(netuid.into()).to_u64()
                    >= T::MinimumReserve::get().get(),
                Error::<T>::ReservesTooLow
            ),
            OrderType::Sell => ensure!(
                T::SubnetInfo::tao_reserve(netuid.into()).to_u64()
                    >= T::MinimumReserve::get().get(),
                Error::<T>::ReservesTooLow
            ),
        }

        Self::maybe_initialize_v3(netuid)?;

        // Because user specifies the limit price, check that it is in fact beoynd the current one
        match order_type {
            OrderType::Buy => ensure!(
                AlphaSqrtPrice::<T>::get(netuid) < limit_sqrt_price,
                Error::<T>::PriceLimitExceeded
            ),
            OrderType::Sell => ensure!(
                AlphaSqrtPrice::<T>::get(netuid) > limit_sqrt_price,
                Error::<T>::PriceLimitExceeded
            ),
        };

        let mut amount_remaining = amount;
        let mut amount_paid_out: u64 = 0;
        let mut iteration_counter: u16 = 0;
        let mut in_acc: u64 = 0;
        let mut fee_acc: u64 = 0;

        log::trace!("======== Start Swap ========");
        log::trace!("Amount Remaining: {amount_remaining}");

        // Swap one tick at a time until we reach one of the stop conditions
        while amount_remaining > 0 {
            log::trace!("\nIteration: {iteration_counter}");
            log::trace!(
                "\tCurrent Liquidity: {}",
                CurrentLiquidity::<T>::get(netuid)
            );

            // Create and execute a swap step
            let mut swap_step = SwapStep::<T>::new(
                netuid,
                order_type,
                amount_remaining,
                limit_sqrt_price,
                drop_fees,
            );

            let swap_result = swap_step.execute()?;

            in_acc = in_acc.saturating_add(swap_result.delta_in);
            fee_acc = fee_acc.saturating_add(swap_result.fee_paid);
            amount_remaining = amount_remaining.saturating_sub(swap_result.amount_to_take);
            amount_paid_out = amount_paid_out.saturating_add(swap_result.delta_out);

            if swap_step.action == SwapStepAction::Stop {
                amount_remaining = 0;
            }

            // The swap step didn't exchange anything
            if swap_result.amount_to_take == 0 {
                amount_remaining = 0;
            }

            iteration_counter = iteration_counter.saturating_add(1);

            ensure!(
                iteration_counter <= MAX_SWAP_ITERATIONS,
                Error::<T>::TooManySwapSteps
            );
        }

        log::trace!("\nAmount Paid Out: {amount_paid_out}");
        log::trace!("======== End Swap ========");

        let (tao_reserve_delta, alpha_reserve_delta) = match order_type {
            OrderType::Buy => (in_acc as i64, (amount_paid_out as i64).neg()),
            OrderType::Sell => ((amount_paid_out as i64).neg(), in_acc as i64),
        };

        Ok(SwapResult {
            amount_paid_in: in_acc,
            amount_paid_out,
            fee_paid: fee_acc,
            tao_reserve_delta,
            alpha_reserve_delta,
        })
    }

    /// Get the tick at the current tick edge for the given direction (order type) If
    /// order type is Buy, then edge tick is the high tick, otherwise it is the low
    /// tick.
    ///
    /// If anything is wrong with tick math and it returns Err, we just abort the deal, i.e. return
    /// the edge that is impossible to execute
    fn tick_edge(netuid: NetUid, current_tick: TickIndex, order_type: OrderType) -> TickIndex {
        match order_type {
            OrderType::Buy => ActiveTickIndexManager::<T>::find_closest_higher(
                netuid,
                current_tick.next().unwrap_or(TickIndex::MAX),
            )
            .unwrap_or(TickIndex::MAX),
            OrderType::Sell => {
                let current_price = Pallet::<T>::current_price_sqrt(netuid);
                let current_tick_price = current_tick.as_sqrt_price_bounded();
                let is_active = ActiveTickIndexManager::<T>::tick_is_active(netuid, current_tick);

                if is_active && current_price > current_tick_price {
                    ActiveTickIndexManager::<T>::find_closest_lower(netuid, current_tick)
                        .unwrap_or(TickIndex::MIN)
                } else {
                    ActiveTickIndexManager::<T>::find_closest_lower(
                        netuid,
                        current_tick.prev().unwrap_or(TickIndex::MIN),
                    )
                    .unwrap_or(TickIndex::MIN)
                }
            }
        }
    }

    /// Calculate fee amount
    ///
    /// Fee is provided by state ops as u16-normalized value.
    fn calculate_fee_amount(netuid: NetUid, amount: u64, drop_fees: bool) -> u64 {
        if drop_fees {
            0
        } else {
            match T::SubnetInfo::mechanism(netuid) {
                1 => {
                    let fee_rate = U64F64::saturating_from_num(FeeRate::<T>::get(netuid))
                        .safe_div(U64F64::saturating_from_num(u16::MAX));
                    U64F64::saturating_from_num(amount)
                        .saturating_mul(fee_rate)
                        .saturating_to_num::<u64>()
                }
                _ => 0,
            }
        }
    }

    /// Add fees to the global fee counters
    fn add_fees(netuid: NetUid, order_type: OrderType, fee: u64) {
        let liquidity_curr = Self::current_liquidity_safe(netuid);

        if liquidity_curr == 0 {
            return;
        }

        let fee_global_tao = FeeGlobalTao::<T>::get(netuid);
        let fee_global_alpha = FeeGlobalAlpha::<T>::get(netuid);
        let fee_fixed = U64F64::saturating_from_num(fee);

        match order_type {
            OrderType::Sell => {
                FeeGlobalAlpha::<T>::set(
                    netuid,
                    fee_global_alpha.saturating_add(fee_fixed.safe_div(liquidity_curr)),
                );
            }
            OrderType::Buy => {
                FeeGlobalTao::<T>::set(
                    netuid,
                    fee_global_tao.saturating_add(fee_fixed.safe_div(liquidity_curr)),
                );
            }
        }
    }

    /// Convert input amount (delta_in) to output amount (delta_out)
    ///
    /// This is the core method of uniswap V3 that tells how much output token is given for an
    /// amount of input token within one price tick.
    pub(super) fn convert_deltas(netuid: NetUid, order_type: OrderType, delta_in: u64) -> u64 {
        // Skip conversion if delta_in is zero
        if delta_in == 0 {
            return 0;
        }

        let liquidity_curr = SqrtPrice::saturating_from_num(CurrentLiquidity::<T>::get(netuid));
        let sqrt_price_curr = Pallet::<T>::current_price_sqrt(netuid);
        let delta_fixed = SqrtPrice::saturating_from_num(delta_in);

        // Calculate result based on order type with proper fixed-point math
        // Using safe math operations throughout to prevent overflows
        let result = match order_type {
            OrderType::Sell => {
                // liquidity_curr / (liquidity_curr / sqrt_price_curr + delta_fixed);
                let denom = liquidity_curr
                    .safe_div(sqrt_price_curr)
                    .saturating_add(delta_fixed);
                let a = liquidity_curr.safe_div(denom);
                // a * sqrt_price_curr;
                let b = a.saturating_mul(sqrt_price_curr);

                // delta_fixed * b;
                delta_fixed.saturating_mul(b)
            }
            OrderType::Buy => {
                // (liquidity_curr * sqrt_price_curr + delta_fixed) * sqrt_price_curr;
                let a = liquidity_curr
                    .saturating_mul(sqrt_price_curr)
                    .saturating_add(delta_fixed)
                    .saturating_mul(sqrt_price_curr);
                // liquidity_curr / a;
                let b = liquidity_curr.safe_div(a);
                // b * delta_fixed;
                b.saturating_mul(delta_fixed)
            }
        };

        result.saturating_to_num::<u64>()
    }

    /// Get the target square root price based on the input amount
    ///
    /// This is the price that would be reached if
    ///    - There are no liquidity positions other than protocol liquidity
    ///    - Full delta_in amount is executed
    ///
    fn sqrt_price_target(
        order_type: OrderType,
        liquidity_curr: U64F64,
        sqrt_price_curr: SqrtPrice,
        delta_in: u64,
    ) -> SqrtPrice {
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let one = U64F64::saturating_from_num(1);

        // No liquidity means that price should go to the limit
        if liquidity_curr == 0 {
            return match order_type {
                OrderType::Sell => SqrtPrice::saturating_from_num(Self::min_price()),
                OrderType::Buy => SqrtPrice::saturating_from_num(Self::max_price()),
            };
        }

        match order_type {
            OrderType::Sell => one.safe_div(
                delta_fixed
                    .safe_div(liquidity_curr)
                    .saturating_add(one.safe_div(sqrt_price_curr)),
            ),
            OrderType::Buy => delta_fixed
                .safe_div(liquidity_curr)
                .saturating_add(sqrt_price_curr),
        }
    }

    /// Update liquidity when crossing a tick
    fn update_liquidity_at_crossing(netuid: NetUid, order_type: OrderType) -> Result<(), Error<T>> {
        let mut liquidity_curr = CurrentLiquidity::<T>::get(netuid);
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);

        // Find the appropriate tick based on order type
        let tick = match order_type {
            OrderType::Sell => {
                // Self::find_closest_lower_active_tick(netuid, current_tick_index)
                let current_price = Pallet::<T>::current_price_sqrt(netuid);
                let current_tick_price = current_tick_index.as_sqrt_price_bounded();
                let is_active =
                    ActiveTickIndexManager::<T>::tick_is_active(netuid, current_tick_index);

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
                Ticks::<T>::get(netuid, lower_tick)
            }
            OrderType::Buy => {
                // Self::find_closest_higher_active_tick(netuid, current_tick_index),
                let upper_tick = ActiveTickIndexManager::<T>::find_closest_higher(
                    netuid,
                    current_tick_index.next().unwrap_or(TickIndex::MAX),
                )
                .unwrap_or(TickIndex::MAX);
                Ticks::<T>::get(netuid, upper_tick)
            }
        }
        .ok_or(Error::<T>::InsufficientLiquidity)?;

        let liquidity_update_abs_u64 = tick.liquidity_net_as_u64();

        // Update liquidity based on the sign of liquidity_net and the order type
        liquidity_curr = match (order_type, tick.liquidity_net >= 0) {
            (OrderType::Sell, true) | (OrderType::Buy, false) => {
                liquidity_curr.saturating_sub(liquidity_update_abs_u64)
            }
            (OrderType::Sell, false) | (OrderType::Buy, true) => {
                liquidity_curr.saturating_add(liquidity_update_abs_u64)
            }
        };

        CurrentLiquidity::<T>::set(netuid, liquidity_curr);

        Ok(())
    }

    pub fn find_closest_lower_active_tick(netuid: NetUid, index: TickIndex) -> Option<Tick> {
        ActiveTickIndexManager::<T>::find_closest_lower(netuid, index)
            .and_then(|ti| Ticks::<T>::get(netuid, ti))
    }

    pub fn find_closest_higher_active_tick(netuid: NetUid, index: TickIndex) -> Option<Tick> {
        ActiveTickIndexManager::<T>::find_closest_higher(netuid, index)
            .and_then(|ti| Ticks::<T>::get(netuid, ti))
    }

    /// Here we subtract minimum safe liquidity from current liquidity to stay in the safe range
    fn current_liquidity_safe(netuid: NetUid) -> U64F64 {
        U64F64::saturating_from_num(
            CurrentLiquidity::<T>::get(netuid).saturating_sub(T::MinimumLiquidity::get()),
        )
    }

    /// Adds liquidity to the specified price range.
    ///
    /// This function allows an account to provide liquidity to a given range of price ticks. The
    /// amount of liquidity to be added can be determined using
    /// [`get_tao_based_liquidity`] and [`get_alpha_based_liquidity`], which compute the required
    /// liquidity based on TAO and Alpha balances for the current price tick.
    ///
    /// ### Behavior:
    /// - If the `protocol` flag is **not set** (`false`), the function will attempt to
    ///   **withdraw balances** from the account using `state_ops.withdraw_balances()`.
    /// - If the `protocol` flag is **set** (`true`), the liquidity is added without modifying balances.
    /// - If swap V3 was not initialized before, updates the value in storage.
    ///
    /// ### Parameters:
    /// - `coldkey_account_id`: A reference to the account coldkey that is providing liquidity.
    /// - `hotkey_account_id`: A reference to the account hotkey that is providing liquidity.
    /// - `tick_low`: The lower bound of the price tick range.
    /// - `tick_high`: The upper bound of the price tick range.
    /// - `liquidity`: The amount of liquidity to be added.
    ///
    /// ### Returns:
    /// - `Ok((u64, u64))`: (tao, alpha) amounts at new position
    /// - `Err(SwapError)`: If the operation fails due to insufficient balance, invalid tick range,
    ///   or other swap-related errors.
    ///
    /// ### Errors:
    /// - [`SwapError::InsufficientBalance`] if the account does not have enough balance.
    /// - [`SwapError::InvalidTickRange`] if `tick_low` is greater than or equal to `tick_high`.
    /// - Other [`SwapError`] variants as applicable.
    pub fn do_add_liquidity(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        hotkey_account_id: &T::AccountId,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Result<(PositionId, u64, u64), Error<T>> {
        ensure!(
            EnabledUserLiquidity::<T>::get(netuid),
            Error::<T>::UserLiquidityDisabled
        );

        let (position, tao, alpha) = Self::add_liquidity_not_insert(
            netuid,
            coldkey_account_id,
            tick_low,
            tick_high,
            liquidity,
        )?;
        let position_id = position.id;

        ensure!(
            T::BalanceOps::tao_balance(coldkey_account_id) >= TaoCurrency::from(tao)
                && T::BalanceOps::alpha_balance(
                    netuid.into(),
                    coldkey_account_id,
                    hotkey_account_id
                ) >= AlphaCurrency::from(alpha),
            Error::<T>::InsufficientBalance
        );

        // Small delta is not allowed
        ensure!(
            liquidity >= T::MinimumLiquidity::get(),
            Error::<T>::InvalidLiquidityValue
        );

        Positions::<T>::insert(&(netuid, coldkey_account_id, position.id), position);

        Ok((position_id, tao, alpha))
    }

    // add liquidity without inserting position into storage (used privately for v3 intiialization).
    // unlike Self::add_liquidity it also doesn't perform account's balance check.
    //
    // the public interface is [`Self::add_liquidity`]
    fn add_liquidity_not_insert(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Result<(Position<T>, u64, u64), Error<T>> {
        ensure!(
            Self::count_positions(netuid, coldkey_account_id) < T::MaxPositions::get() as usize,
            Error::<T>::MaxPositionsExceeded
        );

        // Ensure that tick_high is actually higher than tick_low
        ensure!(tick_high > tick_low, Error::<T>::InvalidTickRange);

        // Add liquidity at tick
        Self::add_liquidity_at_index(netuid, tick_low, liquidity, false);
        Self::add_liquidity_at_index(netuid, tick_high, liquidity, true);

        // Update current tick liquidity
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);
        Self::clamp_sqrt_price(netuid, current_tick_index);

        Self::update_liquidity_if_needed(netuid, tick_low, tick_high, liquidity as i128);

        // New position
        let position_id = PositionId::new::<T>();
        let position = Position::new(position_id, netuid, tick_low, tick_high, liquidity);

        let current_price_sqrt = Pallet::<T>::current_price_sqrt(netuid);
        let (tao, alpha) = position.to_token_amounts(current_price_sqrt)?;

        SwapV3Initialized::<T>::set(netuid, true);

        Ok((position, tao, alpha))
    }

    /// Remove liquidity and credit balances back to (coldkey_account_id, hotkey_account_id) stake.
    /// Removing is allowed even when user liquidity is enabled.
    ///
    /// Account ID and Position ID identify position in the storage map
    pub fn do_remove_liquidity(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        position_id: PositionId,
    ) -> Result<RemoveLiquidityResult, Error<T>> {
        let Some(mut position) = Positions::<T>::get((netuid, coldkey_account_id, position_id))
        else {
            return Err(Error::<T>::LiquidityNotFound);
        };

        // Collect fees and get tao and alpha amounts
        let (fee_tao, fee_alpha) = position.collect_fees();
        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let (tao, alpha) = position.to_token_amounts(current_price)?;

        // Update liquidity at position ticks
        Self::remove_liquidity_at_index(netuid, position.tick_low, position.liquidity, false);
        Self::remove_liquidity_at_index(netuid, position.tick_high, position.liquidity, true);

        // Update current tick liquidity
        Self::update_liquidity_if_needed(
            netuid,
            position.tick_low,
            position.tick_high,
            (position.liquidity as i128).neg(),
        );

        // Remove user position
        Positions::<T>::remove((netuid, coldkey_account_id, position_id));

        Ok(RemoveLiquidityResult {
            tao: tao.into(),
            alpha: alpha.into(),
            fee_tao: fee_tao.into(),
            fee_alpha: fee_alpha.into(),
            tick_low: position.tick_low,
            tick_high: position.tick_high,
            liquidity: position.liquidity,
        })
    }

    pub fn do_modify_position(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        hotkey_account_id: &T::AccountId,
        position_id: PositionId,
        liquidity_delta: i64,
    ) -> Result<UpdateLiquidityResult, Error<T>> {
        ensure!(
            EnabledUserLiquidity::<T>::get(netuid),
            Error::<T>::UserLiquidityDisabled
        );

        // Find the position
        let Some(mut position) = Positions::<T>::get((netuid, coldkey_account_id, position_id))
        else {
            return Err(Error::<T>::LiquidityNotFound);
        };

        // Small delta is not allowed
        ensure!(
            liquidity_delta.abs() >= T::MinimumLiquidity::get() as i64,
            Error::<T>::InvalidLiquidityValue
        );
        let mut delta_liquidity_abs = liquidity_delta.unsigned_abs();

        // Determine the effective price for token calculations
        let current_price_sqrt = Pallet::<T>::current_price_sqrt(netuid);
        let sqrt_pa: SqrtPrice = position
            .tick_low
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_pb: SqrtPrice = position
            .tick_high
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_price_box = if current_price_sqrt < sqrt_pa {
            sqrt_pa
        } else if current_price_sqrt > sqrt_pb {
            sqrt_pb
        } else {
            // Update current liquidity if price is in range
            let new_liquidity_curr = if liquidity_delta > 0 {
                CurrentLiquidity::<T>::get(netuid).saturating_add(delta_liquidity_abs)
            } else {
                CurrentLiquidity::<T>::get(netuid).saturating_sub(delta_liquidity_abs)
            };
            CurrentLiquidity::<T>::set(netuid, new_liquidity_curr);
            current_price_sqrt
        };

        // Calculate token amounts for the liquidity change
        let mul = SqrtPrice::from_num(1)
            .safe_div(sqrt_price_box)
            .saturating_sub(SqrtPrice::from_num(1).safe_div(sqrt_pb));
        let alpha = SqrtPrice::saturating_from_num(delta_liquidity_abs).saturating_mul(mul);
        let tao = SqrtPrice::saturating_from_num(delta_liquidity_abs)
            .saturating_mul(sqrt_price_box.saturating_sub(sqrt_pa));

        // Validate delta
        if liquidity_delta > 0 {
            // Check that user has enough balances
            ensure!(
                T::BalanceOps::tao_balance(coldkey_account_id)
                    >= TaoCurrency::from(tao.saturating_to_num::<u64>())
                    && T::BalanceOps::alpha_balance(netuid, coldkey_account_id, hotkey_account_id)
                        >= AlphaCurrency::from(alpha.saturating_to_num::<u64>()),
                Error::<T>::InsufficientBalance
            );
        } else {
            // Check that position has enough liquidity
            ensure!(
                position.liquidity >= delta_liquidity_abs,
                Error::<T>::InsufficientLiquidity
            );
        }

        // Collect fees
        let (fee_tao, fee_alpha) = position.collect_fees();

        // If delta brings the position liquidity below MinimumLiquidity, eliminate position and
        // withdraw full amounts
        let mut remove = false;
        if (liquidity_delta < 0)
            && (position.liquidity.saturating_sub(delta_liquidity_abs) < T::MinimumLiquidity::get())
        {
            delta_liquidity_abs = position.liquidity;
            remove = true;
        }

        // Adjust liquidity at the ticks based on the delta sign
        if liquidity_delta > 0 {
            // Add liquidity at tick
            Self::add_liquidity_at_index(netuid, position.tick_low, delta_liquidity_abs, false);
            Self::add_liquidity_at_index(netuid, position.tick_high, delta_liquidity_abs, true);

            // Add liquidity to user position
            position.liquidity = position.liquidity.saturating_add(delta_liquidity_abs);
        } else {
            // Remove liquidity at tick
            Self::remove_liquidity_at_index(netuid, position.tick_low, delta_liquidity_abs, false);
            Self::remove_liquidity_at_index(netuid, position.tick_high, delta_liquidity_abs, true);

            // Remove liquidity from user position
            position.liquidity = position.liquidity.saturating_sub(delta_liquidity_abs);
        }

        // Update or, in case if full liquidity is removed, remove the position
        if remove {
            Positions::<T>::remove((netuid, coldkey_account_id, position_id));
        } else {
            Positions::<T>::insert(&(netuid, coldkey_account_id, position.id), position.clone());
        }

        Ok(UpdateLiquidityResult {
            tao: tao.saturating_to_num::<u64>().into(),
            alpha: alpha.saturating_to_num::<u64>().into(),
            fee_tao: fee_tao.into(),
            fee_alpha: fee_alpha.into(),
            removed: remove,
            tick_low: position.tick_low,
            tick_high: position.tick_high,
        })
    }

    /// Adds or updates liquidity at a specific tick index for a subnet
    ///
    /// # Arguments
    /// * `netuid` - The subnet ID
    /// * `tick_index` - The tick index to add liquidity to
    /// * `liquidity` - The amount of liquidity to add
    fn add_liquidity_at_index(netuid: NetUid, tick_index: TickIndex, liquidity: u64, upper: bool) {
        // Convert liquidity to signed value, negating it for upper bounds
        let net_liquidity_change = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        Ticks::<T>::mutate(netuid, tick_index, |maybe_tick| match maybe_tick {
            Some(tick) => {
                tick.liquidity_net = tick.liquidity_net.saturating_add(net_liquidity_change);
                tick.liquidity_gross = tick.liquidity_gross.saturating_add(liquidity);
            }
            None => {
                let current_tick = TickIndex::current_bounded::<T>(netuid);

                let (fees_out_tao, fees_out_alpha) = if tick_index > current_tick {
                    (
                        I64F64::saturating_from_num(FeeGlobalTao::<T>::get(netuid)),
                        I64F64::saturating_from_num(FeeGlobalAlpha::<T>::get(netuid)),
                    )
                } else {
                    (
                        I64F64::saturating_from_num(0),
                        I64F64::saturating_from_num(0),
                    )
                };
                *maybe_tick = Some(Tick {
                    liquidity_net: net_liquidity_change,
                    liquidity_gross: liquidity,
                    fees_out_tao,
                    fees_out_alpha,
                });
            }
        });

        // Update active ticks
        ActiveTickIndexManager::<T>::insert(netuid, tick_index);
    }

    /// Remove liquidity at tick index.
    fn remove_liquidity_at_index(
        netuid: NetUid,
        tick_index: TickIndex,
        liquidity: u64,
        upper: bool,
    ) {
        // Calculate net liquidity addition
        let net_reduction = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        Ticks::<T>::mutate_exists(netuid, tick_index, |maybe_tick| {
            if let Some(tick) = maybe_tick {
                tick.liquidity_net = tick.liquidity_net.saturating_sub(net_reduction);
                tick.liquidity_gross = tick.liquidity_gross.saturating_sub(liquidity);

                // If no liquidity is left at the tick, remove it
                if tick.liquidity_gross == 0 {
                    *maybe_tick = None;

                    // Update active ticks: Final liquidity is zero, remove this tick from active.
                    ActiveTickIndexManager::<T>::remove(netuid, tick_index);
                }
            }
        });
    }

    /// Updates the current liquidity for a subnet if the current tick index is within the specified
    /// range
    ///
    /// This function handles both increasing and decreasing liquidity based on the sign of the
    /// liquidity parameter. It uses i128 to safely handle values up to u64::MAX in both positive
    /// and negative directions.
    fn update_liquidity_if_needed(
        netuid: NetUid,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: i128,
    ) {
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);
        if (tick_low <= current_tick_index) && (current_tick_index < tick_high) {
            CurrentLiquidity::<T>::mutate(netuid, |current_liquidity| {
                let is_neg = liquidity.is_negative();
                let liquidity = liquidity.abs().min(u64::MAX as i128) as u64;
                if is_neg {
                    *current_liquidity = current_liquidity.saturating_sub(liquidity);
                } else {
                    *current_liquidity = current_liquidity.saturating_add(liquidity);
                }
            });
        }
    }

    /// Clamps the subnet's sqrt price when tick index is outside of valid bounds
    fn clamp_sqrt_price(netuid: NetUid, tick_index: TickIndex) {
        if tick_index >= TickIndex::MAX || tick_index <= TickIndex::MIN {
            let corrected_price = tick_index.as_sqrt_price_bounded();
            AlphaSqrtPrice::<T>::set(netuid, corrected_price);
        }
    }

    /// Returns the number of positions for an account in a specific subnet
    ///
    /// # Arguments
    /// * `netuid` - The subnet ID
    /// * `account_id` - The account ID
    ///
    /// # Returns
    /// The number of positions that the account has in the specified subnet
    pub(super) fn count_positions(netuid: NetUid, account_id: &T::AccountId) -> usize {
        Positions::<T>::iter_prefix_values((netuid, account_id.clone())).count()
    }

    /// Returns the protocol account ID
    ///
    /// # Returns
    /// The account ID of the protocol account
    pub fn protocol_account_id() -> T::AccountId {
        T::ProtocolId::get().into_account_truncating()
    }

    /// Dissolve all LPs and clean state.
    pub fn do_dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult {
        if SwapV3Initialized::<T>::get(netuid) {
            // 1) Snapshot only *non‑protocol* positions: (owner, position_id).
            struct CloseItem<A> {
                owner: A,
                pos_id: PositionId,
            }
            let protocol_account = Self::protocol_account_id();

            let mut to_close: sp_std::vec::Vec<CloseItem<T::AccountId>> = sp_std::vec::Vec::new();
            for ((owner, pos_id), _pos) in Positions::<T>::iter_prefix((netuid,)) {
                if owner != protocol_account {
                    to_close.push(CloseItem { owner, pos_id });
                }
            }

            if to_close.is_empty() {
                log::debug!(
                    "dissolve_all_lp: no user positions; netuid={netuid:?}, protocol liquidity untouched"
                );
                return Ok(());
            }

            let mut user_refunded_tao = TaoCurrency::ZERO;
            let mut user_staked_alpha = AlphaCurrency::ZERO;

            let trust: Vec<u16> = T::SubnetInfo::get_validator_trust(netuid.into());
            let permit: Vec<bool> = T::SubnetInfo::get_validator_permit(netuid.into());

            // Helper: pick target validator uid, only among permitted validators, by highest trust.
            let pick_target_uid = |trust: &Vec<u16>, permit: &Vec<bool>| -> Option<u16> {
                let mut best_uid: Option<usize> = None;
                let mut best_trust: u16 = 0;
                for (i, (&t, &p)) in trust.iter().zip(permit.iter()).enumerate() {
                    if p && (best_uid.is_none() || t > best_trust) {
                        best_uid = Some(i);
                        best_trust = t;
                    }
                }
                best_uid.map(|i| i as u16)
            };

            for CloseItem { owner, pos_id } in to_close.into_iter() {
                match Self::do_remove_liquidity(netuid, &owner, pos_id) {
                    Ok(rm) => {
                        // α withdrawn from the pool = principal + accrued fees
                        let alpha_total_from_pool: AlphaCurrency =
                            rm.alpha.saturating_add(rm.fee_alpha);

                        // ---------------- USER: refund τ and convert α → stake ----------------

                        // 1) Refund τ principal directly.
                        if rm.tao > TaoCurrency::ZERO {
                            T::BalanceOps::increase_balance(&owner, rm.tao);
                            user_refunded_tao = user_refunded_tao.saturating_add(rm.tao);
                            T::BalanceOps::decrease_provided_tao_reserve(netuid, rm.tao);
                        }

                        // 2) Stake ALL withdrawn α (principal + fees) to the best permitted validator.
                        if alpha_total_from_pool > AlphaCurrency::ZERO {
                            if let Some(target_uid) = pick_target_uid(&trust, &permit) {
                                let validator_hotkey: T::AccountId =
                                    T::SubnetInfo::hotkey_of_uid(netuid.into(), target_uid).ok_or(
                                        sp_runtime::DispatchError::Other(
                                            "validator_hotkey_missing",
                                        ),
                                    )?;

                                // Stake α from LP owner (coldkey) to chosen validator (hotkey).
                                T::BalanceOps::increase_stake(
                                    &owner,
                                    &validator_hotkey,
                                    NetUid::ROOT,
                                    alpha_total_from_pool,
                                )?;

                                user_staked_alpha =
                                    user_staked_alpha.saturating_add(alpha_total_from_pool);

                                log::debug!(
                                    "dissolve_all_lp: user dissolved & staked α: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, α_staked={alpha_total_from_pool:?}, target_uid={target_uid}"
                                );
                            } else {
                                // No permitted validators; burn to avoid balance drift.
                                log::debug!(
                                    "dissolve_all_lp: no permitted validators; α burned: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, α_total={alpha_total_from_pool:?}"
                                );
                            }

                            T::BalanceOps::decrease_provided_alpha_reserve(
                                netuid,
                                alpha_total_from_pool,
                            );
                        }
                    }
                    Err(e) => {
                        log::debug!(
                            "dissolve_all_lp: force-close failed: netuid={netuid:?}, owner={owner:?}, pos_id={pos_id:?}, err={e:?}"
                        );
                        continue;
                    }
                }
            }

            log::debug!(
                "dissolve_all_liquidity_providers (users-only): netuid={netuid:?}, users_refunded_total_τ={user_refunded_tao:?}, users_staked_total_α={user_staked_alpha:?}; protocol liquidity untouched"
            );

            return Ok(());
        }

        log::debug!(
            "dissolve_all_liquidity_providers: netuid={netuid:?}, mode=V2-or-nonV3, leaving all liquidity/state intact"
        );

        Ok(())
    }

    /// Clear **protocol-owned** liquidity and wipe all swap state for `netuid`.
    pub fn do_clear_protocol_liquidity(netuid: NetUid) -> DispatchResult {
        let protocol_account = Self::protocol_account_id();

        // 1) Force-close only protocol positions, burning proceeds.
        let mut burned_tao = TaoCurrency::ZERO;
        let mut burned_alpha = AlphaCurrency::ZERO;

        // Collect protocol position IDs first to avoid mutating while iterating.
        let protocol_pos_ids: sp_std::vec::Vec<PositionId> = Positions::<T>::iter_prefix((netuid,))
            .filter_map(|((owner, pos_id), _)| {
                if owner == protocol_account {
                    Some(pos_id)
                } else {
                    None
                }
            })
            .collect();

        for pos_id in protocol_pos_ids {
            match Self::do_remove_liquidity(netuid, &protocol_account, pos_id) {
                Ok(rm) => {
                    let alpha_total_from_pool: AlphaCurrency =
                        rm.alpha.saturating_add(rm.fee_alpha);
                    let tao = rm.tao;

                    if tao > TaoCurrency::ZERO {
                        burned_tao = burned_tao.saturating_add(tao);
                    }
                    if alpha_total_from_pool > AlphaCurrency::ZERO {
                        burned_alpha = burned_alpha.saturating_add(alpha_total_from_pool);
                    }

                    log::debug!(
                        "clear_protocol_liquidity: burned protocol pos: netuid={netuid:?}, pos_id={pos_id:?}, τ={tao:?}, α_total={alpha_total_from_pool:?}"
                    );
                }
                Err(e) => {
                    log::debug!(
                        "clear_protocol_liquidity: force-close failed: netuid={netuid:?}, pos_id={pos_id:?}, err={e:?}"
                    );
                    continue;
                }
            }
        }

        // 2) Clear active tick index entries, then all swap state (idempotent even if empty/non‑V3).
        let active_ticks: sp_std::vec::Vec<TickIndex> =
            Ticks::<T>::iter_prefix(netuid).map(|(ti, _)| ti).collect();
        for ti in active_ticks {
            ActiveTickIndexManager::<T>::remove(netuid, ti);
        }

        let _ = Positions::<T>::clear_prefix((netuid,), u32::MAX, None);
        let _ = Ticks::<T>::clear_prefix(netuid, u32::MAX, None);

        FeeGlobalTao::<T>::remove(netuid);
        FeeGlobalAlpha::<T>::remove(netuid);
        CurrentLiquidity::<T>::remove(netuid);
        CurrentTick::<T>::remove(netuid);
        AlphaSqrtPrice::<T>::remove(netuid);
        SwapV3Initialized::<T>::remove(netuid);

        let _ = TickIndexBitmapWords::<T>::clear_prefix((netuid,), u32::MAX, None);
        FeeRate::<T>::remove(netuid);
        EnabledUserLiquidity::<T>::remove(netuid);

        log::debug!(
            "clear_protocol_liquidity: netuid={netuid:?}, protocol_burned: τ={burned_tao:?}, α={burned_alpha:?}; state cleared"
        );

        Ok(())
    }
}

impl<T: Config> SwapHandler<T::AccountId> for Pallet<T> {
    fn swap(
        netuid: NetUid,
        order_t: OrderType,
        amount: u64,
        price_limit: u64,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult, DispatchError> {
        let limit_sqrt_price = SqrtPrice::saturating_from_num(price_limit)
            .safe_div(SqrtPrice::saturating_from_num(1_000_000_000))
            .checked_sqrt(SqrtPrice::saturating_from_num(0.0000000001))
            .ok_or(Error::<T>::PriceLimitExceeded)?;

        Self::do_swap(
            NetUid::from(netuid),
            order_t,
            amount,
            limit_sqrt_price,
            drop_fees,
            should_rollback,
        )
        .map_err(Into::into)
    }

    fn sim_swap(
        netuid: NetUid,
        order_t: OrderType,
        amount: u64,
    ) -> Result<SwapResult, DispatchError> {
        match T::SubnetInfo::mechanism(netuid) {
            1 => {
                let price_limit = match order_t {
                    OrderType::Buy => Self::max_price(),
                    OrderType::Sell => Self::min_price(),
                };

                Self::swap(netuid, order_t, amount, price_limit, false, true)
            }
            _ => {
                let actual_amount = if T::SubnetInfo::exists(netuid) {
                    amount
                } else {
                    0
                };
                Ok(SwapResult {
                    amount_paid_in: actual_amount,
                    amount_paid_out: actual_amount,
                    fee_paid: 0,
                    tao_reserve_delta: 0,
                    alpha_reserve_delta: 0,
                })
            }
        }
    }

    fn approx_fee_amount(netuid: NetUid, amount: u64) -> u64 {
        Self::calculate_fee_amount(netuid.into(), amount, false)
    }

    fn current_alpha_price(netuid: NetUid) -> U96F32 {
        Self::current_price(netuid.into())
    }

    fn min_price() -> u64 {
        TickIndex::min_sqrt_price()
            .saturating_mul(TickIndex::min_sqrt_price())
            .saturating_mul(SqrtPrice::saturating_from_num(1_000_000_000))
            .saturating_to_num()
    }

    fn max_price() -> u64 {
        TickIndex::max_sqrt_price()
            .saturating_mul(TickIndex::max_sqrt_price())
            .saturating_mul(SqrtPrice::saturating_from_num(1_000_000_000))
            .saturating_round()
            .saturating_to_num()
    }

    fn adjust_protocol_liquidity(
        netuid: NetUid,
        tao_delta: TaoCurrency,
        alpha_delta: AlphaCurrency,
    ) {
        Self::adjust_protocol_liquidity(netuid, tao_delta, alpha_delta);
    }

    fn is_user_liquidity_enabled(netuid: NetUid) -> bool {
        EnabledUserLiquidity::<T>::get(netuid)
    }
    fn dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult {
        Self::do_dissolve_all_liquidity_providers(netuid)
    }
    fn toggle_user_liquidity(netuid: NetUid, enabled: bool) {
        EnabledUserLiquidity::<T>::insert(netuid, enabled)
    }
    fn clear_protocol_liquidity(netuid: NetUid) -> DispatchResult {
        Self::do_clear_protocol_liquidity(netuid)
    }
}

#[derive(Debug, PartialEq)]
struct SwapStepResult {
    amount_to_take: u64,
    fee_paid: u64,
    delta_in: u64,
    delta_out: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwapStepAction {
    Crossing,
    Stop,
}
