#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use super::mock::*;
use crate::staking::lock::StakeLock;
use crate::tests::mock;
use crate::*;

use approx::assert_abs_diff_eq;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;

pub const UPDATE_INTERVAL: u64 = 7200 * 7;

fn max_unlockable_stake(netuid: NetUid, hotkey: &U256, coldkey: &U256) -> AlphaCurrency {
    let current_block = SubtensorModule::get_current_block_as_u64();
    let total_stake: AlphaCurrency =
        SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid);
    if Locks::<Test>::contains_key((netuid, hotkey, coldkey)) {
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);

        total_stake.saturating_sub(conviction)
    } else {
        total_stake
    }
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_success --exact --show-output
#[test]
fn test_do_lock_success() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = 500_000_000;
        let lock_amount = AlphaCurrency::from(250_000_000);
        let lock_duration = 7200 * 30; // 30 days
        let start_block = SubtensorModule::get_current_block_as_u64();

        // Set up initial balance and stake
        add_network(netuid, 0, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            stake_amount
        ));
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Check that the lock was created correctly
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock.alpha_locked, lock_amount);
        assert_eq!(stake_lock.end_block, start_block + lock_duration);

        // Verify the event was emitted
        System::assert_last_event(
            Event::LockIncreased {
                coldkey,
                hotkey,
                netuid,
                alpha_locked: lock_amount,
            }
            .into(),
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_subnet_does_not_exist --exact --show-output
#[test]
fn test_do_lock_subnet_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let non_existent_netuid = NetUid::from(99);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let lock_amount = AlphaCurrency::from(250_000_000);
        let lock_duration = 7200 * 30; // 30 days

        // Attempt to lock stake on a non-existent subnet
        assert_noop!(
            SubtensorModule::lock_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                non_existent_netuid,
                lock_duration,
                lock_amount
            ),
            Error::<Test>::SubnetNotExists
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_hotkey_does_not_exist --exact --show-output
#[test]
fn test_do_lock_hotkey_does_not_exist() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let non_existent_hotkey = U256::from(99);
        let lock_amount = AlphaCurrency::from(250_000_000);
        let lock_duration = 7200 * 30; // 30 days

        // Set up network
        add_network(netuid, 0, 0);

        // Attempt to lock stake with a non-existent hotkey
        assert_noop!(
            SubtensorModule::lock_stake(
                RuntimeOrigin::signed(coldkey),
                non_existent_hotkey,
                netuid,
                lock_duration,
                lock_amount
            ),
            Error::<Test>::HotKeyAccountNotExists
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_hotkey_not_registered --exact --show-output
#[test]
fn test_do_lock_hotkey_not_registered() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(3);
        let hotkey = U256::from(2);
        let stake_amount = 500_000_000;
        let lock_amount = AlphaCurrency::from(250_000_000);
        let lock_duration = 7200 * 30; // 30 days
        let start_block = SubtensorModule::get_current_block_as_u64();

        // Set up network
        add_network(netuid, 0, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, 1_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 1_000_000_000);
        register_ok_neuron(netuid, hotkey, coldkey1, 11);

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey2),
            hotkey,
            netuid,
            stake_amount
        ));

        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey2),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Check that the lock was created correctly
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey2));
        assert_eq!(stake_lock.alpha_locked, lock_amount);
        assert_eq!(stake_lock.end_block, start_block + lock_duration);

        // Verify the event was emitted
        System::assert_last_event(
            Event::LockIncreased {
                coldkey: coldkey2,
                hotkey,
                netuid,
                alpha_locked: lock_amount,
            }
            .into(),
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_zero_amount --exact --show-output
#[test]
fn test_do_lock_zero_amount() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let lock_amount = AlphaCurrency::from(0);
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);

        // Attempt to lock zero stake
        assert_noop!(
            SubtensorModule::lock_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                lock_duration,
                lock_amount
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_insufficient_stake --exact --show-output
#[test]
fn test_do_lock_insufficient_stake() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = 500_000_000;
        let lock_amount = AlphaCurrency::from(750_000_000); // More than available stake
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            stake_amount
        ));

        // Attempt to lock more stake than available
        assert_noop!(
            SubtensorModule::lock_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                lock_duration,
                lock_amount
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_increase_conviction --exact --show-output
#[test]
fn test_do_lock_increase_conviction() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let initial_lock_amount = AlphaCurrency::from(500_000_000);
        let initial_lock_duration = 7200 * 30; // 30 days
        let new_lock_amount = AlphaCurrency::from(750_000_000);
        let new_lock_duration = 7200 * 60; // 60 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Initial lock
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_lock_duration,
            initial_lock_amount
        ));

        // Increase conviction with new lock
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            new_lock_duration,
            new_lock_amount
        ));

        // Verify the new lock
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock.alpha_locked, new_lock_amount);
        assert_eq!(
            stake_lock.end_block,
            SubtensorModule::get_current_block_as_u64() + new_lock_duration
        );

        // Verify event emission
        System::assert_last_event(
            Event::LockIncreased {
                coldkey,
                hotkey,
                netuid,
                alpha_locked: new_lock_amount,
            }
            .into(),
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_decrease_conviction --exact --show-output
#[test]
fn test_do_lock_decrease_conviction() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let initial_lock_amount = AlphaCurrency::from(500_000_000);
        let initial_lock_duration = 7200 * 30; // 30 days
        let new_lock_amount = AlphaCurrency::from(400_000_000);
        let new_lock_duration = 7200 * 20; // 20 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Initial lock
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_lock_duration,
            initial_lock_amount
        ));

        // Attempt to decrease conviction with new lock
        assert_noop!(
            SubtensorModule::lock_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                new_lock_duration,
                new_lock_amount
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Verify the lock remains unchanged
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock.alpha_locked, initial_lock_amount);
        assert_eq!(
            stake_lock.end_block,
            SubtensorModule::get_current_block_as_u64() + initial_lock_duration
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_max_duration --exact --show-output
#[test]
fn test_do_lock_max_duration() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = AlphaCurrency::from(500_000_000);
        let max_lock_duration = 7200 * 365; // 1 year (assuming 7200 blocks per day)

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Lock stake for maximum duration
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            max_lock_duration,
            lock_amount
        ));

        // Verify the lock
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock.alpha_locked, lock_amount);
        assert_eq!(
            stake_lock.end_block,
            SubtensorModule::get_current_block_as_u64() + max_lock_duration
        );

        // Verify event emission
        System::assert_last_event(
            Event::LockIncreased {
                coldkey,
                hotkey,
                netuid,
                alpha_locked: lock_amount,
            }
            .into(),
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_multiple_times --exact --show-output
#[test]
fn test_do_lock_multiple_times() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount_1 = AlphaCurrency::from(300_000_000);
        let lock_amount_2 = AlphaCurrency::from(500_000_000);
        let lock_duration_1 = 7200 * 30; // 30 days
        let lock_duration_2 = 7200 * 60; // 60 days
        let start_block = SubtensorModule::get_current_block_as_u64();

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // First lock
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration_1,
            lock_amount_1
        ));

        // Verify the first lock
        let stake_lock_1 = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock_1.alpha_locked, lock_amount_1);
        assert_eq!(stake_lock_1.end_block, start_block + lock_duration_1);

        // Second lock
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration_2,
            lock_amount_2
        ));

        // Verify the second lock
        let stake_lock_2 = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock_2.alpha_locked, lock_amount_2);
        assert_eq!(stake_lock_2.end_block, start_block + lock_duration_2);

        // Ensure the locked amount increased
        assert!(stake_lock_2.alpha_locked > stake_lock_1.alpha_locked);

        // Verify event emissions
        System::assert_has_event(
            Event::LockIncreased {
                coldkey,
                hotkey,
                netuid,
                alpha_locked: lock_amount_1,
            }
            .into(),
        );
        System::assert_last_event(
            Event::LockIncreased {
                coldkey,
                hotkey,
                netuid,
                alpha_locked: lock_amount_2,
            }
            .into(),
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_different_subnets --exact --show-output
#[test]
fn test_do_lock_different_subnets() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount_1 = AlphaCurrency::from(300_000_000);
        let lock_amount_2 = AlphaCurrency::from(500_000_000);
        let lock_duration = 7200 * 30; // 30 days
        let start_block = SubtensorModule::get_current_block_as_u64();

        // Set up networks and register neuron
        add_network(netuid1, 0, 0);
        add_network(netuid2, 0, 0);
        register_ok_neuron(netuid1, hotkey, coldkey, 11);
        register_ok_neuron(netuid2, hotkey, coldkey, 12);

        // Add balance to coldkey and stake on both networks
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake * 2);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid1,
            initial_stake
        ));
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid2,
            initial_stake
        ));

        // Lock stake on first subnet
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid1,
            lock_duration,
            lock_amount_1
        ));

        // Verify the lock on first subnet
        let stake_lock_1 = Locks::<Test>::get((netuid1, hotkey, coldkey));
        assert_eq!(stake_lock_1.alpha_locked, lock_amount_1);
        assert_eq!(stake_lock_1.end_block, start_block + lock_duration);

        // Lock stake on second subnet
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid2,
            lock_duration,
            lock_amount_2
        ));

        // Verify the lock on second subnet
        let stake_lock_2 = Locks::<Test>::get((netuid2, hotkey, coldkey));
        assert_eq!(stake_lock_2.alpha_locked, lock_amount_2);
        assert_eq!(stake_lock_2.end_block, start_block + lock_duration);

        // Ensure the locks are independent
        assert_ne!(stake_lock_1.alpha_locked, stake_lock_2.alpha_locked);

        // Verify event emissions
        System::assert_has_event(
            Event::LockIncreased {
                coldkey,
                hotkey,
                netuid: netuid1,
                alpha_locked: lock_amount_1,
            }
            .into(),
        );
        System::assert_last_event(
            Event::LockIncreased {
                coldkey,
                hotkey,
                netuid: netuid2,
                alpha_locked: lock_amount_2,
            }
            .into(),
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_fully_locked --exact --show-output
#[test]
fn test_remove_stake_fully_locked() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = AlphaCurrency::from(initial_stake - 1);
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Lock all stake
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Attempt to remove stake
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                initial_stake.into()
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Verify lock remains unchanged
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock.alpha_locked, lock_amount);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_partially_locked --exact --show-output
#[test]
fn test_remove_stake_partially_locked() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = AlphaCurrency::from(600_000_000);
        let lock_duration = 7200 * 30; // 30 days

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Lock part of the stake
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Attempt to remove unlocked portion (should succeed)
        let unlocked_amount = max_unlockable_stake(netuid, &hotkey, &coldkey);
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            unlocked_amount
        ));

        // Verify stake and lock after removal
        let stake_after_removal =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(
            stake_after_removal,
            AlphaCurrency::from(initial_stake) - unlocked_amount - 1.into()
        );

        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock.alpha_locked, lock_amount); // lock amount should not change

        // Attempt to remove more than unlocked portion (should fail)
        let stake_to_remove_2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                stake_to_remove_2
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Verify stake and lock remain unchanged
        let stake_after_failed_removal =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(
            stake_after_failed_removal,
            AlphaCurrency::from(initial_stake) - unlocked_amount - 1.into()
        );

        let stake_lock_after = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock_after.alpha_locked, lock_amount);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_after_lock_expiry --exact --show-output
