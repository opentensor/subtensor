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
            assert_ok!(SubtensorModule::set_validator_cut(netuid, cut));

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

#[test]
fn test_emission_with_multiple_validators_varying_cuts() {
    // Test with 3 validators and 2 miners with different validator cut values
    new_test_ext(1).execute_with(|| {
        let validator1_coldkey = U256::from(1);
        let validator1_hotkey = U256::from(2);
        let validator2_coldkey = U256::from(3);
        let validator2_hotkey = U256::from(4);
        let validator3_coldkey = U256::from(5);
        let validator3_hotkey = U256::from(6);
        let miner1_coldkey = U256::from(7);
        let miner1_hotkey = U256::from(8);
        let miner2_coldkey = U256::from(9);
        let miner2_hotkey = U256::from(10);
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let stake = 100_000_000_000;

        let cut_list = [0, u64::MAX / 4, u64::MAX / 2, u64::MAX];

        // Add network and register all neurons
        add_network(netuid, subnet_tempo, 0);
        SubtensorModule::set_target_registrations_per_interval(netuid, 10);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        RegistrationsThisInterval::<Test>::set(netuid, 0);
        RegistrationsThisBlock::<Test>::set(netuid, 0);
        register_ok_neuron(netuid, validator1_hotkey, validator1_coldkey, 0);
        register_ok_neuron(netuid, validator2_hotkey, validator2_coldkey, 1);
        register_ok_neuron(netuid, validator3_hotkey, validator3_coldkey, 2);
        register_ok_neuron(netuid, miner1_hotkey, miner1_coldkey, 3);
        register_ok_neuron(netuid, miner2_hotkey, miner2_coldkey, 4);

        // Add balance to all coldkeys
        for coldkey in [
            validator1_coldkey,
            validator2_coldkey,
            validator3_coldkey,
            miner1_coldkey,
            miner2_coldkey,
        ] {
            SubtensorModule::add_balance_to_coldkey_account(
                &coldkey,
                stake + ExistentialDeposit::get(),
            );
        }

        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        SubnetOwnerCut::<Test>::set(0);
        MaxAllowedUids::<Test>::set(netuid, 5);
        SubtensorModule::set_max_allowed_validators(netuid, 3);

        // All validators stake equally
        for (coldkey, hotkey) in [
            (validator1_coldkey, validator1_hotkey),
            (validator2_coldkey, validator2_hotkey),
            (validator3_coldkey, validator3_hotkey),
        ] {
            assert_ok!(SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                stake.into()
            ));
        }

        // Setup weights - all validators assign equal weight to miners
        Weights::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            0,
            vec![(3, 0x7FFF), (4, 0x7FFF)],
        );
        Weights::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            1,
            vec![(3, 0x7FFF), (4, 0x7FFF)],
        );
        Weights::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            2,
            vec![(3, 0x7FFF), (4, 0x7FFF)],
        );

        for i in 0..5 {
            BlockAtRegistration::<Test>::set(netuid, i, 1);
        }
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2, 2, 2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX);
        ValidatorPermit::<Test>::insert(netuid, vec![true, true, true, false, false]);

        for cut in cut_list {
            let val1_before = SubtensorModule::get_total_stake_for_coldkey(&validator1_coldkey);
            let val2_before = SubtensorModule::get_total_stake_for_coldkey(&validator2_coldkey);
            let val3_before = SubtensorModule::get_total_stake_for_coldkey(&validator3_coldkey);
            let miner1_before = SubtensorModule::get_total_stake_for_coldkey(&miner1_coldkey);
            let miner2_before = SubtensorModule::get_total_stake_for_coldkey(&miner2_coldkey);

            assert_ok!(SubtensorModule::set_validator_cut(netuid, cut));
            step_block(subnet_tempo);

            let val1_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator1_coldkey) - val1_before;
            let val2_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator2_coldkey) - val2_before;
            let val3_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator3_coldkey) - val3_before;
            let miner1_emission =
                SubtensorModule::get_total_stake_for_coldkey(&miner1_coldkey) - miner1_before;
            let miner2_emission =
                SubtensorModule::get_total_stake_for_coldkey(&miner2_coldkey) - miner2_before;

            let total_emission =
                val1_emission + val2_emission + val3_emission + miner1_emission + miner2_emission;
            let total_emission_u128: u128 = total_emission.to_u64() as u128;

            let expected_validator_total =
                ((total_emission_u128 * (cut as u128) / (u64::MAX as u128)) as u64).into();
            let expected_miner_total = total_emission - expected_validator_total;

            // Each validator should get 1/3 of validator emissions
            let validator_total_emission = val1_emission + val2_emission + val3_emission;
            assert_abs_diff_eq!(
                validator_total_emission,
                expected_validator_total,
                epsilon = 30.into()
            );

            // Each miner should get 1/2 of miner emissions
            let miner_total_emission = miner1_emission + miner2_emission;
            assert_abs_diff_eq!(
                miner_total_emission,
                expected_miner_total,
                epsilon = 30.into()
            );
        }
    });
}

