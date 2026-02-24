//! Weights for pallet_subtensor_swap
//!
//! This is a default weight implementation with conservative estimates
//! until actual benchmarks are run.

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{
    traits::Get,
    weights::{Weight, constants::RocksDbWeight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_subtensor_swap.
pub trait WeightInfo {
    fn set_fee_rate() -> Weight;
    fn add_liquidity() -> Weight;
    fn remove_liquidity() -> Weight;
    fn modify_position() -> Weight;
    fn toggle_user_liquidity() -> Weight;
}

/// Default weights for pallet_subtensor_swap.
pub struct DefaultWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DefaultWeight<T> {
    fn set_fee_rate() -> Weight {
        // Conservative weight estimate: one read and one write
        Weight::from_parts(10_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    fn add_liquidity() -> Weight {
        // Conservative weight estimate for add_liquidity
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn remove_liquidity() -> Weight {
        // Conservative weight estimate for remove_liquidity
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn modify_position() -> Weight {
        // Conservative weight estimate for modify_position
        Weight::from_parts(50_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }

    fn toggle_user_liquidity() -> Weight {
        // Conservative weight estimate: one read and one write
        Weight::from_parts(10_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn set_fee_rate() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn add_liquidity() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(5))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    fn remove_liquidity() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    fn modify_position() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(4))
            .saturating_add(RocksDbWeight::get().writes(4))
    }

    fn toggle_user_liquidity() -> Weight {
        Weight::from_parts(10_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
