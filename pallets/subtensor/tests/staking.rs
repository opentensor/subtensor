use frame_support::{assert_ok, traits::{Currency, ReservableCurrency, Imbalance}};
use frame_system::{Config};
mod mock;
use mock::*;
use frame_support::sp_runtime::DispatchError;
use pallet_subtensor::{Error};
use frame_support::dispatch::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};

/***********************************************************
	staking::add_stake() tests
************************************************************/

#[test]
fn test_add_stake_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let hotkey = 0;
		let amount_staked = 5000;
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake{hotkey, amount_staked});
		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: frame_support::weights::Weight::from_ref_time(65000000),
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}
#[test]
fn test_add_stake_ok_no_emission() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 533453;
		let coldkey_account_id = 55453;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Check we have zero staked before transfer
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id ), 0);

		// Also total stake should be zero
		assert_eq!(SubtensorModule::get_total_stake(), 0);

		// Transfer to hotkey account, and check if the result is ok
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, 10000));

		// Check if stake has increased
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 10000);

		// Check if balance has  decreased
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

		// Check if total stake has increased accordingly.
		assert_eq!(SubtensorModule::get_total_stake(), 10000);

	});
}

#[test]
fn test_dividends_with_run_to_block() {
	new_test_ext().execute_with(|| {
		let neuron_src_hotkey_id = 1;
		let neuron_dest_hotkey_id = 2;
		let coldkey_account_id = 667;
		let netuid: u16 = 1;

		let initial_stake:u64 = 5000;

		//add network
		add_network(netuid, 13, 0);

		// Register neuron, this will set a self weight
		SubtensorModule::set_max_registrations_per_block( netuid, 3 );
		SubtensorModule::set_max_allowed_uids(1, 5);
		
		register_ok_neuron( netuid, 0, coldkey_account_id, 2112321);
		register_ok_neuron(netuid, neuron_src_hotkey_id, coldkey_account_id, 192213123);
		register_ok_neuron(netuid, neuron_dest_hotkey_id, coldkey_account_id, 12323);

		// Add some stake to the hotkey account, so we can test for emission before the transfer takes place
		SubtensorModule::increase_stake_on_hotkey_account(&neuron_src_hotkey_id, initial_stake);

		// Check if the initial stake has arrived
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id), initial_stake );

		// Check if all three neurons are registered
		assert_eq!( SubtensorModule::get_subnetwork_n(netuid), 3 );

		// Run a couple of blocks to check if emission works
		run_to_block( 2 );

		// Check if the stake is equal to the inital stake + transfer
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id), initial_stake);

		// Check if the stake is equal to the inital stake + transfer
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&neuron_dest_hotkey_id), 0);
    });
}

#[test]
fn test_add_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 654; // bogus
		let amount = 20000 ; // Not used

		let result = SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::none(), hotkey_account_id, amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}

#[test]
fn test_add_stake_not_registered_key_pair() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 435445;
		let hotkey_account_id = 54544;
		let amount = 1337;
		SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1800);
		assert_eq!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, amount), Err(Error::<Test>::NotRegistered.into()));
	});
}

#[test]
fn test_add_stake_err_neuron_does_not_belong_to_coldkey() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;
		let other_cold_key = 99498;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce : u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		// Give it some $$$ in his coldkey balance
		SubtensorModule::add_balance_to_coldkey_account( &other_cold_key, 100000 );

		// Perform the request which is signed by a different cold key
		let result = SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(other_cold_key), hotkey_id, 1000);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
	});
}

#[test]
fn test_add_stake_err_not_enough_belance() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Lets try to stake with 0 balance in cold key account
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_id), 0);
		let result = SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_id), hotkey_id, 60000);

		assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
	});
}

