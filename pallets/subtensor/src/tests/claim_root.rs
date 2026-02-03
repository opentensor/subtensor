#![allow(clippy::expect_used)]

use crate::RootAlphaDividendsPerSubnet;
use crate::tests::mock::{
    RuntimeOrigin, SubtensorModule, Test, add_dynamic_network, new_test_ext, run_to_block,
};
use crate::{
    DefaultMinRootClaimAmount, Error, MAX_NUM_ROOT_CLAIMS, MAX_ROOT_CLAIM_THRESHOLD, NetworksAdded,
    NumRootClaim, NumStakingColdkeys, PendingRootAlphaDivs, RootClaimable, RootClaimableThreshold,
    StakingColdkeys, StakingColdkeysByIndex, SubnetAlphaIn, SubnetMechanism, SubnetMovingPrice,
    SubnetTAO, SubnetTaoFlow, SubtokenEnabled, Tempo, pallet,
};
use crate::{RootClaimType, RootClaimTypeEnum, RootClaimed};
use approx::assert_abs_diff_eq;
use frame_support::dispatch::RawOrigin;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::Get;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{H256, U256};
use sp_runtime::DispatchError;
use std::collections::BTreeSet;
use substrate_fixed::types::{I96F32, U96F32};
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::SwapHandler;

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
fn test_claim_root_with_drain_emissions() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        let old_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        assert_eq!(old_validator_stake, initial_total_hotkey_alpha.into());

        // Distribute pending root alpha

        let pending_root_alpha = 1_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Check new validator stake
        let validator_take_percent = 0.18f64;

        let new_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        let calculated_validator_stake = (pending_root_alpha as f64) * validator_take_percent
            + (initial_total_hotkey_alpha as f64);

        assert_abs_diff_eq!(
            u64::from(new_validator_stake),
            calculated_validator_stake as u64,
            epsilon = 100u64,
        );

        // Check claimable

        let claimable = *RootClaimable::<Test>::get(hotkey)
            .get(&netuid)
            .expect("claimable must exist at this point");
        let calculated_rate =
            (pending_root_alpha as f64) * (1f64 - validator_take_percent) / (root_stake as f64);

        assert_abs_diff_eq!(
            claimable.saturating_to_num::<f64>(),
            calculated_rate,
            epsilon = 0.001f64,
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        assert_abs_diff_eq!(
            new_stake,
            (I96F32::from(root_stake) * claimable).saturating_to_num::<u64>(),
            epsilon = 10u64,
        );

        // Check root claimed value saved

        let claimed = RootClaimed::<Test>::get((netuid, &hotkey, &coldkey));
        assert_eq!(u128::from(new_stake), claimed);

        // Distribute pending root alpha (round 2)

        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Check claimable (round 2)

        let claimable2 = *RootClaimable::<Test>::get(hotkey)
            .get(&netuid)
            .expect("claimable must exist at this point");
        let calculated_rate =
            (pending_root_alpha as f64) * (1f64 - validator_take_percent) / (root_stake as f64);

        assert_abs_diff_eq!(
            claimable2.saturating_to_num::<f64>(),
            calculated_rate + claimable.saturating_to_num::<f64>(),
            epsilon = 0.001f64,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        let new_stake2: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();
        let calculated_new_stake2 =
            (I96F32::from(root_stake) * claimable2).saturating_to_num::<u64>();

        assert_abs_diff_eq!(
            u64::from(new_stake2),
            calculated_new_stake2,
            epsilon = 10u64,
        );

        // Check root claimed value saved (round 2)

        let claimed = RootClaimed::<Test>::get((netuid, &hotkey, &coldkey));
        assert_eq!(u128::from(u64::from(new_stake2)), claimed);
    });
}

#[test]
fn test_claim_root_adding_stake_proportionally_for_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let other_coldkey = U256::from(10010);
        let hotkey = U256::from(1002);
        let alice_coldkey = U256::from(1003);
        let bob_coldkey = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 1_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let root_stake_rate = 0.1f64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &other_coldkey,
            NetUid::ROOT,
            (8 * root_stake).into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(alice_coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(bob_coldkey),
            RootClaimTypeEnum::Keep
        ),);

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check stakes
        let validator_take_percent = 0.18f64;

        let alice_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let estimated_stake =
            (pending_root_alpha as f64) * (1f64 - validator_take_percent) * root_stake_rate;

        assert_eq!(alice_stake, bob_stake);

        assert_abs_diff_eq!(alice_stake, estimated_stake as u64, epsilon = 100u64,);
    });
}

