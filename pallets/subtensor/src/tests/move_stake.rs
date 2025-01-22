use super::mock::*;
use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{Get, U256};

// 1. test_do_move_success
// Description: Test a successful move of stake between two hotkeys in the same subnet
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_success --exact --nocapture
#[test]
fn test_do_move_success() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, stake_amount, fee);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Perform the move
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            netuid,
            netuid,
            alpha,
        ));

        // Check that the stake has been moved
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid
            ),
            stake_amount - 2 * fee,
            epsilon = stake_amount / 1000
        );
    });
}

// 2. test_do_move_different_subnets
// Description: Test moving stake between two hotkeys in different subnets
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_different_subnets --exact --nocapture
#[test]
fn test_do_move_different_subnets() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake and subnets
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        SubtensorModule::stake_into_subnet(
            &origin_hotkey,
            &coldkey,
            origin_netuid,
            stake_amount,
            fee,
        );
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            origin_netuid,
        );

        // Perform the move
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            origin_netuid,
            destination_netuid,
            alpha,
        ));

        // Check that the stake has been moved
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                origin_netuid
            ),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                destination_netuid
            ),
            stake_amount - 2 * fee,
            epsilon = stake_amount / 1000
        );
    });
}

// 4. test_do_move_nonexistent_subnet
// Description: Attempt to move stake to a non-existent subnet, which should fail
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_nonexistent_subnet --exact --nocapture
#[test]
fn test_do_move_nonexistent_subnet() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let nonexistent_netuid = 99; // Assuming this subnet doesn't exist
        let stake_amount = 1_000_000;
        let fee = 0;

        // Set up initial stake
        SubtensorModule::stake_into_subnet(
            &origin_hotkey,
            &coldkey,
            origin_netuid,
            stake_amount,
            fee,
        );
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            origin_netuid,
        );

        // Attempt to move stake to a non-existent subnet
        assert_noop!(
            SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(coldkey),
                origin_hotkey,
                destination_hotkey,
                origin_netuid,
                nonexistent_netuid,
                alpha,
            ),
            Error::<Test>::SubnetNotExists
        );

        // Check that the stake remains unchanged
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                origin_netuid
            ),
            stake_amount,
            epsilon = 100
        );
    });
}

// 5. test_do_move_nonexistent_origin_hotkey
// Description: Attempt to move stake from a non-existent origin hotkey, which should fail
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_nonexistent_origin_hotkey --exact --nocapture
#[test]
fn test_do_move_nonexistent_origin_hotkey() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let nonexistent_origin_hotkey = U256::from(99); // Assuming this hotkey doesn't exist
        let destination_hotkey = U256::from(3);

        // Attempt to move stake from a non-existent origin hotkey
        assert_noop!(
            SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(coldkey),
                nonexistent_origin_hotkey,
                destination_hotkey,
                netuid,
                netuid,
                123
            ),
            Error::<Test>::HotKeyAccountNotExists
        );

        // Check that no stake was moved
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &nonexistent_origin_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
    });
}

// 6. test_do_move_nonexistent_destination_hotkey
// Description: Attempt to move stake to a non-existent destination hotkey, which should fail
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_nonexistent_destination_hotkey --exact --nocapture
#[test]
fn test_do_move_nonexistent_destination_hotkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let nonexistent_destination_hotkey = U256::from(99); // Assuming this hotkey doesn't exist
        let netuid = 1;
        let stake_amount = 1_000_000;
        let fee = 0;

        // Set up initial stake
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, stake_amount, fee);

        // Attempt to move stake from a non-existent origin hotkey
        add_network(netuid, 0, 0);
        assert_noop!(
            SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(coldkey),
                origin_hotkey,
                nonexistent_destination_hotkey,
                netuid,
                netuid,
                1234
            ),
            Error::<Test>::HotKeyAccountNotExists
        );

        // Check that the stake was moved successfully
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid
            ),
            stake_amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &nonexistent_destination_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
    });
}

// 8. test_do_move_all_stake
// Description: Test moving all stake from one hotkey to another
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_all_stake --exact --nocapture
#[test]
fn test_do_move_all_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, stake_amount, fee);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Move all stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            netuid,
            netuid,
            alpha,
        ));

        // Check that all stake was moved
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid
            ),
            stake_amount - 2 * fee,
            epsilon = stake_amount / 1000
        );
    });
}

#[test]
fn test_do_move_half_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, stake_amount, fee);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Move all stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            netuid,
            netuid,
            alpha / 2,
        ));

        // Check that all stake was moved
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid
            ),
            alpha / 2,
            epsilon = alpha / 1000
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid
            ),
            alpha / 2 - fee,
            epsilon = alpha / 1000
        );
    });
}

// 9. test_do_move_partial_stake
// Description: Test moving a portion of stake from one hotkey to another
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_partial_stake --exact --nocapture
#[test]
fn test_do_move_partial_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let total_stake = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, total_stake, fee);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Move partial stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            netuid,
            netuid,
            alpha,
        ));

        // Check that the correct amount of stake was moved
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid
            ),
            total_stake - 2 * fee,
            epsilon = total_stake / 1000
        );
    });
}

