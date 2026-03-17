//! Benchmarking setup for pallet-subtensor-swap
#![allow(clippy::unwrap_used)]
#![allow(clippy::multiple_bound_locations)]

use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use subtensor_runtime_common::NetUid;

use crate::pallet::{Call, Config, Pallet};

#[allow(dead_code)]
fn init_swap<T: Config>(netuid: NetUid) {
    let _ = Pallet::<T>::maybe_initialize_palswap(netuid, None);
}

#[benchmarks(where T: Config)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_fee_rate() {
        let netuid = NetUid::from(1);
        let rate: u16 = 100; // Some arbitrary fee rate value

        #[extrinsic_call]
        set_fee_rate(RawOrigin::Root, netuid, rate);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
