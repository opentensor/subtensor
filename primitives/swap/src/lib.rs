use core::marker::PhantomData;
use std::ops::Neg;

use safe_math::*;
use substrate_fixed::types::U64F64;

use self::error::SwapError;
use self::tick::{
    Tick, find_closest_higher_active_tick_index, find_closest_lower_active_tick_index,
    sqrt_price_to_tick_index, tick_index_to_sqrt_price,
};
use self::tick_math::{MAX_TICK, MIN_TICK};

mod error;
mod tick;
mod tick_math;

type SqrtPrice = U64F64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
}

pub enum SwapStepAction {
    Crossing,
    StopOn,
    StopIn,
}

pub struct RemoveLiquidityResult {
    tao: u64,
    alpha: u64,
    fee_tao: u64,
    fee_alpha: u64,
}

pub struct SwapResult {
    amount_paid_out: u64,
    refund: u64,
}

struct SwapStepResult {
    amount_to_take: u64,
    delta_out: u64,
}

/// Position designates one liquidity position.
///
/// Alpha price is expressed in rao units per one 10^9 unit. For example,
/// price 1_000_000 is equal to 0.001 TAO per Alpha.
///
/// tick_low - tick index for lower boundary of price
/// tick_high - tick index for higher boundary of price
/// liquidity - position liquidity
/// fees_tao - fees accrued by the position in quote currency (TAO)
/// fees_alpha - fees accrued by the position in base currency (Alpha)
///
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Position {
    tick_low: i32,
    tick_high: i32,
    liquidity: u64,
    fees_tao: u64,
    fees_alpha: u64,
}

impl Position {
    /// Converts position to token amounts
    ///
    /// returns tuple of (TAO, Alpha)
    ///
    pub fn to_token_amounts(&self, current_tick: i32) -> Result<(u64, u64), SwapError> {
        let one: U64F64 = U64F64::saturating_from_num(1);

        let sqrt_price_curr: SqrtPrice =
            tick_index_to_sqrt_price(current_tick).map_err(|_| SwapError::InvalidTickRange)?;
        let sqrt_pa: SqrtPrice =
            tick_index_to_sqrt_price(self.tick_low).map_err(|_| SwapError::InvalidTickRange)?;
        let sqrt_pb: SqrtPrice =
            tick_index_to_sqrt_price(self.tick_high).map_err(|_| SwapError::InvalidTickRange)?;
        let liquidity_fixed: U64F64 = U64F64::saturating_from_num(self.liquidity);

        Ok(if sqrt_price_curr < sqrt_pa {
            (
                liquidity_fixed
                    .saturating_mul(one.safe_div(sqrt_pa).saturating_sub(one.safe_div(sqrt_pb)))
                    .saturating_to_num::<u64>(),
                0,
            )
        } else if sqrt_price_curr > sqrt_pb {
            (
                0,
                liquidity_fixed
                    .saturating_mul(sqrt_pb.saturating_sub(sqrt_pa))
                    .saturating_to_num::<u64>(),
            )
        } else {
            (
                liquidity_fixed
                    .saturating_mul(
                        one.safe_div(sqrt_price_curr)
                            .saturating_sub(one.safe_div(sqrt_pb)),
                    )
                    .saturating_to_num::<u64>(),
                liquidity_fixed
                    .saturating_mul(sqrt_price_curr.saturating_sub(sqrt_pa))
                    .saturating_to_num::<u64>(),
            )
        })
    }
}

