use crate::mock::*;
use frame_support::assert_ok;
use frame_system::Config;
use frame_system::{EventRecord, Phase};
use log::info;
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

#[test]
fn test_root_register_network_exist() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        let root_netuid: u16 = 0;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
        ));
    });
}

#[test]
fn test_root_register_normal_on_root_fails() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        // Test fails because normal registrations are not allowed
        // on the root network.
        let root_netuid: u16 = 0;
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);

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
        migration::migrate_create_root_network::<Test>();
        // Add two networks.
        let root_netuid: u16 = 0;
        let other_netuid: u16 = 1;
        add_network(other_netuid, 0, 0);

        // Set params to allow all registrations to subnet.
        SubtensorModule::set_burn(other_netuid, 0);
        SubtensorModule::set_max_registrations_per_block(other_netuid, 256);
        SubtensorModule::set_target_registrations_per_interval(other_netuid, 256);

        SubtensorModule::set_max_registrations_per_block(root_netuid, 1000);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, 1000);

        // Register 128 accounts with stake to the other network.
        for i in 0..128 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            // Add balance
            SubtensorModule::add_balance_to_coldkey_account(&cold, 1000 + (i as u64));
            // Register
            assert_ok!(SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                other_netuid,
                hot
            ));
            // Add stake on other network
            assert_ok!(SubtensorModule::add_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
                1000 + (i as u64)
            ));
            // Check succesfull registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(other_netuid, &hot).is_ok());
            // Check that they are NOT all delegates
            assert!(!SubtensorModule::hotkey_is_delegate(&hot));
        }

        // Register the first 64 accounts with stake to the root network.
        for i in 0..64 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
            ));
            // Check succesfull registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
            // Check that they are all delegates
            assert!(SubtensorModule::hotkey_is_delegate(&hot));
        }

        // Register the second 64 accounts with stake to the root network.
        // Replaces the first 64
        for i in 64..128 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            assert_ok!(SubtensorModule::root_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
            ));
            // Check succesfull registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
        }

        // Register the first 64 accounts again, this time failing because they
        // dont have enough stake.
        for i in 0..64 {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            assert_eq!(
                SubtensorModule::root_register(
                    <<Test as Config>::RuntimeOrigin>::signed(cold),
                    hot,
                ),
                Err(Error::<Test>::StakeTooLowForRoot.into())
            );
            // Check for unsuccesfull registration.
            assert!(!SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
            // Check that they are NOT senate members
            assert!(!SubtensorModule::is_senate_member(&hot));
        }
    });
}

#[test]
fn test_root_set_weights() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        let n: usize = 10;
        let root_netuid: u16 = 0;
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16);
        for i in 0..n {
            let hotkey_account_id: U256 = U256::from(i);
            let coldkey_account_id: U256 = U256::from(i);
            SubtensorModule::add_balance_to_coldkey_account(
                &coldkey_account_id,
                1_000_000_000_000_000,
            );
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

        log::info!("subnet limit: {:?}", SubtensorModule::get_max_subnets());
        log::info!(
            "current subnet count: {:?}",
            SubtensorModule::get_num_subnets()
        );

        // Lets create n networks
        for netuid in 1..n {
            log::debug!("Adding network with netuid: {}", netuid);
            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(U256::from(netuid))
            ));
        }

        // Set weights into diagonal matrix.
        for i in 0..n {
            let uids: Vec<u16> = vec![i as u16];
            let values: Vec<u16> = vec![1];
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
        assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
        // Check that the emission values have been set.
        for netuid in 1..n {
            log::debug!("check emission for netuid: {}", netuid);
            assert_eq!(
                SubtensorModule::get_subnet_emission_value(netuid as u16),
                99_999_999
            );
        }
        step_block(2);
        // Check that the pending emission values have been set.
        for netuid in 1..n {
            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(netuid as u16)
            );
            assert_eq!(
                SubtensorModule::get_pending_emission(netuid as u16),
                199_999_998
            );
        }
        step_block(1);
        for netuid in 1..n {
            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(netuid as u16)
            );
            assert_eq!(
                SubtensorModule::get_pending_emission(netuid as u16),
                299_999_997
            );
        }
        let step = SubtensorModule::blocks_until_next_epoch(
            10,
            1000,
            SubtensorModule::get_current_block_as_u64(),
        );
        step_block(step as u16);
        assert_eq!(SubtensorModule::get_pending_emission(10), 0);
    });
}