#[test]
fn test_add_stake_total_balance_no_change() {
	// When we add stake, the total balance of the coldkey account should not change
	//    this is because the stake should be part of the coldkey account balance (reserved/locked)
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 551337;
		let coldkey_account_id = 51337;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Give it some $$$ in his coldkey balance
		let initial_balance = 10000;
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, initial_balance );

		// Check we have zero staked before transfer
		let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
		assert_eq!(initial_stake, 0);

		// Check total balance is equal to initial balance
		let initial_total_balance = Balances::total_balance(&coldkey_account_id);
		assert_eq!(initial_total_balance, initial_balance);

		// Also total stake should be zero
		assert_eq!(SubtensorModule::get_total_stake(), 0);

		// Stake to hotkey account, and check if the result is ok
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, 10000));

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
fn test_add_stake_total_issuance_no_change() {
	// When we add stake, the total issuance of the balances pallet should not change
	//    this is because the stake should be part of the coldkey account balance (reserved/locked)
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 561337;
		let coldkey_account_id = 61337;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Give it some $$$ in his coldkey balance
		let initial_balance = 10000;
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, initial_balance );

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
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, 10000));

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
fn test_add_stake_reserved_balance_matches() {
	// When we add stake, the reserved balance on the coldkey account should match the stake
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 561337;
		let coldkey_account_id = 61337;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Give it some $$$ in his coldkey balance
		let initial_balance = 10000;
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id, initial_balance );

		// Check we have zero staked before transfer
		let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
		assert_eq!(initial_stake, 0);

		// Check we have zero reserved balance before transfer
		let initial_reserved_balance = Balances::reserved_balance(&coldkey_account_id);
		assert_eq!(initial_reserved_balance, 0);

		// Stake to hotkey account, and check if the result is ok
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, 10000));

		// Check if stake has increased
		let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
		assert_eq!(new_stake, 10000);

		// Check if free balance has decreased
		let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
		assert_eq!(new_free_balance, 0);

		// Check if total stake has increased accordingly.
		assert_eq!(SubtensorModule::get_total_stake(), 10000);

		// Check if the reserved balance on the coldkey account matches the stake
		let reserved_balance = Balances::reserved_balance(&coldkey_account_id);
		assert_eq!(reserved_balance, 10000);
	});
}

// /***********************************************************
// 	staking::remove_stake() tests
// ************************************************************/

#[test]
fn test_remove_stake_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
        let hotkey = 0;
		let amount_unstaked = 5000;
		let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake{hotkey, amount_unstaked});
		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: frame_support::weights::Weight::from_ref_time(66000000),
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}

#[test]
fn test_remove_stake_ok_no_emission() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 4343;
		let hotkey_account_id = 4968585;
		let amount = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		// Let's spin up a neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Some basic assertions
		assert_eq!(SubtensorModule::get_total_stake(), 0);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

		// Give the neuron some stake to remove
		SubtensorModule::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

		// Do the magic
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, amount));

		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_total_stake(), 0);
	});
}

#[test]
fn test_remove_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id : u64 = 4968585;
		let amount = 10000; // Amount to be removed

		let result = SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::none(), hotkey_account_id, amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}

#[test]
fn test_remove_stake_err_hotkey_does_not_belong_to_coldkey() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 544;
		let hotkey_id = 54544;
		let other_cold_key = 99498;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Perform the request which is signed by a different cold key
		let result = SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(other_cold_key), hotkey_id, 1000);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
	});
}

#[test]
fn test_remove_stake_no_enough_stake() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 544;
		let hotkey_id = 54544;
		let amount = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

		let result = SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_id), hotkey_id, amount);
		assert_eq!(result, Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
	});
}

#[test]
fn test_remove_stake_total_balance_no_change() {
	// When we remove stake, the total balance of the coldkey account should not change
	//    this is because the stake should be part of the coldkey account balance (reserved/locked)
	//    then the removed stake just becomes free balance
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 571337;
		let coldkey_account_id = 71337;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		let amount = 10000;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Some basic assertions
		assert_eq!(SubtensorModule::get_total_stake(), 0);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
		let initial_total_balance = Balances::total_balance(&coldkey_account_id);
		assert_eq!(initial_total_balance, 0);

		// Issue the neuron some stake to remove
		SubtensorModule::issue_stake_to_coldkey_hotkey_account(&coldkey_account_id, &hotkey_account_id, amount);

		// Do the magic
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, amount));

		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_total_stake(), 0);

		// Check total balance is equal to the added stake. Even after remove stake (no fee, includes reserved/locked balance)
		let total_balance = Balances::total_balance(&coldkey_account_id);
		assert_eq!(total_balance, amount);
	});
}

