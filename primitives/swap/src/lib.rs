use safe_math::*;

/// The width of a single price tick. Expressed in rao units.
pub const TICK_SPACING: u64 = 10_000;

type SqrtPrice = U64F64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
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
    /// Converts tick index into SQRT of price
    /// 
    /// python: (1 + self.tick_spacing) ** (i / 2)
    /// 
    pub fn tick_index_to_sqrt_price(tick_idx: u64) -> SqrtPrice {
        // TODO: implement
    }

    /// Converts SQRT price to tick index
    /// 
    /// python: math.floor(math.log(sqrt_p) / math.log(1 + self.tick_spacing)) * 2
    /// 
    pub fn sqrt_price_to_tick_index(sqrt_price: SqrtPrice) -> u64 {
        // TODO: implement
    }

    /// Converts position to quote and base token amounts
    /// 
    /// returns tuple of (TAO, Alpha)
    /// 
    pub fn to_token_amounts(self, current_tick: u64) -> (u64, u64) {
        let one = 1.into();

        let sqrt_price_curr = self.tick_index_to_sqrt_price(current_tick);
        let sqrt_pa = self.tick_index_to_sqrt_price(self.tick_low);
        let sqrt_pb = self.tick_index_to_sqrt_price(self.tick_high);

        if sqrt_price_curr < sqrt_pa {
            (
                liquidity.saturating_mul(one.safe_div(sqrt_pa).saturating_sub(one.safe_div(sqrt_pb))),
                0
            )
        } else if sqrt_price_curr > sqrt_pb {
            (
                0,
                liquidity.saturating_mul(sqrt_pb.saturating_sub(sqrt_pa))
            )
        } else {
            (
                liquidity.saturating_mul(one.save_div(sqrt_price_curr).saturating_sub(one.safe_div(sqrt_pb))),
                liquidity.saturating_mul(sqrt_price_curr.saturating_sub(sqrt_pa))
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
pub trait SwapDataOperations {
    /// Tells if v3 swap is initialized in the state. v2 only provides base and quote 
    /// reserves, while v3 also stores ticks and positions, which need to be initialized
    /// at the first pool creation.
    fn is_v3_initialized() -> bool;
    /// Returns u16::MAX normalized fee rate. For example, 0.3% is approximately 196.
    fn get_fee_rate() -> u16;
    fn get_tick_by_index(tick_index: u64) -> Option<Tick>;
    fn insert_tick_by_index(tick_index: u64, tick: Tick);
    fn get_tao_reserve() -> u64;
    fn set_tao_reserve() -> u64;
    fn get_alpha_reserve() -> u64;
    fn set_alpha_reserve() -> u64;
    fn get_alpha_sqrt_price() -> u64;
    fn set_alpha_sqrt_price() -> u64;

    /// Get current tick liquidity
    fn get_current_liquidity() -> u64;
    /// Set current tick liquidity
    fn set_current_liquidity(liquidity: u64);

    fn withdraw_balances(tao: u64, alpha: u64) -> (u64, u64);
    fn deposit_balances(tao: u64, alpha: u64);
}

/// All main swapping logic abstracted from Runtime implementation is concentrated 
/// in this struct
/// 
#[derive(Debug)]
pub struct Swap<Ops>
where
    Ops: SwapDataOperations,
{
    state_ops: Ops,
}

impl<Ops> Swap<Ops>
where
    Ops: SwapDataOperations,
{
    pub fn new(ops: Ops) -> Self {
        if !ops.is_v3_initialized() {
            // TODO: Initialize the v3
            // Set price, set initial (protocol owned) liquidity and positions, etc.
        }

        Swap {
            state_ops: ops,
        }
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
        }

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
        }

        // TODO: Review why Python code uses this code to find index for the new ticks: 
        // self.get_tick_index(user_position[0]) + 1
        self.state_ops.insert_tick_by_index(tick_index, new_tick);
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
        tick_low: u64,
        tick_high: u64,
        liquidity: u64
    ) -> u64 {
        self.add_liquidity_at_index(tick_low, liquidity, false);
        self.add_liquidity_at_index(tick_high, liquidity, true);

        // Update current tick and liquidity
        // TODO: Review why python uses this code to get the new tick index:
        // k = self.get_tick_index(i)
        let current_price = self.state_ops.get_alpha_sqrt_price();
        let current_tick_index = Position::sqrt_price_to_tick_index(current_price);

        // Update current tick liquidity
        if (tick_low <= current_tick_index) && (current_tick_index <= tick_high) {
            let new_current_liquidity = self.state_ops.get_current_liquidity()
                .saturating_add(liquidity);
            self.state_ops.set_current_liquidity(new_current_liquidity);
        }
        
        # Update positions
        if len(self.user_positions) == 0:
            self.user_positions = np.array([np.append(user_position, [0, 0])])
        else:
            self.user_positions = np.vstack([self.user_positions, np.append(user_position, [0, 0])])
        

        // Update reserves
        let position = Position {
            tick_low,
            tick_high,
            liquidity,
            fees_tao: 0_u64,
            fees_alpha: 0_u64,
        }
        let (tao, alpha) = position.to_token_amounts(current_tick_index);
        let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_add(tao);
        self.state_ops.set_tao_reserve(new_tao_reserve);
        let new_alpha_reserve = self.get_alpha_reserve().saturating_add(alpha);
        self.state_ops.set_alpha_reserve(new_alpha);        
    }

    pub fn remove_liquidity(
        &self,
        position: &Position,
    ) {
        // TODO
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
    ) -> (u64, u64) {
        // TODO
    }
}