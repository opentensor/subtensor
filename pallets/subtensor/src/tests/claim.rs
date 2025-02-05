#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use frame_support::assert_ok;

use super::mock::*;
use crate::*;
use approx::assert_abs_diff_eq;
use sp_core::{Get, U256};
use substrate_fixed::types::U64F64;

// cargo test --package pallet-subtensor --lib -- tests::claim::test_increase_claimable_ok --exact --show-output
#[test]
fn test_increase_claimable_ok() {
    new_test_ext(1).execute_with(|| {
        let amount = 1_000_000; // 500k min + 500k fee => 500k stake
        let (netuid, _coldkey, hotkey) = setup_network_with_stake(amount);
        let actual_stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid);
        SubtensorModule::increase_root_claimable_for_hotkey_and_subnet(&hotkey, netuid, 500_000);
        let expected_claimable_rate: U64F64 =
            U64F64::from_num(500_000) / U64F64::from_num(actual_stake);

        assert_eq!(
            RootClaimable::<Test>::get(hotkey, netuid),
            expected_claimable_rate,
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::claim::test_increase_claimable_overflow --exact --show-output
#[test]
fn test_increase_claimable_overflow() {
    new_test_ext(1).execute_with(|| {
        let amount = 1_000_000;
        let (netuid, _coldkey, hotkey) = setup_network_with_stake(amount);
        let actual_stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid);
        SubtensorModule::increase_root_claimable_for_hotkey_and_subnet(&hotkey, netuid, u64::MAX);
        let expected_claimable_rate: U64F64 =
            U64F64::from_num(u64::MAX) / U64F64::from_num(actual_stake);

        assert_eq!(
            RootClaimable::<Test>::get(hotkey, netuid),
            expected_claimable_rate,
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::claim::test_increase_claimable_underflow --exact --show-output
#[test]
fn test_increase_claimable_underflow() {
    new_test_ext(1).execute_with(|| {
        let amount = 1_000_000_000_000;
        let (netuid, _coldkey, hotkey) = setup_network_with_stake(amount);
        let actual_stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid);
        SubtensorModule::increase_root_claimable_for_hotkey_and_subnet(&hotkey, netuid, 500_000); // min stake
        let expected_claimable_rate: U64F64 =
            U64F64::from_num(500_000) / U64F64::from_num(actual_stake);

        assert_eq!(
            RootClaimable::<Test>::get(hotkey, netuid),
            expected_claimable_rate,
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::claim::test_claim_keep --exact --show-output
#[test]
fn test_claim_keep() {
    new_test_ext(1).execute_with(|| {
        // Test cases: (stake_amount, claimable_amount, root_stake_amount)
        [
            (100_000_000, 100_000_000, 100_000_000),
            (1_000_000_000, 1_000_000_000, 1_000_000_000),
            (100_000_000_000, 100_000_000_000, 100_000_000_000),
            (100_000_000_000, 200_000_000_000, 300_000_000_000),
            (300_000_000_000, 200_000_000_000, 100_000_000_000),
            (1_000_000_000, 200_000_000_000, 100_000_000_000),
            (1_000_000, 1_000_000, 1_000_000),
            (1_000_000, 1_000_000, 1_000_000_000_000),
            (1_000_000_000_000, 1_000_000, 1_000_000),
            (1_000_000, 1_000_000, 1_000_000_000_000),
            (1_000_000, 1_000_000_000_000, 1_000_000_000_000),
            (1_000_000_000_000, 1_000_000, 1_000_000_000_000),
            (1_000_000_000_000, 1_000_000_000_000, 1_000_000),
            (1_000_000_000_000, 1_000_000_000_000, 1_000_000_000_000),
            (10_000_000_000_000, 10_000_000_000_000, 10_000_000_000_000),
            (
                100_000_000_000_000,
                100_000_000_000_000,
                100_000_000_000_000,
            ),
            (
                1_000_000_000_000_000,
                1_000_000_000_000_000,
                1_000_000_000_000_000,
            ),
            (
                21_000_000_000_000_000,
                100_000_000_000_000_000,
                21_000_000_000_000_000,
            ),
        ]
        .iter()
        .for_each(|&(stake_amount, claimable_amount, root_stake_amount)| {
            let (netuid, coldkey, hotkey) = setup_network_with_stake(stake_amount);
            SubtensorModule::increase_root_claimable_for_hotkey_and_subnet(
                &hotkey,
                netuid,
                claimable_amount,
            );

            // Add root stake
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                SubtensorModule::get_root_netuid(),
                root_stake_amount,
            );

            let alpha_stake_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
            let root_stake_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                SubtensorModule::get_root_netuid(),
            );
            let claimable_rate: U64F64 = RootClaimable::<Test>::get(hotkey, netuid);

            SubtensorModule::root_claim_on_subnet(
                &hotkey,
                &coldkey,
                netuid,
                RootClaimTypeEnum::Keep,
            );

            let alpha_stake_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid,
            );
            let root_stake_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                SubtensorModule::get_root_netuid(),
            );

            let expected_stake_increase =
                (U64F64::from_num(root_stake_amount) * claimable_rate).to_num::<u64>();

            // We neglect 13 binary digits, which is 8192
            let tolerance = 8_192;

            // Check the new subnet stake
            assert_abs_diff_eq!(
                alpha_stake_after - alpha_stake_before,
                expected_stake_increase,
                epsilon = (expected_stake_increase / 1_000_000).max(tolerance)
            );

            // Check that debt catches up claimable
            assert_abs_diff_eq!(
                SubtensorModule::get_root_owed_for_hotkey_coldkey(&hotkey, &coldkey, netuid)
                    .to_num::<u64>(),
                0,
                epsilon = tolerance
            );

            // Root stake hasn't changed
            assert_eq!(root_stake_after, root_stake_before);

            // Remove root stake for the next test
            SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                SubtensorModule::get_root_netuid(),
                root_stake_amount,
            );
        });
    });
}

// cargo test --package pallet-subtensor --lib -- tests::claim::test_auto_claim --exact --show-output
#[test]
fn test_auto_claim() {
    new_test_ext(1).execute_with(|| {
        let amount = 1_000_000_000_000;
        let own_validator_stake = 1_000_000_000_000;
        let root_stake_amount = 1_000_000_000_000;
        let nominator_stake = 1_000_000_000_000;
        let (netuid, _coldkey, hotkey) = setup_network_with_stake(amount);

        // Add another validator (with stake)
        let validator_coldkey = U256::from(3);
        let validator_hotkey = U256::from(4);
        let nominator = U256::from(5);
        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(
            &validator_coldkey,
            own_validator_stake + ExistentialDeposit::get(),
        );
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            own_validator_stake
        ));

        // Add nominator stake
        SubtensorModule::add_balance_to_coldkey_account(
            &nominator,
            nominator_stake + ExistentialDeposit::get(),
        );
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(nominator),
            validator_hotkey,
            netuid,
            nominator_stake
        ));
        ColdkeysIndex::<Test>::insert(1, validator_coldkey);
        RootClaimType::<Test>::insert(nominator, RootClaimTypeEnum::Keep);

        println!("RootClaimable::<T>::get(nominator, netuid) = {:?}", RootClaimable::<Test>::get(nominator, netuid));

        // Set YUMA up for emissions
        Weights::<Test>::insert(netuid, 0, vec![(1, 0xFFFF)]);
        Weights::<Test>::insert(netuid, 1, vec![(2, 0xFFFF)]);
        BlockAtRegistration::<Test>::set(netuid, 0, 1);
        BlockAtRegistration::<Test>::set(netuid, 1, 1);
        LastUpdate::<Test>::set(netuid, vec![2, 2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        Tempo::<Test>::insert(netuid, 100);

        let alpha_stake_before =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &nominator, netuid);

        // Run an epoch
        step_epochs(1, netuid);

        let alpha_stake_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &nominator, netuid);

        println!("alpha_stake_after = {:?}", alpha_stake_after);
        println!("alpha_stake_before = {:?}", alpha_stake_before);


        // Add root nominator stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &nominator,
            SubtensorModule::get_root_netuid(),
            root_stake_amount,
        );

        // Run an epoch
        step_epochs(1, netuid);

        let alpha_stake_after2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &nominator, netuid);

        println!("alpha_stake_after2 = {:?}", alpha_stake_after2);
        println!("alpha_stake_after = {:?}", alpha_stake_after);
        println!("alpha_stake_before = {:?}", alpha_stake_before);

    });
}

fn setup_network_with_stake(tao_stake: u64) -> (u16, U256, U256) {
    let subnet_owner_coldkey = U256::from(1001);
    let subnet_owner_hotkey = U256::from(1002);
    let coldkey_account_id = U256::from(1);
    let hotkey_account_id = U256::from(2);
    let fee = DefaultStakingFee::<Test>::get();
    let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
    register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

    // Mock liquidity to avoid insufficient liquidity errors
    SubnetTAO::<Test>::insert(netuid, 1_000_000_000_000_000);
    SubnetAlphaIn::<Test>::insert(netuid, 1_000_000_000_000_000);

    SubtensorModule::add_balance_to_coldkey_account(
        &coldkey_account_id,
        tao_stake + fee + ExistentialDeposit::get(),
    );

    assert_ok!(SubtensorModule::add_stake(
        RuntimeOrigin::signed(coldkey_account_id),
        hotkey_account_id,
        netuid,
        tao_stake + fee
    ));

    (netuid, coldkey_account_id, hotkey_account_id)
}
