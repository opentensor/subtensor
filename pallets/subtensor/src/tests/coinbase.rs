#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use super::mock::*;

use crate::tests::mock;
use crate::*;
use alloc::collections::BTreeMap;
use approx::assert_abs_diff_eq;
use frame_support::assert_ok;
use pallet_subtensor_swap::position::PositionId;
use sp_core::U256;
use substrate_fixed::types::{I64F64, I96F32, U96F32};
use subtensor_runtime_common::AlphaCurrency;
use subtensor_swap_interface::SwapHandler;

#[allow(clippy::arithmetic_side_effects)]
fn close(value: u64, target: u64, eps: u64) {
    assert!(
        (value as i64 - target as i64).abs() < eps as i64,
        "Assertion failed: value = {value}, target = {target}, eps = {eps}"
    )
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_hotkey_take --exact --show-output --nocapture
#[test]
fn test_hotkey_take() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        Delegates::<Test>::insert(hotkey, u16::MAX / 2);
        log::info!(
            "expected: {:?}",
            SubtensorModule::get_hotkey_take_float(&hotkey)
        );
        log::info!(
            "expected: {:?}",
            SubtensorModule::get_hotkey_take_float(&hotkey)
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_dynamic_function_various_values --exact --show-output --nocapture
#[test]
fn test_dynamic_function_various_values() {
    new_test_ext(1).execute_with(|| {
        let price_values: [f64; 9] = [0.001, 0.1, 0.5, 1.0, 2.0, 10.0, 100.0, 200.0, 1000.0];
        let tao_in_values: [u64; 9] = [0, 1, 10, 100, 1_000, 1_000_000, 1_000_000_000, 1_000_000_000_000, 1_000_000_000_000_000 ];
        let alpha_emission_values: [u64; 9] = [0, 1, 10, 100, 1_000, 1_000_000, 1_000_000_000, 1_000_000_000_000, 1_000_000_000_000_000 ];

        for &price in price_values.iter() {
            for &tao_in in tao_in_values.iter() {
                for &alpha_emission in alpha_emission_values.iter() {
                    // Set the price.
                    SubnetMechanism::<Test>::insert(NetUid::from(1), 1);
                    SubnetTAO::<Test>::insert(NetUid::from(1), TaoCurrency::from((price * 1_000_000_000.0) as u64));
                    SubnetAlphaIn::<Test>::insert(NetUid::from(1), AlphaCurrency::from(1_000_000_000));
                    let (tao_in_emission, alpha_in_emission, alpha_out_emission) = SubtensorModule::get_dynamic_tao_emission(1.into(), tao_in, alpha_emission);
                    assert!(tao_in_emission <= tao_in, "tao_in_emission is greater than tao_in");
                    assert!(alpha_in_emission <= alpha_emission, "alpha_in_emission is greater than alpha_emission");
                    assert!(alpha_out_emission <= 2 * alpha_emission, "alpha_out_emission is greater than 2 * alpha_emission");
                    assert!((alpha_in_emission + alpha_out_emission) <= 2 * alpha_emission, "Sum of alpha_in_emission and alpha_out_emission is less than or equal to. 2 * alpha_emission");
                    close( alpha_in_emission + alpha_out_emission, alpha_in_emission + alpha_emission, 10 );
                    // if alpha_in_emission > 0 || tao_in_emission > 0 {
                    //     assert!((tao_in_emission as f64 / alpha_in_emission as f64 - price).abs() < 1e-1, "Ratio of tao_in_emission to alpha_in_emission is not equal to price");
                    // }
                }
            }
        }
    });
}

// Test the base case of running coinbase with zero emission.
// This test verifies that the coinbase mechanism can handle the edge case
// of zero emission without errors or unexpected behavior.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_basecase --exact --show-output --nocapture
#[test]
fn test_coinbase_basecase() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::run_coinbase(U96F32::from_num(0.0));
    });
}

// Test the emission distribution for a single subnet.
// This test verifies that:
// - A single subnet receives the full emission amount
// - The emission is correctly reflected in SubnetTAO
// - Total issuance and total stake are updated appropriately
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_tao_issuance_base --exact --show-output --nocapture
#[test]
fn test_coinbase_tao_issuance_base() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let emission = TaoCurrency::from(1_234_567);
        add_network(netuid, 1, 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid), TaoCurrency::ZERO);
        SubtensorModule::run_coinbase(U96F32::from_num(emission));
        assert_eq!(SubnetTAO::<Test>::get(netuid), emission);
        assert_eq!(TotalIssuance::<Test>::get(), emission);
        assert_eq!(TotalStake::<Test>::get(), emission);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_tao_issuance_base_low --exact --show-output --nocapture
#[test]
fn test_coinbase_tao_issuance_base_low() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let emission = TaoCurrency::from(1);
        add_network(netuid, 1, 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid), TaoCurrency::ZERO);
        SubtensorModule::run_coinbase(U96F32::from_num(emission));
        assert_eq!(SubnetTAO::<Test>::get(netuid), emission);
        assert_eq!(TotalIssuance::<Test>::get(), emission);
        assert_eq!(TotalStake::<Test>::get(), emission);
    });
}

// Test emission distribution across multiple subnets.
// This test verifies that:
// - Multiple subnets receive equal portions of the total emission
// - Each subnet's TAO balance is updated correctly
// - Total issuance and total stake reflect the full emission amount
// - The emission is split evenly between all subnets
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_tao_issuance_multiple --exact --show-output --nocapture
#[test]
fn test_coinbase_tao_issuance_multiple() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let netuid3 = NetUid::from(3);
        let emission = TaoCurrency::from(3_333_333);
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        add_network(netuid3, 1, 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid1), TaoCurrency::ZERO);
        assert_eq!(SubnetTAO::<Test>::get(netuid2), TaoCurrency::ZERO);
        assert_eq!(SubnetTAO::<Test>::get(netuid3), TaoCurrency::ZERO);
        SubtensorModule::run_coinbase(U96F32::from_num(emission));
        assert_eq!(SubnetTAO::<Test>::get(netuid1), emission / 3.into());
        assert_eq!(SubnetTAO::<Test>::get(netuid2), emission / 3.into());
        assert_eq!(SubnetTAO::<Test>::get(netuid3), emission / 3.into());
        assert_eq!(TotalIssuance::<Test>::get(), emission);
        assert_eq!(TotalStake::<Test>::get(), emission);
    });
}

// Test emission distribution with different subnet prices.
// This test verifies that:
// - Subnets with different prices receive proportional emission shares
// - A subnet with double the price receives double the emission
// - Total issuance and total stake reflect the full emission amount
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_tao_issuance_different_prices --exact --show-output --nocapture
#[test]
fn test_coinbase_tao_issuance_different_prices() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let emission = 100_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);

        // Setup prices 0.1 and 0.2
        let initial_tao: u64 = 100_000_u64;
        let initial_alpha1: u64 = initial_tao * 10;
        let initial_alpha2: u64 = initial_tao * 5;
        mock::setup_reserves(netuid1, initial_tao.into(), initial_alpha1.into());
        mock::setup_reserves(netuid2, initial_tao.into(), initial_alpha2.into());

        // Force the swap to initialize
        SubtensorModule::swap_tao_for_alpha(
            netuid1,
            TaoCurrency::ZERO,
            1_000_000_000_000.into(),
            false,
        )
        .unwrap();
        SubtensorModule::swap_tao_for_alpha(
            netuid2,
            TaoCurrency::ZERO,
            1_000_000_000_000.into(),
            false,
        )
        .unwrap();

        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);

        // Set subnet prices.
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));

        // Assert initial TAO reserves.
        assert_eq!(SubnetTAO::<Test>::get(netuid1), initial_tao.into());
        assert_eq!(SubnetTAO::<Test>::get(netuid2), initial_tao.into());

        // Run the coinbase with the emission amount.
        SubtensorModule::run_coinbase(U96F32::from_num(emission));

        // Assert tao emission is split evenly.
        assert_abs_diff_eq!(
            SubnetTAO::<Test>::get(netuid1),
            TaoCurrency::from(initial_tao + emission / 3),
            epsilon = 1.into(),
        );
        assert_abs_diff_eq!(
            SubnetTAO::<Test>::get(netuid2),
            TaoCurrency::from(initial_tao + 2 * emission / 3),
            epsilon = 1.into(),
        );

        // Prices are low => we limit tao issued (buy alpha with it)
        let tao_issued = TaoCurrency::from(((0.1 + 0.2) * emission as f64) as u64);
        assert_abs_diff_eq!(
            TotalIssuance::<Test>::get(),
            tao_issued,
            epsilon = 10.into()
        );
        assert_abs_diff_eq!(
            TotalStake::<Test>::get(),
            emission.into(),
            epsilon = 10.into()
        );
    });
}

