#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::tests::mock::*;
use crate::{
    BasketPrincipal, DefaultMinRootClaimAmount, Error, Keys, MAX_NUM_ROOT_CLAIMS,
    MAX_ROOT_CLAIM_THRESHOLD, NetworksAdded, NumRootClaim, NumStakingColdkeys, RootClaimType,
    RootClaimTypeEnum, RootClaimable, RootClaimableThreshold, RootClaimed, RootWeightDelegate,
    RootWeightTake, StakingColdkeys, StakingColdkeysByIndex, SubnetAlphaIn, SubnetMovingPrice,
    SubnetProtocolFlow, SubnetTAO, SubnetworkN, Tempo, TotalStake, Uids, Weights,
};
use approx::assert_abs_diff_eq;
use frame_support::dispatch::RawOrigin;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::Get;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::U256;
use sp_runtime::DispatchError;
use std::collections::BTreeSet;
use substrate_fixed::types::I96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, NetUidStorageIndex, TaoBalance, Token};

// =============================================================================
// Helpers
// =============================================================================

/// Directly assign a root UID and a beta-basket weight vector `w` to a validator hotkey,
/// bypassing the `set_root_weights` extrinsic's validation (which is exercised separately).
/// `dests` are `(subnet, weight)` pairs.
fn set_root_weights_direct(hotkey: &U256, uid: u16, dests: &[(NetUid, u16)]) {
    Uids::<Test>::insert(NetUid::ROOT, hotkey, uid);
    let zipped: Vec<(u16, u16)> = dests.iter().map(|(n, w)| (u16::from(*n), *w)).collect();
    Weights::<Test>::insert(NetUidStorageIndex::ROOT, uid, zipped);
}

/// Ensure a subnet has deep, balanced AMM reserves so basket swaps execute with negligible
/// slippage and never fail for lack of liquidity.
fn fund_pool(netuid: NetUid) {
    SubnetTAO::<Test>::insert(netuid, TaoBalance::from(1_000_000_000_000u64));
    SubnetAlphaIn::<Test>::insert(netuid, AlphaBalance::from(1_000_000_000_000u64));
}

fn escrow_alpha(hotkey: &U256, netuid: NetUid) -> u64 {
    let escrow = SubtensorModule::get_beta_escrow_account_id();
    SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &escrow, netuid).to_u64()
}

fn root_stake_of(hotkey: &U256, coldkey: &U256) -> u64 {
    SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, NetUid::ROOT)
        .to_u64()
}

// =============================================================================
// Still-valid utility tests (independent of the beta-basket accrual mechanics)
// =============================================================================

#[test]
fn test_claim_root_set_claim_type() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);

        // Swap is the only supported claim type.
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Swap
        ));
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Swap);

        // Keep / KeepSubnets are deprecated no-ops and are rejected so a caller can never set a
        // claim type that silently does nothing.
        assert_noop!(
            SubtensorModule::set_root_claim_type(
                RuntimeOrigin::signed(coldkey),
                RootClaimTypeEnum::Keep
            ),
            Error::<Test>::RootClaimTypeNotSupported
        );
        assert_noop!(
            SubtensorModule::set_root_claim_type(
                RuntimeOrigin::signed(coldkey),
                RootClaimTypeEnum::KeepSubnets {
                    subnets: BTreeSet::from([NetUid::from(1)])
                }
            ),
            Error::<Test>::RootClaimTypeNotSupported
        );
    });
}

#[test]
fn test_populate_staking_maps() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1000);
        let coldkey1 = U256::from(1001);
        let coldkey2 = U256::from(1002);
        let coldkey3 = U256::from(1003);
        let hotkey = U256::from(1004);
        let _netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        let netuid2 = NetUid::from(2);

        let root_stake = 200_000_000u64;
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey1,
            NetUid::ROOT,
            root_stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey2,
            NetUid::ROOT,
            root_stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey3,
            netuid2,
            root_stake.into(),
        );

        assert_eq!(NumStakingColdkeys::<Test>::get(), 0);

        // Populate maps through block step
        run_to_block(2);

        assert_eq!(NumStakingColdkeys::<Test>::get(), 2);

        assert!(StakingColdkeysByIndex::<Test>::contains_key(0));
        assert!(StakingColdkeysByIndex::<Test>::contains_key(1));

        assert!(StakingColdkeys::<Test>::contains_key(coldkey1));
        assert!(StakingColdkeys::<Test>::contains_key(coldkey2));
        assert!(!StakingColdkeys::<Test>::contains_key(coldkey3));
    });
}

#[test]
fn test_sudo_set_num_root_claims() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1003);

        assert_noop!(
            SubtensorModule::sudo_set_num_root_claims(RuntimeOrigin::signed(coldkey), 50u64),
            DispatchError::BadOrigin
        );

        assert_noop!(
            SubtensorModule::sudo_set_num_root_claims(
                RuntimeOrigin::root(),
                MAX_NUM_ROOT_CLAIMS + 1,
            ),
            Error::<Test>::InvalidNumRootClaim
        );

        let new_value = 27u64;
        assert_ok!(SubtensorModule::sudo_set_num_root_claims(
            RuntimeOrigin::root(),
            new_value,
        ),);

        assert_eq!(NumRootClaim::<Test>::get(), new_value);
    });
}

