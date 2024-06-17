use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::*;
use sp_core::{H256, U256};

/***********************************************************
    staking::add_subnet_stake() tests
************************************************************/

// To run just the tests in this file, use the following command:
// cargo test -p pallet-subtensor --test staking

#[test]
#[cfg(not(tarpaulin))]
fn test_add_subnet_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let amount_staked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_subnet_stake {
            hotkey,
            netuid,
            amount_staked,
        });
        let disp_info = call.get_dispatch_info();
        assert!(disp_info.weight.ref_time() != 0);
        assert_eq!(disp_info.class, DispatchClass::Normal,);
        assert_eq!(disp_info.pays_fee, Pays::No,);
    });
}

#[test]
fn test_add_subnet_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        // Check we have zero staked before transfer
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id),
            0
        );

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id),
            10000
        );

        // Check if balance has decreased
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            ExistentialDeposit::get()
        );
    });
}

#[test]
fn test_dividends_with_run_to_block() {
    new_test_ext(1).execute_with(|| {
        let neuron_src_hotkey_id = U256::from(1);
        let neuron_dest_hotkey_id = U256::from(2);
        let coldkey_account_id = U256::from(667);
        let netuid: u16 = 1;

        let initial_stake: u64 = 5000;

        //add network
        add_network(netuid, 13, 0);

        // Register neuron, this will set a self weight
        SubtensorModule::set_max_registrations_per_block(netuid, 3);
        SubtensorModule::set_max_allowed_uids(1, 5);

        register_ok_neuron(netuid, U256::from(0), coldkey_account_id, 2112321);
        register_ok_neuron(netuid, neuron_src_hotkey_id, coldkey_account_id, 192213123);
        register_ok_neuron(netuid, neuron_dest_hotkey_id, coldkey_account_id, 12323);

        // Add some stake to the hotkey account, so we can test for emission before the transfer takes place
        SubtensorModule::increase_subnet_token_on_hotkey_account(
            &neuron_src_hotkey_id,
            netuid,
            initial_stake,
        );

        // Check if the initial stake has arrived
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&neuron_src_hotkey_id),
            initial_stake
        );

        // Check if all three neurons are registered
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);

        // Run a couple of blocks to check if emission works
        run_to_block(2);

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&neuron_src_hotkey_id),
            initial_stake
        );

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&neuron_dest_hotkey_id),
            0
        );
    });
}

#[test]
fn test_add_subnet_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey_account_id = U256::from(654); // bogus
        let amount = 20000; // Not used

        let result = SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            netuid,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_add_subnet_stake_not_registered_key_pair() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey_account_id = U256::from(435445);
        let hotkey_account_id = U256::from(54544);
        let amount = 1337;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1800);
        add_network(netuid, 0, 0);
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
    });
}

#[test]
fn test_add_subnet_stake_err_neuron_does_not_belong_to_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let other_cold_key = U256::from(99498);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&other_cold_key, 100000);

        // Perform the request which is signed by a different cold key
        let result = SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            netuid,
            1000,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
    });
}

#[test]
fn test_add_subnet_stake_err_not_enough_belance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Lets try to stake with 0 balance in cold key account
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_id), 0);
        let result = SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            netuid,
            60000,
        );

        assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
    });
}

#[test]
#[ignore]
fn test_add_subnet_stake_total_balance_no_change() {
    // When we add stake, the total balance of the coldkey account should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(551337);
        let coldkey_account_id = U256::from(51337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        let initial_balance = 10000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total balance has remained the same. (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, initial_total_balance);
    });
}

#[test]
#[ignore]
fn test_add_subnet_stake_total_issuance_no_change() {
    // When we add stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        let initial_balance = 10000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Check total issuance is equal to initial balance
        let initial_total_issuance = Balances::total_issuance();
        assert_eq!(initial_total_issuance, initial_balance);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total issuance has remained the same. (no fee, includes reserved/locked balance)
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, initial_total_issuance);
    });
}

#[test]
fn test_reset_stakes_per_interval() {
    new_test_ext(0).execute_with(|| {
        let coldkey = U256::from(561330);
        let hotkey = U256::from(561337);

        SubtensorModule::set_stake_interval(7);
        SubtensorModule::set_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey, 5, 1);
        step_block(1);

        assert_eq!(
            SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey),
            5
        );

        // block: 7 interval not yet passed
        step_block(6);
        assert_eq!(
            SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey),
            5
        );

        // block 8: interval passed
        step_block(1);
        assert_eq!(
            SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey),
            0
        );
    });
}

#[test]
fn test_add_stake_under_limit() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_stakes = 2;

        SubtensorModule::set_target_stakes_per_interval(max_stakes);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));

        // TODO: get_stakes_this_interval_for_hotkey was replaced or removed?
        // let current_stakes =
        //     SubtensorModule::get_stakes_this_interval_for_hotkey(&hotkey_account_id);
        // assert!(current_stakes <= max_stakes);
    });
}

// TODO: set_stakes_this_interval_for_hotkey and get_stakes_this_interval_for_hotkey are removed. Is this test needed?
// #[test]
// fn test_add_stake_rate_limit_exceeded() {
//     new_test_ext(1).execute_with(|| {
//         let hotkey_account_id = U256::from(561337);
//         let coldkey_account_id = U256::from(61337);
//         let who: <Test as frame_system::Config>::AccountId = hotkey_account_id.into();
//         let netuid: u16 = 1;
//         let start_nonce: u64 = 0;
//         let tempo: u16 = 13;
//         let max_stakes = 2;
//         let block_number = 1;

//         SubtensorModule::set_target_stakes_per_interval(max_stakes);
//         SubtensorModule::set_stakes_this_interval_for_hotkey(
//             &hotkey_account_id,
//             max_stakes,
//             block_number,
//         );

//         let call: pallet_subtensor::Call<Test> = pallet_subtensor::Call::add_stake {
//             hotkey: hotkey_account_id,
//             amount_staked: 1,
//         };
//         let info: DispatchInfo =
//             DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
//         let extension = SubtensorSignedExtension::<Test>::new();
//         let result = extension.validate(&who, &call.into(), &info, 10);

//         assert_err!(result, InvalidTransaction::ExhaustsResources);

//         add_network(netuid, tempo, 0);
//         register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
//         assert_err!(
//             SubtensorModule::add_subnet_stake(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 netuid,
//                 1,
//             ),
//             Error::<Test>::StakeRateLimitExceeded
//         );

