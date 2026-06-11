#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
use frame_support::{assert_err, assert_ok};
use sp_core::U256;
use subtensor_runtime_common::{AlphaBalance, TaoBalance};
use subtensor_swap_interface::SwapHandler;

use super::mock::*;
use crate::*;

fn setup_subnet_and_stake(
    coldkey: U256,
    hotkey: U256,
    stake_tao: u64,
) -> subtensor_runtime_common::NetUid {
    let owner_coldkey = U256::from(999);
    let owner_hotkey = U256::from(998);
    let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
    setup_reserves(
        netuid,
        (stake_tao * 1_000_000).into(),
        (stake_tao * 10_000_000).into(),
    );
    assert_ok!(SubtensorModule::create_account_if_non_existent(
        &coldkey, &hotkey
    ));
    add_balance_to_coldkey_account(&coldkey, stake_tao.into());
    assert_ok!(SubtensorModule::stake_into_subnet(
        &hotkey,
        &coldkey,
        netuid,
        stake_tao.into(),
        <Test as Config>::SwapInterface::max_price(),
        false,
    ));
    netuid
}

// ============================================================
// TAO blocking
// ============================================================

#[test]
fn test_block_receiving_tao_set_and_clear() {
    new_test_ext(1).execute_with(|| {
        let account = U256::from(1);
        let origin = RuntimeOrigin::signed(account);

        // Default: flag is off
        assert!(!BlockReceivingTao::<Test>::get(account));

        // Enable the flag
        assert_ok!(SubtensorModule::set_block_receiving_tao(
            origin.clone(),
            true
        ));
        assert!(BlockReceivingTao::<Test>::get(account));

        // Disable the flag
        assert_ok!(SubtensorModule::set_block_receiving_tao(origin, false));
        assert!(!BlockReceivingTao::<Test>::get(account));
    });
}

#[test]
fn test_block_receiving_tao_prevents_transfer() {
    new_test_ext(1).execute_with(|| {
        let sender = U256::from(1);
        let receiver = U256::from(2);

        // Give sender some balance
        add_balance_to_coldkey_account(&sender, 1_000_000_000.into());

        // Receiver blocks TAO
        assert_ok!(SubtensorModule::set_block_receiving_tao(
            RuntimeOrigin::signed(receiver),
            true
        ));

        // Transfer should fail
        assert_err!(
            SubtensorModule::transfer_tao(&sender, &receiver, 100_000_u64.into()),
            Error::<Test>::ReceivingTaoBlocked
        );
    });
}

#[test]
fn test_block_receiving_tao_allows_transfer_after_clear() {
    new_test_ext(1).execute_with(|| {
        let sender = U256::from(1);
        let receiver = U256::from(2);

        add_balance_to_coldkey_account(&sender, 1_000_000_000.into());

        // Block then unblock
        assert_ok!(SubtensorModule::set_block_receiving_tao(
            RuntimeOrigin::signed(receiver),
            true
        ));
        assert_ok!(SubtensorModule::set_block_receiving_tao(
            RuntimeOrigin::signed(receiver),
            false
        ));

        // Transfer should now succeed
        assert_ok!(SubtensorModule::transfer_tao(
            &sender,
            &receiver,
            100_000_u64.into()
        ));
    });
}

// ============================================================
// Alpha blocking
// ============================================================

#[test]
fn test_block_receiving_alpha_set_and_clear() {
    new_test_ext(1).execute_with(|| {
        let account = U256::from(1);
        let origin = RuntimeOrigin::signed(account);

        assert!(!BlockReceivingAlpha::<Test>::get(account));

        assert_ok!(SubtensorModule::set_block_receiving_alpha(
            origin.clone(),
            true
        ));
        assert!(BlockReceivingAlpha::<Test>::get(account));

        assert_ok!(SubtensorModule::set_block_receiving_alpha(origin, false));
        assert!(!BlockReceivingAlpha::<Test>::get(account));
    });
}

#[test]
fn test_block_receiving_alpha_prevents_stake_into_subnet() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let owner_coldkey = U256::from(999);
        let owner_hotkey = U256::from(998);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        setup_reserves(
            netuid,
            TaoBalance::from(1_000_000_000_000_u64),
            AlphaBalance::from(10_000_000_000_000_u64),
        );
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey, &hotkey
        ));
        add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_u64.into());

        // Block receiving alpha
        assert_ok!(SubtensorModule::set_block_receiving_alpha(
            RuntimeOrigin::signed(coldkey),
            true
        ));

        // Staking should fail
        assert_err!(
            SubtensorModule::stake_into_subnet(
                &hotkey,
                &coldkey,
                netuid,
                500_000_000_u64.into(),
                <Test as Config>::SwapInterface::max_price(),
                false,
            ),
            Error::<Test>::ReceivingAlphaBlocked
        );
    });
}