#[test]
fn test_claim_root_threshold() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        assert_eq!(
            RootClaimableThreshold::<Test>::get(netuid),
            DefaultMinRootClaimAmount::<Test>::get()
        );

        let threshold = 1000u64;
        assert_ok!(SubtensorModule::sudo_set_root_claim_threshold(
            RawOrigin::Root.into(),
            netuid,
            threshold
        ));
        assert_eq!(
            RootClaimableThreshold::<Test>::get(netuid),
            I96F32::from(threshold)
        );

        let threshold = 2000u64;
        assert_ok!(SubtensorModule::sudo_set_root_claim_threshold(
            RawOrigin::Signed(owner_coldkey).into(),
            netuid,
            threshold
        ));
        assert_eq!(
            RootClaimableThreshold::<Test>::get(netuid),
            I96F32::from(threshold)
        );

        // Errors
        assert_err!(
            SubtensorModule::sudo_set_root_claim_threshold(
                RawOrigin::Signed(hotkey).into(),
                netuid,
                threshold
            ),
            DispatchError::BadOrigin,
        );

        assert_err!(
            SubtensorModule::sudo_set_root_claim_threshold(
                RawOrigin::Signed(owner_coldkey).into(),
                netuid,
                MAX_ROOT_CLAIM_THRESHOLD + 1
            ),
            Error::<Test>::InvalidRootClaimThreshold,
        );
    });
}

#[test]
fn test_claim_root_subnet_limits() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1003);

        assert_err!(
            SubtensorModule::claim_root(RuntimeOrigin::signed(coldkey), BTreeSet::new()),
            Error::<Test>::InvalidSubnetNumber
        );

        assert_err!(
            SubtensorModule::claim_root(
                RuntimeOrigin::signed(coldkey),
                BTreeSet::from_iter((0u16..=10u16).map(NetUid::from))
            ),
            Error::<Test>::InvalidSubnetNumber
        );
    });
}

// =============================================================================
// Beta basket: setting weights (extrinsic validation)
// =============================================================================

#[test]
fn test_set_root_weights_rejects_unregistered_hotkey() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        // `hotkey` is not registered on the root subnet, so it cannot set root weights.
        assert_noop!(
            SubtensorModule::set_root_weights(
                RuntimeOrigin::signed(hotkey),
                vec![u16::from(netuid)],
                vec![u16::MAX],
                0,
            ),
            Error::<Test>::HotKeyNotRegisteredInSubNet
        );
    });
}

// =============================================================================
// Beta basket: accrual
// =============================================================================

#[test]
fn test_root_basket_accrues_per_weights() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX); // tao_weight = 1.0

        let root_stake = 2_000_000u64;
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );

        // Route the basket 100% back into this subnet.
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        assert_eq!(escrow_alpha(&hotkey, netuid), 0);
        assert_eq!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)), 0);

        let pending_root_alpha = 1_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            pending_root_alpha.into(),
            AlphaBalance::ZERO,
        );

        // Basket principal recorded, escrow holds the basket alpha, and a claimable rate exists.
        assert!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)) > 0);
        assert!(escrow_alpha(&hotkey, netuid) > 0);
        assert!(RootClaimable::<Test>::get(hotkey).contains_key(&netuid));

        // Escrow value and recorded principal should match (E/P starts at 1).
        assert_abs_diff_eq!(
            escrow_alpha(&hotkey, netuid),
            u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)),
            epsilon = 10u64,
        );
    });
}

#[test]
fn test_root_basket_recycles_without_weights() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );

        // No root weights set for the validator.
        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        // Without weights the root dividend is recycled: no basket, no claimable.
        assert_eq!(escrow_alpha(&hotkey, netuid), 0);
        assert_eq!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)), 0);
        assert!(!RootClaimable::<Test>::get(hotkey).contains_key(&netuid));
    });
}

#[test]
fn test_root_basket_routes_to_target_subnet() {
    new_test_ext(1).execute_with(|| {
        let owner_a = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let owner_b = U256::from(2001);
        let hotkey_b = U256::from(2002);

        let netuid_a = add_dynamic_network(&hotkey, &owner_a);
        let netuid_b = add_dynamic_network(&hotkey_b, &owner_b);
        remove_owner_registration_stake(netuid_a);
        fund_pool(netuid_a);
        fund_pool(netuid_b);

        SubtensorModule::set_tao_weight(u64::MAX);

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_a,
            netuid_a,
            10_000_000u64.into(),
        );

        // Route the basket entirely into subnet B (different from the dividend origin A).
        set_root_weights_direct(&hotkey, 0, &[(netuid_b, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid_a,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        // Basket should be on B, not A.
        assert!(escrow_alpha(&hotkey, netuid_b) > 0);
        assert_eq!(escrow_alpha(&hotkey, netuid_a), 0);
        assert!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid_b)) > 0);
        assert_eq!(
            u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid_a)),
            0
        );
        assert!(RootClaimable::<Test>::get(hotkey).contains_key(&netuid_b));
        assert!(!RootClaimable::<Test>::get(hotkey).contains_key(&netuid_a));
    });
}