//         let current_stakes =
//             SubtensorModule::get_stakes_this_interval_for_hotkey(&hotkey_account_id);
//         assert_eq!(current_stakes, max_stakes);
//     });
// }

// /***********************************************************
// 	staking::remove_subnet_stake() tests
// ************************************************************/
#[test]
fn test_remove_stake_under_limit() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_unstakes = 2;

        SubtensorModule::set_target_stakes_per_interval(max_unstakes);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        SubtensorModule::increase_subnet_token_on_hotkey_account(&hotkey_account_id, netuid, 6000);

        log::info!(
            "Stake amount or hotkey: {:?}",
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &coldkey_account_id,
                &hotkey_account_id,
                netuid
            )
        );
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));

        // TODO: get_stakes_this_interval_for_hotkey is removed. Is this check needed?
        // let current_unstakes =
        //     SubtensorModule::get_stakes_this_interval_for_hotkey(&hotkey_account_id);
        // assert!(current_unstakes <= max_unstakes);
    });
}

// TODO: set_stakes_this_interval_for_hotkey and get_stakes_this_interval_for_hotkey are removed. Is this test needed?
// #[test]
// fn test_remove_stake_rate_limit_exceeded() {
//     new_test_ext(1).execute_with(|| {
//         let hotkey_account_id = U256::from(561337);
//         let coldkey_account_id = U256::from(61337);
//         let who: <Test as frame_system::Config>::AccountId = hotkey_account_id.into();
//         let netuid: u16 = 1;
//         let start_nonce: u64 = 0;
//         let tempo: u16 = 13;
//         let max_unstakes = 1;
//         let block_number = 1;

//         SubtensorModule::set_target_stakes_per_interval(max_unstakes);
//         SubtensorModule::set_stakes_this_interval_for_hotkey(
//             &hotkey_account_id,
//             max_unstakes,
//             block_number,
//         );

//         let call = pallet_subtensor::Call::remove_stake {
//             hotkey: hotkey_account_id,
//             amount_unstaked: 1,
//         };
//         let info: DispatchInfo =
//             DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
//         let extension = SubtensorSignedExtension::<Test>::new();
//         let result = extension.validate(&who, &call.into(), &info, 10);

//         assert_err!(result, InvalidTransaction::ExhaustsResources);

//         add_network(netuid, tempo, 0);
//         register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
//         SubtensorModule::increase_subnet_token_on_hotkey_account(&hotkey_account_id, netuid, 2);
//         assert_err!(
//             SubtensorModule::remove_subnet_stake(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 netuid,
//                 2,
//             ),
//             Error::<Test>::UnstakeRateLimitExceeded
//         );

//         let current_unstakes =
//             SubtensorModule::get_stakes_this_interval_for_hotkey(&hotkey_account_id);
//         assert_eq!(current_unstakes, max_unstakes);
//     });
// }

#[test]
#[cfg(not(tarpaulin))]
fn test_remove_subnet_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let amount_unstaked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_subnet_stake {
            hotkey,
            netuid,
            amount_unstaked,
        });
        let disp_info = call.get_dispatch_info();
        assert!(disp_info.weight.ref_time() != 0);
        assert_eq!(disp_info.class, DispatchClass::Normal,);
        assert_eq!(disp_info.pays_fee, Pays::No,);
    });
}

#[test]
fn test_remove_subnet_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Let's spin up a neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_subnet_token_on_hotkey_account(
            &hotkey_account_id,
            netuid,
            amount,
        );

        // Do the magic
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id),
            0
        );
    });
}

#[test]
fn test_remove_subnet_stake_amount_zero() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Let's spin up a neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_subnet_token_on_hotkey_account(
            &hotkey_account_id,
            netuid,
            amount,
        );

        // Do the magic
        assert_noop!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                0
            ),
            Error::<Test>::StakeToWithdrawIsZero
        );
    });
}

#[test]
fn test_remove_subnet_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000; // Amount to be removed

        let result = SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            netuid,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_remove_subnet_stake_err_hotkey_does_not_belong_to_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let other_cold_key = U256::from(99498);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Perform the request which is signed by a different cold key
        let result = SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            netuid,
            1000,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
    });
}

#[test]
fn test_remove_subnet_stake_no_enough_stake() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let amount = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            0
        );

        let result = SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            netuid,
            amount,
        );
        assert_eq!(result, Err(Error::<Test>::NotEnoughStakeToWithdraw.into()));
    });
}

#[test]
fn test_remove_subnet_stake_total_balance_no_change() {
    // When we remove stake, the total balance of the coldkey account should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(571337);
        let coldkey_account_id = U256::from(71337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        let amount = 10000;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_subnet_token_on_hotkey_account(
            &hotkey_account_id,
            netuid,
            amount,
        );

        // Do the magic
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_account_id),
            0
        );

        // Check total balance is equal to the added stake. Even after remove stake (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, amount);
    });
}

/***********************************************************
    staking::get_coldkey_balance() tests
************************************************************/
#[test]
fn test_get_coldkey_balance_no_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(5454); // arbitrary
        let result = SubtensorModule::get_coldkey_balance(&coldkey_account_id);

        // Arbitrary account should have 0 balance
        assert_eq!(result, 0);
    });
}

#[test]
fn test_get_coldkey_balance_with_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(5454); // arbitrary
        let amount = 1337;

        // Put the balance on the account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        let result = SubtensorModule::get_coldkey_balance(&coldkey_account_id);

        // Arbitrary account should have 0 balance
        assert_eq!(result, amount);
    });
}

// /***********************************************************
// 	staking::add_subnet_stake_to_hotkey_account() tests
// ************************************************************/
#[test]
fn test_add_subnet_stake_to_hotkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount: u64 = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        SubtensorModule::increase_subnet_token_on_hotkey_account(&hotkey_id, netuid, amount);

        // The stake that is now in the account, should equal the amount
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            amount
        );
    });
}

/************************************************************
    staking::remove_subnet_stake_from_hotkey_account() tests
************************************************************/
#[test]
fn test_remove_subnet_stake_from_hotkey_account() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount: u64 = 10000;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Add some stake that can be removed
        SubtensorModule::increase_subnet_token_on_hotkey_account(&hotkey_id, netuid, amount);

        // Prelimiary checks
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_subnet_token_on_hotkey_account(&hotkey_id, netuid, amount);

        // The stake on the hotkey account should be 0
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            0
        );
    });
}

