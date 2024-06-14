use crate::mock::*;
use frame_support::{assert_err, assert_ok};
use frame_system::Config;
use frame_system::{EventRecord, Phase};
use pallet_subtensor::{migration, Error, PendingEmission, SubnetInTransition, TotalSubnetTAO};
use sp_core::{Get, H256, U256};

mod mock;

// To run just the tests in this file, use the following command:
// cargo test -p pallet-subtensor --test ro

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
    new_test_ext(1).execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        let hotkey_account_id: U256 = U256::from(1);
        let coldkey_account_id = U256::from(667);
        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
        ));
    });
}

#[test]
fn test_set_weights_not_root_error() {
    new_test_ext(0).execute_with(|| {
        let netuid: u16 = 1;

        let dests = vec![0];
        let weights = vec![1];
        let version_key: u64 = 0;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 2143124);

        assert_err!(
            SubtensorModule::set_root_weights(
                RuntimeOrigin::signed(coldkey),
                netuid,
                hotkey,
                dests.clone(),
                weights.clone(),
                version_key,
            ),
            Error::<Test>::NotRootSubnet
        );
    });
}

#[test]
fn test_root_register_normal_on_root_fails() {
    new_test_ext(1).execute_with(|| {
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
            Err(Error::<Test>::RegistrationNotPermittedOnRootSubnet.into())
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
            Err(Error::<Test>::RegistrationNotPermittedOnRootSubnet.into())
        );
    });
}

#[test]
fn test_root_register_stake_based_pruning_works() {
    new_test_ext(1).execute_with(|| {
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
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                hot,
                other_netuid,
                1000 + (i as u64)
            ));
            // Check successful registration.
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
            // Check successful registration.
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
            // Check successful registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_ok());
        }

        // Register the first 64 accounts again, this time failing because they
        // don't have enough stake.
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
            // Check for unsuccessful registration.
            assert!(SubtensorModule::get_uid_for_net_and_hotkey(root_netuid, &hot).is_err());
            // Check that they are NOT senate members
            assert!(!SubtensorModule::is_senate_member(&hot));
        }
    });
}

#[test]
fn test_root_subnet_creation_deletion() {
    new_test_ext(1).execute_with(|| {
        System::set_block_number(0);
        migration::migrate_create_root_network::<Test>();
        // Owner of subnets.
        let owner: U256 = U256::from(0);
        let hotkey: U256 = U256::from(1);

        // Add a subnet.
        SubtensorModule::add_balance_to_coldkey_account(&owner, 1_000_000_000_000_000);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 0, mult: 1 lock_cost: 100000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 100_000_000_000);
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 0, lock_reduction_interval: 2, current_block: 1, mult: 1 lock_cost: 100000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
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
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
        ));
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 4, mult: 2 lock_cost: 200000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 200_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 100000000000, min_lock: 100000000000, last_lock_block: 4, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 150000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
        ));
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 5, mult: 2 lock_cost: 300000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 300_000_000_000); // Doubles from previous subnet creation
        step_block(1);
        // last_lock: 150000000000, min_lock: 100000000000, last_lock_block: 5, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 225000000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
        ));
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 6, mult: 2 lock_cost: 450000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 450_000_000_000); // Increasing
        step_block(1);
        // last_lock: 225000000000, min_lock: 100000000000, last_lock_block: 6, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 337500000000
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 675_000_000_000); // Increasing.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
        ));
        // last_lock: 337500000000, min_lock: 100000000000, last_lock_block: 7, lock_reduction_interval: 2, current_block: 7, mult: 2 lock_cost: 675000000000
        assert_eq!(SubtensorModule::get_network_lock_cost(), 1_350_000_000_000); // Double increasing.
        assert_ok!(SubtensorModule::register_network(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            hotkey
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
fn test_subnet_staking_cleared_and_refunded_on_network_removal() {
    new_test_ext(1).execute_with(|| {
        migration::migrate_create_root_network::<Test>();
        let netuid: u16 = 1;
        let hotkey_account_id = U256::from(1);
        let coldkey_account_id = U256::from(667);
        let initial_balance = 100_000_000;
        let burn_amount: u64 = 10;
        let stake_amount = 1_000;

        add_network(netuid, 0, 0);

        // Add initial balance to the coldkey account
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, initial_balance);
        log::info!(
            "Initial balance added to coldkey account: {}",
            initial_balance
        );

        // Set up the network with a specific burn cost (if applicable)
        SubtensorModule::set_burn(netuid, burn_amount);
        log::info!("Burn set to {}", burn_amount);

        // Register the hotkey with the network and stake
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            netuid,
            hotkey_account_id,
        ));
        log::info!("Hotkey registered");

        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            stake_amount,
        ));
        log::info!("Stake added");
        log::info!(
            "Balance after adding stake: {}",
            SubtensorModule::get_coldkey_balance(&coldkey_account_id)
        );
        // Verify the stake has been added
        let stake_before_removal =
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey_account_id, netuid);
        log::info!("Stake before removal: {}", stake_before_removal);
        assert_eq!(stake_before_removal, stake_amount);

        // TODO: Do we have the network removal removed on purpose?
        // Remove the network, triggering stake removal and refund
        // SubtensorModule::remove_network(netuid);
        // log::info!("Network removed");

        // // Verify the stake has been cleared
        // let stake_after_removal =
        //     SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey_account_id, netuid);
        // log::info!("Stake after removal: {}", stake_after_removal);
        // assert_eq!(stake_after_removal, 0);

        // // Verify the balance has been refunded to the coldkey account
        // let balance_after_refund = SubtensorModule::get_coldkey_balance(&coldkey_account_id);
        // log::info!("Balance after refund: {}", balance_after_refund);
        // assert_eq!(balance_after_refund, initial_balance - burn_amount);
    });
}