// =============================================================================
// Beta basket: protocol-flow accounting (symmetric)
// =============================================================================

/// The basket must book protocol flow symmetrically: the origin sell on A is an outflow, each
/// redistribution buy on B/C is an inflow, and the claim sell on B/C is an outflow that nets the
/// deposit-then-claim round-trip back toward zero on the dest pools.
#[test]
fn test_root_basket_records_symmetric_protocol_flow() {
    new_test_ext(1).execute_with(|| {
        let owner_a = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let owner_b = U256::from(2001);
        let hotkey_b = U256::from(2002);
        let owner_c = U256::from(3001);
        let hotkey_c = U256::from(3002);

        let netuid_a = add_dynamic_network(&hotkey, &owner_a);
        let netuid_b = add_dynamic_network(&hotkey_b, &owner_b);
        let netuid_c = add_dynamic_network(&hotkey_c, &owner_c);
        remove_owner_registration_stake(netuid_a);
        fund_pool(netuid_a);
        fund_pool(netuid_b);
        fund_pool(netuid_c);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid_b, I96F32::from_num(0));
        RootClaimableThreshold::<Test>::insert(netuid_c, I96F32::from_num(0));

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_a,
            netuid_a,
            10_000_000u64.into(),
        );

        // Split the basket 50/50 across B and C (neither is the dividend origin A).
        set_root_weights_direct(&hotkey, 0, &[(netuid_b, u16::MAX), (netuid_c, u16::MAX)]);

        // No protocol flow has been recorded on any subnet yet.
        assert_eq!(SubnetProtocolFlow::<Test>::get(netuid_a), 0);
        assert_eq!(SubnetProtocolFlow::<Test>::get(netuid_b), 0);
        assert_eq!(SubnetProtocolFlow::<Test>::get(netuid_c), 0);

        SubtensorModule::distribute_emission(
            netuid_a,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let flow_a = SubnetProtocolFlow::<Test>::get(netuid_a);
        let flow_b = SubnetProtocolFlow::<Test>::get(netuid_b);
        let flow_c = SubnetProtocolFlow::<Test>::get(netuid_c);

        // Origin sell on A is booked as an outflow (negative); the buys on B and C as inflows.
        assert!(flow_a < 0, "origin sell must be an outflow, got {flow_a}");
        assert!(flow_b > 0, "buy on B must be an inflow, got {flow_b}");
        assert!(flow_c > 0, "buy on C must be an inflow, got {flow_c}");

        // Symmetry: every TAO sold on A is spent buying on B and C, so the inflows exactly offset
        // the outflow across subnets.
        assert_abs_diff_eq!(flow_b + flow_c, -flow_a, epsilon = 10i64);

        // Now redeem the basket on B and C. The claim sells alpha back to TAO, booking an outflow
        // on each dest that nets the round-trip back toward zero.
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid_b, netuid_c])
        ));

        let flow_b_after = SubnetProtocolFlow::<Test>::get(netuid_b);
        let flow_c_after = SubnetProtocolFlow::<Test>::get(netuid_c);

        // Claim recorded an outflow: the dest flow decreased, and the deposit+claim round-trip
        // leaves a residual far smaller than the original inflow (only swap fees/slippage remain).
        assert!(
            flow_b_after < flow_b,
            "claim must book an outflow on B: {flow_b_after} !< {flow_b}"
        );
        assert!(
            flow_c_after < flow_c,
            "claim must book an outflow on C: {flow_c_after} !< {flow_c}"
        );
        assert!(
            flow_b_after.abs() < flow_b,
            "round-trip residual on B should be smaller than the inflow: {flow_b_after} vs {flow_b}"
        );
        assert!(
            flow_c_after.abs() < flow_c,
            "round-trip residual on C should be smaller than the inflow: {flow_c_after} vs {flow_c}"
        );
    });
}

