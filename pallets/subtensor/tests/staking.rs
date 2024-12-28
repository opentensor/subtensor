#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::*;
use sp_core::{Get, H256, U256};

/***********************************************************
    staking::add_stake() tests
************************************************************/
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_dispatch_info_ok --exact --nocapture
#[test]
fn test_add_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let amount_staked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            netuid,
            amount_staked,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(1_074_000_000, 0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_ok_no_emission --exact --nocapture
#[test]
fn test_add_stake_ok_no_emission() {
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            0
        );

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            9999
        );

        // Check if balance has decreased
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 1);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 9999);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_dividends_with_run_to_block --exact --nocapture
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
        SubtensorModule::stake_into_subnet(
            &neuron_src_hotkey_id,
            &coldkey_account_id,
            netuid,
            initial_stake,
        );

        // Check if the initial stake has arrived
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&neuron_src_hotkey_id, netuid),
            initial_stake
        );

        // Check if all three neurons are registered
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);

        // Run a couple of blocks to check if emission works
        run_to_block(2);

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&neuron_src_hotkey_id, netuid),
            initial_stake
        );

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&neuron_dest_hotkey_id, netuid),
            0
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_err_signature --exact --nocapture
#[test]
fn test_add_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(654); // bogus
        let amount = 20000; // Not used
        let netuid: u16 = 1;

        let result = SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            netuid,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_not_registered_key_pair --exact --nocapture
// DEPRECATED: allowing stake without registering a neuron.
// #[test]
// fn test_add_stake_not_registered_key_pair() {
//     new_test_ext(1).execute_with(|| {
//         let coldkey_account_id = U256::from(435445);
//         let hotkey_account_id = U256::from(54544);
//         let amount = 1337;
//         let netuid: u16 = 1;
//         add_network(netuid, 13, 0);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1800);
//         assert_eq!(
//             SubtensorModule::add_stake(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 netuid,
//                 amount
//             ),
//             Err(Error::<Test>::HotKeyAccountNotExists.into())
//         );
//     });
// }

// Deprecated

// // SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_err_neuron_does_not_belong_to_coldkey --exact --nocapture
// #[test]
// fn test_add_stake_err_neuron_does_not_belong_to_coldkey() {
//     new_test_ext(1).execute_with(|| {
//         let coldkey_id = U256::from(544);
//         let hotkey_id = U256::from(54544);
//         let other_cold_key = U256::from(99498);
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let start_nonce: u64 = 0;

//         //add network
//         add_network(netuid, tempo, 0);

//         register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
//         // Give it some $$$ in his coldkey balance
//         SubtensorModule::add_balance_to_coldkey_account(&other_cold_key, 100000);

//         // Perform the request which is signed by a different cold key
//         let result = SubtensorModule::add_stake(
//             <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
//             hotkey_id,
//             netuid,
//             1000,
//         );
//         assert_eq!(
//             result,
//             Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
//         );
//     });
// }

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_err_not_enough_belance --exact --nocapture
#[test]
fn test_add_stake_err_not_enough_belance() {
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
        let result = SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            netuid,
            60000,
        );

        assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_total_balance_no_change --exact --nocapture
#[test]
#[ignore]
fn test_add_stake_total_balance_no_change() {
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
        let initial_stake =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid);
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

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_total_issuance_no_change --exact --nocapture
#[test]
#[ignore]
fn test_add_stake_total_issuance_no_change() {
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
        let initial_stake =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid);
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid);
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

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_reset_stakes_per_interval --exact --nocapture
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

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_reset_stakes_per_interval --exact --nocapture
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));

        let current_stakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert!(current_stakes <= max_stakes);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_rate_limit_exceeded --exact --nocapture
#[test]
fn test_add_stake_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_stakes = 2;
        let block_number = 1;

        SubtensorModule::set_target_stakes_per_interval(max_stakes);
        SubtensorModule::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
            max_stakes,
            block_number,
        );

        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        assert_err!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                1,
            ),
            Error::<Test>::StakeRateLimitExceeded
        );

        let current_stakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert_eq!(current_stakes, max_stakes);
    });
}