#[test]
fn test_remove_stake_total_issuance_no_change() {
	// When we remove stake, the total issuance of the balances pallet should not change
	//    this is because the stake should be part of the coldkey account balance (reserved/locked)
	//    then the removed stake just becomes free balance
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 581337;
		let coldkey_account_id = 81337;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		let amount = 10000;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Some basic assertions
		assert_eq!(SubtensorModule::get_total_stake(), 0);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
		let initial_total_balance = Balances::total_balance(&coldkey_account_id);
		assert_eq!(initial_total_balance, 0);
		let inital_total_issuance = Balances::total_issuance();
		assert_eq!(inital_total_issuance, 0);

		// Issue the neuron some stake to remove
		SubtensorModule::issue_stake_to_coldkey_hotkey_account(&coldkey_account_id, &hotkey_account_id, amount);

		let total_issuance_after_stake = Balances::total_issuance();

		// Do the magic
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, amount));

		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_total_stake(), 0);

		// Check if total issuance is equal to the added stake, even after remove stake (no fee, includes reserved/locked balance)
		// Should also be equal to the total issuance after adding stake
		let total_issuance = Balances::total_issuance();
		assert_eq!(total_issuance, total_issuance_after_stake);
		assert_eq!(total_issuance, amount);
	});
}

#[test]
fn test_remove_stake_reserved_matches() {
	// When we remove stake, the reserved balance on the coldkey account should match the stake
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 581337;
		let coldkey_account_id = 81337;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		let amount = 10000;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Some basic assertions
		assert_eq!(SubtensorModule::get_total_stake(), 0);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
		let initial_total_balance = Balances::total_balance(&coldkey_account_id);
		assert_eq!(initial_total_balance, 0);
		let inital_reserved_balance = Balances::reserved_balance(&coldkey_account_id);
		assert_eq!(inital_reserved_balance, 0);

		// Give the neuron some stake to remove
		SubtensorModule::issue_stake_to_coldkey_hotkey_account(&coldkey_account_id, &hotkey_account_id, amount);

		// Check that the reserved balance on the coldkey account matches the stake
		let reserved_balance = Balances::reserved_balance(&coldkey_account_id);
		assert_eq!(reserved_balance, amount);

		// Do the magic
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id), hotkey_account_id, amount));

		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(SubtensorModule::get_total_stake(), 0);

		// Check that the reserved balance on the coldkey account matches the stake
		let new_reserved_balance = Balances::reserved_balance(&coldkey_account_id);
		assert_eq!(new_reserved_balance, 0);
		
	});
}

/***********************************************************
	staking::get_coldkey_balance() tests
************************************************************/
#[test]
fn test_get_coldkey_balance_no_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 5454; // arbitrary
		let result = SubtensorModule::get_coldkey_balance(&coldkey_account_id);

		// Arbitrary account should have 0 balance
		assert_eq!(result, 0);

	});
}

#[test]
fn test_get_coldkey_balance_with_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 5454; // arbitrary
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
	new_test_ext().execute_with(|| {
		let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// There is not stake in the system at first, so result should be 0;
		assert_eq!(SubtensorModule::get_total_stake(), 0);

		SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

		// The stake that is now in the account, should equal the amount
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), amount);

		// The total stake should have been increased by the amount -> 0 + amount = amount
		assert_eq!(SubtensorModule::get_total_stake(), amount);
	});
}

/************************************************************
	staking::remove_stake_from_hotkey_account() tests
************************************************************/
#[test]
fn test_remove_stake_from_hotkey_account() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Add some stake that can be removed
		SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

		// Prelimiary checks
		assert_eq!(SubtensorModule::get_total_stake(), amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), amount);

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
	new_test_ext().execute_with(|| {
		let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;
        let netuid: u16 = 1;
		let netuid_ex = 2;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		//
		add_network(netuid, tempo, 0);
		add_network(netuid_ex, tempo, 0);
		//
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		register_ok_neuron( netuid_ex, hotkey_id, coldkey_id, 48141209);
		
		//let neuron_uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id);
		let neuron_uid ;
        match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => neuron_uid = k,
            Err(e) => panic!("Error: {:?}", e),
        } 
		//let neuron_uid_ex = SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id);
		let neuron_uid_ex ;
        match SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id) {
            Ok(k) => neuron_uid_ex = k,
            Err(e) => panic!("Error: {:?}", e),
        } 
		//Add some stake that can be removed
		SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, amount);

		assert_eq!(SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid), amount);
		assert_eq!(SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex), amount);

		// Remove stake
		SubtensorModule::decrease_stake_on_hotkey_account(&hotkey_id, amount);
		//
		assert_eq!(SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid), 0);
		assert_eq!(SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex), 0);
	});
}