#[test]
fn test_claim_root_adding_stake_disproportionally_for_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let other_coldkey = U256::from(10010);
        let hotkey = U256::from(1002);
        let alice_coldkey = U256::from(1003);
        let bob_coldkey = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let alice_root_stake = 1_000_000u64;
        let bob_root_stake = 2_000_000u64;
        let other_root_stake = 7_000_000u64;

        let alice_root_stake_rate = 0.1f64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            NetUid::ROOT,
            alice_root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            NetUid::ROOT,
            bob_root_stake.into(),
        );

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &other_coldkey,
            NetUid::ROOT,
            (other_root_stake).into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(alice_coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(bob_coldkey),
            RootClaimTypeEnum::Keep
        ),);

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check stakes
        let validator_take_percent = 0.18f64;

        let alice_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let alice_estimated_stake =
            (pending_root_alpha as f64) * (1f64 - validator_take_percent) * alice_root_stake_rate;

        assert_eq!(2 * alice_stake, bob_stake);

        assert_abs_diff_eq!(alice_stake, alice_estimated_stake as u64, epsilon = 100u64,);
    });
}

#[test]
fn test_claim_root_with_changed_stake() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice_coldkey = U256::from(1003);
        let bob_coldkey = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtokenEnabled::<Test>::insert(NetUid::ROOT, true);
        NetworksAdded::<Test>::insert(NetUid::ROOT, true);

        let root_stake = 8_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(alice_coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(bob_coldkey),
            RootClaimTypeEnum::Keep
        ),);

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check stakes
        let validator_take_percent = 0.18f64;

        let alice_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let estimated_stake = (pending_root_alpha as f64) * (1f64 - validator_take_percent) / 2f64;

        assert_eq!(alice_stake, bob_stake);

        assert_abs_diff_eq!(alice_stake, estimated_stake as u64, epsilon = 100u64,);

        // Remove stake
        let stake_decrement = root_stake / 2u64;

        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(bob_coldkey,),
            hotkey,
            NetUid::ROOT,
            stake_decrement.into(),
        ));

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stakes

        let alice_stake2: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake2: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let estimated_stake = (pending_root_alpha as f64) * (1f64 - validator_take_percent) / 3f64;

        let alice_stake_diff = alice_stake2 - alice_stake;
        let bob_stake_diff = bob_stake2 - bob_stake;

        assert_abs_diff_eq!(alice_stake_diff, 2 * bob_stake_diff, epsilon = 100u64,);
        assert_abs_diff_eq!(bob_stake_diff, estimated_stake as u64, epsilon = 100u64,);

        // Add stake
        let stake_increment = root_stake / 2u64;

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(bob_coldkey,),
            hotkey,
            NetUid::ROOT,
            stake_increment.into(),
        ));

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stakes

        let alice_stake3: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake3: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let estimated_stake = (pending_root_alpha as f64) * (1f64 - validator_take_percent) / 2f64;

        let alice_stake_diff2 = alice_stake3 - alice_stake2;
        let bob_stake_diff2 = bob_stake3 - bob_stake2;

        assert_abs_diff_eq!(alice_stake_diff2, bob_stake_diff2, epsilon = 100u64,);
        assert_abs_diff_eq!(bob_stake_diff2, estimated_stake as u64, epsilon = 100u64,);
    });
}

