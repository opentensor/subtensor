#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use crate::mock::*;
mod mock;
use frame_support::{assert_err, assert_ok};
use pallet_subtensor::{
    AdjustmentInterval, Burn, BurnRegistrationsThisInterval, Difficulty, LastAdjustmentBlock,
    MaxDifficulty, MinDifficulty, POWRegistrationsThisInterval, RegistrationsThisBlock,
    RegistrationsThisInterval, TargetRegistrationsPerInterval,
};
use sp_core::U256;

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] increases pow difficulty
// when necesarry.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_pow_difficulty_increase -- --nocapture
#[test]
fn test_pow_difficulty_increase() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 90);
        // Setup registrations_this_interval > target_registrations_this_interval
        RegistrationsThisInterval::<Test>::insert(netuid, 100);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 50);
        // Setup pow_registrations_this_interval > burn_registrations_this_interval
        POWRegistrationsThisInterval::<Test>::insert(netuid, 60);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 40);

        // Run adjustment
        let pow_difficulty_before = Difficulty::<Test>::get(netuid);
        let burn_before = Burn::<Test>::get(netuid);
        System::reset_events();
        SubtensorModule::adjust_registration_terms_for_networks();
        let pow_difficulty_after = Difficulty::<Test>::get(netuid);
        let burn_after = Burn::<Test>::get(netuid);

        assert!(
            pow_difficulty_after > pow_difficulty_before,
            "PoW difficulty must increase"
        );
        assert_eq!(burn_before, burn_after, "Burn cost should remain the same");
        System::assert_has_event(
            pallet_subtensor::Event::DifficultySet(netuid, 18446744073709551615).into(),
        );
    });
}

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] increases burn cost
// when necesarry.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_burn_cost_increase -- --nocapture
#[test]
fn test_burn_cost_increase() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 90);
        // Setup registrations_this_interval > target_registrations_this_interval
        RegistrationsThisInterval::<Test>::insert(netuid, 100);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 50);
        // Setup pow_registrations_this_interval < burn_registrations_this_interval
        POWRegistrationsThisInterval::<Test>::insert(netuid, 40);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 60);

        // Run adjustment
        let pow_difficulty_before = Difficulty::<Test>::get(netuid);
        let burn_before = Burn::<Test>::get(netuid);
        SubtensorModule::adjust_registration_terms_for_networks();
        let pow_difficulty_after = Difficulty::<Test>::get(netuid);
        let burn_after = Burn::<Test>::get(netuid);

        assert!(burn_after > burn_before, "Burn must increase");
        assert_eq!(
            pow_difficulty_before, pow_difficulty_after,
            "PoW difficulty should remain the same"
        );
    });
}

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] increases burn cost and pow difficulty
// when necesarry.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_pow_difficulty_and_burn_cost_increase -- --nocapture
#[test]
fn test_pow_difficulty_and_burn_cost_increase() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 90);
        // Setup registrations_this_interval > target_registrations_this_interval
        RegistrationsThisInterval::<Test>::insert(netuid, 100);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 50);
        // Setup pow_registrations_this_interval == burn_registrations_this_interval
        POWRegistrationsThisInterval::<Test>::insert(netuid, 40);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 40);

        // Run adjustment
        let pow_difficulty_before = Difficulty::<Test>::get(netuid);
        let burn_before = Burn::<Test>::get(netuid);
        SubtensorModule::adjust_registration_terms_for_networks();
        let pow_difficulty_after = Difficulty::<Test>::get(netuid);
        let burn_after = Burn::<Test>::get(netuid);

        assert!(burn_after > burn_before, "Burn must increase");
        assert!(
            pow_difficulty_after > pow_difficulty_before,
            "PoW difficulty must increase"
        );
    });
}

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] decreases burn cost
// when necesarry.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_pow_difficulty_and_burn_cost_increase -- --nocapture
#[test]
fn test_burn_cost_decrease() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 90);
        // Setup registrations_this_interval < target_registrations_this_interval
        RegistrationsThisInterval::<Test>::insert(netuid, 50);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 100);
        // Setup pow_registrations_this_interval > burn_registrations_this_interval
        POWRegistrationsThisInterval::<Test>::insert(netuid, 60);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 40);
        Burn::<Test>::insert(netuid, 2);

        // Run adjustment
        let pow_difficulty_before = Difficulty::<Test>::get(netuid);
        let burn_before = Burn::<Test>::get(netuid);
        SubtensorModule::adjust_registration_terms_for_networks();
        let pow_difficulty_after = Difficulty::<Test>::get(netuid);
        let burn_after = Burn::<Test>::get(netuid);

        assert!(burn_after < burn_before, "Burn must decrease");
        assert!(
            pow_difficulty_after == pow_difficulty_before,
            "PoW difficulty must not change"
        );
    });
}

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] decreases pow difficulty
// when necesarry.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_pow_difficulty_decrease -- --nocapture
#[test]
fn test_pow_difficulty_decrease() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        MaxDifficulty::<Test>::insert(netuid, u64::MAX);
        MinDifficulty::<Test>::insert(netuid, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 90);
        // Setup registrations_this_interval < target_registrations_this_interval
        RegistrationsThisInterval::<Test>::insert(netuid, 1);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 2);
        // Setup pow_registrations_this_interval < burn_registrations_this_interval
        POWRegistrationsThisInterval::<Test>::insert(netuid, 40);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 60);

        // Run adjustment
        let pow_difficulty_before = Difficulty::<Test>::get(netuid);
        let burn_before = Burn::<Test>::get(netuid);
        SubtensorModule::adjust_registration_terms_for_networks();
        let pow_difficulty_after = Difficulty::<Test>::get(netuid);
        let burn_after = Burn::<Test>::get(netuid);

        assert_eq!(burn_after, burn_before, "Burn must not change");
        assert!(
            pow_difficulty_after < pow_difficulty_before,
            "PoW difficulty must decrease"
        );
    });
}

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] decreases pow difficulty
// and burn cost when necesarry.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_pow_difficulty_and_burn_cost_decrease -- --nocapture
#[test]
fn test_pow_difficulty_and_burn_cost_decrease() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        MaxDifficulty::<Test>::insert(netuid, u64::MAX);
        MinDifficulty::<Test>::insert(netuid, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 90);
        Burn::<Test>::insert(netuid, 2);
        // Setup registrations_this_interval < target_registrations_this_interval
        RegistrationsThisInterval::<Test>::insert(netuid, 1);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 2);
        // Setup pow_registrations_this_interval == burn_registrations_this_interval
        POWRegistrationsThisInterval::<Test>::insert(netuid, 40);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 40);

        // Run adjustment
        let pow_difficulty_before = Difficulty::<Test>::get(netuid);
        let burn_before = Burn::<Test>::get(netuid);
        SubtensorModule::adjust_registration_terms_for_networks();
        let pow_difficulty_after = Difficulty::<Test>::get(netuid);
        let burn_after = Burn::<Test>::get(netuid);

        assert!(burn_after < burn_before, "Burn must decrease");
        assert!(
            pow_difficulty_after < pow_difficulty_before,
            "PoW difficulty must decrease"
        );
    });
}

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] drains counters correctly
// when the inverval is reached.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_network_counters_drained_on_interval_reached -- --nocapture
#[test]
fn test_network_counters_drained_on_interval_reached() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        MaxDifficulty::<Test>::insert(netuid, u64::MAX);
        MinDifficulty::<Test>::insert(netuid, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 90);
        RegistrationsThisInterval::<Test>::insert(netuid, 1);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 2);
        POWRegistrationsThisInterval::<Test>::insert(netuid, 1);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 1);
        RegistrationsThisBlock::<Test>::insert(netuid, 1);

        SubtensorModule::adjust_registration_terms_for_networks();

        assert!(LastAdjustmentBlock::<Test>::get(netuid) == 100);
        assert!(RegistrationsThisInterval::<Test>::get(netuid) == 0);
        assert!(POWRegistrationsThisInterval::<Test>::get(netuid) == 0);
        assert!(BurnRegistrationsThisInterval::<Test>::get(netuid) == 0);

        // RegistrationsThisBlock always drained
        assert!(RegistrationsThisBlock::<Test>::get(netuid) == 0);
    });
}