// /************************************************************
// 	staking::increase_total_stake() tests
// ************************************************************/
#[test]
fn test_increase_total_stake_ok() {
	new_test_ext().execute_with(|| {
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
	new_test_ext().execute_with(|| {
        let initial_total_stake = 10000;
		let decrement = 5000;

		SubtensorModule::increase_total_stake(initial_total_stake);
		SubtensorModule::decrease_total_stake(decrement);

		// The total stake remaining should be the difference between the initial stake and the decrement
		assert_eq!(SubtensorModule::get_total_stake(), initial_total_stake - decrement);
	});
}

// /************************************************************
// 	staking::add_balance_to_coldkey_account() tests
// ************************************************************/
#[test]
fn test_add_balance_to_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 4444322;
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
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary
		// Put some $$ on the bank
		SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), ammount);
		// Should be able to withdraw without hassle
		let result = SubtensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(result, true);
	});
}

#[test]
fn test_remove_balance_from_coldkey_account_failed() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary

		// Try to remove stake from the coldkey account. This should fail,
		// as there is no balance, nor does the account exist
		let result = SubtensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(result, false);
	});
}

//************************************************************
// 	staking::hotkey_belongs_to_coldkey() tests
// ************************************************************/
#[test]
fn test_hotkey_belongs_to_coldkey_ok() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4434334;
		let coldkey_id = 34333;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_id), coldkey_id);
	});
}
// /************************************************************
// 	staking::can_remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_can_remove_balane_from_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 87987984;
		let initial_amount = 10000;
		let remove_amount = 5000;
		SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
		assert_eq!(SubtensorModule::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount), true);
	});
}

#[test]
fn test_can_remove_balance_from_coldkey_account_err_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 87987984;
		let initial_amount = 10000;
		let remove_amount = 20000;
		SubtensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
		assert_eq!(SubtensorModule::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount), false);
	});
}
/************************************************************
	staking::has_enough_stake() tests
************************************************************/
#[test]
fn test_has_enough_stake_yes() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;
		let coldkey_id = 87989;
		let intial_amount = 10000;
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, intial_amount);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 10000);
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), 10000);
		assert_eq!(SubtensorModule::has_enough_stake(&coldkey_id, &hotkey_id, 5000), true);
	});
}

#[test]
fn test_has_enough_stake_no() {
	new_test_ext().execute_with(|| {
		let hotkey_id = 4334;
		let coldkey_id = 87989;
		let intial_amount = 0;
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		SubtensorModule::increase_stake_on_hotkey_account(&hotkey_id, intial_amount);
		assert_eq!(SubtensorModule::has_enough_stake(&coldkey_id, &hotkey_id, 5000), false);

	});
}

#[test]
fn test_non_existent_account() {
	new_test_ext().execute_with(|| {
		SubtensorModule::increase_stake_on_coldkey_hotkey_account( &0, &(0 as u64), 10 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &0, &0 ), 10 );
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&(0 as u64)), 10);
	});
}

/************************************************************
	staking::delegating
************************************************************/

#[test]
fn test_delegate_stake_division_by_zero_check(){
    new_test_ext().execute_with(|| { 
        let netuid: u16 = 0;
        let tempo: u16 = 1;
		let hotkey = 1;
		let coldkey = 3;
        add_network( netuid, tempo, 0 );
		register_ok_neuron( netuid, hotkey, coldkey, 2341312 );
        assert_ok!(SubtensorModule::become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey), hotkey) );
        SubtensorModule::emit_inflation_through_hotkey_account( &hotkey, 1000 );
    });
}

