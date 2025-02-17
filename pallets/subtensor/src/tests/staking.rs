#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use frame_system::RawOrigin;

use super::mock::*;
use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use sp_core::{Get, H256, U256};
use substrate_fixed::types::{I96F32, U64F64, U96F32};

/***********************************************************
    staking::add_stake() tests
************************************************************/

#[test]
fn test_add_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_staked = 5000;
        let netuid = 1;
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
#[test]
fn test_add_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        //add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Check we have zero staked before transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        // Transfer to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        // Check if stake has increased
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            amount - fee,
            epsilon = amount / 1000,
        );

        // Check if balance has decreased
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 1);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), amount);
    });
}

#[test]
fn test_dividends_with_run_to_block() {
    new_test_ext(1).execute_with(|| {
        let neuron_src_hotkey_id = U256::from(1);
        let neuron_dest_hotkey_id = U256::from(2);
        let coldkey_account_id = U256::from(667);
        let hotkey_account_id = U256::from(668);
        let initial_stake: u64 = 5000;

        //add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);
        Tempo::<Test>::insert(netuid, 13);

        // Register neuron, this will set a self weight
        SubtensorModule::set_max_registrations_per_block(netuid, 3);
        SubtensorModule::set_max_allowed_uids(1, 5);

        register_ok_neuron(netuid, neuron_src_hotkey_id, coldkey_account_id, 192213123);
        register_ok_neuron(netuid, neuron_dest_hotkey_id, coldkey_account_id, 12323);

        // Add some stake to the hotkey account, so we can test for emission before the transfer takes place
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &neuron_src_hotkey_id,
            &coldkey_account_id,
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
fn test_add_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(654); // bogus
        let amount = 20000; // Not used
        let netuid = 1;

        assert_err!(
            SubtensorModule::add_stake(RawOrigin::None.into(), hotkey_account_id, netuid, amount),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_add_stake_not_registered_key_pair() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let coldkey_account_id = U256::from(435445);
        let hotkey_account_id = U256::from(54544);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount + fee);
        assert_err!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount
            ),
            Error::<Test>::HotKeyAccountNotExists
        );
    });
}

#[test]
fn test_add_stake_ok_neuron_does_not_belong_to_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let other_cold_key = U256::from(99498);
        let netuid: u16 = add_dynamic_network(&hotkey_id, &coldkey_id);
        let stake = DefaultMinStake::<Test>::get() * 10;

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&other_cold_key, stake);

        // Perform the request which is signed by a different cold key
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(other_cold_key),
            hotkey_id,
            netuid,
            stake,
        ));
    });
}

#[test]
fn test_add_stake_err_not_enough_belance() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let stake = DefaultMinStake::<Test>::get() * 10;
        let netuid: u16 = add_dynamic_network(&hotkey_id, &coldkey_id);

        // Lets try to stake with 0 balance in cold key account
        assert!(SubtensorModule::get_coldkey_balance(&coldkey_id) < stake);
        assert_err!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_id),
                hotkey_id,
                netuid,
                stake,
            ),
            Error::<Test>::NotEnoughBalanceToStake
        );
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
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

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
            RuntimeOrigin::signed(coldkey_account_id),
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
fn test_add_stake_total_issuance_no_change() {
    // When we add stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

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
            RuntimeOrigin::signed(coldkey_account_id),
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

#[test]
fn test_remove_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_unstaked = 5000;
        let netuid = 1;
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

#[test]
fn test_remove_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            amount,
        );

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        // we do not expect the exact amount due to slippage
        assert!(SubtensorModule::get_coldkey_balance(&coldkey_account_id) > amount / 10 * 9 - fee);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), fee);
    });
}

#[test]
fn test_remove_stake_amount_too_low() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let amount = 10_000;
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            amount,
        );

        // Do the magic
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                0
            ),
            Error::<Test>::AmountTooLow
        );
    });
}

#[test]
fn test_remove_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(4968585);
        let amount = 10000; // Amount to be removed
        let netuid = 1;

        assert_err!(
            SubtensorModule::remove_stake(
                RawOrigin::None.into(),
                hotkey_account_id,
                netuid,
                amount,
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_remove_stake_ok_hotkey_does_not_belong_to_coldkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let other_cold_key = U256::from(99498);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let netuid: u16 = add_dynamic_network(&hotkey_id, &coldkey_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &other_cold_key,
            netuid,
            amount,
        );

        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(other_cold_key),
            hotkey_id,
            netuid,
            amount,
        ));
    });
}

#[test]
fn test_remove_stake_no_enough_stake() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);

        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);

        assert_err!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_id),
                hotkey_id,
                netuid,
                amount,
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

#[test]
fn test_remove_stake_total_balance_no_change() {
    // When we remove stake, the total balance of the coldkey account should not change
    //    (except for staking fees)
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let hotkey_account_id = U256::from(571337);
        let coldkey_account_id = U256::from(71337);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

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
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            amount,
        );

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount - fee,
            epsilon = amount / 1000,
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(SubtensorModule::get_total_stake(), fee);

        // Check total balance is equal to the added stake. Even after remove stake (no fee, includes reserved/locked balance)
        let total_balance = Balances::total_balance(&coldkey_account_id);
        assert_abs_diff_eq!(total_balance, amount - fee, epsilon = amount / 1000);
    });
}

#[test]
fn test_add_stake_insufficient_liquidity() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let amount_staked = DefaultMinStake::<Test>::get() * 10 + DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Set the liquidity at lowest possible value so that all staking requests fail
        SubnetTAO::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );
        SubnetAlphaIn::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );

        // Check the error
        assert_noop!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                amount_staked
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn test_remove_stake_insufficient_liquidity() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let amount_staked = DefaultMinStake::<Test>::get() * 10 + DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Simulate stake for hotkey
        SubnetTAO::<Test>::insert(netuid, u64::MAX / 1000);
        SubnetAlphaIn::<Test>::insert(netuid, u64::MAX / 1000);
        let alpha = SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, amount_staked, 0);

        // Set the liquidity at lowest possible value so that all staking requests fail
        SubnetTAO::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );
        SubnetAlphaIn::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );

        // Check the error
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, alpha),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn test_remove_stake_total_issuance_no_change() {
    // When we remove stake, the total issuance of the balances pallet should not change
    //    this is because the stake should be part of the coldkey account balance (reserved/locked)
    //    then the removed stake just becomes free balance
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let hotkey_account_id = U256::from(581337);
        let coldkey_account_id = U256::from(81337);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let fee = DefaultStakingFee::<Test>::get();
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Some basic assertions
        assert_eq!(SubtensorModule::get_total_stake(), 0);
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, amount);
        let inital_total_issuance = Balances::total_issuance();

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        let total_issuance_after_stake = Balances::total_issuance();

        // Remove all stake
        let stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            stake
        ));

        let total_issuance_after_unstake = Balances::total_issuance();

        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount - fee * 2,
            epsilon = amount / 1000,
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake(),
            fee * 2,
            epsilon = fee / 1000
        );

        // Check if total issuance is equal to the added stake, even after remove stake (no fee, includes reserved/locked balance)
        assert_abs_diff_eq!(
            inital_total_issuance,
            total_issuance_after_stake + amount,
            epsilon = 1,
        );

        // After staking + unstaking the 2 * fee amount stays in SubnetTAO and TotalStake,
        // so the total issuance should be lower by that amount
        assert_abs_diff_eq!(
            inital_total_issuance,
            total_issuance_after_unstake + 2 * fee,
            epsilon = inital_total_issuance / 10000,
        );
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
// 	staking::increase_stake_for_hotkey_and_coldkey_on_subnet() tests
// ************************************************************/
#[test]
fn test_add_stake_to_hotkey_account_ok() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount = 10_000;
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, 192213123);

        // There is not stake in the system at first, so result should be 0;
        assert_eq!(SubtensorModule::get_total_stake(), 0);

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount,
        );

        // The stake that is now in the account, should equal the amount
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );
    });
}

/************************************************************
    staking::remove_stake_from_hotkey_account() tests
************************************************************/
#[test]
fn test_remove_stake_from_hotkey_account() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount = 10_000;
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, 192213123);

        // Add some stake that can be removed
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount,
        );

        // Prelimiary checks
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount
        );

        // Remove stake
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount,
        );

        // The stake on the hotkey account should be 0
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey_id), 0);
    });
}

