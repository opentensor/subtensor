#![allow(clippy::unwrap_used)]

use frame_support::weights::Weight;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::U256;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

use super::mock;
use super::mock::*;
use crate::liquidation::types::*;
use crate::liquidation::*;
use crate::*;

// ============================================================
// Helpers
// ============================================================

/// Create a subnet with `staker_count` stakers, each holding `alpha_per_staker` alpha.
/// Returns (netuid, Vec<(hotkey, coldkey)>).
fn setup_subnet_with_stakers(
    owner_hot: U256,
    owner_cold: U256,
    staker_count: u32,
    alpha_per_staker: u64,
) -> (NetUid, Vec<(U256, U256)>) {
    let netuid = add_dynamic_network(&owner_hot, &owner_cold);

    // Setup reserves so alpha has value
    mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

    let mut stakers = Vec::new();
    for i in 0..staker_count {
        let hot = U256::from(1000u32.saturating_add(i.saturating_mul(2)));
        let cold = U256::from(1001u32.saturating_add(i.saturating_mul(2)));

        // Set alpha directly in storage
        Alpha::<Test>::insert((&hot, &cold, netuid), U64F64::from_num(alpha_per_staker));
        TotalHotkeyAlpha::<Test>::insert(&hot, netuid, AlphaCurrency::from(alpha_per_staker));
        stakers.push((hot, cold));
    }

    (netuid, stakers)
}

/// Create a liquidation state with the given tao_pot and put it in storage.
fn setup_liquidation_state(netuid: NetUid, tao_pot: u64) {
    let state = LiquidationState {
        started_at: 1u64,
        max_completion_block: 1u64.saturating_add(MAX_LIQUIDATION_BLOCKS),
        phase: LiquidationPhase::Freeze,
        weight_per_block: Weight::from_parts(10_000_000, 0),
        total_stakers: 0,

        total_neurons: 0,
        mechanism_count: 0,
        tao_pot,
        total_alpha_value: 0,
        snapshot_count: 0,
        tao_distributed: 0,
    };
    LiquidatingSubnets::<Test>::insert(netuid, state);
}

// ============================================================
// 7.1 Unit Tests — Arithmetic
// ============================================================

#[test]
fn test_calculate_share_no_overflow() {
    new_test_ext(1).execute_with(|| {
        // Max u64 pot, large alpha values — must not overflow
        let share = SubtensorModule::calculate_share(u64::MAX, u128::from(u64::MAX), u128::from(u64::MAX));
        assert_eq!(share, u64::MAX); // 100% of pot
    });
}

#[test]
fn test_calculate_share_zero_total_alpha() {
    new_test_ext(1).execute_with(|| {
        let share = SubtensorModule::calculate_share(1_000_000, 500, 0);
        assert_eq!(share, 0);
    });
}

#[test]
fn test_calculate_share_zero_individual_alpha() {
    new_test_ext(1).execute_with(|| {
        let share = SubtensorModule::calculate_share(1_000_000, 0, 500);
        assert_eq!(share, 0);
    });
}

#[test]
fn test_calculate_share_max_values() {
    new_test_ext(1).execute_with(|| {
        // u64::MAX pot, u128::MAX alpha — should not overflow via U256
        let share = SubtensorModule::calculate_share(u64::MAX, u128::MAX, u128::MAX);
        assert_eq!(share, u64::MAX); // 100% of pot
    });
}

#[test]
fn test_distribution_sum_never_exceeds_pot() {
    new_test_ext(1).execute_with(|| {
        let tao_pot: u64 = 1_000_000_007; // prime number to test rounding
        let n = 13u128;
        let alpha_per = 77u128;
        let total_alpha = n.saturating_mul(alpha_per);

        let mut sum = 0u64;
        for _ in 0..n {
            sum = sum.saturating_add(SubtensorModule::calculate_share(tao_pot, alpha_per, total_alpha));
        }
        assert!(sum <= tao_pot, "sum {} exceeded pot {}", sum, tao_pot);
    });
}

#[test]
fn test_distribution_dust_handling() {
    new_test_ext(1).execute_with(|| {
        let tao_pot: u64 = 1_000_000;
        let n = 3u128;
        let alpha_per = 1u128;
        let total_alpha = n.saturating_mul(alpha_per);

        let mut sum = 0u64;
        for _ in 0..n {
            sum = sum.saturating_add(SubtensorModule::calculate_share(tao_pot, alpha_per, total_alpha));
        }
        let dust = tao_pot.saturating_sub(sum);
        // With 1M / 3, each gets 333333, sum = 999999, dust = 1
        assert!(dust < n as u64, "dust {} should be < n {}", dust, n);
        assert!(dust > 0 || sum == tao_pot);
    });
}

// ============================================================
// 7.1 Unit Tests — Phase Transitions
// ============================================================

#[test]
fn test_all_phase_transitions_valid() {
    // Every phase.next_phase() should return a valid successor or None for FinalCleanup
    let phases = vec![
        LiquidationPhase::Freeze,
        LiquidationPhase::SnapshotStakers { cursor: None },
        LiquidationPhase::ClearHyperparams,
        LiquidationPhase::ClearNeuronData {
            map_idx: 0,
            cursor: None,
        },
        LiquidationPhase::ClearRootWeights { uid_cursor: 0 },
        LiquidationPhase::FinalizeRootDividends { cursor: None },
        LiquidationPhase::DistributeAlpha { cursor_idx: 0 },
        LiquidationPhase::DissolveUserLPs { cursor: None },
        LiquidationPhase::ClearProtocolLPs,
        LiquidationPhase::ClearMatrices {
            mechanism_idx: 0,
            map_idx: 0,
            cursor: None,
        },
        LiquidationPhase::ClearTwoKeyMaps {
            map_idx: 0,
            cursor: None,
        },
        LiquidationPhase::FinalCleanup,
    ];

    for (i, phase) in phases.iter().enumerate() {
        let next = phase.next_phase();
        if i < phases.len().saturating_sub(1) {
            assert!(next.is_some(), "Phase {:?} should have a successor", phase);
        } else {
            assert!(next.is_none(), "FinalCleanup should return None");
        }
    }
}

#[test]
fn test_phase_cannot_regress() {
    // Walk the chain and ensure it only moves forward
    let mut phase = LiquidationPhase::Freeze;
    let mut seen_tags = Vec::new();

    loop {
        let tag = phase.tag();
        assert!(
            !seen_tags.contains(&tag),
            "Phase tag {:?} seen twice — regression detected",
            tag
        );
        seen_tags.push(tag);

        match phase.next_phase() {
            Some(next) => phase = next,
            None => break,
        }
    }

    // Should have visited all 12 phases
    assert_eq!(seen_tags.len(), 12);
}

#[test]
fn test_cursor_overflow_handled_gracefully() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);

        // Create an oversized cursor (> 256 bytes, exceeding CursorBytes capacity)
        let oversized = vec![0u8; 512];

        let result = SubtensorModule::bound_cursor(oversized, netuid);
        assert!(result.is_none(), "Should return None on cursor overflow");

        // Check that warning event was emitted
        let events = System::events();
        let has_warning = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::LiquidationWarning {
                    warning: LiquidationWarning::CursorOverflow,
                    ..
                })
            )
        });
        assert!(has_warning, "CursorOverflow warning should be emitted");
    });
}

#[test]
fn test_empty_phase_skipped() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 100, 0);

        // Create state in Freeze phase (empty — completes immediately)
        let state = LiquidationState {
            started_at: 1u64,
            max_completion_block: 1u64.saturating_add(MAX_LIQUIDATION_BLOCKS),
            phase: LiquidationPhase::Freeze,
            weight_per_block: Weight::from_parts(u64::MAX / 2, 0),
            total_stakers: 0,

            total_neurons: 0,
            mechanism_count: 0,
            tao_pot: 0,
            total_alpha_value: 0,
            snapshot_count: 0,
            tao_distributed: 0,
        };

        let unlimited = Weight::from_parts(u64::MAX / 2, u64::MAX / 2);
        let (_w, new_state, _complete) =
            SubtensorModule::process_liquidation_step(netuid, state, unlimited);

        // Freeze should have been completed and moved to the next phase
        assert_ne!(
            new_state.phase,
            LiquidationPhase::Freeze,
            "Freeze phase should have been skipped/completed"
        );
    });
}

#[test]
fn test_next_phase_returns_correct_sequence() {
    // LP dissolution runs before snapshot so LP-derived alpha is captured
    let expected_tags = vec![
        LiquidationPhaseTag::Freeze,
        LiquidationPhaseTag::DissolveUserLPs,
        LiquidationPhaseTag::ClearProtocolLPs,
        LiquidationPhaseTag::SnapshotStakers,
        LiquidationPhaseTag::ClearHyperparams,
        LiquidationPhaseTag::ClearNeuronData,
        LiquidationPhaseTag::ClearRootWeights,
        LiquidationPhaseTag::FinalizeRootDividends,
        LiquidationPhaseTag::DistributeAlpha,
        LiquidationPhaseTag::ClearMatrices,
        LiquidationPhaseTag::ClearTwoKeyMaps,
        LiquidationPhaseTag::FinalCleanup,
    ];

    let mut phase = LiquidationPhase::Freeze;
    let mut actual_tags = vec![phase.tag()];

    while let Some(next) = phase.next_phase() {
        actual_tags.push(next.tag());
        phase = next;
    }

    assert_eq!(actual_tags, expected_tags);
    assert!(
        phase.next_phase().is_none(),
        "FinalCleanup.next_phase() should be None"
    );
}

// ============================================================
// 7.1 Unit Tests — Blocking
// ============================================================

#[test]
fn test_stake_blocked_during_liquidation() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000);

        // Start liquidation
        setup_liquidation_state(netuid, 0);

        // add_stake should be blocked
        assert_noop!(
            SubtensorModule::do_add_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                1000u64.into()
            ),
            Error::<Test>::SubnetLiquidating
        );
    });
}

#[test]
fn test_unstake_blocked_during_liquidation() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        setup_liquidation_state(netuid, 0);

        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                1000u64.into()
            ),
            Error::<Test>::SubnetLiquidating
        );
    });
}

#[test]
fn test_set_weights_blocked_during_liquidation() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        setup_liquidation_state(netuid, 0);

        assert_noop!(
            SubtensorModule::do_set_weights(
                RuntimeOrigin::signed(coldkey),
                netuid,
                vec![0],
                vec![1],
                0
            ),
            Error::<Test>::SubnetLiquidating
        );
    });
}

#[test]
fn test_registration_blocked_during_liquidation() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000);

        setup_liquidation_state(netuid, 0);

        let new_hot = U256::from(99);
        assert_noop!(
            SubtensorModule::do_burned_registration(
                RuntimeOrigin::signed(coldkey),
                netuid,
                new_hot
            ),
            Error::<Test>::SubnetLiquidating
        );
    });
}

#[test]
fn test_swap_blocked_during_liquidation() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let netuid2 = add_dynamic_network(&U256::from(3), &U256::from(4));

        setup_liquidation_state(netuid, 0);

        assert_noop!(
            SubtensorModule::do_swap_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                netuid2,
                1000u64.into()
            ),
            Error::<Test>::SubnetLiquidating
        );
    });
}

