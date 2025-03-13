use safe_math::*;
use substrate_fixed::types::U64F64;

/// The width of a single price tick. Expressed in rao units.
pub const TICK_SPACING: u64 = 10_000;

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

struct RemoveLiquidityResult {
    tao: u64,
    alpha: u64,
    fee_tao: u64,
    fee_alpha: u64,
}

struct SwapResult {
    amount_paid_out: u64,
    refund: u64,
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
struct Position {
    tick_low: u64,
    tick_high: u64,
    liquidity: u64,
    fees_tao: u64,
    fees_alpha: u64,
}

impl Position {
    /// Converts tick index into SQRT of lower price of this tick
    /// In order to find the higher price of this tick, call
    /// tick_index_to_sqrt_price(tick_idx + 1)
    ///
    pub fn tick_index_to_sqrt_price(tick_idx: u64) -> SqrtPrice {
        // python: (1 + self.tick_spacing) ** (i / 2)
        let tick_spacing_tao = SqrtPrice::from_num(TICK_SPACING)
            .saturating_div(SqrtPrice::from_num(1e9))
            + SqrtPrice::from_num(1.0);

        tick_spacing_tao
            .checked_pow(tick_idx / 2)
            .unwrap_or_default()
    }

    /// Converts SQRT price to tick index
    /// Because the tick is the range of prices [sqrt_lower_price, sqrt_higher_price),
    /// the resulting tick index matches the price by the following inequality:
    ///    sqrt_lower_price <= sqrt_price < sqrt_higher_price
    ///
    pub fn sqrt_price_to_tick_index(sqrt_price: SqrtPrice) -> u64 {
        let tick_spacing_tao = SqrtPrice::from_num(TICK_SPACING)
            .saturating_div(SqrtPrice::from_num(1e9))
            + SqrtPrice::from_num(1.0);
        // python: math.floor(math.log(sqrt_p) / math.log(1 + self.tick_spacing)) * 2
        todo!()
    }

    /// Converts position to token amounts
    ///
    /// returns tuple of (TAO, Alpha)
    ///
    pub fn to_token_amounts(self, current_tick: u64) -> (u64, u64) {
        let one = 1.into();

        let sqrt_price_curr = Self::tick_index_to_sqrt_price(current_tick);
        let sqrt_pa = Self::tick_index_to_sqrt_price(self.tick_low);
        let sqrt_pb = Self::tick_index_to_sqrt_price(self.tick_high);

        if sqrt_price_curr < sqrt_pa {
            (
                liquidity
                    .saturating_mul(one.safe_div(sqrt_pa).saturating_sub(one.safe_div(sqrt_pb))),
                0,
            )
        } else if sqrt_price_curr > sqrt_pb {
            (0, liquidity.saturating_mul(sqrt_pb.saturating_sub(sqrt_pa)))
        } else {
            (
                liquidity.saturating_mul(
                    one.save_div(sqrt_price_curr)
                        .saturating_sub(one.safe_div(sqrt_pb)),
                ),
                liquidity.saturating_mul(sqrt_price_curr.saturating_sub(sqrt_pa)),
            )
        }
    }
}

/// Tick is the price range determined by tick index (not part of this struct,
/// but is the key at which the Tick is stored in state hash maps). Tick struct
/// stores liquidity and fee information.
///
///   - Net liquidity
///   - Gross liquidity
///   - Fees (above global) in both currencies
///
struct Tick {
    liquidity_net: i128,
    liquidity_gross: u64,
    fees_out_tao: u64,
    fees_out_alpha: u64,
}

/// This trait implementation depends on Runtime and it needs to be implemented
/// in the pallet to be able to work with chain state and per subnet. All subnet
/// swaps are independent and hence netuid is abstracted away from swap implementation.
///
pub trait SwapDataOperations<AccountIdType> {
    /// Tells if v3 swap is initialized in the state. v2 only provides base and quote
    /// reserves, while v3 also stores ticks and positions, which need to be initialized
    /// at the first pool creation.
    fn is_v3_initialized() -> bool;
    /// Returns u16::MAX normalized fee rate. For example, 0.3% is approximately 196.
    fn get_fee_rate() -> u16;
    /// Minimum liquidity that is safe for rounding and integer math.
    fn get_minimum_liquidity() -> u64;
    fn get_tick_by_index(tick_index: u64) -> Option<Tick>;
    fn insert_tick_by_index(tick_index: u64, tick: Tick);
    fn remove_tick_by_index(tick_index: u64);
    /// Minimum sqrt price across all active ticks
    fn get_min_sqrt_price() -> SqrtPrice;
    /// Maximum sqrt price across all active ticks
    fn get_max_sqrt_price() -> SqrtPrice;
    fn get_tao_reserve() -> u64;
    fn set_tao_reserve() -> u64;
    fn get_alpha_reserve() -> u64;
    fn set_alpha_reserve() -> u64;
    fn get_alpha_sqrt_price() -> u64;
    fn set_alpha_sqrt_price() -> u64;