#[test]
fn test_full_with_delegating() {
	new_test_ext().execute_with(|| {

		// Make two accounts.
        let hotkey0 = 1;
        let hotkey1 = 2;

		let coldkey0 = 3;
		let coldkey1 = 4;
		SubtensorModule::set_max_registrations_per_block(1,4);

		// Neither key can add stake because they dont have fundss.
		assert_eq!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 60000), Err(Error::<Test>::NotEnoughBalanceToStake.into()));
		assert_eq!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, 60000), Err(Error::<Test>::NotEnoughBalanceToStake.into()));

		// Add balances.
		SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 60000);
		SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 60000);

		// We have enough, but the keys are not registered.
		assert_eq!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 100), Err(Error::<Test>::NotRegistered.into()));
		assert_eq!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 100), Err(Error::<Test>::NotRegistered.into()));

		// Cant remove either.
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 10), Err(Error::<Test>::NotRegistered.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, 10), Err(Error::<Test>::NotRegistered.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey1, 10), Err(Error::<Test>::NotRegistered.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey0, 10), Err(Error::<Test>::NotRegistered.into()));

		// Neither key can become a delegate either because we are not registered.
		assert_eq!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 100), Err(Error::<Test>::NotRegistered.into()));
		assert_eq!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 100), Err(Error::<Test>::NotRegistered.into()));
		
		// Register the 2 neurons to a new network.
		let netuid = 1;
		add_network(netuid, 0, 0);
		register_ok_neuron( netuid, hotkey0, coldkey0, 124124 );
		register_ok_neuron( netuid, hotkey1, coldkey1, 987907 );
		assert_eq!( SubtensorModule::get_owning_coldkey_for_hotkey( &hotkey0 ), coldkey0 );
		assert_eq!( SubtensorModule::get_owning_coldkey_for_hotkey( &hotkey1 ), coldkey1 );
		assert!( SubtensorModule::coldkey_owns_hotkey( &coldkey0, &hotkey0 ) );
		assert!( SubtensorModule::coldkey_owns_hotkey( &coldkey1, &hotkey1 ) );

		// We try to delegate stake but niether are allowing delegation.
		assert!( !SubtensorModule::hotkey_is_delegate( &hotkey0 ) );
		assert!( !SubtensorModule::hotkey_is_delegate( &hotkey1 ) );
		assert_eq!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey1, 100), Err(Error::<Test>::NonAssociatedColdKey.into()));
		assert_eq!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey0, 100), Err(Error::<Test>::NonAssociatedColdKey.into()));

		// We stake and all is ok.
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 0 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 0 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 0 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 0 );
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 100) );
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, 100) );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 100 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey1 ), 0 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey0 ), 0 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey1 ), 100 );
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey0 ), 100 );
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey1 ), 100 );
		//assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 100 );
		//assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 100 );
		assert_eq!( SubtensorModule::get_total_stake(), 200 );

		// Cant remove these funds because we are not delegating.
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey1, 10), Err(Error::<Test>::NonAssociatedColdKey.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey0, 10), Err(Error::<Test>::NonAssociatedColdKey.into()));

		// Emit inflation through non delegates.
		SubtensorModule::emit_inflation_through_hotkey_account( &hotkey0, 100 );
		SubtensorModule::emit_inflation_through_hotkey_account( &hotkey1, 100 );
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey0 ), 200);
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey1 ), 200 );

		// Try allowing the keys to become delegates, fails because of incorrect coldkeys.
		// Set take to be 0.
		assert_eq!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey1, 0), Err(Error::<Test>::NonAssociatedColdKey.into()));
		assert_eq!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey0, 0), Err(Error::<Test>::NonAssociatedColdKey.into()));

		// Become delegates all is ok.
		assert_ok!( SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 10) ); 
		assert_ok! (SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, 10) );
		assert!( SubtensorModule::hotkey_is_delegate( &hotkey0 ) );
		assert!( SubtensorModule::hotkey_is_delegate( &hotkey1 ) );

		// Cant become a delegate twice.
		assert_eq!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 1000), Err(Error::<Test>::AlreadyDelegate.into()));
		assert_eq!(SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, 1000), Err(Error::<Test>::AlreadyDelegate.into()));

		// This add stake works for delegates.
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 200 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey1 ), 0 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey0 ), 0 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey1 ), 200 );
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey1, 200) );
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey0, 300) );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 200 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey1 ), 200 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey0 ), 300 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey1 ), 200 );
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey0 ), 500 );
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey1 ), 400 );
		//assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey0 ), 400 );
		//assert_eq!( SubtensorModule::get_total_stake_for_coldkey( &coldkey1 ), 500 );
		assert_eq!( SubtensorModule::get_total_stake(), 900 );

		// Lets emit inflation through the hot and coldkeys.
		SubtensorModule::emit_inflation_through_hotkey_account( &hotkey0, 1000 ); 
		SubtensorModule::emit_inflation_through_hotkey_account( &hotkey1, 1000 );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 599 ); // 200 + 1000 x ( 200 / 500 ) = 200 + 400 = 600 ~= 599
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey1 ), 700 ); // 200 + 1000 x ( 200 / 400 ) = 200 + 500 = 700
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey0 ), 899 ); // 300 + 1000 x ( 300 / 500 ) = 300 + 600 = 900 ~= 899
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey1 ), 700 ); // 200 + 1000 x ( 200 / 400 ) = 300 + 600 = 700
		assert_eq!( SubtensorModule::get_total_stake(), 2898 ); // 600 + 700 + 900 + 700 = 2900 ~= 2898 

		// // Try unstaking too much.
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 100000), Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, 100000), Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey1, 100000), Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey0, 100000), Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));

		// unstaking is ok.
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey0, 100) );
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey1, 100) );
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey1, 100) );
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey0, 100) );

		// All the amounts have been decreased.
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey0 ), 499 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey1 ), 600 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey0 ), 799 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey1 ), 600 ); 

		// Lets register and stake a new key.
		let hotkey2 = 5;
		let coldkey2 = 6; 
		register_ok_neuron( netuid, hotkey2, coldkey2, 248123 );
		SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 60000);
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey2), hotkey2, 1000) );
		assert_ok!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey2), hotkey2, 100) );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey2, &hotkey2 ), 900 ); 
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey2, 10), Err(Error::<Test>::NonAssociatedColdKey.into()));
		assert_eq!(SubtensorModule::remove_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey2, 10), Err(Error::<Test>::NonAssociatedColdKey.into()));

		// Lets make this new key a delegate with a 50% take.
		assert_ok!( SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey2), hotkey2, u16::MAX/2) ); 

		// Add nominate some stake.
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey2, 1000) );
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey2, 1000) );
		assert_ok!(SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey2), hotkey2, 100) );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey2, &hotkey2 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey2 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey2 ), 1000 ); 
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey2 ), 3000 ); 
		assert_eq!( SubtensorModule::get_total_stake(), 5_498 );

		// Lets emit inflation through this new key with distributed ownership.
		SubtensorModule::emit_inflation_through_hotkey_account( &hotkey2, 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey2, &hotkey2 ), 1_665 ); // 1000 + 500 + 500 * (1000/3000) = 1500 + 166.6666666667 = 1,666.6666666667
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey2 ), 1_166 ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey2 ), 1_166 ); // 1000 + 500 * (1000/3000) = 1000 + 166.6666666667 = 1166.6
		assert_eq!( SubtensorModule::get_total_stake(), 6_495 );

		step_block(1);

		// Lets register and stake a new key.
		let hotkey3 = 7;
		let coldkey3 = 8; 
		register_ok_neuron( netuid, hotkey3, coldkey3, 4124124 );
		SubtensorModule::add_balance_to_coldkey_account(&coldkey3, 60000);
		assert_ok!( SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey3), hotkey3, 1000) );

		step_block(3);

		assert_ok!( SubtensorModule::do_become_delegate(<<Test as Config>::RuntimeOrigin>::signed(coldkey3), hotkey3, u16::MAX ) ); // Full take. 
		assert_ok!( SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey0), hotkey3, 1000) );
		assert_ok!( SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey1), hotkey3, 1000) );
		assert_ok!( SubtensorModule::add_stake(<<Test as Config>::RuntimeOrigin>::signed(coldkey2), hotkey3, 1000) );
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey3 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey3 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey2, &hotkey3 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey3, &hotkey3 ), 1000 ); 
		assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey3 ), 4000 ); 
		assert_eq!( SubtensorModule::get_total_stake(), 10_495 );
		SubtensorModule::emit_inflation_through_hotkey_account( &hotkey3, 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey0, &hotkey3 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey1, &hotkey3 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey2, &hotkey3 ), 1000 ); 
		assert_eq!( SubtensorModule::get_stake_for_coldkey_and_hotkey( &coldkey3, &hotkey3 ), 2000 ); 
		assert_eq!( SubtensorModule::get_total_stake(), 11_495 );

	});
}


