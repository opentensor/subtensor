#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::arithmetic_side_effects
)]

use approx::assert_abs_diff_eq;
use frame_support::weights::Weight;
use frame_support::{assert_noop, assert_ok};
use sp_core::U256;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaBalance, TaoBalance};
use subtensor_swap_interface::SwapHandler;

use super::mock::*;
use crate::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_subnet_with_stake(
    coldkey: U256,
    hotkey: U256,
    stake_tao: u64,
) -> subtensor_runtime_common::NetUid {
    let subnet_owner_coldkey = U256::from(1001);
    let subnet_owner_hotkey = U256::from(1002);
    let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

    let amount: TaoBalance = (stake_tao).into();
    setup_reserves(
        netuid,
        (stake_tao * 1_000_000).into(),
        (stake_tao * 10_000_000).into(),
    );

    SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
    SubtensorModule::stake_into_subnet(
        &hotkey,
        &coldkey,
        netuid,
        amount,
        <Test as Config>::SwapInterface::max_price(),
        false,
        false,
    )
    .unwrap();

    netuid
}

fn get_alpha(
    hotkey: &U256,
    coldkey: &U256,
    netuid: subtensor_runtime_common::NetUid,
) -> AlphaBalance {
    SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid)
}

// =========================================================================
// GROUP 1: Green-path — basic lock creation
// =========================================================================

#[test]
fn test_lock_stake_creates_new_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let alpha = get_alpha(&hotkey, &coldkey, netuid);
        let lock_amount = alpha.to_u64() / 2;

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount.into(),
        ));

        let lock = Lock::<Test>::get(coldkey, netuid).expect("Lock should exist");
        assert_eq!(lock.hotkey, hotkey);
        assert_eq!(lock.locked_mass, lock_amount.into());
        assert_eq!(lock.conviction, U64F64::saturating_from_num(0));
        assert_eq!(
            lock.last_update,
            SubtensorModule::get_current_block_as_u64()
        );
    });
}

#[test]
fn test_lock_stake_emits_event() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let lock_amount: u64 = 1000;

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount.into(),
        ));

        System::assert_last_event(
            Event::StakeLocked {
                coldkey,
                hotkey,
                netuid,
                amount: lock_amount.into(),
            }
            .into(),
        );
    });
}

#[test]
fn test_lock_stake_full_amount() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total_alpha = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert!(!total_alpha.is_zero());

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            total_alpha,
        ));

        let lock = Lock::<Test>::get(coldkey, netuid).unwrap();
        assert_eq!(lock.locked_mass, total_alpha);
    });
}

// =========================================================================
// GROUP 2: Green-path — lock queries
// =========================================================================

#[test]
fn test_get_current_locked_no_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let netuid = subtensor_runtime_common::NetUid::from(1);
        assert_eq!(
            SubtensorModule::get_current_locked(&coldkey, netuid),
            AlphaBalance::ZERO
        );
    });
}

#[test]
fn test_get_conviction_no_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let netuid = subtensor_runtime_common::NetUid::from(1);
        assert_eq!(
            SubtensorModule::get_conviction(&coldkey, netuid),
            U64F64::saturating_from_num(0)
        );
    });
}

#[test]
fn test_available_to_unstake_no_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let available = SubtensorModule::available_to_unstake(&coldkey, netuid);
        assert_eq!(available, total);
    });
}

#[test]
fn test_available_to_unstake_with_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let lock_amount = total / 2.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount,
        ));

        let available = SubtensorModule::available_to_unstake(&coldkey, netuid);
        assert_eq!(available, total - lock_amount);
    });
}

#[test]
fn test_available_to_unstake_fully_locked() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, total,
        ));

        let available = SubtensorModule::available_to_unstake(&coldkey, netuid);
        assert_eq!(available, AlphaBalance::ZERO);
    });
}

// =========================================================================
// GROUP 3: Incremental locks (top-up)
// =========================================================================

