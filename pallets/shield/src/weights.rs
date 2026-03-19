
//! Placeholder weights for `pallet_shield`.
//!
//! These weights are NOT based on benchmarking output. They are copied from
//! the hardcoded values that were previously inlined in `#[pallet::weight(…)]`
//! annotations and will be replaced once proper benchmarks are run.

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_shield`.
pub trait WeightInfo {
	fn announce_next_key() -> Weight;
	fn submit_encrypted() -> Weight;
}

/// Weights for `pallet_shield` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn announce_next_key() -> Weight {
		Weight::from_parts(33_230_000, 0)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(6_u64))
	}
	fn submit_encrypted() -> Weight {
		Weight::from_parts(207_500_000, 0)
			.saturating_add(T::DbWeight::get().reads(0_u64))
			.saturating_add(T::DbWeight::get().writes(0_u64))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	fn announce_next_key() -> Weight {
		Weight::from_parts(33_230_000, 0)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(6_u64))
	}
	fn submit_encrypted() -> Weight {
		Weight::from_parts(207_500_000, 0)
			.saturating_add(RocksDbWeight::get().reads(0_u64))
			.saturating_add(RocksDbWeight::get().writes(0_u64))
	}
}
