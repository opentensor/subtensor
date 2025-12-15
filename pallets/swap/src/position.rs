use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
// use safe_math::*;
use substrate_fixed::types::U64F64;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::NetUid;

use crate::pallet::{Config, Error, LastPositionId};

/// Position designates one liquidity position.
///
/// Alpha price is expressed in rao units per one 10^9 unit. For example,
/// price 1_000_000 is equal to 0.001 TAO per Alpha.
#[freeze_struct("3f68e54e8969f976")]
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
#[scale_info(skip_type_params(T))]
pub struct Position<T: Config> {
    /// Unique ID of the position
    pub id: PositionId,
    /// Network identifier
    pub netuid: NetUid,
    // /// Position liquidity
    // pub liquidity: u64,
    // /// Fees accrued by the position in quote currency (TAO) relative to global fees
    // pub fees_tao: I64F64,
    // /// Fees accrued by the position in base currency (Alpha) relative to global fees
    // pub fees_alpha: I64F64,
    /// Phantom marker for generic Config type
    pub _phantom: PhantomData<T>,
}

impl<T: Config> Position<T> {
    pub fn new(
        id: PositionId,
        netuid: NetUid,
        // liquidity: u64,
    ) -> Self {
        let position = Position {
            id,
            netuid,
            // liquidity,
            // fees_tao: I64F64::saturating_from_num(0),
            // fees_alpha: I64F64::saturating_from_num(0),
            _phantom: PhantomData,
        };

        // position.fees_tao = position.fees_in_range(true);
        // position.fees_alpha = position.fees_in_range(false);

        position
    }

    /// Converts position to token amounts
    ///
    /// returns tuple of (TAO, Alpha)
    ///
    pub fn to_token_amounts(&self, _price_curr: U64F64) -> Result<(u64, u64), Error<T>> {
        // let one = U64F64::saturating_from_num(1);

        // let sqrt_price_low = self
        //     .tick_low
        //     .try_to_sqrt_price()
        //     .map_err(|_| Error::<T>::InvalidTickRange)?;
        // let sqrt_price_high = self
        //     .tick_high
        //     .try_to_sqrt_price()
        //     .map_err(|_| Error::<T>::InvalidTickRange)?;
        // let liquidity_fixed = U64F64::saturating_from_num(self.liquidity);

        // Ok(if sqrt_price_curr < sqrt_price_low {
        //     (
        //         0,
        //         liquidity_fixed
        //             .saturating_mul(
        //                 one.safe_div(sqrt_price_low)
        //                     .saturating_sub(one.safe_div(sqrt_price_high)),
        //             )
        //             .saturating_to_num::<u64>(),
        //     )
        // } else if sqrt_price_curr > sqrt_price_high {
        //     (
        //         liquidity_fixed
        //             .saturating_mul(sqrt_price_high.saturating_sub(sqrt_price_low))
        //             .saturating_to_num::<u64>(),
        //         0,
        //     )
        // } else {
        //     (
        //         liquidity_fixed
        //             .saturating_mul(sqrt_price_curr.saturating_sub(sqrt_price_low))
        //             .saturating_to_num::<u64>(),
        //         liquidity_fixed
        //             .saturating_mul(
        //                 one.safe_div(sqrt_price_curr)
        //                     .saturating_sub(one.safe_div(sqrt_price_high)),
        //             )
        //             .saturating_to_num::<u64>(),
        //     )
        // })

        todo!()
    }

    /// Collect fees for a position
    /// Updates the position
    pub fn collect_fees(&mut self) -> (u64, u64) {
        todo!()
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
