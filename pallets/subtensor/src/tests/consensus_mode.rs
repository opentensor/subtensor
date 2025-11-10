#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used
)]

use frame_support::{assert_err, assert_ok};
use sp_core::U256;
use substrate_fixed::types::I32F32;
use subtensor_runtime_common::NetUid;

use super::mock::*;
use crate::*;

/// Test setting consensus mode when liquid alpha is disabled
#[test]
fn test_set_consensus_mode_liquid_alpha_disabled() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(1 + 456);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let signer = RuntimeOrigin::signed(coldkey);

        // Setup network
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey));
        assert_ok!(SubtensorModule::register_network(signer.clone(), hotkey));

        // Liquid Alpha is disabled by default
        assert!(!SubtensorModule::get_liquid_alpha_enabled(netuid));

        // Should fail to set consensus mode when liquid alpha is disabled
        assert_err!(
            SubtensorModule::do_set_liquid_alpha_consensus_mode(
                signer.clone(),
                netuid,
                ConsensusMode::Previous
            ),
            Error::<Test>::LiquidAlphaDisabled
        );

        // Enable Liquid Alpha
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);

        // Should now succeed
        assert_ok!(SubtensorModule::do_set_liquid_alpha_consensus_mode(
            signer.clone(),
            netuid,
            ConsensusMode::Previous
        ));
    });
}

/// Test that only subnet owner or root can set consensus mode
#[test]
fn test_set_consensus_mode_permissions() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(1 + 456);
        let non_owner = U256::from(999);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let owner_signer = RuntimeOrigin::signed(coldkey);
        let non_owner_signer = RuntimeOrigin::signed(non_owner);

        // Setup network
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(owner_signer.clone(), hotkey));
        assert_ok!(SubtensorModule::register_network(
            owner_signer.clone(),
            hotkey
        ));

        // Non-owner should fail
        assert_err!(
            SubtensorModule::do_set_liquid_alpha_consensus_mode(
                non_owner_signer,
                netuid,
                ConsensusMode::Previous
            ),
            DispatchError::BadOrigin
        );

        // Owner should succeed
        assert_ok!(SubtensorModule::do_set_liquid_alpha_consensus_mode(
            owner_signer,
            netuid,
            ConsensusMode::Previous
        ));
    });
}

/// Test default consensus mode is Auto
#[test]
fn test_default_consensus_mode() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(1 + 456);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        // Default should be Auto
        let mode = SubtensorModule::get_liquid_alpha_consensus_mode(netuid);
        assert_eq!(mode, ConsensusMode::Auto);
    });
}

/// Test setting and getting all consensus modes
#[test]
fn test_set_and_get_consensus_modes() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(1 + 456);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let signer = RuntimeOrigin::signed(coldkey);

        // Setup network
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey));
        assert_ok!(SubtensorModule::register_network(signer.clone(), hotkey));

        // Test Current mode
        assert_ok!(SubtensorModule::do_set_liquid_alpha_consensus_mode(
            signer.clone(),
            netuid,
            ConsensusMode::Current
        ));
        assert_eq!(
            SubtensorModule::get_liquid_alpha_consensus_mode(netuid),
            ConsensusMode::Current
        );

        // Test Previous mode
        assert_ok!(SubtensorModule::do_set_liquid_alpha_consensus_mode(
            signer.clone(),
            netuid,
            ConsensusMode::Previous
        ));
        assert_eq!(
            SubtensorModule::get_liquid_alpha_consensus_mode(netuid),
            ConsensusMode::Previous
        );

        // Test Max mode
        assert_ok!(SubtensorModule::do_set_liquid_alpha_consensus_mode(
            signer.clone(),
            netuid,
            ConsensusMode::Max
        ));
        assert_eq!(
            SubtensorModule::get_liquid_alpha_consensus_mode(netuid),
            ConsensusMode::Max
        );

        // Test Auto mode
        assert_ok!(SubtensorModule::do_set_liquid_alpha_consensus_mode(
            signer.clone(),
            netuid,
            ConsensusMode::Auto
        ));
        assert_eq!(
            SubtensorModule::get_liquid_alpha_consensus_mode(netuid),
            ConsensusMode::Auto
        );
    });
}

