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