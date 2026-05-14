#![allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::arithmetic_side_effects
)]

use super::mock::*;
use crate::*;
use frame_support::assert_ok;
use sp_core::U256;
use subtensor_runtime_common::{AlphaBalance, NetUid};

fn ref_count(coldkey: &U256) -> u32 {
    RootRegisteredHotkeyCount::<Test>::get(coldkey)
}

#[test]
fn ref_count_helpers_basic_behavior() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(7);

        // Reader on an unset key.
        assert_eq!(ref_count(&coldkey), 0);
        assert!(!SubtensorModule::coldkey_has_root_hotkey(&coldkey));

        // Saturating decrement at zero must not underflow.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert_eq!(ref_count(&coldkey), 0);
        assert!(!RootRegisteredHotkeyCount::<Test>::contains_key(coldkey));

        // Increment populates storage and flips the reader.
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        assert!(RootRegisteredHotkeyCount::<Test>::contains_key(coldkey));
        assert!(SubtensorModule::coldkey_has_root_hotkey(&coldkey));

        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        assert_eq!(ref_count(&coldkey), 2);

        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert_eq!(ref_count(&coldkey), 1);

        // Decrement to zero removes the storage entry.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert!(!RootRegisteredHotkeyCount::<Test>::contains_key(coldkey));

        // Saturating decrement on an absent key must not resurrect the entry.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert!(!RootRegisteredHotkeyCount::<Test>::contains_key(coldkey));
    });
}

#[test]
fn increment_fires_on_added_only_on_zero_to_one_transition() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(10);
        let _ = take_root_registration_log();

        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        assert_eq!(
            take_root_registration_log(),
            vec![RootRegistrationChange::Added(coldkey)]
        );

        // Subsequent increments stay above zero and must not re-fire.
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        assert!(take_root_registration_log().is_empty());
    });
}

#[test]
fn decrement_fires_on_removed_only_on_one_to_zero_transition() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(10);
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        let _ = take_root_registration_log();

        // Above-zero decrements are silent.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert!(take_root_registration_log().is_empty());

        // The 1→0 edge fires once.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert_eq!(
            take_root_registration_log(),
            vec![RootRegistrationChange::Removed(coldkey)]
        );

        // Decrementing a zero count must not fire a spurious `Removed`.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert!(take_root_registration_log().is_empty());
    });
}

#[test]
fn ref_count_invariant_holds_across_mutations() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        // Lift the per-block / per-interval registration caps so the test
        // can register five hotkeys without stepping blocks.
        MaxRegistrationsPerBlock::<Test>::set(NetUid::ROOT, 64);
        TargetRegistrationsPerInterval::<Test>::set(NetUid::ROOT, 64);

        assert_ok!(SubtensorModule::check_root_registered_hotkey_count());

        let cold1 = U256::from(10);
        let cold2 = U256::from(20);
        let cold3 = U256::from(30);
        let h1 = U256::from(11);
        let h2 = U256::from(12);
        let h3 = U256::from(21);
        let h4 = U256::from(31);

        // Mix of registrations across multiple coldkeys.
        root_register_with_stake(&cold1, &h1, alpha);
        root_register_with_stake(&cold1, &h2, alpha);
        root_register_with_stake(&cold2, &h3, alpha);
        root_register_with_stake(&cold3, &h4, alpha);
        assert_ok!(SubtensorModule::check_root_registered_hotkey_count());

        // Replace path through `do_root_register` at the cap.
        MaxAllowedUids::<Test>::set(NetUid::ROOT, 4);
        let cold4 = U256::from(40);
        let h5 = U256::from(41);
        register_ok_neuron(alpha, h5, cold4, 0);
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &h5,
            &cold4,
            NetUid::ROOT,
            AlphaBalance::from(10_000_000_000_u64),
        );
        assert_ok!(SubtensorModule::root_register(
            RuntimeOrigin::signed(cold4),
            h5,
        ));
        assert_ok!(SubtensorModule::check_root_registered_hotkey_count());

        // Coldkey swap moves a multi-hotkey holder's count to a fresh coldkey.
        let cold1_new = U256::from(99);
        assert_ok!(SubtensorModule::do_swap_coldkey(&cold1, &cold1_new));
        assert_ok!(SubtensorModule::check_root_registered_hotkey_count());

        // Trim drops the lowest emitter; tightens the invariant under
        // bulk removal.
        ImmunityPeriod::<Test>::set(NetUid::ROOT, 0);
        MinAllowedUids::<Test>::set(NetUid::ROOT, 1);
        assert_ok!(SubtensorModule::trim_to_max_allowed_uids(NetUid::ROOT, 1));
        assert_ok!(SubtensorModule::check_root_registered_hotkey_count());
    });
}