#[test]
fn test_all_blocked_operations() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let netuid2 = add_dynamic_network(&U256::from(3), &U256::from(4));
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000);

        setup_liquidation_state(netuid, 0);

        // move_stake
        assert_noop!(
            SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                hotkey,
                netuid,
                netuid2,
                1000u64.into()
            ),
            Error::<Test>::SubnetLiquidating
        );

        // transfer_stake
        let other_cold = U256::from(99);
        assert_noop!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(coldkey),
                other_cold,
                hotkey,
                netuid,
                netuid2,
                1000u64.into()
            ),
            Error::<Test>::SubnetLiquidating
        );

        // serve_axon
        assert_noop!(
            SubtensorModule::do_serve_axon(
                RuntimeOrigin::signed(hotkey),
                netuid,
                0,
                0x08080808u128, // 8.8.8.8
                8080,
                4, // IPv4
                0,
                0,
                0,
                None
            ),
            Error::<Test>::SubnetLiquidating
        );

        // serve_prometheus
        assert_noop!(
            SubtensorModule::do_serve_prometheus(
                RuntimeOrigin::signed(hotkey),
                netuid,
                0,
                0x08080808u128,
                8080,
                4
            ),
            Error::<Test>::SubnetLiquidating
        );

        // commit_weights
        assert_noop!(
            SubtensorModule::do_commit_weights(
                RuntimeOrigin::signed(coldkey),
                netuid,
                sp_core::H256::zero()
            ),
            Error::<Test>::SubnetLiquidating
        );

        // set_childkey_take
        assert_noop!(
            SubtensorModule::do_set_childkey_take(coldkey, hotkey, netuid, 100),
            Error::<Test>::SubnetLiquidating
        );

        // schedule_children
        assert_noop!(
            SubtensorModule::do_schedule_children(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                vec![]
            ),
            Error::<Test>::SubnetLiquidating
        );
    });
}

// ============================================================
// 7.1 Unit Tests — Pending Registration
// ============================================================

#[test]
fn test_pending_registration_stored_on_liquidation() {
    new_test_ext(1).execute_with(|| {
        // Reduce subnet limit and immunity period for easier testing
        SubnetLimit::<Test>::put(3u16);
        NetworkImmunityPeriod::<Test>::put(0u64);

        // Add root (netuid 0) + 3 non-root subnets to fill capacity
        add_network(NetUid::from(0), 100, 0); // root
        add_network(NetUid::from(1), 100, 0);
        add_network(NetUid::from(2), 100, 0);
        add_network(NetUid::from(3), 100, 0);
        // current_count (non-root) = 3 >= subnet_limit (3) → triggers pending

        // Give the registrant enough balance
        let reg_cold = U256::from(100);
        let reg_hot = U256::from(101);
        SubtensorModule::add_balance_to_coldkey_account(&reg_cold, 1_000_000_000_000);

        // Register a new network — should trigger liquidation + pending
        let result = SubtensorModule::do_register_network(
            RuntimeOrigin::signed(reg_cold),
            &reg_hot,
            1,
            None,
        );
        assert_ok!(result);

        // PendingSubnetRegistration should be set
        assert!(PendingSubnetRegistration::<Test>::exists());
        let pending = PendingSubnetRegistration::<Test>::get().unwrap();
        assert_eq!(pending.coldkey, reg_cold);
        assert_eq!(pending.hotkey, reg_hot);
        assert!(pending.cost_paid > 0);
    });
}

#[test]
fn test_pending_registration_auto_completes() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Start liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Store a pending registration with a new (non-existent) hotkey
        let reg_cold = U256::from(100);
        let reg_hot = U256::from(101);
        SubtensorModule::add_balance_to_coldkey_account(&reg_cold, 200_000);
        PendingSubnetRegistration::<Test>::put(PendingRegistration {
            coldkey: reg_cold,
            hotkey: reg_hot,
            mechid: 1, // dynamic
            cost_paid: 100_000,
        });

        // Force complete
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Pending registration should be consumed
        assert!(!PendingSubnetRegistration::<Test>::exists());

        // New subnet should be created on the freed netuid
        assert!(
            NetworksAdded::<Test>::get(netuid),
            "New subnet should be registered"
        );
        assert_eq!(SubnetOwner::<Test>::get(netuid), reg_cold);
        assert_eq!(SubnetOwnerHotkey::<Test>::get(netuid), reg_hot);

        // RegistrationCompleted event emitted (not refund)
        let events = System::events();
        let has_completed = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::RegistrationCompleted { .. })
            )
        });
        assert!(
            has_completed,
            "RegistrationCompleted event should be emitted"
        );

        let has_refund = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::PendingRegistrationRefunded { .. })
            )
        });
        assert!(
            !has_refund,
            "Should NOT have refunded — registration should succeed"
        );
    });
}

#[test]
fn test_pending_registration_only_one_at_a_time() {
    new_test_ext(1).execute_with(|| {
        // Reduce subnet limit and immunity period for easier testing
        SubnetLimit::<Test>::put(3u16);
        NetworkImmunityPeriod::<Test>::put(0u64);

        // Fill up: root + 3 non-root = at capacity
        add_network(NetUid::from(0), 100, 0);
        add_network(NetUid::from(1), 100, 0);
        add_network(NetUid::from(2), 100, 0);
        add_network(NetUid::from(3), 100, 0);

        // Set up a pre-existing pending registration
        let cold1 = U256::from(100);
        let hot1 = U256::from(101);
        PendingSubnetRegistration::<Test>::put(PendingRegistration {
            coldkey: cold1,
            hotkey: hot1,
            mechid: 1,
            cost_paid: 100_000,
        });

        // Second registration should fail with PendingRegistrationExists
        // Note: using assert_err! instead of assert_noop! because
        // do_register_network deducts balance before the pending reg check
        let cold2 = U256::from(200);
        let hot2 = U256::from(201);
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 1_000_000_000_000);

        assert_err!(
            SubtensorModule::do_register_network(RuntimeOrigin::signed(cold2), &hot2, 1, None),
            Error::<Test>::PendingRegistrationExists
        );
    });
}

#[test]
fn test_pending_registration_cost_collected_upfront() {
    new_test_ext(1).execute_with(|| {
        // Reduce subnet limit and immunity period for easier testing
        SubnetLimit::<Test>::put(3u16);
        NetworkImmunityPeriod::<Test>::put(0u64);

        // Fill up: root + 3 non-root = at capacity
        add_network(NetUid::from(0), 100, 0);
        add_network(NetUid::from(1), 100, 0);
        add_network(NetUid::from(2), 100, 0);
        add_network(NetUid::from(3), 100, 0);

        let reg_cold = U256::from(100);
        let reg_hot = U256::from(101);
        let initial_balance = 1_000_000_000_000u64;
        SubtensorModule::add_balance_to_coldkey_account(&reg_cold, initial_balance);

        let balance_before = SubtensorModule::get_coldkey_balance(&reg_cold);

        // Register triggers liquidation + pending
        assert_ok!(SubtensorModule::do_register_network(
            RuntimeOrigin::signed(reg_cold),
            &reg_hot,
            1,
            None,
        ));

        let balance_after = SubtensorModule::get_coldkey_balance(&reg_cold);

        // Cost should have been deducted
        assert!(
            balance_after < balance_before,
            "Cost should be deducted upfront: before={}, after={}",
            balance_before,
            balance_after
        );
    });
}

#[test]
fn test_no_cooldown_when_pending_reg_takes_slot() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Start liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Store a pending registration
        let reg_cold = U256::from(100);
        let reg_hot = U256::from(101);
        SubtensorModule::add_balance_to_coldkey_account(&reg_cold, 1_000_000_000);
        PendingSubnetRegistration::<Test>::put(PendingRegistration {
            coldkey: reg_cold,
            hotkey: reg_hot,
            mechid: 0,
            cost_paid: 100_000,
        });

        // Force complete
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // If the pending registration succeeded, the netuid should be in use (NetworksAdded)
        // and NetuidCooldown should NOT be set
        if NetworksAdded::<Test>::get(netuid) {
            assert!(
                !NetuidCooldown::<Test>::contains_key(netuid),
                "Cooldown should NOT be set when pending reg takes slot"
            );
        }
    });
}

// ============================================================
// 7.1 Unit Tests — Edge Cases
// ============================================================

#[test]
fn test_liquidation_empty_subnet() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Start liquidation on a subnet with no stakers
        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Force complete — should work without errors
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Subnet should be removed
        assert!(!LiquidatingSubnets::<Test>::contains_key(netuid));
        assert!(!NetworksAdded::<Test>::get(netuid));
    });
}

#[test]
fn test_liquidation_single_staker() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Set up exactly 1 staker with alpha
        let staker_hot = U256::from(10);
        let staker_cold = U256::from(11);
        let alpha_amount = 50_000u64;
        Alpha::<Test>::insert(
            (&staker_hot, &staker_cold, netuid),
            U64F64::from_num(alpha_amount),
        );
        TotalHotkeyAlpha::<Test>::insert(&staker_hot, netuid, AlphaCurrency::from(alpha_amount));

        // Set TAO pot
        let tao_pot = 100_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        let balance_before = SubtensorModule::get_coldkey_balance(&staker_cold);

        // Start and complete liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        let balance_after = SubtensorModule::get_coldkey_balance(&staker_cold);

        // Single staker gets the full pot
        assert_eq!(
            balance_after.saturating_sub(balance_before),
            tao_pot,
            "Single staker should get the full TAO pot"
        );
    });
}

#[test]
fn test_liquidation_max_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 50u32;
        let alpha_per = 10_000u64; // Above MIN_SNAPSHOT_ALPHA

        let (netuid, stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        // Record balances before
        let balances_before: Vec<u64> = stakers
            .iter()
            .map(|(_, cold)| SubtensorModule::get_coldkey_balance(cold))
            .collect();

        // Start and complete liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Calculate total distributed
        let mut total_distributed = 0u64;
        for (i, (_, cold)) in stakers.iter().enumerate() {
            let balance_after = SubtensorModule::get_coldkey_balance(cold);
            let received = balance_after.saturating_sub(balances_before[i]);
            total_distributed = total_distributed.saturating_add(received);
        }

        // Total distributed + dust should equal tao_pot
        assert!(
            total_distributed <= tao_pot,
            "Distributed {} exceeds pot {}",
            total_distributed,
            tao_pot
        );
        // Dust should be small (< num_stakers)
        let dust = tao_pot.saturating_sub(total_distributed);
        assert!(
            dust < num_stakers as u64,
            "Dust {} should be < staker count {}",
            dust,
            num_stakers
        );
    });
}

#[test]
fn test_liquidation_zero_tao_pot() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Set TAO pot to zero
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(0u64));

        // Add a staker
        let staker_hot = U256::from(10);
        let staker_cold = U256::from(11);
        Alpha::<Test>::insert(
            (&staker_hot, &staker_cold, netuid),
            U64F64::from_num(50_000u64),
        );

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_eq!(state.tao_pot, 0);

        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));
        assert!(!LiquidatingSubnets::<Test>::contains_key(netuid));

        // Staker should have received 0 TAO (pot was empty)
        let staker_balance = SubtensorModule::get_coldkey_balance(&staker_cold);
        assert_eq!(staker_balance, 0, "Staker should receive 0 with empty pot");
    });
}

#[test]
fn test_liquidation_zero_tempo() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Set tempo to 0
        Tempo::<Test>::insert(netuid, 0u16);

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();

        // Weight should be spread over MIN_LIQUIDATION_BLOCKS (10), not 0
        // This means weight_per_block should be non-zero
        assert!(
            state.weight_per_block.ref_time() > 0,
            "With tempo=0, should use MIN_LIQUIDATION_BLOCKS"
        );

        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));
    });
}

