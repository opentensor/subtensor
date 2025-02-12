use super::mock::*;
use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{Get, U256};
use substrate_fixed::types::I96F32;

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
        let fee = DefaultStakingFee::<Test>::get();

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
        let fee = DefaultStakingFee::<Test>::get();

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
        let alpha_fee: I96F32 =
            I96F32::from_num(fee) / SubtensorModule::get_alpha_price(destination_netuid);
        let expected_value = I96F32::from_num(alpha)
            * SubtensorModule::get_alpha_price(origin_netuid)
            / SubtensorModule::get_alpha_price(destination_netuid);
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                destination_netuid
            ),
            (expected_value - alpha_fee).to_num::<u64>(),
            epsilon = (expected_value / 1000).to_num::<u64>()
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
        add_network(netuid, 1, 0);
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
        let fee = DefaultStakingFee::<Test>::get();

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
        let fee = DefaultStakingFee::<Test>::get();

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
        let fee = DefaultStakingFee::<Test>::get();

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
        let fee = DefaultStakingFee::<Test>::get();

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
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = 0;

        // Set up initial stake
        SubtensorModule::stake_into_subnet(&origin_hotkey, &coldkey, netuid, stake_amount, fee);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            netuid,
        );

        // Attempt to move stake with wrong origin
        add_network(netuid, 1, 0);
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
        let fee = DefaultStakingFee::<Test>::get();

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
        let fee = DefaultStakingFee::<Test>::get();

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
        let fee = DefaultStakingFee::<Test>::get();

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
        let alpha_fee =
            I96F32::from_num(fee) / SubtensorModule::get_alpha_price(destination_netuid);
        let alpha2 = I96F32::from_num(alpha) * SubtensorModule::get_alpha_price(origin_netuid)
            / SubtensorModule::get_alpha_price(destination_netuid);
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &destination_hotkey,
                &coldkey,
                destination_netuid
            ),
            (alpha2 - alpha_fee).to_num::<u64>(),
            epsilon = alpha / 1000
        );
    });
}

// 18. test_do_move_max_values
// Description: Test moving the maximum possible stake values to check for overflows
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::move_stake::test_do_move_max_values --exact --show-output
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

        // Add lots of liquidity to bypass low liquidity check
        SubnetTAO::<Test>::insert(netuid, u64::MAX / 1000);
        SubnetAlphaIn::<Test>::insert(netuid, u64::MAX / 1000);

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
            epsilon = alpha / 1_000_000
        );
    });
}

// Verify moving too low amount is impossible
#[test]
fn test_moving_too_little_unstakes() {
    new_test_ext(1).execute_with(|| {
        let hotkey_account_id = U256::from(533453);
        let coldkey_account_id = U256::from(55453);
        let amount = DefaultMinStake::<Test>::get();
        let fee = DefaultStakingFee::<Test>::get();

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

#[test]
fn test_do_transfer_success() {
    new_test_ext(1).execute_with(|| {
        // 1. Create a new dynamic network and IDs.
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let fee = DefaultStakingFee::<Test>::get();

        // 2. Define the origin coldkey, destination coldkey, and hotkey to be used.
        let origin_coldkey = U256::from(1);
        let destination_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;

        // 3. Set up initial stake: (origin_coldkey, hotkey) on netuid.
        SubtensorModule::create_account_if_non_existent(&origin_coldkey, &hotkey);
        SubtensorModule::create_account_if_non_existent(&destination_coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &origin_coldkey, netuid, stake_amount, 0);
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &origin_coldkey,
            netuid,
        );

        // 4. Transfer the entire stake to the destination coldkey on the same subnet (netuid, netuid).
        assert_ok!(SubtensorModule::do_transfer_stake(
            RuntimeOrigin::signed(origin_coldkey),
            destination_coldkey,
            hotkey,
            netuid,
            netuid,
            alpha
        ));

        // 5. Check that the stake has moved.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &origin_coldkey,
                netuid
            ),
            0
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &destination_coldkey,
                netuid
            ),
            stake_amount - fee,
            epsilon = stake_amount / 1000
        );
    });
}

