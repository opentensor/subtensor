//! Benchmarking setup
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Registry;
use frame_benchmarking::v2::*;
use frame_benchmarking::v1::account;
use frame_system::RawOrigin;

use sp_runtime::traits::{StaticLookup, Bounded};
use frame_support::traits::Get;
use sp_std::mem::size_of;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn extrinsic() {
		
	}

	impl_benchmark_test_suite!(Registry, crate::mock::new_test_ext(), crate::mock::Test);
}
