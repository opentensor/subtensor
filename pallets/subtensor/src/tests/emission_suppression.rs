#![allow(clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use super::mock::*;
use crate::*;
use alloc::collections::BTreeMap;
use frame_support::{assert_err, assert_ok};
use sp_core::U256;
use substrate_fixed::types::{U64F64, U96F32};
use subtensor_runtime_common::{AlphaCurrency, NetUid, TaoCurrency};

/// Helper: set up root network + register a hotkey on root with given stake.
/// Returns (hotkey, coldkey).
fn setup_root_validator(hotkey_seed: u64, coldkey_seed: u64, root_stake: u64) -> (U256, U256) {
    let hotkey = U256::from(hotkey_seed);
    let coldkey = U256::from(coldkey_seed);
    assert_ok!(SubtensorModule::root_register(
        RuntimeOrigin::signed(coldkey),
        hotkey,
    ));
    SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
        &hotkey,
        &coldkey,
        NetUid::ROOT,
        root_stake.into(),
    );
    (hotkey, coldkey)
}

/// Helper: create a non-root subnet with TAO flow so it gets shares.
fn setup_subnet_with_flow(netuid: NetUid, tempo: u16, tao_flow: i64) {
    add_network(netuid, tempo, 0);
    SubnetTaoFlow::<Test>::insert(netuid, tao_flow);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: >50% stake votes suppress → share=0, rest renormalized
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_suppression_zeroes_share_majority() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // Directly set suppression > 0.5 for sn1.
        EmissionSuppression::<Test>::insert(sn1, U64F64::from_num(0.6));

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        SubtensorModule::apply_emission_suppression(&mut shares);

        // sn1 should be zeroed.
        assert_eq!(
            shares.get(&sn1).copied().unwrap_or(U64F64::from_num(0)),
            U64F64::from_num(0)
        );
        // sn2 should get the full share (renormalized to 1.0).
        let sn2_share = shares.get(&sn2).copied().unwrap_or(U64F64::from_num(0));
        assert!(
            sn2_share > U64F64::from_num(0.99),
            "sn2 share should be ~1.0, got {sn2_share:?}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: <50% stake votes suppress → share unchanged
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_suppression_no_effect_below_half() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // Set suppression <= 0.5 for sn1.
        EmissionSuppression::<Test>::insert(sn1, U64F64::from_num(0.4));

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        let shares_before = shares.clone();
        SubtensorModule::apply_emission_suppression(&mut shares);

        // Both shares should be unchanged.
        assert_eq!(shares, shares_before);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: Root override=Some(true), no votes → suppressed
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_override_force_suppress() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // No votes, suppression is 0. But override forces suppression.
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
// Test 4: Majority votes suppress, override=Some(false) → not suppressed
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_override_force_unsuppress() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // Set high suppression fraction.
        EmissionSuppression::<Test>::insert(sn1, U64F64::from_num(0.9));
        // But override forces unsuppression.
        EmissionSuppressionOverride::<Test>::insert(sn1, false);

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        let shares_before = shares.clone();
        SubtensorModule::apply_emission_suppression(&mut shares);

        // Shares should be unchanged (not suppressed).
        assert_eq!(shares, shares_before);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: Override=None, votes determine outcome
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_override_none_uses_votes() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // No override (default).
        // Set suppression > 0.5.
        EmissionSuppression::<Test>::insert(sn1, U64F64::from_num(0.7));

        let mut shares = SubtensorModule::get_shares(&[sn1, sn2]);
        SubtensorModule::apply_emission_suppression(&mut shares);

        assert_eq!(
            shares.get(&sn1).copied().unwrap_or(U64F64::from_num(0)),
            U64F64::from_num(0)
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 6: Non-root validator → error
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_vote_requires_root_registration() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // coldkey with no root-registered hotkey.
        let coldkey = U256::from(999);

        assert_err!(
            SubtensorModule::vote_emission_suppression(
                RuntimeOrigin::signed(coldkey),
                sn1,
                Some(true),
            ),
            Error::<Test>::NotEnoughStakeToVote
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 7: Below threshold → error
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_vote_requires_minimum_stake() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Set a non-zero stake threshold.
        StakeThreshold::<Test>::put(1_000_000);

        let hotkey = U256::from(10);
        let coldkey = U256::from(11);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(coldkey),
            hotkey,
        ));
        // Stake below threshold.
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            999_999u64.into(),
        );

        assert_err!(
            SubtensorModule::vote_emission_suppression(
                RuntimeOrigin::signed(coldkey),
                sn1,
                Some(true),
            ),
            Error::<Test>::NotEnoughStakeToVote
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 8: Vote then clear (None) → suppression drops
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_vote_clear() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        let (_hotkey, coldkey) = setup_root_validator(10, 11, 1_000_000);

        // Vote to suppress.
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(coldkey),
            sn1,
            Some(true),
        ));
        assert_eq!(
            EmissionSuppressionVote::<Test>::get(sn1, coldkey),
            Some(true)
        );

        // Clear vote.
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(coldkey),
            sn1,
            None,
        ));
        assert_eq!(EmissionSuppressionVote::<Test>::get(sn1, coldkey), None);

        // Collect votes - should result in 0 suppression.
        SubtensorModule::collect_emission_suppression_votes(sn1);
        let suppression = EmissionSuppression::<Test>::get(sn1);
        assert_eq!(suppression, U64F64::from_num(0));
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 9: Suppression only updates on epoch
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_votes_collected_on_epoch() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        let (_hotkey, coldkey) = setup_root_validator(10, 11, 1_000_000);

        // Vote to suppress.
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(coldkey),
            sn1,
            Some(true),
        ));

        // Before epoch, suppression should still be 0 (default).
        assert_eq!(EmissionSuppression::<Test>::get(sn1), U64F64::from_num(0));

        // Run epochs so vote collection occurs.
        step_epochs(1, sn1);

        // After epoch, suppression should be updated.
        let suppression = EmissionSuppression::<Test>::get(sn1);
        assert!(
            suppression > U64F64::from_num(0),
            "suppression should be > 0 after epoch, got {suppression:?}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 10: Swap coldkey → votes follow
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_coldkey_swap_migrates_votes() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        let (_hotkey, old_coldkey) = setup_root_validator(10, 11, 1_000_000);

        // Vote to suppress.
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(old_coldkey),
            sn1,
            Some(true),
        ));
        assert_eq!(
            EmissionSuppressionVote::<Test>::get(sn1, old_coldkey),
            Some(true)
        );

        // Perform coldkey swap.
        let new_coldkey = U256::from(999);
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey));

        // Vote should be on new coldkey.
        assert_eq!(
            EmissionSuppressionVote::<Test>::get(sn1, new_coldkey),
            Some(true)
        );
        // Old coldkey should have no vote.
        assert_eq!(EmissionSuppressionVote::<Test>::get(sn1, old_coldkey), None);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 11: Dissolve subnet → votes + suppression cleaned
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_dissolution_clears_all() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        let (_hotkey, coldkey) = setup_root_validator(10, 11, 1_000_000);

        // Vote and set suppression.
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(coldkey),
            sn1,
            Some(true),
        ));
        EmissionSuppression::<Test>::insert(sn1, U64F64::from_num(0.8));
        EmissionSuppressionOverride::<Test>::insert(sn1, true);

        // Remove the network.
        SubtensorModule::remove_network(sn1);

        // Everything should be cleaned up.
        assert_eq!(EmissionSuppression::<Test>::get(sn1), U64F64::from_num(0));
        assert_eq!(EmissionSuppressionOverride::<Test>::get(sn1), None);
        assert_eq!(EmissionSuppressionVote::<Test>::get(sn1, coldkey), None);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 12: 3 subnets, suppress 1 → others sum to 1.0
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

        // Suppress sn2.
        EmissionSuppression::<Test>::insert(sn2, U64F64::from_num(0.9));

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
            "shares should sum to 1.0, got {sum_f64}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 13: Extra unstaked TAO → no effect on suppression fraction
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_unstaked_tao_not_in_denominator() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Two root validators: one votes suppress, one doesn't.
        let (_hk1, ck1) = setup_root_validator(10, 11, 1_000_000);
        let (_hk2, _ck2) = setup_root_validator(20, 21, 1_000_000);

        // Only ck1 votes to suppress.
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(ck1),
            sn1,
            Some(true),
        ));

        // Collect votes.
        SubtensorModule::collect_emission_suppression_votes(sn1);

        // Suppression should be 0.5 (1M / 2M).
        let suppression: f64 = EmissionSuppression::<Test>::get(sn1).to_num();
        assert!(
            (suppression - 0.5).abs() < 1e-6,
            "suppression should be 0.5, got {suppression}"
        );

        // Adding free balance (unstaked TAO) to some account should NOT affect denominator.
        let random_account = U256::from(999);
        SubtensorModule::add_balance_to_coldkey_account(&random_account, 100_000_000_000);

        // Re-collect.
        SubtensorModule::collect_emission_suppression_votes(sn1);
        let suppression2: f64 = EmissionSuppression::<Test>::get(sn1).to_num();
        assert!(
            (suppression2 - 0.5).abs() < 1e-6,
            "suppression should still be 0.5 after adding unstaked TAO, got {suppression2}"
        );
    });
}

