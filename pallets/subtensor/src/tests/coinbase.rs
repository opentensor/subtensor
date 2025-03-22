#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use super::mock::*;

use crate::*;
use alloc::collections::BTreeMap;
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

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_tao_issuance_base_low --exact --show-output --nocapture
#[test]
fn test_coinbase_tao_issuance_base_low() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let emission: u64 = 1;
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
        NetworkRegisteredAt::<Test>::insert(netuid, 1);

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
        assert!(I96F32::from_num(1.0) - SubtensorModule::get_moving_alpha_price(netuid) < 0.05);
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
        assert!(
            (I96F32::from_num(0.512325) - SubtensorModule::get_moving_alpha_price(netuid)).abs()
                < 0.001
        );
    });
}

// Test moving price updates slow down at the beginning.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_update_moving_price_initial --exact --show-output --nocapture
#[test]
fn test_update_moving_price_initial() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        // Set current price to 1.0
        SubnetTAO::<Test>::insert(netuid, 1_000_000);
        SubnetAlphaIn::<Test>::insert(netuid, 1_000_000);
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetMovingAlpha::<Test>::set(I96F32::from_num(0.5));
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0));

        // Registered recently
        System::set_block_number(510);
        NetworkRegisteredAt::<Test>::insert(netuid, 500);

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
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        // Set current price to 1.0
        SubnetTAO::<Test>::insert(netuid, 1_000_000);
        SubnetAlphaIn::<Test>::insert(netuid, 1_000_000);
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetMovingAlpha::<Test>::set(I96F32::from_num(0.5));
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(0));

        // Registered long time ago
        System::set_block_number(144_000_500);
        NetworkRegisteredAt::<Test>::insert(netuid, 500);

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
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Set up prices 1 and 1
        let initial: u64 = 1_000_000;
        SubnetTAO::<Test>::insert(netuid1, initial);
        SubnetAlphaIn::<Test>::insert(netuid1, initial);
        SubnetTAO::<Test>::insert(netuid2, initial);
        SubnetAlphaIn::<Test>::insert(netuid2, initial);
        // Check initial
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        // tao_in = 500_000
        // alpha_in = 500_000/price = 500_000
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid1), initial + emission / 2);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid2), initial + emission / 2);
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

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance_with_cap_trigger --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_with_cap_trigger() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);
        // Setup prices 1000000
        let initial: u64 = 1_000;
        let initial_alpha: u64 = initial * 1000000;
        SubnetTAO::<Test>::insert(netuid1, initial);
        SubnetAlphaIn::<Test>::insert(netuid1, initial_alpha); // Make price extremely low.
        SubnetTAO::<Test>::insert(netuid2, initial);
        SubnetAlphaIn::<Test>::insert(netuid2, initial_alpha); // Make price extremely low.
        // Set subnet prices.
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));
        // Run coinbase
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        // tao_in = 333_333
        // alpha_in = 333_333/price > 1_000_000_000 --> 1_000_000_000 + initial_alpha
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid1),
            initial_alpha + 1_000_000_000
        );
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid2), 1_000_000_000);
        // tao_in = 666_666
        // alpha_in = 666_666/price > 1_000_000_000 --> 1_000_000_000 + initial_alpha
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid2),
            initial_alpha + 1_000_000_000
        );
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid2), 1_000_000_000); // Gets full block emission.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_coinbase_alpha_issuance_with_cap_trigger_and_block_emission --exact --show-output --nocapture
#[test]
fn test_coinbase_alpha_issuance_with_cap_trigger_and_block_emission() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let emission: u64 = 1_000_000;
        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        // Make subnets dynamic.
        SubnetMechanism::<Test>::insert(netuid1, 1);
        SubnetMechanism::<Test>::insert(netuid2, 1);
        // Setup prices 1000000
        let initial: u64 = 1_000;
        let initial_alpha: u64 = initial * 1000000;
        SubnetTAO::<Test>::insert(netuid1, initial);
        SubnetAlphaIn::<Test>::insert(netuid1, initial_alpha); // Make price extremely low.
        SubnetTAO::<Test>::insert(netuid2, initial);
        SubnetAlphaIn::<Test>::insert(netuid2, initial_alpha); // Make price extremely low.
        // Set issuance to greater than 21M
        SubnetAlphaOut::<Test>::insert(netuid1, 22_000_000_000_000_000); // Set issuance above 21M
        SubnetAlphaOut::<Test>::insert(netuid2, 22_000_000_000_000_000); // Set issuance above 21M
        // Set subnet prices.
        SubnetMovingPrice::<Test>::insert(netuid1, I96F32::from_num(1));
        SubnetMovingPrice::<Test>::insert(netuid2, I96F32::from_num(2));
        // Run coinbase
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        // tao_in = 333_333
        // alpha_in = 333_333/price > 1_000_000_000 --> 0 + initial_alpha
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid1), initial_alpha);
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid2), 22_000_000_000_000_000);
        // tao_in = 666_666
        // alpha_in = 666_666/price > 1_000_000_000 --> 0 + initial_alpha
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid2), initial_alpha);
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid2), 22_000_000_000_000_000);
        // No emission.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_owner_cut_base --exact --show-output --nocapture
