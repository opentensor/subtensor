use crate::mock::*;
use frame_support::assert_ok;
use frame_system::Config;
use frame_system::{EventRecord, Phase};
use pallet_subtensor::migration;
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

// #[test]
// fn test_root_register_network_exist() {
//     new_test_ext().execute_with(|| {
//         migration::migrate_create_root_network::<Test>();
//         let root_netuid: u16 = 0;
//         let hotkey_account_id: U256 = U256::from(1);
//         let coldkey_account_id = U256::from(667);
//         assert_ok!(SubtensorModule::root_register(
//             <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//             hotkey_account_id,
//         ));
//     });
// }

// #[test]
// fn test_root_register_normal_on_root_fails() {
//     new_test_ext().execute_with(|| {
//         migration::migrate_create_root_network::<Test>();
//         // Test fails because normal registrations are not allowed
//         // on the root network.
//         let root_netuid: u16 = 0;
//         let hotkey_account_id: U256 = U256::from(1);
//         let coldkey_account_id = U256::from(667);

//         // Burn registration fails.
//         SubtensorModule::set_burn(root_netuid, 0);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1);
//         assert_eq!(
//             SubtensorModule::burned_register(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 root_netuid,
//                 hotkey_account_id
//             ),
//             Err(Error::<Test>::OperationNotPermittedonRootSubnet.into())
//         );
//         // Pow registration fails.
//         let block_number: u64 = SubtensorModule::get_current_block_as_u64();
//         let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
//             root_netuid,
//             block_number,
//             0,
//             &hotkey_account_id,
//         );
//         assert_eq!(
//             SubtensorModule::register(
//                 <<Test as frame_system::Config>::RuntimeOrigin>::signed(hotkey_account_id),
//                 root_netuid,
//                 block_number,
//                 nonce,
//                 work,
//                 hotkey_account_id,
//                 coldkey_account_id,
//             ),
//             Err(Error::<Test>::OperationNotPermittedonRootSubnet.into())
//         );
//     });
// }

// #[test]
// fn test_root_register_stake_based_pruning_works() {
//     new_test_ext().execute_with(|| {
//         migration::migrate_create_root_network::<Test>();
//         // Add two networks.
//         let root_netuid: u16 = 0;
//         let other_netuid: u16 = 1;
//         add_network(other_netuid, 0, 0);

//         // Set params to allow all registrations to subnet.
//         SubtensorModule::set_burn(other_netuid, 0);
//         SubtensorModule::set_max_registrations_per_block(other_netuid, 256);
//         SubtensorModule::set_target_registrations_per_interval(other_netuid, 256);

//         SubtensorModule::set_max_registrations_per_block(root_netuid, 1000);
//         SubtensorModule::set_target_registrations_per_interval(root_netuid, 1000);

//         // Register 256 accounts with stake to the other network.
//         for i in 0..256 {
//             let hot: U256 = U256::from(i);
//             let cold: U256 = U256::from(i);
//             // Add balance
//             SubtensorModule::add_balance_to_coldkey_account(&cold, 1000 + (i as u64));
//             // Register
//             assert_ok!(SubtensorModule::burned_register(
//                 <<Test as Config>::RuntimeOrigin>::signed(cold),
//                 other_netuid,
//                 hot
//             ));
//             // Add stake on other network
//             assert_ok!(SubtensorModule::add_stake(
//                 <<Test as Config>::RuntimeOrigin>::signed(cold),
//                 hot,
//                 1000 + (i as u64)
//             ));
//             // Check succesfull registration.
//             assert!(SubtensorModule::get_uid_for_net_and_hotkey(other_netuid, &hot).is_ok());
//             // Check that they are NOT all delegates
//             assert!(!SubtensorModule::hotkey_is_delegate(&hot));
//         }

//         // Register the first 128 accounts with stake to the root network.
//         for i in 0..128 {
//             let hot: U256 = U256::from(i);
//             let cold: U256 = U256::from(i);
//             assert_ok!(SubtensorModule::root_register(
//                 <<Test as Config>::RuntimeOrigin>::signed(cold),
//                 hot,
//             ));
//             // Check succesfull registration.
//             assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
//             // Check that they are all senate members
//             assert!(SubtensorModule::is_senate_member(&hot));
//             // Check that they are all delegates
//             assert!(SubtensorModule::hotkey_is_delegate(&hot));
//         }

