//! Benchmarking setup for pallet-subtensor-swap
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::arithmetic_side_effects)]

use crate::pallet::{Call, Config, Pallet};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks(where T: Config)]
mod benchmarks {
	use super::*; // Use imports from outer scope

	#[benchmark]
	fn set_fee_rate() {
		let netuid: u16 = 1;
		let rate: u16 = 100;  // Some arbitrary fee rate value

		#[extrinsic_call]
		set_fee_rate(RawOrigin::Root, netuid, rate);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
