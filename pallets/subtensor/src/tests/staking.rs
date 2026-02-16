#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use approx::assert_abs_diff_eq;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use frame_system::RawOrigin;
use safe_math::FixedExt;
use sp_core::{Get, H256, U256};
// use sp_runtime::traits::Dispatchable;
use substrate_fixed::traits::FromFixed;
use substrate_fixed::types::{I96F32, I110F18, U64F64, U96F32};
use subtensor_runtime_common::{
    AlphaCurrency, Currency as CurrencyT, NetUid, NetUidStorageIndex, TaoCurrency,
};
use subtensor_swap_interface::{Order, SwapHandler};

use super::mock;
use super::mock::*;
use crate::*;

/***********************************************************
    staking::add_stake() tests
************************************************************/

#[test]
fn test_add_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_staked = TaoCurrency::from(5000);
        let netuid = NetUid::from(1);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            netuid,
            amount_staked,
        });
        let di = call.get_dispatch_info();
        assert_eq!(di.extension_weight, frame_support::weights::Weight::zero(),);
        assert_eq!(di.class, DispatchClass::Normal,);
        assert_eq!(di.pays_fee, Pays::Yes,);
    });
}
#[test]
fn test_add_stake_ok_no_emission() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;

        //add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        mock::setup_reserves(
            netuid,
            (amount * 1_000_000).into(),
            (amount * 10_000_000).into(),
        );

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Check we have zero staked before transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );

        // Also total stake should be equal to the network initial lock
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock()
        );

        // Transfer to hotkey account, and check if the result is ok
        let (alpha_staked, fee) = mock::swap_tao_to_alpha(netuid, amount.into());
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into()
        ));

        let (tao_expected, _) = mock::swap_alpha_to_tao(netuid, alpha_staked);
        let approx_fee = <Test as pallet::Config>::SwapInterface::approx_fee_amount(
            netuid.into(),
            TaoCurrency::from(amount),
        );

        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            tao_expected + approx_fee, // swap returns value after fee, so we need to compensate it
            epsilon = 10000.into(),
        );

        // Check if stake has increased
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            (amount - fee).into(),
            epsilon = 10000.into()
        );

        // Check if balance has decreased
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 1);

        // Check if total stake has increased accordingly.
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock() + amount.into()
        );
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
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);
        Tempo::<Test>::insert(netuid, 13);

        // Register neuron, this will set a self weight
        SubtensorModule::set_max_registrations_per_block(netuid, 3);
        SubtensorModule::set_max_allowed_uids(1.into(), 5);

        register_ok_neuron(netuid, neuron_src_hotkey_id, coldkey_account_id, 192213123);
        register_ok_neuron(netuid, neuron_dest_hotkey_id, coldkey_account_id, 12323);

        // Add some stake to the hotkey account, so we can test for emission before the transfer takes place
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &neuron_src_hotkey_id,
            &coldkey_account_id,
            netuid,
            initial_stake.into(),
        );

        // Check if the initial stake has arrived
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id),
            initial_stake.into(),
            epsilon = 2.into()
        );

        // Check if all three neurons are registered
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 3);

        // Run a couple of blocks to check if emission works
        run_to_block(2);

        // Check if the stake is equal to the inital stake + transfer
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_src_hotkey_id),
            initial_stake.into(),
            epsilon = 2.into()
        );

        // Check if the stake is equal to the inital stake + transfer
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&neuron_dest_hotkey_id),
            TaoCurrency::ZERO
        );
    });
}

#[test]
fn test_add_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(654); // bogus
        let amount = 20000; // Not used
        let netuid = NetUid::from(1);

        assert_err!(
            SubtensorModule::add_stake(
                RawOrigin::None.into(),
                hotkey_account_id,
                netuid,
                amount.into()
            ),
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
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);
        assert_err!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into()
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
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);
        let stake = DefaultMinStake::<Test>::get() * 10.into();

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&other_cold_key, stake.into());

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
        let stake = DefaultMinStake::<Test>::get() * 10.into();
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);

        // Lets try to stake with 0 balance in cold key account
        assert!(SubtensorModule::get_coldkey_balance(&coldkey_id) < stake.to_u64());
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
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give it some $$$ in his coldkey balance
        let initial_balance = 10000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, TaoCurrency::ZERO);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), TaoCurrency::ZERO);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000.into()
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000.into());

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 10000.into());

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
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give it some $$$ in his coldkey balance
        let initial_balance = 10000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Check we have zero staked before transfer
        let initial_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(initial_stake, TaoCurrency::ZERO);

        // Check total balance is equal to initial balance
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, initial_balance);

        // Check total issuance is equal to initial balance
        let initial_total_issuance = Balances::total_issuance();
        assert_eq!(initial_total_issuance, initial_balance);

        // Also total stake should be zero
        assert_eq!(SubtensorModule::get_total_stake(), TaoCurrency::ZERO);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            10000.into()
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 10000.into());

        // Check if free balance has decreased
        let new_free_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_eq!(new_free_balance, 0);

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 10000.into());

        // Check if total issuance has remained the same. (no fee, includes reserved/locked balance)
        let total_issuance = Balances::total_issuance();
        assert_eq!(total_issuance, initial_total_issuance);
    });
}

#[test]
fn test_remove_stake_dispatch_info_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(0);
        let amount_unstaked = AlphaCurrency::from(5000);
        let netuid = NetUid::from(1);
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
            hotkey,
            netuid,
            amount_unstaked,
        });
        assert_eq!(
            call.get_dispatch_info(),
            DispatchInfo {
                call_weight: frame_support::weights::Weight::from_parts(1_671_800_000, 0)
                    .add_proof_size(0),
                extension_weight: frame_support::weights::Weight::zero(),
                class: DispatchClass::Normal,
                pays_fee: Pays::Yes
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
        let amount = DefaultMinStake::<Test>::get() * 10.into();
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            amount.to_u64().into(),
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            amount,
            epsilon = amount / 1000.into()
        );

        // Add subnet TAO for the equivalent amount added at price
        let (amount_tao, fee) = mock::swap_alpha_to_tao(netuid, amount.to_u64().into());
        SubnetTAO::<Test>::mutate(netuid, |v| *v += amount_tao + fee.into());
        TotalStake::<Test>::mutate(|v| *v += amount_tao + fee.into());

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.to_u64().into()
        ));

        // we do not expect the exact amount due to slippage
        assert!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id)
                > amount.to_u64() / 10 * 9 - fee
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO,
            epsilon = 20000.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock() + fee.into(),
            epsilon = SubtensorModule::get_total_stake() / 100_000.into()
        );
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
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            amount.into(),
        );

        // Do the magic
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                AlphaCurrency::ZERO
            ),
            Error::<Test>::AmountTooLow
        );
    });
}

#[test]
fn test_remove_stake_below_min_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        let min_stake = DefaultMinStake::<Test>::get();
        let amount = AlphaCurrency::from(min_stake.to_u64() / 2);

        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            amount,
        );

        // Unstake less than full stake - errors
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount - 1.into()
            ),
            Error::<Test>::AmountTooLow
        );

        // Unstaking full stake - works
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount
        ));
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid,
            )
            .is_zero()
        );
    });
}

#[test]
fn test_add_stake_partial_below_min_stake_fails() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let coldkey_account_id = U256::from(4343);
        let hotkey_account_id = U256::from(4968585);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Stake TAO amount is above min stake
        let min_stake = DefaultMinStake::<Test>::get();
        let amount = min_stake.to_u64() * 2;
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            amount + ExistentialDeposit::get(),
        );

        // Setup reserves so that price is 1.0 and init swap
        mock::setup_reserves(netuid, (amount * 10).into(), (amount * 10).into());

        // Force the swap to initialize
        <Test as pallet::Config>::SwapInterface::init_swap(netuid, None);

        // Get the current price (should be 1.0)
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        assert_eq!(current_price.to_num::<f64>(), 1.0);

        // Set limit price close to 1.0 so that we hit the limit on adding and the amount is lower than min stake
        let limit_price = (1.0001 * 1_000_000_000_f64) as u64;

        // Add stake with partial flag on
        assert_err!(
            SubtensorModule::add_stake_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into(),
                limit_price.into(),
                true
            ),
            Error::<Test>::AmountTooLow
        );

        let new_current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        assert_eq!(new_current_price.to_num::<f64>(), 1.0);
    });
}

#[test]
fn test_remove_stake_err_signature() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(4968585);
        let amount = AlphaCurrency::from(10000); // Amount to be removed
        let netuid = NetUid::from(1);

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
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &other_cold_key,
            netuid,
            amount.into(),
        );

        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(other_cold_key),
            hotkey_id,
            netuid,
            amount.into(),
        ));
    });
}

#[test]
fn test_remove_stake_no_enough_stake() {
    new_test_ext(1).execute_with(|| {
        let coldkey_id = U256::from(544);
        let hotkey_id = U256::from(54544);
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        let netuid = add_dynamic_network(&hotkey_id, &coldkey_id);

        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            TaoCurrency::ZERO
        );

        assert_err!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_id),
                hotkey_id,
                netuid,
                amount.into(),
            ),
            Error::<Test>::AmountTooLow
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
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Set fee rate to 0 so that alpha fee is not moved to block producer 
        pallet_subtensor_swap::FeeRate::<Test>::insert(netuid, 0);
        
        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey_account_id), 0);
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, 0);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            amount.into(),
        );

        // Add subnet TAO for the equivalent amount added at price
        let amount_tao = U96F32::from_num(amount)
            * U96F32::from_num(
                <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into()),
            );
        SubnetTAO::<Test>::mutate(netuid, |v| {
            *v += amount_tao.saturating_to_num::<u64>().into()
        });
        TotalStake::<Test>::mutate(|v| *v += amount_tao.saturating_to_num::<u64>().into());

        // Do the magic
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into()
        ));

        let fee = <Test as Config>::SwapInterface::approx_fee_amount(
            netuid.into(),
            TaoCurrency::from(amount),
        )
        .to_u64();
        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount - fee,
            epsilon = amount / 1000,
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock() + fee.into(),
            epsilon = SubtensorModule::get_total_stake() / 10_000_000.into()
        );

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
        let amount_staked = DefaultMinStake::<Test>::get().to_u64() * 10;

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Set the liquidity at lowest possible value so that all staking requests fail
        let reserve = u64::from(mock::SwapMinimumReserve::get()) - 1;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Check the error
        assert_noop!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                amount_staked.into()
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

/// cargo test --package pallet-subtensor --lib -- tests::staking::test_add_stake_insufficient_liquidity_one_side_ok --exact --show-output
#[test]
fn test_add_stake_insufficient_liquidity_one_side_ok() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let amount_staked = DefaultMinStake::<Test>::get().to_u64() * 10;

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Set the liquidity at lowest possible value so that all staking requests fail
        let reserve_alpha = 1_000_000_000_u64;
        let reserve_tao = u64::from(mock::SwapMinimumReserve::get()) - 1;
        mock::setup_reserves(netuid, reserve_tao.into(), reserve_alpha.into());

        // Check the error
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            amount_staked.into()
        ));
    });
}

