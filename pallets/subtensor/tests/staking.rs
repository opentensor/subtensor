use frame_support::assert_err;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::{Error, SubtensorSignedExtension};
use sp_core::{H256, U256};
use sp_runtime::traits::{DispatchInfoOf, SignedExtension};

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
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(65000000, 0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            9999
        );

        // Check if balance has decreased
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 1);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 9999);
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
        SubtensorModule::increase_stake_on_hotkey_account(
            &neuron_src_hotkey_id,
            netuid,
            initial_stake,
        );

        // Check if the initial stake has arrived
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id),
            initial_stake
        );

        // Check if all three neurons are registered
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);

        // Run a couple of blocks to check if emission works
        run_to_block(2);

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id),
            initial_stake
        );

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_dest_hotkey_id),
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
            Err(Error::<Test>::NotRegistered.into())
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
        assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
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
        let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 10000);

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
        let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Check total issuance is equal to initial balance
        let initial_total_issuance = Balances::total_issuance();
        assert_eq!(initial_total_issuance, initial_balance);

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 10000);

        // Check if total issuance has remained the same. (no fee, includes reserved/locked balance)
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, initial_total_issuance);
    });
}

// TODO: set_stakes_this_interval_for_hotkey is missing. Was it replaced with anythign or removed completely?
// #[test]
// fn test_reset_stakes_per_interval() {
//     new_test_ext(0).execute_with(|| {
//         let hotkey = U256::from(561337);

//         SubtensorModule::set_stake_interval(7);
//         SubtensorModule::set_stakes_this_interval_for_hotkey(&hotkey, 5, 1);
//         step_block(1);

//         assert_eq!(
//             SubtensorModule::get_stakes_this_interval_for_hotkey(&hotkey),
//             5
//         );

//         // block: 7 interval not yet passed
//         step_block(6);
//         assert_eq!(
//             SubtensorModule::get_stakes_this_interval_for_hotkey(&hotkey),
//             5
//         );

//         // block 8: interval passed
//         step_block(1);
//         assert_eq!(
//             SubtensorModule::get_stakes_this_interval_for_hotkey(&hotkey),
//             0
//         );
//     });
// }

#[test]
fn test_add_stake_under_limit() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let who: <Test as frame_system::Config>::AccountId = hotkey_account_id.into();
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_stakes = 2;

        SubtensorModule::set_target_stakes_per_interval(max_stakes);

        let call: pallet_subtensor::Call<Test> = pallet_subtensor::Call::add_stake {
            hotkey: hotkey_account_id,
            amount_staked: 1,
        };
        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorSignedExtension::<Test>::new();
        let result = extension.validate(&who, &call.into(), &info, 10);

        assert_ok!(result);

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
        let who: <Test as frame_system::Config>::AccountId = hotkey_account_id.into();
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_unstakes = 2;

        SubtensorModule::set_target_stakes_per_interval(max_unstakes);

        let call = pallet_subtensor::Call::remove_stake {
            hotkey: hotkey_account_id,
            amount_unstaked: 1,
        };
        let info: DispatchInfo =
            DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();
        let extension = SubtensorSignedExtension::<Test>::new();
        let result = extension.validate(&who, &call.into(), &info, 10);

        assert_ok!(result);

        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, netuid, 6000);

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
//         SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, netuid, 2);
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
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(63000000, 0)
                    .add_proof_size(43991),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
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
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, netuid, amount);

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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);
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
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, netuid, amount);

        // Do the magic
        assert_noop!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                0
            ),
            Error::<Test>::NotEnoughStaketoWithdraw
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
        assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
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

        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        let result = SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            netuid,
            amount,
        );
        assert_eq!(result, Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
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
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, netuid, amount);

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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Check total balance is equal to the added stake. Even after remove stake (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, amount);
    });
}

#[test]
#[ignore]
fn test_remove_subnet_stake_total_issuance_no_change() {
    // When we remove stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(581337);
        let coldkey_account_id = U256::from(81337);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        let amount = 10000;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);
        let inital_total_issuance = Balances::total_issuance();
        assert_eq!(inital_total_issuance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, netuid, amount);

        let total_issuance_after_stake = Balances::total_issuance();

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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Check if total issuance is equal to the added stake, even after remove stake (no fee, includes reserved/locked balance)
        // Should also be equal to the total issuance after adding stake
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, total_issuance_after_stake);
        assert_eq!(total_issuance, amount);
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

        // There is not stake in the system at first, so result should be 0;
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, netuid, amount);

        // The stake that is now in the account, should equal the amount
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // The total stake should have been increased by the amount -> 0 + amount = amount
        assert_eq!(SubtensorModule::get_total_stake(), amount);
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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, netuid, amount);

        // Prelimiary checks
        assert_eq!(SubtensorModule::get_total_stake(), amount);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_stake_on_hotkey_account(&hotkey_id, netuid, amount);

        // The stake on the hotkey account should be 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        // The total amount of stake should be 0
        assert_eq!(SubtensorModule::get_total_stake(), 0);
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
        let neuron_uid;
        match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => neuron_uid = k,
            Err(e) => panic!("Error: {:?}", e),
        }
        //let neuron_uid_ex = SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id);
        let neuron_uid_ex;
        match SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id) {
            Ok(k) => neuron_uid_ex = k,
            Err(e) => panic!("Error: {:?}", e),
        }
        //Add some stake that can be removed
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, netuid, amount);

        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_stake_on_hotkey_account(&hotkey_id, netuid, amount);
        //
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            0
        );
    });
}