#[test]
fn test_do_transfer_nonexistent_subnet() {
    new_test_ext(1).execute_with(|| {
        let origin_coldkey = U256::from(1);
        let destination_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let nonexistent_netuid = 9999;
        let stake_amount = DefaultMinStake::<Test>::get() * 5;

        assert_noop!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(origin_coldkey),
                destination_coldkey,
                hotkey,
                nonexistent_netuid,
                nonexistent_netuid,
                stake_amount
            ),
            Error::<Test>::SubnetNotExists
        );
    });
}

#[test]
fn test_do_transfer_nonexistent_hotkey() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let origin_coldkey = U256::from(1);
        let destination_coldkey = U256::from(2);
        let nonexistent_hotkey = U256::from(999);

        assert_noop!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(origin_coldkey),
                destination_coldkey,
                nonexistent_hotkey,
                netuid,
                netuid,
                100
            ),
            Error::<Test>::HotKeyAccountNotExists
        );
    });
}

#[test]
fn test_do_transfer_insufficient_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let origin_coldkey = U256::from(1);
        let destination_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;

        SubtensorModule::create_account_if_non_existent(&origin_coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &origin_coldkey, netuid, stake_amount, 0);

        let alpha = stake_amount * 2;
        assert_noop!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(origin_coldkey),
                destination_coldkey,
                hotkey,
                netuid,
                netuid,
                alpha
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

#[test]
fn test_do_transfer_wrong_origin() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1010);
        let subnet_owner_hotkey = U256::from(1011);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let origin_coldkey = U256::from(1);
        let wrong_coldkey = U256::from(9999);
        let destination_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        SubtensorModule::create_account_if_non_existent(&origin_coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&origin_coldkey, stake_amount + fee);
        SubtensorModule::stake_into_subnet(&hotkey, &origin_coldkey, netuid, stake_amount, fee);

        assert_noop!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(wrong_coldkey),
                destination_coldkey,
                hotkey,
                netuid,
                netuid,
                stake_amount
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

#[test]
fn test_do_transfer_minimum_stake_check() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let origin_coldkey = U256::from(1);
        let destination_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let stake_amount = DefaultMinStake::<Test>::get();
        SubtensorModule::create_account_if_non_existent(&origin_coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &origin_coldkey, netuid, stake_amount, 0);

        assert_err!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(origin_coldkey),
                destination_coldkey,
                hotkey,
                netuid,
                netuid,
                1
            ),
            Error::<Test>::AmountTooLow
        );
    });
}

#[test]
fn test_do_transfer_different_subnets() {
    new_test_ext(1).execute_with(|| {
        // 1. Create two distinct subnets.
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // 2. Define origin/destination coldkeys and hotkey.
        let origin_coldkey = U256::from(1);
        let destination_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        // 3. Create accounts if needed.
        SubtensorModule::create_account_if_non_existent(&origin_coldkey, &hotkey);
        SubtensorModule::create_account_if_non_existent(&destination_coldkey, &hotkey);

        // 4. Deposit free balance so transaction fees do not reduce staked funds.
        SubtensorModule::add_balance_to_coldkey_account(&origin_coldkey, 1_000_000_000);

        // 5. Stake into the origin subnet.
        SubtensorModule::stake_into_subnet(
            &hotkey,
            &origin_coldkey,
            origin_netuid,
            stake_amount,
            0,
        );

        // 6. Transfer entire stake from origin_netuid -> destination_netuid.
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &origin_coldkey,
            origin_netuid,
        );
        assert_ok!(SubtensorModule::do_transfer_stake(
            RuntimeOrigin::signed(origin_coldkey),
            destination_coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            alpha
        ));

        // 7. Verify origin now has 0 in origin_netuid.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &origin_coldkey,
                origin_netuid
            ),
            0
        );

        // 8. Verify stake ended up in destination subnet for destination coldkey.
        let dest_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &destination_coldkey,
            destination_netuid,
        );
        let expected_value = I96F32::from_num(stake_amount - fee)
            / SubtensorModule::get_alpha_price(destination_netuid);
        assert_abs_diff_eq!(
            dest_stake,
            expected_value.to_num::<u64>(),
            epsilon = (expected_value / 1000).to_num::<u64>()
        );
    });
}