#[test]
fn test_remove_stake_from_hotkey_account_registered_in_various_networks() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(5445);
        let coldkey_id = U256::from(5443433);
        let amount: u64 = 10_000;
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);
        let netuid_ex = add_dynamic_network(&hotkey_id, &coldkey_id);

        let neuron_uid = match SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => k,
            Err(e) => panic!("Error: {:?}", e),
        };

        let neuron_uid_ex = match SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id)
        {
            Ok(k) => k,
            Err(e) => panic!("Error: {:?}", e),
        };

        // Add some stake that can be removed
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount,
        );

        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            0
        );

        // Remove all stake
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount,
        );

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
        let intial_amount = 10_000;
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            intial_amount,
        );

        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            intial_amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid
            ),
            intial_amount
        );
        assert!(SubtensorModule::has_enough_stake_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            intial_amount / 2
        ));
    });
}

#[test]
fn test_has_enough_stake_no() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4334);
        let coldkey_id = U256::from(87989);
        let intial_amount = 10_000;
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            intial_amount,
        );

        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            intial_amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid
            ),
            intial_amount
        );
        assert!(!SubtensorModule::has_enough_stake_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            intial_amount * 2
        ));
    });
}

#[test]
fn test_has_enough_stake_no_for_zero() {
    new_test_ext(1).execute_with(|| {
        let hotkey_id = U256::from(4334);
        let coldkey_id = U256::from(87989);
        let intial_amount = 0;
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);

        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            intial_amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid
            ),
            intial_amount
        );
        assert!(!SubtensorModule::has_enough_stake_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            1_000
        ));
    });
}

#[test]
fn test_non_existent_account() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &U256::from(0),
            &(U256::from(0)),
            netuid,
            10,
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(0),
                &U256::from(0),
                netuid
            ),
            10
        );
        // No subnets => no iteration => zero total stake
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&(U256::from(0))),
            0
        );
    });
}

/************************************************************
    staking::delegating
************************************************************/

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
            RuntimeOrigin::signed(coldkey),
            block_number,
            nonce,
            vec_work
        ));

        #[cfg(not(feature = "pow-faucet"))]
        assert_ok!(SubtensorModule::do_faucet(
            RuntimeOrigin::signed(coldkey),
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
        // Create subnet and accounts.
        let subnet_owner_coldkey = U256::from(10);
        let subnet_owner_hotkey = U256::from(20);
        let hot1 = U256::from(1);
        let hot2 = U256::from(2);
        let cold1 = U256::from(3);
        let cold2 = U256::from(4);
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let amount = DefaultMinStake::<Test>::get() * 10;
        let fee: u64 = DefaultMinStake::<Test>::get();
        let init_balance = amount + fee + ExistentialDeposit::get();

        // Register hot1.
        register_ok_neuron(netuid, hot1, cold1, 0);
        Delegates::<Test>::insert(hot1, SubtensorModule::get_min_delegate_take());
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot1), cold1);

        // Register hot2.
        register_ok_neuron(netuid, hot2, cold2, 0);
        Delegates::<Test>::insert(hot2, SubtensorModule::get_min_delegate_take());
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot2), cold2);

        // Add stake cold1 --> hot1 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, init_balance);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(cold1),
            hot1,
            netuid,
            amount + fee
        ));
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(cold1),
            hot1,
            netuid,
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid)
                - 100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            100
        );

        // Add stake cold2 --> hot1 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, init_balance);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(cold2),
            hot1,
            netuid,
            amount + fee
        ));
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(cold2),
            hot1,
            netuid,
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid)
                - 100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            100
        );

        // Add stake cold1 --> hot2 (non delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold1, init_balance);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(cold1),
            hot2,
            netuid,
            amount + fee
        ));
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(cold1),
            hot2,
            netuid,
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold1, netuid)
                - 100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold1, netuid),
            100
        );
        let balance1_before_cleaning = Balances::free_balance(cold1);

        // Add stake cold2 --> hot2 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, init_balance);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(cold2),
            hot2,
            netuid,
            amount + fee
        ));
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(cold2),
            hot2,
            netuid,
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold2, netuid)
                - 100
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold2, netuid),
            100
        );
        let balance2_before_cleaning = Balances::free_balance(cold2);

        // Run clear all small nominations when min stake is zero (noop)
        SubtensorModule::set_nominator_min_required_stake(0);
        assert_eq!(SubtensorModule::get_nominator_min_required_stake(), 0);
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold1, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            100
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot2, &cold2, netuid),
            100
        );

        // Set min nomination to 10
        // let total_cold1_stake_before = TotalColdkeyAlpha::<Test>::get(cold1, netuid);
        // let total_cold2_stake_before = TotalColdkeyAlpha::<Test>::get(cold2, netuid); (DEPRECATED)
        let total_hot1_stake_before = TotalHotkeyAlpha::<Test>::get(hot1, netuid);
        let total_hot2_stake_before = TotalHotkeyAlpha::<Test>::get(hot2, netuid);
        let total_stake_before = TotalStake::<Test>::get();
        SubtensorModule::set_nominator_min_required_stake(1000);

        // Run clear all small nominations (removes delegations under 10)
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            100
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
            100
        );

        // Balances have been added back into accounts.
        let balance1_after_cleaning = Balances::free_balance(cold1);
        let balance2_after_cleaning = Balances::free_balance(cold2);
        assert_eq!(balance1_before_cleaning + 100, balance1_after_cleaning);
        assert_eq!(balance2_before_cleaning + 100, balance2_after_cleaning);

        assert_abs_diff_eq!(
            TotalHotkeyAlpha::<Test>::get(hot2, netuid),
            total_hot2_stake_before - 100,
            epsilon = 1
        );
        assert_abs_diff_eq!(
            TotalHotkeyAlpha::<Test>::get(hot1, netuid),
            total_hot1_stake_before - 100,
            epsilon = 1
        );
        assert_eq!(TotalStake::<Test>::get(), total_stake_before - 200);
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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take());
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 decreases take to 5%. This should fail as the minimum take is 9%
        assert_err!(
            SubtensorModule::do_decrease_take(
                RuntimeOrigin::signed(coldkey0),
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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates
        Delegates::<Test>::insert(hotkey0, u16::MAX / 10);

        // Coldkey / hotkey 0 decreases take to min
        assert_ok!(SubtensorModule::do_decrease_take(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Set min take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take());

        // Coldkey / hotkey 0 tries to increase take to 12.5%
        assert_eq!(
            SubtensorModule::do_decrease_take(
                RuntimeOrigin::signed(coldkey0),
                hotkey0,
                SubtensorModule::get_max_delegate_take()
            ),
            Err(Error::<Test>::DelegateTakeTooLow.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take());
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 decreases take to 12.5%
        assert_ok!(SubtensorModule::do_increase_take(
            RuntimeOrigin::signed(coldkey0),
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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take());
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 tries to decrease take to 5%
        assert_eq!(
            SubtensorModule::do_increase_take(
                RuntimeOrigin::signed(coldkey0),
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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take());
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultDelegateTake+1
        assert_ok!(SubtensorModule::do_increase_take(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            InitialDefaultDelegateTake::get()
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            InitialDefaultDelegateTake::get()
        );
    });
}

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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take());
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Coldkey / hotkey 0 tries to increase take to InitialDefaultDelegateTake+1
        // (Disable this check if InitialDefaultDelegateTake is u16::MAX)
        if InitialDefaultDelegateTake::get() != u16::MAX {
            assert_eq!(
                SubtensorModule::do_increase_take(
                    RuntimeOrigin::signed(coldkey0),
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
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take());
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Increase take first time
        assert_ok!(SubtensorModule::do_increase_take(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take() + 1
        ));

        // Increase again
        assert_eq!(
            SubtensorModule::do_increase_take(
                RuntimeOrigin::signed(coldkey0),
                hotkey0,
                SubtensorModule::get_min_delegate_take() + 2
            ),
            Err(Error::<Test>::DelegateTxRateLimitExceeded.into())
        );
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take() + 1
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Can increase after waiting
        assert_ok!(SubtensorModule::do_increase_take(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take() + 2
        ));
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take() + 2
        );
    });
}

#[test]
fn test_get_total_delegated_stake_after_unstaking() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let initial_stake = DefaultMinStake::<Test>::get() * 10;
        let unstake_amount = DefaultMinStake::<Test>::get() * 5;
        let existential_deposit = ExistentialDeposit::get();
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let fee = DefaultStakingFee::<Test>::get();

        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

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
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegator),
            initial_stake - existential_deposit - fee,
            epsilon = initial_stake / 1000,
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey),
            initial_stake - existential_deposit - fee,
            epsilon = initial_stake / 1000,
        );

        // Unstake part of the delegation
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            unstake_amount
        ));

        // Calculate the expected delegated stake
        let expected_delegated_stake = initial_stake - unstake_amount - existential_deposit - fee;

        // Debug prints
        log::debug!("Initial stake: {}", initial_stake);
        log::debug!("Unstake amount: {}", unstake_amount);
        log::debug!("Existential deposit: {}", existential_deposit);
        log::debug!("Expected delegated stake: {}", expected_delegated_stake);
        log::debug!(
            "Actual delegated stake: {}",
            SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey)
        );

        // Check the total delegated stake after unstaking
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegator),
            expected_delegated_stake,
            epsilon = expected_delegated_stake / 1000,
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey),
            expected_delegated_stake,
            epsilon = expected_delegated_stake / 1000,
        );
    });
}

