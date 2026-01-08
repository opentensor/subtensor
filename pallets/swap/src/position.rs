use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
// use safe_math::*;
use substrate_fixed::types::U64F64;
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::NetUid;

use crate::pallet::{Config, Error, LastPositionId};

/// Position designates one liquidity position.
#[freeze_struct("6d7ff015e0a73860")]
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
        Position {
            id,
            netuid,
            // liquidity,
            // fees_tao: I64F64::saturating_from_num(0),
            // fees_alpha: I64F64::saturating_from_num(0),
            _phantom: PhantomData,
        }
    }

    /// Converts position to token amounts
    ///
    /// returns tuple of (TAO, Alpha)
    ///
    pub fn to_token_amounts(&self, _price_curr: U64F64) -> Result<(u64, u64), Error<T>> {
        // TODO: Revise when user liquidity is available
        Ok((0, 0))
    }

    /// Collect fees for a position
    /// Updates the position
    pub fn collect_fees(&mut self) -> (u64, u64) {
        // TODO: Revise when user liquidity is available
        (0, 0)
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
