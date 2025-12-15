#![allow(clippy::expect_used)]

use crate::tests::mock::{
    RuntimeEvent, RuntimeOrigin, SubtensorModule, System, Test, add_dynamic_network, new_test_ext,
    run_to_block,
};
use crate::{
    DefaultMinRootClaimAmount, Error, GetTaoForAlpha, MAX_NUM_ROOT_CLAIMS,
    MAX_ROOT_CLAIM_THRESHOLD, NetworksAdded, NumRootClaim, NumStakingColdkeys,
    PendingRootAlphaDivs, RootClaimable, RootClaimableThreshold, StakingColdkeys,
    StakingColdkeysByIndex, SubnetAlphaIn, SubnetMechanism, SubnetMovingPrice, SubnetTAO,
    SubnetTaoFlow, SubtokenEnabled, Tempo, pallet,
};
use crate::{Event, RootAlphaDividendsPerSubnet};
use crate::{RootClaimType, RootClaimTypeEnum, RootClaimed, ValidatorClaimType};
use approx::assert_abs_diff_eq;
use frame_support::dispatch::RawOrigin;
use frame_support::pallet_prelude::Weight;
use frame_support::traits::Get;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{H256, U256};
use sp_runtime::DispatchError;
use std::collections::BTreeSet;
use substrate_fixed::types::{I96F32, U64F64, U96F32};
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::{Order, SwapHandler};

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

        SubnetMovingPrice::<Test>::insert(netuid, I96F32::saturating_from_num(1));
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000_000_000u64));

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

        // Check tao flow
        let moving_price = SubnetMovingPrice::<Test>::get(netuid).saturating_to_num::<u64>();
        let order = GetTaoForAlpha::<Test>::with_amount(new_stake);

        let swapped_tao = <Test as pallet::Config>::SwapInterface::swap(
            netuid.into(),
            order,
            moving_price.into(),
            true,
            true,
        )
        .expect("Swap must work here");

        let tao_inflow: u64 =
            SubtensorModule::calculate_tao_flow(netuid, swapped_tao.amount_paid_out).into();

        assert_abs_diff_eq!(
            SubnetTaoFlow::<Test>::get(netuid),
            tao_inflow as i64,
            epsilon = 10i64,
        );

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
        assert_eq!(SubnetTaoFlow::<Test>::get(netuid), 0);

        // Setup root prop
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(1_000_000_000_000u64));

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

        // Check tao flow
        let tao_outflow: u64 =
            SubtensorModule::calculate_tao_flow(netuid, (estimated_stake_increment as u64).into())
                .into();

        assert_abs_diff_eq!(
            SubnetTaoFlow::<Test>::get(netuid),
            -(tao_outflow as i64),
            epsilon = 10i64,
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
        pallet_subtensor_swap::AlphaSqrtPrice::<Test>::insert(
            netuid,
            U64F64::saturating_from_num(10.0),
        );

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
        pallet_subtensor_swap::AlphaSqrtPrice::<Test>::insert(
            netuid,
            U64F64::saturating_from_num(10.0),
        );

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

        let initial_alpha_issuance = SubtensorModule::get_alpha_issuance(netuid);
        let alpha_emissions: AlphaCurrency = 1_000_000_000u64.into();

        // Set moving price > 1.0 and price > 1.0
        // So we turn ON root sell
        SubnetMovingPrice::<Test>::insert(netuid, I96F32::from_num(2));
        pallet_subtensor_swap::AlphaSqrtPrice::<Test>::insert(
            netuid,
            U64F64::saturating_from_num(10.0),
        );

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
        let mut weight = Weight::zero();

        assert_ok!(SubtensorModule::perform_swap_coldkey(
            &coldkey,
            &new_coldkey,
            &mut weight
        ));

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
fn test_claim_root_with_set_validator_claim_type() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let new_claim_type = RootClaimTypeEnum::Swap;

        // Default check
        assert_eq!(
            ValidatorClaimType::<Test>::get(hotkey, netuid),
            RootClaimTypeEnum::Swap
        );

        // Set new type
        assert_ok!(SubtensorModule::set_validator_claim_type(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            new_claim_type.clone()
        ),);

        // Result check
        assert_eq!(
            ValidatorClaimType::<Test>::get(hotkey, netuid),
            new_claim_type
        );

        let event = System::events().into_iter().find(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::ValidatorClaimTypeSet { .. })
            )
        });
        assert!(event.is_some());

        if let Some(RuntimeEvent::SubtensorModule(Event::ValidatorClaimTypeSet {
            hotkey: ev_hotkey,
            root_claim_type: ev_claim_type,
            netuid: ev_netuid,
        })) = event.map(|e| e.event.clone())
        {
            assert_eq!(ev_hotkey, hotkey);
            assert_eq!(ev_claim_type, new_claim_type);
            assert_eq!(ev_netuid, netuid);
        }

        // Check possible options
        assert_ok!(SubtensorModule::set_validator_claim_type(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            RootClaimTypeEnum::Keep
        ),);
        assert_ok!(SubtensorModule::set_validator_claim_type(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            RootClaimTypeEnum::Swap
        ),);
        assert_err!(
            SubtensorModule::set_validator_claim_type(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                RootClaimTypeEnum::Delegated
            ),
            Error::<Test>::InvalidRootClaimType
        );
        assert_err!(
            SubtensorModule::set_validator_claim_type(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                RootClaimTypeEnum::KeepSubnets {
                    subnets: BTreeSet::from([netuid])
                }
            ),
            Error::<Test>::InvalidRootClaimType
        );
    });
}