#[test]
fn test_do_swap_success() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, origin_netuid, stake_amount, 0);
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            origin_netuid,
        );

        assert_ok!(SubtensorModule::do_swap_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            origin_netuid,
            destination_netuid,
            alpha_before,
        ));

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                origin_netuid
            ),
            0
        );

        let alpha_after = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            destination_netuid,
        );
        let alpha_fee =
            I96F32::from_num(fee) / SubtensorModule::get_alpha_price(destination_netuid);
        let expected_value = I96F32::from_num(alpha_before)
            * SubtensorModule::get_alpha_price(origin_netuid)
            / SubtensorModule::get_alpha_price(destination_netuid);
        assert_abs_diff_eq!(
            alpha_after,
            (expected_value - alpha_fee).to_num::<u64>(),
            epsilon = (expected_value / 1000).to_num::<u64>()
        );
    });
}

#[test]
fn test_do_swap_nonexistent_subnet() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let nonexistent_netuid: u16 = 9999;
        let stake_amount = 1_000_000;

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);

        assert_noop!(
            SubtensorModule::do_swap_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                nonexistent_netuid,
                nonexistent_netuid,
                stake_amount
            ),
            Error::<Test>::SubnetNotExists
        );
    });
}

#[test]
fn test_do_swap_nonexistent_hotkey() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid1 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let netuid2 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let nonexistent_hotkey = U256::from(999);
        let stake_amount = 10_000;

        assert_noop!(
            SubtensorModule::do_swap_stake(
                RuntimeOrigin::signed(coldkey),
                nonexistent_hotkey,
                netuid1,
                netuid2,
                stake_amount
            ),
            Error::<Test>::HotKeyAccountNotExists
        );
    });
}

#[test]
fn test_do_swap_insufficient_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid1 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let netuid2 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = DefaultMinStake::<Test>::get() * 5;
        let attempted_swap = stake_amount * 2;

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid1, stake_amount, 0);

        assert_noop!(
            SubtensorModule::do_swap_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid1,
                netuid2,
                attempted_swap
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

#[test]
fn test_do_swap_wrong_origin() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1010);
        let subnet_owner_hotkey = U256::from(1011);
        let netuid1 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let netuid2 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let real_coldkey = U256::from(1);
        let wrong_coldkey = U256::from(9999);
        let hotkey = U256::from(3);
        let stake_amount = 100_000;

        SubtensorModule::create_account_if_non_existent(&real_coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &real_coldkey, netuid1, stake_amount, 0);

        assert_noop!(
            SubtensorModule::do_swap_stake(
                RuntimeOrigin::signed(wrong_coldkey),
                hotkey,
                netuid1,
                netuid2,
                stake_amount
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}

#[test]
fn test_do_swap_minimum_stake_check() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(3);
        let total_stake = DefaultMinStake::<Test>::get();
        let swap_amount = 1;

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, total_stake, 0);

        assert_err!(
            SubtensorModule::do_swap_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                netuid,
                swap_amount
            ),
            Error::<Test>::AmountTooLow
        );
    });
}