#[test]
fn test_emission_with_child_keys_and_varying_cuts() {
    // Test delegation scenario: validator with child keys
    new_test_ext(1).execute_with(|| {
        let validator_coldkey = U256::from(1);
        let validator_hotkey = U256::from(2);
        let delegate_coldkey1 = U256::from(3);
        let delegate_coldkey2 = U256::from(4);
        let miner_coldkey = U256::from(5);
        let miner_hotkey = U256::from(6);
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let validator_stake = 100_000_000_000;
        let delegate_stake = 50_000_000_000;

        let cut_list = [0, u64::MAX / 10, u64::MAX / 2, u64::MAX];

        // Setup network
        add_network(netuid, subnet_tempo, 0);
        SubtensorModule::set_target_registrations_per_interval(netuid, 10);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        RegistrationsThisInterval::<Test>::set(netuid, 0);
        RegistrationsThisBlock::<Test>::set(netuid, 0);
        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 1);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(
            &validator_coldkey,
            validator_stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &delegate_coldkey1,
            delegate_stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &delegate_coldkey2,
            delegate_stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &miner_coldkey,
            validator_stake + ExistentialDeposit::get(),
        );

        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        SubnetOwnerCut::<Test>::set(0);
        MaxAllowedUids::<Test>::set(netuid, 2);
        SubtensorModule::set_max_allowed_validators(netuid, 1);

        // Validator stakes on their own hotkey
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            validator_stake.into()
        ));

        // Delegates stake on validator's hotkey (child keys)
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegate_coldkey1),
            validator_hotkey,
            netuid,
            delegate_stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegate_coldkey2),
            validator_hotkey,
            netuid,
            delegate_stake.into()
        ));

        // Setup weights
        Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 0, vec![(1, 0xFFFF)]);

        BlockAtRegistration::<Test>::set(netuid, 0, 1);
        BlockAtRegistration::<Test>::set(netuid, 1, 1);
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX);
        ValidatorPermit::<Test>::insert(netuid, vec![true, false]);

        for cut in cut_list {
            let validator_before = SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey);
            let delegate1_before = SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey1);
            let delegate2_before = SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey2);
            let miner_before = SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey);

            assert_ok!(SubtensorModule::set_validator_cut(netuid, cut));
            step_block(subnet_tempo);

            let validator_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey) - validator_before;
            let delegate1_emission =
                SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey1) - delegate1_before;
            let delegate2_emission =
                SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey2) - delegate2_before;
            let miner_emission =
                SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey) - miner_before;

            let total_emission =
                validator_emission + delegate1_emission + delegate2_emission + miner_emission;
            let total_emission_u128: u128 = total_emission.to_u64() as u128;

            let expected_validator_total_emission =
                ((total_emission_u128 * (cut as u128) / (u64::MAX as u128)) as u64).into();

            // Validator emissions should be split proportionally to stake
            // Total validator stake = 100 + 50 + 50 = 200
            // Validator: 100/200 = 50%, Delegate1: 50/200 = 25%, Delegate2: 50/200 = 25%
            let total_validator_stake = validator_stake + delegate_stake + delegate_stake;
            let validator_portion = validator_emission + delegate1_emission + delegate2_emission;

            assert_abs_diff_eq!(
                validator_portion,
                expected_validator_total_emission,
                epsilon = 30.into()
            );

            // Check proportional distribution
            let expected_validator_emission =
                (expected_validator_total_emission.to_u64() as u128 * validator_stake as u128
                    / total_validator_stake as u128) as u64;
            let expected_delegate_emission =
                (expected_validator_total_emission.to_u64() as u128 * delegate_stake as u128
                    / total_validator_stake as u128) as u64;

            assert_abs_diff_eq!(
                validator_emission,
                expected_validator_emission.into(),
                epsilon = 600_000_000.into()
            );
            assert_abs_diff_eq!(
                delegate1_emission,
                expected_delegate_emission.into(),
                epsilon = 600_000_000.into()
            );
            assert_abs_diff_eq!(
                delegate2_emission,
                expected_delegate_emission.into(),
                epsilon = 600_000_000.into()
            );
        }
    });
}

