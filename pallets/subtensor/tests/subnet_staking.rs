use frame_support::{assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::Error;
use sp_core::{H256, U256};

/***********************************************************
    subnet_staking::add_subnet_stake() tests
************************************************************/
#[test]
fn test_add_subnet_stake_ok_no_emission() 
{
    new_test_ext().execute_with(|| {
        let hotkey_account_id:  U256    = U256::from(533453);
        let coldkey_account_id: U256    = U256::from(55453);
        let netuid:             u16     = 1;
        let tempo:              u16     = 13;
        let start_nonce:        u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Give it some $$$ in his coldkey balance
        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, 10000 + 1);

        // Check we have zero staked before transfer
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_account_id),
            0
        );

        // Also total stake should be zero
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            1,
            10000
        ));

        // Check if stake has increased
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_account_id),
            10000
        );

        // Check if balance has  decreased
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 1);

        // Check if total stake has increased accordingly.
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 10000);
        assert_eq!(Subtensor::get_combined_subnet_stake_for_coldkey(&coldkey_account_id), 10000);
    });
}


#[test]
fn test_subnet_dividends_with_run_to_block() {
    new_test_ext().execute_with(|| {
        let neuron_src_hotkey_id:   U256 = U256::from(1);
        let neuron_dest_hotkey_id:  U256 = U256::from(2);
        let coldkey_account_id:     U256 = U256::from(667);
        let netuid:                 u16 = 1;
        let initial_stake:          u64 = 5000;

        //add network
        add_network(netuid, 13, 0);

        // Register neuron, this will set a self weight
        Subtensor::set_max_registrations_per_block(netuid, 3);
        Subtensor::set_max_allowed_uids(1, 5);

        register_ok_neuron(netuid, U256::from(0), coldkey_account_id, 2112321);
        register_ok_neuron(netuid, neuron_src_hotkey_id, coldkey_account_id, 192213123);
        register_ok_neuron(netuid, neuron_dest_hotkey_id, coldkey_account_id, 12323);

        // Add some stake to the hotkey account, so we can test for emission before the transfer takes place
        Subtensor::inc_subnet_total_stake_for_hotkey(1, &neuron_src_hotkey_id, initial_stake);

        // Check if the initial stake has arrived
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &neuron_src_hotkey_id),
            initial_stake
        );

        // Check if all three neurons are registered
        assert_eq!(Subtensor::get_subnetwork_n(netuid), 3);

        // Run a couple of blocks to check if emission works
        run_to_block(2);

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &neuron_src_hotkey_id),
            initial_stake
        );

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &neuron_dest_hotkey_id),
            0
        );
    });
}

#[test]
fn test_subnet_add_stake_err_signature()
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_account_id:  U256    = U256::from(654); // bogus
        let amount:             u64     = 20000; // Not used

        let result = Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            1,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_add_subnet_stake_network_doesnt_exist() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_account_id: U256    = U256::from(435445);
        let hotkey_account_id:  U256    = U256::from(54544);
        let amount:             u64     = 1337;

        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, 1800);

        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                1,
                amount
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
    });
}

#[test]
fn test_add_subnet_stake_not_registered_key_pair() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_account_id: U256    = U256::from(435445);
        let hotkey_account_id:  U256    = U256::from(54544);
        let amount:             u64     = 1337;
        let netuid:             u16     = 1;

        add_network(netuid, 1, 0);

        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, 1800);

        assert_eq!(
            Subtensor::add_subnet_stake(
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
fn test_add_subnet_stake_err_neuron_does_not_belong_to_coldkey() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_id:     U256    = U256::from(544);
        let hotkey_id:      U256    = U256::from(54544);
        let other_cold_key: U256    = U256::from(99498);
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        // Give it some $$$ in his coldkey balance
        Subtensor::add_balance_to_coldkey_account(&other_cold_key, 100000);

        // Perform the request which is signed by a different cold key
        let result = Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            netuid,
            1000
        );
        assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
    });
}


#[test]
fn test_add_subnet_stake_err_not_enough_belance() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_id:     U256    = U256::from(544);
        let hotkey_id:      U256    = U256::from(54544);
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Lets try to stake with 0 balance in cold key account
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_id), 0);
        let result = Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            1,
            60000,
        );

        assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
    });
}