/// A root validator (the delegator) that sets no vector of its own but points at a manager via
/// `RootWeightDelegate` has its dividend deployed per the *manager's* vector, and the manager's
/// `RootWeightTake` is skimmed as a curation fee credited to the manager's own root stake.
#[test]
fn test_root_weight_delegation_uses_manager_vector_and_pays_take() {
    new_test_ext(1).execute_with(|| {
        // Manager (curator): owns the shared vector + take. Delegator: earns the dividend.
        let owner_m = U256::from(1001);
        let hotkey_m = U256::from(1002);
        let owner_a = U256::from(2001);
        let hotkey_a = U256::from(2002);
        let coldkey_a = U256::from(2003);
        let owner_b = U256::from(3001);
        let hotkey_b = U256::from(3002);

        // netuid_a = where the delegator earns its root dividend; netuid_b = manager's basket pick.
        let netuid_a = add_dynamic_network(&hotkey_a, &owner_a);
        let netuid_b = add_dynamic_network(&hotkey_b, &owner_b);
        // Give the manager an Owner mapping (for the fee credit) via a throwaway registration.
        add_dynamic_network(&hotkey_m, &owner_m);
        remove_owner_registration_stake(netuid_a);
        fund_pool(netuid_a);
        fund_pool(netuid_b);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid_b, I96F32::from_num(0));

        // Manager root uid 0 with a vector pointing 100% at B; Keys[ROOT][0] for the fee credit.
        set_root_weights_direct(&hotkey_m, 0, &[(netuid_b, u16::MAX)]);
        Keys::<Test>::insert(NetUid::ROOT, 0u16, hotkey_m);

        // Delegator root uid 1 with NO vector of its own — it relies on the manager.
        Uids::<Test>::insert(NetUid::ROOT, hotkey_a, 1u16);
        RootWeightDelegate::<Test>::insert(1u16, 0u16); // delegator -> manager
        RootWeightTake::<Test>::insert(0u16, 1000u16); // manager charges 10%

        // Delegator needs root stake to apportion against, and alpha on A to earn the dividend.
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_a,
            &coldkey_a,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_a,
            &owner_a,
            netuid_a,
            10_000_000u64.into(),
        );

        // Manager starts with no root stake — the fee must create it.
        assert_eq!(root_stake_of(&hotkey_m, &owner_m), 0);

        SubtensorModule::distribute_emission(
            netuid_a,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        // The delegator's basket was built on B (the manager's pick), not on A or nowhere.
        let basket_b = escrow_alpha(&hotkey_a, netuid_b);
        assert!(basket_b > 0, "delegator basket should follow manager's vector into B");
        assert_eq!(
            escrow_alpha(&hotkey_a, netuid_a),
            0,
            "nothing should land on the origin subnet"
        );

        // The manager received the curation fee as its own root stake.
        let fee = root_stake_of(&hotkey_m, &owner_m);
        assert!(fee > 0, "manager should receive the curation take");

        // The split is take-proportioned and independent of any validator take: with a 10% fee,
        // the basket (~90% of gross, at pool price ~1) is ~9x the fee.
        assert_abs_diff_eq!(basket_b as i64, fee as i64 * 9, epsilon = (fee as i64) / 20);

        // With the take cleared, no fee would be charged — sanity that the fee is driven by the
        // take alone.
        RootWeightTake::<Test>::insert(0u16, 0u16);
        let fee_before = root_stake_of(&hotkey_m, &owner_m);
        SubtensorModule::distribute_emission(
            netuid_a,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );
        assert_eq!(
            root_stake_of(&hotkey_m, &owner_m),
            fee_before,
            "no take => no fee on the second distribution"
        );
    });
}

// =============================================================================
// Beta basket: claiming (always full swap to root TAO)
// =============================================================================

#[test]
fn test_root_basket_claim_swaps_to_root() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        let root_stake = 2_000_000u64;
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );

        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let principal_before = u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid));
        assert!(principal_before > 0);
        let root_before = root_stake_of(&hotkey, &coldkey);
        assert_eq!(root_before, root_stake);

        // Claim: full swap of the basket to TAO, staked on root.
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        // Staker's root stake increased, basket principal consumed, watermark advanced.
        assert!(root_stake_of(&hotkey, &coldkey) > root_before);
        assert!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)) < principal_before);
        assert!(RootClaimed::<Test>::get((netuid, &hotkey, &coldkey)) > 0);
    });
}

#[test]
fn test_root_basket_proportional_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice = U256::from(1003);
        let bob = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        // Equal root stake for both stakers.
        let root_stake = 1_000_000u64;
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice,
            NetUid::ROOT,
            root_stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob,
            NetUid::ROOT,
            root_stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );

        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            10_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let alice_before = root_stake_of(&hotkey, &alice);
        let bob_before = root_stake_of(&hotkey, &bob);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob),
            BTreeSet::from([netuid])
        ));

        let alice_gain = root_stake_of(&hotkey, &alice).saturating_sub(alice_before);
        let bob_gain = root_stake_of(&hotkey, &bob).saturating_sub(bob_before);

        assert!(alice_gain > 0);
        // Equal root stake => equal basket payout (small AMM slippage between the two
        // sequential claims on the same pool).
        assert_abs_diff_eq!(alice_gain, bob_gain, epsilon = 1_000u64);
    });
}

// =============================================================================
// Beta basket: hotkey swap migration
// =============================================================================