// /************************************************************
// 	staking::increase_total_stake() tests
// ************************************************************/
#[test]
fn test_increase_total_stake_ok() {
    new_test_ext(1).execute_with(|| {
        let increment = 10000;
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        SubtensorModule::increase_total_stake(increment);
        assert_eq!(SubtensorModule::get_total_stake(), increment);
    });
}

// /************************************************************
// 	staking::decrease_total_stake() tests
// ************************************************************/
#[test]
fn test_decrease_total_stake_ok() {
    new_test_ext(1).execute_with(|| {
        let initial_total_stake = 10000;
        let decrement = 5000;

        SubtensorModule::increase_total_stake(initial_total_stake);
        SubtensorModule::decrease_total_stake(decrement);

        // The total stake remaining should be the difference between the initial stake and the decrement
        assert_eq!(
            SubtensorModule::get_total_stake(),
            initial_total_stake - decrement
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
        assert_eq!(result, Err(Error::<Test>::BalanceWithdrawalError.into()));
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
        assert_eq!(
            SubtensorModule::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount),
            true
        );
    });
}

#[test]
fn test_can_remove_balance_from_coldkey_account_err_insufficient_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(87987984);
        let initial_amount = 10000;
        let remove_amount = 20000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
        assert_eq!(
            SubtensorModule::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount),
            false
        );
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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, netuid, intial_amount);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
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
        assert_eq!(
            SubtensorModule::has_enough_stake(&coldkey_id, &hotkey_id, netuid, 5000),
            true
        );
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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, netuid, intial_amount);
        assert_eq!(
            SubtensorModule::has_enough_stake(&coldkey_id, &hotkey_id, netuid, 5000),
            false
        );
    });
}

