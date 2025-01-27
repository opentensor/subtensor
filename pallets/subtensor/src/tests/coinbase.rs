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
                    if alpha_in_emission > 0 || tao_in_emission > 0 {
                        assert!((tao_in_emission as f64 / alpha_in_emission as f64 - price).abs() < 1e-1, "Ratio of tao_in_emission to alpha_in_emission is not equal to price");
                    }
                }
            }
        }
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_dynamic_function_price_equal_emission --exact --show-output --nocapture
#[test]
fn test_dynamic_function_price_equal_emission() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let tao_subnet_emission: u64 = 100_000_000;
        let tao_block_emission: u64 = 1_000_000_000;
        let alpha_block_emission: u64 = 1_000_000_000;
        SubnetTAO::<Test>::insert(netuid, 1_000_000_000);
        SubnetAlphaIn::<Test>::insert(netuid, 1_000_000_000);
        add_network(netuid, 110, 100);
        let (tao_in, alpha_in, alpha_out): (u64, u64, u64) =
            SubtensorModule::get_dynamic_tao_emission(
                netuid,
                tao_subnet_emission,
                alpha_block_emission,
            );
        assert_eq!(tao_in, tao_subnet_emission); // at price == tao_in == tao_subnet_emission
        let expected_alpha_in: u64 =
            (alpha_block_emission * tao_subnet_emission) / tao_block_emission;
        close(alpha_in, expected_alpha_in, 10);
        close(alpha_out, alpha_block_emission, 10);
    });
}

// Verifies that the total stake after the coinbase is only increased by the coinbase emission.
// Avoids TAO weight.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_total_stake_after_coinbase_no_tao_weight --exact --show-output --nocapture
#[test]
fn test_total_stake_after_coinbase_no_tao_weight() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        // Set TAO weight to 0
        SubtensorModule::set_tao_weight(0);
        // Set owner cut to ~11.11%
        SubtensorModule::set_subnet_owner_cut(u16::MAX / 9);
        let total_coinbase_emission: I96F32 = I96F32::from_num(1_123_456_789);
        let epsilon: u64 = 100;

        // Define hotkeys and coldkeys
        let hotkey_a: U256 = U256::from(1);
        let hotkey_b: U256 = U256::from(2);
        let hotkey_c: U256 = U256::from(3);
        let coldkey_a: U256 = U256::from(100);
        let coldkey_b: U256 = U256::from(101);
        let coldkey_c: U256 = U256::from(102);

        // Register neurons with decreasing stakes
        register_ok_neuron(netuid, hotkey_a, coldkey_a, 0);
        register_ok_neuron(netuid, hotkey_b, coldkey_b, 0);
        register_ok_neuron(netuid, hotkey_c, coldkey_c, 0);

        // Add initial stakes
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_a, 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_b, 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_c, 1_000);

        // Swap to alpha
        let total_tao: I96F32 = I96F32::from_num(300_000 + 100_000 + 50_000);
        let total_alpha: I96F32 = I96F32::from_num(SubtensorModule::swap_tao_for_alpha(
            netuid,
            total_tao.saturating_to_num::<u64>(),
        ));

        // Set the stakes directly
        // This avoids needing to swap tao to alpha, impacting the initial stake distribution.
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_a,
            &coldkey_a,
            netuid,
            (total_alpha * I96F32::from_num(300_000) / total_tao).saturating_to_num::<u64>(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_b,
            &coldkey_b,
            netuid,
            (total_alpha * I96F32::from_num(100_000) / total_tao).saturating_to_num::<u64>(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_c,
            &coldkey_c,
            netuid,
            (total_alpha * I96F32::from_num(50_000) / total_tao).saturating_to_num::<u64>(),
        );

        // Get the total stake on the network
        let mut total_stake_before = 0;
        for (hotkey, netuid_i, alpha) in TotalHotkeyAlpha::<Test>::iter() {
            if netuid_i == netuid {
                total_stake_before += alpha;
            } else {
                assert!(
                    alpha == 0,
                    "Alpha should be 0 for non-subnet hotkeys, but is {:?} on netuid {:?}",
                    alpha,
                    netuid_i
                );
            }
        }

        log::info!("total_stake_before: {:?}", total_stake_before);

        // Run the coinbase
        SubtensorModule::run_coinbase(total_coinbase_emission);

        // Get the total stake on the network
        let mut total_stake_after = 0;
        for (hotkey, netuid_i, alpha) in TotalHotkeyAlpha::<Test>::iter() {
            if netuid_i == netuid {
                total_stake_after += alpha;
            } else {
                assert!(
                    alpha == 0,
                    "Alpha should be 0 for non-subnet hotkeys, but is {:?} on netuid {:?}",
                    alpha,
                    netuid_i
                );
            }
        }
        assert_abs_diff_eq!(
            total_stake_after,
            total_stake_before + total_coinbase_emission.saturating_to_num::<u64>(),
            epsilon = epsilon
        );
    });
}

