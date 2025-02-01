#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use crate::RootClaimTypeEnum;
use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn root_claim_on_subnet(claim_type: RootClaimTypeEnum) -> Weight;
	fn block_hash_to_indices(k: u64, n: u64) -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn root_claim_on_subnet(_claim_type: RootClaimTypeEnum) -> Weight {
		// TODO should be replaced by benchmarked weights
		// Weight:
		// 	100_000 + 5 reads
		//	1 read, 1 write
		// 	3 reads, 3 writes for increase stake
		// Total: 100_000 + 9 reads + 4 writes
		Weight::default().saturating_add(
			T::DbWeight::get().reads_writes(9_u64, 4_u64)
		).saturating_add(
			Weight::from_parts(100_000, 0)
		)
	}

	fn block_hash_to_indices(k: u64, _n: u64) -> Weight {
		// TODO: should be benchmarked
		Weight::from_parts(k.saturating_mul(100_000), 0)
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	fn root_claim_on_subnet(_root_claim_type: RootClaimTypeEnum) -> Weight {
		// TODO should be replaced by benchmarked weights
		// Weight:
		// 	100_000 + 5 reads
		//	1 read, 1 write
		// 	3 reads, 3 writes for increase stake
		// Total: 100_000 + 9 reads + 4 writes
		Weight::default().saturating_add(
			RocksDbWeight::get().reads_writes(9_u64, 4_u64)
		).saturating_add(
			Weight::from_parts(100_000, 0)
		)
	}

	fn block_hash_to_indices(k: u64, _n: u64) -> Weight {
		// TODO: should be benchmarked
		Weight::from_parts(k.saturating_mul(100_000), 0)
	}
}