#[test]
fn test_non_existent_account() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
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
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&(U256::from(0))),
            10
        );
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

        // Neither key can add stake because they dont have fundss.
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

        // We have enough, but the keys are not registered.
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::NotRegistered.into())
        );

        // Cant remove either.
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );

        // Neither key can become a delegate either because we are not registered.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0
            ),
            Err(Error::<Test>::NotRegistered.into())
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
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 100 );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 100 );
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Cant remove these funds because we are not delegating.
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 200);

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
            Err(Error::<Test>::AlreadyDelegate.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1
            ),
            Err(Error::<Test>::AlreadyDelegate.into())
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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 500);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 400);
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 400 );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 500 );
        assert_eq!(SubtensorModule::get_total_stake(), 900);

        // Lets emit inflation through the hot and coldkeys.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 1000);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 1000);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            601
        ); // 200 + 1000 x ( 200 / 500 ) = 200 + 400 = 600 ~= 601
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            700
        ); // 200 + 1000 x ( 200 / 400 ) = 200 + 500 = 700
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            899
        ); // 300 + 1000 x ( 300 / 500 ) = 300 + 600 = 900 ~= 899
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            700
        ); // 200 + 1000 x ( 200 / 400 ) = 300 + 600 = 700
        assert_eq!(SubtensorModule::get_total_stake(), 2900); // 600 + 700 + 900 + 700 = 2900

        // // Try unstaking too much.
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
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
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            501
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            600
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            799
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            600
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
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Lets make this new key a delegate with a 50% take (default take value in tests).
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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 3_000);
        assert_eq!(SubtensorModule::get_total_stake(), 5_500);

        // Lets emit inflation through this new key with distributed ownership.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, netuid, 0, 1000);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            1_668
        ); // 1000 + 500 + 500 * (1000/3000) = 1500 + 166.6666666667 = 1,668
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(SubtensorModule::get_total_stake(), 6_500); // before + 1_000 = 5_500 + 1_000 = 6_500

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
        )); // 50% take - default value for tests.
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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey3), 4000);
        assert_eq!(SubtensorModule::get_total_stake(), 10_500);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey3, netuid, 0, 1000);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey3, netuid),
            1125
        ); // 1000 + 50% * 1000 * 1000/4000 = 1125
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey3, netuid),
            1125
        ); // 1000 + 50% * 1000 * 1000/4000 = 1125
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey3, netuid),
            1125
        ); // 1000 + 50% * 1000 * 1000/4000 = 1125
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey3, &hotkey3, netuid),
            1625
        ); // 1000 + 125 * 3 + 1000 * 1000/4000 = 1625
        assert_eq!(SubtensorModule::get_total_stake(), 11_500); // before + 1_000 = 10_500 + 1_000 = 11_500
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

        // Neither key can add stake because they dont have fundss.
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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 200);

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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 500);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 400);
        assert_eq!(SubtensorModule::get_total_stake(), 900);

        // Lets emit inflation through the hot and coldkeys.
        // fist emission arg is for a server. This should only go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 123, 2_000); // 2_123 total emission.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            801
        ); // 200 + (200 + 1000 x ( 200 / 500 )) = 200 + (200 + 400) = 800 ~= 801
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            899
        ); // 300 + 1000 x ( 300 / 500 ) = 300 + 600 = 900 ~= 899
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 1_700); // initial + server emission + validator emission = 799 + 899 = 1_698

        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            1_200
        ); // 200 + (0 + 2000 x ( 200 / 400 )) = 200 + (1000) = 1_200
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            1_323
        ); // 200 + (123 + 2000 x ( 200 / 400 )) = 200 + (1_200) = 1_323
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 2_523); // 400 + 2_123
        assert_eq!(SubtensorModule::get_total_stake(), 4_223); // 1_700 + 2_523 = 4_223

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time only server emission. This should go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 150, 0);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            1_151
        ); // + 350 = 1_151
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid),
            1_200
        ); // No change.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid),
            899
        ); // No change.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            1_473
        ); // 1_323 + 150 = 1_473
        assert_eq!(SubtensorModule::get_total_stake(), 4_723); // 4_223 + 500 = 4_823

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
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        assert_eq!(SubtensorModule::get_total_stake(), 5_623); // 4_723 + 900 = 5_623

        // Lets make this new key a delegate with a 50% take (default take for tests).
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2
        ));

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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 3_000);
        assert_eq!(SubtensorModule::get_total_stake(), 7_723); // 5_623 + (1_000 + 1_000 + 100) = 7_723

        // Lets emit inflation through this new key with distributed ownership.
        // We will emit 100 server emission, which should go in-full to the owner of the hotkey.
        // We will emit 1000 validator emission, which should be distributed in-part to the nominators.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, netuid, 100, 1000);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            1_768
        ); // 1000 + 100 + 500 + 500 * (1000/3000) = 100 + 1500 + 166.6666666667 ~= 1,768.6666666667
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(SubtensorModule::get_total_stake(), 8_823); // 7_723 + 1_100 = 8_823

        // Lets emit MORE inflation through this new key with distributed ownership.
        // This time we do ONLY server emission
        // We will emit 123 server emission, which should go in-full to the owner of the hotkey.
        // We will emit *0* validator emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, netuid, 123, 0);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2, netuid),
            1_891
        ); // 1_768 + 123 = 1_891
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2, netuid),
            1_166
        ); // No change.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2, netuid),
            1_166
        ); // No change.
        assert_eq!(SubtensorModule::get_total_stake(), 8_946); // 8_823 + 123 = 8_946
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
            SubtensorModule::get_total_stake_for_hotkey(&delegate),
            // -3 for existential deposit
            (100000 * 3) - 3
        );
        assert_eq!(SubtensorModule::get_total_stake(), (100000 * 3) - 3);
        assert_eq!(
            SubtensorModule::get_total_stake_for_subnet(netuid),
            (100000 * 3) - 3
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&delegate, netuid),
            (100000 * 3) - 3
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegate),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&nominator1),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&nominator2),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&delegate),
            delegate
        );
        assert_eq!(SubtensorModule::hotkey_account_exists(&delegate), true);
        assert_eq!(SubtensorModule::hotkey_account_exists(&nominator1), false);
        assert_eq!(SubtensorModule::hotkey_account_exists(&nominator2), false);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&delegate, &delegate, netuid),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator1,
                &delegate,
                netuid
            ),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator2,
                &delegate,
                netuid
            ),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_coldkey(&delegate, &delegate),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_coldkey(&delegate, &nominator1),
            99_999
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_coldkey(&delegate, &nominator1),
            99_999
        );
        SubtensorModule::emit_inflation_through_hotkey_account(&delegate, netuid, 0, 1000);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&delegate, &delegate, netuid),
            (100000 + 1000 / 3 + 1 - 1) // Need to account for existential deposit
        ); // The +1 is from the residual.
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator1,
                &delegate,
                netuid
            ),
            (100000 + 1000 / 3 - 1) // Need to account for existential deposit
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                &nominator2,
                &delegate,
                netuid
            ),
            (100000 + 1000 / 3 - 1) // Need to account for existential deposit
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

        // Neither key can add stake because they dont have fundss.
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
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 111);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 234);
        // Verify the full emission occurs.
        assert_eq!(SubtensorModule::get_total_stake(), 200 + 111 + 234); // 200 + 111 + 234 = 545

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

        assert_eq!(SubtensorModule::get_total_stake(), 545 + 500); // 545 + 500 = 1045

        // Lets emit inflation with delegatees, with both validator and server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 123, 2_000); // 2_123 total emission.

        assert_eq!(SubtensorModule::get_total_stake(), 1045 + 1_200 + 2_123); // before + 1_200 + 2_123 = 4_368

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time JUSt server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 150, 0);

        assert_eq!(SubtensorModule::get_total_stake(), 4_368 + 350 + 150); // before + 350 + 150 = 4_868

        // Lastly, do only validator emission

        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 12_948);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid, 0, 1_874);

        assert_eq!(SubtensorModule::get_total_stake(), 4_868 + 12_948 + 1_874); // before + 12_948 + 1_874 = 19_690
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
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey0_id,
            &hotkey_id,
            netuid,
            amount,
        );
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey1_id,
            &hotkey_id,
            netuid,
            amount + 2,
        );
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey2_id,
            &hotkey_id,
            netuid,
            amount + 3,
        );
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount * 4 + (2 + 3 + 4)
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        SubtensorModule::unstake_all_coldkeys_from_hotkey_account(&hotkey_id);

        // Verify total stake is 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

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
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey0_id,
            &hotkey_id,
            netuid,
            amount,
        );

        // Verify free balance is 0 for coldkey
        assert_eq!(Balances::free_balance(coldkey0_id), 0);

        // Verify total stake is correct
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        SubtensorModule::unstake_all_coldkeys_from_hotkey_account(&hotkey_id);

        // Verify total stake is 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

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

