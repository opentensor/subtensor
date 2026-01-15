#![allow(clippy::unwrap_used)]
use super::mock::*;
use crate::subnets::symbols::{DEFAULT_SYMBOL, SYMBOLS};
use crate::*;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::Config;
use sp_core::U256;
use subtensor_runtime_common::{AlphaCurrency, TaoCurrency};

use super::mock;

/***************************
  pub fn do_start_call() tests
*****************************/

#[test]
fn test_do_start_call_ok() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);

        add_network_without_emission_block(netuid, tempo, 0);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);
        mock::setup_reserves(netuid, 1_000_000_000.into(), 1_000_000_000.into());

        // account 0 is the default owner for any subnet
        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        let block_number = System::block_number() + StartCallDelay::<Test>::get();
        System::set_block_number(block_number);

        assert_ok!(SubtensorModule::start_call(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid
        ));

        assert_eq!(
            FirstEmissionBlockNumber::<Test>::get(netuid),
            Some(block_number + 1)
        );
    });
}

#[test]
fn test_do_start_call_fail_with_not_existed_subnet() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey_account_id = U256::from(0);
        assert_noop!(
            SubtensorModule::start_call(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                netuid
            ),
            Error::<Test>::SubnetNotExists
        );
    });
}

#[test]
fn test_do_start_call_fail_not_owner() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let wrong_owner_account_id = U256::from(2);
        let burn_cost = TaoCurrency::from(1000);
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        mock::setup_reserves(netuid, 1_000_000_000.into(), 1_000_000_000.into());
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        add_network_without_emission_block(netuid, tempo, 0);

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        System::set_block_number(System::block_number() + StartCallDelay::<Test>::get());

        assert_noop!(
            SubtensorModule::start_call(
                <<Test as Config>::RuntimeOrigin>::signed(wrong_owner_account_id),
                netuid
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn test_do_start_call_can_start_now() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let burn_cost = TaoCurrency::from(1000);
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        mock::setup_reserves(netuid, 1_000_000_000.into(), 1_000_000_000.into());
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        add_network_without_emission_block(netuid, tempo, 0);

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        assert_ok!(SubtensorModule::start_call(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid
        ));
    });
}

#[test]
fn test_do_start_call_fail_for_set_again() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let hotkey_account_id = U256::from(1);
        let burn_cost = TaoCurrency::from(1000);

        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);

        mock::setup_reserves(netuid, 1_000_000_000.into(), 1_000_000_000.into());

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        let block_number = System::block_number() + StartCallDelay::<Test>::get();
        System::set_block_number(block_number);

        assert_ok!(SubtensorModule::start_call(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid
        ));

        assert_noop!(
            SubtensorModule::start_call(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                netuid
            ),
            Error::<Test>::FirstEmissionBlockNumberAlreadySet
        );
    });
}

#[test]
fn test_do_start_call_ok_with_same_block_number_after_coinbase() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);

        add_network_without_emission_block(netuid, tempo, 0);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);
        mock::setup_reserves(netuid, 1_000_000_000.into(), 1_000_000_000.into());

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        let block_number = System::block_number() + StartCallDelay::<Test>::get();
        System::set_block_number(block_number);

        assert_ok!(SubtensorModule::start_call(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid
        ));

        assert_eq!(
            FirstEmissionBlockNumber::<Test>::get(netuid),
            Some(block_number + 1)
        );

        step_block(tempo);
        match FirstEmissionBlockNumber::<Test>::get(netuid) {
            Some(new_emission_block_number) => {
                assert_eq!(new_emission_block_number, block_number + 1)
            }
            None => assert!(FirstEmissionBlockNumber::<Test>::get(netuid).is_some()),
        }
    });
}

#[test]
fn test_register_network_min_burn_at_default() {
    new_test_ext(1).execute_with(|| {
        let sn_owner_coldkey = U256::from(0);
        let sn_owner_hotkey = U256::from(1);
        let cost = SubtensorModule::get_network_lock_cost();

        // Give coldkey enough for lock
        SubtensorModule::add_balance_to_coldkey_account(&sn_owner_coldkey, cost.into());

        // Register network
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner_coldkey),
            sn_owner_hotkey
        ));

        // Get netuid of the new network
        let netuid = match last_event() {
            RuntimeEvent::SubtensorModule(Event::<Test>::NetworkAdded(netuid, _)) => netuid,
            _ => panic!("Expected NetworkAdded event"),
        };

        // Check min burn is set to default
        assert_eq!(MinBurn::<Test>::get(netuid), InitialMinBurn::get().into());

        // Check registration allowed
        assert!(NetworkRegistrationAllowed::<Test>::get(netuid));
        assert!(NetworkPowRegistrationAllowed::<Test>::get(netuid));
    });
}

