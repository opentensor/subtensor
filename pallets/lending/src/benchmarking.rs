//! Placeholder benchmarks for lift-based lending.
//! Measured weights must be produced on reference hardware; do not invent values.
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::unwrap_used)]

use super::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn open() {
        let caller: T::AccountId = whitelisted_caller();
        #[block]
        {
            let _ = Call::<T>::open {
                netuid: 1.into(),
                side: Side::Short,
                collateral: 1u64,
                hotkey: account("hk", 0, 0),
                min_borrow: 0u64,
                min_proceeds: 0u64,
            };
            let _ = caller;
        }
    }

    #[benchmark]
    fn open_exact() {
        #[block]
        {}
    }

    #[benchmark]
    fn open_long_and_sell() {
        #[block]
        {}
    }

    #[benchmark]
    fn close() {
        #[block]
        {}
    }

    #[benchmark]
    fn set_enabled() {
        #[extrinsic_call]
        _(RawOrigin::Root, true);
    }

    #[benchmark]
    fn set_lift_params() {
        #[extrinsic_call]
        _(RawOrigin::Root, LiftParams::default());
    }

    #[benchmark]
    fn set_min_price_impact() {
        #[block]
        {}
    }

    #[benchmark]
    fn collect() {
        #[block]
        {}
    }

    #[benchmark]
    fn update_reference() {
        #[block]
        {}
    }

    #[benchmark]
    fn settle() {
        #[block]
        {}
    }
}