// Test that [`SubtensorModule::adjust_registration_terms_for_networks`] doesn't drains counters
// when the inverval is not reached.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test -p pallet-subtensor --test adjust_registration_terms_for_networks test_network_counters_not_drained_when_interval_not_reached -- --nocapture
#[test]
fn test_network_counters_not_drained_when_interval_not_reached() {
    new_test_ext(100).execute_with(|| {
        // Setup interval reached
        let netuid = 1;
        add_network(netuid, 1, 0);
        MaxDifficulty::<Test>::insert(netuid, u64::MAX);
        MinDifficulty::<Test>::insert(netuid, 0);
        AdjustmentInterval::<Test>::insert(netuid, 10);
        LastAdjustmentBlock::<Test>::insert(netuid, 91);
        RegistrationsThisInterval::<Test>::insert(netuid, 2);
        POWRegistrationsThisInterval::<Test>::insert(netuid, 1);
        BurnRegistrationsThisInterval::<Test>::insert(netuid, 1);
        TargetRegistrationsPerInterval::<Test>::insert(netuid, 1);

        SubtensorModule::adjust_registration_terms_for_networks();

        assert!(RegistrationsThisInterval::<Test>::get(netuid) == 2);
        assert!(POWRegistrationsThisInterval::<Test>::get(netuid) == 1);
        assert!(BurnRegistrationsThisInterval::<Test>::get(netuid) == 1);

        // RegistrationsThisBlock always drained
        assert!(RegistrationsThisBlock::<Test>::get(netuid) == 0);
    });
}
