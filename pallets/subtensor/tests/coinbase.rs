#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use crate::mock::*;
mod mock;
use coinbase::block_emission;
use frame_support::{assert_err, assert_ok};
use pallet_subtensor::*;
use sp_core::Get;
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

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --workspace --test coinbase -- test_dynamic_flow --exact --nocapture
#[test]
fn test_dynamic_flow() {
    new_test_ext(1).execute_with(|| {
        // Create two subnets with different mechanisms
        let netuid = 1;
        let stake: u64 = 1000000;
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        add_network(netuid, 110, 100);
        Tempo::<Test>::insert(netuid, 1);
        SubnetMechanism::<Test>::insert(netuid, 1);
        SubnetTAO::<Test>::insert(netuid, 10_000_000_000); // 10 TAO
        SubnetAlphaIn::<Test>::insert(netuid, 100_000_000_000); // 100 Alpha
        SubtensorModule::register_neuron( netuid, &hotkey );
        SubtensorModule::stake_into_subnet( &hotkey, &coldkey, netuid, stake ); // Add Stake to hotkey.
        step_block(1);
        SubnetTAO::<Test>::insert(netuid, 100_000_000_000); // 100 TAO
        SubnetAlphaIn::<Test>::insert(netuid, 10_000_000_000); // 10 Alpha price = 10.
        step_block(1);

    });
}