#[test]
fn test_lock_stake_topup() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let first_lock = 1000u64;
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            first_lock.into()
        ));

        step_block(100);

        let second_lock = 500u64;
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            second_lock.into()
        ));

        let lock = Lock::<Test>::get(coldkey, netuid).unwrap();
        // locked_mass should be decayed(first_lock) + second_lock
        // Since tau is large (216000), decay over 100 blocks is small; locked_mass ~ 1000 + 500
        assert!(lock.locked_mass > 1490.into());
        assert!(lock.locked_mass < 1501.into());
        // conviction should have grown from the time the first lock was active
        assert!(lock.conviction > U64F64::saturating_from_num(0));
        assert_eq!(
            lock.last_update,
            SubtensorModule::get_current_block_as_u64()
        );
    });
}

#[test]
fn test_lock_stake_topup_multiple_times() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let chunk = 500u64.into();

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, chunk
        ));
        step_block(50);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, chunk
        ));
        step_block(50);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, chunk
        ));

        let lock = Lock::<Test>::get(coldkey, netuid).unwrap();
        // After three top-ups with small decay, should be close to 1500
        assert!(lock.locked_mass > 1490.into());
        assert!(lock.locked_mass <= 1500.into());
        assert!(lock.conviction > U64F64::saturating_from_num(0));
    });
}

#[test]
fn test_lock_stake_topup_same_block() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let first = 1000u64.into();
        let second = 500u64.into();

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, first
        ));
        // No block advancement — same block top-up
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, second
        ));

        let lock = Lock::<Test>::get(coldkey, netuid).unwrap();
        // dt=0 means no decay, simple addition
        assert_eq!(lock.locked_mass, first + second);
        assert_eq!(lock.conviction, U64F64::saturating_from_num(0));
    });
}

// =========================================================================
// GROUP 4: Lock rejection cases
// =========================================================================

#[test]
fn test_lock_stake_zero_amount() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, AlphaBalance::ZERO,),
            Error::<Test>::AmountTooLow
        );
    });
}

#[test]
fn test_lock_stake_exceeds_total_alpha() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let too_much = total + 1.into();

        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, too_much),
            Error::<Test>::InsufficientStakeForLock
        );
    });
}

#[test]
fn test_lock_stake_wrong_hotkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey_a = U256::from(2);
        let hotkey_b = U256::from(3);
        let netuid = setup_subnet_with_stake(coldkey, hotkey_a, 100_000_000_000);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey_a,
            1000u64.into(),
        ));

        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey_b, 500u64.into(),),
            Error::<Test>::LockHotkeyMismatch
        );
    });
}

#[test]
fn test_lock_stake_topup_exceeds_total() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        // Lock 80% initially
        let initial = total * 8.into() / 10.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, initial
        ));

        // Try to top up the remaining 30% (exceeds total by 10%)
        let topup = total * 3.into() / 10.into();
        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, topup),
            Error::<Test>::InsufficientStakeForLock
        );
    });
}

// =========================================================================
// GROUP 5: Exponential decay math
// =========================================================================

#[test]
fn test_exp_decay_zero_dt() {
    new_test_ext(1).execute_with(|| {
        let result = SubtensorModule::exp_decay(0, 216000);
        assert_eq!(result, U64F64::saturating_from_num(1));
    });
}

#[test]
fn test_exp_decay_zero_tau() {
    new_test_ext(1).execute_with(|| {
        let result = SubtensorModule::exp_decay(1000, 0);
        assert_eq!(result, U64F64::saturating_from_num(0));
    });
}

#[test]
fn test_exp_decay_one_tau() {
    new_test_ext(1).execute_with(|| {
        let tau = 216000u64;
        let result = SubtensorModule::exp_decay(tau, tau);
        // exp(-1) ~= 0.36787944
        let expected = U64F64::saturating_from_num(0.36787944f64);
        let diff = if result > expected {
            result - expected
        } else {
            expected - result
        };
        assert!(diff < U64F64::saturating_from_num(0.001));
    });
}

#[test]
fn test_exp_decay_clamps_large_dt_to_min_ratio() {
    new_test_ext(1).execute_with(|| {
        let tau = 216000u64;
        let clamped_result = SubtensorModule::exp_decay(40 * tau, tau);
        let oversized_result = SubtensorModule::exp_decay(100 * tau, tau);

        let diff = if oversized_result > clamped_result {
            oversized_result - clamped_result
        } else {
            clamped_result - oversized_result
        };

        assert!(diff < U64F64::saturating_from_num(0.000000001));
        assert!(oversized_result > U64F64::saturating_from_num(0));
    });
}

