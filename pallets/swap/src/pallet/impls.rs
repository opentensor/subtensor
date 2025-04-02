use core::marker::PhantomData;

use frame_support::{ensure, traits::Get};
use pallet_subtensor_swap_interface::LiquidityDataProvider;
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use sp_runtime::traits::AccountIdConversion;
use substrate_fixed::types::U64F64;

use super::pallet::*;
use crate::{
    NetUid, OrderType, RemoveLiquidityResult, SqrtPrice,
    position::{Position, PositionId},
    tick::{ActiveTickIndexManager, Tick, TickIndex},
};

const MAX_SWAP_ITERATIONS: u16 = 1000;

/// A struct representing a single swap step with all its parameters and state
struct SwapStep<T: Config> {
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
    target_quantity: SqrtPrice,
    edge_quantity: U64F64,
    lim_quantity: U64F64,

    // Result values
    action: SwapStepAction,
    delta_in: u64,
    final_price: SqrtPrice,
    stop_and_refund: bool,

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
        let one = U64F64::from_num(1);
        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let current_liquidity = Pallet::<T>::current_liquidity_safe(netuid);
        let sqrt_price_edge = Pallet::<T>::sqrt_price_edge(current_price, order_type);

        let possible_delta_in = amount_remaining
            .saturating_sub(Pallet::<T>::calculate_fee_amount(netuid, amount_remaining));

        // Target price and quantities
        let sqrt_price_target = Pallet::<T>::sqrt_price_target(
            order_type,
            current_liquidity,
            current_price,
            possible_delta_in,
        );
        let target_quantity = Pallet::<T>::target_quantity(
            order_type,
            current_liquidity,
            current_price,
            possible_delta_in,
        );

        // Quantities for comparison
        let edge_quantity = one.safe_div(sqrt_price_edge.into());
        let lim_quantity = one
            .safe_div(T::MinSqrtPrice::get())
            .saturating_add(one.safe_div(sqrt_price_limit.into()));

