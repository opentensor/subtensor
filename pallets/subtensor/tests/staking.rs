use frame_support::{assert_noop, assert_ok, traits::Currency};
use frame_system::Config;
mod mock;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use mock::*;
use pallet_subtensor::Error;
use sp_core::{H256, U256};

/***********************************************************
    staking::add_stake() tests
************************************************************/

#[test]
#[cfg(not(tarpaulin))]
fn test_add_stake_dispatch_info_ok() {
    new_test_ext().execute_with(|| {
        let hotkey = U256::from(0);
        let amount_staked = 5000;
        let call = RuntimeCall::Subtensor(SubtensorCall::add_stake {
            hotkey,
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

/*
#[test]
#[ignore]
fn test_add_stake_total_balance_no_change() {
    // When we add stake, the total balance of the coldkey account should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    new_test_ext().execute_with(|| {
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
        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = Subtensor::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Also total stake should be zero
        assert_eq!(Subtensor::get_total_stake(), 0);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(Subtensor::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            10000
        ));

        // Check if stake has increased
        let new_stake = Subtensor::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = Subtensor::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(Subtensor::get_total_stake(), 10000);

        // Check if total balance has remained the same. (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(total_balance, initial_total_balance);
    });
}
*/

/*
#[test]
#[ignore]
fn test_add_stake_total_issuance_no_change() {
    // When we add stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    new_test_ext().execute_with(|| {
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
        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = Subtensor::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, 0);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Check total issuance is equal to initial balance
        let initial_total_issuance = Balances::total_issuance();
        assert_eq!(initial_total_issuance, initial_balance);

        // Also total stake should be zero
        assert_eq!(Subtensor::get_total_stake(), 0);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(Subtensor::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            10000
        ));

        // Check if stake has increased
        let new_stake = Subtensor::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000);

        // Check if free balance has decreased
        let new_free_balance = Subtensor::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(Subtensor::get_total_stake(), 10000);

        // Check if total issuance has remained the same. (no fee, includes reserved/locked balance)
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, initial_total_issuance);
    });
}
*/

// /***********************************************************
// 	staking::remove_stake() tests
// ************************************************************/
#[test]
#[cfg(not(tarpaulin))]
fn test_remove_stake_dispatch_info_ok() {
    new_test_ext().execute_with(|| {
        let hotkey = U256::from(0);
        let amount_unstaked = 5000;
        let call = RuntimeCall::Subtensor(SubtensorCall::remove_stake {
            hotkey,
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

/*
#[test]
#[ignore]
fn test_remove_stake_total_issuance_no_change() {
    // When we remove stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext().execute_with(|| {
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
        assert_eq!(Subtensor::get_total_stake(), 0);
        assert_eq!(
            Subtensor::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);
        let inital_total_issuance = Balances::total_issuance();
        assert_eq!(inital_total_issuance, 0);

        // Give the neuron some stake to remove
        Subtensor::increase_stake_on_hotkey_account(&hotkey_account_id, amount);

        let total_issuance_after_stake = Balances::total_issuance();

        // Do the magic
        assert_ok!(Subtensor::remove_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            amount
        ));

        assert_eq!(
            Subtensor::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        assert_eq!(
            Subtensor::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(Subtensor::get_total_stake(), 0);

        // Check if total issuance is equal to the added stake, even after remove stake (no fee, includes reserved/locked balance)
        // Should also be equal to the total issuance after adding stake
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, total_issuance_after_stake);
        assert_eq!(total_issuance, amount);
    });
}
*/

/***********************************************************
    staking::get_coldkey_balance() tests
************************************************************/
#[test]
fn test_get_coldkey_balance_no_balance() {
    new_test_ext().execute_with(|| {
        let coldkey_account_id = U256::from(5454); // arbitrary
        let result = Subtensor::get_coldkey_balance(&coldkey_account_id);

        // Arbitrary account should have 0 balance
        assert_eq!(result, 0);
    });
}

#[test]
fn test_get_coldkey_balance_with_balance() {
    new_test_ext().execute_with(|| {
        let coldkey_account_id = U256::from(5454); // arbitrary
        let amount = 1337;

        // Put the balance on the account
        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        let result = Subtensor::get_coldkey_balance(&coldkey_account_id);

        // Arbitrary account should have 0 balance
        assert_eq!(result, amount);
    });
}

// /************************************************************
// 	staking::add_balance_to_coldkey_account() tests
// ************************************************************/
#[test]
fn test_add_balance_to_coldkey_account_ok() {
    new_test_ext().execute_with(|| {
        let coldkey_id = U256::from(4444322);
        let amount = 50000;
        Subtensor::add_balance_to_coldkey_account(&coldkey_id, amount);
        assert_eq!(Subtensor::get_coldkey_balance(&coldkey_id), amount);
    });
}

// /***********************************************************
// 	staking::remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_remove_balance_from_coldkey_account_ok() {
    new_test_ext().execute_with(|| {
        let coldkey_account_id = U256::from(434324); // Random
        let ammount = 10000 + 1; // Arbitrary
                             // Put some $$ on the bank
        Subtensor::add_balance_to_coldkey_account(&coldkey_account_id, ammount);
        assert_eq!(
            Subtensor::get_coldkey_balance(&coldkey_account_id),
            ammount
        );
        // Should be able to withdraw without hassle
        let result =
            Subtensor::remove_balance_from_coldkey_account(&coldkey_account_id, ammount - 1);
        assert_eq!(result, true);
    });
}

#[test]
fn test_remove_balance_from_coldkey_account_failed() {
    new_test_ext().execute_with(|| {
        let coldkey_account_id = U256::from(434324); // Random
        let ammount = 10000; // Arbitrary

        // Try to remove stake from the coldkey account. This should fail,
        // as there is no balance, nor does the account exist
        let result =
            Subtensor::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
        assert_eq!(result, false);
    });
}

//************************************************************
// 	staking::hotkey_belongs_to_coldkey() tests
// ************************************************************/
#[test]
fn test_hotkey_belongs_to_coldkey_ok() {
    new_test_ext().execute_with(|| {
        let hotkey_id = U256::from(4434334);
        let coldkey_id = U256::from(34333);
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let start_nonce: u64 = 0;
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, start_nonce);
        assert_eq!(
            Subtensor::get_owning_coldkey_for_hotkey(&hotkey_id),
            coldkey_id
        );
    });
}
// /************************************************************
// 	staking::can_remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_can_remove_balane_from_coldkey_account_ok() {
    new_test_ext().execute_with(|| {
        let coldkey_id = U256::from(87987984);
        let initial_amount = 10000;
        let remove_amount = 5000;
        Subtensor::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
        assert_eq!(
            Subtensor::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount),
            true
        );
    });
}

#[test]
fn test_can_remove_balance_from_coldkey_account_err_insufficient_balance() {
    new_test_ext().execute_with(|| {
        let coldkey_id = U256::from(87987984);
        let initial_amount = 10000;
        let remove_amount = 20000;
        Subtensor::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
        assert_eq!(
            Subtensor::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount),
            false
        );
    });
}

/*
#[test]
fn test_faucet_ok() {
    new_test_ext().execute_with(|| {
        let coldkey = U256::from(123560);

        log::info!("Creating work for submission to faucet...");

        let block_number = Subtensor::get_current_block_as_u64();
        let difficulty: U256 = U256::from(10_000_000);
        let mut nonce: u64 = 0;
        let mut work: H256 = Subtensor::create_seal_hash(block_number, nonce, &coldkey);
        while !Subtensor::hash_meets_difficulty(&work, difficulty) {
            nonce = nonce + 1;
            work = Subtensor::create_seal_hash(block_number, nonce, &coldkey);
        }
        let vec_work: Vec<u8> = Subtensor::hash_to_vec(work);

        log::info!("Faucet state: {}", cfg!(feature = "pow-faucet"));

        #[cfg(feature = "pow-faucet")]
        assert_ok!(Subtensor::do_faucet(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            0,
            nonce,
            vec_work
        ));

        #[cfg(not(feature = "pow-faucet"))]
        assert_ok!(
            Subtensor::do_faucet(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                0,
                nonce,
                vec_work
            )
        );
    });
}*/