#[test]
fn test_root_basket_hotkey_swap_migrates() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let new_hotkey = U256::from(10030);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );

        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let basket_before = escrow_alpha(&hotkey, netuid);
        let principal_before = u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid));
        assert!(basket_before > 0);
        assert!(principal_before > 0);

        // Swap the validator's root hotkey: the basket must follow it.
        let mut weight = Weight::zero();
        assert_ok!(SubtensorModule::perform_hotkey_swap_on_one_subnet(
            &hotkey,
            &new_hotkey,
            &mut weight,
            NetUid::ROOT,
            false,
        ));

        // Basket moved to the new hotkey, old slot emptied.
        assert_eq!(escrow_alpha(&hotkey, netuid), 0);
        assert_eq!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)), 0);
        assert_abs_diff_eq!(
            escrow_alpha(&new_hotkey, netuid),
            basket_before,
            epsilon = 10u64
        );
        assert_abs_diff_eq!(
            u64::from(BasketPrincipal::<Test>::get(&new_hotkey, netuid)),
            principal_before,
            epsilon = 10u64,
        );
        assert!(RootClaimable::<Test>::get(new_hotkey).contains_key(&netuid));
        assert!(!RootClaimable::<Test>::get(hotkey).contains_key(&netuid));
    });
}

// =============================================================================
// Beta basket: subnet dissolution liquidates the basket back to root stakers
// =============================================================================

#[test]
fn test_root_basket_dissolve_liquidates_to_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );

        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        assert!(escrow_alpha(&hotkey, netuid) > 0);
        let root_before = root_stake_of(&hotkey, &coldkey);

        // Dissolving the subnet liquidates the basket back to the validator's root stakers.
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        // Basket principal cleared; root stakers credited.
        assert_eq!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)), 0);
        assert!(!RootClaimable::<Test>::get(hotkey).contains_key(&netuid));
        assert!(root_stake_of(&hotkey, &coldkey) > root_before);
    });
}

/// Dissolve liquidation must distribute by each staker's *owed* basket entitlement, NOT by
/// current root-stake share. A "fresh" staker who joined after the basket accrued (zero owed)
/// must receive nothing, even with an equal current root stake — otherwise they'd windfall at
/// the expense of the staker who actually funded the basket.
#[test]
fn test_root_basket_dissolve_distributes_by_owed_not_stake() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice = U256::from(1003);
        let bob = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);

        // Alice is the sole root staker while the basket accrues — she funds all of it.
        let stake = 2_000_000u64;
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice,
            NetUid::ROOT,
            stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );
        assert!(escrow_alpha(&hotkey, netuid) > 0);

        // Bob joins AFTER accrual with the SAME root stake; his watermark is rebased exactly as
        // real `add_stake` would, so his owed entitlement is zero.
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob,
            NetUid::ROOT,
            stake.into(),
        );
        SubtensorModule::add_stake_adjust_root_claimed_for_hotkey_and_coldkey(&hotkey, &bob, stake);

        // Equal current root stake, but only Alice is owed the basket.
        assert_eq!(root_stake_of(&hotkey, &alice), root_stake_of(&hotkey, &bob));
        assert!(SubtensorModule::get_root_owed_for_hotkey_coldkey(&hotkey, &alice, netuid) > 0);
        assert_eq!(
            SubtensorModule::get_root_owed_for_hotkey_coldkey(&hotkey, &bob, netuid),
            0
        );

        let alice_before = root_stake_of(&hotkey, &alice);
        let bob_before = root_stake_of(&hotkey, &bob);

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        let alice_gain = root_stake_of(&hotkey, &alice).saturating_sub(alice_before);
        let bob_gain = root_stake_of(&hotkey, &bob).saturating_sub(bob_before);

        // The basket goes to Alice (who accrued it); Bob (zero owed) gets nothing — even though
        // a stake-proportional split would have handed him ~half.
        assert!(alice_gain > 0, "accruing staker must receive the basket");
        assert_eq!(
            bob_gain, 0,
            "fresh staker with zero owed must receive nothing"
        );
    });
}

// =============================================================================
// Beta basket: conservation invariants ("prove it works")
// =============================================================================

/// TotalStake (the global TAO ledger) must be neutral across both basket distribution
/// (sell origin alpha -> rebuy across w) and redemption (swap basket -> TAO on root):
/// no TAO is minted or destroyed by the round trips.
#[test]
fn test_root_basket_total_stake_conserved() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        // --- Distribution must not move TotalStake (sell + rebuy is TAO-neutral).
        let ts_before_distribute = TotalStake::<Test>::get().to_u64();
        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );
        let ts_after_distribute = TotalStake::<Test>::get().to_u64();
        assert_eq!(
            ts_before_distribute, ts_after_distribute,
            "distribution must be TotalStake-neutral"
        );

        // --- Redemption must also be TotalStake-neutral (swap out then stake on root).
        let ts_before_claim = TotalStake::<Test>::get().to_u64();
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));
        let ts_after_claim = TotalStake::<Test>::get().to_u64();
        assert_eq!(
            ts_before_claim, ts_after_claim,
            "redemption must be TotalStake-neutral"
        );
    });
}