#[test]
fn test_get_total_delegated_stake_no_delegations() {
    new_test_ext(1).execute_with(|| {
        let delegate = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = 1u16;

        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, delegate, coldkey, 0);

        // Check that there's no delegated stake
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&delegate), 0);
    });
}

#[test]
fn test_get_total_delegated_stake_single_delegator() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10 - 1;
        let existential_deposit = ExistentialDeposit::get();
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let fee = DefaultStakingFee::<Test>::get();

        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Add stake from delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, stake_amount);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            stake_amount
        ));

        // Debug prints
        log::debug!("Delegate coldkey: {:?}", delegate_coldkey);
        log::debug!("Delegate hotkey: {:?}", delegate_hotkey);
        log::debug!("Delegator: {:?}", delegator);
        log::debug!("Stake amount: {}", stake_amount);
        log::debug!("Existential deposit: {}", existential_deposit);
        log::debug!(
            "Total stake for hotkey: {}",
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey)
        );
        log::debug!(
            "Delegated stake for coldkey: {}",
            SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey)
        );

        // Calculate expected delegated stake
        let expected_delegated_stake = stake_amount - existential_deposit - fee;
        let actual_delegated_stake = SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey);
        let actual_delegator_stake = SubtensorModule::get_total_stake_for_coldkey(&delegator);

        assert_abs_diff_eq!(
            actual_delegated_stake,
            expected_delegated_stake,
            epsilon = expected_delegated_stake / 1000,
        );
        assert_abs_diff_eq!(
            actual_delegator_stake,
            expected_delegated_stake,
            epsilon = expected_delegated_stake / 1000,
        );
    });
}

#[test]
fn test_get_alpha_share_stake_multiple_delegators() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey1 = U256::from(2);
        let hotkey2 = U256::from(20);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);
        let existential_deposit = 2;
        let stake1 = DefaultMinStake::<Test>::get() * 10;
        let stake2 = DefaultMinStake::<Test>::get() * 10 - 1;
        let fee = DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey1, coldkey1, 0);
        register_ok_neuron(netuid, hotkey2, coldkey2, 0);

        // Add stake from delegator1
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, stake1);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey1),
            hotkey1,
            netuid,
            stake1
        ));

        // Add stake from delegator2
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, stake2);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey2),
            hotkey2,
            netuid,
            stake2
        ));

        // Debug prints
        println!("Delegator1 stake: {}", stake1);
        println!("Delegator2 stake: {}", stake2);
        println!(
            "Alpha share for for 1: {}",
            SubtensorModule::get_alpha_share_pool(hotkey1, netuid).get_value(&coldkey1)
        );
        println!(
            "Alpha share for for 2: {}",
            SubtensorModule::get_alpha_share_pool(hotkey2, netuid).get_value(&coldkey2)
        );

        // Calculate expected total delegated stake
        let expected_total_stake = stake1 + stake2 - existential_deposit * 2 - fee * 2;
        let actual_total_stake = SubtensorModule::get_alpha_share_pool(hotkey1, netuid)
            .get_value(&coldkey1)
            + SubtensorModule::get_alpha_share_pool(hotkey2, netuid).get_value(&coldkey2);

        // Total subnet stake should match the sum of delegators' stakes minus existential deposits.
        assert_abs_diff_eq!(
            actual_total_stake,
            expected_total_stake,
            epsilon = expected_total_stake / 1000
        );
    });
}

#[test]
fn test_get_total_delegated_stake_exclude_owner_stake() {
    new_test_ext(1).execute_with(|| {
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let owner_stake = DefaultMinStake::<Test>::get() * 10;
        let delegator_stake = DefaultMinStake::<Test>::get() * 10 - 1;
        let fee = DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&delegate_hotkey, &delegate_coldkey);

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
        println!(
            "Total stake for hotkey: {}",
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey)
        );
        println!(
            "Delegated stake for coldkey: {}",
            SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey)
        );

        // Check the total delegated stake (should exclude owner's stake)
        let expected_delegated_stake = delegator_stake - fee;
        let actual_delegated_stake =
            SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey);

        assert_abs_diff_eq!(
            actual_delegated_stake,
            expected_delegated_stake,
            epsilon = expected_delegated_stake / 1000
        );
    });
}

/// Test that emission is distributed correctly between one validator, one
/// vali-miner, and one miner
#[test]
fn test_mining_emission_distribution_validator_valiminer_miner() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let validator = 2;
        let validator_miner = 3;
        let miner = U256::from(4);
        let netuid: u16 = 1;
        let root_id: u16 = 0;
        let root_tempo = 9; // neet root epoch to happen before subnet tempo
        let subnet_tempo = 10;
        let stake = 100_000_000_000;

        // Add network, register hotkeys, and setup network parameters
        add_network(root_id, root_tempo, 0);
        add_network(netuid, subnet_tempo, 0);
        register_ok_neuron(netuid, validator.into(), coldkey, 0);
        register_ok_neuron(netuid, validator_miner.into(), coldkey, 1);
        register_ok_neuron(netuid, miner, coldkey, 2);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            3 * stake + ExistentialDeposit::get(),
        );
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        step_block(subnet_tempo);
        crate::SubnetOwnerCut::<Test>::set(0);
        // All stake is active
        crate::ActivityCutoff::<Test>::set(netuid, u16::MAX);
        // There are two validators and three neurons
        crate::MaxAllowedUids::<Test>::set(netuid, 3);
        SubtensorModule::set_max_allowed_validators(netuid, 2);

        // Setup stakes:
        //   Stake from validator
        //   Stake from valiminer
        crate::Stake::<Test>::set(U256::from(validator), coldkey, stake);
        crate::Stake::<Test>::set(U256::from(validator_miner), coldkey, stake);

        // Setup YUMA so that it creates emissions:
        //   Validator 1 sets weight for valiminer       |- to achieve equal incentive for both miners
        //   Valiminer sets weights for the second miner |
        //   Validator registers on root and
        //   Sets root weights
        //   Last weight update is after block at registration
        crate::Weights::<Test>::insert(netuid, 0, vec![(1, 0xFFFF)]);
        crate::Weights::<Test>::insert(netuid, 1, vec![(2, 0xFFFF)]);
        assert_ok!(SubtensorModule::do_root_register(
            RuntimeOrigin::signed(coldkey),
            U256::from(validator),
        ));
        crate::Weights::<Test>::insert(root_id, 0, vec![(0, 0xFFFF), (1, 0xFFFF)]);
        crate::BlockAtRegistration::<Test>::set(netuid, 0, 1);
        crate::BlockAtRegistration::<Test>::set(netuid, 1, 1);
        crate::LastUpdate::<Test>::set(netuid, vec![2, 2, 2]);
        crate::Kappa::<Test>::set(netuid, u16::MAX / 5);

        // Run run_coinbase until emissions are drained
        step_block(subnet_tempo * 4);

        // Verify how emission is split between keys
        //   - 50% goes to miners and 50% goes to validators
        //   - Validator gets 25% because there are two validators
        //   - Valiminer gets 25% as a validator and 25% as miner
        //   - Miner gets 25% as miner
        let validator_emission = crate::Stake::<Test>::get(U256::from(validator), coldkey) - stake;
        let valiminer_emission =
            crate::Stake::<Test>::get(U256::from(validator_miner), coldkey) - stake;
        let miner_emission = crate::Stake::<Test>::get(miner, coldkey);
        let total_emission = validator_emission + valiminer_emission + miner_emission;

        assert_eq!(validator_emission, total_emission / 4);
        assert_eq!(valiminer_emission, total_emission / 2);
        assert_eq!(miner_emission, total_emission / 4);
    });
}