#[test]
fn test_register_network_use_symbol_for_subnet_if_available() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::set_max_subnets(SYMBOLS.len() as u16);
        for i in 0..(SYMBOLS.len() - 1) {
            let coldkey = U256::from(1_000_000 + i);
            let hotkey = U256::from(2_000_000 + i);
            let cost = SubtensorModule::get_network_lock_cost();
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, cost.into());

            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey
            ));

            let netuid = match last_event() {
                RuntimeEvent::SubtensorModule(Event::<Test>::NetworkAdded(netuid, _)) => netuid,
                _ => panic!("Expected NetworkAdded event"),
            };

            // Ensure the symbol correspond to the netuid has been set
            let expected_symbol = SYMBOLS.get(usize::from(u16::from(netuid))).unwrap();
            assert_eq!(TokenSymbol::<Test>::get(netuid), *expected_symbol);

            // Check registration allowed
            assert!(NetworkRegistrationAllowed::<Test>::get(netuid));
            assert!(NetworkPowRegistrationAllowed::<Test>::get(netuid));
        }
    });
}

#[test]
fn test_register_network_use_next_available_symbol_if_symbol_for_subnet_is_taken() {
    new_test_ext(1).execute_with(|| {
        // Register 50 networks (additionnaly to the root network)
        for i in 0..50 {
            let coldkey = U256::from(1_000_000 + i);
            let hotkey = U256::from(2_000_000 + i);
            let cost = SubtensorModule::get_network_lock_cost();
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, cost.into());

            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey
            ));

            let netuid = match last_event() {
                RuntimeEvent::SubtensorModule(Event::<Test>::NetworkAdded(netuid, _)) => netuid,
                _ => panic!("Expected NetworkAdded event"),
            };

            // Ensure the symbol correspond to the netuid has been set
            let expected_symbol = SYMBOLS.get(usize::from(u16::from(netuid))).unwrap();
            assert_eq!(TokenSymbol::<Test>::get(netuid), *expected_symbol);

            // Check registration allowed
            assert!(NetworkRegistrationAllowed::<Test>::get(netuid));
            assert!(NetworkPowRegistrationAllowed::<Test>::get(netuid));
        }

        // Swap some of the network symbol for the network 25 to network 51 symbol (not registered yet)
        TokenSymbol::<Test>::insert(NetUid::from(25), SYMBOLS.get(51).unwrap().to_vec());

        // Register a new network
        let coldkey = U256::from(1_000_000 + 50);
        let hotkey = U256::from(2_000_000 + 50);
        let cost = SubtensorModule::get_network_lock_cost();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, cost.into());

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));

        // Get netuid of the new network
        let netuid = match last_event() {
            RuntimeEvent::SubtensorModule(Event::<Test>::NetworkAdded(netuid, _)) => netuid,
            _ => panic!("Expected NetworkAdded event"),
        };

        // We expect the symbol to be the one that was previously taken by network 25, before it was swapped
        let expected_symbol = SYMBOLS.get(25).unwrap();
        assert_eq!(TokenSymbol::<Test>::get(netuid), *expected_symbol);
    });
}