/// The basket compounds: if the escrow position grows (validator earns more on the subnet)
/// after accrual, a sole staker redeems MORE than their recorded principal — the `E/P`
/// multiplier carries the growth through to the staker.
#[test]
fn test_root_basket_compounds_when_escrow_grows() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        // Single root staker => owns 100% of the basket.
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let principal = u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid));
        let escrow_before = escrow_alpha(&hotkey, netuid);
        assert!(principal > 0);

        // Validator earns more nominator dividends on the subnet => escrow value grows,
        // principal stays fixed (E/P rises above 1).
        SubtensorModule::increase_stake_for_hotkey_on_subnet(
            &hotkey,
            netuid,
            100_000_000u64.into(),
        );
        let escrow_after = escrow_alpha(&hotkey, netuid);
        assert!(
            escrow_after > escrow_before,
            "escrow must grow with dividends"
        );

        let root_before = root_stake_of(&hotkey, &coldkey);
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));
        let gain = root_stake_of(&hotkey, &coldkey).saturating_sub(root_before);

        // The sole staker realizes the *grown* basket, strictly more than principal.
        assert!(
            gain > principal,
            "compounding: realized {gain} must exceed principal {principal}"
        );
    });
}

/// Claiming drains the basket exactly: after all stakers redeem, the escrow position and the
/// outstanding basket principal both go to ~zero (Σ payouts == escrow value; no residual,
/// no over-draw).
#[test]
fn test_root_basket_fully_drains_on_claims() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice = U256::from(1003);
        let bob = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice,
            NetUid::ROOT,
            1_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob,
            NetUid::ROOT,
            3_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            10_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let escrow_filled = escrow_alpha(&hotkey, netuid);
        assert!(escrow_filled > 0);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob),
            BTreeSet::from([netuid])
        ));

        // Escrow and principal fully drained (allow tiny rounding dust).
        assert!(
            escrow_alpha(&hotkey, netuid) <= 10,
            "escrow must be drained, got {}",
            escrow_alpha(&hotkey, netuid)
        );
        assert!(
            u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)) <= 10,
            "principal must be drained, got {}",
            u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid))
        );
    });
}

/// Disproportionate root stake yields proportionate payout: a staker with 2x the root stake
/// redeems ~2x the TAO.
#[test]
fn test_root_basket_disproportional_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice = U256::from(1003);
        let bob = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        // Bob has 2x Alice's root stake.
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice,
            NetUid::ROOT,
            1_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            10_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let alice_before = root_stake_of(&hotkey, &alice);
        let bob_before = root_stake_of(&hotkey, &bob);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob),
            BTreeSet::from([netuid])
        ));

        let alice_gain = root_stake_of(&hotkey, &alice).saturating_sub(alice_before);
        let bob_gain = root_stake_of(&hotkey, &bob).saturating_sub(bob_before);

        assert!(alice_gain > 0);
        // Bob staked 2x => ~2x payout (small AMM slippage between sequential claims).
        assert_abs_diff_eq!(bob_gain, 2 * alice_gain, epsilon = 2_000u64);
    });
}

/// A weight vector that spans multiple subnets splits the basket across them in proportion
/// to the weights.
#[test]
fn test_root_basket_splits_across_multiple_subnets() {
    new_test_ext(1).execute_with(|| {
        let owner_a = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let owner_b = U256::from(2001);
        let hotkey_b = U256::from(2002);
        let owner_c = U256::from(3001);
        let hotkey_c = U256::from(3002);

        let netuid_a = add_dynamic_network(&hotkey, &owner_a);
        let netuid_b = add_dynamic_network(&hotkey_b, &owner_b);
        let netuid_c = add_dynamic_network(&hotkey_c, &owner_c);
        remove_owner_registration_stake(netuid_a);
        fund_pool(netuid_a);
        fund_pool(netuid_b);
        fund_pool(netuid_c);

        SubtensorModule::set_tao_weight(u64::MAX);

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_a,
            netuid_a,
            10_000_000u64.into(),
        );

        // 50/50 split between B and C (neither is the origin A).
        set_root_weights_direct(&hotkey, 0, &[(netuid_b, u16::MAX), (netuid_c, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid_a,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            10_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let basket_b = escrow_alpha(&hotkey, netuid_b);
        let basket_c = escrow_alpha(&hotkey, netuid_c);

        assert!(basket_b > 0 && basket_c > 0, "both targets must be funded");
        assert_eq!(
            escrow_alpha(&hotkey, netuid_a),
            0,
            "origin must hold nothing"
        );
        // Equal weights + equal-depth pools => ~equal split.
        assert_abs_diff_eq!(basket_b, basket_c, epsilon = 1_000u64);
    });
}

/// The `set_root_weights` extrinsic stores the validator's vector under the root weights index.
#[test]
fn test_set_root_weights_stores_vector() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        // Register the validator on root (uid 0) and give it stake.
        NetworksAdded::<Test>::insert(NetUid::ROOT, true);
        SubnetworkN::<Test>::insert(NetUid::ROOT, 1);
        Uids::<Test>::insert(NetUid::ROOT, hotkey, 0u16);
        Keys::<Test>::insert(NetUid::ROOT, 0u16, hotkey);
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );

        assert_ok!(SubtensorModule::set_root_weights(
            RuntimeOrigin::signed(hotkey),
            vec![u16::from(netuid)],
            vec![u16::MAX],
            0,
        ));

        let stored = Weights::<Test>::get(NetUidStorageIndex::ROOT, 0u16);
        assert_eq!(stored, vec![(u16::from(netuid), u16::MAX)]);
    });
}

