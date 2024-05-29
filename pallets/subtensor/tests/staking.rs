use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::*;
use sp_core::{H256, U256};

/***********************************************************
    staking::add_stake() tests
************************************************************/

#[test]
#[cfg(not(tarpaulin))]
fn test_add_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_staked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            amount_staked,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(124_000_000, 0),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}
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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
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
        SubtensorModule::increase_stake_on_hotkey_account(&neuron_src_hotkey_id, initial_stake);

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
fn test_add_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(654); // bogus
        let amount = 20000; // Not used

        let result = SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_add_stake_not_registered_key_pair() {
    new_test_ext(1).execute_with(|| {
        let coldkey_account_id = U256::from(435445);
        let hotkey_account_id = U256::from(54544);
        let amount = 1337;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1800);
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                amount
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
    });
}

#[test]
fn test_add_stake_err_neuron_does_not_belong_to_coldkey() {
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
        let result = SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            1000,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
    });
}

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
            60000,
        );

        assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
    });
}

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
        let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));

        let current_stakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert!(current_stakes <= max_stakes);
    });
}

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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, 2);

        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
        ));

        let current_unstakes = SubtensorModule::get_stakes_this_interval_for_coldkey_hotkey(
            &coldkey_account_id,
            &hotkey_account_id,
        );
        assert!(current_unstakes <= max_unstakes);
    });
}

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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, 2);
        assert_err!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
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

#[test]
#[cfg(not(tarpaulin))]
fn test_remove_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_unstaked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
            hotkey,
            amount_unstaked,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                weight: frame_support::weights::Weight::from_parts(111_000_000, 0)
                    .add_proof_size(43991),
                class: DispatchClass::Normal,
                pays_fee: Pays::No
            }
        );
    });
}

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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        // Do the magic
        assert_noop!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                0
            ),
            Error::<Test>::StakeToWithdrawIsZero
        );
    });
}

#[test]
fn test_remove_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000; // Amount to be removed

        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_remove_stake_err_hotkey_does_not_belong_to_coldkey() {
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
        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            1000,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
    });
}

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

        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        let result = SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            amount,
        );
        assert_eq!(result, Err(Error::<Test>::NotEnoughStakeToWithdraw.into()));
    });
}

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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
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
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);
        let inital_total_issuance = Balances::total_issuance();
        assert_eq!(inital_total_issuance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        let total_issuance_after_stake = Balances::total_issuance();

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
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
// 	staking::add_stake_to_hotkey_account() tests
// ************************************************************/
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

        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

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
    staking::remove_stake_from_hotkey_account() tests
************************************************************/
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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

        // Prelimiary checks
        assert_eq!(SubtensorModule::get_total_stake(), amount);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_stake_on_hotkey_account(&hotkey_id, amount);

        // The stake on the hotkey account should be 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        // The total amount of stake should be 0
        assert_eq!(SubtensorModule::get_total_stake(), 0);
    });
}

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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_stake_on_hotkey_account(&hotkey_id, amount);
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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, intial_amount);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            10000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id),
            10000
        );
        assert!(SubtensorModule::has_enough_stake(
            &coldkey_id,
            &hotkey_id,
            5000
        ));
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
        SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, intial_amount);
        assert!(!SubtensorModule::has_enough_stake(
            &coldkey_id,
            &hotkey_id,
            5000
        ));
    });
}

