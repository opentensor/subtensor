use frame_support::{ensure, traits::Get};
use pallet_subtensor_swap_interface::LiquidityDataProvider;
use safe_math::*;
use sp_arithmetic::helpers_128bit;
use sp_runtime::traits::AccountIdConversion;
use substrate_fixed::types::U64F64;

use super::pallet::*;
use crate::{
    NetUid, OrderType, RemoveLiquidityResult, SqrtPrice, SwapResult, SwapStepAction,
    SwapStepResult,
    position::{Position, PositionId},
    tick::{ActiveTickIndexManager, Tick, TickIndex},
};

impl<T: Config> Pallet<T> {
    // initializes V3 swap for a subnet if needed
    fn maybe_initialize_v3(netuid: NetUid) -> Result<(), Error<T>> {
        if SwapV3Initialized::<T>::get(netuid) {
            return Ok(());
        }

        // Initialize the v3:
        // Reserves are re-purposed, nothing to set, just query values for liquidity and price calculation
        let tao_reserve = <T as Config>::LiquidityDataProvider::tao_reserve();
        let alpha_reserve = <T as Config>::LiquidityDataProvider::alpha_reserve();

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
        let protocol_account_id = T::ProtocolId::get().into_account_truncating();

        let _ = Self::add_liquidity(
            netuid,
            &protocol_account_id,
            TickIndex::MIN,
            TickIndex::MAX,
            liquidity,
        )?;

        Ok(())
    }