#[test]
fn ref_count_invariant_detects_stale_overcount() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        let coldkey = U256::from(10);
        root_register_with_stake(&coldkey, &U256::from(11), alpha);
        assert_ok!(SubtensorModule::check_root_registered_hotkey_count());

        // Simulate a buggy code path that incremented the counter without a
        // matching root registration. The invariant must surface the drift.
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        assert!(SubtensorModule::check_root_registered_hotkey_count().is_err());
    });
}

#[test]
fn ref_count_invariant_detects_missing_index_entry() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        let coldkey = U256::from(10);
        root_register_with_stake(&coldkey, &U256::from(11), alpha);
        assert_ok!(SubtensorModule::check_root_registered_hotkey_count());

        // Simulate a buggy path that registered a root hotkey without
        // updating the reverse index. The invariant must catch the
        // coldkey that now has root hotkeys but no counter entry.
        RootRegisteredHotkeyCount::<Test>::remove(coldkey);
        assert!(SubtensorModule::check_root_registered_hotkey_count().is_err());
    });
}

#[test]
fn inspector_invariant_passes_when_set_matches() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        MaxRegistrationsPerBlock::<Test>::set(NetUid::ROOT, 64);
        TargetRegistrationsPerInterval::<Test>::set(NetUid::ROOT, 64);

        let cold1 = U256::from(10);
        let cold2 = U256::from(20);
        // Two hotkeys under cold1, one under cold2: the expected root-registered
        // set is the two distinct coldkeys, not three.
        root_register_with_stake(&cold1, &U256::from(11), alpha);
        root_register_with_stake(&cold1, &U256::from(12), alpha);
        root_register_with_stake(&cold2, &U256::from(21), alpha);

        set_mock_root_registered_inspector_members(Some(vec![cold1, cold2]));
        assert_ok!(SubtensorModule::check_root_registered_matches_inspector());
    });
}

#[test]
fn inspector_invariant_skips_when_none() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        root_register_with_stake(&U256::from(10), &U256::from(11), alpha);

        // Inspector unset by default: the check must silently no-op even
        // when the on-chain root set is non-empty.
        set_mock_root_registered_inspector_members(None);
        assert_ok!(SubtensorModule::check_root_registered_matches_inspector());
    });
}

#[test]
fn inspector_invariant_fails_on_mismatch() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        let cold = U256::from(10);
        root_register_with_stake(&cold, &U256::from(11), alpha);

        // Inspector forgot to include the root-registered coldkey.
        set_mock_root_registered_inspector_members(Some(vec![]));
        assert!(SubtensorModule::check_root_registered_matches_inspector().is_err());

        // Inspector holds a coldkey that has no root hotkey.
        set_mock_root_registered_inspector_members(Some(vec![cold, U256::from(999)]));
        assert!(SubtensorModule::check_root_registered_matches_inspector().is_err());
    });
}

#[test]
fn ema_lifecycle_init_clear_and_reentry() {
    use substrate_fixed::types::U64F64;
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        let coldkey = U256::from(10);
        assert!(!RootRegisteredStakeEma::<Test>::contains_key(coldkey));

        // First root registration seeds a zero-valued slot.
        root_register_with_stake(&coldkey, &U256::from(11), alpha);
        let state = RootRegisteredStakeEma::<Test>::get(coldkey);
        assert_eq!(state.ema, U64F64::from_num(0));
        assert_eq!(state.samples, 0);

        // Advance the sampler so we can prove re-entry resets it.
        SubtensorModule::tick_root_registered_stake_ema(1);
        SubtensorModule::tick_root_registered_stake_ema(2);
        assert_eq!(RootRegisteredStakeEma::<Test>::get(coldkey).samples, 2);

        // Drop to zero hotkeys: the EMA slot is cleared.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert!(!RootRegisteredStakeEma::<Test>::contains_key(coldkey));

        // Re-register: state starts fresh.
        root_register_with_stake(&coldkey, &U256::from(12), alpha);
        let state = RootRegisteredStakeEma::<Test>::get(coldkey);
        assert_eq!(state.ema, U64F64::from_num(0));
        assert_eq!(state.samples, 0);
    });
}