// Verify that InitialDefaultTake is between 50% and u16::MAX-1, this is important for other tests
#[test]
fn test_delegate_take_limit() {
    new_test_ext(1).execute_with(|| {
        assert_eq!(InitialDefaultTake::get() >= u16::MAX / 2, true);
        assert_eq!(InitialDefaultTake::get() <= u16::MAX - 1, true);
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

        // Coldkey / hotkey 0 become delegates with 5% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            InitialDefaultTake::get()
        );

        // Coldkey / hotkey 0 decreases take to 10%
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

        // Coldkey / hotkey 0 become delegates with 5% take
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
            u16::MAX / 20
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 20
        );

        // Coldkey / hotkey 0 tries to increase take to 10%
        assert_eq!(
            SubtensorModule::do_decrease_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                u16::MAX / 10
            ),
            Err(Error::<Test>::InvalidTake.into())
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 20
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

        // Coldkey / hotkey 0 become delegates with 5% take
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
            u16::MAX / 20
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 20
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 decreases take to 10%
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

        // Coldkey / hotkey 0 become delegates with 10% take
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
            u16::MAX / 10
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 10
        );

        // Coldkey / hotkey 0 tries to decrease take to 5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                u16::MAX / 20
            ),
            Err(Error::<Test>::InvalidTake.into())
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 10
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

        // Coldkey / hotkey 0 become delegates with 10% take
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
            u16::MAX / 10
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 10
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

        // Coldkey / hotkey 0 attempt to become delegates with take above maximum
        // (Disable this check if InitialDefaultTake is u16::MAX)
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
                Err(Error::<Test>::InvalidTake.into())
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
        assert_eq!(SubtensorModule::get_total_stake(), 100);
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
        assert_eq!(SubtensorModule::get_total_stake(), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 0);

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
            400
        ); // 100 + 50% * 400 + 50% * 200 = 400
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
        assert_eq!(SubtensorModule::get_total_stake(), 100);
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
        assert_eq!(SubtensorModule::get_total_stake(), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 0);

        // Coldkey / hotkey 0 decrease take to 10%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            u16::MAX / 10
        ));

        // Lets emit inflation through this new key with distributed ownership.
        // We will emit 0 server emission (which should go in-full to the owner of the hotkey).
        // We will emit 400 validator emission, which should be distributed in-part to the nominators.
        //
        // Total initial stake is 200
        // Delegate's initial stake is 100, which is 50% of total stake
        //  => Delegate will receive 50% of emission (200) + 10% take (20) of nominator reward (200)
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid, 0, 400);
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid),
            320
        ); // 100 + 50% * 400 + 10% * 200 = 320
    });
}

#[test]
fn test_can_set_different_take_per_subnet() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);
        let netuid1 = 1;
        let netuid2 = 2;

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Add networks
        add_network(netuid1, 0, 0);
        add_network(netuid2, 0, 0);

        // Register the neuron to networks
        register_ok_neuron(netuid1, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid2, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid1),
            InitialDefaultTake::get()
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid2),
            InitialDefaultTake::get()
        );

        // Decrease delegate take to 10% on subnet 1
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid1,
            u16::MAX / 10
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid1),
            u16::MAX / 10
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid2),
            InitialDefaultTake::get()
        );

        // Decrease delegate take to 5% on subnet 2
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid2,
            u16::MAX / 20
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid1),
            u16::MAX / 10
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid2),
            u16::MAX / 20
        );
    });
}

#[test]
fn test_different_subnet_take_different_distribution() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = 1;
        let netuid2 = 2;
        // Make two accounts.
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);

        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);
        SubtensorModule::set_max_registrations_per_block(netuid1, 4);
        SubtensorModule::set_max_allowed_uids(netuid1, 10); // Allow at least 10 to be registered at once, so no unstaking occurs
        SubtensorModule::set_max_registrations_per_block(netuid2, 4);
        SubtensorModule::set_max_allowed_uids(netuid2, 10); // Allow at least 10 to be registered at once, so no unstaking occurs

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 100000);

        // Register the 2 neurons to new networks.
        add_network(netuid1, 0, 0);
        add_network(netuid2, 0, 0);
        register_ok_neuron(netuid1, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid2, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));

        // Coldkey / hotkey 0 remains at 50% take on subnet 1
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid1),
            u16::MAX / 2
        );

        // Coldkey / hotkey 0 sets the take on subnet 2 to 10%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid2,
            u16::MAX / 10
        ));

        // Stake 100 from coldkey/hotkey 0 to subnet 1
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid1,
            100
        ));

        // Stake 100 from coldkey/hotkey 0 to subnet 2
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid2,
            100
        ));

        // Coldkey 1 adds 100 delegated stake to coldkey/hotkey 0 on subnet 1
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid1,
            100
        ));

        // Coldkey 1 adds 100 delegated stake to coldkey/hotkey 0 on subnet 2
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid2,
            100
        ));

        // Stake assertions
        //   Subnet 1:
        //           hot0    hot1
        //   cold0   100     0
        //   cold1   100     0
        //
        //   Subnet 2:
        //           hot0    hot1
        //   cold0   100     0
        //   cold1   100     0
        //   ----------------------
        //   total   400  +  0     = 400
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid1),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid1),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid1),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid1),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid2),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid2),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid2),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid2),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 400);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 400);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 0);

        // Subnet 1 emission
        //
        // Emit inflation through hotkey0 on subnet 1.
        // We will emit 0 server emission (which should go in-full to the owner of the hotkey).
        // We will emit 400 validator emission, which should be distributed in-part to the nominators.
        //
        // Total subnet initial stake is 200
        //
        // Stake ratio of coldkey 0 on subnet 1: 50%
        // Rewards
        //              take               nomination
        //     cold0    50%*400 = 200      50%*200 = 100
        //     cold1    0                  50%*200 = 100
        //
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid1, 0, 400);

        // New stake values
        //   Subnet 1:
        //           hot0    hot1
        //   cold0   400     0
        //   cold1   200     0
        //
        //   Subnet 2:
        //           hot0    hot1
        //   cold0   100     0
        //   cold1   100     0
        //   ----------------------
        //   total   800  +  0     = 800
        //
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid1),
            400
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid1),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid1),
            200
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid1),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid2),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid2),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid2),
            100
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid2),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 800);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 800);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 0);

        // Subnet 2 emission
        //
        // Emit inflation through hotkey0 on subnet 2.
        // We will emit 0 server emission (which should go in-full to the owner of the hotkey).
        // We will emit 400 validator emission, which should be distributed in-part to the nominators.
        //
        // Total subnet initial stake is 200
        //
        // Stake ratio of coldkey 0 on subnet 2: 50%
        // Rewards
        //              take               nomination
        //     cold0    10%*400 = 40       50%*360 = 180
        //     cold1    0                  50%*360 = 180
        //
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid2, 0, 400);

        // New stake values
        //   Subnet 1:
        //           hot0    hot1
        //   cold0   400     0
        //   cold1   200     0
        //
        //   Subnet 2:
        //           hot0    hot1
        //   cold0   320     0
        //   cold1   280     0
        //   ----------------------
        //   total   1200 +  0     = 1200
        //
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid1),
            400
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid1),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid1),
            200
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid1),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0, netuid2),
            320
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1, netuid2),
            0
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0, netuid2),
            280
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid2),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 1200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 1200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 0);
    });
}