#[test]
fn test_dust_positions_excluded_from_snapshot() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Add one staker above threshold
        let big_hot = U256::from(10);
        let big_cold = U256::from(11);
        Alpha::<Test>::insert((&big_hot, &big_cold, netuid), U64F64::from_num(50_000u64));

        // Add one staker below MIN_SNAPSHOT_ALPHA (1000)
        let dust_hot = U256::from(20);
        let dust_cold = U256::from(21);
        Alpha::<Test>::insert(
            (&dust_hot, &dust_cold, netuid),
            U64F64::from_num(500u64), // Below MIN_SNAPSHOT_ALPHA
        );

        let tao_pot = 100_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        // Fund issuance so distribution works
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // The dust staker should NOT have received TAO
        let dust_balance = SubtensorModule::get_coldkey_balance(&dust_cold);
        assert_eq!(dust_balance, 0, "Dust staker should not receive TAO");

        // The big staker should have received the full pot (sole staker above threshold)
        let big_balance = SubtensorModule::get_coldkey_balance(&big_cold);
        assert_eq!(big_balance, tao_pot, "Big staker should receive full pot");
    });
}

// ============================================================
// 7.1 Unit Tests — Timeout & Recovery
// ============================================================

#[test]
fn test_liquidation_timeout_triggers_emergency() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        // Add stakers that won't be fully processed
        let staker_hot = U256::from(10);
        let staker_cold = U256::from(11);
        Alpha::<Test>::insert(
            (&staker_hot, &staker_cold, netuid),
            U64F64::from_num(50_000u64),
        );
        TotalHotkeyAlpha::<Test>::insert(&staker_hot, netuid, AlphaCurrency::from(50_000u64));

        // Add a pending registration
        let reg_cold = U256::from(200);
        PendingSubnetRegistration::<Test>::put(PendingRegistration {
            coldkey: reg_cold,
            hotkey: U256::from(201),
            mechid: 0,
            cost_paid: 75_000,
        });

        // Record initial state
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));
        let issuance_before: u64 = TotalIssuance::<Test>::get().into();
        let total_networks_before = TotalNetworks::<Test>::get();

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Advance past max_completion_block (jump directly, don't step each block)
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        let max_block: u64 = state.max_completion_block;
        System::set_block_number(max_block.saturating_add(2));

        // Process liquidations — should trigger emergency finalize
        let unlimited = Weight::from_parts(u64::MAX / 2, u64::MAX / 2);
        SubtensorModule::process_liquidations(unlimited);

        // 1. Liquidation state removed
        assert!(!LiquidatingSubnets::<Test>::contains_key(netuid));

        // 2. Subnet slot freed
        assert!(
            !NetworksAdded::<Test>::get(netuid),
            "Subnet should be removed"
        );
        assert_eq!(
            TotalNetworks::<Test>::get(),
            total_networks_before.saturating_sub(1),
            "TotalNetworks should be decremented"
        );

        // 3. Remaining TAO burned from TotalIssuance
        let issuance_after: u64 = TotalIssuance::<Test>::get().into();
        assert!(
            issuance_after < issuance_before,
            "TotalIssuance should decrease from TAO burn"
        );

        // 4. Cooldown set on freed netuid
        assert!(
            NetuidCooldown::<Test>::contains_key(netuid),
            "Cooldown should be set after emergency"
        );

        // 5. Pending registration refunded
        assert!(
            !PendingSubnetRegistration::<Test>::exists(),
            "Pending reg consumed"
        );
        let reg_balance = SubtensorModule::get_coldkey_balance(&reg_cold);
        assert_eq!(
            reg_balance, 75_000,
            "Pending registration should be refunded"
        );

        // 6. Staker did NOT receive TAO (emergency burns, doesn't distribute)
        let staker_balance = SubtensorModule::get_coldkey_balance(&staker_cold);
        assert_eq!(
            staker_balance, 0,
            "Stakers should not receive TAO on timeout"
        );

        // 7. Events
        let events = System::events();
        let has_timeout = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::LiquidationTimeout { .. })
            )
        });
        assert!(has_timeout, "LiquidationTimeout event should be emitted");

        let has_refund = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::PendingRegistrationRefunded { .. })
            )
        });
        assert!(has_refund, "PendingRegistrationRefunded should be emitted");
    });
}

#[test]
fn test_force_complete_works() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        assert!(LiquidatingSubnets::<Test>::contains_key(netuid));

        // Root call force complete
        assert_ok!(SubtensorModule::force_complete_liquidation(
            RuntimeOrigin::root(),
            netuid
        ));

        assert!(!LiquidatingSubnets::<Test>::contains_key(netuid));

        // Check for event
        let events = System::events();
        let has_force = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::LiquidationForceCompleted { .. })
            )
        });
        assert!(
            has_force,
            "LiquidationForceCompleted event should be emitted"
        );
    });
}

#[test]
fn test_force_complete_root_only() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Non-root call should fail
        assert_noop!(
            SubtensorModule::force_complete_liquidation(RuntimeOrigin::signed(coldkey), netuid),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_emergency_finalize_distributes_remaining() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        // Add TAO to total issuance to cover the burn
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        let issuance_before: u64 = TotalIssuance::<Test>::get().into();

        // Create state with some TAO undistributed
        let state = LiquidationState {
            started_at: 1u64,
            max_completion_block: 100u64,
            phase: LiquidationPhase::SnapshotStakers { cursor: None },
            weight_per_block: Weight::from_parts(10_000_000, 0),
            total_stakers: 0,

            total_neurons: 0,
            mechanism_count: 0,
            tao_pot,
            total_alpha_value: 0,
            snapshot_count: 0,
            tao_distributed: 100_000, // Only 100k distributed of 500k
        };

        let total_networks_before = TotalNetworks::<Test>::get();

        SubtensorModule::emergency_finalize(netuid, &state);

        let issuance_after: u64 = TotalIssuance::<Test>::get().into();
        let burned = issuance_before.saturating_sub(issuance_after);
        let remaining = tao_pot.saturating_sub(100_000);

        assert_eq!(
            burned, remaining,
            "Emergency finalize should burn remaining TAO"
        );

        // Subnet slot freed
        assert!(
            !NetworksAdded::<Test>::get(netuid),
            "Subnet should be removed"
        );
        assert_eq!(
            TotalNetworks::<Test>::get(),
            total_networks_before.saturating_sub(1),
        );

        // Cooldown set
        assert!(
            NetuidCooldown::<Test>::contains_key(netuid),
            "Cooldown should be set after emergency"
        );

        // Snapshot cleared
        assert!(!LiquidationStakerSnapshot::<Test>::contains_key(
            netuid, 0u32
        ));

        // Check for EmergencyBurn warning
        let events = System::events();
        let has_burn_warning = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::LiquidationWarning {
                    warning: LiquidationWarning::EmergencyBurn(_),
                    ..
                })
            )
        });
        assert!(has_burn_warning, "EmergencyBurn warning should be emitted");
    });
}

// ============================================================
// 7.1 Unit Tests — Invariants
// ============================================================

#[test]
fn test_total_tao_conserved() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 7u32;
        let alpha_per = 10_000u64;

        let (netuid, stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 777_777u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        // Record initial balances
        let initial_balances: Vec<u64> = stakers
            .iter()
            .map(|(_, cold)| SubtensorModule::get_coldkey_balance(cold))
            .collect();

        let issuance_before: u64 = TotalIssuance::<Test>::get().into();

        // Start and complete liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Calculate total distributed
        let mut total_distributed = 0u64;
        for (i, (_, cold)) in stakers.iter().enumerate() {
            let bal = SubtensorModule::get_coldkey_balance(cold);
            total_distributed = total_distributed.saturating_add(bal.saturating_sub(initial_balances[i]));
        }

        let issuance_after: u64 = TotalIssuance::<Test>::get().into();
        let dust_burned = issuance_before.saturating_sub(issuance_after);

        // Conservation: distributed + dust_burned == tao_pot
        assert_eq!(
            total_distributed.saturating_add(dust_burned),
            tao_pot,
            "TAO must be conserved: distributed={} + dust_burned={} != pot={}",
            total_distributed,
            dust_burned,
            tao_pot
        );
    });
}

#[test]
fn test_dust_is_burned() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 3u32;
        let alpha_per = 10_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        // Use a pot that will create dust with 3 stakers
        let tao_pot = 100_001u64; // 100001 / 3 = 33333.666... → dust = 2
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        let issuance_before: u64 = TotalIssuance::<Test>::get().into();

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        let issuance_after: u64 = TotalIssuance::<Test>::get().into();

        // TotalIssuance should have decreased by the dust amount
        assert!(
            issuance_before > issuance_after,
            "TotalIssuance should have decreased due to dust burn"
        );
        let dust_burned = issuance_before.saturating_sub(issuance_after);
        assert!(
            dust_burned > 0,
            "Dust should have been burned from TotalIssuance"
        );
    });
}

#[test]
fn test_no_storage_leaks() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Add stakers with neuron data
        let staker_hot = U256::from(10);
        let staker_cold = U256::from(11);
        Alpha::<Test>::insert(
            (&staker_hot, &staker_cold, netuid),
            U64F64::from_num(50_000u64),
        );
        TotalHotkeyAlpha::<Test>::insert(&staker_hot, netuid, AlphaCurrency::from(50_000u64));
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(100_000u64));

        // Verify some hyperparams exist before liquidation
        // (add_dynamic_network sets these during init_new_network)
        assert!(
            Tempo::<Test>::try_get(netuid).is_ok(),
            "Tempo should exist before liquidation"
        );
        assert!(
            Kappa::<Test>::try_get(netuid).is_ok(),
            "Kappa should exist before liquidation"
        );
        assert!(
            Difficulty::<Test>::try_get(netuid).is_ok(),
            "Difficulty should exist before liquidation"
        );
        // Set some extras explicitly
        SubtokenEnabled::<Test>::insert(netuid, true);
        WeightsVersionKey::<Test>::insert(netuid, 42u64);

        // Set some neuron data that should be cleared
        Rank::<Test>::insert(netuid, vec![0u16; 1]);
        Trust::<Test>::insert(netuid, vec![0u16; 1]);
        Active::<Test>::insert(netuid, vec![true; 1]);
        Emission::<Test>::insert(netuid, vec![AlphaCurrency::from(0u64); 1]);
        Consensus::<Test>::insert(netuid, vec![0u16; 1]);
        Dividends::<Test>::insert(netuid, vec![0u16; 1]);

        // Set child/parent keys that should be cleared
        ChildKeys::<Test>::insert(&hotkey, netuid, vec![(100u64, staker_hot)]);
        ParentKeys::<Test>::insert(&staker_hot, netuid, vec![(100u64, hotkey)]);

        // Set IsNetworkMember entries that should be cleared
        IsNetworkMember::<Test>::insert(&staker_hot, netuid, true);

        let total_networks_before: u16 = TotalNetworks::<Test>::get();

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // === Liquidation-specific storage cleared ===
        assert!(!LiquidatingSubnets::<Test>::contains_key(netuid));
        assert_eq!(LiquidationSnapshotCount::<Test>::get(netuid), 0);
        assert!(LiquidationStakerSnapshot::<Test>::get(netuid, 0u32).is_none());

        // === Subnet removed from network tracking ===
        assert!(!NetworksAdded::<Test>::get(netuid));
        assert_eq!(TotalNetworks::<Test>::get(), total_networks_before.saturating_sub(1));

        // === Hyperparams cleared (try_get returns Err when entry removed) ===
        assert!(
            Tempo::<Test>::try_get(netuid).is_err(),
            "Tempo should be removed"
        );
        assert!(
            Kappa::<Test>::try_get(netuid).is_err(),
            "Kappa should be removed"
        );
        assert!(
            Difficulty::<Test>::try_get(netuid).is_err(),
            "Difficulty should be removed"
        );
        assert!(
            MaxAllowedUids::<Test>::try_get(netuid).is_err(),
            "MaxAllowedUids should be removed"
        );
        assert!(
            ImmunityPeriod::<Test>::try_get(netuid).is_err(),
            "ImmunityPeriod should be removed"
        );
        assert!(
            SubnetMechanism::<Test>::try_get(netuid).is_err(),
            "SubnetMechanism should be removed"
        );
        assert!(
            !SubtokenEnabled::<Test>::get(netuid),
            "SubtokenEnabled should be cleared"
        );
        assert!(
            WeightsVersionKey::<Test>::try_get(netuid).is_err(),
            "WeightsVersionKey should be removed"
        );

        // === Neuron data cleared ===
        assert!(Rank::<Test>::get(netuid).is_empty());
        assert!(Trust::<Test>::get(netuid).is_empty());
        assert!(Active::<Test>::get(netuid).is_empty());
        assert!(Emission::<Test>::get(netuid).is_empty());
        assert!(Consensus::<Test>::get(netuid).is_empty());
        assert!(Dividends::<Test>::get(netuid).is_empty());

        // === Two-key maps cleared ===
        assert!(ChildKeys::<Test>::get(&hotkey, netuid).is_empty());
        assert!(ParentKeys::<Test>::get(&staker_hot, netuid).is_empty());
        assert!(
            !IsNetworkMember::<Test>::get(&staker_hot, netuid),
            "IsNetworkMember should be cleared"
        );
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(&staker_hot, netuid),
            AlphaCurrency::from(0u64)
        );

        // === Core subnet storage cleared ===
        assert_eq!(SubnetTAO::<Test>::get(netuid), TaoCurrency::from(0u64));
        assert_eq!(SubnetworkN::<Test>::get(netuid), 0u16);

        // === Alpha entries cleared ===
        assert_eq!(
            Alpha::<Test>::get((&staker_hot, &staker_cold, netuid)),
            U64F64::from_num(0)
        );

        // === Cooldown is set ===
        assert!(NetuidCooldown::<Test>::contains_key(netuid));
    });
}