#[test]
fn test_remove_subnet_stake_ok_no_emission() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_account_id: U256    = U256::from(4343);
        let hotkey_account_id:  U256    = U256::from(4968585);
        let amount:             u64     = 10000;
        let netuid:             u16     = 1;
        let tempo:              u16     = 13;
        let start_nonce:        u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Let's spin up a neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_account_id),
            0
        );
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_account_id, &hotkey_account_id, amount);

        // Do the magic
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        assert_eq!(
            Subtensor::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_account_id),
            0
        );
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);
    });
}


#[test]
fn test_remove_subnet_stake_amount_zero() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_account_id: U256    = U256::from(4343);
        let hotkey_account_id:  U256    = U256::from(4968585);
        let amount:             u64     = 10000;
        let netuid:             u16     = 1;
        let tempo:              u16     = 13;
        let start_nonce:        u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        // Let's spin up a neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_account_id, &hotkey_account_id),
            0
        );
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_account_id, &hotkey_account_id, amount);

        // Do the magic
        assert_noop!(
            Subtensor::remove_subnet_stake(
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
fn test_remove_subnet_stake_err_signature() 
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_account_id:  U256    = U256::from(4968585);
        let amount:             u64     = 10000; // Amount to be removed

        let result = Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::none(),
            hotkey_account_id,
            1,
            amount,
        );
        assert_eq!(result, DispatchError::BadOrigin.into());
    });
}

#[test]
fn test_remove_subnet_stake_err_hotkey_does_not_belong_to_coldkey() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_id:     U256    = U256::from(544);
        let hotkey_id:      U256    = U256::from(54544);
        let other_cold_key: U256    = U256::from(99498);
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Perform the request which is signed by a different cold key
        let result = Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(other_cold_key),
            hotkey_id,
            netuid,
            1000,
        );
        assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
    });
}


#[test]
fn test_remove_subnet_stake_no_enough_stake() 
{
    new_test_ext().execute_with(|| 
    {
        let coldkey_id:     U256    = U256::from(544);
        let hotkey_id:      U256    = U256::from(54544);
        let amount:         u64     = 10000;
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        assert_eq!(Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id), 0);

        let result = Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_id),
            hotkey_id,
            netuid,
            amount,
        );
        assert_eq!(result, Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
    });
}

#[test]
fn test_remove_subnet_stake_total_balance_no_change() 
{
    // When we remove stake, the total balance of the coldkey account should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext().execute_with(|| 
    {
        let hotkey_account_id:  U256    = U256::from(571337);
        let coldkey_account_id: U256    = U256::from(71337);
        let netuid:             u16     = 1;
        let tempo:              u16     = 13;
        let start_nonce:        u64     = 0;
        let amount:             u64     = 10000;

        //add network
        add_network(netuid, tempo, 0);

        // Register neuron
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, start_nonce);

        // Some basic assertions
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_account_id, &hotkey_account_id),
            0
        );
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);

        // Give the neuron some stake to remove
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_account_id, &hotkey_account_id, amount);

        // Do the magic
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        assert_eq!(
            Subtensor::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_account_id, &hotkey_account_id),
            0
        );
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);

        // Check total balance is equal to the added stake. Even after remove stake (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, amount);
    });
}

#[test]
fn test_add_stake_to_hotkey_account_ok() 
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_id:      U256    = U256::from(5445);
        let coldkey_id:     U256    = U256::from(5443433);
        let amount:         u64     = 10000;
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // There is not stake in the system at first, so result should be 0;
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);

        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id, amount);

        // The stake that is now in the account, should equal the amount
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id),
            amount
        );

        // The total stake should have been increased by the amount -> 0 + amount = amount
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), amount);
    });
}

#[test]
fn test_remove_subnet_stake_from_hotkey_account() 
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_id:      U256    = U256::from(5445);
        let coldkey_id:     U256    = U256::from(5443433);
        let amount:         u64     = 10000;
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;

        //add network
        add_network(netuid, tempo, 0);

        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);

        // Add some stake that can be removed
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id, amount);

        // Prelimiary checks
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), amount);
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id),
            amount
        );

        // Remove stake
        Subtensor::dec_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id, amount);

        // The stake on the hotkey account should be 0
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id), 0);

        // The total amount of stake should be 0
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 0);
    });
}


