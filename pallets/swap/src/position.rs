use core::marker::PhantomData;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_math::*;
use substrate_fixed::types::{I64F64, U64F64};
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::NetUid;

use crate::SqrtPrice;
use crate::pallet::{Config, Error, FeeGlobalAlpha, FeeGlobalTao, LastPositionId};
use crate::tick::TickIndex;

/// Position designates one liquidity position.
///
/// Alpha price is expressed in rao units per one 10^9 unit. For example,
/// price 1_000_000 is equal to 0.001 TAO per Alpha.
#[freeze_struct("27a1bf8c59480f0")]
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
#[scale_info(skip_type_params(T))]
pub struct Position<T: Config> {
    /// Unique ID of the position
    pub id: PositionId,
    /// Network identifier
    pub netuid: NetUid,
    /// Tick index for lower boundary of price
    pub tick_low: TickIndex,
    /// Tick index for higher boundary of price
    pub tick_high: TickIndex,
    /// Position liquidity
    pub liquidity: u64,
    /// Fees accrued by the position in quote currency (TAO) relative to global fees
    pub fees_tao: I64F64,
    /// Fees accrued by the position in base currency (Alpha) relative to global fees
    pub fees_alpha: I64F64,
    /// Phantom marker for generic Config type
    pub _phantom: PhantomData<T>,
}

impl<T: Config> Position<T> {
    pub fn new(
        id: PositionId,
        netuid: NetUid,
        tick_low: TickIndex,
        tick_high: TickIndex,
        liquidity: u64,
    ) -> Self {
        let mut position = Position {
            id,
            netuid,
            tick_low,
            tick_high,
            liquidity,
            fees_tao: I64F64::saturating_from_num(0),
            fees_alpha: I64F64::saturating_from_num(0),
            _phantom: PhantomData,
        };

        position.fees_tao = position.fees_in_range(true);
        position.fees_alpha = position.fees_in_range(false);

        position
    }

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
    pub fn to_token_amounts(&self, sqrt_price_curr: SqrtPrice) -> Result<(u64, u64), Error<T>> {
        let one = U64F64::saturating_from_num(1);

        let sqrt_price_low = self
            .tick_low
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let sqrt_price_high = self
            .tick_high
            .try_to_sqrt_price()
            .map_err(|_| Error::<T>::InvalidTickRange)?;
        let liquidity_fixed = U64F64::saturating_from_num(self.liquidity);

        Ok(if sqrt_price_curr < sqrt_price_low {
            (
                0,
                liquidity_fixed
                    .saturating_mul(
                        one.safe_div(sqrt_price_low)
                            .saturating_sub(one.safe_div(sqrt_price_high)),
                    )
                    .saturating_to_num::<u64>(),
            )
        } else if sqrt_price_curr > sqrt_price_high {
            (
                liquidity_fixed
                    .saturating_mul(sqrt_price_high.saturating_sub(sqrt_price_low))
                    .saturating_to_num::<u64>(),
                0,
            )
        } else {
            (
                liquidity_fixed
                    .saturating_mul(sqrt_price_curr.saturating_sub(sqrt_price_low))
                    .saturating_to_num::<u64>(),
                liquidity_fixed
                    .saturating_mul(
                        one.safe_div(sqrt_price_curr)
                            .saturating_sub(one.safe_div(sqrt_price_high)),
                    )
                    .saturating_to_num::<u64>(),
            )
        })
    }

    /// Collect fees for a position
    /// Updates the position
    pub fn collect_fees(&mut self) -> (u64, u64) {
        let fee_tao_agg = self.fees_in_range(true);
        let fee_alpha_agg = self.fees_in_range(false);

        let mut fee_tao = fee_tao_agg.saturating_sub(self.fees_tao);
        let mut fee_alpha = fee_alpha_agg.saturating_sub(self.fees_alpha);

        self.fees_tao = fee_tao_agg;
        self.fees_alpha = fee_alpha_agg;

        let liquidity_frac = I64F64::saturating_from_num(self.liquidity);

        fee_tao = liquidity_frac.saturating_mul(fee_tao);
        fee_alpha = liquidity_frac.saturating_mul(fee_alpha);

        (
            fee_tao.saturating_to_num::<u64>(),
            fee_alpha.saturating_to_num::<u64>(),
        )
    }

    /// Get fees in a position's range
    ///
    /// If quote flag is true, Tao is returned, otherwise alpha.
    fn fees_in_range(&self, quote: bool) -> I64F64 {
        if quote {
            I64F64::saturating_from_num(FeeGlobalTao::<T>::get(self.netuid))
        } else {
            I64F64::saturating_from_num(FeeGlobalAlpha::<T>::get(self.netuid))
        }
        .saturating_sub(self.tick_low.fees_below::<T>(self.netuid, quote))
        .saturating_sub(self.tick_high.fees_above::<T>(self.netuid, quote))
    }
}

#[freeze_struct("8501fa251c9d74c")]
#[derive(
    Clone,
    Copy,
    Decode,
    DecodeWithMemTracking,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
)]
pub struct PositionId(u128);

impl PositionId {
    /// Create a new position ID
    pub fn new<T: Config>() -> Self {
        let new = LastPositionId::<T>::get().saturating_add(1);
        LastPositionId::<T>::put(new);

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
