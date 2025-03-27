use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_math::*;
use substrate_fixed::types::U64F64;
use uuid::Uuid;

use crate::SqrtPrice;
use crate::pallet::{Config, Error};
use crate::tick::TickIndex;

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
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct Position {
    pub id: PositionId,
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
    pub fn to_token_amounts<T: Config>(
        &self,
        sqrt_price_curr: SqrtPrice,
    ) -> Result<(u64, u64), Error<T>> {
        let one = U64F64::saturating_from_num(1);

        let sqrt_pa: SqrtPrice = self
            .tick_low
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_pb: SqrtPrice = self
            .tick_high
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
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

#[derive(
    Clone, Copy, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
)]
pub struct PositionId([u8; 16]);

impl PositionId {
    /// Create a new position ID using UUID v4
    pub fn new() -> Self {
        Self(Uuid::new_v4().into_bytes())
    }
}

impl From<Uuid> for PositionId {
    fn from(value: Uuid) -> Self {
        Self(value.into_bytes())
    }
}

impl From<PositionId> for Uuid {
    fn from(value: PositionId) -> Self {
        Uuid::from_bytes(value.0)
    }
}