#[test]
fn test_remove_subnet_stake_from_hotkey_account_registered_in_various_networks() 
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_id:      U256    = U256::from(5445);
        let coldkey_id:     U256    = U256::from(5443433);
        let amount:         u64     = 10000;
        let netuid:         u16     = 1;
        let netuid_ex:      u16     = 2;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;
        //
        add_network(netuid, tempo, 0);
        add_network(netuid_ex, tempo, 0);
        //
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        register_ok_neuron(netuid_ex, hotkey_id, coldkey_id, 48141209);

        //let neuron_uid = Subtensor::get_uid_for_net_and_hotkey(netuid, &hotkey_id);
        let neuron_uid;
        match Subtensor::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => neuron_uid = k,
            Err(e) => panic!("Error: {:?}", e),
        }
        //let neuron_uid_ex = Subtensor::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id);
        let neuron_uid_ex;
        match Subtensor::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id) {
            Ok(k) => neuron_uid_ex = k,
            Err(e) => panic!("Error: {:?}", e),
        }
        //Add some stake that can be removed
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id, amount);

        assert_eq!(
            Subtensor::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            amount
        );
        assert_eq!(
            Subtensor::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            0
        );

        // Remove stake
        Subtensor::dec_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id, amount);
        //
        assert_eq!(
            Subtensor::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            0
        );
        assert_eq!(
            Subtensor::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            0
        );
    });
}

#[test]
fn test_increase_subnet_total_stake_ok() 
{
    new_test_ext().execute_with(|| 
    {
        add_network(1, 1, 0);

        let increment = 10000;
        assert_eq!(Subtensor::get_subnet_total_stake(1), 0);
        Subtensor::inc_subnet_total_stake(1, increment);
        assert_eq!(Subtensor::get_subnet_total_stake(1), increment);
    });
}

#[test]
fn test_decrease_subnet_total_stake_ok() 
{
    new_test_ext().execute_with(|| 
    {
        add_network(1, 1, 0);

        let initial_total_stake = 10000;
        let decrement = 5000;

        Subtensor::inc_subnet_total_stake(1, initial_total_stake);
        Subtensor::dec_subnet_total_stake(1, decrement);

        // The total stake remaining should be the difference between the initial stake and the decrement
        assert_eq!(
            Subtensor::get_subnet_total_stake(1),
            initial_total_stake - decrement
        );
    });
}

#[test]
fn test_has_enough_subnet_stake_yes() 
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_id:      U256    = U256::from(4334);
        let coldkey_id:     U256    = U256::from(87989);
        let intial_amount:  u64     = 10000;
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id, intial_amount);
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id),
            10000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey_id, &hotkey_id),
            10000
        );
        assert_eq!(
            Subtensor::does_coldkey_hotkey_have_enough_subnet_stake(netuid, &coldkey_id, &hotkey_id, 5000),
            true
        );
    });
}

#[test]
fn test_has_enough_subnet_stake_no() 
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_id:      U256    = U256::from(4334);
        let coldkey_id:     U256    = U256::from(87989);
        let amount:         u64     = 10000;
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        assert_eq!(
            Subtensor::does_coldkey_hotkey_have_enough_subnet_stake(netuid, &coldkey_id, &hotkey_id, amount),
            false
        );
    });
}

#[test]
fn test_subnet_non_existent_account() {
    new_test_ext().execute_with(|| 
    {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);

        Subtensor::inc_subnet_stake_for_coldkey_hotkey(
            netuid,
            &U256::from(0),
            &(U256::from(0)),
            10,
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &U256::from(0), &U256::from(0)),
            10
        );
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_coldkey(netuid, &(U256::from(0))),
            10
        );
    });
}