// Verify staking too low amount is impossible
#[test]
fn test_staking_too_little_fails() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = 10_000;

        //add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Coldkey / hotkey 0 decreases take to 5%. This should fail as the minimum take is 9%
        assert_err!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                1
            ),
            Error::<Test>::AmountTooLow
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_add_stake_fee_goes_to_subnet_tao --exact --show-output --nocapture
#[test]
fn test_add_stake_fee_goes_to_subnet_tao() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let existential_deposit = ExistentialDeposit::get();
        let tao_to_stake = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        let subnet_tao_before = SubnetTAO::<Test>::get(netuid);

        // Add stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, tao_to_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            tao_to_stake
        ));

        // Calculate expected stake
        let expected_alpha = tao_to_stake - existential_deposit - fee;
        let actual_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let subnet_tao_after = SubnetTAO::<Test>::get(netuid);

        // Total subnet stake should match the sum of delegators' stakes minus existential deposits.
        assert_abs_diff_eq!(
            actual_alpha,
            expected_alpha,
            epsilon = expected_alpha / 1000
        );

        // Subnet TAO should have increased by the full tao_to_stake amount
        assert_abs_diff_eq!(
            subnet_tao_before + tao_to_stake,
            subnet_tao_after,
            epsilon = 10
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_stake_fee_goes_to_subnet_tao --exact --show-output --nocapture
#[test]
fn test_remove_stake_fee_goes_to_subnet_tao() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let tao_to_stake = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        let subnet_tao_before = SubnetTAO::<Test>::get(netuid);

        // Add stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, tao_to_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            tao_to_stake
        ));

        // Remove all stake
        let alpha_to_unstake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            alpha_to_unstake
        ));
        let subnet_tao_after = SubnetTAO::<Test>::get(netuid);

        // Subnet TAO should have increased by 2x fee as a result of staking + unstaking
        assert_abs_diff_eq!(
            subnet_tao_before + 2 * fee,
            subnet_tao_after,
            epsilon = alpha_to_unstake / 1000
        );

        // User balance should decrease by 2x fee as a result of staking + unstaking
        let balance_after = SubtensorModule::get_coldkey_balance(&coldkey);
        assert_abs_diff_eq!(
            balance_after + 2 * fee,
            tao_to_stake,
            epsilon = tao_to_stake / 1000
        );
    });
}

#[test]
fn test_stake_below_min_validate() {
    // Testing the signed extension validate function
    // correctly filters the `add_stake` transaction.

    new_test_ext(0).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let amount_staked = DefaultMinStake::<Test>::get() + DefaultStakingFee::<Test>::get() - 1;

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Add stake call
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            netuid,
            amount_staked,
        });

        let info: crate::DispatchInfo =
            crate::DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = crate::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&coldkey, &call.clone(), &info, 10);

        // Should fail due to insufficient stake
        assert_err!(
            result_no_stake,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::StakeAmountTooLow.into()
            ))
        );

        // Increase the stake to be equal to the minimum, but leave the balance low
        let amount_staked = DefaultMinStake::<Test>::get() + DefaultStakingFee::<Test>::get();
        let call_2 = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            netuid,
            amount_staked,
        });

        // Submit to the signed extension validate function
        let result_low_balance = extension.validate(&coldkey, &call_2.clone(), &info, 10);

        // Still doesn't pass, but with a different reason (balance too low)
        assert_err!(
            result_low_balance,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::BalanceTooLow.into()
            ))
        );

        // Increase the coldkey balance to match the minimum
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1);

        // Submit to the signed extension validate function
        let result_min_stake = extension.validate(&coldkey, &call_2.clone(), &info, 10);

        // Now the call passes
        assert_ok!(result_min_stake);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_add_stake_limit_validate --exact --show-output
#[test]
fn test_add_stake_limit_validate() {
    // Testing the signed extension validate function
    // correctly filters the `add_stake` transaction.

    new_test_ext(0).execute_with(|| {
        let hotkey = U256::from(533453);
        let coldkey = U256::from(55453);
        let amount = 900_000_000_000;

        // add network
        let netuid: u16 = add_dynamic_network(&hotkey, &coldkey);

        // Force-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve: U96F32 = U96F32::from_num(150_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 = U96F32::from_num(SubtensorModule::get_alpha_price(netuid));
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount);

        // Setup limit price so that it doesn't peak above 4x of current price
        // The amount that can be executed at this price is 450 TAO only
        let limit_price = 6_000_000_000;

        // Add stake limit call
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake_limit {
            hotkey,
            netuid,
            amount_staked: amount,
            limit_price,
            allow_partial: false,
        });

        let info: crate::DispatchInfo =
            crate::DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = crate::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&coldkey, &call.clone(), &info, 10);

        // Should fail due to slippage
        assert_err!(
            result_no_stake,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::SlippageTooHigh.into()
            ))
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_stake_limit_validate --exact --show-output
#[test]
fn test_remove_stake_limit_validate() {
    // Testing the signed extension validate function
    // correctly filters the `add_stake` transaction.

    new_test_ext(0).execute_with(|| {
        let hotkey = U256::from(533453);
        let coldkey = U256::from(55453);
        let stake_amount = 300_000_000_000;
        let unstake_amount = 150_000_000_000;

        // add network
        let netuid: u16 = add_dynamic_network(&hotkey, &coldkey);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_amount,
        );

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve: U96F32 = U96F32::from_num(150_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 = U96F32::from_num(SubtensorModule::get_alpha_price(netuid));
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Setup limit price so that it doesn't drop by more than 10% from current price
        let limit_price = 1_350_000_000;

        // Remove stake limit call
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake_limit {
            hotkey,
            netuid,
            amount_unstaked: unstake_amount,
            limit_price,
            allow_partial: false,
        });

        let info: crate::DispatchInfo =
            crate::DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = crate::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&coldkey, &call.clone(), &info, 10);

        // Should fail due to slippage
        assert_err!(
            result_no_stake,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::SlippageTooHigh.into()
            ))
        );
    });
}

#[test]
fn test_stake_overflow() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey_account_id = U256::from(435445);
        let hotkey_account_id = U256::from(54544);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let amount = 21_000_000_000_000_000; // Max TAO supply
        let fee = DefaultStakingFee::<Test>::get();
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Setup liquidity with 21M TAO values
        SubnetTAO::<Test>::insert(netuid, amount);
        SubnetAlphaIn::<Test>::insert(netuid, amount);

        // Stake and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));

        // Check if stake has increased properly (staking 1:1 to SubnetTAO results in SubnetTAO/2 alpha)
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            (amount - fee) / 2,
            epsilon = amount / 1_000_000,
        );

        // Check if total stake has increased accordingly.
        assert_abs_diff_eq!(SubtensorModule::get_total_stake(), amount, epsilon = 10);
    });
}

#[test]
fn test_stake_low_liquidity_validate() {
    // Testing the signed extension validate function
    // correctly filters the `add_stake` transaction.

    new_test_ext(0).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let amount_staked = DefaultMinStake::<Test>::get() * 10 + DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Set the liquidity at lowest possible value so that all staking requests fail
        SubnetTAO::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );
        SubnetAlphaIn::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );

        // Add stake call
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            netuid,
            amount_staked,
        });

        let info: crate::DispatchInfo =
            crate::DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = crate::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&coldkey, &call.clone(), &info, 10);

        // Should fail due to insufficient stake
        assert_err!(
            result_no_stake,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::InsufficientLiquidity.into()
            ))
        );
    });
}

#[test]
fn test_unstake_low_liquidity_validate() {
    // Testing the signed extension validate function
    // correctly filters the `add_stake` transaction.

    new_test_ext(0).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let amount_staked = DefaultMinStake::<Test>::get() * 10 + DefaultStakingFee::<Test>::get();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Simulate stake for hotkey
        SubnetTAO::<Test>::insert(netuid, u64::MAX / 1000);
        SubnetAlphaIn::<Test>::insert(netuid, u64::MAX / 1000);
        let alpha = SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, amount_staked, 0);

        // Set the liquidity at lowest possible value so that all staking requests fail
        SubnetTAO::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );
        SubnetAlphaIn::<Test>::insert(
            netuid,
            DefaultMinimumPoolLiquidity::<Test>::get().to_num::<u64>(),
        );

        // Remove stake call
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
            hotkey,
            netuid,
            amount_unstaked: alpha,
        });

        let info: crate::DispatchInfo =
            crate::DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = crate::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&coldkey, &call.clone(), &info, 10);

        // Should fail due to insufficient stake
        assert_err!(
            result_no_stake,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::InsufficientLiquidity.into()
            ))
        );
    });
}

