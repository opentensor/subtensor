/// Position designates one liquidity position. 
/// 
/// Alpha price is expressed in rao units per one 10^9 unit. For example, 
/// price 1_000_000 is equal to 0.001 TAO per Alpha.
/// 
/// price_low - lower boundary of price
/// price_high - higher boundary of price
/// liquidity - position liquidity
/// fees_tao - fees accrued by the position in quote currency (TAO)
/// fees_alpha - fees accrued by the position in base currency (Alpha)
/// 
struct Position {
    price_low: u64,
    price_high: u64,
    liquidity: u128, // ???
    fees_tao: u64,
    fees_alpha: u64,
}

