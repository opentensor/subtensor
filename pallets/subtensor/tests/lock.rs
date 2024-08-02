mod mock;
use mock::*;
use sp_core::U256;
use pallet_subtensor::*;
use frame_support::{assert_ok, assert_noop};
use frame_system::RawOrigin;

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_success --exact --nocapture
#[test]
fn test_do_lock_success() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = 500_000_000;
        let lock_amount = 250_000_000;
        let lock_duration = 7200 * 30; // 30 days

        // Set up initial balance and stake
        add_network(netuid, 0, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, stake_amount));
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration, lock_amount));

        // Check that the lock was created correctly
        let (locked_amount, start_block, end_block) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount, lock_amount);
        assert_eq!(end_block, start_block + lock_duration);

        // Verify the event was emitted
        System::assert_last_event(Event::LockIncreased(coldkey, hotkey, netuid, lock_amount).into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_subnet_not_exists --exact --nocapture
#[test]
fn test_do_lock_subnet_not_exists() {
    new_test_ext(1).execute_with(|| {
        let non_existent_netuid = 99;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let lock_amount = 250_000_000;
        let lock_duration = 7200 * 30; // 30 days

        // Attempt to lock stake on a non-existent subnet
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, non_existent_netuid, lock_duration, lock_amount),
            Error::<Test>::SubnetNotExists
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_hotkey_not_exists --exact --nocapture
#[test]
fn test_do_lock_hotkey_not_exists() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let non_existent_hotkey = U256::from(99);
        let lock_amount = 250_000_000;
        let lock_duration = 7200 * 30; // 30 days

        // Set up network
        add_network(netuid, 0, 0);

        // Attempt to lock stake with a non-existent hotkey
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), non_existent_hotkey, netuid, lock_duration, lock_amount),
            Error::<Test>::HotKeyAccountNotExists
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_hotkey_not_registered --exact --nocapture
#[test]
fn test_do_lock_hotkey_not_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = 1;
        let netuid2 = 2;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let lock_amount = 250_000_000;
        let lock_duration = 7200 * 30; // 30 days

        // Set up network
        add_network(netuid1, 0, 0);
        add_network(netuid2, 0, 0);
        // Make hotkey exist.
        register_ok_neuron(netuid2, hotkey, coldkey, 11);

        // Add balance to coldkey
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);

        // Attempt to lock stake with an unregistered hotkey
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid1, lock_duration, lock_amount),
            Error::<Test>::HotKeyNotRegisteredInSubNet
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_zero_amount --exact --nocapture
#[test]
fn test_do_lock_zero_amount() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let lock_amount = 0;
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);

        // Attempt to lock zero stake
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration, lock_amount),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_insufficient_stake --exact --nocapture
#[test]
fn test_do_lock_insufficient_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = 500_000_000;
        let lock_amount = 750_000_000; // More than available stake
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, stake_amount));

        // Attempt to lock more stake than available
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration, lock_amount),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_increase_conviction --exact --nocapture
#[test]
fn test_do_lock_increase_conviction() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let initial_lock_amount = 500_000_000;
        let initial_lock_duration = 7200 * 30; // 30 days
        let new_lock_amount = 750_000_000;
        let new_lock_duration = 7200 * 60; // 60 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Initial lock
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_lock_duration, initial_lock_amount));

        // Increase conviction with new lock
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, new_lock_duration, new_lock_amount));

        // Verify the new lock
        let (locked_amount, start_block, end_block) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount, new_lock_amount);
        assert_eq!(end_block, SubtensorModule::get_current_block_as_u64() + new_lock_duration);

        // Verify event emission
        System::assert_last_event(Event::LockIncreased(coldkey, hotkey, netuid, new_lock_amount).into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_decrease_conviction --exact --nocapture
#[test]
fn test_do_lock_decrease_conviction() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let initial_lock_amount = 500_000_000;
        let initial_lock_duration = 7200 * 30; // 30 days
        let new_lock_amount = 400_000_000;
        let new_lock_duration = 7200 * 20; // 20 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Initial lock
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_lock_duration, initial_lock_amount));

        // Attempt to decrease conviction with new lock
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, new_lock_duration, new_lock_amount),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Verify the lock remains unchanged
        let (locked_amount, _, end_block) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount, initial_lock_amount);
        assert_eq!(end_block, SubtensorModule::get_current_block_as_u64() + initial_lock_duration);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_max_duration --exact --nocapture