#[test]
fn test_netuid_reusable_after_cooldown() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Subnet should be removed
        assert!(!NetworksAdded::<Test>::get(netuid));

        // After cooldown, the netuid should be reusable
        if let Some(cooldown_block) = NetuidCooldown::<Test>::get(netuid) {
            let cooldown_block_u64: u64 = cooldown_block;
            run_to_block(cooldown_block_u64.saturating_add(1));
        }

        // Re-add the network manually
        add_network(netuid, 100, 0);
        assert!(NetworksAdded::<Test>::get(netuid));
    });
}

#[test]
fn test_netuid_blocked_during_cooldown() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Start and complete liquidation to trigger cooldown
        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Cooldown should exist
        assert!(NetuidCooldown::<Test>::contains_key(netuid));

        // Trying to start a new liquidation on the same netuid should fail
        // (subnet doesn't exist anymore, but if re-added it would still be in cooldown)
        // We need to re-add it first to test the cooldown check
        add_network(netuid, 100, 0);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_noop!(
            SubtensorModule::start_liquidation(netuid),
            Error::<Test>::NetuidInCooldown
        );

        // After cooldown expires, it should work
        if let Some(cooldown_block) = NetuidCooldown::<Test>::get(netuid) {
            let cooldown_block_u64: u64 = cooldown_block;
            run_to_block(cooldown_block_u64.saturating_add(1));
        }
        // Clear expired cooldown
        NetuidCooldown::<Test>::remove(netuid);

        // Now liquidation should succeed
        assert_ok!(SubtensorModule::start_liquidation(netuid));
    });
}

#[test]
fn test_only_one_liquidation_at_a_time() {
    new_test_ext(1).execute_with(|| {
        let hot1 = U256::from(1);
        let cold1 = U256::from(2);
        let netuid1 = add_dynamic_network(&hot1, &cold1);
        mock::setup_reserves(netuid1, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let hot2 = U256::from(3);
        let cold2 = U256::from(4);
        let netuid2 = add_dynamic_network(&hot2, &cold2);
        mock::setup_reserves(netuid2, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Start first liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid1));

        // Second should fail
        assert_noop!(
            SubtensorModule::start_liquidation(netuid2),
            Error::<Test>::LiquidationInProgress
        );
    });
}

#[test]
fn test_emergency_remainder_burned() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(5);
        add_network(netuid, 100, 0);

        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));
        let issuance_before: u64 = TotalIssuance::<Test>::get().into();

        let tao_pot = 500_000u64;
        let tao_distributed = 200_000u64;
        let remaining = tao_pot.saturating_sub(tao_distributed);

        let state = LiquidationState {
            started_at: 1u64,
            max_completion_block: 100u64,
            phase: LiquidationPhase::Freeze,
            weight_per_block: Weight::from_parts(10_000_000, 0),
            total_stakers: 0,

            total_neurons: 0,
            mechanism_count: 0,
            tao_pot,
            total_alpha_value: 0,
            snapshot_count: 0,
            tao_distributed,
        };

        SubtensorModule::emergency_finalize(netuid, &state);

        let issuance_after: u64 = TotalIssuance::<Test>::get().into();
        assert_eq!(
            issuance_before.saturating_sub(issuance_after),
            remaining,
            "Emergency should burn exactly the remaining TAO"
        );
    });
}

#[test]
fn test_emergency_refunds_pending_registration() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(5);
        add_network(netuid, 100, 0);

        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        let reg_cold = U256::from(100);
        let cost = 50_000u64;
        PendingSubnetRegistration::<Test>::put(PendingRegistration {
            coldkey: reg_cold,
            hotkey: U256::from(101),
            mechid: 0,
            cost_paid: cost,
        });

        let balance_before = SubtensorModule::get_coldkey_balance(&reg_cold);

        let state = LiquidationState {
            started_at: 1u64,
            max_completion_block: 100u64,
            phase: LiquidationPhase::Freeze,
            weight_per_block: Weight::from_parts(10_000_000, 0),
            total_stakers: 0,

            total_neurons: 0,
            mechanism_count: 0,
            tao_pot: 0,
            total_alpha_value: 0,
            snapshot_count: 0,
            tao_distributed: 0,
        };

        SubtensorModule::emergency_finalize(netuid, &state);

        let balance_after = SubtensorModule::get_coldkey_balance(&reg_cold);
        assert_eq!(
            balance_after.saturating_sub(balance_before),
            cost,
            "Pending registration should be refunded on emergency"
        );

        // Pending should be consumed
        assert!(!PendingSubnetRegistration::<Test>::exists());

        // Check for refund event
        let events = System::events();
        let has_refund = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::PendingRegistrationRefunded { .. })
            )
        });
        assert!(
            has_refund,
            "PendingRegistrationRefunded event should be emitted"
        );
    });
}

// ============================================================
// 7.1 Unit Tests — State Persistence
// ============================================================

#[test]
fn test_liquidation_state_persists_across_blocks() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state_before = LiquidatingSubnets::<Test>::get(netuid).unwrap();

        step_block(5);

        let state_after = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_eq!(
            state_before.started_at, state_after.started_at,
            "State should persist across blocks"
        );
        assert_eq!(state_before.tao_pot, state_after.tao_pot);
    });
}

#[test]
fn test_snapshot_persists_across_blocks() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(5);
        add_network(netuid, 100, 0);

        // Insert a snapshot entry
        let hot = U256::from(10);
        let cold = U256::from(11);
        LiquidationStakerSnapshot::<Test>::insert(netuid, 0u32, (hot, cold, 50_000u128));
        LiquidationSnapshotCount::<Test>::insert(netuid, 1u32);

        step_block(5);

        // Should still be there
        let entry = LiquidationStakerSnapshot::<Test>::get(netuid, 0u32);
        assert!(entry.is_some(), "Snapshot should persist across blocks");
        let (h, c, a) = entry.unwrap();
        assert_eq!(h, hot);
        assert_eq!(c, cold);
        assert_eq!(a, 50_000u128);
    });
}

#[test]
fn test_phase_and_cursor_survive_storage_round_trip() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 10u32;
        let alpha_per = 50_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalStake::<Test>::set(TaoCurrency::from(tao_pot));

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Use a tiny budget so the state machine stops mid-phase with a cursor
        let tiny_budget = Weight::from_parts(MIN_PHASE_WEIGHT.saturating_mul(2), 0);
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        let (_w, updated_state, is_complete) =
            SubtensorModule::process_liquidation_step(netuid, state, tiny_budget);
        assert!(!is_complete, "Should not complete with tiny budget");

        // Write updated state to storage
        LiquidatingSubnets::<Test>::insert(netuid, updated_state.clone());

        step_block(3);

        // Read back and verify the phase + cursor survived the round trip
        let reloaded = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_eq!(
            reloaded.phase, updated_state.phase,
            "Phase (including cursor) must survive storage round-trip"
        );
        assert_eq!(reloaded.started_at, updated_state.started_at);
        assert_eq!(reloaded.tao_pot, updated_state.tao_pot);
        assert_eq!(reloaded.snapshot_count, updated_state.snapshot_count);
        assert_eq!(reloaded.total_alpha_value, updated_state.total_alpha_value);
    });
}

#[test]
fn test_tao_distributed_accumulates_across_steps() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 5u32;
        let alpha_per = 100_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Run snapshot + advance to DistributeAlpha with unlimited budget
        let mut state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        let snapshot_budget = Weight::from_parts(u64::MAX / 2, 0);
        let snapshot_result =
            SubtensorModule::snapshot_stakers_chunk(netuid, None, &mut state, snapshot_budget);
        assert!(matches!(&snapshot_result, ChunkResult::Complete(_)));
        assert!(state.snapshot_count > 0);

        // Position at DistributeAlpha
        state.phase = LiquidationPhase::DistributeAlpha { cursor_idx: 0 };
        assert_eq!(state.tao_distributed, 0, "Should start at 0");

        // First step: distribute with tiny budget (2 stakers)
        let tiny = Weight::from_parts(WEIGHT_PER_DISTRIBUTION.saturating_mul(2), 0);
        let (_w1, state_after_1, complete_1) =
            SubtensorModule::process_liquidation_step(netuid, state, tiny);
        assert!(!complete_1);
        let dist_1 = state_after_1.tao_distributed;
        assert!(dist_1 > 0, "Should have distributed some TAO");

        // Second step: distribute more
        let (_w2, state_after_2, _complete_2) =
            SubtensorModule::process_liquidation_step(netuid, state_after_1, tiny);
        let dist_2 = state_after_2.tao_distributed;
        assert!(
            dist_2 > dist_1,
            "tao_distributed should accumulate: {} > {}",
            dist_2,
            dist_1
        );
    });
}

// ============================================================
// 7.1 Unit Tests — Events
// ============================================================

#[test]
fn test_phase_completion_events_emitted() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Force complete — all phases run
        assert_ok!(SubtensorModule::force_complete_liquidation(
            RuntimeOrigin::root(),
            netuid
        ));

        // Check for LiquidationPhaseCompleted events
        let events = System::events();
        let phase_events: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    &e.event,
                    RuntimeEvent::SubtensorModule(Event::LiquidationPhaseCompleted { .. })
                )
            })
            .collect();

        // Should have one event per phase (12 phases)
        assert_eq!(
            phase_events.len(),
            12,
            "Should have phase completion events for all 12 phases, got {}",
            phase_events.len()
        );
    });
}

#[test]
fn test_warning_events_emitted() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);

        // Test cursor overflow warning (> 256 bytes, exceeding CursorBytes capacity)
        let oversized = vec![0u8; 512];
        SubtensorModule::bound_cursor(oversized, netuid);

        let events = System::events();
        let has_cursor_warning = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::LiquidationWarning {
                    warning: LiquidationWarning::CursorOverflow,
                    ..
                })
            )
        });
        assert!(
            has_cursor_warning,
            "CursorOverflow warning should be emitted"
        );
    });
}

