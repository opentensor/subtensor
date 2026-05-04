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

    assert_ok!(SubtensorModule::create_account_if_non_existent(
        &coldkey, &hotkey
    ));
    add_balance_to_coldkey_account(&coldkey, amount);
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

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).expect("Lock should exist");
        assert_eq!(lock.locked_mass, lock_amount.into());
        assert_eq!(lock.conviction, U64F64::from_num(0));
        assert_eq!(
            lock.last_update,
            SubtensorModule::get_current_block_as_u64()
        );

        // Hotkey lock should also be created
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey);
        assert!(hotkey_lock.is_some());
        assert_eq!(hotkey_lock.unwrap().locked_mass, lock_amount.into());
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

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).unwrap();
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
            U64F64::from_num(0)
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
        let available = SubtensorModule::available_stake(&coldkey, netuid);
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

        let available = SubtensorModule::available_stake(&coldkey, netuid);
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

        let available = SubtensorModule::available_stake(&coldkey, netuid);
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

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).unwrap();
        // locked_mass should be decayed(first_lock) + second_lock
        // Since tau is large (216000), decay over 100 blocks is small; locked_mass ~ 1000 + 500
        assert!(lock.locked_mass > 1490.into());
        assert!(lock.locked_mass < 1501.into());
        // conviction should have grown from the time the first lock was active
        assert!(lock.conviction > U64F64::from_num(0));
        assert_eq!(
            lock.last_update,
            SubtensorModule::get_current_block_as_u64()
        );

        // Hotkey lock should also be created
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey).unwrap();
        assert!(hotkey_lock.locked_mass > 1490.into());
        assert_eq!(hotkey_lock.locked_mass, lock.locked_mass);
        assert!(hotkey_lock.conviction > U64F64::from_num(0));
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

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).unwrap();
        // After three top-ups with small decay, should be close to 1500
        assert!(lock.locked_mass > 1490.into());
        assert!(lock.locked_mass <= 1500.into());
        assert!(lock.conviction > U64F64::from_num(0));

        // Hotkey lock should also be updated
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey).unwrap();
        assert!(hotkey_lock.locked_mass > 1490.into());
        assert_eq!(hotkey_lock.locked_mass, lock.locked_mass);
        assert!(hotkey_lock.conviction > U64F64::from_num(0));
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

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).unwrap();
        // dt=0 means no decay, simple addition
        assert_eq!(lock.locked_mass, first + second);
        assert_eq!(lock.conviction, U64F64::from_num(0));

        // Hotkey lock should also be updated
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey).unwrap();
        assert_eq!(hotkey_lock.locked_mass, first + second);
        assert_eq!(hotkey_lock.conviction, U64F64::from_num(0));
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
        assert_eq!(result, U64F64::from_num(1));
    });
}

#[test]
fn test_exp_decay_zero_tau() {
    new_test_ext(1).execute_with(|| {
        let result = SubtensorModule::exp_decay(1000, 0);
        assert_eq!(result, U64F64::from_num(0));
    });
}

#[test]
fn test_exp_decay_one_tau() {
    new_test_ext(1).execute_with(|| {
        let tau = 216000u64;
        let result = SubtensorModule::exp_decay(tau, tau);
        // exp(-1) ~= 0.36787944
        let expected = U64F64::from_num(0.36787944f64);
        let diff = if result > expected {
            result - expected
        } else {
            expected - result
        };
        assert!(diff < U64F64::from_num(0.001));
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

        assert!(diff < U64F64::from_num(0.000000001));
        assert!(oversized_result > U64F64::from_num(0));
    });
}

#[test]
fn test_roll_forward_locked_mass_no_change() {
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
        let tau = MaturityRate::<Test>::get();
        let target = System::block_number() + tau;
        System::set_block_number(target);

        let locked = SubtensorModule::get_current_locked(&coldkey, netuid);

        // No changes to locked mass
        assert_eq!(locked, lock_amount.into());
    });
}

#[test]
fn test_roll_forward_conviction_converges_to_lock() {
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
        assert_eq!(c0, U64F64::from_num(0));

        // After some time, conviction should have grown
        step_block(100);
        let c1 = SubtensorModule::get_conviction(&coldkey, netuid);
        assert!(c1 > U64F64::from_num(0));

        // After more time, conviction should be even higher
        step_block(1000);
        let c2 = SubtensorModule::get_conviction(&coldkey, netuid);
        println!("c1 = {}", c1);
        println!("c2 = {}", c2);
        // assert!(c2 > c1);

        // After a very long time (many taus), conviction is close to lock amount
        let tau = MaturityRate::<Test>::get();
        let target = System::block_number() + tau * 1000;
        System::set_block_number(target);
        let c_late = SubtensorModule::get_conviction(&coldkey, netuid);
        println!("c_late = {}", c_late);
        assert_abs_diff_eq!(
            c_late.to_num::<f64>(),
            u64::from(lock_amount) as f64,
            epsilon = 0.0000001
        );
    });
}

