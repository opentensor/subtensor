#![allow(clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use frame_support::{assert_err, assert_noop, assert_ok, sp_runtime::FixedU128, traits::Currency};
use frame_system::Config;
use sp_core::U256;
use subtensor_runtime_common::{AlphaCurrency, Currency as CurrencyT, NetUid, NetUidStorageIndex};

/********************************************
    subscribing::subscribe() tests
*********************************************/

#[test]
fn test_burn_price_init_defaults() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        let now = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::initialize_burn_price_if_needed(netuid, now);

        // Defaults configured by initialize_burn_price_if_needed
        assert_eq!(BurnHalfLife::<Test>::get(netuid), 360);
        assert_eq!(
            BurnIncreaseMult::<Test>::get(netuid),
            FixedU128::from_u32(2)
        );
        assert_eq!(BurnPrice::<Test>::get(netuid), 1_000_000_000u128); // 1 TAO base units
        assert_eq!(BurnLastUpdate::<Test>::get(netuid), now);
        assert_eq!(BurnPendingBumps::<Test>::get(netuid), 0);
        assert_eq!(BurnPendingFrom::<Test>::get(netuid), 0);
    });
}

#[test]
fn test_burn_increase_mult_default_getter() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        // Clear to zero (default). Getter should return 2.0
        BurnIncreaseMult::<Test>::insert(netuid, FixedU128::zero());
        assert_eq!(
            SubtensorModule::get_burn_increase_mult(netuid),
            FixedU128::from_u32(2)
        );

        // Non-zero should be returned verbatim.
        BurnIncreaseMult::<Test>::insert(netuid, FixedU128::from_u32(3));
        assert_eq!(
            SubtensorModule::get_burn_increase_mult(netuid),
            FixedU128::from_u32(3)
        );
    });
}

#[test]
fn test_burn_price_halves_after_half_life() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        add_network(netuid, 13, 0);

        // Initialize to set defaults (half-life, multiplier, initial price).
        let now = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::initialize_burn_price_if_needed(netuid, now);

        let price0 = BurnPrice::<Test>::get(netuid);
        let hl_u64 = BurnHalfLife::<Test>::get(netuid);
        assert!(hl_u64 > 0);

        // step_block expects u16; in our tests the half-life is small (e.g., 360), so this fits.
        let hl_steps: u16 = hl_u64.try_into().expect("half-life fits u16 in tests");
        step_block(hl_steps);

        let now2 = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::update_burn_price_to_block(netuid, now2);

        // Integer halving with possible +1 rounding tolerance.
        let price1 = BurnPrice::<Test>::get(netuid);
        let target = price0 / 2;
        assert!(price1 == target || price1 == target.saturating_add(1));
    });
}

#[test]
fn test_burn_price_multiple_halvings() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(7);
        add_network(netuid, 5, 0);

        let start = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::initialize_burn_price_if_needed(netuid, start);
        let p0 = BurnPrice::<Test>::get(netuid);
        let hl_u64 = BurnHalfLife::<Test>::get(netuid);

        // Advance by 3 * half-life; convert to u16 for the test harness.
        let three_hl_u64 = hl_u64.saturating_mul(3);
        let steps: u16 = three_hl_u64
            .try_into()
            .expect("3*half-life fits u16 in tests");
        step_block(steps);

        let now3 = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::update_burn_price_to_block(netuid, now3);

        // After 3 halvings, price ≈ p0 / 8 (allow +1 rounding tolerance).
        let p3 = BurnPrice::<Test>::get(netuid);
        let tgt = p0 / 8;
        assert!(p3 == tgt || p3 == tgt.saturating_add(1));
    });
}

#[test]
fn test_burn_price_bump_applies_next_block() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(11);
        add_network(netuid, 13, 0);

        // Initialize
        let now = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::initialize_burn_price_if_needed(netuid, now);

        // Prepare swap reserves and funds
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let price_now = SubtensorModule::get_burn(netuid);
        let cold = U256::from(999);
        let hot = U256::from(123);
        SubtensorModule::add_balance_to_coldkey_account(&cold, price_now.saturating_mul(10));

        // One registration this block schedules a bump for next block
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            netuid,
            hot
        ));

        // Price must be unchanged within the same block
        assert_eq!(SubtensorModule::get_burn(netuid), price_now);

        // Next block: multiplier applies
        step_block(1);
        let price_next = SubtensorModule::get_burn(netuid);
        assert_eq!(price_next, price_now.saturating_mul(2));
    });
}

#[test]
fn test_burn_price_multi_bumps_same_block_coalesce() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(12);
        add_network(netuid, 13, 0);

        // Initialize
        let start = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::initialize_burn_price_if_needed(netuid, start);

        // Reserves and funds
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let price_now = SubtensorModule::get_burn(netuid);
        let cold = U256::from(7777);
        let hk1 = U256::from(1);
        let hk2 = U256::from(2);
        // Two regs in the same block both pay the same price_now
        SubtensorModule::add_balance_to_coldkey_account(&cold, price_now.saturating_mul(10));

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            netuid,
            hk1
        ));
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            netuid,
            hk2
        ));

        // Not applied yet
        assert_eq!(SubtensorModule::get_burn(netuid), price_now);

        // Next block: two bumps coalesce -> 4x
        step_block(1);
        let price_next = SubtensorModule::get_burn(netuid);
        assert_eq!(price_next, price_now.saturating_mul(4));
    });
}