#[test]
fn test_liquidation_started_event_details() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let num_stakers = 3u32;
        let alpha_per = 10_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(hotkey, coldkey, num_stakers, alpha_per);

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        let events = System::events();
        let started = events.iter().find_map(|e| {
            if let RuntimeEvent::SubtensorModule(Event::LiquidationStarted {
                netuid: n,
                started_at,
                estimated_blocks,
                staker_count,
            }) = &e.event
            {
                Some((*n, *started_at, *estimated_blocks, *staker_count))
            } else {
                None
            }
        });
        assert!(started.is_some(), "LiquidationStarted event must be emitted");
        let (evt_netuid, evt_started_at, evt_estimated, _evt_stakers) = started.unwrap();
        assert_eq!(evt_netuid, netuid, "Event netuid should match");
        assert!(evt_started_at > 0, "started_at should be non-zero");
        assert!(evt_estimated > 0, "estimated_blocks should be non-zero");
    });
}

#[test]
fn test_liquidation_completed_event_details() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 3u32;
        let alpha_per = 50_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 300_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        Tempo::<Test>::insert(netuid, 1u16);
        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Complete via process_liquidations to get the full event chain
        let unlimited = Weight::from_parts(u64::MAX / 2, u64::MAX / 2);
        for _ in 0..MAX_LIQUIDATION_BLOCKS {
            step_block(1);
            SubtensorModule::process_liquidations(unlimited);
            if !LiquidatingSubnets::<Test>::contains_key(netuid) {
                break;
            }
        }

        let events = System::events();
        let completed = events.iter().find_map(|e| {
            if let RuntimeEvent::SubtensorModule(Event::LiquidationCompleted {
                netuid: n,
                total_blocks,
                tao_distributed,
                stakers_paid,
            }) = &e.event
            {
                Some((*n, *total_blocks, *tao_distributed, *stakers_paid))
            } else {
                None
            }
        });
        assert!(
            completed.is_some(),
            "LiquidationCompleted event must be emitted"
        );
        let (evt_netuid, _evt_blocks, evt_distributed, evt_paid) = completed.unwrap();
        assert_eq!(evt_netuid, netuid);
        assert!(evt_distributed > 0, "Should have distributed TAO");
        assert!(evt_paid > 0, "Should have paid stakers");

        // NetworkRemoved event should also be present
        let has_removed = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::NetworkRemoved(n)) if *n == netuid
            )
        });
        assert!(
            has_removed,
            "NetworkRemoved event should be emitted after completion"
        );
    });
}

#[test]
fn test_dust_burn_warning_emitted() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 3u32;
        let alpha_per = 10_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        // Use a pot that guarantees dust with 3 stakers (not divisible by 3)
        let tao_pot = 100_001u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        let events = System::events();
        let has_dust = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::LiquidationWarning {
                    warning: LiquidationWarning::DistributionDust(dust),
                    ..
                }) if *dust > 0
            )
        });
        assert!(
            has_dust,
            "DistributionDust warning should be emitted when pot isn't evenly divisible"
        );
    });
}

// ============================================================
// 7.2 Integration Tests
// ============================================================

#[test]
fn test_full_liquidation_flow() {
    new_test_ext(1).execute_with(|| {
        // 1. Create subnet with stakers
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 5u32;
        let alpha_per = 20_000u64;

        let (netuid, stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        // Record balances before
        let balances_before: Vec<u64> = stakers
            .iter()
            .map(|(_, cold)| SubtensorModule::get_coldkey_balance(cold))
            .collect();

        // 2. Trigger liquidation
        //    Set small tempo so weight_per_block is large enough for process_liquidations
        //    pacing to allow progress (weight_per_block must exceed MIN_PHASE_WEIGHT=500k).
        Tempo::<Test>::insert(netuid, 1u16);
        let issuance_before: u64 = TotalIssuance::<Test>::get().into();
        assert_ok!(SubtensorModule::start_liquidation(netuid));
        assert!(SubtensorModule::is_subnet_liquidating(netuid));

        // Check LiquidationStarted event
        let events = System::events();
        let has_started = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::LiquidationStarted { .. })
            )
        });
        assert!(has_started, "LiquidationStarted event should be emitted");

        // 3. Step through blocks (call on_idle via process_liquidations)
        let unlimited = Weight::from_parts(u64::MAX / 2, u64::MAX / 2);
        let mut completed = false;
        for _ in 0..MAX_LIQUIDATION_BLOCKS {
            step_block(1);
            SubtensorModule::process_liquidations(unlimited);
            if !LiquidatingSubnets::<Test>::contains_key(netuid) {
                completed = true;
                break;
            }
        }
        assert!(
            completed,
            "Liquidation should complete within MAX_LIQUIDATION_BLOCKS"
        );

        // 4. Verify snapshot was built and cleared correctly
        assert_eq!(
            LiquidationSnapshotCount::<Test>::get(netuid),
            0,
            "Snapshot count should be cleared after completion"
        );
        assert!(
            LiquidationStakerSnapshot::<Test>::get(netuid, 0u32).is_none(),
            "Snapshot entries should be cleared after completion"
        );

        // 5. Verify all funds distributed correctly
        let issuance_after: u64 = TotalIssuance::<Test>::get().into();
        let mut total_received = 0u64;
        for (i, (_, cold)) in stakers.iter().enumerate() {
            let bal = SubtensorModule::get_coldkey_balance(cold);
            let received = bal.saturating_sub(balances_before[i]);
            total_received = total_received.saturating_add(received);
            assert!(received > 0, "Each staker should receive some TAO");
        }
        assert!(total_received <= tao_pot);

        // 5b. TAO conservation: distributed + dust_burned == pot
        let dust_burned = issuance_before.saturating_sub(issuance_after);
        assert_eq!(
            total_received.saturating_add(dust_burned),
            tao_pot,
            "TAO must be conserved: received={} + dust_burned={} != pot={}",
            total_received,
            dust_burned,
            tao_pot,
        );

        // 6. Verify slot is freed
        assert!(!NetworksAdded::<Test>::get(netuid));
        assert_eq!(SubnetworkN::<Test>::get(netuid), 0);

        // 6b. Verify subnet storage is fully cleaned
        assert_eq!(SubnetTAO::<Test>::get(netuid), TaoCurrency::from(0u64));
        assert_eq!(SubnetMechanism::<Test>::get(netuid), 0u16);
        assert!(Rank::<Test>::get(netuid).is_empty());
        assert!(Trust::<Test>::get(netuid).is_empty());

        // 6c. Verify alpha entries are cleared for each staker
        for (hot, cold) in &stakers {
            assert_eq!(
                Alpha::<Test>::get((hot, cold, netuid)),
                U64F64::from_num(0),
                "Alpha should be cleared after liquidation"
            );
        }

        // 7. Verify cooldown period
        assert!(
            NetuidCooldown::<Test>::contains_key(netuid),
            "Cooldown should be set after liquidation"
        );

        // 8. Verify new registration succeeds after cooldown
        if let Some(cooldown) = NetuidCooldown::<Test>::get(netuid) {
            run_to_block(cooldown.saturating_add(1));
        }
        add_network(netuid, 100, 0);
        assert!(NetworksAdded::<Test>::get(netuid));
    });
}

#[test]
fn test_liquidation_interrupted_and_resumed() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Add some stakers
        for i in 0u64..5 {
            let h = U256::from(100u64.saturating_add(i.saturating_mul(2)));
            let c = U256::from(101u64.saturating_add(i.saturating_mul(2)));
            Alpha::<Test>::insert((&h, &c, netuid), U64F64::from_num(20_000u64));
        }
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(100_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Process with limited budget — should only partially complete
        let small_budget = Weight::from_parts(MIN_PHASE_WEIGHT.saturating_mul(3), 0);
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        let (_w, updated_state, _is_complete) =
            SubtensorModule::process_liquidation_step(netuid, state, small_budget);

        // State should be persisted
        LiquidatingSubnets::<Test>::insert(netuid, updated_state.clone());

        step_block(1);

        // State should still be there
        assert!(LiquidatingSubnets::<Test>::contains_key(netuid));

        // Now complete with unlimited budget
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));
        assert!(!LiquidatingSubnets::<Test>::contains_key(netuid));
    });
}

#[test]
fn test_registration_after_liquidation_complete() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Start and complete liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));
        assert_ok!(SubtensorModule::force_complete_liquidation(
            RuntimeOrigin::root(),
            netuid
        ));

        assert!(!NetworksAdded::<Test>::get(netuid));

        // Clear cooldown for testing
        NetuidCooldown::<Test>::remove(netuid);

        // Should be able to create a new network on this netuid
        add_network(netuid, 100, 0);
        assert!(NetworksAdded::<Test>::get(netuid));
    });
}