#[test]
// Set up 32 subnets with a total of 1024 nodes each, and a root network with 1024 nodes.
// Each subnet has a total of 1024 nodes, and a root network has 1024 nodes.
// Register 10 neurons on each subnet.
// Add a stake of 100 TAO to each neuron.
// Run epochs for each subnet.
// Check that the total stake is correct.
fn test_subnet_stake_calculation() {
    new_test_ext(1).execute_with(|| {
        pallet_subtensor::migration::migrate_create_root_network::<Test>();
        // Setup constants
        const NUM_SUBNETS: u16 = 32;
        const NUM_NEURONS_PER_SUBNET: u16 = 10;
        const ROOT_STAKE_PER_NEURON: u64 = 1000; // Stake at the root level per neuron
        const SUBNET_STAKE_PER_NEURON: u64 = 100; // Stake at the subnet level per neuron

        let root: u16 = 0;
        let tempo: u16 = 13;

        add_network(root, tempo, 0);

        // Add networks for each subnet UID
        for netuid in 1..=NUM_SUBNETS {
            add_network(netuid, tempo, 0);
        }

        // Setup variables to track total expected stakes
        let mut total_root_stake: u64 = 0;
        let mut total_subnet_stake: u64 = 0;

        for netuid in 1..=NUM_SUBNETS {
            for neuron_index in 0..NUM_NEURONS_PER_SUBNET {
                let hotkey = U256::from((netuid as u64) * 1000 + neuron_index as u64); // Unique hotkey for each neuron
                let coldkey = U256::from((netuid as u64) * 10000 + neuron_index as u64); // Unique coldkey for each neuron
                SubtensorModule::set_target_stakes_per_interval(10000);
                SubtensorModule::set_max_registrations_per_block(netuid, 500);
                SubtensorModule::set_target_registrations_per_interval(netuid, 500);

                // Increase balance for coldkey account
                SubtensorModule::add_balance_to_coldkey_account(
                    &coldkey,
                    ROOT_STAKE_PER_NEURON + SUBNET_STAKE_PER_NEURON,
                );
                register_ok_neuron(netuid, hotkey, coldkey, 0);

                // Add stakes at both the root and subnet levels
                assert_ok!(SubtensorModule::add_stake(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                    hotkey,
                    ROOT_STAKE_PER_NEURON
                ));

                assert_ok!(SubtensorModule::add_subnet_stake(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                    hotkey,
                    netuid,
                    SUBNET_STAKE_PER_NEURON
                ));

                // Update total stakes
                total_root_stake += ROOT_STAKE_PER_NEURON;
                total_subnet_stake += SUBNET_STAKE_PER_NEURON;
            }
        }

        step_block(1);

        // SubtensorModule::epoch(, 0);

        // Print Subnet Emission Value for each netuid after the block step
        for netuid in 1..=NUM_SUBNETS {
            // let emission_values = SubtensorModule::get_emission(netuid);
            // for emission_value in emission_values {
            SubtensorModule::epoch(netuid, 100_000_000);
            // }
            let emission_value = SubtensorModule::get_subnet_emission_value(netuid);
            println!(
                "Subnet Emission Value for netuid {}: {}",
                netuid, emission_value
            );
        }

        let total_neurons = NUM_SUBNETS as u64 * NUM_NEURONS_PER_SUBNET as u64;

        // Check total stakes across all subnets
        let expected_total_stake_adjusted = total_root_stake + total_subnet_stake - total_neurons;
        let actual_total_stake = SubtensorModule::get_total_stake();
        assert_eq!(
            actual_total_stake, expected_total_stake_adjusted,
            "The total stake across all subnets did not match the expected value."
        );

        // After checking the total stake, proceed to remove the stakes
        for netuid in 1..=NUM_SUBNETS {
            for neuron_index in 0..NUM_NEURONS_PER_SUBNET {
                let hotkey = U256::from((netuid as u64) * 1000 + neuron_index as u64);
                let coldkey = U256::from((netuid as u64) * 10000 + neuron_index as u64);

                // Remove subnet stake first
                assert_ok!(SubtensorModule::remove_subnet_stake(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                    hotkey,
                    netuid,
                    // Need to account for existential deposit
                    SUBNET_STAKE_PER_NEURON - 1
                ));

                total_subnet_stake -= SUBNET_STAKE_PER_NEURON;
            }
        }

        step_block(1);

        // Print Subnet Emission Value for each netuid after the block step
        for netuid in 1..=NUM_SUBNETS {
            let emission_value = SubtensorModule::get_subnet_emission_value(netuid);
            println!(
                "Subnet Emission Value for netuid {}: {}",
                netuid, emission_value
            );
        }

        // Verify that the total stake has been correctly reduced to just the root stake
        let expected_total_stake_after_removal = total_root_stake;
        let actual_total_stake_after_removal = SubtensorModule::get_total_stake();
        assert_eq!(
            actual_total_stake_after_removal, expected_total_stake_after_removal,
            "The total stake after removal did not match the expected value."
        );

        // Finally , remove the root stake
        for netuid in 1..=NUM_SUBNETS {
            for neuron_index in 0..NUM_NEURONS_PER_SUBNET {
                let hotkey = U256::from((netuid as u64) * 1000 + neuron_index as u64);
                let coldkey = U256::from((netuid as u64) * 10000 + neuron_index as u64);

                assert_ok!(SubtensorModule::remove_stake(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                    hotkey,
                    // netuid,
                    ROOT_STAKE_PER_NEURON
                ));

                // Update total stakes to reflect removal
                total_root_stake -= ROOT_STAKE_PER_NEURON;
            }
        }

        step_block(1);

        // Print Subnet Emission Value for each netuid after the block step
        for netuid in 1..=NUM_SUBNETS {
            let emission_value = SubtensorModule::get_subnet_emission_value(netuid);
            println!(
                "Subnet Emission Value for netuid {}: {}",
                netuid, emission_value
            );
        }

        // Verify that the total stake has been correctly reduced to 0
        let expected_total_stake_after_removal = 0;
        let actual_total_stake_after_removal = SubtensorModule::get_total_stake();
        assert_eq!(
            actual_total_stake_after_removal, expected_total_stake_after_removal,
            "The total stake after removal did not match the expected value."
        );
    });
}