// /***********************************************************
// 	staking::remove_stake() tests
// ************************************************************/
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_under_limit --exact --nocapture
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
        SubtensorModule::stake_into_subnet(&hotkey_account_id, &coldkey_account_id, netuid, 2);

        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            1,
        ));

        let current_unstakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert!(current_unstakes <= max_unstakes);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_under_limit --exact --nocapture
#[test]
fn test_remove_stake_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = 1;
        let start_nonce: u64 = 0;
        let tempo: u16 = 13;
        let max_unstakes = 1;
        let block_number = 1;

        SubtensorModule::set_target_stakes_per_interval(max_unstakes);
        SubtensorModule::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
            max_unstakes,
            block_number,
        );

        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 60000);
        SubtensorModule::stake_into_subnet(&hotkey_account_id, &coldkey_account_id, netuid, 2);
        assert_err!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                2,
            ),
            Error::<Test>::UnstakeRateLimitExceeded
        );

        let current_unstakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert_eq!(current_unstakes, max_unstakes);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_dispatch_info_ok --exact --nocapture
#[test]
fn test_remove_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let amount_unstaked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
            hotkey,
            netuid,
            amount_unstaked,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(1_061_000_000, 0)
                    .add_proof_size(43991),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_ok_no_emission --exact --nocapture
#[test]
fn test_remove_stake_ok_no_emission() {
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::stake_into_subnet(&hotkey_account_id, &coldkey_account_id, netuid, amount);

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_amount_zero --exact --nocapture
#[test]
fn test_remove_stake_amount_zero() {
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::stake_into_subnet(&hotkey_account_id, &coldkey_account_id, netuid, amount);

        // Do the magic
        assert_noop!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                0
            ),
            Error::<Test>::StakeToWithdrawIsZero
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_err_signature --exact --nocapture
#[test]
fn test_remove_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000; // Amount to be removed

        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            netuid,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

//  Deprecated
// // SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_err_hotkey_does_not_belong_to_coldkey --exact --nocapture
// #[test]
// fn test_remove_stake_err_hotkey_does_not_belong_to_coldkey() {
//     new_test_ext(1).execute_with(|| {
//         let coldkey_id = U256::from(544);
//         let hotkey_id = U256::from(54544);
//         let other_cold_key = U256::from(99498);
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let start_nonce: u64 = 0;

//         //add network
//         add_network(netuid, tempo, 0);

//         register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

//         // Perform the request which is signed by a different cold key
//         let result = SubtensorModule::remove_stake(
//             <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
//             hotkey_id,
//             netuid,
//             1000,
//         );
//         assert_eq!(
//             result,
//             Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
//         );
//     });
// }

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_no_enough_stake --exact --nocapture
#[test]
fn test_remove_stake_no_enough_stake() {
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
            0
        );

        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            netuid,
            amount,
        );
        assert_eq!(result, Err(Error::<Test>::NotEnoughStakeToWithdraw.into()));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_total_balance_no_change --exact --nocapture
#[test]
fn test_remove_stake_total_balance_no_change() {
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::stake_into_subnet(&hotkey_account_id, &coldkey_account_id, netuid, amount);

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Check total balance is equal to the added stake. Even after remove stake (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, amount);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_total_issuance_no_change --exact --nocapture
#[test]
#[ignore]
fn test_remove_stake_total_issuance_no_change() {
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);
        let inital_total_issuance = Balances::total_issuance();
        assert_eq!(inital_total_issuance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::stake_into_subnet(&hotkey_account_id, &coldkey_account_id, netuid, amount);

        let total_issuance_after_stake = Balances::total_issuance();

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
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
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
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
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_get_coldkey_balance_no_balance --exact --nocapture
#[test]
fn test_get_coldkey_balance_no_balance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(5454); // arbitrary
        let result = SubtensorModule::get_coldkey_balance(&coldkey_account_id);

        // Arbitrary account should have 0 balance
        assert_eq!(result, 0);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_get_coldkey_balance_with_balance --exact --nocapture
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
// 	staking::add_stake_to_hotkey_account() tests
// ************************************************************/
//  SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_to_hotkey_account_ok --exact --nocapture
#[test]
fn test_add_stake_to_hotkey_account_ok() {
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

        SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey_id, netuid, amount);

        // The stake that is now in the account, should equal the amount
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
            amount
        );

        // The total stake should have been increased by the amount -> 0 + amount = amount
        assert_eq!(SubtensorModule::get_total_stake(), amount);
    });
}