#[test]
fn test_max_amount_add_root() {
    new_test_ext(0).execute_with(|| {
        // 0 price on root => max is 0
        assert_eq!(SubtensorModule::get_max_amount_add(0, 0), 0);

        // 0.999999... price on root => max is 0
        assert_eq!(SubtensorModule::get_max_amount_add(0, 999_999_999), 0);

        // 1.0 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(0, 1_000_000_000),
            u64::MAX
        );

        // 1.000...001 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(0, 1_000_000_001),
            u64::MAX
        );

        // 2.0 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(0, 2_000_000_000),
            u64::MAX
        );
    });
}

#[test]
fn test_max_amount_add_stable() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);

        // 0 price => max is 0
        assert_eq!(SubtensorModule::get_max_amount_add(netuid, 0), 0);

        // 0.999999... price => max is 0
        assert_eq!(SubtensorModule::get_max_amount_add(netuid, 999_999_999), 0);

        // 1.0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, 1_000_000_000),
            u64::MAX
        );

        // 1.000...001 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, 1_000_000_001),
            u64::MAX
        );

        // 2.0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, 2_000_000_000),
            u64::MAX
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_add_dynamic --exact --show-output
#[test]
fn test_max_amount_add_dynamic() {
    new_test_ext(0).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Test cases are generated with help with this limit-staking calculator:
        // https://docs.google.com/spreadsheets/d/1pfU-PVycd3I4DbJIc0GjtPohy4CbhdV6CWqgiy__jKE
        // This is for reference only; verify before use.
        //
        // CSV backup for this spreadhsheet:
        //
        // SubnetTAO,AlphaIn,initial price,limit price,max swappable
        // 100,100,=A2/B2,4,=B8*D8-A8
        //
        // tao_in, alpha_in, limit_price, expected_max_swappable
        [
            // Zero handling (no panics)
            (0, 1_000_000_000, 100, 0),
            (1_000_000_000, 0, 100, 0),
            (1_000_000_000, 1_000_000_000, 0, 0),
            // Low bounds
            (1, 1, 0, 0),
            (1, 1, 1, 0),
            (1, 1, 2, 0),
            (1, 1, 50_000_000_000, 49),
            // Basic math
            (1_000, 1_000, 2_000_000_000, 1_000),
            (1_000, 1_000, 4_000_000_000, 3_000),
            (1_000, 1_000, 16_000_000_000, 15_000),
            (
                1_000_000_000_000,
                1_000_000_000_000,
                16_000_000_000,
                15_000_000_000_000,
            ),
            // Normal range values with edge cases
            (150_000_000_000, 100_000_000_000, 0, 0),
            (150_000_000_000, 100_000_000_000, 100_000_000, 0),
            (150_000_000_000, 100_000_000_000, 500_000_000, 0),
            (150_000_000_000, 100_000_000_000, 1_499_999_999, 0),
            (150_000_000_000, 100_000_000_000, 1_500_000_000, 0),
            (150_000_000_000, 100_000_000_000, 1_500_000_001, 100),
            (
                150_000_000_000,
                100_000_000_000,
                3_000_000_000,
                150_000_000_000,
            ),
            // Miscellaneous overflows and underflows
            (150_000_000_000, 100_000_000_000, u64::MAX, u64::MAX),
            (150_000_000_000, 100_000_000_000, u64::MAX / 2, u64::MAX),
            (1_000_000, 1_000_000_000_000_000_000_u64, 1, 999_000_000),
            (1_000_000, 1_000_000_000_000_000_000_u64, 2, 1_999_000_000),
            (
                1_000_000,
                1_000_000_000_000_000_000_u64,
                10_000,
                9_999_999_000_000,
            ),
            (
                1_000_000,
                1_000_000_000_000_000_000_u64,
                100_000,
                99_999_999_000_000,
            ),
            (
                1_000_000,
                1_000_000_000_000_000_000_u64,
                1_000_000,
                999_999_999_000_000,
            ),
            (
                1_000_000,
                1_000_000_000_000_000_000_u64,
                1_000_000_000,
                999_999_999_999_000_000,
            ),
            (
                21_000_000_000_000_000,
                10_000_000,
                4_200_000_000_000_000_000,
                21_000_000_000_000_000,
            ),
            (
                21_000_000_000_000_000,
                1_000_000_000_000_000_000_u64,
                u64::MAX,
                u64::MAX,
            ),
            (
                21_000_000_000_000_000,
                1_000_000_000_000_000_000_u64,
                42_000_000,
                21_000_000_000_000_000,
            ),
        ]
        .iter()
        .for_each(|&(tao_in, alpha_in, limit_price, expected_max_swappable)| {
            // Forse-set alpha in and tao reserve to achieve relative price of subnets
            SubnetTAO::<Test>::insert(netuid, tao_in);
            SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

            if alpha_in != 0 {
                let expected_price = I96F32::from_num(tao_in) / I96F32::from_num(alpha_in);
                assert_eq!(SubtensorModule::get_alpha_price(netuid), expected_price);
            }

            assert_eq!(
                SubtensorModule::get_max_amount_add(netuid, limit_price),
                expected_max_swappable,
            );
        });
    });
}

#[test]
fn test_max_amount_remove_root() {
    new_test_ext(0).execute_with(|| {
        // 0 price on root => max is u64::MAX
        assert_eq!(SubtensorModule::get_max_amount_remove(0, 0), u64::MAX);

        // 0.5 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(0, 500_000_000),
            u64::MAX
        );

        // 0.999999... price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(0, 999_999_999),
            u64::MAX
        );

        // 1.0 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(0, 1_000_000_000),
            u64::MAX
        );

        // 1.000...001 price on root => max is 0
        assert_eq!(SubtensorModule::get_max_amount_remove(0, 1_000_000_001), 0);

        // 2.0 price on root => max is 0
        assert_eq!(SubtensorModule::get_max_amount_remove(0, 2_000_000_000), 0);
    });
}

