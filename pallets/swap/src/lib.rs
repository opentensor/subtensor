#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use core::ops::Neg;

use pallet_subtensor_swap_interface::OrderType;
use safe_math::*;
use sp_arithmetic::helpers_128bit::sqrt;
use substrate_fixed::types::U64F64;
use frame_support::pallet_prelude::*;

use self::tick::{
    MAX_TICK_INDEX, MIN_TICK_INDEX, Tick, TickIndex,
};

pub mod pallet;
mod tick;

type SqrtPrice = U64F64;

/// All tick indexes are offset by TICK_OFFSET for the search and active tick storage needs 
/// so that tick indexes are positive, which simplifies bit logic
pub const TICK_OFFSET: u32 = 887272;

pub enum SwapStepAction {
    Crossing,
    StopOn,
    StopIn,
}

#[derive(Debug, PartialEq)]
pub struct RemoveLiquidityResult {
    tao: u64,
    alpha: u64,
    fee_tao: u64,
    fee_alpha: u64,
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
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct Position {
    pub tick_low: TickIndex,
    pub tick_high: TickIndex,
    pub liquidity: u64,
    pub fees_tao: u64,
    pub fees_alpha: u64,
}

impl Position {
    /// Converts position to token amounts
    ///
    /// returns tuple of (TAO, Alpha)
    ///
    /// Pseudocode:
    ///     if self.sqrt_price_curr < sqrt_pa:
    ///         tao = 0
    ///         alpha = L * (1 / sqrt_pa - 1 / sqrt_pb)
    ///     elif self.sqrt_price_curr > sqrt_pb:
    ///         tao = L * (sqrt_pb - sqrt_pa)
    ///         alpha = 0
    ///     else:
    ///         tao = L * (self.sqrt_price_curr - sqrt_pa)
    ///         alpha = L * (1 / self.sqrt_price_curr - 1 / sqrt_pb)
    ///
    pub fn to_token_amounts(&self, sqrt_price_curr: SqrtPrice) -> Result<(u64, u64), SwapError> {
        let one: U64F64 = U64F64::saturating_from_num(1);

        let sqrt_pa: SqrtPrice =
            self.tick_low.try_to_sqrt_price().map_err(|_| SwapError::InvalidTickRange)?;
        let sqrt_pb: SqrtPrice =
            self.tick_high.try_to_sqrt_price().map_err(|_| SwapError::InvalidTickRange)?;
        let liquidity_fixed: U64F64 = U64F64::saturating_from_num(self.liquidity);

        Ok(if sqrt_price_curr < sqrt_pa {
            (
                0,
                liquidity_fixed
                    .saturating_mul(one.safe_div(sqrt_pa).saturating_sub(one.safe_div(sqrt_pb)))
                    .saturating_to_num::<u64>(),
            )
        } else if sqrt_price_curr > sqrt_pb {
            (
                liquidity_fixed
                    .saturating_mul(sqrt_pb.saturating_sub(sqrt_pa))
                    .saturating_to_num::<u64>(),
                0,
            )
        } else {
            (
                liquidity_fixed
                    .saturating_mul(sqrt_price_curr.saturating_sub(sqrt_pa))
                    .saturating_to_num::<u64>(),
                liquidity_fixed
                    .saturating_mul(
                        one.safe_div(sqrt_price_curr)
                            .saturating_sub(one.safe_div(sqrt_pb)),
                    )
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
    fn set_v3_initialized(&mut self);
    /// Returns u16::MAX normalized fee rate. For example, 0.3% is approximately 196.
    fn get_fee_rate(&self) -> u16;
    /// Minimum liquidity that is safe for rounding and integer math.
    fn get_minimum_liquidity(&self) -> u64;
    fn get_tick_by_index(&self, tick_index: TickIndex) -> Option<Tick>;
    fn insert_tick_by_index(&mut self, tick_index: TickIndex, tick: Tick);
    fn remove_tick_by_index(&mut self, tick_index: TickIndex);
    /// Minimum sqrt price across all active ticks
    fn get_min_sqrt_price(&self) -> SqrtPrice;
    /// Maximum sqrt price across all active ticks
    fn get_max_sqrt_price(&self) -> SqrtPrice;
    fn get_tao_reserve(&self) -> u64;
    fn set_tao_reserve(&mut self, tao: u64) -> u64;
    fn get_alpha_reserve(&self) -> u64;
    fn set_alpha_reserve(&mut self, alpha: u64) -> u64;
    fn get_alpha_sqrt_price(&self) -> SqrtPrice;
    fn set_alpha_sqrt_price(&mut self, sqrt_price: SqrtPrice);

    // Getters/setters for global accrued fees in alpha and tao per subnet
    fn get_fee_global_tao(&self) -> U64F64;
    fn set_fee_global_tao(&mut self, fee: U64F64);
    fn get_fee_global_alpha(&self) -> U64F64;
    fn set_fee_global_alpha(&mut self, fee: U64F64);

    /// Get current tick liquidity
    fn get_current_liquidity(&self) -> u64;
    /// Set current tick liquidity
    fn set_current_liquidity(&mut self, liquidity: u64);

    // User account operations
    fn get_protocol_account_id(&self) -> AccountIdType;
    fn get_max_positions(&self) -> u16;
    fn withdraw_balances(
        &mut self,
        account_id: &AccountIdType,
        tao: u64,
        alpha: u64,
    ) -> Result<(u64, u64), SwapError>;
    fn deposit_balances(&mut self, account_id: &AccountIdType, tao: u64, alpha: u64);
    fn get_position_count(&self, account_id: &AccountIdType) -> u16;
    fn get_position(&self, account_id: &AccountIdType, position_id: u16) -> Option<Position>;
    fn create_position(&mut self, account_id: &AccountIdType, positions: Position) -> u16;
    fn update_position(
        &mut self,
        account_id: &AccountIdType,
        position_id: u16,
        positions: Position,
    );
    fn remove_position(&mut self, account_id: &AccountIdType, position_id: u16);

    // Tick index storage
    // Storage is organized in 3 layers:
    //    Layer 0 consists of one u128 that stores 55 bits. Each bit indicates which layer 1 words are active.
    //    Layer 1 consists of up to 55 u128's that store 6932 bits for the layer 2 words.
    //    Layer 2 consists of up to 6932 u128's that store 887272 bits for active/inactive ticks.
    fn get_layer0_word(&self, word_index: u32) -> u128;
    fn get_layer1_word(&self, word_index: u32) -> u128;
    fn get_layer2_word(&self, word_index: u32) -> u128;
    fn set_layer0_word(&mut self, word_index: u32, word: u128);
    fn set_layer1_word(&mut self, word_index: u32, word: u128);
    fn set_layer2_word(&mut self, word_index: u32, word: u128);
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
    pub(crate) state_ops: Ops,
    phantom_key: PhantomData<AccountIdType>,
}

impl<AccountIdType, Ops> Swap<AccountIdType, Ops>
where
    AccountIdType: Eq,
    Ops: SwapDataOperations<AccountIdType>,
{
    pub fn new(mut ops: Ops) -> Self {
        if !ops.is_v3_initialized() {
            // Initialize the v3:
            // Reserves are re-purposed, nothing to set, just query values for liquidity and price calculation
            let tao_reserve = ops.get_tao_reserve();
            let alpha_reserve = ops.get_alpha_reserve();

            // Set price
            let price: U64F64 = U64F64::saturating_from_num(tao_reserve)
                .safe_div(U64F64::saturating_from_num(alpha_reserve));

            let epsilon: U64F64 = U64F64::saturating_from_num(0.000001);
            ops.set_alpha_sqrt_price(
                price
                    .checked_sqrt(epsilon)
                    .unwrap_or(U64F64::saturating_from_num(0)),
            );

            // Set initial (protocol owned) liquidity and positions
            // Protocol liquidity makes one position from TickIndex::MIN to TickIndex::MAX
            // We are using the sp_arithmetic sqrt here, which works for u128
            let liquidity: u64 = sqrt(tao_reserve as u128 * alpha_reserve as u128) as u64;
            let mut swap = Swap {
                state_ops: ops,
                phantom_key: PhantomData,
            };
            let protocol_account_id = swap.state_ops.get_protocol_account_id();
            let _ = swap.add_liquidity(
                &protocol_account_id,
                TickIndex::MIN,
                TickIndex::MAX,
                liquidity,
                true,
            );

            swap
        } else {
            Swap {
                state_ops: ops,
                phantom_key: PhantomData,
            }
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
    fn add_liquidity_at_index(&mut self, tick_index: TickIndex, liquidity: u64, upper: bool) {
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
    fn remove_liquidity_at_index(&mut self, tick_index: TickIndex, liquidity: u64, upper: bool) {
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

    /// Adds liquidity to the specified price range.
    ///
    /// This function allows an account to provide liquidity to a given range of price ticks.
    /// The amount of liquidity to be added can be determined using the functions
    /// [`get_tao_based_liquidity`] and [`get_alpha_based_liquidity`], which compute the
    /// required liquidity based on TAO and Alpha balances for the current price tick.
    ///
    /// ### Behavior:
    /// - If the `protocol` flag is **not set** (`false`), the function will attempt to
    ///   **withdraw balances** from the account using `state_ops.withdraw_balances()`.
    /// - If the `protocol` flag is **set** (`true`), the liquidity is added without modifying balances.
    ///
    /// ### Parameters:
    /// - `account_id`: A reference to the account that is providing liquidity.
    /// - `tick_low`: The lower bound of the price tick range.
    /// - `tick_high`: The upper bound of the price tick range.
    /// - `liquidity`: The amount of liquidity to be added.
    /// - `protocol`: A boolean flag indicating whether the operation is protocol-managed:
    ///   - `true` -> Do not use this value outside of this implementation. Liquidity is added **without**
    ///               withdrawing balances.
    ///   - `false` -> Use this value for all user transactions. Liquidity is added
    ///               **after withdrawing balances**.
    ///
    /// ### Returns:
    /// - `Ok(u64)`: The final liquidity amount added.
    /// - `Err(SwapError)`: If the operation fails due to insufficient balance, invalid tick range,
    ///   or other swap-related errors.
    ///
    /// ### Errors:
    /// - [`SwapError::InsufficientBalance`] if the account does not have enough balance.
    /// - [`SwapError::InvalidTickRange`] if `tick_low` is greater than or equal to `tick_high`.
    /// - Other [`SwapError`] variants as applicable.
    ///
    pub fn add_liquidity(
        &mut self,
        account_id: &AccountIdType,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
        protocol: bool,
    ) -> Result<(), SwapError> {
        // Check if we can add a position
        let position_count = self.state_ops.get_position_count(account_id);
        let max_positions = self.state_ops.get_max_positions();
        if position_count >= max_positions {
            return Err(SwapError::MaxPositionsExceeded);
        }

        // Add liquidity at tick
        self.add_liquidity_at_index(tick_low, liquidity, false);
        self.add_liquidity_at_index(tick_high, liquidity, true);

        // Update current tick liquidity
        let current_tick_index = self.get_current_tick_index();
        if (tick_low <= current_tick_index) && (current_tick_index <= tick_high) {
            let new_current_liquidity = self
                .state_ops
                .get_current_liquidity()
                .saturating_add(liquidity);
            self.state_ops.set_current_liquidity(new_current_liquidity);
        }

        // New position
        let position = Position {
            tick_low,
            tick_high,
            liquidity,
            fees_tao: 0_u64,
            fees_alpha: 0_u64,
        };

        // If this is a user transaction, withdraw balances and update reserves
        if !protocol {
            let current_price: SqrtPrice = self.state_ops.get_alpha_sqrt_price();
            let (tao, alpha) = position.to_token_amounts(current_price)?;
            self.state_ops.withdraw_balances(account_id, tao, alpha)?;

            // Update reserves
            let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_add(tao);
            self.state_ops.set_tao_reserve(new_tao_reserve);
            let new_alpha_reserve = self.state_ops.get_alpha_reserve().saturating_add(alpha);
            self.state_ops.set_alpha_reserve(new_alpha_reserve);
        }

        // Create a new user position
        self.state_ops.create_position(account_id, position);

        Ok(())
    }

    /// Remove liquidity and credit balances back to account_id
    ///
    /// Account ID and Position ID identify position in the storage map
    ///
    pub fn remove_liquidity(
        &mut self,
        account_id: &AccountIdType,
        position_id: u16,
    ) -> Result<RemoveLiquidityResult, SwapError> {
        // Check if position exists
        if let Some(mut pos) = self.state_ops.get_position(account_id, position_id) {
            // Get current price
            let current_tick_index = self.get_current_tick_index();

            // Collect fees and get tao and alpha amounts
            let (fee_tao, fee_alpha) = self.collect_fees(&mut pos);
            let current_price: SqrtPrice = self.state_ops.get_alpha_sqrt_price();
            let (tao, alpha) = pos.to_token_amounts(current_price)?;

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

            // Deposit balances
            self.state_ops.deposit_balances(account_id, tao, alpha);

            // Update reserves
            let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_sub(tao);
            self.state_ops.set_tao_reserve(new_tao_reserve);
            let new_alpha_reserve = self.state_ops.get_alpha_reserve().saturating_sub(alpha);
            self.state_ops.set_alpha_reserve(new_alpha_reserve);

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
        &mut self,
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

    fn get_current_tick_index(&mut self) -> TickIndex {
        let current_price = self.state_ops.get_alpha_sqrt_price();
        let maybe_current_tick_index = TickIndex::try_from_sqrt_price(current_price);
        if let Ok(index) = maybe_current_tick_index {
            index
        } else {
            // Current price is out of allow the min-max range, and it should be corrected to
            // maintain the range.
            let max_price = TickIndex::MAX.try_to_sqrt_price()
                .unwrap_or(SqrtPrice::saturating_from_num(1000));
            let min_price = TickIndex::MIN.try_to_sqrt_price()
                .unwrap_or(SqrtPrice::saturating_from_num(0.000001));
            if current_price > max_price {
                self.state_ops.set_alpha_sqrt_price(max_price);
                TickIndex::MAX
            } else {
                self.state_ops.set_alpha_sqrt_price(min_price);
                TickIndex::MIN
            }
        }
    }

    /// Process a single step of a swap
    ///
    fn swap_step(
        &mut self,
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

        println!("fee = {:?}", fee);

        self.add_fees(order_type, fee);
        let delta_out = self.convert_deltas(order_type, delta_in);

        self.update_reserves(order_type, delta_in, delta_out);

        // Get current tick
        let TickIndex(current_tick_index) = self.get_current_tick_index();

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
                        .insert_tick_by_index(TickIndex(current_tick_index), tick);
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
                            .insert_tick_by_index(TickIndex(current_tick_index), tick);
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
            OrderType::Buy => TickIndex::MIN.try_to_sqrt_price(),
            OrderType::Sell => TickIndex::MAX.try_to_sqrt_price(),
        })
        .unwrap_or(SqrtPrice::saturating_from_num(0));

        let current_price = self.state_ops.get_alpha_sqrt_price();
        let maybe_current_tick_index = TickIndex::try_from_sqrt_price(current_price);

        if let Ok(current_tick_index) = maybe_current_tick_index {
            match order_type {
                OrderType::Buy => TickIndex::new_unchecked(current_tick_index.get().saturating_add(1)),
                OrderType::Sell => current_tick_index,
            }
            .try_to_sqrt_price()
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
    fn add_fees(&mut self, order_type: &OrderType, fee: u64) {
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
    fn update_reserves(&mut self, order_type: &OrderType, amount_in: u64, amount_out: u64) {
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
    fn update_liquidity_at_crossing(&mut self, order_type: &OrderType) -> Result<(), SwapError> {
        let mut liquidity_curr = self.state_ops.get_current_liquidity();
        let TickIndex(current_tick_index) = self.get_current_tick_index();
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
    fn collect_fees(&mut self, position: &mut Position) -> (u64, u64) {
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
    fn get_fees_in_range(&mut self, position: &mut Position, quote: bool) -> u64 {
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
    fn get_fees_above(&mut self, tick_index: TickIndex, quote: bool) -> U64F64 {
        let maybe_tick_index = tick_index.find_closest_lower_active(&self.state_ops);
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
    fn get_fees_below(&mut self, tick_index: TickIndex, quote: bool) -> U64F64 {
        let maybe_tick_index = tick_index.find_closest_lower_active(&self.state_ops);
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

    /// Active tick operations
    /// 
    /// Data structure: 
    ///    Active ticks are stored in three hash maps, each representing a "Level"
    ///    Level 0 stores one u128 word, where each bit represents one Level 1 word.
    ///    Level 1 words each store also a u128 word, where each bit represents one Level 2 word.
    ///    Level 2 words each store u128 word, where each bit represents a tick.
    /// 
    /// Insertion: 3 reads, 3 writes
    /// Search: 3-5 reads
    /// Deletion: 2 reads, 1-3 writes
    ///

    // Addresses an index as (word, bit)
    fn index_to_address(index: u32) -> (u32, u32) {
        let word: u32 = index.safe_div(128);
        let bit: u32 = index.checked_rem(128).unwrap_or_default();
        (word, bit)
    }

    // Reconstructs an index from address in lower level
    fn address_to_index(word: u32, bit: u32) -> u32 {
        word.saturating_mul(128).saturating_add(bit)
    }

    pub fn insert_active_tick(&mut self, index: i32) {
        // Check the range
        if (index < MIN_TICK_INDEX) || (index > MAX_TICK_INDEX) {
            return;
        }

        // Ticks are between -443636 and 443636, so there is no
        // overflow here. The u32 offset_index is the format we store indexes in the tree
        // to avoid working with the sign bit.
        let offset_index = (index.saturating_add(TICK_OFFSET as i32)) as u32;

        // Calculate index in each layer
        let (layer2_word, layer2_bit) = Self::index_to_address(offset_index);
        let (layer1_word, layer1_bit) = Self::index_to_address(layer2_word);
        let (layer0_word, layer0_bit) = Self::index_to_address(layer1_word);

        // Update layer words
        let mut word0_value = self.state_ops.get_layer0_word(layer0_word);
        let mut word1_value = self.state_ops.get_layer1_word(layer1_word);
        let mut word2_value = self.state_ops.get_layer2_word(layer2_word);        

        let bit: u128 = 1;
        word0_value = word0_value | bit.wrapping_shl(layer0_bit);
        word1_value = word1_value | bit.wrapping_shl(layer1_bit);
        word2_value = word2_value | bit.wrapping_shl(layer2_bit);

        self.state_ops.set_layer0_word(layer0_word, word0_value);
        self.state_ops.set_layer1_word(layer1_word, word1_value);
        self.state_ops.set_layer2_word(layer2_word, word2_value);        
    }

    pub fn remove_active_tick(&mut self, index: i32) {
        // Check the range
        if (index < MIN_TICK_INDEX) || (index > MAX_TICK_INDEX) {
            return;
        }

        // Ticks are between -443636 and 443636, so there is no
        // overflow here. The u32 offset_index is the format we store indexes in the tree
        // to avoid working with the sign bit.
        let offset_index = (index.saturating_add(TICK_OFFSET as i32)) as u32;

        // Calculate index in each layer
        let (layer2_word, layer2_bit) = Self::index_to_address(offset_index);
        let (layer1_word, layer1_bit) = Self::index_to_address(layer2_word);
        let (layer0_word, layer0_bit) = Self::index_to_address(layer1_word);

        // Update layer words
        let mut word0_value = self.state_ops.get_layer0_word(layer0_word);
        let mut word1_value = self.state_ops.get_layer1_word(layer1_word);
        let mut word2_value = self.state_ops.get_layer2_word(layer2_word);        

        // Turn the bit off (& !bit) and save as needed
        let bit: u128 = 1;
        word2_value = word2_value & !bit.wrapping_shl(layer2_bit);
        self.state_ops.set_layer2_word(layer2_word, word2_value);        
        if word2_value == 0 {
            word1_value = word1_value & !bit.wrapping_shl(layer1_bit);
            self.state_ops.set_layer1_word(layer1_word, word1_value);
        }
        if word1_value == 0 {
            word0_value = word0_value & !bit.wrapping_shl(layer0_bit);
            self.state_ops.set_layer0_word(layer0_word, word0_value);
        }
    }

    // Finds the closest active bit and, if active bit exactly matches bit, then the next one
    // Exact match: return Some([next, bit])
    // Non-exact match: return Some([next])
    // No match: return None
    fn find_closest_active_bit_candidates(&self, word: u128, bit: u32, lower: bool) -> Vec<u32> {
        let mut result = vec![];
        let mut mask: u128 = 1_u128.wrapping_shl(bit);
        let mut layer0_active_bit: u32 = bit;
        while mask > 0 {
            if mask & word != 0 {
                result.push(layer0_active_bit);
                if layer0_active_bit != bit {
                    break;
                }
            }
            mask = if lower {
                layer0_active_bit = layer0_active_bit.saturating_sub(1);
                mask.wrapping_shr(1)
            } else {
                layer0_active_bit = layer0_active_bit.saturating_add(1);
                mask.wrapping_shl(1)
            };
        }
        result
    }

    pub fn find_closest_active_tick_index(
        &self,
        index: i32,
        lower: bool,
    ) -> Option<i32>
    {
        // Check the range
        if (index < MIN_TICK_INDEX) || (index > MAX_TICK_INDEX) {
            return None;
        }

        // Ticks are between -443636 and 443636, so there is no
        // overflow here. The u32 offset_index is the format we store indexes in the tree
        // to avoid working with the sign bit.
        let offset_index = (index.saturating_add(TICK_OFFSET as i32)) as u32;
        let mut found = false;
        let mut result: u32 = 0;

        // Calculate index in each layer
        let (layer2_word, layer2_bit) = Self::index_to_address(offset_index);
        let (layer1_word, layer1_bit) = Self::index_to_address(layer2_word);
        let (layer0_word, layer0_bit) = Self::index_to_address(layer1_word);

        // Find the closest active bits in layer 0, then 1, then 2

        ///////////////
        // Level 0
        let word0 = self.state_ops.get_layer0_word(layer0_word);
        let closest_bits_l0 = self.find_closest_active_bit_candidates(word0, layer0_bit, lower);

        closest_bits_l0.iter().for_each(|&closest_bit_l0| {
            ///////////////
            // Level 1
            let word1_index = Self::address_to_index(0, closest_bit_l0);

            // Layer 1 words are different, shift the bit to the word edge
            let start_from_l1_bit = 
                if word1_index < layer1_word {
                    127
                } else if word1_index > layer1_word {
                    0
                } else {
                    layer1_bit
                };
            let word1_value = self.state_ops.get_layer1_word(word1_index);

            let closest_bits_l1 = self.find_closest_active_bit_candidates(word1_value, start_from_l1_bit, lower);
            closest_bits_l1.iter().for_each(|&closest_bit_l1| {
                ///////////////
                // Level 2
                let word2_index = Self::address_to_index(word1_index, closest_bit_l1);

                // Layer 2 words are different, shift the bit to the word edge
                let start_from_l2_bit = 
                    if word2_index < layer2_word {
                        127
                    } else if word2_index > layer2_word {
                        0
                    } else {
                        layer2_bit
                    };

                let word2_value = self.state_ops.get_layer2_word(word2_index);
                let closest_bits_l2 = self.find_closest_active_bit_candidates(word2_value, start_from_l2_bit, lower);

                if closest_bits_l2.len() > 0 {
                    // The active tick is found, restore its full index and return
                    let offset_found_index = Self::address_to_index(word2_index, closest_bits_l2[0]);

                    if lower {
                        if (offset_found_index > result) || (!found) {
                            result = offset_found_index;
                            found = true;
                        }
                    } else {
                        if (offset_found_index < result) || (!found) {
                            result = offset_found_index;
                            found = true;
                        }
                    }
                }
            });
        });

        if found {
            Some((result as i32).saturating_sub(TICK_OFFSET as i32))
        } else {
            None
        }
    }

    pub fn find_closest_lower_active_tick_index(
        &self,
        index: i32,
    ) -> Option<i32>
    {
        self.find_closest_active_tick_index(index, true)
    }

    pub fn find_closest_higher_active_tick_index(
        &self,
        index: i32,
    ) -> Option<i32>
    {
        self.find_closest_active_tick_index(index, false)
    }

    pub fn find_closest_lower_active_tick(&self, index: i32) -> Option<Tick> {
        let maybe_tick_index = self.find_closest_lower_active_tick_index(index);
        if let Some(tick_index) = maybe_tick_index {
            self.state_ops.get_tick_by_index(TickIndex(tick_index))
        } else {
            None
        }
    }

    pub fn find_closest_higher_active_tick(&self, index: i32) -> Option<Tick> {
        let maybe_tick_index = self.find_closest_higher_active_tick_index(index);
        if let Some(tick_index) = maybe_tick_index {
            self.state_ops.get_tick_by_index(TickIndex(tick_index))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SwapError {
    /// The provided amount is insufficient for the swap.
    InsufficientInputAmount,

    /// The provided liquidity is insufficient for the operation.
    InsufficientLiquidity,

    /// The operation would exceed the price limit.
    PriceLimitExceeded,

    /// The caller does not have enough balance for the operation.
    InsufficientBalance,

    /// Attempted to remove liquidity that does not exist.
    LiquidityNotFound,

    /// The provided tick range is invalid.
    InvalidTickRange,

    /// Maximum user positions exceeded
    MaxPositionsExceeded,

    /// Too many swap steps
    TooManySwapSteps,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use sp_arithmetic::helpers_128bit::sqrt;
    use std::collections::HashMap;

    #[derive(Debug, Clone)]
    pub struct MockSwapDataOperations {
        is_initialized: bool,
        fee_rate: u16,
        minimum_liquidity: u64,
        ticks: HashMap<TickIndex, Tick>,
        min_sqrt_price: SqrtPrice,
        max_sqrt_price: SqrtPrice,
        tao_reserve: u64,
        alpha_reserve: u64,
        alpha_sqrt_price: SqrtPrice,
        fee_global_tao: U64F64,
        fee_global_alpha: U64F64,
        current_liquidity: u64,
        max_positions: u16,
        balances: HashMap<u16, (u64, u64)>,
        positions: HashMap<u16, HashMap<u16, Position>>,
        tick_index_l0: HashMap<u32, u128>,
        tick_index_l1: HashMap<u32, u128>,
        tick_index_l2: HashMap<u32, u128>,
    }

    impl MockSwapDataOperations {
        pub fn new() -> Self {
            Self {
                is_initialized: false,
                fee_rate: 196,
                minimum_liquidity: 1000,
                ticks: HashMap::new(),
                min_sqrt_price: SqrtPrice::from_num(0.01),
                max_sqrt_price: SqrtPrice::from_num(10),
                tao_reserve: 0,
                alpha_reserve: 0,
                alpha_sqrt_price: SqrtPrice::from_num(0),
                fee_global_tao: U64F64::from_num(0),
                fee_global_alpha: U64F64::from_num(0),
                current_liquidity: 0,
                max_positions: 100,
                balances: HashMap::new(),
                positions: HashMap::new(),
                tick_index_l0: HashMap::new(),
                tick_index_l1: HashMap::new(),
                tick_index_l2: HashMap::new(),
            }
        }
    }

    impl SwapDataOperations<u16> for MockSwapDataOperations {
        fn is_v3_initialized(&self) -> bool {
            self.is_initialized
        }

        fn set_v3_initialized(&mut self) {
            self.is_initialized = true;
        }

        fn get_fee_rate(&self) -> u16 {
            self.fee_rate
        }

        fn get_minimum_liquidity(&self) -> u64 {
            self.minimum_liquidity
        }

        fn get_tick_by_index(&self, tick_index: TickIndex) -> Option<Tick> {
            self.ticks.get(&tick_index).cloned()
        }

        fn insert_tick_by_index(&mut self, tick_index: TickIndex, tick: Tick) {
            self.ticks.insert(tick_index, tick);
        }

        fn remove_tick_by_index(&mut self, tick_index: TickIndex) {
            self.ticks.remove(&tick_index);
        }

        fn get_min_sqrt_price(&self) -> SqrtPrice {
            self.min_sqrt_price
        }

        fn get_max_sqrt_price(&self) -> SqrtPrice {
            self.max_sqrt_price
        }

        fn get_tao_reserve(&self) -> u64 {
            self.tao_reserve
        }

        fn set_tao_reserve(&mut self, tao: u64) -> u64 {
            self.tao_reserve = tao;
            tao
        }

        fn get_alpha_reserve(&self) -> u64 {
            self.alpha_reserve
        }

        fn set_alpha_reserve(&mut self, alpha: u64) -> u64 {
            self.alpha_reserve = alpha;
            alpha
        }

        fn get_alpha_sqrt_price(&self) -> SqrtPrice {
            self.alpha_sqrt_price
        }

        fn set_alpha_sqrt_price(&mut self, sqrt_price: SqrtPrice) {
            self.alpha_sqrt_price = sqrt_price;
        }

        fn get_fee_global_tao(&self) -> U64F64 {
            self.fee_global_tao
        }

        fn set_fee_global_tao(&mut self, fee: U64F64) {
            self.fee_global_tao = fee;
        }

        fn get_fee_global_alpha(&self) -> U64F64 {
            self.fee_global_alpha
        }

        fn set_fee_global_alpha(&mut self, fee: U64F64) {
            self.fee_global_alpha = fee;
        }

        fn get_current_liquidity(&self) -> u64 {
            self.current_liquidity
        }

        fn set_current_liquidity(&mut self, liquidity: u64) {
            self.current_liquidity = liquidity;
        }

        fn get_max_positions(&self) -> u16 {
            self.max_positions
        }

        fn withdraw_balances(
            &mut self,
            account_id: &u16,
            tao: u64,
            alpha: u64,
        ) -> Result<(u64, u64), SwapError> {
            let (current_tao, current_alpha) =
                self.balances.get(account_id).cloned().unwrap_or((0, 0));

            if (tao > current_tao) || (alpha > current_alpha) {
                return Err(SwapError::InsufficientBalance);
            }

            self.balances
                .insert(*account_id, (current_tao - tao, current_alpha - alpha));

            Ok((tao, alpha))
        }

        fn deposit_balances(&mut self, account_id: &u16, tao: u64, alpha: u64) {
            let (current_tao, current_alpha) =
                self.balances.get(account_id).cloned().unwrap_or((0, 0));
            self.balances.insert(
                account_id.clone(),
                (current_tao + tao, current_alpha + alpha),
            );
        }

        fn get_protocol_account_id(&self) -> u16 {
            0xFFFF
        }

        fn get_position_count(&self, account_id: &u16) -> u16 {
            self.positions.get(account_id).map_or(0, |p| p.len() as u16)
        }

        fn get_position(&self, account_id: &u16, position_id: u16) -> Option<Position> {
            self.positions
                .get(account_id)
                .and_then(|p| p.get(&position_id).cloned())
        }

        fn create_position(&mut self, account_id: &u16, position: Position) -> u16 {
            let entry = self
                .positions
                .entry(account_id.clone())
                .or_insert_with(HashMap::new);

            // Find the next available position ID
            let new_position_id = entry.keys().max().map_or(0, |max_id| max_id + 1);

            entry.insert(new_position_id, position);
            new_position_id
        }

        fn update_position(&mut self, account_id: &u16, position_id: u16, position: Position) {
            if let Some(account_positions) = self.positions.get_mut(account_id) {
                account_positions.insert(position_id, position);
            }
        }

        fn remove_position(&mut self, account_id: &u16, position_id: u16) {
            if let Some(account_positions) = self.positions.get_mut(account_id) {
                account_positions.remove(&position_id);
            }
        }

        fn get_layer0_word(&self, word_index: u32) -> u128 {
            *self.tick_index_l0.get(&word_index).unwrap_or(&0_u128)
        }
        fn get_layer1_word(&self, word_index: u32) -> u128 {
            *self.tick_index_l1.get(&word_index).unwrap_or(&0_u128)
        }
        fn get_layer2_word(&self, word_index: u32) -> u128 {
            *self.tick_index_l2.get(&word_index).unwrap_or(&0_u128)
        }
        fn set_layer0_word(&mut self, word_index: u32, word: u128) {
            self.tick_index_l0.insert(word_index, word);
        }
        fn set_layer1_word(&mut self, word_index: u32, word: u128) {
            self.tick_index_l1.insert(word_index, word);
        }
        fn set_layer2_word(&mut self, word_index: u32, word: u128) {
            self.tick_index_l2.insert(word_index, word);
        }
    }

    #[test]
    fn test_swap_initialization() {
        let tao = 1_000_000_000;
        let alpha = 4_000_000_000;

        let mut mock_ops = MockSwapDataOperations::new();
        mock_ops.set_tao_reserve(tao);
        mock_ops.set_alpha_reserve(alpha);
        let swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        // Active ticks
        let tick_low = swap.state_ops.get_tick_by_index(TickIndex::MIN).unwrap();
        let tick_high = swap.state_ops.get_tick_by_index(TickIndex::MAX).unwrap();
        let liquidity = sqrt(alpha as u128 * tao as u128) as u64;
        let expected_liquidity_net_low: i128 = liquidity as i128;
        let expected_liquidity_gross_low: u64 = liquidity;
        let expected_liquidity_net_high: i128 = (liquidity as i128).neg();
        let expected_liquidity_gross_high: u64 = liquidity;
        assert_eq!(tick_low.liquidity_net, expected_liquidity_net_low,);
        assert_eq!(tick_low.liquidity_gross, expected_liquidity_gross_low,);
        assert_eq!(tick_high.liquidity_net, expected_liquidity_net_high,);
        assert_eq!(tick_high.liquidity_gross, expected_liquidity_gross_high,);

        // Liquidity position at correct ticks
        let account_id = swap.state_ops.get_protocol_account_id();
        assert_eq!(swap.state_ops.get_position_count(&account_id), 1);

        let position = swap.state_ops.get_position(&account_id, 0).unwrap();
        assert_eq!(position.liquidity, liquidity);
        assert_eq!(position.tick_low, TickIndex::MIN);
        assert_eq!(position.tick_high, TickIndex::MAX);
        assert_eq!(position.fees_alpha, 0);
        assert_eq!(position.fees_tao, 0);

        // Current liquidity
        assert_eq!(swap.state_ops.get_current_liquidity(), liquidity);

        // Current price
        let sqrt_price = swap.state_ops.get_alpha_sqrt_price();
        assert_abs_diff_eq!(sqrt_price.to_num::<f64>(), 0.50, epsilon = 0.00001,);
    }

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
            },
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
    fn test_tick_price_sanity_check() {
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        assert!(min_price > 0.);
        assert!(max_price > 0.);
        assert!(max_price > min_price);
        assert!(min_price < 0.000001);
        assert!(max_price > 10.);

        // Roundtrip conversions
        let min_price_sqrt: SqrtPrice = TickIndex::MIN.try_to_sqrt_price().unwrap();
        let min_tick = TickIndex::try_from_sqrt_price(min_price_sqrt).unwrap();
        assert_eq!(min_tick, TickIndex::MIN);

        let max_price_sqrt: SqrtPrice = TickIndex::MAX.try_to_sqrt_price().unwrap();
        let max_tick = TickIndex::try_from_sqrt_price(max_price_sqrt).unwrap();
        assert_eq!(max_tick, TickIndex::MAX);
    }

    // Test adding liquidity on top of the existing protocol liquidity
    #[test]
    fn test_add_liquidity_basic() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;
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
        .iter()
        .for_each(|(price_low, price_high, liquidity, tao, alpha)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Get tick infos and liquidity before adding (to account for protocol liquidity)
            let tick_low_info_before = swap
                .state_ops
                .get_tick_by_index(tick_low)
                .unwrap_or_default();
            let tick_high_info_before = swap
                .state_ops
                .get_tick_by_index(tick_high)
                .unwrap_or_default();
            let liquidity_before = swap.state_ops.get_current_liquidity();

            // Add liquidity
            assert!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false)
                    .is_ok()
            );

            // Check that low and high ticks appear in the state and are properly updated
            let tick_low_info = swap.state_ops.get_tick_by_index(tick_low).unwrap();
            let tick_high_info = swap.state_ops.get_tick_by_index(tick_high).unwrap();
            let expected_liquidity_net_low: i128 = *liquidity as i128;
            let expected_liquidity_gross_low: u64 = *liquidity;
            let expected_liquidity_net_high: i128 = (*liquidity as i128).neg();
            let expected_liquidity_gross_high: u64 = *liquidity;
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

            // Balances are withdrawn
            let (user_tao_after, user_alpha_after) =
                swap.state_ops.balances.get(&account_id).unwrap();
            let tao_withdrawn = user_tao - user_tao_after;
            let alpha_withdrawn = user_alpha - user_alpha_after;
            assert_abs_diff_eq!(tao_withdrawn, *tao, epsilon = *tao / 1000);
            assert_abs_diff_eq!(alpha_withdrawn, *alpha, epsilon = *alpha / 1000);

            // Liquidity position at correct ticks
            assert_eq!(swap.state_ops.get_position_count(&account_id), 1);

            let position = swap.state_ops.get_position(&account_id, 0).unwrap();
            assert_eq!(position.liquidity, *liquidity);
            assert_eq!(position.tick_low, tick_low);
            assert_eq!(position.tick_high, tick_high);
            assert_eq!(position.fees_alpha, 0);
            assert_eq!(position.fees_tao, 0);

            // Current liquidity is updated only when price range includes the current price
            if (*price_high >= current_price) && (*price_low <= current_price) {
                assert_eq!(
                    swap.state_ops.get_current_liquidity(),
                    liquidity_before + *liquidity
                );
            } else {
                assert_eq!(swap.state_ops.get_current_liquidity(), liquidity_before);
            }

            // Reserves are updated
            assert_eq!(
                swap.state_ops.get_tao_reserve(),
                tao_withdrawn + protocol_tao,
            );
            assert_eq!(
                swap.state_ops.get_alpha_reserve(),
                alpha_withdrawn + protocol_alpha,
            );
        });
    }

    #[test]
    fn test_add_liquidity_out_of_bounds() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 2_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;

        [
            // For our tests, we'll construct TickIndex values that are intentionally
            // outside the valid range for testing purposes only
            (TickIndex::new_unchecked(TickIndex::MIN.get() - 1), TickIndex::MAX, 1_000_000_000_u64),
            (TickIndex::MIN, TickIndex::new_unchecked(TickIndex::MAX.get() + 1), 1_000_000_000_u64),
            (TickIndex::new_unchecked(TickIndex::MIN.get() - 1), TickIndex::new_unchecked(TickIndex::MAX.get() + 1), 1_000_000_000_u64),
            (
                TickIndex::new_unchecked(TickIndex::MIN.get() - 100),
                TickIndex::new_unchecked(TickIndex::MAX.get() + 100),
                1_000_000_000_u64,
            ),
        ]
        .iter()
        .for_each(|(tick_low, tick_high, liquidity)| {
            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Add liquidity
            assert_eq!(
                swap.add_liquidity(&account_id, *tick_low, *tick_high, *liquidity, false),
                Err(SwapError::InvalidTickRange),
            );
        });
    }

    #[test]
    fn test_add_liquidity_over_balance() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 1_000_000_000;
        let user_alpha = 1_000_000_000;
        let account_id = 1;

        [
            // Lower than price (not enough alpha)
            (0.1, 0.2, 100_000_000_000_u64),
            // Higher than price (not enough tao)
            (0.3, 0.4, 100_000_000_000_u64),
            // Around the price (not enough both)
            (0.1, 0.4, 100_000_000_000_u64),
        ]
        .iter()
        .for_each(|(price_low, price_high, liquidity)| {
            // Calculate ticks
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Add liquidity
            assert_eq!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false),
                Err(SwapError::InsufficientBalance),
            );
        });
    }

    // Test removing liquidity
    #[test]
    fn test_remove_liquidity_basic() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;
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
        .iter()
        .for_each(|(price_low, price_high, liquidity, tao, alpha)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
            let liquidity_before = swap.state_ops.get_current_liquidity();

            // Add liquidity
            assert!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false)
                    .is_ok()
            );

            // Remove liquidity
            let remove_result = swap.remove_liquidity(&account_id, 0).unwrap();
            assert_abs_diff_eq!(remove_result.tao, *tao, epsilon = *tao / 1000);
            assert_abs_diff_eq!(remove_result.alpha, *alpha, epsilon = *alpha / 1000);
            assert_eq!(remove_result.fee_tao, 0);
            assert_eq!(remove_result.fee_alpha, 0);

            // Balances are returned
            let (user_tao_after, user_alpha_after) =
                swap.state_ops.balances.get(&account_id).unwrap();
            assert_eq!(user_tao, *user_tao_after);
            assert_eq!(user_alpha, *user_alpha_after);

            // Liquidity position is removed
            assert_eq!(swap.state_ops.get_position_count(&account_id), 0);
            assert!(swap.state_ops.get_position(&account_id, 0).is_none());

            // Current liquidity is updated (back where it was)
            assert_eq!(swap.state_ops.get_current_liquidity(), liquidity_before);

            // Reserves are updated (back where they were)
            assert_eq!(swap.state_ops.get_tao_reserve(), protocol_tao,);
            assert_eq!(swap.state_ops.get_alpha_reserve(), protocol_alpha,);
        });
    }

    #[test]
    fn test_remove_liquidity_nonexisting_position() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;
        let user_tao = 100_000_000_000;
        let user_alpha = 100_000_000_000;
        let account_id = 1;
        let min_price = tick_to_price(TickIndex::MIN);
        let max_price = tick_to_price(TickIndex::MAX);
        let max_tick = price_to_tick(max_price);
        assert_eq!(max_tick.get(), TickIndex::MAX.get());

        // Test case is (price_low, price_high, liquidity)
        [
            // Repeat the protocol liquidity at maximum range: Expect all the same values
            (min_price, max_price, 2_000_000_000_u64),
        ]
        .iter()
        .for_each(|(price_low, price_high, liquidity)| {
            // Calculate ticks (assuming tick math is tested separately)
            let tick_low = price_to_tick(*price_low);
            let tick_high = price_to_tick(*price_high);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            mock_ops.deposit_balances(&account_id, user_tao, user_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Add liquidity
            assert!(
                swap.add_liquidity(&account_id, tick_low, tick_high, *liquidity, false)
                    .is_ok()
            );

            // Remove liquidity
            assert_eq!(
                swap.remove_liquidity(&account_id, 1),
                Err(SwapError::LiquidityNotFound),
            );
        });
    }

    // Test swapping against protocol liquidity only
    #[test]
    fn test_swap_basic() {
        let protocol_tao = 1_000_000_000;
        let protocol_alpha = 4_000_000_000;

        // Current price is 0.25
        // Test case is (order_type, liquidity, limit_price, output_amount)
        [
            (OrderType::Buy, 500_000_000u64, 1000.0_f64, 3990_u64),
        ]
        .iter()
        .for_each(|(order_type, liquidity, limit_price, output_amount)| {
            // Consumed liquidity ticks
            let tick_low = TickIndex(MIN_TICK_INDEX);
            let tick_high = TickIndex(MAX_TICK_INDEX);

            // Setup swap
            let mut mock_ops = MockSwapDataOperations::new();
            mock_ops.set_tao_reserve(protocol_tao);
            mock_ops.set_alpha_reserve(protocol_alpha);
            let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

            // Get tick infos before the swap
            let tick_low_info_before = swap
                .state_ops
                .get_tick_by_index(tick_low)
                .unwrap_or_default();
            let tick_high_info_before = swap
                .state_ops
                .get_tick_by_index(tick_high)
                .unwrap_or_default();
            let liquidity_before = swap.state_ops.get_current_liquidity();

            // Swap
            let sqrt_limit_price: SqrtPrice = SqrtPrice::from_num((*limit_price).sqrt());
            let swap_result = swap.swap(order_type, *liquidity, sqrt_limit_price);
            // assert_abs_diff_eq!(
            //     swap_result.unwrap().amount_paid_out,
            //     *output_amount,
            //     epsilon = *output_amount/1000
            // );

            // Check that low and high ticks' fees were updated properly, and liquidity values were not updated
            let tick_low_info = swap.state_ops.get_tick_by_index(tick_low).unwrap();
            let tick_high_info = swap.state_ops.get_tick_by_index(tick_high).unwrap();
            let expected_liquidity_net_low: i128 = tick_low_info_before.liquidity_net;
            let expected_liquidity_gross_low: u64 = tick_low_info_before.liquidity_gross;
            let expected_liquidity_net_high: i128 = tick_high_info_before.liquidity_net;
            let expected_liquidity_gross_high: u64 = tick_high_info_before.liquidity_gross;
            assert_eq!(
                tick_low_info.liquidity_net,
                expected_liquidity_net_low,
            );
            assert_eq!(
                tick_low_info.liquidity_gross,
                expected_liquidity_gross_low,
            );
            assert_eq!(
                tick_high_info.liquidity_net,
                expected_liquidity_net_high,
            );
            assert_eq!(
                tick_high_info.liquidity_gross,
                expected_liquidity_gross_high,
            );

            // Fees should 

            let fee_tao: U64F64 = swap.state_ops.get_fee_global_tao();
            let fee_alpha: U64F64 = swap.state_ops.get_fee_global_alpha();

            println!("fee_tao {:?}", fee_tao);
            println!("fee_alpha {:?}", fee_alpha);


            // Liquidity position should not be updated
            let protocol_id = swap.state_ops.get_protocol_account_id();

            let position = swap.state_ops.get_position(&protocol_id, 0).unwrap();
            assert_eq!(position.liquidity, sqrt(protocol_tao as u128 * protocol_alpha as u128) as u64);
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
        });
    }


    // cargo test --package pallet-subtensor-swap --lib -- tests::test_tick_search_basic --exact --show-output
    #[test]
    fn test_tick_search_basic() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        swap.insert_active_tick(MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX/2).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX - 1).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX+1).unwrap(), MIN_TICK_INDEX);

        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_higher_active_tick_index(MAX_TICK_INDEX), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MAX_TICK_INDEX/2), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MAX_TICK_INDEX - 1), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX+1), None);

