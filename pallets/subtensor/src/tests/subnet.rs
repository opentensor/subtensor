use frame_support::{assert_noop, assert_ok};
use frame_system::Config;
use sp_core::U256;

use super::mock;
use super::mock::*;
use crate::*;

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
        mock::setup_reserves(netuid, 1_000_000_000, 1_000_000_000);

        // account 0 is the default owner for any subnet
        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        let block_number = System::block_number() + DurationOfStartCall::get();
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
            Error::<Test>::SubNetworkDoesNotExist
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
        let burn_cost = 1000;
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        mock::setup_reserves(netuid, 1_000_000_000, 1_000_000_000);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        add_network_without_emission_block(netuid, tempo, 0);

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        System::set_block_number(System::block_number() + DurationOfStartCall::get());

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
fn test_do_start_call_fail_with_cannot_start_call_now() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let burn_cost = 1000;
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        mock::setup_reserves(netuid, 1_000_000_000, 1_000_000_000);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        add_network_without_emission_block(netuid, tempo, 0);

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        assert_noop!(
            SubtensorModule::start_call(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                netuid
            ),
            Error::<Test>::NeedWaitingMoreBlocksToStarCall
        );
    });
}

#[test]
fn test_do_start_call_fail_for_set_again() {
    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let hotkey_account_id = U256::from(1);
        let burn_cost = 1000;

        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);

        mock::setup_reserves(netuid, 1_000_000_000, 1_000_000_000);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        let block_number = System::block_number() + DurationOfStartCall::get();
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
        mock::setup_reserves(netuid, 1_000_000_000, 1_000_000_000);

        assert_eq!(SubnetOwner::<Test>::get(netuid), coldkey_account_id);

        let block_number = System::block_number() + DurationOfStartCall::get();
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
        SubtensorModule::add_balance_to_coldkey_account(&sn_owner_coldkey, cost + 10_000_000_000);

        // Register network
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(sn_owner_coldkey),
            sn_owner_hotkey
        ));
        // Get last events
        let events = System::events();
        let min_burn_event = events
            .iter()
            .filter(|event| {
                matches!(
                    event.event,
                    RuntimeEvent::SubtensorModule(Event::<Test>::NetworkAdded(..))
                )
            })
            .next_back();

        let netuid = match min_burn_event.map(|event| event.event.clone()) {
            Some(RuntimeEvent::SubtensorModule(Event::<Test>::NetworkAdded(netuid, _))) => netuid,
            _ => panic!("Expected NetworkAdded event"),
        };

        // Check min burn is set to default
        assert_eq!(MinBurn::<Test>::get(netuid), InitialMinBurn::get());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::subnet::test_no_duplicates_in_get_symbol_for_subnet --exact --show-output
#[test]
fn test_no_duplicates_in_get_symbol_for_subnet() {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for netuid in 0u16..=438 {
        let netuid = NetUid::from(netuid);
        let symbol = Pallet::<Test>::get_symbol_for_subnet(netuid);
        assert!(
            seen.insert(symbol.clone()),
            "Duplicate symbol found for netuid {}: {:?}",
            netuid,
            symbol
        );
    }
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

        let block_number = System::block_number() + DurationOfStartCall::get();
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
        let amount = DefaultMinStake::<Test>::get() * 10;

        let stake_bal = 10_000_000_000; // 10 Alpha

        let limit_price = 1_000_000_000; // not important

        add_network_disable_subtoken(netuid, 10, 0);
        add_network_disable_subtoken(netuid2, 10, 0);

        assert!(!SubtokenEnabled::<Test>::get(netuid));
        assert!(!SubtokenEnabled::<Test>::get(netuid2));

        // Set liq high enough to not trigger other errors
        SubnetTAO::<Test>::set(netuid, 20_000_000_000);
        SubnetAlphaIn::<Test>::set(netuid, 20_000_000_000);

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
                amount
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::add_stake_limit(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount,
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
            amount,
            limit_price,
            false,
        )
        .unwrap();

        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                amount
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::recycle_alpha(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                amount,
                netuid
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::burn_alpha(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                amount,
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
                amount,
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
                amount,
            ),
            Error::<Test>::SubtokenDisabled
        );

        assert_noop!(
            SubtensorModule::swap_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                netuid,
                netuid2,
                amount,
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
        let stake_amount = DefaultMinStake::<Test>::get() * 10000;
        // unstake, transfer, swap just very little
        let unstake_amount = DefaultMinStake::<Test>::get() * 10;

        add_network(netuid, 10, 0);
        add_network(netuid2, 10, 0);

        let reserve = stake_amount * 1000;
        mock::setup_reserves(netuid, reserve, reserve);
        mock::setup_reserves(netuid2, reserve, reserve);
        SubnetAlphaOut::<Test>::insert(netuid, reserve);
        SubnetAlphaOut::<Test>::insert(netuid2, reserve);

        // Register so staking works
        register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 0);
        register_ok_neuron(netuid2, hotkey_account_id, coldkey_account_id, 100);
        register_ok_neuron(netuid, hotkey_account_2_id, coldkey_account_id, 0);
        register_ok_neuron(netuid2, hotkey_account_2_id, coldkey_account_id, 100);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, stake_amount * 10);

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

        let burn_cost = 1000;
        // Set the burn cost
        SubtensorModule::set_burn(netuid, burn_cost);
        // Add the networks with subtoken disabled
        add_network_disable_subtoken(netuid, 10, 0);
        add_network_disable_subtoken(netuid2, 10, 0);
        // Give enough to burned register
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, burn_cost * 2 + 5_000);

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