#[test]
fn test_roll_forward_locked_mass_decays() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let lock_amount = 10000u64;
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount.into()
        ));

        // Advance one full tau via direct block number jump (step_block overflows u16 for tau=216000)
        let tau = TauBlocks::<Test>::get();
        let target = System::block_number() + tau;
        System::set_block_number(target);

        let locked = SubtensorModule::get_current_locked(&coldkey, netuid);
        // After one tau, locked should be ~36.8% of original
        assert!(locked < lock_amount.into());
        let expected = lock_amount as f64 * 0.368;
        assert_abs_diff_eq!(
            u64::from(locked) as f64,
            expected,
            epsilon = lock_amount as f64 / 10.
        );
    });
}

#[test]
fn test_roll_forward_conviction_grows_then_decays() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let lock_amount = 10000u64.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount
        ));

        // Conviction at t=0 is 0
        let c0 = SubtensorModule::get_conviction(&coldkey, netuid);
        assert_eq!(c0, U64F64::saturating_from_num(0));

        // After some time, conviction should have grown
        step_block(1000);
        let c1 = SubtensorModule::get_conviction(&coldkey, netuid);
        assert!(c1 > U64F64::saturating_from_num(0));

        // After more time, conviction should be even higher
        step_block(1000);
        let c2 = SubtensorModule::get_conviction(&coldkey, netuid);
        assert!(c2 > c1);

        // After a very long time (many taus), conviction starts to decay back
        // because locked_mass has mostly decayed away
        let tau = TauBlocks::<Test>::get();
        let target = System::block_number() + tau * 10;
        System::set_block_number(target);
        let c_late = SubtensorModule::get_conviction(&coldkey, netuid);
        assert!(c_late < c2);
    });
}

#[test]
fn test_roll_forward_no_change_when_now_equals_last_update() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(2);
        let lock = LockState {
            hotkey,
            locked_mass: 5000.into(),
            conviction: U64F64::saturating_from_num(1234),
            last_update: 100,
        };
        let rolled = SubtensorModule::roll_forward_lock(lock.clone(), 100);
        assert_eq!(rolled.locked_mass, lock.locked_mass);
        assert_eq!(rolled.conviction, lock.conviction);
        assert_eq!(rolled.last_update, 100);
    });
}

// =========================================================================
// GROUP 6: Unstake invariant enforcement
// =========================================================================

#[test]
fn test_unstake_allowed_when_no_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let alpha = get_alpha(&hotkey, &coldkey, netuid);
        assert!(alpha > AlphaBalance::ZERO);

        assert_ok!(SubtensorModule::do_remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            alpha,
        ));
    });
}

#[test]
fn test_unstake_allowed_up_to_available() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let lock_amount = total / 2.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount
        ));

        // Unstake the unlocked half
        let alpha = get_alpha(&hotkey, &coldkey, netuid);
        let available_alpha: u64 = (alpha.to_u64()) / 2;
        // Need to step a block to pass rate limiter
        step_block(1);
        assert_ok!(SubtensorModule::do_remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            available_alpha.into(),
        ));
    });
}

#[test]
fn test_unstake_blocked_by_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        // Lock the entire amount
        assert_ok!(SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, total));

        step_block(1);

        let alpha = get_alpha(&hotkey, &coldkey, netuid);
        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                alpha,
            ),
            Error::<Test>::CannotUnstakeLock
        );
    });
}

#[test]
fn test_unstake_allowed_after_decay() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, total
        ));

        // Advance many taus so lock decays to near-zero (use set_block_number to avoid u16 overflow)
        let tau = TauBlocks::<Test>::get();
        let target = System::block_number() + tau * 50;
        System::set_block_number(target);
        // Step one block to clear rate limiter state from on_finalize
        step_block(1);

        // Lock should have decayed to near zero
        let locked = SubtensorModule::get_current_locked(&coldkey, netuid);
        assert!(locked.is_zero());

        // Should now be able to unstake (subtract 1 to avoid U64F64/AlphaBalance rounding edge)
        let alpha = get_alpha(&hotkey, &coldkey, netuid);
        if alpha > 1.into() {
            assert_ok!(SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                alpha.saturating_sub(1.into()),
            ));
        }
    });
}

