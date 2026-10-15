#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use super::mock::*;
use crate::weights::WeightInfo;
use crate::*;
use frame_support::{assert_noop, assert_ok, weights::Weight};
use sp_core::U256;
use subtensor_runtime_common::AlphaBalance;

fn setup_claim_subnet() -> (U256, U256, NetUid) {
    let cold = U256::from(1);
    let hot = U256::from(2);
    let netuid = add_dynamic_network(&hot, &cold);
    ClaimBasedMinerEmissions::<Test>::insert(netuid, true);
    // Keep claim fee at zero in unit tests so stake math stays exact.
    MinerClaimAlphaFeePerWeightSecond::<Test>::put(0);
    (cold, hot, netuid)
}

#[test]
fn claim_based_accrues_instead_of_staking() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        let amount: AlphaBalance = 1_000_000u64.into();

        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot, amount);

        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&hot, netuid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_pending_miner_emission_total(netuid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &cold, netuid),
            AlphaBalance::ZERO
        );
    });
}

#[test]
fn claim_miner_emission_stakes_pending() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        let amount: AlphaBalance = 2_000_000u64.into();
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot, amount);

        assert_ok!(SubtensorModule::claim_miner_emission(
            RuntimeOrigin::signed(cold),
            hot,
            netuid,
        ));

        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&hot, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_pending_miner_emission_total(netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &cold, netuid),
            amount
        );
    });
}

#[test]
fn claim_miner_emission_charges_weight_proportional_fee() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        // 1 alpha per second of weight → a non-zero fee for any positive weight.
        MinerClaimAlphaFeePerWeightSecond::<Test>::put(1_000_000_000);
        let amount: AlphaBalance = 5_000_000_000u64.into();
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot, amount);

        let weight = <Test as Config>::WeightInfo::claim_miner_emission();
        let expected_fee = SubtensorModule::miner_claim_alpha_fee(weight);
        assert!(!expected_fee.is_zero(), "test needs a non-zero fee");

        let alpha_out_before = SubnetAlphaOut::<Test>::get(netuid);
        // Mint alpha into outstanding so recycle has something to reduce against
        // the artificial accrue (which does not mint).
        SubnetAlphaOut::<Test>::insert(netuid, amount);

        assert_ok!(SubtensorModule::claim_miner_emission(
            RuntimeOrigin::signed(cold),
            hot,
            netuid,
        ));

        let staked =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &cold, netuid);
        assert_eq!(staked, amount.saturating_sub(expected_fee));
        assert_eq!(
            SubnetAlphaOut::<Test>::get(netuid),
            amount.saturating_sub(expected_fee)
        );
        let _ = alpha_out_before;
    });
}

#[test]
fn claim_succeeds_after_emergency_disable() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        let amount: AlphaBalance = 100u64.into();
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot, amount);

        // Emergency disable stops accrual but must not strand prior pending.
        ClaimBasedMinerEmissions::<Test>::insert(netuid, false);
        assert_ok!(SubtensorModule::claim_miner_emission(
            RuntimeOrigin::signed(cold),
            hot,
            netuid
        ));
        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&hot, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot, &cold, netuid),
            amount
        );

        assert_noop!(
            SubtensorModule::claim_miner_emission(RuntimeOrigin::signed(cold), hot, netuid),
            Error::<Test>::AmountTooLow
        );
    });
}

#[test]
fn force_payout_on_replace_neuron() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        // Register a second neuron we can use as replacement target context.
        let new_hot = U256::from(3);
        let new_cold = U256::from(4);
        SubtensorModule::create_account_if_non_existent(&new_cold, &new_hot).unwrap();

        // Ensure hot is registered as uid 0 (owner hotkey from add_dynamic_network).
        // Use a non-owner miner for replace.
        let miner_cold = U256::from(10);
        let miner_hot = U256::from(11);
        register_ok_neuron(netuid, miner_hot, miner_cold, 0);

        let amount: AlphaBalance = 777_000u64.into();
        SubtensorModule::accrue_pending_miner_alpha(netuid, &miner_hot, amount);

        let uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &miner_hot).unwrap();
        let block = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::replace_neuron(netuid, uid, &new_hot, block);

        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&miner_hot, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &miner_hot,
                &miner_cold,
                netuid
            ),
            amount
        );
        let _ = (cold, hot);
    });
}

