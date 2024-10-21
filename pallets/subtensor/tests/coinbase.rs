#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use crate::mock::*;
mod mock;
use pallet_subtensor::*;
use sp_core::Get;
// use frame_support::{assert_err, assert_ok};
use sp_core::U256;

// Test the ability to hash all sorts of hotkeys.
#[test]

fn test_hotkey_hashing() {
    new_test_ext(1).execute_with(|| {
        for i in 0..10000 {
            SubtensorModule::hash_hotkey_to_u64(&U256::from(i));
        }
    });
}

// Test drain tempo on hotkeys.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_hotkey_drain_time -- --nocapture
#[test]

fn test_hotkey_drain_time() {
    new_test_ext(1).execute_with(|| {
        // Block 0
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(0), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(1), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(2), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(3), 0, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(4), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(5), 0, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(6), 0, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(7), 0, 1));

        // Block 1
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(0), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(1), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(2), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(3), 1, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(4), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(5), 1, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(6), 1, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(7), 1, 1));
    });
}

// To run this test specifically, use the following command:
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_coinbase_basic -- --nocapture
#[test]

fn test_coinbase_basic() {
    new_test_ext(1).execute_with(|| {
        // // Define network ID
        // let netuid: u16 = 1;
        // let hotkey = U256::from(0);
        // let coldkey = U256::from(3);

        // // Create a network with a tempo 1
        // add_network(netuid, 1, 0);
        // register_ok_neuron(netuid, hotkey, coldkey, 100000);
        // SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        // increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1000, netuid);

        // // Set the subnet emission value to 1.
        // SubtensorModule::set_emission_values(&[netuid], vec![1]).unwrap();
        // assert_eq!(SubtensorModule::get_subnet_emission_value(netuid), 1);

        // // Hotkey has no pending emission
        // assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // // Hotkey has same stake
        // assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 1000);

        // // Subnet has no pending emission.
        // assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // // Step block
        // next_block();

        // // Hotkey has no pending emission
        // assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // // Hotkey has same stake
        // assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 1000);

        // // Subnet has no pending emission of 1 ( from coinbase )
        // assert_eq!(SubtensorModule::get_pending_emission(netuid), 1);

        // // Step block releases
        // next_block();

        // // Subnet pending has been drained.
        // assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // // Hotkey pending immediately drained.
        // assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // // Hotkey has NEW stake
        // assert_eq!(
        //     SubtensorModule::get_total_stake_for_hotkey(&hotkey),
        //     1000 + 2
        // );

        // // Set the hotkey drain time to 2 block.
        // SubtensorModule::set_hotkey_emission_tempo(2);

        // // Step block releases
        // next_block();

        // // Subnet pending increased by 1
        // assert_eq!(SubtensorModule::get_pending_emission(netuid), 1);

        // // Hotkey pending not increased (still on subnet)
        // assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // // Hotkey has same stake
        // assert_eq!(
        //     SubtensorModule::get_total_stake_for_hotkey(&hotkey),
        //     1000 + 2
        // );

        // // Step block releases
        // next_block();

        // // Subnet pending has been drained.
        // assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // // Hotkey pending drained.
        // assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // // Hotkey has 2 new TAO.
        // assert_eq!(
        //     SubtensorModule::get_total_stake_for_hotkey(&hotkey),
        //     1000 + 4
        // );
    });
}

// Test getting and setting hotkey emission tempo
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_set_and_get_hotkey_emission_tempo -- --nocapture
#[test]

fn test_set_and_get_hotkey_emission_tempo() {
    new_test_ext(1).execute_with(|| {
        // Get the default hotkey emission tempo
        let default_tempo = SubtensorModule::get_hotkey_emission_tempo();
        assert_eq!(default_tempo, 0); // default is 0 in mock.rs

        // Set a new hotkey emission tempo
        let new_tempo = 5;
        SubtensorModule::set_hotkey_emission_tempo(new_tempo);

        // Get the updated hotkey emission tempo
        let updated_tempo = SubtensorModule::get_hotkey_emission_tempo();
        assert_eq!(updated_tempo, new_tempo);
    });
}

// Test run_coinbase with no subnets
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_run_coinbase_no_subnets -- --exact --nocapture
#[test]
fn test_run_coinbase_no_subnets() {
    new_test_ext(1).execute_with(|| {
        // Ensure there are no subnets
        assert_eq!(SubtensorModule::get_all_subnet_netuids().len(), 0);

        // Run coinbase
        SubtensorModule::run_coinbase();

        // Check that no emissions were distributed
        assert_eq!(SubtensorModule::get_total_issuance(), 0);
        assert_eq!(EmissionValues::<Test>::iter().count(), 0);
        assert_eq!(PendingEmission::<Test>::iter().count(), 0);
    });
}

