#![allow(clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use super::mock::*;
use crate::*;
use alloc::collections::BTreeMap;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;
use sp_core::U256;
use substrate_fixed::types::U96F32;
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
// Test 1: Override force suppress → zero TAO emission, rest gets full share
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

        let block_emission = U96F32::from_num(1_000_000);
        let emissions =
            SubtensorModule::get_subnet_block_emissions(&[sn1, sn2], block_emission);

        // sn1 gets zero TAO emission.
        let sn1_emission = emissions.get(&sn1).copied().unwrap_or(U96F32::from_num(0));
        assert_eq!(sn1_emission, U96F32::from_num(0));

        // sn2 gets the full block emission.
        let sn2_emission = emissions.get(&sn2).copied().unwrap_or(U96F32::from_num(0));
        assert!(
            sn2_emission > U96F32::from_num(999_000),
            "sn2 should get ~full emission, got {sn2_emission:?}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: Override=Some(false) → not suppressed (same as None, reserved for future use)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_override_force_unsuppress() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Some(false) is accepted but is currently identical to None: not suppressed.
        EmissionSuppressionOverride::<Test>::insert(sn1, false);

        assert!(
            !SubtensorModule::is_subnet_emission_suppressed(sn1),
            "Some(false) should not suppress"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: No override → not suppressed (default)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_no_override_not_suppressed() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // No override at all — default is not suppressed.
        assert!(
            !SubtensorModule::is_subnet_emission_suppressed(sn1),
            "no override means not suppressed"
        );
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
// Test 5: 3 subnets, suppress 1 → suppressed gets 0, others split full emission
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_suppress_one_of_three() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        let sn3 = NetUid::from(3);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 200_000_000);
        setup_subnet_with_flow(sn3, 10, 300_000_000);

        // Suppress sn2 via override.
        EmissionSuppressionOverride::<Test>::insert(sn2, true);

        let block_emission = U96F32::from_num(1_000_000);
        let emissions =
            SubtensorModule::get_subnet_block_emissions(&[sn1, sn2, sn3], block_emission);

        // sn2 should get 0 TAO.
        let sn2_emission = emissions.get(&sn2).copied().unwrap_or(U96F32::from_num(0));
        assert_eq!(sn2_emission, U96F32::from_num(0));

        // sn1 + sn3 should get the full block emission.
        let sn1_emission: u64 = emissions
            .get(&sn1)
            .copied()
            .unwrap_or(U96F32::from_num(0))
            .saturating_to_num();
        let sn3_emission: u64 = emissions
            .get(&sn3)
            .copied()
            .unwrap_or(U96F32::from_num(0))
            .saturating_to_num();
        let total = sn1_emission.saturating_add(sn3_emission);
        assert!(
            total >= 999_000,
            "sn1 + sn3 should get ~full emission, got {total}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 6: All subnets suppressed → zero TAO emissions
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

        // Total TAO emission via get_subnet_block_emissions should be zero.
        let emissions =
            SubtensorModule::get_subnet_block_emissions(&[sn1, sn2], U96F32::from_num(1_000_000));
        let total: u64 = emissions
            .values()
            .map(|e| e.saturating_to_num::<u64>())
            .fold(0u64, |a, b| a.saturating_add(b));
        assert_eq!(total, 0, "all-suppressed should yield zero TAO emission");
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

// ─────────────────────────────────────────────────────────────────────────────
// Test 9: Non-root origin is rejected with BadOrigin
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_sudo_override_rejects_non_root_origin() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        let non_root_account = U256::from(42);

        assert_noop!(
            SubtensorModule::sudo_set_emission_suppression_override(
                RuntimeOrigin::signed(non_root_account),
                sn1,
                Some(true),
            ),
            DispatchError::BadOrigin
        );

        // Storage must remain untouched.
        assert_eq!(EmissionSuppressionOverride::<Test>::get(sn1), None);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 10: Non-existent subnet is rejected with SubnetNotExists
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_sudo_override_rejects_nonexistent_subnet() {
    new_test_ext(1).execute_with(|| {
        let missing_netuid = NetUid::from(99);
        // Deliberately do not create subnet 99.

        assert_noop!(
            SubtensorModule::sudo_set_emission_suppression_override(
                RuntimeOrigin::root(),
                missing_netuid,
                Some(true),
            ),
            Error::<Test>::SubnetNotExists
        );

        // Storage must remain untouched.
        assert_eq!(EmissionSuppressionOverride::<Test>::get(missing_netuid), None);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 11: Root subnet (netuid 0) is rejected with CannotSuppressRootSubnet
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_sudo_override_rejects_root_subnet() {
    new_test_ext(1).execute_with(|| {
        // Register the root network so it passes the SubnetNotExists check and
        // the CannotSuppressRootSubnet guard is reached.
        add_network(NetUid::ROOT, 1, 0);

        assert_noop!(
            SubtensorModule::sudo_set_emission_suppression_override(
                RuntimeOrigin::root(),
                NetUid::ROOT,
                Some(true),
            ),
            Error::<Test>::CannotSuppressRootSubnet
        );

        // Storage must remain untouched.
        assert_eq!(EmissionSuppressionOverride::<Test>::get(NetUid::ROOT), None);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 12: Clearing the override via None removes the storage entry
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_sudo_override_clear_removes_storage() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Set Some(true) first.
        assert_ok!(SubtensorModule::sudo_set_emission_suppression_override(
            RuntimeOrigin::root(),
            sn1,
            Some(true),
        ));
        assert_eq!(EmissionSuppressionOverride::<Test>::get(sn1), Some(true));

        // Clear via None — the storage entry must be removed entirely.
        assert_ok!(SubtensorModule::sudo_set_emission_suppression_override(
            RuntimeOrigin::root(),
            sn1,
            None,
        ));
        assert_eq!(
            EmissionSuppressionOverride::<Test>::get(sn1),
            None,
            "storage entry should be absent after clearing with None"
        );

        // With the override gone the subnet should no longer be suppressed.
        assert!(
            !SubtensorModule::is_subnet_emission_suppressed(sn1),
            "subnet should not be suppressed after override is cleared"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: Suppressed subnet still receives alpha emissions (TAO-only suppression)
//
// Emission suppression zeroes the TAO share but alpha issuance is independent
// (driven by the subnet's own halving curve). This is intentional: suppressed
// subnets continue to mint and distribute alpha to miners, validators, owner,
// and root validators — only TAO injection into the AMM pool is suppressed.
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_suppressed_subnet_still_receives_alpha_emissions() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Register a root validator so root_proportion > 0.
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

        // Zero out all pending accumulators so we can measure what gets added.
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);
        PendingServerEmission::<Test>::insert(sn1, AlphaCurrency::ZERO);
        PendingValidatorEmission::<Test>::insert(sn1, AlphaCurrency::ZERO);

        let alpha_out_before = SubnetAlphaOut::<Test>::get(sn1);

        // Build emission map: suppressed subnet gets zero TAO share.
        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(0));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // --- Alpha issuance is independent of TAO emission ---

        // SubnetAlphaOut must have grown (new alpha was minted).
        let alpha_out_after = SubnetAlphaOut::<Test>::get(sn1);
        assert!(
            alpha_out_after > alpha_out_before,
            "suppressed subnet must still mint alpha: before={alpha_out_before:?} after={alpha_out_after:?}"
        );

        // Miners received pending alpha.
        let pending_server = PendingServerEmission::<Test>::get(sn1);
        assert!(
            pending_server > AlphaCurrency::ZERO,
            "miners must receive alpha on suppressed subnet"
        );

        // Validators received pending alpha.
        let pending_validator = PendingValidatorEmission::<Test>::get(sn1);
        assert!(
            pending_validator > AlphaCurrency::ZERO,
            "validators must receive alpha on suppressed subnet"
        );

        // Root validators received pending alpha.
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert!(
            pending_root > AlphaCurrency::ZERO,
            "root must receive alpha on suppressed subnet"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test: end-to-end run_coinbase with one suppressed dynamic subnet
//
// Integration test that calls the full run_coinbase pipeline with two dynamic
// subnets.  One is suppressed via EmissionSuppressionOverride before the call.
// After run_coinbase the suppressed subnet must have received zero TAO
// (SubnetTAO unchanged, SubnetTaoInEmission == 0) while the active subnet
// absorbs the entire block emission.
// ─────────────────────────────────────────────────────────────────────────────
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib \
//   -- tests::emission_suppression::test_run_coinbase_suppressed_subnet_gets_zero_tao \
//   --exact --show-output --nocapture
#[test]
fn test_run_coinbase_suppressed_subnet_gets_zero_tao() {
    new_test_ext(1).execute_with(|| {
        // --- Setup: two dynamic subnets with AMM pools seeded with reserves.
        let subnet_owner_hk = U256::from(100);
        let subnet_owner_ck = U256::from(101);

        // sn_active receives emission; sn_suppressed is blocked.
        let sn_active = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);
        let sn_suppressed = add_dynamic_network(&subnet_owner_hk, &subnet_owner_ck);

        // Seed AMM pools: 100_000 TAO, 1_000_000 alpha (price ≈ 0.1 TAO/alpha).
        let initial_tao: u64 = 100_000_u64;
        let initial_alpha: u64 = 1_000_000_u64;
        // setup_reserves is pub(crate) and glob-imported from super::mock::*.
        setup_reserves(sn_active, initial_tao.into(), initial_alpha.into());
        setup_reserves(sn_suppressed, initial_tao.into(), initial_alpha.into());

        // Initialise swap-engine positions for both subnets with a zero-value swap
        // so the pool is ready to price TAO emissions.
        SubtensorModule::swap_tao_for_alpha(
            sn_active,
            TaoCurrency::ZERO,
            1_000_000_000_000_u64.into(),
            false,
        )
        .ok();
        SubtensorModule::swap_tao_for_alpha(
            sn_suppressed,
            TaoCurrency::ZERO,
            1_000_000_000_u64.into(),
            false,
        )
        .ok();

        // Mark both subnets as dynamic (mechanism index 1).
        SubnetMechanism::<Test>::insert(sn_active, 1u16);
        SubnetMechanism::<Test>::insert(sn_suppressed, 1u16);

        // Give both subnets equal, positive TAO flow so they would normally split
        // the block emission 50/50 without suppression.
        SubnetTaoFlow::<Test>::insert(sn_active, 100_000_000_i64);
        SubnetTaoFlow::<Test>::insert(sn_suppressed, 100_000_000_i64);

        // Snapshot state before the coinbase run.
        let tao_active_before = SubnetTAO::<Test>::get(sn_active);
        let tao_suppressed_before = SubnetTAO::<Test>::get(sn_suppressed);
        let total_issuance_before = TotalIssuance::<Test>::get();

        // --- Act: suppress sn_suppressed, then run the complete coinbase pipeline.
        EmissionSuppressionOverride::<Test>::insert(sn_suppressed, true);

        let block_emission: u64 = 1_000_000_000_u64; // 1 TAO in planck units
        SubtensorModule::run_coinbase(U96F32::from_num(block_emission));

        // --- Assert 1: suppressed subnet received no direct TAO injection.
        //
        // SubnetTAO is mutated only by the direct `tao_in` injection path inside
        // inject_and_maybe_swap.  For a suppressed subnet get_subnet_terms inserts
        // zero for tao_in, so SubnetTAO must be unchanged.
        let tao_suppressed_after = SubnetTAO::<Test>::get(sn_suppressed);
        assert_eq!(
            tao_suppressed_after,
            tao_suppressed_before,
            "suppressed subnet SubnetTAO must not change: before={:?} after={:?}",
            tao_suppressed_before,
            tao_suppressed_after,
        );

        // --- Assert 2: per-block TAO emission record for suppressed subnet is zero.
        //
        // SubnetTaoInEmission is written by inject_and_maybe_swap for every subnet
        // in the emit-to list; for the suppressed subnet it must be zero.
        let tao_in_emission_suppressed = SubnetTaoInEmission::<Test>::get(sn_suppressed);
        assert_eq!(
            tao_in_emission_suppressed,
            TaoCurrency::ZERO,
            "SubnetTaoInEmission for suppressed subnet must be zero, got {:?}",
            tao_in_emission_suppressed,
        );

        // --- Assert 3: active subnet received positive TAO injection.
        //
        // All of the emission share (100 %) went to sn_active, so its SubnetTAO
        // must have grown.  The exact amount is price-capped (the AMM splits the
        // block emission into a direct injection and an excess-TAO swap), but the
        // direct injection must be strictly positive.
        let tao_active_after = SubnetTAO::<Test>::get(sn_active);
        assert!(
            tao_active_after > tao_active_before,
            "active subnet must receive TAO emission: before={:?} after={:?}",
            tao_active_before,
            tao_active_after,
        );

        // --- Assert 4: the full block emission appears in TotalIssuance.
        //
        // TotalIssuance is incremented by both tao_in AND excess_tao for every
        // emitting subnet (see inject_and_maybe_swap).  Since the suppressed subnet
        // contributes neither, the entire block_emission must flow to sn_active and
        // therefore into TotalIssuance.  A tolerance of 2 planck is allowed for
        // fixed-point rounding in the U96F32 arithmetic used by the emission pipeline.
        let total_issuance_after = TotalIssuance::<Test>::get();
        let issuance_delta =
            u64::from(total_issuance_after).saturating_sub(u64::from(total_issuance_before));
        let rounding_tolerance: u64 = 2;
        let undershoot = block_emission.saturating_sub(issuance_delta);
        let overshoot = issuance_delta.saturating_sub(block_emission);
        assert!(
            undershoot <= rounding_tolerance && overshoot <= rounding_tolerance,
            "TotalIssuance must grow by ~block_emission (±{rounding_tolerance} planck): \
             got {issuance_delta}, expected {block_emission}",
        );
    });
}