        Self {
            netuid,
            order_type,
            sqrt_price_limit,
            current_price,
            current_liquidity,
            sqrt_price_edge,
            possible_delta_in,
            sqrt_price_target,
            target_quantity,
            edge_quantity,
            lim_quantity,
            action: SwapStepAction::StopIn,
            delta_in: 0,
            final_price: sqrt_price_target,
            stop_and_refund: false,
            _phantom: PhantomData,
        }
    }

    /// Execute the swap step and return the result
    fn execute(&mut self) -> Result<SwapStepResult, Error<T>> {
        self.determine_action();
        self.process_swap()
    }

    /// Determine the appropriate action for this swap step
    fn determine_action(&mut self) {
        // Case 1: target_quantity < edge_quantity
        if self.target_quantity < self.edge_quantity {
            if self.target_quantity <= self.lim_quantity {
                // Stop at target price (no refund needed)
                self.action = SwapStepAction::StopIn;
                self.delta_in = self.possible_delta_in;
                self.final_price = self.sqrt_price_target;
                self.stop_and_refund = false;
            } else {
                // Stop at limit price (refund needed)
                self.action = SwapStepAction::StopIn;
                self.delta_in = Pallet::<T>::delta_in(
                    self.order_type,
                    self.current_liquidity,
                    self.current_price,
                    self.sqrt_price_limit,
                );
                self.final_price = self.sqrt_price_limit;
                self.stop_and_refund = true;
            }
        }
        // Case 2: target_quantity > edge_quantity
        else if self.target_quantity > self.edge_quantity {
            if self.edge_quantity < self.lim_quantity {
                // Cross at edge price
                self.action = SwapStepAction::Crossing;
                self.delta_in = Pallet::<T>::delta_in(
                    self.order_type,
                    self.current_liquidity,
                    self.current_price,
                    self.sqrt_price_edge,
                );
                self.final_price = self.sqrt_price_edge;
                self.stop_and_refund = false;
            } else if self.edge_quantity > self.lim_quantity {
                // Stop at limit price (refund needed)
                self.action = SwapStepAction::StopIn;
                self.delta_in = Pallet::<T>::delta_in(
                    self.order_type,
                    self.current_liquidity,
                    self.current_price,
                    self.sqrt_price_limit,
                );
                self.final_price = self.sqrt_price_limit;
                self.stop_and_refund = true;
            } else {
                // Stop on edge (refund needed)
                self.action = SwapStepAction::StopOn;
                self.delta_in = Pallet::<T>::delta_in(
                    self.order_type,
                    self.current_liquidity,
                    self.current_price,
                    self.sqrt_price_edge,
                );
                self.final_price = self.sqrt_price_edge;
                self.stop_and_refund = true;
            }
        }
        // Case 3: target_quantity = edge_quantity
        else {
            if self.target_quantity <= self.lim_quantity {
                // Stop on edge price
                self.delta_in = Pallet::<T>::delta_in(
                    self.order_type,
                    self.current_liquidity,
                    self.current_price,
                    self.sqrt_price_edge,
                );
                self.final_price = self.sqrt_price_edge;
                self.action = if self.delta_in > 0 {
                    SwapStepAction::StopOn
                } else {
                    SwapStepAction::Crossing
                };
                self.stop_and_refund = false;
            } else {
                // Stop at limit price (refund needed)
                self.action = SwapStepAction::StopIn;
                self.delta_in = Pallet::<T>::delta_in(
                    self.order_type,
                    self.current_liquidity,
                    self.current_price,
                    self.sqrt_price_limit,
                );
                self.final_price = self.sqrt_price_limit;
                self.stop_and_refund = true;
            }
        }
    }

    /// Process a single step of a swap
    fn process_swap(&self) -> Result<SwapStepResult, Error<T>> {
        // amount_swapped = delta_in / (1 - self.fee_size)
        let fee_rate = U64F64::saturating_from_num(FeeRate::<T>::get(self.netuid));
        let u16_max = U64F64::saturating_from_num(u16::MAX);
        let delta_fixed = U64F64::saturating_from_num(self.delta_in);
        let amount_swapped =
            delta_fixed.saturating_mul(u16_max.safe_div(u16_max.saturating_sub(fee_rate)));

        // Hold the fees
        let fee = Pallet::<T>::calculate_fee_amount(
            self.netuid,
            amount_swapped.saturating_to_num::<u64>(),
        );
        Pallet::<T>::add_fees(self.netuid, self.order_type, fee);
        let delta_out = Pallet::<T>::convert_deltas(self.netuid, self.order_type, self.delta_in);

        // TODO (look inside method)
        // Self::update_reserves(netuid, order_type, delta_in, delta_out);

        // Get current tick
        let current_tick_index = TickIndex::current_bounded::<T>(self.netuid);

        match self.action {
            SwapStepAction::Crossing => {
                let mut tick = match self.order_type {
                    OrderType::Sell => {
                        Pallet::<T>::find_closest_lower_active_tick(self.netuid, current_tick_index)
                    }
                    OrderType::Buy => Pallet::<T>::find_closest_higher_active_tick(
                        self.netuid,
                        current_tick_index,
                    ),
                }
                .ok_or(Error::<T>::InsufficientLiquidity)?;

                tick.fees_out_tao =
                    FeeGlobalTao::<T>::get(self.netuid).saturating_sub(tick.fees_out_tao);
                tick.fees_out_alpha =
                    FeeGlobalAlpha::<T>::get(self.netuid).saturating_sub(tick.fees_out_alpha);
                Pallet::<T>::update_liquidity_at_crossing(self.netuid, self.order_type)?;
                Ticks::<T>::insert(self.netuid, current_tick_index, tick);
            }

            SwapStepAction::StopOn => match self.order_type {
                OrderType::Buy => {
                    Pallet::<T>::update_liquidity_at_crossing(self.netuid, self.order_type)?;
                    let Some(mut tick) = Pallet::<T>::find_closest_higher_active_tick(
                        self.netuid,
                        current_tick_index,
                    ) else {
                        return Err(Error::<T>::InsufficientLiquidity);
                    };

                    tick.fees_out_tao =
                        FeeGlobalTao::<T>::get(self.netuid).saturating_sub(tick.fees_out_tao);
                    tick.fees_out_alpha =
                        FeeGlobalAlpha::<T>::get(self.netuid).saturating_sub(tick.fees_out_alpha);
                    Ticks::<T>::insert(self.netuid, current_tick_index, tick);
                }
                OrderType::Sell => {}
            },

            SwapStepAction::StopIn => {}
        }

        // Update current price, which effectively updates current tick too
        AlphaSqrtPrice::<T>::set(self.netuid, self.final_price);

        Ok(SwapStepResult {
            amount_to_take: amount_swapped.saturating_to_num::<u64>(),
            delta_out,
        })
    }
}