/// This trait implementation depends on Runtime and it needs to be implemented
/// in the pallet to be able to work with chain state and per subnet. All subnet
/// swaps are independent and hence netuid is abstracted away from swap implementation.
///
pub trait SwapDataOperations<AccountIdType> {
    /// Tells if v3 swap is initialized in the state. v2 only provides base and quote
    /// reserves, while v3 also stores ticks and positions, which need to be initialized
    /// at the first pool creation.
    fn is_v3_initialized(&self) -> bool;
    /// Returns u16::MAX normalized fee rate. For example, 0.3% is approximately 196.
    fn get_fee_rate(&self) -> u16;
    /// Minimum liquidity that is safe for rounding and integer math.
    fn get_minimum_liquidity(&self) -> u64;
    fn get_tick_by_index(&self, tick_index: i32) -> Option<Tick>;
    fn insert_tick_by_index(&self, tick_index: i32, tick: Tick);
    fn remove_tick_by_index(&self, tick_index: i32);
    /// Minimum sqrt price across all active ticks
    fn get_min_sqrt_price(&self) -> SqrtPrice;
    /// Maximum sqrt price across all active ticks
    fn get_max_sqrt_price(&self) -> SqrtPrice;
    fn get_tao_reserve(&self) -> u64;
    fn set_tao_reserve(&self, tao: u64) -> u64;
    fn get_alpha_reserve(&self) -> u64;
    fn set_alpha_reserve(&self, alpha: u64) -> u64;
    fn get_alpha_sqrt_price(&self) -> SqrtPrice;
    fn set_alpha_sqrt_price(&self, sqrt_price: SqrtPrice) -> u64;

    // Getters/setters for global accrued fees in alpha and tao per subnet
    fn get_fee_global_tao(&self) -> U64F64;
    fn set_fee_global_tao(&self, fee: U64F64);
    fn get_fee_global_alpha(&self) -> U64F64;
    fn set_fee_global_alpha(&self, fee: U64F64);

    /// Get current tick liquidity
    fn get_current_liquidity(&self) -> u64;
    /// Set current tick liquidity
    fn set_current_liquidity(&self, liquidity: u64);

    // User account operations
    fn get_max_positions(&self) -> u16;
    fn withdraw_balances(&self, account_id: &AccountIdType, tao: u64, alpha: u64) -> (u64, u64);
    fn deposit_balances(&self, account_id: &AccountIdType, tao: u64, alpha: u64);
    fn get_position_count(&self, account_id: &AccountIdType) -> u16;
    fn get_position(&self, account_id: &AccountIdType, position_id: u16) -> Option<Position>;
    fn create_position(&self, account_id: &AccountIdType, positions: Position);
    fn update_position(&self, account_id: &AccountIdType, position_id: u16, positions: Position);
    fn remove_position(&self, account_id: &AccountIdType, position_id: u16);
}

/// All main swapping logic abstracted from Runtime implementation is concentrated
/// in this struct
///
#[derive(Debug)]
pub struct Swap<AccountIdType, Ops>
where
    AccountIdType: Eq,
    Ops: SwapDataOperations<AccountIdType>,
{
    state_ops: Ops,
    phantom_key: PhantomData<AccountIdType>,
}