// Test moving price updates with different alpha values.
// This test verifies that:
// - Moving price stays constant when alpha is 1.0
// - Moving price converges to real price at expected rate with alpha 0.1
// - Moving price updates correctly over multiple iterations
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_moving_prices --exact --show-output --nocapture
#[test]
fn test_coinbase_moving_prices() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        // Set price to 1.0
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(1_000_000));
        SubnetAlphaIn::<Test>::insert(netuid, AlphaCurrency::from(1_000_000));
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(1));
        FirstEmissionBlockNumber::<Test>::insert(netuid, 1);

        // Updating the moving price keeps it the same.
        assert_eq!(
            SubtensorModule::get_moving_alpha_price(netuid),
            I96F32::from_num(1)
        );
        // Skip some blocks so that EMA price is not slowed down
        System::set_block_number(7_200_000);

        SubtensorModule::update_moving_price(netuid);
        assert_eq!(
            SubtensorModule::get_moving_alpha_price(netuid),
            I96F32::from_num(1)
        );
        // Check alpha of 1.
        // Set price to zero.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0));
        SubnetMovingAlpha::<Test>::set(I96F32::from_num(1.0));
        // Run moving 1 times.
        SubtensorModule::update_moving_price(netuid);
        // Assert price is ~ 100% of the real price.
        assert!(U96F32::from_num(1.0) - SubtensorModule::get_moving_alpha_price(netuid) < 0.05);
        // Set price to zero.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0));
        SubnetMovingAlpha::<Test>::set(I96F32::from_num(0.1));

        // EMA price 28 days after registration
        System::set_block_number(7_200 * 28);

        // Run moving 14 times.
        for _ in 0..14 {
            SubtensorModule::update_moving_price(netuid);
        }

        // Assert price is > 50% of the real price.
        assert_abs_diff_eq!(
            0.512325,
            SubtensorModule::get_moving_alpha_price(netuid).to_num::<f64>(),
            epsilon = 0.001
        );
    });
}

// Test moving price updates slow down at the beginning.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_update_moving_price_initial --exact --show-output --nocapture
#[test]
fn test_update_moving_price_initial() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        // Set current price to 1.0
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(1_000_000));
        SubnetAlphaIn::<Test>::insert(netuid, AlphaCurrency::from(1_000_000));
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetMovingAlpha::<Test>::set(I96F32::from_num(0.5));
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0));

        // Registered recently
        System::set_block_number(510);
        FirstEmissionBlockNumber::<Test>::insert(netuid, 500);

        SubtensorModule::update_moving_price(netuid);

        let new_price = SubnetMovingPrice::<Test>::get(netuid);
        assert!(new_price.to_num::<f64>() < 0.001);
    });
}

// Test moving price updates slow down at the beginning.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_update_moving_price_after_time --exact --show-output --nocapture
#[test]
fn test_update_moving_price_after_time() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        // Set current price to 1.0
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(1_000_000));
        SubnetAlphaIn::<Test>::insert(netuid, AlphaCurrency::from(1_000_000));
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetMovingAlpha::<Test>::set(I96F32::from_num(0.5));
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0));

        // Registered long time ago
        System::set_block_number(144_000_500);
        FirstEmissionBlockNumber::<Test>::insert(netuid, 500);

        SubtensorModule::update_moving_price(netuid);

        let new_price = SubnetMovingPrice::<Test>::get(netuid);
        assert!((new_price.to_num::<f64>() - 0.5).abs() < 0.001);
    });
}

// Test basic alpha issuance in coinbase mechanism.
// This test verifies that:
// - Alpha issuance is initialized to 0 for new subnets
// - Alpha issuance is split evenly between subnets during coinbase
// - Each subnet receives the expected fraction of total emission
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance_base --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_base() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Set up prices 1 and 1
        let initial: u64 = 1_000_000;
        SubnetTAO::<Test>::insert(netuid1, TaoCurrency::from(initial));
        SubnetAlphaIn::<Test>::insert(netuid1, AlphaCurrency::from(initial));
        SubnetTAO::<Test>::insert(netuid2, TaoCurrency::from(initial));
        SubnetAlphaIn::<Test>::insert(netuid2, AlphaCurrency::from(initial));
        // Check initial
        SubtensorModule::run_coinbase(U96F32::from_num(emission));
        // tao_in = 500_000
        // alpha_in = 500_000/price = 500_000
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid1),
            (initial + emission / 2).into()
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid2),
            (initial + emission / 2).into()
        );
    });
}

// Test alpha issuance with different subnet prices.
// This test verifies that:
// - Alpha issuance is proportional to subnet prices
// - Higher priced subnets receive more TAO emission
// - Alpha issuance is correctly calculated based on price ratios
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance_different --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_different() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);
        // Setup prices 1 and 1
        let initial: u64 = 1_000_000;
        SubnetTAO::<Test>::insert(netuid1, TaoCurrency::from(initial));
        SubnetAlphaIn::<Test>::insert(netuid1, AlphaCurrency::from(initial));
        SubnetTAO::<Test>::insert(netuid2, TaoCurrency::from(initial));
        SubnetAlphaIn::<Test>::insert(netuid2, AlphaCurrency::from(initial));
        // Set subnet prices.
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));
        // Run coinbase
        SubtensorModule::run_coinbase(U96F32::from_num(emission));
        // tao_in = 333_333
        // alpha_in = 333_333/price = 333_333 + initial
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid1),
            (initial + emission / 3).into()
        );
        // tao_in = 666_666
        // alpha_in = 666_666/price = 666_666 + initial
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid2),
            (initial + emission / 3 + emission / 3).into()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance_with_cap_trigger --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_with_cap_trigger() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);
        // Setup prices 1000000
        let initial: u64 = 1_000;
        let initial_alpha: u64 = initial * 1000000;
        SubnetTAO::<Test>::insert(netuid1, TaoCurrency::from(initial));
        SubnetAlphaIn::<Test>::insert(netuid1, AlphaCurrency::from(initial_alpha)); // Make price extremely low.
        SubnetTAO::<Test>::insert(netuid2, TaoCurrency::from(initial));
        SubnetAlphaIn::<Test>::insert(netuid2, AlphaCurrency::from(initial_alpha)); // Make price extremely low.
        // Set subnet prices.
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));
        // Run coinbase
        SubtensorModule::run_coinbase(U96F32::from_num(emission));
        // tao_in = 333_333
        // alpha_in = 333_333/price > 1_000_000_000 --> 1_000_000_000 + initial_alpha
        assert!(SubnetAlphaIn::<Test>::get(netuid1) < (initial_alpha + 1_000_000_000).into());
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid2), 1_000_000_000.into());
        // tao_in = 666_666
        // alpha_in = 666_666/price > 1_000_000_000 --> 1_000_000_000 + initial_alpha
        assert!(SubnetAlphaIn::<Test>::get(netuid2) < (initial_alpha + 1_000_000_000).into());
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid2), 1_000_000_000.into()); // Gets full block emission.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance_with_cap_trigger_and_block_emission --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_with_cap_trigger_and_block_emission() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);

        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);

        // Setup prices 0.000001
        let initial_tao: u64 = 10_000_u64;
        let initial_alpha: u64 = initial_tao * 100_000_u64;
        mock::setup_reserves(netuid1, initial_tao.into(), initial_alpha.into());
        mock::setup_reserves(netuid2, initial_tao.into(), initial_alpha.into());

        // Enable emission
        FirstEmissionBlockNumber::<Test>::insert(netuid1, 0);
        FirstEmissionBlockNumber::<Test>::insert(netuid2, 0);
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));

        // Force the swap to initialize
        SubtensorModule::swap_tao_for_alpha(
            netuid1,
            TaoCurrency::ZERO,
            1_000_000_000_000.into(),
            false,
        )
        .unwrap();
        SubtensorModule::swap_tao_for_alpha(
            netuid2,
            TaoCurrency::ZERO,
            1_000_000_000_000.into(),
            false,
        )
        .unwrap();

        // Get the prices before the run_coinbase
        let price_1_before = <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid1);
        let price_2_before = <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid2);

        // Set issuance at 21M
        SubnetAlphaOut::<Test>::insert(netuid1, AlphaCurrency::from(21_000_000_000_000_000)); // Set issuance above 21M
        SubnetAlphaOut::<Test>::insert(netuid2, AlphaCurrency::from(21_000_000_000_000_000)); // Set issuance above 21M

        // Run coinbase
        SubtensorModule::run_coinbase(U96F32::from_num(emission));

        // Get the prices after the run_coinbase
        let price_1_after = <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid1);
        let price_2_after = <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid2);

        // AlphaIn gets decreased beacuse of a buy
        assert!(u64::from(SubnetAlphaIn::<Test>::get(netuid1)) < initial_alpha);
        assert_eq!(
            u64::from(SubnetAlphaOut::<Test>::get(netuid2)),
            21_000_000_000_000_000_u64
        );
        assert!(u64::from(SubnetAlphaIn::<Test>::get(netuid2)) < initial_alpha);
        assert_eq!(
            u64::from(SubnetAlphaOut::<Test>::get(netuid2)),
            21_000_000_000_000_000_u64
        );

        assert!(price_1_after > price_1_before);
        assert!(price_2_after > price_2_before);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_owner_cut_base --exact --show-output --nocapture
