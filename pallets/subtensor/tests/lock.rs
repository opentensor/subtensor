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
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
        assert_ok!(SubtensorModule::register(RuntimeOrigin::signed(coldkey), netuid, hotkey));
        assert_ok!(SubtensorModule::increase_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, stake_amount));

        // Perform the lock
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, lock_amount, lock_duration));

        // Check that the lock was created correctly
        let (locked_amount, start_block, end_block) = Locks::<Test>::get((netuid, hotkey, coldkey)).unwrap();
        assert_eq!(locked_amount, lock_amount);
        assert_eq!(end_block, start_block + lock_duration);

        // Verify the event was emitted
        System::assert_last_event(Event::LockIncreased(coldkey, hotkey, netuid, lock_amount).into());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_do_lock_failures --exact --nocapture
#[test]
fn test_do_lock_failures() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let non_existent_netuid = 99;
        let non_existent_hotkey = U256::from(99);
        let stake_amount = 500_000_000;
        let lock_amount = 250_000_000;
        let lock_duration = 7200 * 30; // 30 days

        // Set up initial balance and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
        assert_ok!(SubtensorModule::register(RuntimeOrigin::signed(coldkey), netuid, hotkey));
        assert_ok!(SubtensorModule::increase_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, stake_amount));

        // Test: Subnet does not exist
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, non_existent_netuid, lock_amount, lock_duration),
            Error::<Test>::SubnetNotExists
        );

        // Test: Hotkey account does not exist
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), non_existent_hotkey, netuid, lock_amount, lock_duration),
            Error::<Test>::HotKeyAccountNotExists
        );

        // Test: Hotkey not registered in subnet
        let unregistered_hotkey = U256::from(3);
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), unregistered_hotkey, netuid, lock_amount, lock_duration),
            Error::<Test>::HotKeyNotRegisteredInSubNet
        );

        // Test: Lock amount is zero
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, 0, lock_duration),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Test: Not enough stake to lock
        assert_noop!(
            SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey), hotkey, netuid, stake_amount + 1, lock_duration),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}


// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_distribute_owner_cut --exact --nocapture
#[test]
fn test_distribute_owner_cut() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey1 = U256::from(1);
        let hotkey1 = U256::from(2);
        let coldkey2 = U256::from(3);
        let hotkey2 = U256::from(4);

        // Set up initial balances and stakes
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 1_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 1_000_000_000);
        
        // Register hotkeys
        assert_ok!(SubtensorModule::register(RuntimeOrigin::signed(coldkey1), netuid, hotkey1));
        assert_ok!(SubtensorModule::register(RuntimeOrigin::signed(coldkey2), netuid, hotkey2));

        // Lock stakes
        assert_ok!(SubtensorModule::increase_stake(RuntimeOrigin::signed(coldkey1), hotkey1, netuid, 500_000_000));
        assert_ok!(SubtensorModule::increase_stake(RuntimeOrigin::signed(coldkey2), hotkey2, netuid, 250_000_000));

        // Lock the stakes
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey1), hotkey1, netuid, 500_000_000, 7200 * 30)); // 30 days
        assert_ok!(SubtensorModule::lock_stake(RuntimeOrigin::signed(coldkey2), hotkey2, netuid, 250_000_000, 7200 * 15)); // 15 days

        // Distribute owner cut
        let amount_to_distribute = 1_000_000;
        let remaining = SubtensorModule::distribute_owner_cut(netuid, amount_to_distribute);

        // Check that all funds were distributed
        assert_eq!(remaining, 0);

        // Check that the locks were updated correctly
        let (lock1, _, _) = Locks::<Test>::get((netuid, hotkey1, coldkey1)).unwrap();
        let (lock2, _, _) = Locks::<Test>::get((netuid, hotkey2, coldkey2)).unwrap();

        // The exact distribution might vary, but hotkey1 should receive more than hotkey2
        assert!(lock1 > 500_000_000);
        assert!(lock2 > 250_000_000);
        assert!(lock1 - 500_000_000 > lock2 - 250_000_000);

        // Check that the total distributed amount matches the input amount
        assert_eq!(lock1 - 500_000_000 + lock2 - 250_000_000, amount_to_distribute);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_distribute_owner_cut_no_conviction --exact --nocapture
#[test]
fn test_distribute_owner_cut_no_conviction() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let amount_to_distribute = 1_000_000;

        // Distribute owner cut when there are no stakes
        let remaining = SubtensorModule::distribute_owner_cut(netuid, amount_to_distribute);

        // Check that all funds were returned as there were no stakes to distribute to
        assert_eq!(remaining, amount_to_distribute);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_calculate_max_allowed_unstakable --exact --nocapture