/// cargo test --package pallet-subtensor --lib -- tests::staking::test_add_stake_insufficient_liquidity_one_side_fail --exact --show-output
#[test]
fn test_add_stake_insufficient_liquidity_one_side_fail() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let amount_staked = DefaultMinStake::<Test>::get().to_u64() * 10;

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Set the liquidity at lowest possible value so that all staking requests fail
        let reserve_alpha = u64::from(mock::SwapMinimumReserve::get()) - 1;
        let reserve_tao = u64::from(mock::SwapMinimumReserve::get());
        mock::setup_reserves(netuid, reserve_tao.into(), reserve_alpha.into());

        // Check the error
        assert_noop!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                amount_staked.into()
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
        let amount_staked = DefaultMinStake::<Test>::get().to_u64() * 10;

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount_staked);

        // Simulate stake for hotkey
        let reserve = u64::MAX / 1000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        let alpha = SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid,
            amount_staked.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        // Set the liquidity at lowest possible value so that all staking requests fail
        let reserve = u64::from(mock::SwapMinimumReserve::get()) - 1;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Check the error
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, alpha),
            Error::<Test>::InsufficientLiquidity
        );

        // Mock more liquidity - remove becomes successful
        SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(amount_staked + 1));
        SubnetAlphaIn::<Test>::insert(netuid, AlphaCurrency::from(1));
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            alpha
        ),);
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
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Set fee rate to 0 so that alpha fee is not moved to block producer 
        pallet_subtensor_swap::FeeRate::<Test>::insert(netuid, 0);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        mock::setup_reserves(netuid, (amount * 100).into(), (amount * 100).into());

        // Some basic assertions
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount
        );
        let initial_total_balance = Balances::total_balance(&coldkey_account_id);
        assert_eq!(initial_total_balance, amount);
        let inital_total_issuance = Balances::total_issuance();

        // Stake to hotkey account, and check if the result is ok
        let (_, fee) = mock::swap_tao_to_alpha(netuid, amount.into());
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into()
        ));

        let total_issuance_after_stake = Balances::total_issuance();

        // Remove all stake
        let stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );

        let total_fee = mock::swap_alpha_to_tao(netuid, stake).1 + fee;

        remove_stake_rate_limit_for_tests(&hotkey_account_id, &coldkey_account_id, netuid);

        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            stake
        ));

        let total_issuance_after_unstake = Balances::total_issuance();

        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount - total_fee,
            epsilon = 50
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock() + total_fee.into(),
            epsilon = TaoCurrency::from(fee) / 1000.into() + 1.into()
        );

        // Check if total issuance is equal to the added stake, even after remove stake (no fee,
        // includes reserved/locked balance)
        assert_abs_diff_eq!(
            inital_total_issuance,
            total_issuance_after_stake + amount,
            epsilon = 1,
        );

        // After staking + unstaking the 2 * fee amount stays in SubnetTAO and TotalStake,
        // so the total issuance should be lower by that amount
        assert_abs_diff_eq!(
            inital_total_issuance,
            total_issuance_after_unstake + total_fee,
            epsilon = inital_total_issuance / 10000,
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_prev_epoch_stake --exact --show-output --nocapture
#[test]
fn test_remove_prev_epoch_stake() {
    new_test_ext(1).execute_with(|| {
        // Test case: (amount_to_stake, AlphaDividendsPerSubnet, TotalHotkeyAlphaLastEpoch, expected_fee)
        [
            // No previous epoch stake and low hotkey stake
            (
                DefaultMinStake::<Test>::get().to_u64() * 10,
                0_u64,
                1000_u64,
            ),
            // Same, but larger amount to stake - we get 0.005% for unstake
            (1_000_000_000, 0_u64, 1000_u64),
            (100_000_000_000, 0_u64, 1000_u64),
            // Lower previous epoch stake than current stake
            // Staking/unstaking 100 TAO, divs / total = 0.1 => fee is 1 TAO
            (100_000_000_000, 1_000_000_000_u64, 10_000_000_000_u64),
            // Staking/unstaking 100 TAO, divs / total = 0.001 => fee is 0.01 TAO
            (100_000_000_000, 10_000_000_u64, 10_000_000_000_u64),
            // Higher previous epoch stake than current stake
            (1_000_000_000, 100_000_000_000_u64, 100_000_000_000_000_u64),
        ]
        .into_iter()
        .for_each(|(amount_to_stake, alpha_divs, hotkey_alpha)| {
            let alpha_divs = AlphaCurrency::from(alpha_divs);
            let hotkey_alpha = AlphaCurrency::from(hotkey_alpha);
            let subnet_owner_coldkey = U256::from(1);
            let subnet_owner_hotkey = U256::from(2);
            let hotkey_account_id = U256::from(581337);
            let coldkey_account_id = U256::from(81337);
            let amount = amount_to_stake;
            let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
            register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

            // Give it some $$$ in his coldkey balance
            SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);
            AlphaDividendsPerSubnet::<Test>::insert(netuid, hotkey_account_id, alpha_divs);
            TotalHotkeyAlphaLastEpoch::<Test>::insert(hotkey_account_id, netuid, hotkey_alpha);
            let balance_before = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
            mock::setup_reserves(
                netuid,
                (amount_to_stake * 10).into(),
                (amount_to_stake * 10).into(),
            );

            // Stake to hotkey account, and check if the result is ok
            let (_, fee) = mock::swap_tao_to_alpha(netuid, amount.into());
            assert_ok!(SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into()
            ));

            // Remove all stake
            let stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid,
            );

            remove_stake_rate_limit_for_tests(&hotkey_account_id, &coldkey_account_id, netuid);
            let fee = mock::swap_alpha_to_tao(netuid, stake).1 + fee;
            assert_ok!(SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                stake
            ));

            // Measure actual fee
            let balance_after = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
            let actual_fee = balance_before - balance_after;

            assert_abs_diff_eq!(actual_fee, fee, epsilon = fee / 100);
        });
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_staking_sets_div_variables --exact --show-output --nocapture
#[test]
fn test_staking_sets_div_variables() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let hotkey_account_id = U256::from(581337);
        let coldkey_account_id = U256::from(81337);
        let amount = 100_000_000_000;
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let tempo = 10;
        Tempo::<Test>::insert(netuid, tempo);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Verify that divident variables are clear in the beginning
        assert_eq!(
            AlphaDividendsPerSubnet::<Test>::get(netuid, hotkey_account_id),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            TotalHotkeyAlphaLastEpoch::<Test>::get(hotkey_account_id, netuid),
            AlphaCurrency::ZERO
        );

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into()
        ));

        // Verify that divident variables are still clear in the beginning
        assert_eq!(
            AlphaDividendsPerSubnet::<Test>::get(netuid, hotkey_account_id),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            TotalHotkeyAlphaLastEpoch::<Test>::get(hotkey_account_id, netuid),
            AlphaCurrency::ZERO
        );

        // Wait for 1 epoch
        step_block(tempo + 1);

        // Verify that divident variables have been set
        let stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );

        assert!(
            AlphaDividendsPerSubnet::<Test>::get(netuid, hotkey_account_id) > AlphaCurrency::ZERO
        );
        assert_abs_diff_eq!(
            TotalHotkeyAlphaLastEpoch::<Test>::get(hotkey_account_id, netuid),
            stake,
            epsilon = stake / 100_000.into()
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
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, 192213123);

        // There is no stake in the system at first, other than the network initial lock so result;
        assert_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock()
        );

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount.into(),
        );

        // The stake that is now in the account, should equal the amount
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount.into(),
            epsilon = 2.into()
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
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_id, coldkey_id, 192213123);

        // Add some stake that can be removed
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount.into(),
        );

        // Prelimiary checks
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            amount.into(),
            epsilon = 10.into()
        );

        // Remove stake
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount.into(),
        );

        // The stake on the hotkey account should be 0
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            TaoCurrency::ZERO
        );
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
            Err(e) => panic!("Error: {e:?}"),
        };

        let neuron_uid_ex = match SubtensorModule::get_uid_for_net_and_hotkey(netuid_ex, &hotkey_id)
        {
            Ok(k) => k,
            Err(e) => panic!("Error: {e:?}"),
        };

        // Add some stake that can be removed
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount.into(),
        );

        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            amount.into()
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            AlphaCurrency::ZERO
        );

        // Remove all stake
        SubtensorModule::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            amount.into(),
        );

        //
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, neuron_uid),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid_ex, neuron_uid_ex),
            AlphaCurrency::ZERO
        );
    });
}

// /************************************************************
// 	staking::increase_total_stake() tests
// ************************************************************/
#[test]
fn test_increase_total_stake_ok() {
    new_test_ext(1).execute_with(|| {
        let increment = TaoCurrency::from(10000);
        assert_eq!(SubtensorModule::get_total_stake(), TaoCurrency::ZERO);
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
        let initial_total_stake = TaoCurrency::from(10000);
        let decrement = TaoCurrency::from(5000);

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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(add_dynamic_network(&hotkey_id, &coldkey_id));
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            intial_amount.into(),
        );

        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            intial_amount.into(),
            epsilon = 2.into()
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid
            ),
            intial_amount.into()
        );
        assert_ok!(SubtensorModule::calculate_reduced_stake_on_subnet(
            &hotkey_id,
            &coldkey_id,
            netuid,
            (intial_amount / 2).into()
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
            intial_amount.into(),
        );

        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_id),
            intial_amount.into(),
            epsilon = 2.into()
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid
            ),
            intial_amount.into()
        );
        assert_err!(
            SubtensorModule::calculate_reduced_stake_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid,
                (intial_amount * 2).into()
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
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
            intial_amount.into()
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid
            ),
            intial_amount.into()
        );
        assert_err!(
            SubtensorModule::calculate_reduced_stake_on_subnet(
                &hotkey_id,
                &coldkey_id,
                netuid,
                1_000.into()
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

#[test]
fn test_non_existent_account() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &U256::from(0),
            &(U256::from(0)),
            netuid,
            10.into(),
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &U256::from(0),
                &U256::from(0),
                netuid
            ),
            10.into()
        );
        // No subnets => no iteration => zero total stake
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&(U256::from(0))),
            TaoCurrency::ZERO
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
///
/// cargo test --package pallet-subtensor --lib -- tests::staking::test_clear_small_nominations --exact --show-output
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
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        let fee = DefaultMinStake::<Test>::get().to_u64();
        let init_balance = amount + fee + ExistentialDeposit::get();

        // Set fee rate to 0 so that alpha fee is not moved to block producer 
        pallet_subtensor_swap::FeeRate::<Test>::insert(netuid, 0);

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
            amount.into()
        ));
        let alpha_stake1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid);
        let unstake_amount1 = AlphaCurrency::from(alpha_stake1.to_u64() * 997 / 1000);
        let small1 = alpha_stake1 - unstake_amount1;
        remove_stake_rate_limit_for_tests(&hot1, &cold1, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(cold1),
            hot1,
            netuid,
            unstake_amount1
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            small1
        );

        // Add stake cold2 --> hot1 (is delegation.)
        SubtensorModule::add_balance_to_coldkey_account(&cold2, init_balance);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(cold2),
            hot1,
            netuid,
            amount.into()
        ));
        let alpha_stake2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid);
        let unstake_amount2 = AlphaCurrency::from(alpha_stake2.to_u64() * 997 / 1000);
        let small2 = alpha_stake2 - unstake_amount2;
        remove_stake_rate_limit_for_tests(&hot1, &cold2, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(cold2),
            hot1,
            netuid,
            unstake_amount2
        ));
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            small2
        );

        let balance1_before_cleaning = Balances::free_balance(cold1);
        let balance2_before_cleaning = Balances::free_balance(cold2);

        // Run clear all small nominations when min stake is zero (noop)
        SubtensorModule::set_nominator_min_required_stake(0);
        assert_eq!(SubtensorModule::get_nominator_min_required_stake(), 0);
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            small1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            small2
        );

        // Set min nomination to above small1 and small2
        let total_hot1_stake_before = TotalHotkeyAlpha::<Test>::get(hot1, netuid);
        let total_stake_before = TotalStake::<Test>::get();
        SubtensorModule::set_nominator_min_required_stake(
            (small1.to_u64().min(small2.to_u64()) * 2).into(),
        );

        // Run clear all small nominations (removes delegations under 10)
        SubtensorModule::clear_small_nominations();
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold1, netuid),
            small1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hot1, &cold2, netuid),
            AlphaCurrency::ZERO
        );

        // Balances have been added back into accounts.
        let balance1_after_cleaning = Balances::free_balance(cold1);
        let balance2_after_cleaning = Balances::free_balance(cold2);
        assert_eq!(balance1_before_cleaning, balance1_after_cleaning);
        assert!(balance2_before_cleaning < balance2_after_cleaning);

        assert_abs_diff_eq!(
            TotalHotkeyAlpha::<Test>::get(hot1, netuid),
            total_hot1_stake_before - small2,
            epsilon = 1.into()
        );
        assert!(TotalStake::<Test>::get() < total_stake_before);
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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(1);
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
        let netuid = NetUid::from(1);
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