/************************************************************
 staking::increase_reserved_on_coldkey_account* tests
 ************************************************************/

 #[test]
 fn test_increase_reserved_on_coldkey_account() {
	 new_test_ext().execute_with(|| {
		 let hotkey_id = 4334;
 
		 let coldkey_id = 87989;
		 let amount = 10_000;
 
		 let inital_free_balance = amount + 23_000;
		 
		 let netuid = 1;
		 let tempo: u16 = 13;
		 let start_nonce: u64 = 0;
		 add_network(netuid, tempo, 0);
 
		 register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
 
		 // Sanity checks
		 assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));
		 assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), 0);
 
		 // Issue free balance
		 assert_eq!(Balances::deposit_creating(&coldkey_id, Balance::from(inital_free_balance)).peek(), Balance::from(inital_free_balance));
		 
		 // Verify the reserved balance and stake
		 assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));
 
		 // Reserve balance and add stake using the helper
		 assert_ok!(SubtensorModule::increase_reserved_on_coldkey_account(&coldkey_id, amount));
 
		 // Verify the reserved balance on the coldkey account matches new reserved amount
		 assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from( amount ));
		 // Verify free balance matches new free balance
		 assert_eq!(Balances::free_balance(&coldkey_id), Balance::from(
			 inital_free_balance - amount
		 ));
 
		 // Sanity checks
		 assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey_id), 0); // Nothing was staked
		 assert_eq!(SubtensorModule::get_total_stake(), 0);
		 assert_eq!(Balances::total_balance(&coldkey_id), Balance::from(inital_free_balance)); // Free + reserved
		 assert_eq!(Balances::total_issuance(), Balance::from(inital_free_balance)); // We don't issue any new TAO
	 });
 }

