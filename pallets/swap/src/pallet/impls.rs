use core::marker::PhantomData;
use core::ops::Neg;

use frame_support::storage::{TransactionOutcome, transactional};
use frame_support::{ensure, pallet_prelude::DispatchError, traits::Get};
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use sp_runtime::{DispatchResult, traits::AccountIdConversion};
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

    /// Distribute `alpha_total` back to the coldkey's hotkeys for `netuid`.
    /// - Pro‑rata by current α stake on this subnet; if all zero, split evenly.
    /// - Deterministic "largest remainders" rounding to ensure exact conservation.
    /// - Robust to partial deposit failures: retries across successes, final fallback to (cold, cold).
    pub fn refund_alpha(netuid: NetUid, coldkey: &T::AccountId, alpha_total: AlphaCurrency) {
        if alpha_total.is_zero() {
            return;
        }

        // 1) Recipient set
        let mut hotkeys: sp_std::vec::Vec<T::AccountId> = T::SubnetInfo::get_owned_hotkeys(coldkey);
        if hotkeys.is_empty() {
            hotkeys.push(coldkey.clone());
        }

        // 2) Weights = current α stake per hotkey; if all zero -> even split
        let weights: sp_std::vec::Vec<u128> = hotkeys
            .iter()
            .map(|hk| u128::from(T::BalanceOps::alpha_balance(netuid, coldkey, hk).to_u64()))
            .collect();

        let sum_weights: u128 = weights
            .iter()
            .copied()
            .fold(0u128, |acc, w| acc.saturating_add(w));
        let total_u128: u128 = u128::from(alpha_total.to_u64());
        let n = hotkeys.len();

        // (account, planned_amount_u64)
        let mut shares: sp_std::vec::Vec<(T::AccountId, u64)> = sp_std::vec::Vec::with_capacity(n);

        if sum_weights > 0 {
            // 3a) Pro‑rata base + largest remainders (deterministic)
            let mut bases: sp_std::vec::Vec<u128> = sp_std::vec::Vec::with_capacity(n);
            let mut remainders: sp_std::vec::Vec<(usize, u128)> =
                sp_std::vec::Vec::with_capacity(n);

            let mut base_sum: u128 = 0;
            for (i, (&w, hk)) in weights.iter().zip(hotkeys.iter()).enumerate() {
                let numer = total_u128.saturating_mul(w);
                let base = numer.checked_div(sum_weights).unwrap_or(0);
                let rem = numer.checked_rem(sum_weights).unwrap_or(0);
                bases.push(base);
                remainders.push((i, rem));
                base_sum = base_sum.saturating_add(base);
                shares.push((hk.clone(), u64::try_from(base).unwrap_or(u64::MAX)));
            }

            // Distribute leftover ones to the largest remainders; tie‑break by index for determinism
            let mut leftover = total_u128.saturating_sub(base_sum);
            if leftover > 0 {
                remainders.sort_by(|a, b| {
                    // Descending by remainder, then ascending by index
                    b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
                });
                let mut k = 0usize;
                while leftover > 0 && k < remainders.len() {
                    if let Some((idx, _)) = remainders.get(k) {
                        if let Some((_, amt)) = shares.get_mut(*idx) {
                            *amt = amt.saturating_add(1);
                        }
                    }
                    leftover = leftover.saturating_sub(1);
                    k = k.saturating_add(1);
                }
            }
        } else {
            // 3b) Even split with deterministic round‑robin remainder
            let base = total_u128.checked_div(n as u128).unwrap_or(0);
            let mut rem = total_u128.checked_rem(n as u128).unwrap_or(0);
            for hk in hotkeys.iter() {
                let mut amt = u64::try_from(base).unwrap_or(u64::MAX);
                if rem > 0 {
                    amt = amt.saturating_add(1);
                    rem = rem.saturating_sub(1);
                }
                shares.push((hk.clone(), amt));
            }
        }

        // 4) Deposit to (coldkey, each hotkey). Track leftover if any deposit fails.
        let mut leftover: u64 = 0;
        let mut successes: sp_std::vec::Vec<T::AccountId> = sp_std::vec::Vec::new();

        for (hk, amt_u64) in shares.iter() {
            if *amt_u64 == 0 {
                continue;
            }
            let amt: AlphaCurrency = (*amt_u64).into();
            match T::BalanceOps::increase_stake(coldkey, hk, netuid, amt) {
                Ok(_) => successes.push(hk.clone()),
                Err(e) => {
                    log::warn!(
                        "refund_alpha: increase_stake failed (cold={coldkey:?}, hot={hk:?}, netuid={netuid:?}, amt={amt_u64:?}): {e:?}"
                    );
                    leftover = leftover.saturating_add(*amt_u64);
                }
            }
        }

        // 5) Retry: spread any leftover across the hotkeys that succeeded in step 4.
        if leftover > 0 && !successes.is_empty() {
            let count = successes.len() as u64;
            let base = leftover.checked_div(count).unwrap_or(0);
            let mut rem = leftover.checked_rem(count).unwrap_or(0);

            let mut leftover_retry: u64 = 0;
            for hk in successes.iter() {
                let add: u64 = base.saturating_add(if rem > 0 {
                    rem = rem.saturating_sub(1);
                    1
                } else {
                    0
                });
                if add == 0 {
                    continue;
                }
                if let Err(e) = T::BalanceOps::increase_stake(coldkey, hk, netuid, add.into()) {
                    log::warn!(
                        "refund_alpha(retry): increase_stake failed (cold={coldkey:?}, hot={hk:?}, netuid={netuid:?}, amt={add:?}): {e:?}"
                    );
                    leftover_retry = leftover_retry.saturating_add(add);
                }
            }
            leftover = leftover_retry;
        }

        // 6) Final fallback: deposit any remainder to (coldkey, coldkey).
        if leftover > 0 {
            let _ = T::BalanceOps::increase_stake(coldkey, coldkey, netuid, leftover.into());
        }
    }

    /// Dissolve all LPs for `netuid`, refund providers, and reset all swap state.
    ///
    /// - **V3 path** (mechanism == 1 && SwapV3Initialized):
    ///   * Remove **all** positions via `do_remove_liquidity`.
    ///   * **Refund** each owner:
    ///       - TAO = Σ(position.tao + position.fee_tao) → credited to the owner's **coldkey** free balance.
    ///       - ALPHA = Σ(position.alpha + position.fee_alpha) → credited back via `refund_alpha`.
    ///   * Decrease "provided reserves" (principal only) for non‑protocol owners.
    ///   * Clear ActiveTickIndexManager entries, ticks, fee globals, price, tick, liquidity,
    ///     init flag, bitmap words, fee rate knob, and user LP flag.
    ///
    /// - **V2 / non‑V3 path**:
    ///   * No per‑position records exist; still defensively clear the same V3 storages (safe no‑ops).
    pub fn do_dissolve_all_liquidity_providers(netuid: NetUid) -> DispatchResult {
        let mechid = T::SubnetInfo::mechanism(netuid.into());
        let v3_initialized = SwapV3Initialized::<T>::get(netuid);
        let user_lp_enabled =
        <Self as subtensor_swap_interface::SwapHandler<T::AccountId>>::is_user_liquidity_enabled(netuid);

        let is_v3_mode = mechid == 1 && v3_initialized;

        if is_v3_mode {
            // -------- V3: close every position, aggregate refunds, clear state --------

            // 1) Snapshot all (owner, position_id).
            struct CloseItem<A> {
                owner: A,
                pos_id: PositionId,
            }
            let mut to_close: sp_std::vec::Vec<CloseItem<T::AccountId>> = sp_std::vec::Vec::new();

            for ((owner, pos_id), _pos) in Positions::<T>::iter_prefix((netuid,)) {
                to_close.push(CloseItem { owner, pos_id });
            }

            let protocol_account = Self::protocol_account_id();

            // 2) Aggregate refunds per owner while removing positions.
            use sp_std::collections::btree_map::BTreeMap;
            let mut refunds: BTreeMap<T::AccountId, (TaoCurrency, AlphaCurrency)> = BTreeMap::new();

            for CloseItem { owner, pos_id } in to_close.into_iter() {
                let rm = Self::do_remove_liquidity(netuid, &owner, pos_id)?;

                // Accumulate (TAO, α) refund: principal + fees.
                let tao_add = rm.tao.saturating_add(rm.fee_tao);
                let alpha_add = rm.alpha.saturating_add(rm.fee_alpha);

                refunds
                    .entry(owner.clone())
                    .and_modify(|(t, a)| {
                        *t = t.saturating_add(tao_add);
                        *a = a.saturating_add(alpha_add);
                    })
                    .or_insert((tao_add, alpha_add));

                if owner != protocol_account {
                    T::BalanceOps::decrease_provided_tao_reserve(netuid, rm.tao);
                    T::BalanceOps::decrease_provided_alpha_reserve(netuid, rm.alpha);
                }
            }

            // 3) Process refunds per owner.
            for (owner, (tao_sum, alpha_sum)) in refunds.into_iter() {
                // TAO → coldkey free balance
                if tao_sum > TaoCurrency::ZERO {
                    T::BalanceOps::increase_balance(&owner, tao_sum);
                }

                // α → split across all hotkeys owned by `owner`.
                if !alpha_sum.is_zero() && owner != protocol_account {
                    Self::refund_alpha(netuid, &owner, alpha_sum);
                }
            }

            // 4) Clear active tick index set by walking ticks we are about to clear.
            let active_ticks: sp_std::vec::Vec<TickIndex> =
                Ticks::<T>::iter_prefix(netuid).map(|(ti, _)| ti).collect();
            for ti in active_ticks {
                ActiveTickIndexManager::<T>::remove(netuid, ti);
            }

            // 5) Clear storage for this netuid.
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
                "dissolve_all_liquidity_providers: netuid={netuid:?}, mode=V3, user_lp_enabled={user_lp_enabled}, v3_state_cleared + refunds"
            );

            return Ok(());
        }

        // -------- V2 / non‑V3: no positions to close; still nuke any V3 residues --------

        let _ = Positions::<T>::clear_prefix((netuid,), u32::MAX, None);

        let active_ticks: sp_std::vec::Vec<TickIndex> =
            Ticks::<T>::iter_prefix(netuid).map(|(ti, _)| ti).collect();
        for ti in active_ticks {
            ActiveTickIndexManager::<T>::remove(netuid, ti);
        }

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
            "dissolve_all_liquidity_providers: netuid={netuid:?}, mode=V2-or-nonV3, user_lp_enabled={user_lp_enabled}, state_cleared"
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