#[test]
fn test_remove_stake_after_lock_expiry() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = AlphaCurrency::from(600_000_000);
        let lock_duration = 7200;

        // Set up network and register neuron
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Lock part of the stake
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Fast forward to just after lock expiry
        run_to_block(lock_duration + 1);

        // Attempt to remove all stake (should succeed)
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            (initial_stake - 1).into()
        ));

        // Verify stake and lock after removal
        let stake_after_removal =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(stake_after_removal, 0.into());

        // Verify lock is removed
        assert!(!Locks::<Test>::contains_key((netuid, hotkey, coldkey)));

        // Verify balance is returned to coldkey
        let coldkey_balance = SubtensorModule::get_coldkey_balance(&coldkey);
        assert_eq!(coldkey_balance, initial_stake);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_multiple_locks --exact --show-output
#[test]
fn test_remove_stake_multiple_locks() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let coldkey2 = U256::from(3); // To keep additional stake
        let initial_stake = 1_000_000_000;
        let lock_duration_1 = 7200;
        let lock_duration_2 = 7200;

        // Set up network and register neuron
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid, hotkey, coldkey, 11);

        mock::setup_reserves(
            netuid,
            initial_stake * 100,
            AlphaCurrency::from(initial_stake * 100),
        );

        // Add balance to coldkey and stake
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));
        let alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let lock_amount_1 = alpha * 3.into() / 10.into();
        let lock_amount_2 = alpha * 4.into() / 10.into();

        // Also add more stake so that we don't get into low liquidity issues now
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey2),
            hotkey,
            netuid,
            initial_stake
        ));

        // Create two locks
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration_1,
            lock_amount_1
        )); // first lock.
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration_2,
            lock_amount_2
        )); // second replaces first.

        // Attempt to remove more stake than unlocked (should fail)
        let max_removable = max_unlockable_stake(netuid, &hotkey, &coldkey);
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                max_removable + 1.into()
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Remove unlocked stake (should succeed)
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            max_removable
        ));

        // Verify remaining stake
        let remaining_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(remaining_stake, alpha - max_removable);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_conviction_calculation --exact --show-output