    /// Perform a swap
    ///
    /// Returns a tuple (amount, refund), where amount is the resulting paid out amount
    pub fn swap(
        netuid: NetUid,
        order_type: OrderType,
        amount: u64,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<SwapResult, Error<T>> {
        Self::maybe_initialize_v3(netuid)?;

        let one = U64F64::from_num(1);

        // Here we store the remaining amount that needs to be exchanged
        // If order_type is Buy, then it expresses Tao amount, if it is Sell,
        // then amount_remaining is Alpha.
        let mut amount_remaining = amount;
        let mut amount_paid_out: u64 = 0;
        let mut refund: u64 = 0;

        // A bit of fool proofing
        let mut iteration_counter: u16 = 0;
        let iter_limit: u16 = 1000;

        // Swap one tick at a time until we reach one of the following conditions:
        //   - Swap all provided amount
        //   - Reach limit price
        //   - Use up all liquidity (up to safe minimum)
        while amount_remaining > 0 {
            let current_price = AlphaSqrtPrice::<T>::get(netuid);
            let current_liquidity = Self::current_liquidity_safe(netuid);
            let sqrt_price_edge = Self::sqrt_price_edge(current_price, order_type);
            let possible_delta_in = amount_remaining
                .saturating_sub(Self::calculate_fee_amount(netuid, amount_remaining));
            let sqrt_price_target = Self::sqrt_price_target(
                order_type,
                current_liquidity,
                current_price,
                possible_delta_in,
            );
            let target_quantity = Self::target_quantity(
                order_type,
                current_liquidity,
                current_price,
                possible_delta_in,
            );
            let edge_quantity = U64F64::from_num(1).safe_div(sqrt_price_edge.into());
            let lim_quantity = one
                .safe_div(T::MinSqrtPrice::get())
                .saturating_add(one.safe_div(sqrt_price_limit.into()));

            let action: SwapStepAction;
            let delta_in;
            let final_price;
            let mut stop_and_refund = false;

            if target_quantity < edge_quantity {
                if target_quantity <= lim_quantity {
                    // stop_in at price target
                    action = SwapStepAction::StopIn;
                    delta_in = possible_delta_in;
                    final_price = sqrt_price_target;
                } else {
                    // stop_in at price limit
                    action = SwapStepAction::StopIn;
                    delta_in = Self::delta_in(
                        order_type,
                        current_liquidity,
                        current_price,
                        sqrt_price_limit,
                    );
                    final_price = sqrt_price_limit;
                    stop_and_refund = true;
                }
            } else if target_quantity > edge_quantity {
                if edge_quantity < lim_quantity {
                    // do crossing at price edge
                    action = SwapStepAction::Crossing;
                    delta_in = Self::delta_in(
                        order_type,
                        current_liquidity,
                        current_price,
                        sqrt_price_edge,
                    );
                    final_price = sqrt_price_edge;
                } else if edge_quantity > lim_quantity {
                    // stop_in at price limit
                    action = SwapStepAction::StopIn;
                    delta_in = Self::delta_in(
                        order_type,
                        current_liquidity,
                        current_price,
                        sqrt_price_limit,
                    );
                    final_price = sqrt_price_limit;
                    stop_and_refund = true;
                } else {
                    // stop_on at price limit
                    action = SwapStepAction::StopOn;
                    delta_in = Self::delta_in(
                        order_type,
                        current_liquidity,
                        current_price,
                        sqrt_price_edge,
                    );
                    final_price = sqrt_price_edge;
                    stop_and_refund = true;
                }
            } else {
                // targetQuantity = edgeQuantity
                if target_quantity <= lim_quantity {
                    // stop_on at price edge
                    delta_in = Self::delta_in(
                        order_type,
                        current_liquidity,
                        current_price,
                        sqrt_price_edge,
                    );
                    final_price = sqrt_price_edge;
                    action = if delta_in > 0 {
                        SwapStepAction::StopOn
                    } else {
                        SwapStepAction::Crossing
                    };
                } else {
                    // targetQuantity > limQuantity
                    // stop_in at price lim
                    action = SwapStepAction::StopIn;
                    delta_in = Self::delta_in(
                        order_type,
                        current_liquidity,
                        current_price,
                        sqrt_price_limit,
                    );
                    final_price = sqrt_price_limit;
                    stop_and_refund = true;
                }
            }

            let swap_result = Self::swap_step(netuid, order_type, delta_in, final_price, action)?;
            amount_remaining = amount_remaining.saturating_sub(swap_result.amount_to_take);
            amount_paid_out = amount_paid_out.saturating_add(swap_result.delta_out);

            if stop_and_refund {
                refund = amount_remaining;
                amount_remaining = 0;
            }

            iteration_counter = iteration_counter.saturating_add(1);
            if iteration_counter > iter_limit {
                return Err(Error::<T>::TooManySwapSteps);
            }
        }

        Ok(SwapResult {
            amount_paid_out,
            refund,
        })
    }

    /// Process a single step of a swap
    fn swap_step(
        netuid: NetUid,
        order_type: OrderType,
        delta_in: u64,
        sqrt_price_final: SqrtPrice,
        action: SwapStepAction,
    ) -> Result<SwapStepResult, Error<T>> {
        // amount_swapped = delta_in / (1 - self.fee_size)
        let fee_rate = U64F64::saturating_from_num(FeeRate::<T>::get(netuid));
        let u16_max = U64F64::saturating_from_num(u16::MAX);
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let amount_swapped =
            delta_fixed.saturating_mul(u16_max.safe_div(u16_max.saturating_sub(fee_rate)));

        // Hold the fees
        let fee = Self::calculate_fee_amount(netuid, amount_swapped.saturating_to_num::<u64>());
        Self::add_fees(netuid, order_type, fee);
        let delta_out = Self::convert_deltas(netuid, order_type, delta_in);

        // TODO (look inside method)
        Self::update_reserves(netuid, order_type, delta_in, delta_out);

        // Get current tick
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);

        match action {
            SwapStepAction::Crossing => {
                let mut tick = match order_type {
                    OrderType::Sell => {
                        Self::find_closest_lower_active_tick(netuid, current_tick_index)
                    }
                    OrderType::Buy => {
                        Self::find_closest_higher_active_tick(netuid, current_tick_index)
                    }
                }
                .ok_or(Error::<T>::InsufficientLiquidity)?;

                tick.fees_out_tao =
                    FeeGlobalTao::<T>::get(netuid).saturating_sub(tick.fees_out_tao);
                tick.fees_out_alpha =
                    FeeGlobalAlpha::<T>::get(netuid).saturating_sub(tick.fees_out_alpha);
                Self::update_liquidity_at_crossing(netuid, order_type)?;
                Ticks::<T>::insert(netuid, current_tick_index, tick);
            }

            SwapStepAction::StopOn => match order_type {
                OrderType::Buy => {
                    Self::update_liquidity_at_crossing(netuid, order_type)?;
                    let Some(mut tick) =
                        Self::find_closest_higher_active_tick(netuid, current_tick_index)
                    else {
                        return Err(Error::<T>::InsufficientLiquidity);
                    };

                    tick.fees_out_tao =
                        FeeGlobalTao::<T>::get(netuid).saturating_sub(tick.fees_out_tao);
                    tick.fees_out_alpha =
                        FeeGlobalAlpha::<T>::get(netuid).saturating_sub(tick.fees_out_alpha);
                    Ticks::<T>::insert(netuid, current_tick_index, tick);
                }
                OrderType::Sell => {}
            },

            SwapStepAction::StopIn => {}
        }

        // Update current price, which effectively updates current tick too
        AlphaSqrtPrice::<T>::set(netuid, sqrt_price_final);

        Ok(SwapStepResult {
            amount_to_take: amount_swapped.saturating_to_num::<u64>(),
            delta_out,
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
        let liquidity_curr = SqrtPrice::saturating_from_num(CurrentLiquidity::<T>::get(netuid));
        let sqrt_price_curr = AlphaSqrtPrice::<T>::get(netuid);
        let delta_fixed = SqrtPrice::saturating_from_num(delta_in);

        // TODO: Implement in safe and non-overflowing math
        // Intentionally using unsafe math here to trigger CI

        // // Prevent overflows:
        // // If liquidity or delta are too large, reduce their precision and
        // // save their factor for final correction. Price can take full U64F64
        // // range, and it will not overflow u128 divisions or multiplications.
        // let mut liquidity_factor: u64 = 1;
        // if liquidity_curr > u32::MAX as u64 {
        //     liquidity_factor = u32::MAX as u64;
        //     liquidity_curr = liquidity_curr.safe_div(liquidity_factor);
        // }
        // let mut delta = delta_in as u64;
        // let mut delta_factor: u64 = 1;
        // if delta > u32::MAX as u64 {
        //     delta_factor = u32::MAX as u64;
        //     delta = delta.safe_div(delta_factor);
        // }

        // // This product does not overflow because we limit both
        // // multipliers by u32::MAX (despite the u64 type)
        // let delta_liquidity = delta.saturating_mul(liquidity);

        // // This is product of delta_in * liquidity_curr * sqrt_price_curr
        // let delta_liquidity_price: u128 =
        //     Self::mul_u64_u64f64(delta_liquidity, sqrt_price_curr.into());

        if delta_in > 0 {
            (match order_type {
                OrderType::Sell => {
                    liquidity_curr * sqrt_price_curr * delta_fixed
                        / (liquidity_curr / sqrt_price_curr + delta_fixed)
                }
                OrderType::Buy => {
                    liquidity_curr / sqrt_price_curr * delta_fixed
                        / (liquidity_curr * sqrt_price_curr + delta_fixed)
                }
            })
            .to_num::<u64>()
        } else {
            0
        }
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

    /// Update token reserves after a swap
    fn update_reserves(netuid: NetUid, order_type: OrderType, amount_in: u64, amount_out: u64) {
        // TODO can we return the values from Self::swap, so the depender updated their state
        // (instead of us mutating it)
        todo!("update_reserves needs consideration")
        // let (new_tao_reserve, new_alpha_reserve) = match order_type {
        //     OrderType::Sell => (
        //         self.state_ops.get_tao_reserve().saturating_add(amount_in),
        //         self.state_ops
        //             .get_alpha_reserve()
        //             .saturating_sub(amount_out),
        //     ),
        //     OrderType::Buy => (
        //         self.state_ops.get_tao_reserve().saturating_sub(amount_in),
        //         self.state_ops
        //             .get_alpha_reserve()
        //             .saturating_add(amount_out),
        //     ),
        // };

        // self.state_ops.set_tao_reserve(new_tao_reserve);
        // self.state_ops.set_alpha_reserve(new_alpha_reserve);
    }

    /// Update liquidity when crossing a tick
    fn update_liquidity_at_crossing(netuid: NetUid, order_type: OrderType) -> Result<(), Error<T>> {
        let mut liquidity_curr = CurrentLiquidity::<T>::get(netuid);
        let current_tick_index = TickIndex::current_bounded::<T>(netuid);
        match order_type {
            OrderType::Sell => {
                let Some(tick) = Self::find_closest_lower_active_tick(netuid, current_tick_index)
                else {
                    return Err(Error::<T>::InsufficientLiquidity);
                };

                let liquidity_update_abs_u64 = tick.liquidity_net_as_u64();

                liquidity_curr = if tick.liquidity_net >= 0 {
                    liquidity_curr.saturating_sub(liquidity_update_abs_u64)
                } else {
                    liquidity_curr.saturating_add(liquidity_update_abs_u64)
                };
            }
            OrderType::Buy => {
                let Some(tick) = Self::find_closest_higher_active_tick(netuid, current_tick_index)
                else {
                    return Err(Error::<T>::InsufficientLiquidity);
                };
                let liquidity_update_abs_u64 = tick.liquidity_net_as_u64();

                liquidity_curr = if tick.liquidity_net >= 0 {
                    liquidity_curr.saturating_add(liquidity_update_abs_u64)
                } else {
                    liquidity_curr.saturating_sub(liquidity_update_abs_u64)
                };
            }
        }

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
    ) -> Result<(u64, u64), Error<T>> {
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
        let position = Position {
            id: PositionId::new(),
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

        Positions::<T>::insert(&(netuid, account_id, position.id), position);

        SwapV3Initialized::<T>::set(netuid, true);

        Ok((tao, alpha))
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
}