#[test]
fn ema_tick_writes_state_and_advances_cursor() {
    use substrate_fixed::types::U64F64;
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        MaxRegistrationsPerBlock::<Test>::set(NetUid::ROOT, 64);
        TargetRegistrationsPerInterval::<Test>::set(NetUid::ROOT, 64);

        let cold_a = U256::from(10);
        let cold_b = U256::from(20);
        root_register_with_stake(&cold_a, &U256::from(11), alpha);
        root_register_with_stake(&cold_b, &U256::from(21), alpha);
        let _ = take_ema_strategy_log();

        // Strategy returns a deterministic non-zero value so the EMA write
        // is observable in storage.
        set_ema_strategy_next(|_, _| U64F64::from_num(42));

        // Two consecutive ticks at SamplingInterval = 1: each picks a
        // distinct member, cursor advances.
        assert_eq!(EmaSampleCursor::<Test>::get(), 0);
        SubtensorModule::tick_root_registered_stake_ema(1);
        assert_eq!(EmaSampleCursor::<Test>::get(), 1);
        SubtensorModule::tick_root_registered_stake_ema(2);
        assert_eq!(EmaSampleCursor::<Test>::get(), 2);

        let log = take_ema_strategy_log();
        let touched: Vec<U256> = log.iter().map(|(k, _)| *k).collect();
        assert_eq!(touched.len(), 2);
        assert!(touched.contains(&cold_a) && touched.contains(&cold_b));

        // Both members have the strategy's return value persisted and
        // their sample counter incremented to 1.
        let state_a = RootRegisteredStakeEma::<Test>::get(cold_a);
        assert_eq!(state_a.ema, U64F64::from_num(42));
        assert_eq!(state_a.samples, 1);
        let state_b = RootRegisteredStakeEma::<Test>::get(cold_b);
        assert_eq!(state_b.ema, U64F64::from_num(42));
        assert_eq!(state_b.samples, 1);

        // A third tick revisits one of the members and bumps its counter to 2.
        SubtensorModule::tick_root_registered_stake_ema(3);
        assert_eq!(EmaSampleCursor::<Test>::get(), 3);
        let revisited_samples = RootRegisteredStakeEma::<Test>::get(cold_a).samples
            + RootRegisteredStakeEma::<Test>::get(cold_b).samples;
        assert_eq!(revisited_samples, 3);

        clear_ema_strategy_next();
    });
}

#[test]
fn ema_tick_is_no_op_when_no_members() {
    new_test_ext(1).execute_with(|| {
        // No registrations: the iterator is empty so the tick must not
        // touch the cursor or call the strategy.
        let _ = take_ema_strategy_log();
        let cursor_before = EmaSampleCursor::<Test>::get();
        SubtensorModule::tick_root_registered_stake_ema(1);
        assert_eq!(EmaSampleCursor::<Test>::get(), cursor_before);
        assert!(take_ema_strategy_log().is_empty());
    });
}

#[test]
fn ema_tick_is_no_op_when_interval_is_zero() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        root_register_with_stake(&U256::from(10), &U256::from(11), alpha);
        let _ = take_ema_strategy_log();

        // Zero interval disables sampling entirely: the early guard must
        // return before any storage read.
        set_ema_sampling_interval(0);
        let cursor_before = EmaSampleCursor::<Test>::get();
        SubtensorModule::tick_root_registered_stake_ema(1);
        SubtensorModule::tick_root_registered_stake_ema(100);
        assert_eq!(EmaSampleCursor::<Test>::get(), cursor_before);
        assert!(take_ema_strategy_log().is_empty());

        set_ema_sampling_interval(1);
    });
}

#[test]
fn ema_tick_acts_only_on_blocks_that_are_multiples_of_interval() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        let coldkey = U256::from(10);
        root_register_with_stake(&coldkey, &U256::from(11), alpha);
        let _ = take_ema_strategy_log();

        set_ema_sampling_interval(5);

        // Off-interval blocks 1..=4 must no-op.
        let cursor_before = EmaSampleCursor::<Test>::get();
        for block in 1..=4 {
            SubtensorModule::tick_root_registered_stake_ema(block);
        }
        assert_eq!(EmaSampleCursor::<Test>::get(), cursor_before);
        assert!(take_ema_strategy_log().is_empty());

        // Block 5 is a multiple of the interval: tick acts.
        SubtensorModule::tick_root_registered_stake_ema(5);
        assert_eq!(EmaSampleCursor::<Test>::get(), cursor_before + 1);
        let log = take_ema_strategy_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].0, coldkey);

        set_ema_sampling_interval(1);
    });
}

#[test]
fn ema_tick_returns_weight_including_strategy_contribution() {
    use frame_support::weights::Weight;
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        root_register_with_stake(&U256::from(10), &U256::from(11), alpha);

        // Strategy reports a non-zero per-call weight; the tick must
        // surface it through its return value so on_initialize can bill
        // the actual cost.
        set_ema_strategy_weights(Weight::from_parts(12_345, 0), Weight::zero());
        let on_tick = SubtensorModule::tick_root_registered_stake_ema(1);
        assert!(
            on_tick.ref_time() >= 12_345,
            "tick weight must include strategy contribution, got {on_tick:?}"
        );
    });
}

#[test]
fn ema_tick_default_unit_strategy_freezes_value() {
    use substrate_fixed::types::U64F64;
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        let coldkey = U256::from(10);
        root_register_with_stake(&coldkey, &U256::from(11), alpha);

        // No `set_ema_strategy_next`: MockEmaStrategy returns `previous`,
        // matching the `()` default. EMA stays at the init value (0)
        // but the sample counter still advances.
        let _ = take_ema_strategy_log();
        SubtensorModule::tick_root_registered_stake_ema(1);

        let state = RootRegisteredStakeEma::<Test>::get(coldkey);
        assert_eq!(state.ema, U64F64::from_num(0));
        assert_eq!(state.samples, 1);
    });
}