#[test]
fn test_calculate_max_allowed_unstakable() {
    new_test_ext(1).execute_with(|| {
        let alpha_locked = 1_000_000;
        let start_block = 1000;
        let lock_interval_blocks = 7200 * 365; // One year in blocks

        // Test immediately after locking
        let current_block = start_block;
        let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
        assert_eq!(max_unstakable, 0, "Should not be able to unstake immediately after locking");

        // Test after 25% of the lock period
        let current_block = start_block + (lock_interval_blocks / 4);
        let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
        assert!(max_unstakable > 0 && max_unstakable < alpha_locked / 2, "Should be able to unstake some, but less than half after 25% of lock period");

        // Test after 50% of the lock period
        let current_block = start_block + (lock_interval_blocks / 2);
        let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
        assert!(max_unstakable > alpha_locked / 2 && max_unstakable < alpha_locked, "Should be able to unstake more than half, but not all after 50% of lock period");

        // Test after full lock period
        let current_block = start_block + lock_interval_blocks;
        let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
        assert_eq!(max_unstakable, alpha_locked, "Should be able to unstake all after full lock period");

        // Test long after lock period
        let current_block = start_block + (lock_interval_blocks * 2);
        let max_unstakable = SubtensorModule::calculate_max_allowed_unstakable(alpha_locked, start_block, current_block);
        assert_eq!(max_unstakable, alpha_locked, "Should still be able to unstake all long after lock period");
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_update_subnet_owner --exact --nocapture
#[test]
fn test_update_subnet_owner() {
    new_test_ext(1).execute_with(|| {
        let netuid = 1;
        let coldkey1 = U256::from(1);
        let hotkey1 = U256::from(2);
        let coldkey2 = U256::from(3);
        let hotkey2 = U256::from(4);

        // Add balance and register hotkeys
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 1_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 1_000_000_000);
        assert_ok!(SubtensorModule::register(&hotkey1, &coldkey1, netuid));
        assert_ok!(SubtensorModule::register(&hotkey2, &coldkey2, netuid));

        // Create locks for both hotkeys
        let current_block = SubtensorModule::get_current_block_as_u64();
        let lock_amount1 = 500_000_000;
        let lock_amount2 = 750_000_000;
        let duration = 7200 * 365; // One year in blocks

        Locks::<Test>::insert(
            (netuid, hotkey1, coldkey1),
            (lock_amount1, current_block, current_block + duration)
        );
        Locks::<Test>::insert(
            (netuid, hotkey2, coldkey2),
            (lock_amount2, current_block, current_block + duration)
        );

        // Update subnet owner
        SubtensorModule::update_subnet_owner(netuid);

        // Check that the subnet owner is set to coldkey2 (which has a higher lock amount)
        assert_eq!(SubnetOwner::<Test>::get(netuid), Some(coldkey2));

        // Advance blocks and update locks to change conviction scores
        run_to_block(current_block + duration / 2);
        
        // Update lock for hotkey1 to have higher conviction
        Locks::<Test>::insert(
            (netuid, hotkey1, coldkey1),
            (lock_amount1 * 2, current_block, current_block + duration * 2)
        );

        // Update subnet owner again
        SubtensorModule::update_subnet_owner(netuid);

        // Check that the subnet owner is now set to coldkey1
        assert_eq!(SubnetOwner::<Test>::get(netuid), Some(coldkey1));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test lock -- test_calculate_conviction --exact --nocapture
#[test]
fn test_calculate_conviction() {
    new_test_ext(1).execute_with(|| {
        let lock_amount = 1_000_000;
        let current_block = 1000;
        let end_block = current_block + 7200 * 365; // One year from now

        // Test conviction at the start of the lock
        let conviction_start = SubtensorModule::calculate_conviction(lock_amount, end_block, current_block);
        assert!(conviction_start > 0, "Conviction should be positive at the start of the lock");
        assert!(conviction_start < lock_amount, "Conviction should be less than the lock amount");

        // Test conviction at the middle of the lock period
        let mid_block = current_block + (7200 * 365 / 2);
        let conviction_mid = SubtensorModule::calculate_conviction(lock_amount, end_block, mid_block);
        assert!(conviction_mid > conviction_start, "Conviction should increase over time");
        assert!(conviction_mid < lock_amount, "Conviction should still be less than the lock amount");

        // Test conviction near the end of the lock period
        let near_end_block = end_block - 1000;
        let conviction_near_end = SubtensorModule::calculate_conviction(lock_amount, end_block, near_end_block);
        assert!(conviction_near_end > conviction_mid, "Conviction should be higher near the end");
        assert!(conviction_near_end < lock_amount, "Conviction should still be less than the lock amount");

        // Test conviction with different lock amounts
        let larger_lock = lock_amount * 2;
        let conviction_larger = SubtensorModule::calculate_conviction(larger_lock, end_block, current_block);
        assert!(conviction_larger > conviction_start, "Larger lock should have higher conviction");

        // Test conviction with very short lock duration
        let short_end_block = current_block + 100;
        let conviction_short = SubtensorModule::calculate_conviction(lock_amount, short_end_block, current_block);
        assert!(conviction_short < conviction_start, "Short lock should have lower conviction");

        // Test conviction at the exact end of the lock
        let conviction_end = SubtensorModule::calculate_conviction(lock_amount, end_block, end_block);
        assert_eq!(conviction_end, lock_amount, "Conviction should equal lock amount at the end");
    });
}