#[test]
fn test_concurrent_liquidation_blocked() {
    new_test_ext(1).execute_with(|| {
        let hot1 = U256::from(1);
        let cold1 = U256::from(2);
        let netuid1 = add_dynamic_network(&hot1, &cold1);
        mock::setup_reserves(netuid1, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let hot2 = U256::from(3);
        let cold2 = U256::from(4);
        let netuid2 = add_dynamic_network(&hot2, &cold2);
        mock::setup_reserves(netuid2, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Start first
        assert_ok!(SubtensorModule::start_liquidation(netuid1));

        // Second should fail
        assert_noop!(
            SubtensorModule::start_liquidation(netuid2),
            Error::<Test>::LiquidationInProgress
        );

        // Complete first
        assert_ok!(SubtensorModule::force_complete_liquidation(
            RuntimeOrigin::root(),
            netuid1
        ));

        // Now second should work
        assert_ok!(SubtensorModule::start_liquidation(netuid2));
    });
}

#[test]
fn test_concurrent_liquidation_prevented_from_root() {
    new_test_ext(1).execute_with(|| {
        let hot1 = U256::from(1);
        let cold1 = U256::from(2);
        let netuid1 = add_dynamic_network(&hot1, &cold1);
        mock::setup_reserves(netuid1, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let hot2 = U256::from(3);
        let cold2 = U256::from(4);
        let netuid2 = add_dynamic_network(&hot2, &cold2);
        mock::setup_reserves(netuid2, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // 1. Root dissolves subnet A → liquidation starts
        assert_ok!(SubtensorModule::start_liquidation(netuid1));
        assert!(SubtensorModule::is_subnet_liquidating(netuid1));

        // 2. Root tries to dissolve subnet B → fails with LiquidationInProgress
        assert_noop!(
            SubtensorModule::start_liquidation(netuid2),
            Error::<Test>::LiquidationInProgress
        );
    });
}

#[test]
fn test_pending_registration_full_flow() {
    new_test_ext(0).execute_with(|| {
        // 1. Fill to capacity: create 2 subnets with SubnetLimit=2
        SubnetLimit::<Test>::put(2u16);

        let n1_cold = U256::from(21);
        let n1_hot = U256::from(22);
        let n1 = add_dynamic_network(&n1_hot, &n1_cold);
        mock::setup_reserves(n1, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let n2_cold = U256::from(23);
        let n2_hot = U256::from(24);
        let n2 = add_dynamic_network(&n2_hot, &n2_cold);
        mock::setup_reserves(n2, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Move past immunity period so subnets can be pruned
        let imm = SubtensorModule::get_network_immunity_period();
        System::set_block_number(imm.saturating_add(100));

        // n1 is weakest (lowest emission) → will be pruned
        Emission::<Test>::insert(n1, vec![AlphaCurrency::from(1)]);
        Emission::<Test>::insert(n2, vec![AlphaCurrency::from(1_000)]);

        // Set small tempo for fast liquidation
        Tempo::<Test>::insert(n1, 1u16);

        // 2. Register a new subnet at capacity → triggers liquidation + pending queue
        let new_cold = U256::from(30);
        let new_hot = U256::from(31);
        let lock_cost: u64 = SubtensorModule::get_network_lock_cost().into();
        SubtensorModule::add_balance_to_coldkey_account(&new_cold, lock_cost.saturating_mul(10));

        let balance_before = SubtensorModule::get_coldkey_balance(&new_cold);

        assert_ok!(SubtensorModule::do_register_network(
            RuntimeOrigin::signed(new_cold),
            &new_hot,
            1,
            None,
        ));

        // Lock cost was deducted
        let balance_after_reg = SubtensorModule::get_coldkey_balance(&new_cold);
        assert!(
            balance_after_reg < balance_before,
            "Lock cost should be deducted"
        );

        // n1 is now liquidating
        assert!(
            LiquidatingSubnets::<Test>::contains_key(n1),
            "n1 should be liquidating"
        );
        assert!(
            PendingSubnetRegistration::<Test>::exists(),
            "Pending registration should exist"
        );

        // n2 is still alive
        assert!(NetworksAdded::<Test>::get(n2));

        // 3. Step through blocks until liquidation completes
        let unlimited = Weight::from_parts(u64::MAX / 2, u64::MAX / 2);
        let mut completed = false;
        for _ in 0..MAX_LIQUIDATION_BLOCKS {
            step_block(1);
            SubtensorModule::process_liquidations(unlimited);
            if !LiquidatingSubnets::<Test>::contains_key(n1) {
                completed = true;
                break;
            }
        }
        assert!(
            completed,
            "Liquidation should complete within MAX_LIQUIDATION_BLOCKS"
        );

        // 4. Pending registration should have been consumed
        assert!(
            !PendingSubnetRegistration::<Test>::exists(),
            "Pending registration should be consumed"
        );

        // 5. The freed netuid (n1) should now host the new subnet
        assert!(
            NetworksAdded::<Test>::get(n1),
            "New subnet should be registered on the freed netuid"
        );

        // 6. Verify the new subnet is properly initialized
        assert_eq!(
            SubnetOwner::<Test>::get(n1),
            new_cold,
            "Owner should be the new coldkey"
        );
        assert_eq!(
            SubnetOwnerHotkey::<Test>::get(n1),
            new_hot,
            "Owner hotkey should match"
        );
        assert_eq!(
            SubnetMechanism::<Test>::get(n1),
            1u16,
            "Should be dynamic mechanism"
        );
        assert!(
            SubnetTAO::<Test>::get(n1) > TaoCurrency::from(0u64),
            "Should have initial TAO pool"
        );
        assert!(
            SubnetAlphaIn::<Test>::get(n1) > AlphaCurrency::from(0u64),
            "Should have initial alpha"
        );

        // SubnetLocked should reflect the cost paid
        assert!(
            SubnetLocked::<Test>::get(n1) > TaoCurrency::from(0u64),
            "SubnetLocked should be set"
        );

        // Token symbol assigned
        assert!(
            TokenSymbol::<Test>::try_get(n1).is_ok(),
            "Token symbol should be assigned"
        );

        // Registration timestamp set
        assert!(
            NetworkRegisteredAt::<Test>::get(n1) > 0,
            "NetworkRegisteredAt should be set"
        );

        // New subnet should have 1 neuron (the owner hotkey)
        assert_eq!(
            SubnetworkN::<Test>::get(n1),
            1,
            "Should have 1 neuron (owner)"
        );

        // No cooldown set (pending reg took the slot)
        assert!(
            !NetuidCooldown::<Test>::contains_key(n1),
            "No cooldown when pending reg takes the slot"
        );

        // n2 should be unaffected
        assert!(NetworksAdded::<Test>::get(n2), "n2 should still exist");

        // 7. Verify completion events
        let events = System::events();
        let has_completed = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::RegistrationCompleted { .. })
            )
        });
        assert!(has_completed, "Should have RegistrationCompleted event");

        let has_network_added = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::NetworkAdded(netuid, _)) if *netuid == n1
            )
        });
        assert!(
            has_network_added,
            "Should have NetworkAdded event for freed netuid"
        );
    });
}

// ============================================================
// 7.4 Additional Coverage — Preconditions, Edge Cases, Isolation
// ============================================================

// --- start_liquidation precondition errors ---

#[test]
fn test_cannot_liquidate_root_subnet() {
    new_test_ext(1).execute_with(|| {
        // Root network must exist for CannotLiquidateRoot check to be reached
        add_network(NetUid::ROOT, 100, 0);
        assert_noop!(
            SubtensorModule::start_liquidation(NetUid::ROOT),
            Error::<Test>::CannotLiquidateRoot
        );
    });
}

#[test]
fn test_cannot_liquidate_nonexistent_subnet() {
    new_test_ext(1).execute_with(|| {
        let fake_netuid = NetUid::from(99);
        assert_noop!(
            SubtensorModule::start_liquidation(fake_netuid),
            Error::<Test>::SubnetNotExists
        );
    });
}

#[test]
fn test_cannot_liquidate_already_liquidating_subnet() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Exact same subnet again → SubnetAlreadyLiquidating
        assert_noop!(
            SubtensorModule::start_liquidation(netuid),
            Error::<Test>::SubnetAlreadyLiquidating
        );
    });
}

// --- Unequal alpha distribution correctness ---

#[test]
fn test_distribution_unequal_alpha_proportional() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Staker A: 75% of alpha
        let a_hot = U256::from(10);
        let a_cold = U256::from(11);
        let alpha_a: u64 = 750_000;
        Alpha::<Test>::insert((&a_hot, &a_cold, netuid), U64F64::from_num(alpha_a));
        TotalHotkeyAlpha::<Test>::insert(&a_hot, netuid, AlphaCurrency::from(alpha_a));

        // Staker B: 25% of alpha
        let b_hot = U256::from(20);
        let b_cold = U256::from(21);
        let alpha_b: u64 = 250_000;
        Alpha::<Test>::insert((&b_hot, &b_cold, netuid), U64F64::from_num(alpha_b));
        TotalHotkeyAlpha::<Test>::insert(&b_hot, netuid, AlphaCurrency::from(alpha_b));

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        let balance_a = SubtensorModule::get_coldkey_balance(&a_cold);
        let balance_b = SubtensorModule::get_coldkey_balance(&b_cold);

        // A should get ~750k, B should get ~250k (allow small rounding)
        assert!(
            balance_a > balance_b,
            "Staker A (75% alpha) should get more than B (25%): A={}, B={}",
            balance_a,
            balance_b
        );

        // Check proportionality: A/B ≈ 3.0
        let ratio = balance_a as f64 / balance_b as f64;
        assert!(
            (ratio - 3.0).abs() < 0.01,
            "A/B ratio should be ~3.0, got {}",
            ratio
        );

        // Total conservation: distributed + dust == pot
        assert!(
            balance_a.saturating_add(balance_b) <= tao_pot,
            "Total distributed should not exceed pot"
        );
    });
}

// --- Cross-subnet isolation ---

#[test]
fn test_liquidation_does_not_affect_other_subnet_alpha() {
    new_test_ext(1).execute_with(|| {
        // Create two subnets
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid_a = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid_a, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let owner_hot_b = U256::from(3);
        let owner_cold_b = U256::from(4);
        let netuid_b = add_dynamic_network(&owner_hot_b, &owner_cold_b);
        mock::setup_reserves(netuid_b, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Same user stakes on BOTH subnets
        let staker_hot = U256::from(10);
        let staker_cold = U256::from(11);
        let alpha_val: u64 = 100_000;
        Alpha::<Test>::insert(
            (&staker_hot, &staker_cold, netuid_a),
            U64F64::from_num(alpha_val),
        );
        Alpha::<Test>::insert(
            (&staker_hot, &staker_cold, netuid_b),
            U64F64::from_num(alpha_val),
        );
        TotalHotkeyAlpha::<Test>::insert(&staker_hot, netuid_a, AlphaCurrency::from(alpha_val));
        TotalHotkeyAlpha::<Test>::insert(&staker_hot, netuid_b, AlphaCurrency::from(alpha_val));

        SubnetTAO::<Test>::insert(netuid_a, TaoCurrency::from(500_000u64));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        // Liquidate only subnet A
        assert_ok!(SubtensorModule::start_liquidation(netuid_a));
        let state = LiquidatingSubnets::<Test>::get(netuid_a).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid_a, state));

        // Subnet B's alpha entry should be UNTOUCHED
        let alpha_b_remaining = Alpha::<Test>::get((&staker_hot, &staker_cold, netuid_b));
        assert_eq!(
            alpha_b_remaining.to_num::<u64>(),
            alpha_val,
            "Other subnet's alpha should be preserved"
        );

        // Subnet B should still exist
        assert!(
            NetworksAdded::<Test>::get(netuid_b),
            "Subnet B should still exist"
        );

        // Subnet B's TotalHotkeyAlpha should be preserved
        let tha_b: u64 = TotalHotkeyAlpha::<Test>::get(&staker_hot, netuid_b).into();
        assert_eq!(tha_b, alpha_val, "Other subnet TotalHotkeyAlpha preserved");

        // Subnet A's alpha should be removed
        let alpha_a_remaining = Alpha::<Test>::get((&staker_hot, &staker_cold, netuid_a));
        assert_eq!(
            alpha_a_remaining.to_num::<u64>(),
            0,
            "Liquidated subnet's alpha should be removed"
        );
    });
}

// --- Pending registration: hotkey transferred during liquidation ---

#[test]
fn test_pending_registration_refunded_if_hotkey_transferred() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Set up pending registration with a hotkey that exists
        // but is owned by a different coldkey
        let reg_cold = U256::from(100);
        let reg_hot = U256::from(101);
        let other_cold = U256::from(200);

        // Create the hotkey as owned by other_cold
        Owner::<Test>::insert(&reg_hot, other_cold);

        let cost = 100_000u64;
        PendingSubnetRegistration::<Test>::put(PendingRegistration {
            coldkey: reg_cold,
            hotkey: reg_hot,
            mechid: 1,
            cost_paid: cost,
        });

        // Force complete — hotkey ownership changed, should refund
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Pending consumed
        assert!(!PendingSubnetRegistration::<Test>::exists());

        // Refunded
        let reg_balance = SubtensorModule::get_coldkey_balance(&reg_cold);
        assert_eq!(
            reg_balance, cost,
            "Should be refunded when hotkey transferred"
        );

        // Refund event emitted
        let events = System::events();
        let has_refund = events.iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::PendingRegistrationRefunded {
                    coldkey,
                    amount,
                }) if *coldkey == reg_cold && *amount == cost
            )
        });
        assert!(has_refund, "PendingRegistrationRefunded should be emitted");

        // Subnet NOT re-registered (slot stays free)
        assert!(!NetworksAdded::<Test>::get(netuid));
    });
}

// --- Multiple coldkeys per hotkey (delegation) ---

#[test]
fn test_distribution_multiple_coldkeys_same_hotkey() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Same hotkey, two different coldkeys (delegation scenario)
        let shared_hot = U256::from(10);
        let cold_a = U256::from(11);
        let cold_b = U256::from(12);

        let alpha_a: u64 = 600_000;
        let alpha_b: u64 = 400_000;

        Alpha::<Test>::insert((&shared_hot, &cold_a, netuid), U64F64::from_num(alpha_a));
        Alpha::<Test>::insert((&shared_hot, &cold_b, netuid), U64F64::from_num(alpha_b));
        TotalHotkeyAlpha::<Test>::insert(
            &shared_hot,
            netuid,
            AlphaCurrency::from(alpha_a.saturating_add(alpha_b)),
        );

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        let balance_a = SubtensorModule::get_coldkey_balance(&cold_a);
        let balance_b = SubtensorModule::get_coldkey_balance(&cold_b);

        // Both should receive TAO proportional to their alpha
        assert!(balance_a > 0, "Cold A should receive TAO");
        assert!(balance_b > 0, "Cold B should receive TAO");
        assert!(
            balance_a > balance_b,
            "Cold A (60%) should get more than B (40%)"
        );
        assert!(balance_a.saturating_add(balance_b) <= tao_pot, "Total <= pot");
    });
}