#[test]
fn test_burn_price_bump_and_decay_commute() {
    // With half-life = 1 block and one registration in block N:
    // Next block (N+1) applies a 2x bump and a halving -> net effect ≈ 1x (original).
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(21);
        add_network(netuid, 13, 0);

        // Force HL=1 for this subnet
        BurnHalfLife::<Test>::insert(netuid, 1);

        // Initialize
        let now = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::initialize_burn_price_if_needed(netuid, now);
        let p0 = BurnPrice::<Test>::get(netuid);

        // Reserves + funds
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let cold = U256::from(404);
        let hot = U256::from(505);
        SubtensorModule::add_balance_to_coldkey_account(
            &cold,
            SubtensorModule::get_burn(netuid).saturating_mul(5),
        );

        // Register once in this block -> schedules 2x bump for next block
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            netuid,
            hot
        ));

        // Advance 1 block (HL) -> bump (2x) then halving (/2) => ~p0 again
        step_block(1);
        let p1 = BurnPrice::<Test>::get(netuid);
        assert!(
            p1 == p0 || p1 == p0.saturating_add(1),
            "bump and decay commute"
        );
    });
}

#[test]
fn test_burn_price_floor_nonzero() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(99);
        add_network(netuid, 13, 0);

        // Force a tiny price and fast halving; it must not drop to 0
        BurnPrice::<Test>::insert(netuid, 1u128);
        BurnLastUpdate::<Test>::insert(netuid, 0u64);
        BurnHalfLife::<Test>::insert(netuid, 1u64);
        BurnPendingBumps::<Test>::insert(netuid, 0u32);
        BurnPendingFrom::<Test>::insert(netuid, 0u64);

        step_block(100);
        let now = SubtensorModule::get_current_block_as_u64();
        SubtensorModule::update_burn_price_to_block(netuid, now);

        assert_eq!(BurnPrice::<Test>::get(netuid), 1u128, "price floored at 1");
        assert_eq!(SubtensorModule::get_burn(netuid) as u128, 1u128);
    });
}

/* -------------------------------------------------------------------------- */
/*                           Burned registration flow                          */
/* -------------------------------------------------------------------------- */

#[test]
fn test_registration_ok() {
    // "Registration OK" under the new burned registration path.
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey = U256::from(1);
        let coldkey = U256::from(667);

        add_network(netuid, tempo, 0);

        // Setup swap reserves and sufficient coldkey funds
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());

        let burn_cost = SubtensorModule::get_burn(netuid);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, burn_cost.saturating_mul(10));

        // Register
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));

        // Neuron added
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 1);

        // Ownership mapping
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            coldkey
        );

        // UID mapping + zero alpha stake
        let uid = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey).expect("uid exists");
        assert_eq!(
            SubtensorModule::get_stake_for_uid_and_subnetwork(netuid, uid),
            AlphaCurrency::ZERO
        );
    });
}

#[test]
fn test_registration_already_active_hotkey() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 13;
        let hotkey = U256::from(1);
        let coldkey = U256::from(667);

        add_network(netuid, tempo, 0);

        // Reserves + funds
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let burn = SubtensorModule::get_burn(netuid);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, burn.saturating_mul(10));

        // First registration OK
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey
        ));

        // Second registration with same hotkey must fail
        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            netuid,
            hotkey,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::HotKeyAlreadyRegisteredInSubNet.into())
        );
    });
}

#[test]
fn test_burned_registration_non_associated_coldkey() {
    // Pre‑associate a hotkey with another coldkey. Attempting to register it using
    // a different coldkey must fail with NonAssociatedColdKey.
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(2);
        add_network(netuid, 13, 0);

        // Pre‑bind hotkey -> cold_A
        let cold_a = U256::from(100);
        let cold_b = U256::from(200);
        let hk = U256::from(12345);
        Owner::<Test>::insert(hk, cold_a);

        // Setup reserves + fund cold_b
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let burn = SubtensorModule::get_burn(netuid);
        SubtensorModule::add_balance_to_coldkey_account(&cold_b, burn.saturating_mul(10));

        // Attempt with cold_b
        assert_err!(
            SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold_b),
                netuid,
                hk
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn test_burned_registration_disabled() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(3);
        add_network(netuid, 13, 0);

        // Disable registration explicitly
        SubtensorModule::set_network_registration_allowed(netuid, false);

        // Reserves + funds (even if funded, it should be blocked)
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let burn = SubtensorModule::get_burn(netuid);
        let cold = U256::from(668);
        let hot = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&cold, burn.saturating_mul(2));

        let result = SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            netuid,
            hot,
        );
        assert_eq!(
            result,
            Err(Error::<Test>::SubNetRegistrationDisabled.into())
        );
    });
}