#[test]
fn test_remove_subnet_stake_from_hotkey_account_registered_in_various_networks() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount: u64 = 10000;
        let netuid: u16 = 1;
        let netuid_ex = 2;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        //
        add_network(netuid, tempo, 0);
        add_network(netuid_ex, tempo, 0);
        //
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        register_ok_neuron(netuid_ex, hotkey_id, coldkey_id, 48141209);

        //let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id);

        let neuron_uid = match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => k,
            Err(e) => panic!("Error: {:?}", e),
        };
        //let neuron_uid_ex = SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id);

        let neuron_uid_ex = match SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id)
        {
            Ok(k) => k,
            Err(e) => panic!("Error: {:?}", e),
        };
        //Add some stake that can be removed
        SubtensorModule::increase_subnet_token_on_hotkey_account(&hotkey_id, netuid, amount);

        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_subnet_token_on_hotkey_account(&hotkey_id, netuid, amount);
        //
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            0
        );
    });
}

// /************************************************************
// 	staking::add_balance_to_coldkey_account() tests
// ************************************************************/
#[test]
fn test_add_balance_to_coldkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(4444322);
        let amount = 50000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, amount);
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_id), amount);
    });
}

// /***********************************************************
// 	staking::remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_remove_balance_from_coldkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(434324); // Random
        let ammount = 10000; // Arbitrary
                             // Put some $$ on the bank
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, ammount);
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            ammount
        );
        // Should be able to withdraw without hassle
        let result =
            SubtensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
        assert!(result.is_ok());
    });
}

#[test]
fn test_remove_balance_from_coldkey_account_failed() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(434324); // Random
        let ammount = 10000; // Arbitrary

        // Try to remove stake from the coldkey account. This should fail,
        // as there is no balance, nor does the account exist
        let result =
            SubtensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
        assert_eq!(result, Err(Error::<Test>::ZeroBalanceAfterWithdrawn.into()));
    });
}

//************************************************************
// 	staking::hotkey_belongs_to_coldkey() tests
// ************************************************************/
#[test]
fn test_hotkey_belongs_to_coldkey_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4434334);
        let coldkey_id = U256::from(34333);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_id),
            coldkey_id
        );
    });
}
// /************************************************************
// 	staking::can_remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_can_remove_balane_from_coldkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(87987984);
        let initial_amount = 10000;
        let remove_amount = 5000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
        assert!(SubtensorModule::can_remove_balance_from_coldkey_account(
            &coldkey_id,
            remove_amount
        ));
    });
}

#[test]
fn test_can_remove_balance_from_coldkey_account_err_insufficient_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(87987984);
        let initial_amount = 10000;
        let remove_amount = 20000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
        assert!(!SubtensorModule::can_remove_balance_from_coldkey_account(
            &coldkey_id,
            remove_amount
        ));
    });
}
/************************************************************
    staking::has_enough_stake() tests
************************************************************/
#[test]
fn test_has_enough_stake_yes() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4334);
        let coldkey_id = U256::from(87989);
        let intial_amount = 10000;
        let netuid = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        SubtensorModule::increase_subnet_token_on_hotkey_account(&hotkey_id, netuid, intial_amount);
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            10000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &coldkey_id,
                &hotkey_id,
                netuid
            ),
            10000
        );
        assert!(SubtensorModule::has_enough_stake(
            &coldkey_id,
            &hotkey_id,
            netuid,
            5000
        ),);
    });
}

#[test]
fn test_has_enough_stake_no() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4334);
        let coldkey_id = U256::from(87989);
        let intial_amount = 0;
        let netuid = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        SubtensorModule::increase_subnet_token_on_hotkey_account(&hotkey_id, netuid, intial_amount);
        assert!(!SubtensorModule::has_enough_stake(
            &coldkey_id,
            &hotkey_id,
            netuid,
            5000
        ));
    });
}

#[test]
fn test_non_existent_account() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
            &U256::from(0),
            &(U256::from(0)),
            netuid,
            10,
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &U256::from(0),
                &U256::from(0),
                netuid
            ),
            10
        );
        assert_eq!(get_total_stake_for_coldkey(&(U256::from(0))), 10);
    });
}

/************************************************************
    staking::delegating
************************************************************/

