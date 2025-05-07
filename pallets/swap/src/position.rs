use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_math::*;
use substrate_fixed::types::U64F64;
use subtensor_macros::freeze_struct;

use crate::pallet::{Config, Error, FeeGlobalAlpha, FeeGlobalTao, NextPositionId};
use crate::tick::TickIndex;
use crate::{NetUid, SqrtPrice};

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
#[freeze_struct("1a4924d76eb084f1")]
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
pub struct Position {
    pub id: PositionId,
    pub netuid: NetUid,
    pub tick_low: TickIndex,
    pub tick_high: TickIndex,
    pub liquidity: u64,
    pub fees_tao: U64F64,
    pub fees_alpha: U64F64,
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

    /// Collect fees for a position
    /// Updates the position
    pub fn collect_fees<T: Config>(&mut self) -> (u64, u64) {
        let fee_tao_agg = self.fees_in_range::<T>(true);
        let fee_alpha_agg = self.fees_in_range::<T>(false);

        let mut fee_tao = fee_tao_agg.saturating_sub(self.fees_tao);
        let mut fee_alpha = fee_alpha_agg.saturating_sub(self.fees_alpha);

        self.fees_tao = fee_tao_agg;
        self.fees_alpha = fee_alpha_agg;

        let liquidity_frac = U64F64::saturating_from_num(self.liquidity);
        fee_tao = liquidity_frac.saturating_mul(fee_tao);
        fee_alpha = liquidity_frac.saturating_mul(fee_alpha);

        (fee_tao.saturating_to_num::<u64>(), fee_alpha.saturating_to_num::<u64>())
    }

    /// Get fees in a position's range
    ///
    /// If quote flag is true, Tao is returned, otherwise alpha.
    fn fees_in_range<T: Config>(&self, quote: bool) -> U64F64 {
        if quote {
            FeeGlobalTao::<T>::get(self.netuid)
        } else {
            FeeGlobalAlpha::<T>::get(self.netuid)
        }
        .saturating_sub(self.tick_low.fees_below::<T>(self.netuid, quote))
        .saturating_sub(self.tick_high.fees_above::<T>(self.netuid, quote))
    }
}

#[freeze_struct("1f02550d787d80da")]
#[derive(
    Clone, Copy, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
)]
pub struct PositionId(u128);

impl PositionId {
    /// Create a new position ID
    pub fn new<T: Config>() -> Self {
        let new = NextPositionId::<T>::get().saturating_add(1);
        NextPositionId::<T>::put(new);

        Self(new)
    }
}

impl From<u128> for PositionId {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<PositionId> for u128 {
    fn from(value: PositionId) -> Self {
        value.0
    }
}