#[test]
fn test_owner_cut_base() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        SubtensorModule::set_tempo(netuid, 10000); // Large number (dont drain)
        SubtensorModule::set_subnet_owner_cut(0);
        SubtensorModule::run_coinbase(I96F32::from_num(0));
        assert_eq!(PendingOwnerCut::<Test>::get(netuid), 0); // No cut
        SubtensorModule::set_subnet_owner_cut(u16::MAX);
        SubtensorModule::run_coinbase(I96F32::from_num(0));
        assert_eq!(PendingOwnerCut::<Test>::get(netuid), 1_000_000_000); // Full cut.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_pending_swapped --exact --show-output --nocapture
#[test]
fn test_pending_swapped() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let emission: u64 = 1_000_000;
        add_network(netuid, 1, 0);
        SubtensorModule::run_coinbase(I96F32::from_num(0));
        assert_eq!(PendingAlphaSwapped::<Test>::get(netuid), 0); // Zero tao weight and no root.
        SubnetTAO::<Test>::insert(0, 1_000_000_000); // Add root weight.
        SubtensorModule::run_coinbase(I96F32::from_num(0));
        assert_eq!(PendingAlphaSwapped::<Test>::get(netuid), 0); // Zero tao weight with 1 root.
        SubtensorModule::set_tempo(netuid, 10000); // Large number (dont drain)
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtensorModule::run_coinbase(I96F32::from_num(0));
        assert_eq!(PendingAlphaSwapped::<Test>::get(netuid), 125000000); // 1 TAO / ( 1 + 3 ) = 0.25 * 1 / 2 = 125000000
        assert_eq!(
            PendingEmission::<Test>::get(netuid),
            1_000_000_000 - 125000000
        ); // 1 - swapped.
        assert_eq!(PendingRootDivs::<Test>::get(netuid), 125000000); // swapped * (price = 1)
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base --exact --show-output --nocapture
#[test]
fn test_drain_base() {
    new_test_ext(1).execute_with(|| SubtensorModule::drain_pending_emission(0, 0, 0, 0, 0));
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        SubtensorModule::drain_pending_emission(netuid, 0, 0, 0, 0)
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_single_staker_not_registered --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_single_staker_not_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_before: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_before,
        );
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, 0, 0, 0);
        let stake_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(stake_before, stake_after); // Not registered.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_single_staker_registered --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_single_staker_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_before: u64 = 1_000_000_000;
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_before,
        );
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, 0, 0, 0);
        let stake_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        close(stake_before + pending_alpha, stake_after, 10); // Registered gets all emission.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_single_staker_registered_root_weight --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_single_staker_registered_root_weight() {
    new_test_ext(1).execute_with(|| {
        let root: u16 = 0;
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let stake_before: u64 = 1_000_000_000;
        // register_ok_neuron(root, hotkey, coldkey, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        Delegates::<Test>::insert(hotkey, 0);
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            root,
            stake_before,
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_before,
        );
        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, pending_tao, 0, 0);
        let stake_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let root_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, root);
        close(stake_before + pending_alpha / 2, stake_after, 10); // Registered gets all alpha emission.
        close(stake_before + pending_tao, root_after, 10); // Registered gets all tao emission
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before: u64 = 1_000_000_000;
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
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, 0, 0, 0);
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        close(stake_before + pending_alpha / 2, stake_after1, 10); // Registered gets 1/2 emission
        close(stake_before + pending_alpha / 2, stake_after2, 10); // Registered gets 1/2 emission.
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered_and_root --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered_and_root() {
    new_test_ext(1).execute_with(|| {
        let root: u16 = 0;
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before: u64 = 1_000_000_000;
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
            root,
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
            root,
            stake_before,
        );
        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, pending_tao, 0, 0);
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let root_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, root);
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        let root_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, root);
        close(stake_before + pending_alpha / 4, stake_after1, 10); // Registered gets 1/2 emission
        close(stake_before + pending_alpha / 4, stake_after2, 10); // Registered gets 1/2 emission.
        close(stake_before + pending_tao / 2, root_after1, 10); // Registered gets 1/2 tao emission
        close(stake_before + pending_tao / 2, root_after2, 10); // Registered gets 1/2 tao emission
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts() {
    new_test_ext(1).execute_with(|| {
        let root: u16 = 0;
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before: u64 = 1_000_000_000;
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
            root,
            2 * stake_before, // Hotkey 1 has twice as much root weight.
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
            root,
            stake_before,
        );
        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, pending_tao, 0, 0);
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let root_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, root);
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        let root_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, root);
        let expected_stake = I96F32::from_num(stake_before)
            + (I96F32::from_num(pending_alpha)
                * I96F32::from_num(3.0 / 5.0)
                * I96F32::from_num(1.0 / 3.0));
        close(expected_stake.to_num::<u64>(), stake_after1, 10); // Registered gets 60% of emission
        let expected_stake2 = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_alpha)
                * I96F32::from_num(2.0 / 5.0)
                * I96F32::from_num(1.0 / 2.0);
        close(expected_stake2.to_num::<u64>(), stake_after2, 10); // Registered gets 40% emission
        let expected_root1 = I96F32::from_num(2 * stake_before)
            + I96F32::from_num(pending_tao) * I96F32::from_num(2.0 / 3.0);
        close(expected_root1.to_num::<u64>(), root_after1, 10); // Registered gets 2/3 tao emission
        let expected_root2 = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_tao) * I96F32::from_num(1.0 / 3.0);
        close(expected_root2.to_num::<u64>(), root_after2, 10); // Registered gets 1/3 tao emission
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts_half_tao_weight --exact --show-output --nocapture
#[test]
fn test_drain_base_with_subnet_with_two_stakers_registered_and_root_different_amounts_half_tao_weight()
 {
    new_test_ext(1).execute_with(|| {
        let root: u16 = 0;
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before: u64 = 1_000_000_000;
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
            root,
            2 * stake_before, // Hotkey 1 has twice as much root weight.
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
            root,
            stake_before,
        );
        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, pending_tao, 0, 0);
        let stake_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid);
        let root_after1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, root);
        let stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid);
        let root_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, root);
        // hotkey 1 has (1 + (2 * 0.5))/( 1 + 1*0.5 + 1 + (2 * 0.5)) = 0.5714285714 of the hotkey emission.
        let expected_stake = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_alpha)
                * I96F32::from_num(0.5714285714)
                * I96F32::from_num(1.0 / 2.0);
        close(expected_stake.to_num::<u64>(), stake_after1, 10);
        // hotkey 2 has (1 + 1*0.5)/( 1 + 1*0.5 + 1 + (2 * 0.5)) = 0.4285714286 of the hotkey emission.
        let expected_stake2 = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_alpha)
                * I96F32::from_num(0.4285714286)
                * I96F32::from_num(2.0 / 3.0);
        close(expected_stake2.to_num::<u64>(), stake_after2, 10);
        // hotkey 1 has 2 / 3 root tao
        let expected_root1 = I96F32::from_num(2 * stake_before)
            + I96F32::from_num(pending_tao) * I96F32::from_num(2.0 / 3.0);
        close(expected_root1.to_num::<u64>(), root_after1, 10);
        // hotkey 1 has 1 / 3 root tao
        let expected_root2 = I96F32::from_num(stake_before)
            + I96F32::from_num(pending_tao) * I96F32::from_num(1.0 / 3.0);
        close(expected_root2.to_num::<u64>(), root_after2, 10);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_drain_alpha_childkey_parentkey --exact --show-output --nocapture