// Test rate-limiting on an increase take just after a decrease take
// Prevents a Validator from decreasing take and then increasing it immediately after.
#[test]
fn test_rate_limits_enforced_on_decrease_before_increase_take() {
    new_test_ext(1).execute_with(|| {
        // Make account
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);

        // Add balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, 100000);

        // Register the neuron to a new network
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey0, coldkey0, 124124);

        // Coldkey / hotkey 0 become delegates with 9% take
        Delegates::<Test>::insert(hotkey0, SubtensorModule::get_min_delegate_take() + 1);
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take() + 1
        );

        // Decrease take
        assert_ok!(SubtensorModule::do_decrease_take(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take()
        )); // Verify decrease
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        // Increase take immediately after
        assert_eq!(
            SubtensorModule::do_increase_take(
                RuntimeOrigin::signed(coldkey0),
                hotkey0,
                SubtensorModule::get_min_delegate_take() + 1
            ),
            Err(Error::<Test>::DelegateTxRateLimitExceeded.into())
        ); // Verify no change
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take()
        );

        step_block(1 + InitialTxDelegateTakeRateLimit::get() as u16);

        // Can increase after waiting
        assert_ok!(SubtensorModule::do_increase_take(
            RuntimeOrigin::signed(coldkey0),
            hotkey0,
            SubtensorModule::get_min_delegate_take() + 1
        )); // Verify increase
        assert_eq!(
            SubtensorModule::get_hotkey_take(&hotkey0),
            SubtensorModule::get_min_delegate_take() + 1
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_get_total_delegated_stake_after_unstaking --exact --show-output
#[test]
fn test_get_total_delegated_stake_after_unstaking() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let initial_stake = DefaultMinStake::<Test>::get().to_u64() * 10;
        let existential_deposit = ExistentialDeposit::get();
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Add balance to delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, initial_stake);

        // Delegate stake
        let (_, fee) = mock::swap_tao_to_alpha(netuid, initial_stake.into());
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            initial_stake.into()
        ));

        // Check initial delegated stake
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegator),
            (initial_stake - existential_deposit - fee).into(),
            epsilon = TaoCurrency::from(initial_stake / 100),
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey),
            (initial_stake - existential_deposit - fee).into(),
            epsilon = TaoCurrency::from(initial_stake / 100),
        );
        let delegated_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &delegate_hotkey,
            &delegator,
            netuid,
        );
        remove_stake_rate_limit_for_tests(&delegator, &delegate_hotkey, netuid);
        // Unstake part of the delegation
        let unstake_amount_alpha = delegated_alpha / 2.into();
        remove_stake_rate_limit_for_tests(&delegate_hotkey, &delegator, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            unstake_amount_alpha.into()
        ));
        let current_price = U96F32::from_num(
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into()),
        );

        // Calculate the expected delegated stake
        let unstake_amount =
            (current_price * U96F32::from_num(unstake_amount_alpha)).to_num::<u64>();
        let expected_delegated_stake = initial_stake - unstake_amount - existential_deposit - fee;

        // Debug prints
        log::debug!("Initial stake: {initial_stake}");
        log::debug!("Unstake amount: {unstake_amount}");
        log::debug!("Existential deposit: {existential_deposit}");
        log::debug!("Expected delegated stake: {expected_delegated_stake}");
        log::debug!(
            "Actual delegated stake: {}",
            SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey)
        );

        // Check the total delegated stake after unstaking
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegator),
            expected_delegated_stake.into(),
            epsilon = TaoCurrency::from(expected_delegated_stake / 1000),
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate_hotkey),
            expected_delegated_stake.into(),
            epsilon = TaoCurrency::from(expected_delegated_stake / 1000),
        );
    });
}

#[test]
fn test_get_total_delegated_stake_no_delegations() {
    new_test_ext(1).execute_with(|| {
        let delegate = U256::from(1);
        let coldkey = U256::from(2);
        let netuid = NetUid::from(1u16);

        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, delegate, coldkey, 0);

        // Check that there's no delegated stake
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegate),
            TaoCurrency::ZERO
        );
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
        let stake_amount = DefaultMinStake::<Test>::get().to_u64() * 10 - 1;
        let existential_deposit = ExistentialDeposit::get();
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);

        // Add stake from delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, stake_amount);

        let (_, fee) = mock::swap_tao_to_alpha(netuid, stake_amount.into());

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            stake_amount.into()
        ));

        // Debug prints
        log::debug!("Delegate coldkey: {delegate_coldkey:?}");
        log::debug!("Delegate hotkey: {delegate_hotkey:?}");
        log::debug!("Delegator: {delegator:?}");
        log::debug!("Stake amount: {stake_amount}");
        log::debug!("Existential deposit: {existential_deposit}");
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
            expected_delegated_stake.into(),
            epsilon = TaoCurrency::from(expected_delegated_stake / 100),
        );
        assert_abs_diff_eq!(
            actual_delegator_stake,
            expected_delegated_stake.into(),
            epsilon = TaoCurrency::from(expected_delegated_stake / 100),
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
        let stake1 = DefaultMinStake::<Test>::get() * 10.into();
        let stake2 = DefaultMinStake::<Test>::get() * 10.into() - 1.into();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey1, coldkey1, 0);
        register_ok_neuron(netuid, hotkey2, coldkey2, 0);

        // Add stake from delegator1
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey1,
            stake1.to_u64() + existential_deposit,
        );
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey1),
            hotkey1,
            netuid,
            stake1
        ));

        // Add stake from delegator2
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey2,
            stake2.to_u64() + existential_deposit,
        );
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey2),
            hotkey2,
            netuid,
            stake2
        ));

        // Calculate expected total delegated stake
        let alpha1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1, &coldkey1, netuid,
        );
        let alpha2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2, &coldkey2, netuid,
        );
        let expected_total_stake = alpha1 + alpha2;
        let actual_total_stake = SubtensorModule::get_alpha_share_pool(hotkey1, netuid)
            .get_value(&coldkey1)
            + SubtensorModule::get_alpha_share_pool(hotkey2, netuid).get_value(&coldkey2);

        // Total subnet stake should match the sum of delegators' stakes minus existential deposits.
        assert_abs_diff_eq!(
            AlphaCurrency::from(actual_total_stake),
            expected_total_stake,
            epsilon = expected_total_stake / 1000.into()
        );
    });
}

#[test]
fn test_get_total_delegated_stake_exclude_owner_stake() {
    new_test_ext(1).execute_with(|| {
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);
        let owner_stake = DefaultMinStake::<Test>::get().to_u64() * 10;
        let delegator_stake = DefaultMinStake::<Test>::get().to_u64() * 10 - 1;

        let netuid = add_dynamic_network(&delegate_hotkey, &delegate_coldkey);

        // Add owner stake
        SubtensorModule::add_balance_to_coldkey_account(&delegate_coldkey, owner_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey,
            netuid,
            owner_stake.into()
        ));

        // Add delegator stake
        SubtensorModule::add_balance_to_coldkey_account(&delegator, delegator_stake);
        let (_, fee) = mock::swap_tao_to_alpha(netuid, delegator_stake.into());
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            netuid,
            delegator_stake.into()
        ));

        // Check the total delegated stake (should exclude owner's stake)
        let expected_delegated_stake = delegator_stake - fee;
        let actual_delegated_stake =
            SubtensorModule::get_total_stake_for_coldkey(&delegate_coldkey);

        assert_abs_diff_eq!(
            actual_delegated_stake,
            expected_delegated_stake.into(),
            epsilon = TaoCurrency::from(expected_delegated_stake / 100)
        );
    });
}

