#![allow(clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use super::mock::*;
use crate::*;
use alloc::collections::BTreeMap;
use frame_support::assert_ok;
use sp_core::U256;
use substrate_fixed::types::{U64F64, U96F32};
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

/// Helper: create a non-root subnet with TAO flow so it gets shares.
fn setup_subnet_with_flow(netuid: NetUid, tempo: u16, tao_flow: i64) {
    add_network(netuid, tempo, 0);
    SubnetTaoFlow::<Test>::insert(netuid, tao_flow);
}

/// Helper: seed root + subnet TAO/alpha so root_proportion is nonzero.
fn setup_root_with_tao(sn: NetUid) {
    // Set SubnetTAO for root so root_proportion numerator is nonzero.
    SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000_000));
    // Set alpha issuance for subnet so denominator is meaningful.
    SubnetAlphaOut::<Test>::insert(sn, AlphaCurrency::from(1_000_000_000));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: Override force suppress → share=0, rest renormalized
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_override_force_suppress() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // Override forces suppression.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        SubtensorModule::apply_emission_suppression(&mut shares);

        assert_eq!(
            shares.get(&sn1).copied().unwrap_or(U64F64::from_num(0)),
            U64F64::from_num(0)
        );
        let sn2_share = shares.get(&sn2).copied().unwrap_or(U64F64::from_num(0));
        assert!(
            sn2_share > U64F64::from_num(0.99),
            "sn2 share should be ~1.0, got {sn2_share:?}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: Override=Some(false) → not suppressed
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_override_force_unsuppress() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // Override forces unsuppression.
        EmissionSuppressionOverride::<Test>::insert(sn1, false);

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        let shares_before = shares.clone();
        SubtensorModule::apply_emission_suppression(&mut shares);

        // Shares should be unchanged (not suppressed).
        assert_eq!(shares, shares_before);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: No override → not suppressed (default)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_no_override_not_suppressed() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // No override at all — default is not suppressed.
        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        let shares_before = shares.clone();
        SubtensorModule::apply_emission_suppression(&mut shares);

        // Shares should be unchanged.
        assert_eq!(shares, shares_before);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4: Dissolution clears override
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_dissolution_clears_override() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        // Remove the network.
        SubtensorModule::remove_network(sn1);

        // Override should be cleaned up.
        assert_eq!(EmissionSuppressionOverride::<Test>::get(sn1), None);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: 3 subnets, suppress 1 → others sum to 1.0
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_shares_renormalize() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        let sn3 = NetUid::from(3);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 200_000_000);
        setup_subnet_with_flow(sn3, 10, 300_000_000);

        // Suppress sn2 via override.
        EmissionSuppressionOverride::<Test>::insert(sn2, true);

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2, sn3]);
        SubtensorModule::apply_emission_suppression(&mut shares);

        // sn2 should be 0.
        assert_eq!(
            shares.get(&sn2).copied().unwrap_or(U64F64::from_num(0)),
            U64F64::from_num(0)
        );

        // Remaining shares should sum to ~1.0.
        let sum: U64F64 = shares
            .values()
            .copied()
            .fold(U64F64::from_num(0), |a, b| a.saturating_add(b));
        let sum_f64: f64 = sum.to_num();
        assert!(
            (sum_f64 - 1.0).abs() < 1e-9,
            "remaining shares should sum to ~1.0, got {sum_f64}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 6: All subnets suppressed → all shares 0, zero emissions
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_all_subnets_suppressed() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // Suppress both via override.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);
        EmissionSuppressionOverride::<Test>::insert(sn2, true);

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        SubtensorModule::apply_emission_suppression(&mut shares);

        // Both should be zero.
        let s1 = shares.get(&sn1).copied().unwrap_or(U64F64::from_num(0));
        let s2 = shares.get(&sn2).copied().unwrap_or(U64F64::from_num(0));
        assert_eq!(s1, U64F64::from_num(0));
        assert_eq!(s2, U64F64::from_num(0));

        // Total emission via get_subnet_block_emissions should be zero.
        let emissions =
            SubtensorModule::get_subnet_block_emissions(&[sn1, sn2], U96F32::from_num(1_000_000));
        let total: u64 = emissions
            .values()
            .map(|e| e.saturating_to_num::<u64>())
            .fold(0u64, |a, b| a.saturating_add(b));
        assert_eq!(total, 0, "all-suppressed should yield zero total emission");
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 7: Suppressed subnet → root still accumulates alpha (hardcoded behavior)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_suppressed_subnet_root_alpha_accumulated() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Register a root validator and add stake on root so root_proportion > 0.
        let hotkey = U256::from(10);
        let coldkey = U256::from(11);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(coldkey),
            hotkey,
        ));
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            1_000_000_000u64.into(),
        );
        // Set TAO weight so root_proportion is nonzero.
        SubtensorModule::set_tao_weight(u64::MAX);
        setup_root_with_tao(sn1);

        // Force-suppress sn1.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        // Clear any pending emissions.
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);

        // Build emission map with some emission for sn1.
        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // Root should have received some alpha (pending root alpha divs > 0).
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert!(
            pending_root > AlphaCurrency::ZERO,
            "root should still get alpha on suppressed subnet"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 8: sudo_set_emission_suppression_override emits event
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_sudo_override_emits_event() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        System::set_block_number(1);
        System::reset_events();

        assert_ok!(SubtensorModule::sudo_set_emission_suppression_override(
            RuntimeOrigin::root(),
            sn1,
            Some(true),
        ));

        assert!(
            System::events().iter().any(|e| {
                matches!(
                    &e.event,
                    RuntimeEvent::SubtensorModule(
                        Event::EmissionSuppressionOverrideSet { netuid, override_value }
                    ) if *netuid == sn1 && *override_value == Some(true)
                )
            }),
            "should emit EmissionSuppressionOverrideSet event"
        );
    });
}