#[test]
fn test_delegate_stake_division_by_zero_check() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(3);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 2341312);
        assert_ok!(SubtensorModule::become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey, netuid, 0, 1000);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_full_with_delegating() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        add_network(netuid, 0, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_target_registrations_per_interval(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 4); // Allow all 4 to be registered at once
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Neither key can add stake because they are not registered (registration check comes before balance check)
        // We have enough, but the keys are not registered.
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Cant remove either.
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Neither key can become a delegate either because we are not registered.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Register the 2 neurons to a new network.
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We try to delegate stake but niether are allowing delegation.
        assert!(!SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(!SubtensorModule::hotkey_is_delegate(&hotkey1));
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                100
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // We stake and all is ok.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            100
        );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 100 );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 100 );

        // Cant remove these funds because we are not delegating.
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 100);
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            200
        );

        // Try allowing the keys to become delegates, fails because of incorrect coldkeys.
        // Set take to be 0.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // Cant become a delegate twice.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0
            ),
            Err(Error::<Test>::HotKeyAlreadyDelegate.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1
            ),
            Err(Error::<Test>::HotKeyAlreadyDelegate.into())
        );

        // This add stake works for delegates.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            200
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            200
        );
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            200
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            300
        ));

        let mut substake_cold0_hot0 = 200;
        let mut substake_cold0_hot1 = 200;
        let mut substake_cold1_hot0 = 300;
        let mut substake_cold1_hot1 = 200;

        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            substake_cold0_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            substake_cold0_hot1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            substake_cold1_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            substake_cold1_hot1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            substake_cold0_hot0 + substake_cold1_hot0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            substake_cold0_hot1 + substake_cold1_hot1
        );
        assert_eq!(
            get_total_stake_for_coldkey(&coldkey0),
            substake_cold0_hot0 + substake_cold0_hot1
        );
        assert_eq!(
            get_total_stake_for_coldkey(&coldkey1),
            substake_cold1_hot0 + substake_cold1_hot1
        );

        // Lets emit inflation through the hot and coldkeys.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 1000);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 1000);

        let take0 = SubtensorModule::get_delegate_take(&hotkey0, netuid) as f32 / u16::MAX as f32;
        let take1 = SubtensorModule::get_delegate_take(&hotkey1, netuid) as f32 / u16::MAX as f32;

        let cold0hot0weight =
            substake_cold0_hot0 as f32 / (substake_cold0_hot0 + substake_cold1_hot0) as f32;
        let cold0hot1weight =
            substake_cold0_hot1 as f32 / (substake_cold0_hot1 + substake_cold1_hot1) as f32;
        let cold1hot0weight =
            substake_cold1_hot0 as f32 / (substake_cold0_hot0 + substake_cold1_hot0) as f32;
        let cold1hot1weight =
            substake_cold1_hot1 as f32 / (substake_cold0_hot1 + substake_cold1_hot1) as f32;
        let delegate_take_hot0 = 1000. * take0;
        let delegate_take_hot1 = 1000. * take1;
        let emission0_remainder = 1000. - delegate_take_hot0;
        let emission1_remainder = 1000. - delegate_take_hot1;

        // cold0 owns hot0, hence delegate_take_hot0 goes to cold0 substake. +1 for rounding errors
        substake_cold0_hot0 +=
            (delegate_take_hot0 + emission0_remainder * cold0hot0weight) as u64 + 1;
        substake_cold1_hot0 += (emission0_remainder * cold1hot0weight) as u64;
        substake_cold0_hot1 += (emission1_remainder * cold0hot1weight) as u64;
        substake_cold1_hot1 +=
            (delegate_take_hot1 + emission1_remainder * cold1hot1weight) as u64 + 1;
        // initial + rewards, server emission goes to cold0 in dtao

        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            substake_cold0_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            substake_cold0_hot1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            substake_cold1_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            substake_cold1_hot1
        );

        // // Try unstaking too much.
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );

        // unstaking is ok.
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            100
        ));
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            100
        ));

        // All the amounts have been decreased.
        substake_cold0_hot0 -= 100;
        substake_cold1_hot0 -= 100;
        substake_cold0_hot1 -= 100;
        substake_cold1_hot1 -= 100;

        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            substake_cold0_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            substake_cold0_hot1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            substake_cold1_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            substake_cold1_hot1
        );

        // Lets register and stake a new key.
        let hotkey2 = U256::from(5);
        let coldkey2 = U256::from(6);
        register_ok_neuron(netuid, hotkey2, coldkey2, 248_123);
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey0
        ));
        assert!(SubtensorModule::is_hotkey_registered_on_any_network(
            &hotkey1
        ));

        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 60_000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            1000
        ));
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            900
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // Lets make this new key a delegate with an 18% take (default take value in tests).
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2
        ));

        // Add nominate some stake.
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            netuid,
            1_000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            netuid,
            1_000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            1_000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            1_000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            1_000
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey2),
            3_000
        );

        // Lets emit inflation through this new key with distributed ownership.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, netuid, 0, 1000);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            1_454
        ); // 1000 + 180 + 820 * (1000/3000) = 1500 + 453.3 ~ 1454
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            1_273
        ); // 1000 + 820 * (1000/3000) = 1000 + 273.3 = 1273
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            1_273
        ); // 1000 + 820 * (1000/3000) = 1000 + 273.3 = 1273

        step_block(1);

        // Lets register and stake a new key.
        let hotkey3 = U256::from(7);
        let coldkey3 = U256::from(8);
        register_ok_neuron(netuid, hotkey3, coldkey3, 4124124);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey3, 60000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3,
            netuid,
            1000
        ));

        step_block(3);

        // 100% take is not a valid business case, changing the rest of this test to 50%
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3
        )); // 18% take - default value for tests.
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey3,
            netuid,
            1000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey3,
            netuid,
            1000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey3,
            netuid,
            1000
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey3, netuid),
            1000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey3, netuid),
            1000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey3, netuid),
            1000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey3, &hotkey3, netuid),
            1000
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey3),
            4000
        );
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey3, netuid, 0, 1000);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey3, netuid),
            1205
        ); // 1000 + 82% * 1000 * 1000/4000 = 1205
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey3, netuid),
            1205
        ); // 1000 + 82% * 1000 * 1000/4000 = 1205
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey3, netuid),
            1205
        ); // 1000 + 82% * 1000 * 1000/4000 = 1205
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey3, &hotkey3, netuid),
            1385
        ); // 1000 + 180 + 820 * 1000/4000 = 1385
    });
}