#[test]
fn test_claim_root_with_drain_emissions_and_swap_claim_type() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let other_coldkey = U256::from(10010);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubnetMechanism::<Test>::insert(netuid, 1);

        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .saturating_to_num::<f64>();
        assert_eq!(current_price, 0.5f64);

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        let root_stake_rate = 0.1f64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &other_coldkey,
            NetUid::ROOT,
            (9 * root_stake).into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Claim root alpha

        let validator_take_percent = 0.18f64;

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Swap
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Swap);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stake

        let new_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
        )
        .into();

        let estimated_stake_increment = (pending_root_alpha as f64)
            * (1f64 - validator_take_percent)
            * current_price
            * root_stake_rate;

        assert_abs_diff_eq!(
            new_stake,
            root_stake + estimated_stake_increment as u64,
            epsilon = 10000u64,
        );

        // Distribute and claim pending root alpha (round 2)

        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stake (2)

        let new_stake2: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
        )
        .into();

        // new root stake / new total stake
        let root_stake_rate2 = (root_stake as f64 + estimated_stake_increment)
            / (root_stake as f64 / root_stake_rate + estimated_stake_increment);
        let estimated_stake_increment2 = (pending_root_alpha as f64)
            * (1f64 - validator_take_percent)
            * current_price
            * root_stake_rate2;

        assert_abs_diff_eq!(
            new_stake2,
            new_stake + estimated_stake_increment2 as u64,
            epsilon = 10000u64,
        );
        // Distribute and claim pending root alpha (round 3)

        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stake (3)

        let new_stake3: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
        )
        .into();

        // new root stake / new total stake
        let root_stake_rate3 =
            (root_stake as f64 + estimated_stake_increment + estimated_stake_increment2)
                / (root_stake as f64 / root_stake_rate
                    + estimated_stake_increment
                    + estimated_stake_increment2);
        let estimated_stake_increment3 = (pending_root_alpha as f64)
            * (1f64 - validator_take_percent)
            * current_price
            * root_stake_rate3;

        assert_abs_diff_eq!(
            new_stake3,
            new_stake2 + estimated_stake_increment3 as u64,
            epsilon = 10000u64,
        );
    });
}