#[test]
fn test_three_subnets_with_different_stakes() {
    new_test_ext(1).execute_with(|| {
        pallet_subtensor::migration::migrate_create_root_network::<Test>();
        // Setup constants
        const NUM_SUBNETS: u16 = 3; // Only 3 subnets
        const NUM_NEURONS_PER_SUBNET: u16 = 10;
        // Different stake amounts for each subnet
        const STAKE_AMOUNTS: [u64; NUM_SUBNETS as usize] = [100, 200, 300];

        let root: u16 = 0;
        let tempo: u16 = 13;

        add_network(root, tempo, 0);

        // Add networks for each subnet UID
        for netuid in 1..=NUM_SUBNETS {
            add_network(netuid, tempo, 0);
        }

        for netuid in 1..=NUM_SUBNETS {
            for neuron_index in 0..NUM_NEURONS_PER_SUBNET {
                let hotkey = U256::from((netuid as u64) * 1000 + neuron_index as u64);
                let coldkey = U256::from((netuid as u64) * 10000 + neuron_index as u64);

                SubtensorModule::set_max_registrations_per_block(netuid, 500);
                SubtensorModule::set_target_registrations_per_interval(netuid, 500);

                // Increase balance for coldkey account
                SubtensorModule::add_balance_to_coldkey_account(
                    &coldkey,
                    STAKE_AMOUNTS[netuid as usize - 1],
                );
                register_ok_neuron(netuid, hotkey, coldkey, 0);

                // Add stake at the subnet level
                assert_ok!(SubtensorModule::add_subnet_stake(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                    hotkey,
                    netuid,
                    STAKE_AMOUNTS[netuid as usize - 1],
                ));

                // Assert individual stake amounts
                let stake_for_neuron = SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(
                    &coldkey, &hotkey, netuid,
                );
                assert_eq!(
                    stake_for_neuron,
                    // Need to account for existential deposit
                    STAKE_AMOUNTS[netuid as usize - 1] - 1,
                    "The stake for neuron {} in subnet {} did not match the expected value.",
                    neuron_index,
                    netuid
                );
            }
        }

        // Verify the total stake for each subnet
        for netuid in 1..=NUM_SUBNETS {
            let total_stake_for_subnet = SubtensorModule::get_total_stake_for_subnet(netuid);
            // Adjust the expected total stake to account for the existential deposit for each neuron
            let expected_total_stake =
                (STAKE_AMOUNTS[netuid as usize - 1] - 1) * NUM_NEURONS_PER_SUBNET as u64;
            assert_eq!(
                total_stake_for_subnet, expected_total_stake,
                "The total stake for subnet {} did not match the expected value.",
                netuid
            );
        }
    });
}

#[test]
fn test_register_neurons_and_stake_different_amounts() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;

        // Setup the network
        add_network(netuid, tempo, 0);

        SubtensorModule::set_max_registrations_per_block(netuid, NUM_NEURONS);
        SubtensorModule::set_target_registrations_per_interval(netuid, NUM_NEURONS);

        // Define the number of neurons and their stake amounts
        const NUM_NEURONS: u16 = 10;
        let stake_amounts: [u64; NUM_NEURONS as usize] =
            [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];

        for i in 0..NUM_NEURONS {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i + 100); // Ensure coldkey is different but consistent

            // Increase balance for coldkey account
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake_amounts[i as usize]);

            // Register neuron
            register_ok_neuron(netuid, hotkey, coldkey, start_nonce);

            // Stake the specified amount
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                stake_amounts[i as usize],
            ));

            // Assert the stake for the neuron is as expected
            let stake_for_neuron =
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey, &hotkey, netuid);
            assert_eq!(
                stake_for_neuron,
                stake_amounts[i as usize] - 1, // Need to account for existential deposit
                "The stake for neuron {} did not match the expected value.",
                i
            );
        }

        // verify the total stake for the subnet if needed
        let total_stake_for_subnet = SubtensorModule::get_total_stake_for_subnet(netuid);
        // Adjust the expected total stake to account for the existential deposit
        let expected_total_stake: u64 = stake_amounts.iter().sum::<u64>() - (NUM_NEURONS as u64);
        assert_eq!(
            total_stake_for_subnet, expected_total_stake,
            "The total stake for subnet {} did not match the expected value.",
            netuid
        );
    });
}