#[test]
fn test_root_set_weights_out_of_order_netuids() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        let n: usize = 10;
        let root_netuid: u16 = 0;
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16);
        for i in 0..n {
            let hotkey_account_id: U256 = U256::from(i);
            let coldkey_account_id: U256 = U256::from(i);
            SubtensorModule::add_balance_to_coldkey_account(
                &coldkey_account_id,
                1_000_000_000_000_000,
            );
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

        log::info!("subnet limit: {:?}", SubtensorModule::get_max_subnets());
        log::info!(
            "current subnet count: {:?}",
            SubtensorModule::get_num_subnets()
        );

        // Lets create n networks
        for netuid in 1..n {
            log::debug!("Adding network with netuid: {}", netuid);

            if netuid % 2 == 0 {
                assert_ok!(SubtensorModule::register_network(
                    <<Test as Config>::RuntimeOrigin>::signed(U256::from(netuid))
                ));
            } else {
                add_network(netuid as u16 * 10, 1000, 0)
            }
        }

        log::info!("netuids: {:?}", SubtensorModule::get_all_subnet_netuids());
        log::info!(
            "root network count: {:?}",
            SubtensorModule::get_subnetwork_n(0)
        );

        let subnets = SubtensorModule::get_all_subnet_netuids();
        // Set weights into diagonal matrix.
        for (i, netuid) in subnets.iter().enumerate() {
            let uids: Vec<u16> = vec![*netuid];

            let values: Vec<u16> = vec![1];
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
        assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
        // Check that the emission values have been set.
        for netuid in subnets.iter() {
            log::debug!("check emission for netuid: {}", netuid);
            assert_eq!(
                SubtensorModule::get_subnet_emission_value(*netuid),
                99_999_999
            );
        }
        step_block(2);
        // Check that the pending emission values have been set.
        for netuid in subnets.iter() {
            if *netuid == 0 {
                continue;
            }

            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(*netuid)
            );
            assert_eq!(SubtensorModule::get_pending_emission(*netuid), 199_999_998);
        }
        step_block(1);
        for netuid in subnets.iter() {
            if *netuid == 0 {
                continue;
            }

            log::debug!(
                "check pending emission for netuid {} has pending {}",
                netuid,
                SubtensorModule::get_pending_emission(*netuid)
            );
            assert_eq!(SubtensorModule::get_pending_emission(*netuid), 299_999_997);
        }
        let step = SubtensorModule::blocks_until_next_epoch(
            9,
            1000,
            SubtensorModule::get_current_block_as_u64(),
        );
        step_block(step as u16);
        assert_eq!(SubtensorModule::get_pending_emission(9), 0);
    });
}

#[test]
fn test_root_subnet_creation_deletion() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        // Owner of subnets.
        let owner: U256 = U256::from(0);

        // Add a subnet.
        SubtensorModule::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000);
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 1, mult: 1 lock_cost: 100000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 1, mult: 2 lock_cost: 200000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 2, mult: 2 lock_cost: 150000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 150_000_000_000); // Reduced by 50%
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 3, mult: 2 lock_cost: 100000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // Reduced another 50%
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 100000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // Reaches min value
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 200000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 150000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 300000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 300_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 225000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 450000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 450_000_000_000); // Increasing
        step_block(1);
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 337500000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 675_000_000_000); // Increasing.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 1_350_000_000_000); // Double increasing.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        assert_eq!(SubtensorModule::get_network_lock_cost(), 2_700_000_000_000); // Double increasing again.

        // Now drop it like its hot to min again.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 2_025_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 1_350_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 675_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000); // 675_000_000_000 decreasing with 100000000000 min
    });
}

#[test]
fn test_network_pruning() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        assert_eq!(SubtensorModule::get_total_issuance(), 0);

        let n: usize = 10;
        let root_netuid: u16 = 0;
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_target_registrations_per_interval(root_netuid, n as u16);
        SubtensorModule::set_max_allowed_uids(root_netuid, n as u16 + 1);
        SubtensorModule::set_tempo(root_netuid, 1);
        // No validators yet.
        assert_eq!(SubtensorModule::get_subnetwork_n(root_netuid), 0);

        for i in 0..n {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            let uids: Vec<u16> = (0..i as u16).collect();
            let values: Vec<u16> = vec![1; i];
            SubtensorModule::add_balance_to_coldkey_account(&cold, 1_000_000_000_000_000);
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
            log::debug!("Adding network with netuid: {}", (i as u16) + 1);
            assert!(SubtensorModule::if_subnet_exist((i as u16) + 1));
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
            assert_eq!(
                SubtensorModule::get_total_issuance(),
                1_000 * ((i as u64) + 1)
            );
            assert_eq!(
                SubtensorModule::get_subnetwork_n(root_netuid),
                (i as u16) + 1
            );
        }

        // All stake values.
        assert_eq!(SubtensorModule::get_total_issuance(), 10000);

        step_block(1);
        assert_ok!(SubtensorModule::root_epoch(1_000_000_000));
        assert_eq!(SubtensorModule::get_subnet_emission_value(0), 277_820_113);
        assert_eq!(SubtensorModule::get_subnet_emission_value(1), 246_922_263);
        assert_eq!(SubtensorModule::get_subnet_emission_value(2), 215_549_466);
        assert_eq!(SubtensorModule::get_subnet_emission_value(3), 176_432_500);
        assert_eq!(SubtensorModule::get_subnet_emission_value(4), 77_181_559);
        assert_eq!(SubtensorModule::get_subnet_emission_value(5), 5_857_251);
        assert_eq!(SubtensorModule::get_total_issuance(), 10000);
        step_block(1);
        assert_eq!(SubtensorModule::get_pending_emission(0), 0); // root network gets no pending emission.
        assert_eq!(SubtensorModule::get_pending_emission(1), 246_922_263);
        assert_eq!(SubtensorModule::get_pending_emission(2), 0); // This has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(3), 176_432_500);
        assert_eq!(SubtensorModule::get_pending_emission(4), 0); // This network has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(5), 5_857_251);
        step_block(1);
        assert_eq!(SubtensorModule::get_total_issuance(), 585_930_498);
    });
}