/// Helper: set up root + subnet with proper SubnetTAO and alpha issuance
/// so that root_proportion returns a meaningful nonzero value.
fn setup_root_with_tao(sn: NetUid) {
    // Set SubnetTAO for root so root_proportion numerator is nonzero.
    SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000_000));
    // Set alpha issuance for subnet so denominator is meaningful.
    SubnetAlphaOut::<Test>::insert(sn, AlphaCurrency::from(1_000_000_000));
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 14: Suppress subnet, Enable mode → root still gets alpha
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
// Test 15: Suppress subnet, Disable mode → root gets no alpha
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

        // But validator emission should be non-zero (all alpha goes to validators).
        let pending_validator = PendingValidatorEmission::<Test>::get(sn1);
        assert!(
            pending_validator > AlphaCurrency::ZERO,
            "validators should receive all alpha when root alpha is zeroed"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 16: Non-suppressed subnet → root alpha normal regardless of mode
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
// Test 17: Voting on root subnet returns CannotVoteOnRootSubnet
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_vote_on_root_subnet_rejected() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        let (_hk, ck) = setup_root_validator(10, 11, 1_000_000);

        assert_err!(
            SubtensorModule::vote_emission_suppression(
                RuntimeOrigin::signed(ck),
                NetUid::ROOT,
                Some(true),
            ),
            Error::<Test>::CannotVoteOnRootSubnet
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 18: Some(false) vote is stored and treated as no-suppress weight
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_vote_explicit_false() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Single root validator votes Some(false).
        let (_hk, ck) = setup_root_validator(10, 11, 1_000_000);

        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(ck),
            sn1,
            Some(false),
        ));
        assert_eq!(EmissionSuppressionVote::<Test>::get(sn1, ck), Some(false));

        // Collect votes: sole validator voted false → suppression should be 0.
        SubtensorModule::collect_emission_suppression_votes(sn1);
        let suppression: f64 = EmissionSuppression::<Test>::get(sn1).to_num();
        assert!(
            suppression.abs() < 1e-9,
            "explicit false vote should produce 0 suppression, got {suppression}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 19: All subnets suppressed → all zeroed, no panic
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_all_subnets_suppressed() {
    new_test_ext(1).execute_with(|| {
        let sn1 = NetUid::from(1);
        let sn2 = NetUid::from(2);
        setup_subnet_with_flow(sn1, 10, 100_000_000);
        setup_subnet_with_flow(sn2, 10, 100_000_000);

        // Suppress both.
        EmissionSuppression::<Test>::insert(sn1, U64F64::from_num(0.9));
        EmissionSuppression::<Test>::insert(sn2, U64F64::from_num(0.8));

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
// Test 20: Coldkey swap blocked by existing votes on destination
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_coldkey_swap_blocked_by_existing_votes() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Set up old coldkey with a vote.
        let (_hk_old, old_ck) = setup_root_validator(10, 11, 1_000_000);
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(old_ck),
            sn1,
            Some(true),
        ));

        // Set up new coldkey that already has a vote via direct storage insert.
        let new_ck = U256::from(999);
        EmissionSuppressionVote::<Test>::insert(sn1, new_ck, false);

        // Swap should fail.
        assert_err!(
            SubtensorModule::do_swap_coldkey(&old_ck, &new_ck),
            Error::<Test>::DestinationColdkeyHasExistingVotes
        );

        // Old coldkey's vote should still be intact (no partial state change).
        assert_eq!(
            EmissionSuppressionVote::<Test>::get(sn1, old_ck),
            Some(true)
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 21: Coldkey with multiple root hotkeys → vote weight = sum of stakes
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_multi_hotkey_coldkey_vote_weight() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        let coldkey = U256::from(100);
        let hk1 = U256::from(1);
        let hk2 = U256::from(2);
        let hk3 = U256::from(3);

        // Register all 3 hotkeys on root under the same coldkey.
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(coldkey),
            hk1,
        ));
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(coldkey),
            hk2,
        ));
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(coldkey),
            hk3,
        ));

        // Stake: hk1=100, hk2=200, hk3=300 → total root stake = 600.
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hk1,
            &coldkey,
            NetUid::ROOT,
            100u64.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hk2,
            &coldkey,
            NetUid::ROOT,
            200u64.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hk3,
            &coldkey,
            NetUid::ROOT,
            300u64.into(),
        );

        // Vote to suppress.
        assert_ok!(SubtensorModule::vote_emission_suppression(
            RuntimeOrigin::signed(coldkey),
            sn1,
            Some(true),
        ));

        // Collect votes. Only coldkey's hotkeys exist on root,
        // and all stakes belong to the suppressing coldkey.
        SubtensorModule::collect_emission_suppression_votes(sn1);

        // Suppression should be 1.0 (all stake voted suppress).
        let suppression: f64 = EmissionSuppression::<Test>::get(sn1).to_num();
        assert!(
            (suppression - 1.0).abs() < 1e-6,
            "suppression should be 1.0 when all root stake votes suppress, got {suppression}"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 22: sudo_set_emission_suppression_override emits event
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
// Test 23: sudo_set_root_sell_pressure_on_suppressed_subnets_mode emits event
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
// Test 24: collect_emission_suppression_votes(ROOT) is a no-op
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_collect_votes_skips_root() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);

        // Ensure no EmissionSuppression entry for ROOT.
        assert_eq!(
            EmissionSuppression::<Test>::get(NetUid::ROOT),
            U64F64::from_num(0)
        );

        // Call collect on ROOT — should be a no-op.
        SubtensorModule::collect_emission_suppression_votes(NetUid::ROOT);

        // Still no entry.
        assert_eq!(
            EmissionSuppression::<Test>::get(NetUid::ROOT),
            U64F64::from_num(0)
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 25: default mode is Recycle
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
// Test 26: Recycle mode, suppressed subnet → alpha swapped to TAO, TAO burned
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_recycle_mode_suppressed_subnet_swaps_and_recycles() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Make it a dynamic subnet so swap_alpha_for_tao actually works via AMM.
        SubnetMechanism::<Test>::insert(sn1, 1);

        // Seed the pool with TAO and alpha reserves.
        let initial_tao = TaoCurrency::from(500_000_000u64);
        let initial_alpha_in = AlphaCurrency::from(500_000_000u64);
        SubnetTAO::<Test>::insert(sn1, initial_tao);
        SubnetAlphaIn::<Test>::insert(sn1, initial_alpha_in);

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

        // Record TotalIssuance before emission.
        let issuance_before = TotalIssuance::<Test>::get();

        // Clear pending.
        PendingRootAlphaDivs::<Test>::insert(sn1, AlphaCurrency::ZERO);

        // Build emission map.
        let mut subnet_emissions = BTreeMap::new();
        subnet_emissions.insert(sn1, U96F32::from_num(1_000_000));

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // PendingRootAlphaDivs should be 0 (root did NOT accumulate alpha).
        let pending_root = PendingRootAlphaDivs::<Test>::get(sn1);
        assert_eq!(
            pending_root,
            AlphaCurrency::ZERO,
            "in Recycle mode, PendingRootAlphaDivs should be 0"
        );

        // SubnetAlphaIn should have increased (alpha was swapped into pool).
        let alpha_in_after = SubnetAlphaIn::<Test>::get(sn1);
        assert!(
            alpha_in_after > initial_alpha_in,
            "SubnetAlphaIn should increase after swap"
        );

        // TotalIssuance should have decreased (TAO was recycled/burned).
        let issuance_after = TotalIssuance::<Test>::get();
        assert!(
            issuance_after < issuance_before,
            "TotalIssuance should decrease (TAO recycled)"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 27: Recycle mode on non-suppressed subnet → normal PendingRootAlphaDivs
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
// Test 28: Recycle mode ignores RootClaimType (alpha never enters claim flow)
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_recycle_mode_ignores_root_claim_type() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Dynamic subnet for AMM swap.
        SubnetMechanism::<Test>::insert(sn1, 1);
        SubnetTAO::<Test>::insert(sn1, TaoCurrency::from(500_000_000u64));
        SubnetAlphaIn::<Test>::insert(sn1, AlphaCurrency::from(500_000_000u64));
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
        // But Recycle mode should override and swap+burn regardless.
        RootClaimType::<Test>::insert(coldkey, RootClaimTypeEnum::Keep);

        // Default mode is Recycle.
        assert_eq!(
            KeepRootSellPressureOnSuppressedSubnets::<Test>::get(),
            RootSellPressureOnSuppressedSubnetsMode::Recycle,
        );

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
            "Recycle mode should swap+burn regardless of RootClaimType"
        );

        // TAO was burned.
        let issuance_after = TotalIssuance::<Test>::get();
        assert!(
            issuance_after < issuance_before,
            "TotalIssuance should decrease even with RootClaimType::Keep"
        );
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 29: sudo_set_mode all 3 variants emit events
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

            assert_eq!(
                KeepRootSellPressureOnSuppressedSubnets::<Test>::get(),
                mode,
            );

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
// Test 30: Recycle mode decreases price and flow EMA; Disable/Enable do not
// ─────────────────────────────────────────────────────────────────────────────
#[test]
fn test_recycle_mode_decreases_price_and_flow_ema() {
    new_test_ext(1).execute_with(|| {
        add_network(NetUid::ROOT, 1, 0);
        let sn1 = NetUid::from(1);
        setup_subnet_with_flow(sn1, 10, 100_000_000);

        // Dynamic subnet.
        SubnetMechanism::<Test>::insert(sn1, 1);

        // Large pool reserves to ensure swaps produce measurable effects.
        let pool_reserve = 1_000_000_000u64;
        SubnetTAO::<Test>::insert(sn1, TaoCurrency::from(pool_reserve));
        SubnetAlphaIn::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(pool_reserve));
        SubnetAlphaOut::<Test>::insert(sn1, AlphaCurrency::from(pool_reserve));

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

        // Record price before.
        let price_before = SubnetMovingPrice::<Test>::get(sn1);

        SubtensorModule::emit_to_subnets(&[sn1], &subnet_emissions, true);

        // SubnetTaoFlow should be negative (TAO left the pool via swap).
        let flow_after = SubnetTaoFlow::<Test>::get(sn1);
        assert!(
            flow_after < 0,
            "Recycle mode: SubnetTaoFlow should be negative (TAO outflow), got {flow_after}"
        );

        // Moving price should have decreased (alpha was sold into pool for TAO).
        let price_after = SubnetMovingPrice::<Test>::get(sn1);
        assert!(
            price_after < price_before,
            "Recycle mode: SubnetMovingPrice should decrease, before={price_before:?} after={price_after:?}"
        );
    });
}