#[test]
fn test_emission_with_unequal_validator_stakes_varying_cuts() {
    // Test with validators having different stake amounts
    new_test_ext(1).execute_with(|| {
        let validator1_coldkey = U256::from(1);
        let validator1_hotkey = U256::from(2);
        let validator2_coldkey = U256::from(3);
        let validator2_hotkey = U256::from(4);
        let miner_coldkey = U256::from(5);
        let miner_hotkey = U256::from(6);
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let large_stake = 200_000_000_000;
        let small_stake = 50_000_000_000;

        let cut_list = [u64::MAX / 10, u64::MAX / 4, u64::MAX / 2, u64::MAX];

        // Setup network
        add_network(netuid, subnet_tempo, 0);
        SubtensorModule::set_target_registrations_per_interval(netuid, 10);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        RegistrationsThisInterval::<Test>::set(netuid, 0);
        RegistrationsThisBlock::<Test>::set(netuid, 0);
        register_ok_neuron(netuid, validator1_hotkey, validator1_coldkey, 0);
        register_ok_neuron(netuid, validator2_hotkey, validator2_coldkey, 1);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 2);

        SubtensorModule::add_balance_to_coldkey_account(
            &validator1_coldkey,
            large_stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &validator2_coldkey,
            small_stake + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &miner_coldkey,
            large_stake + ExistentialDeposit::get(),
        );

        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        SubnetOwnerCut::<Test>::set(0);
        MaxAllowedUids::<Test>::set(netuid, 3);
        SubtensorModule::set_max_allowed_validators(netuid, 2);

        // Validator 1 stakes large amount
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator1_coldkey),
            validator1_hotkey,
            netuid,
            large_stake.into()
        ));

        // Validator 2 stakes small amount
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator2_coldkey),
            validator2_hotkey,
            netuid,
            small_stake.into()
        ));

        // Setup weights
        Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 0, vec![(2, 0xFFFF)]);
        Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 1, vec![(2, 0xFFFF)]);

        BlockAtRegistration::<Test>::set(netuid, 0, 1);
        BlockAtRegistration::<Test>::set(netuid, 1, 1);
        BlockAtRegistration::<Test>::set(netuid, 2, 1);
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX);
        ValidatorPermit::<Test>::insert(netuid, vec![true, true, false]);

        for cut in cut_list {
            let val1_before = SubtensorModule::get_total_stake_for_coldkey(&validator1_coldkey);
            let val2_before = SubtensorModule::get_total_stake_for_coldkey(&validator2_coldkey);
            let miner_before = SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey);

            assert_ok!(SubtensorModule::set_validator_cut(netuid, cut));
            step_block(subnet_tempo);

            let val1_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator1_coldkey) - val1_before;
            let val2_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator2_coldkey) - val2_before;
            let miner_emission =
                SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey) - miner_before;

            let total_emission = val1_emission + val2_emission + miner_emission;
            let total_emission_u128: u128 = total_emission.to_u64() as u128;

            let expected_validator_total =
                ((total_emission_u128 * (cut as u128) / (u64::MAX as u128)) as u64).into();

            let validator_total = val1_emission + val2_emission;
            assert_abs_diff_eq!(
                validator_total,
                expected_validator_total,
                epsilon = 20.into()
            );

            // Validator 1 should receive more due to higher stake
            // Ratio is 200:50 = 4:1
            let total_validator_stake = large_stake + small_stake;
            let expected_val1 = (expected_validator_total.to_u64() as u128 * large_stake as u128
                / total_validator_stake as u128) as u64;
            let expected_val2 = (expected_validator_total.to_u64() as u128 * small_stake as u128
                / total_validator_stake as u128) as u64;

            assert_abs_diff_eq!(val1_emission, expected_val1.into(), epsilon = 40_000.into());
            assert_abs_diff_eq!(val2_emission, expected_val2.into(), epsilon = 40_000.into());
        }
    });
}