#[test]
fn test_register_network_use_default_symbol_if_all_symbols_are_taken() {
    new_test_ext(1).execute_with(|| {
        // Register networks until we have exhausted all symbols
        SubtensorModule::set_max_subnets(SYMBOLS.len() as u16);
        for i in 0..(SYMBOLS.len() - 1) {
            let coldkey = U256::from(1_000_000 + i);
            let hotkey = U256::from(2_000_000 + i);
            let cost = SubtensorModule::get_network_lock_cost();
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, cost.into());

            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey
            ));
        }

        // Register a new network
        let coldkey = U256::from(1_000_000 + 50);
        let hotkey = U256::from(2_000_000 + 50);
        let cost = SubtensorModule::get_network_lock_cost();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, cost.into());

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey
        ));

        // Get netuid of the new network
        let netuid = match last_event() {
            RuntimeEvent::SubtensorModule(Event::<Test>::NetworkAdded(netuid, _)) => netuid,
            _ => panic!("Expected NetworkAdded event"),
        };
        assert_eq!(netuid, NetUid::from(SYMBOLS.len() as u16));

        // We expect the symbol to be the default symbol
        assert_eq!(TokenSymbol::<Test>::get(netuid), *DEFAULT_SYMBOL);

        // Check registration allowed
        assert!(NetworkRegistrationAllowed::<Test>::get(netuid));
        assert!(NetworkPowRegistrationAllowed::<Test>::get(netuid));
    });
}
// cargo test --package pallet-subtensor --lib -- tests::subnet::test_subtoken_enable --exact --show-output
#[test]
fn test_subtoken_enable() {
    // ensure_subtoken_enabled
    new_test_ext(1).execute_with(|| {
        let account = U256::from(0);
        let netuid = NetUid::from(1);
        // let to_be_set: u64 = 10
        add_network_disable_subtoken(netuid, 10, 0);
        assert!(!SubtokenEnabled::<Test>::get(netuid));

        let block_number = System::block_number() + StartCallDelay::<Test>::get();
        System::set_block_number(block_number);

        assert_ok!(SubtensorModule::start_call(
            <<Test as Config>::RuntimeOrigin>::signed(account),
            netuid
        ));

        assert!(SubtokenEnabled::<Test>::get(netuid));
    });
}

// cargo test --package pallet-subtensor --lib --
// tests::subnet::test_subtoken_enable_reject_trading_before_enable --exact --show-output
#[allow(clippy::unwrap_used)]
#[test]
fn test_subtoken_enable_reject_trading_before_enable() {
    // ensure_subtoken_enabled
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(2);
        let hotkey_account_2_id: U256 = U256::from(3);
        let amount = DefaultMinStake::<Test>::get().to_u64() * 10;

        let stake_bal = AlphaCurrency::from(10_000_000_000); // 10 Alpha

        let limit_price = TaoCurrency::from(1_000_000_000); // not important

        add_network_disable_subtoken(netuid, 10, 0);
        add_network_disable_subtoken(netuid2, 10, 0);

        assert!(!SubtokenEnabled::<Test>::get(netuid));
        assert!(!SubtokenEnabled::<Test>::get(netuid2));

        // Set liq high enough to not trigger other errors
        SubnetTAO::<Test>::set(netuid, TaoCurrency::from(20_000_000_000));
        SubnetAlphaIn::<Test>::set(netuid, AlphaCurrency::from(20_000_000_000));

        // Register so staking *could* work
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 0);
        register_ok_neuron(netuid2, hotkey_account_id, coldkey_account_id, 100);
        register_ok_neuron(netuid, hotkey_account_2_id, coldkey_account_id, 0);
        register_ok_neuron(netuid2, hotkey_account_2_id, coldkey_account_id, 100);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10_000);

        // Give some stake
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account_id,
            &coldkey_account_id,
            netuid,
            stake_bal,
        );

        // all trading extrinsic should be rejected.
        assert_noop!(
            SubtensorModule::add_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into()
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::add_stake_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into(),
                limit_price,
                false
            ),
            Error::<Test>::SubtokenDisabled
        );

        // For unstake_all the result is Ok, but the
        // operation is not performed.
        assert_ok!(SubtensorModule::unstake_all(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id
        ));
        // Check that the stake is still the same
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid
            ),
            stake_bal
        );

        // For unstake_all_alpha, the result is AmountTooLow because no re-staking happens.
        assert_noop!(
            SubtensorModule::unstake_all_alpha(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id
            ),
            Error::<Test>::AmountTooLow
        );
        // Check that the stake is still the same
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey_account_id,
                &coldkey_account_id,
                netuid
            ),
            stake_bal
        );

        SubtensorModule::remove_stake_limit(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount.into(),
            limit_price,
            false,
        )
        .unwrap();

        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount.into()
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::recycle_alpha(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                amount.into(),
                netuid
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::burn_alpha(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                amount.into(),
                netuid
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::move_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                hotkey_account_2_id,
                netuid,
                netuid2,
                amount.into(),
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::transfer_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                hotkey_account_2_id,
                netuid,
                netuid2,
                amount.into(),
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::swap_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                netuid2,
                amount.into(),
            ),
            Error::<Test>::SubtokenDisabled
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::subnet::test_subtoken_enable_trading_ok_with_enable --exact --show-output
#[test]
fn test_subtoken_enable_trading_ok_with_enable() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(2);
        let hotkey_account_2_id: U256 = U256::from(3);
        // stake big enough
        let stake_amount = DefaultMinStake::<Test>::get() * 10000.into();
        // unstake, transfer, swap just very little
        let unstake_amount = AlphaCurrency::from(DefaultMinStake::<Test>::get().to_u64() * 10);

        add_network(netuid, 10, 0);
        add_network(netuid2, 10, 0);

        let reserve = stake_amount.to_u64() * 1000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());
        mock::setup_reserves(netuid2, reserve.into(), reserve.into());
        SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(reserve));
        SubnetAlphaOut::<Test>::insert(netuid2, AlphaCurrency::from(reserve));

        // Register so staking works
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 0);
        register_ok_neuron(netuid2, hotkey_account_id, coldkey_account_id, 100);
        register_ok_neuron(netuid, hotkey_account_2_id, coldkey_account_id, 0);
        register_ok_neuron(netuid2, hotkey_account_2_id, coldkey_account_id, 100);

        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            stake_amount.to_u64() * 10,
        );

        // all trading extrinsic should be possible now that subtoken is enabled.
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            stake_amount
        ));

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid2,
            stake_amount
        ));

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_2_id,
            netuid,
            stake_amount
        ));

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_2_id,
            netuid2,
            stake_amount
        ));

        remove_stake_rate_limit_for_tests(&hotkey_account_id, &coldkey_account_id, netuid);

        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            unstake_amount
        ));

        assert_ok!(SubtensorModule::recycle_alpha(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            unstake_amount,
            netuid
        ));

        assert_ok!(SubtensorModule::burn_alpha(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            unstake_amount,
            netuid
        ));

        assert_ok!(SubtensorModule::move_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            hotkey_account_2_id,
            netuid,
            netuid2,
            unstake_amount,
        ));

        remove_stake_rate_limit_for_tests(&hotkey_account_2_id, &coldkey_account_id, netuid);

        assert_ok!(SubtensorModule::transfer_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            hotkey_account_2_id,
            netuid,
            netuid2,
            unstake_amount,
        ));

        assert_ok!(SubtensorModule::swap_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            netuid2,
            unstake_amount,
        ));

        assert_ok!(SubtensorModule::unstake_all_alpha(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
        ));
    });
}

