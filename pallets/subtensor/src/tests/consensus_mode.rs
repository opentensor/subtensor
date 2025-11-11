use frame_support::{assert_err, assert_ok};
use sp_core::U256;
use substrate_fixed::types::I32F32;
use subtensor_runtime_common::NetUid;

use super::consensus::{fixed, fixed_proportion_to_u16};
use super::mock::*;
use crate::*;

// ============================================================================
// Test Helper Functions
// ============================================================================

/// Sets up a network with full owner permissions (root registration + network creation)
/// Returns (netuid, hotkey, coldkey, signer)
fn setup_network_with_owner() -> (NetUid, U256, U256, RuntimeOrigin) {
    let hotkey = U256::from(1);
    let coldkey = U256::from(457);
    let netuid = add_dynamic_network(&hotkey, &coldkey);
    let signer = RuntimeOrigin::signed(coldkey);

    migrations::migrate_create_root_network::migrate_create_root_network::<Test>();
    SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
    assert_ok!(SubtensorModule::root_register(signer.clone(), hotkey));
    assert_ok!(SubtensorModule::register_network(signer.clone(), hotkey));

    (netuid, hotkey, coldkey, signer)
}

/// Sets up a basic network environment for consensus testing
/// Creates network and enables liquid alpha
fn setup_consensus_test_environment(netuid: NetUid) {
    add_network(netuid, 0, 0);
    SubtensorModule::set_liquid_alpha_enabled(netuid, true);
}

/// Creates test consensus data and stores it in the system
/// Returns (current_consensus, previous_values)
fn create_test_consensus_data(netuid: NetUid) -> (Vec<I32F32>, Vec<f32>) {
    let current_consensus: Vec<I32F32> = vec![
        I32F32::from_num(0.2),
        I32F32::from_num(0.3),
        I32F32::from_num(0.4),
        I32F32::from_num(0.1),
    ];

    let previous_values: Vec<f32> = vec![0.1, 0.2, 0.3, 0.4];
    let previous_consensus_u16: Vec<u16> = previous_values
        .iter()
        .map(|&v| fixed_proportion_to_u16(fixed(v)))
        .collect();
    Consensus::<Test>::insert(netuid, previous_consensus_u16);

    (current_consensus, previous_values)
}

// ============================================================================
// Tests
// ============================================================================

/// Test setting consensus mode when liquid alpha is disabled
#[test]
fn test_set_consensus_mode_liquid_alpha_disabled() {
    new_test_ext(1).execute_with(|| {
        let (netuid, _hotkey, _coldkey, signer) = setup_network_with_owner();

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
        let (netuid, _hotkey, _coldkey, owner_signer) = setup_network_with_owner();
        let non_owner = U256::from(999);
        let non_owner_signer = RuntimeOrigin::signed(non_owner);

        // Enable liquid alpha for this test
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);

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
        let coldkey = U256::from(457);
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
        let (netuid, _hotkey, _coldkey, signer) = setup_network_with_owner();

        // Enable liquid alpha for this test
        SubtensorModule::set_liquid_alpha_enabled(netuid, true);

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

        // Setup network and test data
        setup_consensus_test_environment(netuid);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Current);
        let (current_consensus, _previous_values) = create_test_consensus_data(netuid);

        // Compute consensus for liquid alpha
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        // Should return current consensus (not previous)
        assert_eq!(result.len(), n);
        for (res, curr) in result.iter().zip(current_consensus.iter()) {
            assert_eq!(res, curr);
        }
    });
}

/// Test compute_consensus_for_liquid_alpha with Previous mode
#[test]
fn test_compute_consensus_previous_mode() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 1.into();
        let n: usize = 4;

        // Setup network and test data
        setup_consensus_test_environment(netuid);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Previous);
        let (current_consensus, previous_values) = create_test_consensus_data(netuid);

        // Compute consensus for liquid alpha
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        // Should return previous consensus from storage (not current)
        assert_eq!(result.len(), n);
        for (res, &prev) in result.iter().zip(previous_values.iter()) {
            let expected = I32F32::from_num(prev);
            // Allow small floating point difference
            let diff = if *res > expected {
                *res - expected
            } else {
                expected - *res
            };
            assert!(diff < I32F32::from_num(0.001), "Values should be approximately equal");
        }
    });
}

/// Test compute_consensus_for_liquid_alpha with Auto mode
#[test]
fn test_compute_consensus_auto_mode() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 1.into();
        let n: usize = 4;

        // Setup network and test data
        setup_consensus_test_environment(netuid);
        SubtensorModule::set_liquid_alpha_consensus_mode(netuid, ConsensusMode::Auto);
        let (current_consensus, previous_values) = create_test_consensus_data(netuid);

        // Test 1: bond_penalty != 1, should use Current
        SubtensorModule::set_bonds_penalty(netuid, u16::MAX / 2); // 0.5
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        assert_eq!(result.len(), n);
        for (res, curr) in result.iter().zip(current_consensus.iter()) {
            assert_eq!(res, curr, "Should use current consensus when bond_penalty != 1");
        }

        // Test 2: bond_penalty == 1, should use Previous
        SubtensorModule::set_bonds_penalty(netuid, u16::MAX); // 1.0
        let result = SubtensorModule::compute_consensus_for_liquid_alpha(netuid, &current_consensus);

        assert_eq!(result.len(), n);
        for (res, &prev) in result.iter().zip(previous_values.iter()) {
            let expected = I32F32::from_num(prev);
            let diff = if *res > expected {
                *res - expected
            } else {
                expected - *res
            };
            assert!(diff < I32F32::from_num(0.001), "Should use previous consensus when bond_penalty == 1");
        }
    });
}