/************************************************************
    staking::remove_stake_from_hotkey_account() tests
************************************************************/
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_from_hotkey_account --exact --nocapture
#[test]
fn test_remove_stake_from_hotkey_account() {
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
        SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey_id, netuid, amount);

        // Prelimiary checks
        assert_eq!(SubtensorModule::get_total_stake(), amount);
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
            amount
        );

        // Remove stake
        SubtensorModule::unstake_from_subnet(&hotkey_id, &coldkey_id, netuid, amount);

        // The stake on the hotkey account should be 0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
            0
        );

        // The total amount of stake should be 0
        assert_eq!(SubtensorModule::get_total_stake(), 0);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_from_hotkey_account_registered_in_various_networks --exact --nocapture
#[test]
fn test_remove_stake_from_hotkey_account_registered_in_various_networks() {
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

        //Add some stake that can be removed
        SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey_id, netuid, amount);

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
            amount
        );
        // None on the other network.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid_ex),
            0
        );

        // Remove stake
        SubtensorModule::unstake_from_subnet(&hotkey_id, &coldkey_id, netuid, amount);
        //
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
            0
        );
        // None on the other network.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid_ex),
            0
        );
    });
}

// /************************************************************
// 	staking::add_balance_to_coldkey_account() tests
// ************************************************************/
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_balance_to_coldkey_account_ok --exact --nocapture
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
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_balance_from_coldkey_account_ok --exact --nocapture
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

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_balance_from_coldkey_account_failed --exact --nocapture
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
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_hotkey_belongs_to_coldkey_ok --exact --nocapture
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
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_can_remove_balane_from_coldkey_account_ok --exact --nocapture
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

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_can_remove_balance_from_coldkey_account_err_insufficient_balance --exact --nocapture
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
    staking::has_enough_stake_on_subnet() tests
************************************************************/
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_has_enough_stake_yes --exact --nocapture
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
        SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey_id, netuid, intial_amount);
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
            10000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid
            ),
            10000
        );
        assert!(SubtensorModule::has_enough_stake_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            5000
        ));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_has_enough_stake_no --exact --nocapture
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
        SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey_id, netuid, intial_amount);
        assert!(!SubtensorModule::has_enough_stake_on_subnet(
            &coldkey_id,
            &hotkey_id,
            netuid,
            5000
        ));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_non_existent_account --exact --nocapture
#[test]
fn test_non_existent_account() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::stake_into_subnet(&U256::from(0), &U256::from(0), 1, 10);
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(0),
                &U256::from(0),
                1
            ),
            10
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_on_subnet(&(U256::from(0)), 1),
            10
        );
    });
}

/************************************************************
    staking::delegating
************************************************************/

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_non_existent_account --exact --nocapture
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
    });
}

/************************************************************
    staking::unstake_all_coldkeys_from_hotkey_account() tests
************************************************************/

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_unstake_all_coldkeys_from_hotkey_account --exact --nocapture
// DEPRECATED.
// #[test]
// fn test_unstake_all_coldkeys_from_hotkey_account() {
//     new_test_ext(1).execute_with(|| {
//         let hotkey_id = U256::from(123570);
//         let coldkey0_id = U256::from(123560);

//         let coldkey1_id = U256::from(123561);
//         let coldkey2_id = U256::from(123562);
//         let coldkey3_id = U256::from(123563);

//         let amount: u64 = 10000;

//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let start_nonce: u64 = 0;

//         // Make subnet
//         add_network(netuid, tempo, 0);
//         // Register delegate
//         register_ok_neuron(netuid, hotkey_id, coldkey0_id, start_nonce);

//         match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
//             Ok(_k) => (),
//             Err(e) => panic!("Error: {:?}", e),
//         }

//         //Add some stake that can be removed
//         SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey0_id, netuid, amount);
//         SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey1_id, netuid, amount + 2);
//         SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey2_id, netuid, amount + 3);
//         SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey3_id, netuid, amount + 4);

