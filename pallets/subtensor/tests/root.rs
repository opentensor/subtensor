use crate::mock::*;
use frame_support::assert_ok;
use frame_system::Config;
use frame_system::{EventRecord, Phase};
use pallet_subtensor::Error;
use sp_core::{H256, U256};
mod mock;

#[allow(dead_code)]
fn record(event: RuntimeEvent) -> EventRecord<RuntimeEvent, H256> {
    EventRecord {
        phase: Phase::Initialization,
        event,
        topics: vec![],
    }
}

#[test]
fn test_root_register_network_does_not_exist() {
    new_test_ext().execute_with(|| {
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        assert_eq!(
            SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
            ),
            Err(Error::<Test>::NetworkDoesNotExist.into())
        );
    });
}

#[test]
fn test_root_register_network_exist() {
    new_test_ext().execute_with(|| {
        let root_netuid: u16 = 0;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        add_network(root_netuid, 0, 0);
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
        ));
    });
}

#[test]
fn test_root_register_normal_on_root_fails() {
    new_test_ext().execute_with(|| {
        // Test fails because normal registrations are not allowed
        // on the root network.
        let root_netuid: u16 = 0;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        add_network(root_netuid, 0, 0);

        // Burn registration fails.
        SubtensorModule::set_burn(root_netuid, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1);
        assert_eq!(
            SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                root_netuid,
                hotkey_account_id
            ),
            Err(Error::<Test>::OperationNotPermittedonRootSubnet.into())
        );
        // Pow registration fails.
        let block_number: u64 = SubtensorModule::get_current_block_as_u64();
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            root_netuid,
            block_number,
            0,
            &hotkey_account_id,
        );
        assert_eq!(
            SubtensorModule::register(
                <<Test as frame_system::Config>::RuntimeOrigin>::signed(hotkey_account_id),
                root_netuid,
                block_number,
                nonce,
                work,
                hotkey_account_id,
                coldkey_account_id,
            ),
            Err(Error::<Test>::OperationNotPermittedonRootSubnet.into())
        );
    });
}

#[test]
fn test_root_register_stake_based_pruning_works() {
    new_test_ext().execute_with(|| {
        // Add two networks.
        let root_netuid: u16 = 0;
        let other_netuid: u16 = 1;
        add_network(other_netuid, 0, 0);
        add_network(root_netuid, 0, 0);

        // Set burn cost on other network.
        SubtensorModule::set_burn(other_netuid, 0);

        // Create two accounts with balances.
        let hotkey_account_id_1: U256 = U256::from(1);
        let coldkey_account_id_1: U256 = U256::from(1);
        let hotkey_account_id_2: U256 = U256::from(2);
        let coldkey_account_id_2: U256 = U256::from(2);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 500);

        // Register both accounts on subnet other.
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_1),
            other_netuid,
            hotkey_account_id_1
        ));
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_2),
            other_netuid,
            hotkey_account_id_2
        ));

        // Add stake on both accounts.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_1),
            hotkey_account_id_1,
            1000
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_2),
            hotkey_account_id_2,
            500
        ));

        // Register first account on subnet 1.
        SubtensorModule::set_max_allowed_uids(root_netuid, 1);
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_1),
            hotkey_account_id_1,
        ));
        // Register the second account on subnet 1.
        // This fails because the first account has more stake.
        assert_eq!(
            SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_2),
                hotkey_account_id_2,
            ),
            Err(Error::<Test>::StakeTooLowForRoot.into())
        );

        // Check that the first account is still registered
        let neuron_uid =
            SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hotkey_account_id_1).unwrap();
        assert!(
            SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hotkey_account_id_1).is_ok()
        );

        // Lets add more stake to the second account.
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_2),
            hotkey_account_id_2,
            10000
        ));

        // Now the registration is successful and the account is replaced.
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id_2),
            hotkey_account_id_2,
        ));

        // Check the account association.
        //check if hotkey is added to the Hotkeys
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id_2),
            coldkey_account_id_2
        );

        // Check if the account is registered on the correct network.
        let _ =
            SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hotkey_account_id_2).unwrap();
        assert!(
            SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hotkey_account_id_2).is_ok()
        );

        // Check that the first account is no longer registered
        assert!(
            SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hotkey_account_id_1).is_err()
        );
    });
}

#[test]
fn test_root_set_weights() {
    new_test_ext().execute_with(|| {
        let n: usize = 10;
        let root_netuid: u16 = 0;
        add_network(root_netuid, 0, 0);
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16);
        for i in 0..n {
            let hotkey_account_id: U256 = U256::from(i);
            let coldkey_account_id: U256 = U256::from(i);
            SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1000);
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
            ));
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
                hotkey_account_id,
                1000
            ));
        }

        // Set weights into diagonal matrix.
        for i in 0..n {
            let uids: Vec<u16> = vec![i as u16];
            let values: Vec<u16> = vec![i as u16];
            assert_ok!(SubtensorModule::set_weights(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(i)),
                root_netuid,
                uids,
                values,
                0,
            ));
        }
        // Run the root epoch
        log::debug!("Running Root epoch");
        SubtensorModule::set_tempo(root_netuid, 1);
        // This will fail because their are not enough netuids.
        assert!(SubtensorModule::root_epoch(1_000_000_000).is_err());
        // Lets create n networks
        for netuid in 1..n  {
            log::debug!("Adding network with netuid: {}", netuid);
            add_network(netuid as u16, 13, 0);
        }
        // The root network is on tempo.
        assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
        // Check that the emission values have been set.
        for netuid in 1..n {
            log::debug!("check emission for netuid: {}", netuid);
            assert_eq!( SubtensorModule::get_subnet_emission_value( netuid as u16 ), 111111111 );
        }
        step_block(1);
        // Check that the pending emission values have been set.
        for netuid in 1..n {
            log::debug!("check pending emission for netuid {} has pending {}", netuid, SubtensorModule::get_pending_emission( netuid as u16 ));
            assert_eq!( SubtensorModule::get_pending_emission( netuid as u16 ), 111111111 );
        }
        step_block(1);
        for netuid in 1..n {
            log::debug!("check pending emission for netuid {} has pending {}", netuid, SubtensorModule::get_pending_emission( netuid as u16 ));
            assert_eq!( SubtensorModule::get_pending_emission( netuid as u16 ), 222222222 );
        }
        // Step block clears the emission on subnet 9.
        step_block(1);
        assert_eq!( SubtensorModule::get_pending_emission( 9 ), 0 );
    });
}