#[test]
fn test_claim_root_with_delegated_claim_type() {
    new_test_ext(1).execute_with(|| {
        // Setup: Create network with validator (hotkey/owner_coldkey) and two stakers
        let owner_coldkey = U256::from(1001); // Validator's coldkey
        let other_coldkey = U256::from(10010); // Other staker (not tested)
        let hotkey = U256::from(1002); // Validator's hotkey
        let alice_coldkey = U256::from(1003); // Staker who will delegate claim type
        let bob_coldkey = U256::from(1004); // Staker who will set explicit claim type
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        // Configure TAO weight and subnet mechanism for swap functionality
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        SubnetMechanism::<Test>::insert(netuid, 1); // Enable subnet mechanism for swaps

        // Setup swap pool with reserves to enable Swap claim type
        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        // Verify the alpha-to-TAO exchange rate is 0.5
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .saturating_to_num::<f64>();
        assert_eq!(current_price, 0.5f64);

        // Setup root network stakes: Alice and Bob each have 10% of total stake
        let root_stake = 2_000_000u64;
        let root_stake_rate = 0.1f64; // Each staker owns 10% of root stake
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
        // Other coldkey has remaining 80% of stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &other_coldkey,
            NetUid::ROOT,
            (8 * root_stake).into(),
        );

        // Setup subnet alpha stake for validator
        let initial_total_hotkey_alpha = 10_000_000u64;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &owner_coldkey,
            netuid,
            initial_total_hotkey_alpha.into(),
        );

        // SCENARIO 1: Validator sets Keep claim type, Alice uses default (Delegated)
        // Alice should inherit the validator's Keep claim type and receive alpha stake
        assert_ok!(SubtensorModule::set_validator_claim_type(
            RuntimeOrigin::signed(owner_coldkey),
            hotkey,
            netuid,
            RootClaimTypeEnum::Keep
        ),);
        assert_eq!(
            ValidatorClaimType::<Test>::get(hotkey, netuid),
            RootClaimTypeEnum::Keep
        );

        // Bob explicitly sets Keep claim type (same as validator, but not delegated)
        assert_ok!(SubtensorModule::set_root_claim_type(
            RuntimeOrigin::signed(bob_coldkey),
            RootClaimTypeEnum::Keep
        ),);

        // Alice has default Delegated claim type (not explicitly set)
        assert_eq!(
            RootClaimType::<Test>::get(alice_coldkey),
            RootClaimTypeEnum::Delegated
        );

        // Distribute pending root alpha emissions to create claimable rewards
        let pending_root_alpha = 10_000_000u64;
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Alice claims with delegated claim type (should use validator's Keep)
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));

        // Bob claims with explicit Keep claim type
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Verify both stakers received alpha stake (Keep claim type behavior)
        // With Keep, rewards are staked as alpha on the subnet
        let validator_take_percent = 0.18f64;
        let expected_stake_per_user =
            (pending_root_alpha as f64) * (1f64 - validator_take_percent) * root_stake_rate;

        let alice_alpha_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            netuid,
        )
        .into();

        let bob_alpha_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            netuid,
        )
        .into();

        // Both should have equal alpha stakes since they both used Keep claim type
        assert_eq!(alice_alpha_stake, bob_alpha_stake);
        assert_abs_diff_eq!(
            alice_alpha_stake,
            expected_stake_per_user as u64,
            epsilon = 100u64
        );

        // Verify neither received TAO stake (would happen with Swap claim type)
        let alice_tao_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &alice_coldkey,
            NetUid::ROOT,
        )
        .into();
        let bob_tao_stake: u64 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &bob_coldkey,
            NetUid::ROOT,
        )
        .into();
        // TAO stake should remain unchanged at initial amount
        assert_eq!(alice_tao_stake, root_stake);
        assert_eq!(bob_tao_stake, root_stake);

        // SCENARIO 2: Validator changes to Swap claim type
        // Alice (with Delegated) should now use Swap, Bob (explicit Keep) stays with Keep
        assert_ok!(SubtensorModule::set_validator_claim_type(
            RuntimeOrigin::signed(owner_coldkey),
            hotkey,
            netuid,
            RootClaimTypeEnum::Swap
        ),);

        // Distribute more pending root alpha for second round of claims
        SubtensorModule::distribute_emission(
            netuid,
            AlphaCurrency::ZERO,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Both stakers claim again
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(alice_coldkey),
            BTreeSet::from([netuid])
        ));
        assert_ok!(SubtensorModule::claim_root(
            RuntimeOrigin::signed(bob_coldkey),
            BTreeSet::from([netuid])
        ));

        // Alice's alpha stake should remain the same (Swap doesn't add alpha stake)
        let alice_alpha_stake_round2: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &alice_coldkey,
                netuid,
            )
            .into();

        // Bob's alpha stake should increase (Keep adds alpha stake)
        let bob_alpha_stake_round2: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &bob_coldkey,
                netuid,
            )
            .into();

        // Alice used Swap (delegated from validator), so no new alpha stake
        assert_abs_diff_eq!(
            alice_alpha_stake_round2,
            alice_alpha_stake,
            epsilon = 100u64
        );

        // Bob used Keep (explicit), so alpha stake increased
        assert_abs_diff_eq!(
            bob_alpha_stake_round2,
            alice_alpha_stake + expected_stake_per_user as u64,
            epsilon = 100u64
        );

        // Alice used Swap, so TAO stake should increase
        let alice_tao_stake_round2: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &alice_coldkey,
                NetUid::ROOT,
            )
            .into();

        // Bob used Keep, so TAO stake should remain the same
        let bob_tao_stake_round2: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &bob_coldkey,
                NetUid::ROOT,
            )
            .into();

        // Alice's TAO stake increased (swapped alpha to TAO and staked on root)
        let expected_tao_increase = expected_stake_per_user * current_price;
        assert_abs_diff_eq!(
            alice_tao_stake_round2,
            root_stake + expected_tao_increase as u64,
            epsilon = 10000u64
        );

        // Bob's TAO stake unchanged (used Keep, not Swap)
        assert_eq!(bob_tao_stake_round2, root_stake);

        // SUMMARY: This test demonstrates that:
        // 1. Stakers with Delegated claim type inherit the validator's claim type
        // 2. Stakers with explicit claim types use their own setting regardless of validator
        // 3. Keep claim type stakes rewards as alpha on the subnet
        // 4. Swap claim type converts alpha to TAO and stakes on root network
        // 5. Changing validator's claim type affects delegated stakers immediately
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
fn test_claim_root_calculate_tao_flow() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1002);
        let coldkey = U256::from(1003);
        let netuid = add_dynamic_network(&hotkey, &coldkey);

        // Setup root prop
        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        let tao_amount = 10u64;

        // Check unbounded tao outflow (coefficient < 10)
        let tao_reserve = TaoCurrency::from(1_000_000_000_000u64);
        SubnetTAO::<Test>::insert(NetUid::ROOT, tao_reserve);

        let tao_flow_unbounded: u64 =
            SubtensorModule::calculate_tao_flow(netuid, tao_amount.into()).into();

        assert_abs_diff_eq!(tao_flow_unbounded, tao_amount, epsilon = 1u64,);

        // Check bounded tao flow (coefficient > 10).
        let tao_reserve = TaoCurrency::from(10_000_000_000u64);
        SubnetTAO::<Test>::insert(NetUid::ROOT, tao_reserve);

        let tao_flow_bounded: u64 =
            SubtensorModule::calculate_tao_flow(netuid, tao_amount.into()).into();
        let bounded_root_prop_multiplier = 10u64;

        assert_eq!(tao_flow_bounded, bounded_root_prop_multiplier * tao_amount,);
    });
}

#[test]
fn test_claim_root_default_mode() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1003);

        assert_eq!(
            RootClaimType::<Test>::get(coldkey),
            RootClaimTypeEnum::Delegated
        );
    });
}

#[test]
fn test_claim_root_default_validator_mode() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1003);
        let netuid = NetUid::from(100u16);

        assert_eq!(
            ValidatorClaimType::<Test>::get(coldkey, netuid),
            RootClaimTypeEnum::Swap
        );
    });
}