#[test]
fn test_unstake_partial_after_partial_decay() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, total
        ));

        // Advance one tau: lock ~ 37% of original
        let tau = TauBlocks::<Test>::get();
        let target = System::block_number() + tau;
        System::set_block_number(target);

        let locked_now = SubtensorModule::get_current_locked(&coldkey, netuid);
        let total_now = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert!(total_now > locked_now);

        // Unstake up to the available amount
        let available = total_now - locked_now;
        let unstake_amount: u64 = u64::from(available);
        if unstake_amount > 0 {
            assert_ok!(SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                unstake_amount.into(),
            ));

            // Verify remaining alpha is still >= locked
            let remaining = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
            let locked_after = SubtensorModule::get_current_locked(&coldkey, netuid);
            assert!(remaining >= locked_after);
        }
    });
}

// =========================================================================
// GROUP 7: Move/transfer invariant enforcement
// =========================================================================

#[test]
fn test_move_stake_same_coldkey_same_subnet_allowed() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey_a = U256::from(2);
        let hotkey_b = U256::from(3);
        let netuid = setup_subnet_with_stake(coldkey, hotkey_a, 100_000_000_000);

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey_b);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        // Lock the full amount to hotkey_a
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey_a, total
        ));

        // Move from hotkey_a to hotkey_b on same subnet — total coldkey alpha unchanged
        let alpha = get_alpha(&hotkey_a, &coldkey, netuid);
        let move_amount = alpha / 2.into();
        assert_ok!(SubtensorModule::do_move_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey_a,
            hotkey_b,
            netuid,
            netuid,
            move_amount,
        ));
    });
}

#[test]
fn test_move_stake_cross_subnet_blocked_by_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid_a = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let subnet_owner2_ck = U256::from(2001);
        let subnet_owner2_hk = U256::from(2002);
        let netuid_b = add_dynamic_network(&subnet_owner2_hk, &subnet_owner2_ck);
        setup_reserves(
            netuid_b,
            (100_000_000_000u64 * 1_000_000).into(),
            (100_000_000_000u64 * 10_000_000).into(),
        );

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid_a);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid_a, &hotkey, total
        ));

        step_block(1);

        let alpha = get_alpha(&hotkey, &coldkey, netuid_a);
        assert_noop!(
            SubtensorModule::do_move_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                hotkey,
                netuid_a,
                netuid_b,
                alpha,
            ),
            Error::<Test>::CannotUnstakeLock
        );
    });
}

#[test]
fn test_transfer_stake_cross_coldkey_blocked_by_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey_sender = U256::from(1);
        let coldkey_receiver = U256::from(5);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey_sender, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey_sender, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey_sender,
            netuid,
            &hotkey,
            total,
        ));

        step_block(1);

        let alpha = get_alpha(&hotkey, &coldkey_sender, netuid);
        assert_noop!(
            SubtensorModule::do_transfer_stake(
                RuntimeOrigin::signed(coldkey_sender),
                coldkey_receiver,
                hotkey,
                netuid,
                netuid,
                alpha,
            ),
            Error::<Test>::CannotUnstakeLock
        );
    });
}

#[test]
fn test_transfer_stake_cross_coldkey_allowed_partial() {
    new_test_ext(1).execute_with(|| {
        let coldkey_sender = U256::from(1);
        let coldkey_receiver = U256::from(5);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey_sender, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey_sender, netuid);
        let lock_half = total / 2.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey_sender,
            netuid,
            &hotkey,
            lock_half,
        ));

        step_block(1);

        // Transfer the unlocked portion
        let alpha = get_alpha(&hotkey, &coldkey_sender, netuid);
        let transfer_amount = alpha / 4.into(); // well within the unlocked half
        assert_ok!(SubtensorModule::do_transfer_stake(
            RuntimeOrigin::signed(coldkey_sender),
            coldkey_receiver,
            hotkey,
            netuid,
            netuid,
            transfer_amount,
        ));
    });
}

// =========================================================================
// GROUP 8: Multi-subnet locks
// =========================================================================