#[test]
fn test_roll_forward_no_change_when_now_equals_last_update() {
    new_test_ext(1).execute_with(|| {
        let lock = LockState {
            locked_mass: 5000.into(),
            unlocked_mass: 0.into(),
            conviction: U64F64::from_num(1234),
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
            Error::<Test>::StakeUnavailable
        );
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

        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey, &hotkey_b
        ));

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
            Error::<Test>::StakeUnavailable
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
            Error::<Test>::StakeUnavailable
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
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey, &hotkey_b
        ));
        add_balance_to_coldkey_account(&coldkey, 100_000_000_000u64.into());
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

        let lock_a = Lock::<Test>::get((coldkey, netuid_a, hotkey_a)).unwrap();
        let lock_b = Lock::<Test>::get((coldkey, netuid_b, hotkey_b)).unwrap();
        assert_eq!(lock_a.locked_mass, 1000u64.into());
        assert_eq!(lock_b.locked_mass, 2000u64.into());

        // Hotkey locks should also be separate
        let hotkey_lock_a = HotkeyLock::<Test>::get(netuid_a, hotkey_a).unwrap();
        let hotkey_lock_b = HotkeyLock::<Test>::get(netuid_b, hotkey_b).unwrap();
        assert_eq!(hotkey_lock_a.locked_mass, 1000u64.into());
        assert_eq!(hotkey_lock_b.locked_mass, 2000u64.into());
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
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey, &hotkey
        ));
        add_balance_to_coldkey_account(&coldkey, 100_000_000_000u64.into());
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
        let lock_a = Lock::<Test>::get((coldkey, netuid_a, hotkey)).unwrap();
        assert_eq!(lock_a.locked_mass, 5000u64.into());

        // Hotkey lock on subnet A also unaffected
        let hotkey_lock_a = HotkeyLock::<Test>::get(netuid_a, hotkey).unwrap();
        assert_eq!(hotkey_lock_a.locked_mass, 5000u64.into());
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
        assert_eq!(c, U64F64::from_num(0));

        // After time, conviction grows
        step_block(1000);
        let c = SubtensorModule::hotkey_conviction(&hotkey, netuid);
        assert!(c > U64F64::from_num(0));
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
        add_balance_to_coldkey_account(&coldkey2, 100_000_000_000u64.into());
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey2, &hotkey
        ));
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
        assert!(diff < U64F64::from_num(1));
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

        add_balance_to_coldkey_account(&coldkey2, 100_000_000_000u64.into());
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey2, &hotkey_b
        ));
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
// GROUP 10: Lock force-reduction
// =========================================================================

#[test]
fn test_reduce_lock_removes_dust() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);
        let lock_amount = AlphaBalance::from(50u64);

        // Lock a small amount
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount,
        ));

        // Advance many taus so everything decays well below dust (100)
        let tau = MaturityRate::<Test>::get();
        let target = System::block_number() + tau * 50;
        System::set_block_number(target);

        // Remove full lock amount
        SubtensorModule::force_reduce_lock(&coldkey, netuid, lock_amount);

        assert!(Lock::<Test>::get((coldkey, netuid, hotkey)).is_none());
        assert!(HotkeyLock::<Test>::get(netuid, hotkey).is_none());
    });
}

#[test]
fn test_reduce_lock_partial_reduction() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);
        let lock_amount = AlphaBalance::from(100u64);
        let reduce_amount = AlphaBalance::from(40u64);
        let now = SubtensorModule::get_current_block_as_u64();

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            lock_amount,
        ));

        let conviction = U64F64::from_num(100);
        Lock::<Test>::insert(
            (coldkey, netuid, hotkey),
            LockState {
                locked_mass: lock_amount,
                unlocked_mass: 0.into(),
                conviction,
                last_update: now,
            },
        );
        HotkeyLock::<Test>::insert(
            netuid,
            hotkey,
            LockState {
                locked_mass: lock_amount,
                unlocked_mass: 0.into(),
                conviction,
                last_update: now,
            },
        );

        SubtensorModule::force_reduce_lock(&coldkey, netuid, reduce_amount);

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).expect("lock should remain");
        assert_eq!(lock.locked_mass, 60u64.into());
        assert_abs_diff_eq!(lock.conviction.to_num::<f64>(), 60., epsilon = 0.0000000001);

        let hotkey_lock =
            HotkeyLock::<Test>::get(netuid, hotkey).expect("hotkey lock should remain");
        assert_eq!(hotkey_lock.locked_mass, 60u64.into());
        assert_abs_diff_eq!(
            hotkey_lock.conviction.to_num::<f64>(),
            60.,
            epsilon = 0.0000000001
        );
    });
}

#[test]
fn test_reduce_lock_no_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let netuid = subtensor_runtime_common::NetUid::from(1);
        // Should be a no-op, no panic
        SubtensorModule::force_reduce_lock(&coldkey, netuid, 100u64.into());
        assert!(
            Lock::<Test>::iter_prefix((coldkey, netuid))
                .next()
                .is_none()
        );
    });
}