/// Test that emission is distributed correctly between one validator, one
/// vali-miner, and one miner
#[test]
fn test_mining_emission_distribution_validator_valiminer_miner() {
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

// Verify staking too low amount is impossible
#[test]
fn test_staking_too_little_fails() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = 10_000;

        //add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Coldkey / hotkey 0 decreases take to 5%. This should fail as the minimum take is 9%
        assert_err!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                1.into()
            ),
            Error::<Test>::AmountTooLow
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_add_stake_fee_goes_to_subnet_tao --exact --show-output --nocapture
#[ignore = "fee now goes to liquidity provider"]
#[test]
fn test_add_stake_fee_goes_to_subnet_tao() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let existential_deposit = ExistentialDeposit::get();
        let tao_to_stake = DefaultMinStake::<Test>::get() * 10.into();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        let subnet_tao_before = SubnetTAO::<Test>::get(netuid);

        // Add stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, tao_to_stake.to_u64());
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            tao_to_stake
        ));

        // Calculate expected stake
        let expected_alpha = AlphaCurrency::from(tao_to_stake.to_u64() - existential_deposit);
        let actual_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let subnet_tao_after = SubnetTAO::<Test>::get(netuid);

        // Total subnet stake should match the sum of delegators' stakes minus existential deposits.
        assert_abs_diff_eq!(
            actual_alpha,
            expected_alpha,
            epsilon = expected_alpha / 1000.into()
        );

        // Subnet TAO should have increased by the full tao_to_stake amount
        assert_abs_diff_eq!(
            subnet_tao_before + tao_to_stake,
            subnet_tao_after,
            epsilon = 10.into()
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_stake_fee_goes_to_subnet_tao --exact --show-output --nocapture
#[ignore = "fees no go to liquidity providers"]
#[test]
fn test_remove_stake_fee_goes_to_subnet_tao() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let tao_to_stake = DefaultMinStake::<Test>::get() * 10.into();

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        let subnet_tao_before = SubnetTAO::<Test>::get(netuid);

        // Add stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, tao_to_stake.into());
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
            subnet_tao_before,
            subnet_tao_after,
            epsilon = (alpha_to_unstake.to_u64() / 1000).into()
        );

        // User balance should decrease by 2x fee as a result of staking + unstaking
        let balance_after = SubtensorModule::get_coldkey_balance(&coldkey);
        assert_abs_diff_eq!(
            balance_after,
            tao_to_stake.to_u64(),
            epsilon = tao_to_stake.to_u64() / 1000
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_stake_fee_realistic_values --exact --show-output --nocapture
#[ignore = "fees are now calculated on the SwapInterface side"]
#[test]
fn test_remove_stake_fee_realistic_values() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let alpha_to_unstake = AlphaCurrency::from(111_180_000_000);
        let alpha_divs = AlphaCurrency::from(2_816_190);

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);

        // Mock a realistic scenario:
        //   Subnet 1 has 3896 TAO and 128_011 Alpha in reserves, which
        //   makes its price ~0.03.
        //   A hotkey has 111 Alpha stake and is unstaking all Alpha.
        //   Alpha dividends of this hotkey are ~0.0028
        //   This makes fee be equal ~0.0028 Alpha ~= 84000 rao
        let tao_reserve = 3_896_056_559_708_u64;
        let alpha_in = 128_011_331_299_964_u64;
        mock::setup_reserves(netuid, tao_reserve.into(), alpha_in.into());
        AlphaDividendsPerSubnet::<Test>::insert(netuid, hotkey, alpha_divs);
        TotalHotkeyAlphaLastEpoch::<Test>::insert(hotkey, netuid, alpha_to_unstake);

        // Add stake first time to init TotalHotkeyAlpha
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            alpha_to_unstake,
        );

        // Remove stake to measure fee
        let balance_before = SubtensorModule::get_coldkey_balance(&coldkey);
        let (expected_tao, expected_fee) = mock::swap_alpha_to_tao(netuid, alpha_to_unstake);

        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            alpha_to_unstake
        ));

        // Calculate expected fee
        let balance_after = SubtensorModule::get_coldkey_balance(&coldkey);
        // FIXME since fee is calculated by SwapInterface and the values here are after fees, the
        // actual_fee is 0. but it's left here to discuss in review
        let actual_fee = expected_tao.to_u64() - (balance_after - balance_before);
        log::info!("Actual fee: {actual_fee:?}");

        assert_abs_diff_eq!(actual_fee, expected_fee, epsilon = expected_fee / 1000);
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
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Setup liquidity with 21M TAO values
        mock::setup_reserves(netuid, amount.into(), amount.into());

        // Stake and check if the result is ok
        let (expected_alpha, _) = mock::swap_tao_to_alpha(netuid, amount.into());
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into()
        ));

        // Check if stake has increased properly
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey_account_id, netuid),
            expected_alpha
        );

        // Check if total stake has increased accordingly.
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake(),
            SubtensorModule::get_network_min_lock() + amount.into(),
            epsilon = 1.into()
        );
    });
}

#[test]
fn test_max_amount_add_root() {
    new_test_ext(0).execute_with(|| {
        // 0 price on root => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_add(NetUid::ROOT, TaoCurrency::ZERO),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 0.999999... price on root => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_add(NetUid::ROOT, TaoCurrency::from(999_999_999)),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 1.0 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(NetUid::ROOT, TaoCurrency::from(1_000_000_000)),
            Ok(u64::MAX)
        );

        // 1.000...001 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(NetUid::ROOT, TaoCurrency::from(1_000_000_001)),
            Ok(u64::MAX)
        );

        // 2.0 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(NetUid::ROOT, TaoCurrency::from(2_000_000_000)),
            Ok(u64::MAX)
        );
    });
}

#[test]
fn test_max_amount_add_stable() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        // 0 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, TaoCurrency::ZERO),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 0.999999... price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, TaoCurrency::from(999_999_999)),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 1.0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, TaoCurrency::from(1_000_000_000)),
            Ok(u64::MAX)
        );

        // 1.000...001 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, TaoCurrency::from(1_000_000_001)),
            Ok(u64::MAX)
        );

        // 2.0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_add(netuid, TaoCurrency::from(2_000_000_000)),
            Ok(u64::MAX)
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_add_dynamic --exact --show-output
#[test]
fn test_max_amount_add_dynamic() {
    // tao_in, alpha_in, limit_price, expected_max_swappable (with 0.05% fees)
    [
        // Zero handling (no panics)
        (
            1_000_000_000,
            1_000_000_000,
            0,
            Err(DispatchError::from(
                pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
            )),
        ),
        // Low bounds
        (100, 100, 1_100_000_000, Ok(4)),
        (1_000, 1_000, 1_100_000_000, Ok(48)),
        (10_000, 10_000, 1_100_000_000, Ok(488)),
        // Basic math
        (1_000_000, 1_000_000, 4_000_000_000, Ok(1_000_500)),
        (1_000_000, 1_000_000, 9_000_000_000, Ok(2_001_000)),
        (1_000_000, 1_000_000, 16_000_000_000, Ok(3_001_500)),
        (
            1_000_000_000_000,
            1_000_000_000_000,
            16_000_000_000,
            Ok(3_001_500_000_000),
        ),
        // Normal range values with edge cases
        (
            150_000_000_000,
            100_000_000_000,
            0,
            Err(DispatchError::from(
                pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
            )),
        ),
        (
            150_000_000_000,
            100_000_000_000,
            100_000_000,
            Err(DispatchError::from(
                pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
            )),
        ),
        (
            150_000_000_000,
            100_000_000_000,
            500_000_000,
            Err(DispatchError::from(
                pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
            )),
        ),
        (
            150_000_000_000,
            100_000_000_000,
            1_499_999_999,
            Err(DispatchError::from(
                pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
            )),
        ),
        (
            150_000_000_000,
            100_000_000_000,
            1_500_000_000,
            Err(DispatchError::from(
                pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
            )),
        ),
        (150_000_000_000, 100_000_000_000, 1_500_000_001, Ok(49)),
        (
            150_000_000_000,
            100_000_000_000,
            6_000_000_000,
            Ok(150_075_000_000),
        ),
        // Miscellaneous overflows and underflows
        (u64::MAX / 2, u64::MAX, u64::MAX, Ok(u64::MAX)),
    ]
    .into_iter()
    .for_each(|(tao_in, alpha_in, limit_price, expected_max_swappable)| {
        new_test_ext(0).execute_with(|| {
            let alpha_in = AlphaCurrency::from(alpha_in);
            let subnet_owner_coldkey = U256::from(1001);
            let subnet_owner_hotkey = U256::from(1002);
            let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

            // Forse-set alpha in and tao reserve to achieve relative price of subnets
            SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_in));
            SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

            // Force the swap to initialize
            <Test as pallet::Config>::SwapInterface::init_swap(netuid, None);

            if !alpha_in.is_zero() {
                let expected_price = U96F32::from_num(tao_in) / U96F32::from_num(alpha_in);
                assert_abs_diff_eq!(
                    <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                        .to_num::<f64>(),
                    expected_price.to_num::<f64>(),
                    epsilon = expected_price.to_num::<f64>() / 1_000_f64
                );
            }

            match expected_max_swappable {
                Err(e) => assert_err!(
                    SubtensorModule::get_max_amount_add(netuid, limit_price.into()),
                    e
                ),
                Ok(v) => assert_abs_diff_eq!(
                    SubtensorModule::get_max_amount_add(netuid, limit_price.into()).unwrap(),
                    v,
                    epsilon = v / 10000
                ),
            }
        });
    });
}

#[test]
fn test_max_amount_remove_root() {
    new_test_ext(0).execute_with(|| {
        // 0 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(NetUid::ROOT, TaoCurrency::ZERO),
            Ok(AlphaCurrency::MAX)
        );

        // 0.5 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(NetUid::ROOT, TaoCurrency::from(500_000_000)),
            Ok(AlphaCurrency::MAX)
        );

        // 0.999999... price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(NetUid::ROOT, TaoCurrency::from(999_999_999)),
            Ok(AlphaCurrency::MAX)
        );

        // 1.0 price on root => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(NetUid::ROOT, TaoCurrency::from(1_000_000_000)),
            Ok(AlphaCurrency::MAX)
        );

        // 1.000...001 price on root => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_remove(NetUid::ROOT, TaoCurrency::from(1_000_000_001)),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 2.0 price on root => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_remove(NetUid::ROOT, TaoCurrency::from(2_000_000_000)),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );
    });
}

#[test]
fn test_max_amount_remove_stable() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        // 0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, TaoCurrency::ZERO),
            Ok(AlphaCurrency::MAX)
        );

        // 0.999999... price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, TaoCurrency::from(999_999_999)),
            Ok(AlphaCurrency::MAX)
        );

        // 1.0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, TaoCurrency::from(1_000_000_000)),
            Ok(AlphaCurrency::MAX)
        );

        // 1.000...001 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, TaoCurrency::from(1_000_000_001)),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 2.0 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_remove(netuid, TaoCurrency::from(2_000_000_000)),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
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

        // tao_in, alpha_in, limit_price, expected_max_swappable (+ 0.05% fee)
        [
            // Zero handling (no panics)
            (
                0,
                1_000_000_000,
                100,
                Err(DispatchError::from(
                    pallet_subtensor_swap::Error::<Test>::ReservesTooLow,
                )),
            ),
            (
                1_000_000_000,
                0,
                100,
                Err(DispatchError::from(
                    pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
                )),
            ),
            (10_000_000_000, 10_000_000_000, 0, Ok(u64::MAX)),
            // Low bounds (numbers are empirical, it is only important that result
            // is sharply decreasing when limit price increases)
            (1_000, 1_000, 0, Ok(u64::MAX)),
            (1_001, 1_001, 0, Ok(u64::MAX)),
            (1_001, 1_001, 1, Ok(17_646)),
            (1_001, 1_001, 2, Ok(17_646)),
            (1_001, 1_001, 1_001, Ok(17_646)),
            (1_001, 1_001, 10_000, Ok(17_646)),
            (1_001, 1_001, 100_000, Ok(17_646)),
            (1_001, 1_001, 1_000_000, Ok(17_646)),
            (1_001, 1_001, 10_000_000, Ok(9_103)),
            (1_001, 1_001, 100_000_000, Ok(2_186)),
            // Basic math
            (1_000_000, 1_000_000, 250_000_000, Ok(1_010_000)),
            (1_000_000, 1_000_000, 62_500_000, Ok(3_030_000)),
            (
                1_000_000_000_000,
                1_000_000_000_000,
                62_500_000,
                Ok(3_030_000_000_000),
            ),
            // Normal range values with edge cases and sanity checks
            (200_000_000_000, 100_000_000_000, 0, Ok(u64::MAX)),
            (
                200_000_000_000,
                100_000_000_000,
                500_000_000,
                Ok(101_000_000_000),
            ),
            (
                200_000_000_000,
                100_000_000_000,
                125_000_000,
                Ok(303_000_000_000),
            ),
            (
                200_000_000_000,
                100_000_000_000,
                2_000_000_000,
                Err(DispatchError::from(
                    pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
                )),
            ),
            (
                200_000_000_000,
                100_000_000_000,
                2_000_000_001,
                Err(DispatchError::from(
                    pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
                )),
            ),
            (200_000_000_000, 100_000_000_000, 1_999_999_999, Ok(24)),
            (200_000_000_000, 100_000_000_000, 1_999_999_990, Ok(250)),
            // Miscellaneous overflows and underflows
            (
                21_000_000_000_000_000,
                1_000_000,
                21_000_000_000_000_000,
                Ok(17_630_088),
            ),
            (21_000_000_000_000_000, 1_000_000, u64::MAX, Ok(67_000)),
            (
                21_000_000_000_000_000,
                1_000_000_000_000_000_000,
                u64::MAX,
                Err(DispatchError::from(
                    pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded,
                )),
            ),
            (
                21_000_000_000_000_000,
                1_000_000_000_000_000_000,
                20_000_000,
                Ok(24_700_000_000_000_000),
            ),
            (
                21_000_000_000_000_000,
                21_000_000_000_000_000,
                999_999_999,
                Ok(10_605_000),
            ),
            (
                21_000_000_000_000_000,
                21_000_000_000_000_000,
                0,
                Ok(u64::MAX),
            ),
        ]
        .into_iter()
        .for_each(|(tao_in, alpha_in, limit_price, expected_max_swappable)| {
            let alpha_in = AlphaCurrency::from(alpha_in);
            // Forse-set alpha in and tao reserve to achieve relative price of subnets
            SubnetTAO::<Test>::insert(netuid, TaoCurrency::from(tao_in));
            SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

            if !alpha_in.is_zero() {
                let expected_price = U64F64::from_num(tao_in) / U64F64::from_num(alpha_in);
                assert_eq!(
                    <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into()),
                    expected_price
                );
            }

            match expected_max_swappable {
                Err(e) => assert_err!(
                    SubtensorModule::get_max_amount_remove(netuid, limit_price.into()),
                    DispatchError::from(e)
                ),
                Ok(v) => {
                    let v = AlphaCurrency::from(v);
                    assert_abs_diff_eq!(
                        SubtensorModule::get_max_amount_remove(netuid, limit_price.into()).unwrap(),
                        v,
                        epsilon = v / 100.into()
                    );
                }
            }
        });
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_root_root --exact --show-output
#[test]
fn test_max_amount_move_root_root() {
    new_test_ext(0).execute_with(|| {
        // 0 price on (root, root) exchange => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(NetUid::ROOT, NetUid::ROOT, TaoCurrency::ZERO),
            Ok(AlphaCurrency::MAX)
        );

        // 0.5 price on (root, root) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                NetUid::ROOT,
                TaoCurrency::from(500_000_000)
            ),
            Ok(AlphaCurrency::MAX)
        );

        // 0.999999... price on (root, root) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                NetUid::ROOT,
                TaoCurrency::from(999_999_999)
            ),
            Ok(AlphaCurrency::MAX)
        );

        // 1.0 price on (root, root) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                NetUid::ROOT,
                TaoCurrency::from(1_000_000_000)
            ),
            Ok(AlphaCurrency::MAX)
        );

        // 1.000...001 price on (root, root) => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                NetUid::ROOT,
                TaoCurrency::from(1_000_000_001)
            ),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 2.0 price on (root, root) => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                NetUid::ROOT,
                TaoCurrency::from(2_000_000_000)
            ),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_root_stable --exact --show-output