#[test]
fn test_drain_alpha_childkey_parentkey() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        let parent = U256::from(1);
        let child = U256::from(2);
        let coldkey = U256::from(3);
        let stake_before: u64 = 1_000_000_000;
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

        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(netuid, pending_alpha, 0, 0, 0);
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
        close(expected.to_num::<u64>(), parent_stake_after, 10_000);
        let expected = I96F32::from_num(pending_alpha) / I96F32::from_num(10);
        close(expected.to_num::<u64>(), child_stake_after, 10_000);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children --exact --show-output --nocapture
#[test]
fn test_get_root_children() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let root: u16 = 0;
        let alpha: u16 = 1;
        add_network(root, 1, 0);
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
        let alice_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold,
            root,
            alice_root_stake,
        );
        let bob_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold,
            root,
            alice_root_stake,
        );

        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold,
            alpha,
            bob_alpha_stake,
        );

        // Set Bob as 100% child of Alice on root.
        // mock_set_children_no_epochs( root, &alice, &[(u64::MAX, bob)]);
        mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX, bob)]);

        // Assert Alice and Bob stake on root and netuid
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&alice, root),
            alice_root_stake
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&bob, root),
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
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&alice, root),
            alice_root_stake
        );
        assert_eq!(
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&alice, alpha),
            0
        );
        assert_eq!(
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&bob, root),
            bob_root_stake
        );
        assert_eq!(
            SubtensorModule::get_inherited_for_hotkey_on_subnet(&bob, alpha),
            bob_alpha_stake + alice_alpha_stake
        );

        // Assert Alice and Bob TAO inherited stakes
        assert_eq!(
            SubtensorModule::get_tao_inherited_for_hotkey_on_subnet(&alice, alpha),
            0
        );
        assert_eq!(
            SubtensorModule::get_tao_inherited_for_hotkey_on_subnet(&bob, alpha),
            bob_root_stake + alice_root_stake
        );

        // Get Alice stake amounts on subnet alpha.
        let (alice_total, alice_alpha, alice_tao): (I64F64, I64F64, I64F64) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&alice, alpha);
        assert_eq!(alice_total, I64F64::from_num(0));

        // Get Bob stake amounts on subnet alpha.
        let (bob_total, bob_alpha, bob_tao): (I64F64, I64F64, I64F64) =
            SubtensorModule::get_stake_weights_for_hotkey_on_subnet(&bob, alpha);
        assert_eq!(bob_total, I64F64::from_num(4 * bob_root_stake));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let root: u16 = 0;
        let alpha: u16 = 1;
        add_network(root, 1, 0);
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
        let alice_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            root,
            alice_root_stake,
        );
        let bob_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            root,
            alice_root_stake,
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake: u64 = 1_000_000_000;
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
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(alpha, pending_alpha, 0, 0, 0);

        // Alice and Bob both made half of the dividends.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&alice, alpha),
            alice_alpha_stake + pending_alpha / 4
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&bob, alpha),
            bob_alpha_stake + pending_alpha / 4
        );

        // Lets drain
        let pending_alpha: u64 = 1_000_000_000;
        let pending_root: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(alpha, pending_alpha, pending_root, 0, 0);

        // Alice and Bob both made half of the dividends.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&alice, root),
            alice_root_stake + pending_root / 2
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&bob, root),
            bob_root_stake + pending_root / 2
        );

        // Lets change the take value. (Bob is greedy.)
        ChildkeyTake::<Test>::insert(bob, alpha, u16::MAX);

        // Lets drain
        let pending_alpha: u64 = 1_000_000_000;
        let pending_root: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(alpha, pending_alpha, pending_root, 0, 0);

        // Alice makes nothing
        assert_eq!(AlphaDividendsPerSubnet::<Test>::get(alpha, alice), 0);
        assert_eq!(TaoDividendsPerSubnet::<Test>::get(alpha, alice), 0);
        // Bob makes it all.
        assert_eq!(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob),
            (I96F32::from_num(pending_alpha) * I96F32::from_num(1.0 - 0.495412844)).to_num::<u64>()
        );
        assert_eq!(TaoDividendsPerSubnet::<Test>::get(alpha, bob), pending_root);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain_half_proportion --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain_half_proportion() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let root: u16 = 0;
        let alpha: u16 = 1;
        add_network(root, 1, 0);
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
        let alice_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            root,
            alice_root_stake,
        );
        let bob_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            root,
            alice_root_stake,
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake: u64 = 1_000_000_000;
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
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(alpha, pending_alpha, 0, 0, 0);

        // Alice and Bob make the same amount.
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, alice),
            pending_alpha / 4,
            10,
        );
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob),
            pending_alpha / 4,
            10,
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain_with_take --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain_with_take() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let root: u16 = 0;
        let alpha: u16 = 1;
        add_network(root, 1, 0);
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
        let alice_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            root,
            alice_root_stake,
        );
        let bob_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            root,
            alice_root_stake,
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            alpha,
            bob_alpha_stake,
        );
        // Set Bob as 100% child of Alice on root.
        ChildkeyTake::<Test>::insert(bob, alpha, u16::MAX);
        mock_set_children_no_epochs(alpha, &alice, &[(u64::MAX, bob)]);
        // Set Bob childkey take to zero.
        Delegates::<Test>::insert(alice, 0);
        Delegates::<Test>::insert(bob, 0);

        // Lets drain!
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(alpha, pending_alpha, 0, 0, 0);

        // Alice and Bob make the same amount.
        close(AlphaDividendsPerSubnet::<Test>::get(alpha, alice), 0, 10);
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob),
            pending_alpha / 2,
            10,
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_drain_with_half_take --exact --show-output --nocapture
#[test]
fn test_get_root_children_drain_with_half_take() {
    new_test_ext(1).execute_with(|| {
        // Init netuid 1
        let root: u16 = 0;
        let alpha: u16 = 1;
        add_network(root, 1, 0);
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
        let alice_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            root,
            alice_root_stake,
        );
        let bob_root_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &bob,
            &cold_bob,
            root,
            alice_root_stake,
        );
        // Add stake for Alice and Bob on netuid.
        let alice_alpha_stake: u64 = 1_000_000_000;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &alice,
            &cold_alice,
            alpha,
            alice_alpha_stake,
        );
        let bob_alpha_stake: u64 = 1_000_000_000;
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
        let pending_alpha: u64 = 1_000_000_000;
        SubtensorModule::drain_pending_emission(alpha, pending_alpha, 0, 0, 0);

        // Alice and Bob make the same amount.
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, alice),
            pending_alpha / 8,
            10000,
        );
        close(
            AlphaDividendsPerSubnet::<Test>::get(alpha, bob),
            3 * (pending_alpha / 8),
            10000,
        );
    });
}

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_get_root_children_with_weights --exact --show-output --nocapture
// #[test]
// fn test_get_root_children_with_weights() {
//     new_test_ext(1).execute_with(|| {
//         // Init netuid 1
//         let root: u16 = 0;
//         let alpha: u16 = 1;
//         add_network(root, 1, 0);
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
//         let alice_root_stake: u64 = 1_000_000_000;
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &alice,
//             &cold,
//             root,
//             alice_root_stake,
//         );
//         let bob_root_stake: u64 = 1_000_000_000;
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &bob,
//             &cold,
//             root,
//             alice_root_stake,
//         );
//         // Add stake for Alice and Bob on netuid.
//         let alice_alpha_stake: u64 = 1_000_000_000;
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &alice,
//             &cold,
//             alpha,
//             alice_alpha_stake,
//         );
//         let bob_alpha_stake: u64 = 1_000_000_000;
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
//         let pending_alpha: u64 = 1_000_000_000;
//         SubtensorModule::drain_pending_emission(alpha, pending_alpha, 0, 0, 0);

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

        let netuid = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

        let pending_tao: u64 = 1_000_000_000;
        let pending_alpha: u64 = 0; // None to valis
        let owner_cut: u64 = 0;
        let mut incentives: BTreeMap<U256, u64> = BTreeMap::new();

        // Give incentive to other_hk
        incentives.insert(other_hk, 10_000_000);

        // Give incentives to subnet_owner_hk
        incentives.insert(subnet_owner_hk, 10_000_000);

        // Verify stake before
        let subnet_owner_stake_before =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&subnet_owner_hk, netuid);
        assert_eq!(subnet_owner_stake_before, 0);
        let other_stake_before = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);
        assert_eq!(other_stake_before, 0);

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
        assert_eq!(subnet_owner_stake_after, 0);
        let other_stake_after = SubtensorModule::get_stake_for_hotkey_on_subnet(&other_hk, netuid);
        assert!(other_stake_after > 0);
    });
}

