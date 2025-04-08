#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use subtensor_swap_interface::OrderType;
use substrate_fixed::types::U64F64;

pub mod pallet;
mod position;
mod tick;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
pub(crate) mod mock;

type SqrtPrice = U64F64;

#[derive(Debug, PartialEq)]
pub struct RemoveLiquidityResult {
    tao: u64,
    alpha: u64,
    fee_tao: u64,
    fee_alpha: u64,
}

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