#[test]
fn test_do_lock_max_duration() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = 500_000_000;
        let max_lock_duration = 7200 * 365; // 1 year (assuming 7200 blocks per day)

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Lock stake for maximum duration
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, max_lock_duration, lock_amount));

        // Verify the lock
        let (locked_amount, start_block, end_block) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount, lock_amount);
        assert_eq!(end_block, SubtensorModule::get_current_block_as_u64() + max_lock_duration);

        // Verify event emission
        System::assert_last_event(Event::LockIncreased(coldkey, hotkey, netuid, lock_amount).into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_multiple_times --exact --nocapture
#[test]
fn test_do_lock_multiple_times() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount_1 = 300_000_000;
        let lock_amount_2 = 500_000_000;
        let lock_duration_1 = 7200 * 30; // 30 days
        let lock_duration_2 = 7200 * 60; // 60 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // First lock
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration_1, lock_amount_1));

        // Verify the first lock
        let (locked_amount_1, start_block_1, end_block_1) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount_1, lock_amount_1);
        assert_eq!(end_block_1, start_block_1 + lock_duration_1);

        // Second lock
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration_2, lock_amount_2));

        // Verify the second lock
        let (locked_amount_2, start_block_2, end_block_2) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount_2, lock_amount_2);
        assert_eq!(end_block_2, start_block_2 + lock_duration_2);

        // Ensure the locked amount increased
        assert!(locked_amount_2 > locked_amount_1);

        // Verify event emissions
        System::assert_has_event(Event::LockIncreased(coldkey, hotkey, netuid, lock_amount_1).into());
        System::assert_last_event(Event::LockIncreased(coldkey, hotkey, netuid, lock_amount_2).into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_different_subnets --exact --nocapture
#[test]
fn test_do_lock_different_subnets() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = 1;
        let netuid2 = 2;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount_1 = 300_000_000;
        let lock_amount_2 = 500_000_000;
        let lock_duration = 7200 * 30; // 30 days

        // Set up networks and register neuron
        add_network(netuid1, 0, 0);
        add_network(netuid2, 0, 0);
        register_ok_neuron(netuid1, hotkey, coldkey, 11);
        register_ok_neuron(netuid2, hotkey, coldkey, 12);

        // Add balance to coldkey and stake on both networks
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake * 2);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid1, initial_stake));
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid2, initial_stake));

        // Lock stake on first subnet
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid1, lock_duration, lock_amount_1));

        // Verify the lock on first subnet
        let (locked_amount_1, start_block_1, end_block_1) = Locks::<Test>::get((netuid1, hotkey, coldkey));
        assert_eq!(locked_amount_1, lock_amount_1);
        assert_eq!(end_block_1, start_block_1 + lock_duration);

        // Lock stake on second subnet
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid2, lock_duration, lock_amount_2));

        // Verify the lock on second subnet
        let (locked_amount_2, start_block_2, end_block_2) = Locks::<Test>::get((netuid2, hotkey, coldkey));
        assert_eq!(locked_amount_2, lock_amount_2);
        assert_eq!(end_block_2, start_block_2 + lock_duration);

        // Ensure the locks are independent
        assert_ne!(locked_amount_1, locked_amount_2);

        // Verify event emissions
        System::assert_has_event(Event::LockIncreased(coldkey, hotkey, netuid1, lock_amount_1).into());
        System::assert_last_event(Event::LockIncreased(coldkey, hotkey, netuid2, lock_amount_2).into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_fully_locked --exact --nocapture
#[test]
fn test_remove_stake_fully_locked() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = initial_stake - 1;
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Lock all stake
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration, lock_amount));

        // Attempt to remove stake
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Verify lock remains unchanged
        let (locked_amount, _, _) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount, lock_amount);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_partially_locked --exact --nocapture