#[test]
fn test_max_amount_move_root_stable() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 1, 0);

        // 0 price on (root, stable) exchange => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(NetUid::ROOT, netuid, TaoCurrency::ZERO),
            Ok(AlphaCurrency::MAX)
        );

        // 0.5 price on (root, stable) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                netuid,
                TaoCurrency::from(500_000_000)
            ),
            Ok(AlphaCurrency::MAX)
        );

        // 0.999999... price on (root, stable) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                netuid,
                TaoCurrency::from(999_999_999)
            ),
            Ok(AlphaCurrency::MAX)
        );

        // 1.0 price on (root, stable) => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                netuid,
                TaoCurrency::from(1_000_000_000)
            ),
            Ok(AlphaCurrency::MAX)
        );

        // 1.000...001 price on (root, stable) => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                netuid,
                TaoCurrency::from(1_000_000_001)
            ),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );

        // 2.0 price on (root, stable) => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                NetUid::ROOT,
                netuid,
                TaoCurrency::from(2_000_000_000)
            ),
            Err(Error::<Test>::ZeroMaxStakeAmount.into())
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_stable_dynamic --exact --show-output
#[test]
fn test_max_amount_move_stable_dynamic() {
    new_test_ext(0).execute_with(|| {
        // Add stable subnet
        let stable_netuid = NetUid::from(1);
        add_network(stable_netuid, 1, 0);

        // Add dynamic subnet
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let dynamic_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Force-set alpha in and tao reserve to make price equal 0.5
        let tao_reserve = TaoCurrency::from(50_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(dynamic_netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(dynamic_netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(dynamic_netuid.into());
        assert_eq!(current_price, U96F32::from_num(0.5));

        // The tests below just mimic the add_stake_limit tests for reverted price

        // 0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, TaoCurrency::ZERO),
            Ok(AlphaCurrency::MAX)
        );

        // 2.0 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                stable_netuid,
                dynamic_netuid,
                TaoCurrency::from(2_000_000_000)
            ),
            Err(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded.into())
        );

        // 3.0 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                stable_netuid,
                dynamic_netuid,
                TaoCurrency::from(3_000_000_000)
            ),
            Err(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded.into())
        );

        // 2x price => max is 1x TAO
        assert_abs_diff_eq!(
            SubtensorModule::get_max_amount_move(
                stable_netuid,
                dynamic_netuid,
                TaoCurrency::from(500_000_000)
            )
            .unwrap(),
            AlphaCurrency::from(
                tao_reserve.to_u64() + (tao_reserve.to_u64() as f64 * 0.003) as u64
            ),
            epsilon = AlphaCurrency::from(tao_reserve.to_u64() / 100),
        );

        // Precision test:
        // 1.99999..9000 price => max > 0
        assert!(
            SubtensorModule::get_max_amount_move(
                stable_netuid,
                dynamic_netuid,
                TaoCurrency::from(1_999_999_000)
            )
            .unwrap()
                > AlphaCurrency::ZERO
        );

        // Max price doesn't panic and returns something meaningful
        assert_eq!(
            SubtensorModule::get_max_amount_move(stable_netuid, dynamic_netuid, TaoCurrency::MAX),
            Err(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded.into())
        );
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                stable_netuid,
                dynamic_netuid,
                TaoCurrency::MAX - 1.into()
            ),
            Err(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded.into())
        );
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                stable_netuid,
                dynamic_netuid,
                TaoCurrency::MAX / 2.into()
            ),
            Err(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded.into())
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_max_amount_move_dynamic_stable --exact --show-output
#[test]
fn test_max_amount_move_dynamic_stable() {
    new_test_ext(0).execute_with(|| {
        // Add stable subnet
        let stable_netuid = NetUid::from(1);
        add_network(stable_netuid, 1, 0);

        // Add dynamic subnet
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let dynamic_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(dynamic_netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(dynamic_netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(dynamic_netuid.into());
        assert_eq!(current_price, U96F32::from_num(1.5));

        // The tests below just mimic the remove_stake_limit tests

        // 0 price => max is u64::MAX
        assert_eq!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, TaoCurrency::ZERO),
            Ok(AlphaCurrency::MAX)
        );

        // Low price values don't blow things up
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 1.into()).unwrap()
                > AlphaCurrency::ZERO
        );
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 2.into()).unwrap()
                > AlphaCurrency::ZERO
        );
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 3.into()).unwrap()
                > AlphaCurrency::ZERO
        );

        // 1.5000...1 price => max is 0
        assert_eq!(
            SubtensorModule::get_max_amount_move(
                dynamic_netuid,
                stable_netuid,
                1_500_000_001.into()
            ),
            Err(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded.into())
        );

        // 1.5 price => max is 0 because of non-zero slippage
        assert_abs_diff_eq!(
            SubtensorModule::get_max_amount_move(
                dynamic_netuid,
                stable_netuid,
                1_500_000_000.into()
            )
            .unwrap_or(AlphaCurrency::ZERO),
            AlphaCurrency::ZERO,
            epsilon = 10_000.into()
        );

        // 1/4 price => max is 1x Alpha
        assert_abs_diff_eq!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, 375_000_000.into())
                .unwrap(),
            alpha_in + alpha_in / 2000.into(), // + 0.05% fee
            epsilon = alpha_in / 10_000.into(),
        );

        // Precision test:
        // 1.499999.. price => max > 0
        assert!(
            SubtensorModule::get_max_amount_move(
                dynamic_netuid,
                stable_netuid,
                1_499_999_999.into()
            )
            .unwrap()
                > AlphaCurrency::ZERO
        );

        // Max price doesn't panic and returns something meaningful
        assert!(
            SubtensorModule::get_max_amount_move(dynamic_netuid, stable_netuid, TaoCurrency::MAX)
                .unwrap_or(AlphaCurrency::ZERO)
                < 21_000_000_000_000_000.into()
        );
        assert!(
            SubtensorModule::get_max_amount_move(
                dynamic_netuid,
                stable_netuid,
                TaoCurrency::MAX - 1.into()
            )
            .unwrap_or(AlphaCurrency::ZERO)
                < 21_000_000_000_000_000.into()
        );
        assert!(
            SubtensorModule::get_max_amount_move(
                dynamic_netuid,
                stable_netuid,
                TaoCurrency::MAX / 2.into()
            )
            .unwrap_or(AlphaCurrency::ZERO)
                < 21_000_000_000_000_000.into()
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
                let expected_max_swappable = AlphaCurrency::from(expected_max_swappable);
                // Forse-set alpha in and tao reserve to achieve relative price of subnets
                SubnetTAO::<Test>::insert(origin_netuid, TaoCurrency::from(tao_in_1));
                SubnetAlphaIn::<Test>::insert(origin_netuid, AlphaCurrency::from(alpha_in_1));
                SubnetTAO::<Test>::insert(destination_netuid, TaoCurrency::from(tao_in_2));
                SubnetAlphaIn::<Test>::insert(destination_netuid, AlphaCurrency::from(alpha_in_2));

                if !alpha_in_1.is_zero() && !alpha_in_2.is_zero() {
                    let origin_price = tao_in_1 as f64 / alpha_in_1 as f64;
                    let dest_price = tao_in_2 as f64 / alpha_in_2 as f64;
                    if dest_price != 0. {
                        let expected_price = origin_price / dest_price;
                        assert_abs_diff_eq!(
                            (<Test as pallet::Config>::SwapInterface::current_alpha_price(
                                origin_netuid.into()
                            ) / <Test as pallet::Config>::SwapInterface::current_alpha_price(
                                destination_netuid.into()
                            ))
                            .to_num::<f64>(),
                            expected_price,
                            epsilon = 0.000_000_001
                        );
                    }
                }

                assert_abs_diff_eq!(
                    SubtensorModule::get_max_amount_move(
                        origin_netuid,
                        destination_netuid,
                        limit_price.into()
                    )
                    .unwrap_or(AlphaCurrency::ZERO),
                    expected_max_swappable,
                    epsilon = precision.into()
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

        // add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Setup limit price so that it doesn't peak above 4x of current price
        // The amount that can be executed at this price is 450 TAO only
        // Alpha produced will be equal to 75 = 450*100/(450+150)
        let limit_price = TaoCurrency::from(24_000_000_000);
        let expected_executed_stake = AlphaCurrency::from(75_000_000_000);

        // Add stake with slippage safety and check if the result is ok
        assert_ok!(SubtensorModule::add_stake_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into(),
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
            expected_executed_stake,
            epsilon = expected_executed_stake / 1000.into(),
        );

        // Check that 450 TAO less fees balance still remains free on coldkey
        let fee = <tests::mock::Test as pallet::Config>::SwapInterface::approx_fee_amount(
            netuid.into(),
            TaoCurrency::from(amount / 2),
        )
        .to_u64() as f64;
        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            amount / 2 - fee as u64,
            epsilon = amount / 2 / 1000
        );

        // Check that price has updated to ~24 = (150+450) / (100 - 75)
        let exp_price = U96F32::from_num(24.0);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        assert_abs_diff_eq!(
            exp_price.to_num::<f64>(),
            current_price.to_num::<f64>(),
            epsilon = 0.001,
        );
    });
}

