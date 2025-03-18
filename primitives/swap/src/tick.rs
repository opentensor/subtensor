use substrate_fixed::types::U64F64;

use crate::SqrtPrice;
use crate::tick_math::{
    MAX_TICK, MIN_TICK, TickMathError, get_sqrt_ratio_at_tick, get_tick_at_sqrt_ratio,
    u64f64_to_u256_q64_96, u256_q64_96_to_u64f64,
};

/// Tick is the price range determined by tick index (not part of this struct,
/// but is the key at which the Tick is stored in state hash maps). Tick struct
/// stores liquidity and fee information.
///
///   - Net liquidity
///   - Gross liquidity
///   - Fees (above global) in both currencies
///
#[derive(Default)]
pub struct Tick {
    pub liquidity_net: i128,
    pub liquidity_gross: u64,
    pub fees_out_tao: U64F64,
    pub fees_out_alpha: U64F64,
}

/// Converts tick index into SQRT of lower price of this tick
/// In order to find the higher price of this tick, call
/// tick_index_to_sqrt_price(tick_idx + 1)
pub fn tick_index_to_sqrt_price(tick_idx: i32) -> Result<SqrtPrice, TickMathError> {
    // because of u256->u128 conversion we have twice less values for min/max ticks
    if !(MIN_TICK / 2..=MAX_TICK / 2).contains(&tick_idx) {
        return Err(TickMathError::TickOutOfBounds);
    }
    get_sqrt_ratio_at_tick(tick_idx).and_then(u256_q64_96_to_u64f64)
}

/// Converts SQRT price to tick index
/// Because the tick is the range of prices [sqrt_lower_price, sqrt_higher_price),
/// the resulting tick index matches the price by the following inequality:
///    sqrt_lower_price <= sqrt_price < sqrt_higher_price
pub fn sqrt_price_to_tick_index(sqrt_price: SqrtPrice) -> Result<i32, TickMathError> {
    let tick = get_tick_at_sqrt_ratio(u64f64_to_u256_q64_96(sqrt_price))?;

    // Correct for rounding error during conversions between different fixed-point formats
    Ok(if tick == 0 {
        tick
    } else {
        tick.saturating_add(1)
    })
}

pub fn find_closest_lower_active_tick(_index: i32) -> Option<Tick> {
    todo!()
}

pub fn find_closest_higher_active_tick(_index: i32) -> Option<Tick> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use safe_math::FixedExt;

    #[test]
    fn test_tick_index_to_sqrt_price() {
        let tick_spacing = SqrtPrice::from_num(1.0001);

        // check tick bounds
        assert_eq!(
            tick_index_to_sqrt_price(MIN_TICK),
            Err(TickMathError::TickOutOfBounds)
        );

        assert_eq!(
            tick_index_to_sqrt_price(MAX_TICK),
            Err(TickMathError::TickOutOfBounds),
        );

        // At tick index 0, the sqrt price should be 1.0
        let sqrt_price = tick_index_to_sqrt_price(0).unwrap();
        assert_eq!(sqrt_price, SqrtPrice::from_num(1.0));

        let sqrt_price = tick_index_to_sqrt_price(2).unwrap();
        assert!(sqrt_price.abs_diff(tick_spacing) < SqrtPrice::from_num(1e-10));

        let sqrt_price = tick_index_to_sqrt_price(4).unwrap();
        // Calculate the expected value: (1 + TICK_SPACING/1e9 + 1.0)^2
        let expected = tick_spacing * tick_spacing;
        assert!(sqrt_price.abs_diff(expected) < SqrtPrice::from_num(1e-10));

        // Test with tick index 10
        let sqrt_price = tick_index_to_sqrt_price(10).unwrap();
        // Calculate the expected value: (1 + TICK_SPACING/1e9 + 1.0)^5
        let expected = tick_spacing.checked_pow(5).unwrap();
        assert!(
            sqrt_price.abs_diff(expected) < SqrtPrice::from_num(1e-10),
            "diff: {}",
            sqrt_price.abs_diff(expected),
        );
    }

    #[test]
    fn test_sqrt_price_to_tick_index() {
        let tick_spacing = SqrtPrice::from_num(1.0001);
        let tick_index = sqrt_price_to_tick_index(SqrtPrice::from_num(1.0)).unwrap();
        assert_eq!(tick_index, 0);

        // Test with sqrt price equal to tick_spacing_tao (should be tick index 2)
        let tick_index = sqrt_price_to_tick_index(tick_spacing).unwrap();
        assert_eq!(tick_index, 2);

        // Test with sqrt price equal to tick_spacing_tao^2 (should be tick index 4)
        let sqrt_price = tick_spacing * tick_spacing;
        let tick_index = sqrt_price_to_tick_index(sqrt_price).unwrap();
        assert_eq!(tick_index, 4);

        // Test with sqrt price equal to tick_spacing_tao^5 (should be tick index 10)
        let sqrt_price = tick_spacing.checked_pow(5).unwrap();
        let tick_index = sqrt_price_to_tick_index(sqrt_price).unwrap();
        assert_eq!(tick_index, 10);
    }

    #[test]
    fn test_roundtrip_tick_index_sqrt_price() {
        for tick_index in [
            MIN_TICK / 2,
            -1000,
            -100,
            -10,
            -4,
            -2,
            0,
            2,
            4,
            10,
            100,
            1000,
            MAX_TICK / 2,
        ]
        .iter()
        {
            let sqrt_price = tick_index_to_sqrt_price(*tick_index).unwrap();
            let round_trip_tick_index = sqrt_price_to_tick_index(sqrt_price).unwrap();
            assert_eq!(round_trip_tick_index, *tick_index);
        }
    }
}