#[test]
fn test_remove_stake_partially_locked() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = 600_000_000;
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        SubtensorModule::set_target_stakes_per_interval(10);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Lock part of the stake
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration, lock_amount));

        // Attempt to remove unlocked portion (should succeed)
        let unlocked_amount = initial_stake - lock_amount;
        assert_ok!(SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, unlocked_amount));

        // Verify stake and lock after removal
        let stake_after_removal = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(stake_after_removal, lock_amount-1);

        let remaining_lock = lock_amount - unlocked_amount;
        let (locked_amount, _, _) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount, remaining_lock);

        // Attempt to remove more than unlocked portion (should fail)
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, 1),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Verify stake and lock remain unchanged
        let stake_after_failed_removal = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(stake_after_failed_removal, lock_amount-1);

        let (locked_amount_after_failed_removal, _, _) = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(locked_amount_after_failed_removal, remaining_lock);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_after_lock_expiry --exact --nocapture
#[test]
fn test_remove_stake_after_lock_expiry() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = 600_000_000;
        let lock_duration = 10;// 10 blocks

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        SubtensorModule::set_target_stakes_per_interval(10);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Lock part of the stake
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration, lock_amount));

        // Fast forward to just after lock expiry
        run_to_block(lock_duration + 1);

        // Attempt to remove all stake (should succeed)
        assert_ok!(SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake-1));

        // Verify stake and lock after removal
        let stake_after_removal = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(stake_after_removal, 0);

        // Verify lock is removed
        assert!(!Locks::<Test>::contains_key((netuid, hotkey, coldkey)));

        // Verify balance is returned to coldkey
        let coldkey_balance = SubtensorModule::get_coldkey_balance(&coldkey);
        assert_eq!(coldkey_balance, initial_stake);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_multiple_locks --exact --nocapture
#[test]
fn test_remove_stake_multiple_locks() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount_1 = 300_000_000;
        let lock_amount_2 = 400_000_000;
        let lock_duration_1 = 10; // 10 blocks
        let lock_duration_2 = 10; // 10 blocks

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        SubtensorModule::set_target_stakes_per_interval(10);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Create two locks
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration_1, lock_amount_1)); // first lock.
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration_2, lock_amount_2)); // second replaces first.

        // Attempt to remove more stake than unlocked (should fail)
        let lock_with_unlock: u64 = (lock_amount_2 -  SubtensorModule::calculate_conviction(lock_amount_2, lock_duration_2, 0));
        let unlocked_amount = (initial_stake - lock_with_unlock); // unstake more than allowed.
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, unlocked_amount + 1),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Remove unlocked stake (should succeed)
        assert_ok!(SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, unlocked_amount-1));

        // Verify remaining stake
        let remaining_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(remaining_stake, lock_with_unlock);

        // Fast forward to after first lock expiry
        run_to_block(lock_duration_2 + 1);

        // Attempt to remove more stake than available (should fail)
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Verify remaining stake
        let remaining_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let lock_with_unlock: u64 = (lock_amount_2 -  SubtensorModule::calculate_conviction(lock_amount_2, lock_duration_2, 0));
        let unlocked_amount = (remaining_stake - lock_with_unlock); // unstake more than allowed.
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, unlocked_amount),
            Error::<Test>::StakeToWithdrawIsZero
        );

        // Remove remainder no problem.
        SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid));

        // Verify no stake remains
        let final_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(final_stake, 0);

        // Verify all locks are removed
        assert!(!Locks::<Test>::contains_key((netuid, hotkey, coldkey)));

        // Verify balance is returned to coldkey
        let coldkey_balance = SubtensorModule::get_coldkey_balance(&coldkey);
        assert_eq!(coldkey_balance, initial_stake);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_conviction_calculation --exact --nocapture
