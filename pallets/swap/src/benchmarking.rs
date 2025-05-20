//! Benchmarking setup for pallet-subtensor-swap
#![allow(clippy::unwrap_used)]
#![allow(clippy::multiple_bound_locations)]

use frame_benchmarking::v2::*;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use substrate_fixed::types::U64F64;

use crate::{
    NetUid,
    pallet::{
        AlphaSqrtPrice, Call, Config, CurrentLiquidity, CurrentTick, Pallet, Positions,
        SwapV3Initialized,
    },
    position::{Position, PositionId},
    tick::TickIndex,
};

#[benchmarks(where T: Config)]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn set_fee_rate() {
        let netuid: u16 = 1;
        let rate: u16 = 100; // Some arbitrary fee rate value

        #[extrinsic_call]
        set_fee_rate(RawOrigin::Root, netuid, rate);
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

        #[extrinsic_call]
        add_liquidity(
            RawOrigin::Signed(caller),
            hotkey,
            netuid.into(),
            -10000,
            10000,
            1000,
        );
    }

    #[benchmark]
    fn remove_liquidity() {
        let netuid = NetUid::from(1);

        if !SwapV3Initialized::<T>::get(netuid) {
            SwapV3Initialized::<T>::insert(netuid, true);
            AlphaSqrtPrice::<T>::insert(netuid, U64F64::from_num(1));
            CurrentTick::<T>::insert(netuid, TickIndex::new(0).unwrap());
            CurrentLiquidity::<T>::insert(netuid, T::MinimumLiquidity::get());
        }

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);
        let id = PositionId::from(1u128);

        Positions::<T>::insert(
            (netuid, caller.clone(), id),
            Position {
                id,
                netuid,
                tick_low: TickIndex::new(-10000).unwrap(),
                tick_high: TickIndex::new(10000).unwrap(),
                liquidity: 1000,
                fees_tao: U64F64::from_num(0),
                fees_alpha: U64F64::from_num(0),
            },
        );

        #[extrinsic_call]
        remove_liquidity(RawOrigin::Signed(caller), hotkey, netuid.into(), id.into());
    }

    #[benchmark]
    fn modify_position() {
        let netuid = NetUid::from(1);

        if !SwapV3Initialized::<T>::get(netuid) {
            SwapV3Initialized::<T>::insert(netuid, true);
            AlphaSqrtPrice::<T>::insert(netuid, U64F64::from_num(1));
            CurrentTick::<T>::insert(netuid, TickIndex::new(0).unwrap());
            CurrentLiquidity::<T>::insert(netuid, T::MinimumLiquidity::get());
        }

        let caller: T::AccountId = whitelisted_caller();
        let hotkey: T::AccountId = account("hotkey", 0, 0);
        let id = PositionId::from(1u128);

        Positions::<T>::insert(
            (netuid, caller.clone(), id),
            Position {
                id,
                netuid,
                tick_low: TickIndex::new(-10000).unwrap(),
                tick_high: TickIndex::new(10000).unwrap(),
                liquidity: 10000,
                fees_tao: U64F64::from_num(0),
                fees_alpha: U64F64::from_num(0),
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

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