#[test]
fn test_reduce_lock_two_coldkeys() {
    new_test_ext(1).execute_with(|| {
        let coldkey1 = U256::from(1001);
        let coldkey2 = U256::from(1002);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey1, hotkey, 100_000_000_000);

        // Add stake on coldkey 2
        add_balance_to_coldkey_account(&coldkey2, 100_000_000_000u64.into());
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey2, &hotkey
        ));
        SubtensorModule::stake_into_subnet(
            &hotkey,
            &coldkey2,
            netuid,
            100_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        // Mock a non-zero conviction for both coldkeys
        let lock1 = Lock::<Test>::get((coldkey1, netuid, hotkey)).unwrap_or(LockState {
            locked_mass: 0.into(),
            unlocked_mass: 0.into(),
            conviction: U64F64::from_num(1234),
            last_update: System::block_number(),
        });
        let lock2 = Lock::<Test>::get((coldkey2, netuid, hotkey)).unwrap_or(LockState {
            locked_mass: 0.into(),
            unlocked_mass: 0.into(),
            conviction: U64F64::from_num(1234),
            last_update: System::block_number(),
        });
        Lock::<Test>::insert((coldkey1, netuid, hotkey), lock1);
        Lock::<Test>::insert((coldkey2, netuid, hotkey), lock2);
        HotkeyLock::<Test>::insert(
            netuid,
            hotkey,
            LockState {
                locked_mass: 0.into(),
                unlocked_mass: 0.into(),
                conviction: U64F64::from_num(1234 * 2),
                last_update: System::block_number(),
            },
        );

        // Lock a small amount from both coldkeys
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey1,
            netuid,
            &hotkey,
            50u64.into(),
        ));
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey2,
            netuid,
            &hotkey,
            50u64.into(),
        ));

        SubtensorModule::force_reduce_lock(&coldkey1, netuid, 50u64.into());

        // Should only clean up coldkey1's lock, not coldkey2's
        assert!(
            Lock::<Test>::iter_prefix((coldkey1, netuid))
                .next()
                .is_none()
        );
        assert!(Lock::<Test>::get((coldkey2, netuid, hotkey)).is_some());

        // Hotkey lock should reduce according to coldkey1 lock
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey).unwrap();
        assert_eq!(hotkey_lock.locked_mass, 50u64.into());

        // Conviction should be reduced by coldkey1's lock conviction,
        // but not fully reset because coldkey2 still has a lock
        assert!(hotkey_lock.conviction == U64F64::from_num(1234));
    });
}

// =========================================================================
// GROUP 11: Coldkey swap interaction
// =========================================================================

#[test]
fn test_coldkey_swap_swaps_lock() {
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

        // Lock removed on old coldkey
        assert!(
            Lock::<Test>::iter_prefix((old_coldkey, netuid))
                .next()
                .is_none()
        );
        // New coldkey now has the lock
        assert!(Lock::<Test>::get((new_coldkey, netuid, hotkey)).is_some());
    });
}

#[test]
fn test_coldkey_swap_lock_blocks_unstake() {
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

        // New coldkey should not be able to unstake
        let alpha = get_alpha(&hotkey, &new_coldkey, netuid);
        assert!(alpha > AlphaBalance::ZERO);
        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(new_coldkey),
                hotkey,
                netuid,
                alpha,
            ),
            Error::<Test>::StakeUnavailable
        );
    });
}

#[test]
// When both coldkeys already have unlocked-only lock state on the same subnet, the destination
// hotkey key should be preserved and unlocked_mass should be accumulated onto that record.
fn test_coldkey_swap_adds_unlocked_mass_into_existing_destination_lock() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(10);
        let old_hotkey = U256::from(2);
        let new_hotkey = U256::from(20);
        let netuid = subtensor_runtime_common::NetUid::from(1);
        let old_unlocked = AlphaBalance::from(4_000u64);
        let new_unlocked = AlphaBalance::from(6_000u64);

        // Seed unlocked-only lock rows on both coldkeys so the helper has to merge into
        // the destination record instead of creating a second lock entry on the subnet.
        SubtensorModule::insert_lock_state(
            &old_coldkey,
            netuid,
            &old_hotkey,
            LockState {
                locked_mass: AlphaBalance::ZERO,
                unlocked_mass: old_unlocked,
                conviction: U64F64::from_num(0),
                last_update: SubtensorModule::get_current_block_as_u64(),
            },
        );
        SubtensorModule::insert_lock_state(
            &new_coldkey,
            netuid,
            &new_hotkey,
            LockState {
                locked_mass: AlphaBalance::ZERO,
                unlocked_mass: new_unlocked,
                conviction: U64F64::from_num(0),
                last_update: SubtensorModule::get_current_block_as_u64(),
            },
        );

        SubtensorModule::swap_coldkey_locks(&old_coldkey, &new_coldkey);

        assert!(
            Lock::<Test>::iter_prefix((old_coldkey, netuid))
                .next()
                .is_none()
        );
        assert!(Lock::<Test>::get((new_coldkey, netuid, old_hotkey)).is_none());

        let merged_lock = Lock::<Test>::get((new_coldkey, netuid, new_hotkey))
            .expect("destination lock should remain under its original hotkey key");
        assert_eq!(merged_lock.locked_mass, AlphaBalance::ZERO);
        assert_eq!(merged_lock.unlocked_mass, old_unlocked + new_unlocked);
        assert_eq!(Lock::<Test>::iter_prefix((new_coldkey, netuid)).count(), 1);
    });
}

#[test]
// The public coldkey swap extrinsic runs inside a storage layer, so a late failure rolls back the earlier writes.
fn test_failed_coldkey_swap_extrinsic_rolls_back_state_changes() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let old_hotkey = U256::from(2);
        let new_coldkey = U256::from(3);
        let blocked_hotkey = U256::from(4);
        let netuid = setup_subnet_with_stake(old_coldkey, old_hotkey, 100_000_000_000);

        let original_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &old_hotkey,
            &old_coldkey,
            netuid,
        );
        assert!(!original_stake.is_zero());
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &old_hotkey,
                &new_coldkey,
                netuid
            ),
            AlphaBalance::ZERO
        );

        // Seed a lock directly on the destination coldkey so the swap reaches ActiveLockExists
        // without tripping the earlier "already associated" guard.
        SubtensorModule::insert_lock_state(
            &new_coldkey,
            netuid,
            &blocked_hotkey,
            LockState {
                locked_mass: 1u64.into(),
                unlocked_mass: AlphaBalance::ZERO,
                conviction: U64F64::from_num(0),
                last_update: SubtensorModule::get_current_block_as_u64(),
            },
        );

        assert_noop!(
            SubtensorModule::swap_coldkey(
                RuntimeOrigin::root(),
                old_coldkey,
                new_coldkey,
                TaoBalance::ZERO,
            ),
            Error::<Test>::ActiveLockExists
        );

        // The failed extrinsic should roll back the earlier stake transfer.
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &old_hotkey,
                &old_coldkey,
                netuid
            ),
            original_stake
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &old_hotkey,
                &new_coldkey,
                netuid
            ),
            AlphaBalance::ZERO
        );
    });
}