impl<T: Config> Pallet<T> {
    // initializes V3 swap for a subnet if needed
    fn maybe_initialize_v3(netuid: NetUid) -> Result<(), Error<T>> {
        if SwapV3Initialized::<T>::get(netuid) {
            return Ok(());
        }

        // Initialize the v3:
        // Reserves are re-purposed, nothing to set, just query values for liquidity and price calculation
        let tao_reserve = <T as Config>::LiquidityDataProvider::tao_reserve(netuid.into());
        let alpha_reserve = <T as Config>::LiquidityDataProvider::alpha_reserve(netuid.into());

        // Set price
        let price = U64F64::saturating_from_num(tao_reserve)
            .safe_div(U64F64::saturating_from_num(alpha_reserve));

        let epsilon = U64F64::saturating_from_num(0.000001);

        AlphaSqrtPrice::<T>::set(
            netuid,
            price.checked_sqrt(epsilon).unwrap_or(U64F64::from_num(0)),
        );

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
    /// Returns a tuple (amount_paid_out, refund), where amount_paid_out is the resulting paid out amount
    /// and refund is any unswapped amount returned to the caller
    pub fn swap(
        netuid: NetUid,
        order_type: OrderType,
        amount: u64,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<SwapResult, Error<T>> {
        Self::maybe_initialize_v3(netuid)?;

        let mut amount_remaining = amount;
        let mut amount_paid_out: u64 = 0;
        let mut refund: u64 = 0;
        let mut iteration_counter: u16 = 0;

        // Swap one tick at a time until we reach one of the stop conditions
        while amount_remaining > 0 {
            // Create and execute a swap step
            let mut swap_step =
                SwapStep::<T>::new(netuid, order_type, amount_remaining, sqrt_price_limit);

            let swap_result = swap_step.execute()?;

            amount_remaining = amount_remaining.saturating_sub(swap_result.amount_to_take);
            amount_paid_out = amount_paid_out.saturating_add(swap_result.delta_out);

            if swap_step.stop_and_refund {
                refund = amount_remaining;
                amount_remaining = 0;
            }

            iteration_counter = iteration_counter.saturating_add(1);
            if iteration_counter > MAX_SWAP_ITERATIONS {
                return Err(Error::<T>::TooManySwapSteps);
            }
        }

        Ok(SwapResult {
            amount_paid_out,
            refund,
        })
    }

    /// Get the square root price at the current tick edge for the given direction (order type) If
    /// order type is Buy, then price edge is the high tick bound price, otherwise it is the low
    /// tick bound price.
    ///
    /// If anything is wrong with tick math and it returns Err, we just abort the deal, i.e. return
    /// the edge that is impossible to execute
    fn sqrt_price_edge(current_price: U64F64, order_type: OrderType) -> SqrtPrice {
        let fallback_price_edge_value = (match order_type {
            OrderType::Buy => TickIndex::MIN.try_to_sqrt_price(),
            OrderType::Sell => TickIndex::MAX.try_to_sqrt_price(),
        })
        .unwrap_or(SqrtPrice::from_num(0));

        TickIndex::try_from_sqrt_price(current_price)
            .and_then(|current_tick_index| {
                match order_type {
                    OrderType::Buy => {
                        TickIndex::new_unchecked(current_tick_index.get().saturating_add(1))
                    }
                    OrderType::Sell => current_tick_index,
                }
                .try_to_sqrt_price()
            })
            .unwrap_or(fallback_price_edge_value)
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
                FeeGlobalTao::<T>::set(
                    netuid,
                    fee_global_tao.saturating_add(fee_fixed.safe_div(liquidity_curr)),
                );
            }
            OrderType::Buy => {
                FeeGlobalAlpha::<T>::set(
                    netuid,
                    fee_global_alpha.saturating_add(fee_fixed.safe_div(liquidity_curr)),
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
                liquidity_curr * sqrt_price_curr * delta_fixed
                    / (liquidity_curr / sqrt_price_curr + delta_fixed)
            }
            OrderType::Buy => {
                liquidity_curr / sqrt_price_curr * delta_fixed
                    / (liquidity_curr * sqrt_price_curr + delta_fixed)
            }
        };

        result.to_num::<u64>()
    }

    /// Get the target square root price based on the input amount
    fn sqrt_price_target(
        order_type: OrderType,
        liquidity_curr: U64F64,
        sqrt_price_curr: U64F64,
        delta_in: u64,
    ) -> SqrtPrice {
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let one = U64F64::saturating_from_num(1);

        if liquidity_curr == 0 {
            // No liquidity means price should remain current
            return sqrt_price_curr;
        }

        match order_type {
            OrderType::Buy => one.safe_div(
                delta_fixed
                    .safe_div(liquidity_curr)
                    .saturating_add(one.safe_div(sqrt_price_curr)),
            ),
            OrderType::Sell => delta_fixed
                .safe_div(liquidity_curr)
                .saturating_add(sqrt_price_curr),
        }
    }

    /// Get the target quantity, which is
    ///     `1 / (target square root price)` in case of sell order
    ///     `target square root price` in case of buy order
    ///
    /// ...based on the input amount, current liquidity, and current alpha price
    fn target_quantity(
        order_type: OrderType,
        liquidity_curr: U64F64,
        sqrt_price_curr: U64F64,
        delta_in: u64,
    ) -> SqrtPrice {
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let one = U64F64::saturating_from_num(1);

        if liquidity_curr == 0 {
            // No liquidity means zero
            return SqrtPrice::saturating_from_num(0);
        }

        match order_type {
            OrderType::Buy => delta_fixed
                .safe_div(liquidity_curr)
                .saturating_add(sqrt_price_curr)
                .into(),
            OrderType::Sell => delta_fixed
                .safe_div(liquidity_curr)
                .saturating_add(one.safe_div(sqrt_price_curr))
                .into(),
        }
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
    /// - `account_id`: A reference to the account that is providing liquidity.
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
    pub fn add_liquidity(
        netuid: NetUid,
        account_id: &T::AccountId,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Result<(PositionId, u64, u64), Error<T>> {
        let (position, tao, alpha) =
            Self::add_liquidity_not_insert(netuid, account_id, tick_low, tick_high, liquidity)?;
        let position_id = position.id;

        ensure!(
            T::LiquidityDataProvider::tao_balance(netuid.into(), account_id) >= tao
                && T::LiquidityDataProvider::alpha_balance(netuid.into(), account_id) >= alpha,
            Error::<T>::InsufficientBalance
        );

        Positions::<T>::insert(&(netuid, account_id, position.id), position);

        Ok((position_id, tao, alpha))
    }

    // add liquidity without inserting position into storage (used privately for v3 intiialization).
    // unlike Self::add_liquidity it also doesn't perform account's balance check.
    //
    // the public interface is [`Self::add_liquidity`]
    fn add_liquidity_not_insert(
        netuid: NetUid,
        account_id: &T::AccountId,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Result<(Position, u64, u64), Error<T>> {
        ensure!(
            Self::count_positions(netuid, account_id) <= T::MaxPositions::get() as usize,
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
        let position_id = PositionId::new();
        let position = Position {
            id: position_id,
            netuid,
            tick_low,
            tick_high,
            liquidity,
            fees_tao: 0,
            fees_alpha: 0,
        };

        let current_price = AlphaSqrtPrice::<T>::get(netuid);
        let (tao, alpha) = position.to_token_amounts(current_price)?;

        // If this is a user transaction, withdraw balances and update reserves
        // TODO this should be returned (tao, alpha) from this function to prevent
        // mutation of outside storage - the logic should be passed to the user of
        // pallet_subtensor_swap_interface
        // if !protocol {
        //     let current_price = self.state_ops.get_alpha_sqrt_price();
        //     let (tao, alpha) = position.to_token_amounts(current_price)?;
        //     self.state_ops.withdraw_balances(account_id, tao, alpha)?;

        //     // Update reserves
        //     let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_add(tao);
        //     self.state_ops.set_tao_reserve(new_tao_reserve);
        //     let new_alpha_reserve = self.state_ops.get_alpha_reserve().saturating_add(alpha);
        //     self.state_ops.set_alpha_reserve(new_alpha_reserve);
        // }

        SwapV3Initialized::<T>::set(netuid, true);

        Ok((position, tao, alpha))
    }

    /// Remove liquidity and credit balances back to account_id
    ///
    /// Account ID and Position ID identify position in the storage map
    pub fn remove_liquidity(
        netuid: NetUid,
        account_id: &T::AccountId,
        position_id: PositionId,
    ) -> Result<RemoveLiquidityResult, Error<T>> {
        let Some(mut position) = Positions::<T>::get((netuid, account_id, position_id)) else {
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
            -(position.liquidity as i128),
        );

        // Remove user position
        Positions::<T>::remove((netuid, account_id, position_id));

        {
            // TODO we move this logic to the outside depender to prevent mutating its state
            //     // Deposit balances
            //     self.state_ops.deposit_balances(account_id, tao, alpha);

            //     // Update reserves
            //     let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_sub(tao);
            //     self.state_ops.set_tao_reserve(new_tao_reserve);
            //     let new_alpha_reserve = self.state_ops.get_alpha_reserve().saturating_sub(alpha);
            //     self.state_ops.set_alpha_reserve(new_alpha_reserve);
        }

        // TODO: Clear with R&D
        // Update current price (why?)
        // AlphaSqrtPrice::<T>::set(netuid, sqrt_price);

        Ok(RemoveLiquidityResult {
            tao,
            alpha,
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
            -(liquidity as i128)
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
            -(liquidity as i128)
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
            let corrected_price = tick_index.to_sqrt_price_bounded();
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

#[derive(Debug, PartialEq)]
pub struct SwapResult {
    amount_paid_out: u64,
    refund: u64,
}

#[derive(Debug, PartialEq)]
struct SwapStepResult {
    amount_to_take: u64,
    delta_out: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SwapStepAction {
    Crossing,
    StopOn,
    StopIn,
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use frame_support::{assert_err, assert_ok};
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

            let sqrt_price = AlphaSqrtPrice::<Test>::get(netuid);
            let expected_sqrt_price = U64F64::from_num(tao)
                .safe_div(U64F64::from_num(alpha))
                .checked_sqrt(U64F64::from_num(0.000001))
                .unwrap();
            assert_eq!(sqrt_price, expected_sqrt_price);

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
                (0.25, max_price, 2_000_000_000_u64, 0, 4_000_000_000),
                // Repeat the protocol liquidity at min to current range: Expect all the same tao
                (min_price, 0.24999, 2_000_000_000_u64, 1_000_000_000, 0),
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
                    let (position_id, tao, alpha) = Pallet::<Test>::add_liquidity(
                        netuid,
                        &OK_ACCOUNT_ID,
                        tick_low,
                        tick_high,
                        liquidity,
                    )
                    .unwrap();

                    // dbg!((tao, expected_tao), (alpha, expected_alpha));

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
                    assert_eq!(Pallet::<Test>::count_positions(netuid, &OK_ACCOUNT_ID), 1);

                    let position =
                        Positions::<Test>::get(&(netuid, OK_ACCOUNT_ID, position_id)).unwrap();
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
                    Swap::add_liquidity(netuid, &OK_ACCOUNT_ID, tick_low, tick_high, liquidity),
                    Error::<Test>::InvalidTickRange,
                );
            });
        });
    }

    #[test]
    fn test_add_liquidity_over_balance() {
        new_test_ext().execute_with(|| {
            let account_id = 2;

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
                    Pallet::<Test>::add_liquidity(
                        netuid,
                        &account_id,
                        tick_low,
                        tick_high,
                        liquidity
                    ),
                    Error::<Test>::InsufficientBalance,
                );
            });
        });
    }

    #[test]
    fn test_remove_liquidity_basic() {
        new_test_ext().execute_with(|| {
            let min_price = tick_to_price(TickIndex::MIN);
            let max_price = tick_to_price(TickIndex::MAX);
            let max_tick = price_to_tick(max_price);
            assert_eq!(max_tick, TickIndex::MAX);

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
                (0.25, max_price, 2_000_000_000_u64, 0, 4_000_000_000),
                // Repeat the protocol liquidity at min to current range: Expect all the same tao
                (min_price, 0.24999, 2_000_000_000_u64, 1_000_000_000, 0),
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
                let (position_id, _, _) = Pallet::<Test>::add_liquidity(
                    netuid,
                    &OK_ACCOUNT_ID,
                    tick_low,
                    tick_high,
                    liquidity,
                )
                .unwrap();

                // Remove liquidity
                let remove_result =
                    Pallet::<Test>::remove_liquidity(netuid, &OK_ACCOUNT_ID, position_id).unwrap();
                assert_abs_diff_eq!(remove_result.tao, tao, epsilon = tao / 1000);
                assert_abs_diff_eq!(remove_result.alpha, alpha, epsilon = alpha / 1000);
                assert_eq!(remove_result.fee_tao, 0);
                assert_eq!(remove_result.fee_alpha, 0);

                // Liquidity position is removed
                assert_eq!(Pallet::<Test>::count_positions(netuid, &OK_ACCOUNT_ID), 0);
                assert!(Positions::<Test>::get((netuid, OK_ACCOUNT_ID, position_id)).is_none());

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
            assert_ok!(Pallet::<Test>::add_liquidity(
                netuid,
                &OK_ACCOUNT_ID,
                tick_low,
                tick_high,
                liquidity,
            ));

            assert!(Pallet::<Test>::count_positions(netuid, &OK_ACCOUNT_ID) > 0);

            // Remove liquidity
            assert_err!(
                Pallet::<Test>::remove_liquidity(netuid, &OK_ACCOUNT_ID, PositionId::new()),
                Error::<Test>::LiquidityNotFound,
            );
        });
    }

    #[test]
    fn test_swap_basic() {
        new_test_ext().execute_with(|| {
            // Current price is 0.25
            // Test case is (order_type, liquidity, limit_price, output_amount)
            [(OrderType::Buy, 500_000_000u64, 1000.0_f64, 3990_u64)]
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

                        // Swap
                        let sqrt_limit_price = SqrtPrice::from_num((limit_price).sqrt());
                        let swap_result =
                            Pallet::<Test>::swap(netuid, order_type, liquidity, sqrt_limit_price);
                        // assert_abs_diff_eq!(
                        //     swap_result.unwrap().amount_paid_out,
                        //     *output_amount,
                        //     epsilon = *output_amount/1000
                        // );

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
                        let expected_fee = output_amount * fee_rate as u64;

                        // Global fees should be updated
                        let actual_global_fee = match order_type {
                            OrderType::Buy => FeeGlobalAlpha::<Test>::get(netuid),
                            OrderType::Sell => FeeGlobalTao::<Test>::get(netuid),
                        };
                        println!("actual_global_fee {:?}", actual_global_fee);
                        assert_eq!(actual_global_fee, expected_fee);

                        // Tick fees should be updated

                        // Liquidity position should not be updated
                        let protocol_id = Pallet::<Test>::protocol_account_id();
                        let positions =
                            Positions::<Test>::iter_prefix_values((netuid, protocol_id))
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

                        // // Current liquidity is updated only when price range includes the current price
                        // if (*price_high >= current_price) && (*price_low <= current_price) {
                        //     assert_eq!(
                        //         swap.state_ops.get_current_liquidity(),
                        //         liquidity_before + *liquidity
                        //     );
                        // } else {
                        //     assert_eq!(swap.state_ops.get_current_liquidity(), liquidity_before);
                        // }

                        // // Reserves are updated
                        // assert_eq!(
                        //     swap.state_ops.get_tao_reserve(),
                        //     tao_withdrawn + protocol_tao,
                        // );
                        // assert_eq!(
                        //     swap.state_ops.get_alpha_reserve(),
                        //     alpha_withdrawn + protocol_alpha,
                        // );
                    },
                );
        });
    }
}