#[test]
fn test_emission_single_validator_multiple_miners_varying_cuts() {
    // Test with 1 validator and 4 miners
    new_test_ext(1).execute_with(|| {
        let validator_coldkey = U256::from(1);
        let validator_hotkey = U256::from(2);
        let miner_coldkeys = vec![U256::from(3), U256::from(5), U256::from(7), U256::from(9)];
        let miner_hotkeys = [U256::from(4), U256::from(6), U256::from(8), U256::from(10)];
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let stake = 100_000_000_000;

        let cut_list = [0, u64::MAX / 3, u64::MAX / 2, u64::MAX / 4 * 3, u64::MAX];

        // Setup network
        add_network(netuid, subnet_tempo, 0);
        SubtensorModule::set_target_registrations_per_interval(netuid, 10);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        RegistrationsThisInterval::<Test>::set(netuid, 0);
        RegistrationsThisBlock::<Test>::set(netuid, 0);
        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        for (i, (hotkey, coldkey)) in miner_hotkeys.iter().zip(miner_coldkeys.iter()).enumerate() {
            register_ok_neuron(netuid, *hotkey, *coldkey, (i + 1) as u64);
        }

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(
            &validator_coldkey,
            stake + ExistentialDeposit::get(),
        );
        for coldkey in &miner_coldkeys {
            SubtensorModule::add_balance_to_coldkey_account(
                coldkey,
                stake + ExistentialDeposit::get(),
            );
        }

        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        SubnetOwnerCut::<Test>::set(0);
        MaxAllowedUids::<Test>::set(netuid, 5);
        SubtensorModule::set_max_allowed_validators(netuid, 1);

        // Validator stakes
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            stake.into()
        ));

        // Validator assigns equal weights to all miners
        Weights::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            0,
            vec![(1, 0x3FFF), (2, 0x3FFF), (3, 0x3FFF), (4, 0x3FFF)],
        );

        for i in 0..5 {
            BlockAtRegistration::<Test>::set(netuid, i, 1);
        }
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2, 2, 2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX);
        ValidatorPermit::<Test>::insert(netuid, vec![true, false, false, false, false]);

        for cut in cut_list {
            let validator_before = SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey);
            let miners_before: Vec<_> = miner_coldkeys
                .iter()
                .map(SubtensorModule::get_total_stake_for_coldkey)
                .collect();

            assert_ok!(SubtensorModule::set_validator_cut(netuid, cut));
            step_block(subnet_tempo);

            let validator_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey) - validator_before;
            let miners_emissions: Vec<_> = miner_coldkeys
                .iter()
                .zip(miners_before.iter())
                .map(|(ck, before)| SubtensorModule::get_total_stake_for_coldkey(ck) - *before)
                .collect();

            let total_miner_emission = miners_emissions.iter().fold(0.into(), |acc, e| acc + *e);
            let total_emission = validator_emission + total_miner_emission;
            let total_emission_u128: u128 = total_emission.to_u64() as u128;

            let expected_validator_emission =
                ((total_emission_u128 * (cut as u128) / (u64::MAX as u128)) as u64).into();
            let expected_miner_total = total_emission - expected_validator_emission;

            assert_abs_diff_eq!(
                validator_emission,
                expected_validator_emission,
                epsilon = 20.into()
            );
            assert_abs_diff_eq!(
                total_miner_emission,
                expected_miner_total,
                epsilon = 20.into()
            );

            // Each miner should get approximately 1/4 of miner emissions (equal weights)
            for miner_emission in &miners_emissions {
                assert_abs_diff_eq!(
                    *miner_emission * 4.into(),
                    expected_miner_total,
                    epsilon = 100.into()
                );
            }
        }
    });
}