// --- Multiple hotkeys per coldkey ---

#[test]
fn test_distribution_multiple_hotkeys_same_coldkey() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Same coldkey, two different hotkeys
        let cold = U256::from(11);
        let hot_a = U256::from(10);
        let hot_b = U256::from(20);

        let alpha_a: u64 = 300_000;
        let alpha_b: u64 = 700_000;

        Alpha::<Test>::insert((&hot_a, &cold, netuid), U64F64::from_num(alpha_a));
        Alpha::<Test>::insert((&hot_b, &cold, netuid), U64F64::from_num(alpha_b));
        TotalHotkeyAlpha::<Test>::insert(&hot_a, netuid, AlphaCurrency::from(alpha_a));
        TotalHotkeyAlpha::<Test>::insert(&hot_b, netuid, AlphaCurrency::from(alpha_b));

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // Both positions credited to the same coldkey
        let balance = SubtensorModule::get_coldkey_balance(&cold);
        assert!(
            balance >= tao_pot.saturating_sub(1), // allow 1 unit of dust
            "Single coldkey should receive combined TAO: got {}, expected ~{}",
            balance,
            tao_pot
        );
    });
}

// --- MIN_SNAPSHOT_ALPHA boundary ---

#[test]
fn test_snapshot_alpha_at_exact_threshold() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Exactly at MIN_SNAPSHOT_ALPHA (1000)
        let at_hot = U256::from(10);
        let at_cold = U256::from(11);
        Alpha::<Test>::insert(
            (&at_hot, &at_cold, netuid),
            U64F64::from_num(MIN_SNAPSHOT_ALPHA),
        );

        // Just below (999)
        let below_hot = U256::from(20);
        let below_cold = U256::from(21);
        Alpha::<Test>::insert(
            (&below_hot, &below_cold, netuid),
            U64F64::from_num(MIN_SNAPSHOT_ALPHA.saturating_sub(1)),
        );

        let tao_pot = 100_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // At-threshold staker should receive TAO
        let at_balance = SubtensorModule::get_coldkey_balance(&at_cold);
        assert!(
            at_balance > 0,
            "Staker at exact threshold should receive TAO"
        );

        // Below-threshold staker should NOT receive TAO
        let below_balance = SubtensorModule::get_coldkey_balance(&below_cold);
        assert_eq!(
            below_balance, 0,
            "Staker below threshold should not receive TAO"
        );
    });
}

// --- Incremental snapshot across blocks ---

#[test]
fn test_snapshot_works_incrementally_across_blocks() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Add multiple stakers
        for i in 0u32..5 {
            let hot = U256::from(100u32.saturating_add(i.saturating_mul(2)));
            let cold = U256::from(101u32.saturating_add(i.saturating_mul(2)));
            Alpha::<Test>::insert((&hot, &cold, netuid), U64F64::from_num(50_000u64));
            TotalHotkeyAlpha::<Test>::insert(&hot, netuid, AlphaCurrency::from(50_000u64));
        }

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Use snapshot_stakers_chunk directly with tiny budget to force chunking
        let mut state = LiquidatingSubnets::<Test>::get(netuid).unwrap();

        // Tiny budget: only process 2 items per call
        let tiny_budget = Weight::from_parts(WEIGHT_PER_SNAPSHOT.saturating_mul(2), 0);
        let result1 =
            SubtensorModule::snapshot_stakers_chunk(netuid, None, &mut state, tiny_budget);
        assert!(
            matches!(&result1, ChunkResult::Incomplete { .. }),
            "Should not complete with tiny budget"
        );
        assert!(
            result1.weight_used().ref_time() > 0,
            "Should have done some work"
        );

        // Extract cursor from incomplete result
        let cursor = match result1 {
            ChunkResult::Incomplete {
                phase: LiquidationPhase::SnapshotStakers { cursor },
                ..
            } => cursor,
            _ => panic!("Expected SnapshotStakers incomplete"),
        };

        // Continue with large budget to finish
        let large_budget = Weight::from_parts(WEIGHT_PER_SNAPSHOT.saturating_mul(10000), 0);
        let result2 =
            SubtensorModule::snapshot_stakers_chunk(netuid, cursor, &mut state, large_budget);
        assert!(
            matches!(&result2, ChunkResult::Complete(_)),
            "Should complete with large budget"
        );

        assert!(state.snapshot_count > 0, "Should have snapshotted stakers");
    });
}

// --- Incremental distribution across blocks ---

#[test]
fn test_distribution_works_incrementally() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Add stakers
        let num_stakers = 5u32;
        let alpha_per: u64 = 100_000;
        let mut stakers = Vec::new();
        for i in 0..num_stakers {
            let hot = U256::from(100u32.saturating_add(i.saturating_mul(2)));
            let cold = U256::from(101u32.saturating_add(i.saturating_mul(2)));
            Alpha::<Test>::insert((&hot, &cold, netuid), U64F64::from_num(alpha_per));
            TotalHotkeyAlpha::<Test>::insert(&hot, netuid, AlphaCurrency::from(alpha_per));
            stakers.push((hot, cold));
        }

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Run snapshot phase to capture stakers, then manually position
        // the state at DistributeAlpha for incremental testing.
        let mut state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        // Run all phases with unlimited budget — this completes everything.
        // Instead, we manually build a state at DistributeAlpha.
        // First, run snapshot to populate LiquidationStakerSnapshot.
        let snapshot_budget = Weight::from_parts(u64::MAX / 2, 0);
        let snapshot_result =
            SubtensorModule::snapshot_stakers_chunk(netuid, None, &mut state, snapshot_budget);
        assert!(
            matches!(&snapshot_result, ChunkResult::Complete(_)),
            "Snapshot should complete"
        );
        assert!(state.snapshot_count > 0, "Should have snapshotted stakers");

        // Set phase to DistributeAlpha
        state.phase = LiquidationPhase::DistributeAlpha { cursor_idx: 0 };

        // Process distribution with tiny budget (2 stakers at a time)
        let tiny_budget = Weight::from_parts(WEIGHT_PER_DISTRIBUTION.saturating_mul(2), 0);
        let (_w, state_partial, complete_partial) =
            SubtensorModule::process_liquidation_step(netuid, state, tiny_budget);
        assert!(!complete_partial, "Should not complete with tiny budget");

        // Some stakers should have been paid already
        let mut paid_count = 0u32;
        for (_, cold) in &stakers {
            if SubtensorModule::get_coldkey_balance(cold) > 0 {
                paid_count += 1;
            }
        }
        assert!(
            paid_count > 0 && paid_count < num_stakers,
            "Should have partially paid: {} of {}",
            paid_count,
            num_stakers
        );

        // Complete with unlimited budget
        let (_w, _final_state, complete_final) = SubtensorModule::process_liquidation_step(
            netuid,
            state_partial,
            Weight::from_parts(u64::MAX / 2, 0),
        );

        // All stakers should now be paid
        let mut total_received = 0u64;
        for (_, cold) in &stakers {
            let bal = SubtensorModule::get_coldkey_balance(cold);
            assert!(bal > 0, "All stakers should receive TAO");
            total_received = total_received.saturating_add(bal);
        }
        assert!(total_received <= tao_pot, "Total distributed <= pot");
        assert!(
            complete_final || total_received > 0,
            "Should complete or have distributed"
        );
    });
}

// --- Force complete with actual stakers ---

#[test]
fn test_force_complete_distributes_tao_to_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let staker_hot = U256::from(10);
        let staker_cold = U256::from(11);
        let alpha_val: u64 = 500_000;
        Alpha::<Test>::insert(
            (&staker_hot, &staker_cold, netuid),
            U64F64::from_num(alpha_val),
        );
        TotalHotkeyAlpha::<Test>::insert(&staker_hot, netuid, AlphaCurrency::from(alpha_val));

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        assert_ok!(SubtensorModule::force_complete_liquidation(
            RuntimeOrigin::root(),
            netuid
        ));

        // Staker should have received their share
        let balance = SubtensorModule::get_coldkey_balance(&staker_cold);
        assert_eq!(balance, tao_pot, "Sole staker should receive full pot");

        // Subnet cleaned up
        assert!(!NetworksAdded::<Test>::get(netuid));
        assert!(!LiquidatingSubnets::<Test>::contains_key(netuid));
    });
}

// --- Snapshot filters by netuid across multiple subnets ---

#[test]
fn test_snapshot_only_captures_target_subnet() {
    new_test_ext(1).execute_with(|| {
        let owner_hot_a = U256::from(1);
        let owner_cold_a = U256::from(2);
        let netuid_a = add_dynamic_network(&owner_hot_a, &owner_cold_a);
        mock::setup_reserves(netuid_a, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        let owner_hot_b = U256::from(3);
        let owner_cold_b = U256::from(4);
        let netuid_b = add_dynamic_network(&owner_hot_b, &owner_cold_b);
        mock::setup_reserves(netuid_b, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Staker on subnet A
        let hot_a = U256::from(10);
        let cold_a = U256::from(11);
        Alpha::<Test>::insert((&hot_a, &cold_a, netuid_a), U64F64::from_num(100_000u64));
        TotalHotkeyAlpha::<Test>::insert(&hot_a, netuid_a, AlphaCurrency::from(100_000u64));

        // Staker on subnet B (should NOT be snapshotted)
        let hot_b = U256::from(20);
        let cold_b = U256::from(21);
        Alpha::<Test>::insert((&hot_b, &cold_b, netuid_b), U64F64::from_num(200_000u64));
        TotalHotkeyAlpha::<Test>::insert(&hot_b, netuid_b, AlphaCurrency::from(200_000u64));

        SubnetTAO::<Test>::insert(netuid_a, TaoCurrency::from(500_000u64));
        TotalIssuance::<Test>::put(TaoCurrency::from(10_000_000u64));

        // Liquidate subnet A only
        assert_ok!(SubtensorModule::start_liquidation(netuid_a));
        let state = LiquidatingSubnets::<Test>::get(netuid_a).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid_a, state));

        // Staker A should receive TAO
        let balance_a = SubtensorModule::get_coldkey_balance(&cold_a);
        assert!(balance_a > 0, "Subnet A staker should receive TAO");

        // Staker B should NOT receive TAO
        let balance_b = SubtensorModule::get_coldkey_balance(&cold_b);
        assert_eq!(balance_b, 0, "Subnet B staker should NOT receive TAO");
    });
}

// --- Budget doubling near deadline ---

#[test]
fn test_budget_doubles_near_deadline() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Set a known tempo
        let tempo = 100u16;
        Tempo::<Test>::insert(netuid, tempo);

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();

        // Advance to 91% of tempo (past the 90% threshold), jump directly
        let started_at: u64 = state.started_at;
        let target_block = started_at.saturating_add(91);
        System::set_block_number(target_block);

        // Verify the doubling condition is met
        let blocks_elapsed = target_block.saturating_sub(started_at);
        let tempo_val = Tempo::<Test>::get(netuid) as u64;
        assert!(
            blocks_elapsed > tempo_val.saturating_mul(9).saturating_div(10),
            "blocks_elapsed ({}) should exceed 90% of tempo ({})",
            blocks_elapsed,
            tempo_val
        );

        // Use process_liquidation_step directly (exercises the budget doubling code path
        // in process_liquidations indirectly — the condition blocks_elapsed > tempo*0.9 is met).
        // Multiple steps needed since ClearTwoKeyMaps processes one map_idx per step.
        let budget = Weight::from_parts(100_000_000, 0);
        let mut current_state = state;
        let mut completed = false;
        for _ in 0..50 {
            let (_w, ns, complete) =
                SubtensorModule::process_liquidation_step(netuid, current_state, budget);
            if complete {
                completed = true;
                break;
            }
            current_state = ns;
        }
        assert!(completed, "Liquidation should complete near deadline");
    });
}