#[test]
fn test_max_amount_remove_stable() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);

        // 0 price => max is u64::MAX
        assert_eq!(SubtensorModule::get_max_amount_remove(netuid, 0), u64::MAX);

        // 0.999999... price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, 999_999_999),
            u64::MAX
        );

        // 1.0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, 1_000_000_000),
            u64::MAX
        );

        // 1.000...001 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, 1_000_000_001),
            0
        );

        // 2.0 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, 2_000_000_000),
            0
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_remove_dynamic --exact --show-output
#[test]
fn test_max_amount_remove_dynamic() {
    new_test_ext(0).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Test cases are generated with help with this limit-staking calculator:
        // https://docs.google.com/spreadsheets/d/1pfU-PVycd3I4DbJIc0GjtPohy4CbhdV6CWqgiy__jKE
        // This is for reference only; verify before use.
        //
        // CSV backup for this spreadhsheet:
        //
        // SubnetTAO,AlphaIn,initial price,limit price,max swappable
        // 100,100,=A2/B2,4,=A2/D2-B2
        //
        // tao_in, alpha_in, limit_price, expected_max_swappable
        [
            // Zero handling (no panics)
            (0, 1_000_000_000, 100, 0),
            (1_000_000_000, 0, 100, 0),
            (1_000_000_000, 1_000_000_000, 0, u64::MAX),
            // Low bounds
            (1, 1, 0, u64::MAX),
            (1, 1, 1, 999_999_999),
            (1, 1, 2, 499_999_999),
            (1, 1, 250_000_000, 3),
            // Basic math
            (1_000, 1_000, 250_000_000, 3_000),
            (1_000, 1_000, 62_500_000, 15_000),
            (
                1_000_000_000_000,
                1_000_000_000_000,
                62_500_000,
                15_000_000_000_000,
            ),
            // Normal range values with edge cases
            (200_000_000_000, 100_000_000_000, 0, u64::MAX),
            (
                200_000_000_000,
                100_000_000_000,
                1_000_000_000,
                100_000_000_000,
            ),
            (
                200_000_000_000,
                100_000_000_000,
                500_000_000,
                300_000_000_000,
            ),
            (200_000_000_000, 100_000_000_000, 2_000_000_000, 0),
            (200_000_000_000, 100_000_000_000, 2_000_000_001, 0),
            (200_000_000_000, 100_000_000_000, 1_999_999_999, 50),
            (200_000_000_000, 100_000_000_000, 1_999_999_990, 500),
            // Miscellaneous overflows and underflows
            (2_000_000_000_000, 100_000_000_000, u64::MAX, 0),
            (200_000_000_000, 100_000_000_000, u64::MAX / 2, 0),
            (1_000_000, 1_000_000_000_000_000_000_u64, 1, 0),
            (1_000_000, 1_000_000_000_000_000_000_u64, 10, 0),
            (1_000_000, 1_000_000_000_000_000_000_u64, 100, 0),
            (1_000_000, 1_000_000_000_000_000_000_u64, 1_000, 0),
            (1_000_000, 1_000_000_000_000_000_000_u64, u64::MAX, 0),
            (
                21_000_000_000_000_000,
                1_000_000,
                21_000_000_000_000_000,
                999_000_000,
            ),
            (21_000_000_000_000_000, 1_000_000, u64::MAX, 138_412),
            (
                21_000_000_000_000_000,
                1_000_000_000_000_000_000_u64,
                u64::MAX,
                0,
            ),
            (
                21_000_000_000_000_000,
                1_000_000_000_000_000_000_u64,
                20_000_000,
                50_000_000_000_000_000,
            ),
        ]
        .iter()
        .for_each(|&(tao_in, alpha_in, limit_price, expected_max_swappable)| {
            // Forse-set alpha in and tao reserve to achieve relative price of subnets
            SubnetTAO::<Test>::insert(netuid, tao_in);
            SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

            if alpha_in != 0 {
                let expected_price = I96F32::from_num(tao_in) / I96F32::from_num(alpha_in);
                assert_eq!(SubtensorModule::get_alpha_price(netuid), expected_price);
            }

            assert_eq!(
                SubtensorModule::get_max_amount_remove(netuid, limit_price),
                expected_max_swappable,
            );
        });
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_root_root --exact --show-output
#[test]
fn test_max_amount_move_root_root() {
    new_test_ext(0).execute_with(|| {
        // 0 price on (root, root) exchange => max is u64::MAX
        assert_eq!(SubtensorModule::get_max_amount_move(0, 0, 0), u64::MAX);

        // 0.5 price on (root, root) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, 0, 500_000_000),
            u64::MAX
        );

        // 0.999999... price on (root, root) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, 0, 999_999_999),
            u64::MAX
        );

        // 1.0 price on (root, root) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, 0, 1_000_000_000),
            u64::MAX
        );

        // 1.000...001 price on (root, root) => max is 0
        assert_eq!(SubtensorModule::get_max_amount_move(0, 0, 1_000_000_001), 0);

        // 2.0 price on (root, root) => max is 0
        assert_eq!(SubtensorModule::get_max_amount_move(0, 0, 2_000_000_000), 0);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_root_stable --exact --show-output
#[test]
fn test_max_amount_move_root_stable() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid, 1, 0);

        // 0 price on (root, stable) exchange => max is u64::MAX
        assert_eq!(SubtensorModule::get_max_amount_move(0, netuid, 0), u64::MAX);

        // 0.5 price on (root, stable) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, netuid, 500_000_000),
            u64::MAX
        );

        // 0.999999... price on (root, stable) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, netuid, 999_999_999),
            u64::MAX
        );

        // 1.0 price on (root, stable) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, netuid, 1_000_000_000),
            u64::MAX
        );

        // 1.000...001 price on (root, stable) => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, netuid, 1_000_000_001),
            0
        );

        // 2.0 price on (root, stable) => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(0, netuid, 2_000_000_000),
            0
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_stable_dynamic --exact --show-output
#[test]
fn test_max_amount_move_stable_dynamic() {
    new_test_ext(0).execute_with(|| {
        // Add stable subnet
        let stable_netuid: u16 = 1;
        add_network(stable_netuid, 1, 0);

        // Add dynamic subnet
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let dynamic_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Forse-set alpha in and tao reserve to make price equal 0.5
        let tao_reserve: U96F32 = U96F32::from_num(50_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(dynamic_netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(dynamic_netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 =
            U96F32::from_num(SubtensorModule::get_alpha_price(dynamic_netuid));
        assert_eq!(current_price, U96F32::from_num(0.5));

        // The tests below just mimic the add_stake_limit tests for reverted price

        // 0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, 0),
            u64::MAX
        );

        // 2.0 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, 2_000_000_000),
            0
        );

        // 3.0 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, 3_000_000_000),
            0
        );

        // 2x price => max is 1x TAO
        assert_abs_diff_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, 1_000_000_000),
            50_000_000_000,
            epsilon = 10_000,
        );

        // Precision test:
        // 1.99999..9000 price => max > 0
        assert!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, 1_999_999_000) > 0
        );

        // Max price doesn't panic and returns something meaningful
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, u64::MAX),
            0
        );
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, u64::MAX - 1),
            0
        );
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, u64::MAX / 2),
            0
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_dynamic_stable --exact --show-output
#[test]
fn test_max_amount_move_dynamic_stable() {
    new_test_ext(0).execute_with(|| {
        // Add stable subnet
        let stable_netuid: u16 = 1;
        add_network(stable_netuid, 1, 0);

        // Add dynamic subnet
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let dynamic_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve: U96F32 = U96F32::from_num(150_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(dynamic_netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(dynamic_netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 =
            U96F32::from_num(SubtensorModule::get_alpha_price(dynamic_netuid));
        assert_eq!(current_price, U96F32::from_num(1.5));

        // The tests below just mimic the remove_stake_limit tests

        // 0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 0),
            u64::MAX
        );

        // Low price values don't blow things up
        assert!(SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 1) > 0);
        assert!(SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 2) > 0);
        assert!(SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 3) > 0);

        // 1.5000...1 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 1_500_000_001),
            0
        );

        // 1.5 price => max is 0 because of non-zero slippage
        assert_abs_diff_eq!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 1_500_000_000),
            0,
            epsilon = 10_000
        );

        // 1/2 price => max is 1x Alpha
        assert_abs_diff_eq!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 750_000_000),
            100_000_000_000,
            epsilon = 10_000,
        );

        // Precision test:
        // 1.499999.. price => max > 0
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 1_499_999_999) > 0
        );

        // Max price doesn't panic and returns something meaningful
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, u64::MAX)
                < 21_000_000_000_000_000
        );
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, u64::MAX - 1)
                < 21_000_000_000_000_000
        );
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, u64::MAX / 2)
                < 21_000_000_000_000_000
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_dynamic_dynamic --exact --show-output
#[test]
fn test_max_amount_move_dynamic_dynamic() {
    new_test_ext(0).execute_with(|| {
        // Add two dynamic subnets
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Test cases are generated with help with this limit-staking calculator:
        // https://docs.google.com/spreadsheets/d/1pfU-PVycd3I4DbJIc0GjtPohy4CbhdV6CWqgiy__jKE
        // This is for reference only; verify before use.
        //
        // CSV backup for this spreadhsheet:
        //
        // SubnetTAO 1,AlphaIn 1,SubnetTAO 2,AlphaIn 2,,initial price,limit price,max swappable
        // 150,100,100,100,,=(A2/B2)/(C2/D2),0.1,=(D2*A2-B2*C2*G2)/(G2*(A2+C2))
        //
        // tao_in_1, alpha_in_1, tao_in_2, alpha_in_2, limit_price, expected_max_swappable, precision
        [
            // Zero handling (no panics)
            (0, 1_000_000_000, 1_000_000_000, 1_000_000_000, 100, 0, 1),
            (1_000_000_000, 0, 1_000_000_000, 1_000_000_000, 100, 0, 1),
            (1_000_000_000, 1_000_000_000, 0, 1_000_000_000, 100, 0, 1),
            (1_000_000_000, 1_000_000_000, 1_000_000_000, 0, 100, 0, 1),
            // Low bounds
            (1, 1, 1, 1, 0, u64::MAX, 1),
            (1, 1, 1, 1, 1, 500_000_000, 1),
            (1, 1, 1, 1, 2, 250_000_000, 1),
            (1, 1, 1, 1, 3, 166_666_666, 1),
            (1, 1, 1, 1, 4, 125_000_000, 1),
            (1, 1, 1, 1, 1_000, 500_000, 1),
            // Basic math
            (1_000, 1_000, 1_000, 1_000, 500_000_000, 500, 1),
            (1_000, 1_000, 1_000, 1_000, 100_000_000, 4_500, 1),
            // Normal range values edge cases
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000,
                560_000_000_000,
                1_000_000,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                500_000_000,
                80_000_000_000,
                1_000_000,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                750_000_000,
                40_000_000_000,
                1_000_000,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                1_000_000_000,
                20_000_000_000,
                1_000,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                1_250_000_000,
                8_000_000_000,
                1_000,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                1_499_999_999,
                27,
                1,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                1_500_000_000,
                0,
                1,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                1_500_000_001,
                0,
                1,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                1_500_001_000,
                0,
                1,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                2_000_000_000,
                0,
                1,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                u64::MAX,
                0,
                1,
            ),
            (
                100_000_000_000,
                200_000_000_000,
                300_000_000_000,
                400_000_000_000,
                500_000_000,
                50_000_000_000,
                1_000,
            ),
            // Miscellaneous overflows
            (
                1_000_000_000,
                1_000_000_000,
                1_000_000_000,
                1_000_000_000,
                1,
                499_999_999_500_000_000,
                100_000_000,
            ),
            (
                1_000_000,
                1_000_000,
                21_000_000_000_000_000,
                1_000_000_000_000_000_000_u64,
                1,
                48_000_000_000_000_000,
                1_000_000_000_000_000,
            ),
            (
                150_000_000_000,
                100_000_000_000,
                100_000_000_000,
                100_000_000_000,
                u64::MAX,
                0,
                1,
            ),
            (
                1_000_000,
                1_000_000,
                21_000_000_000_000_000,
                1_000_000_000_000_000_000_u64,
                u64::MAX,
                0,
                1,
            ),
        ]
        .iter()
        .for_each(
            |&(
                tao_in_1,
                alpha_in_1,
                tao_in_2,
                alpha_in_2,
                limit_price,
                expected_max_swappable,
                precision,
            )| {
                // Forse-set alpha in and tao reserve to achieve relative price of subnets
                SubnetTAO::<Test>::insert(origin_netuid, tao_in_1);
                SubnetAlphaIn::<Test>::insert(origin_netuid, alpha_in_1);
                SubnetTAO::<Test>::insert(destination_netuid, tao_in_2);
                SubnetAlphaIn::<Test>::insert(destination_netuid, alpha_in_2);

                if (alpha_in_1 != 0) && (alpha_in_2 != 0) {
                    let origin_price = I96F32::from_num(tao_in_1) / I96F32::from_num(alpha_in_1);
                    let dest_price = I96F32::from_num(tao_in_2) / I96F32::from_num(alpha_in_2);
                    if dest_price != 0 {
                        let expected_price = origin_price / dest_price;
                        assert_eq!(
                            SubtensorModule::get_alpha_price(origin_netuid)
                                / SubtensorModule::get_alpha_price(destination_netuid),
                            expected_price
                        );
                    }
                }

                assert_abs_diff_eq!(
                    SubtensorModule::get_max_amount_move(
                        origin_netuid,
                        destination_netuid,
                        limit_price
                    ),
                    expected_max_swappable,
                    epsilon = precision
                );
            },
        );
    });
}