/// This test checks the halving mechanism of the emission schedule.
/// Run this test using the following command:
/// `cargo test --package pallet-subtensor --test root test_issance_bounds`
#[test]
fn test_issuance_bounds() {
    new_test_ext(1).execute_with(|| {
        // Simulate 100 halvings convergence to 21M. Note that the total issuance never reaches 21M because of rounding errors.
        // We converge to 20_999_999_989_500_000 (< 1 TAO away).
        let n_halvings: usize = 100;
        let total_issuance = (0..n_halvings).fold(0, |total, _| {
            let block_emission_10_500_000x: u64 =
                SubtensorModule::get_block_emission_for_issuance(total).unwrap() * 10_500_000;
            total + block_emission_10_500_000x
        });
        assert_eq!(total_issuance, 20_999_999_989_500_000);
    })
}

/// This test checks the halving mechanism of the emission schedule.
/// Run this test using the following command:
/// `cargo test --package pallet-subtensor --test root test_halving`
#[test]
fn test_halving() {
    new_test_ext(1).execute_with(|| {
        let expected_emissions: [(u64, u64); 43] = [
            (0, 1_000_000_000), // Testing at zero issuance.
            (1_776_000, 1_000_000_000),
            (1_776_000_000, 1_000_000_000),
            (1_776_000_000_000, 1_000_000_000),
            (10_500_000_000_000_000, 500_000_000), // First halving event
            (10_999_999_000_000_000, 500_000_000),
            (11_000_000_000_000_000, 500_000_000),
            (12_000_999_000_000_000, 500_000_000),
            (15_749_999_000_000_000, 500_000_000),
            (15_800_000_000_000_000, 250_000_000), // Second halving event
            (16_400_999_000_000_000, 250_000_000),
            (16_499_999_000_000_000, 250_000_000),
            (17_624_999_000_000_000, 250_000_000),
            (18_400_000_000_000_000, 125_000_000), // Third halving event
            (19_312_500_000_000_000, 125_000_000),
            (19_700_000_000_000_000, 62_500_000), // Fourth halving event
            (19_906_249_000_000_000, 62_500_000),
            (20_400_000_000_000_000, 31_250_000), // Fifth halving event
            (20_500_000_000_000_000, 31_250_000),
            (20_700_000_000_000_000, 15_625_000), // Sixth halving event
            (20_800_000_000_000_000, 15_625_000),
            (20_900_000_000_000_000, 7_812_500), // Seventh halving event
            (20_917_970_000_000_000, 3_906_250), // Eighth halving event
            (20_958_985_000_000_000, 1_953_125), // Ninth halving event
            (20_979_493_000_000_000, 976_562),   // Tenth halving event
            (20_989_747_000_000_000, 488_281),   // Eleventh halving event
            (20_994_874_000_000_000, 244_140),   // Twelfth halving event
            (20_997_437_000_000_000, 122_070),   // Thirteenth halving event
            (20_998_719_000_000_000, 61_035),    // Fourteenth halving event
            (20_999_360_000_000_000, 30_517),    // Fifteenth halving event
            (20_999_680_000_000_000, 15_258),    // Sixteenth halving event
            (20_999_840_000_000_000, 7_629),     // Seventeenth halving event
            (20_999_920_000_000_000, 3_814),     // Eighteenth halving event
            (20_999_960_000_000_000, 1_907),     // Nineteenth halving event
            (20_999_980_000_000_000, 953),       // Twentieth halving event
            (20_999_990_000_000_000, 476),       // Twenty-first halving event
            (20_999_990_500_000_000, 476),
            (20_999_995_000_000_000, 238), // Twenty-second halving event
            (20_999_998_000_000_000, 119), // Twenty-third halving event
            (20_999_999_000_000_000, 59),  // Twenty-fourth halving event
            (21_000_000_000_000_000, 0),   // Total supply reached, emissions stop
            (21_100_000_000_000_000, 0),   // Just for fun
            (u64::MAX, 0),                 // Testing bounds
        ];

        for (issuance, expected_emission) in expected_emissions.iter() {
            SubtensorModule::set_total_issuance(*issuance);
            step_block(1);

            let current_emission = SubtensorModule::get_block_emission().unwrap();
            assert_eq!(
                current_emission, *expected_emission,
                "Incorrect emission {} at total issuance {}",
                current_emission, issuance
            );
        }
    });
}

