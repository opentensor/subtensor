use core::marker::PhantomData;

use safe_math::*;
use substrate_fixed::types::U64F64;

/// The width of a single price tick. Expressed in rao units.
pub const TICK_SPACING: u64 = 10_000;
pub const MAX_TICK: i32 = 887272;
pub const MIN_SQRT_RATIO: u128 = 4295128739;
pub const MAX_SQRT_RATIO: u128 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFF;

type SqrtPrice = U64F64;

/// Calculates sqrt(1.0001^tick) * 2^96
pub fn get_sqrt_ratio_at_tick(tick: i32) -> u128 {
    // Ensure tick is within bounds
    let abs_tick = if tick < 0 { -tick } else { tick } as u32;
    assert!(abs_tick <= MAX_TICK as u32, "Tick outside of allowed range");

    // Initial ratio value
    let mut ratio: u128 = if abs_tick & 0x1 != 0 {
        0xfffcb933bd6fad37 << 64 | 0xaa2d162d1a594001
    } else {
        0x1 << 96
    };

    // Safe multiply and shift function
    fn mul_shr(a: u128, b_hi: u64, b_lo: u64, shift: u8) -> u128 {
        // Split a into high and low parts
        let a_hi = a >> 64;
        let a_lo = a & 0xFFFFFFFFFFFFFFFF;

        // Multiply parts separately to avoid overflow
        // (a_hi * 2^64 + a_lo) * (b_hi * 2^64 + b_lo)
        // Only keep the high part since we're shifting by at least 64 anyway
        let hi_hi = a_hi.checked_mul(b_hi as u128).unwrap_or(0);

        // These parts will be shifted by 64
        let hi_lo = a_hi.checked_mul(b_lo as u128).unwrap_or(0);
        let lo_hi = a_lo.checked_mul(b_hi as u128).unwrap_or(0);

        // Combine parts, adding with overflow checking
        let mut result = hi_hi << 64;
        result = result.checked_add(hi_lo).unwrap_or(result);
        result = result.checked_add(lo_hi).unwrap_or(result);

        // Final shift (already accounted for 64 bits)
        if shift <= 64 {
            result >> (shift - 64)
        } else if shift < 128 {
            result >> (shift - 64)
        } else {
            0
        }
    }

    // Multiply ratio by appropriate factor based on each bit
    // Break each constant into high and low 64-bit parts
    if abs_tick & 0x2 != 0 {
        ratio = mul_shr(ratio, 0xfff97272373d4132, 0x59a46990580e213a, 128);
    }
    if abs_tick & 0x4 != 0 {
        ratio = mul_shr(ratio, 0xfff2e50f5f656932, 0xef12357cf3c7fdcc, 128);
    }
    if abs_tick & 0x8 != 0 {
        ratio = mul_shr(ratio, 0xffe5caca7e10e4e6, 0x1c3624eaa0941cd0, 128);
    }
    if abs_tick & 0x10 != 0 {
        ratio = mul_shr(ratio, 0xffcb9843d60f6159, 0xc9db58835c926644, 128);
    }
    if abs_tick & 0x20 != 0 {
        ratio = mul_shr(ratio, 0xff973b41fa98c081, 0x472e6896dfb254c0, 128);
    }
    if abs_tick & 0x40 != 0 {
        ratio = mul_shr(ratio, 0xff2ea16466c96a38, 0x43ec78b326b52861, 128);
    }
    if abs_tick & 0x80 != 0 {
        ratio = mul_shr(ratio, 0xfe5dee046a99a2a8, 0x11c461f1969c3053, 128);
    }
    if abs_tick & 0x100 != 0 {
        ratio = mul_shr(ratio, 0xfcbe86c7900a88ae, 0xdcffc83b479aa3a4, 128);
    }
    if abs_tick & 0x200 != 0 {
        ratio = mul_shr(ratio, 0xf987a7253ac41317, 0x6f2b074cf7815e54, 128);
    }
    if abs_tick & 0x400 != 0 {
        ratio = mul_shr(ratio, 0xf3392b0822b70005, 0x940c7a398e4b70f3, 128);
    }
    if abs_tick & 0x800 != 0 {
        ratio = mul_shr(ratio, 0xe7159475a2c29b74, 0x43b29c7fa6e889d9, 128);
    }
    if abs_tick & 0x1000 != 0 {
        ratio = mul_shr(ratio, 0xd097f3bdfd2022b8, 0x845ad8f792aa5825, 128);
    }
    if abs_tick & 0x2000 != 0 {
        ratio = mul_shr(ratio, 0xa9f746462d870fdf, 0x8a65dc1f90e061e5, 128);
    }
    if abs_tick & 0x4000 != 0 {
        ratio = mul_shr(ratio, 0x70d869a156d2a1b8, 0x90bb3df62baf32f7, 128);
    }
    if abs_tick & 0x8000 != 0 {
        ratio = mul_shr(ratio, 0x31be135f97d08fd9, 0x81231505542fcfa6, 128);
    }
    if abs_tick & 0x10000 != 0 {
        ratio = mul_shr(ratio, 0x9aa508b5b7a84e1c, 0x677de54f3e99bc9, 128);
    }
    if abs_tick & 0x20000 != 0 {
        ratio = mul_shr(ratio, 0x5d6af8dedb811966, 0x99c329225ee604, 128);
    }
    if abs_tick & 0x40000 != 0 {
        ratio = mul_shr(ratio, 0x2216e584f5fa1ea9, 0x26041bedfe98, 128);
    }
    if abs_tick & 0x80000 != 0 {
        ratio = mul_shr(ratio, 0x48a170391f7dc424, 0x44e8fa2, 128);
    }

    // Invert ratio if tick is positive
    if tick > 0 {
        if ratio == 0 {
            return u128::MAX;
        }
        ratio = u128::MAX / ratio;
    }

    // Convert from Q128.128 to Q64.96
    let sqrt_price_x96 = if ratio >= (1 << 32) {
        let result = ratio >> 32;
        if ratio % (1 << 32) != 0 {
            result + 1
        } else {
            result
        }
    } else {
        1
    };

    sqrt_price_x96
}

