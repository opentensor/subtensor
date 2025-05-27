use core::marker::PhantomData;
use core::ops::Neg;

use frame_support::storage::{TransactionOutcome, transactional};
use frame_support::{ensure, pallet_prelude::DispatchError, traits::Get};
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use sp_runtime::traits::AccountIdConversion;
use substrate_fixed::types::{U64F64, U96F32};
use subtensor_swap_interface::{
    BalanceOps, SubnetInfo, SwapHandler, SwapResult, UpdateLiquidityResult,
};

use super::pallet::*;
use crate::{
    NetUid, OrderType, SqrtPrice,
    position::{Position, PositionId},
    tick::{ActiveTickIndexManager, Tick, TickIndex},
};

const MAX_SWAP_ITERATIONS: u16 = 1000;

/// A struct representing a single swap step with all its parameters and state
struct SwapStep<T: frame_system::Config> {
    // Input parameters
    netuid: NetUid,
    order_type: OrderType,
    sqrt_price_limit: SqrtPrice,

    // Computed values
    current_price: U64F64,
    current_liquidity: U64F64,
    sqrt_price_edge: SqrtPrice,
    possible_delta_in: u64,
    sqrt_price_target: SqrtPrice,

    // Result values
    action: SwapStepAction,
    delta_in: u64,
    final_price: SqrtPrice,

    // Phantom data to use T
    _phantom: PhantomData<T>,
}

impl<T: Config> SwapStep<T> {
    /// Creates and initializes a new swap step
    fn new(
        netuid: NetUid,
        order_type: OrderType,
        amount_remaining: u64,
        sqrt_price_limit: SqrtPrice,
    ) -> Self {
        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let current_liquidity = Pallet::<T>::current_liquidity_safe(netuid);
        let sqrt_price_edge = Pallet::<T>::sqrt_price_edge(netuid, current_price, order_type);

        let possible_delta_in = amount_remaining
            .saturating_sub(Pallet::<T>::calculate_fee_amount(netuid, amount_remaining));

        // println!("SwapStep::new order_type = {:?}", order_type);
        // println!("SwapStep::new sqrt_price_limit = {:?}", sqrt_price_limit);

        // Target price and quantities
        let sqrt_price_target = Pallet::<T>::sqrt_price_target(
            order_type,
            current_liquidity,
            current_price,
            possible_delta_in,
        );

        Self {
            netuid,
            order_type,
            sqrt_price_limit,
            current_price,
            current_liquidity,
            sqrt_price_edge,
            possible_delta_in,
            sqrt_price_target,
            action: SwapStepAction::Stop,
            delta_in: 0,
            final_price: sqrt_price_target,
            _phantom: PhantomData,
        }
    }

    /// Execute the swap step and return the result
    fn execute(&mut self) -> Result<SwapStepResult, Error<T>> {
        self.determine_action();
        self.process_swap()
    }

    /// Returns True is price1 is closer to the current price than price2
    /// in terms of order direction.
    ///    For buying:  price1 <= price2
    ///    For selling: price1 >= price2
    ///
    fn price_is_closer(&self, price1: &SqrtPrice, price2: &SqrtPrice) -> bool {
        match self.order_type {
            OrderType::Buy => price1 <= price2,
            OrderType::Sell => price1 >= price2,
        }
    }

    /// Determine the appropriate action for this swap step
    fn determine_action(&mut self) {
        // Calculate the stopping price: The price at which we either reach the limit price,
        // exchange the full amount, or reach the edge price.

        if self.price_is_closer(&self.sqrt_price_target, &self.sqrt_price_limit)
            && self.price_is_closer(&self.sqrt_price_target, &self.sqrt_price_edge)
        {
            // Case 1. target_quantity is the lowest
            // The trade completely happens within one tick, no tick crossing happens.
            self.action = SwapStepAction::Stop;
            self.final_price = self.sqrt_price_target;
            self.delta_in = self.possible_delta_in;
            // println!("Case 1. Delta in = {:?}", self.delta_in);
        } else if self.price_is_closer(&self.sqrt_price_limit, &self.sqrt_price_target)
            && self.price_is_closer(&self.sqrt_price_limit, &self.sqrt_price_edge)
        {
            // Case 2. lim_quantity is the lowest
            // The trade also completely happens within one tick, no tick crossing happens.
            self.action = SwapStepAction::Stop;
            self.final_price = self.sqrt_price_limit;
            self.delta_in = Self::delta_in(
                self.order_type,
                self.current_liquidity,
                self.current_price,
                self.sqrt_price_limit,
            );
            // println!("Case 2. Delta in = {:?}", self.delta_in);
            // println!("Case 2. sqrt_price_limit = {:?}", self.sqrt_price_limit);
        } else {
            // Case 3. edge_quantity is the lowest
            // Tick crossing is likely
            self.action = SwapStepAction::Crossing;
            self.delta_in = Self::delta_in(
                self.order_type,
                self.current_liquidity,
                self.current_price,
                self.sqrt_price_edge,
            );
            self.final_price = self.sqrt_price_edge;
            // println!("Case 3. Delta in = {:?}", self.delta_in);
        }

        // Now correct the action if we stopped exactly at the edge no matter what was the case above
        // Because order type buy moves the price up and tick semi-open interval doesn't include its right
        // point, we cross on buys and stop on sells.
        let natural_reason_stop_price =
            if self.price_is_closer(&self.sqrt_price_limit, &self.sqrt_price_target) {
                self.sqrt_price_limit
            } else {
                self.sqrt_price_target
            };
        if natural_reason_stop_price == self.sqrt_price_edge {
            self.action = match self.order_type {
                OrderType::Buy => SwapStepAction::Crossing,
                OrderType::Sell => SwapStepAction::Stop,
            };
        }
    }

    /// Process a single step of a swap
    fn process_swap(&self) -> Result<SwapStepResult, Error<T>> {
        // total_cost = delta_in / (1 - self.fee_size)
        let fee_rate = U64F64::saturating_from_num(FeeRate::<T>::get(self.netuid));
        let u16_max = U64F64::saturating_from_num(u16::MAX);
        let delta_fixed = U64F64::saturating_from_num(self.delta_in);
        let total_cost =
            delta_fixed.saturating_mul(u16_max.safe_div(u16_max.saturating_sub(fee_rate)));

        // println!("Executing swap step. order_type = {:?}", self.order_type);
        // println!("Executing swap step. delta_in = {:?}", self.delta_in);

        // Hold the fees
        let fee =
            Pallet::<T>::calculate_fee_amount(self.netuid, total_cost.saturating_to_num::<u64>());
        Pallet::<T>::add_fees(self.netuid, self.order_type, fee);
        let delta_out = Pallet::<T>::convert_deltas(self.netuid, self.order_type, self.delta_in);

        // Get current tick
        let current_tick_index = TickIndex::current_bounded::<T>(self.netuid);

        if self.action == SwapStepAction::Crossing {
            let mut tick = match self.order_type {
                OrderType::Sell => {
                    Pallet::<T>::find_closest_lower_active_tick(self.netuid, current_tick_index)
                }
                OrderType::Buy => {
                    Pallet::<T>::find_closest_higher_active_tick(self.netuid, current_tick_index)
                }
            }
            .ok_or(Error::<T>::InsufficientLiquidity)?;

            tick.fees_out_tao =
                FeeGlobalTao::<T>::get(self.netuid).saturating_sub(tick.fees_out_tao);
            tick.fees_out_alpha =
                FeeGlobalAlpha::<T>::get(self.netuid).saturating_sub(tick.fees_out_alpha);
            Pallet::<T>::update_liquidity_at_crossing(self.netuid, self.order_type)?;
            Ticks::<T>::insert(self.netuid, current_tick_index, tick);
        }

        // Update current price
        AlphaSqrtPrice::<T>::set(self.netuid, self.final_price);

        // Update current tick
        let new_current_tick = TickIndex::from_sqrt_price_bounded(self.final_price);
        CurrentTick::<T>::set(self.netuid, new_current_tick);

        Ok(SwapStepResult {
            amount_to_take: total_cost.saturating_to_num::<u64>(),
            fee_paid: fee,
            delta_in: self.delta_in,
            delta_out,
        })
    }