/// Test compute_consensus_for_liquid_alpha with Current mode
#[test]
fn test_compute_consensus_current_mode() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 1.into();
        let n: usize = 4;

        // Create network
        add_network(netuid, 0, 0);
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Current);

        // Setup some current consensus values
        let current_consensus: Vec<I32F32> = vec![
            I32F32::from_num(0.2),
            I32F32::from_num(0.3),
            I32F32::from_num(0.4),
            I32F32::from_num(0.1),
        ];

        // Store some different previous consensus values in storage
        let previous_consensus_u16: Vec<u16> = vec![
            (0.1 * u16::MAX as f32) as u16,
            (0.2 * u16::MAX as f32) as u16,
            (0.3 * u16::MAX as f32) as u16,
            (0.4 * u16::MAX as f32) as u16,
        ];
        Consensus::<Test>::insert(netuid, previous_consensus_u16);

        // Compute consensus for liquid alpha
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        // Should return current consensus (not previous)
        assert_eq!(result.len(), n);
        for i in 0..n {
            assert_eq!(result[i], current_consensus[i]);
        }
    });
}

/// Test compute_consensus_for_liquid_alpha with Previous mode
#[test]
fn test_compute_consensus_previous_mode() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 1.into();
        let n: usize = 4;

        // Create network
        add_network(netuid, 0, 0);
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Previous);

        // Setup some current consensus values
        let current_consensus: Vec<I32F32> = vec![
            I32F32::from_num(0.2),
            I32F32::from_num(0.3),
            I32F32::from_num(0.4),
            I32F32::from_num(0.1),
        ];

        // Store some different previous consensus values in storage
        let previous_values: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
        let previous_consensus_u16: Vec<u16> = previous_values
            .iter()
            .map(|&v| (v * u16::MAX as f32) as u16)
            .collect();
        Consensus::<Test>::insert(netuid, previous_consensus_u16.clone());

        // Compute consensus for liquid alpha
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        // Should return previous consensus from storage (not current)
        assert_eq!(result.len(), n);
        for i in 0..n {
            let expected = I32F32::from_num(previous_values[i]);
            // Allow small floating point difference
            let diff = if result[i] > expected {
                result[i] - expected
            } else {
                expected - result[i]
            };
            assert!(diff < I32F32::from_num(0.001), "Values should be approximately equal");
        }
    });
}

/// Test compute_consensus_for_liquid_alpha with Max mode
#[test]
fn test_compute_consensus_max_mode() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 1.into();
        let n: usize = 4;

        // Create network
        add_network(netuid, 0, 0);
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Max);

        // Setup current consensus values (some larger, some smaller than previous)
        let current_consensus: Vec<I32F32> = vec![
            I32F32::from_num(0.2), // larger than 0.1
            I32F32::from_num(0.2), // same as 0.2
            I32F32::from_num(0.25), // smaller than 0.3
            I32F32::from_num(0.5), // larger than 0.4
        ];

        // Store previous consensus values in storage
        let previous_values: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
        let previous_consensus_u16: Vec<u16> = previous_values
            .iter()
            .map(|&v| (v * u16::MAX as f32) as u16)
            .collect();
        Consensus::<Test>::insert(netuid, previous_consensus_u16);

        // Compute consensus for liquid alpha
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        // Should return element-wise max of current and previous
        assert_eq!(result.len(), n);

        // Expected max values
        let expected: Vec<f32> = vec![0.2, 0.2, 0.3, 0.5];
        for i in 0..n {
            let expected_val = I32F32::from_num(expected[i]);
            let diff = if result[i] > expected_val {
                result[i] - expected_val
            } else {
                expected_val - result[i]
            };
            assert!(diff < I32F32::from_num(0.001), "Values should be approximately equal at index {}", i);
        }
    });
}