#[test]
fn test_delegate_stake_division_by_zero_check() 
{
    new_test_ext().execute_with(|| 
    {
        let netuid:     u16     = 1;
        let tempo:      u16     = 1;
        let hotkey:     U256    = U256::from(1);
        let coldkey:    U256    = U256::from(3);
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 2341312);
        assert_ok!(Subtensor::become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey, 0, 1000);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_subnet_full_with_delegating() {
    new_test_ext().execute_with(|| {
        let netuid:     u16     = 1;
        // Make two accounts.
        let hotkey0:    U256    = U256::from(1);
        let hotkey1:    U256    = U256::from(2);
        let coldkey0:   U256    = U256::from(3);
        let coldkey1:   U256    = U256::from(4);

        add_network(netuid, 0, 0);

        Subtensor::set_max_registrations_per_block(netuid, 4);
        Subtensor::set_target_registrations_per_interval(netuid, 4);
        Subtensor::set_max_allowed_uids(netuid, 4); // Allow all 4 to be registered at once

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        Subtensor::add_balance_to_coldkey_account(&coldkey0, 60000);
        Subtensor::add_balance_to_coldkey_account(&coldkey1, 60000);

        // We have enough, but the keys are not registered.
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::NotRegistered.into())
        );

        // Cant remove either.
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::NotRegistered.into())
        );

        // Neither key can become a delegate either because we are not registered.
        assert_eq!(
            Subtensor::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::NotRegistered.into())
        );
        assert_eq!(
            Subtensor::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                100
            ),
            Err(Error::<Test>::NotRegistered.into())
        );

        // Register the 2 neurons to a new network.
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            Subtensor::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            Subtensor::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(Subtensor::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(Subtensor::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We try to delegate stake but niether are allowing delegation.
        assert!(!Subtensor::hotkey_is_delegate(&hotkey0));
        assert!(!Subtensor::hotkey_is_delegate(&hotkey1));
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                100
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                100
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // We stake and all is ok.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            0
        );
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            100
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 100);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 100);
        //assert_eq!( Subtensor::get_total_stake_for_coldkey( &coldkey0 ), 100 );
        //assert_eq!( Subtensor::get_total_stake_for_coldkey( &coldkey1 ), 100 );
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 200);

        // Cant remove these funds because we are not delegating.
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Emit inflation through non delegates.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 0, 100);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 0, 100);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 200);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 200);

        // Try allowing the keys to become delegates, fails because of incorrect coldkeys.
        // Set take to be 0.
        assert_eq!(
            Subtensor::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                0
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            Subtensor::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                0
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Become delegates all is ok.
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            10
        ));
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            10
        ));
        assert!(Subtensor::hotkey_is_delegate(&hotkey0));
        assert!(Subtensor::hotkey_is_delegate(&hotkey1));

        // Cant become a delegate twice.
        assert_eq!(
            Subtensor::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                1000
            ),
            Err(Error::<Test>::AlreadyDelegate.into())
        );
        assert_eq!(
            Subtensor::do_become_delegate(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                1000
            ),
            Err(Error::<Test>::AlreadyDelegate.into())
        );

        // This add stake works for delegates.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            200
        );
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            200
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            300
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            200
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            300
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            200
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 500);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 400);
        //assert_eq!( Subtensor::get_total_stake_for_coldkey( &coldkey0 ), 400 );
        //assert_eq!( Subtensor::get_total_stake_for_coldkey( &coldkey1 ), 500 );
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 900);

        // Lets emit inflation through the hot and coldkeys.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 0, 1000);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 0, 1000);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            601
        ); // 200 + 1000 x ( 200 / 500 ) = 200 + 400 = 600 ~= 601
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            700
        ); // 200 + 1000 x ( 200 / 400 ) = 200 + 500 = 700
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            899
        ); // 300 + 1000 x ( 300 / 500 ) = 300 + 600 = 900 ~= 899
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            700
        ); // 200 + 1000 x ( 200 / 400 ) = 300 + 600 = 700
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 2900); // 600 + 700 + 900 + 700 = 2900

        // // Try unstaking too much.
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey1,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey0,
                netuid,
                100000
            ),
            Err(Error::<Test>::NotEnoughStaketoWithdraw.into())
        );

        // unstaking is ok.
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            100
        ));
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            100
        ));

        // All the amounts have been decreased.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            501
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            600
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            799
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            600
        );

        // Lets register and stake a new key.
        let hotkey2 = U256::from(5);
        let coldkey2 = U256::from(6);
        register_ok_neuron(netuid, hotkey2, coldkey2, 248_123);
        assert!(Subtensor::is_hotkey_registered_on_any_network(
            &hotkey0
        ));
        assert!(Subtensor::is_hotkey_registered_on_any_network(
            &hotkey1
        ));

        Subtensor::add_balance_to_coldkey_account(&coldkey2, 60_000);
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            1000
        ));
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey2),
            900
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        // Lets make this new key a delegate with a 50% take.
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            u16::MAX / 2
        ));

        // Add nominate some stake.
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            netuid,
            1_000
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            netuid,
            1_000
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey2),
            1_000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey2),
            1_000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey2),
            1_000
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey2), 3_000);
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 5_500);

        // Lets emit inflation through this new key with distributed ownership.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey2, 0, 1000);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey2),
            1_668
        ); // 1000 + 500 + 500 * (1000/3000) = 1500 + 166.6666666667 = 1,668
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey2),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey2),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 6_500); // before + 1_000 = 5_500 + 1_000 = 6_500

        step_block(1);

        // Lets register and stake a new key.
        let hotkey3 = U256::from(7);
        let coldkey3 = U256::from(8);
        register_ok_neuron(netuid, hotkey3, coldkey3, 4124124);
        Subtensor::add_balance_to_coldkey_account(&coldkey3, 60000);
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3,
            netuid,
            1000
        ));

        step_block(3);

        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey3),
            hotkey3,
            u16::MAX
        )); // Full take.
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey3,
            netuid,
            1000
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey3,
            netuid,
            1000
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey3,
            netuid,
            1000
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey3),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey3),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey3),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey3, &hotkey3),
            1000
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey3), 4000);
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 10_500);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey3, 0, 1000);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey3),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey3),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey3),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey3, &hotkey3),
            2000
        );
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 11_500); // before + 1_000 = 10_500 + 1_000 = 11_500
    });
}

