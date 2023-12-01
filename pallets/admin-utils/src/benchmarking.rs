//! Benchmarking setup
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as AdminUtils;
use frame_benchmarking::v2::*;
use frame_benchmarking::v1::account;
use frame_system::RawOrigin;
use frame_support::BoundedVec;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn swap_authorities(a: Linear<0, 32>) {
		let mut value: BoundedVec<
			<T as pallet::Config>::AuthorityId,
			<T as pallet::Config>::MaxAuthorities
		> = BoundedVec::new();

		for idx in 1..=a {
			let authority: <T as pallet::Config>::AuthorityId = account("Authority", idx, 0u32);
			let result = value.try_push(authority.clone());
			if result.is_err() {
				// Handle the error, perhaps by breaking the loop or logging an error message
			}
		}

		#[extrinsic_call]
		_(RawOrigin::Root, value);
	}

	//impl_benchmark_test_suite!(AdminUtils, crate::mock::new_test_ext(), crate::mock::Test);
}