//         // Verify free balance is 0 for all coldkeys
//         assert_eq!(Balances::free_balance(coldkey0_id), 0);
//         assert_eq!(Balances::free_balance(coldkey1_id), 0);
//         assert_eq!(Balances::free_balance(coldkey2_id), 0);
//         assert_eq!(Balances::free_balance(coldkey3_id), 0);

//         // Verify total stake is correct
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
//             amount * 4 + (2 + 3 + 4)
//         );

//         // Run unstake_all_coldkeys_from_hotkey_account
//         SubtensorModule::unstake_all_coldkeys_from_hotkey_account_on_network(&hotkey_id, netuid);

//         // Verify total stake is 0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
//             0
//         );

//         // Vefify stake for all coldkeys is 0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey_id,
//                 &coldkey0_id,
//                 netuid
//             ),
//             0
//         );
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey_id,
//                 &coldkey1_id,
//                 netuid
//             ),
//             0
//         );
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey_id,
//                 &coldkey2_id,
//                 netuid
//             ),
//             0
//         );
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey_id,
//                 &coldkey3_id,
//                 netuid
//             ),
//             0
//         );

//         // Verify free balance is correct for all coldkeys
//         assert_eq!(Balances::free_balance(coldkey0_id), amount);
//         assert_eq!(Balances::free_balance(coldkey1_id), amount + 2);
//         assert_eq!(Balances::free_balance(coldkey2_id), amount + 3);
//         assert_eq!(Balances::free_balance(coldkey3_id), amount + 4);
//     });
// }

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_unstake_all_coldkeys_from_hotkey_account_single_staker --exact --nocapture
// DEPRECATED.
// #[test]
// fn test_unstake_all_coldkeys_from_hotkey_account_single_staker() {
//     new_test_ext(1).execute_with(|| {
//         let hotkey_id = U256::from(123570);
//         let coldkey0_id = U256::from(123560);

//         let amount: u64 = 891011;

//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let start_nonce: u64 = 0;

//         // Make subnet
//         add_network(netuid, tempo, 0);
//         // Register delegate
//         register_ok_neuron(netuid, hotkey_id, coldkey0_id, start_nonce);

//         match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
//             Ok(_) => (),
//             Err(e) => panic!("Error: {:?}", e),
//         }

//         //Add some stake that can be removed
//         SubtensorModule::stake_into_subnet(&hotkey_id, &coldkey0_id, netuid, amount);

//         // Verify free balance is 0 for coldkey
//         assert_eq!(Balances::free_balance(coldkey0_id), 0);

//         // Verify total stake is correct
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
//             amount
//         );

//         // Run unstake_all_coldkeys_from_hotkey_account
//         SubtensorModule::unstake_all_coldkeys_from_hotkey_account_on_network(&hotkey_id, netuid);

//         // Verify total stake is 0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_id, netuid),
//             0
//         );

//         // Vefify stake for single coldkey is 0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey_id,
//                 &coldkey0_id,
//                 netuid
//             ),
//             0
//         );