#[test]
fn test_add_stake_limit_fill_or_kill() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = 300_000_000_000; // over the maximum

        // add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Force-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        // FIXME it's failing because in the swap pallet, the alpha price is set only after an
        // initial swap
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Setup limit price so that it doesn't peak above 4x of current price
        let limit_price = TaoCurrency::from(6_000_000_000);

        // Add stake with slippage safety and check if it fails
        assert_noop!(
            SubtensorModule::add_stake_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into(),
                limit_price,
                false
            ),
            Error::<Test>::SlippageTooHigh
        );

        // Lower the amount and it should succeed now
        let amount_ok = TaoCurrency::from(150_000_000_000); // fits the maximum
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
fn test_add_stake_limit_partial_zero_max_stake_amount_error() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);

        // Exact values from the error:
        // https://taostats.io/extrinsic/5338471-0009?network=finney
        let amount = 19980000000;
        let limit_price = TaoCurrency::from(26953618);
        let tao_reserve = TaoCurrency::from(5_032_494_439_940);
        let alpha_in = AlphaCurrency::from(186_268_425_402_874);

        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        assert_noop!(
            SubtensorModule::add_stake_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into(),
                limit_price,
                true
            ),
            DispatchError::from(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded)
        );
    });
}

#[test]
fn test_remove_stake_limit_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let stake_amount = 300_000_000_000;

        // add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            stake_amount + ExistentialDeposit::get(),
        );

        // Forse-set sufficient reserves
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            stake_amount.into()
        ));
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );

        // Setup limit price to 99% of current price
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        let limit_price = (current_price.to_num::<f64>() * 990_000_000_f64) as u64;

        // Alpha unstaked - calculated using formula from delta_in()
        let expected_alpha_reduction = (0.00138 * (alpha_in.to_u64() as f64)) as u64;
        let fee: u64 = (expected_alpha_reduction as f64 * 0.003) as u64;

        // Remove stake with slippage safety
        remove_stake_rate_limit_for_tests(&hotkey_account_id, &coldkey_account_id, netuid);
        assert_ok!(SubtensorModule::remove_stake_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            alpha_before / 2.into(),
            limit_price.into(),
            true
        ));
        let alpha_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );

        // Check if stake has decreased properly
        assert_abs_diff_eq!(
            alpha_before - alpha_after,
            AlphaCurrency::from(expected_alpha_reduction + fee),
            epsilon = AlphaCurrency::from(expected_alpha_reduction / 10),
        );
    });
}

#[test]
fn test_remove_stake_limit_fill_or_kill() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let stake_amount = AlphaCurrency::from(300_000_000_000);
        let unstake_amount = AlphaCurrency::from(150_000_000_000);

        // add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            stake_amount,
        );

        // Forse-set alpha in and tao reserve to make price equal 1.5
        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        assert_eq!(current_price, U96F32::from_num(1.5));

        // Setup limit price so that it doesn't drop by more than 10% from current price
        let limit_price = TaoCurrency::from(1_350_000_000);

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
            unstake_amount / 100.into(),
            limit_price.into(),
            false
        ),);
    });
}

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
        let existing_stake = AlphaCurrency::from(36_711_495_953);

        let tao_in = TaoCurrency::from(2_409_892_148_947);
        let alpha_in = AlphaCurrency::from(15_358_708_513_716);

        let tao_staked = TaoCurrency::from(200_000_000);

        //add network
        let netuid = add_dynamic_network(&sn_owner_coldkey, &sn_owner_coldkey);

        // Register hotkey on netuid
        register_ok_neuron(netuid, hotkey_account_id, hotkey_owner_account_id, 0);
        // Check we have zero staked
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
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
            tao_staked.to_u64() + 1_000_000_000,
        );

        // Add stake as new hotkey
        let order = GetAlphaForTao::<Test>::with_amount(tao_staked);
        let expected_alpha = <Test as Config>::SwapInterface::swap(
            netuid.into(),
            order,
            <Test as Config>::SwapInterface::max_price(),
            false,
            true,
        )
        .map(|v| v.amount_paid_out)
        .unwrap_or_default();
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            tao_staked,
        ));

        // Check we have non-zero staked
        assert!(expected_alpha > AlphaCurrency::ZERO);
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid
            ),
            expected_alpha,
            epsilon = expected_alpha / 1000.into()
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_99_999_per_cent_stake_removes_all --exact --show-output
#[test]
fn test_remove_99_9991_per_cent_stake_removes_all() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let hotkey_account_id = U256::from(581337);
        let coldkey_account_id = U256::from(81337);
        let amount = 10_000_000_000;
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Set fee rate to 0 so that alpha fee is not moved to block producer 
        // and the hotkey stake does drop to 0
        pallet_subtensor_swap::FeeRate::<Test>::insert(netuid, 0);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into()
        ));

        // Remove 99.9991% stake
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );
        remove_stake_rate_limit_for_tests(&hotkey_account_id, &coldkey_account_id, netuid);
        let remove_amount = AlphaCurrency::from(
            (U64F64::from_num(alpha) * U64F64::from_num(0.999991)).to_num::<u64>(),
        );
        // we expected the entire stake to be returned
        let (expected_balance, _) = mock::swap_alpha_to_tao(netuid, alpha);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            remove_amount,
        ));

        // Check that all alpha was unstaked and all TAO balance was returned (less fees)
        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            expected_balance.to_u64(),
            epsilon = 10,
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id),
            TaoCurrency::ZERO
        );
        let new_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );
        assert!(new_alpha.is_zero());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_99_9989_per_cent_stake_leaves_a_little --exact --show-output
#[test]
fn test_remove_99_9989_per_cent_stake_leaves_a_little() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1);
        let subnet_owner_hotkey = U256::from(2);
        let hotkey_account_id = U256::from(581337);
        let coldkey_account_id = U256::from(81337);
        let amount = 10_000_000_000;
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

        // Set fee rate to 0 so that alpha fee is not moved to block producer 
        // to avoid false success in this test
        pallet_subtensor_swap::FeeRate::<Test>::insert(netuid, 0);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

        // Stake to hotkey account, and check if the result is ok
        let (_, fee) = mock::swap_tao_to_alpha(netuid, amount.into());
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into()
        ));

        // Remove 99.9989% stake
        remove_stake_rate_limit_for_tests(&hotkey_account_id, &coldkey_account_id, netuid);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );
        let fee =
            mock::swap_alpha_to_tao(netuid, ((alpha.to_u64() as f64 * 0.99) as u64).into()).1 + fee;
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            (U64F64::from_num(alpha.to_u64()) * U64F64::from_num(0.99))
                .to_num::<u64>()
                .into()
        ));

        // Check that all alpha was unstaked and 99% TAO balance was returned (less fees)
        // let fee = <Test as Config>::SwapInterface::approx_fee_amount(netuid.into(), (amount as f64 * 0.99) as u64);
        assert_abs_diff_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey_account_id),
            (amount as f64 * 0.99) as u64 - fee,
            epsilon = amount / 1000,
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id).to_u64(),
            (amount as f64 * 0.01) as u64,
            epsilon = amount / 1000,
        );
        let new_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
        );
        assert_abs_diff_eq!(
            new_alpha,
            AlphaCurrency::from((alpha.to_u64() as f64 * 0.01) as u64),
            epsilon = 10.into()
        );
    });
}

#[test]
fn test_move_stake_limit_partial() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = AlphaCurrency::from(150_000_000_000);
        let move_amount = AlphaCurrency::from(150_000_000_000);

        // add network
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(origin_netuid, hotkey, coldkey, 192213123);
        register_ok_neuron(destination_netuid, hotkey, coldkey, 192213123);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            origin_netuid,
            stake_amount,
        );

        // Forse-set alpha in and tao reserve to make price equal 1.5 on both origin and destination,
        // but there's much more liquidity on destination, so its price wouldn't go up when restaked
        let tao_reserve = TaoCurrency::from(150_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(origin_netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(origin_netuid, alpha_in);
        SubnetTAO::<Test>::insert(destination_netuid, tao_reserve * 100_000.into());
        SubnetAlphaIn::<Test>::insert(destination_netuid, alpha_in * 100_000.into());
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(origin_netuid.into());
        assert_eq!(current_price, U96F32::from_num(1.5));

        // The relative price between origin and destination subnets is 1.
        // Setup limit relative price so that it doesn't drop by more than 1% from current price
        let limit_price = TaoCurrency::from(990_000_000);

        // Move stake with slippage safety - executes partially
        assert_ok!(SubtensorModule::swap_stake_limit(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            origin_netuid,
            destination_netuid,
            move_amount,
            limit_price,
            true,
        ));

        let new_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            origin_netuid,
        );

        assert_abs_diff_eq!(
            new_alpha,
            AlphaCurrency::from(149_000_000_000),
            epsilon = 100_000_000.into()
        );
    });
}

/// cargo test --package pallet-subtensor --lib -- tests::staking::test_unstake_all_hits_liquidity_min --exact --show-output
#[test]
fn test_unstake_all_hits_liquidity_min() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let stake_amount = AlphaCurrency::from(190_000_000_000); // 190 Alpha

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey, coldkey, 192213123);
        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_amount,
        );

        // Setup the Alpha pool so that removing all the Alpha will bring liqudity below the minimum
        let remaining_tao = TaoCurrency::from(u64::from(mock::SwapMinimumReserve::get()) - 1);
        let alpha_reserves = AlphaCurrency::from(stake_amount.to_u64() + 10_000_000);
        mock::setup_reserves(netuid, remaining_tao, alpha_reserves);

        // Try to unstake, but we reduce liquidity too far

        assert_ok!(SubtensorModule::unstake_all(
            RuntimeOrigin::signed(coldkey),
            hotkey,
        ));

        // Expect nothing to be unstaked
        let new_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_abs_diff_eq!(new_alpha, stake_amount, epsilon = AlphaCurrency::ZERO);
    });
}

#[test]
fn test_unstake_all_alpha_hits_liquidity_min() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let stake_amount = 100_000_000_000; // 100 TAO

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey, coldkey, 192213123);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount + ExistentialDeposit::get(),
        );
        // Give the neuron some stake to remove
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            stake_amount.into()
        ));

        // Setup the pool so that removing all the TAO will bring liqudity below the minimum
        let remaining_tao = I96F32::from_num(u64::from(mock::SwapMinimumReserve::get()) - 1)
            .saturating_sub(I96F32::from(1));
        let alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let alpha_reserves = I110F18::from(u64::from(alpha) + 10_000_000);

        let k = I110F18::from_fixed(remaining_tao)
            .saturating_mul(alpha_reserves.saturating_add(I110F18::from(u64::from(alpha))));
        let tao_reserves = k.safe_div(alpha_reserves);

        mock::setup_reserves(
            netuid,
            (tao_reserves.to_num::<u64>() / 100_u64).into(),
            alpha_reserves.to_num::<u64>().into(),
        );

        // Try to unstake, but we reduce liquidity too far

        assert_err!(
            SubtensorModule::unstake_all_alpha(RuntimeOrigin::signed(coldkey), hotkey),
            Error::<Test>::AmountTooLow
        );

        // Expect nothing to be unstaked
        let new_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(new_alpha, alpha);
    });
}

