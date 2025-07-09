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
        let initial_reserves = 100_000_000_000_000u64;
        let initial_weight = U96F32::saturating_from_num(0.15); // 15%

        TaoReservesAtLastBlock::<Test>::set(initial_reserves);
        Pallet::<Test>::set_tao_weight_from_float(initial_weight);
        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);
        SubnetTAO::<Test>::insert(netuid1, 50_000_000_000_000u64);
        SubnetTAO::<Test>::insert(netuid2, 60_000_000_000_000u64);

        // Mock the subnet list (excluding ROOT)
        let mut subnets = [netuid1, netuid2];

        // Set block emission
        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Execute the function
        Pallet::<Test>::update_tao_weight(block_emission);

        // Verify the weight was updated
        let new_weight = Pallet::<Test>::get_tao_weight();
        assert_ne!(new_weight, initial_weight);

        // Verify reserves were updated
        let new_reserves = TaoReservesAtLastBlock::<Test>::get();
        assert_eq!(new_reserves, 110_000_000_000_000u64); // 50k + 60k from subnets
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_overfilled_scenario -- --nocapture
#[test]
fn test_update_tao_weight_overfilled_scenario() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total > expected total (overfilled)
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        // 100k tao
        let initial_reserves = 100_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at 15%
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.15));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);

        // Set high subnet TAO amounts (overfilled, 10 tao growth)
        SubnetTAO::<Test>::insert(netuid1, 50_010_000_000_000u64);
        SubnetTAO::<Test>::insert(netuid2, 50_000_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 100_010_000_000_000, Expected: 100_001_000_000_000, Diff: +9_000_000_000 (overfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        log::debug!(
            "New weight float: {}, Initial weight float: {}, New weight u64: {}, Initial weight u64: {}",
            new_weight_float,
            initial_weight_float,
            new_weight_u64,
            initial_weight_u64
        );

        // Weight should increase when tao reserves growth is greater than emission
        assert!(new_weight_u64 > initial_weight_u64);
        assert!(new_weight_float > initial_weight_float);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_underfilled_scenario -- --nocapture
#[test]
fn test_update_tao_weight_underfilled_scenario() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total < expected total (underfilled)
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        let initial_reserves = 100_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at 15%
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.15));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);

        // Set low subnet TAO amounts (underfilled)
        SubnetTAO::<Test>::insert(netuid1, 50_000_000_000_000u64);
        SubnetTAO::<Test>::insert(netuid2, 50_000_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 1100, Expected: 1500, Diff: -400 (underfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        log::debug!(
            "New weight float: {}, Initial weight float: {}, New weight u64: {}, Initial weight u64: {}",
            new_weight_float,
            initial_weight_float,
            new_weight_u64,
            initial_weight_u64
        );

        // Weight should decrease when tao reserves growth is less than emission
        assert!(new_weight_u64 < initial_weight_u64);
        assert!(new_weight_float < initial_weight_float);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_min_bound_clamping -- --nocapture
#[test]
fn test_update_tao_weight_min_bound_clamping() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total < expected total (underfilled) to force weight down
        let netuid1 = NetUid::from(1);

        let initial_reserves = 100_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at min
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.09));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        // Create massive underfill to force weight down
        NetworksAdded::<Test>::insert(netuid1, true);
        SubnetTAO::<Test>::insert(netuid1, 50_000_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 50,000, Expected: 100,001, Diff: -49,999 (underfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        // Weight should be clamped to minimum
        assert_eq!(new_weight_u64, initial_weight_u64);
        assert_eq!(new_weight_float, initial_weight_float);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_max_bound_clamping -- --nocapture
#[test]
fn test_update_tao_weight_max_bound_clamping() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total > expected total (overfilled) to force weight up
        let netuid1 = NetUid::from(1);

        let initial_reserves = 100_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at max
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.18));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        // Create massive overfill to force weight up
        NetworksAdded::<Test>::insert(netuid1, true);
        SubnetTAO::<Test>::insert(netuid1, 150_000_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 150,000, Expected: 150,001, Diff: +49,999 (overfilled)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        // Weight should be clamped to maximum
        assert_eq!(new_weight_u64, initial_weight_u64);
        assert_eq!(new_weight_float, initial_weight_float);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_equal_growth -- --nocapture
#[test]
fn test_update_tao_weight_equal_growth() {
    new_test_ext(1).execute_with(|| {
        // Setup: Current total > expected total (overfilled) to force weight up
        let netuid1 = NetUid::from(1);

        let initial_reserves = 100_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at max
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.15));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        // Create massive overfill to force weight up
        NetworksAdded::<Test>::insert(netuid1, true);
        SubnetTAO::<Test>::insert(netuid1, 100_001_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 100,001, Expected: 100,001, Diff: 0 (equal growth)
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        // Weight should stay the same
        assert_eq!(new_weight_u64, initial_weight_u64);
        assert_eq!(new_weight_float, initial_weight_float);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_full_swing_up -- --nocapture
#[test]
fn test_update_tao_weight_full_swing_up() {
    new_test_ext(1).execute_with(|| {
        // Setup single block swing of 1k TAO
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        // realistic current reserves of 900k TAO
        let initial_reserves = 900_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at min
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.09));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);

        // Create massive overfill of 270k TAO, enough to swing the full 9% up
        SubnetTAO::<Test>::insert(netuid1, 500_000_000_000_000u64);
        SubnetTAO::<Test>::insert(netuid2, 670_001_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 1_170_001_000_000_000, Expected: 900_001_000_000_000, Diff: 270_000_000_000_000
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        log::debug!(
            "New weight: {}, New weight float: {}, Initial weight: {}, Initial weight float: {}",
            new_weight_u64,
            new_weight_float,
            initial_weight_u64,
            initial_weight_float
        );

        // Assert that new weight is roughly max weight of 0.18
        let expected_max_weight_u64 =
            Pallet::<Test>::convert_float_to_u64(U96F32::saturating_from_num(0.18));
        let epsilon = u64::MAX / 1000; // 0.1% of the full range
        assert_abs_diff_eq!(new_weight_u64, expected_max_weight_u64, epsilon = epsilon);
        // manually added for a bit of clarity about the final value
        assert_eq!(new_weight_float, U96F32::saturating_from_num(0.1799587754));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_full_swing_down -- --nocapture
#[test]
fn test_update_tao_weight_full_swing_down() {
    new_test_ext(1).execute_with(|| {
        // Setup single block swing of 1k TAO
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        // realistic current reserves of 900k TAO
        let initial_reserves = 900_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight at min
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.18));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);

        // Create massive overfill of 270k TAO, enough to swing the full 9% up
        SubnetTAO::<Test>::insert(netuid1, 500_000_000_000_000u64);
        SubnetTAO::<Test>::insert(netuid2, 130_001_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 630_001_000_000_000, Expected: 900_001_000_000_000, Diff: -270_000_000_000_000
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        log::debug!(
            "New weight: {}, New weight float: {}, Initial weight: {}, Initial weight float: {}",
            new_weight_u64,
            new_weight_float,
            initial_weight_u64,
            initial_weight_float
        );

        // Assert that new weight is roughly min weight of 0.09
        let expected_min_weight_u64 =
            Pallet::<Test>::convert_float_to_u64(U96F32::saturating_from_num(0.09));
        let epsilon = u64::MAX / 1000; // 0.1% of the full range
        assert_abs_diff_eq!(new_weight_u64, expected_min_weight_u64, epsilon = epsilon);
        // manually added for a bit of clarity about the final value
        assert_eq!(new_weight_float, U96F32::saturating_from_num(0.0900412237));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_half_swing_up -- --nocapture
#[test]
fn test_update_tao_weight_half_swing_up() {
    new_test_ext(1).execute_with(|| {
        // Setup single block swing of 1k TAO
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        // realistic current reserves of 900k TAO
        let initial_reserves = 900_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight 11% (testing for 4.5% swing)
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.11));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);

        // Create massive overfill of 270k TAO, enough to swing the full 9% up
        SubnetTAO::<Test>::insert(netuid1, 500_000_000_000_000u64);
        SubnetTAO::<Test>::insert(netuid2, 535_001_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 1_035_001_000_000_000, Expected: 900_001_000_000_000, Diff: 135_000_000_000_000
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        log::debug!(
            "New weight: {}, New weight float: {}, Initial weight: {}, Initial weight float: {}",
            new_weight_u64,
            new_weight_float,
            initial_weight_u64,
            initial_weight_float
        );

        // Assert that new weight is roughly 15.5%
        let expected_weight_u64 =
            Pallet::<Test>::convert_float_to_u64(U96F32::saturating_from_num(0.155));
        let epsilon = u64::MAX / 1000; // 0.1% of the full range
        assert_abs_diff_eq!(new_weight_u64, expected_weight_u64, epsilon = epsilon);
        // manually added for a bit of clarity about the final value
        assert_eq!(new_weight_float, U96F32::saturating_from_num(0.1549793875));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=DEBUG cargo test --release -p pallet-subtensor test_update_tao_weight_half_swing_down -- --nocapture
#[test]
fn test_update_tao_weight_half_swing_down() {
    new_test_ext(1).execute_with(|| {
        // Setup single block swing of 1k TAO
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        // realistic current reserves of 900k TAO
        let initial_reserves = 900_000_000_000_000u64;
        TaoReservesAtLastBlock::<Test>::set(initial_reserves);

        // Set inital weight 16% (testing for 4.5% swing)
        Pallet::<Test>::set_tao_weight_from_float(U96F32::saturating_from_num(0.16));
        let initial_weight_float = Pallet::<Test>::get_tao_weight();
        let initial_weight_u64 = TaoWeight::<Test>::get();

        NetworksAdded::<Test>::insert(netuid1, true);
        NetworksAdded::<Test>::insert(netuid2, true);

        // Create massive overfill of 270k TAO, enough to swing the full 9% up
        SubnetTAO::<Test>::insert(netuid1, 500_000_000_000_000u64);
        SubnetTAO::<Test>::insert(netuid2, 264_999_000_000_000u64);

        let block_emission = U96F32::saturating_from_num(1_000_000_000u64);

        // Current total: 764_999_000_000_000, Expected: 900_001_000_000_000, Diff: -135_000_000_000_000
        Pallet::<Test>::update_tao_weight(block_emission);

        let new_weight_u64 = TaoWeight::<Test>::get();
        let new_weight_float = Pallet::<Test>::get_tao_weight();

        log::debug!(
            "New weight: {}, New weight float: {}, Initial weight: {}, Initial weight float: {}",
            new_weight_u64,
            new_weight_float,
            initial_weight_u64,
            initial_weight_float
        );

        // Assert that new weight is roughly 11.5%
        let expected_weight_u64 =
            Pallet::<Test>::convert_float_to_u64(U96F32::saturating_from_num(0.115));
        let epsilon = u64::MAX / 1000; // 0.1% of the full range
        assert_abs_diff_eq!(new_weight_u64, expected_weight_u64, epsilon = epsilon);
        // manually added for a bit of clarity about the final value
        assert_eq!(new_weight_float, U96F32::saturating_from_num(0.1150199452));
    });
}

// TODO multi block tests
