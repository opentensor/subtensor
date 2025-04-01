#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use core::ops::Neg;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use pallet_subtensor_swap_interface::OrderType;
use safe_math::*;
use substrate_fixed::types::U64F64;
use uuid::Uuid;

use self::tick::{LayerLevel, Tick, TickIndex, TickIndexBitmap};
use crate::pallet::{Config, Error};

pub mod pallet;
mod position;
mod tick;

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

#[derive(Debug, PartialEq)]
pub struct SwapResult {
    amount_paid_out: u64,
    refund: u64,
}

#[derive(Debug, PartialEq)]
struct SwapStepResult {
    amount_to_take: u64,
    delta_out: u64,
}

pub enum SwapStepAction {
    Crossing,
    StopOn,
    StopIn,
}

#[derive(Debug, Decode, Encode, Eq, PartialEq)]
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