//         // Verify free balance is correct for single coldkey
//         assert_eq!(Balances::free_balance(coldkey0_id), amount);
//     });
// }

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_faucet_ok --exact --nocapture
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
// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_clear_small_nominations --exact --nocapture
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
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot1), cold1);

        // Register hot2.
        register_ok_neuron(netuid, hot2, cold2, 0);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot2), cold2);

        // Add stake cold1 --> hot1 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot1,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 4);

        // Add stake cold2 --> hot1 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot1,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 4);

        // Add stake cold1 --> hot2 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot2,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold1, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 8);

        // Add stake cold2 --> hot2 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
            netuid,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold2, netuid),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 8);

        // Run clear all small nominations when min stake is zero (noop)
        SubtensorModule::set_nominator_min_required_stake(0);
        assert_eq!(SubtensorModule::get_nominator_min_required_stake(), 0);
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold1, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold2, netuid),
            1
        );

        // Set min nomination to 10
        // let total_cold1_stake_before = TotalColdkeyAlpha::<Test>::get(cold1, netuid);
        // let total_cold2_stake_before = TotalColdkeyAlpha::<Test>::get(cold2, netuid); (DEPRECATED)
        let total_hot1_stake_before = TotalHotkeyAlpha::<Test>::get(hot1, netuid);
        let total_hot2_stake_before = TotalHotkeyAlpha::<Test>::get(hot2, netuid);
        let total_stake_before = TotalStake::<Test>::get();
        SubtensorModule::set_nominator_min_required_stake(10);

        // Run clear all small nominations (removes delegations under 10)
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold1, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold2, netuid),
            1
        );

        // Balances have been added back into accounts.
        assert_eq!(Balances::free_balance(cold1), 9);
        assert_eq!(Balances::free_balance(cold2), 9);

        // Internal storage is updated
        // assert_eq!(
        //     TotalColdkeyAlpha::<Test>::get(cold2, netuid),
        //     total_cold2_stake_before - 1
        // ); (DEPRECATED)
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hot2, netuid),
            total_hot2_stake_before - 1
        );
        // assert_eq!(
        //     TotalColdkeyAlpha::<Test>::get(cold1, netuid),
        //     total_cold1_stake_before - 1
        // ); (DEPRECATED)
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hot1, netuid),
            total_hot1_stake_before - 1
        );
        assert_eq!(TotalStake::<Test>::get(), total_stake_before - 2);
    });
}

//SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_add_stake_below_minimum_threshold --exact --nocapture
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            amount_below
        ));

        // Nomination stake cannot stake below min threshold.
        assert_noop!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
                hotkey1,
                netuid,
                amount_below
            ),
            pallet_subtensor::Error::<Test>::NomStakeBelowMinimumThreshold
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_remove_stake_below_minimum_threshold --exact --nocapture
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

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, initial_balance);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, initial_balance);
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
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid
            ),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid
            ),
            0
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            initial_stake
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid
            ),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid
            ),
            initial_stake
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            netuid,
            initial_stake
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid
            ),
            initial_stake
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid
            ),
            initial_stake
        );

        // Coldkey staking on its own hotkey can unstake below min threshold.
        let stake_amount_to_remove = 51_000;
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            stake_amount_to_remove
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid
            ),
            initial_stake
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid
            ),
            initial_stake - stake_amount_to_remove
        );

        // Nomination stake cannot unstake below min threshold,
        // without unstaking all and removing the nomination.
        let total_hotkey_stake_before =
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey1, netuid);
        let bal_before = Balances::free_balance(coldkey2);
        let staked_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1, &coldkey2, netuid,
        );
        let total_network_stake_before = SubtensorModule::get_total_stake();
        let total_issuance_before = SubtensorModule::get_total_issuance();
        // check the premise of the test is correct
        assert!(initial_stake - stake_amount_to_remove < minimum_threshold);
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid
            ),
            initial_stake
        );
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            netuid,
            stake_amount_to_remove
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid
            ),
            0
        );
        let stake_removed = staked_before; // All stake was removed
                                           // Has the full balance
        assert_eq!(Balances::free_balance(coldkey2), bal_before + stake_removed);

        // Stake map entry is removed
        assert!(Alpha::<Test>::try_get((hotkey1, coldkey2, netuid)).is_err(),);
        // Stake tracking is updated
        // assert!(TotalColdkeyAlpha::<Test>::try_get(coldkey2, netuid).is_err()); (DEPRECATED)
        assert_eq!(
            TotalHotkeyAlpha::<Test>::try_get(hotkey1, netuid).unwrap(),
            total_hotkey_stake_before - stake_removed // Stake was removed from hotkey1 tracker
        );
        assert_eq!(
            TotalStake::<Test>::try_get().unwrap(),
            total_network_stake_before - stake_removed
        );

        // Total issuance is the same
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            total_issuance_before // Nothing was issued
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_delegate_take_can_be_decreased --exact --nocapture

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
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 decreases take to 5%. This should fail as the minimum take is 9%
        assert_err!(
            SubtensorModule::do_decrease_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 20
            ),
            Error::<Test>::DelegateTakeTooLow
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_can_set_min_take_ok --exact --nocapture
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
            u16::MAX / 10
        ));

        // Coldkey / hotkey 0 decreases take to min
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_delegate_take_can_not_be_increased_with_decrease_take --exact --nocapture
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
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 tries to increase take to 12.5%
        assert_eq!(
            SubtensorModule::do_decrease_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 8
            ),
            Err(Error::<Test>::DelegateTakeTooLow.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_delegate_take_can_be_increased --exact --nocapture
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
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 decreases take to 12.5%
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            u16::MAX / 8
        ));
        assert_eq!(SubtensorModule::get_hotkey_take(&hotkey0), u16::MAX / 8);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_delegate_take_can_not_be_decreased_with_increase_take --exact --nocapture
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
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 tries to decrease take to 5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 20
            ),
            Err(Error::<Test>::DelegateTakeTooLow.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_delegate_take_can_be_increased_to_limit --exact --nocapture
// Verify delegate take can be increased up to InitialDefaultDelegateTake (18%)
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
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultDelegateTake+1
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            InitialDefaultDelegateTake::get()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            InitialDefaultDelegateTake::get()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_delegate_take_can_not_be_set_beyond_limit --exact --nocapture
// Verify delegate take can not be set above InitialDefaultDelegateTake
#[test]
fn test_delegate_take_can_not_be_set_beyond_limit() {
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
        let before = SubtensorModule::get_hotkey_take(&hotkey0);

        // Coldkey / hotkey 0 attempt to become delegates with take above maximum
        // (Disable this check if InitialDefaultDelegateTake is u16::MAX)
        if InitialDefaultDelegateTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_become_delegate(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    InitialDefaultDelegateTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(SubtensorModule::get_hotkey_take(&hotkey0), before);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_delegate_take_can_not_be_increased_beyond_limit --exact --nocapture
// Verify delegate take can not be increased above InitialDefaultDelegateTake (18%)
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

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultDelegateTake+1
        // (Disable this check if InitialDefaultDelegateTake is u16::MAX)
        if InitialDefaultDelegateTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_increase_take(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    InitialDefaultDelegateTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_rate_limits_enforced_on_increase_take --exact --nocapture
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

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 increases take to 12.5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                u16::MAX / 8
            ),
            Err(Error::<Test>::DelegateTxRateLimitExceeded.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Can increase after waiting
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            u16::MAX / 8
        ));
        assert_eq!(SubtensorModule::get_hotkey_take(&hotkey0), u16::MAX / 8);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_get_total_delegated_stake_after_unstaking --exact --nocapture
#[test]
fn test_get_total_delegated_stake_after_unstaking() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let initial_stake = 2000;
        let unstake_amount = 500;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add balance to delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, initial_stake);

        // Delegate stake
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            initial_stake
        ));

        // Check initial delegated stake
        assert_eq!(
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey),
            initial_stake - existential_deposit,
            "Initial delegated stake is incorrect"
        );

        // Unstake part of the delegation
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            unstake_amount
        ));

        // Calculate the expected delegated stake
        let expected_delegated_stake = initial_stake - unstake_amount - existential_deposit;

        // Debug prints
        println!("Initial stake: {}", initial_stake);
        println!("Unstake amount: {}", unstake_amount);
        println!("Existential deposit: {}", existential_deposit);
        println!("Expected delegated stake: {}", expected_delegated_stake);
        println!(
            "Actual delegated stake: {}",
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey)
        );

        // Check the total delegated stake after unstaking
        assert_eq!(
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey),
            expected_delegated_stake,
            "Delegated stake mismatch after unstaking"
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_get_total_delegated_stake_no_delegations --exact --nocapture

#[test]
fn test_get_total_delegated_stake_no_delegations() {
    new_test_ext(1).execute_with(|| {
        let delegate = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = 1u16;

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate, coldkey, 0);

        // Make the delegate a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(coldkey),
            delegate
        ));

        // Check that there's no delegated stake
        assert_eq!(SubtensorModule::get_total_delegated_stake(&delegate), 0);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_get_total_delegated_stake_single_delegator --exact --nocapture