    // Getters/setters for global accrued fees in alpha and tao per subnet
    fn get_fee_global_tao() -> U64F64;
    fn set_fee_global_tao(fee: U64F64);
    fn get_fee_global_alpha() -> U64F64;
    fn set_fee_global_alpha(fee: U64F64);

    /// Get current tick liquidity
    fn get_current_liquidity() -> u64;
    /// Set current tick liquidity
    fn set_current_liquidity(liquidity: u64);

    // User account operations
    fn get_max_positions() -> u16;
    fn withdraw_balances(account_id: &AccountIdType, tao: u64, alpha: u64) -> (u64, u64);
    fn deposit_balances(account_id: &AccountIdType, tao: u64, alpha: u64);
    fn get_position_count(account_id: &AccountIdType) -> u16;
    fn get_position(account_id: &AccountIdType, position_id: u16) -> Option<Position>;
    fn create_position(account_id: &AccountIdType, positions: Position);
    fn update_position(account_id: &AccountIdType, position_id: u16, positions: Position);
    fn remove_position(account_id: &AccountIdType, position_id: u16);
}

/// All main swapping logic abstracted from Runtime implementation is concentrated
/// in this struct
///
#[derive(Debug)]
pub struct Swap<AccountIdType, Ops>
where
    AccountIdType: Eq,
    Ops: SwapDataOperations,
{
    state_ops: Ops,
    phantom_key: marker::PhantomData<AccountIdType>,
}

impl<AccountIdType, Ops> Swap<AccountIdType, Ops>
where
    AccountIdType: Eq,
    Ops: SwapDataOperations,
{
    pub fn new(ops: Ops) -> Self {
        if !ops.is_v3_initialized() {
            // TODO: Initialize the v3
            // Set price, set initial (protocol owned) liquidity and positions, etc.
        }

        Swap { state_ops: ops }
    }

    /// Auxiliary method to calculate Alpha amount to match given TAO
    /// amount at the current price for liquidity.
    ///
    /// Returns (Alpha, Liquidity) tuple
    ///
    pub fn get_tao_based_liquidity(&self, tao: u64) -> (u64, u64) {
        let current_price = self.state_ops.get_alpha_sqrt_price();

        // TODO
    }

    /// Auxiliary method to calculate TAO amount to match given Alpha
    /// amount at the current price for liquidity.
    ///
    /// Returns (TAO, Liquidity) tuple
    ///
    pub fn get_alpha_based_liquidity(&self, alpha: u64) -> (u64, u64) {
        let current_price = self.state_ops.get_alpha_sqrt_price();

        // TODO
    }

    /// Add liquidity at tick index. Creates new tick if it doesn't exist
    ///
    fn add_liquidity_at_index(tick_index: u64, liquidity: u64, upper: bool) {
        // Calculate net liquidity addition
        let net_addition = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        // Find tick by index
        let new_tick = if let Some(tick) = self.state_ops.get_tick_by_index(tick_index) {
            tick.liquidity_net = tick.liquidity_net.saturating_add(net_addition);
            tick.liquidity_gross = tick.liquidity_gross.saturating_add(liquidity);
        } else {
            // Create a new tick
            Tick {
                liquidity_net: net_addition,
                liquidity_gross: liquidity,
                fees_out_tao: 0_u64,
                fees_out_alpha: 0_u64,
            }
        };

        // TODO: Review why Python code uses this code to find index for the new ticks:
        // self.get_tick_index(user_position[0]) + 1
        self.state_ops.insert_tick_by_index(tick_index, new_tick);
    }

    /// Remove liquidity at tick index.
    ///
    fn remove_liquidity_at_index(tick_index: u64, liquidity: u64, upper: bool) {
        // Calculate net liquidity addition
        let net_reduction = if upper {
            (liquidity as i128).neg()
        } else {
            liquidity as i128
        };

        // Find tick by index
        let new_tick = if let Some(tick) = self.state_ops.get_tick_by_index(tick_index) {
            tick.liquidity_net = tick.liquidity_net.saturating_sub(net_reduction);
            tick.liquidity_gross = tick.liquidity_gross.saturating_sub(liquidity);
        };

        // If any liquidity is left at the tick, update it, otherwise remove
        if tick.liquidity_gross == 0 {
            self.state_ops.remove_tick(tick_index);
        } else {
            self.state_ops.insert_tick_by_index(tick_index, new_tick);
        }
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
        tick_low: u64,
        tick_high: u64,
        liquidity: u64,
    ) -> Result<u64, ()> {
        // Check if we can add a position
        let position_count = self.state_ops.get_position_count(account_id);
        let max_positions = get_max_positions();
        if position_count >= max_positions {
            return Err(());
        }

        // Add liquidity at tick
        self.add_liquidity_at_index(tick_low, liquidity, false);
        self.add_liquidity_at_index(tick_high, liquidity, true);

        // Update current tick and liquidity
        // TODO: Review why python uses this code to get the new tick index:
        // k = self.get_tick_index(i)
        let current_price = self.state_ops.get_alpha_sqrt_price();
        let current_tick_index = Position::sqrt_price_to_tick_index(current_price);

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
        let (tao, alpha) = position.to_token_amounts(current_tick_index);
        self.state_ops.withdraw_balances(account_id, tao, alpha);

        // Update reserves
        let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_add(tao);
        self.state_ops.set_tao_reserve(new_tao_reserve);
        let new_alpha_reserve = self.get_alpha_reserve().saturating_add(alpha);
        self.state_ops.set_alpha_reserve(new_alpha);

        // Update user positions
        let position_id = position_count.saturating_add(1);
        self.state_ops
            .set_position(account_id, position_id, position);

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
    ) -> Result<RemoveLiquidityResult, ()> {
        // Check if position exists
        if let Some(pos) = self.state_ops.get_position(account_id, position_id) {
            // Get current price
            let current_price = self.state_ops.get_alpha_sqrt_price();
            let current_tick_index = Position::sqrt_price_to_tick_index(current_price);

            // Collect fees and get tao and alpha amounts
            let (fee_tao, fee_alpha) = self.collect_fees(pos);
            let (tao, alpha) = pos.to_token_amounts(current_tick_index);

            // Update liquidity at position ticks
            self.remove_liquidity_at_index(pos.tick_low, pos.liquidity, false);
            self.remove_liquidity_at_index(pos.tick_high, pos.liquidity, true);

            // Update current tick liquidity
            if (pos.tick_low <= current_tick_index) && (current_tick_index <= pos.tick_high) {
                let new_current_liquidity = self
                    .state_ops
                    .get_current_liquidity()
                    .saturating_sub(liquidity);
                self.state_ops.set_current_liquidity(new_current_liquidity);
            }

            // Remove user position
            self.state_ops.remove_position(account_id, position_id);

            // Update current price (why?)
            // i = self.sqrt_price_to_tick(self.sqrt_price_curr)
            // k = self.get_tick_index(i)
            // self.i_curr = self.active_ticks[k]
            todo!();

            // Update reserves
            let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_sub(tao);
            self.state_ops.set_tao_reserve(new_tao_reserve);
            let new_alpha_reserve = self.get_alpha_reserve().saturating_sub(alpha);
            self.state_ops.set_alpha_reserve(new_alpha);

            // Return Ok result
            Ok(RemoveLiquidityResult {
                tao,
                alpha,
                fee_tao,
                fee_alpha,
            })
        } else {
            // Position doesn't exist
            Err(())
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
    ) -> Result<SwapResult, ()> {
        let one = U64F64::saturating_from_num(1);

        // Here we store the remaining amount that needs to be exchanged
        // If order_type is Buy, then it expresses Tao amount, if it is Sell,
        // then amount_remaining is Alpha.
        let mut amount_remaining = amount;
        let mut amount_paid_out = 0;

        // A bit of fool proofing
        let mut iteration_counter = 0;
        let iter_limit = 1000;

        // Swap one tick at a time until we reach one of the following conditions:
        //   - Swap all provided amount
        //   - Reach limit price
        //   - Use up all liquidity (up to safe minimum)
        while amount_remaining > 0 {
            let sqrt_price_edge = self.get_sqrt_price_edge(order_type);
            let possible_delta_in =
                amount_remaining.saturating_sub(self.get_fee_amount(amount_remaining));
            let target_quantity = self.get_target_quantity(order_type, possible_delta_in);
            let edge_quantity = U64F64::saturating_from_num(1).safe_div(sqrt_price_edge.into());
            let lim_quantity = one
                .safe_div(self.state_ops.get_min_sqrt_price())
                .saturating_add(one.safe_div(sqrt_price_limit.into()));

            let mut action: SwapStepAction;
            let mut delta_in;
            let mut final_price;
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
                    delta_in = self.get_delta_in(order_type, sqrt_price_lim);
                    final_price = sqrt_price_lim;
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
                    delta_in = self.get_delta_in(zero_for_one, sqrt_price_lim);
                    final_price = sqrt_price_lim;
                    stop_and_refund = true;
                } else {
                    // stop_on at price limit
                    action = SwapStepAction::StopOn;
                    delta_in = self.get_delta_in(zero_for_one, sqrt_price_edge);
                    final_price = sqrt_price_edge;
                    stop_and_refund = true;
                }
            } else {
                // targetQuantity = edgeQuantity
                if target_quantity <= lim_quantity {
                    // stop_on at price edge
                    delta_in = self.get_delta_in(zero_for_one, sqrt_price_edge);
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
                    delta_in = self.get_delta_in(zero_for_one, sqrt_price_lim);
                    final_price = sqrt_price_lim;
                    stop_and_refund = true;
                }
            }

            let (amount_to_take, delta_out) =
                self.swap_step(zero_for_one, delta_in, final_price, action);
            amount_remaining = amount_remaining.saturating_sub(amount_to_take);
            amount_paid_out = amount_paid_out.saturating_add(delta_out);

            if stop_and_refund {
                refund = amount_remaining;
                amount_remaining = 0;
            }

            iteration_counter = iteration_counter.saturating_add(1);
            if iteration_counter > iter_limit {
                return Err(());
            }
        }

        Ok(SwapResult {
            amount_paid_out,
            refund,
        })
    }

