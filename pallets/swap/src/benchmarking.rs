//! Benchmarking setup for pallet-subtensor-swap
#![allow(clippy::unwrap_used)]
#![allow(clippy::multiple_bound_locations)]

use core::marker::PhantomData;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use subtensor_runtime_common::NetUid;

use crate::{
    pallet::{
        Call, Config, Pallet, PositionsV2,
    },
    position::{Position, PositionId},
};

fn init_swap<T: Config>(netuid: NetUid) {
    let _ = Pallet::<T>::maybe_initialize_palswap(netuid);
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

    #[benchmark]
    fn add_liquidity() {
        let netuid = NetUid::from(1);

        init_swap::<T>(netuid);

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);

        #[extrinsic_call]
        add_liquidity(
            RawOrigin::Signed(caller),
            hotkey,
            netuid,
            1000,
        );
    }

    #[benchmark]
    fn remove_liquidity() {
        let netuid = NetUid::from(1);

        init_swap::<T>(netuid);

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);
        let id = PositionId::from(1u128);

        PositionsV2::<T>::insert(
            (netuid, caller.clone(), id),
            Position {
                id,
                netuid,
                // liquidity: 1000,
                // fees_tao: I64F64::from_num(0),
                // fees_alpha: I64F64::from_num(0),
                _phantom: PhantomData,
            },
        );

        #[extrinsic_call]
        remove_liquidity(RawOrigin::Signed(caller), hotkey, netuid.into(), id.into());
    }

    #[benchmark]
    fn modify_position() {
        let netuid = NetUid::from(1);

        init_swap::<T>(netuid);

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);
        let id = PositionId::from(1u128);

        PositionsV2::<T>::insert(
            (netuid, caller.clone(), id),
            Position {
                id,
                netuid,
                // liquidity: 10000,
                // fees_tao: I64F64::from_num(0),
                // fees_alpha: I64F64::from_num(0),
                _phantom: PhantomData,
            },
        );

        #[extrinsic_call]
        modify_position(
            RawOrigin::Signed(caller),
            hotkey,
            netuid.into(),
            id.into(),
            -5000,
        );
    }

    // #[benchmark]
    // fn toggle_user_liquidity() {
    //     let netuid = NetUid::from(101);

    //     assert!(!EnabledUserLiquidity::<T>::get(netuid));

    //     #[extrinsic_call]
    //     toggle_user_liquidity(RawOrigin::Root, netuid.into(), true);

    //     assert!(EnabledUserLiquidity::<T>::get(netuid));
    // }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