#[test]
fn test_burned_registration_without_neuron_slot() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(4);
        let tempo: u16 = 13;
        let hot = U256::from(1);
        let cold = U256::from(667);

        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_allowed_uids(netuid, 0);

        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let burn = SubtensorModule::get_burn(netuid);
        SubtensorModule::add_balance_to_coldkey_account(&cold, burn.saturating_mul(2));

        assert_noop!(
            SubtensorModule::burned_register(
                <<Test as Config>::RuntimeOrigin>::signed(cold),
                netuid,
                hot
            ),
            Error::<Test>::NoNeuronIdAvailable
        );
    });
}

#[test]
fn test_last_update_correctness_burned() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(5);
        let tempo: u16 = 13;
        let hot = U256::from(1);
        let cold = U256::from(667);

        add_network(netuid, tempo, 0);

        // Simulate existing neurons
        let existing: u16 = 3;
        SubnetworkN::<Test>::insert(netuid, existing);

        // No LastUpdate yet
        LastUpdate::<Test>::remove(NetUidStorageIndex::from(netuid));

        // Reserves + fund
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());
        let burn = SubtensorModule::get_burn(netuid);
        SubtensorModule::add_balance_to_coldkey_account(&cold, burn.saturating_mul(5));

        // Register
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold),
            netuid,
            hot
        ));

        // LastUpdate length should be existing + 1
        assert_eq!(
            LastUpdate::<Test>::get(NetUidStorageIndex::from(netuid)).len(),
            (existing + 1) as usize
        );
    });
}

#[test]
fn test_burn_registration_increase_recycled_rao_dynamic() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = NetUid::from(6);
        let netuid2 = NetUid::from(7);

        let cold1 = U256::from(1); // Also used as hotkey
        let cold2 = U256::from(2);
        let hot1 = cold1;
        let hot2 = cold2;

        // Pre‑fund coldkeys generously
        let _ = Balances::deposit_creating(&cold1, Balance::from(1_000_000_000_000_u64));
        let _ = Balances::deposit_creating(&cold2, Balance::from(1_000_000_000_000_u64));

        // Setup reserves for both nets
        let reserve = 1_000_000_000_000u64;
        add_network(netuid1, 13, 0);
        add_network(netuid2, 13, 0);
        setup_reserves(netuid1, reserve.into(), reserve.into());
        setup_reserves(netuid2, reserve.into(), reserve.into());

        run_to_block(1);

        let burn1 = SubtensorModule::get_burn(netuid1);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            netuid1,
            hot1
        ));
        assert_eq!(SubtensorModule::get_rao_recycled(netuid1), burn1.into());

        run_to_block(2);

        let burn2 = SubtensorModule::get_burn(netuid2);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold1),
            netuid2,
            hot1
        ));
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(cold2),
            netuid2,
            hot2
        ));
        assert_eq!(
            SubtensorModule::get_rao_recycled(netuid2),
            burn2.saturating_mul(2).into()
        );
        // Validate netuid1 is not affected
        assert_eq!(SubtensorModule::get_rao_recycled(netuid1), burn1.into());
    });
}

/* -------------------------------------------------------------------------- */
/*                               Pruning behavior                              */
/* -------------------------------------------------------------------------- */

#[test]
fn test_register_prunes_lowest_score_when_full() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(8);
        add_network(netuid, 13, 0);

        SubtensorModule::set_max_allowed_uids(netuid, 2);
        SubtensorModule::set_immunity_period(netuid, 0);

        // Reserves + funds for repeated registrations
        let reserve = 1_000_000_000_000u64;
        setup_reserves(netuid, reserve.into(), reserve.into());

        let burn = SubtensorModule::get_burn(netuid);
        let payer = U256::from(1000);
        SubtensorModule::add_balance_to_coldkey_account(&payer, burn.saturating_mul(100));

        // Register two hotkeys
        let hk0 = U256::from(10);
        let hk1 = U256::from(11);

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(payer),
            netuid,
            hk0
        ));
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(payer),
            netuid,
            hk1
        ));

        // Ensure both exist and capture UIDs 0 and 1
        let uid0 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hk0).unwrap();
        let uid1 = SubtensorModule::get_uid_for_net_and_hotkey(netuid, &hk1).unwrap();
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 2);

        // Set pruning scores so uid1 is "worse" (lower) than uid0
        SubtensorModule::set_pruning_score_for_uid(netuid, uid0, 5);
        SubtensorModule::set_pruning_score_for_uid(netuid, uid1, 1);

        // Third registration should prune uid1
        let hk2 = U256::from(12);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(payer),
            netuid,
            hk2
        ));

        // Still 2 neurons
        assert_eq!(SubtensorModule::get_subnetwork_n(netuid), 2);

        // uid1 should now map to hk2
        let now_hotkey_at_uid1 = SubtensorModule::get_hotkey_for_net_and_uid(netuid, uid1).unwrap();
        assert_eq!(now_hotkey_at_uid1, hk2);

        // uid0 should still map to hk0
        let now_hotkey_at_uid0 = SubtensorModule::get_hotkey_for_net_and_uid(netuid, uid0).unwrap();
        assert_eq!(now_hotkey_at_uid0, hk0);
    });
}