#[test]
fn test_unstake_all_alpha_works() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let stake_amount = 190_000_000_000; // 190 TAO

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey, coldkey, 192213123);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        // Give the neuron some stake to remove
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            stake_amount.into()
        ));

        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        // Setup the pool so that removing all the TAO will keep liq above min
        mock::setup_reserves(
            netuid,
            (stake_amount * 10).into(),
            (stake_amount * 100).into(),
        );

        // Unstake all alpha to root
        assert_ok!(SubtensorModule::unstake_all_alpha(
            RuntimeOrigin::signed(coldkey),
            hotkey,
        ));

        let new_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_abs_diff_eq!(new_alpha, AlphaCurrency::ZERO, epsilon = 1_000.into());
        let new_root = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
        );
        assert!(new_root > 100_000.into());
    });
}

#[test]
fn test_unstake_all_works() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let stake_amount = 190_000_000_000; // 190 TAO

        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey, coldkey, 192213123);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        // Give the neuron some stake to remove
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            stake_amount.into()
        ));

        // Setup the pool so that removing all the TAO will keep liq above min
        mock::setup_reserves(
            netuid,
            (stake_amount * 10).into(),
            (stake_amount * 100).into(),
        );
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        // Unstake all alpha to free balance
        assert_ok!(SubtensorModule::unstake_all(
            RuntimeOrigin::signed(coldkey),
            hotkey,
        ));

        let new_alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_abs_diff_eq!(new_alpha, AlphaCurrency::ZERO, epsilon = 1_000.into());
        let new_balance = SubtensorModule::get_coldkey_balance(&coldkey);
        assert!(new_balance > 100_000);
    });
}

#[test]
fn test_stake_into_subnet_ok() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let amount = 100_000_000;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Forse-set alpha in and tao reserve to make price equal 0.01
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(1_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .to_num::<f64>();

        // Initialize swap v3
        let order = GetAlphaForTao::<Test>::with_amount(0);
        assert_ok!(<tests::mock::Test as pallet::Config>::SwapInterface::swap(
            netuid.into(),
            order,
            TaoCurrency::MAX,
            false,
            true
        ));

        // Add stake with slippage safety and check if the result is ok
        assert_ok!(SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid,
            amount.into(),
            TaoCurrency::MAX,
            false,
            false,
        ));
        let fee_rate = pallet_subtensor_swap::FeeRate::<Test>::get(NetUid::from(netuid)) as f64
            / u16::MAX as f64;
        let expected_stake = (amount as f64) * (1. - fee_rate) / current_price;

        // Check if stake has increased
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                .to_u64() as f64,
            expected_stake,
            epsilon = expected_stake / 1000.,
        );
    });
}

#[test]
fn test_stake_into_subnet_low_amount() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let amount = 10;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Forse-set alpha in and tao reserve to make price equal 0.01
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(1_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .to_num::<f64>();

        // Initialize swap v3
        let order = GetAlphaForTao::<Test>::with_amount(0);
        assert_ok!(<tests::mock::Test as pallet::Config>::SwapInterface::swap(
            netuid.into(),
            order,
            TaoCurrency::MAX,
            false,
            true
        ));

        // Add stake with slippage safety and check if the result is ok
        assert_ok!(SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid,
            amount.into(),
            TaoCurrency::MAX,
            false,
            false,
        ));
        let expected_stake = (amount as f64) * 0.997 / current_price;

        // Check if stake has increased
        assert_abs_diff_eq!(
            u64::from(SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid
            )) as f64,
            expected_stake,
            epsilon = expected_stake / 100.
        );
    });
}

#[test]
fn test_unstake_from_subnet_low_amount() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let amount = 10;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Forse-set alpha in and tao reserve to make price equal 0.01
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(1_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);

        // Initialize swap v3
        let order = GetAlphaForTao::<Test>::with_amount(0);
        assert_ok!(<tests::mock::Test as pallet::Config>::SwapInterface::swap(
            netuid.into(),
            order,
            TaoCurrency::MAX,
            false,
            true
        ));

        // Add stake and check if the result is ok
        assert_ok!(SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid,
            amount.into(),
            TaoCurrency::MAX,
            false,
            false,
        ));

        // Remove stake
        let alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::unstake_from_subnet(
            &hotkey,
            &coldkey,
            netuid,
            alpha,
            TaoCurrency::ZERO,
            false,
        ));

        // Check if stake is zero
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid),
            AlphaCurrency::ZERO,
        );
    });
}

#[test]
fn test_stake_into_subnet_prohibitive_limit() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(4);
        let amount = 100_000_000;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount);

        // Forse-set alpha in and tao reserve to make price equal 0.01
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(1_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);

        // Initialize swap v3
        let order = GetAlphaForTao::<Test>::with_amount(0);
        assert_ok!(<tests::mock::Test as pallet::Config>::SwapInterface::swap(
            netuid.into(),
            order,
            TaoCurrency::MAX,
            false,
            true
        ));

        // Add stake and check if the result is ok
        // Use prohibitive limit price
        assert_err!(
            SubtensorModule::add_stake_limit(
                RuntimeOrigin::signed(coldkey),
                owner_hotkey,
                netuid,
                amount.into(),
                TaoCurrency::ZERO,
                true,
            ),
            DispatchError::from(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded)
        );

        // Check if stake has NOT increased
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &owner_hotkey,
                &coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );

        // Check if balance has NOT decreased
        assert_eq!(SubtensorModule::get_coldkey_balance(&coldkey), amount);
    });
}

#[test]
fn test_unstake_from_subnet_prohibitive_limit() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(4);
        let amount = 100_000_000;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount);

        // Forse-set alpha in and tao reserve to make price equal 0.01
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(1_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);

        // Initialize swap v3
        let order = GetAlphaForTao::<Test>::with_amount(0);
        assert_ok!(<tests::mock::Test as pallet::Config>::SwapInterface::swap(
            netuid.into(),
            order,
            TaoCurrency::MAX,
            false,
            true
        ));

        // Add stake and check if the result is ok
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hotkey,
            &coldkey,
            netuid,
            amount.into(),
            TaoCurrency::MAX,
            false,
            false,
        ));

        // Remove stake
        // Use prohibitive limit price
        let balance_before = SubtensorModule::get_coldkey_balance(&coldkey);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &owner_hotkey,
            &coldkey,
            netuid,
        );
        assert_err!(
            SubtensorModule::remove_stake_limit(
                RuntimeOrigin::signed(coldkey),
                owner_hotkey,
                netuid,
                alpha,
                TaoCurrency::MAX,
                true,
            ),
            DispatchError::from(pallet_subtensor_swap::Error::<Test>::PriceLimitExceeded)
        );

        // Check if stake has NOT decreased
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &owner_hotkey,
                &coldkey,
                netuid
            ),
            alpha
        );

        // Check if balance has NOT increased
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&coldkey),
            balance_before,
        );
    });
}

#[test]
fn test_unstake_full_amount() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(4);
        let amount = 100_000_000;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, amount);

        // Forse-set alpha in and tao reserve to make price equal 0.01
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(1_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);

        // Initialize swap v3
        let order = GetAlphaForTao::<Test>::with_amount(0);
        assert_ok!(<tests::mock::Test as pallet::Config>::SwapInterface::swap(
            netuid.into(),
            order,
            TaoCurrency::MAX,
            false,
            true
        ));

        // Add stake and check if the result is ok
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hotkey,
            &coldkey,
            netuid,
            amount.into(),
            TaoCurrency::MAX,
            false,
            false,
        ));

        // Remove stake
        // Use prohibitive limit price
        let balance_before = SubtensorModule::get_coldkey_balance(&coldkey);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &owner_hotkey,
            &coldkey,
            netuid,
        );
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            owner_hotkey,
            netuid,
            alpha,
        ));

        // Check if stake is zero
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &owner_hotkey,
                &coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );

        // Check if balance has increased accordingly
        let balance_after = SubtensorModule::get_coldkey_balance(&coldkey);
        let actual_balance_increase = (balance_after - balance_before) as f64;
        let fee_rate = pallet_subtensor_swap::FeeRate::<Test>::get(NetUid::from(netuid)) as f64
            / u16::MAX as f64;
        let expected_balance_increase = amount as f64 * (1. - fee_rate) / (1. + fee_rate);
        assert_abs_diff_eq!(
            actual_balance_increase,
            expected_balance_increase,
            epsilon = expected_balance_increase / 10_000.
        );
    });
}

/// Test correctness of swap fees:
///   1. TAO is not minted or burned
///   2. Fees match FeeRate
#[test]
fn test_swap_fees_tao_correctness() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(4);
        let amount = 1_000_000_000;
        let owner_balance_before = amount * 10;
        let user_balance_before = amount * 100;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        SubtensorModule::add_balance_to_coldkey_account(&owner_coldkey, owner_balance_before);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, user_balance_before);
        let fee_rate = pallet_subtensor_swap::FeeRate::<Test>::get(NetUid::from(netuid)) as f64
            / u16::MAX as f64;

        // Forse-set alpha in and tao reserve to make price equal 0.25
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(400_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);

        // Check starting "total TAO"
        let total_tao_before =
            user_balance_before + owner_balance_before + SubnetTAO::<Test>::get(netuid).to_u64();

        // Get alpha for owner
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(owner_coldkey),
            owner_hotkey,
            netuid,
            amount.into(),
        ));
        let mut fees = (fee_rate * amount as f64) as u64;

        // Add owner coldkey Alpha as concentrated liquidity
        // between current price current price + 0.01
        let current_price =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into())
                .to_num::<f64>()
                + 0.0001;
        let limit_price = current_price + 0.01;

        // Limit-buy and then sell all alpha for user to hit owner liquidity
        assert_ok!(SubtensorModule::add_stake_limit(
            RuntimeOrigin::signed(coldkey),
            owner_hotkey,
            netuid,
            amount.into(),
            ((limit_price * u64::MAX as f64) as u64).into(),
            true
        ));
        fees += (fee_rate * amount as f64) as u64;

        let user_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &owner_hotkey,
            &coldkey,
            netuid,
        );
        remove_stake_rate_limit_for_tests(&owner_hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            owner_hotkey,
            netuid,
            user_alpha,
        ));
        // Do not add fees because selling fees are in alpha

        // Check ending "total TAO"
        let owner_balance_after = SubtensorModule::get_coldkey_balance(&owner_coldkey);
        let user_balance_after = SubtensorModule::get_coldkey_balance(&coldkey);
        let total_tao_after = user_balance_after
            + owner_balance_after
            + SubnetTAO::<Test>::get(netuid).to_u64()
            + fees;

        // Total TAO does not change, leave some epsilon for rounding
        assert_abs_diff_eq!(total_tao_before, total_tao_after, epsilon = 2);
    });
}

