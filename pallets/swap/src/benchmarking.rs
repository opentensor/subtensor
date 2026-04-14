//! Benchmarking setup for pallet-subtensor-swap
#![allow(clippy::unwrap_used)]
#![allow(clippy::multiple_bound_locations)]

use core::marker::PhantomData;

use frame_benchmarking::v2::*;
use frame_support::assert_err;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use substrate_fixed::types::{I64F64, U64F64};
use subtensor_runtime_common::NetUid;

use crate::{
    Error,
    pallet::{
        AlphaSqrtPrice, BenchmarkHelper, Call, Config, CurrentLiquidity, CurrentTick,
        EnabledUserLiquidity, Pallet, Positions, SwapV3Initialized,
    },
    position::{Position, PositionId},
    tick::TickIndex,
};

#[benchmarks(where T: Config)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_fee_rate() {
        let netuid = NetUid::from(1);
        let rate: u16 = 100;

        #[extrinsic_call]
        _(RawOrigin::Root, netuid, rate);
    }

    #[benchmark]
    fn add_liquidity() {
        let netuid = NetUid::from(1);

        if !SwapV3Initialized::<T>::get(netuid) {
            SwapV3Initialized::<T>::insert(netuid, true);
            AlphaSqrtPrice::<T>::insert(netuid, U64F64::from_num(1));
            CurrentTick::<T>::insert(netuid, TickIndex::new(0).unwrap());
            CurrentLiquidity::<T>::insert(netuid, T::MinimumLiquidity::get());
        }

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);
        let tick_low = TickIndex::new_unchecked(-1000);
        let tick_high = TickIndex::new_unchecked(1000);

        #[block]
        {
            assert_err!(
                Pallet::<T>::add_liquidity(
                    RawOrigin::Signed(caller).into(),
                    hotkey,
                    netuid,
                    tick_low,
                    tick_high,
                    1000,
                ),
                Error::<T>::UserLiquidityDisabled
            );
        }
    }

    #[benchmark]
    fn remove_liquidity() {
        let netuid = NetUid::from(1);

        T::BenchmarkHelper::setup_subnet(netuid);

        if !SwapV3Initialized::<T>::get(netuid) {
            SwapV3Initialized::<T>::insert(netuid, true);
            AlphaSqrtPrice::<T>::insert(netuid, U64F64::from_num(1));
            CurrentTick::<T>::insert(netuid, TickIndex::new(0).unwrap());
            CurrentLiquidity::<T>::insert(netuid, T::MinimumLiquidity::get());
        }

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);
        T::BenchmarkHelper::register_hotkey(&hotkey, &caller);
        let id = PositionId::from(1u128);

        Positions::<T>::insert(
            (netuid, caller.clone(), id),
            Position {
                id,
                netuid,
                tick_low: TickIndex::new(-10000).unwrap(),
                tick_high: TickIndex::new(10000).unwrap(),
                liquidity: 1000,
                fees_tao: I64F64::from_num(0),
                fees_alpha: I64F64::from_num(0),
                _phantom: PhantomData,
            },
        );

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), hotkey, netuid.into(), id.into());
    }

    #[benchmark]
    fn modify_position() {
        let netuid = NetUid::from(1);

        T::BenchmarkHelper::setup_subnet(netuid);
        EnabledUserLiquidity::<T>::insert(netuid, true);

        if !SwapV3Initialized::<T>::get(netuid) {
            SwapV3Initialized::<T>::insert(netuid, true);
            AlphaSqrtPrice::<T>::insert(netuid, U64F64::from_num(1));
            CurrentTick::<T>::insert(netuid, TickIndex::new(0).unwrap());
            CurrentLiquidity::<T>::insert(netuid, T::MinimumLiquidity::get());
        }

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);
        T::BenchmarkHelper::register_hotkey(&hotkey, &caller);
        let id = PositionId::from(1u128);

        Positions::<T>::insert(
            (netuid, caller.clone(), id),
            Position {
                id,
                netuid,
                tick_low: TickIndex::new(-10000).unwrap(),
                tick_high: TickIndex::new(10000).unwrap(),
                liquidity: 10000,
                fees_tao: I64F64::from_num(0),
                fees_alpha: I64F64::from_num(0),
                _phantom: PhantomData,
            },
        );

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            hotkey,
            netuid.into(),
            id.into(),
            -5000,
        );
    }

    #[benchmark]
    fn disable_lp() {
        // Create a single user LP position so that do_dissolve_all_liquidity_providers
        // executes its main path at least once.
        let caller: T::AccountId = whitelisted_caller();
        let id = PositionId::from(1u128);

        for index in 1..=128 {
            let netuid = NetUid::from(index);

            SwapV3Initialized::<T>::insert(netuid, true);
            AlphaSqrtPrice::<T>::insert(netuid, U64F64::from_num(1));
            CurrentTick::<T>::insert(netuid, TickIndex::new(0).unwrap());
            CurrentLiquidity::<T>::insert(netuid, T::MinimumLiquidity::get());

            Positions::<T>::insert(
                (netuid, caller.clone(), id),
                Position {
                    id,
                    netuid,
                    tick_low: TickIndex::new(-10000).unwrap(),
                    tick_high: TickIndex::new(10000).unwrap(),
                    liquidity: 1_000,
                    fees_tao: I64F64::from_num(0),
                    fees_alpha: I64F64::from_num(0),
                    _phantom: PhantomData,
                },
            );

            // Enable user liquidity on this subnet so the toggle path is exercised.
            EnabledUserLiquidity::<T>::insert(netuid, true);
        }

        #[extrinsic_call]
        disable_lp(RawOrigin::Root);
    }

    #[benchmark]
    fn toggle_user_liquidity() {
        let netuid = NetUid::from(101);
        T::BenchmarkHelper::setup_subnet(netuid);

        assert!(!EnabledUserLiquidity::<T>::get(netuid));

        #[extrinsic_call]
        _(RawOrigin::Root, netuid.into(), true);
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