#[test]
fn test_substake_increases_stake_of_only_targeted_neuron() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;

        // Setup the network
        add_network(netuid, tempo, 0);

        SubtensorModule::set_max_registrations_per_block(netuid, NUM_NEURONS);
        SubtensorModule::set_target_registrations_per_interval(netuid, NUM_NEURONS);

        // Define the number of neurons and initial stake amounts
        const NUM_NEURONS: u16 = 3;
        let initial_stake: u64 = 1000;

        // Register neurons and stake an initial amount
        for i in 0..NUM_NEURONS {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i + 100); // Ensure coldkey is different but consistent

            // Increase balance for coldkey account
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake * 2);

            // Register neuron and add initial stake
            register_ok_neuron(netuid, hotkey, coldkey, 0);
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                initial_stake,
            ));
        }

        // Perform a substake operation on the first neuron
        let substake_amount: u64 = 500;
        let target_neuron_hotkey = U256::from(0);
        let target_neuron_coldkey = U256::from(100);
        SubtensorModule::set_target_stakes_per_interval(10000);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(target_neuron_coldkey),
            target_neuron_hotkey,
            netuid,
            substake_amount,
        ));

        // Verify that only the stake of the targeted neuron has increased
        for i in 0..NUM_NEURONS {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i + 100);
            let expected_stake = if hotkey == target_neuron_hotkey {
                initial_stake + substake_amount
            } else {
                initial_stake
            };

            let actual_stake =
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey, &hotkey, netuid);
            assert_eq!(
                actual_stake, expected_stake,
                "Stake for neuron {} did not match the expected value.",
                i
            );
        }
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

        // Coldkey / hotkey 0 become delegates with 50% take
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
            u16::MAX / 20
        ));
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 20
        );

        // Coldkey / hotkey 0 increases take to 10%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                u16::MAX / 10
            ),
            Err(Error::<Test>::TxRateLimitExceeded.into())
        );
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey0, netuid),
            u16::MAX / 20
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
fn add_weighted_stake_success() {
    new_test_ext(1).execute_with(|| {
        // Setup
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuids = vec![1, 2];
        let values = vec![2, 1]; // Weights for the networks

        // Add balance to the coldkey account
        let initial_balance = 100000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_balance);
        log::info!("Added balance {} to coldkey {:?}", initial_balance, coldkey);

        // Add networks and register neurons
        let mut total_initial_stake = 0;
        for &netuid in &netuids {
            add_network(netuid, 0, 0); // Assuming tempo and other parameters are zero for simplicity
            register_ok_neuron(netuid, hotkey, coldkey, 0); // Assuming start_nonce is zero
            log::info!(
                "Network {} added and neuron registered for hotkey {:?}, coldkey {:?}",
                netuid,
                hotkey,
                coldkey
            );

            // Set registration limits for each network based on netuid
            SubtensorModule::set_max_registrations_per_block(netuid, netuid as u16);
            SubtensorModule::set_target_registrations_per_interval(netuid, netuid as u16);
            log::info!(
                "Set max and target registrations for netuid {} to {}",
                netuid,
                netuid
            );

            // Initially add some stake to each subnet
            let initial_stake = 10000; // Arbitrary initial stake for simplicity
            assert_ok!(SubtensorModule::add_subnet_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                initial_stake,
            ));
            total_initial_stake += initial_stake;
            log::info!(
                "Initial stake of {} added to netuid {}",
                initial_stake,
                netuid
            );
        }

        // Perform the weighted stake redistribution
        assert_ok!(SubtensorModule::add_weighted_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuids.clone(),
            values.clone()
        ));
        log::info!(
            "Weighted stake redistributed for hotkey {:?} across netuids {:?} with values {:?}",
            hotkey,
            netuids,
            values
        );

        // Assertions
        let total_stake: u64 = SubtensorModule::get_coldkey_balance(&coldkey);
        log::info!("Total stake after redistribution: {}", total_stake);
        assert!(
            total_stake < initial_balance,
            "Stake should be less than initial balance due to redistribution."
        );

        let total_weights: u16 = values.iter().sum();
        for (i, &netuid) in netuids.iter().enumerate() {
            let expected_stake =
                (total_initial_stake as u32 * values[i] as u32 / total_weights as u32) as u64;
            let stake =
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey, &hotkey, netuid);
            log::info!(
                "Expected redistributed stake for netuid {}: {}, Actual stake: {}",
                netuid,
                expected_stake,
                stake
            );
            assert_eq!(
                stake, expected_stake,
                "Redistributed stake for netuid {} did not match the expected value.",
                netuid
            );
        }
    });
}