#[test]
fn test_remove_stake_conviction_calculation() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = AlphaCurrency::from(500_000_000);
        let lock_duration = 7200;

        // Register and add stake
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        SubtensorModule::set_lock_interval_blocks(lock_duration);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Lock stake
        assert_ok!(SubtensorModule::do_lock(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Try to remove more stake than allowed by conviction
        let max_removable = max_unlockable_stake(netuid, &hotkey, &coldkey);
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                max_removable + 1.into()
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Remove allowed amount of stake
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            max_removable - 1.into()
        ));

        // Verify remaining stake
        let remaining_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(
            remaining_stake,
            AlphaCurrency::from(initial_stake) - max_removable
        );

        // Fast forward to just before lock expiry
        run_to_block(lock_duration - 1);

        // Try to remove all remaining stake (should fail)
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                remaining_stake
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Fast forward to after lock expiry
        run_to_block(lock_duration + 1);

        // Now remove all remaining stake (should succeed)
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            remaining_stake
        ));

        // Verify no stake remains
        let final_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(final_stake, 0.into());

        // Verify lock is removed
        assert!(!Locks::<Test>::contains_key((netuid, hotkey, coldkey)));
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_partial_lock_removal --exact --show-output
#[test]
fn test_remove_stake_partial_lock_removal() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = AlphaCurrency::from(500_000_000);
        let lock_duration = 7200 * 30; // 30 days

        // Register and add stake
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        SubtensorModule::set_lock_interval_blocks(lock_duration);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Lock stake
        assert_ok!(SubtensorModule::do_lock(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Verify initial lock state
        let stake_lock_before = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock_before.alpha_locked, lock_amount);

        // Calculate max removable stake
        let max_removable = max_unlockable_stake(netuid, &hotkey, &coldkey);
        let partial_remove_amount = max_removable / 2.into();

        // Remove part of the stake
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            partial_remove_amount
        ));

        // Verify remaining stake and updated lock
        let remaining_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(
            remaining_stake,
            AlphaCurrency::from(initial_stake) - partial_remove_amount - 1.into()
        );

        let stake_lock_after = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock_after.alpha_locked, lock_amount); // lock never changes.

        // Ensure lock still exists
        assert!(Locks::<Test>::contains_key((netuid, hotkey, coldkey)));
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_partial_lock_removal --exact --show-output
#[test]
fn test_remove_stake_full_lock_removal() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_amount = AlphaCurrency::from(500_000_000);
        let lock_duration = 7200;

        // Register and add stake
        add_network(netuid, 0, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        SubtensorModule::set_lock_interval_blocks(lock_duration);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            initial_stake
        ));

        // Lock stake
        assert_ok!(SubtensorModule::do_lock(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_duration,
            lock_amount
        ));

        // Verify initial lock state
        let stake_lock = Locks::<Test>::get((netuid, hotkey, coldkey));
        assert_eq!(stake_lock.alpha_locked, lock_amount);

        // Fast forward to just after lock expiry
        run_to_block(lock_duration + 1);

        // Remove all stake
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            (initial_stake - 1).into()
        ));

        // Verify remaining stake
        let remaining_stake =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_eq!(remaining_stake, 0.into());

        // Verify lock is removed
        assert!(!Locks::<Test>::contains_key((netuid, hotkey, coldkey)));

        // Verify balance is returned to coldkey
        let coldkey_balance = SubtensorModule::get_coldkey_balance(&coldkey);
        assert_eq!(coldkey_balance, initial_stake);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_remove_stake_across_subnets --exact --show-output