// cargo test --package pallet-subtensor --lib -- tests::subnet::test_subtoken_enable_ok_for_burn_register_before_enable --exact --show-output
#[test]
fn test_subtoken_enable_ok_for_burn_register_before_enable() {
    // ensure_subtoken_enabled
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(2);
        let hotkey_account_2_id: U256 = U256::from(3);

        let burn_cost = TaoCurrency::from(1000);
        // Set the burn cost
        SubtensorModule::set_burn(netuid, burn_cost);
        // Add the networks with subtoken disabled
        add_network_disable_subtoken(netuid, 10, 0);
        add_network_disable_subtoken(netuid2, 10, 0);
        // Give enough to burned register
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey_account_id,
            burn_cost.to_u64() * 2 + 5_000,
        );

        // Should be possible to burned register before enable is activated
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid2,
            hotkey_account_2_id
        ));
    });
}

// #[test]
// fn test_user_liquidity_access_control() {
//     new_test_ext(1).execute_with(|| {
//         let owner_hotkey = U256::from(1);
//         let owner_coldkey = U256::from(2);
//         let not_owner = U256::from(999); // arbitrary non-owner

//         // add network
//         let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

//         // Not owner, not root: should fail
//         assert_noop!(
//             Swap::toggle_user_liquidity(RuntimeOrigin::signed(not_owner), netuid, true),
//             DispatchError::BadOrigin
//         );

//         // Subnet owner can enable
//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::signed(owner_coldkey),
//             netuid,
//             true
//         ));
//         assert!(pallet_subtensor_swap::EnabledUserLiquidity::<Test>::get(
//             NetUid::from(netuid)
//         ));

//         // Root can disable
//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid,
//             false
//         ));
//         assert!(!pallet_subtensor_swap::EnabledUserLiquidity::<Test>::get(
//             NetUid::from(netuid)
//         ));