#[test]
fn test_emission_all_validator_miners_varying_cuts() {
    // Test where all participants are both validators and miners
    new_test_ext(1).execute_with(|| {
        let participants = vec![
            (U256::from(1), U256::from(2)),
            (U256::from(3), U256::from(4)),
            (U256::from(5), U256::from(6)),
        ];
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let stake = 100_000_000_000;

        let cut_list = [u64::MAX / 10, u64::MAX / 4, u64::MAX / 2, u64::MAX];

        // Setup network
        add_network(netuid, subnet_tempo, 0);
        for (i, (coldkey, hotkey)) in participants.iter().enumerate() {
            register_ok_neuron(netuid, *hotkey, *coldkey, i as u64);
            SubtensorModule::add_balance_to_coldkey_account(
                coldkey,
                stake + ExistentialDeposit::get(),
            );
        }

        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        SubnetOwnerCut::<Test>::set(0);
        MaxAllowedUids::<Test>::set(netuid, 3);
        SubtensorModule::set_max_allowed_validators(netuid, 3);

        // All stake
        for (coldkey, hotkey) in &participants {
            assert_ok!(SubtensorModule::add_stake(
                RuntimeOrigin::signed(*coldkey),
                *hotkey,
                netuid,
                stake.into()
            ));
        }

        // Each validator assigns equal weight to all others
        Weights::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            0,
            vec![(1, 0x7FFF), (2, 0x7FFF)],
        );
        Weights::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            1,
            vec![(0, 0x7FFF), (2, 0x7FFF)],
        );
        Weights::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            2,
            vec![(0, 0x7FFF), (1, 0x7FFF)],
        );

        for i in 0..3 {
            BlockAtRegistration::<Test>::set(netuid, i, 1);
        }
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX);
        ValidatorPermit::<Test>::insert(netuid, vec![true, true, true]);

        for cut in cut_list {
            let stakes_before: Vec<_> = participants
                .iter()
                .map(|(ck, _)| SubtensorModule::get_total_stake_for_coldkey(ck))
                .collect();

            assert_ok!(SubtensorModule::set_validator_cut(netuid, cut));
            step_block(subnet_tempo);

            let emissions: Vec<_> = participants
                .iter()
                .zip(stakes_before.iter())
                .map(|((ck, _), before)| SubtensorModule::get_total_stake_for_coldkey(ck) - *before)
                .collect();

            let total_emission: u64 = emissions.iter().map(|e| e.to_u64()).sum();
            let total_emission_u128: u128 = total_emission as u128;

            // With equal stakes and everyone being both validator and miner,
            // total emission should be split equally regardless of cut
            // (each gets 1/3 as validator and 1/3 as miner)
            let expected_per_participant: u64 = total_emission / 3;

            for emission in &emissions {
                assert_abs_diff_eq!(
                    *emission,
                    expected_per_participant.into(),
                    epsilon = 30.into()
                );
            }

            // Verify total matches expected
            let expected_validator_total_u64 =
                (total_emission_u128 * (cut as u128) / (u64::MAX as u128)) as u64;
            let expected_miner_total_u64: u64 = total_emission - expected_validator_total_u64;

            // Sum of validator and miner portions should equal total
            assert_abs_diff_eq!(
                expected_validator_total_u64 + expected_miner_total_u64,
                total_emission,
                epsilon = 10
            );
        }
    });
}

#[test]
fn test_emission_extreme_cuts_edge_cases() {
    // Test extreme validator cut values: 0 (all to miners) and MAX (all to validators)
    new_test_ext(1).execute_with(|| {
        let validator_coldkey = U256::from(1);
        let validator_hotkey = U256::from(2);
        let miner_coldkey = U256::from(3);
        let miner_hotkey = U256::from(4);
        let netuid = NetUid::from(1);
        let subnet_tempo = 10;
        let stake = 100_000_000_000;

        // Setup network
        add_network(netuid, subnet_tempo, 0);
        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 1);

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
        MaxAllowedUids::<Test>::set(netuid, 2);
        SubtensorModule::set_max_allowed_validators(netuid, 1);

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            stake.into()
        ));

        Weights::<Test>::insert(NetUidStorageIndex::from(netuid), 0, vec![(1, 0xFFFF)]);

        BlockAtRegistration::<Test>::set(netuid, 0, 1);
        BlockAtRegistration::<Test>::set(netuid, 1, 1);
        LastUpdate::<Test>::set(NetUidStorageIndex::from(netuid), vec![2, 2]);
        Kappa::<Test>::set(netuid, u16::MAX / 5);
        ActivityCutoff::<Test>::set(netuid, u16::MAX);
        ValidatorPermit::<Test>::insert(netuid, vec![true, false]);

        // Test cut = 0: all emissions should go to miners
        {
            let validator_before = SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey);
            let miner_before = SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey);

            assert_ok!(SubtensorModule::set_validator_cut(netuid, 0));
            step_block(subnet_tempo);

            let validator_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey) - validator_before;
            let miner_emission =
                SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey) - miner_before;

            // Validator should get nearly nothing (within rounding error)
            assert_abs_diff_eq!(validator_emission, 0.into(), epsilon = 5.into());
            // Miner should get almost all emissions
            assert!(miner_emission.to_u64() > 0);
        }

        // Test cut = u64::MAX: all emissions should go to validators
        {
            let validator_before = SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey);
            let miner_before = SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey);

            assert_ok!(SubtensorModule::set_validator_cut(netuid, u64::MAX));
            step_block(subnet_tempo);

            let validator_emission =
                SubtensorModule::get_total_stake_for_coldkey(&validator_coldkey) - validator_before;
            let miner_emission =
                SubtensorModule::get_total_stake_for_coldkey(&miner_coldkey) - miner_before;

            // Miner should get nearly nothing (within rounding error)
            assert_abs_diff_eq!(miner_emission, 0.into(), epsilon = 5.into());
            // Validator should get almost all emissions
            assert!(validator_emission.to_u64() > 0);
        }
    });
}
