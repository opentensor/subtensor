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
fn test_root_subnet_creation_deletion() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        // Owner of subnets.
        let owner: U256 = U256::from(0);

        // Add a subnet.
        Subtensor::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 100_000_000_000);
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 1, mult: 1 lock_cost: 100000000000
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 1, mult: 2 lock_cost: 200000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 2, mult: 2 lock_cost: 150000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 150_000_000_000); // Reduced by 50%
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 3, mult: 2 lock_cost: 100000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 100_000_000_000); // Reduced another 50%
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 1, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 100000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 100_000_000_000); // Reaches min value
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 200000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 150000000000
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 300000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 300_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 225000000000
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 450000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 450_000_000_000); // Increasing
        step_block(1);
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 337500000000
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 675_000_000_000); // Increasing.
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(Subtensor::get_network_lock_cost(), 1_350_000_000_000); // Double increasing.
        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        assert_eq!(Subtensor::get_network_lock_cost(), 2_700_000_000_000); // Double increasing again.

        // Now drop it like its hot to min again.
        step_block(1);
        assert_eq!(Subtensor::get_network_lock_cost(), 2_025_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(Subtensor::get_network_lock_cost(), 1_350_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(Subtensor::get_network_lock_cost(), 675_000_000_000); // 675_000_000_000 decreasing.
        step_block(1);
        assert_eq!(Subtensor::get_network_lock_cost(), 100_000_000_000); // 675_000_000_000 decreasing with 100000000000 min
    });
}

#[test]
fn test_network_pruning() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        assert_eq!(Subtensor::get_total_issuance(), 0);

        let n: usize = 10;
        let root_netuid: u16 = 0;
        Subtensor::set_max_registrations_per_block(root_netuid, n as u16);
        Subtensor::set_target_registrations_per_interval(root_netuid, n as u16);
        Subtensor::set_max_allowed_uids(root_netuid, n as u16 + 1);
        Subtensor::set_tempo(root_netuid, 1);
        // No validators yet.
        assert_eq!(Subtensor::get_subnetwork_n(root_netuid), 0);

        for i in 0..n {
            let hot: U256 = U256::from(i);
            let cold: U256 = U256::from(i);
            let uids: Vec<u16> = (0..i as u16).collect();
            let values: Vec<u16> = vec![1; i];
            Subtensor::add_balance_to_coldkey_account(&cold, 1_000_000_000_000_000);
            assert_ok!(Subtensor::register_network(
                <<Test as Config>::RuntimeOrigin>::signed(cold)
            ));
            assert_ok!(Subtensor::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                (i as u16) + 1,
                hot
            ));
            log::debug!("Adding network with netuid: {}", (i as u16) + 1);
            assert!(Subtensor::if_subnet_exist((i as u16) + 1));

            assert_ok!(Subtensor::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
                (i as u16) + 1,
                1_000 * ((i as u64) + 1)
            ));

            assert!(Subtensor::get_uid_for_net_and_hotkey((i as u16) + 1, &hot).is_ok());

            Subtensor::set_tempo((i as u16) + 1, 1);
            Subtensor::set_burn((i as u16) + 1, 0);
            assert_eq!(
                Subtensor::get_subnetwork_n((i as u16) + 1),
                1
            );
        }

        assert_eq!(
            Subtensor::get_combined_subnet_stake(),
            55_000
        );

        step_block(1);
        //assert_ok!(Subtensor::root_epoch(1_000_000_000));
        //assert_eq!(Subtensor::get_subnet_emission_value(0), 277_820_113);
        assert_eq!(Subtensor::get_subnet_emission_value(1), 18_181_818);
        assert_eq!(Subtensor::get_subnet_emission_value(2), 36_363_636);
        assert_eq!(Subtensor::get_subnet_emission_value(3), 54_545_454);
        assert_eq!(Subtensor::get_subnet_emission_value(4), 72_727_272);
        assert_eq!(Subtensor::get_subnet_emission_value(5), 90_909_090);

        step_block(1);
        //assert_eq!(Subtensor::get_pending_emission(0), 0); // root network gets no pending emission.
        assert_eq!(Subtensor::get_pending_emission(1), 18_181_818);
        assert_eq!(Subtensor::get_pending_emission(2), 0); // This has been drained.
        assert_eq!(Subtensor::get_pending_emission(3), 54_545_454);
        assert_eq!(Subtensor::get_pending_emission(4), 0); // This network has been drained.
        assert_eq!(Subtensor::get_pending_emission(5), 90_909_090);
        step_block(1);
        assert_eq!(Subtensor::get_combined_subnet_stake(), 1_545_509_536);
    });
}

#[test]
fn test_network_prune_results() {
    new_test_ext().execute_with(|| {
        migration::migrate_create_root_network::<Test>();

        Subtensor::set_network_immunity_period(3);
        Subtensor::set_network_min_lock(0);
        Subtensor::set_network_rate_limit(0);

        let owner: U256 = U256::from(0);
        Subtensor::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);

        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        assert_ok!(Subtensor::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner)
        ));
        step_block(3);

        // lowest emission
        Subtensor::set_emission_values(&vec![1u16, 2u16, 3u16], vec![5u64, 4u64, 4u64]);
        assert_eq!(Subtensor::get_subnet_to_prune(), 2u16);

        // equal emission, creation date
        Subtensor::set_emission_values(&vec![1u16, 2u16, 3u16], vec![5u64, 5u64, 4u64]);
        assert_eq!(Subtensor::get_subnet_to_prune(), 3u16);

        // equal emission, creation date
        Subtensor::set_emission_values(&vec![1u16, 2u16, 3u16], vec![4u64, 5u64, 5u64]);
        assert_eq!(Subtensor::get_subnet_to_prune(), 1u16);
    });
}