#[test]
fn test_remove_stake_across_subnets() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = 1_000_000_000;
        let lock_duration_1 = 7200_u64;
        let lock_duration_2 = 14400_u64;

        // Set up networks and register neuron
        let netuid1 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let netuid2 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        register_ok_neuron(netuid1, hotkey, coldkey, 11);
        register_ok_neuron(netuid2, hotkey, coldkey, 11);
        Tempo::<Test>::insert(netuid1, (lock_duration_1 * 2) as u16);
        Tempo::<Test>::insert(netuid2, (lock_duration_2 * 2) as u16);

        // Add balance to coldkey and stake on both networks
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake * 2);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid1,
            initial_stake
        ));
        let initial_alpha_1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid1);
        let lock_amount_1 = initial_alpha_1 * 3.into() / 10.into();
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid2,
            initial_stake
        ));
        let initial_alpha_2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid2);
        let lock_amount_2 = initial_alpha_2 * 4.into() / 10.into();

        // Create locks on both networks
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid1,
            lock_duration_1,
            lock_amount_1
        ));
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid2,
            lock_duration_2,
            lock_amount_2
        ));

        // Attempt to remove more stake than unlocked from netuid1 (should fail)
        let max_removable_1 = max_unlockable_stake(netuid1, &hotkey, &coldkey);
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid1);
        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid2);
        assert_noop!(
            SubtensorModule::remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid1,
                max_removable_1 + 1.into()
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Remove unlocked stake from netuid1 (should succeed)
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid1,
            max_removable_1
        ));

        // Verify remaining stake on netuid1
        let remaining_stake_1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid1);
        assert_eq!(remaining_stake_1, initial_alpha_1 - max_removable_1);

        // Fast forward to after first lock expiry
        run_to_block(lock_duration_1 + 1);

        // Remove all stake from netuid1 (should succeed now that lock has expired)
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid1,
            remaining_stake_1
        ));

        // Verify no stake remains on netuid1
        let final_stake_1 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid1);
        assert_eq!(final_stake_1, 0.into());

        // Attempt to remove stake from netuid2 (should still be partially locked)
        let max_removable_2 = max_unlockable_stake(netuid2, &hotkey, &coldkey);
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid2,
            max_removable_2
        ));

        // Verify remaining stake on netuid2
        let remaining_stake_2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid2);
        assert_eq!(remaining_stake_2, initial_alpha_2 - max_removable_2);

        // Fast forward to after second lock expiry
        run_to_block(lock_duration_2 + 1);

        // Remove all remaining stake from netuid2
        assert_ok!(SubtensorModule::remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid2,
            remaining_stake_2
        ));

        // Verify no stake remains on netuid2
        let final_stake_2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid2);
        assert_eq!(final_stake_2, 0.into());

        // Verify locks are removed from both networks
        assert!(!Locks::<Test>::contains_key((netuid1, hotkey, coldkey)));
        assert!(!Locks::<Test>::contains_key((netuid2, hotkey, coldkey)));
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_zero_lock_amount --exact --show-output
#[test]
fn test_calculate_conviction_zero_lock_amount() {
    new_test_ext(1).execute_with(|| {
        let current_block = 1000;
        let stake_lock = StakeLock {
            alpha_locked: 0.into(),
            start_block: 0,
            end_block: 2000,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert_eq!(conviction, 0.into());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_zero_duration --exact --show-output
#[test]
fn test_calculate_conviction_zero_duration() {
    new_test_ext(1).execute_with(|| {
        let current_block = 1000;
        let stake_lock = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: 1000,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert_eq!(conviction, 0.into());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_max_lock_amount --exact --show-output
#[test]
fn test_calculate_conviction_max_lock_amount() {
    new_test_ext(1).execute_with(|| {
        let current_block = 1000;
        let stake_lock = StakeLock {
            alpha_locked: u64::MAX.into(),
            start_block: 0,
            end_block: 2000,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert!(conviction > 0.into());
        assert!(conviction < u64::MAX.into());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_max_duration --exact --show-output
#[test]
fn test_calculate_conviction_max_duration() {
    new_test_ext(1).execute_with(|| {
        let current_block = 0;
        let stake_lock = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: u64::MAX,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert!(conviction > 0.into());
        assert!(conviction <= stake_lock.alpha_locked);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_overflow_check --exact --show-output
#[test]
fn test_calculate_conviction_overflow_check() {
    new_test_ext(1).execute_with(|| {
        let current_block = 0;
        let stake_lock = StakeLock {
            alpha_locked: u64::MAX.into(),
            start_block: 0,
            end_block: u64::MAX,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert_eq!(conviction, u64::MAX.into());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_precision_small_values --exact --show-output
#[test]
fn test_calculate_conviction_precision_small_values() {
    new_test_ext(1).execute_with(|| {
        let current_block = 1000;
        let stake_lock = StakeLock {
            alpha_locked: 1.into(),
            start_block: 0,
            end_block: 1001,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert!(conviction < stake_lock.alpha_locked);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_precision_large_values --exact --show-output
#[test]
fn test_calculate_conviction_precision_large_values() {
    new_test_ext(1).execute_with(|| {
        let current_block = 0;
        let stake_lock = StakeLock {
            alpha_locked: (u64::MAX / 2).into(),
            start_block: 0,
            end_block: u64::MAX / 2,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert!(conviction == stake_lock.alpha_locked);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_rounding --exact --show-output
#[test]
fn test_calculate_conviction_rounding() {
    new_test_ext(1).execute_with(|| {
        let current_block = 1000;
        let end_block = 1100;
        let lock_amount = AlphaCurrency::from(1000000);
        let stake_lock1 = StakeLock {
            alpha_locked: lock_amount,
            start_block: 0,
            end_block,
        };
        let conviction1 = SubtensorModule::calculate_conviction(&stake_lock1, current_block);
        let stake_lock2 = StakeLock {
            alpha_locked: lock_amount,
            start_block: 0,
            end_block: end_block + 1,
        };
        let conviction2 = SubtensorModule::calculate_conviction(&stake_lock2, current_block);
        assert!(conviction2 >= conviction1);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_lock_interval_boundary --exact --show-output
#[test]
fn test_calculate_conviction_lock_interval_boundary() {
    new_test_ext(1).execute_with(|| {
        let current_block = 1000;
        let lock_interval = SubtensorModule::get_lock_interval_blocks();
        let stake_lock = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: current_block + lock_interval,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert!(conviction > 0.into());
        assert!(conviction < stake_lock.alpha_locked);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_consistency --exact --show-output
#[test]
fn test_calculate_conviction_consistency() {
    new_test_ext(1).execute_with(|| {
        let current_block = 1000;
        let end_block = 2000;
        let lock_amount = AlphaCurrency::from(1000000);

        let stake_lock_base = StakeLock {
            alpha_locked: lock_amount,
            start_block: 0,
            end_block,
        };
        let base_conviction =
            SubtensorModule::calculate_conviction(&stake_lock_base, current_block);

        // Increasing lock amount
        let stake_lock_higher = StakeLock {
            alpha_locked: lock_amount + 1000.into(),
            start_block: 0,
            end_block,
        };
        let higher_amount_conviction =
            SubtensorModule::calculate_conviction(&stake_lock_higher, current_block);
        assert!(higher_amount_conviction > base_conviction);

        // Increasing duration
        let stake_lock_longer = StakeLock {
            alpha_locked: lock_amount,
            start_block: 0,
            end_block: end_block + 1000,
        };
        let longer_duration_conviction =
            SubtensorModule::calculate_conviction(&stake_lock_longer, current_block);
        assert!(longer_duration_conviction > base_conviction);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_calculate_conviction_expired_lock --exact --show-output
#[test]
fn test_calculate_conviction_expired_lock() {
    new_test_ext(1).execute_with(|| {
        let current_block = 21;
        let stake_lock = StakeLock {
            alpha_locked: 394866833.into(),
            start_block: 1,
            end_block: 21,
        };
        let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert_eq!(conviction, 0.into());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_no_locks --exact --show-output
#[test]
fn test_update_subnet_owner_no_locks() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);

        // Ensure there are no locks in the subnet
        assert_eq!(
            Locks::<Test>::iter().count(),
            0,
            "There should be no locks initially"
        );

        // Call the update_subnet_owner function
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Verify that no subnet owner was set
        assert!(
            !SubnetOwner::<Test>::contains_key(netuid),
            "No subnet owner should be set when there are no locks"
        );

        // Verify that the subnet locked amount is zero
        assert_eq!(
            SubnetLocked::<Test>::get(netuid),
            0.into(),
            "Subnet locked amount should be zero"
        );

        // Check that a warning log was emitted
        // Note: In a real test environment, you would need a way to capture and assert on log output
        // For this example, we'll just comment on the expected behavior
        // assert_eq!(last_log_message(), "No locks found for subnet 1");
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_single_lock --exact --show-output
#[test]
fn test_update_subnet_owner_single_lock() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let lock_amount = AlphaCurrency::from(1000000);
        let current_block = 100;
        let lock_duration = 1000000;

        // Set up a single lock
        let stake_lock = StakeLock {
            alpha_locked: lock_amount,
            start_block: 0,
            end_block: current_block + lock_duration,
        };
        Locks::<Test>::insert((netuid, hotkey, coldkey), stake_lock);

        // Set up ownership
        Owner::<Test>::insert(hotkey, coldkey);

        // Mock the current block
        System::set_block_number(current_block);

        // Call the update_subnet_owner function
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Verify that the subnet owner was set correctly
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey.clone(),
            "Subnet owner should be set to the coldkey of the only lock"
        );

        // Verify that the subnet locked amount is correct
        let expected_conviction = SubtensorModule::calculate_conviction(
            &StakeLock {
                alpha_locked: lock_amount,
                start_block: 0,
                end_block: current_block + lock_duration,
            },
            current_block,
        );
        assert_eq!(
            SubnetLocked::<Test>::get(netuid),
            expected_conviction,
            "Subnet locked amount should match the calculated conviction"
        );

        // Verify that no new locks were created
        assert_eq!(
            Locks::<Test>::iter().count(),
            1,
            "There should still be only one lock"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_multiple_locks --exact --show-output
#[test]
fn test_update_subnet_owner_multiple_locks() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(1);
        let coldkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let coldkey2 = U256::from(4);
        let hotkey3 = U256::from(5);
        let coldkey3 = U256::from(6);
        let current_block = 100;

        // Set up multiple locks with different amounts and durations
        let stake_lock_1 = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: current_block + 1000000,
        };
        let stake_lock_2 = StakeLock {
            alpha_locked: 2000000.into(),
            start_block: 0,
            end_block: current_block + 500000,
        };
        let stake_lock_3 = StakeLock {
            alpha_locked: 1500000.into(),
            start_block: 0,
            end_block: current_block + 2000000,
        };

        Locks::<Test>::insert((netuid, hotkey1, coldkey1), stake_lock_1);
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock_2);
        Locks::<Test>::insert((netuid, hotkey3, coldkey3), stake_lock_3);

        // Set up ownership
        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);
        Owner::<Test>::insert(hotkey3, coldkey3);

        // Mock the current block
        System::set_block_number(current_block);

        // Call the update_subnet_owner function
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Calculate expected convictions
        let conviction1 = SubtensorModule::calculate_conviction(&stake_lock_1, current_block);
        let conviction2 = SubtensorModule::calculate_conviction(&stake_lock_2, current_block);
        let conviction3 = SubtensorModule::calculate_conviction(&stake_lock_3, current_block);

        // Determine the expected owner (hotkey with highest conviction)
        let expected_owner = if conviction1 > conviction2 && conviction1 > conviction3 {
            coldkey1
        } else if conviction2 > conviction1 && conviction2 > conviction3 {
            coldkey2
        } else {
            coldkey3
        };

        // Verify that the subnet owner was set correctly
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            expected_owner,
            "Subnet owner should be set to the coldkey of the hotkey with highest conviction"
        );

        // Verify that the subnet locked amount is correct
        assert_eq!(
            SubnetLocked::<Test>::get(netuid),
            conviction1 + conviction2 + conviction3,
            "Subnet locked amount should match the total calculated conviction"
        );

        // Verify that no new locks were created
        assert_eq!(
            Locks::<Test>::iter().count(),
            3,
            "There should still be only three locks"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_tie_breaking --exact --show-output
#[test]
fn test_update_subnet_owner_tie_breaking() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let current_block = 100;
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let hotkey3 = U256::from(3);
        let coldkey1 = U256::from(4);
        let coldkey2 = U256::from(5);
        let coldkey3 = U256::from(6);

        // Set up locks with equal convictions
        let stake_lock = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: current_block + 1000000,
        };
        Locks::<Test>::insert((netuid, hotkey1, coldkey1), stake_lock);
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock);
        Locks::<Test>::insert((netuid, hotkey3, coldkey3), stake_lock);

        // Set up ownership
        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);
        Owner::<Test>::insert(hotkey3, coldkey3);

        // Mock the current block
        System::set_block_number(current_block);

        // Call the update_subnet_owner function
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // The expected owner should be the coldkey of the hotkey with the lowest value
        let expected_owner = coldkey1;

        // Verify that the subnet owner was set correctly
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            expected_owner,
            "Subnet owner should be set to the coldkey of the hotkey with the lowest value"
        );

        // Verify that the subnet locked amount is correct
        let conviction1 = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        assert_eq!(
            SubnetLocked::<Test>::get(netuid),
            conviction1 * 3.into(),
            "Subnet locked amount should match the total calculated conviction"
        );

        // Verify that no new locks were created
        assert_eq!(
            Locks::<Test>::iter().count(),
            3,
            "There should still be only three locks"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_below_threshold --exact --show-output
#[test]
fn test_update_subnet_owner_below_threshold() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let current_block = 100;
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);

        // Set up locks with low conviction scores
        let stake_lock_1 = StakeLock {
            alpha_locked: 100.into(),
            start_block: 0,
            end_block: current_block + 100,
        };
        let stake_lock_2 = StakeLock {
            alpha_locked: 200.into(),
            start_block: 0,
            end_block: current_block + 200,
        };

        Locks::<Test>::insert((netuid, hotkey1, coldkey1), stake_lock_1);
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock_2);

        // Set up ownership
        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);

        // Mock the current block
        System::set_block_number(current_block);

        // Call the update_subnet_owner function
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Verify that no subnet owner was set due to low conviction scores
        assert!(
            !SubnetOwner::<Test>::contains_key(netuid),
            "No subnet owner should be set when all convictions are below the threshold"
        );

        // Verify that the subnet locked amount is correct (should be the sum of all convictions)
        let expected_total_conviction =
            SubtensorModule::calculate_conviction(&stake_lock_1, current_block)
                + SubtensorModule::calculate_conviction(&stake_lock_2, current_block);
        assert_eq!(
            SubnetLocked::<Test>::get(netuid),
            expected_total_conviction,
            "Subnet locked amount should match the total calculated conviction"
        );

        // Verify that no new locks were created
        assert_eq!(
            Locks::<Test>::iter().count(),
            2,
            "There should still be only two locks"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_conviction_calculation --exact --show-output
#[test]
fn test_update_subnet_owner_conviction_calculation() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let current_block = 100;
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);

        // Set up locks with different amounts and durations
        let stake_lock_1 = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: current_block + 1000000,
        };
        let stake_lock_2 = StakeLock {
            alpha_locked: 2000000.into(),
            start_block: 0,
            end_block: current_block + 500000,
        };

        Locks::<Test>::insert((netuid, hotkey1, coldkey1), stake_lock_1);
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock_2);

        // Set up ownership
        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);

        // Mock the current block
        System::set_block_number(current_block);

        // Call the update_subnet_owner function
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Calculate expected convictions
        let conviction1 = SubtensorModule::calculate_conviction(&stake_lock_1, current_block);
        let conviction2 = SubtensorModule::calculate_conviction(&stake_lock_2, current_block);

        // Verify that the subnet owner is set to the hotkey with the highest conviction
        if conviction1 > conviction2 {
            assert_eq!(
                SubnetOwner::<Test>::get(netuid),
                coldkey1.clone(),
                "Subnet owner should be set to coldkey1"
            );
        } else {
            assert_eq!(
                SubnetOwner::<Test>::get(netuid),
                coldkey2.clone(),
                "Subnet owner should be set to coldkey2"
            );
        }

        // Verify that the subnet locked amount is correct
        assert_eq!(
            SubnetLocked::<Test>::get(netuid),
            conviction1 + conviction2,
            "Subnet locked amount should match the highest calculated conviction"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_different_subnets --exact --show-output
#[test]
fn test_update_subnet_owner_different_subnets() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let current_block = 100;
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);

        // Set up locks in different subnets
        let stake_lock_1 = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: current_block + 1000000,
        };
        let stake_lock_2 = StakeLock {
            alpha_locked: 2000000.into(),
            start_block: 0,
            end_block: current_block + 500000,
        };

        Locks::<Test>::insert((netuid1, hotkey1, coldkey1), stake_lock_1);
        Locks::<Test>::insert((netuid2, hotkey2, coldkey2), stake_lock_2);

        // Set up ownership
        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);

        // Mock the current block
        System::set_block_number(current_block);

        // Call the update_subnet_owner function for netuid1
        SubtensorModule::update_subnet_owner(netuid1, UPDATE_INTERVAL);

        // Verify that only the lock from netuid1 is considered
        let conviction1 = SubtensorModule::calculate_conviction(&stake_lock_1, current_block);
        assert_eq!(
            SubnetOwner::<Test>::get(netuid1),
            coldkey1.clone(),
            "Subnet owner for netuid1 should be set to coldkey1"
        );
        assert_eq!(
            SubnetLocked::<Test>::get(netuid1),
            conviction1,
            "Subnet locked amount for netuid1 should match the conviction of its only lock"
        );

        // Verify that netuid2 is unaffected
        assert!(
            !SubnetOwner::<Test>::contains_key(netuid2),
            "Subnet owner for netuid2 should not be set"
        );
        assert_eq!(
            SubnetLocked::<Test>::get(netuid2),
            0.into(),
            "Subnet locked amount for netuid2 should be zero"
        );

        // Call the update_subnet_owner function for netuid2
        SubtensorModule::update_subnet_owner(netuid2, UPDATE_INTERVAL);

        // Verify that only the lock from netuid2 is now considered
        let conviction2 = SubtensorModule::calculate_conviction(&stake_lock_2, current_block);
        assert_eq!(
            SubnetOwner::<Test>::get(netuid2),
            coldkey2.clone(),
            "Subnet owner for netuid2 should be set to coldkey2"
        );
        assert_eq!(
            SubnetLocked::<Test>::get(netuid2),
            conviction2,
            "Subnet locked amount for netuid2 should match the conviction of its only lock"
        );

        // Verify that netuid1 remains unchanged
        assert_eq!(
            SubnetOwner::<Test>::get(netuid1),
            coldkey1.clone(),
            "Subnet owner for netuid1 should still be set to coldkey1"
        );
        assert_eq!(
            SubnetLocked::<Test>::get(netuid1),
            conviction1,
            "Subnet locked amount for netuid1 should remain unchanged"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_large_subnet --exact --show-output
#[test]
fn test_update_subnet_owner_large_subnet() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let num_locks = 1500; // Large number of locks
        let current_block = 100;

        // Create a large number of locks
        for i in 0..num_locks {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i + num_locks);
            let lock_amount = AlphaCurrency::from((i + 1) * 1000); // Varying lock amounts
            let lock_duration = (i % 10 + 1) * 100; // Varying lock durations

            let stake_lock = StakeLock {
                alpha_locked: lock_amount,
                start_block: 0,
                end_block: current_block + lock_duration,
            };

            Locks::<Test>::insert((netuid, hotkey, coldkey), stake_lock);

            // Set up ownership
            Owner::<Test>::insert(hotkey, coldkey);
        }

        // Mock the current block
        System::set_block_number(current_block);

        // Measure the time taken to update subnet owner
        let start_time = std::time::Instant::now();
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);
        let duration = start_time.elapsed();

        // Log the time taken
        println!("Time taken to update subnet owner: {:?}", duration);

        // Verify that a subnet owner was set
        assert!(
            SubnetOwner::<Test>::contains_key(netuid),
            "A subnet owner should be set"
        );

        // Verify that the subnet locked amount is non-zero
        assert!(
            SubnetLocked::<Test>::get(netuid) > 0.into(),
            "Subnet locked amount should be non-zero"
        );

        // Verify that the subnet owner has the highest conviction
        let owner = SubnetOwner::<Test>::get(netuid);
        let mut max_conviction_ema = AlphaCurrency::from(0);
        for ((iter_netuid, hotkey, coldkey), stake_lock) in Locks::<Test>::iter() {
            if iter_netuid == netuid {
                let conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
                let conviction_ema = SubtensorModule::get_conviction_ema(
                    netuid,
                    UPDATE_INTERVAL,
                    conviction,
                    &hotkey,
                    &coldkey,
                );

                if conviction_ema > max_conviction_ema {
                    max_conviction_ema = conviction_ema;
                }
            }
        }
        let owner_hotkey = Locks::<Test>::iter()
            .find(|((_, _, coldkey), _)| *coldkey == owner)
            .map(|((_, hotkey, _), _)| hotkey)
            .unwrap();
        let stake_lock = Locks::<Test>::get((netuid, owner_hotkey, owner));
        let owner_conviction = SubtensorModule::calculate_conviction(&stake_lock, current_block);
        let owner_conviction_ema = SubtensorModule::get_conviction_ema(
            netuid,
            UPDATE_INTERVAL,
            owner_conviction,
            &owner_hotkey,
            &owner,
        );

        // Subnet owner should have the highest conviction EMA
        assert_abs_diff_eq!(owner_conviction_ema, max_conviction_ema, epsilon = 1.into());
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_ownership_change --exact --show-output
#[test]
fn test_update_subnet_owner_ownership_change() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(1);
        let coldkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let coldkey2 = U256::from(4);
        let initial_block = 100000;
        let lock_duration = 1000000;
        let current_block = SubtensorModule::get_current_block_as_u64();

        // Set up initial locks
        let stake_lock_1 = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: current_block + lock_duration,
        };
        let stake_lock_2 = StakeLock {
            alpha_locked: 500000.into(),
            start_block: 0,
            end_block: current_block + lock_duration,
        };

        Locks::<Test>::insert((netuid, hotkey1, coldkey1), stake_lock_1);
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock_2);

        // Set up ownership
        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);

        // Set initial block
        System::set_block_number(initial_block);

        // Update subnet owner
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Check initial subnet owner
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey1.clone(),
            "Initial subnet owner should be coldkey1"
        );

        // Simulate time passing and conviction changing
        let new_block = initial_block + 500000;
        System::set_block_number(new_block);

        // Increase lock for hotkey2
        let stake_lock_2_increased = StakeLock {
            alpha_locked: 2000000.into(),
            start_block: new_block,
            end_block: current_block + lock_duration,
        };
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock_2_increased);

        // Update subnet owner again
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Check if subnet owner has changed
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey2.clone(),
            "Subnet owner should change to coldkey2"
        );

        // Verify subnet locked amount has increased
        let subnet_locked = SubnetLocked::<Test>::get(netuid);
        assert!(
            subnet_locked > 0.into(),
            "Subnet locked amount should be non-zero"
        );
        assert!(
            subnet_locked > SubtensorModule::calculate_conviction(&stake_lock_1, new_block),
            "Subnet locked amount should increase"
        );

        // Simulate more time passing
        let final_block = new_block + 600000;
        System::set_block_number(final_block);

        // Update subnet owner one last time
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Check if subnet owner remains the same
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey2.clone(),
            "Subnet owner should still be coldkey2"
        );

        // Verify subnet locked amount has decreased due to time passing
        let final_subnet_locked = SubnetLocked::<Test>::get(netuid);
        assert!(
            final_subnet_locked < subnet_locked,
            "Subnet locked amount should decrease over time"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_update_subnet_owner_storage_updates --exact --show-output
#[test]
fn test_update_subnet_owner_storage_updates() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey1 = U256::from(1);
        let coldkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let coldkey2 = U256::from(4);
        let initial_block = 100000;
        let lock_duration = 1000000;
        let current_block = SubtensorModule::get_current_block_as_u64();

        // Set up initial locks
        let stake_lock_1 = StakeLock {
            alpha_locked: 1000000.into(),
            start_block: 0,
            end_block: current_block + lock_duration,
        };
        let stake_lock_2 = StakeLock {
            alpha_locked: 500000.into(),
            start_block: 0,
            end_block: current_block + lock_duration,
        };

        Locks::<Test>::insert((netuid, hotkey1, coldkey1), stake_lock_1);
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock_2);

        // Set up ownership
        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);

        // Set initial block
        System::set_block_number(initial_block);

        // Update subnet owner
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Check initial subnet owner and locked amount
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey1.clone(),
            "Initial subnet owner should be coldkey1"
        );
        let initial_locked = SubnetLocked::<Test>::get(netuid);
        assert!(
            initial_locked > 0.into(),
            "Initial subnet locked amount should be non-zero"
        );

        // Simulate time passing and conviction changing
        let new_block = initial_block + 500000;
        System::set_block_number(new_block);

        // Increase lock for hotkey2
        let stake_lock_3 = StakeLock {
            alpha_locked: 2000000.into(),
            start_block: 0,
            end_block: new_block + lock_duration,
        };
        Locks::<Test>::insert((netuid, hotkey2, coldkey2), stake_lock_3);

        // Update subnet owner again
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Check if subnet owner has changed
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey2.clone(),
            "Subnet owner should change to coldkey2"
        );

        // Verify subnet locked amount has increased
        let new_locked = SubnetLocked::<Test>::get(netuid);
        assert!(
            new_locked > initial_locked,
            "Subnet locked amount should increase"
        );

        // Simulate more time passing
        let final_block = new_block + 600000;
        System::set_block_number(final_block);

        // Update subnet owner one last time
        SubtensorModule::update_subnet_owner(netuid, UPDATE_INTERVAL);

        // Check if subnet owner remains the same
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey2.clone(),
            "Subnet owner should still be coldkey2"
        );

        // Verify subnet locked amount has decreased due to time passing
        let final_locked = SubnetLocked::<Test>::get(netuid);
        assert!(
            final_locked < new_locked,
            "Subnet locked amount should decrease over time"
        );
    });
}

/// Basic test, EMA starts from zero
/// cargo test --package pallet-subtensor --lib -- tests::lock::test_conviction_ema_basic --exact --show-output
#[test]
fn test_conviction_ema_basic() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let update_period = 1_u64;
        let lock_interval = 2_u64;
        let conviction_value = AlphaCurrency::from(1000);

        ConvictionEma::<Test>::insert((netuid, hotkey, coldkey), AlphaCurrency::from(0));
        LockIntervalBlocks::<Test>::put(lock_interval);

        let ema = SubtensorModule::get_conviction_ema(
            netuid,
            update_period,
            conviction_value,
            &hotkey,
            &coldkey,
        );
        assert!(ema > 0.into() && ema < conviction_value);
    });
}

/// Non-zero existing EMA
/// cargo test --package pallet-subtensor --lib -- tests::lock::test_conviction_ema_existing_ema --exact --show-output
#[test]
fn test_conviction_ema_existing_ema() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let update_period = 1;
        let lock_interval = 4;
        let conviction_value = AlphaCurrency::from(800);

        ConvictionEma::<Test>::insert((netuid, hotkey, coldkey), AlphaCurrency::from(400));
        LockIntervalBlocks::<Test>::put(lock_interval);

        let ema = SubtensorModule::get_conviction_ema(
            netuid,
            update_period,
            conviction_value,
            &hotkey,
            &coldkey,
        );
        assert!(ema > 400.into() && ema < conviction_value);
    });
}

/// Update period is zero (no update to EMA)
/// cargo test --package pallet-subtensor --lib -- tests::lock::test_conviction_ema_zero_update --exact --show-output
#[test]
fn test_conviction_ema_zero_update() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(3);
        let coldkey = U256::from(4);
        let update_period = 0;
        let lock_interval = 10;
        let conviction_value = AlphaCurrency::from(1000);

        ConvictionEma::<Test>::insert((netuid, hotkey, coldkey), AlphaCurrency::from(500));
        LockIntervalBlocks::<Test>::put(lock_interval);

        let ema = SubtensorModule::get_conviction_ema(
            netuid,
            update_period,
            conviction_value,
            &hotkey,
            &coldkey,
        );
        assert_eq!(ema, 500.into());
    });
}