#[test]
fn test_do_swap_same_subnet() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1100);
        let subnet_owner_hotkey = U256::from(1101);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid, stake_amount, 0);

        let alpha_before =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let fee_as_alpha = SubtensorModule::swap_tao_for_alpha(netuid, fee);

        assert_ok!(SubtensorModule::do_swap_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            netuid,
            alpha_before
        ));

        let alpha_after =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        assert_abs_diff_eq!(
            alpha_after,
            alpha_before - fee_as_alpha,
            epsilon = alpha_after / 10000
        );
    });
}

#[test]
fn test_do_swap_partial_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1100);
        let subnet_owner_hotkey = U256::from(1101);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let total_stake = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, origin_netuid, total_stake, 0);

        let swap_amount = total_stake / 2;
        assert_ok!(SubtensorModule::do_swap_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            origin_netuid,
            destination_netuid,
            swap_amount,
        ));

        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                origin_netuid
            ),
            total_stake - swap_amount,
            epsilon = total_stake / 1000
        );

        let alpha_fee =
            I96F32::from_num(fee) / SubtensorModule::get_alpha_price(destination_netuid);
        let expected_value = I96F32::from_num(swap_amount)
            * SubtensorModule::get_alpha_price(origin_netuid)
            / SubtensorModule::get_alpha_price(destination_netuid);
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                destination_netuid
            ),
            (expected_value - alpha_fee).to_num::<u64>(),
            epsilon = (expected_value / 1000).to_num::<u64>()
        );
    });
}

#[test]
fn test_do_swap_storage_updates() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1300);
        let subnet_owner_hotkey = U256::from(1301);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, origin_netuid, stake_amount, 0);

        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            origin_netuid,
        );
        assert_ok!(SubtensorModule::do_swap_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            origin_netuid,
            destination_netuid,
            alpha
        ));

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                origin_netuid
            ),
            0
        );

        let alpha_fee =
            I96F32::from_num(fee) / SubtensorModule::get_alpha_price(destination_netuid);
        let expected_value = I96F32::from_num(alpha)
            * SubtensorModule::get_alpha_price(origin_netuid)
            / SubtensorModule::get_alpha_price(destination_netuid);
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &coldkey,
                destination_netuid
            ),
            (expected_value - alpha_fee).to_num::<u64>(),
            epsilon = (expected_value / 1000).to_num::<u64>()
        );
    });
}

#[test]
fn test_do_swap_multiple_times() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1500);
        let subnet_owner_hotkey = U256::from(1501);
        let netuid1 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let netuid2 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let initial_stake = DefaultMinStake::<Test>::get() * 10;
        let fee = DefaultStakingFee::<Test>::get();

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, netuid1, initial_stake, 0);

        let mut total_alpha1_fee = 0;
        for _ in 0..3 {
            let alpha1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid1,
            );
            if alpha1 > 0 {
                assert_ok!(SubtensorModule::do_swap_stake(
                    RuntimeOrigin::signed(coldkey),
                    hotkey,
                    netuid1,
                    netuid2,
                    alpha1
                ));

                let fee_as_alpha = SubtensorModule::swap_tao_for_alpha(netuid1, fee);
                total_alpha1_fee += fee_as_alpha;
            }
            let alpha2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, &coldkey, netuid2,
            );
            if alpha2 > 0 {
                assert_ok!(SubtensorModule::do_swap_stake(
                    RuntimeOrigin::signed(coldkey),
                    hotkey,
                    netuid2,
                    netuid1,
                    alpha2
                ));

                let fee_as_alpha = SubtensorModule::swap_tao_for_alpha(netuid1, fee);
                total_alpha1_fee += fee_as_alpha;
            }
        }

        let final_stake_netuid1: u64 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid1);
        let final_stake_netuid2 =
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid2);
        let expected_stake = initial_stake - total_alpha1_fee;
        assert_abs_diff_eq!(
            final_stake_netuid1,
            expected_stake,
            epsilon = initial_stake / 10000
        );
        assert_eq!(final_stake_netuid2, 0);
    });
}