/// cargo test --package pallet-subtensor --lib -- tests::claim_root::test_claim_root_with_run_coinbase --exact --nocapture
#[test]
fn test_claim_root_with_run_coinbase() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        Tempo::<Test>::insert(netuid, 1);
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 200_000_000u64;
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(root_stake));

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Set moving price > 1.0 and price > 1.0
        // So we turn ON root sell
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(2));
        let tao = TaoCurrency::from(10_000_000_000_000_u64);
        let alpha = AlphaCurrency::from(1_000_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao);
        SubnetAlphaIn::<Test>::insert(netuid, alpha);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .saturating_to_num::<f64>();
        assert_eq!(current_price, 10.0f64);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        // Make sure we are root selling, so we have root alpha divs.
        let root_sell_flag = SubtensorModule::get_network_root_sell_flag(&[netuid]);
        assert!(root_sell_flag, "Root sell flag should be true");

        // Distribute pending root alpha

        let initial_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();
        assert_eq!(initial_stake, 0u64);

        let block_emissions = 1_000_000u64;
        SubtensorModule::run_coinbase(U96F32::from(block_emissions));

        // Claim root alpha

        let initial_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();
        assert_eq!(initial_stake, 0u64);

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        assert!(new_stake > 0);
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
fn test_claim_root_with_block_emissions() {
    new_test_ext(0).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        Tempo::<Test>::insert(netuid, 1);
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 200_000_000u64;
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(root_stake));

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::maybe_add_coldkey_index(&coldkey);

        // Set moving price > 1.0 and price > 1.0
        // So we turn ON root sell
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(2));
        let tao = TaoCurrency::from(10_000_000_000_000_u64);
        let alpha = AlphaCurrency::from(1_000_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao);
        SubnetAlphaIn::<Test>::insert(netuid, alpha);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .saturating_to_num::<f64>();
        assert_eq!(current_price, 10.0f64);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        // Make sure we are root selling, so we have root alpha divs.
        let root_sell_flag = SubtensorModule::get_network_root_sell_flag(&[netuid]);
        assert!(root_sell_flag, "Root sell flag should be true");

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);

        // Distribute pending root alpha

        let initial_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();
        assert_eq!(initial_stake, 0u64);

        run_to_block(2);

        // Check stake after block emissions

        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        assert!(new_stake > 0);
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
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey1,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey2,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
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
fn test_claim_root_coinbase_distribution() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        Tempo::<Test>::insert(netuid, 1);
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 200_000_000u64;
        let initial_tao = 200_000_000u64;
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(initial_tao));

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Set moving price > 1.0 and price > 1.0
        // So we turn ON root sell
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(2));
        let tao = TaoCurrency::from(100_000_000_000_u64);
        let alpha = AlphaCurrency::from(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao);
        SubnetAlphaIn::<Test>::insert(netuid, alpha);
        // let current_price =
        //     <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
        //         .saturating_to_num::<f64>();
        // assert_eq!(current_price, 2.0f64);
        RootClaimableThreshold::<Test>::insert(netuid, I96F32::from_num(0));

        let initial_alpha_issuance = SubtensorModule::get_alpha_issuance(netuid);
        let alpha_emissions: AlphaCurrency = 1_000_000_000u64.into();

        // Make sure we are root selling, so we have root alpha divs.
        let root_sell_flag = SubtensorModule::get_network_root_sell_flag(&[netuid]);
        assert!(root_sell_flag, "Root sell flag should be true");

        // Set TAOFlow > 0
        SubnetTaoFlow::<Test>::insert(netuid, 2222_i64);

        // Check total issuance (saved to pending alpha divs)
        run_to_block(2);

        let alpha_issuance = SubtensorModule::get_alpha_issuance(netuid);
        // We went two blocks so we should have 2x the alpha emissions
        assert_eq!(
            initial_alpha_issuance + alpha_emissions.saturating_mul(2.into()),
            alpha_issuance
        );

        let root_prop = initial_tao as f64 / (u64::from(alpha_issuance) + initial_tao) as f64;
        let root_validators_share = 0.5f64;

        let expected_pending_root_alpha_divs =
            u64::from(alpha_emissions) as f64 * root_prop * root_validators_share;
        assert_abs_diff_eq!(
            u64::from(PendingRootAlphaDivs::<Test>::get(netuid)) as f64,
            expected_pending_root_alpha_divs,
            epsilon = 100f64
        );

        // Epoch pending alphas divs is distributed

        run_to_block(3);

        assert_eq!(u64::from(PendingRootAlphaDivs::<Test>::get(netuid)), 0u64);

        let claimable = *RootClaimable::<Test>::get(hotkey)
            .get(&netuid)
            .expect("claimable must exist at this point");

        let validator_take_percent = 0.18f64;
        let calculated_rate = (expected_pending_root_alpha_divs * 2f64)
            * (1f64 - validator_take_percent)
            / (root_stake as f64);

        assert_abs_diff_eq!(
            claimable.saturating_to_num::<f64>(),
            calculated_rate,
            epsilon = 0.001f64,
        );
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
fn test_claim_root_with_swap_coldkey() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        let old_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        assert_eq!(old_validator_stake, initial_total_hotkey_alpha.into());

        // Distribute pending root alpha

        let pending_root_alpha = 1_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        // Check root claimed value saved
        let new_coldkey = U256::from(10030);

        assert_eq!(
            u128::from(new_stake),
            RootClaimed::<Test>::get((netuid, &hotkey, &coldkey))
        );
        assert_eq!(
            0u128,
            RootClaimed::<Test>::get((netuid, &hotkey, &new_coldkey))
        );

        // Swap coldkey
        assert_ok!(SubtensorModule::do_swap_coldkey(&coldkey, &new_coldkey,));

        // Check swapped keys claimed values

        assert_eq!(0u128, RootClaimed::<Test>::get((netuid, &hotkey, &coldkey)));
        assert_eq!(
            u128::from(new_stake),
            RootClaimed::<Test>::get((netuid, &hotkey, &new_coldkey,))
        );
    });
}
#[test]
fn test_claim_root_with_swap_hotkey() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        let old_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        assert_eq!(old_validator_stake, initial_total_hotkey_alpha.into());

        // Distribute pending root alpha

        let pending_root_alpha = 1_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Keep);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        // Check root claimed value saved
        let new_hotkey = U256::from(10030);

        assert_eq!(
            u128::from(new_stake),
            RootClaimed::<Test>::get((netuid, &hotkey, &coldkey,))
        );
        assert_eq!(
            0u128,
            RootClaimed::<Test>::get((netuid, &new_hotkey, &coldkey,))
        );

        let _old_claimable = *RootClaimable::<Test>::get(hotkey)
            .get(&netuid)
            .expect("claimable must exist at this point");

        assert!(!RootClaimable::<Test>::get(new_hotkey).contains_key(&netuid));

        // Swap hotkey
        let mut weight = Weight::zero();
        assert_ok!(SubtensorModule::perform_hotkey_swap_on_one_subnet(
            &hotkey,
            &new_hotkey,
            &mut weight,
            netuid
        ));

        // Check swapped keys claimed values

        assert_eq!(
            0u128,
            RootClaimed::<Test>::get((netuid, &hotkey, &coldkey,))
        );
        assert_eq!(
            u128::from(new_stake),
            RootClaimed::<Test>::get((netuid, &new_hotkey, &coldkey,))
        );

        assert!(!RootClaimable::<Test>::get(hotkey).contains_key(&netuid));

        let _new_claimable = *RootClaimable::<Test>::get(new_hotkey)
            .get(&netuid)
            .expect("claimable must exist at this point");
    });
}