#[test]
fn test_lock_on_multiple_subnets() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey_a = U256::from(2);
        let hotkey_b = U256::from(3);

        let netuid_a = setup_subnet_with_stake(coldkey, hotkey_a, 100_000_000_000);

        let subnet_owner2_ck = U256::from(2001);
        let subnet_owner2_hk = U256::from(2002);
        let netuid_b = add_dynamic_network(&subnet_owner2_hk, &subnet_owner2_ck);
        setup_reserves(
            netuid_b,
            (100_000_000_000u64 * 1_000_000).into(),
            (100_000_000_000u64 * 10_000_000).into(),
        );
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey_b);
        SubtensorModule::stake_into_subnet(
            &hotkey_b,
            &coldkey,
            netuid_b,
            100_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        // Lock on subnet A to hotkey_a
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid_a,
            &hotkey_a,
            1000u64.into(),
        ));

        // Lock on subnet B to hotkey_b (different hotkey is fine — different subnet)
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid_b,
            &hotkey_b,
            2000u64.into(),
        ));

        let lock_a = Lock::<Test>::get(coldkey, netuid_a).unwrap();
        let lock_b = Lock::<Test>::get(coldkey, netuid_b).unwrap();
        assert_eq!(lock_a.hotkey, hotkey_a);
        assert_eq!(lock_b.hotkey, hotkey_b);
        assert_eq!(lock_a.locked_mass, 1000u64.into());
        assert_eq!(lock_b.locked_mass, 2000u64.into());
    });
}

#[test]
fn test_unstake_one_subnet_does_not_affect_other() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid_a = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Lock on subnet A
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid_a,
            &hotkey,
            5000u64.into(),
        ));

        // Subnet B — no lock, just stake
        let subnet_owner2_ck = U256::from(2001);
        let subnet_owner2_hk = U256::from(2002);
        let netuid_b = add_dynamic_network(&subnet_owner2_hk, &subnet_owner2_ck);
        setup_reserves(
            netuid_b,
            (100_000_000_000u64 * 1_000_000).into(),
            (100_000_000_000u64 * 10_000_000).into(),
        );
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid_b,
            100_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        step_block(1);

        // Unstake from subnet B — should succeed (no lock there)
        let alpha_b = get_alpha(&hotkey, &coldkey, netuid_b);
        assert_ok!(SubtensorModule::do_remove_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid_b,
            alpha_b,
        ));

        // Lock on subnet A unaffected
        let lock_a = Lock::<Test>::get(coldkey, netuid_a).unwrap();
        assert_eq!(lock_a.locked_mass, 5000u64.into());
    });
}

// =========================================================================
// GROUP 9: Hotkey conviction and subnet king
// =========================================================================

#[test]
fn test_hotkey_conviction_single_locker() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            5000u64.into(),
        ));

        // Initially conviction is 0 (just created)
        let c = SubtensorModule::hotkey_conviction(&hotkey, netuid);
        assert_eq!(c, U64F64::saturating_from_num(0));

        // After time, conviction grows
        step_block(1000);
        let c = SubtensorModule::hotkey_conviction(&hotkey, netuid);
        assert!(c > U64F64::saturating_from_num(0));
    });
}

#[test]
fn test_hotkey_conviction_multiple_lockers() {
    new_test_ext(1).execute_with(|| {
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(5);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey1, hotkey, 100_000_000_000);

        // Also give coldkey2 stake on same hotkey
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 100_000_000_000u64.into());
        SubtensorModule::create_account_if_non_existent(&coldkey2, &hotkey);
        SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey2,
            netuid,
            50_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey1,
            netuid,
            &hotkey,
            3000u64.into(),
        ));
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey2,
            netuid,
            &hotkey,
            2000u64.into(),
        ));

        step_block(500);

        let total_conviction = SubtensorModule::hotkey_conviction(&hotkey, netuid);
        let c1 = SubtensorModule::get_conviction(&coldkey1, netuid);
        let c2 = SubtensorModule::get_conviction(&coldkey2, netuid);

        // Total conviction should be approximately sum of individual convictions
        let diff = if total_conviction > (c1 + c2) {
            total_conviction - (c1 + c2)
        } else {
            (c1 + c2) - total_conviction
        };
        assert!(diff < U64F64::saturating_from_num(1));
    });
}

#[test]
fn test_subnet_king_single_hotkey() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            5000u64.into(),
        ));

        step_block(100);

        let king = SubtensorModule::subnet_king(netuid);
        assert_eq!(king, Some(hotkey));
    });
}