#[test]
fn hotkey_swap_moves_pending() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        let new_hot = U256::from(99);
        SubtensorModule::create_account_if_non_existent(&cold, &new_hot).unwrap();

        let amount: AlphaBalance = 123_456u64.into();
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot, amount);
        SubtensorModule::transfer_pending_miner_alpha(netuid, &hot, &new_hot);

        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&hot, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&new_hot, netuid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_pending_miner_emission_total(netuid),
            amount
        );
    });
}

#[test]
fn distribute_path_accrues_when_enabled() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        // Owner hotkeys are withheld; use a separate miner.
        let miner_cold = U256::from(20);
        let miner_hot = U256::from(21);
        register_ok_neuron(netuid, miner_hot, miner_cold, 0);

        let amount: AlphaBalance = 50_000u64.into();
        let mut incentives = sp_std::collections::btree_map::BTreeMap::new();
        incentives.insert(miner_hot, amount);

        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            AlphaBalance::ZERO,
            incentives,
            Default::default(),
            Default::default(),
        );

        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&miner_hot, netuid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &miner_hot,
                &miner_cold,
                netuid
            ),
            AlphaBalance::ZERO
        );
        let _ = (cold, hot);
    });
}

#[test]
fn distribute_path_stakes_when_disabled() {
    new_test_ext(0).execute_with(|| {
        let (_cold, _hot, netuid) = setup_claim_subnet();
        ClaimBasedMinerEmissions::<Test>::insert(netuid, false);

        let miner_cold = U256::from(30);
        let miner_hot = U256::from(31);
        register_ok_neuron(netuid, miner_hot, miner_cold, 0);

        let amount: AlphaBalance = 50_000u64.into();
        let mut incentives = sp_std::collections::btree_map::BTreeMap::new();
        incentives.insert(miner_hot, amount);

        SubtensorModule::distribute_dividends_and_incentives(
            netuid,
            AlphaBalance::ZERO,
            incentives,
            Default::default(),
            Default::default(),
        );

        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&miner_hot, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &miner_hot,
                &miner_cold,
                netuid
            ),
            amount
        );
    });
}

#[test]
fn migration_enables_all_live_subnets() {
    new_test_ext(0).execute_with(|| {
        use crate::migrations::migrate_enable_claim_based_miner_emissions::{
            MIGRATION_NAME, migrate_enable_claim_based_miner_emissions,
        };

        let cold = U256::from(1);
        let hot = U256::from(2);
        let netuid = add_dynamic_network(&hot, &cold);
        ClaimBasedMinerEmissions::<Test>::insert(netuid, false);

        let _first = migrate_enable_claim_based_miner_emissions::<Test>();
        assert!(ClaimBasedMinerEmissions::<Test>::get(netuid));
        assert!(HasMigrationRun::<Test>::get(MIGRATION_NAME.to_vec()));

        ClaimBasedMinerEmissions::<Test>::insert(netuid, false);
        let _second = migrate_enable_claim_based_miner_emissions::<Test>();
        // Already-run migration must not re-enable a sudo off.
        assert!(!ClaimBasedMinerEmissions::<Test>::get(netuid));
    });
}

#[test]
fn init_new_network_enables_claim_based() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(42);
        SubtensorModule::init_new_network(netuid, 100);
        assert!(ClaimBasedMinerEmissions::<Test>::get(netuid));
    });
}

