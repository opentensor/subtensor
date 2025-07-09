#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]

use approx::assert_abs_diff_eq;
use codec::Encode;
use frame_support::weights::Weight;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{Config, RawOrigin};
use sp_core::{Get, H160, H256, U256};
use sp_runtime::SaturatedConversion;
use substrate_fixed::types::U96F32;
use subtensor_swap_interface::SwapHandler;

use super::mock;
use super::mock::*;
use crate::*;

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_basic_functionality -- --nocapture
#[test]
fn test_update_tao_weight_basic_functionality() {
    new_test_ext(1).execute_with(|| {
        // Setup: Create some test subnets with TAO
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        // Set initial TAO reserves and weight
        let initial_reserves = 100u64;
        let initial_weight = U96F32::saturating_from_num(0.15); // 15%

        TaoReservesAtLastBlock::<Test>::set(initial_reserves);
        Pallet::<Test>::set_tao_weight_from_float(initial_weight);
        NetworksAdded::<Test>::insert(&netuid1, true);
        NetworksAdded::<Test>::insert(&netuid2, true);
        SubnetTAO::<Test>::insert(&netuid1, 500u64);
        SubnetTAO::<Test>::insert(&netuid2, 600u64);

        // Mock the subnet list (excluding ROOT)
        let mut subnets = vec![netuid1, netuid2];

        // Set block emission
        let block_emission = U96F32::saturating_from_num(1);

        // Execute the function
        Pallet::<Test>::update_tao_weight(block_emission);

        // Verify the weight was updated
        let new_weight = Pallet::<Test>::get_tao_weight();
        assert_ne!(new_weight, initial_weight);

        // Verify reserves were updated
        let new_reserves = TaoReservesAtLastBlock::<Test>::get();
        log::debug!("New reserves: {}, Expected: {}", new_reserves, 500 + 600);
        assert_eq!(new_reserves, 1100u64); // 500 + 600 from subnets
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_overfilled_scenario -- --nocapture
#[test]
fn test_update_tao_weight_overfilled_scenario() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total > expected total (overfilled)
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        let initial_reserves = 1000u64;
        let initial_weight = U96F32::saturating_from_num(0.15);

        TaoReservesAtLastBlock::<Test>::set(initial_reserves);
        Pallet::<Test>::set_tao_weight_from_float(initial_weight);
        NetworksAdded::<Test>::insert(&netuid1, true);
        NetworksAdded::<Test>::insert(&netuid2, true);

        // Set high subnet TAO amounts (overfilled)
        SubnetTAO::<Test>::insert(&netuid1, 800u64);
        SubnetTAO::<Test>::insert(&netuid2, 500u64);

        let block_emission = U96F32::saturating_from_num(100);

        // Current total: 1300, Expected: 1100, Diff: +200 (overfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight = TaoWeight::<Test>::get();

        // Weight should increase when tao reserves growth is greater than emission
        assert!(new_weight > Pallet::<Test>::convert_float_to_u64(initial_weight));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_underfilled_scenario -- --nocapture
#[test]
fn test_update_tao_weight_underfilled_scenario() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total < expected total (underfilled)
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        let initial_reserves = 1000u64;
        let initial_weight = U96F32::saturating_from_num(0.15);

        TaoReservesAtLastBlock::<Test>::set(initial_reserves);
        Pallet::<Test>::set_tao_weight_from_float(initial_weight);
        NetworksAdded::<Test>::insert(&netuid1, true);
        NetworksAdded::<Test>::insert(&netuid2, true);

        // Set low subnet TAO amounts (underfilled)
        SubnetTAO::<Test>::insert(&netuid1, 500u64);
        SubnetTAO::<Test>::insert(&netuid2, 600u64);

        let block_emission = U96F32::saturating_from_num(500);

        // Current total: 1100, Expected: 1500, Diff: -400 (underfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight = TaoWeight::<Test>::get();

        // Weight should decrease when tao reserves growth is less than emission
        assert!(new_weight < Pallet::<Test>::convert_float_to_u64(initial_weight));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_min_bound_clamping -- --nocapture
#[test]
fn test_update_tao_weight_min_bound_clamping() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total < expected total (underfilled) to force weight down
        let netuid1 = NetUid::from(1);

        let initial_reserves = 1_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at min
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.09));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        // Create massive underfill to force weight down
        NetworksAdded::<Test>::insert(&netuid1, true);
        SubnetTAO::<Test>::insert(&netuid1, 100u64);

        let block_emission = U96F32::saturating_from_num(1000);

        // Current total: 100, Expected: 2_000, Diff: -1_900 (underfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight = TaoWeight::<Test>::get();
        let min_weight = Pallet::<Test>::convert_float_to_u64(U96F32::saturating_from_num(0.09));

        // Weight should be clamped to minimum
        assert_eq!(new_weight, min_weight);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_max_bound_clamping -- --nocapture
#[test]
fn test_update_tao_weight_max_bound_clamping() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total > expected total (overfilled) to force weight up
        let netuid1 = NetUid::from(1);

        let initial_reserves = 1000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at max
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.18));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        // Create massive overfill to force weight up
        NetworksAdded::<Test>::insert(&netuid1, true);
        SubnetTAO::<Test>::insert(&netuid1, 10_000u64);

        let block_emission = U96F32::saturating_from_num(1000);

        // Current total: 10_000, Expected: 2_000, Diff: +8_000 (overfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight = TaoWeight::<Test>::get();
        let max_weight = Pallet::<Test>::convert_float_to_u64(U96F32::saturating_from_num(0.18));

        // Weight should be clamped to maximum
        assert_eq!(new_weight, max_weight);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_equal_growth -- --nocapture
#[test]
fn test_update_tao_weight_equal_growth() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total > expected total (overfilled) to force weight up
        let netuid1 = NetUid::from(1);

        let initial_reserves = 1000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at max
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.15));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        // Create massive overfill to force weight up
        NetworksAdded::<Test>::insert(&netuid1, true);
        SubnetTAO::<Test>::insert(&netuid1, 1_100u64);

        let block_emission = U96F32::saturating_from_num(100);

        // Current total: 1_100, Expected: 1_100, Diff: 0 (equal growth)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight = TaoWeight::<Test>::get();

        // Weight should be clamped to maximum
        assert_eq!(new_weight, initial_weight_u64);
    });
}