// =========================================================================
// GROUP 12: Hotkey swap interaction
// =========================================================================

#[test]
fn test_hotkey_swap_swaps_locks_and_convictions() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let old_hotkey = U256::from(2);
        let new_hotkey = U256::from(20);
        let netuid = setup_subnet_with_stake(coldkey, old_hotkey, 100_000_000_000);
        Owner::<Test>::insert(old_hotkey, coldkey);
        Owner::<Test>::insert(new_hotkey, coldkey);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &old_hotkey,
            5000u64.into(),
        ));

        // Mock a non-zero conviction
        let mut lock = Lock::<Test>::get((coldkey, netuid, old_hotkey)).unwrap();
        lock.conviction = U64F64::from_num(1234);
        Lock::<Test>::insert((coldkey, netuid, old_hotkey), lock);
        let mut hotkey_lock = HotkeyLock::<Test>::get(netuid, old_hotkey).unwrap();
        hotkey_lock.conviction = U64F64::from_num(1234);
        HotkeyLock::<Test>::insert(netuid, old_hotkey, hotkey_lock);

        // Perform hotkey swap
        let mut weight = Weight::zero();
        assert_ok!(SubtensorModule::perform_hotkey_swap_on_all_subnets(
            &old_hotkey,
            &new_hotkey,
            &coldkey,
            &mut weight,
            false
        ));

        // Lock references new_hotkey, conviction is not reset
        let lock = Lock::<Test>::get((coldkey, netuid, new_hotkey)).unwrap();
        assert_eq!(lock.locked_mass, 5000u64.into());
        assert!(lock.conviction > U64F64::from_num(0));

        // Hotkey lock data also updated, conviction is not reset
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, new_hotkey).unwrap();
        assert_eq!(hotkey_lock.locked_mass, 5000u64.into());
        assert!(hotkey_lock.conviction > U64F64::from_num(0));

        // Trying to top up to new_hotkey works
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &new_hotkey,
            100u64.into()
        ));

        // Trying to top up to old_hotkey fails (old_hotkey is no longer associated with coldkey)
        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &old_hotkey, 100u64.into()),
            Error::<Test>::LockHotkeyMismatch
        );
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

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).expect("Lock should exist");
        assert_eq!(lock.locked_mass, lock_amount.into());
        assert_eq!(lock.conviction, U64F64::from_num(0));

        // Hotkey lock should also be updated
        let hotkey_lock =
            HotkeyLock::<Test>::get(netuid, hotkey).expect("Hotkey lock should exist");
        assert_eq!(hotkey_lock.locked_mass, lock_amount.into());
        assert_eq!(hotkey_lock.conviction, U64F64::from_num(0));
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
            Error::<Test>::StakeUnavailable
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
            Error::<Test>::StakeUnavailable
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
            Error::<Test>::StakeUnavailable
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
        assert!(Lock::<Test>::get((coldkey, netuid, hotkey)).is_some());

        // Dissolve the subnet
        assert_ok!(SubtensorModule::do_dissolve_network(netuid));

        // All Alpha entries are gone
        assert_eq!(
            SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid),
            AlphaBalance::ZERO
        );

        // Lock entries are not orphaned
        let lock = Lock::<Test>::get((coldkey, netuid, hotkey));
        assert!(lock.is_none());

        // Hotkey lock is also removed
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey);
        assert!(hotkey_lock.is_none());
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

        // No stale lock from old subnet remains
        let stale_lock = Lock::<Test>::get((coldkey, netuid, hotkey_old));
        assert!(stale_lock.is_none());

        // No stale hotkey lock remains
        let stale_hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey_old);
        assert!(stale_hotkey_lock.is_none());
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
        add_balance_to_coldkey_account(&nominator, 100_000_000_000u64.into());
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &nominator,
            &owner_hotkey
        ));
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

        // clear_small_nomination removes the lock and unstakes alpha
        SubtensorModule::clear_small_nomination_if_required(&owner_hotkey, &nominator, netuid);

        // Nominator alpha has been removed despite lock
        let nominator_alpha_after = get_alpha(&owner_hotkey, &nominator, netuid);
        assert_eq!(nominator_alpha_after, AlphaBalance::ZERO);

        // Lock entry doesn't exist anymore
        assert!(
            Lock::<Test>::iter_prefix((nominator, netuid))
                .next()
                .is_none()
        );

        // Hotkey lock should also be removed
        let hotkey_lock = HotkeyLock::<Test>::get(netuid, owner_hotkey);
        assert!(hotkey_lock.is_none());
    });
}

