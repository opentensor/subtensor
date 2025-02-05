#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use super::mock::*;

use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::assert_ok;
use sp_core::U256;
use substrate_fixed::types::I64F64;
use substrate_fixed::types::I96F32;

#[allow(clippy::arithmetic_side_effects)]
fn close(value: u64, target: u64, eps: u64) {
    assert!(
        (value as i64 - target as i64).abs() < eps as i64,
        "Assertion failed: value = {}, target = {}, eps = {}",
        value,
        target,
        eps
    )
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
                    SubnetMechanism::<Test>::insert(1, 1);
                    SubnetTAO::<Test>::insert(1, (price * 1_000_000_000.0) as u64);
                    SubnetAlphaIn::<Test>::insert(1, 1_000_000_000);
                    let (tao_in_emission, alpha_in_emission, alpha_out_emission) = SubtensorModule::get_dynamic_tao_emission( 1, tao_in, alpha_emission);
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
        SubtensorModule::run_coinbase(I96F32::from_num(0.0));
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
        let netuid: u16 = 1;
        let emission: u64 = 1_234_567;
        add_network(netuid, 1, 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid), 0);
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
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
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let netuid3: u16 = 3;
        let emission: u64 = 3_333_333;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        add_network(netuid3, 1, 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid1), 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid2), 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid3), 0);
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        assert_eq!(SubnetTAO::<Test>::get(netuid1), emission / 3);
        assert_eq!(SubnetTAO::<Test>::get(netuid2), emission / 3);
        assert_eq!(SubnetTAO::<Test>::get(netuid3), emission / 3);
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
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let emission: u64 = 100_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);
        // Set subnet prices.
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));
        // Assert initial TAO reserves.
        assert_eq!(SubnetTAO::<Test>::get(netuid1), 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid2), 0);
        // Run the coinbase with the emission amount.
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        // Assert tao emission is split evenly.
        assert_eq!(SubnetTAO::<Test>::get(netuid1), emission / 3);
        assert_eq!(SubnetTAO::<Test>::get(netuid2), emission / 3 + emission / 3);
        close(TotalIssuance::<Test>::get(), emission, 2);
        close(TotalStake::<Test>::get(), emission, 2);
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
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        // Set price to 1.0
        SubnetTAO::<Test>::insert(netuid, 1_000_000);
        SubnetAlphaIn::<Test>::insert(netuid, 1_000_000);
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(1));
        // Updating the moving price keeps it the same.
        assert_eq!(
            SubtensorModule::get_moving_alpha_price(netuid),
            I96F32::from_num(1)
        );
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
        // Assert price is == 100% of the real price.
        assert_eq!(
            SubtensorModule::get_moving_alpha_price(netuid),
            I96F32::from_num(1.0)
        );
        // Set price to zero.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0));
        SubnetMovingAlpha::<Test>::set(I96F32::from_num(0.1));
        // Run moving 6 times.
        SubtensorModule::update_moving_price(netuid);
        SubtensorModule::update_moving_price(netuid);
        SubtensorModule::update_moving_price(netuid);
        SubtensorModule::update_moving_price(netuid);
        SubtensorModule::update_moving_price(netuid);
        SubtensorModule::update_moving_price(netuid);
        // Assert price is > 50% of the real price.
        assert_eq!(
            SubtensorModule::get_moving_alpha_price(netuid),
            I96F32::from_num(0.468559)
        );
    });
}

// Test basic alpha issuance in coinbase mechanism.
// This test verifies that:
// - Alpha issuance is initialized to 0 for new subnets
// - Alpha issuance is split evenly between subnets during coinbase
// - Each subnet receives the expected fraction of total emission
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_base() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Set up prices 1 and 1
        SubnetTAO::<Test>::insert(netuid1, 1_000_000);
        SubnetAlphaIn::<Test>::insert(netuid1, 1_000_000);
        SubnetTAO::<Test>::insert(netuid2, 1_000_000);
        SubnetAlphaIn::<Test>::insert(netuid2, 1_000_000);
        // Check initial
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid1), 0);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid2), 0);
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        // tao_in = 500_000
        // alpha_in = 500_000/price = 500_000
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid1), emission / 2);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid2), emission / 2);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance_different --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_different() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);
        // Setup prices 1 and 1
        let initial: u64 = 1_000_000;
        SubnetTAO::<Test>::insert(netuid1, initial);
        SubnetAlphaIn::<Test>::insert(netuid1, initial);
        SubnetTAO::<Test>::insert(netuid2, initial);
        SubnetAlphaIn::<Test>::insert(netuid2, initial);
        // Set subnet prices.
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));
        // Run coinbase
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        // tao_in = 333_333
        // alpha_in = 333_333/price = 333_333 + initial
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid1), initial + emission / 3);
        // tao_in = 666_666
        // alpha_in = 666_666/price = 666_666 + initial
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid2),
            initial + emission / 3 + emission / 3
        );
    });
}