#[test]
fn test_add_weighted_stake_success_32_networks() {
    new_test_ext(1).execute_with(|| {
        // Setup
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let num_networks = 32;
        let netuids: Vec<u16> = (1..=num_networks).collect();
        let values: Vec<u16> = vec![1; num_networks as usize]; // Equal weights for simplicity

        // Add balance to the coldkey account
        let initial_balance = 100000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_balance);
        log::info!("Added balance {} to coldkey {:?}", initial_balance, coldkey);
        SubtensorModule::set_target_stakes_per_interval(1000);

        // Add networks and register neurons
        let mut total_initial_stake = 0;
        let initial_stake_per_network = 1000; // Arbitrary initial stake for simplicity
        for &netuid in &netuids {
            add_network(netuid, 0, 0); // Assuming tempo and other parameters are zero for simplicity
            register_ok_neuron(netuid, hotkey, coldkey, 0); // Assuming start_nonce is zero
            log::info!(
                "Network {} added and neuron registered for hotkey {:?}, coldkey {:?}",
                netuid,
                hotkey,
                coldkey
            );

            // Set registration limits for each network based on netuid
            SubtensorModule::set_max_registrations_per_block(netuid, 50);
            SubtensorModule::set_target_registrations_per_interval(netuid, 50);
            log::info!(
                "Set max and target registrations for netuid {} to {}",
                netuid,
                netuid
            );

            // Initially add some stake to each subnet
            assert_ok!(SubtensorModule::add_subnet_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                initial_stake_per_network,
            ));
            total_initial_stake += initial_stake_per_network;
            log::info!(
                "Initial stake of {} added to netuid {}",
                initial_stake_per_network,
                netuid
            );
        }

        // Perform the weighted stake redistribution
        assert_ok!(SubtensorModule::add_weighted_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuids.clone(),
            values.clone()
        ));
        log::info!(
            "Weighted stake redistributed for hotkey {:?} across netuids {:?} with values {:?}",
            hotkey,
            netuids,
            values
        );

        // Assertions
        let total_stake: u64 = SubtensorModule::get_coldkey_balance(&coldkey);
        log::info!("Total stake after redistribution: {}", total_stake);
        assert!(
            total_stake < initial_balance,
            "Stake should be less than initial balance due to redistribution."
        );

        let total_weights: u16 = values.iter().sum();
        for (i, &netuid) in netuids.iter().enumerate() {
            let expected_stake =
                (total_initial_stake as u32 * values[i] as u32 / total_weights as u32) as u64;
            let stake =
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey, &hotkey, netuid);
            log::info!(
                "Expected redistributed stake for netuid {}: {}, Actual stake: {}",
                netuid,
                expected_stake,
                stake
            );
            assert_eq!(
                stake, expected_stake,
                "Redistributed stake for netuid {} did not match the expected value.",
                netuid
            );
        }
    });
}

#[test]
fn add_weighted_stake_success_3_to_32_networks() {
    new_test_ext(1).execute_with(|| {
        // Setup
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let num_networks = 32; // Total networks
        let initial_stake_networks = 3; // Networks to initially stake
        let netuids: Vec<u16> = (1..=num_networks).collect();
        let values: Vec<u16> = vec![1; num_networks as usize]; // Equal weights for simplicity
        const NUM_NEURONS: u16 = 10; // Number of neurons per network

        // Add balance to the coldkey account
        let initial_balance = 100000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_balance);
        SubtensorModule::set_target_stakes_per_interval(1000);

        log::info!("Added balance {} to coldkey {:?}", initial_balance, coldkey);

        // Add networks, register neurons, and set registration limits
        let mut total_initial_stake = 0;
        let initial_stake_per_network = 10000; // Arbitrary initial stake for simplicity
        for &netuid in &netuids {
            add_network(netuid, 0, 0); // Assuming tempo and other parameters are zero for simplicity
            register_ok_neuron(netuid, hotkey, coldkey, 0); // Assuming start_nonce is zero
            log::info!(
                "Network {} added and neuron registered for hotkey {:?}, coldkey {:?}",
                netuid,
                hotkey,
                coldkey
            );

            // Set registration limits for each network
            SubtensorModule::set_max_registrations_per_block(netuid, 50);
            SubtensorModule::set_target_registrations_per_interval(netuid, 50);
            log::info!(
                "Set max and target registrations for netuid {} to {}",
                netuid,
                NUM_NEURONS
            );

            // Initially add some stake to each subnet (only for the first 3 networks)
            if netuid <= initial_stake_networks {
                assert_ok!(SubtensorModule::add_subnet_stake(
                    RuntimeOrigin::signed(coldkey),
                    hotkey,
                    netuid,
                    initial_stake_per_network,
                ));
                total_initial_stake += initial_stake_per_network;
                log::info!(
                    "Initial stake of {} added to netuid {}",
                    initial_stake_per_network,
                    netuid
                );
            }
        }

        // Perform the weighted stake redistribution across all 32 networks
        assert_ok!(SubtensorModule::add_weighted_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuids.clone(),
            values.clone()
        ));
        log::info!(
            "Weighted stake redistributed for hotkey {:?} across netuids {:?} with values {:?}",
            hotkey,
            netuids,
            values
        );

        // Assertions
        let total_stake: u64 = SubtensorModule::get_coldkey_balance(&coldkey);
        log::info!("Total stake after redistribution: {}", total_stake);
        assert!(
            total_stake < initial_balance,
            "Stake should be less than initial balance due to redistribution."
        );

        let total_weights: u16 = values.iter().sum();
        for (i, &netuid) in netuids.iter().enumerate() {
            let expected_stake =
                (total_initial_stake as u32 * values[i] as u32 / total_weights as u32) as u64;
            let stake =
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey, &hotkey, netuid);
            log::info!(
                "Expected redistributed stake for netuid {}: {}, Actual stake: {}",
                netuid,
                expected_stake,
                stake
            );
            assert_eq!(
                stake, expected_stake,
                "Redistributed stake for netuid {} did not match the expected value.",
                netuid
            );
        }
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
            hotkey.into(),
            takes.clone()
        ));

        for (netuid, take) in takes {
            let actual_take = SubtensorModule::get_delegate_take(&hotkey.into(), netuid);
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

        // Assuming default take value is 32767, adjust if different
        assert_eq!(
            SubtensorModule::get_delegate_take(&hotkey, 1),
            32767,
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
            Error::<Test>::NetworkDoesNotExist
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
            Error::<Test>::InvalidTake
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
            hotkey.into(),
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
            Error::<Test>::TxRateLimitExceeded
        );
    });
}
