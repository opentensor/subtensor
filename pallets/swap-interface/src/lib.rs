#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
}

pub trait SwapHandler<AccountId> {
    fn swap(
        netuid: u16,
        order_t: OrderType,
        amount: u64,
        price_limit: u64,
    ) -> Result<SwapResult, DispatchError>;
    fn add_liquidity(
        netuid: u16,
        account_id: &AccountId,
        tick_low: i32,
        tick_high: i32,
        liquidity: u64,
    ) -> Result<(u64, u64), DispatchError>;
    fn remove_liquidity(
        netuid: u16,
        account_id: &AccountId,
        position_id: PositionId,
    ) -> Result<(u64, u64), DispatchError>;
    fn max_price() -> u64;
    fn min_price() -> u64;
}

#[derive(Debug, PartialEq)]
pub struct SwapResult {
    pub amount_paid_out: u64,
    pub refund: u64,
    // calculated new tao/alpha reserves
    pub new_tao_reserve: u64,
    pub new_alpha_reserve: u64,
}

pub trait LiquidityDataProvider<AccountId> {
    fn tao_reserve(netuid: u16) -> u64;
    fn alpha_reserve(netuid: u16) -> u64;
    fn tao_balance(account_id: &AccountId) -> u64;
    fn alpha_balance(netuid: u16, account_id: &AccountId) -> u64;
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

impl From<[u8; 16]> for PositionId {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}