#[test]
fn test_increased_reserved_by_hotkey() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;

		let coldkey_id = 87989;
		let amount = 10_000;
		
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);
		
		// Give the coldkey account some reserved balance 
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));

		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		
		// Increase reserved balance using the hotkey
		SubtensorModule::increase_reserved_on_coldkey_account_issuing_using_hotkey( &hotkey_id, amount );

		// Verify the reserved balance on the coldkey account matches
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(amount));
	});
}

#[test]
fn test_increase_reserved_on_coldkey_account_issuing() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 87989;
		let amount = 10_000;
		
		// Increase reserved balance using the helper
		SubtensorModule::increase_reserved_on_coldkey_account_issuing( &coldkey_id, amount );

		// Verify the reserved balance on the coldkey account matches
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(amount));
	});
}


/************************************************************
 staking::decrease_reserved_on_coldkey_account tests
 ************************************************************/

#[test]
fn test_decrease_reserved_on_coldkey_account() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 87989;
		let amount = 10_000;
		let initial_reserved = amount + 1000;

		// Issue some reserved balance to the coldkey account
		assert_eq!( Balances::deposit_creating(&coldkey_id, initial_reserved).peek(), initial_reserved );
		assert_ok!( Balances::reserve(&coldkey_id, initial_reserved) );
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(initial_reserved) );
		
		// Increase reserved balance using the helper
		SubtensorModule::decrease_reserved_on_coldkey_account( &coldkey_id, amount );

		// Verify the reserved balance is now zero
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(initial_reserved - amount));
	});
}

/************************************************************
 staking::issue_stake_to_* tests
 ************************************************************/

#[test]
fn test_issue_stake_to_coldkey_hotkey_account() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;

		let coldkey_id = 87989;
		let amount = 10_000;
		
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);

		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Sanity checks
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), 0);
		
		// Issue reserved balance and stake using the helper
		SubtensorModule::issue_stake_to_coldkey_hotkey_account(&coldkey_id, &hotkey_id, amount);

		// Verify the reserved balance on the coldkey account matches
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(amount));
		// Verify the stake matches
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), amount);

		// Sanity check
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey_id), amount);
		assert_eq!(SubtensorModule::get_total_stake(), amount);
		assert_eq!(Balances::total_balance(&coldkey_id), Balance::from(amount));
		assert_eq!(Balances::total_issuance(), Balance::from(amount));
	});
}