#[test]
fn test_owner_cut_base() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        mock::setup_reserves(netuid, 1_000_000_000_000.into(), 1_000_000_000_000.into());
        SubtensorModule::set_tempo(netuid, 10000); // Large number (dont drain)
        SubtensorModule::set_subnet_owner_cut(0);
        SubtensorModule::run_coinbase(U96F32::from_num(0));
        assert_eq!(PendingOwnerCut::<Test>::get(netuid), 0.into()); // No cut
        SubtensorModule::set_subnet_owner_cut(u16::MAX);
        SubtensorModule::run_coinbase(U96F32::from_num(0));
        assert_eq!(PendingOwnerCut::<Test>::get(netuid), 1_000_000_000.into()); // Full cut.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_pending_swapped --exact --show-output --nocapture
#[test]
fn test_pending_swapped() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let emission: u64 = 1_000_000;
        add_network(netuid, 1, 0);
        mock::setup_reserves(netuid, 1_000_000.into(), 1.into());
        SubtensorModule::run_coinbase(U96F32::from_num(0));
        assert_eq!(PendingAlphaSwapped::<Test>::get(netuid), 0.into()); // Zero tao weight and no root.
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000_000)); // Add root weight.
        SubtensorModule::run_coinbase(U96F32::from_num(0));
        assert_eq!(PendingAlphaSwapped::<Test>::get(netuid), 0.into()); // Zero tao weight with 1 root.
        SubtensorModule::set_tempo(netuid, 10000); // Large number (dont drain)
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtensorModule::run_coinbase(U96F32::from_num(0));
        // 1 TAO / ( 1 + 3 ) = 0.25 * 1 / 2 = 125000000
        assert_abs_diff_eq!(
            u64::from(PendingAlphaSwapped::<Test>::get(netuid)),
            125000000,
            epsilon = 1
        );
        assert_abs_diff_eq!(
            u64::from(PendingEmission::<Test>::get(netuid)),
            1_000_000_000 - 125000000,
            epsilon = 1
        ); // 1 - swapped.
        assert_abs_diff_eq!(
            u64::from(PendingRootDivs::<Test>::get(netuid)),
            125000000,
            epsilon = 1
        ); // swapped * (price = 1)
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base --exact --show-output --nocapture
#[test]
fn test_drain_base() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::drain_pending_emission(
            0.into(),
            AlphaCurrency::ZERO,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        )
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        SubtensorModule::drain_pending_emission(
            netuid,
            AlphaCurrency::ZERO,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        )
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_single_staker_not_registered --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_single_staker_not_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_before,
        );
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha.into(),
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let stake_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(stake_before, stake_after); // Not registered.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_single_staker_registered --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_single_staker_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_before,
        );
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let stake_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        close(
            (stake_before + pending_alpha).into(),
            stake_after.into(),
            10,
        ); // Registered gets all emission.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_single_staker_registered_root_weight --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_single_staker_registered_root_weight() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        // register_ok_neuron(root, hotkey, coldkey, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        Delegates::<Test>::insert(hotkey, 0);
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_before,
        );
        let pending_tao = TaoCurrency::from(1_000_000_000);
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), TaoCurrency::ZERO);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            pending_tao,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let stake_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let root_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
        );
        close(
            (stake_before + pending_alpha).into(),
            stake_after.into(),
            10,
        ); // Registered gets all alpha emission.
        close(
            stake_before.to_u64() + pending_tao.to_u64(),
            root_after.into(),
            10,
        ); // Registered gets all tao emission
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), pending_tao);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        register_ok_neuron(netuid, hotkey1, coldkey, 0);
        register_ok_neuron(netuid, hotkey2, coldkey, 0);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            netuid,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            netuid,
            stake_before,
        );
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        close(
            (stake_before + pending_alpha / 2.into()).into(),
            stake_after1.into(),
            10,
        ); // Registered gets 1/2 emission
        close(
            (stake_before + pending_alpha / 2.into()).into(),
            stake_after2.into(),
            10,
        ); // Registered gets 1/2 emission.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered_and_root --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered_and_root() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        register_ok_neuron(netuid, hotkey1, coldkey, 0);
        register_ok_neuron(netuid, hotkey2, coldkey, 0);
        Delegates::<Test>::insert(hotkey1, 0);
        Delegates::<Test>::insert(hotkey2, 0);
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            netuid,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            NetUid::ROOT,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            netuid,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            NetUid::ROOT,
            stake_before,
        );
        let pending_tao = TaoCurrency::from(1_000_000_000);
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), TaoCurrency::ZERO);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            pending_tao,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let root_after1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            NetUid::ROOT,
        );
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        let root_after2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            NetUid::ROOT,
        );
        close(
            (stake_before + pending_alpha / 2.into()).into(),
            stake_after1.into(),
            10,
        ); // Registered gets 1/2 emission
        close(
            (stake_before + pending_alpha / 2.into()).into(),
            stake_after2.into(),
            10,
        ); // Registered gets 1/2 emission.
        close(
            stake_before.to_u64() + pending_tao.to_u64() / 2,
            root_after1.into(),
            10,
        ); // Registered gets 1/2 tao emission
        close(
            stake_before.to_u64() + pending_tao.to_u64() / 2,
            root_after2.into(),
            10,
        ); // Registered gets 1/2 tao emission
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), pending_tao);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        Delegates::<Test>::insert(hotkey1, 0);
        Delegates::<Test>::insert(hotkey2, 0);
        register_ok_neuron(netuid, hotkey1, coldkey, 0);
        register_ok_neuron(netuid, hotkey2, coldkey, 0);
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            netuid,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            NetUid::ROOT,
            stake_before * 2.into(), // Hotkey 1 has twice as much root weight.
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            netuid,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            NetUid::ROOT,
            stake_before,
        );
        let pending_tao = TaoCurrency::from(1_000_000_000);
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), TaoCurrency::ZERO);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            pending_tao,
            0.into(),
            0.into(),
        );
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let root_after1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            NetUid::ROOT,
        );
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        let root_after2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            NetUid::ROOT,
        );
        let expected_stake = I96F32::from_num(stake_before)
            + (I96F32::from_num(pending_alpha) * I96F32::from_num(1.0 / 2.0));
        assert_abs_diff_eq!(
            expected_stake.to_num::<u64>(),
            stake_after1.into(),
            epsilon = 10
        ); // Registered gets 50% of alpha emission
        let expected_stake2 = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_alpha) * I96F32::from_num(1.0 / 2.0);
        assert_abs_diff_eq!(
            expected_stake2.to_num::<u64>(),
            stake_after2.into(),
            epsilon = 10
        ); // Registered gets 50% emission
        let expected_root1 = I96F32::from_num(2 * u64::from(stake_before))
            + I96F32::from_num(pending_tao.to_u64()) * I96F32::from_num(2.0 / 3.0);
        assert_abs_diff_eq!(
            expected_root1.to_num::<u64>(),
            root_after1.into(),
            epsilon = 10
        ); // Registered gets 2/3 tao emission
        let expected_root2 = I96F32::from_num(u64::from(stake_before))
            + I96F32::from_num(pending_tao.to_u64()) * I96F32::from_num(1.0 / 3.0);
        assert_abs_diff_eq!(
            expected_root2.to_num::<u64>(),
            root_after2.into(),
            epsilon = 10
        ); // Registered gets 1/3 tao emission
        assert_abs_diff_eq!(
            SubnetTAO::<Test>::get(NetUid::ROOT),
            pending_tao,
            epsilon = 10.into()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts_half_tao_weight --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts_half_tao_weight()
 {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        Delegates::<Test>::insert(hotkey1, 0);
        Delegates::<Test>::insert(hotkey2, 0);
        register_ok_neuron(netuid, hotkey1, coldkey, 0);
        register_ok_neuron(netuid, hotkey2, coldkey, 0);
        SubtensorModule::set_tao_weight(u64::MAX / 2); // Set TAO weight to 0.5
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            netuid,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            NetUid::ROOT,
            stake_before * 2.into(), // Hotkey 1 has twice as much root weight.
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            netuid,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            NetUid::ROOT,
            stake_before,
        );
        let pending_tao = TaoCurrency::from(1_000_000_000);
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), TaoCurrency::ZERO);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            pending_tao,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let root_after1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey,
            NetUid::ROOT,
        );
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        let root_after2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &coldkey,
            NetUid::ROOT,
        );
        let expected_stake = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_alpha) * I96F32::from_num(1.0 / 2.0);
        assert_abs_diff_eq!(
            expected_stake.to_num::<u64>(),
            u64::from(stake_after1),
            epsilon = 10
        );
        let expected_stake2 = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_alpha) * I96F32::from_num(1.0 / 2.0);
        assert_abs_diff_eq!(
            expected_stake2.to_num::<u64>(),
            u64::from(stake_after2),
            epsilon = 10
        );
        // hotkey 1 has 2 / 3 root tao
        let expected_root1 = I96F32::from_num(2 * u64::from(stake_before))
            + I96F32::from_num(pending_tao) * I96F32::from_num(2.0 / 3.0);
        assert_abs_diff_eq!(
            expected_root1.to_num::<u64>(),
            u64::from(root_after1),
            epsilon = 10
        );
        // hotkey 1 has 1 / 3 root tao
        let expected_root2 = I96F32::from_num(u64::from(stake_before))
            + I96F32::from_num(pending_tao) * I96F32::from_num(1.0 / 3.0);
        assert_abs_diff_eq!(
            expected_root2.to_num::<u64>(),
            u64::from(root_after2),
            epsilon = 10
        );
        assert_abs_diff_eq!(
            SubnetTAO::<Test>::get(NetUid::ROOT),
            pending_tao,
            epsilon = 10.into()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_alpha_childkey_parentkey --exact --show-output --nocapture
#[test]
fn test_drain_alpha_childkey_parentkey() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        SubtensorModule::set_ck_burn(0);
        let parent = U256::from(1);
        let child = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        register_ok_neuron(netuid, child, coldkey, 0);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &parent,
            &coldkey,
            netuid,
            stake_before,
        );
        mock_set_children_no_epochs(netuid, &parent, &[(u64::MAX, child)]);

        // Childkey take is 10%
        ChildkeyTake::<Test>::insert(child, netuid, u16::MAX / 10);

        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let parent_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&parent, netuid);
        let child_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&child, netuid);

        // Child gets 10%, parent gets 90%
        let expected = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_alpha) * I96F32::from_num(9.0 / 10.0);
        log::info!(
            "expected: {:?}, parent_stake_after: {:?}",
            expected.to_num::<u64>(),
            parent_stake_after
        );
        close(expected.to_num::<u64>(), parent_stake_after.into(), 10_000);
        let expected = I96F32::from_num(u64::from(pending_alpha)) / I96F32::from_num(10);
        close(expected.to_num::<u64>(), child_stake_after.into(), 10_000);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children --exact --show-output --nocapture
