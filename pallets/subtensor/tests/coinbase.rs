#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use crate::mock::*;
mod mock;
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
        // Define network ID
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let coldkey = U256::from(3);

        // Create a network with a tempo of 1
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 100000);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, 1000);

        // Step block
        next_block();
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0); // Hotkey not pending yet.
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 1_000_000_000); // Subnet gets all pending emission.
        assert_eq!(SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid), 1000);

        // Step block
        next_block();
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);
        assert_eq!(SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid), 2_000_000_998); //1000 + 2 x block emission.
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