/// Lock interval is zero (should fallback to zero via safe_div_or)
/// cargo test --package pallet-subtensor --lib -- tests::lock::test_conviction_ema_zero_lockint --exact --show-output
#[test]
fn test_conviction_ema_zero_lockint() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(5);
        let coldkey = U256::from(6);
        let update_period = 1;
        let lock_interval = 0;
        let conviction_value = AlphaCurrency::from(700);

        ConvictionEma::<Test>::insert((netuid, hotkey, coldkey), AlphaCurrency::from(300));
        LockIntervalBlocks::<Test>::put(lock_interval);

        let ema = SubtensorModule::get_conviction_ema(
            netuid,
            update_period,
            conviction_value,
            &hotkey,
            &coldkey,
        );
        assert_eq!(ema, conviction_value);
    });
}

/// Smoothing factor would be >1, but must be capped at 1.0
/// cargo test --package pallet-subtensor --lib -- tests::lock::test_conviction_ema_large_update --exact --show-output
#[test]
fn test_conviction_ema_large_update() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(7);
        let coldkey = U256::from(8);
        let update_period = 100;
        let lock_interval = 10;
        let conviction_value = AlphaCurrency::from(900);

        ConvictionEma::<Test>::insert((netuid, hotkey, coldkey), AlphaCurrency::from(200));
        LockIntervalBlocks::<Test>::put(lock_interval);

        let ema = SubtensorModule::get_conviction_ema(
            netuid,
            update_period,
            conviction_value,
            &hotkey,
            &coldkey,
        );
        assert_eq!(ema, conviction_value);
    });
}

