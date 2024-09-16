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

        // Create a network with a tempo 1
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 100000);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1000);

        // Set the subnet emission value to 1.
        SubtensorModule::set_emission_values(&[netuid], vec![1]).unwrap();
        assert_eq!(SubtensorModule::get_subnet_emission_value(netuid), 1);

        // Hotkey has no pending emission
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has same stake
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 1000);

        // Subnet has no pending emission.
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // Step block
        next_block();

        // Hotkey has no pending emission
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has same stake
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 1000);

        // Subnet has no pending emission of 1 ( from coinbase )
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 1);

        // Step block releases
        next_block();

        // Subnet pending has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // Hotkey pending immediately drained.
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has NEW stake
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            1000 + 2
        );

        // Set the hotkey drain time to 2 block.
        SubtensorModule::set_hotkey_emission_tempo(2);

        // Step block releases
        next_block();

        // Subnet pending increased by 1
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 1);

        // Hotkey pending not increased (still on subnet)
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has same stake
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            1000 + 2
        );

        // Step block releases
        next_block();

        // Subnet pending has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // Hotkey pending drained.
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has 2 new TAO.
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            1000 + 4
        );
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

mod adjust_registration_terms_for_networks {
    #![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
    use super::*;
    use crate::mock::*;
    use frame_support::{assert_err, assert_ok};
    use pallet_subtensor::{
        AdjustmentInterval, Burn, BurnRegistrationsThisInterval, Difficulty, LastAdjustmentBlock,
        MaxDifficulty, MinDifficulty, POWRegistrationsThisInterval, RegistrationsThisBlock,
        RegistrationsThisInterval, TargetRegistrationsPerInterval,
    };
    use sp_core::U256;

    #[test]
    fn case_a_pow_difficulty_increase() {
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

    #[test]
    fn case_b_burn_cost_increase() {
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

    #[test]
    fn case_f_pow_difficulty_and_burn_cost_increase() {
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

    #[test]
    fn case_c_burn_cost_decrease() {
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

    #[test]
    fn case_d_pow_difficulty_decrease() {
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

    #[test]
    fn case_e_pow_difficulty_and_burn_cost_decrease() {
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

    #[test]
    fn network_counters_drained_on_interval_reached() {
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

    #[test]
    fn network_counters_not_drained_when_interval_not_reached() {
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
}