#[test]
fn test_get_emission_across_entire_issuance_range() {
    new_test_ext(1).execute_with(|| {
        let total_supply: u64 = pallet_subtensor::TotalSupply::<Test>::get();
        let original_emission: u64 = pallet_subtensor::DefaultBlockEmission::<Test>::get();
        let halving_issuance: u64 = total_supply / 2;

        let mut issuance = 0;

        // Issuance won't reach total supply.
        while issuance <= 20_900_000_000_000_000 {
            SubtensorModule::set_total_issuance(issuance);

            let issuance_f64 = issuance as f64;
            let h = f64::log2(1.0 / (1.0 - issuance_f64 / (2.0 * halving_issuance as f64)));
            let h = h.floor();
            let emission_percentage = f64::powf(2.0, -h);

            let expected_emission: u64 = if issuance < total_supply {
                (original_emission as f64 * emission_percentage) as u64
            } else {
                0
            };
            assert_eq!(
                SubtensorModule::get_block_emission().unwrap(),
                expected_emission,
                "Issuance: {}",
                issuance_f64
            );

            issuance += expected_emission;
        }
    });
}

#[test]
fn test_dissolve_network_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let hotkey = U256::from(1);

        add_network(netuid, 0, 0);
        let owner_coldkey = SubtensorModule::get_subnet_owner(netuid);
        register_ok_neuron(netuid, hotkey, owner_coldkey, 3);

        assert!(SubtensorModule::if_subnet_exist(netuid));
        assert_ok!(SubtensorModule::dissolve_network(
            RuntimeOrigin::signed(owner_coldkey),
            netuid
        ));
        assert!(!SubtensorModule::if_subnet_exist(netuid))
    });
}

#[test]
fn test_dissolve_network_refund_coldkey_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let hotkey = U256::from(1);
        let subnet_locked_balance = 1000;

        add_network(netuid, 0, 0);
        let owner_coldkey = SubtensorModule::get_subnet_owner(netuid);
        register_ok_neuron(netuid, hotkey, owner_coldkey, 3);

        SubtensorModule::set_subnet_locked_balance(netuid, subnet_locked_balance);
        let coldkey_balance = SubtensorModule::get_coldkey_balance(&owner_coldkey);

        assert!(SubtensorModule::if_subnet_exist(netuid));
        assert_ok!(SubtensorModule::dissolve_network(
            RuntimeOrigin::signed(owner_coldkey),
            netuid
        ));
        assert!(!SubtensorModule::if_subnet_exist(netuid));

        let coldkey_new_balance = SubtensorModule::get_coldkey_balance(&owner_coldkey);

        assert!(coldkey_new_balance > coldkey_balance);
        assert_eq!(coldkey_new_balance, coldkey_balance + subnet_locked_balance);
    });
}

#[test]
fn test_dissolve_network_not_owner_err() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let hotkey = U256::from(1);
        let owner_coldkey = U256::from(2);
        let random_coldkey = U256::from(3);

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, owner_coldkey, 3);

        assert_err!(
            SubtensorModule::dissolve_network(RuntimeOrigin::signed(random_coldkey), netuid),
            Error::<Test>::NotSubnetOwner
        );
    });
}

#[test]
fn test_dissolve_network_does_not_exist_err() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 30;
        let coldkey = U256::from(2);

        assert_err!(
            SubtensorModule::dissolve_network(RuntimeOrigin::signed(coldkey), netuid),
            Error::<Test>::SubNetworkDoesNotExist
        );
    });
}