#[test]
fn test_get_root_children() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        // Set TAO weight to 1.
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.

        // Create keys.
        let cold = U256::from(0);
        let alice = U256::from(1);
        let bob = U256::from(2);

        // Register Alice and Bob to the root network and alpha subnet.
        register_ok_neuron(alpha, alice, cold, 0);
        register_ok_neuron(alpha, bob, cold, 0);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold).clone(),
            alice,
        ));
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold).clone(),
            bob,
        ));

        // Add stake for Alice and Bob on root.
        let alice_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold,
            NetUid::ROOT,
            alice_root_stake,
        );
        let bob_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold,
            NetUid::ROOT,
            alice_root_stake,
        );

        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold,
            alpha,
            bob_alpha_stake,
        );

        // Set Bob as 100% child of Alice on root.
        // mock_set_children_no_epochs( NetUid::ROOT, &alice, &[(u64::MAX, bob)]);
        mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX, bob)]);

        // Assert Alice and Bob stake on root and netuid
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&alice, NetUid::ROOT),
            alice_root_stake
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&bob, NetUid::ROOT),
            bob_root_stake
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&alice, alpha),
            alice_alpha_stake
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&bob, alpha),
            bob_alpha_stake
        );

        // Assert Alice and Bob inherited stakes
        assert_eq!(
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&alice, NetUid::ROOT),
            alice_root_stake
        );
        assert_eq!(
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&alice, alpha),
            0.into()
        );
        assert_eq!(
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&bob, NetUid::ROOT),
            bob_root_stake
        );
        assert_eq!(
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&bob, alpha),
            bob_alpha_stake + alice_alpha_stake
        );

        // Assert Alice and Bob TAO inherited stakes
        assert_eq!(
            SubtensorModule::get_tao_inherited_for_hotkey_on_subnet(&alice, alpha),
            TaoCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_tao_inherited_for_hotkey_on_subnet(&bob, alpha),
            u64::from(bob_root_stake + alice_root_stake).into()
        );

        // Get Alice stake amounts on subnet alpha.
        let (alice_total, alice_alpha, alice_tao): (I64F64, I64F64, I64F64) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&alice, alpha);
        assert_eq!(alice_total, I64F64::from_num(0));

        // Get Bob stake amounts on subnet alpha.
        let (bob_total, bob_alpha, bob_tao): (I64F64, I64F64, I64F64) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&bob, alpha);
        assert_eq!(
            bob_total,
            I64F64::from_num(u64::from(bob_root_stake * 4.into()))
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        SubtensorModule::set_ck_burn(0);
        // Set TAO weight to 1.
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.
        // Create keys.
        let cold_alice = U256::from(0);
        let cold_bob = U256::from(1);
        let alice = U256::from(2);
        let bob = U256::from(3);
        // Register Alice and Bob to the root network and alpha subnet.
        register_ok_neuron(alpha, alice, cold_alice, 0);
        register_ok_neuron(alpha, bob, cold_bob, 0);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_alice).clone(),
            alice,
        ));
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_bob).clone(),
            bob,
        ));
        // Add stake for Alice and Bob on root.
        let alice_root_stake = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            NetUid::ROOT,
            alice_root_stake.into(),
        );
        let bob_root_stake = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            NetUid::ROOT,
            bob_root_stake.into(),
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            alpha,
            bob_alpha_stake,
        );
        // Set Bob as 100% child of Alice on root.
        mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX, bob)]);
        // Set Bob childkey take to zero.
        ChildkeyTake::<Test>::insert(bob, alpha, 0);
        Delegates::<Test>::insert(alice, 0);
        Delegates::<Test>::insert(bob, 0);

        // Get Alice stake amounts on subnet alpha.
        let (alice_total, alice_alpha, alice_tao): (I64F64, I64F64, I64F64) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&alice, alpha);
        assert_eq!(alice_total, I64F64::from_num(0));

        // Get Bob stake amounts on subnet alpha.
        let (bob_total, bob_alpha, bob_tao): (I64F64, I64F64, I64F64) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&bob, alpha);
        assert_eq!(bob_total, I64F64::from_num(4 * bob_root_stake));

        // Lets drain
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            alpha,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Alice and Bob both made half of the dividends.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&alice, alpha),
            alice_alpha_stake + pending_alpha / 2.into()
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&bob, alpha),
            bob_alpha_stake + pending_alpha / 2.into()
        );

        // There should be no TAO on the root subnet.
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), TaoCurrency::ZERO);

        // Lets drain
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        let pending_root1 = TaoCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            alpha,
            pending_alpha,
            pending_root1,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Alice and Bob both made half of the dividends.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&alice, NetUid::ROOT),
            AlphaCurrency::from(alice_root_stake + pending_root1.to_u64() / 2)
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&bob, NetUid::ROOT),
            AlphaCurrency::from(bob_root_stake + pending_root1.to_u64() / 2)
        );

        // The pending root dividends should be present in root subnet.
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), pending_root1);

        // Lets change the take value. (Bob is greedy.)
        ChildkeyTake::<Test>::insert(bob, alpha, u16::MAX);

        // Lets drain
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        let pending_root2 = TaoCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            alpha,
            pending_alpha,
            pending_root2,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Alice makes nothing
        assert_eq!(
            AlphaDividendsPerSubnet::<Test>::get(alpha, alice),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            TaoDividendsPerSubnet::<Test>::get(alpha, alice),
            TaoCurrency::ZERO
        );
        // Bob makes it all.
        assert_abs_diff_eq!(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob),
            pending_alpha,
            epsilon = 1.into()
        );
        assert_eq!(
            TaoDividendsPerSubnet::<Test>::get(alpha, bob),
            pending_root2
        );
        // The pending root dividends should be present in root subnet.
        assert_eq!(
            SubnetTAO::<Test>::get(NetUid::ROOT),
            pending_root1 + pending_root2
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain_half_proportion --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain_half_proportion() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        SubtensorModule::set_ck_burn(0);
        // Set TAO weight to 1.
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.
        // Create keys.
        let cold_alice = U256::from(0);
        let cold_bob = U256::from(1);
        let alice = U256::from(2);
        let bob = U256::from(3);
        // Register Alice and Bob to the root network and alpha subnet.
        register_ok_neuron(alpha, alice, cold_alice, 0);
        register_ok_neuron(alpha, bob, cold_bob, 0);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_alice).clone(),
            alice,
        ));
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_bob).clone(),
            bob,
        ));
        // Add stake for Alice and Bob on root.
        let alice_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            NetUid::ROOT,
            alice_root_stake,
        );
        let bob_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            NetUid::ROOT,
            alice_root_stake,
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            alpha,
            bob_alpha_stake,
        );
        // Set Bob as 100% child of Alice on root.
        mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX / 2, bob)]);

        // Set Bob childkey take to zero.
        ChildkeyTake::<Test>::insert(bob, alpha, 0);
        Delegates::<Test>::insert(alice, 0);
        Delegates::<Test>::insert(bob, 0);

        // Lets drain!
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            alpha,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Alice and Bob make the same amount.
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, alice).into(),
            (pending_alpha / 2.into()).into(),
            10,
        );
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob).into(),
            (pending_alpha / 2.into()).into(),
            10,
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain_with_take --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain_with_take() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        // Set TAO weight to 1.
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.
        // Create keys.
        let cold_alice = U256::from(0);
        let cold_bob = U256::from(1);
        let alice = U256::from(2);
        let bob = U256::from(3);
        // Register Alice and Bob to the root network and alpha subnet.
        register_ok_neuron(alpha, alice, cold_alice, 0);
        register_ok_neuron(alpha, bob, cold_bob, 0);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_alice).clone(),
            alice,
        ));
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_bob).clone(),
            bob,
        ));
        // Add stake for Alice and Bob on root.
        let alice_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            NetUid::ROOT,
            alice_root_stake,
        );
        let bob_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            NetUid::ROOT,
            alice_root_stake,
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            alpha,
            bob_alpha_stake,
        );
        // Set Bob as 100% child of Alice on root.
        ChildkeyTake::<Test>::insert(bob, alpha, u16::MAX);
        mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX, bob)]);
        // Set Bob validator take to zero.
        Delegates::<Test>::insert(alice, 0);
        Delegates::<Test>::insert(bob, 0);

        // Lets drain!
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            alpha,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Bob makes it all.
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, alice).into(),
            0,
            10,
        );
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob).into(),
            pending_alpha.into(),
            10,
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain_with_half_take --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain_with_half_take() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        // Set TAO weight to 1.
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.
        SubtensorModule::set_ck_burn(0);
        // Create keys.
        let cold_alice = U256::from(0);
        let cold_bob = U256::from(1);
        let alice = U256::from(2);
        let bob = U256::from(3);
        // Register Alice and Bob to the root network and alpha subnet.
        register_ok_neuron(alpha, alice, cold_alice, 0);
        register_ok_neuron(alpha, bob, cold_bob, 0);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_alice).clone(),
            alice,
        ));
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold_bob).clone(),
            bob,
        ));
        // Add stake for Alice and Bob on root.
        let alice_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            NetUid::ROOT,
            alice_root_stake,
        );
        let bob_root_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            NetUid::ROOT,
            alice_root_stake,
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            alpha,
            bob_alpha_stake,
        );
        // Set Bob as 100% child of Alice on root.
        ChildkeyTake::<Test>::insert(bob, alpha, u16::MAX / 2);
        mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX, bob)]);
        // Set Bob childkey take to zero.
        Delegates::<Test>::insert(alice, 0);
        Delegates::<Test>::insert(bob, 0);

        // Lets drain!
        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            alpha,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Alice and Bob make the same amount.
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, alice).into(),
            (pending_alpha / 4.into()).into(),
            10000,
        );
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob).into(),
            3 * u64::from(pending_alpha / 4.into()),
            10000,
        );
    });
}

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_with_weights --exact --show-output --nocapture
// #[test]
// fn test_get_root_children_with_weights() {
//     new_test_ext(1).execute_with(|| {
//         // Init netuid 1
//         let alpha = NetUid::from(1);
//         add_network(NetUid::ROOT, 1, 0);
//         add_network(alpha, 1, 0);
//         // Set TAO weight to 1.
//         SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.
//                                                    // Create keys.
//         let cold = U256::from(0);
//         let alice = U256::from(1);
//         let bob = U256::from(2);
//         // Register Alice and Bob to the root network and alpha subnet.
//         register_ok_neuron(alpha, alice, cold, 0);
//         register_ok_neuron(alpha, bob, cold, 0);
//         assert_ok!(SubtensorModule::root_register(
//             RuntimeOrigin::signed(cold).clone(),
//             alice,
//         ));
//         assert_ok!(SubtensorModule::root_register(
//             RuntimeOrigin::signed(cold).clone(),
//             bob,
//         ));
//         // Add stake for Alice and Bob on root.
//         let alice_root_stake = AlphaCurrency::from(1_000_000_000);
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &alice,
//             &cold,
//             NetUid::ROOT,
//             alice_root_stake,
//         );
//         let bob_root_stake = AlphaCurrency::from(1_000_000_000);
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &bob,
//             &cold,
//             NetUid::ROOT,
//             alice_root_stake,
//         );
//         // Add stake for Alice and Bob on netuid.
//         let alice_alpha_stake = AlphaCurrency::from(1_000_000_000);
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &alice,
//             &cold,
//             alpha,
//             alice_alpha_stake,
//         );
//         let bob_alpha_stake = AlphaCurrency::from(1_000_000_000);
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &bob,
//             &cold,
//             alpha,
//             bob_alpha_stake,
//         );
//         // Set Bob as 100% child of Alice on root.
//         mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX, bob)]);