#[test]
// If one coldkey has a large nomination on one hotkey and a tiny nomination on another,
// clearing the tiny nomination should reduce the lock state only by that tiny alpha amount.
fn test_clear_small_nomination_reduces_only_tiny_amount_from_lock_state() {
    new_test_ext(1).execute_with(|| {
        let coldkey_large = U256::from(100);
        let hotkey_large = U256::from(101);
        let netuid = setup_subnet_with_stake(coldkey_large, hotkey_large, 100_000_000_000);

        let coldkey_tiny = U256::from(102);
        let hotkey_tiny = U256::from(103);
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey_tiny,
            &hotkey_tiny
        ));

        let nominator = U256::from(200);
        let large_tao = TaoBalance::from(50_000_000_000u64);
        let tiny_tao = TaoBalance::from(1_000_000u64);
        add_balance_to_coldkey_account(&nominator, large_tao + tiny_tao);

        // Create one large nomination and one tiny nomination on the same subnet.
        SubtensorModule::stake_into_subnet(
            &hotkey_large,
            &nominator,
            netuid,
            large_tao,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();
        SubtensorModule::stake_into_subnet(
            &hotkey_tiny,
            &nominator,
            netuid,
            tiny_tao,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        let large_alpha_before = get_alpha(&hotkey_large, &nominator, netuid);
        let tiny_alpha_before = get_alpha(&hotkey_tiny, &nominator, netuid);
        assert!(large_alpha_before > tiny_alpha_before);

        // Lock against the large nomination hotkey and seed non-zero unlocked_mass + conviction
        // so we can verify each field is reduced only by the tiny nomination's alpha amount.
        let total_before = SubtensorModule::total_coldkey_alpha_on_subnet(&nominator, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &nominator,
            netuid,
            &hotkey_large,
            total_before,
        ));

        let unlocked_before = AlphaBalance::from(tiny_alpha_before.to_u64() + 1_000);
        let conviction_before = U64F64::from_num(tiny_alpha_before.to_u64() + 2_000);
        let last_update = SubtensorModule::get_current_block_as_u64();
        Lock::<Test>::insert(
            (nominator, netuid, hotkey_large),
            LockState {
                locked_mass: total_before,
                unlocked_mass: unlocked_before,
                conviction: conviction_before,
                last_update,
            },
        );
        HotkeyLock::<Test>::insert(
            netuid,
            hotkey_large,
            LockState {
                locked_mass: total_before,
                unlocked_mass: AlphaBalance::ZERO,
                conviction: conviction_before,
                last_update,
            },
        );

        // Force the tiny nomination to qualify as "small" and clear only that nomination.
        SubtensorModule::set_nominator_min_required_stake(u64::MAX);
        SubtensorModule::clear_small_nomination_if_required(&hotkey_tiny, &nominator, netuid);

        // The large nomination stays, the tiny one is removed.
        let large_alpha_after = get_alpha(&hotkey_large, &nominator, netuid);
        let tiny_alpha_after = get_alpha(&hotkey_tiny, &nominator, netuid);
        assert_eq!(large_alpha_after, large_alpha_before);
        assert!(!large_alpha_after.is_zero());
        assert_eq!(tiny_alpha_after, AlphaBalance::ZERO);

        // Only the tiny alpha amount should be shaved off the coldkey lock state.
        let lock_after = Lock::<Test>::get((nominator, netuid, hotkey_large)).unwrap();
        let tiny_alpha_fixed = U64F64::from_num(tiny_alpha_before.to_u64());
        assert!(!lock_after.locked_mass.is_zero());
        assert_eq!(lock_after.locked_mass, total_before - tiny_alpha_before);
        assert!(!lock_after.unlocked_mass.is_zero());
        assert_eq!(
            lock_after.unlocked_mass,
            unlocked_before - tiny_alpha_before
        );
        assert!(lock_after.conviction != U64F64::from_num(0));
        assert_eq!(lock_after.conviction, conviction_before - tiny_alpha_fixed);

        // The aggregate hotkey lock on the locked hotkey should also only shrink by the tiny amount.
        let hotkey_lock_after = HotkeyLock::<Test>::get(netuid, hotkey_large).unwrap();
        assert_eq!(
            hotkey_lock_after.locked_mass,
            total_before - tiny_alpha_before
        );
        assert_eq!(
            hotkey_lock_after.conviction,
            conviction_before - tiny_alpha_fixed
        );
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
        let available = SubtensorModule::available_stake(&coldkey, netuid);
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
        assert!(Lock::<Test>::get((coldkey, netuid, hotkey)).is_some());

        // Hotkey lock still references original hotkey
        assert!(HotkeyLock::<Test>::get(netuid, hotkey).is_some());
    });
}

// =========================================================================
// GROUP 19: Moving lock
// =========================================================================

#[test]
fn test_moving_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey_origin = U256::from(2);
        let hotkey_destination = U256::from(3);
        let netuid = setup_subnet_with_stake(coldkey, hotkey_origin, 100_000_000_000);

        let lock_amount = 5000u64.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey_origin,
            lock_amount
        ));

        // Mock a non-zero conviction
        let mut lock = Lock::<Test>::get((coldkey, netuid, hotkey_origin)).unwrap();
        lock.conviction = U64F64::from_num(1234);
        Lock::<Test>::insert((coldkey, netuid, hotkey_origin), lock);
        let mut hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey_origin).unwrap();
        hotkey_lock.conviction = U64F64::from_num(1234);
        HotkeyLock::<Test>::insert(netuid, hotkey_origin, hotkey_lock);

        assert_ok!(SubtensorModule::move_lock(
            RuntimeOrigin::signed(coldkey),
            hotkey_destination,
            netuid,
        ));
        let lock = Lock::<Test>::get((coldkey, netuid, hotkey_destination)).unwrap();
        assert_eq!(lock.locked_mass, lock_amount);
        assert_eq!(lock.conviction, U64F64::from_num(0));

        // Hotkey lock is removed on origin and added on destination
        assert!(HotkeyLock::<Test>::get(netuid, hotkey_origin).is_none());
        let hotkey_lock_destination_after =
            HotkeyLock::<Test>::get(netuid, hotkey_destination).unwrap();
        assert_eq!(hotkey_lock_destination_after.locked_mass, lock_amount);
        assert_eq!(
            hotkey_lock_destination_after.conviction,
            U64F64::from_num(0)
        );
    });
}

