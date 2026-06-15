#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::tests::mock::*;
use crate::{
    BasketPrincipal, DefaultMinRootClaimAmount, Error, Keys, MAX_NUM_ROOT_CLAIMS,
    MAX_ROOT_CLAIM_THRESHOLD, NetworksAdded, NumRootClaim, NumStakingColdkeys, RootClaimType,
    RootClaimTypeEnum, RootClaimable, RootClaimableThreshold, RootClaimed, StakingColdkeys,
    StakingColdkeysByIndex, SubnetAlphaIn, SubnetMovingPrice, SubnetTAO, SubnetworkN, Tempo,
    TotalStake, Uids, Weights,
};
use approx::assert_abs_diff_eq;
use frame_support::dispatch::RawOrigin;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::Get;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{H256, U256};
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

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);

        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);
    });
}

#[test]
fn test_claim_root_block_hash_indices() {
    new_test_ext(1).execute_with(|| {
        let k = 15u64;
        let n = 15000u64;

        // 0
        let indices =
            SubtensorModule::block_hash_to_indices(H256(sp_core::keccak_256(b"zero")), 0, n);
        assert!(indices.is_empty());

        // 1
        let hash = sp_core::keccak_256(b"some");
        let mut indices = SubtensorModule::block_hash_to_indices(H256(hash), k, n);
        indices.sort();

        assert!(indices.len() <= k as usize);
        assert!(!indices.iter().any(|i| *i >= n));
        // precomputed values
        let expected_result = vec![
            265, 630, 1286, 1558, 4496, 4861, 5517, 5789, 6803, 8096, 9092, 11034, 11399, 12055,
            12327,
        ];
        assert_eq!(indices, expected_result);

        // 2
        let hash = sp_core::keccak_256(b"some2");
        let mut indices = SubtensorModule::block_hash_to_indices(H256(hash), k, n);
        indices.sort();

        assert!(indices.len() <= k as usize);
        assert!(!indices.iter().any(|i| *i >= n));
        // precomputed values
        let expected_result = vec![
            61, 246, 1440, 2855, 3521, 5236, 6130, 6615, 8511, 9405, 9890, 11786, 11971, 13165,
            14580,
        ];
        assert_eq!(indices, expected_result);
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