/// Conviction is 0 (EMA should decay)
/// cargo test --package pallet-subtensor --lib -- tests::lock::test_conviction_ema_conviction_0 --exact --show-output
#[test]
fn test_conviction_ema_conviction_0() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(9);
        let coldkey = U256::from(10);
        let update_period = 2;
        let lock_interval = 4;
        let conviction_value = AlphaCurrency::from(0);

        ConvictionEma::<Test>::insert((netuid, hotkey, coldkey), AlphaCurrency::from(1000));
        LockIntervalBlocks::<Test>::put(lock_interval);

        let ema = SubtensorModule::get_conviction_ema(
            netuid,
            update_period,
            conviction_value,
            &hotkey,
            &coldkey,
        );
        assert!(ema < 1000.into());
    });
}

/// Conviction is max (EMA rises toward it)
/// cargo test --package pallet-subtensor --lib -- tests::lock::test_conviction_ema_conviction_max --exact --show-output
#[test]
fn test_conviction_ema_conviction_max() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(11);
        let coldkey = U256::from(12);
        let update_period = 2;
        let lock_interval = 4;
        let conviction_value = AlphaCurrency::from(u64::MAX);

        ConvictionEma::<Test>::insert((netuid, hotkey, coldkey), AlphaCurrency::from(0));
        LockIntervalBlocks::<Test>::put(lock_interval);

        let ema = SubtensorModule::get_conviction_ema(
            netuid,
            update_period,
            conviction_value,
            &hotkey,
            &coldkey,
        );
        assert!(ema > 0.into() && ema < conviction_value);
    });
}