#[test]
fn test_moving_partial_lock() {
    new_test_ext(1).execute_with(|| {
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let hotkey_origin = U256::from(3);
        let hotkey_destination = U256::from(4);
        let netuid = setup_subnet_with_stake(coldkey1, hotkey_origin, 100_000_000_000);

        // Make hotkey_origin and hotkey_destination owned by different coldkeys
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey1,
            &hotkey_origin
        ));
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey2,
            &hotkey_destination
        ));

        // Add coldkey2 stake
        add_balance_to_coldkey_account(&coldkey2, 100_000_000_000u64.into());
        SubtensorModule::stake_into_subnet(
            &hotkey_origin,
            &coldkey2,
            netuid,
            50_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        let lock_amount = 5000u64.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey1,
            netuid,
            &hotkey_origin,
            lock_amount
        ));
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey2,
            netuid,
            &hotkey_origin,
            lock_amount
        ));

        // Mock a non-zero conviction
        let mut lock1 = Lock::<Test>::get((coldkey1, netuid, hotkey_origin)).unwrap();
        lock1.conviction = U64F64::from_num(1000);
        Lock::<Test>::insert((coldkey1, netuid, hotkey_origin), lock1);
        let mut lock2 = Lock::<Test>::get((coldkey2, netuid, hotkey_origin)).unwrap();
        lock2.conviction = U64F64::from_num(1000);
        Lock::<Test>::insert((coldkey2, netuid, hotkey_origin), lock2);
        let mut hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey_origin).unwrap();
        hotkey_lock.conviction = U64F64::from_num(2000);
        HotkeyLock::<Test>::insert(netuid, hotkey_origin, hotkey_lock);

        // Move lock for coldkey1 to hotkey_destination, coldkey2's lock should be unaffected
        assert_ok!(SubtensorModule::move_lock(
            RuntimeOrigin::signed(coldkey1),
            hotkey_destination,
            netuid,
        ));
        let lock1_after = Lock::<Test>::get((coldkey1, netuid, hotkey_destination)).unwrap();
        let lock2_after = Lock::<Test>::get((coldkey2, netuid, hotkey_origin)).unwrap();
        assert_eq!(lock1_after.locked_mass, lock_amount);
        assert_eq!(lock1_after.conviction, U64F64::from_num(0));
        assert_eq!(lock2_after.locked_mass, lock_amount);
        assert_eq!(lock2_after.conviction, U64F64::from_num(1000));

        // Hotkey lock is removed on origin and added on destination
        let hotkey_lock_origin_after = HotkeyLock::<Test>::get(netuid, hotkey_origin).unwrap();
        let hotkey_lock_destination_after =
            HotkeyLock::<Test>::get(netuid, hotkey_destination).unwrap();
        assert_eq!(hotkey_lock_origin_after.locked_mass, lock_amount);
        assert_eq!(hotkey_lock_origin_after.conviction, U64F64::from_num(1000));
        assert_eq!(hotkey_lock_destination_after.locked_mass, lock_amount);
        assert_eq!(
            hotkey_lock_destination_after.conviction,
            U64F64::from_num(0)
        );
    });
}

#[test]
fn test_moving_partial_lock_same_owners() {
    new_test_ext(1).execute_with(|| {
        let coldkey1 = U256::from(1);
        let coldkey2 = U256::from(2);
        let hotkey_origin = U256::from(3);
        let hotkey_destination = U256::from(4);
        let netuid = setup_subnet_with_stake(coldkey1, hotkey_origin, 100_000_000_000);

        // Add coldkey2 stake
        add_balance_to_coldkey_account(&coldkey2, 100_000_000_000u64.into());

        // Make hotkey_origin and hotkey_destination both owned by coldkey1
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey1,
            &hotkey_origin
        ));
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &coldkey1,
            &hotkey_destination
        ));
        SubtensorModule::stake_into_subnet(
            &hotkey_origin,
            &coldkey2,
            netuid,
            50_000_000_000u64.into(),
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        )
        .unwrap();

        let lock_amount = 5000u64.into();
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey1,
            netuid,
            &hotkey_origin,
            lock_amount
        ));
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey2,
            netuid,
            &hotkey_origin,
            lock_amount
        ));

        // Mock a non-zero conviction
        let mut lock1 = Lock::<Test>::get((coldkey1, netuid, hotkey_origin)).unwrap();
        lock1.conviction = U64F64::from_num(1000);
        Lock::<Test>::insert((coldkey1, netuid, hotkey_origin), lock1);
        let mut lock2 = Lock::<Test>::get((coldkey2, netuid, hotkey_origin)).unwrap();
        lock2.conviction = U64F64::from_num(1000);
        Lock::<Test>::insert((coldkey2, netuid, hotkey_origin), lock2);
        let mut hotkey_lock = HotkeyLock::<Test>::get(netuid, hotkey_origin).unwrap();
        hotkey_lock.conviction = U64F64::from_num(2000);
        HotkeyLock::<Test>::insert(netuid, hotkey_origin, hotkey_lock);

        // Move lock for coldkey1 to hotkey_destination, coldkey2's lock should be unaffected
        assert_ok!(SubtensorModule::move_lock(
            RuntimeOrigin::signed(coldkey1),
            hotkey_destination,
            netuid,
        ));
        let lock1_after = Lock::<Test>::get((coldkey1, netuid, hotkey_destination)).unwrap();
        let lock2_after = Lock::<Test>::get((coldkey2, netuid, hotkey_origin)).unwrap();
        assert_eq!(lock1_after.locked_mass, lock_amount);
        assert_eq!(lock1_after.conviction, U64F64::from_num(1000));
        assert_eq!(lock2_after.locked_mass, lock_amount);
        assert_eq!(lock2_after.conviction, U64F64::from_num(1000));

        // Hotkey lock is moved to destination with conviction
        let hotkey_lock_origin_after = HotkeyLock::<Test>::get(netuid, hotkey_origin).unwrap();
        let hotkey_lock_destination_after =
            HotkeyLock::<Test>::get(netuid, hotkey_destination).unwrap();
        assert_eq!(hotkey_lock_origin_after.locked_mass, lock_amount);
        assert_eq!(hotkey_lock_origin_after.conviction, U64F64::from_num(1000));
        assert_eq!(hotkey_lock_destination_after.locked_mass, lock_amount);
        assert_eq!(
            hotkey_lock_destination_after.conviction,
            U64F64::from_num(1000)
        );
    });
}

