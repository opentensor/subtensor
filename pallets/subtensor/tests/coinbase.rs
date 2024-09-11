#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
use crate::mock::*;
mod mock;
use frame_support::assert_ok;
use pallet_subtensor::TargetStakesPerInterval;
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

// Test getting nonviable stake
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_get_nonviable_stake -- --nocapture
#[test]
fn test_get_nonviable_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1u16;
        let delegate_coldkey = U256::from(1);
        let delegate_hotkey = U256::from(2);
        let delegator = U256::from(3);

        let owner_added_stake = 123;
        let owner_removed_stake = 456;
        let owner_stake = 1_000 + owner_removed_stake;
        // Add more than removed to test that the delta is updated correctly
        let owner_adds_more_stake = owner_removed_stake + 1;

        let delegator_added_stake = 999;

        // Set stake rate limit very high
        TargetStakesPerInterval::<Test>::put(1e9 as u64);

        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, delegate_hotkey, delegate_coldkey, 0);
        // Give extra stake to the owner
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &delegate_coldkey,
            &delegate_hotkey,
            owner_stake,
        );

        // Register as a delegate
        assert_ok!(SubtensorModule::become_delegate(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey
        ));

        // Verify that the key starts with 0 nonviable stake
        assert_eq!(
            SubtensorModule::get_nonviable_stake(&delegate_hotkey, &delegate_coldkey),
            0
        );

        // Give the coldkey some balance; extra just in-case
        SubtensorModule::add_balance_to_coldkey_account(
            &delegate_coldkey,
            owner_added_stake + owner_adds_more_stake,
        );

        // Add some stake
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey,
            owner_added_stake
        ));

        // Verify the nonviable stake is the same as the added stake
        assert_eq!(
            SubtensorModule::get_nonviable_stake(&delegate_hotkey, &delegate_coldkey),
            owner_added_stake
        );

        // Add some stake from a delegator
        SubtensorModule::add_balance_to_coldkey_account(&delegator, delegator_added_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            delegate_hotkey,
            delegator_added_stake
        ));

        // Verify that the nonviable stake doesn't change when a different account adds stake
        assert_eq!(
            SubtensorModule::get_nonviable_stake(&delegate_hotkey, &delegate_coldkey),
            owner_added_stake
        );

        // Remove some stake
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey,
            owner_removed_stake
        ));

        // The stake delta is negative, so the nonviable stake should be 0
        assert_eq!(
            SubtensorModule::get_nonviable_stake(&delegate_hotkey, &delegate_coldkey),
            0
        );

        // Add more stake than was removed
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegate_coldkey),
            delegate_hotkey,
            owner_adds_more_stake
        ));

        // Verify that the nonviable stake is the net of the operations
        assert_eq!(
            SubtensorModule::get_nonviable_stake(&delegate_hotkey, &delegate_coldkey),
            owner_adds_more_stake - owner_removed_stake + owner_added_stake
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_coinbase_nominator_drainage -- --nocapture
#[test]
fn test_coinbase_nominator_drainage() {
    new_test_ext(1).execute_with(|| {
        // 1. Set up the network and accounts
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let coldkey = U256::from(3);
        let nominator1 = U256::from(1);
        let nominator2 = U256::from(2);

        log::debug!("Setting up network with netuid: {}", netuid);
        log::debug!("Hotkey: {:?}, Coldkey: {:?}", hotkey, coldkey);
        log::debug!("Nominators: {:?}, {:?}", nominator1, nominator2);

        // 2. Create network and register neuron
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 100000);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);

        log::debug!("Network created and neuron registered");

        // 3. Set up balances and stakes
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator1, 1500);
        SubtensorModule::add_balance_to_coldkey_account(&nominator2, 1500);

        log::debug!("Balances added to accounts");

        // 4. Make the hotkey a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            (u16::MAX as u64 / 10) as u16
        ));

        log::debug!("Hotkey became a delegate with minimum take");

        // Add stakes for nominators
        SubtensorModule::add_stake(RuntimeOrigin::signed(nominator1), hotkey, 100);

        SubtensorModule::add_stake(RuntimeOrigin::signed(nominator2), hotkey, 100);

        // Log the stakes for hotkey, nominator1, and nominator2
        log::debug!(
            "Initial stakes - Hotkey: {}, Nominator1: {}, Nominator2: {}",
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey, &hotkey),
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator1, &hotkey),
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator2, &hotkey)
        );
        log::debug!("Stakes added for nominators");

        // 5. Set emission and verify initial states
        SubtensorModule::set_emission_values(&[netuid], vec![10]).unwrap();
        assert_eq!(SubtensorModule::get_subnet_emission_value(netuid), 10);
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 200);
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        log::debug!("Emission set and initial states verified");

        // 6. Set hotkey emission tempo
        SubtensorModule::set_hotkey_emission_tempo(1);
        log::debug!("Hotkey emission tempo set to 1");

        // 7. Simulate blocks and check emissions
        next_block();
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 10);
        log::debug!(
            "After first block, pending emission: {}",
            SubtensorModule::get_pending_emission(netuid)
        );

        next_block();
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);
        log::debug!("After second block, pending emission drained");

        // 8. Check final stakes
        let hotkey_stake = SubtensorModule::get_stake_for_coldkey_and_hotkey(&coldkey, &hotkey);
        let nominator1_stake =
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator1, &hotkey);
        let nominator2_stake =
            SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator2, &hotkey);

        log::debug!(
            "Final stakes - Hotkey: {}, Nominator1: {}, Nominator2: {}",
            hotkey_stake,
            nominator1_stake,
            nominator2_stake
        );

        // 9. Verify distribution
        let min_take = SubtensorModule::get_min_delegate_take() as u64;
        let total_emission = 20; // 10 per block for 2 blocks
        let hotkey_emission = total_emission * min_take / u16::MAX as u64;
        let remaining_emission = total_emission - hotkey_emission;
        let nominator_emission = remaining_emission / 2;

        log::debug!(
            "Calculated emissions - Hotkey: {}, Each Nominator: {}",
            hotkey_emission,
            nominator_emission
        );

        // Debug: Print the actual stakes
        log::debug!("Actual hotkey stake: {}", hotkey_stake);
        log::debug!("Actual nominator1 stake: {}", nominator1_stake);
        log::debug!("Actual nominator2 stake: {}", nominator2_stake);

        // Debug: Check the total stake for the hotkey
        let total_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey);
        log::debug!("Total stake for hotkey: {}", total_stake);

        // Assertions
        assert_eq!(hotkey_stake, 2, "Hotkey stake mismatch");
        assert_eq!(
            nominator1_stake,
            100 + nominator_emission,
            "Nominator1 stake mismatch"
        );
        assert_eq!(
            nominator2_stake,
            100 + nominator_emission,
            "Nominator2 stake mismatch"
        );

        // 10. Check total stake
        assert_eq!(total_stake, 200 + total_emission, "Total stake mismatch");

        log::debug!("Test completed");
    });
}