impl<AccountIdType, Ops> Swap<AccountIdType, Ops>
where
    AccountIdType: Eq,
    Ops: SwapDataOperations<AccountIdType>,
{
    pub fn new(ops: Ops) -> Self {
        if !ops.is_v3_initialized() {
            // TODO: Initialize the v3
            // Set price, set initial (protocol owned) liquidity and positions, etc.
        }

        Swap {
            state_ops: ops,
            phantom_key: PhantomData,
        }
    }

    /// Auxiliary method to calculate Alpha amount to match given TAO
    /// amount at the current price for liquidity.
    ///
    /// Returns (Alpha, Liquidity) tuple
    ///
    pub fn get_tao_based_liquidity(&self, _tao: u64) -> (u64, u64) {
        // let current_price = self.state_ops.get_alpha_sqrt_price();
        todo!()
    }

    /// Auxiliary method to calculate TAO amount to match given Alpha
    /// amount at the current price for liquidity.
    ///
    /// Returns (TAO, Liquidity) tuple
    ///
    pub fn get_alpha_based_liquidity(&self, _alpha: u64) -> (u64, u64) {
        // let current_price = self.state_ops.get_alpha_sqrt_price();

        todo!()
    }

    /// Add liquidity at tick index. Creates new tick if it doesn't exist
    ///
    fn add_liquidity_at_index(&self, tick_index: i32, liquidity: u64, upper: bool) {
        // Calculate net liquidity addition
        let net_addition = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        // Find tick by index
        let new_tick = if let Some(mut tick) = self.state_ops.get_tick_by_index(tick_index) {
            tick.liquidity_net = tick.liquidity_net.saturating_add(net_addition);
            tick.liquidity_gross = tick.liquidity_gross.saturating_add(liquidity);
            tick
        } else {
            // Create a new tick
            Tick {
                liquidity_net: net_addition,
                liquidity_gross: liquidity,
                fees_out_tao: U64F64::saturating_from_num(0),
                fees_out_alpha: U64F64::saturating_from_num(0),
            }
        };

        // TODO: Review why Python code uses this code to find index for the new ticks:
        // self.get_tick_index(user_position[0]) + 1
        self.state_ops.insert_tick_by_index(tick_index, new_tick);
    }

    /// Remove liquidity at tick index.
    ///
    fn remove_liquidity_at_index(&self, tick_index: i32, liquidity: u64, upper: bool) {
        // Calculate net liquidity addition
        let net_reduction = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        // Find tick by index
        if let Some(mut tick) = self.state_ops.get_tick_by_index(tick_index) {
            tick.liquidity_net = tick.liquidity_net.saturating_sub(net_reduction);
            tick.liquidity_gross = tick.liquidity_gross.saturating_sub(liquidity);

            // If any liquidity is left at the tick, update it, otherwise remove
            if tick.liquidity_gross == 0 {
                self.state_ops.remove_tick_by_index(tick_index);
            } else {
                self.state_ops.insert_tick_by_index(tick_index, tick);
            }
        };
    }

    /// Add liquidity
    ///
    /// The added liquidity amount can be calculated from TAO and Alpha
    /// amounts using get_tao_based_liquidity and get_alpha_based_liquidity
    /// for the current price tick.
    ///
    /// Removes the balances using state_ops.withdraw_balances()
    ///
    pub fn add_liquidity(
        &self,
        account_id: &AccountIdType,
        tick_low: i32,
        tick_high: i32,
        liquidity: u64,
    ) -> Result<u64, SwapError> {
        // Check if we can add a position
        let position_count = self.state_ops.get_position_count(account_id);
        let max_positions = self.state_ops.get_max_positions();
        if position_count >= max_positions {
            return Err(SwapError::MaxPositionsExceeded);
        }

        // Add liquidity at tick
        self.add_liquidity_at_index(tick_low, liquidity, false);
        self.add_liquidity_at_index(tick_high, liquidity, true);

        // Update current tick and liquidity
        // TODO: Review why python uses this code to get the new tick index:
        // k = self.get_tick_index(i)
        let current_tick_index = self.get_current_tick_index();

        // Update current tick liquidity
        if (tick_low <= current_tick_index) && (current_tick_index <= tick_high) {
            let new_current_liquidity = self
                .state_ops
                .get_current_liquidity()
                .saturating_add(liquidity);
            self.state_ops.set_current_liquidity(new_current_liquidity);
        }

        // Update balances
        let position = Position {
            tick_low,
            tick_high,
            liquidity,
            fees_tao: 0_u64,
            fees_alpha: 0_u64,
        };
        let (tao, alpha) = position.to_token_amounts(current_tick_index)?;
        self.state_ops.withdraw_balances(account_id, tao, alpha);

        // Update reserves
        let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_add(tao);
        self.state_ops.set_tao_reserve(new_tao_reserve);
        let new_alpha_reserve = self.state_ops.get_alpha_reserve().saturating_add(alpha);
        self.state_ops.set_alpha_reserve(new_alpha_reserve);

        // Update user positions
        let position_id = position_count.saturating_add(1);
        self.state_ops
            .update_position(account_id, position_id, position);

        Ok(liquidity)
    }

    /// Remove liquidity and credit balances back to account_id
    ///
    /// Account ID and Position ID identify position in the storage map
    ///
    pub fn remove_liquidity(
        &self,
        account_id: &AccountIdType,
        position_id: u16,
    ) -> Result<RemoveLiquidityResult, SwapError> {
        // Check if position exists
        if let Some(mut pos) = self.state_ops.get_position(account_id, position_id) {
            // Get current price
            let current_tick_index = self.get_current_tick_index();

            // Collect fees and get tao and alpha amounts
            let (fee_tao, fee_alpha) = self.collect_fees(&mut pos);
            let (tao, alpha) = pos.to_token_amounts(current_tick_index)?;

            // Update liquidity at position ticks
            self.remove_liquidity_at_index(pos.tick_low, pos.liquidity, false);
            self.remove_liquidity_at_index(pos.tick_high, pos.liquidity, true);

            // Update current tick liquidity
            if (pos.tick_low <= current_tick_index) && (current_tick_index <= pos.tick_high) {
                let new_current_liquidity = self
                    .state_ops
                    .get_current_liquidity()
                    .saturating_sub(pos.liquidity);
                self.state_ops.set_current_liquidity(new_current_liquidity);
            }

            // Remove user position
            self.state_ops.remove_position(account_id, position_id);

            // TODO: Clear with R&D
            // Update current price (why?)
            // self.state_ops.set_alpha_sqrt_price(sqrt_price);

            // Return Ok result
            Ok(RemoveLiquidityResult {
                tao,
                alpha,
                fee_tao,
                fee_alpha,
            })
        } else {
            // Position doesn't exist
            Err(SwapError::LiquidityNotFound)
        }
    }

    /// Perform a swap
    ///
    /// Returns a tuple (amount, refund), where amount is the resulting paid out amount
    ///
    pub fn swap(
        &self,
        order_type: &OrderType,
        amount: u64,
        sqrt_price_limit: SqrtPrice,
    ) -> Result<SwapResult, SwapError> {
        let one = U64F64::saturating_from_num(1);

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
            let sqrt_price_edge = self.get_sqrt_price_edge(order_type);
            let possible_delta_in =
                amount_remaining.saturating_sub(self.get_fee_amount(amount_remaining));
            let sqrt_price_target = self.get_sqrt_price_target(order_type, possible_delta_in);
            let target_quantity = self.get_target_quantity(order_type, possible_delta_in);
            let edge_quantity = U64F64::saturating_from_num(1).safe_div(sqrt_price_edge.into());
            let lim_quantity = one
                .safe_div(self.state_ops.get_min_sqrt_price())
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
                    delta_in = self.get_delta_in(order_type, sqrt_price_limit);
                    final_price = sqrt_price_limit;
                    stop_and_refund = true;
                }
            } else if target_quantity > edge_quantity {
                if edge_quantity < lim_quantity {
                    // do crossing at price edge
                    action = SwapStepAction::Crossing;
                    delta_in = self.get_delta_in(order_type, sqrt_price_edge);
                    final_price = sqrt_price_edge;
                } else if edge_quantity > lim_quantity {
                    // stop_in at price limit
                    action = SwapStepAction::StopIn;
                    delta_in = self.get_delta_in(order_type, sqrt_price_limit);
                    final_price = sqrt_price_limit;
                    stop_and_refund = true;
                } else {
                    // stop_on at price limit
                    action = SwapStepAction::StopOn;
                    delta_in = self.get_delta_in(order_type, sqrt_price_edge);
                    final_price = sqrt_price_edge;
                    stop_and_refund = true;
                }
            } else {
                // targetQuantity = edgeQuantity
                if target_quantity <= lim_quantity {
                    // stop_on at price edge
                    delta_in = self.get_delta_in(order_type, sqrt_price_edge);
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
                    delta_in = self.get_delta_in(order_type, sqrt_price_limit);
                    final_price = sqrt_price_limit;
                    stop_and_refund = true;
                }
            }

            let swap_result = self.swap_step(order_type, delta_in, final_price, action)?;
            amount_remaining = amount_remaining.saturating_sub(swap_result.amount_to_take);
            amount_paid_out = amount_paid_out.saturating_add(swap_result.delta_out);

            if stop_and_refund {
                refund = amount_remaining;
                amount_remaining = 0;
            }

            iteration_counter = iteration_counter.saturating_add(1);
            if iteration_counter > iter_limit {
                return Err(SwapError::TooManySwapSteps);
            }
        }

        Ok(SwapResult {
            amount_paid_out,
            refund,
        })
    }

    fn get_current_tick_index(&self) -> i32 {
        let current_price = self.state_ops.get_alpha_sqrt_price();
        let maybe_current_tick_index = sqrt_price_to_tick_index(current_price);
        if let Ok(index) = maybe_current_tick_index {
            index
        } else {
            // Current price is out of allow the min-max range, and it should be corrected to
            // maintain the range.
            let max_price =
                tick_index_to_sqrt_price(MAX_TICK).unwrap_or(SqrtPrice::saturating_from_num(1000));
            let min_price = tick_index_to_sqrt_price(MIN_TICK)
                .unwrap_or(SqrtPrice::saturating_from_num(0.000001));
            if current_price > max_price {
                self.state_ops.set_alpha_sqrt_price(max_price);
                MAX_TICK
            } else {
                self.state_ops.set_alpha_sqrt_price(min_price);
                MIN_TICK
            }
        }
    }

    /// Process a single step of a swap
    ///
    fn swap_step(
        &self,
        order_type: &OrderType,
        delta_in: u64,
        sqrt_price_final: SqrtPrice,
        action: SwapStepAction,
    ) -> Result<SwapStepResult, SwapError> {
        // amount_swapped = delta_in / (1 - self.fee_size)
        let fee_rate = U64F64::saturating_from_num(self.state_ops.get_fee_rate());
        let u16_max = U64F64::saturating_from_num(u16::MAX);
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let amount_swapped =
            delta_fixed.saturating_mul(u16_max.safe_div(u16_max.saturating_sub(fee_rate)));

        // Hold the fees
        let fee = self.get_fee_amount(amount_swapped.saturating_to_num::<u64>());
        self.add_fees(order_type, fee);
        let delta_out = self.convert_deltas(order_type, delta_in);

        self.update_reserves(order_type, delta_in, delta_out);

        // Get current tick
        let current_tick_index = self.get_current_tick_index();

        match action {
            SwapStepAction::Crossing => {
                let maybe_tick = match order_type {
                    OrderType::Sell => self.find_closest_lower_active_tick(current_tick_index),
                    OrderType::Buy => self.find_closest_higher_active_tick(current_tick_index),
                };
                if let Some(mut tick) = maybe_tick {
                    tick.fees_out_tao = self
                        .state_ops
                        .get_fee_global_tao()
                        .saturating_sub(tick.fees_out_tao);
                    tick.fees_out_alpha = self
                        .state_ops
                        .get_fee_global_alpha()
                        .saturating_sub(tick.fees_out_alpha);
                    self.update_liquidity_at_crossing(order_type)?;
                    self.state_ops
                        .insert_tick_by_index(current_tick_index, tick);
                } else {
                    return Err(SwapError::InsufficientLiquidity);
                }
            }
            SwapStepAction::StopOn => match order_type {
                OrderType::Sell => {}
                OrderType::Buy => {
                    self.update_liquidity_at_crossing(order_type)?;
                    let maybe_tick = self.find_closest_higher_active_tick(current_tick_index);

                    if let Some(mut tick) = maybe_tick {
                        tick.fees_out_tao = self
                            .state_ops
                            .get_fee_global_tao()
                            .saturating_sub(tick.fees_out_tao);
                        tick.fees_out_alpha = self
                            .state_ops
                            .get_fee_global_alpha()
                            .saturating_sub(tick.fees_out_alpha);
                        self.state_ops
                            .insert_tick_by_index(current_tick_index, tick);
                    } else {
                        return Err(SwapError::InsufficientLiquidity);
                    }
                }
            },
            SwapStepAction::StopIn => {}
        }

        // Update current price, which effectively updates current tick too
        self.state_ops.set_alpha_sqrt_price(sqrt_price_final);

        Ok(SwapStepResult {
            amount_to_take: amount_swapped.saturating_to_num::<u64>(),
            delta_out,
        })
    }

    /// Get the square root price at the current tick edge for the given direction (order type)
    /// If order type is Buy, then price edge is the high tick bound price, otherwise it is
    /// the low tick bound price.
    ///
    /// If anything is wrong with tick math and it returns Err, we just abort the deal, i.e.
    /// return the edge that is impossible to execute
    ///
    fn get_sqrt_price_edge(&self, order_type: &OrderType) -> SqrtPrice {
        let fallback_price_edge_value = (match order_type {
            OrderType::Buy => tick_index_to_sqrt_price(MIN_TICK),
            OrderType::Sell => tick_index_to_sqrt_price(MAX_TICK),
        })
        .unwrap_or(SqrtPrice::saturating_from_num(0));

        let current_price = self.state_ops.get_alpha_sqrt_price();
        let maybe_current_tick_index = sqrt_price_to_tick_index(current_price);

        if let Ok(current_tick_index) = maybe_current_tick_index {
            tick_index_to_sqrt_price(match order_type {
                OrderType::Buy => current_tick_index.saturating_add(1),
                OrderType::Sell => current_tick_index,
            })
            .unwrap_or(fallback_price_edge_value)
        } else {
            fallback_price_edge_value
        }
    }

    /// Calculate fee amount
    ///
    /// Fee is provided by state ops as u16-normalized value.
    ///
    fn get_fee_amount(&self, amount: u64) -> u64 {
        let fee_rate = U64F64::saturating_from_num(self.state_ops.get_fee_rate())
            .safe_div(U64F64::saturating_from_num(u16::MAX));
        U64F64::saturating_from_num(amount)
            .saturating_mul(fee_rate)
            .saturating_to_num::<u64>()
    }

    /// Here we subtract minimum safe liquidity from current liquidity to stay in the
    /// safe range
    ///
    fn get_safe_current_liquidity(&self) -> U64F64 {
        U64F64::saturating_from_num(
            self.state_ops
                .get_current_liquidity()
                .saturating_sub(self.state_ops.get_minimum_liquidity()),
        )
    }

    /// Get the target square root price based on the input amount
    ///
    fn get_sqrt_price_target(&self, order_type: &OrderType, delta_in: u64) -> SqrtPrice {
        let liquidity_curr = self.get_safe_current_liquidity();
        let sqrt_price_curr = self.state_ops.get_alpha_sqrt_price().into();
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let one = U64F64::saturating_from_num(1);

        if liquidity_curr > 0 {
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
        } else {
            // No liquidity means price should remain current
            sqrt_price_curr
        }
    }

    /// Get the target quantity, which is
    ///     `1 / (target square root price)` in case of sell order
    ///     `target square root price` in case of buy order
    ///
    /// ...based on the input amount, current liquidity, and current alpha price
    ///
    fn get_target_quantity(&self, order_type: &OrderType, delta_in: u64) -> SqrtPrice {
        let liquidity_curr = self.get_safe_current_liquidity();
        let sqrt_price_curr = self.state_ops.get_alpha_sqrt_price().into();
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let one = U64F64::saturating_from_num(1);

        if liquidity_curr > 0 {
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
        } else {
            // No liquidity means zero
            SqrtPrice::saturating_from_num(0)
        }
    }

    /// Get the input amount needed to reach the target price
    ///
    fn get_delta_in(&self, order_type: &OrderType, sqrt_price_target: SqrtPrice) -> u64 {
        let liquidity_curr = self.get_safe_current_liquidity();
        let one = U64F64::saturating_from_num(1);
        let sqrt_price_curr = self.state_ops.get_alpha_sqrt_price().into();

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

    /// Add fees to the global fee counters
    fn add_fees(&self, order_type: &OrderType, fee: u64) {
        let liquidity_curr = self.get_safe_current_liquidity();
        if liquidity_curr > 0 {
            let fee_global_tao: U64F64 = self.state_ops.get_fee_global_tao();
            let fee_global_alpha: U64F64 = self.state_ops.get_fee_global_alpha();
            let fee_fixed: U64F64 = U64F64::saturating_from_num(fee);

            match order_type {
                OrderType::Sell => {
                    self.state_ops.set_fee_global_tao(
                        fee_global_tao.saturating_add(fee_fixed.safe_div(liquidity_curr)),
                    );
                }
                OrderType::Buy => {
                    self.state_ops.set_fee_global_alpha(
                        fee_global_alpha.saturating_add(fee_fixed.safe_div(liquidity_curr)),
                    );
                }
            }
        }
    }

    /// Convert input amount (delta_in) to output amount (delta_out)
    ///
    /// This is the core method of uniswap V3 that tells how much
    /// output token is given for an amount of input token within one
    /// price tick.
    ///
    fn convert_deltas(&self, order_type: &OrderType, delta_in: u64) -> u64 {
        let liquidity_curr = SqrtPrice::saturating_from_num(self.state_ops.get_current_liquidity());
        let sqrt_price_curr = self.state_ops.get_alpha_sqrt_price();
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

    /// Multiplies a `u64` by a `U64F64` and returns a `u128` result without overflow.
    // pub fn mul_u64_u64f64(a: u64, b: U64F64) -> u128 {
    //     // Multiply a by integer part of b in integer math.
    //     // Result doesn't overflow u128 because both multipliers are 64 bit
    //     let int_b: u64 = b.saturating_to_num::<u64>();
    //     let high = (a as u128).saturating_mul(int_b as u128);

    //     // Multiply a by fractional part of b using U64F64
    //     let frac_b = b.saturating_sub(U64F64::saturating_from_num(int_b));
    //     let low = U64F64::saturating_from_num(a).saturating_mul(frac_b);

    //     // The only possible overflow (that is cut off by saturating math)
    //     // is when a is u64::MAX, int_b is u64::MAX, and frac_b is non-zero,
    //     // which is negligible error if we saturate and return u128::MAX
    //     high.saturating_add(low).saturating_to_num::<u128>()
    // }

    /// Update token reserves after a swap
    ///
    fn update_reserves(&self, order_type: &OrderType, amount_in: u64, amount_out: u64) {
        let (new_tao_reserve, new_alpha_reserve) = match order_type {
            OrderType::Sell => (
                self.state_ops.get_tao_reserve().saturating_add(amount_in),
                self.state_ops
                    .get_alpha_reserve()
                    .saturating_sub(amount_out),
            ),
            OrderType::Buy => (
                self.state_ops.get_tao_reserve().saturating_sub(amount_in),
                self.state_ops
                    .get_alpha_reserve()
                    .saturating_add(amount_out),
            ),
        };

        self.state_ops.set_tao_reserve(new_tao_reserve);
        self.state_ops.set_alpha_reserve(new_alpha_reserve);
    }

    fn get_liquidity_update_u64(&self, tick: &Tick) -> u64 {
        let liquidity_update_abs_i128 = tick.liquidity_net.abs();
        if liquidity_update_abs_i128 > u64::MAX as i128 {
            u64::MAX
        } else {
            liquidity_update_abs_i128 as u64
        }
    }

    /// Update liquidity when crossing a tick
    ///
    fn update_liquidity_at_crossing(&self, order_type: &OrderType) -> Result<(), SwapError> {
        let mut liquidity_curr = self.state_ops.get_current_liquidity();
        let current_tick_index = self.get_current_tick_index();
        match order_type {
            OrderType::Sell => {
                let maybe_tick = self.find_closest_lower_active_tick(current_tick_index);
                if let Some(tick) = maybe_tick {
                    let liquidity_update_abs_u64 = self.get_liquidity_update_u64(&tick);

                    liquidity_curr = if tick.liquidity_net >= 0 {
                        liquidity_curr.saturating_sub(liquidity_update_abs_u64)
                    } else {
                        liquidity_curr.saturating_add(liquidity_update_abs_u64)
                    };
                } else {
                    return Err(SwapError::InsufficientLiquidity);
                }
            }
            OrderType::Buy => {
                let maybe_tick = self.find_closest_higher_active_tick(current_tick_index);
                if let Some(tick) = maybe_tick {
                    let liquidity_update_abs_u64 = self.get_liquidity_update_u64(&tick);

                    liquidity_curr = if tick.liquidity_net >= 0 {
                        liquidity_curr.saturating_add(liquidity_update_abs_u64)
                    } else {
                        liquidity_curr.saturating_sub(liquidity_update_abs_u64)
                    };
                } else {
                    return Err(SwapError::InsufficientLiquidity);
                }
            }
        }

        self.state_ops.set_current_liquidity(liquidity_curr);
        Ok(())
    }

    /// Collect fees for a position
    /// Updates the position
    ///
    fn collect_fees(&self, position: &mut Position) -> (u64, u64) {
        let mut fee_tao = self.get_fees_in_range(position, true);
        let mut fee_alpha = self.get_fees_in_range(position, false);

        fee_tao = fee_tao.saturating_sub(position.fees_tao);
        fee_alpha = fee_alpha.saturating_sub(position.fees_alpha);

        position.fees_tao = fee_tao;
        position.fees_alpha = fee_alpha;

        fee_tao = position.liquidity.saturating_mul(fee_tao);
        fee_alpha = position.liquidity.saturating_mul(fee_alpha);

        (fee_tao, fee_alpha)
    }

    /// Get fees in a position's range
    ///
    /// If quote flag is true, Tao is returned, otherwise alpha.
    ///
    fn get_fees_in_range(&self, position: &mut Position, quote: bool) -> u64 {
        let i_lower = position.tick_low;
        let i_upper = position.tick_high;

        let fee_global = if quote {
            self.state_ops.get_fee_global_tao()
        } else {
            self.state_ops.get_fee_global_alpha()
        };

        fee_global
            .saturating_sub(self.get_fees_below(i_lower, quote))
            .saturating_sub(self.get_fees_above(i_upper, quote))
            .saturating_to_num::<u64>()
    }

    /// Get fees above a tick
    ///
    fn get_fees_above(&self, tick_index: i32, quote: bool) -> U64F64 {
        let maybe_tick_index = find_closest_lower_active_tick_index(&self.state_ops, tick_index);
        let current_tick = self.get_current_tick_index();

        if let Some(tick_index) = maybe_tick_index {
            let tick = self
                .state_ops
                .get_tick_by_index(tick_index)
                .unwrap_or_default();
            if tick_index <= current_tick {
                if quote {
                    self.state_ops
                        .get_fee_global_tao()
                        .saturating_sub(tick.fees_out_tao)
                } else {
                    self.state_ops
                        .get_fee_global_alpha()
                        .saturating_sub(tick.fees_out_alpha)
                }
            } else {
                if quote {
                    tick.fees_out_tao
                } else {
                    tick.fees_out_alpha
                }
            }
        } else {
            U64F64::saturating_from_num(0)
        }
    }

    /// Get fees below a tick
    fn get_fees_below(&self, tick_index: i32, quote: bool) -> U64F64 {
        let maybe_tick_index = find_closest_lower_active_tick_index(&self.state_ops, tick_index);
        let current_tick = self.get_current_tick_index();

        if let Some(tick_index) = maybe_tick_index {
            let tick = self
                .state_ops
                .get_tick_by_index(tick_index)
                .unwrap_or_default();
            if tick_index <= current_tick {
                if quote {
                    tick.fees_out_tao
                } else {
                    tick.fees_out_alpha
                }
            } else {
                if quote {
                    self.state_ops
                        .get_fee_global_tao()
                        .saturating_sub(tick.fees_out_tao)
                } else {
                    self.state_ops
                        .get_fee_global_alpha()
                        .saturating_sub(tick.fees_out_alpha)
                }
            }
        } else {
            U64F64::saturating_from_num(0)
        }
    }

    pub fn find_closest_lower_active_tick(&self, index: i32) -> Option<Tick> {
        let maybe_tick_index = find_closest_lower_active_tick_index(&self.state_ops, index);
        if let Some(tick_index) = maybe_tick_index {
            self.state_ops.get_tick_by_index(tick_index)
        } else {
            None
        }
    }

    pub fn find_closest_higher_active_tick(&self, index: i32) -> Option<Tick> {
        let maybe_tick_index = find_closest_higher_active_tick_index(&self.state_ops, index);
        if let Some(tick_index) = maybe_tick_index {
            self.state_ops.get_tick_by_index(tick_index)
        } else {
            None
        }
    }
}