    /// Get the input amount needed to reach the target price
    fn delta_in(
        order_type: OrderType,
        liquidity_curr: U64F64,
        sqrt_price_curr: U64F64,
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
                let sqrt_price = AlphaSqrtPrice::<T>::get(NetUid::from(netuid));
                let tao_reserve = T::SubnetInfo::tao_reserve(netuid.into());
                let alpha_reserve = T::SubnetInfo::alpha_reserve(netuid.into());

                if sqrt_price == 0 && tao_reserve > 0 && alpha_reserve > 0 {
                    U96F32::saturating_from_num(tao_reserve)
                        .saturating_div(U96F32::saturating_from_num(alpha_reserve))
                } else {
                    U96F32::saturating_from_num(sqrt_price.saturating_mul(sqrt_price))
                }
            }
            _ => U96F32::saturating_from_num(1),
        }
    }

    // initializes V3 swap for a subnet if needed
    fn maybe_initialize_v3(netuid: NetUid) -> Result<(), Error<T>> {
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
        let liquidity =
            helpers_128bit::sqrt((tao_reserve as u128).saturating_mul(alpha_reserve as u128))
                as u64;
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

    /// Perform a swap
    ///
    /// Returns a tuple (amount_paid_out, refund), where amount_paid_out is the resulting paid out
    /// amount and refund is any unswapped amount returned to the caller
    ///
    /// The function can be used without writing into the storage by setting `should_rollback` to
    /// `true`.
    pub fn do_swap(
        netuid: NetUid,
        order_type: OrderType,
        amount: u64,
        sqrt_price_limit: SqrtPrice,
        should_rollback: bool,
    ) -> Result<SwapResult, DispatchError> {
        transactional::with_transaction(|| {
            let result =
                Self::swap_inner(netuid, order_type, amount, sqrt_price_limit).map_err(Into::into);

            if should_rollback || result.is_err() {
                TransactionOutcome::Rollback(result)
            } else {
                TransactionOutcome::Commit(result)
            }
        })
    }

    fn swap_inner(
        netuid: NetUid,
        order_type: OrderType,
        amount: u64,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<SwapResult, Error<T>> {
        ensure!(
            T::SubnetInfo::tao_reserve(netuid.into()) >= T::MinimumReserve::get().get()
                && T::SubnetInfo::alpha_reserve(netuid.into()) >= T::MinimumReserve::get().get(),
            Error::<T>::ReservesTooLow
        );

        Self::maybe_initialize_v3(netuid)?;

        let mut amount_remaining = amount;
        let mut amount_paid_out: u64 = 0;
        let mut iteration_counter: u16 = 0;
        let mut in_acc: u64 = 0;
        let mut fee_acc: u64 = 0;

        // Swap one tick at a time until we reach one of the stop conditions
        while amount_remaining > 0 {
            // Create and execute a swap step
            let mut swap_step =
                SwapStep::<T>::new(netuid, order_type, amount_remaining, sqrt_price_limit);

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

        let tao_reserve = T::SubnetInfo::tao_reserve(netuid.into());
        let alpha_reserve = T::SubnetInfo::alpha_reserve(netuid.into());

        let checked_reserve = match order_type {
            OrderType::Buy => alpha_reserve,
            OrderType::Sell => tao_reserve,
        };

        ensure!(
            checked_reserve >= amount_paid_out,
            Error::<T>::InsufficientLiquidity
        );

        let (new_tao_reserve, new_alpha_reserve) = match order_type {
            OrderType::Buy => (
                tao_reserve.saturating_add(in_acc),
                alpha_reserve.saturating_sub(amount_paid_out),
            ),
            OrderType::Sell => (
                tao_reserve.saturating_sub(amount_paid_out),
                alpha_reserve.saturating_add(in_acc),
            ),
        };

        Ok(SwapResult {
            amount_paid_in: in_acc,
            amount_paid_out,
            fee_paid: fee_acc,
            new_tao_reserve,
            new_alpha_reserve,
        })
    }

    /// Get the square root price at the current tick edge for the given direction (order type) If
    /// order type is Buy, then price edge is the high tick bound price, otherwise it is the low
    /// tick bound price.
    ///
    /// If anything is wrong with tick math and it returns Err, we just abort the deal, i.e. return
    /// the edge that is impossible to execute
    fn sqrt_price_edge(netuid: NetUid, current_price: U64F64, order_type: OrderType) -> SqrtPrice {
        // Current price falls into a tick that may or may not be active. This tick represents
        // the set of prices [tick_min_price, tick_max_price) (excluding the tick_max_price point).
        //
        // If this tick is active:
        //    the lower edge (for sell order) is the tick_min_price,
        //    the higher edge (for buy order) is the tick_max_price
        //
        // If this tick is not active:
        //    the lower edge (for sell order) is the lower active tick low price
        //    the higher edge (for buy order) is the higher active tick low price
        //

        let fallback_tick = match order_type {
            OrderType::Buy => TickIndex::MIN,
            OrderType::Sell => TickIndex::MAX,
        };

        let current_price_tick =
            TickIndex::try_from_sqrt_price(current_price).unwrap_or(fallback_tick);
        let roundtrip_current_price = current_price_tick
            .try_to_sqrt_price()
            .unwrap_or(SqrtPrice::from_num(0));

        (match order_type {
            OrderType::Buy => {
                let higher_tick =
                    ActiveTickIndexManager::find_closest_higher::<T>(netuid, current_price_tick)
                        .unwrap_or(TickIndex::MAX);
                if higher_tick < TickIndex::MAX {
                    higher_tick.saturating_add(1)
                } else {
                    higher_tick
                }
            }
            OrderType::Sell => {
                let mut lower_tick =
                    ActiveTickIndexManager::find_closest_lower::<T>(netuid, current_price_tick)
                        .unwrap_or(TickIndex::MIN);

                if current_price == roundtrip_current_price {
                    lower_tick = ActiveTickIndexManager::find_closest_lower::<T>(
                        netuid,
                        lower_tick.prev().unwrap_or(TickIndex::MIN),
                    )
                    .unwrap_or(TickIndex::MIN);
                }
                lower_tick
            }
        })
        .try_to_sqrt_price()
        .unwrap_or(SqrtPrice::from_num(0))
    }

    /// Calculate fee amount
    ///
    /// Fee is provided by state ops as u16-normalized value.
    fn calculate_fee_amount(netuid: NetUid, amount: u64) -> u64 {
        let fee_rate = U64F64::saturating_from_num(FeeRate::<T>::get(netuid))
            .safe_div(U64F64::saturating_from_num(u16::MAX));

        U64F64::saturating_from_num(amount)
            .saturating_mul(fee_rate)
            .saturating_to_num::<u64>()
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
    fn convert_deltas(netuid: NetUid, order_type: OrderType, delta_in: u64) -> u64 {
        // Skip conversion if delta_in is zero
        if delta_in == 0 {
            return 0;
        }

        let liquidity_curr = SqrtPrice::saturating_from_num(CurrentLiquidity::<T>::get(netuid));
        let sqrt_price_curr = AlphaSqrtPrice::<T>::get(netuid);
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
        sqrt_price_curr: U64F64,
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
            OrderType::Sell => Self::find_closest_lower_active_tick(netuid, current_tick_index),
            OrderType::Buy => Self::find_closest_higher_active_tick(netuid, current_tick_index),
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
        ActiveTickIndexManager::find_closest_lower::<T>(netuid, index)
            .and_then(|ti| Ticks::<T>::get(netuid, ti))
    }

    pub fn find_closest_higher_active_tick(netuid: NetUid, index: TickIndex) -> Option<Tick> {
        ActiveTickIndexManager::find_closest_higher::<T>(netuid, index)
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
            T::BalanceOps::tao_balance(coldkey_account_id) >= tao
                && T::BalanceOps::alpha_balance(
                    netuid.into(),
                    coldkey_account_id,
                    hotkey_account_id
                ) >= alpha,
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
    ) -> Result<(Position, u64, u64), Error<T>> {
        ensure!(
            Self::count_positions(netuid, coldkey_account_id) <= T::MaxPositions::get() as usize,
            Error::<T>::MaxPositionsExceeded
        );

        // Add liquidity at tick
        Self::add_liquidity_at_index(netuid, tick_low, liquidity, false);
        Self::add_liquidity_at_index(netuid, tick_high, liquidity, true);

        // Update current tick liquidity
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);
        Self::clamp_sqrt_price(netuid, current_tick_index);

        Self::update_liquidity_if_needed(netuid, tick_low, tick_high, liquidity as i128);

        // New position
        let position_id = PositionId::new::<T>();
        let position = Position {
            id: position_id,
            netuid,
            tick_low,
            tick_high,
            liquidity,
            fees_tao: U64F64::saturating_from_num(0),
            fees_alpha: U64F64::saturating_from_num(0),
        };

        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let (tao, alpha) = position.to_token_amounts(current_price)?;

        SwapV3Initialized::<T>::set(netuid, true);

        Ok((position, tao, alpha))
    }

    /// Remove liquidity and credit balances back to (coldkey_account_id, hotkey_account_id) stake
    ///
    /// Account ID and Position ID identify position in the storage map
    pub fn do_remove_liquidity(
        netuid: NetUid,
        coldkey_account_id: &T::AccountId,
        position_id: PositionId,
    ) -> Result<UpdateLiquidityResult, Error<T>> {
        ensure!(
            EnabledUserLiquidity::<T>::get(netuid),
            Error::<T>::UserLiquidityDisabled
        );

        let Some(mut position) = Positions::<T>::get((netuid, coldkey_account_id, position_id))
        else {
            return Err(Error::<T>::LiquidityNotFound);
        };

        // Collect fees and get tao and alpha amounts
        let (fee_tao, fee_alpha) = position.collect_fees::<T>();
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

        Ok(UpdateLiquidityResult {
            tao,
            alpha,
            fee_tao,
            fee_alpha,
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
        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let sqrt_pa: SqrtPrice = position
            .tick_low
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_pb: SqrtPrice = position
            .tick_high
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_price_box = if current_price < sqrt_pa {
            sqrt_pa
        } else if current_price > sqrt_pb {
            sqrt_pb
        } else {
            // Update current liquidity if price is in range
            let new_liquidity_curr = if liquidity_delta > 0 {
                CurrentLiquidity::<T>::get(netuid).saturating_add(delta_liquidity_abs)
            } else {
                CurrentLiquidity::<T>::get(netuid).saturating_sub(delta_liquidity_abs)
            };
            CurrentLiquidity::<T>::set(netuid, new_liquidity_curr);
            current_price
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
                T::BalanceOps::tao_balance(coldkey_account_id) >= tao
                    && T::BalanceOps::alpha_balance(
                        netuid.into(),
                        coldkey_account_id,
                        hotkey_account_id
                    ) >= alpha,
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
        let (fee_tao, fee_alpha) = position.collect_fees::<T>();

        // If delta brings the position liquidity below MinimumLiquidity, eliminate position and withdraw full amounts
        if (liquidity_delta < 0)
            && (position.liquidity.saturating_sub(delta_liquidity_abs) < T::MinimumLiquidity::get())
        {
            delta_liquidity_abs = position.liquidity;
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
            Self::remove_liquidity_at_index(netuid, position.tick_low, position.liquidity, false);
            Self::remove_liquidity_at_index(netuid, position.tick_high, position.liquidity, true);

            // Remove liquidity from user position
            position.liquidity = position.liquidity.saturating_sub(delta_liquidity_abs);
        }
        Positions::<T>::insert(&(netuid, coldkey_account_id, position.id), position);

        // TODO: Withdraw balances and update pool reserves

        Ok(UpdateLiquidityResult {
            tao: tao.saturating_to_num::<u64>(),
            alpha: alpha.saturating_to_num::<u64>(),
            fee_tao,
            fee_alpha,
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
                *maybe_tick = Some(Tick {
                    liquidity_net: net_liquidity_change,
                    liquidity_gross: liquidity,
                    fees_out_tao: U64F64::from_num(0),
                    fees_out_alpha: U64F64::from_num(0),
                });
            }
        });

        // Update active ticks
        ActiveTickIndexManager::insert::<T>(netuid, tick_index);
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
                    ActiveTickIndexManager::remove::<T>(netuid, tick_index);
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
        if (tick_low <= current_tick_index) && (current_tick_index <= tick_high) {
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
    fn count_positions(netuid: NetUid, account_id: &T::AccountId) -> usize {
        Positions::<T>::iter_prefix_values((netuid, account_id.clone())).count()
    }

    /// Returns the protocol account ID
    ///
    /// # Returns
    /// The account ID of the protocol account
    pub fn protocol_account_id() -> T::AccountId {
        T::ProtocolId::get().into_account_truncating()
    }
}

impl<T: Config> SwapHandler<T::AccountId> for Pallet<T> {
    fn swap(
        netuid: u16,
        order_t: OrderType,
        amount: u64,
        price_limit: u64,
        should_rollback: bool,
    ) -> Result<SwapResult, DispatchError> {
        let sqrt_price_limit = SqrtPrice::saturating_from_num(price_limit)
            .safe_div(SqrtPrice::saturating_from_num(1_000_000_000))
            .checked_sqrt(SqrtPrice::saturating_from_num(0.0000000001))
            .ok_or(Error::<T>::PriceLimitExceeded)?;

        Self::do_swap(
            NetUid::from(netuid),
            order_t,
            amount,
            sqrt_price_limit,
            should_rollback,
        )
        .map_err(Into::into)
    }

    fn sim_swap(netuid: u16, order_t: OrderType, amount: u64) -> Result<SwapResult, DispatchError> {
        match T::SubnetInfo::mechanism(netuid) {
            1 => {
                let price_limit = match order_t {
                    OrderType::Buy => Self::max_price(),
                    OrderType::Sell => Self::min_price(),
                };

                Self::swap(netuid, order_t, amount, price_limit, true)
            }
            _ => Ok(SwapResult {
                amount_paid_in: amount,
                amount_paid_out: amount,
                fee_paid: 0,
                new_tao_reserve: 0,
                new_alpha_reserve: 0,
            }),
        }
    }

    fn approx_fee_amount(netuid: u16, amount: u64) -> u64 {
        Self::calculate_fee_amount(netuid.into(), amount)
    }

    fn current_alpha_price(netuid: u16) -> U96F32 {
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

// cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests --show-output
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
#[allow(clippy::arithmetic_side_effects)]
#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use frame_support::{assert_err, assert_noop, assert_ok};
    use sp_arithmetic::helpers_128bit;

    use super::*;
    use crate::{mock::*, pallet::*};

    // this function is used to convert price (NON-SQRT price!) to TickIndex. it's only utility for
    // testing, all the implementation logic is based on sqrt prices
    fn price_to_tick(price: f64) -> TickIndex {
        let price_sqrt: SqrtPrice = SqrtPrice::from_num(price.sqrt());
        // Handle potential errors in the conversion
        match TickIndex::try_from_sqrt_price(price_sqrt) {
            Ok(mut tick) => {
                // Ensure the tick is within bounds
                if tick > TickIndex::MAX {
                    tick = TickIndex::MAX;
                } else if tick < TickIndex::MIN {
                    tick = TickIndex::MIN;
                }
                tick
            }
            // Default to a reasonable value when conversion fails
            Err(_) => {
                if price > 1.0 {
                    TickIndex::MAX
                } else {
                    TickIndex::MIN
                }
            }
        }
    }

    fn get_ticked_prices_around_current_price() -> (f64, f64) {
        // Get current price, ticks around it, and prices on the tick edges for test cases
        let netuid = NetUid::from(1);
        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
        let current_price_sqrt = AlphaSqrtPrice::<Test>::get(netuid);
        let tick_index_for_current_price_low =
            TickIndex::try_from_sqrt_price(current_price_sqrt).unwrap();
        let tick_index_for_current_price_high = tick_index_for_current_price_low
            .next()
            .unwrap()
            .next()
            .unwrap();

        // Low and high prices that match to a lower and higher tick that doesn't contain the current price
        let current_price_low_sqrt =
            TickIndex::try_to_sqrt_price(&tick_index_for_current_price_low)
                .unwrap()
                .to_num::<f64>();
        let current_price_high_sqrt =
            TickIndex::try_to_sqrt_price(&tick_index_for_current_price_high)
                .unwrap()
                .to_num::<f64>();
        let current_price_low = current_price_low_sqrt * current_price_low_sqrt;
        let current_price_high = current_price_high_sqrt * current_price_high_sqrt;

        (current_price_low, current_price_high)
    }

    // this function is used to convert tick index NON-SQRT (!) price. it's only utility for
    // testing, all the implementation logic is based on sqrt prices
    fn tick_to_price(tick: TickIndex) -> f64 {
        // Handle errors gracefully
        match tick.try_to_sqrt_price() {
            Ok(price_sqrt) => (price_sqrt * price_sqrt).to_num::<f64>(),
            Err(_) => {
                // Return a sensible default based on whether the tick is above or below the valid range
                if tick > TickIndex::MAX {
                    tick_to_price(TickIndex::MAX) // Use the max valid tick price
                } else {
                    tick_to_price(TickIndex::MIN) // Use the min valid tick price
                }
            }
        }
    }

    #[test]
    fn test_swap_initialization() {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(1);

            // Get reserves from the mock provider
            let tao = MockLiquidityProvider::tao_reserve(netuid.into());
            let alpha = MockLiquidityProvider::alpha_reserve(netuid.into());

            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            assert!(SwapV3Initialized::<Test>::get(netuid));

            // Verify current price is set
            let sqrt_price = AlphaSqrtPrice::<Test>::get(netuid);
            let expected_sqrt_price = U64F64::from_num(0.5_f64);
            assert_abs_diff_eq!(
                sqrt_price.to_num::<f64>(),
                expected_sqrt_price.to_num::<f64>(),
                epsilon = 0.000000001
            );

            // Verify that current tick is set
            let current_tick = CurrentTick::<Test>::get(netuid);
            let expected_current_tick = TickIndex::from_sqrt_price_bounded(expected_sqrt_price);
            assert_eq!(current_tick, expected_current_tick);

            // Calculate expected liquidity
            let expected_liquidity =
                helpers_128bit::sqrt((tao as u128).saturating_mul(alpha as u128)) as u64;

            // Get the protocol account
            let protocol_account_id = Pallet::<Test>::protocol_account_id();

            // Verify position created for protocol account
            let positions = Positions::<Test>::iter_prefix_values((netuid, protocol_account_id))
                .collect::<Vec<_>>();
            assert_eq!(positions.len(), 1);

            let position = &positions[0];
            assert_eq!(position.liquidity, expected_liquidity);
            assert_eq!(position.tick_low, TickIndex::MIN);
            assert_eq!(position.tick_high, TickIndex::MAX);
            assert_eq!(position.fees_tao, 0);
            assert_eq!(position.fees_alpha, 0);

            // Verify ticks were created
            let tick_low = Ticks::<Test>::get(netuid, TickIndex::MIN).unwrap();
            let tick_high = Ticks::<Test>::get(netuid, TickIndex::MAX).unwrap();

            // Check liquidity values
            assert_eq!(tick_low.liquidity_net, expected_liquidity as i128);
            assert_eq!(tick_low.liquidity_gross, expected_liquidity);
            assert_eq!(tick_high.liquidity_net, -(expected_liquidity as i128));
            assert_eq!(tick_high.liquidity_gross, expected_liquidity);

            // Verify current liquidity is set
            assert_eq!(CurrentLiquidity::<Test>::get(netuid), expected_liquidity);
        });
    }

    // Test adding liquidity on top of the existing protocol liquidity
    #[test]
    fn test_add_liquidity_basic() {
        new_test_ext().execute_with(|| {
            let min_price = tick_to_price(TickIndex::MIN);
            let max_price = tick_to_price(TickIndex::MAX);
            let max_tick = price_to_tick(max_price);
            let current_price = 0.25;
            assert_eq!(max_tick, TickIndex::MAX);

            let (current_price_low, current_price_high) = get_ticked_prices_around_current_price();

            // As a user add liquidity with all possible corner cases
            //   - Initial price is 0.25
            //   - liquidity is expressed in RAO units
            // Test case is (price_low, price_high, liquidity, tao, alpha)
            [
                // Repeat the protocol liquidity at maximum range: Expect all the same values
                (
                    min_price,
                    max_price,
                    2_000_000_000_u64,
                    1_000_000_000_u64,
                    4_000_000_000_u64,
                ),
                // Repeat the protocol liquidity at current to max range: Expect the same alpha
                (
                    current_price_high,
                    max_price,
                    2_000_000_000_u64,
                    0,
                    4_000_000_000,
                ),
                // Repeat the protocol liquidity at min to current range: Expect all the same tao
                (
                    min_price,
                    current_price_low,
                    2_000_000_000_u64,
                    1_000_000_000,
                    0,
                ),
                // Half to double price - just some sane wothdraw amounts
                (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
                // Both below price - tao is non-zero, alpha is zero
                (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
                // Both above price - tao is zero, alpha is non-zero
                (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
            ]
            .into_iter()
            .enumerate()
            .map(|(n, v)| (NetUid::from(n as u16 + 1), v.0, v.1, v.2, v.3, v.4))
            .for_each(
                |(netuid, price_low, price_high, liquidity, expected_tao, expected_alpha)| {
                    assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

                    // Calculate ticks (assuming tick math is tested separately)
                    let tick_low = price_to_tick(price_low);
                    let tick_high = price_to_tick(price_high);

                    // Get tick infos and liquidity before adding (to account for protocol liquidity)
                    let tick_low_info_before =
                        Ticks::<Test>::get(netuid, tick_low).unwrap_or_default();
                    let tick_high_info_before =
                        Ticks::<Test>::get(netuid, tick_high).unwrap_or_default();
                    let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

                    // Add liquidity
                    let (position_id, tao, alpha) = Pallet::<Test>::do_add_liquidity(
                        netuid,
                        &OK_COLDKEY_ACCOUNT_ID,
                        &OK_HOTKEY_ACCOUNT_ID,
                        tick_low,
                        tick_high,
                        liquidity,
                    )
                    .unwrap();

                    assert_abs_diff_eq!(tao, expected_tao, epsilon = tao / 1000);
                    assert_abs_diff_eq!(alpha, expected_alpha, epsilon = alpha / 1000);

                    // Check that low and high ticks appear in the state and are properly updated
                    let tick_low_info = Ticks::<Test>::get(netuid, tick_low).unwrap();
                    let tick_high_info = Ticks::<Test>::get(netuid, tick_high).unwrap();
                    let expected_liquidity_net_low = liquidity as i128;
                    let expected_liquidity_gross_low = liquidity;
                    let expected_liquidity_net_high = -(liquidity as i128);
                    let expected_liquidity_gross_high = liquidity;

                    assert_eq!(
                        tick_low_info.liquidity_net - tick_low_info_before.liquidity_net,
                        expected_liquidity_net_low,
                    );
                    assert_eq!(
                        tick_low_info.liquidity_gross - tick_low_info_before.liquidity_gross,
                        expected_liquidity_gross_low,
                    );
                    assert_eq!(
                        tick_high_info.liquidity_net - tick_high_info_before.liquidity_net,
                        expected_liquidity_net_high,
                    );
                    assert_eq!(
                        tick_high_info.liquidity_gross - tick_high_info_before.liquidity_gross,
                        expected_liquidity_gross_high,
                    );

                    // Liquidity position at correct ticks
                    assert_eq!(
                        Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                        1
                    );

                    let position =
                        Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id))
                            .unwrap();
                    assert_eq!(position.liquidity, liquidity);
                    assert_eq!(position.tick_low, tick_low);
                    assert_eq!(position.tick_high, tick_high);
                    assert_eq!(position.fees_alpha, 0);
                    assert_eq!(position.fees_tao, 0);

                    // Current liquidity is updated only when price range includes the current price
                    let expected_liquidity =
                        if (price_high >= current_price) && (price_low <= current_price) {
                            liquidity_before + liquidity
                        } else {
                            liquidity_before
                        };

                    assert_eq!(CurrentLiquidity::<Test>::get(netuid), expected_liquidity)
                },
            );
        });
    }

    #[test]
    fn test_add_liquidity_out_of_bounds() {
        new_test_ext().execute_with(|| {
            [
                // For our tests, we'll construct TickIndex values that are intentionally
                // outside the valid range for testing purposes only
                (
                    TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
                    TickIndex::MAX,
                    1_000_000_000_u64,
                ),
                (
                    TickIndex::MIN,
                    TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
                    1_000_000_000_u64,
                ),
                (
                    TickIndex::new_unchecked(TickIndex::MIN.get() - 1),
                    TickIndex::new_unchecked(TickIndex::MAX.get() + 1),
                    1_000_000_000_u64,
                ),
                (
                    TickIndex::new_unchecked(TickIndex::MIN.get() - 100),
                    TickIndex::new_unchecked(TickIndex::MAX.get() + 100),
                    1_000_000_000_u64,
                ),
            ]
            .into_iter()
            .enumerate()
            .map(|(n, v)| (NetUid::from(n as u16), v.0, v.1, v.2))
            .for_each(|(netuid, tick_low, tick_high, liquidity)| {
                assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

                // Add liquidity
                assert_err!(
                    Swap::do_add_liquidity(
                        netuid,
                        &OK_COLDKEY_ACCOUNT_ID,
                        &OK_HOTKEY_ACCOUNT_ID,
                        tick_low,
                        tick_high,
                        liquidity
                    ),
                    Error::<Test>::InvalidTickRange,
                );
            });
        });
    }

    #[test]
    fn test_add_liquidity_over_balance() {
        new_test_ext().execute_with(|| {
            let coldkey_account_id = 2;
            let hotkey_account_id = 3;

            [
                // Lower than price (not enough alpha)
                (0.1, 0.2, 100_000_000_000_u64),
                // Higher than price (not enough tao)
                (0.3, 0.4, 100_000_000_000_u64),
                // Around the price (not enough both)
                (0.1, 0.4, 100_000_000_000_u64),
            ]
            .into_iter()
            .enumerate()
            .map(|(n, v)| (NetUid::from(n as u16), v.0, v.1, v.2))
            .for_each(|(netuid, price_low, price_high, liquidity)| {
                // Calculate ticks
                let tick_low = price_to_tick(price_low);
                let tick_high = price_to_tick(price_high);

                assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

                // Add liquidity
                assert_err!(
                    Pallet::<Test>::do_add_liquidity(
                        netuid,
                        &coldkey_account_id,
                        &hotkey_account_id,
                        tick_low,
                        tick_high,
                        liquidity
                    ),
                    Error::<Test>::InsufficientBalance,
                );
            });
        });
    }

    // cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_remove_liquidity_basic --exact --show-output
    #[test]
    fn test_remove_liquidity_basic() {
        new_test_ext().execute_with(|| {
            let min_price = tick_to_price(TickIndex::MIN);
            let max_price = tick_to_price(TickIndex::MAX);
            let max_tick = price_to_tick(max_price);
            assert_eq!(max_tick, TickIndex::MAX);

            let (current_price_low, current_price_high) = get_ticked_prices_around_current_price();

            // As a user add liquidity with all possible corner cases
            //   - Initial price is 0.25
            //   - liquidity is expressed in RAO units
            // Test case is (price_low, price_high, liquidity, tao, alpha)
            [
                // Repeat the protocol liquidity at maximum range: Expect all the same values
                (
                    min_price,
                    max_price,
                    2_000_000_000_u64,
                    1_000_000_000_u64,
                    4_000_000_000_u64,
                ),
                // Repeat the protocol liquidity at current to max range: Expect the same alpha
                (
                    current_price_high,
                    max_price,
                    2_000_000_000_u64,
                    0,
                    4_000_000_000,
                ),
                // Repeat the protocol liquidity at min to current range: Expect all the same tao
                (
                    min_price,
                    current_price_low,
                    2_000_000_000_u64,
                    1_000_000_000,
                    0,
                ),
                // Half to double price - just some sane wothdraw amounts
                (0.125, 0.5, 2_000_000_000_u64, 293_000_000, 1_171_000_000),
                // Both below price - tao is non-zero, alpha is zero
                (0.12, 0.13, 2_000_000_000_u64, 28_270_000, 0),
                // Both above price - tao is zero, alpha is non-zero
                (0.3, 0.4, 2_000_000_000_u64, 0, 489_200_000),
            ]
            .into_iter()
            .enumerate()
            .map(|(n, v)| (NetUid::from(n as u16), v.0, v.1, v.2, v.3, v.4))
            .for_each(|(netuid, price_low, price_high, liquidity, tao, alpha)| {
                // Calculate ticks (assuming tick math is tested separately)
                let tick_low = price_to_tick(price_low);
                let tick_high = price_to_tick(price_high);

                assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
                let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

                // Add liquidity
                let (position_id, _, _) = Pallet::<Test>::do_add_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    liquidity,
                )
                .unwrap();

                // Remove liquidity
                let remove_result = Pallet::<Test>::do_remove_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    position_id,
                )
                .unwrap();
                assert_abs_diff_eq!(remove_result.tao, tao, epsilon = tao / 1000);
                assert_abs_diff_eq!(remove_result.alpha, alpha, epsilon = alpha / 1000);
                assert_eq!(remove_result.fee_tao, 0);
                assert_eq!(remove_result.fee_alpha, 0);

                // Liquidity position is removed
                assert_eq!(
                    Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                    0
                );
                assert!(
                    Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).is_none()
                );

                // Current liquidity is updated (back where it was)
                assert_eq!(CurrentLiquidity::<Test>::get(netuid), liquidity_before);
            });
        });
    }

    #[test]
    fn test_remove_liquidity_nonexisting_position() {
        new_test_ext().execute_with(|| {
            let min_price = tick_to_price(TickIndex::MIN);
            let max_price = tick_to_price(TickIndex::MAX);
            let max_tick = price_to_tick(max_price);
            assert_eq!(max_tick.get(), TickIndex::MAX.get());

            let liquidity = 2_000_000_000_u64;
            let netuid = NetUid::from(1);

            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(min_price);
            let tick_high = price_to_tick(max_price);

            // Setup swap
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Add liquidity
            assert_ok!(Pallet::<Test>::do_add_liquidity(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                tick_low,
                tick_high,
                liquidity,
            ));

            assert!(Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID) > 0);

            // Remove liquidity
            assert_err!(
                Pallet::<Test>::do_remove_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    PositionId::new::<Test>()
                ),
                Error::<Test>::LiquidityNotFound,
            );
        });
    }

    #[test]
    fn test_modify_position_basic() {
        new_test_ext().execute_with(|| {
            let max_price = tick_to_price(TickIndex::MAX);
            let max_tick = price_to_tick(max_price);
            let limit_price = 1000.0_f64;
            assert_eq!(max_tick, TickIndex::MAX);
            let (_current_price_low, current_price_high) = get_ticked_prices_around_current_price();

            // As a user add liquidity with all possible corner cases
            //   - Initial price is 0.25
            //   - liquidity is expressed in RAO units
            // Test case is (price_low, price_high, liquidity, tao, alpha)
            [
                // Repeat the protocol liquidity at current to max range: Expect the same alpha
                (
                    current_price_high,
                    max_price,
                    2_000_000_000_u64,
                    4_000_000_000,
                ),
            ]
            .into_iter()
            .enumerate()
            .map(|(n, v)| (NetUid::from(n as u16), v.0, v.1, v.2, v.3))
            .for_each(|(netuid, price_low, price_high, liquidity, alpha)| {
                // Calculate ticks (assuming tick math is tested separately)
                let tick_low = price_to_tick(price_low);
                let tick_high = price_to_tick(price_high);

                assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

                // Add liquidity
                let (position_id, _, _) = Pallet::<Test>::do_add_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    liquidity,
                )
                .unwrap();

                // Swap to create fees on the position
                let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
                Pallet::<Test>::do_swap(
                    netuid,
                    OrderType::Buy,
                    liquidity / 10,
                    sqrt_limit_price,
                    false,
                )
                .unwrap();

                // Modify liquidity (also causes claiming of fees)
                let liquidity_before = CurrentLiquidity::<Test>::get(netuid);
                let modify_result = Pallet::<Test>::do_modify_position(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    position_id,
                    -((liquidity / 10) as i64),
                )
                .unwrap();
                assert_abs_diff_eq!(modify_result.alpha, alpha / 10, epsilon = alpha / 1000);
                assert!(modify_result.fee_tao > 0);
                assert_eq!(modify_result.fee_alpha, 0);

                // Liquidity position is reduced
                assert_eq!(
                    Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                    1
                );

                // Current liquidity is reduced with modify_position
                assert!(CurrentLiquidity::<Test>::get(netuid) < liquidity_before);

                // Position liquidity is reduced
                let position =
                    Positions::<Test>::get((netuid, OK_COLDKEY_ACCOUNT_ID, position_id)).unwrap();
                assert_eq!(position.liquidity, liquidity * 9 / 10);
                assert_eq!(position.tick_low, tick_low);
                assert_eq!(position.tick_high, tick_high);

                // Modify liquidity again (ensure fees aren't double-collected)
                let modify_result = Pallet::<Test>::do_modify_position(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    position_id,
                    -((liquidity / 100) as i64),
                )
                .unwrap();

                assert_abs_diff_eq!(modify_result.alpha, alpha / 100, epsilon = alpha / 1000);
                assert_eq!(modify_result.fee_tao, 0);
                assert_eq!(modify_result.fee_alpha, 0);
            });
        });
    }

    // cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_basic --exact --show-output
    #[test]
    fn test_swap_basic() {
        new_test_ext().execute_with(|| {
            // Current price is 0.25
            // Test case is (order_type, liquidity, limit_price, output_amount)
            [
                (OrderType::Buy, 1_000u64, 1000.0_f64, 3990_u64),
                (OrderType::Sell, 1_000u64, 0.0001_f64, 250_u64),
                (OrderType::Buy, 500_000_000, 1000.0, 2_000_000_000),
            ]
            .into_iter()
            .enumerate()
            .map(|(n, v)| (NetUid::from(n as u16), v.0, v.1, v.2, v.3))
            .for_each(
                |(netuid, order_type, liquidity, limit_price, output_amount)| {
                    // Consumed liquidity ticks
                    let tick_low = TickIndex::MIN;
                    let tick_high = TickIndex::MAX;

                    // Setup swap
                    assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

                    // Get tick infos before the swap
                    let tick_low_info_before =
                        Ticks::<Test>::get(netuid, tick_low).unwrap_or_default();
                    let tick_high_info_before =
                        Ticks::<Test>::get(netuid, tick_high).unwrap_or_default();
                    let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

                    // Get current price
                    let sqrt_current_price = AlphaSqrtPrice::<Test>::get(netuid);
                    let current_price = (sqrt_current_price * sqrt_current_price).to_num::<f64>();

                    // Swap
                    let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
                    let swap_result = Pallet::<Test>::do_swap(
                        netuid,
                        order_type,
                        liquidity,
                        sqrt_limit_price,
                        false,
                    )
                    .unwrap();
                    assert_abs_diff_eq!(
                        swap_result.amount_paid_out,
                        output_amount,
                        epsilon = output_amount / 100
                    );

                    let (tao_expected, alpha_expected) = match order_type {
                        OrderType::Buy => (
                            MockLiquidityProvider::tao_reserve(netuid.into()) + liquidity,
                            MockLiquidityProvider::alpha_reserve(netuid.into()) - output_amount,
                        ),
                        OrderType::Sell => (
                            MockLiquidityProvider::tao_reserve(netuid.into()) + output_amount,
                            MockLiquidityProvider::alpha_reserve(netuid.into()) - liquidity,
                        ),
                    };

                    assert_abs_diff_eq!(
                        swap_result.new_alpha_reserve,
                        alpha_expected,
                        epsilon = alpha_expected / 100
                    );
                    assert_abs_diff_eq!(
                        swap_result.new_tao_reserve,
                        tao_expected,
                        epsilon = tao_expected / 100
                    );

                    // Check that low and high ticks' fees were updated properly, and liquidity values were not updated
                    let tick_low_info = Ticks::<Test>::get(netuid, tick_low).unwrap();
                    let tick_high_info = Ticks::<Test>::get(netuid, tick_high).unwrap();
                    let expected_liquidity_net_low = tick_low_info_before.liquidity_net;
                    let expected_liquidity_gross_low = tick_low_info_before.liquidity_gross;
                    let expected_liquidity_net_high = tick_high_info_before.liquidity_net;
                    let expected_liquidity_gross_high = tick_high_info_before.liquidity_gross;
                    assert_eq!(tick_low_info.liquidity_net, expected_liquidity_net_low,);
                    assert_eq!(tick_low_info.liquidity_gross, expected_liquidity_gross_low,);
                    assert_eq!(tick_high_info.liquidity_net, expected_liquidity_net_high,);
                    assert_eq!(
                        tick_high_info.liquidity_gross,
                        expected_liquidity_gross_high,
                    );

                    // Expected fee amount
                    let fee_rate = FeeRate::<Test>::get(netuid) as f64 / u16::MAX as f64;
                    let expected_fee = (liquidity as f64 * fee_rate) as u64;

                    // Global fees should be updated
                    let actual_global_fee = ((match order_type {
                        OrderType::Buy => FeeGlobalTao::<Test>::get(netuid),
                        OrderType::Sell => FeeGlobalAlpha::<Test>::get(netuid),
                    })
                    .to_num::<f64>()
                        * (liquidity_before as f64))
                        as u64;

                    assert!((swap_result.fee_paid as i64 - expected_fee as i64).abs() <= 1);
                    assert!((actual_global_fee as i64 - expected_fee as i64).abs() <= 1);

                    // Tick fees should be updated

                    // Liquidity position should not be updated
                    let protocol_id = Pallet::<Test>::protocol_account_id();
                    let positions = Positions::<Test>::iter_prefix_values((netuid, protocol_id))
                        .collect::<Vec<_>>();
                    let position = positions.first().unwrap();

                    assert_eq!(
                        position.liquidity,
                        helpers_128bit::sqrt(
                            MockLiquidityProvider::tao_reserve(netuid.into()) as u128
                                * MockLiquidityProvider::alpha_reserve(netuid.into()) as u128
                        ) as u64
                    );
                    assert_eq!(position.tick_low, tick_low);
                    assert_eq!(position.tick_high, tick_high);
                    assert_eq!(position.fees_alpha, 0);
                    assert_eq!(position.fees_tao, 0);

                    // Current liquidity is not updated
                    assert_eq!(CurrentLiquidity::<Test>::get(netuid), liquidity_before);

                    // Assert that price movement is in correct direction
                    let sqrt_current_price_after = AlphaSqrtPrice::<Test>::get(netuid);
                    let current_price_after =
                        (sqrt_current_price_after * sqrt_current_price_after).to_num::<f64>();
                    match order_type {
                        OrderType::Buy => assert!(current_price_after > current_price),
                        OrderType::Sell => assert!(current_price_after < current_price),
                    }

                    // Assert that current tick is updated
                    let current_tick = CurrentTick::<Test>::get(netuid);
                    let expected_current_tick =
                        TickIndex::from_sqrt_price_bounded(sqrt_current_price_after);
                    assert_eq!(current_tick, expected_current_tick);
                },
            );
        });
    }

    // In this test the swap starts and ends within one (large liquidity) position
    // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_single_position --exact --show-output --nocapture
    #[test]
    fn test_swap_single_position() {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        let current_price = 0.25;
        let netuid = NetUid(1);
        assert_eq!(max_tick, TickIndex::MAX);

        let mut current_price_low = 0_f64;
        let mut current_price_high = 0_f64;
        new_test_ext().execute_with(|| {
            let (low, high) = get_ticked_prices_around_current_price();
            current_price_low = low;
            current_price_high = high;
        });

        // Current price is 0.25
        // The test case is based on the current price and position prices are defined as a price
        // offset from the current price
        // Outer part of test case is Position: (price_low_offset, price_high_offset, liquidity)
        [
            // Very localized position at the current price
            (-0.1, 0.1, 500_000_000_000_u64),
            // Repeat the protocol liquidity at maximum range
            (
                min_price - current_price,
                max_price - current_price,
                2_000_000_000_u64,
            ),
            // Repeat the protocol liquidity at current to max range
            (
                current_price_high - current_price,
                max_price - current_price,
                2_000_000_000_u64,
            ),
            // Repeat the protocol liquidity at min to current range
            (
                min_price - current_price,
                current_price_low - current_price,
                2_000_000_000_u64,
            ),
            // Half to double price
            (-0.125, 0.25, 2_000_000_000_u64),
            // A few other price ranges and liquidity volumes
            (-0.1, 0.1, 2_000_000_000_u64),
            (-0.1, 0.1, 10_000_000_000_u64),
            (-0.1, 0.1, 100_000_000_000_u64),
            (-0.01, 0.01, 100_000_000_000_u64),
            (-0.001, 0.001, 100_000_000_000_u64),
        ]
        .into_iter()
        .for_each(
            |(price_low_offset, price_high_offset, position_liquidity)| {
                // Inner part of test case is Order: (order_type, order_liquidity, limit_price)
                // order_liquidity is represented as a fraction of position_liquidity
                [
                    // (OrderType::Buy, 0.0001, 1000.0_f64),
                    // (OrderType::Sell, 0.0001, 0.0001_f64),
                    // (OrderType::Buy, 0.001, 1000.0_f64),
                    // (OrderType::Sell, 0.001, 0.0001_f64),
                    // (OrderType::Buy, 0.01, 1000.0_f64),
                    // (OrderType::Sell, 0.01, 0.0001_f64),
                    (OrderType::Buy, 0.1, 1000.0_f64),
                    (OrderType::Sell, 0.1, 0.0001),
                    // (OrderType::Buy, 0.2, 1000.0_f64),
                    // (OrderType::Sell, 0.2, 0.0001),
                    // (OrderType::Buy, 0.5, 1000.0),
                    // (OrderType::Sell, 0.5, 0.0001),
                    // (OrderType::Buy, 0.9999, 1000.0),
                    // (OrderType::Sell, 0.9999, 0.0001),
                    // (OrderType::Buy, 1.0, 1000.0),
                    // (OrderType::Sell, 1.0, 0.0001),
                ]
                .into_iter()
                .for_each(|(order_type, order_liquidity_fraction, limit_price)| {
                    new_test_ext().execute_with(|| {
                        //////////////////////////////////////////////
                        // Initialize pool and add the user position
                        assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));
                        let tao_reserve = MockLiquidityProvider::tao_reserve(netuid.into());
                        let alpha_reserve = MockLiquidityProvider::alpha_reserve(netuid.into());
                        let protocol_liquidity = (tao_reserve as f64 * alpha_reserve as f64).sqrt();

                        // Add liquidity
                        let sqrt_current_price = AlphaSqrtPrice::<Test>::get(netuid);
                        let current_price =
                            (sqrt_current_price * sqrt_current_price).to_num::<f64>();

                        let price_low = price_low_offset + current_price;
                        let price_high = price_high_offset + current_price;
                        let tick_low = price_to_tick(price_low);
                        let tick_high = price_to_tick(price_high);
                        let (_position_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
                            netuid,
                            &OK_COLDKEY_ACCOUNT_ID,
                            &OK_HOTKEY_ACCOUNT_ID,
                            tick_low,
                            tick_high,
                            position_liquidity,
                        )
                        .unwrap();

                        // Liquidity position at correct ticks
                        assert_eq!(
                            Pallet::<Test>::count_positions(netuid, &OK_COLDKEY_ACCOUNT_ID),
                            1
                        );

                        // Get tick infos before the swap
                        let tick_low_info_before =
                            Ticks::<Test>::get(netuid, tick_low).unwrap_or_default();
                        let tick_high_info_before =
                            Ticks::<Test>::get(netuid, tick_high).unwrap_or_default();
                        let liquidity_before = CurrentLiquidity::<Test>::get(netuid);
                        assert_abs_diff_eq!(
                            liquidity_before as f64,
                            protocol_liquidity + position_liquidity as f64,
                            epsilon = liquidity_before as f64 / 1000.
                        );

                        //////////////////////////////////////////////
                        // Swap

                        // Calculate the expected output amount for the cornercase of one step
                        let order_liquidity = order_liquidity_fraction * position_liquidity as f64;

                        let output_amount = match order_type {
                            OrderType::Buy => {
                                let denom = sqrt_current_price.to_num::<f64>()
                                    * (sqrt_current_price.to_num::<f64>()
                                        * liquidity_before as f64
                                        + order_liquidity);
                                let per_order_liq = liquidity_before as f64 / denom;
                                per_order_liq * order_liquidity
                            }
                            OrderType::Sell => {
                                let denom = liquidity_before as f64
                                    / sqrt_current_price.to_num::<f64>()
                                    + order_liquidity;
                                let per_order_liq = sqrt_current_price.to_num::<f64>()
                                    * liquidity_before as f64
                                    / denom;
                                per_order_liq * order_liquidity
                            }
                        };

                        // Do the swap
                        let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
                        let swap_result = Pallet::<Test>::do_swap(
                            netuid,
                            order_type,
                            order_liquidity as u64,
                            sqrt_limit_price,
                            false,
                        )
                        .unwrap();
                        assert_abs_diff_eq!(
                            swap_result.amount_paid_out as f64,
                            output_amount,
                            epsilon = output_amount / 10.
                        );

                        if order_liquidity_fraction <= 0.001 {
                            let tao_reserve_f64 = tao_reserve as f64;
                            let alpha_reserve_f64 = alpha_reserve as f64;
                            let (tao_expected, alpha_expected) = match order_type {
                                OrderType::Buy => (
                                    tao_reserve_f64 + order_liquidity,
                                    alpha_reserve_f64 - output_amount,
                                ),
                                OrderType::Sell => (
                                    tao_reserve_f64 - output_amount,
                                    alpha_reserve_f64 + order_liquidity,
                                ),
                            };
                            assert_abs_diff_eq!(
                                swap_result.new_alpha_reserve as f64,
                                alpha_expected,
                                epsilon = alpha_expected / 10.0
                            );
                            assert_abs_diff_eq!(
                                swap_result.new_tao_reserve as f64,
                                tao_expected,
                                epsilon = tao_expected / 10.0
                            );
                        }

                        // Assert that price movement is in correct direction
                        let sqrt_current_price_after = AlphaSqrtPrice::<Test>::get(netuid);
                        let current_price_after =
                            (sqrt_current_price_after * sqrt_current_price_after).to_num::<f64>();
                        match order_type {
                            OrderType::Buy => assert!(current_price_after > current_price),
                            OrderType::Sell => assert!(current_price_after < current_price),
                        }

                        // Assert that for small amounts price stays within the user position
                        if (order_liquidity_fraction <= 0.001)
                            && (price_low_offset > 0.0001)
                            && (price_high_offset > 0.0001)
                        {
                            assert!(current_price_after <= price_high);
                            assert!(current_price_after >= price_low);
                        }

                        // Check that low and high ticks' fees were updated properly
                        let tick_low_info = Ticks::<Test>::get(netuid, tick_low).unwrap();
                        let tick_high_info = Ticks::<Test>::get(netuid, tick_high).unwrap();
                        let expected_liquidity_net_low = tick_low_info_before.liquidity_net;
                        let expected_liquidity_gross_low = tick_low_info_before.liquidity_gross;
                        let expected_liquidity_net_high = tick_high_info_before.liquidity_net;
                        let expected_liquidity_gross_high = tick_high_info_before.liquidity_gross;
                        assert_eq!(tick_low_info.liquidity_net, expected_liquidity_net_low,);
                        assert_eq!(tick_low_info.liquidity_gross, expected_liquidity_gross_low,);
                        assert_eq!(tick_high_info.liquidity_net, expected_liquidity_net_high,);
                        assert_eq!(
                            tick_high_info.liquidity_gross,
                            expected_liquidity_gross_high,
                        );

                        // Expected fee amount
                        let fee_rate = FeeRate::<Test>::get(netuid) as f64 / u16::MAX as f64;
                        let expected_fee =
                            (order_liquidity - order_liquidity / (1.0 + fee_rate)) as u64;

                        // Global fees should be updated
                        let actual_global_fee = ((match order_type {
                            OrderType::Buy => FeeGlobalTao::<Test>::get(netuid),
                            OrderType::Sell => FeeGlobalAlpha::<Test>::get(netuid),
                        })
                        .to_num::<f64>()
                            * (liquidity_before as f64))
                            as u64;

                        assert_abs_diff_eq!(
                            swap_result.fee_paid,
                            expected_fee,
                            epsilon = expected_fee / 10
                        );
                        assert_abs_diff_eq!(
                            actual_global_fee,
                            expected_fee,
                            epsilon = expected_fee / 10
                        );

                        // Tick fees should be updated

                        // Liquidity position should not be updated
                        let positions =
                            Positions::<Test>::iter_prefix_values((netuid, OK_COLDKEY_ACCOUNT_ID))
                                .collect::<Vec<_>>();
                        let position = positions.first().unwrap();

                        assert_eq!(position.liquidity, position_liquidity,);
                        assert_eq!(position.tick_low, tick_low);
                        assert_eq!(position.tick_high, tick_high);
                        assert_eq!(position.fees_alpha, 0);
                        assert_eq!(position.fees_tao, 0);
                    });
                });
            },
        );
    }

    // This test is a sanity check for swap and multiple positions
    // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_multiple_positions --exact --show-output --nocapture
    #[test]
    fn test_swap_multiple_positions() {
        new_test_ext().execute_with(|| {
            let min_price = tick_to_price(TickIndex::MIN);
            let max_price = tick_to_price(TickIndex::MAX);
            let max_tick = price_to_tick(max_price);
            let netuid = NetUid(1);
            assert_eq!(max_tick, TickIndex::MAX);

            //////////////////////////////////////////////
            // Initialize pool and add the user position
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Add liquidity
            let sqrt_current_price = AlphaSqrtPrice::<Test>::get(netuid);
            let current_price = (sqrt_current_price * sqrt_current_price).to_num::<f64>();

            // Current price is 0.25
            // All positions below are placed at once
            [
                // Very localized position at the current price
                (-0.1, 0.1, 500_000_000_000_u64),
                // Repeat the protocol liquidity at maximum range
                (
                    min_price - current_price,
                    max_price - current_price,
                    2_000_000_000_u64,
                ),
                // Repeat the protocol liquidity at current to max range
                (0.0, max_price - current_price, 2_000_000_000_u64),
                // Repeat the protocol liquidity at min to current range
                (min_price - current_price, 0.0, 2_000_000_000_u64),
                // Half to double price
                (-0.125, 0.25, 2_000_000_000_u64),
                // A few other price ranges and liquidity volumes
                (-0.1, 0.1, 2_000_000_000_u64),
                (-0.1, 0.1, 10_000_000_000_u64),
                (-0.1, 0.1, 100_000_000_000_u64),
                (-0.01, 0.01, 100_000_000_000_u64),
                (-0.001, 0.001, 100_000_000_000_u64),
                // A few (overlapping) positions up the range
                (0.01, 0.02, 100_000_000_000_u64),
                (0.02, 0.03, 100_000_000_000_u64),
                (0.03, 0.04, 100_000_000_000_u64),
                (0.03, 0.05, 100_000_000_000_u64),
                // A few (overlapping) positions down the range
                (-0.02, -0.01, 100_000_000_000_u64),
                (-0.03, -0.02, 100_000_000_000_u64),
                (-0.04, -0.03, 100_000_000_000_u64),
                (-0.05, -0.03, 100_000_000_000_u64),
            ]
            .into_iter()
            .for_each(
                |(price_low_offset, price_high_offset, position_liquidity)| {
                    let price_low = price_low_offset + current_price;
                    let price_high = price_high_offset + current_price;
                    let tick_low = price_to_tick(price_low);
                    let tick_high = price_to_tick(price_high);
                    let (_position_id, _tao, _alpha) = Pallet::<Test>::do_add_liquidity(
                        netuid,
                        &OK_COLDKEY_ACCOUNT_ID,
                        &OK_HOTKEY_ACCOUNT_ID,
                        tick_low,
                        tick_high,
                        position_liquidity,
                    )
                    .unwrap();
                },
            );

            // All these orders are executed without swap reset
            [
                (OrderType::Buy, 100_000_u64, 1000.0_f64),
                (OrderType::Sell, 100_000, 0.0001_f64),
                (OrderType::Buy, 1_000_000, 1000.0_f64),
                (OrderType::Sell, 1_000_000, 0.0001_f64),
                (OrderType::Buy, 10_000_000, 1000.0_f64),
                (OrderType::Sell, 10_000_000, 0.0001_f64),
                (OrderType::Buy, 100_000_000, 1000.0),
                (OrderType::Sell, 100_000_000, 0.0001),
                (OrderType::Buy, 200_000_000, 1000.0_f64),
                (OrderType::Sell, 200_000_000, 0.0001),
                (OrderType::Buy, 500_000_000, 1000.0),
                (OrderType::Sell, 500_000_000, 0.0001),
                (OrderType::Buy, 1_000_000_000, 1000.0),
                (OrderType::Sell, 1_000_000_000, 0.0001),
                (OrderType::Buy, 10_000_000_000, 1000.0),
                (OrderType::Sell, 10_000_000_000, 0.0001),
            ]
            .into_iter()
            .for_each(|(order_type, order_liquidity, limit_price)| {
                //////////////////////////////////////////////
                // Swap
                let sqrt_current_price = AlphaSqrtPrice::<Test>::get(netuid);
                let current_price = (sqrt_current_price * sqrt_current_price).to_num::<f64>();
                let liquidity_before = CurrentLiquidity::<Test>::get(netuid);

                let output_amount = match order_type {
                    OrderType::Buy => {
                        let denom = sqrt_current_price.to_num::<f64>()
                            * (sqrt_current_price.to_num::<f64>() * liquidity_before as f64
                                + order_liquidity as f64);
                        let per_order_liq = liquidity_before as f64 / denom;
                        per_order_liq * order_liquidity as f64
                    }
                    OrderType::Sell => {
                        let denom = liquidity_before as f64 / sqrt_current_price.to_num::<f64>()
                            + order_liquidity as f64;
                        let per_order_liq =
                            sqrt_current_price.to_num::<f64>() * liquidity_before as f64 / denom;
                        per_order_liq * order_liquidity as f64
                    }
                };

                // Do the swap
                let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
                let swap_result = Pallet::<Test>::do_swap(
                    netuid,
                    order_type,
                    order_liquidity,
                    sqrt_limit_price,
                    false,
                )
                .unwrap();
                assert_abs_diff_eq!(
                    swap_result.amount_paid_out as f64,
                    output_amount,
                    epsilon = output_amount / 10.
                );

                let tao_reserve = MockLiquidityProvider::tao_reserve(netuid.into());
                let alpha_reserve = MockLiquidityProvider::alpha_reserve(netuid.into());
                let output_amount = output_amount as u64;

                assert!(output_amount > 0);

                if alpha_reserve > order_liquidity && tao_reserve > order_liquidity {
                    let (tao_expected, alpha_expected) = match order_type {
                        OrderType::Buy => {
                            (tao_reserve + order_liquidity, alpha_reserve - output_amount)
                        }
                        OrderType::Sell => {
                            (tao_reserve - output_amount, alpha_reserve + order_liquidity)
                        }
                    };
                    assert_abs_diff_eq!(
                        swap_result.new_alpha_reserve,
                        alpha_expected,
                        epsilon = alpha_expected / 100
                    );
                    assert_abs_diff_eq!(
                        swap_result.new_tao_reserve,
                        tao_expected,
                        epsilon = tao_expected / 100
                    );
                }

                // Assert that price movement is in correct direction
                let sqrt_current_price_after = AlphaSqrtPrice::<Test>::get(netuid);
                let current_price_after =
                    (sqrt_current_price_after * sqrt_current_price_after).to_num::<f64>();
                match order_type {
                    OrderType::Buy => assert!(current_price_after > current_price),
                    OrderType::Sell => assert!(current_price_after < current_price),
                }
            });

            // Current price shouldn't be much different from the original
            let sqrt_current_price_after = AlphaSqrtPrice::<Test>::get(netuid);
            let current_price_after =
                (sqrt_current_price_after * sqrt_current_price_after).to_num::<f64>();
            assert_abs_diff_eq!(
                current_price,
                current_price_after,
                epsilon = current_price / 10.
            )
        });
    }

    // cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_swap_precision_edge_case --exact --show-output
    #[test]
    fn test_swap_precision_edge_case() {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(123); // 123 is netuid with low edge case liquidity
            let order_type = OrderType::Sell;
            let liquidity = 1_000_000_000_000_000_000;
            let tick_low = TickIndex::MIN;

            let sqrt_limit_price: SqrtPrice = tick_low.try_to_sqrt_price().unwrap();

            // Setup swap
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            // Swap
            let swap_result =
                Pallet::<Test>::do_swap(netuid, order_type, liquidity, sqrt_limit_price, true)
                    .unwrap();

            assert!(swap_result.amount_paid_out > 0);
        });
    }

    // cargo test --package pallet-subtensor-swap --lib -- pallet::impls::tests::test_price_tick_price_roundtrip --exact --show-output
    #[test]
    fn test_price_tick_price_roundtrip() {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(1);

            // Setup swap
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            let current_price = SqrtPrice::from_num(0.500_000_512_192_122_7);
            let tick = TickIndex::try_from_sqrt_price(current_price).unwrap();

            let round_trip_price = TickIndex::try_to_sqrt_price(&tick).unwrap();
            assert!(round_trip_price <= current_price);

            let roundtrip_tick = TickIndex::try_from_sqrt_price(round_trip_price).unwrap();
            assert!(tick == roundtrip_tick);
        });
    }

    #[test]
    fn test_convert_deltas() {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(1);
            assert_ok!(Pallet::<Test>::maybe_initialize_v3(netuid));

            for (sqrt_price, delta_in, expected_buy, expected_sell) in [
                (SqrtPrice::from_num(1.5), 1, 0, 2),
                (SqrtPrice::from_num(1.5), 10000, 4444, 22500),
                (SqrtPrice::from_num(1.5), 1000000, 444444, 2250000),
                (
                    SqrtPrice::from_num(1.5),
                    u64::MAX,
                    2000000000000,
                    3000000000000,
                ),
                (
                    TickIndex::MIN.as_sqrt_price_bounded(),
                    1,
                    18406523739291577836,
                    465,
                ),
                (TickIndex::MIN.as_sqrt_price_bounded(), 10000, u64::MAX, 465),
                (
                    TickIndex::MIN.as_sqrt_price_bounded(),
                    1000000,
                    u64::MAX,
                    465,
                ),
                (
                    TickIndex::MIN.as_sqrt_price_bounded(),
                    u64::MAX,
                    u64::MAX,
                    464,
                ),
                (
                    TickIndex::MAX.as_sqrt_price_bounded(),
                    1,
                    0,
                    18406523745214495085,
                ),
                (TickIndex::MAX.as_sqrt_price_bounded(), 10000, 0, u64::MAX),
                (TickIndex::MAX.as_sqrt_price_bounded(), 1000000, 0, u64::MAX),
                (
                    TickIndex::MAX.as_sqrt_price_bounded(),
                    u64::MAX,
                    2000000000000,
                    u64::MAX,
                ),
            ] {
                {
                    AlphaSqrtPrice::<Test>::insert(netuid, sqrt_price);

                    assert_abs_diff_eq!(
                        Pallet::<Test>::convert_deltas(netuid, OrderType::Sell, delta_in),
                        expected_sell,
                        epsilon = 2
                    );
                    assert_abs_diff_eq!(
                        Pallet::<Test>::convert_deltas(netuid, OrderType::Buy, delta_in),
                        expected_buy,
                        epsilon = 2
                    );
                }
            }
        });
    }

    #[test]
    fn test_user_liquidity_disabled() {
        new_test_ext().execute_with(|| {
            // Use a netuid above 100 since our mock enables liquidity for 0-100
            let netuid = NetUid::from(101);
            let tick_low = TickIndex::new_unchecked(-1000);
            let tick_high = TickIndex::new_unchecked(1000);
            let position_id = 1;
            let liquidity = 1_000_000_000;
            let liquidity_delta = 500_000_000;

            assert!(!EnabledUserLiquidity::<Test>::get(netuid));

            assert_noop!(
                Swap::do_add_liquidity(
                    netuid,
                    &OK_COLDKEY_ACCOUNT_ID,
                    &OK_HOTKEY_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    liquidity
                ),
                Error::<Test>::UserLiquidityDisabled
            );

            assert_noop!(
                Swap::do_remove_liquidity(netuid, &OK_COLDKEY_ACCOUNT_ID, position_id.into()),
                Error::<Test>::UserLiquidityDisabled
            );

            assert_noop!(
                Swap::modify_position(
                    RuntimeOrigin::signed(OK_COLDKEY_ACCOUNT_ID),
                    OK_HOTKEY_ACCOUNT_ID,
                    netuid.into(),
                    position_id,
                    liquidity_delta
                ),
                Error::<Test>::UserLiquidityDisabled
            );

            assert_ok!(Swap::set_enabled_user_liquidity(
                RuntimeOrigin::root(),
                netuid.into()
            ));

            let position_id = Swap::do_add_liquidity(
                netuid,
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                tick_low,
                tick_high,
                liquidity,
            )
            .unwrap()
            .0;

            assert_ok!(Swap::do_modify_position(
                netuid.into(),
                &OK_COLDKEY_ACCOUNT_ID,
                &OK_HOTKEY_ACCOUNT_ID,
                position_id,
                liquidity_delta,
            ));

            assert_ok!(Swap::do_remove_liquidity(
                netuid.into(),
                &OK_COLDKEY_ACCOUNT_ID,
                position_id,
            ));
        });
    }
}
