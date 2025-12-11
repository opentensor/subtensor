/// Weight functions for pallet-balancer-swap
///
/// These are placeholder weights until proper benchmarking is performed.

use frame_support::weights::{Weight, constants::RocksDbWeight};

pub trait WeightInfo {
    fn add_liquidity() -> Weight;
    fn remove_liquidity() -> Weight;
    fn set_pool_weights() -> Weight;
    fn set_swap_fee() -> Weight;
}

/// Default weight implementation using database constants
pub struct DefaultWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for DefaultWeight<T> {
    fn add_liquidity() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn remove_liquidity() -> Weight {
        Weight::from_parts(50_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(3))
            .saturating_add(RocksDbWeight::get().writes(3))
    }

    fn set_pool_weights() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }

    fn set_swap_fee() -> Weight {
        Weight::from_parts(30_000_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}