// Verify delegates with servers get the full server inflation.
#[test]
fn test_full_with_delegating_some_servers() 
{
    new_test_ext().execute_with(|| 
    {
        let netuid:     u16 = 1;
        add_network(netuid, 1, 0);
        // Make two accounts.
        let hotkey0:    U256 = U256::from(1);
        let hotkey1:    U256 = U256::from(2);

        let coldkey0:   U256 = U256::from(3);
        let coldkey1:   U256 = U256::from(4);

        Subtensor::set_max_registrations_per_block(netuid, 4);
        Subtensor::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        Subtensor::add_balance_to_coldkey_account(&coldkey0, 60000);
        Subtensor::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        let netuid = 1;
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            Subtensor::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            Subtensor::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(Subtensor::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(Subtensor::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We stake and all is ok.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            0
        );
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            100
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 100);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 100);
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 200);

        // Emit inflation through non delegates.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 0, 100);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 0, 100);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 200);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 200);

        // Become delegates all is ok.
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            10
        ));
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            10
        ));
        assert!(Subtensor::hotkey_is_delegate(&hotkey0));
        assert!(Subtensor::hotkey_is_delegate(&hotkey1));

        // This add stake works for delegates.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            200
        );
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            200
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            300
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            200
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            200
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            300
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            200
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 500);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 400);
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 900);

        // Lets emit inflation through the hot and coldkeys.
        // fist emission arg is for a server. This should only go to the owner of the hotkey.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 200, 1_000); // 1_200 total emission.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 123, 2_000); // 2_123 total emission.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            801
        ); // 200 + (200 + 1000 x ( 200 / 500 )) = 200 + (200 + 400) = 800 ~= 801
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            899
        ); // 300 + 1000 x ( 300 / 500 ) = 300 + 600 = 900 ~= 899
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 1_700); // initial + server emission + validator emission = 799 + 899 = 1_698

        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            1_200
        ); // 200 + (0 + 2000 x ( 200 / 400 )) = 200 + (1000) = 1_200
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            1_323
        ); // 200 + (123 + 2000 x ( 200 / 400 )) = 200 + (1_200) = 1_323
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 2_523); // 400 + 2_123
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 4_223); // 1_700 + 2_523 = 4_223

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time only server emission. This should go to the owner of the hotkey.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 350, 0);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 150, 0);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            1_151
        ); // + 350 = 1_151
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            1_200
        ); // No change.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            899
        ); // No change.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            1_473
        ); // 1_323 + 150 = 1_473
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 4_723); // 4_223 + 500 = 4_823

        // Lets register and stake a new key.
        let hotkey2 = U256::from(5);
        let coldkey2 = U256::from(6);
        register_ok_neuron(netuid, hotkey2, coldkey2, 248123);
        Subtensor::add_balance_to_coldkey_account(&coldkey2, 60_000);
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            1_000
        ));
        assert_ok!(Subtensor::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey2),
            900
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );
        assert_eq!(
            Subtensor::remove_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey2,
                netuid,
                10
            ),
            Err(Error::<Test>::NonAssociatedColdKey.into())
        );

        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 5_623); // 4_723 + 900 = 5_623

        // Lets make this new key a delegate with a 50% take.
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            u16::MAX / 2
        ));

        // Add nominate some stake.
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey2,
            netuid,
            1000
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey2,
            netuid,
            1000
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey2,
            netuid,
            100
        ));
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey2),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey2),
            1000
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey2),
            1000
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey2), 3_000);
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 7_723); // 5_623 + (1_000 + 1_000 + 100) = 7_723

        // Lets emit inflation through this new key with distributed ownership.
        // We will emit 100 server emission, which should go in-full to the owner of the hotkey.
        // We will emit 1000 validator emission, which should be distributed in-part to the nominators.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey2, 100, 1000);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey2),
            1_768
        ); // 1000 + 100 + 500 + 500 * (1000/3000) = 100 + 1500 + 166.6666666667 ~= 1,768.6666666667
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey2),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey2),
            1_166
        ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 8_823); // 7_723 + 1_100 = 8_823

        // Lets emit MORE inflation through this new key with distributed ownership.
        // This time we do ONLY server emission
        // We will emit 123 server emission, which should go in-full to the owner of the hotkey.
        // We will emit *0* validator emission.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey2, 123, 0);
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2, &hotkey2),
            1_891
        ); // 1_768 + 123 = 1_891
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey2),
            1_166
        ); // No change.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey2),
            1_166
        ); // No change.
        assert_eq!(Subtensor::get_subnet_total_stake(netuid), 8_946); // 8_823 + 123 = 8_946
    });
}