// cargo test --package pallet-subtensor --lib -- tests::move_stake::test_do_swap_allows_non_owned_hotkey --exact --show-output
#[test]
fn test_do_swap_allows_non_owned_hotkey() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let foreign_coldkey = U256::from(3);
        let stake_amount = DefaultMinStake::<Test>::get() * 10;

        SubtensorModule::create_account_if_non_existent(&foreign_coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(&hotkey, &coldkey, origin_netuid, stake_amount, 0);
        let alpha_before = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &coldkey,
            origin_netuid,
        );

        assert_ok!(SubtensorModule::do_swap_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            origin_netuid,
            destination_netuid,
            alpha_before,
        ));
    });
}

// cargo test --package pallet-subtensor --lib -- tests::move_stake::test_swap_stake_limit_validate --exact --show-output
#[test]
fn test_swap_stake_limit_validate() {
    // Testing the signed extension validate function
    // correctly filters the `add_stake` transaction.

    new_test_ext(0).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let stake_amount = 100_000_000_000;

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        let unstake_amount =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, origin_netuid, stake_amount, 0);

        // Setup limit price so that it doesn't allow much slippage at all
        let limit_price = ((SubtensorModule::get_alpha_price(origin_netuid)
            / SubtensorModule::get_alpha_price(destination_netuid))
            * I96F32::from_num(1_000_000_000))
        .to_num::<u64>()
            - 1_u64;

        // Swap stake limit call
        let call = RuntimeCall::SubtensorModule(SubtensorCall::swap_stake_limit {
            hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount: unstake_amount,
            limit_price,
            allow_partial: false,
        });

        let info: crate::DispatchInfo =
            crate::DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = crate::SubtensorSignedExtension::<Test>::new();
        // Submit to the signed extension validate function
        let result_no_stake = extension.validate(&coldkey, &call.clone(), &info, 10);

        // Should fail due to slippage
        assert_err!(
            result_no_stake,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::SlippageTooHigh.into()
            ))
        );
    });
}

#[test]
fn test_stake_transfers_disabled_validate() {
    // Testing the signed extension validate function
    // correctly filters the `transfer_stake` transaction.

    new_test_ext(0).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let origin_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let destination_netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let destination_coldkey = U256::from(3);
        let stake_amount = 100_000_000_000;

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        let unstake_amount =
            SubtensorModule::stake_into_subnet(&hotkey, &coldkey, origin_netuid, stake_amount, 0);

        // Swap stake limit call
        let call = RuntimeCall::SubtensorModule(SubtensorCall::transfer_stake {
            destination_coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount: unstake_amount,
        });

        let info: crate::DispatchInfo =
            crate::DispatchInfoOf::<<Test as frame_system::Config>::RuntimeCall>::default();

        let extension = crate::SubtensorSignedExtension::<Test>::new();

        // Disable transfers in origin subnet
        TransferToggle::<Test>::insert(origin_netuid, false);
        TransferToggle::<Test>::insert(destination_netuid, true);

        // Submit to the signed extension validate function
        let result1 = extension.validate(&coldkey, &call.clone(), &info, 10);
        assert_err!(
            result1,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::TransferDisallowed.into()
            ))
        );

        // Disable transfers in destination subnet
        TransferToggle::<Test>::insert(origin_netuid, true);
        TransferToggle::<Test>::insert(destination_netuid, false);

        // Submit to the signed extension validate function
        let result2 = extension.validate(&coldkey, &call.clone(), &info, 10);
        assert_err!(
            result2,
            crate::TransactionValidityError::Invalid(crate::InvalidTransaction::Custom(
                CustomTransactionError::TransferDisallowed.into()
            ))
        );

        // Enable transfers
        TransferToggle::<Test>::insert(origin_netuid, true);
        TransferToggle::<Test>::insert(destination_netuid, true);

        // Submit to the signed extension validate function
        let result3 = extension.validate(&coldkey, &call.clone(), &info, 10);
        assert_ok!(result3);
    });
}