#[test]
fn test_calculate_dividend_distribution_totals() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (u64, u64)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, I96F32> = BTreeMap::new();

        let pending_validator_alpha: u64 = 183_123_567_452;
        let pending_tao: u64 = 837_120_949_872;
        let tao_weight: I96F32 = I96F32::saturating_from_num(0.18); // 18%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302, 2_342_352));
        stake_map.insert(hotkeys[1], (23_423, 859_273));
        dividends.insert(hotkeys[0], 77_783_738.into());
        dividends.insert(hotkeys[1], 19_283_940.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<I96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<I96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            pending_validator_alpha,
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao,
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividend_distribution_total_only_tao() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (u64, u64)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, I96F32> = BTreeMap::new();

        let pending_validator_alpha: u64 = 0;
        let pending_tao: u64 = 837_120_949_872;
        let tao_weight: I96F32 = I96F32::saturating_from_num(0.18); // 18%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302, 2_342_352));
        stake_map.insert(hotkeys[1], (23_423, 859_273));
        dividends.insert(hotkeys[0], 77_783_738.into());
        dividends.insert(hotkeys[1], 19_283_940.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<I96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<I96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            pending_validator_alpha,
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao,
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividend_distribution_total_no_tao_weight() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (u64, u64)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, I96F32> = BTreeMap::new();

        let pending_validator_alpha: u64 = 183_123_567_452;
        let pending_tao: u64 = 0; // If tao weight is 0, then only alpha dividends should be input.
        let tao_weight: I96F32 = I96F32::saturating_from_num(0.0); // 0%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302, 2_342_352));
        stake_map.insert(hotkeys[1], (23_423, 859_273));
        dividends.insert(hotkeys[0], 77_783_738.into());
        dividends.insert(hotkeys[1], 19_283_940.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<I96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<I96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            pending_validator_alpha,
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao,
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividend_distribution_total_only_alpha() {
    new_test_ext(1).execute_with(|| {
        let mut stake_map: BTreeMap<U256, (u64, u64)> = BTreeMap::new();
        let mut dividends: BTreeMap<U256, I96F32> = BTreeMap::new();

        let pending_validator_alpha: u64 = 183_123_567_452;
        let pending_tao: u64 = 0;
        let tao_weight: I96F32 = I96F32::saturating_from_num(0.18); // 18%

        let hotkeys = [U256::from(0), U256::from(1)];

        // Stake map and dividends shouldn't matter for this test.
        stake_map.insert(hotkeys[0], (4_859_302, 2_342_352));
        stake_map.insert(hotkeys[1], (23_423, 859_273));
        dividends.insert(hotkeys[0], 77_783_738.into());
        dividends.insert(hotkeys[1], 19_283_940.into());

        let (alpha_dividends, tao_dividends) = SubtensorModule::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        // Verify the total of each dividends type is close to the inputs.
        let total_alpha_dividends = alpha_dividends.values().sum::<I96F32>();
        let total_tao_dividends = tao_dividends.values().sum::<I96F32>();

        assert_abs_diff_eq!(
            total_alpha_dividends.saturating_to_num::<u64>(),
            pending_validator_alpha,
            epsilon = 1_000
        );
        assert_abs_diff_eq!(
            total_tao_dividends.saturating_to_num::<u64>(),
            pending_tao,
            epsilon = 1_000
        );
    });
}

#[test]
fn test_calculate_dividends_and_incentives_only_alpha() {
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
            1,
        );

        let pending_alpha = 123_456_789;
        let pending_swapped = 0; // Only alpha output.

        let (incentives, dividends) = SubtensorModule::calculate_dividends_and_incentives(
            netuid,
            pending_alpha,
            pending_swapped,
        );

        let incentives_total = incentives.values().sum::<u64>();
        let dividends_total = dividends
            .values()
            .sum::<I96F32>()
            .saturating_to_num::<u64>();

        assert_eq!(
            dividends_total.saturating_add(incentives_total),
            pending_alpha
        );
    });
}