// =============================================================================
// Claims 1-4: the staker-facing guarantees, proven directly.
// =============================================================================

/// CLAIM 1 — staking principal can never be lost: the basket only ever deploys the validator's
/// dividends, never the staker's root principal. A distribution leaves the staker's root stake
/// untouched, and a claim only ever *adds* to it.
#[test]
fn test_claim1_principal_never_lost() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        let principal = 2_000_000u64;
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            principal.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        // Dividend distribution did not touch the staker's root principal.
        assert_eq!(root_stake_of(&hotkey, &coldkey), principal);

        // Claiming only adds TAO to the root principal (never subtracts).
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));
        assert!(root_stake_of(&hotkey, &coldkey) >= principal);
    });
}

/// CLAIM 2 — accrued beta is unaffected by *others* staking the same validator: another staker
/// joining does not change your already-accrued basket value, and they accrue nothing of yours.
#[test]
fn test_claim2_accrued_basket_unchanged_when_others_stake() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice = U256::from(1003);
        let bob = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let alice_before = SubtensorModule::get_basket_payout_alpha(&hotkey, &alice, netuid);
        assert!(alice_before > 0);

        // Bob stakes the same validator (no new distribution). The mock stake helper bypasses the
        // root-claimed watermark that the real add_stake applies, so set it explicitly.
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob,
            NetUid::ROOT,
            5_000_000u64.into(),
        );
        SubtensorModule::add_stake_adjust_root_claimed_for_hotkey_and_coldkey(
            &hotkey,
            &bob,
            5_000_000u64,
        );

        // Alice's accrued basket is unchanged; Bob has accrued nothing of it.
        assert_eq!(
            SubtensorModule::get_basket_payout_alpha(&hotkey, &alice, netuid),
            alice_before
        );
        assert_eq!(
            SubtensorModule::get_basket_payout_alpha(&hotkey, &bob, netuid),
            0
        );
    });
}

/// CLAIM 3 — earned beta compounds: while it sits staked under the validator it earns the
/// validator's subnet dividends, so the staker's claimable value grows beyond what they earned.
#[test]
fn test_claim3_basket_compounds() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let before = SubtensorModule::get_basket_payout_alpha(&hotkey, &coldkey, netuid);
        assert!(before > 0);

        // The validator earns subnet dividends on the basket position (escrow value grows).
        let escrow = SubtensorModule::get_beta_escrow_account_id();
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &escrow,
            netuid,
            before.into(),
        );

        // The sole staker's claimable value compounded upward.
        assert!(SubtensorModule::get_basket_payout_alpha(&hotkey, &coldkey, netuid) > before);
    });
}

/// CLAIM 4 — a late staker can neither claim the existing basket nor skim its past compounding.
/// Proven two ways: (a) a fresh staker's owed is zero, and (b) a deposit into an already
/// compounded basket leaves the `E/P` multiplier unchanged (deposit-at-NAV), so the late
/// staker only ever earns their fair share of *new* distributions — never the old compounding.
#[test]
fn test_claim4_no_dilution_or_skim_on_late_stake() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice = U256::from(1003);
        let bob = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid);

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        // Equal root stake for Alice and Bob.
        let stake = 2_000_000u64;
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice,
            NetUid::ROOT,
            stake.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        // Alice accrues a basket.
        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        // The basket compounds heavily (escrow value grows ~4x; principal unchanged).
        let escrow = SubtensorModule::get_beta_escrow_account_id();
        let e0 = escrow_alpha(&hotkey, netuid);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &escrow,
            netuid,
            (3 * e0).into(),
        );

        let mult = |hk: &U256| -> f64 {
            let e = escrow_alpha(hk, netuid) as f64;
            let p = u64::from(BasketPrincipal::<Test>::get(hk, netuid)) as f64;
            e / p
        };
        let mult_before = mult(&hotkey);
        assert!(
            mult_before > 3.0,
            "basket should have compounded, got {mult_before}"
        );
        let alice_before = SubtensorModule::get_basket_payout_alpha(&hotkey, &alice, netuid);

        // Bob stakes the heavily-compounded validator. The mock stake helper bypasses the
        // root-claimed watermark that the real add_stake applies, so set it explicitly.
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob,
            NetUid::ROOT,
            stake.into(),
        );
        SubtensorModule::add_stake_adjust_root_claimed_for_hotkey_and_coldkey(&hotkey, &bob, stake);

        // (4a) Bob cannot claim any of the existing basket; Alice's accrual is untouched.
        assert_eq!(
            SubtensorModule::get_basket_payout_alpha(&hotkey, &bob, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_basket_payout_alpha(&hotkey, &alice, netuid),
            alice_before
        );

        // A new distribution deposits into the already-compounded basket.
        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        // (4b) Deposit-at-NAV: the E/P multiplier is unchanged, so no dilution occurred.
        let mult_after = mult(&hotkey);
        assert_abs_diff_eq!(mult_after, mult_before, epsilon = 0.02);

        let alice_after = SubtensorModule::get_basket_payout_alpha(&hotkey, &alice, netuid);
        let bob_after = SubtensorModule::get_basket_payout_alpha(&hotkey, &bob, netuid);

        // Alice was not diluted: her value only grew.
        assert!(alice_after >= alice_before);

        // The new distribution split fairly (equal root stake) — and crucially, Bob's *entire*
        // basket equals only Alice's *increment* from the new distribution. Bob captured none of
        // Alice's pre-existing compounding (alice_before).
        let alice_increment = alice_after.saturating_sub(alice_before);
        assert!(bob_after > 0);
        assert_abs_diff_eq!(alice_increment, bob_after, epsilon = 1_000u64);
        assert!(
            bob_after < alice_before,
            "late staker skimmed past compounding: bob={bob_after} alice_before={alice_before}"
        );
    });
}

