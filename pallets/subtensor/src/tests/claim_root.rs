use crate::tests::mock::{RuntimeOrigin, SubtensorModule, Test, add_dynamic_network, new_test_ext};
use crate::{RootClaimType, RootClaimTypeEnum, RootClaimed};
use crate::{RootClaimable};
use approx::assert_abs_diff_eq;
use frame_support::assert_ok;
use sp_core::U256;
use substrate_fixed::types::I96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid};

#[test]
fn test_set_root_claim_type() {
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
        SubtensorModule::drain_pending_emission(
            netuid,
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

        let claimable = RootClaimable::<Test>::get(hotkey, netuid);
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

        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(coldkey),));

        let new_stake: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .into();

        assert_abs_diff_eq!(
            new_stake,
            (I96F32::from(root_stake) * claimable).saturating_to_num::<u64>(),
            epsilon = 10u64,
        );

        // Check root claimed value saved

        let claimed = RootClaimed::<Test>::get((&hotkey, &coldkey, netuid));
        assert_eq!(u128::from(new_stake), claimed);

        // Distribute pending root alpha (round 2)

        SubtensorModule::drain_pending_emission(
            netuid,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        // Check claimable (round 2)

        let claimable2 = RootClaimable::<Test>::get(hotkey, netuid);
        let calculated_rate =
            (pending_root_alpha as f64) * (1f64 - validator_take_percent) / (root_stake as f64);

        assert_abs_diff_eq!(
            claimable2.saturating_to_num::<f64>(),
            calculated_rate + claimable.saturating_to_num::<f64>(),
            epsilon = 0.001f64,
        );

        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(coldkey),));

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

        let claimed = RootClaimed::<Test>::get((&hotkey, &coldkey, netuid));
        assert_eq!(u128::from(u64::from(new_stake2)), claimed);
    });
}

#[test]
fn test_adding_stake_proportionally_for_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
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
        SubtensorModule::drain_pending_emission(
            netuid,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(
            alice_coldkey
        ),));
        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(
            bob_coldkey
        ),));

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
    });
}

#[test]
fn test_adding_stake_disproportionally_for_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(1001);
        let hotkey = U256::from(1002);
        let alice_coldkey = U256::from(1003);
        let bob_coldkey = U256::from(1004);
        let netuid = add_dynamic_network(&hotkey, &owner_coldkey);

        SubtensorModule::set_tao_weight(u64::MAX); // Set TAO weight to 1.0

        let alice_root_stake = 1_000_000u64;
        let bob_root_stake = 2_000_000u64;
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
        SubtensorModule::drain_pending_emission(
            netuid,
            AlphaCurrency::ZERO,
            pending_root_alpha.into(),
            AlphaCurrency::ZERO,
        );

        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(
            alice_coldkey
        ),));
        assert_ok!(SubtensorModule::claim_root(RuntimeOrigin::signed(
            bob_coldkey
        ),));

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

        let estimated_stake = (pending_root_alpha as f64) * (1f64 - validator_take_percent) / 3f64;

        assert_eq!(2 * alice_stake, bob_stake);

        assert_abs_diff_eq!(alice_stake, estimated_stake as u64, epsilon = 100u64,);
    });
}