#[test]
fn test_subnet_king_highest_conviction_wins() {
    new_test_ext(1).execute_with(|| {
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(5);
        let hotkey_a = U256::from(2);
        let hotkey_b = U256::from(3);

        let netuid = setup_subnet_with_stake(coldkey1, hotkey_a, 100_000_000_000);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, 100_000_000_000u64.into());
        SubtensorModule::create_account_if_non_existent(&coldkey2, &hotkey_b);
        SubtensorModule::stake_into_subnet(
            &hotkey_b,
            &coldkey2,
            netuid,
            50_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        // coldkey1 locks more to hotkey_a
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey1,
            netuid,
            &hotkey_a,
            8000u64.into(),
        ));
        // coldkey2 locks less to hotkey_b
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey2,
            netuid,
            &hotkey_b,
            2000u64.into(),
        ));

        step_block(500);

        let king = SubtensorModule::subnet_king(netuid);
        assert_eq!(king, Some(hotkey_a));
    });
}

#[test]
fn test_subnet_king_no_locks() {
    new_test_ext(1).execute_with(|| {
        let netuid = subtensor_runtime_common::NetUid::from(99);
        let king = SubtensorModule::subnet_king(netuid);
        assert_eq!(king, None);
    });
}

// =========================================================================
// GROUP 10: Lock cleanup
// =========================================================================

#[test]
fn test_maybe_cleanup_lock_removes_dust() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Lock a small amount
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            50u64.into(),
        ));

        // Advance many taus so everything decays well below dust (100)
        let tau = TauBlocks::<Test>::get();
        let target = System::block_number() + tau * 50;
        System::set_block_number(target);

        SubtensorModule::maybe_cleanup_lock(&coldkey, netuid);

        assert!(Lock::<Test>::get(coldkey, netuid).is_none());
    });
}

#[test]
fn test_maybe_cleanup_lock_no_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let netuid = subtensor_runtime_common::NetUid::from(1);
        // Should be a no-op, no panic
        SubtensorModule::maybe_cleanup_lock(&coldkey, netuid);
        assert!(Lock::<Test>::get(coldkey, netuid).is_none());
    });
}

// =========================================================================
// GROUP 11: Coldkey swap interaction
// =========================================================================

#[test]
fn test_coldkey_swap_orphans_lock() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(10);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(old_coldkey, hotkey, 100_000_000_000);

        assert_ok!(SubtensorModule::do_lock_stake(
            &old_coldkey,
            netuid,
            &hotkey,
            5000u64.into(),
        ));

        // Perform coldkey swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey));

        // Lock remains on old coldkey (orphaned)
        assert!(Lock::<Test>::get(old_coldkey, netuid).is_some());
        // New coldkey has no lock
        assert!(Lock::<Test>::get(new_coldkey, netuid).is_none());
    });
}

#[test]
fn test_coldkey_swap_lock_no_longer_blocks_unstake() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(10);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(old_coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&old_coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &old_coldkey,
            netuid,
            &hotkey,
            total,
        ));

        // Swap coldkey
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey));

        step_block(1);

        // New coldkey should be able to unstake freely — no lock on new_coldkey
        let alpha = get_alpha(&hotkey, &new_coldkey, netuid);
        if alpha > AlphaBalance::ZERO {
            assert_ok!(SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(new_coldkey),
                hotkey,
                netuid,
                alpha,
            ));
        }
    });
}

// =========================================================================
// GROUP 12: Hotkey swap interaction
// =========================================================================

#[test]
fn test_hotkey_swap_lock_becomes_stale() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let old_hotkey = U256::from(2);
        let new_hotkey = U256::from(20);
        let netuid = setup_subnet_with_stake(coldkey, old_hotkey, 100_000_000_000);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &old_hotkey,
            5000u64.into(),
        ));

        // Perform hotkey swap
        let mut weight = Weight::zero();
        assert_ok!(SubtensorModule::perform_hotkey_swap_on_all_subnets(
            &old_hotkey,
            &new_hotkey,
            &coldkey,
            &mut weight,
            false
        ));

        // Lock still references old_hotkey
        let lock = Lock::<Test>::get(coldkey, netuid).unwrap();
        assert_eq!(lock.hotkey, old_hotkey);

        // Trying to top up to new_hotkey fails with mismatch
        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &new_hotkey, 100u64.into(),),
            Error::<Test>::LockHotkeyMismatch
        );
    });
}