#[test]
// Moving a lock after partially unlocking it should preserve the coldkey's unavailable amount.
fn test_moving_unlocked_lock_preserves_unavailable_amount() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let destination_coldkey = U256::from(9);
        let hotkey_origin = U256::from(2);
        let hotkey_destination = U256::from(3);
        let netuid = setup_subnet_with_stake(coldkey, hotkey_origin, 100_000_000_000);

        // Make the destination hotkey exist under a different owner so the move is a real transfer of lock ownership.
        assert_ok!(SubtensorModule::create_account_if_non_existent(
            &destination_coldkey,
            &hotkey_destination
        ));

        // Lock some stake, then unlock part of it so unavailable stake is split across locked and unlocked mass.
        let lock_amount = AlphaBalance::from(5_000u64);
        let unlock_amount = AlphaBalance::from(2_000u64);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey_origin,
            lock_amount,
        ));
        assert_ok!(SubtensorModule::do_unlock_stake(
            &coldkey,
            netuid,
            unlock_amount,
        ));

        // Capture the coldkey-level availability view before moving the lock.
        let available_before = SubtensorModule::available_stake(&coldkey, netuid);
        let locked_before = SubtensorModule::get_current_locked(&coldkey, netuid);
        let unlocked_before = SubtensorModule::get_current_unlocked(&coldkey, netuid);
        let unavailable_before = locked_before + unlocked_before;

        // Move the lock to the destination hotkey.
        assert_ok!(SubtensorModule::move_lock(
            RuntimeOrigin::signed(coldkey),
            hotkey_destination,
            netuid,
        ));

        // The origin entry should be gone and the destination entry should preserve locked/unlocked mass.
        assert!(Lock::<Test>::get((coldkey, netuid, hotkey_origin)).is_none());
        let moved_lock = Lock::<Test>::get((coldkey, netuid, hotkey_destination)).unwrap();
        assert_eq!(moved_lock.locked_mass, locked_before);
        assert_eq!(moved_lock.unlocked_mass, unlocked_before);

        // The coldkey's unavailable and available stake should be unchanged by the move.
        let available_after = SubtensorModule::available_stake(&coldkey, netuid);
        let locked_after = SubtensorModule::get_current_locked(&coldkey, netuid);
        let unlocked_after = SubtensorModule::get_current_unlocked(&coldkey, netuid);
        let unavailable_after = locked_after + unlocked_after;
        assert_eq!(available_after, available_before);
        assert_eq!(unavailable_after, unavailable_before);
    });
}

// =========================================================================
// GROUP 20: Unlocking behavior
// =========================================================================

#[test]
// Fully unlocked stake should still be unavailable on the very next block.
fn test_unlocked_amount_cannot_be_unstaked_immediately() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Lock then immediately unlock the entire position.
        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, total
        ));
        assert_ok!(SubtensorModule::do_unlock_stake(&coldkey, netuid, total));

        // Right after unlock, everything sits in unlocked_mass and nothing is available yet.
        assert_eq!(SubtensorModule::available_stake(&coldkey, netuid), AlphaBalance::ZERO);
        assert_eq!(SubtensorModule::get_current_locked(&coldkey, netuid), AlphaBalance::ZERO);
        assert_eq!(SubtensorModule::get_current_unlocked(&coldkey, netuid), total);

        // Move one block to avoid unrelated rate-limit behavior on stake operations.
        step_block(1);

        // Unstaking the just-unlocked amount should still be blocked.
        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                total,
            ),
            Error::<Test>::StakeUnavailable
        );
    });
}

#[test]
// Fully unlocked stake should also be unavailable for immediate re-locking.
fn test_unlocked_amount_cannot_be_relocked_immediately() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Lock then immediately unlock the entire position.
        let total = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey, netuid, &hotkey, total
        ));
        assert_ok!(SubtensorModule::do_unlock_stake(&coldkey, netuid, total));

        // Nothing should be available to lock again yet.
        assert_eq!(
            SubtensorModule::available_stake(&coldkey, netuid),
            AlphaBalance::ZERO
        );
        assert_eq!(
            SubtensorModule::get_current_unlocked(&coldkey, netuid),
            total
        );

        // Even a tiny re-lock should fail because available stake is still zero.
        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, 1u64.into()),
            Error::<Test>::InsufficientStakeForLock
        );
    });
}