/// Test compute_consensus_for_liquid_alpha with Max mode when network grows
/// (current consensus is longer than previous consensus)
#[test]
fn test_compute_consensus_max_mode_network_growth() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 1.into();

        // Create network
        add_network(netuid, 0, 0);
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Max);

        // Setup current consensus with 6 values (network has grown)
        let current_consensus: Vec<I32F32> = vec![
            I32F32::from_num(0.2), // larger than 0.1
            I32F32::from_num(0.15), // smaller than 0.2
            I32F32::from_num(0.25), // smaller than 0.3
            I32F32::from_num(0.5), // larger than 0.4
            I32F32::from_num(0.1), // new position (no previous) - should use current
            I32F32::from_num(0.3), // new position (no previous) - should use current
        ];

        // Store previous consensus values in storage (only 4 values - network was smaller)
        let previous_values: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
        let previous_consensus_u16: Vec<u16> = previous_values
            .iter()
            .map(|&v| (v * u16::MAX as f32) as u16)
            .collect();
        Consensus::<Test>::insert(netuid, previous_consensus_u16);

        // Compute consensus for liquid alpha
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        // Should return element-wise max of current and previous
        // For positions where previous doesn't exist, treat previous as 0 (so use current)
        assert_eq!(result.len(), 6);

        // Expected max values:
        // [0] max(0.2, 0.1) = 0.2
        // [1] max(0.15, 0.2) = 0.2
        // [2] max(0.25, 0.3) = 0.3
        // [3] max(0.5, 0.4) = 0.5
        // [4] max(0.1, 0) = 0.1 (no previous, treat as 0)
        // [5] max(0.3, 0) = 0.3 (no previous, treat as 0)
        let expected: Vec<f32> = vec![0.2, 0.2, 0.3, 0.5, 0.1, 0.3];
        for i in 0..6 {
            let expected_val = I32F32::from_num(expected[i]);
            let diff = if result[i] > expected_val {
                result[i] - expected_val
            } else {
                expected_val - result[i]
            };
            assert!(
                diff < I32F32::from_num(0.001),
                "Values should be approximately equal at index {}. Expected: {}, Got: {}",
                i,
                expected[i],
                result[i]
            );
        }
    });
}

/// Test compute_consensus_for_liquid_alpha with Auto mode
#[test]
fn test_compute_consensus_auto_mode() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 1.into();
        let n: usize = 4;

        // Create network
        add_network(netuid, 0, 0);
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Auto);

        // Setup current consensus values
        let current_consensus: Vec<I32F32> = vec![
            I32F32::from_num(0.2),
            I32F32::from_num(0.3),
            I32F32::from_num(0.4),
            I32F32::from_num(0.1),
        ];

        // Store previous consensus values in storage
        let previous_values: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
        let previous_consensus_u16: Vec<u16> = previous_values
            .iter()
            .map(|&v| (v * u16::MAX as f32) as u16)
            .collect();
        Consensus::<Test>::insert(netuid, previous_consensus_u16);

        // Test 1: bond_penalty != 1, should use Current
        SubtensorModule::set_bonds_penalty(netuid, u16::MAX / 2); // 0.5
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        assert_eq!(result.len(), n);
        for i in 0..n {
            assert_eq!(result[i], current_consensus[i], "Should use current consensus when bond_penalty != 1");
        }

        // Test 2: bond_penalty == 1, should use Previous
        SubtensorModule::set_bonds_penalty(netuid, u16::MAX); // 1.0
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        assert_eq!(result.len(), n);
        for i in 0..n {
            let expected = I32F32::from_num(previous_values[i]);
            let diff = if result[i] > expected {
                result[i] - expected
            } else {
                expected - result[i]
            };
            assert!(diff < I32F32::from_num(0.001), "Should use previous consensus when bond_penalty == 1");
        }
    });
}