/// The read-only views (RPC surface) report the basket correctly: a sole staker's "owed TAO"
/// equals the validator NAV equals the network total, and the breakdown lists the slot.
#[test]
fn test_root_basket_rpc_views() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);
        fund_pool(netuid); // price ~= 1.0 (TAO reserve == alpha reserve)

        SubtensorModule::set_tao_weight(u64::MAX);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        // Empty baskets read as zero everywhere.
        assert_eq!(SubtensorModule::get_root_basket_total_nav_tao().to_u64(), 0);
        assert_eq!(
            SubtensorModule::get_validator_basket_nav_tao(&hotkey).to_u64(),
            0
        );

        // Single staker => owns 100% of the basket.
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            2_000_000u64.into(),
        );
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        SubtensorModule::distribute_emission(
            netuid,
            AlphaBalance::ZERO,
            AlphaBalance::ZERO,
            1_000_000u64.into(),
            AlphaBalance::ZERO,
        );

        let nav = SubtensorModule::get_validator_basket_nav_tao(&hotkey).to_u64();
        let total = SubtensorModule::get_root_basket_total_nav_tao().to_u64();
        let owed = SubtensorModule::get_root_basket_owed_tao(&coldkey).to_u64();
        let basket = SubtensorModule::get_validator_basket(&hotkey);

        assert!(nav > 0, "validator NAV must be positive");
        // Single validator => network total == this validator's NAV.
        assert_eq!(total, nav);
        // Sole staker => owed (marked) == NAV (marked), both value the same escrow alpha.
        assert_abs_diff_eq!(owed, nav, epsilon = 10u64);

        // Breakdown lists exactly the one funded subnet, and its TAO value sums to the NAV.
        assert_eq!(basket.len(), 1);
        assert_eq!(basket[0].0, netuid);
        assert!(basket[0].1.to_u64() > 0); // alpha held
        assert_eq!(basket[0].2.to_u64(), nav); // tao value == NAV
    });
}

/// End-to-end through the real coinbase path (block_step -> run_coinbase -> emit_to_subnets
/// -> drain_pending -> distribute_emission), proving the basket forms from actual block
/// emission rather than a direct `distribute_emission` call.
#[test]
fn test_root_basket_end_to_end_via_coinbase() {
    new_test_ext(0).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);
        remove_owner_registration_stake(netuid);

        Tempo::<Test>::insert(netuid, 1);
        SubtensorModule::set_tao_weight(u64::MAX);

        let root_stake = 200_000_000u64;
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoBalance::from(root_stake));
        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        // Turn root-sell ON: moving price + spot price > 1.
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(2));
        SubnetTAO::<Test>::insert(netuid, TaoBalance::from(10_000_000_000_000u64));
        SubnetAlphaIn::<Test>::insert(netuid, AlphaBalance::from(1_000_000_000_000u64));
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));
        assert!(
            SubtensorModule::get_network_root_sell_flag(&[netuid]),
            "root sell flag must be ON"
        );

        mock_increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            10_000_000u64.into(),
        );

        // Validator routes its basket back into the subnet.
        set_root_weights_direct(&hotkey, 0, &[(netuid, u16::MAX)]);

        assert_eq!(escrow_alpha(&hotkey, netuid), 0);

        // Run real blocks: emission accrues and drains through the coinbase.
        run_to_block(3);

        // The basket formed end-to-end from actual block emission.
        assert!(
            escrow_alpha(&hotkey, netuid) > 0,
            "basket must form from coinbase emission"
        );
        assert!(u64::from(BasketPrincipal::<Test>::get(&hotkey, netuid)) > 0);
        assert!(RootClaimable::<Test>::get(hotkey).contains_key(&netuid));

        // And it is redeemable to root TAO.
        let root_before = root_stake_of(&hotkey, &coldkey);
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));
        assert!(root_stake_of(&hotkey, &coldkey) > root_before);
    });
}
