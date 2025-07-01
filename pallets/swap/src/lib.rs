#![cfg_attr(not(feature = "std"), no_std)]
use core::fmt::{self, Display, Formatter};
use core::ops::{Add, Mul};

use codec::{Compact, CompactAs, Decode, Encode, Error as CodecError, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use safe_math::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use substrate_fixed::types::U64F64;
use subtensor_macros::freeze_struct;
use subtensor_swap_interface::OrderType;

pub mod pallet;
pub mod position;
pub mod tick;
pub mod weights;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;

type SqrtPrice = U64F64;

#[freeze_struct("91109ca21993a3bf")]
#[repr(transparent)]
#[derive(
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Decode,
    Default,
    Encode,
    Eq,
    Hash,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
)]
#[serde(transparent)]
pub struct FeeRateT(u16);

impl FeeRateT {
    pub fn as_normalized_f64(&self) -> f64 {
        (self.0 as f64) / (u16::MAX as f64)
    }

    pub fn as_normalized_fixed(&self) -> U64F64 {
        U64F64::saturating_from_num(self.0).safe_div(U64F64::from_num(u16::MAX))
    }

    pub fn as_f64(&self) -> f64 {
        self.0 as f64
    }

    pub fn as_fixed(&self) -> U64F64 {
        U64F64::saturating_from_num(self.0)
    }
}

impl Display for FeeRateT {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.as_normalized_f64(), f)
    }
}

impl CompactAs for FeeRateT {
    type As = u16;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<FeeRateT>> for FeeRateT {
    fn from(c: Compact<FeeRateT>) -> Self {
        c.0
    }
}

impl From<FeeRateT> for u16 {
    fn from(val: FeeRateT) -> Self {
        val.0
    }
}

impl From<u16> for FeeRateT {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Add<FeeRateT> for FeeRateT {
    type Output = Self;

    #[allow(clippy::arithmetic_side_effects)]
    fn add(self, rhs: Self) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl Mul<FeeRateT> for FeeRateT {
    type Output = Self;

    #[allow(clippy::arithmetic_side_effects)]
    fn mul(self, rhs: Self) -> Self::Output {
        (self.0 * rhs.0).into()
    }
}