    /// Process a single step of a swap
    ///
    fn swap_step(
        &self,
        order_type: &OrderType,
        delta_in: u64,
        sqrt_price_final: SqrtPrice,
        action: SwapStepAction,
    ) -> SwapResult {
        // amount_swapped = delta_in / (1 - self.fee_size)
        let fee_rate = self.state_ops.get_fee_rate();
        let u16_max = U64F64::saturating_from_num(u16::MAX);
        let delta_fixed = U64F64::saturating_from_num(delta_in);
        let amount_swapped =
            delta_fixed.saturating_mul(u16_max.safe_div(u16_max.saturagin_sub(fee_rate)));

        // Hold the fees
        let fee = self.get_fee_amount(amount_swapped);
        self.add_fees(zero_for_one, fee);
        delta_out = self.convert_deltas(zero_for_one, delta_in);

        todo!()
        // self.update_reserves(zero_for_one, delta_in, delta_out)
        // self.sqrt_price_curr = sqrt_price_final

        // if action == "crossing":
        //     if zero_for_one:
        //         k = self.get_tick_index(self.i_curr)
        //         old_value = self.fee_outside0[k]
        //         new_value = self.fee_global0 - old_value
        //         self.fee_outside0[k] = new_value
        //         self.fee_outside1[k] = self.fee_global1 - self.fee_outside1[k]
        //         self.update_liquidity_at_crossing(zero_for_one)
        //         self.i_curr = self.active_ticks[k - 1]
        //     else:
        //         k = self.get_tick_index(self.i_curr)
        //         self.fee_outside0[k + 1] = self.fee_global0 - self.fee_outside0[k + 1]
        //         self.fee_outside1[k + 1] = self.fee_global1 - self.fee_outside1[k + 1]
        //         self.update_liquidity_at_crossing(zero_for_one)
        //         self.i_curr = self.active_ticks[k + 1]
        //     if self.print_stuff:
        //         print(f"crossing tick into i={self.i_curr}")
        // elif action == "stop_on":
        //     if not zero_for_one:
        //         self.update_liquidity_at_crossing(zero_for_one)
        //         k = self.get_tick_index(self.i_curr)
        //         self.fee_outside0[k + 1] = self.fee_global0 - self.fee_outside0[k + 1]
        //         self.fee_outside1[k + 1] = self.fee_global1 - self.fee_outside1[k + 1]
        //         self.i_curr = self.active_ticks[k + 1]
        //     if self.print_stuff:
        //         print(f"stopping ON i={self.i_curr}")
        // else:  # stop_in
        //     if self.print_stuff:
        //         print(f"stopping IN i={self.i_curr}")

        // return amount_to_take, delta_out
    }