#[test]
fn test_stao_dtao_transition_basic() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let hotkey1 = U256::from(1);
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        // Make sure TotalSubnetTAO and SubStake were initialized
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            lock_cost,
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1, netuid),
            stake,
        );
        assert_eq!(TotalSubnetTAO::<Test>::get(netuid), lock_cost + stake,);

        let coldkey1_balance_before = SubtensorModule::get_coldkey_balance(&coldkey1);
        let coldkey2_balance_before = SubtensorModule::get_coldkey_balance(&coldkey2);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid,));

        // Let transition run
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that everybody kept their stake
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            lock_cost,
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1, netuid),
            stake,
        );

        // TotalSubnetTAO is not changed
        assert_eq!(TotalSubnetTAO::<Test>::get(netuid), lock_cost + stake);

        // Re-staked balance of owner and delegators is not available as balance
        let coldkey1_balance_after = SubtensorModule::get_coldkey_balance(&coldkey1);
        let coldkey2_balance_after = SubtensorModule::get_coldkey_balance(&coldkey2);
        assert_eq!(coldkey1_balance_after, coldkey1_balance_before);
        assert_eq!(coldkey2_balance_after, coldkey2_balance_before);
    });
}

// TODOSDT: Unignore and fix
#[ignore]
#[test]
fn test_stao_dtao_transition_non_owner_fail() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        // Start transition using non-owner coldkey
        assert_err!(
            SubtensorModule::do_start_stao_dtao_transition(netuid),
            Error::<Test>::NotSubnetOwner
        );
    });
}

#[test]
fn test_stao_dtao_transition_waits_for_drain() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let hotkey1 = U256::from(1);
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;

        // We'll need two subnets so that new alpha stakes are different from old tao stakes
        create_staked_stao_network(netuid1, lock_cost, stake);
        create_staked_stao_network(netuid2, lock_cost, stake);

        // Set emission values for this subnet
        PendingEmission::<Test>::insert(netuid1, 123);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid1));

        // Let transition run (pending emission is non-zero)
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that everybody's SubStake is still the same
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid1),
            lock_cost,
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1, netuid1),
            stake,
        );

        // Drain emission
        PendingEmission::<Test>::insert(netuid1, 0);

        // Let transition run (pending emission is zero)
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that everybody's SubStake is now different
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid1),
            lock_cost * 2,
        );
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey2, &hotkey1, netuid1),
            stake * 2,
        );

        // TAO amount is still the same
        assert_eq!(TotalSubnetTAO::<Test>::get(netuid1), lock_cost + stake);
    });
}

#[test]
fn test_staking_during_dtao_transition_fails() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey2 = U256::from(2);
        let hotkey1 = U256::from(1);
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, stake);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Check that staking fails
        assert_err!(
            SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
                hotkey1,
                netuid,
                stake
            ),
            Error::<Test>::TemporarilyNotAllowed
        );
    });
}

#[test]
fn test_staking_after_dtao_transition_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let hotkey1 = U256::from(1);
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);
        SubtensorModule::add_balance_to_coldkey_account(
            &coldkey2,
            stake + ExistentialDeposit::get(),
        );

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Let transition run
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that everybody keeps their stakes
        assert_eq!(
            SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey1, &hotkey1, netuid),
            lock_cost,
        );
        assert_eq!(TotalSubnetTAO::<Test>::get(netuid), lock_cost + stake);

        // Check that staking succeeds
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey2),
            hotkey1,
            netuid,
            stake
        ));
    });
}

#[test]
fn test_run_coinbase_during_dtao_transition_no_effect() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Check that run_coinbase doesn't increase PendingEmission or TotalSubnetTAO for this subnet
        SubtensorModule::run_coinbase(2);
        assert_eq!(PendingEmission::<Test>::get(netuid), 0,);
        assert_eq!(TotalSubnetTAO::<Test>::get(netuid), lock_cost + stake);
    });
}

#[test]
fn test_run_coinbase_after_dtao_transition_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Let transition run
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that run_coinbase increases PendingEmission or TotalSubnetTAO for this subnet
        let total_tao_before = TotalSubnetTAO::<Test>::get(netuid);
        SubtensorModule::run_coinbase(2);
        let total_tao_after = TotalSubnetTAO::<Test>::get(netuid);
        assert_eq!(
            PendingEmission::<Test>::get(netuid),
            SubtensorModule::get_block_emission().unwrap(),
        );
        assert!(total_tao_before < total_tao_after);
    });
}

