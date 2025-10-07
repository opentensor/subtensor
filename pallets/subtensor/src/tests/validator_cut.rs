#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use approx::assert_abs_diff_eq;
use frame_support::assert_ok;
use sp_core::U256;
use subtensor_runtime_common::{Currency as CurrencyT, NetUid, NetUidStorageIndex};

use super::mock::*;
use crate::*;

#[test]
fn test_emission_with_different_cut() {
    new_test_ext(1).execute_with(|| {
        let validator_coldkey = U256::from(1);
        let validator_hotkey = U256::from(2);
        let miner_coldkey = U256::from(5);
        let miner_hotkey = U256::from(6);
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let stake = 100_000_000_000;

        let cut_list = [
            0,
            10000,
            1000000,
            u64::MAX / 100,
            u64::MAX / 10,
            u64::MAX / 4,
            u64::MAX / 2,
            u64::MAX,
        ];

        // Add network, register hotkeys, and setup network parameters
        add_network(netuid, subnet_tempo, 0);
        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 2);
        SubtensorModule::add_balance_to_coldkey_account(
            &validator_coldkey,
            stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &miner_coldkey,
            stake + ExistentialDeposit::get(),
        );
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        SubnetOwnerCut::<Test>::set(0);
        // There are one validator and two neurons
        MaxAllowedUids::<Test>::set(netuid, 3);
        SubtensorModule::set_max_allowed_validators(netuid, 2);

        // Setup stakes:
        //   Stake from validator
        //   Stake from valiminer
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            stake.into()
        ));

        // Setup YUMA so that it creates emissions
        // set weight for two minder as same value, miner 1 and miner 2
        Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 0, vec![(1, 0xFFFF)]);
        // Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 1, vec![(2, 0xFFFF)]);

        BlockAtRegistration::<Test>::set(netuid, 0, 1);
        BlockAtRegistration::<Test>::set(netuid, 1, 1);
        // BlockAtRegistration::<Test>::set(netuid, 2, 1);
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX); // makes all stake active
        ValidatorPermit::<Test>::insert(netuid, vec![true, false]);

        // Run run_coinbase until emissions are drained

        for cut in cut_list {
            let validator_stake_before =
                SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey);
            let miner_stake_before = SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey);
            SubtensorModule::set_validator_cut(netuid, cut);

            step_block(subnet_tempo);

            // Verify how emission is split between keys
            //   - Owner cut is zero => 50% goes to miners and 50% goes to validators
            //   - Validator gets 25% because there are two validators
            //   - Valiminer gets 25% as a validator and 25% as miner
            //   - Miner gets 25% as miner
            let validator_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey)
                    - validator_stake_before;
            let miner_emission =
                SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey) - miner_stake_before;
            let total_emission = validator_emission + miner_emission;
            let total_emission_u128: u128 = total_emission.to_u64() as u128;

            let expected_validator_emission =
                ((total_emission_u128 * (cut as u128) / (u64::MAX as u128)) as u64).into();

            assert_abs_diff_eq!(
                validator_emission,
                expected_validator_emission,
                epsilon = 10.into()
            );
            assert_abs_diff_eq!(
                miner_emission,
                total_emission - expected_validator_emission,
                epsilon = 10.into()
            );
        }
    });
}

#[test]
fn test_emission_with_defualt_cut_2_validators_2_miners() {
    new_test_ext(1).execute_with(|| {
        let validator_coldkey = U256::from(1);
        let validator_hotkey = U256::from(2);
        let validator_miner_coldkey = U256::from(3);
        let validator_miner_hotkey = U256::from(4);
        let miner_coldkey = U256::from(5);
        let miner_hotkey = U256::from(6);
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let stake = 100_000_000_000;

        // Add network, register hotkeys, and setup network parameters
        add_network(netuid, subnet_tempo, 0);
        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        register_ok_neuron(netuid, validator_miner_hotkey, validator_miner_coldkey, 1);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 2);
        SubtensorModule::add_balance_to_coldkey_account(
            &validator_coldkey,
            stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &validator_miner_coldkey,
            stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &miner_coldkey,
            stake + ExistentialDeposit::get(),
        );
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        SubnetOwnerCut::<Test>::set(0);
        // There are two validators and three neurons
        MaxAllowedUids::<Test>::set(netuid, 3);
        SubtensorModule::set_max_allowed_validators(netuid, 2);

        // Setup stakes:
        //   Stake from validator
        //   Stake from valiminer
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator_miner_coldkey),
            validator_miner_hotkey,
            netuid,
            stake.into()
        ));

        // Setup YUMA so that it creates emissions
        // set weight for two minder as same value, miner 1 and miner 2
        Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 0, vec![(1, 0xFFFF)]);
        Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 1, vec![(2, 0xFFFF)]);

        BlockAtRegistration::<Test>::set(netuid, 0, 1);
        BlockAtRegistration::<Test>::set(netuid, 1, 1);
        BlockAtRegistration::<Test>::set(netuid, 2, 1);
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX); // makes all stake active
        ValidatorPermit::<Test>::insert(netuid, vec![true, true, false]);

        // Run run_coinbase until emissions are drained
        let validator_stake_before =
            SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey);
        let valiminer_stake_before =
            SubtensorModule::get_total_stake_for_coldkey(&validator_miner_coldkey);
        let miner_stake_before = SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey);

        step_block(subnet_tempo);

        // Verify how emission is split between keys
        //   - Owner cut is zero => 50% goes to miners and 50% goes to validators
        //   - Validator gets 25% because there are two validators
        //   - Valiminer gets 25% as a validator and 25% as miner
        //   - Miner gets 25% as miner
        let validator_emission = SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey)
            - validator_stake_before;
        let valiminer_emission =
            SubtensorModule::get_total_stake_for_coldkey(&validator_miner_coldkey)
                - valiminer_stake_before;
        let miner_emission =
            SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey) - miner_stake_before;
        let total_emission = validator_emission + valiminer_emission + miner_emission;

        assert_abs_diff_eq!(
            validator_emission,
            total_emission / 4.into(),
            epsilon = 10.into()
        );
        assert_abs_diff_eq!(
            valiminer_emission,
            total_emission / 2.into(),
            epsilon = 10.into()
        );
        assert_abs_diff_eq!(
            miner_emission,
            total_emission / 4.into(),
            epsilon = 10.into()
        );
    });
}
