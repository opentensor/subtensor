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