// Own stake of subnet owner key is converted to dynamic pool as if it was the creation of the dynamic subnet.
#[test]
fn test_stao_dtao_transition_dynamic_variables() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey1 = U256::from(1);
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Let transition run
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check dynamic variables
        assert_eq!(
            SubtensorModule::get_hotkey_global_dynamic_tao(&hotkey1),
            lock_cost + stake,
        );
        assert_eq!(
            pallet_subtensor::DynamicTAOReserve::<Test>::get(netuid),
            lock_cost + stake,
        );
        assert_eq!(
            pallet_subtensor::DynamicAlphaReserve::<Test>::get(netuid),
            lock_cost + stake,
        );
        assert_eq!(
            pallet_subtensor::DynamicAlphaOutstanding::<Test>::get(netuid),
            lock_cost + stake,
        );
        assert_eq!(
            pallet_subtensor::DynamicK::<Test>::get(netuid),
            (lock_cost + stake) as u128 * (lock_cost + stake) as u128,
        );
        assert!(pallet_subtensor::IsDynamic::<Test>::get(netuid));

        // DynamicTAOReserve will be set to equal the new value of TotalSubnetTAO (test)
        assert_eq!(
            pallet_subtensor::DynamicTAOReserve::<Test>::get(netuid),
            TotalSubnetTAO::<Test>::get(netuid),
        );
    });
}

#[test]
fn test_stao_dtao_transition_keeps_staker() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let hotkey1 = U256::from(1);
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Let transition run
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check staker map for owner and for delegator (should remain)
        assert!(pallet_subtensor::Staker::<Test>::get(hotkey1, coldkey1));
        assert!(pallet_subtensor::Staker::<Test>::get(hotkey1, coldkey2));
    });
}

// Subnet tempo is set to default value
#[test]
fn test_stao_dtao_transition_resets_tempo() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Let transition run
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that tempo went default
        assert_eq!(
            pallet_subtensor::Tempo::<Test>::get(netuid),
            <mock::Test as pallet_subtensor::Config>::InitialTempo::get(),
        );
    });
}

// High weight test - many SubStake records, so that do_continue_stao_dtao_transition runs multiple times
#[test]
fn test_stao_dtao_transition_high_weight_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let hotkey1 = U256::from(1);
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid, lock_cost, stake);

        let items = 1000;

        for i in 3..=items + 2 {
            let coldkey = U256::from(i);
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake);
            SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
                &coldkey, &hotkey1, netuid, stake,
            );
            TotalSubnetTAO::<Test>::mutate(netuid, |locked| *locked = locked.saturating_add(stake));
        }

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition(netuid));

        // Let transition run one time
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that transition hasn't finished yet
        assert!(SubnetInTransition::<Test>::get(netuid).is_some());

        // Check that transition finishes eventually
        loop {
            SubtensorModule::do_continue_stao_dtao_transition();

            if SubnetInTransition::<Test>::get(netuid).is_none() {
                break;
            }
        }
    });
}

#[test]
fn test_stao_dtao_transition_multi_network() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid1, lock_cost, stake);
        create_staked_stao_network(netuid2, lock_cost, stake);

        // Start transition
        assert_ok!(SubtensorModule::do_start_stao_dtao_transition_for_all());

        // Check that transition started for all networks
        assert!(pallet_subtensor::IsDynamic::<Test>::get(netuid1));
        assert!(pallet_subtensor::IsDynamic::<Test>::get(netuid2));
        assert!(SubnetInTransition::<Test>::get(netuid1).is_some());
        assert!(SubnetInTransition::<Test>::get(netuid2).is_some());

        // Let transition run (two times)
        SubtensorModule::do_continue_stao_dtao_transition();
        SubtensorModule::do_continue_stao_dtao_transition();

        // Check that all transitions finished
        assert!(SubnetInTransition::<Test>::get(netuid1).is_none());
        assert!(SubnetInTransition::<Test>::get(netuid2).is_none());
    });
}

#[test]
fn test_stao_dtao_transition_multi_network_fails_on_no_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let lock_cost = 100_000_000_000;
        let stake = 100_000_000_000;
        create_staked_stao_network(netuid1, lock_cost, stake);
        create_staked_stao_network(netuid2, lock_cost, stake);

        // Remove stake from netuid 2
        pallet_subtensor::TotalSubnetTAO::<Test>::insert(netuid2, 0);

        // Start transition
        assert_err!(
            SubtensorModule::do_start_stao_dtao_transition_for_all(),
            Error::<Test>::NoStakeInSubnet
        );
    });
}