#[test]
fn test_issue_stake_to_hotkey_owner_account() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;

		let coldkey_id = 87989;
		let amount = 10_000;
		
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);

		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Sanity checks
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), 0);
		
		// Issue reserved balance and stake using the helper
		SubtensorModule::issue_stake_to_hotkey_owner_account(&hotkey_id, amount);

		// Verify the reserved balance on the coldkey account matches
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(amount));
		// Verify the stake matches
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), amount);

		// Sanity check
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey_id), amount);
		assert_eq!(SubtensorModule::get_total_stake(), amount);
		assert_eq!(Balances::total_balance(&coldkey_id), Balance::from(amount));
		assert_eq!(Balances::total_issuance(), Balance::from(amount));
	});
}

/************************************************************
 staking::(un|)reserve_stake_(to|from)_coldkey_hotkey_account tests
 ************************************************************/

#[test]
fn test_reserve_stake_to_coldkey_hotkey_account() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;

		let coldkey_id = 87989;
		let amount = 10_000;

		let inital_free_balance = amount + 23_000;
		
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);

		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Sanity checks
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), 0);

		// Issue free balance
		assert_eq!(Balances::deposit_creating(&coldkey_id, Balance::from(inital_free_balance)).peek(), Balance::from(inital_free_balance));
		
		// Verify the reserved balance and stake
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), 0);

		// Reserve balance and add stake using the helper
		assert_ok!(SubtensorModule::reserve_stake_to_coldkey_hotkey_account(&coldkey_id, &hotkey_id, amount));

		// Verify the reserved balance on the coldkey account matches new reserved amount
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from( amount ));
		// Verify the stake matches new staked amount
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), amount);
		// Verify free balance matches new free balance
		assert_eq!(Balances::free_balance(&coldkey_id), Balance::from(
			inital_free_balance - amount
		));

		// Sanity checks
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey_id), amount);
		assert_eq!(SubtensorModule::get_total_stake(), amount);
		assert_eq!(Balances::total_balance(&coldkey_id), Balance::from(inital_free_balance)); // Free + reserved
		assert_eq!(Balances::total_issuance(), Balance::from(inital_free_balance)); // We don't issue any new TAO
	});
}

fn test_unreserve_stake_from_coldkey_hotkey_account() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;

		let coldkey_id = 87989;
		let amount = 10_000;

		let initial_staked = amount + 23_000;
		
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		add_network(netuid, tempo, 0);

		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Sanity checks
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(0 as u64));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), 0);

		// Issue reserved balance and stake
		assert_eq!(Balances::deposit_creating(&coldkey_id, Balance::from(initial_staked)).peek(), Balance::from(initial_staked));
		// -- Reserve the balance
		assert_ok!(Balances::reserve(&coldkey_id, Balance::from(initial_staked)));
		// -- Set the stake
		SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey_id, &hotkey_id, initial_staked);
		
		// Verify the reserved balance and stake
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(initial_staked));
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), initial_staked);

		// Unreserve balance and remove stake using the helper
		SubtensorModule::unreserve_stake_from_coldkey_hotkey_account(&coldkey_id, &hotkey_id, amount);

		// Verify the reserved balance on the coldkey account matches decreased reserved amount
		assert_eq!(Balances::reserved_balance(&coldkey_id), Balance::from(
			initial_staked - amount
		 ));
		// Verify the stake matches decreased staked amount
		assert_eq!(SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey_id, &hotkey_id), initial_staked - amount );
		// Verify free balance matches remmoved amount
		assert_eq!(Balances::free_balance(&coldkey_id), Balance::from( amount ));

		// Sanity checks
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&coldkey_id), initial_staked - amount);
		assert_eq!(SubtensorModule::get_total_stake(), initial_staked - amount);
		assert_eq!(Balances::total_balance(&coldkey_id), Balance::from(initial_staked)); // Free + reserved
		assert_eq!(Balances::total_issuance(), Balance::from(initial_staked)); // We don't issue any new TAO or burn any
	});
}