// Verify delegates with servers get the full server inflation.
#[test]
fn test_full_with_delegating_some_servers() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs
        add_network(netuid, 0, 0);
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Neither key can add stake because they are not registered (registration check is now done before balance check).
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We stake and all is ok.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            100
        );

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 100);
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            200
        );

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));
        let take0 = SubtensorModule::get_delegate_take(&hotkey0, netuid) as f32 / u16::MAX as f32;
        let take1 = SubtensorModule::get_delegate_take(&hotkey1, netuid) as f32 / u16::MAX as f32;

        // This add stake works for delegates.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            200
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            200
        );
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            200
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            300
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            200
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            200
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            300
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            200
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            500
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            400
        );

        // Set global stake weight to be 1
        SubtensorModule::set_global_stake_weight(u16::MAX);

        // Lets emit inflation through the hot and coldkeys.
        // fist emission arg is for a server. This should only go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 123, 2_000); // 2_123 total emission.

        // Global stake weights = 0 for now, so all nominator rewards are calculated off their global stake proportion
        // which is (for non-dynamic networks) the sum of all nominator stakes to this delegate in all subnets divided
        // by sum of all delegate stakes in all subnets
        let cold0hot0weight = 200. / 500.;
        let cold0hot1weight = 200. / 400.;
        let cold1hot0weight = 300. / 500.;
        let cold1hot1weight = 200. / 400.;
        let delegate_take_hot0 = 1000. * take0;
        let delegate_take_hot1 = 2000. * take1;
        let emission0_remainder = 1000. - delegate_take_hot0;
        let emission1_remainder = 2000. - delegate_take_hot1;

        // cold0 owns hot0, hence delegate_take_hot0 goes to cold0 substake. +1 for rounding errors
        let substake_cold0_hot0 =
            200 + (delegate_take_hot0 + emission0_remainder * cold0hot0weight) as u64 + 1;
        let substake_cold1_hot0 = 300 + (emission0_remainder * cold1hot0weight) as u64;
        let substake_cold0_hot1 = 200 + (emission1_remainder * cold0hot1weight) as u64;
        let substake_cold1_hot1 =
            200 + (delegate_take_hot1 + emission1_remainder * cold1hot1weight) as u64 + 1;
        // initial + rewards, server emission goes to cold0 in dtao
        let total_hot0 = 500 + (delegate_take_hot0 + emission0_remainder) as u64;
        let total_hot1 = 400 + (delegate_take_hot1 + emission1_remainder) as u64;

        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            substake_cold0_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            substake_cold1_hot0
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            total_hot0
        );

        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            substake_cold0_hot1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            substake_cold1_hot1
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            total_hot1
        );

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time only server emission. This should go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 150, 0);

        // No change
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            substake_cold0_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            substake_cold0_hot1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            substake_cold1_hot0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            substake_cold1_hot1
        );

        // Lets register and stake a new key.
        let hotkey2 = U256::from(5);
        let coldkey2 = U256::from(6);
        register_ok_neuron(netuid, hotkey2, coldkey2, 248123);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 60_000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            1_000
        ));
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            900
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // Lets make this new key a delegate with a default take.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2
        ));
        let take2 = SubtensorModule::get_delegate_take(&hotkey2, netuid) as f32 / u16::MAX as f32;

        // Add nominate some stake.
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            netuid,
            1000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            netuid,
            1000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            1000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            1000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            1000
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey2),
            3_000
        );

        // Lets emit inflation through this new key with distributed ownership.
        // We will emit 100 server emission, which should go in-full to the owner of the hotkey.
        // We will emit 1000 validator emission, which should be distributed in-part to the nominators.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, netuid, 100, 1000);

        let delegate_take_hot2 = 1000. * take2;
        let emission2_remainder = 1000. - delegate_take_hot2;
        let cold0hot2weight = 1000. / 3000.;
        let cold1hot2weight = 1000. / 3000.;
        let cold2hot2weight = 1000. / 3000.;
        let substake_cold0_hot2 = 1000 + (emission2_remainder * cold0hot2weight) as u64;
        let substake_cold1_hot2 = 1000 + (emission2_remainder * cold1hot2weight) as u64;
        let substake_cold2_hot2 =
            1000 + (delegate_take_hot2 + emission2_remainder * cold2hot2weight) as u64 + 1;

        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            substake_cold2_hot2
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            substake_cold1_hot2
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            substake_cold0_hot2
        );
        let cold2balance_before = SubtensorModule::get_coldkey_balance(&coldkey2);

        // Lets emit MORE inflation through this new key with distributed ownership.
        // This time we do ONLY server emission
        // We will emit 123 server emission, which should go in-full to the owner of the hotkey.
        // We will emit *0* validator emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, netuid, 123, 0);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            substake_cold2_hot2
        ); // No change.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            substake_cold1_hot2
        ); // No change.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            substake_cold0_hot2
        ); // No change.

        let cold2balance_after = SubtensorModule::get_coldkey_balance(&coldkey2);
        assert_eq!(cold2balance_after - cold2balance_before, 123);
    });
}

#[test]
fn test_stao_delegation() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let delegate = U256::from(1);
        let nominator1 = U256::from(2);
        let nominator2 = U256::from(3);

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate, delegate, 124124);
        SubtensorModule::set_target_stakes_per_interval(10000);
        SubtensorModule::add_balance_to_coldkey_account(&delegate, 100000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator1, 100000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator2, 100000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate),
            delegate,
            netuid,
            100000
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(delegate),
            delegate
        ));
        let take = SubtensorModule::get_delegate_take(&delegate, netuid) as f32 / u16::MAX as f32;

        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator1),
            delegate,
            netuid,
            100000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator2),
            delegate,
            netuid,
            100000
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&delegate));
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&delegate),
            100000 * 3
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_subnet(netuid),
            100000 * 3
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&delegate, netuid),
            100000 * 3
        );
        assert_eq!(get_total_stake_for_coldkey(&delegate), 100_000);
        assert_eq!(get_total_stake_for_coldkey(&nominator1), 100_000);
        assert_eq!(get_total_stake_for_coldkey(&nominator2), 100_000);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&delegate),
            delegate
        );
        assert!(SubtensorModule::hotkey_account_exists(&delegate));
        assert!(!SubtensorModule::hotkey_account_exists(&nominator1));
        assert!(!SubtensorModule::hotkey_account_exists(&nominator2));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&delegate, &delegate, netuid),
            100_000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator1,
                &delegate,
                netuid
            ),
            100_000
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator2,
                &delegate,
                netuid
            ),
            100_000
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_coldkey(&delegate, &delegate),
            100_000
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_coldkey(&delegate, &nominator1),
            100_000
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_coldkey(&delegate, &nominator1),
            100_000
        );
        SubtensorModule::emit_inflation_through_hotkey_account(&delegate, netuid, 0, 1000);
        let nominator_reward = ((1000. * (1. - take)) as u64) / 3;
        let delegate_take = 1000 - nominator_reward * 3;
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&delegate, &delegate, netuid),
            100000 + delegate_take + nominator_reward
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator1,
                &delegate,
                netuid
            ),
            100000 + nominator_reward
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator2,
                &delegate,
                netuid
            ),
            100000 + nominator_reward
        );
    })
}

#[test]
fn test_full_block_emission_occurs() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);

        add_network(netuid, 0, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Neither key can add stake because they are not registered
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(SubtensorModule::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We stake and all is ok.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            0
        );
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            100
        );

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 111);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 234);

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // Add some delegate stake
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            200
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            300
        ));

        // Lets emit inflation with delegatees, with both validator and server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 123, 2_000); // 2_123 total emission.

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time JUSt server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 150, 0);

        // Lastly, do only validator emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 12_948);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 1_874);
    });
}

/************************************************************
    staking::unstake_all_coldkeys_from_hotkey_account() tests
************************************************************/

