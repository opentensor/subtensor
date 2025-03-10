
/// The width of a single price tick. Expressed in rao units.
pub const TICK_SPACING: u64 = 10_000;

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
    fn get_tick_by_index(tick_index: u64);
    fn insert_tick_by_index(tick_index: u64);
    fn get_tao_reserve() -> u64;
    fn set_tao_reserve() -> u64;
    fn get_alpha_reserve() -> u64;
    fn set_alpha_reserve() -> u64;
    fn get_alpha_sqrt_price() -> u64;
    fn set_alpha_sqrt_price() -> u64;

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

    /// Add 
    pub fn add_liquidity(
        tick_low: u64,
        tick_high: u64,
        liquidity: u64
    ) -> u64 {
        // TODO
    }

    pub fn remove_liquidity() {
        // TODO
    }

    pub fn swap() {
        // TODO
    }
}