#[test]
// Unlocking more than the currently locked mass must be rejected and leave the lock untouched.
fn test_unlock_stake_rejects_amount_above_locked_mass() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        let locked_amount = AlphaBalance::from(1_000u64);
        let unlock_amount_too_high = AlphaBalance::from(1_001u64);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            locked_amount,
        ));

        assert_noop!(
            SubtensorModule::do_unlock_stake(&coldkey, netuid, unlock_amount_too_high),
            Error::<Test>::UnlockAmountTooHigh
        );

        let lock = Lock::<Test>::get((coldkey, netuid, hotkey)).expect("Lock should exist");
        assert_eq!(lock.locked_mass, locked_amount);
        assert_eq!(lock.unlocked_mass, AlphaBalance::ZERO);
    });
}

#[test]
// After one full UnlockRate period, unlocked_mass should decay to about e^-1 of its original value.
fn test_roll_forward_unlocked_mass_decays() {
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

        // Unlock all
        assert_ok!(SubtensorModule::do_unlock_stake(
            &coldkey,
            netuid,
            lock_amount.into()
        ));

        // Advance one full unlock rate via direct block number jump (step_block overflows u16 for tau=216000)
        let rate = UnlockRate::<Test>::get();
        let target = System::block_number() + rate;
        System::set_block_number(target);

        // There should be no locked amount
        let locked = SubtensorModule::get_current_locked(&coldkey, netuid);
        assert_eq!(locked, 0.into());

        // After one UnlockRate, unlocked should be ~36.8% of original
        let unlocked = SubtensorModule::get_current_unlocked(&coldkey, netuid);
        let expected = lock_amount as f64 * 0.368;
        assert_abs_diff_eq!(
            u64::from(unlocked) as f64,
            expected,
            epsilon = lock_amount as f64 / 10.
        );
    });
}

#[test]
// Even after one UnlockRate period, a large fraction of a fully unlocked position should remain unavailable.
fn test_unlock_decay_blocks_eighty_percent() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Start with a full lock, then fully unlock it.
        let original_lock = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let attempted_amount = original_lock * 8.into() / 10.into();

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            original_lock,
        ));
        assert_ok!(SubtensorModule::do_unlock_stake(
            &coldkey,
            netuid,
            original_lock,
        ));

        // Advance exactly one unlock time constant.
        let rate = UnlockRate::<Test>::get();
        let target = System::block_number() + rate;
        System::set_block_number(target);

        // Only about 36.8% should remain unavailable here, so 80% is still too much.
        let unlocked = SubtensorModule::get_current_unlocked(&coldkey, netuid);
        assert!(unlocked < attempted_amount);

        // The same oversized amount should fail for both unstake and re-lock.
        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                attempted_amount,
            ),
            Error::<Test>::StakeUnavailable
        );

        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, attempted_amount),
            Error::<Test>::InsufficientStakeForLock
        );
    });
}

#[test]
// If only half the position is unlocked, even 40% of the original position should still be blocked after one UnlockRate.
fn test_unlock_decay_blocks_forty_percent_after_half_unlock() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Lock the full position, then unlock only half of it.
        let original_lock = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let unlocked_amount = original_lock / 2.into();
        let attempted_amount = original_lock * 4.into() / 10.into();

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            original_lock,
        ));
        assert_ok!(SubtensorModule::do_unlock_stake(
            &coldkey,
            netuid,
            unlocked_amount,
        ));

        // Advance exactly one unlock time constant.
        let rate = UnlockRate::<Test>::get();
        let target = System::block_number() + rate;
        System::set_block_number(target);

        // Since only half the original position entered unlocked_mass, 40% of the original is still unavailable.
        let unlocked = SubtensorModule::get_current_unlocked(&coldkey, netuid);
        assert!(unlocked < attempted_amount);

        // The same oversized amount should fail for both unstake and re-lock.
        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                attempted_amount,
            ),
            Error::<Test>::StakeUnavailable
        );

        assert_noop!(
            SubtensorModule::do_lock_stake(&coldkey, netuid, &hotkey, attempted_amount),
            Error::<Test>::InsufficientStakeForLock
        );
    });
}

#[test]
// After one UnlockRate on a fully unlocked position, 60% of the original should be available to re-lock,
// and once re-locked it should no longer be immediately available to unstake.
fn test_unlock_decay_allows_relock_then_blocks_unstake() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = setup_subnet_with_stake(coldkey, hotkey, 100_000_000_000);

        // Lock the full position, then fully unlock it.
        let original_lock = SubtensorModule::total_coldkey_alpha_on_subnet(&coldkey, netuid);
        let relock_amount = original_lock * 6.into() / 10.into();

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            original_lock,
        ));
        assert_ok!(SubtensorModule::do_unlock_stake(
            &coldkey,
            netuid,
            original_lock,
        ));

        // Advance exactly one unlock time constant.
        let rate = UnlockRate::<Test>::get();
        let target = System::block_number() + rate;
        System::set_block_number(target);

        // About 63.2% of the original position should now be available again, so 60% can be re-locked.
        let available = SubtensorModule::available_stake(&coldkey, netuid);
        assert!(available >= relock_amount);

        assert_ok!(SubtensorModule::do_lock_stake(
            &coldkey,
            netuid,
            &hotkey,
            relock_amount,
        ));

        // Once re-locked, that amount should no longer be immediately available to unstake.
        step_block(1);
        assert_noop!(
            SubtensorModule::do_remove_stake(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                netuid,
                relock_amount,
            ),
            Error::<Test>::StakeUnavailable
        );
    });
}
