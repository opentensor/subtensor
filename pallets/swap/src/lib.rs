#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use substrate_fixed::types::U64F64;
use subtensor_macros::freeze_struct;
use subtensor_swap_interface::OrderType;

pub mod pallet;
mod position;
mod tick;
pub mod weights;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;

type SqrtPrice = U64F64;

#[freeze_struct("2a62496e31bbcddc")]
#[derive(
    Clone, Copy, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
)]
pub struct NetUid(u16);

impl From<NetUid> for u16 {
    fn from(val: NetUid) -> Self {
        val.0
    }
}

impl From<u16> for NetUid {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