// Verifies that the total stake after the coinbase is only increased by the coinbase emission.
// Includes TAO weight.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_total_stake_after_coinbase --exact --show-output --nocapture
#[test]
fn test_total_stake_after_coinbase() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        // Set TAO weight to 18%
        SubtensorModule::set_tao_weight(I96F32::from_num(0.18).saturating_to_num::<u64>());
        // Set owner cut to ~11.11%
        SubtensorModule::set_subnet_owner_cut(u16::MAX / 9);
        let total_coinbase_emission: I96F32 = I96F32::from_num(1_123_456_789);
        let epsilon: u64 = 100;

        // Define hotkeys and coldkeys
        let hotkey_a: U256 = U256::from(1);
        let hotkey_b: U256 = U256::from(2);
        let hotkey_c: U256 = U256::from(3);
        let coldkey_a: U256 = U256::from(100);
        let coldkey_b: U256 = U256::from(101);
        let coldkey_c: U256 = U256::from(102);

        // Register neurons with decreasing stakes
        register_ok_neuron(netuid, hotkey_a, coldkey_a, 0);
        register_ok_neuron(netuid, hotkey_b, coldkey_b, 0);
        register_ok_neuron(netuid, hotkey_c, coldkey_c, 0);

        // Add initial stakes
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_a, 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_b, 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_c, 1_000);

        // Swap to alpha
        let total_tao: I96F32 = I96F32::from_num(300_000 + 100_000 + 50_000);
        let total_alpha: I96F32 = I96F32::from_num(SubtensorModule::swap_tao_for_alpha(
            netuid,
            total_tao.saturating_to_num::<u64>(),
        ));

        // Set the stakes directly
        // This avoids needing to swap tao to alpha, impacting the initial stake distribution.
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_a,
            &coldkey_a,
            netuid,
            (total_alpha * I96F32::from_num(300_000) / total_tao).saturating_to_num::<u64>(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_b,
            &coldkey_b,
            netuid,
            (total_alpha * I96F32::from_num(100_000) / total_tao).saturating_to_num::<u64>(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_c,
            &coldkey_c,
            netuid,
            (total_alpha * I96F32::from_num(50_000) / total_tao).saturating_to_num::<u64>(),
        );

        // Stake some to root
        let stake_to_root: u64 = 10_000_000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_a, stake_to_root);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_a,
            &coldkey_a,
            netuid,
            stake_to_root,
        );

        let alpha_price = SubtensorModule::get_alpha_price(netuid);
        log::info!("alpha_price: {:?}", alpha_price);

        // Get the total stake on the network
        let mut total_stake_before = 0;
        for (hotkey, netuid_i, alpha) in TotalHotkeyAlpha::<Test>::iter() {
            if netuid_i == netuid {
                total_stake_before += alpha;
            } else if netuid == SubtensorModule::get_root_netuid() {
                let as_alpha: I96F32 = I96F32::from_num(alpha) / alpha_price;
                total_stake_before += as_alpha.saturating_to_num::<u64>();
            } else {
                assert!(
                    alpha == 0,
                    "Alpha should be 0 for non-subnet hotkeys, but is {:?} on netuid {:?}",
                    alpha,
                    netuid_i
                );
            }
        }

        log::info!("total_stake_before: {:?}", total_stake_before);

        // Run the coinbase
        SubtensorModule::run_coinbase(total_coinbase_emission);

        // Get the total stake on the network
        let mut total_stake_after = 0;
        for (hotkey, netuid_i, alpha) in TotalHotkeyAlpha::<Test>::iter() {
            if netuid_i == netuid {
                total_stake_after += alpha;
            } else if netuid == SubtensorModule::get_root_netuid() {
                let as_alpha: I96F32 = I96F32::from_num(alpha) / alpha_price;
                total_stake_after += as_alpha.saturating_to_num::<u64>();
            } else {
                assert!(
                    alpha == 0,
                    "Alpha should be 0 for non-subnet hotkeys, but is {:?} on netuid {:?}",
                    alpha,
                    netuid_i
                );
            }
        }
        assert_abs_diff_eq!(
            total_stake_after,
            total_stake_before + total_coinbase_emission.saturating_to_num::<u64>(),
            epsilon = epsilon
        );
    });
}

