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
fn coldkey_has_root_hotkey_is_false_when_count_is_zero() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(7);
        assert_eq!(ref_count(&coldkey), 0);
        assert!(!SubtensorModule::coldkey_has_root_hotkey(&coldkey));
    });
}

#[test]
fn increment_decrement_helpers_saturate() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);

        // Decrement at zero must not underflow.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert_eq!(ref_count(&coldkey), 0);

        // Increment normally.
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        assert_eq!(ref_count(&coldkey), 2);

        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert_eq!(ref_count(&coldkey), 1);
    });
}

#[test]
fn decrement_to_zero_removes_storage_entry() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);

        SubtensorModule::increment_root_registered_hotkey_count(&coldkey);
        assert!(RootRegisteredHotkeyCount::<Test>::contains_key(coldkey));

        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert!(!RootRegisteredHotkeyCount::<Test>::contains_key(coldkey));

        // Saturating decrement on an absent key must not resurrect the entry.
        SubtensorModule::decrement_root_registered_hotkey_count(&coldkey);
        assert!(!RootRegisteredHotkeyCount::<Test>::contains_key(coldkey));
    });
}

#[test]
fn try_state_invariant_holds_across_mutations() {
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
fn try_state_invariant_detects_stale_overcount() {
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
fn economic_eligible_invariant_passes_when_set_matches() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        MaxRegistrationsPerBlock::<Test>::set(NetUid::ROOT, 64);
        TargetRegistrationsPerInterval::<Test>::set(NetUid::ROOT, 64);

        let cold1 = U256::from(10);
        let cold2 = U256::from(20);
        // Two hotkeys under cold1, one under cold2: the expected EconomicEligible
        // set is the two distinct coldkeys, not three.
        root_register_with_stake(&cold1, &U256::from(11), alpha);
        root_register_with_stake(&cold1, &U256::from(12), alpha);
        root_register_with_stake(&cold2, &U256::from(21), alpha);

        set_mock_economic_eligible_members(Some(vec![cold1, cold2]));
        assert_ok!(SubtensorModule::check_economic_eligible_matches_root_registered());
    });
}

#[test]
fn economic_eligible_invariant_skips_when_inspector_returns_none() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);
        root_register_with_stake(&U256::from(10), &U256::from(11), alpha);

        // Inspector unset by default: the check must silently no-op even
        // when the on-chain root set is non-empty.
        set_mock_economic_eligible_members(None);
        assert_ok!(SubtensorModule::check_economic_eligible_matches_root_registered());
    });
}

#[test]
fn economic_eligible_invariant_fails_on_missing_member() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        let cold = U256::from(10);
        root_register_with_stake(&cold, &U256::from(11), alpha);

        // Collective forgot to include the root-registered coldkey.
        set_mock_economic_eligible_members(Some(vec![]));
        assert!(SubtensorModule::check_economic_eligible_matches_root_registered().is_err());
    });
}

#[test]
fn economic_eligible_invariant_fails_on_extra_member() {
    new_test_ext(1).execute_with(|| {
        let alpha = NetUid::from(1);
        add_network(NetUid::ROOT, 1, 0);
        add_network(alpha, 1, 0);

        let cold = U256::from(10);
        root_register_with_stake(&cold, &U256::from(11), alpha);

        // Collective holds a coldkey that has no root hotkey.
        set_mock_economic_eligible_members(Some(vec![cold, U256::from(999)]));
        assert!(SubtensorModule::check_economic_eligible_matches_root_registered().is_err());
    });
}