#[test]
fn test_remove_stake_conviction_calculation() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = 500_000_000;
        let lock_duration = 10; // 10 blocks

        // Register and add stake
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        SubtensorModule::set_target_stakes_per_interval(10);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, initial_stake));

        // Lock stake
        assert_ok!(SubtensorModule::do_lock(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_duration, lock_amount));

        // Calculate conviction
        let current_block = SubtensorModule::get_current_block_as_u64();
        let conviction = SubtensorModule::calculate_conviction(lock_amount, current_block + lock_duration, current_block);

        // Try to remove more stake than allowed by conviction
        let max_removable = initial_stake - (lock_amount - conviction);
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, max_removable),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Remove allowed amount of stake
        assert_ok!(SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, max_removable-1));

        // Verify remaining stake
        let remaining_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(remaining_stake, lock_amount - conviction);

        // Fast forward to just before lock expiry
        run_to_block(lock_duration-1);

        // Try to remove all remaining stake (should fail)
        assert_noop!(
            SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, remaining_stake),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Fast forward to after lock expiry
        run_to_block(lock_duration + 1);

        // Now remove all remaining stake (should succeed)
        assert_ok!(SubtensorModule::remove_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, remaining_stake));

        // Verify no stake remains
        let final_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(final_stake, 0);

        // Verify lock is removed
        assert!(!Locks::<Test>::contains_key((netuid, hotkey, coldkey)));
    });
}

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_partial_lock_removal --exact --nocapture
// #[test]
// fn test_remove_stake_partial_lock_removal() {
//     // Test removing part of a locked stake
//     // Should update the lock amount correctly if partial removal is allowed
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_full_lock_removal --exact --nocapture
// #[test]
// fn test_remove_stake_full_lock_removal() {
//     // Test removing all of a locked stake
//     // Should remove the lock entirely if all locked stake is removed
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_across_subnets --exact --nocapture
// #[test]
// fn test_remove_stake_across_subnets() {
//     // Test removing stake from different subnets with different lock conditions
//     // Should respect locks on each subnet independently
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_rate_limiting --exact --nocapture
// #[test]
// fn test_remove_stake_rate_limiting() {
//     // Test that stake removal respects the rate limiting rules
//     // Should fail if trying to remove stake too frequently
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_remove_stake_hotkey_not_registered --exact --nocapture
// #[test]
// fn test_remove_stake_hotkey_not_registered() {
//     // Test removing stake for a hotkey that is not registered
//     // Should fail with appropriate error
// }


// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_failures --exact --nocapture
// #[test]
// fn test_do_lock_failures() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = 1;
//         let coldkey = U256::from(1);
//         let hotkey = U256::from(2);
//         let non_existent_netuid = 99;
//         let non_existent_hotkey = U256::from(99);
//         let stake_amount = 500_000_000;
//         let lock_amount = 250_000_000;
//         let lock_duration = 7200 * 30; // 30 days

//         // Set up initial balance and stake
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
//         assert_ok!(SubtensorModule::register(RuntimeOrigin::signed(coldkey), netuid, hotkey));
//         assert_ok!(SubtensorModule::increase_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, stake_amount));

//         // Test: Subnet does not exist
//         assert_noop!(
//             SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, non_existent_netuid, lock_amount, lock_duration),
//             Error::<Test>::SubnetNotExists
//         );

//         // Test: Hotkey account does not exist
//         assert_noop!(
//             SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), non_existent_hotkey, netuid, lock_amount, lock_duration),
//             Error::<Test>::HotKeyAccountNotExists
//         );

//         // Test: Hotkey not registered in subnet
//         let unregistered_hotkey = U256::from(3);
//         assert_noop!(
//             SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), unregistered_hotkey, netuid, lock_amount, lock_duration),
//             Error::<Test>::HotKeyNotRegisteredInSubNet
//         );

//         // Test: Lock amount is zero
//         assert_noop!(
//             SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, 0, lock_duration),
//             Error::<Test>::NotEnoughStakeToWithdraw
//         );

//         // Test: Not enough stake to lock
//         assert_noop!(
//             SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, stake_amount + 1, lock_duration),
//             Error::<Test>::NotEnoughStakeToWithdraw
//         );
//     });
// }


// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_distribute_owner_cut --exact --nocapture
// #[test]
// fn test_distribute_owner_cut() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = 1;
//         let coldkey1 = U256::from(1);
//         let hotkey1 = U256::from(2);
//         let coldkey2 = U256::from(3);
//         let hotkey2 = U256::from(4);