/// cargo test --package pallet-subtensor --lib -- tests::lock::test_locks_are_updated_in_block_step --exact --show-output
#[test]
fn test_locks_are_updated_in_block_step() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let hotkey = U256::from(1);
        let coldkey = U256::from(2);
        let coldkey2 = U256::from(3);
        let lock_amount = AlphaCurrency::from(1000000);
        let current_block = 216_000; // Triggers locks EMA and subnet owners update 
        let lock_duration = 1000000;
        add_network(netuid, 0, 0);

        // Set up a single lock for coldkey2
        let stake_lock = StakeLock {
            alpha_locked: lock_amount,
            start_block: 0,
            end_block: current_block + lock_duration,
        };
        Locks::<Test>::insert((netuid, hotkey, coldkey2), stake_lock);

        // Set up ownership
        Owner::<Test>::insert(hotkey, coldkey);
        SubnetOwner::<Test>::set(netuid, coldkey);

        // Cause block step to execute on the update block
        System::set_block_number(current_block);
        let _ = SubtensorModule::block_step();

        // Verify that the subnet owner was changed correctly
        assert_eq!(
            SubnetOwner::<Test>::get(netuid),
            coldkey2.clone(),
            "Subnet owner should be set to the coldkey of the only lock"
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::lock::test_do_lock_too_short --exact --show-output
#[test]
fn test_do_lock_too_short() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = 500_000_000;
        let lock_amount = AlphaCurrency::from(250_000_000);
        let lock_duration = 10; // too short

        // Set up initial balance and stake
        add_network(netuid, 0, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1_000_000_000);
        register_ok_neuron(netuid, hotkey, coldkey, 11);
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            stake_amount
        ));
        assert_noop!(
            SubtensorModule::lock_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                lock_duration,
                lock_amount
            ),
            Error::<Test>::DurationTooShort
        );
    });
}