//         // Set Bob childkey take to zero.
//         ChildkeyTake::<Test>::insert(bob, alpha, 0);
//         Delegates::<Test>::insert(alice, 0);
//         Delegates::<Test>::insert(bob, 0);

//         // Set weights on the subnet.
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(alice),
//             alpha,
//             vec![0, 1],
//             vec![1, 1],
//             0,
//         ));
//         assert_ok!(SubtensorModule::set_weights(
//             RuntimeOrigin::signed(bob),
//             alpha,
//             vec![0, 1],
//             vec![1, 1],
//             0,
//         ));

//         // Lets drain!
//         let pending_alpha = AlphaCurrency::from(1_000_000_000);
//         SubtensorModule::drain_pending_emission(alpha, pending_alpha, 0, 0.into(), 0.into());

//         // Alice and Bob make the same amount.
//         close(
//             AlphaDividendsPerSubnet::<Test>::get(alpha, alice),
//             pending_alpha / 2,
//             10,
//         );
//         close(
//             AlphaDividendsPerSubnet::<Test>::get(alpha, bob),
//             pending_alpha / 2,
//             10,
//         );
//     });
// }

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_incentive_to_subnet_owner_is_burned --exact --show-output --nocapture
#[test]
fn test_incentive_to_subnet_owner_is_burned() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_ck = U256::from(0);
        let subnet_owner_hk = U256::from(1);

        let other_ck = U256::from(2);
        let other_hk = U256::from(3);
        Owner::<Test>::insert(other_hk, other_ck);

        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha = AlphaCurrency::ZERO; // None to valis
        let owner_cut = AlphaCurrency::ZERO;
        let mut incentives: BTreeMap<U256, AlphaCurrency> = BTreeMap::new();

        // Give incentive to other_hk
        incentives.insert(other_hk, 10_000_000.into());

        // Give incentives to subnet_owner_hk
        incentives.insert(subnet_owner_hk, 10_000_000.into());

        // Verify stake before
        let subnet_owner_stake_before =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_before, 0.into());
        let other_stake_before = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);
        assert_eq!(other_stake_before, 0.into());

        // Distribute dividends and incentives
        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            owner_cut,
            incentives,
            BTreeMap::new(),
            BTreeMap::new(),
        );

        // Verify stake after
        let subnet_owner_stake_after =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_after, 0.into());
        let other_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);
        assert!(other_stake_after > 0.into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_incentive_to_subnet_owners_hotkey_is_burned --exact --show-output --nocapture
#[test]
fn test_incentive_to_subnet_owners_hotkey_is_burned() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_ck = U256::from(0);
        let subnet_owner_hk = U256::from(1);

        // Other hk owned by owner
        let other_hk = U256::from(3);
        Owner::<Test>::insert(other_hk, subnet_owner_ck);
        OwnedHotkeys::<Test>::insert(subnet_owner_ck, vec![subnet_owner_hk, other_hk]);

        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);
        Uids::<Test>::insert(netuid, other_hk, 1);

        // Set the burn key limit to 2
        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 2);

        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha = AlphaCurrency::ZERO; // None to valis
        let owner_cut = AlphaCurrency::ZERO;
        let mut incentives: BTreeMap<U256, AlphaCurrency> = BTreeMap::new();

        // Give incentive to other_hk
        incentives.insert(other_hk, 10_000_000.into());

        // Give incentives to subnet_owner_hk
        incentives.insert(subnet_owner_hk, 10_000_000.into());

        // Verify stake before
        let subnet_owner_stake_before =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_before, 0.into());
        let other_stake_before = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);
        assert_eq!(other_stake_before, 0.into());

        // Distribute dividends and incentives
        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            owner_cut,
            incentives,
            BTreeMap::new(),
            BTreeMap::new(),
        );

        // Verify stake after
        let subnet_owner_stake_after =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_after, 0.into());
        let other_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);
        assert_eq!(other_stake_after, 0.into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_incentive_to_subnet_owners_hotkey_is_burned_with_limit --exact --show-output --nocapture
#[test]
fn test_incentive_to_subnet_owners_hotkey_is_burned_with_limit() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_ck = U256::from(0);
        let subnet_owner_hk = U256::from(1);

        // Other hk owned by owner
        let other_hk = U256::from(3);
        Owner::<Test>::insert(other_hk, subnet_owner_ck);
        OwnedHotkeys::<Test>::insert(subnet_owner_ck, vec![subnet_owner_hk, other_hk]);

        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);
        Uids::<Test>::insert(netuid, other_hk, 1);

        // Set the burn key limit to 1 - testing the limits
        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 1);

        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha = AlphaCurrency::ZERO; // None to valis
        let owner_cut = AlphaCurrency::ZERO;
        let mut incentives: BTreeMap<U256, AlphaCurrency> = BTreeMap::new();

        // Give incentive to other_hk
        incentives.insert(other_hk, 10_000_000.into());

        // Give incentives to subnet_owner_hk
        incentives.insert(subnet_owner_hk, 10_000_000.into());

        // Verify stake before
        let subnet_owner_stake_before =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_before, 0.into());
        let other_stake_before = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);
        assert_eq!(other_stake_before, 0.into());

        // Distribute dividends and incentives
        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            owner_cut,
            incentives,
            BTreeMap::new(),
            BTreeMap::new(),
        );

        // Verify stake after
        let subnet_owner_stake_after =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_after, 0.into());
        let other_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);

        // Testing the limit - should be not burned
        assert!(other_stake_after > 0.into());
    });
}

