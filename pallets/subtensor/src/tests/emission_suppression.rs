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
// Test 7: Suppress subnet, Enable mode → root still gets alpha
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_suppressed_subnet_root_alpha_by_default() {
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

        // Default mode is Recycle; verify that, then set to Enable for this test.
        assert_eq!(
            KeepRootSellPressureOnSuppressedSubnets::<Test>::get(),
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        );
        KeepRootSellPressureOnSuppressedSubnets::<Test>::put(
            RootSellPressureOnSuppressedSubnetsMode::Enable,
        );

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
            "with Enable mode, root should still get alpha on suppressed subnet"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 8: Suppress subnet, Disable mode → root gets no alpha, validators get more
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_suppressed_subnet_no_root_alpha_flag_off() {
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
        SubtensorModule::set_tao_weight(u64::MAX);
        setup_root_with_tao(sn1);

        // Force-suppress sn1.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        // Set mode to Disable: no root sell pressure on suppressed subnets.
        KeepRootSellPressureOnSuppressedSubnets::<Test>::put(
            RootSellPressureOnSuppressedSubnetsMode::Disable,
        );

        // Clear any pending emissions.
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);
        PendingValidatorEmission::<Test>::insert(sn1, AlphaCurrency::ZERO);

        // Build emission map.
        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // Root should get NO alpha.
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert_eq!(
            pending_root,
            AlphaCurrency::ZERO,
            "with Disable mode, root should get no alpha on suppressed subnet"
        );

        // Validator emission should be non-zero (root alpha recycled to validators).
        let pending_validator = PendingValidatorEmission::<Test>::get(sn1);
        assert!(
            pending_validator > AlphaCurrency::ZERO,
            "validators should receive recycled root alpha"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 9: Disable mode actually recycles root alpha to validators
// (validators get more than with Enable mode)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_disable_mode_recycles_root_alpha_to_validators() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

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
        SubtensorModule::set_tao_weight(u64::MAX);
        setup_root_with_tao(sn1);

        // Force-suppress sn1.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        // ── Run with Enable mode first to get baseline ──
        KeepRootSellPressureOnSuppressedSubnets::<Test>::put(
            RootSellPressureOnSuppressedSubnetsMode::Enable,
        );
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);
        PendingValidatorEmission::<Test>::insert(sn1, AlphaCurrency::ZERO);

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        let enable_validator = PendingValidatorEmission::<Test>::get(sn1);
        let enable_root = PendingRootAlphaDivs::<Test>::get(sn1);

        // In Enable mode, root should accumulate some alpha.
        assert!(
            enable_root > AlphaCurrency::ZERO,
            "Enable mode: root should get alpha"
        );

        // ── Now run with Disable mode ──
        KeepRootSellPressureOnSuppressedSubnets::<Test>::put(
            RootSellPressureOnSuppressedSubnetsMode::Disable,
        );
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);
        PendingValidatorEmission::<Test>::insert(sn1, AlphaCurrency::ZERO);

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        let disable_validator = PendingValidatorEmission::<Test>::get(sn1);
        let disable_root = PendingRootAlphaDivs::<Test>::get(sn1);

        // In Disable mode, root should get nothing.
        assert_eq!(
            disable_root,
            AlphaCurrency::ZERO,
            "Disable mode: root should get no alpha"
        );

        // Disable validators should get MORE than Enable validators because
        // root alpha is recycled to them instead of going to root.
        assert!(
            disable_validator > enable_validator,
            "Disable mode validators ({disable_validator:?}) should get more \
             than Enable mode ({enable_validator:?}) because root alpha is recycled"
        );

        // The difference should equal the root alpha from Enable mode
        // (root alpha is recycled to validators instead).
        assert_eq!(
            disable_validator.saturating_sub(enable_validator),
            enable_root,
            "difference should equal the root alpha that was recycled"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 10: Non-suppressed subnet → root alpha normal regardless of mode
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_unsuppressed_subnet_unaffected_by_flag() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

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
        SubtensorModule::set_tao_weight(u64::MAX);
        setup_root_with_tao(sn1);

        // sn1 is NOT suppressed.
        // Set mode to Disable (should not matter for unsuppressed subnets).
        KeepRootSellPressureOnSuppressedSubnets::<Test>::put(
            RootSellPressureOnSuppressedSubnetsMode::Disable,
        );

        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);

        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // Root should still get alpha since subnet is not suppressed.
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert!(
            pending_root > AlphaCurrency::ZERO,
            "non-suppressed subnet should still give root alpha regardless of mode"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 11: sudo_set_emission_suppression_override emits event
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

// ─────────────────────────────────────────────────────────────────────────────
// Test 12: sudo_set_root_sell_pressure_on_suppressed_subnets_mode emits event
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_sudo_sell_pressure_emits_event() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(1);
        System::reset_events();

        assert_ok!(
            SubtensorModule::sudo_set_root_sell_pressure_on_suppressed_subnets_mode(
                RuntimeOrigin::root(),
                RootSellPressureOnSuppressedSubnetsMode::Disable,
            )
        );

        assert!(
            System::events().iter().any(|e| {
                matches!(
                    &e.event,
                    RuntimeEvent::SubtensorModule(
                        Event::RootSellPressureOnSuppressedSubnetsModeSet { mode }
                    ) if *mode == RootSellPressureOnSuppressedSubnetsMode::Disable
                )
            }),
            "should emit RootSellPressureOnSuppressedSubnetsModeSet event"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 13: Default mode is Recycle
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_default_mode_is_recycle() {
    new_test_ext(1).execute_with(|| {
        assert_eq!(
            KeepRootSellPressureOnSuppressedSubnets::<Test>::get(),
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 14: Recycle mode, suppressed subnet → root alpha swapped to TAO via
//          AMM, then TAO recycled (removed from TotalIssuance).
//
// The full flow is:
//   1. Root alpha that would go to root validators is instead sold into the
//      subnet's AMM pool (alpha in, TAO out).
//   2. The TAO received from the swap is recycled via `recycle_tao`, which
//      decreases TotalIssuance (TAO is permanently removed from circulation).
//
// We verify every step:
//   - PendingRootAlphaDivs stays 0 (root did NOT accumulate alpha).
//   - SubnetAlphaIn increases (alpha entered the pool via the swap).
//   - SubnetTAO decreases (TAO left the pool via the swap).
//   - TotalIssuance decreases by exactly the TAO that left the pool
//     (proving that TAO was recycled, not sent elsewhere).
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_recycle_mode_suppressed_subnet_swaps_and_recycles() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        // Use add_dynamic_network to properly initialize the AMM.
        let owner_hk = U256::from(50);
        let owner_ck = U256::from(51);
        let sn1 = add_dynamic_network(&owner_hk, &owner_ck);

        // Seed the pool with TAO and alpha reserves.
        let initial_alpha_in = AlphaCurrency::from(500_000_000u64);
        SubnetTAO::<Test>::insert(sn1, TaoCurrency::from(500_000_000u64));
        SubnetAlphaIn::<Test>::insert(sn1, initial_alpha_in);
        SubnetTaoFlow::<Test>::insert(sn1, 100_000_000i64);

        // Also set root TAO so root_proportion is nonzero.
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000_000));
        SubnetAlphaOut::<Test>::insert(sn1, AlphaCurrency::from(1_000_000_000));

        // Register a root validator.
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
        SubtensorModule::set_tao_weight(u64::MAX);

        // Force-suppress sn1.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        // Default mode is Recycle.
        assert_eq!(
            KeepRootSellPressureOnSuppressedSubnets::<Test>::get(),
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        );

        // Clear pending.
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);

        // Snapshot state before emission.
        // Note: emit_to_subnets calls inject_and_maybe_swap first which adds TAO
        // to the pool, so we snapshot SubnetTAO *after* a dry run would inject.
        // Instead we record TotalIssuance and SubnetTAO, and check relative changes.
        let issuance_before = TotalIssuance::<Test>::get();
        let subnet_tao_before = SubnetTAO::<Test>::get(sn1);

        // Build emission map.
        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // 1. Root did NOT accumulate alpha — it was recycled instead.
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert_eq!(
            pending_root,
            AlphaCurrency::ZERO,
            "Recycle mode: PendingRootAlphaDivs must be 0"
        );

        // 2. Alpha entered the pool (swap sold alpha into AMM).
        let alpha_in_after = SubnetAlphaIn::<Test>::get(sn1);
        assert!(
            alpha_in_after > initial_alpha_in,
            "Recycle mode: SubnetAlphaIn must increase (alpha entered pool via swap)"
        );

        // 3. TAO left the pool (AMM paid out TAO for the alpha).
        //    emit_to_subnets also injects TAO via inject_and_maybe_swap, so
        //    SubnetTAO may have increased from that injection first; but the
        //    net SubnetTaoFlow being negative (checked in test 18) proves
        //    the swap outflow dominated. Here we check the pool TAO decreased
        //    relative to where it started before both inject + swap.
        let subnet_tao_after = SubnetTAO::<Test>::get(sn1);
        assert!(
            subnet_tao_after < subnet_tao_before,
            "Recycle mode: SubnetTAO must decrease (TAO left pool via swap), \
             before={subnet_tao_before:?} after={subnet_tao_after:?}"
        );

        // 4. The TAO that left the pool was recycled (removed from TotalIssuance).
        //    The issuance drop should equal the TAO that left the subnet pool.
        let issuance_after = TotalIssuance::<Test>::get();
        let tao_recycled = issuance_before.saturating_sub(issuance_after);
        let tao_left_pool = subnet_tao_before.saturating_sub(subnet_tao_after);
        assert!(
            tao_recycled > TaoCurrency::ZERO,
            "Recycle mode: TotalIssuance must decrease (TAO was recycled)"
        );
        assert_eq!(
            tao_recycled, tao_left_pool,
            "Recycle mode: TotalIssuance drop ({tao_recycled:?}) must equal TAO that \
             left the pool ({tao_left_pool:?}) — all swap proceeds were recycled"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 15: Recycle mode on non-suppressed subnet → normal PendingRootAlphaDivs
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_recycle_mode_non_suppressed_subnet_normal() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

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
        SubtensorModule::set_tao_weight(u64::MAX);
        setup_root_with_tao(sn1);

        // sn1 is NOT suppressed. Mode is Recycle (default).
        assert_eq!(
            KeepRootSellPressureOnSuppressedSubnets::<Test>::get(),
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        );

        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);

        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // Root should still get alpha — Recycle only affects suppressed subnets.
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert!(
            pending_root > AlphaCurrency::ZERO,
            "non-suppressed subnet should still give root alpha in Recycle mode"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 16: Recycle mode ignores RootClaimType (alpha never enters claim flow)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_recycle_mode_ignores_root_claim_type() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        // Use add_dynamic_network to properly initialize the AMM.
        let owner_hk = U256::from(50);
        let owner_ck = U256::from(51);
        let sn1 = add_dynamic_network(&owner_hk, &owner_ck);

        SubnetTAO::<Test>::insert(sn1, TaoCurrency::from(500_000_000u64));
        SubnetAlphaIn::<Test>::insert(sn1, AlphaCurrency::from(500_000_000u64));
        SubnetTaoFlow::<Test>::insert(sn1, 100_000_000i64);
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000_000));
        SubnetAlphaOut::<Test>::insert(sn1, AlphaCurrency::from(1_000_000_000));

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
        SubtensorModule::set_tao_weight(u64::MAX);

        // Force-suppress sn1.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        // Set RootClaimType to Keep — in normal flow this would keep alpha.
        // But Recycle mode should override and swap+recycle regardless.
        RootClaimType::<Test>::insert(coldkey, RootClaimTypeEnum::Keep);

        // Default mode is Recycle.
        assert_eq!(
            KeepRootSellPressureOnSuppressedSubnets::<Test>::get(),
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        );

        let subnet_tao_before = SubnetTAO::<Test>::get(sn1);

        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);

        let issuance_before = TotalIssuance::<Test>::get();

        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // PendingRootAlphaDivs should still be 0 (recycled, not claimed).
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert_eq!(
            pending_root,
            AlphaCurrency::ZERO,
            "Recycle mode should swap+recycle regardless of RootClaimType"
        );

        // TAO was recycled (removed from circulation).
        let issuance_after = TotalIssuance::<Test>::get();
        let subnet_tao_after = SubnetTAO::<Test>::get(sn1);
        let tao_recycled = issuance_before.saturating_sub(issuance_after);
        let tao_left_pool = subnet_tao_before.saturating_sub(subnet_tao_after);
        assert!(
            tao_recycled > TaoCurrency::ZERO,
            "TotalIssuance must decrease even with RootClaimType::Keep"
        );
        assert_eq!(
            tao_recycled, tao_left_pool,
            "all TAO from the swap must be recycled (removed from TotalIssuance)"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 17: sudo_set_mode all 3 variants emit events
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_sudo_set_mode_all_variants_emit_events() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(1);

        for mode in [
            RootSellPressureOnSuppressedSubnetsMode::Disable,
            RootSellPressureOnSuppressedSubnetsMode::Enable,
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        ] {
            System::reset_events();

            assert_ok!(
                SubtensorModule::sudo_set_root_sell_pressure_on_suppressed_subnets_mode(
                    RuntimeOrigin::root(),
                    mode,
                )
            );

            assert_eq!(KeepRootSellPressureOnSuppressedSubnets::<Test>::get(), mode,);

            assert!(
                System::events().iter().any(|e| {
                    matches!(
                        &e.event,
                        RuntimeEvent::SubtensorModule(
                            Event::RootSellPressureOnSuppressedSubnetsModeSet { mode: m }
                        ) if *m == mode
                    )
                }),
                "should emit RootSellPressureOnSuppressedSubnetsModeSet for {mode:?}"
            );
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 18: Recycle mode decreases price and flow EMA; Disable/Enable do not
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_recycle_mode_decreases_price_and_flow_ema() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        // Use add_dynamic_network to properly initialize the AMM.
        let owner_hk = U256::from(50);
        let owner_ck = U256::from(51);
        let sn1 = add_dynamic_network(&owner_hk, &owner_ck);

        // Large pool reserves to ensure swaps produce measurable effects.
        let pool_reserve = 1_000_000_000u64;
        SubnetTAO::<Test>::insert(sn1, TaoCurrency::from(pool_reserve));
        SubnetAlphaIn::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(pool_reserve));
        SubnetAlphaOut::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));
        SubnetTaoFlow::<Test>::insert(sn1, 100_000_000i64);

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
        SubtensorModule::set_tao_weight(u64::MAX);

        // Force-suppress sn1.
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        let emission_amount = U96F32::from_num(10_000_000);
        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, emission_amount);

        // ── First: verify that Disable and Enable modes do NOT cause TAO outflow ──

        for mode in [
            RootSellPressureOnSuppressedSubnetsMode::Disable,
            RootSellPressureOnSuppressedSubnetsMode::Enable,
        ] {
            // Reset pool state.
            SubnetTAO::<Test>::insert(sn1, TaoCurrency::from(pool_reserve));
            SubnetAlphaIn::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));
            SubnetTaoFlow::<Test>::insert(sn1, 0i64);
            PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);
            SubnetAlphaOut::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));

            KeepRootSellPressureOnSuppressedSubnets::<Test>::put(mode);

            SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

            let flow = SubnetTaoFlow::<Test>::get(sn1);
            assert!(
                flow >= 0,
                "mode {mode:?}: SubnetTaoFlow should not be negative, got {flow}"
            );
        }

        // ── Now: verify that Recycle mode DOES cause TAO outflow ──

        // Reset pool state.
        SubnetTAO::<Test>::insert(sn1, TaoCurrency::from(pool_reserve));
        SubnetAlphaIn::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));
        SubnetTaoFlow::<Test>::insert(sn1, 0i64);
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);
        SubnetAlphaOut::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));

        // Set Recycle mode.
        KeepRootSellPressureOnSuppressedSubnets::<Test>::put(
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        );

        // Record TAO reserve before.
        let tao_before = SubnetTAO::<Test>::get(sn1);

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // SubnetTaoFlow should be negative (TAO left the pool via swap).
        let flow_after = SubnetTaoFlow::<Test>::get(sn1);
        assert!(
            flow_after < 0,
            "Recycle mode: SubnetTaoFlow should be negative (TAO outflow), got {flow_after}"
        );

        // SubnetTAO should have decreased (TAO left the pool in the swap).
        // Note: emit_to_subnets injects some TAO via inject_and_maybe_swap,
        // but the swap_alpha_for_tao pulls TAO back out. The net flow recorded
        // as negative proves outflow dominated.
        let tao_after = SubnetTAO::<Test>::get(sn1);
        assert!(
            tao_after < tao_before,
            "Recycle mode: SubnetTAO should decrease (TAO outflow), before={tao_before:?} after={tao_after:?}"
        );
    });
}
