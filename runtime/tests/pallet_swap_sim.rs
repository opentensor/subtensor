#![allow(clippy::unwrap_used)]

use frame_support::assert_ok;
use node_subtensor_runtime::{BuildStorage, Runtime, RuntimeGenesisConfig, System};
use pallet_subtensor::{
    SubnetAlphaIn, SubnetAlphaInProvided, SubnetMechanism, SubnetTAO, SubnetTaoProvided,
};
use pallet_subtensor_swap::{Pallet as SwapPallet, SwapV3Initialized};
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance};
use subtensor_swap_interface::SwapHandler;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig::default()
        .build_storage()
        .unwrap()
        .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

/// Set storage so the real `TaoReserve` / `AlphaReserve` implementations return
/// the requested values.  Both implementations sum the base map and the
/// "provided" map, so writing to the base map alone is sufficient when
/// `*Provided` stays zero (the default).
fn set_reserves(netuid: NetUid, tao: u64, alpha: u64) {
    SubnetTAO::<Runtime>::insert(netuid, TaoBalance::from(tao));
    SubnetAlphaIn::<Runtime>::insert(netuid, AlphaBalance::from(alpha));
}

/// Mark the subnet as having mechanism 1 (dynamic / V3) so `sim_swap_pure`
/// and `sim_swap` both take the V3 code-path.
fn set_dynamic_mechanism(netuid: NetUid) {
    SubnetMechanism::<Runtime>::insert(netuid, 1u16);
}

#[test]
fn sim_swap_pure_real_runtime_basic_buy_agrees() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        set_reserves(netuid, 1_000_000_000_000, 4_000_000_000_000);
        set_dynamic_mechanism(netuid);
        assert_ok!(SwapPallet::<Runtime>::maybe_initialize_v3(netuid));

        let order = pallet_subtensor::GetAlphaForTao::<Runtime>::with_amount(1_000_000_u64);

        let result_sim = SwapPallet::<Runtime>::sim_swap(netuid, order.clone())
            .expect("sim_swap (buy) must succeed");
        let result_pure = SwapPallet::<Runtime>::sim_swap_pure(netuid, order)
            .expect("sim_swap_pure (buy) must succeed");

        assert_eq!(
            result_sim, result_pure,
            "sim_swap and sim_swap_pure must agree for buy with real runtime reserves"
        );
    });
}

#[test]
fn sim_swap_pure_real_runtime_basic_sell_agrees() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        set_reserves(netuid, 1_000_000_000_000, 4_000_000_000_000);
        set_dynamic_mechanism(netuid);
        assert_ok!(SwapPallet::<Runtime>::maybe_initialize_v3(netuid));

        let order = pallet_subtensor::GetTaoForAlpha::<Runtime>::with_amount(4_000_000_u64);

        let result_sim = SwapPallet::<Runtime>::sim_swap(netuid, order.clone())
            .expect("sim_swap (sell) must succeed");
        let result_pure = SwapPallet::<Runtime>::sim_swap_pure(netuid, order)
            .expect("sim_swap_pure (sell) must succeed");

        assert_eq!(
            result_sim, result_pure,
            "sim_swap and sim_swap_pure must agree for sell with real runtime reserves"
        );
    });
}

#[test]
fn sim_swap_pure_real_runtime_uninitialized_with_provided_reserve_agrees() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        // Write split reserves: TaoReserve::reserve() = 600T + 400T = 1T
        //                        AlphaReserve::reserve() = 2T + 2T = 4T
        SubnetTAO::<Runtime>::insert(netuid, TaoBalance::from(600_000_000_000_u64));
        SubnetTaoProvided::<Runtime>::insert(netuid, TaoBalance::from(400_000_000_000_u64));
        SubnetAlphaIn::<Runtime>::insert(netuid, AlphaBalance::from(2_000_000_000_000_u64));
        SubnetAlphaInProvided::<Runtime>::insert(netuid, AlphaBalance::from(2_000_000_000_000_u64));
        set_dynamic_mechanism(netuid);

        // Pool must remain uninitialized for this test.
        assert!(
            !SwapV3Initialized::<Runtime>::get(netuid),
            "pool must be uninitialized"
        );

        // Buy direction.
        let buy_order =
            pallet_subtensor::GetAlphaForTao::<Runtime>::with_amount(1_000_000_u64);

        let result_sim_buy = SwapPallet::<Runtime>::sim_swap(netuid, buy_order.clone())
            .expect("sim_swap (buy, uninitialized) must succeed");
        let result_pure_buy = SwapPallet::<Runtime>::sim_swap_pure(netuid, buy_order)
            .expect("sim_swap_pure (buy, uninitialized) must succeed");

        assert_eq!(
            result_sim_buy, result_pure_buy,
            "sim_swap and sim_swap_pure must agree for buy with split provided reserves (uninitialized pool)"
        );

        // Sell direction.
        let sell_order =
            pallet_subtensor::GetTaoForAlpha::<Runtime>::with_amount(1_000_000_u64);

        let result_sim_sell = SwapPallet::<Runtime>::sim_swap(netuid, sell_order.clone())
            .expect("sim_swap (sell, uninitialized) must succeed");
        let result_pure_sell = SwapPallet::<Runtime>::sim_swap_pure(netuid, sell_order)
            .expect("sim_swap_pure (sell, uninitialized) must succeed");

        assert_eq!(
            result_sim_sell, result_pure_sell,
            "sim_swap and sim_swap_pure must agree for sell with split provided reserves (uninitialized pool)"
        );
    });
}

#[test]
fn sim_swap_pure_real_runtime_large_reserves_agrees() {
    new_test_ext().execute_with(|| {
        let netuid = NetUid::from(1);

        set_reserves(netuid, u64::MAX / 4, u64::MAX / 4);
        set_dynamic_mechanism(netuid);

        // Pool is intentionally left uninitialized — exercise the bootstrap path.
        assert!(
            !SwapV3Initialized::<Runtime>::get(netuid),
            "pool must be uninitialized"
        );

        // Buy direction.
        let buy_order =
            pallet_subtensor::GetAlphaForTao::<Runtime>::with_amount(1_000_000_u64);

        let result_sim_buy = SwapPallet::<Runtime>::sim_swap(netuid, buy_order.clone())
            .expect("sim_swap (buy, large reserves) must succeed");
        let result_pure_buy = SwapPallet::<Runtime>::sim_swap_pure(netuid, buy_order)
            .expect("sim_swap_pure (buy, large reserves) must succeed");

        assert_eq!(
            result_sim_buy, result_pure_buy,
            "sim_swap and sim_swap_pure must agree for buy with large reserves"
        );

        // Sell direction.
        let sell_order =
            pallet_subtensor::GetTaoForAlpha::<Runtime>::with_amount(1_000_000_u64);

        let result_sim_sell = SwapPallet::<Runtime>::sim_swap(netuid, sell_order.clone())
            .expect("sim_swap (sell, large reserves) must succeed");
        let result_pure_sell = SwapPallet::<Runtime>::sim_swap_pure(netuid, sell_order)
            .expect("sim_swap_pure (sell, large reserves) must succeed");

        assert_eq!(
            result_sim_sell, result_pure_sell,
            "sim_swap and sim_swap_pure must agree for sell with large reserves"
        );
    });
}