#[test]
fn test_get_total_delegated_stake_single_delegator() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let stake_amount = 999;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add stake from delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, stake_amount);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            stake_amount
        ));

        // Debug prints
        println!("Delegate coldkey: {:?}", delegate_coldkey);
        println!("Delegate hotkey: {:?}", delegate_hotkey);
        println!("Delegator: {:?}", delegator);
        println!("Stake amount: {}", stake_amount);
        println!("Existential deposit: {}", existential_deposit);
        println!("Total stake for hotkey: {}", SubtensorModule::get_stake_for_hotkey_on_subnet(&delegate_hotkey, netuid));
        println!("Delegated stake for coldkey: {}", SubtensorModule::get_total_delegated_stake(&delegate_coldkey));

        // Calculate expected delegated stake
        let expected_delegated_stake = stake_amount - existential_deposit;
        let actual_delegated_stake = SubtensorModule::get_total_delegated_stake(&delegate_coldkey);

        assert_eq!(
            actual_delegated_stake,
            expected_delegated_stake,
            "Total delegated stake should match the delegator's stake minus existential deposit. Expected: {}, Actual: {}",
            expected_delegated_stake,
            actual_delegated_stake
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_get_total_delegated_stake_multiple_delegators --exact --nocapture
#[test]
fn test_get_total_delegated_stake_multiple_delegators() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator1 = U256::from(3);
        let delegator2 = U256::from(4);
        let stake1 = 1000;
        let stake2 = 1999;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add stake from delegator1
        SubtensorModule::add_balance_to_coldkey_account(&delegator1, stake1);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator1),
            delegate_hotkey,
            netuid,
            stake1
        ));

        // Add stake from delegator2
        SubtensorModule::add_balance_to_coldkey_account(&delegator2, stake2);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator2),
            delegate_hotkey,
            netuid,
            stake2
        ));

        // Debug prints
        println!("Delegator1 stake: {}", stake1);
        println!("Delegator2 stake: {}", stake2);
        println!("Existential deposit: {}", existential_deposit);
        println!("Total stake for hotkey: {}", SubtensorModule::get_stake_for_hotkey_on_subnet(&delegate_hotkey, netuid));
        println!("Delegated stake for coldkey: {}", SubtensorModule::get_total_delegated_stake(&delegate_coldkey));

        // Calculate expected total delegated stake
        let expected_total_delegated = stake1 + stake2 - 2 * existential_deposit;
        let actual_total_delegated = SubtensorModule::get_total_delegated_stake(&delegate_coldkey);

        assert_eq!(
            actual_total_delegated,
            expected_total_delegated,
            "Total delegated stake should match the sum of delegators' stakes minus existential deposits. Expected: {}, Actual: {}",
            expected_total_delegated,
            actual_total_delegated
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=info cargo test --test staking -- test_get_total_delegated_stake_exclude_owner_stake --exact --nocapture
#[test]
fn test_get_total_delegated_stake_exclude_owner_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let owner_stake = 1000;
        let delegator_stake = 999;
        let existential_deposit = 1; // Account for the existential deposit

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Make the account a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Add owner stake
        SubtensorModule::add_balance_to_coldkey_account(&delegate_coldkey, owner_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey,
            netuid,
            owner_stake
        ));

        // Add delegator stake
        SubtensorModule::add_balance_to_coldkey_account(&delegator, delegator_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            delegator_stake
        ));

        // Debug prints
        println!("Owner stake: {}", owner_stake);
        println!("Delegator stake: {}", delegator_stake);
        println!("Existential deposit: {}", existential_deposit);
        println!(
            "Total stake for hotkey: {}",
            SubtensorModule::get_stake_for_hotkey_on_subnet(&delegate_hotkey, netuid)
        );
        println!(
            "Delegated stake for coldkey: {}",
            SubtensorModule::get_total_delegated_stake(&delegate_coldkey)
        );

        // Check the total delegated stake (should exclude owner's stake)
        let expected_delegated_stake = delegator_stake - existential_deposit;
        let actual_delegated_stake = SubtensorModule::get_total_delegated_stake(&delegate_coldkey);

        assert_eq!(
            actual_delegated_stake, expected_delegated_stake,
            "Delegated stake should exclude owner's stake. Expected: {}, Actual: {}",
            expected_delegated_stake, actual_delegated_stake
        );
    });
}