        swap.insert_active_tick(MAX_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX).unwrap(), MAX_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX/2).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX - 1).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX + 1).unwrap(), MIN_TICK_INDEX);
    }

    #[test]
    fn test_tick_search_sparse_queries() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        swap.insert_active_tick(MIN_TICK_INDEX + 10);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX + 10).unwrap(), MIN_TICK_INDEX + 10);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX + 11).unwrap(), MIN_TICK_INDEX + 10);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX + 12).unwrap(), MIN_TICK_INDEX + 10);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX), None);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX + 9), None);

        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX + 10).unwrap(), MIN_TICK_INDEX + 10);
        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX + 11), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX + 12), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX).unwrap(), MIN_TICK_INDEX + 10);
        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX + 9).unwrap(), MIN_TICK_INDEX + 10); 
    }

    #[test]
    fn test_tick_search_many_lows() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        for i in 0..1000 {
            swap.insert_active_tick(MIN_TICK_INDEX + i);
        }
        for i in 0..1000 {
            assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX + i).unwrap(), MIN_TICK_INDEX + i);
            assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX + i).unwrap(), MIN_TICK_INDEX + i);
        }
    }

    #[test]
    fn test_tick_search_many_sparse() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let count: i32 = 1000;

        for i in 0..=count {
            swap.insert_active_tick(i * 10);
        }
        for i in 1..count {
            assert_eq!(swap.find_closest_lower_active_tick_index(i * 10).unwrap(), i * 10);
            assert_eq!(swap.find_closest_higher_active_tick_index(i * 10).unwrap(), i * 10);
            for j in 1..=9 {
                assert_eq!(swap.find_closest_lower_active_tick_index(i * 10 - j).unwrap(), (i-1) * 10);
                assert_eq!(swap.find_closest_lower_active_tick_index(i * 10 + j).unwrap(), i * 10);
                assert_eq!(swap.find_closest_higher_active_tick_index(i * 10 - j).unwrap(), i * 10);
                assert_eq!(swap.find_closest_higher_active_tick_index(i * 10 + j).unwrap(), (i+1) * 10);
            }
        }
    }

    #[test]
    fn test_tick_search_many_lows_sparse_reversed() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let count: i32 = 1000;

        for i in (0..=count).rev() {
            swap.insert_active_tick(i * 10);
        }
        for i in 1..count {
            assert_eq!(swap.find_closest_lower_active_tick_index(i * 10).unwrap(), i * 10);
            assert_eq!(swap.find_closest_higher_active_tick_index(i * 10).unwrap(), i * 10);
            for j in 1..=9 {
                assert_eq!(swap.find_closest_lower_active_tick_index(i * 10 - j).unwrap(), (i-1) * 10);
                assert_eq!(swap.find_closest_lower_active_tick_index(i * 10 + j).unwrap(), i * 10);
                assert_eq!(swap.find_closest_higher_active_tick_index(i * 10 - j).unwrap(), i * 10);
                assert_eq!(swap.find_closest_higher_active_tick_index(i * 10 + j).unwrap(), (i+1) * 10);
            }
        }        
    }

    #[test]
    fn test_tick_search_repeated_insertions() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let count: i32 = 1000;

        for _ in 0..10 {
            for i in 0..=count {
                swap.insert_active_tick(i * 10);
            }
            for i in 1..count {
                assert_eq!(swap.find_closest_lower_active_tick_index(i * 10).unwrap(), i * 10);
                assert_eq!(swap.find_closest_higher_active_tick_index(i * 10).unwrap(), i * 10);
                for j in 1..=9 {
                    assert_eq!(swap.find_closest_lower_active_tick_index(i * 10 - j).unwrap(), (i-1) * 10);
                    assert_eq!(swap.find_closest_lower_active_tick_index(i * 10 + j).unwrap(), i * 10);
                    assert_eq!(swap.find_closest_higher_active_tick_index(i * 10 - j).unwrap(), i * 10);
                    assert_eq!(swap.find_closest_higher_active_tick_index(i * 10 + j).unwrap(), (i+1) * 10);
                }
            }
        }
    }

    #[test]
    fn test_tick_search_full_range() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let step= 1019;
        let count = (MAX_TICK_INDEX - MIN_TICK_INDEX) / step;

        for i in 0..=count {
            let index = MIN_TICK_INDEX + i * step;
            swap.insert_active_tick(index);
        }
        for i in 1..count {
            let index = MIN_TICK_INDEX + i * step;

            assert_eq!(swap.find_closest_lower_active_tick_index(index - step).unwrap(), index - step);
            assert_eq!(swap.find_closest_lower_active_tick_index(index).unwrap(), index);
            assert_eq!(swap.find_closest_lower_active_tick_index(index + step - 1).unwrap(), index);
            assert_eq!(swap.find_closest_lower_active_tick_index(index + step/2).unwrap(), index);

            assert_eq!(swap.find_closest_higher_active_tick_index(index).unwrap(), index);
            assert_eq!(swap.find_closest_higher_active_tick_index(index + step).unwrap(), index + step);
            assert_eq!(swap.find_closest_higher_active_tick_index(index + step/2).unwrap(), index + step);
            assert_eq!(swap.find_closest_higher_active_tick_index(index + step - 1).unwrap(), index + step);
            for j in 1..=9 {
                assert_eq!(swap.find_closest_lower_active_tick_index(index - j).unwrap(), index - step);
                assert_eq!(swap.find_closest_lower_active_tick_index(index + j).unwrap(), index);
                assert_eq!(swap.find_closest_higher_active_tick_index(index - j).unwrap(), index);
                assert_eq!(swap.find_closest_higher_active_tick_index(index + j).unwrap(), index + step);
            }
        }
    }    

    #[test]
    fn test_tick_remove_basic() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);

        swap.insert_active_tick(MIN_TICK_INDEX);
        swap.insert_active_tick(MAX_TICK_INDEX);
        swap.remove_active_tick(MAX_TICK_INDEX);

        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX/2).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MAX_TICK_INDEX - 1).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_lower_active_tick_index(MIN_TICK_INDEX+1).unwrap(), MIN_TICK_INDEX);

        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX).unwrap(), MIN_TICK_INDEX);
        assert_eq!(swap.find_closest_higher_active_tick_index(MAX_TICK_INDEX), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MAX_TICK_INDEX/2), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MAX_TICK_INDEX - 1), None);
        assert_eq!(swap.find_closest_higher_active_tick_index(MIN_TICK_INDEX+1), None);
    }    

    #[test]
    fn test_tick_remove_full_range() {
        let mock_ops = MockSwapDataOperations::new();
        let mut swap = Swap::<u16, MockSwapDataOperations>::new(mock_ops);
        let step= 1019;
        let count = (MAX_TICK_INDEX - MIN_TICK_INDEX) / step;
        let remove_frequency = 5; // Remove every 5th tick

        // Insert ticks
        for i in 0..=count {
            let index = MIN_TICK_INDEX + i * step;
            swap.insert_active_tick(index);
        }

        // Remove some ticks
        for i in 1..count {
            if i % remove_frequency == 0 {
                let index = MIN_TICK_INDEX + i * step;
                swap.remove_active_tick(index);
            }
        }

        // Verify
        for i in 1..count {
            let index = MIN_TICK_INDEX + i * step;

            if i % remove_frequency == 0 {
                let lower = swap.find_closest_lower_active_tick_index(index);
                let higher = swap.find_closest_higher_active_tick_index(index);
                assert!(lower != Some(index));
                assert!(higher != Some(index));
            } else {
                assert_eq!(swap.find_closest_lower_active_tick_index(index).unwrap(), index);
                assert_eq!(swap.find_closest_higher_active_tick_index(index).unwrap(), index);
            }
        }
    }
}