#[test]
fn test_increase_stake_for_hotkey_and_coldkey_on_subnet_adds_to_staking_hotkeys_map() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let coldkey1 = U256::from(2);
        let hotkey = U256::from(3);

        let netuid = NetUid::from(1);
        let stake_amount = 100_000_000_000;

        // Check no entry in the staking hotkeys map
        assert!(!StakingHotkeys::<Test>::contains_key(coldkey));
        // insert manually
        StakingHotkeys::<Test>::insert(coldkey, Vec::<U256>::new());
        // check entry has no hotkey
        assert!(!StakingHotkeys::<Test>::get(coldkey).contains(&hotkey));

        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            netuid,
            stake_amount.into(),
        );

        // Check entry exists in the staking hotkeys map
        assert!(StakingHotkeys::<Test>::contains_key(coldkey));
        // check entry has hotkey
        assert!(StakingHotkeys::<Test>::get(coldkey).contains(&hotkey));

        // Check no entry in the staking hotkeys map for coldkey1
        assert!(!StakingHotkeys::<Test>::contains_key(coldkey1));

        // Run increase stake for hotkey and coldkey1 on subnet
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey1,
            netuid,
            stake_amount.into(),
        );

        // Check entry exists in the staking hotkeys map for coldkey1
        assert!(StakingHotkeys::<Test>::contains_key(coldkey1));
        // check entry has hotkey
        assert!(StakingHotkeys::<Test>::get(coldkey1).contains(&hotkey));
    });
}

#[test]
fn test_remove_stake_full_limit_ok() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(2);
        let stake_amount = AlphaCurrency::from(10_000_000_000);

        // add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            stake_amount,
        );

        let tao_reserve = TaoCurrency::from(100_000_000_000_u64);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        let limit_price = TaoCurrency::from(90_000_000);

        // Remove stake with slippage safety
        assert_ok!(SubtensorModule::remove_stake_full_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            Some(limit_price),
        ));

        // Check if stake has decreased to zero
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid
            ),
            AlphaCurrency::ZERO
        );

        let new_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_abs_diff_eq!(new_balance, 9_086_700_000, epsilon = 1_000_000);
    });
}

#[test]
fn test_remove_stake_full_limit_fails_slippage_too_high() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(2);
        let stake_amount = AlphaCurrency::from(10_000_000_000);

        // add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            stake_amount,
        );

        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        let invalid_limit_price = TaoCurrency::from(910_000_000);

        // Remove stake with slippage safety
        assert_err!(
            SubtensorModule::remove_stake_full_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                Some(invalid_limit_price),
            ),
            Error::<Test>::SlippageTooHigh
        );
    });
}

#[test]
fn test_remove_stake_full_limit_ok_with_no_limit_price() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(2);
        let stake_amount = AlphaCurrency::from(10_000_000_000);

        // add network
        let netuid = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give the neuron some stake to remove
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            stake_amount,
        );

        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(100_000_000_000);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_in);

        // Remove stake with slippage safety
        assert_ok!(SubtensorModule::remove_stake_full_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            None,
        ));

        // Check if stake has decreased to zero
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid
            ),
            AlphaCurrency::ZERO
        );

        let new_balance = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        assert_abs_diff_eq!(new_balance, 9_086_700_000, epsilon = 1_000_000);
    });
}

/// This test verifies that minimum stake amount is sufficient to move price and apply
/// non-zero staking fees
#[test]
fn test_default_min_stake_sufficiency() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(4);
        let min_tao_stake = DefaultMinStake::<Test>::get().to_u64() * 2;
        let amount = min_tao_stake;
        let owner_balance_before = amount * 10;
        let user_balance_before = amount * 100;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        SubtensorModule::add_balance_to_coldkey_account(&owner_coldkey, owner_balance_before);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, user_balance_before);
        let fee_rate = pallet_subtensor_swap::FeeRate::<Test>::get(NetUid::from(netuid)) as f64
            / u16::MAX as f64;

        // Set some extreme, but realistic TAO and Alpha reserves to minimize slippage
        // 1% of TAO max supply
        // 0.01 Alpha price
        let tao_reserve = TaoCurrency::from(210_000_000_000_000);
        let alpha_in = AlphaCurrency::from(21_000_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);
        let current_price_before =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());

        // Stake and unstake
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            owner_hotkey,
            netuid,
            amount.into(),
        ));
        let fee_stake = (fee_rate * amount as f64) as u64;
        let current_price_after_stake =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        remove_stake_rate_limit_for_tests(&owner_hotkey, &coldkey, netuid);
        let user_alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &owner_hotkey,
            &coldkey,
            netuid,
        );
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            owner_hotkey,
            netuid,
            user_alpha,
        ));
        let fee_unstake = (fee_rate * user_alpha.to_u64() as f64) as u64;
        let current_price_after_unstake =
            <Test as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());

        assert!(fee_stake > 0);
        assert!(fee_unstake > 0);
        assert!(current_price_after_stake > current_price_before);
        assert!(current_price_after_stake > current_price_after_unstake);
    });
}

#[test]
fn test_large_swap() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let coldkey = U256::from(100);

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_000);
        let tao = TaoCurrency::from(100_000_000u64);
        let alpha = AlphaCurrency::from(1_000_000_000_000_000_u64);
        SubnetTAO::<Test>::insert(netuid, tao);
        SubnetAlphaIn::<Test>::insert(netuid, alpha);

        // Force the swap to initialize
        <Test as pallet::Config>::SwapInterface::init_swap(netuid, None);

        let swap_amount = TaoCurrency::from(100_000_000_000_000);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            owner_hotkey,
            netuid,
            swap_amount,
        ));
    });
}

#[test]
fn test_stake_rate_limits() {
    new_test_ext(0).execute_with(|| {
        // Create subnet and accounts.
        let subnet_owner_coldkey = U256::from(10);
        let subnet_owner_hotkey = U256::from(20);
        let hot1 = U256::from(1);
        let cold1 = U256::from(3);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        let fee = DefaultMinStake::<Test>::get().to_u64();
        let init_balance = amount + fee + ExistentialDeposit::get();

        register_ok_neuron(netuid, hot1, cold1, 0);
        Delegates::<Test>::insert(hot1, SubtensorModule::get_min_delegate_take());
        assert_eq!(SubtensorModule::get_owning_coldkey_for_hotkey(&hot1), cold1);

        SubtensorModule::add_balance_to_coldkey_account(&cold1, init_balance);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(cold1),
            hot1,
            netuid,
            (amount + fee).into()
        ));

        assert_err!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(cold1),
                hot1,
                netuid,
                amount.into()
            ),
            Error::<Test>::StakingOperationRateLimitExceeded
        );

        // Test limit clear each block
        assert!(StakingOperationRateLimiter::<Test>::contains_key((
            hot1, cold1, netuid
        )));

        next_block();

        assert!(!StakingOperationRateLimiter::<Test>::contains_key((
            hot1, cold1, netuid
        )));
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_add_root_updates_counters --exact --show-output
#[test]
fn test_add_root_updates_counters() {
    new_test_ext(0).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        add_network(NetUid::ROOT, 10, 0);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(coldkey_account_id).clone(),
            hotkey_account_id,
        ));
        let stake_amount = 1_000_000_000;

        // Give it some $$$ in his coldkey balance
        let initial_balance = stake_amount + ExistentialDeposit::get();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Setup SubnetAlphaIn (because we are going to stake)
        SubnetAlphaIn::<Test>::insert(NetUid::ROOT, AlphaCurrency::from(stake_amount));

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            NetUid::ROOT,
            stake_amount.into()
        ));

        // Check if stake has increased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, stake_amount.into());

        // Check if total stake has increased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), stake_amount.into());

        // SubnetTAO updated
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), stake_amount.into());

        // SubnetAlphaIn updated
        assert_eq!(SubnetAlphaIn::<Test>::get(NetUid::ROOT), 0.into());

        // SubnetAlphaOut updated
        assert_eq!(
            SubnetAlphaOut::<Test>::get(NetUid::ROOT),
            stake_amount.into()
        );

        // SubnetVolume updated
        assert_eq!(
            SubnetVolume::<Test>::get(NetUid::ROOT),
            stake_amount as u128
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_remove_root_updates_counters --exact --show-output
#[test]
fn test_remove_root_updates_counters() {
    new_test_ext(0).execute_with(|| {
        let hotkey_account_id = U256::from(561337);
        let coldkey_account_id = U256::from(61337);
        add_network(NetUid::ROOT, 10, 0);
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(coldkey_account_id).clone(),
            hotkey_account_id,
        ));
        let stake_amount = 1_000_000_000;

        // Give it some $$$ in his coldkey balance
        let initial_balance = stake_amount + ExistentialDeposit::get();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);

        // Setup existing stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            NetUid::ROOT,
            stake_amount.into(),
        );

        // Setup TotalStake, SubnetAlphaOut and SubnetTAO (because we are going to unstake)
        TotalStake::<Test>::set(TaoCurrency::from(stake_amount));
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(stake_amount));
        SubnetAlphaOut::<Test>::insert(NetUid::ROOT, AlphaCurrency::from(stake_amount));

        // Stake to hotkey account, and check if the result is ok
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            NetUid::ROOT,
            stake_amount.into()
        ));

        // Check if stake has been decreased
        let new_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey_account_id);
        assert_eq!(new_stake, 0.into());

        // Check if total stake has decreased accordingly.
        assert_eq!(SubtensorModule::get_total_stake(), 0.into());

        // SubnetTAO updated
        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), 0.into());

        // SubnetAlphaIn updated
        assert_eq!(
            SubnetAlphaIn::<Test>::get(NetUid::ROOT),
            stake_amount.into()
        );

        // SubnetAlphaOut updated
        assert_eq!(SubnetAlphaOut::<Test>::get(NetUid::ROOT), 0.into());

        // SubnetVolume updated
        assert_eq!(
            SubnetVolume::<Test>::get(NetUid::ROOT),
            stake_amount as u128
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::staking::test_staking_records_flow --exact --show-output
#[test]
fn test_staking_records_flow() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let amount = 100_000_000;

        // add network
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        // Forse-set alpha in and tao reserve to make price equal 0.01
        let tao_reserve = TaoCurrency::from(100_000_000_000);
        let alpha_in = AlphaCurrency::from(1_000_000_000_000);
        mock::setup_reserves(netuid, tao_reserve, alpha_in);

        // Initialize swap v3
        SubtensorModule::swap_tao_for_alpha(
            netuid,
            TaoCurrency::ZERO,
            1_000_000_000_000.into(),
            false,
        )
        .unwrap();

        // Add stake with slippage safety and check if the result is ok
        assert_ok!(SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid,
            amount.into(),
            TaoCurrency::MAX,
            false,
            false,
        ));
        let fee_rate = pallet_subtensor_swap::FeeRate::<Test>::get(NetUid::from(netuid)) as f64
            / u16::MAX as f64;
        let expected_flow = (amount as f64) * (1. - fee_rate);

        // Check that flow has been recorded (less unstaking fees)
        assert_abs_diff_eq!(
            SubnetTaoFlow::<Test>::get(netuid),
            expected_flow as i64,
            epsilon = 1_i64
        );

        // Remove stake
        let alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::unstake_from_subnet(
            &hotkey,
            &coldkey,
            netuid,
            alpha,
            TaoCurrency::ZERO,
            false,
        ));

        // Check that outflow has been recorded (less unstaking fees)
        let expected_unstake_fee = expected_flow * fee_rate;
        assert_abs_diff_eq!(
            SubnetTaoFlow::<Test>::get(netuid),
            expected_unstake_fee as i64,
            epsilon = (expected_unstake_fee / 100.0) as i64
        );
    });
}