// Test run_coinbase with single subnet
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_run_coinbase_single_subnet -- --exact --nocapture
#[test]
fn test_run_coinbase_single_subnet() {
    new_test_ext(1).execute_with(|| {
        // Create a single subnet
        let netuid = 1;
        add_network(netuid, 110, 100);

        // Set initial values
        let initial_issuance = 1_000_000;
        TotalIssuance::<Test>::put(initial_issuance);
        let block_emission = SubtensorModule::get_block_emission().unwrap();
        SubnetTAO::<Test>::insert(netuid, initial_issuance);

        // Run coinbase
        SubtensorModule::run_coinbase();

        // Check that emissions were distributed correctly
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_issuance + block_emission
        );
        assert_eq!(EmissionValues::<Test>::get(netuid), block_emission);
        assert_eq!(PendingEmission::<Test>::get(netuid), block_emission);
        assert_eq!(
            SubnetTAO::<Test>::get(netuid),
            block_emission + initial_issuance
        );
    });
}

// Test run_coinbase with multiple subnets
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_run_coinbase_multiple_subnets -- --exact --nocapture
#[test]
fn test_run_coinbase_multiple_subnets() {
    new_test_ext(1).execute_with(|| {
        // Create multiple subnets
        let netuids = vec![1, 2, 3];
        for netuid in &netuids {
            add_network(*netuid, 110, 100);
        }

        // Set initial values
        let initial_issuance = 1_000_000;
        TotalIssuance::<Test>::put(initial_issuance);
        let block_emission = SubtensorModule::get_block_emission().unwrap();
        let subnet_emission = block_emission / netuids.len() as u64;

        for netuid in &netuids {
            SubnetTAO::<Test>::insert(netuid, initial_issuance / netuids.len() as u64);
        }

        // Run coinbase
        SubtensorModule::run_coinbase();
        let total_emitted: u64 = EmissionValues::<Test>::iter().map(|(_, value)| value).sum();

        // Check that emissions were distributed correctly
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_issuance + total_emitted
        );

        for netuid in &netuids {
            assert!(
                (EmissionValues::<Test>::get(netuid) as i64 - subnet_emission as i64).abs() <= 1000
            );
            assert!(
                (PendingEmission::<Test>::get(netuid) as i64 - subnet_emission as i64).abs()
                    <= 1000
            );
            assert!(
                (SubnetTAO::<Test>::get(netuid) as i64
                    - (subnet_emission + initial_issuance / netuids.len() as u64) as i64)
                    .abs()
                    <= 1000,
                "SubnetTAO value is not within the expected range"
            );
        }
    });
}

// Test run_coinbase with zero block emission
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_run_coinbase_zero_emission -- --exact --nocapture
#[test]
fn test_run_coinbase_zero_emission() {
    new_test_ext(1).execute_with(|| {
        // Create a single subnet
        let netuid = 1;
        add_network(netuid, 110, 100);

        // Set initial values
        let initial_issuance = TotalSupply::<Test>::get();
        TotalIssuance::<Test>::put(initial_issuance);
        SubnetTAO::<Test>::insert(netuid, initial_issuance);

        // Run coinbase
        SubtensorModule::run_coinbase();

        // Check that no emissions were distributed
        assert_eq!(SubtensorModule::get_total_issuance(), initial_issuance);
        assert_eq!(EmissionValues::<Test>::get(netuid), 0);
        assert_eq!(PendingEmission::<Test>::get(netuid), 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid), initial_issuance);
    });
}

// Test run_coinbase with different subnet mechanisms
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_run_coinbase_different_mechanisms -- --exact --nocapture
#[test]
fn test_run_coinbase_different_mechanisms() {
    new_test_ext(1).execute_with(|| {
        // Create two subnets with different mechanisms
        let netuid1 = 1;
        let netuid2 = 2;
        add_network(netuid1, 110, 100);
        add_network(netuid2, 110, 100);

        // Set different mechanisms for each subnet
        SubnetMechanism::<Test>::insert(netuid1, 0); // Stable mechanism
        SubnetMechanism::<Test>::insert(netuid2, 1); // Dynamic mechanism

        // Set initial values
        let initial_issuance = 1_000_000;
        let block_emission = SubtensorModule::get_block_emission().unwrap();
        TotalIssuance::<Test>::put(initial_issuance);
        SubnetTAO::<Test>::insert(netuid1, initial_issuance / 2);
        SubnetTAO::<Test>::insert(netuid2, initial_issuance / 2);

        // Run coinbase
        SubtensorModule::run_coinbase();

        // Check that emissions were distributed correctly
        let total_emitted: u64 = EmissionValues::<Test>::iter().map(|(_, value)| value).sum();
        assert!(
            total_emitted > 0,
            "Total emitted should be greater than zero"
        );

        // Check subnet-specific behavior
        let emission1 = EmissionValues::<Test>::get(netuid1);
        let emission2 = EmissionValues::<Test>::get(netuid2);

        // For stable mechanism (netuid1)
        assert_eq!(
            PendingEmission::<Test>::get(netuid1),
            emission1,
            "Pending emission should equal emission for stable mechanism"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid1),
            0,
            "SubnetAlphaIn should be zero for stable mechanism"
        );

        // For dynamic mechanism (netuid2)
        assert_eq!(
            PendingEmission::<Test>::get(netuid2),
            block_emission,
            "Pending emission should be equal to mechanism emission for dynamic mechanism"
        );
        assert_eq!(
            SubnetAlphaIn::<Test>::get(netuid2),
            block_emission,
            "SubnetAlphaIn should equal to mechanism emission for dynamic mechanism"
        );

        // Check total issuance
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_issuance + total_emitted,
            "Total issuance should increase by total emitted"
        );
    });
}