//         // Root can enable again
//         assert_ok!(Swap::toggle_user_liquidity(
//             RuntimeOrigin::root(),
//             netuid,
//             true
//         ));
//         assert!(pallet_subtensor_swap::EnabledUserLiquidity::<Test>::get(
//             NetUid::from(netuid)
//         ));

//         // Subnet owner cannot disable (only root can disable)
//         assert_noop!(
//             Swap::toggle_user_liquidity(RuntimeOrigin::signed(owner_coldkey), netuid, false),
//             DispatchError::BadOrigin
//         );
//         assert!(pallet_subtensor_swap::EnabledUserLiquidity::<Test>::get(
//             NetUid::from(netuid)
//         ));
//     });
// }

// cargo test --package pallet-subtensor --lib -- tests::subnet::test_no_duplicates_in_symbol_static --exact --show-output
#[test]
fn test_no_duplicates_in_symbol_static() {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for netuid in 0..(SYMBOLS.len() as u16) {
        let symbol = SYMBOLS.get(usize::from(netuid)).unwrap();
        assert!(
            seen.insert(symbol.to_vec()),
            "Duplicate symbol found for netuid {netuid}: {symbol:?}"
        );
    }
}

#[test]
fn test_get_symbol_for_subnet_returns_default_symbol_if_netuid_is_out_of_bounds() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(SYMBOLS.len() as u16 + 1);
        let symbol = Pallet::<Test>::get_symbol_for_subnet(netuid);
        assert_eq!(symbol, DEFAULT_SYMBOL);
    });
}

#[test]
fn test_update_symbol_works_as_root_if_symbol_exists_and_available() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 10, 0);

        // only one network so we can set any symbol, except the root symbol
        for i in 1..SYMBOLS.len() {
            let symbol = SYMBOLS.get(i).unwrap().to_vec();
            assert_ok!(SubtensorModule::update_symbol(
                <Test as Config>::RuntimeOrigin::root(),
                netuid,
                symbol.clone()
            ));

            assert_eq!(TokenSymbol::<Test>::get(netuid), symbol);
            assert_last_event::<Test>(Event::SymbolUpdated { netuid, symbol }.into());
        }
    });
}

#[test]
fn test_update_symbol_works_as_subnet_owner_if_symbol_exists_and_available() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let netuid = NetUid::from(1);
        add_network(netuid, 10, 0);
        SubnetOwner::<Test>::insert(netuid, coldkey);

        // only one network so we can set any symbol, except the root symbol
        for i in 1..SYMBOLS.len() {
            let symbol = SYMBOLS.get(i).unwrap().to_vec();

            assert_ok!(SubtensorModule::update_symbol(
                <Test as Config>::RuntimeOrigin::signed(coldkey),
                netuid,
                symbol.clone()
            ));

            assert_eq!(TokenSymbol::<Test>::get(netuid), symbol);
            assert_last_event::<Test>(Event::SymbolUpdated { netuid, symbol }.into());
        }
    });
}

#[test]
fn test_update_symbol_fails_if_symbol_doesnt_exist() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let netuid = NetUid::from(1);
        add_network(netuid, 10, 0);
        SubnetOwner::<Test>::insert(netuid, coldkey);

        assert_err!(
            SubtensorModule::update_symbol(
                <Test as Config>::RuntimeOrigin::signed(coldkey),
                netuid,
                b"TEST".to_vec()
            ),
            Error::<Test>::SymbolDoesNotExist
        );
    });
}

#[test]
fn test_update_symbol_fails_if_symbol_already_in_use() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let netuid = NetUid::from(1);
        add_network(netuid, 10, 0);
        SubnetOwner::<Test>::insert(netuid, coldkey);

        let coldkey2 = U256::from(2);
        let netuid2 = NetUid::from(2);
        add_network(netuid2, 10, 0);
        SubnetOwner::<Test>::insert(netuid2, coldkey2);

        assert_ok!(SubtensorModule::update_symbol(
            <Test as Config>::RuntimeOrigin::signed(coldkey),
            netuid,
            SYMBOLS.get(42).unwrap().to_vec()
        ));

        assert_err!(
            SubtensorModule::update_symbol(
                <Test as Config>::RuntimeOrigin::signed(coldkey2),
                netuid2,
                SYMBOLS.get(42).unwrap().to_vec()
            ),
            Error::<Test>::SymbolAlreadyInUse
        );
    });
}