#[test]
fn claim_rejects_non_owner() {
    new_test_ext(0).execute_with(|| {
        let (_cold, hot, netuid) = setup_claim_subnet();
        let stranger = U256::from(999);
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot, 100u64.into());
        assert_noop!(
            SubtensorModule::claim_miner_emission(RuntimeOrigin::signed(stranger), hot, netuid),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn dissolve_then_hotkey_swap_preserves_pending_owner() {
    new_test_ext(0).execute_with(|| {
        let (cold, hot, netuid) = setup_claim_subnet();
        let new_hot = U256::from(77);
        SubtensorModule::create_account_if_non_existent(&cold, &new_hot).unwrap();

        let amount: AlphaBalance = 777_000u64.into();
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot, amount);

        // Queue dissolution the same way production does: drop NetworksAdded and
        // park the netuid on the cleanup queue while PendingMinerAlpha remains.
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));
        assert!(!NetworksAdded::<Test>::get(netuid));
        assert!(DissolveCleanupQueue::<Test>::get().contains(&netuid));
        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&hot, netuid),
            amount
        );

        // All-subnet swap with keep_stake=false retires Owner[old]. The dissolving
        // subnet is absent from get_all_subnet_netuids, so the dedicated dissolving
        // transfer must still move the pending row.
        let mut weight = Weight::from_parts(u64::MAX / 4, u64::MAX / 4);
        assert_ok!(SubtensorModule::perform_hotkey_swap_on_all_subnets(
            &hot,
            &new_hot,
            &cold,
            &mut weight,
            false,
        ));
        assert!(!Owner::<Test>::contains_key(hot));
        assert_eq!(Owner::<Test>::get(new_hot), cold);
        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&hot, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&new_hot, netuid),
            amount
        );

        SubtensorModule::force_payout_pending_miner_alpha(netuid, &new_hot);
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&new_hot, &cold, netuid),
            amount
        );
    });
}

#[test]
fn dissolve_pending_row_weight_covers_claim_envelope() {
    // The per-row dissolve meter must dominate the extrinsic claim weight so a
    // force payout cannot exceed what the cleanup phase reserved.
    let row = SubtensorModule::dissolve_pending_miner_alpha_row_weight();
    let claim = <Test as Config>::WeightInfo::claim_miner_emission();
    assert!(row.ref_time() >= claim.ref_time());
    assert!(row.proof_size() >= claim.proof_size());
}

#[test]
fn dissolve_resume_after_hotkey_swap_does_not_skip_rows() {
    new_test_ext(0).execute_with(|| {
        use frame_support::weights::WeightMeter;

        let (cold, hot_a, netuid) = setup_claim_subnet();
        let hot_b = U256::from(3);
        let hot_c = U256::from(4);
        let new_hot = U256::from(33);
        register_ok_neuron(netuid, hot_b, cold, 0);
        register_ok_neuron(netuid, hot_c, cold, 0);
        SubtensorModule::create_account_if_non_existent(&cold, &new_hot).unwrap();

        let amount: AlphaBalance = 10_000u64.into();
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot_a, amount);
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot_b, amount);
        SubtensorModule::accrue_pending_miner_alpha(netuid, &hot_c, amount);

        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        // Pay exactly one row with a tight meter, leaving two unpaid.
        let row_w = SubtensorModule::dissolve_pending_miner_alpha_row_weight();
        let mut meter = WeightMeter::with_limit(row_w);
        let (done, _) =
            SubtensorModule::dissolve_pending_miner_alpha_to_stake(netuid, &mut meter, None);
        assert!(!done);
        assert_eq!(
            SubtensorModule::get_pending_miner_emission_total(netuid),
            amount.saturating_mul(2u64.into())
        );

        // Pick an unpaid hotkey and rekey it. Prefix restart (not a hashed
        // cursor) must still visit the new key on resume.
        let unpaid = [hot_a, hot_b, hot_c]
            .into_iter()
            .find(|h| !SubtensorModule::get_pending_miner_emission(h, netuid).is_zero())
            .expect("expected an unpaid pending row");
        let mut weight = Weight::from_parts(u64::MAX / 4, u64::MAX / 4);
        assert_ok!(SubtensorModule::perform_hotkey_swap_on_all_subnets(
            &unpaid,
            &new_hot,
            &cold,
            &mut weight,
            false,
        ));
        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&new_hot, netuid),
            amount
        );

        let mut meter = WeightMeter::with_limit(Weight::from_parts(u64::MAX / 4, u64::MAX / 4));
        let (done, _) =
            SubtensorModule::dissolve_pending_miner_alpha_to_stake(netuid, &mut meter, None);
        assert!(done);
        assert_eq!(
            SubtensorModule::get_pending_miner_emission_total(netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_pending_miner_emission(&new_hot, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&new_hot, &cold, netuid),
            amount
        );
    });
}