#[test]
fn test_network_prune_results() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        SubtensorModule::set_network_immunity_period(3);
        SubtensorModule::set_network_min_lock(0);
        SubtensorModule::set_network_rate_limit(0);

        let owner: U256 = U256::from(0);
        SubtensorModule::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        // lowest emission
        SubtensorModule::set_emission_values(&vec![1u16, 2u16, 3u16], vec![5u64, 4u64, 4u64]);
        assert_eq!(SubtensorModule::get_subnet_to_prune(), 2u16);

        // equal emission, creation date
        SubtensorModule::set_emission_values(&vec![1u16, 2u16, 3u16], vec![5u64, 5u64, 4u64]);
        assert_eq!(SubtensorModule::get_subnet_to_prune(), 3u16);

        // equal emission, creation date
        SubtensorModule::set_emission_values(&vec![1u16, 2u16, 3u16], vec![4u64, 5u64, 5u64]);
        assert_eq!(SubtensorModule::get_subnet_to_prune(), 1u16);
    });
}

#[test]
fn test_weights_after_network_pruning() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        assert_eq!(SubtensorModule::get_total_issuance(), 0);

        // Set up N subnets, with max N + 1 allowed UIDs
        let n: usize = 2;
        let root_netuid: u16 = 0;
        SubtensorModule::set_network_immunity_period(3);
        SubtensorModule::set_max_registrations_per_block(root_netuid, n as u16);
        SubtensorModule::set_max_subnets(n as u16);
        SubtensorModule::set_weights_set_rate_limit(root_netuid, 0 as u64);

        // No validators yet.
        assert_eq!(SubtensorModule::get_subnetwork_n(root_netuid), 0);

        for i in 0..n {
            // Register a validator
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);

            SubtensorModule::add_balance_to_coldkey_account(&cold, 1_000_000_000_000);

            // Register a network
            assert_ok!(SubtensorModule::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(cold)
            ));

            log::debug!("Adding network with netuid: {}", (i as u16) + 1);
            assert!(SubtensorModule::if_subnet_exist((i as u16) + 1));
            step_block(3);
        }

        // Register a validator in subnet 0
        let hot: U256 = U256::from((n as u64) - 1);
        let cold: U256 = U256::from((n as u64) - 1);

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            hot
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            hot,
            1_000
        ));

        // Let's give these subnets some weights
        let uids: Vec<u16> = (0..(n as u16) + 1).collect();
        let values: Vec<u16> = vec![4u16, 2u16, 6u16];
        log::info!("uids set: {:?}", uids);
        log::info!("values set: {:?}", values);
        log::info!("In netuid: {:?}", root_netuid);
        assert_ok!(SubtensorModule::set_weights(
            <<Test as Config>::RuntimeOrigin>::signed(hot),
            root_netuid,
            uids,
            values,
            0
        ));

        log::info!(
            "Root network weights before extra network registration: {:?}",
            SubtensorModule::get_root_weights()
        );
        log::info!("Max subnets: {:?}", SubtensorModule::get_max_subnets());
        let i = (n as u16) + 1;
        let hot: U256 = U256::from(i);
        let cold: U256 = U256::from(i);

        SubtensorModule::add_balance_to_coldkey_account(&cold, 1_000_000_000_000_000_000);
        let subnet_to_prune = SubtensorModule::get_subnet_to_prune();

        // Subnet 1 should be pruned here.
        assert_eq!(subnet_to_prune, 1);
        log::info!("Removing subnet: {:?}", subnet_to_prune);

        // Check that the weights have been set appropriately.
        let latest_weights = SubtensorModule::get_root_weights();
        log::info!("Weights before register network: {:?}", latest_weights);
        // We expect subnet 1 to be deregistered as it is oldest and has lowest emissions
        assert_eq!(latest_weights[0][1], 21845);

        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(cold)
        ));

        // Subnet should not exist, as it would replace a previous subnet.
        assert!(!SubtensorModule::if_subnet_exist((i as u16) + 1));

        log::info!(
            "Root network weights: {:?}",
            SubtensorModule::get_root_weights()
        );

        let latest_weights = SubtensorModule::get_root_weights();
        log::info!(
            "Weights after register network: {:?}",
            SubtensorModule::get_root_weights()
        );

        // Subnet 0 should be kicked, and thus its weight should be 0
        assert_eq!(latest_weights[0][1], 0);
    });
}