#[test]
fn test_hotkey_swap_conviction_not_migrated() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let old_hotkey = U256::from(2);
        let new_hotkey = U256::from(20);
        let netuid = setup_subnet_with_stake(coldkey, old_hotkey, 100_000_000_000);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &old_hotkey,
            5000u64.into(),
        ));

        step_block(500);
        let conviction_before = SubtensorModule::hotkey_conviction(&old_hotkey, netuid);
        assert!(conviction_before > U64F64::saturating_from_num(0));

        // Swap hotkey
        let mut weight = Weight::zero();
        assert_ok!(SubtensorModule::perform_hotkey_swap_on_all_subnets(
            &old_hotkey,
            &new_hotkey,
            &coldkey,
            &mut weight,
            false
        ));

        // New hotkey has no conviction
        let conviction_new = SubtensorModule::hotkey_conviction(&new_hotkey, netuid);
        assert_eq!(conviction_new, U64F64::saturating_from_num(0));

        // Old hotkey still has conviction (lock still points there)
        let conviction_old = SubtensorModule::hotkey_conviction(&old_hotkey, netuid);
        assert!(conviction_old > U64F64::saturating_from_num(0));
    });
}

// =========================================================================
// GROUP 13: Lock extrinsic via dispatch
// =========================================================================

#[test]
fn test_lock_stake_extrinsic() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let lock_amount: u64 = 5000;
        assert_ok!(SubtensorModule::lock_stake(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            netuid,
            lock_amount.into(),
        ));

        let lock = Lock::<Test>::get(coldkey, netuid).expect("Lock should exist");
        assert_eq!(lock.hotkey, hotkey);
        assert_eq!(lock.locked_mass, lock_amount.into());
        assert_eq!(lock.conviction, U64F64::saturating_from_num(0));
    });
}

// =========================================================================
// GROUP 14: Recycle/burn alpha checks against lock
// =========================================================================

#[test]
fn test_recycle_alpha_checks_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, total));

        step_block(1);

        // Unstake should be blocked
        let alpha = get_alpha(&hotkey, &coldkey, netuid);
        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                alpha,
            ),
            Error::<Test>::CannotUnstakeLock
        );

        // recycle_alpha checks lock and should fail if it would reduce alpha below locked amount
        let recycle_amount = alpha / 2.into();
        assert_noop!(
            SubtensorModule::do_recycle_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                recycle_amount,
                netuid,
            ),
            Error::<Test>::CannotUnstakeLock
        );

        // Alpha is not below locked_mass
        let total_after = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let locked = SubtensorModule::get_current_locked(&coldkey, netuid);
        assert!(total_after >= locked);
    });
}

#[test]
fn test_burn_alpha_checks_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, total
        ));

        step_block(1);

        // burn_alpha checks lock and should fail if it would reduce alpha below locked amount
        let alpha = get_alpha(&hotkey, &coldkey, netuid);
        let burn_amount = alpha / 2.into();
        assert_noop!(
            SubtensorModule::do_burn_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                burn_amount,
                netuid,
            ),
            Error::<Test>::CannotUnstakeLock
        );

        // Alpha is not below locked_mass
        let total_after = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let locked = SubtensorModule::get_current_locked(&coldkey, netuid);
        assert!(total_after >= locked);
    });
}

// =========================================================================
// GROUP 15: Subnet dissolution
// =========================================================================

#[test]
fn test_subnet_dissolution_orphans_locks() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            5000u64.into(),
        ));
        assert!(Lock::<Test>::get(coldkey, netuid).is_some());

        // Dissolve the subnet
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        // All Alpha entries are gone
        assert_eq!(
            SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid),
            AlphaBalance::ZERO
        );

        // BUG: Lock entry is orphaned — still present despite no alpha
        assert!(Lock::<Test>::get(coldkey, netuid).is_some());
    });
}