    /// Get the square root price at the current tick edge for the given direction (order type)
    /// If order type is Buy, then price edge is the high tick bound price, otherwise it is
    /// the low tick bound price.
    ///
    fn get_sqrt_price_edge(&self, order_type: &OrderType) {
        let current_price = self.state_ops.get_alpha_sqrt_price();
        let current_tick_index = Position::sqrt_price_to_tick_index(current_price);
        Position::tick_index_to_sqrt_price(match order_type {
            OrderType::Buy => current_tick_index.saturating_add(1),
            OrderType::Sell => current_tick_index,
        })
    }

    /// Calculate fee amount
    ///
    /// Fee is provided by state ops as u16-normalized value.
    ///
    fn get_fee_amount(&self, amount: u64) -> u64 {
        let fee_rate = U64I64::saturating_from_num(self.state_ops.get_fee_rate())
            .save_div(U64I64::saturating_from_num(u16::MAX));
        U64I64::saturating_from_num(amount)
            .saturating_mul(fee_rate)
            .saturating_to_num::<u64>()
    }

    /// Here we subtract minimum safe liquidity from current liquidity to stay in the
    /// safe range
    ///
    fn get_safe_current_liquidity(&self) -> U64F64 {
        U64F64::saturating_from_num(self.state_ops.get_current_liquidity())
            .saturating_sub(self.state_ops.get_minimum_liquidity())
    }