#[test]
fn test_claim_root_on_network_deregistration() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let other_coldkey = U256::from(10010);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubnetMechanism::<Test>::insert(netuid, 1);

        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .saturating_to_num::<f64>();
        assert_eq!(current_price, 0.5f64);

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &other_coldkey,
            NetUid::ROOT,
            (9 * root_stake).into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        assert!(RootClaimable::<Test>::get(hotkey).contains_key(&netuid));

        assert!(RootClaimed::<Test>::contains_key((
            netuid, &hotkey, &coldkey,
        )));

        // Claim root via network deregistration

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        assert!(!RootClaimed::<Test>::contains_key((
            netuid, &hotkey, &coldkey,
        )));
        assert!(!RootClaimable::<Test>::get(hotkey).contains_key(&netuid));
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
                BTreeSet::from_iter((0u16..=10u16).into_iter().map(NetUid::from))
            ),
            Error::<Test>::InvalidSubnetNumber
        );
    });
}

#[test]
fn test_claim_root_with_unrelated_subnets() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        let old_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        assert_eq!(old_validator_stake, initial_total_hotkey_alpha.into());

        // Distribute pending root alpha

        let pending_root_alpha = 1_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            RootClaimTypeEnum::Keep
        ),);

        // Claim root alpha on unrelated subnets

        let unrelated_subnet_uid = NetUid::from(100u16);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([unrelated_subnet_uid])
        ));

        let new_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            unrelated_subnet_uid,
        )
        .into();

        assert_eq!(new_stake, 0u64,);

        // Check root claim for correct subnet

        // before
        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        assert_eq!(new_stake, 0u64,);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        // after
        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        assert!(new_stake > 0u64);

        // Check root claimed value saved

        let claimed = RootClaimed::<Test>::get((netuid, &hotkey, &coldkey));
        assert_eq!(u128::from(new_stake), claimed);
    });
}

#[test]
fn test_claim_root_fill_root_alpha_dividends_per_subnet() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let other_coldkey = U256::from(10010);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubnetMechanism::<Test>::insert(netuid, 1);

        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &other_coldkey,
            NetUid::ROOT,
            (9 * root_stake).into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Check RootAlphaDividendsPerSubnet is empty on start
        assert!(!RootAlphaDividendsPerSubnet::<Test>::contains_key(
            netuid, hotkey
        ));

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Check RootAlphaDividendsPerSubnet value
        let root_claim_dividends1 = RootAlphaDividendsPerSubnet::<Test>::get(netuid, hotkey);

        let validator_take_percent = 0.18f64;
        let estimated_root_claim_dividends =
            (pending_root_alpha as f64) * (1f64 - validator_take_percent);

        assert_abs_diff_eq!(
            estimated_root_claim_dividends as u64,
            u64::from(root_claim_dividends1),
            epsilon = 100u64,
        );

        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        let root_claim_dividends2 = RootAlphaDividendsPerSubnet::<Test>::get(netuid, hotkey);

        // Check RootAlphaDividendsPerSubnet is cleaned each epoch
        assert_eq!(root_claim_dividends1, root_claim_dividends2);
    });
}

#[test]
fn test_claim_root_with_keep_subnets() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        let old_validator_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
        );
        assert_eq!(old_validator_stake, initial_total_hotkey_alpha.into());

        // Distribute pending root alpha

        let pending_root_alpha = 1_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        let claimable = *RootClaimable::<Test>::get(hotkey)
            .get(&netuid)
            .expect("claimable must exist at this point");

        // Claim root alpha
        assert_err!(
            SubtensorModule::set_root_claim_type(
                RuntimeOrigin::signed(coldkey),
                RootClaimTypeEnum::KeepSubnets {
                    subnets: BTreeSet::new()
                },
            ),
            Error::<Test>::InvalidSubnetNumber
        );

        let keep_subnets = RootClaimTypeEnum::KeepSubnets {
            subnets: BTreeSet::from([netuid]),
        };
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            keep_subnets.clone(),
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), keep_subnets);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        assert_abs_diff_eq!(
            new_stake,
            (I96F32::from(root_stake) * claimable).saturating_to_num::<u64>(),
            epsilon = 10u64,
        );
    });
}