// Test that if number of sn owner hotkeys is greater than ImmuneOwnerUidsLimit, then the ones with
// higher BlockAtRegistration are used to burn
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_burn_key_sorting --exact --show-output --nocapture
#[test]
fn test_burn_key_sorting() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_ck = U256::from(0);
        let subnet_owner_hk = U256::from(1);

        // Other hk owned by owner
        let other_hk_1 = U256::from(3);
        let other_hk_2 = U256::from(4);
        let other_hk_3 = U256::from(5);
        Owner::<Test>::insert(other_hk_1, subnet_owner_ck);
        Owner::<Test>::insert(other_hk_2, subnet_owner_ck);
        Owner::<Test>::insert(other_hk_3, subnet_owner_ck);
        OwnedHotkeys::<Test>::insert(
            subnet_owner_ck,
            vec![subnet_owner_hk, other_hk_1, other_hk_2, other_hk_3],
        );

        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

        // Set block of registration and UIDs for other hotkeys
        // HK1 has block of registration 2
        // HK2 and HK3 have the same block of registration 1, so they are sorted by UID
        // Set HK2 UID = 3 and HK3 UID = 2 so that HK3 is burned and HK2 is not
        // Summary: HK1 and HK3 should be burned, HK2 should be not.
        // Let's test it now.
        BlockAtRegistration::<Test>::insert(netuid, 1, 2);
        BlockAtRegistration::<Test>::insert(netuid, 3, 1);
        BlockAtRegistration::<Test>::insert(netuid, 2, 1);
        Uids::<Test>::insert(netuid, other_hk_1, 1);
        Uids::<Test>::insert(netuid, other_hk_2, 3);
        Uids::<Test>::insert(netuid, other_hk_3, 2);

        // Set the burn key limit to 3 because we also have sn owner
        ImmuneOwnerUidsLimit::<Test>::insert(netuid, 3);

        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha = AlphaCurrency::ZERO; // None to valis
        let owner_cut = AlphaCurrency::ZERO;
        let mut incentives: BTreeMap<U256, AlphaCurrency> = BTreeMap::new();

        // Give incentive to hotkeys
        incentives.insert(other_hk_1, 10_000_000.into());
        incentives.insert(other_hk_2, 10_000_000.into());
        incentives.insert(other_hk_3, 10_000_000.into());

        // Give incentives to subnet_owner_hk
        incentives.insert(subnet_owner_hk, 10_000_000.into());

        // Distribute dividends and incentives
        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            owner_cut,
            incentives,
            BTreeMap::new(),
            BTreeMap::new(),
        );

        // SN owner is burned
        let subnet_owner_stake_after =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_after, 0.into());

        // Testing the limits - HK1 and HK3 should be burned, HK2 should be not burned
        let other_stake_after_1 =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk_1, netuid);
        let other_stake_after_2 =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk_2, netuid);
        let other_stake_after_3 =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk_3, netuid);
        assert_eq!(other_stake_after_1, 0.into());
        assert!(other_stake_after_2 > 0.into());
        assert_eq!(other_stake_after_3, 0.into());
    });
}

#[test]
fn test_calculate_dividend_distribution_totals() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (AlphaCurrency, AlphaCurrency)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        let pending_validator_alpha = AlphaCurrency::from(183_123_567_452);
        let pending_tao = TaoCurrency::from(837_120_949_872);
        let tao_weight: U96F32 = U96F32::saturating_from_num(0.18); // 18%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302.into(), 2_342_352.into()));
        stake_map.insert(hotkeys[1], (23_423.into(), 859_273.into()));
        dividends.insert(hotkeys[0], 77_783_738_u64.into());
        dividends.insert(hotkeys[1], 19_283_940_u64.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<U96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<U96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            u64::from(pending_validator_alpha),
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao.to_u64(),
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividend_distribution_total_only_tao() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (AlphaCurrency, AlphaCurrency)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        let pending_validator_alpha = AlphaCurrency::ZERO;
        let pending_tao = TaoCurrency::from(837_120_949_872);
        let tao_weight: U96F32 = U96F32::saturating_from_num(0.18); // 18%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302.into(), 2_342_352.into()));
        stake_map.insert(hotkeys[1], (23_423.into(), 859_273.into()));
        dividends.insert(hotkeys[0], 77_783_738_u64.into());
        dividends.insert(hotkeys[1], 19_283_940_u64.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<U96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<U96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            u64::from(pending_validator_alpha),
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao.to_u64(),
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividend_distribution_total_no_tao_weight() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (AlphaCurrency, AlphaCurrency)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        let pending_validator_alpha = AlphaCurrency::from(183_123_567_452);
        let pending_tao = TaoCurrency::ZERO; // If tao weight is 0, then only alpha dividends should be input.
        let tao_weight: U96F32 = U96F32::saturating_from_num(0.0); // 0%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302.into(), 2_342_352.into()));
        stake_map.insert(hotkeys[1], (23_423.into(), 859_273.into()));
        dividends.insert(hotkeys[0], 77_783_738_u64.into());
        dividends.insert(hotkeys[1], 19_283_940_u64.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<U96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<U96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            u64::from(pending_validator_alpha),
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao.to_u64(),
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividend_distribution_total_only_alpha() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (AlphaCurrency, AlphaCurrency)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, U96F32> = BTreeMap::new();

        let pending_validator_alpha = AlphaCurrency::from(183_123_567_452);
        let pending_tao = TaoCurrency::ZERO;
        let tao_weight: U96F32 = U96F32::saturating_from_num(0.18); // 18%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302.into(), 2_342_352.into()));
        stake_map.insert(hotkeys[1], (23_423.into(), 859_273.into()));
        dividends.insert(hotkeys[0], 77_783_738_u64.into());
        dividends.insert(hotkeys[1], 19_283_940_u64.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<U96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<U96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            u64::from(pending_validator_alpha),
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao.to_u64(),
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividend_and_incentive_distribution() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_hk = U256::from(0);
        let sn_owner_ck = U256::from(1);
        let netuid = add_dynamic_network(&sn_owner_hk, &sn_owner_ck);

        // Register a single neuron.
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        // Give non-zero alpha
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            1.into(),
        );

        let pending_alpha = AlphaCurrency::from(123_456_789);
        let pending_validator_alpha = pending_alpha / 2.into(); // Pay half to validators.
        let pending_tao = TaoCurrency::ZERO;
        let pending_swapped = 0; // Only alpha output.
        let tao_weight: U96F32 = U96F32::saturating_from_num(0.0); // 0%

        // Hotkey, Incentive, Dividend
        let hotkey_emission = vec![(hotkey, pending_alpha / 2.into(), pending_alpha / 2.into())];

        let (incentives, (alpha_dividends, tao_dividends)) =
            SubtensorModule::calculate_dividend_and_incentive_distribution(
                netuid,
                pending_tao,
                pending_validator_alpha,
                hotkey_emission,
                tao_weight,
            );

        let incentives_total = incentives.values().copied().map(u64::from).sum::<u64>();
        let dividends_total = alpha_dividends
            .values()
            .sum::<U96F32>()
            .saturating_to_num::<u64>();

        assert_abs_diff_eq!(
            dividends_total.saturating_add(incentives_total),
            u64::from(pending_alpha),
            epsilon = 2
        );
    });
}

#[test]
fn test_calculate_dividend_and_incentive_distribution_all_to_validators() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_hk = U256::from(0);
        let sn_owner_ck = U256::from(1);
        let netuid = add_dynamic_network(&sn_owner_hk, &sn_owner_ck);

        // Register a single neuron.
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        // Give non-zero alpha
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            1.into(),
        );

        let pending_alpha = AlphaCurrency::from(123_456_789);
        let pending_validator_alpha = pending_alpha; // Pay all to validators.
        let pending_tao = TaoCurrency::ZERO;
        let tao_weight: U96F32 = U96F32::saturating_from_num(0.0); // 0%

        // Hotkey, Incentive, Dividend
        let hotkey_emission = vec![(hotkey, 0.into(), pending_alpha)];

        let (incentives, (alpha_dividends, tao_dividends)) =
            SubtensorModule::calculate_dividend_and_incentive_distribution(
                netuid,
                pending_tao,
                pending_validator_alpha,
                hotkey_emission,
                tao_weight,
            );

        let incentives_total = incentives.values().copied().map(u64::from).sum::<u64>();
        let dividends_total = alpha_dividends
            .values()
            .sum::<U96F32>()
            .saturating_to_num::<u64>();

        assert_eq!(
            AlphaCurrency::from(dividends_total.saturating_add(incentives_total)),
            pending_alpha
        );
    });
}