#[test]
fn test_unstake_all_coldkeys_from_hotkey_account() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(123570);
        let coldkey0_id = U256::from(123560);

        let coldkey1_id = U256::from(123561);
        let coldkey2_id = U256::from(123562);
        let coldkey3_id = U256::from(123563);

        let amount: u64 = 10000;

        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        // Make subnet
        add_network(netuid, tempo, 0);
        // Register delegate
        register_ok_neuron(netuid, hotkey_id, coldkey0_id, start_nonce);

        match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(_k) => (),
            Err(e) => panic!("Error: {:?}", e),
        }

        //Add some stake that can be removed
        SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
            &coldkey0_id,
            &hotkey_id,
            netuid,
            amount,
        );
        SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
            &coldkey1_id,
            &hotkey_id,
            netuid,
            amount + 2,
        );
        SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
            &coldkey2_id,
            &hotkey_id,
            netuid,
            amount + 3,
        );
        SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
            &coldkey3_id,
            &hotkey_id,
            netuid,
            amount + 4,
        );

        // Verify free balance is 0 for all coldkeys
        assert_eq!(Balances::free_balance(coldkey0_id), 0);
        assert_eq!(Balances::free_balance(coldkey1_id), 0);
        assert_eq!(Balances::free_balance(coldkey2_id), 0);
        assert_eq!(Balances::free_balance(coldkey3_id), 0);

        // Verify total stake is correct
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            amount * 4 + (2 + 3 + 4)
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        SubtensorModule::unstake_all_coldkeys_from_hotkey_account(&hotkey_id);

        // Verify total stake is 0
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            0
        );

        // Vefify stake for all coldkeys is 0
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &coldkey0_id,
                &hotkey_id,
                netuid
            ),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &coldkey1_id,
                &hotkey_id,
                netuid
            ),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &coldkey2_id,
                &hotkey_id,
                netuid
            ),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &coldkey3_id,
                &hotkey_id,
                netuid
            ),
            0
        );

        // Verify free balance is correct for all coldkeys
        assert_eq!(Balances::free_balance(coldkey0_id), amount);
        assert_eq!(Balances::free_balance(coldkey1_id), amount + 2);
        assert_eq!(Balances::free_balance(coldkey2_id), amount + 3);
        assert_eq!(Balances::free_balance(coldkey3_id), amount + 4);
    });
}

#[test]
fn test_unstake_all_coldkeys_from_hotkey_account_single_staker() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(123570);
        let coldkey0_id = U256::from(123560);

        let amount: u64 = 891011;

        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        // Make subnet
        add_network(netuid, tempo, 0);
        // Register delegate
        register_ok_neuron(netuid, hotkey_id, coldkey0_id, start_nonce);

        match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(_) => (),
            Err(e) => panic!("Error: {:?}", e),
        }

        //Add some stake that can be removed
        SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
            &coldkey0_id,
            &hotkey_id,
            netuid,
            amount,
        );

        // Verify free balance is 0 for coldkey
        assert_eq!(Balances::free_balance(coldkey0_id), 0);

        // Verify total stake is correct
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            amount
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        SubtensorModule::unstake_all_coldkeys_from_hotkey_account(&hotkey_id);

        // Verify total stake is 0
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey_id),
            0
        );

        // Vefify stake for single coldkey is 0
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &coldkey0_id,
                &hotkey_id,
                netuid
            ),
            0
        );

        // Verify free balance is correct for single coldkey
        assert_eq!(Balances::free_balance(coldkey0_id), amount);
    });
}

#[test]
fn test_faucet_ok() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(123560);

        log::info!("Creating work for submission to faucet...");

        let block_number = SubtensorModule::get_current_block_as_u64();
        let difficulty: U256 = U256::from(10_000_000);
        let mut nonce: u64 = 0;
        let mut work: H256 = SubtensorModule::create_seal_hash(block_number, nonce, &coldkey);
        while !SubtensorModule::hash_meets_difficulty(&work, difficulty) {
            nonce += 1;
            work = SubtensorModule::create_seal_hash(block_number, nonce, &coldkey);
        }
        let vec_work: Vec<u8> = SubtensorModule::hash_to_vec(work);

        log::info!("Faucet state: {}", cfg!(feature = "pow-faucet"));

        #[cfg(feature = "pow-faucet")]
        assert_ok!(SubtensorModule::do_faucet(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            block_number,
            nonce,
            vec_work
        ));

        #[cfg(not(feature = "pow-faucet"))]
        assert_ok!(SubtensorModule::do_faucet(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            block_number,
            nonce,
            vec_work
        ));
    });
}

/// This test ensures that the clear_small_nominations function works as expected.
/// It creates a network with two hotkeys and two coldkeys, and then registers a nominator account for each hotkey.
/// When we call set_nominator_min_required_stake, it should clear all small nominations that are below the minimum required stake.
/// Run this test using: cargo test --package pallet-subtensor --test staking test_clear_small_nominations
#[test]
fn test_clear_small_nominations() {
    new_test_ext(0).execute_with(|| {
        System::set_block_number(1);

        // Create accounts.
        let netuid = 1;
        let hot1 = U256::from(1);
        let hot2 = U256::from(2);
        let cold1 = U256::from(3);
        let cold2 = U256::from(4);

        SubtensorModule::set_target_stakes_per_interval(10);
        // Register hot1 and hot2 .
        add_network(netuid, 0, 0);

        // Register hot1.
        register_ok_neuron(netuid, hot1, cold1, 0);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot1,
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot1), cold1);

        // Register hot2.
        register_ok_neuron(netuid, hot2, cold2, 0);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot2), cold2);

        // Add stake cold1 --> hot1 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5 + ExistentialDeposit::get());
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot1,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold1, &hot1, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 4 + ExistentialDeposit::get());

        // Add stake cold2 --> hot1 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5 + ExistentialDeposit::get());
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot1,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold2, &hot1, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 4 + ExistentialDeposit::get());

        // Add stake cold1 --> hot2 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot2,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold1, &hot2, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 8 + ExistentialDeposit::get());

        // Add stake cold2 --> hot2 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold2, &hot2, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 8 + ExistentialDeposit::get());

        // Run clear all small nominations when min stake is zero (noop)
        SubtensorModule::set_nominator_min_required_stake(0);
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold1, &hot1, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold1, &hot2, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold2, &hot1, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold2, &hot2, netuid),
            1
        );

        // Set min nomination to 10
        let _ = Staker::<Test>::try_get(hot2, cold1).unwrap(); // ensure exists before
        let _ = Staker::<Test>::try_get(hot1, cold2).unwrap(); // ensure exists before
        SubtensorModule::set_nominator_min_required_stake(10);

        // Run clear all small nominations (removes delegations under 10)
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold1, &hot1, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold1, &hot2, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold2, &hot1, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&cold2, &hot2, netuid),
            1
        );

        // Balances have been added back into accounts.
        assert_eq!(Balances::free_balance(cold1), 9 + ExistentialDeposit::get());
        assert_eq!(Balances::free_balance(cold2), 9 + ExistentialDeposit::get());

        // Internal storage is updated
        Staker::<Test>::try_get(hot2, cold1).unwrap_err();
    });
}