#[test]
fn test_subnet_dissolution_and_netuid_reuse() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey_old = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey_old, 100_000_000_000);

        // Lock on the old subnet
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey_old,
            5000u64.into(),
        ));

        // Dissolve old subnet
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        // The stale lock from old subnet remains
        let stale_lock = Lock::<Test>::get(coldkey, netuid);
        assert!(stale_lock.is_some());
        assert_eq!(stale_lock.unwrap().hotkey, hotkey_old);
    });
}

// =========================================================================
// GROUP 16: Clear small nomination checks lock
// =========================================================================

#[test]
fn test_clear_small_nomination_checks_lock() {
    new_test_ext(1).execute_with(|| {
        let owner_coldkey = U256::from(100);
        let owner_hotkey = U256::from(101);
        let netuid = setup_subnet_with_stake(owner_coldkey, owner_hotkey, 100_000_000_000);

        // Set up a nominator (different coldkey, does NOT own the hotkey)
        let nominator = U256::from(200);
        SubtensorModule::add_balance_to_coldkey_account(&nominator, 100_000_000_000u64.into());
        SubtensorModule::create_account_if_non_existent(&nominator, &owner_hotkey);
        SubtensorModule::stake_into_subnet(
            &owner_hotkey,
            &nominator,
            netuid,
            50_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        let nominator_alpha = get_alpha(&owner_hotkey, &nominator, netuid);
        assert!(nominator_alpha > AlphaBalance::ZERO);

        // Nominator locks their full stake
        let nominator_total = SubtensorModule::total_coldkey_alpha_on_subnet(&nominator, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &nominator,
            netuid,
            &owner_hotkey,
            nominator_total,
        ));

        // Set a high nominator min stake so the current stake is "small"
        SubtensorModule::set_nominator_min_required_stake(u64::MAX);

        // BUG: clear_small_nomination bypasses the lock and removes alpha
        SubtensorModule::clear_small_nomination_if_required(&owner_hotkey, &nominator, netuid);

        // Nominator alpha has been removed despite lock
        let nominator_alpha_after = get_alpha(&owner_hotkey, &nominator, netuid);
        assert_eq!(nominator_alpha_after, AlphaBalance::ZERO);

        // Lock entry still exists, now orphaned
        assert!(Lock::<Test>::get(nominator, netuid).is_none());
    });
}

// =========================================================================
// GROUP 17: Emission interaction
// =========================================================================

#[test]
fn test_emissions_do_not_break_lock_invariant() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let total_alpha_before = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            total_alpha_before
        ));

        // Simulate emission: directly increase alpha for the hotkey on subnet
        // This increases the pool value for all share holders (including our coldkey)
        let emission_amount: AlphaBalance = 10_000_000u64.into();
        SubtensorModule::increase_stake_for_hotkey_on_subnet(&hotkey, netuid, emission_amount);

        // After emission, total alpha should increase by emission_amount
        let total_alpha_after = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_eq!(total_alpha_after, total_alpha_before + emission_amount);

        // Lock invariant still holds: total_alpha >= locked_mass
        let locked = SubtensorModule::get_current_locked(&coldkey, netuid);
        assert!(total_alpha_after >= locked);

        // Available becomes emission_amount
        let available = SubtensorModule::available_to_unstake(&coldkey, netuid);
        assert_eq!(available, emission_amount);
    });
}

// =========================================================================
// GROUP 18: Neuron replacement
// =========================================================================

#[test]
fn test_neuron_replacement_does_not_affect_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Register the hotkey as a neuron
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        let lock_amount = 5000u64.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount
        ));

        let total_before = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let locked_before = SubtensorModule::get_current_locked(&coldkey, netuid);

        // Replace the neuron with a different hotkey
        let new_hotkey = U256::from(99);
        let uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).unwrap();
        SubtensorModule::replace_neuron(
            netuid,
            uid,
            &new_hotkey,
            SubtensorModule::get_current_block_as_u64(),
        );

        // Alpha and lock should be unaffected by neuron replacement
        let total_after = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let locked_after = SubtensorModule::get_current_locked(&coldkey, netuid);

        assert_eq!(total_after, total_before);
        assert_eq!(locked_after, locked_before);

        // Lock still references original hotkey
        let lock = Lock::<Test>::get(coldkey, netuid).unwrap();
        assert_eq!(lock.hotkey, hotkey);
    });
}