// 10. test_do_move_multiple_times
// Description: Test moving stake multiple times between the same hotkeys
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_multiple_times --exact --nocapture
#[test]
fn test_do_move_multiple_times() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let hotkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let initial_stake = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey1);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey2);
        SubtensorModule::stake_into_subnet(&hotkey1, &coldkey, netuid, initial_stake, fee);

        // Move stake multiple times
        for _ in 0..3 {
            let alpha1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey, netuid,
            );
            assert_ok!(SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey1,
                hotkey2,
                netuid,
                netuid,
                alpha1,
            ));
            let alpha2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2, &coldkey, netuid,
            );
            assert_ok!(SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey2,
                hotkey1,
                netuid,
                netuid,
                alpha2,
            ));
        }

        // Check final stake distribution
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey1, &coldkey, netuid),
            initial_stake - 7 * fee,
            epsilon = initial_stake / 1000
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey2, &coldkey, netuid),
            0
        );
    });
}

// 13. test_do_move_wrong_origin
// Description: Attempt to move stake with a different origin than the coldkey, which should fail
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_wrong_origin --exact --nocapture
#[test]
fn test_do_move_wrong_origin() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let wrong_coldkey = U256::from(99);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let netuid = 1;
        let stake_amount = 1000;
        let fee = 0;

        // Set up initial stake
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, stake_amount, fee);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Attempt to move stake with wrong origin
        add_network(netuid, 0, 0);
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        assert_err!(
            SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(wrong_coldkey),
                origin_hotkey,
                destination_hotkey,
                netuid,
                netuid,
                alpha,
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Check that no stake was moved
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid
            ),
            stake_amount
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
    });
}

// 14. test_do_move_same_hotkey
// Description: Attempt to move stake to the same hotkey, which should fail or have no effect
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_same_hotkey --exact --nocapture
#[test]
fn test_do_move_same_hotkey() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, stake_amount, fee);
        let alpha =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        // Attempt to move stake to the same hotkey
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            hotkey,
            netuid,
            netuid,
            alpha,
        ));

        // Check that stake remains unchanged
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid),
            alpha - fee,
            epsilon = alpha / 1000
        );
    });
}

// 15. test_do_move_event_emission
// Description: Verify that the correct event is emitted after a successful move
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_event_emission --exact --nocapture
#[test]
fn test_do_move_event_emission() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, stake_amount, 0); // use 0 fee for precision
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Move stake and capture events
        System::reset_events();
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            netuid,
            netuid,
            alpha,
        ));

        // Check for the correct event emission
        System::assert_last_event(
            Event::StakeMoved(
                coldkey,
                origin_hotkey,
                netuid,
                destination_hotkey,
                netuid,
                stake_amount - fee - 1, // Should be TAO equivalent
            )
            .into(),
        );
    });
}

// 16. test_do_move_storage_updates
// Description: Verify that all relevant storage items are correctly updated after a move
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_storage_updates --exact --nocapture
#[test]
fn test_do_move_storage_updates() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultMinStake::<Test>::get();

        // Set up initial stake
        SubtensorModule::stake_into_subnet(
            &origin_hotkey,
            &coldkey,
            origin_netuid,
            stake_amount,
            fee,
        );

        // Move stake
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            origin_netuid,
        );

        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            origin_netuid,
            destination_netuid,
            alpha,
        ));

        // Verify storage updates
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                origin_netuid
            ),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                destination_netuid
            ),
            alpha - fee,
            epsilon = alpha / 1000
        );
    });
}

// 18. test_do_move_max_values
// Description: Test moving the maximum possible stake values to check for overflows
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test move -- test_do_move_max_values --exact --nocapture
#[test]
fn test_do_move_max_values() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let coldkey = U256::from(1);
        let origin_hotkey = U256::from(2);
        let destination_hotkey = U256::from(3);
        let max_stake = u64::MAX;
        let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let fee = 0;

        // Set up initial stake with maximum value
        SubtensorModule::create_account_if_non_existent(&coldkey, &origin_hotkey);
        SubtensorModule::create_account_if_non_existent(&coldkey, &destination_hotkey);
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, max_stake, fee);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Move maximum stake
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            origin_hotkey,
            destination_hotkey,
            netuid,
            netuid,
            alpha,
        ));

        // Verify stake movement without overflow
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &origin_hotkey,
                &coldkey,
                netuid
            ),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                netuid
            ),
            alpha,
            epsilon = 5
        );
    });
}

// Verify moving too low amount is impossible
#[test]
fn test_moving_too_little_fails() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = DefaultMinStake::<Test>::get();
        let fee = DefaultMinStake::<Test>::get();

        //add network
        let netuid: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);
        let netuid2: u16 = add_dynamic_network(&hotkey_account_id, &coldkey_account_id);

        // Give it some $$$ in his coldkey balance
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount + fee);

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(coldkey_account_id),
            hotkey_account_id,
            netuid,
            amount + fee
        ));

        // Coldkey / hotkey 0 decreases take to 5%. This should fail as the minimum take is 9%
        assert_err!(
            SubtensorModule::move_stake(
                RuntimeOrigin::signed(coldkey_account_id),
                hotkey_account_id,
                hotkey_account_id,
                netuid,
                netuid2,
                1
            ),
            Error::<Test>::AmountTooLow
        );
    });
}