/// Test that the nominator minimum staking threshold is enforced when stake is added.
#[test]
fn test_add_stake_below_minimum_threshold() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey1 = U256::from(0);
        let hotkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let minimum_threshold = 10_000_000;
        let amount_below = 50_000;

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 100_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 100_000);
        SubtensorModule::set_nominator_min_required_stake(minimum_threshold);
        SubtensorModule::set_target_stakes_per_interval(10);

        // Create network
        add_network(netuid, 0, 0);

        // Register the neuron to a new network.
        register_ok_neuron(netuid, hotkey1, coldkey1, 0);
        assert_ok!(SubtensorModule::become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1
        ));

        // Coldkey staking on its own hotkey can stake below min threshold.
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            amount_below
        ));

        // Nomination stake cannot stake below min threshold.
        assert_noop!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
                hotkey1,
                netuid,
                amount_below
            ),
            pallet_subtensor::Error::<Test>::NomStakeBelowMinimumThreshold
        );
    });
}

/// Test that the nominator minimum staking threshold is enforced when stake is removed.
#[test]
fn test_remove_stake_below_minimum_threshold() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey1 = U256::from(0);
        let hotkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let initial_balance = 200_000_000;
        let initial_stake = 100_000;
        let minimum_threshold = 50_000;
        let stake_amount_to_remove = 51_000;

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey1,
            initial_balance + ExistentialDeposit::get(),
        );
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey2,
            initial_balance + ExistentialDeposit::get(),
        );
        SubtensorModule::set_nominator_min_required_stake(minimum_threshold);
        SubtensorModule::set_target_stakes_per_interval(10);

        // Create network
        add_network(netuid, 0, 0);

        // Register the neuron to a new network.
        register_ok_neuron(netuid, hotkey1, coldkey1, 0);
        assert_ok!(SubtensorModule::become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            initial_stake
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            netuid,
            initial_stake
        ));

        // Coldkey staking on its own hotkey can unstake below min threshold.
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            stake_amount_to_remove
        ));

        // Nomination stake cannot unstake below min threshold,
        // without unstaking all and removing the nomination.
        let bal_before = Balances::free_balance(coldkey2);
        let staked_before =
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1, netuid);
        let total_issuance_before = SubtensorModule::get_total_issuance();
        // check the premise of the test is correct
        assert!(initial_stake - stake_amount_to_remove < minimum_threshold);
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
                hotkey1,
                netuid,
                stake_amount_to_remove
            ),
            Err(Error::<Test>::NomStakeBelowMinimumThreshold.into())
        );

        // Unstake all
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            netuid,
            initial_stake
        ));

        // Has no stake now
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1, netuid),
            0
        );
        // All stake was removed
        let stake_removed = staked_before;
        // Has the full balance
        assert_eq!(Balances::free_balance(coldkey2), bal_before + stake_removed);

        // Staker map entry is removed
        assert!(Staker::<Test>::try_get(hotkey1, coldkey2).is_err(),);

        // Total issuance is the same
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            total_issuance_before // Nothing was issued
        );
    });
}

// Verify delegate take can be decreased
#[test]
fn test_delegate_take_can_be_decreased() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Coldkey / hotkey 0 decreases take
        let lower_take = SubtensorModule::get_delegate_take(&hotkey0, netuid) - 1;
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            lower_take
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            lower_take
        );
    });
}

// Verify delegate take can be decreased
#[test]
fn test_can_set_min_take_ok() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
        ));

        // Coldkey / hotkey 0 decreases take to min
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );
    });
}

// Verify delegate take can not be increased with do_decrease_take
#[test]
fn test_delegate_take_can_not_be_increased_with_decrease_take() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 10% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Decrease delegate take to 5%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 tries to increase take to 12.5%
        assert_eq!(
            SubtensorModule::do_decrease_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                u16::MAX / 8
            ),
            Err(Error::<Test>::DelegateTakeTooLow.into())
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );
    });
}

// Verify delegate take can be increased
#[test]
fn test_delegate_take_can_be_increased() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Decrease delegate take to 5%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 decreases take to 12.5%
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            u16::MAX / 8
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 8
        );
    });
}

// Verify delegate take can not be decreased with increase_take
#[test]
fn test_delegate_take_can_not_be_decreased_with_increase_take() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Decrease delegate take to 10%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 tries to decrease take to 5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                u16::MAX / 20
            ),
            Err(Error::<Test>::DelegateTakeTooLow.into())
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );
    });
}

// Verify delegate take can be increased up to InitialDefaultTake (18%)
#[test]
fn test_delegate_take_can_be_increased_to_limit() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Decrease delegate take to 10%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultTake+1
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            InitialDefaultTake::get()
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );
    });
}

// Verify delegate take can not be set above InitialDefaultTake
#[test]
fn test_delegate_take_can_not_be_increased_beyond_limit() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        let before = SubtensorModule::get_delegate_take(&hotkey0, netuid);

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        if InitialDefaultTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_increase_take(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    netuid,
                    InitialDefaultTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(SubtensorModule::get_delegate_take(&hotkey0, netuid), before);
    });
}

// Verify delegate take affects emission distribution
#[test]
fn test_delegate_take_affects_distribution() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 100000);

        // Register the 2 neurons to a new network.
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);

        // Stake 100 from coldkey/hotkey 0
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            100
        );

        // Coldkey / hotkey 0 become delegates with 50% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Hotkey 1 adds 100 delegated stake to coldkey/hotkey 0
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            0
        );
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            200
        );
        assert_eq!(SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1), 0);

        // Lets emit inflation through this new key with distributed ownership.
        // We will emit 0 server emission (which should go in-full to the owner of the hotkey).
        // We will emit 400 validator emission, which should be distributed in-part to the nominators.
        //
        // Total initial stake is 200
        // Delegate's initial stake is 100, which is 50% of total stake
        //  => Delegate will receive 50% of emission (200) + 50% take (100) of nominator reward (200)
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 400);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            336
        ); // 100 + 18% * 400 + 82% * 200 = 336
    });
}