//         // Set up initial balances and stakes
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 1_000_000_000);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 1_000_000_000);
        
//         // Register hotkeys
//         assert_ok!(SubtensorModule::register(RuntimeOrigin::signed(coldkey1), netuid, hotkey1));
//         assert_ok!(SubtensorModule::register(RuntimeOrigin::signed(coldkey2), netuid, hotkey2));

//         // Lock stakes
//         assert_ok!(SubtensorModule::increase_stake(RuntimeOrigin::signed(coldkey1), hotkey1, netuid, 500_000_000));
//         assert_ok!(SubtensorModule::increase_stake(RuntimeOrigin::signed(coldkey2), hotkey2, netuid, 250_000_000));

//         // Lock the stakes
//         assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey1), hotkey1, netuid, 500_000_000, 7200 * 30)); // 30 days
//         assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey2), hotkey2, netuid, 250_000_000, 7200 * 15)); // 15 days

//         // Distribute owner cut
//         let amount_to_distribute = 1_000_000;
//         let remaining = SubtensorModule::distribute_owner_cut(netuid, amount_to_distribute);

//         // Check that all funds were distributed
//         assert_eq!(remaining, 0);

//         // Check that the locks were updated correctly
//         let (lock1, _, _) = Locks::<Test>::get((netuid, hotkey1, coldkey1)).unwrap();
//         let (lock2, _, _) = Locks::<Test>::get((netuid, hotkey2, coldkey2)).unwrap();

//         // The exact distribution might vary, but hotkey1 should receive more than hotkey2
//         assert!(lock1 > 500_000_000);
//         assert!(lock2 > 250_000_000);
//         assert!(lock1 - 500_000_000 > lock2 - 250_000_000);

//         // Check that the total distributed amount matches the input amount
//         assert_eq!(lock1 - 500_000_000 + lock2 - 250_000_000, amount_to_distribute);
//     });
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_distribute_owner_cut_no_conviction --exact --nocapture
// #[test]
// fn test_distribute_owner_cut_no_conviction() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = 1;
//         let amount_to_distribute = 1_000_000;

//         // Distribute owner cut when there are no stakes
//         let remaining = SubtensorModule::distribute_owner_cut(netuid, amount_to_distribute);

//         // Check that all funds were returned as there were no stakes to distribute to
//         assert_eq!(remaining, amount_to_distribute);
//     });
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_calculate_max_allowed_unstakable --exact --nocapture
// #[test]
// fn test_calculate_max_allowed_unstakable() {
//     new_test_ext(1).execute_with(|| {
//         let alpha_locked = 1_000_000;
//         let start_block = 1000;
//         let lock_interval_blocks = 7200 * 365; // One year in blocks

//         // Test immediately after locking
//         let current_block = start_block;
//         let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
//         assert_eq!(max_unstakable, 0, "Should not be able to unstake immediately after locking");

//         // Test after 25% of the lock period
//         let current_block = start_block + (lock_interval_blocks / 4);
//         let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
//         assert!(max_unstakable > 0 && max_unstakable < alpha_locked / 2, "Should be able to unstake some, but less than half after 25% of lock period");

//         // Test after 50% of the lock period
//         let current_block = start_block + (lock_interval_blocks / 2);
//         let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
//         assert!(max_unstakable > alpha_locked / 2 && max_unstakable < alpha_locked, "Should be able to unstake more than half, but not all after 50% of lock period");

//         // Test after full lock period
//         let current_block = start_block + lock_interval_blocks;
//         let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
//         assert_eq!(max_unstakable, alpha_locked, "Should be able to unstake all after full lock period");

//         // Test long after lock period
//         let current_block = start_block + (lock_interval_blocks * 2);
//         let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
//         assert_eq!(max_unstakable, alpha_locked, "Should still be able to unstake all long after lock period");
//     });
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_update_subnet_owner --exact --nocapture
// #[test]
// fn test_update_subnet_owner() {
//     new_test_ext(1).execute_with(|| {
//         let netuid = 1;
//         let coldkey1 = U256::from(1);
//         let hotkey1 = U256::from(2);
//         let coldkey2 = U256::from(3);
//         let hotkey2 = U256::from(4);