#[test]
fn test_non_existent_account() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &U256::from(0),
            &(U256::from(0)),
            10,
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&U256::from(0), &U256::from(0)),
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
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey, 0, 1000);
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
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // We have enough, but the keys are not registered.
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Cant remove either.
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                10
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );

        // Neither key can become a delegate either because we are not registered.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyAccountNotExists.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
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
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                100
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                100
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // We stake and all is ok.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            100
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 100 );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 100 );
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Cant remove these funds because we are not delegating.
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 200);

        // Try allowing the keys to become delegates, fails because of incorrect coldkeys.
        // Set take to be 0.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                0
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                0
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            SubtensorModule::get_min_take()
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // Cant become a delegate twice.
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                SubtensorModule::get_min_take()
            ),
            Err(Error::<Test>::HotKeyAlreadyDelegate.into())
        );
        assert_eq!(
            SubtensorModule::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                u16::MAX / 10
            ),
            Err(Error::<Test>::HotKeyAlreadyDelegate.into())
        );

        // This add stake works for delegates.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            200
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            300
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            300
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 500);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 400);
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 400 );
        //assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 500 );
        assert_eq!(SubtensorModule::get_total_stake(), 900);

        // Lets emit inflation through the hot and coldkeys.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 1000);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 1000);

        // validator_take = take * validator_emission = 10% * 1000 = 100
        // old_stake + (validator_emission - validator_take) * stake_for_coldkey_and_hotkey / total_stake_for_hotkey + validator_take
        // =
        // 200 + 900 * 200 / 500 + 100 = 660
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            654
        );
        // validator_take = take * validator_emission = 9% * 1000 = 90
        // old_stake + (validator_emission - validator_take) * stake_for_coldkey_and_hotkey / total_stake_for_hotkey
        // =
        // 200 + (1000 - 90) * 200 / 400 = 655 ~ 654
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            655
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            846
        ); // 300 + 910 x ( 300 / 500 ) = 300 + 546 = 846
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            745
        ); // 200 + 1090 x ( 200 / 400 )  = 300 + 545 = 745
        assert_eq!(SubtensorModule::get_total_stake(), 2900); // 600 + 700 + 900 + 750 = 2900

        // // Try unstaking too much.
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                100000
            ),
            Err(Error::<Test>::NotEnoughStakeToWithdraw.into())
        );

        // unstaking is ok.
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            100
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            100
        ));

        // All the amounts have been decreased.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            554
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            555
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            746
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            645
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            1000
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            900
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        // Lets make this new key a delegate with a 10% take.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            SubtensorModule::get_min_take()
        ));

        // Add nominate some stake.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            1_000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            1_000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 3_000);
        assert_eq!(SubtensorModule::get_total_stake(), 5_500);

        // Lets emit inflation through this new key with distributed ownership.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, 0, 1000);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_394
        ); // 1000 + 94 + 900 * (1000/3000) = 1400
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1300
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1300
        assert_eq!(SubtensorModule::get_total_stake(), 6_500); // before + 1_000 = 5_500 + 1_000 = 6_500

        step_block(1);

        // Lets register and stake a new key.
        let hotkey3 = U256::from(7);
        let coldkey3 = U256::from(8);
        register_ok_neuron(netuid, hotkey3, coldkey3, 4124124);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey3, 60000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3,
            1000
        ));

        step_block(3);

        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3,
            SubtensorModule::get_min_take()
        )); // Full take.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey3,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey3,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey3,
            1000
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey3),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey3),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey3),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey3, &hotkey3),
            1000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey3), 4000);
        assert_eq!(SubtensorModule::get_total_stake(), 10_500);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey3, 0, 1000);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey3),
            1227
        ); // 1000 + 90% * 1000 * 1000/4000 = 1225
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey3),
            1227
        ); // 1000 + 90% * 1000 * 1000/4000 = 1225
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey3),
            1227
        ); // 1000 + 90% * 1000 * 1000/4000 = 1225
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey3, &hotkey3),
            1319
        ); // 1000 + 25 * 3 + 1000 * 1000/4000 = 1325
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
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        let netuid = 1;
        add_network(netuid, 0, 0);
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
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            100
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 100);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 200);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 200);

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            SubtensorModule::get_min_take()
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // This add stake works for delegates.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            200
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            300
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            200
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            300
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            200
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 500);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 400);
        assert_eq!(SubtensorModule::get_total_stake(), 900);

        // Lets emit inflation through the hot and coldkeys.
        // fist emission arg is for a server. This should only go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 123, 2_000); // 2_123 total emission.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            854
        ); // 200 + (200 + 910 x ( 200 / 500 ))  = 200 + (200 + 400) + 60 = 854
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            846
        ); // 300 + 910 x ( 300 / 500 ) = 300 + 546 = 846
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 1_700); // initial + server emission + validator emission = 799 + 899 = 1_698

        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            1_110
        ); // 200 + (0 + 2000 x ( 200 / 400 )) - 100 = 200 + (1000) - 100= 1_110
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            1_413
        ); // 200 + (123 + 2000 x ( 200 / 400 )) + 100 = 200 + (1_200)+ 100 = 1_423
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 2_523); // 400 + 2_123
        assert_eq!(SubtensorModule::get_total_stake(), 4_223); // 2_100 + 2_123 = 4_223

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time only server emission. This should go to the owner of the hotkey.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 150, 0);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            1_204
        ); // + 350 + 54 = 1_204
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            1_110
        ); // No change.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            846
        ); // No change.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            1_563
        ); // 1_323 + 150 + 90 = 1_573
        assert_eq!(SubtensorModule::get_total_stake(), 4_723); // 4_223 + 500 = 4_823

        // Lets register and stake a new key.
        let hotkey2 = U256::from(5);
        let coldkey2 = U256::from(6);
        register_ok_neuron(netuid, hotkey2, coldkey2, 248123);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 60_000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            1_000
        ));
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            900
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );
        assert_eq!(
            SubtensorModule::remove_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                10
            ),
            Err(Error::<Test>::HotKeyNotDelegateAndSignerNotOwnHotKey.into())
        );

        assert_eq!(SubtensorModule::get_total_stake(), 5_623); // 4_723 + 900 = 5_623

        // Lets make this new key a delegate with a 9% take.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            SubtensorModule::get_min_take()
        ));

        // Add nominate some stake.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey2), 3_000);
        assert_eq!(SubtensorModule::get_total_stake(), 7_723); // 5_623 + (1_000 + 1_000 + 100) = 7_723

        // Lets emit inflation through this new key with distributed ownership.
        // We will emit 100 server emission, which should go in-full to the owner of the hotkey.
        // We will emit 1000 validator emission, which should be distributed in-part to the nominators.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, 100, 1000);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_494
        ); // 1000 + 100 + 94 + 900 * (1000/3000) = 1000 + 200 + 300 = 1_494
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1000 + 300 = 1_303
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_303
        ); // 1000 + 900 * (1000/3000) = 1000 + 300 = 1300
        assert_eq!(SubtensorModule::get_total_stake(), 8_823); // 7_723 + 1_100 = 8_823

        // Lets emit MORE inflation through this new key with distributed ownership.
        // This time we do ONLY server emission
        // We will emit 123 server emission, which should go in-full to the owner of the hotkey.
        // We will emit *0* validator emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey2, 123, 0);
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey2),
            1_617
        ); // 1_500 + 117 = 1_617
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey2),
            1_303
        ); // No change.
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey2),
            1_303
        ); // No change.
        assert_eq!(SubtensorModule::get_total_stake(), 8_946); // 8_823 + 123 = 8_946
    });
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
        SubtensorModule::set_max_registrations_per_block(netuid, 4);
        SubtensorModule::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs
        SubtensorModule::set_target_stakes_per_interval(10); // Increase max stakes per interval

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        let netuid = 1;
        add_network(netuid, 0, 0);
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
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            0
        );
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            100
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1),
            100
        );
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey0), 100);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey1), 100);
        assert_eq!(SubtensorModule::get_total_stake(), 200);

        // Emit inflation through non delegates.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 111);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 234);
        // Verify the full emission occurs.
        assert_eq!(SubtensorModule::get_total_stake(), 200 + 111 + 234); // 200 + 111 + 234 = 545

        // Become delegates all is ok.
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            SubtensorModule::get_min_take()
        ));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey0));
        assert!(SubtensorModule::hotkey_is_delegate(&hotkey1));

        // Add some delegate stake
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            200
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            300
        ));

        assert_eq!(SubtensorModule::get_total_stake(), 545 + 500); // 545 + 500 = 1045

        // Lets emit inflation with delegatees, with both validator and server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 200, 1_000); // 1_200 total emission.
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 123, 2_000); // 2_123 total emission.

        assert_eq!(SubtensorModule::get_total_stake(), 1045 + 1_200 + 2_123); // before + 1_200 + 2_123 = 4_368

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time JUSt server emission
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 350, 0);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 150, 0);

        assert_eq!(SubtensorModule::get_total_stake(), 4_368 + 350 + 150); // before + 350 + 150 = 4_868

        // Lastly, do only validator emission

        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, 0, 12_948);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, 0, 1_874);

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
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0_id, &hotkey_id, amount);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey1_id,
            &hotkey_id,
            amount + 2,
        );
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey2_id,
            &hotkey_id,
            amount + 3,
        );
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey3_id,
            &hotkey_id,
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
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0_id, &hotkey_id),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey1_id, &hotkey_id),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2_id, &hotkey_id),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey3_id, &hotkey_id),
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
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey0_id, &hotkey_id, amount);

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
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey0_id, &hotkey_id),
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
            SubtensorModule::get_min_take()
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot1), cold1);

        // Register hot2.
        register_ok_neuron(netuid, hot2, cold2, 0);
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot2), cold2);

        // Add stake cold1 --> hot1 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot1,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 4);

        // Add stake cold2 --> hot1 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot1,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 4);

        // Add stake cold1 --> hot2 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            hot2,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
            1
        );
        assert_eq!(Balances::free_balance(cold1), 8);

        // Add stake cold2 --> hot2 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, 5);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            hot2,
            1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
            1
        );
        assert_eq!(Balances::free_balance(cold2), 8);

        // Run clear all small nominations when min stake is zero (noop)
        SubtensorModule::set_nominator_min_required_stake(0);
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
            1
        );

        // Set min nomination to 10
        let total_cold1_stake_before = TotalColdkeyStake::<Test>::get(cold1);
        let total_cold2_stake_before = TotalColdkeyStake::<Test>::get(cold2);
        let total_hot1_stake_before = TotalHotkeyStake::<Test>::get(hot1);
        let total_hot2_stake_before = TotalHotkeyStake::<Test>::get(hot2);
        let _ = Stake::<Test>::try_get(hot2, cold1).unwrap(); // ensure exists before
        let _ = Stake::<Test>::try_get(hot1, cold2).unwrap(); // ensure exists before
        let total_stake_before = TotalStake::<Test>::get();
        SubtensorModule::set_nominator_min_required_stake(10);

        // Run clear all small nominations (removes delegations under 10)
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot1),
            1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold1, &hot2),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot1),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&cold2, &hot2),
            1
        );

        // Balances have been added back into accounts.
        assert_eq!(Balances::free_balance(cold1), 9);
        assert_eq!(Balances::free_balance(cold2), 9);

        // Internal storage is updated
        assert_eq!(
            TotalColdkeyStake::<Test>::get(cold2),
            total_cold2_stake_before - 1
        );
        assert_eq!(
            TotalHotkeyStake::<Test>::get(hot2),
            total_hot2_stake_before - 1
        );
        Stake::<Test>::try_get(hot2, cold1).unwrap_err();
        Stake::<Test>::try_get(hot1, cold2).unwrap_err();
        assert_eq!(
            TotalColdkeyStake::<Test>::get(cold1),
            total_cold1_stake_before - 1
        );
        assert_eq!(
            TotalHotkeyStake::<Test>::get(hot1),
            total_hot1_stake_before - 1
        );
        Stake::<Test>::try_get(hot2, cold1).unwrap_err();
        assert_eq!(TotalStake::<Test>::get(), total_stake_before - 2);
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            amount_below
        ));

        // Nomination stake cannot stake below min threshold.
        assert_noop!(
            SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
                hotkey1,
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
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            initial_stake
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            initial_stake
        ));

        // Coldkey staking on its own hotkey can unstake below min threshold.
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            stake_amount_to_remove
        ));

        // Nomination stake cannot unstake below min threshold,
        // without unstaking all and removing the nomination.
        let total_hotkey_stake_before = SubtensorModule::get_total_stake_for_hotkey(&hotkey1);
        let bal_before = Balances::free_balance(coldkey2);
        let staked_before = SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1);
        let total_network_stake_before = SubtensorModule::get_total_stake();
        let total_issuance_before = SubtensorModule::get_total_issuance();
        // check the premise of the test is correct
        assert!(initial_stake - stake_amount_to_remove < minimum_threshold);
        assert_ok!(SubtensorModule::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            stake_amount_to_remove
        ));

        // Has no stake now
        assert_eq!(
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1),
            0
        );
        let stake_removed = staked_before; // All stake was removed
                                           // Has the full balance
        assert_eq!(Balances::free_balance(coldkey2), bal_before + stake_removed);

        // Stake map entry is removed
        assert!(Stake::<Test>::try_get(hotkey1, coldkey2).is_err(),);
        // Stake tracking is updated
        assert_eq!(
            TotalColdkeyStake::<Test>::try_get(coldkey2).unwrap(),
            0 // Did not have any stake before; Entry is NOT removed
        );
        assert_eq!(
            TotalHotkeyStake::<Test>::try_get(hotkey1).unwrap(),
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
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
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
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
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
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
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
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
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
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
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
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultTake+1
        assert_ok!(SubtensorModule::do_increase_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            InitialDefaultTake::get()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            InitialDefaultTake::get()
        );
    });
}

// Verify delegate take can not be set above InitialDefaultTake
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
        // (Disable this check if InitialDefaultTake is u16::MAX)
        if InitialDefaultTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_become_delegate(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    InitialDefaultTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(SubtensorModule::get_hotkey_take(&hotkey0), before);
    });
}

// Verify delegate take can not be increased above InitialDefaultTake (18%)
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
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
        );

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultTake+1
        // (Disable this check if InitialDefaultTake is u16::MAX)
        if InitialDefaultTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_increase_take(
                    <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                    hotkey0,
                    InitialDefaultTake::get() + 1
                ),
                Err(Error::<Test>::DelegateTakeTooHigh.into())
            );
        }
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
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

        // Coldkey / hotkey 0 become delegates with 9% take
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_take()
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
            SubtensorModule::get_min_take()
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