// Verify changing delegate take also changes emission distribution
#[test]
fn test_changing_delegate_take_changes_distribution() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 100000);

        // Register the 2 neurons to a new network.
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);

        // Stake 100 from coldkey/hotkey 0
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            100
        );

        // Coldkey / hotkey 0 become delegates with 50% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Hotkey 1 adds 100 delegated stake to coldkey/hotkey 0
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            0
        );
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            100
        ));
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey0),
            200
        );
        assert_eq!(SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1), 0);

        // Coldkey / hotkey 0 decrease take to 10%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            u16::MAX / 10
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 10
        );

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultTake+1
        // (Disable this check if InitialDefaultTake is u16::MAX)
        if InitialDefaultTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_increase_take(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    netuid,
                    InitialDefaultTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 10
        );
    });
}

// Test rate-limiting on increase_take
#[test]
fn test_rate_limits_enforced_on_increase_take() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with InitialDefaultTake take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Decrease delegate take to get_min_take
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 increases take to 12.5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                u16::MAX / 8
            ),
            Err(Error::<Test>::DelegateTxRateLimitExceeded.into())
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            SubtensorModule::get_min_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Can increase after waiting
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            u16::MAX / 10
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 10
        );
    });
}

#[test]
fn set_delegate_takes_updates_delegates_correctly() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let takes = vec![(1u16, 10u16), (2u16, 15u16)]; // Ensure these values are within the InitialDefaultTake limit

        // Create subnets and register as delegates
        let tempo: u16 = 13;
        for (netuid, _) in &takes {
            add_network(*netuid, tempo, 0);
            register_ok_neuron(*netuid, hotkey, coldkey, 0);
        }

        // Action: Call set_delegate_takes
        assert_ok!(SubtensorModule::set_delegate_takes(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            takes.clone()
        ));

        for (netuid, take) in takes {
            let actual_take = SubtensorModule::get_delegate_take(&hotkey, netuid);
            log::info!(
                "Checking delegate take for netuid {}: Expected take: {}, Actual take: {}",
                netuid,
                take,
                actual_take
            );
            assert_eq!(
                actual_take, take,
                "The delegate take for netuid {} should be updated to {}",
                netuid, take
            );
        }
    });
}

#[test]
fn set_delegate_takes_handles_empty_vector() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(1);
        let takes: Vec<(u16, u16)> = vec![];

        // Create subnet and register as delegate
        let tempo: u16 = 13;
        add_network(1, tempo, 0);
        register_ok_neuron(1, hotkey, coldkey, 0);

        // Ensure coldkey is associated as a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            RuntimeOrigin::signed(coldkey),
            hotkey
        ));

        assert_ok!(SubtensorModule::set_delegate_takes(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            takes
        ));

        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey, 1),
            InitialDefaultTake::get(),
            "Delegate take should be the default take value for netuid 1 after empty update"
        );
    });
}

#[test]
fn set_delegate_takes_rejects_invalid_netuid() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(1);
        let takes = vec![(999u16, 10u16)]; // Invalid netuid

        // Create subnet and register as delegate for a valid network first
        let tempo: u16 = 13;
        add_network(1, tempo, 0); // Adding a valid network
        register_ok_neuron(1, hotkey, coldkey, 0); // Registering neuron on the valid network

        // Ensure coldkey is associated as a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            RuntimeOrigin::signed(coldkey),
            hotkey
        ));

        // Now test with an invalid network ID
        assert_err!(
            SubtensorModule::set_delegate_takes(RuntimeOrigin::signed(coldkey), hotkey, takes),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn set_delegate_takes_rejects_excessive_take() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(1);
        let takes = vec![(1u16, 32_767 * 2)]; // Excessive take value

        // Create subnet and register as delegate
        let tempo: u16 = 13;
        add_network(1, tempo, 0);
        register_ok_neuron(1, hotkey, coldkey, 0);

        // Ensure coldkey is associated as a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            RuntimeOrigin::signed(coldkey),
            hotkey
        ));

        // Now test with an excessive take value
        assert_err!(
            SubtensorModule::set_delegate_takes(RuntimeOrigin::signed(coldkey), hotkey, takes),
            Error::<Test>::DelegateTakeTooHigh
        );
    });
}

#[test]
fn set_delegate_takes_enforces_rate_limit() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let takes_initial = vec![(1u16, 10u16), (2u16, 15u16)];
        let takes_second = vec![(1u16, 11u16), (2u16, 16u16)]; // Slightly increased takes

        // Create subnets and register as delegates
        let tempo: u16 = 13;
        for (netuid, _) in &takes_initial {
            add_network(*netuid, tempo, 0);
            register_ok_neuron(*netuid, hotkey, coldkey, 0);
        }

        // First call to set_delegate_takes should succeed
        assert_ok!(SubtensorModule::set_delegate_takes(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            takes_initial
        ));

        // Second call to set_delegate_takes should fail due to rate limit
        // Now test with an excessive take value
        assert_err!(
            SubtensorModule::set_delegate_takes(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                takes_second
            ),
            Error::<Test>::DelegateTxRateLimitExceeded
        );
    });
}

#[test]
fn test_log_subnet_emission_values_dynamic_registration() {
    new_test_ext(1).execute_with(|| {
        let num_networks = 10;

        // Create dynamic subnets through user registration
        for i in 1..=num_networks {
            let netuid = i;
            let tempo = 13;
            let block_number = 0;
            let cold_id = i * 100; // Generate a unique cold ID for each network
            let hot_id = cold_id + 1; // Generate a unique hot ID for each network

            // Add the network
            add_network(netuid, tempo, 0);

            // Create work for the user
            let hotkey_account_id = U256::from(hot_id);
            let coldkey_account_id = U256::from(cold_id);
            SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

            let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
                netuid,
                block_number,
                i as u64,
                &hotkey_account_id,
            );

            // Register the user in the network by signing
            assert_ok!(SubtensorModule::register(
                <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id),
                netuid,
                block_number,
                nonce,
                work,
                hotkey_account_id,
                coldkey_account_id
            ));

            // Become Delelegate
            assert_ok!(SubtensorModule::do_become_delegate(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id
            ));
        }
        run_to_block(1000);
        // step_block(1000);
        // Log the emission values for each subnet using subnet_info
        for i in 1..=num_networks {
            let netuid = i;
            let subnet_emission_value = SubtensorModule::get_emission_value(netuid);
            log::info!(
                "tao per alpha price = {:?}",
                SubtensorModule::get_tao_per_alpha_price(netuid)
            );
            log::info!(
                "Subnet {}: Emission Value = {:?}",
                netuid,
                subnet_emission_value
            );
        }
    });
}