#[test]
fn test_subnet_full_block_emission_occurs() {
    new_test_ext().execute_with(|| {
        let netuid:     u16     = 1;
        add_network(netuid, 0, 0);

        // Make two accounts.
        let hotkey0:    U256    = U256::from(1);
        let hotkey1:    U256    = U256::from(2);
        let coldkey0:   U256    = U256::from(3);
        let coldkey1:   U256    = U256::from(4);

        Subtensor::set_max_registrations_per_block(netuid, 4);
        Subtensor::set_max_allowed_uids(netuid, 10); // Allow at least 10 to be registered at once, so no unstaking occurs

        // Neither key can add stake because they dont have fundss.
        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
                hotkey0,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        assert_eq!(
            Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
                hotkey1,
                netuid,
                60000
            ),
            Err(Error::<Test>::NotEnoughBalanceToStake.into())
        );

        // Add balances.
        Subtensor::add_balance_to_coldkey_account(&coldkey0, 60000);
        Subtensor::add_balance_to_coldkey_account(&coldkey1, 60000);

        // Register the 2 neurons to a new network.
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);
        register_ok_neuron(netuid, hotkey1, coldkey1, 987907);
        assert_eq!(
            Subtensor::get_owning_coldkey_for_hotkey(&hotkey0),
            coldkey0
        );
        assert_eq!(
            Subtensor::get_owning_coldkey_for_hotkey(&hotkey1),
            coldkey1
        );
        assert!(Subtensor::coldkey_owns_hotkey(&coldkey0, &hotkey0));
        assert!(Subtensor::coldkey_owns_hotkey(&coldkey1, &hotkey1));

        // We stake and all is ok.
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            0
        );

        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            100
        ));

        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid,
            100
        ));

        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey0),
            100
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0, &hotkey1),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey0),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1, &hotkey1),
            100
        );
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0), 100);
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1), 100);
        
        //assert_eq!(Subtensor::get_total_stake(), 200);

        // Emit inflation through non delegates.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 0, 111);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 0, 234);
        // Verify the full emission occurs.

        let total_stake = Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0)
                         + Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1);

        assert_eq!(total_stake, 200 + 111 + 234); // 200 + 111 + 234 = 545

        // Become delegates all is ok.
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            10
        ));
        
        assert_ok!(Subtensor::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            10
        ));

        assert!(Subtensor::hotkey_is_delegate(&hotkey0));
        assert!(Subtensor::hotkey_is_delegate(&hotkey1));

        // Add some delegate stake
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid,
            200
        ));
        assert_ok!(Subtensor::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid,
            300
        ));

        let total_stake = Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0)
                         + Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1);
        assert_eq!(total_stake, 545 + 500); // 545 + 500 = 1045

        // Lets emit inflation with delegatees, with both validator and server emission
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 200, 1_000); // 1_200 total emission.
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 123, 2_000); // 2_123 total emission.
        
        let total_stake = Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0)
                         + Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1);
        assert_eq!(total_stake, 1045 + 1_200 + 2_123); // before + 1200 + 2123 = 4368

        // Lets emit MORE inflation through the hot and coldkeys.
        // This time JUSt server emission
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 350, 0);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 150, 0);

        let total_stake = Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0)
                         + Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1);
        assert_eq!(total_stake, 4_368 + 350 + 150); // before + 350 + 150 = 4_868

        // Lastly, do only validator emission

        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey0, 0, 12_948);
        Subtensor::emit_inflation_through_hotkey_account(netuid, &hotkey1, 0, 1_874);

        let total_stake = Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey0)
                         + Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey1);
        assert_eq!(total_stake, 4_868 + 12_948 + 1_874); // before + 12_948 + 1_874 = 19_690
    });
}