#[test]
fn test_add_stake_limit_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = 900_000_000_000; // over the maximum
        let fee = DefaultStakingFee::<Test>::get();

        // add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve: U96F32 = U96F32::from_num(150_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 = U96F32::from_num(SubtensorModule::get_alpha_price(netuid));
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Setup limit price so that it doesn't peak above 4x of current price
        // The amount that can be executed at this price is 450 TAO only
        // Alpha produced will be equal to 75 = 450*100/(450+150)
        let limit_price = 6_000_000_000;
        let expected_executed_stake = 75_000_000_000;

        // Add stake with slippage safety and check if the result is ok
        assert_ok!(SubtensorModule::add_stake_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount,
            limit_price,
            true
        ));

        // Check if stake has increased only by 75 Alpha
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid
            ),
            expected_executed_stake - fee,
            epsilon = expected_executed_stake / 1000,
        );

        // Check that 450 TAO balance still remains free on coldkey
        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            450_000_000_000,
            epsilon = 10_000
        );

        // Check that price has updated to ~24 = (150+450) / (100 - 75)
        let exp_price = U96F32::from_num(24.0);
        let current_price: U96F32 = U96F32::from_num(SubtensorModule::get_alpha_price(netuid));
        assert!(exp_price.saturating_sub(current_price) < 0.0001);
        assert!(current_price.saturating_sub(exp_price) < 0.0001);
    });
}

#[test]
fn test_add_stake_limit_fill_or_kill() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = 900_000_000_000; // over the maximum

        // add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Force-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve: U96F32 = U96F32::from_num(150_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 = U96F32::from_num(SubtensorModule::get_alpha_price(netuid));
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Setup limit price so that it doesn't peak above 4x of current price
        // The amount that can be executed at this price is 450 TAO only
        // Alpha produced will be equal to 25 = 100 - 450*100/(150+450)
        let limit_price = 6_000_000_000;

        // Add stake with slippage safety and check if it fails
        assert_noop!(
            SubtensorModule::add_stake_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount,
                limit_price,
                false
            ),
            Error::<Test>::SlippageTooHigh
        );

        // Lower the amount and it should succeed now
        let amount_ok = 450_000_000_000; // fits the maximum
        assert_ok!(SubtensorModule::add_stake_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount_ok,
            limit_price,
            false
        ));
    });
}

#[test]
fn test_remove_stake_limit_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let stake_amount = 300_000_000_000;
        let unstake_amount = 150_000_000_000;
        let fee = DefaultStakingFee::<Test>::get();

        // add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            stake_amount,
        );
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve: U96F32 = U96F32::from_num(150_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 = U96F32::from_num(SubtensorModule::get_alpha_price(netuid));
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Setup limit price so resulting average price doesn't drop by more than 10% from current price
        let limit_price = 1_350_000_000;

        // Alpha unstaked = 150 / 1.35 - 100 ~ 11.1
        let expected_alpha_reduction = 11_111_111_111;

        // Remove stake with slippage safety
        assert_ok!(SubtensorModule::remove_stake_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            unstake_amount,
            limit_price,
            true
        ));

        // Check if stake has decreased only by
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid
            ),
            alpha_before - expected_alpha_reduction - fee,
            epsilon = expected_alpha_reduction / 1_000,
        );
    });
}

#[test]
fn test_remove_stake_limit_fill_or_kill() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let stake_amount = 300_000_000_000;
        let unstake_amount = 150_000_000_000;

        // add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            stake_amount,
        );

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve: U96F32 = U96F32::from_num(150_000_000_000_u64);
        let alpha_in: U96F32 = U96F32::from_num(100_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao_reserve.to_num::<u64>());
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in.to_num::<u64>());
        let current_price: U96F32 = U96F32::from_num(SubtensorModule::get_alpha_price(netuid));
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Setup limit price so that it doesn't drop by more than 10% from current price
        let limit_price = 1_350_000_000;

        // Remove stake with slippage safety - fails
        assert_noop!(
            SubtensorModule::remove_stake_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                unstake_amount,
                limit_price,
                false
            ),
            Error::<Test>::SlippageTooHigh
        );

        // Lower the amount: Should succeed
        assert_ok!(SubtensorModule::remove_stake_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            unstake_amount / 100,
            limit_price,
            false
        ),);
    });
}

// #[test]
// fn test_add_stake_specific() {
//     new_test_ext(1).execute_with(|| {
//         let sn_owner_coldkey = U256::from(55453);

//         let hotkey_account_id = U256::from(533453);
//         let coldkey_account_id = U256::from(55454);
//         let hotkey_owner_account_id = U256::from(533454);

//         let existing_shares: U64F64 =
//             U64F64::from_num(161_986_254).saturating_div(U64F64::from_num(u64::MAX));
//         let existing_stake = 36_711_495_953;
//         let amount_added = 1_274_280_132;

//         //add network
//         let netuid: u16 = add_dynamic_network(&sn_owner_coldkey, &sn_owner_coldkey);

//         // Register hotkey on netuid
//         register_ok_neuron(netuid, hotkey_account_id, hotkey_owner_account_id, 0);
//         // Check we have zero staked
//         assert_eq!(
//             SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
//             0
//         );

