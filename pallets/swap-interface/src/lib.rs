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
    fn add_liquidity(account_id: AccountId, liquidity: u64) -> Result<(u64, u64), Box<dyn Error>>;
    fn remove_liquidity(account_id: AccountId) -> Result<(u64, u64), Box<dyn Error>>;
}

pub trait LiquidityDataProvider<AccountId> {
    fn tao_reserve(netuid: u16) -> u64;
    fn alpha_reserve(netuid: u16) -> u64;
    fn tao_balance(netuid: u16, account_id: &AccountId) -> u64;
    fn alpha_balance(netuid: u16, account_id: &AccountId) -> u64;
}
