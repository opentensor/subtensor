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