// Verifies that the total issuance after the coinbase is only increased by the coinbase emission.
// Includes TAO weight.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::coinbase::test_total_issuance_after_coinbase --exact --show-output --nocapture
#[test]
fn test_total_issuance_after_coinbase() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);
        // Set TAO weight to 18%
        SubtensorModule::set_tao_weight(I96F32::from_num(0.18).saturating_to_num::<u64>());
        // Set owner cut to ~11.11%
        SubtensorModule::set_subnet_owner_cut(u16::MAX / 9);
        let total_coinbase_emission: I96F32 = I96F32::from_num(1_123_456_789);
        let epsilon: u64 = 100;

        // Define hotkeys and coldkeys
        let hotkey_a: U256 = U256::from(1);
        let hotkey_b: U256 = U256::from(2);
        let hotkey_c: U256 = U256::from(3);
        let coldkey_a: U256 = U256::from(100);
        let coldkey_b: U256 = U256::from(101);
        let coldkey_c: U256 = U256::from(102);

        // Register neurons with decreasing stakes
        register_ok_neuron(netuid, hotkey_a, coldkey_a, 0);
        register_ok_neuron(netuid, hotkey_b, coldkey_b, 0);
        register_ok_neuron(netuid, hotkey_c, coldkey_c, 0);

        // Add initial stakes
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_a, 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_b, 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_c, 1_000);

        // Swap to alpha
        let total_tao: I96F32 = I96F32::from_num(300_000 + 100_000 + 50_000);
        let total_alpha: I96F32 = I96F32::from_num(SubtensorModule::swap_tao_for_alpha(
            netuid,
            total_tao.saturating_to_num::<u64>(),
        ));

        // Set the stakes directly
        // This avoids needing to swap tao to alpha, impacting the initial stake distribution.
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_a,
            &coldkey_a,
            netuid,
            (total_alpha * I96F32::from_num(300_000) / total_tao).saturating_to_num::<u64>(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_b,
            &coldkey_b,
            netuid,
            (total_alpha * I96F32::from_num(100_000) / total_tao).saturating_to_num::<u64>(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_c,
            &coldkey_c,
            netuid,
            (total_alpha * I96F32::from_num(50_000) / total_tao).saturating_to_num::<u64>(),
        );

        // Stake some to root
        let stake_to_root: u64 = 10_000_000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_a, stake_to_root);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_a,
            &coldkey_a,
            netuid,
            stake_to_root,
        );

        let alpha_price = SubtensorModule::get_alpha_price(netuid);
        log::info!("alpha_price: {:?}", alpha_price);

        // Get the total issuance
        let mut total_issuance_before = TotalIssuance::<Test>::get();
        log::info!("total_issuance_before: {:?}", total_issuance_before);

        // Run the coinbase
        SubtensorModule::run_coinbase(total_coinbase_emission);

        // Compare
        let total_issuance_after = TotalIssuance::<Test>::get();
        assert_abs_diff_eq!(
            total_issuance_after,
            total_issuance_before + total_coinbase_emission.saturating_to_num::<u64>(),
            epsilon = epsilon
        );
    });
}
