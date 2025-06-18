#![cfg_attr(not(feature = "std"), no_std)]
use core::fmt::{self, Display, Formatter};

use codec::{Compact, CompactAs, Decode, Encode, Error as CodecError, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use substrate_fixed::types::U64F64;
use subtensor_swap_interface::OrderType;
use frame_support::pallet_prelude::*;
use subtensor_macros::freeze_struct;

pub mod pallet;
mod position;
pub mod tick;
pub mod weights;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;

type SqrtPrice = U64F64;

#[freeze_struct("7dfb986361b2098b")]
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
pub struct FeeRate(u16);

impl FeeRate {
	pub fn as_normalized_f64(&self) -> f64 {
		(self.0 as f64) / (u16::MAX as f64)
	}
}

impl Display for FeeRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.as_normalized_f64(), f)
    }
}

impl CompactAs for FeeRate {
    type As = u16;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<FeeRate>> for FeeRate {
    fn from(c: Compact<FeeRate>) -> Self {
        c.0
    }
}

impl From<FeeRate> for u16 {
    fn from(val: FeeRate) -> Self {
		val.0
    }
}

impl From<u16> for FeeRate {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

