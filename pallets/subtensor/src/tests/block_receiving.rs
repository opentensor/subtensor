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
        // The TAO block check fires in transition_stake_internal when a cross-subnet stake
        // move transfers TAO between different coldkeys. Set up a sender with stake on one
        // subnet and transfer it to a receiver coldkey that blocks TAO.
        let sender_coldkey = U256::from(1);
        let receiver_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let netuid_src = setup_subnet_and_stake(sender_coldkey, hotkey, 1_000_000_000_u64);
        // Second subnet for cross-subnet move
        let owner_ck2 = U256::from(997);
        let owner_hk2 = U256::from(996);
        let netuid_dst = add_dynamic_network(&owner_hk2, &owner_ck2);
        setup_reserves(
            netuid_dst,
            TaoBalance::from(1_000_000_000_000_u64),
            AlphaBalance::from(10_000_000_000_000_u64),
        );

        // Receiver blocks TAO
        assert_ok!(SubtensorModule::set_block_receiving_tao(
            RuntimeOrigin::signed(receiver_coldkey),
            true
        ));

        // Cross-subnet transfer stake to a different coldkey should fail with ReceivingTaoBlocked
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &sender_coldkey,
            netuid_src,
        );
        assert_err!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(sender_coldkey),
                receiver_coldkey,
                hotkey,
                netuid_src,
                netuid_dst,
                alpha,
            ),
            Error::<Test>::ReceivingTaoBlocked
        );
    });
}

#[test]
fn test_block_receiving_tao_allows_transfer_after_clear() {
    new_test_ext(1).execute_with(|| {
        let sender_coldkey = U256::from(1);
        let receiver_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let netuid_src = setup_subnet_and_stake(sender_coldkey, hotkey, 1_000_000_000_u64);
        let owner_ck2 = U256::from(997);
        let owner_hk2 = U256::from(996);
        let netuid_dst = add_dynamic_network(&owner_hk2, &owner_ck2);
        setup_reserves(
            netuid_dst,
            TaoBalance::from(1_000_000_000_000_u64),
            AlphaBalance::from(10_000_000_000_000_u64),
        );

        // Block then unblock TAO
        assert_ok!(SubtensorModule::set_block_receiving_tao(
            RuntimeOrigin::signed(receiver_coldkey),
            true
        ));
        assert_ok!(SubtensorModule::set_block_receiving_tao(
            RuntimeOrigin::signed(receiver_coldkey),
            false
        ));

        // Also enable alpha receiving (required for cross-coldkey cross-subnet transfers)
        assert_ok!(SubtensorModule::set_receiving_alpha_enabled(
            RuntimeOrigin::signed(receiver_coldkey),
            true
        ));

        // Cross-subnet transfer stake should now succeed
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &sender_coldkey,
            netuid_src,
        );
        assert_ok!(SubtensorModule::do_transfer_stake(
            RuntimeOrigin::signed(sender_coldkey),
            receiver_coldkey,
            hotkey,
            netuid_src,
            netuid_dst,
            alpha,
        ));
    });
}

// ============================================================
// Alpha blocking (opt-in)
// ============================================================

#[test]
fn test_receiving_alpha_enabled_set_and_clear() {
    new_test_ext(1).execute_with(|| {
        let account = U256::from(1);
        let origin = RuntimeOrigin::signed(account);

        // Default: flag is off (alpha transfers disabled)
        assert!(!ReceivingAlphaEnabled::<Test>::get(account));

        // Enable
        assert_ok!(SubtensorModule::set_receiving_alpha_enabled(
            origin.clone(),
            true
        ));
        assert!(ReceivingAlphaEnabled::<Test>::get(account));

        // Disable
        assert_ok!(SubtensorModule::set_receiving_alpha_enabled(origin, false));
        assert!(!ReceivingAlphaEnabled::<Test>::get(account));
    });
}

#[test]
fn test_receiving_alpha_disabled_allows_self_staking() {
    new_test_ext(1).execute_with(|| {
        // Even when ReceivingAlphaEnabled is false (default), a coldkey can always
        // stake their own TAO — the opt-in flag only blocks cross-coldkey transfers.
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

        // ReceivingAlphaEnabled is false by default — self-staking should still work
        assert!(!ReceivingAlphaEnabled::<Test>::get(coldkey));
        assert_ok!(SubtensorModule::do_add_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            500_000_000_u64.into(),
        ));
    });
}

#[test]
fn test_receiving_alpha_disabled_blocks_same_subnet_transfer() {
    new_test_ext(1).execute_with(|| {
        let sender_coldkey = U256::from(1);
        let receiver_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let netuid = setup_subnet_and_stake(sender_coldkey, hotkey, 1_000_000_000_u64);

        // receiver_coldkey has NOT enabled receiving alpha (default = disabled)
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &sender_coldkey,
            netuid,
        );
        assert_err!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(sender_coldkey),
                receiver_coldkey,
                hotkey,
                netuid,
                netuid,
                alpha,
            ),
            Error::<Test>::ReceivingAlphaBlocked
        );
    });
}

#[test]
fn test_receiving_alpha_disabled_blocks_cross_subnet_transfer() {
    new_test_ext(1).execute_with(|| {
        let sender_coldkey = U256::from(1);
        let receiver_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let netuid_src = setup_subnet_and_stake(sender_coldkey, hotkey, 1_000_000_000_u64);
        let owner_ck2 = U256::from(997);
        let owner_hk2 = U256::from(996);
        let netuid_dst = add_dynamic_network(&owner_hk2, &owner_ck2);
        setup_reserves(
            netuid_dst,
            TaoBalance::from(1_000_000_000_000_u64),
            AlphaBalance::from(10_000_000_000_000_u64),
        );

        // receiver_coldkey has NOT enabled receiving alpha (default = disabled)
        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &sender_coldkey,
            netuid_src,
        );
        assert_err!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(sender_coldkey),
                receiver_coldkey,
                hotkey,
                netuid_src,
                netuid_dst,
                alpha,
            ),
            Error::<Test>::ReceivingAlphaBlocked
        );
    });
}

#[test]
fn test_receiving_alpha_enabled_allows_cross_subnet_transfer() {
    new_test_ext(1).execute_with(|| {
        let sender_coldkey = U256::from(1);
        let receiver_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        let netuid_src = setup_subnet_and_stake(sender_coldkey, hotkey, 1_000_000_000_u64);
        let owner_ck2 = U256::from(997);
        let owner_hk2 = U256::from(996);
        let netuid_dst = add_dynamic_network(&owner_hk2, &owner_ck2);
        setup_reserves(
            netuid_dst,
            TaoBalance::from(1_000_000_000_000_u64),
            AlphaBalance::from(10_000_000_000_000_u64),
        );

        // receiver_coldkey opts in to receiving alpha
        assert_ok!(SubtensorModule::set_receiving_alpha_enabled(
            RuntimeOrigin::signed(receiver_coldkey),
            true
        ));

        let alpha = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &sender_coldkey,
            netuid_src,
        );
        assert_ok!(SubtensorModule::do_transfer_stake(
            RuntimeOrigin::signed(sender_coldkey),
            receiver_coldkey,
            hotkey,
            netuid_src,
            netuid_dst,
            alpha,
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