#[test]
fn test_claim_root_keep_subnets_swap_claim_type() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let other_coldkey = U256::from(10010);
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubnetMechanism::<Test>::insert(netuid, 1);

        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .saturating_to_num::<f64>();
        assert_eq!(current_price, 0.5f64);

        let root_stake = 2_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        let root_stake_rate = 0.1f64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &other_coldkey,
            NetUid::ROOT,
            (9 * root_stake).into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Claim root alpha

        let validator_take_percent = 0.18f64;
        // Set to keep 'another' subnet
        let keep_subnets = RootClaimTypeEnum::KeepSubnets {
            subnets: BTreeSet::from([NetUid::from(100u16)]),
        };
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(coldkey),
            keep_subnets.clone()
        ),);
        assert_eq!(RootClaimType::<Test>::get(coldkey), keep_subnets);

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stake

        let new_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
        )
        .into();

        let estimated_stake_increment = (pending_root_alpha as f64)
            * (1f64 - validator_take_percent)
            * current_price
            * root_stake_rate;

        assert_abs_diff_eq!(
            new_stake,
            root_stake + estimated_stake_increment as u64,
            epsilon = 10000u64,
        );
    });
}

#[test]
fn test_claim_root_default_mode_keep() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1003);

        assert_eq!(RootClaimType::<Test>::get(coldkey), RootClaimTypeEnum::Swap);
    });
}

#[test]
fn test_claim_root_with_moved_stake() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice_coldkey = U256::from(1003);
        let bob_coldkey = U256::from(1004);
        let eve_coldkey = U256::from(1005);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubtokenEnabled::<Test>::insert(NetUid::ROOT, true);
        NetworksAdded::<Test>::insert(NetUid::ROOT, true);

        let root_stake = 8_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            NetUid::ROOT,
            root_stake.into(),
        );

        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // Claim root alpha

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(alice_coldkey),
            RootClaimTypeEnum::Keep
        ),);
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(bob_coldkey),
            RootClaimTypeEnum::Keep
        ),);

        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(eve_coldkey),
            RootClaimTypeEnum::Keep
        ),);

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check stakes
        let validator_take_percent = 0.18f64;

        let alice_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let estimated_stake = (pending_root_alpha as f64) * (1f64 - validator_take_percent) / 2f64;

        assert_eq!(alice_stake, bob_stake);

        assert_abs_diff_eq!(alice_stake, estimated_stake as u64, epsilon = 100u64,);

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Transfer stake to other coldkey
        let stake_decrement = root_stake / 2u64;

        assert_ok!(SubtensorModule::transfer_stake(
            RuntimeOrigin::signed(bob_coldkey,),
            eve_coldkey,
            hotkey,
            NetUid::ROOT,
            NetUid::ROOT,
            stake_decrement.into(),
        ));

        let eve_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &eve_coldkey,
            netuid,
        )
        .into();

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(eve_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stakes

        let alice_stake2: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake2: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let eve_stake2: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &eve_coldkey,
            netuid,
        )
        .into();

        // Eve should not have gotten any root claim
        let eve_stake_diff = eve_stake2 - eve_stake;
        assert_abs_diff_eq!(eve_stake_diff, 0, epsilon = 100u64,);

        let estimated_stake = (pending_root_alpha as f64) * (1f64 - validator_take_percent) / 2f64;

        let alice_stake_diff = alice_stake2 - alice_stake;
        let bob_stake_diff = bob_stake2 - bob_stake;

        assert_abs_diff_eq!(alice_stake_diff, bob_stake_diff, epsilon = 100u64,);
        assert_abs_diff_eq!(bob_stake_diff, estimated_stake as u64, epsilon = 100u64,);

        // Transfer stake back
        let stake_increment = stake_decrement;

        assert_ok!(SubtensorModule::transfer_stake(
            RuntimeOrigin::signed(eve_coldkey,),
            bob_coldkey,
            hotkey,
            NetUid::ROOT,
            NetUid::ROOT,
            stake_increment.into(),
        ));

        // Distribute pending root alpha

        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Check new stakes

        let alice_stake3: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_stake3: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        let estimated_stake = (pending_root_alpha as f64) * (1f64 - validator_take_percent) / 2f64;

        let alice_stake_diff2 = alice_stake3 - alice_stake2;
        let bob_stake_diff2 = bob_stake3 - bob_stake2;

        assert_abs_diff_eq!(alice_stake_diff2, bob_stake_diff2, epsilon = 100u64,);
        assert_abs_diff_eq!(bob_stake_diff2, estimated_stake as u64, epsilon = 100u64,);
    });
}
