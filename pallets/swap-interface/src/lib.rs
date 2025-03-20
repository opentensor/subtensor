#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use core::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Sell,
    Buy,
}

pub trait SwapHandler<AccountId> {
    fn swap(order_t: OrderType, amount: u64) -> Result<(), Box<dyn Error>>;
    fn add_liquidity(account_id: AccountId, liquidity: u64) -> Result<(), Box<dyn Error>>;
    fn remove_liquidity(account_id: AccountId) -> Result<(), Box<dyn Error>>;
}

pub trait LiquidityDataProvider<First, Second> {
    fn first_reserve() -> First;
    fn second_reserve() -> Second;
}