/// A simpler implementation for get_tick_at_sqrt_ratio that avoids complex bit operations
/// This may not match Uniswap's exact implementation but should be accurate enough
/// for roundtrip conversions
/// Calculates the greatest tick value such that get_sqrt_ratio_at_tick(tick) <= sqrt_ratio_x96
pub fn get_tick_at_sqrt_ratio(sqrt_ratio_x96: u128) -> i32 {
    // Special case for tick 0 - which is exactly 1.0 * 2^96
    if sqrt_ratio_x96 == (1 << 96) {
        return 0;
    }

    // Ensure ratio is within bounds - relax the upper bound slightly
    // This solves the "Ratio outside of allowed range" error
    assert!(
        sqrt_ratio_x96 >= MIN_SQRT_RATIO && sqrt_ratio_x96 <= MAX_SQRT_RATIO,
        "Ratio outside of allowed range"
    );

    // Binary search to find the closest tick
    let mut lower = -MAX_TICK;
    let mut upper = MAX_TICK;

    while lower < upper {
        let mid = (lower + upper) / 2;

        let price = get_sqrt_ratio_at_tick(mid);

        if price <= sqrt_ratio_x96 {
            lower = mid + 1;
        } else {
            upper = mid;
        }
    }

    // We need the largest tick such that get_sqrt_ratio_at_tick(tick) <= sqrt_ratio_x96
    let tick = lower - 1;

    // Double-check our result is correct
    let lower_price = get_sqrt_ratio_at_tick(tick);
    let upper_price = get_sqrt_ratio_at_tick(tick + 1);

    if lower_price <= sqrt_ratio_x96 && upper_price > sqrt_ratio_x96 {
        tick
    } else if sqrt_ratio_x96.abs_diff(1 << 96) < 1000 {
        // Very close to tick 0
        0
    } else {
        // Safeguard
        tick
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
}

struct RemoveLiquidityResult {
    tao: u64,
    alpha: u64,
    fee_tao: u64,
    fee_alpha: u64,
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
struct Position {
    tick_low: u64,
    tick_high: u64,
    liquidity: u64,
    fees_tao: u64,
    fees_alpha: u64,
}

impl Position {
    /// Converts tick index into SQRT of price
    pub fn tick_index_to_sqrt_price(tick_idx: u64) -> SqrtPrice {
        // Special case for tick 0
        if tick_idx == 0 {
            return SqrtPrice::from_num(1.0);
        }

        // Convert u64 tick to i32 and handle potential overflows
        let tick_i32 = if tick_idx > i32::MAX as u64 {
            i32::MAX
        } else {
            tick_idx as i32
        };

        // Convert the U128 result to your SqrtPrice type
        let sqrt_price_x96 = get_sqrt_ratio_at_tick(tick_i32);
        SqrtPrice::from_bits(sqrt_price_x96 as _)
    }

    /// Converts SQRT price to tick index
    pub fn sqrt_price_to_tick_index(sqrt_price: SqrtPrice) -> u64 {
        // Special case for price exactly equal to 1.0
        if sqrt_price == SqrtPrice::from_num(1.0) {
            return 0;
        }

        // Special case for prices very close to 1.0
        if (sqrt_price - SqrtPrice::from_num(1.0)) < SqrtPrice::from_num(0.0000001) {
            return 0;
        }

        // Convert your SqrtPrice type to U128
        let sqrt_price_x96 = sqrt_price.to_bits() as u128;

        // Get the tick and convert back to u64
        let tick = get_tick_at_sqrt_ratio(sqrt_price_x96);

        // Handle negative ticks (if your system doesn't support negative ticks)
        if tick < 0 { 0 } else { tick as u64 }
    }

    fn tick_spacing_tao() -> SqrtPrice {
        // SqrtPrice::from_num(TICK_SPACING)
        //     .saturating_div(SqrtPrice::from_num(1e9))
        //     .saturating_add(SqrtPrice::from_num(1.0))
        SqrtPrice::from_num(1.0001)
    }

    /// Converts position to token amounts
    ///
    /// returns tuple of (TAO, Alpha)
    ///
    pub fn to_token_amounts(self, current_tick: u64) -> (u64, u64) {
        // let one = 1.into();

        // let sqrt_price_curr = Self::tick_index_to_sqrt_price(current_tick);
        // let sqrt_pa = Self::tick_index_to_sqrt_price(self.tick_low);
        // let sqrt_pb = Self::tick_index_to_sqrt_price(self.tick_high);

        // if sqrt_price_curr < sqrt_pa {
        //     (
        //         liquidity
        //             .saturating_mul(one.safe_div(sqrt_pa).saturating_sub(one.safe_div(sqrt_pb))),
        //         0,
        //     )
        // } else if sqrt_price_curr > sqrt_pb {
        //     (0, liquidity.saturating_mul(sqrt_pb.saturating_sub(sqrt_pa)))
        // } else {
        //     (
        //         liquidity.saturating_mul(
        //             one.save_div(sqrt_price_curr)
        //                 .saturating_sub(one.safe_div(sqrt_pb)),
        //         ),
        //         liquidity.saturating_mul(sqrt_price_curr.saturating_sub(sqrt_pa)),
        //     )
        // }
        todo!()
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
    fn get_tick_by_index(tick_index: u64) -> Option<Tick>;
    fn insert_tick_by_index(tick_index: u64, tick: Tick);
    fn remove_tick_by_index(tick_index: u64);
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
        // if !ops.is_v3_initialized() {
        //     // TODO: Initialize the v3
        //     // Set price, set initial (protocol owned) liquidity and positions, etc.
        // }

        // Swap { state_ops: ops }
        todo!()
    }

    /// Auxiliary method to calculate Alpha amount to match given TAO
    /// amount at the current price for liquidity.
    ///
    /// Returns (Alpha, Liquidity) tuple
    ///
    pub fn get_tao_based_liquidity(&self, tao: u64) -> (u64, u64) {
        // let current_price = self.state_ops.get_alpha_sqrt_price();
        todo!()
    }

    /// Auxiliary method to calculate TAO amount to match given Alpha
    /// amount at the current price for liquidity.
    ///
    /// Returns (TAO, Liquidity) tuple
    ///
    pub fn get_alpha_based_liquidity(&self, alpha: u64) -> (u64, u64) {
        // let current_price = self.state_ops.get_alpha_sqrt_price();

        todo!()
    }

    /// Add liquidity at tick index. Creates new tick if it doesn't exist
    ///
    fn add_liquidity_at_index(tick_index: u64, liquidity: u64, upper: bool) {
        // Calculate net liquidity addition
        // let net_addition = if upper {
        //     (liquidity as i128).neg()
        // } else {
        //     liquidity as i128
        // }

        // // Find tick by index
        // let new_tick = if let Some(tick) = self.state_ops.get_tick_by_index(tick_index) {
        //     tick.liquidity_net = tick.liquidity_net.saturating_add(net_addition);
        //     tick.liquidity_gross = tick.liquidity_gross.saturating_add(liquidity);
        // } else {
        //     // Create a new tick
        //     Tick {
        //         liquidity_net: net_addition,
        //         liquidity_gross: liquidity,
        //         fees_out_tao: 0_u64,
        //         fees_out_alpha: 0_u64,
        //     }
        // }

        // // TODO: Review why Python code uses this code to find index for the new ticks:
        // // self.get_tick_index(user_position[0]) + 1
        // self.state_ops.insert_tick_by_index(tick_index, new_tick);
    }

    /// Remove liquidity at tick index.
    ///
    fn remove_liquidity_at_index(tick_index: u64, liquidity: u64, upper: bool) {
        // // Calculate net liquidity addition
        // let net_reduction = if upper {
        //     (liquidity as i128).neg()
        // } else {
        //     liquidity as i128
        // }

        // // Find tick by index
        // let new_tick = if let Some(tick) = self.state_ops.get_tick_by_index(tick_index) {
        //     tick.liquidity_net = tick.liquidity_net.saturating_sub(net_reduction);
        //     tick.liquidity_gross = tick.liquidity_gross.saturating_sub(liquidity);
        // }

        // // If any liquidity is left at the tick, update it, otherwise remove
        // if tick.liquidity_gross == 0 {
        //     self.state_ops.remove_tick(tick_index);
        // } else {
        //     self.state_ops.insert_tick_by_index(tick_index, new_tick);
        // }
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
        // // Check if we can add a position
        // let position_count = self.state_ops.get_position_count(account_id);
        // let max_positions = get_max_positions();
        // if position_count >= max_positions {
        //     return Err(());
        // }

        // // Add liquidity at tick
        // self.add_liquidity_at_index(tick_low, liquidity, false);
        // self.add_liquidity_at_index(tick_high, liquidity, true);

        // // Update current tick and liquidity
        // // TODO: Review why python uses this code to get the new tick index:
        // // k = self.get_tick_index(i)
        // let current_price = self.state_ops.get_alpha_sqrt_price();
        // let current_tick_index = Position::sqrt_price_to_tick_index(current_price);

        // // Update current tick liquidity
        // if (tick_low <= current_tick_index) && (current_tick_index <= tick_high) {
        //     let new_current_liquidity = self.state_ops.get_current_liquidity()
        //         .saturating_add(liquidity);
        //     self.state_ops.set_current_liquidity(new_current_liquidity);
        // }

        // // Update balances
        // let position = Position {
        //     tick_low,
        //     tick_high,
        //     liquidity,
        //     fees_tao: 0_u64,
        //     fees_alpha: 0_u64,
        // }
        // let (tao, alpha) = position.to_token_amounts(current_tick_index);
        // self.state_ops.withdraw_balances(account_id, tao, alpha);

        // // Update reserves
        // let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_add(tao);
        // self.state_ops.set_tao_reserve(new_tao_reserve);
        // let new_alpha_reserve = self.get_alpha_reserve().saturating_add(alpha);
        // self.state_ops.set_alpha_reserve(new_alpha);

        // // Update user positions
        // let position_id = position_count.saturating_add(1);
        // self.state_ops.set_position(account_id, position_id, position);

        // Ok(liquidity)
        todo!()
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
        // if let Some(pos) = self.state_ops.get_position(account_id, position_id) {
        //     // Get current price
        //     let current_price = self.state_ops.get_alpha_sqrt_price();
        //     let current_tick_index = Position::sqrt_price_to_tick_index(current_price);

        //     // Collect fees and get tao and alpha amounts
        //     let (fee_tao, fee_alpha) = self.collect_fees(pos);
        //     let (tao, alpha) = pos.to_token_amounts(current_tick_index);

        //     // Update liquidity at position ticks
        //     self.remove_liquidity_at_index(pos.tick_low, pos.liquidity, false);
        //     self.remove_liquidity_at_index(pos.tick_high, pos.liquidity, true);

        //     // Update current tick liquidity
        //     if (pos.tick_low <= current_tick_index) && (current_tick_index <= pos.tick_high) {
        //         let new_current_liquidity = self.state_ops.get_current_liquidity()
        //             .saturating_sub(liquidity);
        //         self.state_ops.set_current_liquidity(new_current_liquidity);
        //     }

        //     // Remove user position
        //     self.state_ops.remove_position(account_id, position_id);

        //     // Update current price (why?)
        //     // i = self.sqrt_price_to_tick(self.sqrt_price_curr)
        //     // k = self.get_tick_index(i)
        //     // self.i_curr = self.active_ticks[k]
        //     todo!();

        //     // Update reserves
        //     let new_tao_reserve = self.state_ops.get_tao_reserve().saturating_sub(tao);
        //     self.state_ops.set_tao_reserve(new_tao_reserve);
        //     let new_alpha_reserve = self.get_alpha_reserve().saturating_sub(alpha);
        //     self.state_ops.set_alpha_reserve(new_alpha);

        //     // Return Ok result
        //     Ok(RemoveLiquidityResult{
        //         tao,
        //         alpha,
        //         fee_tao,
        //         fee_alpha,
        //     })
        // } else {
        //     // Position doesn't exist
        //     Err(())
        // }
        todo!()
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
        todo!()
    }

    /// Updates position
    fn collect_fees(&self, position: &mut Position) -> (u64, u64) {
        // """Collect fees for a position"""
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_index_to_sqrt_price() {
        // At tick index 0, the sqrt price should be 1.0
        let sqrt_price = Position::tick_index_to_sqrt_price(0);
        assert_eq!(sqrt_price, SqrtPrice::from_num(1.0));

        let sqrt_price = Position::tick_index_to_sqrt_price(2);
        let tick_spacing_tao = Position::tick_spacing_tao();
        assert_eq!(sqrt_price, tick_spacing_tao);

        let sqrt_price = Position::tick_index_to_sqrt_price(4);
        // Calculate the expected value: (1 + TICK_SPACING/1e9 + 1.0)^2
        let expected = tick_spacing_tao * tick_spacing_tao;
        assert_eq!(sqrt_price, expected);

        // Test with tick index 10
        let sqrt_price = Position::tick_index_to_sqrt_price(10);
        // Calculate the expected value: (1 + TICK_SPACING/1e9 + 1.0)^5
        let expected_sqrt_price_10 = tick_spacing_tao.checked_pow(5).unwrap();
        assert_eq!(sqrt_price, expected_sqrt_price_10);
    }

    #[test]
    fn test_sqrt_price_to_tick_index() {
        let tick_spacing_tao = Position::tick_spacing_tao();

        let tick_index = Position::sqrt_price_to_tick_index(SqrtPrice::from_num(1.0));
        assert_eq!(tick_index, 0);

        // Test with sqrt price equal to tick_spacing_tao (should be tick index 2)
        let tick_index = Position::sqrt_price_to_tick_index(tick_spacing_tao);
        assert_eq!(tick_index, 2);

        // Test with sqrt price equal to tick_spacing_tao^2 (should be tick index 4)
        let sqrt_price = tick_spacing_tao * tick_spacing_tao;
        let tick_index = Position::sqrt_price_to_tick_index(sqrt_price);
        assert_eq!(tick_index, 4);

        // Test with sqrt price equal to tick_spacing_tao^5 (should be tick index 10)
        let sqrt_price = tick_spacing_tao.checked_pow(5).unwrap();
        let tick_index = Position::sqrt_price_to_tick_index(sqrt_price);
        assert_eq!(tick_index, 10,);
    }

    #[test]
    fn test_roundtrip_tick_index_sqrt_price() {
        for tick_index in [0, 2, 4, 10, 100, 1000].iter() {
            let sqrt_price = Position::tick_index_to_sqrt_price(*tick_index);
            let round_trip_tick_index = Position::sqrt_price_to_tick_index(sqrt_price);
            dbg!(tick_index, round_trip_tick_index);
            // assert_eq!(round_trip_tick_index, *tick_index);
        }
    }
}