#[test]
fn test_block_receiving_alpha_allows_stake_after_clear() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let owner_coldkey = U256::from(999);
        let owner_hotkey = U256::from(998);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        setup_reserves(
            netuid,
            TaoBalance::from(1_000_000_000_000_u64),
            AlphaBalance::from(10_000_000_000_000_u64),
        );
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey, &hotkey
        ));
        add_balance_to_coldkey_account(&coldkey, 1_000_000_000_000_u64.into());

        // Block then unblock
        assert_ok!(SubtensorModule::set_block_receiving_alpha(
            RuntimeOrigin::signed(coldkey),
            true
        ));
        assert_ok!(SubtensorModule::set_block_receiving_alpha(
            RuntimeOrigin::signed(coldkey),
            false
        ));

        // Staking should now succeed
        assert_ok!(SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid,
            500_000_000_u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
        ));
    });
}

// ============================================================
// Locked Alpha blocking
// ============================================================

#[test]
fn test_block_receiving_locked_alpha_set_and_clear() {
    new_test_ext(1).execute_with(|| {
        let account = U256::from(1);
        let origin = RuntimeOrigin::signed(account);

        assert!(!BlockReceivingLockedAlpha::<Test>::get(account));

        assert_ok!(SubtensorModule::set_block_receiving_locked_alpha(
            origin.clone(),
            true
        ));
        assert!(BlockReceivingLockedAlpha::<Test>::get(account));

        assert_ok!(SubtensorModule::set_block_receiving_locked_alpha(
            origin, false
        ));
        assert!(!BlockReceivingLockedAlpha::<Test>::get(account));
    });
}

#[test]
fn test_block_receiving_locked_alpha_prevents_transfer_lock() {
    new_test_ext(1).execute_with(|| {
        let origin_coldkey = U256::from(1);
        let dest_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let netuid = setup_subnet_and_stake(origin_coldkey, hotkey, 1_000_000_000_u64);

        // Get all alpha staked and lock it all, so any transfer includes locked alpha.
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &origin_coldkey,
            netuid,
        );
        assert!(alpha > AlphaBalance::ZERO, "alpha should be positive");

        // Lock all alpha so that a transfer of any amount must include locked alpha.
        assert_ok!(SubtensorModule::do_lock_stake(
            &origin_coldkey,
            netuid,
            &hotkey,
            alpha,
        ));

        // Destination coldkey blocks receiving locked alpha
        assert_ok!(SubtensorModule::set_block_receiving_locked_alpha(
            RuntimeOrigin::signed(dest_coldkey),
            true
        ));

        // Transfer any positive amount of locked stake should fail
        assert_err!(
            SubtensorModule::transfer_lock(&origin_coldkey, &dest_coldkey, netuid, alpha),
            Error::<Test>::ReceivingLockedAlphaBlocked
        );
    });
}

#[test]
fn test_block_receiving_locked_alpha_allows_transfer_after_clear() {
    new_test_ext(1).execute_with(|| {
        let origin_coldkey = U256::from(1);
        let dest_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let netuid = setup_subnet_and_stake(origin_coldkey, hotkey, 1_000_000_000_u64);

        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &origin_coldkey,
            netuid,
        );

        // Lock all alpha so the transfer involves locked alpha.
        assert_ok!(SubtensorModule::do_lock_stake(
            &origin_coldkey,
            netuid,
            &hotkey,
            alpha,
        ));

        // Block then unblock
        assert_ok!(SubtensorModule::set_block_receiving_locked_alpha(
            RuntimeOrigin::signed(dest_coldkey),
            true
        ));
        assert_ok!(SubtensorModule::set_block_receiving_locked_alpha(
            RuntimeOrigin::signed(dest_coldkey),
            false
        ));

        // Transfer should now succeed
        assert_ok!(SubtensorModule::transfer_lock(
            &origin_coldkey,
            &dest_coldkey,
            netuid,
            alpha
        ));
    });
}