    /// Get the target quantity, which is
    ///     `1 / (target square root price)` in case of sell order
    ///     `target square root price` in case of buy order
    ///
    /// ...based on the input amount, current liquidity, and current alpha price
    ///
    fn get_target_quantity(self, order_type: &OrderType, delta_in: u64) -> SqrtPrice {
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
            0.into()
        }
    }

    /// Get the input amount needed to reach the target price
    ///
    fn get_delta_in(&self, order_type: &OrderType, sqrt_price_target: SqrtPrice) {
        let liquidity_curr = self.get_safe_current_liquidity();
        let one = U64F64::saturating_from_num(1);
        let sqrt_price_curr = self.state_ops.get_alpha_sqrt_price().into();

        match order_type {
            OrderType::Sell => liquidity_curr.saturating_mul(
                one.safe_div(sqrt_price_target.into())
                    .saturating_sub(one.safe_div(sqrt_price_curr)),
            ),
            OrderType::Buy => {
                liquidity_curr.saturating_mul(sqrt_price_target.saturating_sub(sqrt_price_curr))
            }
        }
    }

    /// Add fees to the global fee counters
    fn add_fees(&self, order_type: &OrderType, fee: u64) {
        let liquidity_curr = self.get_safe_current_liquidity();
        if liquidity_curr > 0 {
            let fee_global_tao = self.state_ops.get_fee_global_tao();
            let fee_global_alpha = self.state_ops.get_fee_global_alpha();

            match order_type {
                OrderType::Sell => {
                    self.state_ops.set_fee_global_tao(
                        fee_global_tao.saturating_add(fee.safe_div(liquidity_curr)),
                    );
                }
                OrderType::Buy => {
                    self.state_ops.set_fee_global_alpha(
                        fee_global_alpha.saturating_add(fee.safe_div(liquidity_curr)),
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
    fn convert_deltas(self, order_type: &OrderType, delta_in: u64) {
        let mut liquidity_curr: u64 = self.state_ops.get_current_liquidity();
        let mut sqrt_price_curr = self.state_ops.get_alpha_sqrt_price().into();

        // Prevent overflows:
        // If liquidity or delta are too large, reduce their precision and
        // save their factor for final correction. Price can take full U64F64
        // range, and it will not overflow u128 divisions or multiplications.
        let mut liquidity_factor: u64 = 1;
        if liquidity_curr > u32::MAX as u64 {
            liquidity_factor = u32::MAX as u64;
            liquidity_curr = liquidity_curr.safe_div(liquidity_factor);
        }
        let mut delta = delta_in as u64;
        let mut delta_factor: u64 = 1;
        if delta > u32::MAX as u64 {
            delta_factor = u32::MAX as u64;
            delta = delta.safe_div(delta_factor);
        }

        // This product does not overflow because we limit both
        // multipliers by u32::MAX (despite the u64 type)
        let delta_liquidity = delta.saturating_mul(liquidity);

        // This is product of delta_in * liquidity_curr * sqrt_price_curr
        let delta_liquidity_price: u128 =
            Self::mul_u64_u64f64(delta_liquidity, sqrt_price_curr.into());

        if delta_in > 0 {
            match order_type {
                OrderType::Sell => {
                    todo!()
                    // liquidity_curr.saturating_mul(sqrt_price_curr).saturating_mul(delta_in).safe_div(
                    //     liquidity_curr.safe_div(sqrt_price_curr).saturating_add(delta_in)
                    // )
                }
                OrderType::Buy => {
                    todo!()
                    // (self.liquidity_curr / self.sqrt_price_curr * delta_in) / (
                    //     self.liquidity_curr * self.sqrt_price_curr + delta_in
                    // )
                }
            }
        } else {
            0
        }
    }

    /// Multiplies a `u64` by a `U64F64` and returns a `u128` result without overflow.
    pub fn mul_u64_u64f64(a: u64, b: U64F64) -> u128 {
        // Multiply a by integer part of b in integer math.
        // Result doesn't overflow u128 because both multipliers are 64 bit
        let int_b: u64 = b.saturating_to_num::<u64>();
        let high = (a as u128).saturating_mul(int_b as u128);

        // Multiply a by fractional part of b using U64F64
        let frac_b = b.saturating_sub(U64F64::saturating_from_num(int_b));
        let low = U64F64::saturating_from_num(a).saturating_mul(frac_b);

        // The only possible overflow (that is cut off by saturating math)
        // is when a is u64::MAX, int_b is u64::MAX, and frac_b is non-zero,
        // which is negligible error if we saturate and return u128::MAX
        high.saturating_add(low).saturating_to_num::<u128>()
    }

    // def update_reserves(self, zero_for_one, amount_in, amount_out):
    //     """Update token reserves after a swap"""
    //     if zero_for_one:
    //         self.reserves0 = self.reserves0 + amount_in
    //         self.reserves1 = self.reserves1 - amount_out
    //     else:
    //         self.reserves0 = self.reserves0 - amount_out
    //         self.reserves1 = self.reserves1 + amount_in

    // def update_liquidity_at_crossing(self, zero_for_one):
    //     """Update liquidity when crossing a tick"""
    //     if zero_for_one:
    //         k = self.get_tick_index(self.i_curr)
    //         self.liquidity_curr = self.liquidity_curr - self.liquidity_net[k]
    //     else:
    //         k = self.get_tick_index(self.i_curr)
    //         self.liquidity_curr = self.liquidity_curr + self.liquidity_net[k + 1]

    /// Collect fees for a position
    /// Updates the position
    ///
    fn collect_fees(&self, position: &mut Position) -> (u64, u64) {
        // liquidity = self.user_positions[user_idx, 2]
        // fee0 = self.get_fees_in_range(user_idx, 0)
        // fee1 = self.get_fees_in_range(user_idx, 1)

        // fee0 = fee0 - self.user_positions[user_idx, 3]
        // fee1 = fee1 - self.user_positions[user_idx, 4]

        // self.user_positions[user_idx, 3] = fee0
        // self.user_positions[user_idx, 4] = fee1

        // fee0 = liquidity * fee0
        // fee1 = liquidity * fee1

        // return fee0, fee1

        todo!()
    }
}
