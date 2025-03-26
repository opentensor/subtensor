use super::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::Config;
use sp_core::U256;

/***************************
  pub fn do_start_call() tests
*****************************/

#[test]
fn test_do_start_call_ok() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let hotkey_account_id = U256::from(1);
        let burn_cost = 1000;
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);

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

        assert_eq!(
            FirstEmissionBlockNumber::<Test>::get(netuid),
            Some(block_number + 1)
        );
    });
}

#[test]
fn test_do_start_call_fail_with_not_existed_subnet() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;
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
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let hotkey_account_id = U256::from(1);
        let wrong_owner_account_id = U256::from(2);
        let burn_cost = 1000;
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));

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
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let hotkey_account_id = U256::from(1);
        let burn_cost = 1000;
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 10000);

        // Subscribe and check extrinsic output
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id
        ));

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
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let hotkey_account_id = U256::from(1);
        let burn_cost = 1000;
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);

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
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let coldkey_account_id = U256::from(0);
        let hotkey_account_id = U256::from(1);
        let burn_cost = 1000;
        //add network
        SubtensorModule::set_burn(netuid, burn_cost);
        add_network_without_emission_block(netuid, tempo, 0);
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), None);

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
    new_test_ext(0).execute_with(|| {
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
                    RuntimeEvent::SubtensorModule(SubtensorEvent::NetworkAdded(_, _))
                )
            })
            .last()
            .unwrap();
        let netuid = match min_burn_event.event {
            RuntimeEvent::SubtensorModule(SubtensorEvent::NetworkAdded(netuid, _)) => netuid,
            _ => panic!("Expected NetworkAdded event"),
        };

        // Check min burn is set to default
        assert_eq!(MinBurn::<Test>::get(netuid), InitialMinBurn::get());
    });
}