#[test]
fn test_calculate_dividends_and_incentives() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_hk = U256::from(0);
        let sn_owner_ck = U256::from(1);
        let netuid = add_dynamic_network(&sn_owner_hk, &sn_owner_ck);

        // Register a single neuron.
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        // Give non-zero alpha
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            1.into(),
        );

        let divdends = AlphaCurrency::from(123_456_789);
        let incentive = AlphaCurrency::from(683_051_923);
        let total_emission = divdends.saturating_add(incentive);

        // Hotkey, Incentive, Dividend
        let hotkey_emission = vec![(hotkey, incentive, divdends)];

        let (incentives, dividends) =
            SubtensorModule::calculate_dividends_and_incentives(netuid, hotkey_emission);

        let incentives_total = incentives
            .values()
            .copied()
            .fold(AlphaCurrency::ZERO, |acc, x| acc + x);
        let dividends_total = AlphaCurrency::from(
            dividends
                .values()
                .sum::<U96F32>()
                .saturating_to_num::<u64>(),
        );

        assert_eq!(
            dividends_total.saturating_add(incentives_total),
            total_emission
        );
    });
}

#[test]
fn test_calculate_dividends_and_incentives_only_validators() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_hk = U256::from(0);
        let sn_owner_ck = U256::from(1);
        let netuid = add_dynamic_network(&sn_owner_hk, &sn_owner_ck);

        // Register a single neuron.
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        // Give non-zero alpha
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            1.into(),
        );

        let divdends = AlphaCurrency::from(123_456_789);
        let incentive = AlphaCurrency::ZERO;

        // Hotkey, Incentive, Dividend
        let hotkey_emission = vec![(hotkey, incentive, divdends)];

        let (incentives, dividends) =
            SubtensorModule::calculate_dividends_and_incentives(netuid, hotkey_emission);

        let incentives_total = incentives
            .values()
            .copied()
            .fold(AlphaCurrency::ZERO, |acc, x| acc + x);
        let dividends_total = AlphaCurrency::from(
            dividends
                .values()
                .sum::<U96F32>()
                .saturating_to_num::<u64>(),
        );

        assert_eq!(dividends_total, divdends);
        assert_eq!(incentives_total, AlphaCurrency::ZERO);
    });
}

#[test]
fn test_calculate_dividends_and_incentives_only_miners() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_hk = U256::from(0);
        let sn_owner_ck = U256::from(1);
        let netuid = add_dynamic_network(&sn_owner_hk, &sn_owner_ck);

        // Register a single neuron.
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        // Give non-zero alpha
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            1.into(),
        );

        let divdends = AlphaCurrency::ZERO;
        let incentive = AlphaCurrency::from(123_456_789);

        // Hotkey, Incentive, Dividend
        let hotkey_emission = vec![(hotkey, incentive, divdends)];

        let (incentives, dividends) =
            SubtensorModule::calculate_dividends_and_incentives(netuid, hotkey_emission);

        let incentives_total = incentives
            .values()
            .copied()
            .fold(AlphaCurrency::ZERO, |acc, x| acc + x);
        let dividends_total = AlphaCurrency::from(
            dividends
                .values()
                .sum::<U96F32>()
                .saturating_to_num::<u64>(),
        );

        assert_eq!(incentives_total, incentive);
        assert_eq!(dividends_total, divdends);
    });
}

#[test]
fn test_drain_pending_emission_no_miners_all_drained() {
    new_test_ext(1).execute_with(|| {
        let netuid = add_dynamic_network(&U256::from(1), &U256::from(2));
        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let init_stake = 1;
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        // Give non-zero stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            init_stake.into(),
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            init_stake.into()
        );

        // Set the weight of root TAO to be 0%, so only alpha is effective.
        SubtensorModule::set_tao_weight(0);

        // Set the emission to be 1 million.
        let emission = AlphaCurrency::from(1_000_000);
        // Run drain pending without any miners.
        SubtensorModule::drain_pending_emission(
            netuid,
            emission,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Get the new stake of the hotkey.
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey);
        // We expect this neuron to get *all* the emission.
        // Slight epsilon due to rounding (hotkey_take).
        assert_abs_diff_eq!(
            new_stake,
            u64::from(emission.saturating_add(init_stake.into())).into(),
            epsilon = 1.into()
        );
    });
}

#[test]
fn test_drain_pending_emission_zero_emission() {
    new_test_ext(1).execute_with(|| {
        let netuid = add_dynamic_network_disable_commit_reveal(&U256::from(1), &U256::from(2));
        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let miner_hk = U256::from(5);
        let miner_ck = U256::from(6);
        let init_stake: u64 = 100_000_000_000_000;
        let tempo = 2;
        SubtensorModule::set_tempo(netuid, tempo);
        // Set weight-set limit to 0.
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);

        register_ok_neuron(netuid, hotkey, coldkey, 0);
        register_ok_neuron(netuid, miner_hk, miner_ck, 0);
        // Give non-zero stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            init_stake.into(),
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            init_stake.into()
        );

        // Set the weight of root TAO to be 0%, so only alpha is effective.
        SubtensorModule::set_tao_weight(0);

        run_to_block_no_epoch(netuid, 50);

        // Run epoch for initial setup.
        SubtensorModule::epoch(netuid, AlphaCurrency::ZERO);

        // Set weights on miner
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            vec![0, 1, 2],
            vec![0, 0, 1],
            0,
        ));

        run_to_block_no_epoch(netuid, 50);

        // Clear incentive and dividends.
        Incentive::<Test>::remove(netuid);
        Dividends::<Test>::remove(netuid);

        // Set the emission to be ZERO.
        SubtensorModule::drain_pending_emission(
            netuid,
            0.into(),
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );

        // Get the new stake of the hotkey.
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey);
        // We expect the stake to remain unchanged.
        assert_eq!(new_stake, init_stake.into());

        // Check that the incentive and dividends are set by epoch.
        assert!(Incentive::<Test>::get(netuid).iter().sum::<u16>() > 0);
        assert!(Dividends::<Test>::get(netuid).iter().sum::<u16>() > 0);
    });
}

#[test]
fn test_run_coinbase_not_started() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo = 2;

        let sn_owner_hk = U256::from(7);
        let sn_owner_ck = U256::from(8);

        add_network_without_emission_block(netuid, tempo, 0);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, false);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);

        SubnetOwner::<Test>::insert(netuid, sn_owner_ck);
        SubnetOwnerHotkey::<Test>::insert(netuid, sn_owner_hk);

        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let miner_hk = U256::from(5);
        let miner_ck = U256::from(6);
        let init_stake: u64 = 100_000_000_000_000;
        let tempo = 2;
        SubtensorModule::set_tempo(netuid, tempo);
        // Set weight-set limit to 0.
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);

        let reserve = init_stake * 1000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        register_ok_neuron(netuid, hotkey, coldkey, 0);
        register_ok_neuron(netuid, miner_hk, miner_ck, 0);
        register_ok_neuron(netuid, sn_owner_hk, sn_owner_ck, 0);
        // Give non-zero stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            init_stake.into(),
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            init_stake.into()
        );

        // Set the weight of root TAO to be 0%, so only alpha is effective.
        SubtensorModule::set_tao_weight(0);

        run_to_block_no_epoch(netuid, 30);

        // Run epoch for initial setup.
        SubtensorModule::epoch(netuid, AlphaCurrency::ZERO);

        // Set weights on miner
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            vec![0, 1, 2],
            vec![0, 0, 1],
            0,
        ));

        // Clear incentive and dividends.
        Incentive::<Test>::remove(netuid);
        Dividends::<Test>::remove(netuid);

        // Step so tempo should run.
        next_block_no_epoch(netuid);
        next_block_no_epoch(netuid);
        next_block_no_epoch(netuid);
        let current_block = System::block_number();
        assert!(SubtensorModule::should_run_epoch(netuid, current_block));

        // Run coinbase with emission.
        SubtensorModule::run_coinbase(U96F32::saturating_from_num(100_000_000));

        // We expect that the epoch ran.
        assert_eq!(BlocksSinceLastStep::<Test>::get(netuid), 0);

        // Get the new stake of the hotkey. We expect no emissions.
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey);
        // We expect the stake to remain unchanged.
        assert_eq!(new_stake, init_stake.into());

        // Check that the incentive and dividends are set.
        assert!(Incentive::<Test>::get(netuid).iter().sum::<u16>() > 0);
        assert!(Dividends::<Test>::get(netuid).iter().sum::<u16>() > 0);
    });
}