//         // Register the second 128 accounts with stake to the root network.
//         // Replaces the first 128
//         for i in 128..256 {
//             let hot: U256 = U256::from(i);
//             let cold: U256 = U256::from(i);
//             assert_ok!(SubtensorModule::root_register(
//                 <<Test as Config>::RuntimeOrigin>::signed(cold),
//                 hot,
//             ));
//             // Check succesfull registration.
//             assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
//             // Check that they are all senate members
//             assert!(SubtensorModule::is_senate_member(&hot));
//         }

//         // Register the first 128 accounts again, this time failing because they
//         // dont have enough stake.
//         for i in 0..128 {
//             let hot: U256 = U256::from(i);
//             let cold: U256 = U256::from(i);
//             assert_eq!(
//                 SubtensorModule::root_register(
//                     <<Test as Config>::RuntimeOrigin>::signed(cold),
//                     hot,
//                 ),
//                 Err(Error::<Test>::StakeTooLowForRoot.into())
//             );
//             // Check for unsuccesfull registration.
//             assert!(!SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
//             // Check that they are NOT senate members
//             assert!(!SubtensorModule::is_senate_member(&hot));
//         }
//     });
// }

// #[test]
// fn test_root_set_weights() {
//     new_test_ext().execute_with(|| {
//         migration::migrate_create_root_network::<Test>();

//         let n: usize = 10;
//         let root_netuid: u16 = 0;
//         SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
//         SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
//         SubtensorModule::set_max_allowed_uids(root_netuid, n as u16);
//         for i in 0..n {
//             let hotkey_account_id: U256 = U256::from(i);
//             let coldkey_account_id: U256 = U256::from(i);
//             SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1_000_000_000_000_000);
//             assert_ok!(SubtensorModule::root_register(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//             ));
//             assert_ok!(SubtensorModule::add_stake(
//                 <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
//                 hotkey_account_id,
//                 1000
//             ));
//         }

//         // Lets create n networks
//         for netuid in 1..n {
//             log::debug!("Adding network with netuid: {}", netuid);
//             add_network(netuid as u16, 13, 0);
//             assert_ok!(SubtensorModule::register_network(
//                 <<Test as Config>::RuntimeOrigin>::signed(U256::from(netuid))
//             ));
//         }

//         // Set weights into diagonal matrix.
//         for i in 0..n {
//             let uids: Vec<u16> = vec![i as u16];
//             let values: Vec<u16> = vec![i as u16];
//             assert_ok!(SubtensorModule::set_weights(
//                 <<Test as Config>::RuntimeOrigin>::signed(U256::from(i)),
//                 root_netuid,
//                 uids,
//                 values,
//                 0,
//             ));
//         }
//         // Run the root epoch
//         log::debug!("Running Root epoch");
//         SubtensorModule::set_tempo(root_netuid, 1);
//         assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
//         // Check that the emission values have been set.
//         for netuid in 1..n {
//             log::debug!("check emission for netuid: {}", netuid);
//             assert_eq!(
//                 SubtensorModule::get_subnet_emission_value(netuid as u16),
//                 111111111
//             );
//         }
//         step_block(1);
//         // Check that the pending emission values have been set.
//         for netuid in 1..n {
//             log::debug!(
//                 "check pending emission for netuid {} has pending {}",
//                 netuid,
//                 SubtensorModule::get_pending_emission(netuid as u16)
//             );
//             assert_eq!(
//                 SubtensorModule::get_pending_emission(netuid as u16),
//                 111111111
//             );
//         }
//         step_block(1);
//         for netuid in 1..n {
//             log::debug!(
//                 "check pending emission for netuid {} has pending {}",
//                 netuid,
//                 SubtensorModule::get_pending_emission(netuid as u16)
//             );
//             assert_eq!(
//                 SubtensorModule::get_pending_emission(netuid as u16),
//                 222222222
//             );
//         }
//         // Step block clears the emission on subnet 9.
//         step_block(1);
//         assert_eq!(SubtensorModule::get_pending_emission(9), 0);
//     });
// }

// #[test]
// fn test_root_subnet_creation_deletion() {
//     new_test_ext().execute_with(|| {
//         migration::migrate_create_root_network::<Test>();
//         // Owner of subnets.
//         let owner: U256 = U256::from(0);