#[test]
fn test_unstake_all_coldkeys_from_hotkey_account() 
{
    new_test_ext().execute_with(|| 
    {
        let hotkey_id:      U256    = U256::from(123570);
        let coldkey0_id:    U256    = U256::from(123560);

        let coldkey1_id:    U256    = U256::from(123561);
        let coldkey2_id:    U256    = U256::from(123562);
        let coldkey3_id:    U256    = U256::from(123563);

        let amount:         u64     = 10000;
        let netuid:         u16     = 1;
        let tempo:          u16     = 13;
        let start_nonce:    u64     = 0;

        // Make subnet
        add_network(netuid, tempo, 0);
        // Register delegate
        register_ok_neuron(netuid, hotkey_id, coldkey0_id, start_nonce);

        match Subtensor::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(_k) => (),
            Err(e) => panic!("Error: {:?}", e),
        }

        //Add some stake that can be removed
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0_id, &hotkey_id, amount);
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(
            netuid,
            &coldkey1_id,
            &hotkey_id,
            amount + 2,
        );
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(
            netuid,
            &coldkey2_id,
            &hotkey_id,
            amount + 3,
        );
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(
            netuid,
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
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id),
            amount * 4 + (2 + 3 + 4)
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        Subtensor::remove_all_subnet_stake_for_hotkey(netuid, &hotkey_id);

        // Verify total stake is 0
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id), 0);

        // Vefify stake for all coldkeys is 0
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0_id, &hotkey_id),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey1_id, &hotkey_id),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey2_id, &hotkey_id),
            0
        );
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey3_id, &hotkey_id),
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
    new_test_ext().execute_with(|| {
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

        match Subtensor::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(_) => (),
            Err(e) => panic!("Error: {:?}", e),
        }

        //Add some stake that can be removed
        Subtensor::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0_id, &hotkey_id, amount);

        // Verify free balance is 0 for coldkey
        assert_eq!(Balances::free_balance(coldkey0_id), 0);

        // Verify total stake is correct
        assert_eq!(
            Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id),
            amount
        );

        // Run unstake_all_coldkeys_from_hotkey_account
        Subtensor::remove_all_subnet_stake_for_hotkey(netuid, &hotkey_id);

        // Verify total stake is 0
        assert_eq!(Subtensor::get_subnet_total_stake_for_hotkey(netuid, &hotkey_id), 0);

        // Vefify stake for single coldkey is 0
        assert_eq!(
            Subtensor::get_subnet_stake_for_coldkey_hotkey(netuid, &coldkey0_id, &hotkey_id),
            0
        );

        // Verify free balance is correct for single coldkey
        assert_eq!(Balances::free_balance(coldkey0_id), amount);
    });
}