#[test]
fn test_run_coinbase_not_started_start_after() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo = 2;

        let sn_owner_hk = U256::from(7);
        let sn_owner_ck = U256::from(8);

        add_network_without_emission_block(netuid, tempo, 0);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid, false);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);

        SubnetOwner::<Test>::insert(netuid, sn_owner_ck);
        SubnetOwnerHotkey::<Test>::insert(netuid, sn_owner_hk);

        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let miner_hk = U256::from(5);
        let miner_ck = U256::from(6);
        let init_stake: u64 = 100_000_000_000_000;
        let tempo = 2;
        SubtensorModule::set_tempo(netuid, tempo);
        // Set weight-set limit to 0.
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);

        register_ok_neuron(netuid, hotkey, coldkey, 0);
        register_ok_neuron(netuid, miner_hk, miner_ck, 0);
        register_ok_neuron(netuid, sn_owner_hk, sn_owner_ck, 0);
        // Give non-zero stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            init_stake.into(),
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            init_stake.into()
        );

        // Set the weight of root TAO to be 0%, so only alpha is effective.
        SubtensorModule::set_tao_weight(0);

        run_to_block_no_epoch(netuid, 30);

        // Run epoch for initial setup.
        SubtensorModule::epoch(netuid, AlphaCurrency::ZERO);

        // Set weights on miner
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkey),
            netuid,
            vec![0, 1, 2],
            vec![0, 0, 1],
            0,
        ));

        // Clear incentive and dividends.
        Incentive::<Test>::remove(netuid);
        Dividends::<Test>::remove(netuid);

        // Step so tempo should run.
        next_block_no_epoch(netuid);
        next_block_no_epoch(netuid);
        next_block_no_epoch(netuid);
        let current_block = System::block_number();
        assert!(SubtensorModule::should_run_epoch(netuid, current_block));

        // Run coinbase with emission.
        SubtensorModule::run_coinbase(U96F32::saturating_from_num(100_000_000));
        // We expect that the epoch ran.
        assert_eq!(BlocksSinceLastStep::<Test>::get(netuid), 0);

        let block_number = DurationOfStartCall::get();
        run_to_block_no_epoch(netuid, block_number);

        let current_block = System::block_number();

        // Run start call.
        assert_ok!(SubtensorModule::start_call(
            RuntimeOrigin::signed(sn_owner_ck),
            netuid
        ));
        assert_eq!(
            FirstEmissionBlockNumber::<Test>::get(netuid),
            Some(current_block + 1)
        );

        // Run coinbase with emission.
        SubtensorModule::run_coinbase(U96F32::saturating_from_num(100_000_000));
        // We expect that the epoch ran.
        assert_eq!(BlocksSinceLastStep::<Test>::get(netuid), 0);

        // Get the new stake of the hotkey. We expect no emissions.
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey);
        // We expect the stake to remain unchanged.
        assert!(new_stake > init_stake.into());
        log::info!("new_stake: {new_stake}");
    });
}

/// Test that coinbase updates protocol position liquidity
/// cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_v3_liquidity_update --exact --show-output
#[test]
fn test_coinbase_v3_liquidity_update() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Force the swap to initialize
        SubtensorModule::swap_tao_for_alpha(
            netuid,
            TaoCurrency::ZERO,
            1_000_000_000_000.into(),
            false,
        )
        .unwrap();

        let protocol_account_id = pallet_subtensor_swap::Pallet::<Test>::protocol_account_id();
        let position = pallet_subtensor_swap::Positions::<Test>::get((
            netuid,
            protocol_account_id,
            PositionId::from(1),
        ))
        .unwrap();
        let liquidity_before = position.liquidity;

        // Enable emissions and run coinbase (which will increase position liquidity)
        let emission: u64 = 1_234_567;
        FirstEmissionBlockNumber::<Test>::insert(netuid, 0);
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0.5));
        SubtensorModule::run_coinbase(U96F32::from_num(emission));

        let position_after = pallet_subtensor_swap::Positions::<Test>::get((
            netuid,
            protocol_account_id,
            PositionId::from(1),
        ))
        .unwrap();
        let liquidity_after = position_after.liquidity;

        assert!(liquidity_before < liquidity_after);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_alpha_childkey_parentkey_with_burn --exact --show-output --nocapture
#[test]
fn test_drain_alpha_childkey_parentkey_with_burn() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        let parent = U256::from(1);
        let child = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before = AlphaCurrency::from(1_000_000_000);
        register_ok_neuron(netuid, child, coldkey, 0);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &parent,
            &coldkey,
            netuid,
            stake_before,
        );
        mock_set_children_no_epochs(netuid, &parent, &[(u64::MAX, child)]);

        // Childkey take is 10%
        ChildkeyTake::<Test>::insert(child, netuid, u16::MAX / 10);

        let burn_rate = SubtensorModule::get_ck_burn();
        let parent_stake_before = SubtensorModule::get_stake_for_hotkey_on_subnet(&parent, netuid);
        let child_stake_before = SubtensorModule::get_stake_for_hotkey_on_subnet(&child, netuid);

        let pending_alpha = AlphaCurrency::from(1_000_000_000);
        SubtensorModule::drain_pending_emission(
            netuid,
            pending_alpha,
            TaoCurrency::ZERO,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
        );
        let parent_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&parent, netuid);
        let child_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&child, netuid);

        let expected_ck_burn = I96F32::from_num(pending_alpha)
            * I96F32::from_num(9.0 / 10.0)
            * I96F32::from_num(burn_rate);

        let expected_total = I96F32::from_num(pending_alpha) - expected_ck_burn;
        let parent_ratio = (I96F32::from_num(pending_alpha) * I96F32::from_num(9.0 / 10.0)
            - expected_ck_burn)
            / expected_total;
        let child_ratio = (I96F32::from_num(pending_alpha) / I96F32::from_num(10)) / expected_total;

        let expected =
            I96F32::from_num(stake_before) + I96F32::from_num(pending_alpha) * parent_ratio;
        log::info!(
            "expected: {:?}, parent_stake_after: {:?}",
            expected.to_num::<u64>(),
            parent_stake_after
        );

        close(
            expected.to_num::<u64>(),
            parent_stake_after.into(),
            3_000_000,
        );
        let expected = I96F32::from_num(u64::from(pending_alpha)) * child_ratio;
        close(
            expected.to_num::<u64>(),
            child_stake_after.into(),
            3_000_000,
        );
    });
}

#[test]
fn test_incentive_is_autostaked_to_owner_destination() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_ck = U256::from(0);
        let subnet_owner_hk = U256::from(1);

        let miner_ck = U256::from(10);
        let miner_hk = U256::from(11);
        let dest_hk = U256::from(12);

        Owner::<Test>::insert(miner_hk, miner_ck);
        Owner::<Test>::insert(dest_hk, miner_ck);
        OwnedHotkeys::<Test>::insert(miner_ck, vec![miner_hk, dest_hk]);

        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

        Uids::<Test>::insert(netuid, miner_hk, 1);
        Uids::<Test>::insert(netuid, dest_hk, 2);

        // Set autostake destination for the miner's coldkey
        assert_ok!(SubtensorModule::set_coldkey_auto_stake_hotkey(
            RuntimeOrigin::signed(miner_ck),
            netuid,
            dest_hk,
        ));

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&miner_hk, netuid),
            0.into()
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&dest_hk, netuid),
            0.into()
        );

        // Distribute an incentive to the miner hotkey
        let mut incentives: BTreeMap<U256, AlphaCurrency> = BTreeMap::new();
        let incentive: AlphaCurrency = 10_000_000u64.into();
        incentives.insert(miner_hk, incentive);

        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            AlphaCurrency::ZERO, // owner_cut
            incentives,
            BTreeMap::new(), // alpha_dividends
            BTreeMap::new(), // tao_dividends
        );

        // Expect the stake to land on the destination hotkey (not the original miner hotkey)
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&miner_hk, netuid),
            0.into()
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&dest_hk, netuid),
            incentive
        );
    });
}

#[test]
fn test_incentive_goes_to_hotkey_when_no_autostake_destination() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_ck = U256::from(0);
        let subnet_owner_hk = U256::from(1);

        let miner_ck = U256::from(20);
        let miner_hk = U256::from(21);

        Owner::<Test>::insert(miner_hk, miner_ck);
        OwnedHotkeys::<Test>::insert(miner_ck, vec![miner_hk]);

        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

        Uids::<Test>::insert(netuid, miner_hk, 1);

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&miner_hk, netuid),
            0.into()
        );

        // Distribute an incentive to the miner hotkey
        let mut incentives: BTreeMap<U256, AlphaCurrency> = BTreeMap::new();
        let incentive: AlphaCurrency = 5_000_000u64.into();
        incentives.insert(miner_hk, incentive);

        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            AlphaCurrency::ZERO, // owner_cut
            incentives,
            BTreeMap::new(), // alpha_dividends
            BTreeMap::new(), // tao_dividends
        );

        // With no autostake destination, the incentive should be staked to the original hotkey
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&miner_hk, netuid),
            incentive
        );
    });
}