//         // Add a subnet.
//         SubtensorModule::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000);
//         step_block(1);
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 1, mult: 1 lock_cost: 100000000000
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 1, mult: 2 lock_cost: 200000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
//         step_block(1);
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 2, mult: 2 lock_cost: 150000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 150_000_000_000); // Reduced by 50%
//         step_block(1);
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 3, mult: 2 lock_cost: 100000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // Reduced another 50%
//         step_block(1);
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 100000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // Reaches min value
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 200000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
//         step_block(1);
//         // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 150000000000
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 300000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 300_000_000_000); // Doubles from previous subnet creation
//         step_block(1);
//         // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 225000000000
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 450000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 450_000_000_000); // Increasing
//         step_block(1);
//         // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 337500000000
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 675_000_000_000); // Increasing.
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 1_350_000_000_000); // Double increasing.
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner)
//         ));
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 2_700_000_000_000); // Double increasing again.

//         // Now drop it like its hot to min again.
//         step_block(1);
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 2_025_000_000_000); // 675_000_000_000 decreasing.
//         step_block(1);
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 1_350_000_000_000); // 675_000_000_000 decreasing.
//         step_block(1);
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 675_000_000_000); // 675_000_000_000 decreasing.
//         step_block(1);
//         assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // 675_000_000_000 decreasing with 100000000000 min
//     });
// }

#[test]
fn test_network_pruning() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        assert_eq!( SubtensorModule::get_total_issuance(), 0 );

        let n: usize = 10;
        let root_netuid: u16 = 0;
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16 + 1);
        SubtensorModule::set_tempo(root_netuid, 1);
        // No validators yet.
        assert_eq!( SubtensorModule::get_subnetwork_n( root_netuid ), 0 );

        for i in 0..n {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            let uids: Vec<u16> = (0..i as u16).collect();
            let values: Vec<u16> = vec![1; i];
            SubtensorModule::add_balance_to_coldkey_account(&cold, 1_000_000_000_000_000 );
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot
            ));
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
                1_000
            ));
            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(cold)
            ));
            log::debug!("Adding network with netuid: {}", (i as u16) + 1 );
            assert!( SubtensorModule::if_subnet_exist( (i as u16) + 1 ) );
            assert!(SubtensorModule::is_hotkey_registered_on_network(
                root_netuid,
                &hot
            ));
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
            assert_ok!(SubtensorModule::set_weights(
                <<Test as Config>::RuntimeOrigin>::signed(hot),
                root_netuid,
                uids,
                values,
                0
            ));
            SubtensorModule::set_tempo((i as u16) + 1, 1);
            SubtensorModule::set_burn((i as u16) + 1, 0);
            assert_ok!(SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                (i as u16) + 1,
                hot
            ));
            assert_eq!( SubtensorModule::get_total_issuance(), 1_000 * ((i as u64) + 1));
            assert_eq!( SubtensorModule::get_subnetwork_n( root_netuid ), (i as u16) + 1 );
        }

        // All stake values.
        assert_eq!( SubtensorModule::get_total_issuance(), 10000 );

        step_block(1);
        assert_ok!( SubtensorModule::root_epoch(1_000_000_000));
        assert_eq!( SubtensorModule::get_subnet_emission_value(0), 199999999);
        assert_eq!( SubtensorModule::get_subnet_emission_value(1), 177777777);
        assert_eq!( SubtensorModule::get_subnet_emission_value(2), 155555555);
        assert_eq!( SubtensorModule::get_subnet_emission_value(3), 133333333);
        assert_eq!( SubtensorModule::get_subnet_emission_value(4), 111111111);
        assert_eq!( SubtensorModule::get_subnet_emission_value(5), 88888888);
        assert_eq!( SubtensorModule::get_total_issuance(), 10000 );
        step_block(1);
        assert_eq!(SubtensorModule::get_pending_emission(0), 0); // root network gets no pending emission.
        assert_eq!(SubtensorModule::get_pending_emission(1), 177777777);
        assert_eq!(SubtensorModule::get_pending_emission(2), 0); // This has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(3), 133333333);
        assert_eq!(SubtensorModule::get_pending_emission(4), 0); // This network has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(5), 88888888);
        step_block(1);
        assert_eq!( SubtensorModule::get_total_issuance(), 711121108 );
    });
}