//         // Set a hotkey pool for the hotkey
//         let mut hotkey_pool = SubtensorModule::get_alpha_share_pool(hotkey_account_id, netuid);
//         hotkey_pool.update_value_for_one(&hotkey_owner_account_id, 1234); // Doesn't matter, will be overridden

//         // Adjust the total hotkey stake and shares to match the existing values
//         TotalHotkeyShares::<Test>::insert(hotkey_account_id, netuid, existing_shares);
//         TotalHotkeyAlpha::<Test>::insert(hotkey_account_id, netuid, existing_stake);

//         // Make the hotkey a delegate
//         Delegates::<Test>::insert(hotkey_account_id, 0);

//         // Add stake as new hotkey
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &hotkey_account_id,
//             &coldkey_account_id,
//             netuid,
//             amount_added,
//         );

//         // Check the stake and shares are correct
//         assert!(Alpha::<Test>::get((&hotkey_account_id, &coldkey_account_id, netuid)) > 0);
//         assert_eq!(
//             TotalHotkeyAlpha::<Test>::get(hotkey_account_id, netuid),
//             amount_added + existing_stake
//         );
//     });
// }

// #[test]
// // RUST_LOG=info cargo test --package pallet-subtensor --lib -- tests::staking::test_add_stake_specific_stake_into_subnet --exact --show-output
// fn test_add_stake_specific_stake_into_subnet() {
//     new_test_ext(1).execute_with(|| {
//         let sn_owner_coldkey = U256::from(55453);

//         let hotkey_account_id = U256::from(533453);
//         let coldkey_account_id = U256::from(55454);
//         let hotkey_owner_account_id = U256::from(533454);

//         let existing_shares: U64F64 =
//             U64F64::from_num(161_986_254).saturating_div(U64F64::from_num(u64::MAX));
//         let existing_stake = 36_711_495_953;

//         let tao_in = 2_409_892_148_947;
//         let alpha_in = 15_358_708_513_716;

//         let tao_staked = 200_000_000;
//         let fee = DefaultStakingFee::<Test>::get();

//         //add network
//         let netuid: u16 = add_dynamic_network(&sn_owner_coldkey, &sn_owner_coldkey);

//         // Register hotkey on netuid
//         register_ok_neuron(netuid, hotkey_account_id, hotkey_owner_account_id, 0);
//         // Check we have zero staked
//         assert_eq!(
//             SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
//             0
//         );

//         // Set a hotkey pool for the hotkey
//         let mut hotkey_pool = SubtensorModule::get_alpha_share_pool(hotkey_account_id, netuid);
//         hotkey_pool.update_value_for_one(&hotkey_owner_account_id, 1234); // Doesn't matter, will be overridden

//         // Adjust the total hotkey stake and shares to match the existing values
//         TotalHotkeyShares::<Test>::insert(hotkey_account_id, netuid, existing_shares);
//         TotalHotkeyAlpha::<Test>::insert(hotkey_account_id, netuid, existing_stake);

//         // Make the hotkey a delegate
//         Delegates::<Test>::insert(hotkey_account_id, 0);

//         // Setup Subnet pool
//         SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
//         SubnetTAO::<Test>::insert(netuid, tao_in);

//         // Add stake as new hotkey
//         SubtensorModule::stake_into_subnet(
//             &hotkey_account_id,
//             &coldkey_account_id,
//             netuid,
//             tao_staked,
//             fee,
//         );

//         // Check the stake and shares are correct
//         assert!(Alpha::<Test>::get((&hotkey_account_id, &coldkey_account_id, netuid)) > 0);
//         log::info!(
//             "Alpha: {}",
//             Alpha::<Test>::get((&hotkey_account_id, &coldkey_account_id, netuid))
//         );
//         log::info!(
//             "TotalHotkeyAlpha: {}",
//             TotalHotkeyAlpha::<Test>::get(hotkey_account_id, netuid)
//         );
//     });
// }

#[test]
// RUST_LOG=info cargo test --package pallet-subtensor --lib -- tests::staking::test_add_stake_specific_stake_into_subnet_fail --exact --show-output
fn test_add_stake_specific_stake_into_subnet_fail() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_coldkey = U256::from(55453);

        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55454);
        let hotkey_owner_account_id = U256::from(533454);

        let existing_shares: U64F64 =
            U64F64::from_num(161_986_254).saturating_div(U64F64::from_num(u64::MAX));
        let existing_stake = 36_711_495_953;

        let tao_in = 2_409_892_148_947;
        let alpha_in = 15_358_708_513_716;

        let tao_staked = 200_000_000;

        //add network
        let netuid: u16 = add_dynamic_network(&sn_owner_coldkey, &sn_owner_coldkey);

        // Register hotkey on netuid
        register_ok_neuron(netuid, hotkey_account_id, hotkey_owner_account_id, 0);
        // Check we have zero staked
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );

        // Set a hotkey pool for the hotkey
        let mut hotkey_pool = SubtensorModule::get_alpha_share_pool(hotkey_account_id, netuid);
        hotkey_pool.update_value_for_one(&hotkey_owner_account_id, 1234); // Doesn't matter, will be overridden

        // Adjust the total hotkey stake and shares to match the existing values
        TotalHotkeyShares::<Test>::insert(hotkey_account_id, netuid, existing_shares);
        TotalHotkeyAlpha::<Test>::insert(hotkey_account_id, netuid, existing_stake);

        // Make the hotkey a delegate
        Delegates::<Test>::insert(hotkey_account_id, 0);

        // Setup Subnet pool
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        SubnetTAO::<Test>::insert(netuid, tao_in);

        // Give TAO balance to coldkey
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            tao_staked + 1_000_000_000,
        );

        // Add stake as new hotkey
        assert_noop!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                tao_staked,
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
// RUST_LOG=info cargo test --package pallet-subtensor --lib -- tests::staking::test_move_stake_specific_stake_into_subnet_fail --exact --show-output
fn test_move_stake_specific_stake_into_subnet_fail() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_coldkey = U256::from(55453);

        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55454);
        let hotkey_owner_account_id = U256::from(533454);

        let existing_shares: U64F64 =
            U64F64::from_num(161_986_254).saturating_div(U64F64::from_num(u64::MAX));
        let existing_stake = 36_711_495_953;

        let tao_in = 2_409_892_148_947;
        let alpha_in = 15_358_708_513_716;

        let tao_staked = 200_000_000;

        //add network
        let netuid: u16 = add_dynamic_network(&sn_owner_coldkey, &sn_owner_coldkey);

        let origin_netuid: u16 = add_dynamic_network(&sn_owner_coldkey, &sn_owner_coldkey);

        // Register hotkey on netuid
        register_ok_neuron(netuid, hotkey_account_id, hotkey_owner_account_id, 0);
        // Register hotkey on origin netuid
        register_ok_neuron(origin_netuid, hotkey_account_id, hotkey_owner_account_id, 0);

        // Check we have zero staked
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            0
        );

        // Set a hotkey pool for the hotkey on destination subnet
        let mut hotkey_pool = SubtensorModule::get_alpha_share_pool(hotkey_account_id, netuid);
        hotkey_pool.update_value_for_one(&hotkey_owner_account_id, 1234); // Doesn't matter, will be overridden

        // Adjust the total hotkey stake and shares to match the existing values
        TotalHotkeyShares::<Test>::insert(hotkey_account_id, netuid, existing_shares);
        TotalHotkeyAlpha::<Test>::insert(hotkey_account_id, netuid, existing_stake);

        // Make the hotkey a delegate
        Delegates::<Test>::insert(hotkey_account_id, 0);

        // Setup Subnet pool
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        SubnetTAO::<Test>::insert(netuid, tao_in);

        // Give TAO balance to coldkey
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            tao_staked + 1_000_000_000,
        );

        // Setup Subnet pool for origin netuid
        SubnetAlphaIn::<Test>::insert(origin_netuid, alpha_in + 10_000_000);
        SubnetTAO::<Test>::insert(origin_netuid, tao_in + 10_000_000);

        // Add stake as new hotkey
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            origin_netuid,
            tao_staked,
        ),);
        let alpha_to_move = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            origin_netuid,
        );

        // Move stake to destination subnet
        assert_noop!(
            SubtensorModule::move_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                hotkey_account_id,
                origin_netuid,
                netuid,
                alpha_to_move,
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}