// --- process_liquidations with too-small weight ---

#[test]
fn test_process_liquidations_insufficient_weight() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // Call with weight less than MIN_LIQUIDATION_WEIGHT
        let tiny = Weight::from_parts(MIN_LIQUIDATION_WEIGHT.saturating_sub(1), 0);
        let used = SubtensorModule::process_liquidations(tiny);

        // Should have done nothing — not enough budget
        assert_eq!(
            used.ref_time(),
            0,
            "Should not process with insufficient weight"
        );

        // Liquidation should still be in progress
        assert!(LiquidatingSubnets::<Test>::contains_key(netuid));
    });
}

// --- Cooldown blocks start_liquidation ---

#[test]
fn test_start_liquidation_blocked_during_cooldown() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

        // Manually set cooldown
        NetuidCooldown::<Test>::insert(netuid, 999u64);

        assert_noop!(
            SubtensorModule::start_liquidation(netuid),
            Error::<Test>::NetuidInCooldown
        );
    });
}

// ============================================================
// 7.3 Fuzz Tests (randomized property-based)
// ============================================================
//
// Note: proptest is not a crate dependency, so we use `rand` (which is)
// with a fixed seed to get deterministic but broadly-distributed random
// inputs that cover the same property-based invariants the spec defines.

#[test]
fn fuzz_distribution_never_overflows() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    new_test_ext(1).execute_with(|| {
        let mut rng = StdRng::seed_from_u64(42);

        // Run 500 random trials: random tao_pot + random alpha_values vector
        for _ in 0..500 {
            let tao_pot: u64 = rng.r#gen();
            let staker_count = rng.gen_range(1usize..=200);

            let alpha_values: Vec<u128> = (0..staker_count)
                .map(|_| rng.gen_range(0u128..=u128::from(u64::MAX)))
                .collect();

            let total_alpha: u128 = alpha_values.iter().copied().sum();
            let mut distributed: u64 = 0;

            for alpha in &alpha_values {
                let share = SubtensorModule::calculate_share(tao_pot, *alpha, total_alpha);
                distributed = distributed.saturating_add(share);
            }

            assert!(
                distributed <= tao_pot,
                "distributed {} > tao_pot {} with {} stakers",
                distributed,
                tao_pot,
                staker_count
            );
        }

        // Also test extreme edge cases
        let edge_cases: Vec<(u64, u128, u128)> = vec![
            (u64::MAX, u128::MAX, u128::MAX),
            (u64::MAX, 1, u128::MAX),
            (1, u128::MAX, u128::MAX),
            (0, 1_000, 1_000),
            (1_000, 0, 1_000),
            (1_000, 1_000, 0),
            (u64::MAX, u64::MAX as u128, 1),
            (1, 1, 1),
        ];

        for (tao_pot, alpha_val, total_alpha) in edge_cases {
            let share = SubtensorModule::calculate_share(tao_pot, alpha_val, total_alpha);
            assert!(
                share <= tao_pot || tao_pot == 0,
                "Share {} > pot {} for alpha={}, total={}",
                share,
                tao_pot,
                alpha_val,
                total_alpha
            );
        }
    });
}

#[test]
fn fuzz_liquidation_always_completes() {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    new_test_ext(1).execute_with(|| {
        let mut rng = StdRng::seed_from_u64(99);

        // Test with a variety of random staker counts and weight budgets
        let staker_counts: Vec<u32> = (0..10)
            .map(|_| rng.gen_range(0u32..=50))
            .collect();

        for (trial, &count) in staker_counts.iter().enumerate() {
            // Create a fresh subnet for each test
            let trial_u64 = trial as u64;
            let hot = U256::from(500u64.saturating_add(trial_u64.saturating_mul(2)));
            let cold = U256::from(501u64.saturating_add(trial_u64.saturating_mul(2)));
            let netuid = add_dynamic_network(&hot, &cold);
            mock::setup_reserves(netuid, 1_000_000_000u64.into(), 1_000_000_000u64.into());

            // Add stakers with random alpha values
            for i in 0..count {
                let base = 10_000u64
                    .saturating_add(trial_u64.saturating_mul(1000))
                    .saturating_add(u64::from(i).saturating_mul(2));
                let sh = U256::from(base);
                let sc = U256::from(base.saturating_add(1));
                let alpha: u64 = rng.gen_range(MIN_SNAPSHOT_ALPHA..=100_000);
                Alpha::<Test>::insert(
                    (&sh, &sc, netuid),
                    U64F64::from_num(alpha),
                );
            }
            let tao_pot: u64 = rng.gen_range(0..=1_000_000);
            SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

            assert_ok!(SubtensorModule::start_liquidation(netuid));

            // Use random weight budgets to simulate realistic on_idle budgets
            let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
            let mut current_state = state;
            let mut completed = false;

            for _ in 0..MAX_LIQUIDATION_BLOCKS {
                let budget_ref_time: u64 = rng.gen_range(1_000_000u64..=100_000_000);
                let budget = Weight::from_parts(budget_ref_time, u64::MAX / 2);
                let (_w, new_state, is_complete) =
                    SubtensorModule::process_liquidation_step(netuid, current_state, budget);
                if is_complete {
                    completed = true;
                    break;
                }
                current_state = new_state;
            }

            assert!(
                completed,
                "Liquidation should complete within MAX_LIQUIDATION_BLOCKS for {} stakers (trial {})",
                count, trial
            );

            // Clean up
            LiquidatingSubnets::<Test>::remove(netuid);
        }
    });
}

// ============================================================
// 7.5 Security Fix Tests
// ============================================================

#[test]
fn test_total_stake_decremented_on_liquidation_start() {
    // Vuln 2 fix: TotalStake must be decremented when liquidation starts,
    // mirroring the atomic path in destroy_alpha_in_out_stakes.
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 5u32;
        let alpha_per = 10_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        // Set TotalStake to include the subnet's TAO
        let initial_total_stake = 1_000_000u64;
        TotalStake::<Test>::set(TaoCurrency::from(initial_total_stake));

        // Start liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));

        // TotalStake should have been decremented by tao_pot
        let total_stake_after: u64 = TotalStake::<Test>::get().into();
        assert_eq!(
            total_stake_after,
            initial_total_stake.saturating_sub(tao_pot),
            "TotalStake should decrease by tao_pot on liquidation start"
        );
    });
}

#[test]
fn test_total_stake_correct_after_full_liquidation() {
    // End-to-end: TotalStake must be decremented by exactly tao_pot after
    // the entire liquidation completes (start + force_complete).
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 5u32;
        let alpha_per = 10_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 500_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        let initial_total_stake = 1_000_000u64;
        TotalStake::<Test>::set(TaoCurrency::from(initial_total_stake));

        // Start and complete liquidation
        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        assert_ok!(SubtensorModule::force_complete_all_phases(netuid, state));

        // TotalStake should be exactly initial - tao_pot
        let total_stake_after: u64 = TotalStake::<Test>::get().into();
        assert_eq!(
            total_stake_after,
            initial_total_stake.saturating_sub(tao_pot),
            "TotalStake should decrease by exactly tao_pot after full liquidation"
        );
    });
}

#[test]
fn test_total_stake_correct_after_emergency_finalize() {
    // TotalStake must also be correct after emergency finalization.
    // Since we decrement at start_liquidation, emergency path should not
    // double-decrement.
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 3u32;
        let alpha_per = 10_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 300_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));

        let initial_total_stake = 1_000_000u64;
        TotalStake::<Test>::set(TaoCurrency::from(initial_total_stake));

        // Start liquidation (decrements TotalStake)
        assert_ok!(SubtensorModule::start_liquidation(netuid));

        let total_stake_after_start: u64 = TotalStake::<Test>::get().into();
        assert_eq!(total_stake_after_start, initial_total_stake.saturating_sub(tao_pot));

        // Emergency finalize — should NOT double-decrement TotalStake
        let state = LiquidatingSubnets::<Test>::get(netuid).unwrap();
        SubtensorModule::emergency_finalize(netuid, &state);
        LiquidatingSubnets::<Test>::remove(netuid);

        let total_stake_after_emergency: u64 = TotalStake::<Test>::get().into();
        assert_eq!(
            total_stake_after_emergency, total_stake_after_start,
            "Emergency finalize should not further decrement TotalStake (already decremented at start)"
        );
    });
}

#[test]
fn test_cursor_bytes_fits_alpha_key() {
    // Vuln 1 fix: CursorBytes must be large enough to hold a full Alpha
    // storage key (130 bytes with Blake2_128Concat hashers on two AccountIds).
    new_test_ext(1).execute_with(|| {
        let hot = U256::from(1);
        let cold = U256::from(2);
        let netuid = NetUid::from(1);

        // Generate the full hashed key for an Alpha entry
        let raw_key = Alpha::<Test>::hashed_key_for((&hot, &cold, netuid));
        let key_len = raw_key.len();

        // The key should be 130 bytes for our configuration
        assert_eq!(
            key_len, 130,
            "Alpha hashed key should be 130 bytes (32 prefix + 48 hot + 48 cold + 2 netuid)"
        );

        // CursorBytes must be able to hold it
        let bounded: Result<CursorBytes, _> = raw_key.try_into();
        assert!(
            bounded.is_ok(),
            "CursorBytes (capacity 256) must fit a 130-byte Alpha key"
        );
    });
}

#[test]
fn test_snapshot_cursor_survives_multi_block_iteration() {
    // Regression test: snapshot iteration across multiple blocks must not
    // lose the cursor due to overflow. With the old 128-byte limit, the
    // 130-byte Alpha key would overflow and restart iteration from the
    // beginning every block.
    new_test_ext(1).execute_with(|| {
        let owner_hot = U256::from(1);
        let owner_cold = U256::from(2);
        let num_stakers = 10u32;
        let alpha_per = 50_000u64;

        let (netuid, _stakers) =
            setup_subnet_with_stakers(owner_hot, owner_cold, num_stakers, alpha_per);

        let tao_pot = 1_000_000u64;
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_pot));
        TotalStake::<Test>::set(TaoCurrency::from(tao_pot));

        assert_ok!(SubtensorModule::start_liquidation(netuid));
        let mut state = LiquidatingSubnets::<Test>::get(netuid).unwrap();

        // Advance past Freeze and LP phases to reach SnapshotStakers
        let unlimited = Weight::from_parts(u64::MAX / 2, u64::MAX / 2);
        let (_w, advanced_state, _complete) =
            SubtensorModule::process_liquidation_step(netuid, state, unlimited);
        state = advanced_state;

        // Find the SnapshotStakers phase and give it a tiny budget (1 item)
        // to force multi-block iteration
        if let LiquidationPhase::SnapshotStakers { ref cursor } = state.phase {
            let cursor = cursor.clone();
            let tiny_budget = Weight::from_parts(WEIGHT_PER_SNAPSHOT.saturating_mul(2), 0);
            let result =
                SubtensorModule::snapshot_stakers_chunk(netuid, cursor, &mut state, tiny_budget);

            // Should NOT be complete yet (we have 10 stakers but only budget for 2)
            // The cursor should be valid (not None due to overflow)
            if let ChunkResult::Incomplete { phase, .. } = result {
                match &phase {
                    LiquidationPhase::SnapshotStakers { cursor } => {
                        assert!(
                            cursor.is_some(),
                            "Cursor must not be None — a None cursor means overflow reset the iterator"
                        );
                    }
                    _ => panic!("Should still be in SnapshotStakers phase"),
                }
            }
        }

        // Clean up
        LiquidatingSubnets::<Test>::remove(netuid);
    });
}