//         // Add balance and register hotkeys
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 1_000_000_000);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 1_000_000_000);
//         assert_ok!(SubtensorModule::register(&hotkey1, &coldkey1, netuid));
//         assert_ok!(SubtensorModule::register(&hotkey2, &coldkey2, netuid));

//         // Create locks for both hotkeys
//         let current_block = SubtensorModule::get_current_block_as_u64();
//         let lock_amount1 = 500_000_000;
//         let lock_amount2 = 750_000_000;
//         let duration = 7200 * 365; // One year in blocks

//         Locks::<Test>::insert(
//             (netuid, hotkey1, coldkey1),
//             (lock_amount1, current_block, current_block + duration)
//         );
//         Locks::<Test>::insert(
//             (netuid, hotkey2, coldkey2),
//             (lock_amount2, current_block, current_block + duration)
//         );

//         // Update subnet owner
//         SubtensorModule::update_subnet_owner(netuid);

//         // Check that the subnet owner is set to coldkey2 (which has a higher lock amount)
//         assert_eq!(SubnetOwner::<Test>::get(netuid), Some(coldkey2));

//         // Advance blocks and update locks to change conviction scores
//         run_to_block(current_block + duration / 2);
        
//         // Update lock for hotkey1 to have higher conviction
//         Locks::<Test>::insert(
//             (netuid, hotkey1, coldkey1),
//             (lock_amount1 * 2, current_block, current_block + duration * 2)
//         );

//         // Update subnet owner again
//         SubtensorModule::update_subnet_owner(netuid);

//         // Check that the subnet owner is now set to coldkey1
//         assert_eq!(SubnetOwner::<Test>::get(netuid), Some(coldkey1));
//     });
// }

// // SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_calculate_conviction --exact --nocapture
// #[test]
// fn test_calculate_conviction() {
//     new_test_ext(1).execute_with(|| {
//         let lock_amount = 1_000_000;
//         let current_block = 1000;
//         let end_block = current_block + 7200 * 365; // One year from now

//         // Test conviction at the start of the lock
//         let conviction_start = SubtensorModule::calculate_conviction(lock_amount, end_block, current_block);
//         assert!(conviction_start > 0, "Conviction should be positive at the start of the lock");
//         assert!(conviction_start < lock_amount, "Conviction should be less than the lock amount");

//         // Test conviction at the middle of the lock period
//         let mid_block = current_block + (7200 * 365 / 2);
//         let conviction_mid = SubtensorModule::calculate_conviction(lock_amount, end_block, mid_block);
//         assert!(conviction_mid > conviction_start, "Conviction should increase over time");
//         assert!(conviction_mid < lock_amount, "Conviction should still be less than the lock amount");

//         // Test conviction near the end of the lock period
//         let near_end_block = end_block - 1000;
//         let conviction_near_end = SubtensorModule::calculate_conviction(lock_amount, end_block, near_end_block);
//         assert!(conviction_near_end > conviction_mid, "Conviction should be higher near the end");
//         assert!(conviction_near_end < lock_amount, "Conviction should still be less than the lock amount");

//         // Test conviction with different lock amounts
//         let larger_lock = lock_amount * 2;
//         let conviction_larger = SubtensorModule::calculate_conviction(larger_lock, end_block, current_block);
//         assert!(conviction_larger > conviction_start, "Larger lock should have higher conviction");

//         // Test conviction with very short lock duration
//         let short_end_block = current_block + 100;
//         let conviction_short = SubtensorModule::calculate_conviction(lock_amount, short_end_block, current_block);
//         assert!(conviction_short < conviction_start, "Short lock should have lower conviction");

//         // Test conviction at the exact end of the lock
//         let conviction_end = SubtensorModule::calculate_conviction(lock_amount, end_block, end_block);
//         assert_eq!(conviction_end, lock_amount, "Conviction should equal lock amount at the end");
//     });
// }

