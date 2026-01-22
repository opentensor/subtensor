use approx::assert_abs_diff_eq;
use frame_support::{assert_noop, assert_ok, traits::Currency};
use share_pool::{SafeFloat, SafeFloatSerializable};
use sp_core::U256;
use subtensor_runtime_common::{AlphaCurrency, Currency as CurrencyT};

use super::mock;
use super::mock::*;
use crate::*;

#[test]
fn test_recycle_success() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let owner_coldkey = U256::from(1001);
        let owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        // associate coldkey and hotkey
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        let stake = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // get initial total issuance and alpha out
        let initial_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // amount to recycle
        let recycle_amount = AlphaCurrency::from(stake / 2);

        // recycle
        assert_ok!(SubtensorModule::recycle_alpha(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            recycle_amount,
            netuid
        ));

        assert!(TotalHotkeyAlpha::<Test>::get(hotkey, netuid) < initial_alpha);
        assert!(SubnetAlphaOut::<Test>::get(netuid) < initial_net_alpha);
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                < initial_alpha
        );

        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::AlphaRecycled(..))
            )
        }));
    });
}

#[test]
fn test_recycle_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let other_coldkey = U256::from(3);

        let owner_coldkey = U256::from(1001);
        let owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        // associate coldkey and hotkey
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        let stake = 200_000;
        let (expected_alpha, _) = mock::swap_tao_to_alpha(netuid, stake.into());
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // add some stake to other coldkey on same hotkey.
        increase_stake_on_coldkey_hotkey_account(&other_coldkey, &hotkey, stake.into(), netuid);

        // get initial total issuance and alpha out
        let initial_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // amount to recycle
        let recycle_amount = AlphaCurrency::from(stake / 2);

        // recycle
        assert_ok!(SubtensorModule::recycle_alpha(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            recycle_amount,
            netuid
        ));

        assert!(TotalHotkeyAlpha::<Test>::get(hotkey, netuid) < initial_alpha);
        assert!(SubnetAlphaOut::<Test>::get(netuid) < initial_net_alpha);
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                < stake.into()
        );
        // Make sure the other coldkey has no change
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &other_coldkey,
                netuid
            ),
            expected_alpha,
            epsilon = 2.into()
        );

        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::AlphaRecycled(..))
            )
        }));
    });
}

#[test]
fn test_recycle_staker_is_nominator() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let other_coldkey = U256::from(3);

        let owner_coldkey = U256::from(1001);
        let owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        // associate coldkey and hotkey
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        let stake = 200_000;
        let (expected_alpha, _) = mock::swap_tao_to_alpha(netuid, stake.into());
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // add some stake to other coldkey on same hotkey.
        // Note: this coldkey DOES NOT own the hotkey, so it is a nominator.
        increase_stake_on_coldkey_hotkey_account(&other_coldkey, &hotkey, stake.into(), netuid);
        // Verify the ownership
        assert_ne!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&hotkey),
            other_coldkey
        );

        // get initial total issuance and alpha out
        let initial_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // amount to recycle
        let recycle_amount = AlphaCurrency::from(stake / 2);

        // recycle from nominator coldkey
        assert_ok!(SubtensorModule::recycle_alpha(
            RuntimeOrigin::signed(other_coldkey),
            hotkey,
            recycle_amount,
            netuid
        ));

        assert!(TotalHotkeyAlpha::<Test>::get(hotkey, netuid) < initial_alpha);
        assert!(SubnetAlphaOut::<Test>::get(netuid) < initial_net_alpha);
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &other_coldkey,
                netuid
            ) < stake.into()
        );
        // Make sure the other coldkey has no change
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid),
            expected_alpha,
            epsilon = 2.into()
        );

        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::AlphaRecycled(..))
            )
        }));
    });
}

#[test]
fn test_burn_success() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let owner_coldkey = U256::from(1001);
        let owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        // associate coldkey and hotkey
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        let stake = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // get initial total issuance and alpha out
        let initial_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // amount to recycle
        let burn_amount = stake / 2;

        // burn
        assert_ok!(SubtensorModule::burn_alpha(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            burn_amount.into(),
            netuid
        ));

        assert!(TotalHotkeyAlpha::<Test>::get(hotkey, netuid) < initial_alpha);
        assert!(SubnetAlphaOut::<Test>::get(netuid) == initial_net_alpha);
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                < stake.into()
        );

        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::AlphaBurned(..))
            )
        }));
    });
}

#[test]
fn test_burn_staker_is_nominator() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let other_coldkey = U256::from(3);

        let owner_coldkey = U256::from(1001);
        let owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        // associate coldkey and hotkey
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        let stake = 200_000;
        let (expected_alpha, _) = mock::swap_tao_to_alpha(netuid, stake.into());
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // add some stake to other coldkey on same hotkey.
        // Note: this coldkey DOES NOT own the hotkey, so it is a nominator.
        increase_stake_on_coldkey_hotkey_account(&other_coldkey, &hotkey, stake.into(), netuid);

        // get initial total issuance and alpha out
        let initial_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // amount to recycle
        let burn_amount = AlphaCurrency::from(stake / 2);

        // burn from nominator coldkey
        assert_ok!(SubtensorModule::burn_alpha(
            RuntimeOrigin::signed(other_coldkey),
            hotkey,
            burn_amount,
            netuid
        ));

        assert!(TotalHotkeyAlpha::<Test>::get(hotkey, netuid) < initial_alpha);
        assert!(SubnetAlphaOut::<Test>::get(netuid) == initial_net_alpha);
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &other_coldkey,
                netuid
            ) < stake.into()
        );
        // Make sure the other coldkey has no change
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid),
            expected_alpha,
            epsilon = 2.into()
        );

        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::AlphaBurned(..))
            )
        }));
    });
}

#[test]
fn test_burn_two_stakers() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let other_coldkey = U256::from(3);

        let owner_coldkey = U256::from(1001);
        let owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        // associate coldkey and hotkey
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        let stake = 200_000;
        let (expected_alpha, _) = mock::swap_tao_to_alpha(netuid, stake.into());
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // add some stake to other coldkey on same hotkey.
        increase_stake_on_coldkey_hotkey_account(&other_coldkey, &hotkey, stake.into(), netuid);

        // get initial total issuance and alpha out
        let initial_alpha = TotalHotkeyAlpha::<Test>::get(hotkey, netuid);
        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // amount to recycle
        let burn_amount = AlphaCurrency::from(stake / 2);

        // burn from coldkey
        assert_ok!(SubtensorModule::burn_alpha(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            burn_amount,
            netuid
        ));

        assert!(TotalHotkeyAlpha::<Test>::get(hotkey, netuid) < initial_alpha);
        assert!(SubnetAlphaOut::<Test>::get(netuid) == initial_net_alpha);
        assert!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid)
                < stake.into()
        );
        // Make sure the other coldkey has no change
        assert_abs_diff_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &other_coldkey,
                netuid
            ),
            expected_alpha,
            epsilon = 2.into()
        );

        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::AlphaBurned(..))
            )
        }));
    });
}

#[test]
fn test_recycle_errors() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let wrong_hotkey = U256::from(3);

        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Create root subnet
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        let stake_amount = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_amount.into(), netuid);

        assert_noop!(
            SubtensorModule::recycle_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                100_000.into(),
                99.into() // non-existent subnet
            ),
            Error::<Test>::SubnetNotExists
        );

        assert_noop!(
            SubtensorModule::recycle_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                100_000.into(),
                NetUid::ROOT,
            ),
            Error::<Test>::CannotBurnOrRecycleOnRootSubnet
        );

        assert_noop!(
            SubtensorModule::recycle_alpha(
                RuntimeOrigin::signed(coldkey),
                wrong_hotkey,
                100_000.into(),
                netuid
            ),
            Error::<Test>::HotKeyAccountNotExists
        );

        // make it pass the stake check
        TotalHotkeyAlpha::<Test>::set(
            hotkey,
            netuid,
            SubnetAlphaOut::<Test>::get(netuid).saturating_mul(2.into()),
        );

        assert_noop!(
            SubtensorModule::recycle_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                SubnetAlphaOut::<Test>::get(netuid) + 1.into(),
                netuid
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn test_burn_errors() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let wrong_hotkey = U256::from(3);

        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        // Create root subnet
        migrations::migrate_create_root_network::migrate_create_root_network::<Test>();

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        let stake_amount = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_amount.into(), netuid);

        assert_noop!(
            SubtensorModule::burn_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                100_000.into(),
                99.into() // non-existent subnet
            ),
            Error::<Test>::SubnetNotExists
        );

        assert_noop!(
            SubtensorModule::burn_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                100_000.into(),
                NetUid::ROOT,
            ),
            Error::<Test>::CannotBurnOrRecycleOnRootSubnet
        );

        assert_noop!(
            SubtensorModule::burn_alpha(
                RuntimeOrigin::signed(coldkey),
                wrong_hotkey,
                100_000.into(),
                netuid
            ),
            Error::<Test>::HotKeyAccountNotExists
        );

        // make it pass the hotkey alpha check
        TotalHotkeyAlpha::<Test>::set(
            hotkey,
            netuid,
            SubnetAlphaOut::<Test>::get(netuid).saturating_mul(2.into()),
        );

        assert_noop!(
            SubtensorModule::burn_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                SubnetAlphaOut::<Test>::get(netuid) + 1.into(),
                netuid
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn test_recycle_precision() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let stake = 200_000_u64;
        let tao_reserve = TaoCurrency::from(1_000_000_000_u64);
        let alpha_reserve = AlphaCurrency::from(1_000_000_000_u64);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_reserve);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);

        Balances::make_free_balance_be(&coldkey, 1_000_000_000);
        // sanity check
        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // amount to recycle
        let recycle_amount = AlphaCurrency::from(stake / 2);

        // Modify the alpha pool denominator so it's low-precision (denominator = share = 1e-9)
        let denominator = SafeFloat::from(1).div(&SafeFloat::from(1_000_000_000)).unwrap_or_default();
        TotalHotkeySharesV2::<Test>::insert(hotkey, netuid, SafeFloatSerializable::from(&denominator));
        AlphaV2::<Test>::insert((&hotkey, &coldkey, netuid), SafeFloatSerializable::from(&denominator));
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, AlphaCurrency::from(stake));

        // recycle, expect error due to precision loss
        assert_ok!(
            SubtensorModule::recycle_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                recycle_amount,
                netuid
            )
        );

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid),
            AlphaCurrency::from(stake / 2)
        );
    });
}

#[test]
fn test_burn_precision() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let netuid = add_dynamic_network(&hotkey, &coldkey);
        let stake = 200_000_u64;
        let tao_reserve = TaoCurrency::from(1_000_000_000_u64);
        let alpha_reserve = AlphaCurrency::from(1_000_000_000_u64);
        SubnetAlphaIn::<Test>::insert(netuid, alpha_reserve);
        SubnetTAO::<Test>::insert(netuid, tao_reserve);

        Balances::make_free_balance_be(&coldkey, 1_000_000_000);
        // sanity check
        assert!(SubtensorModule::if_subnet_exist(netuid));

        // add stake to coldkey-hotkey pair so we can recycle it
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // amount to recycle
        let burn_amount = AlphaCurrency::from(stake / 2);

        // Modify the alpha pool denominator so it's low-precision (denominator = share = 1e-9)
        let denominator = SafeFloat::from(1).div(&SafeFloat::from(1_000_000_000)).unwrap_or_default();
        TotalHotkeySharesV2::<Test>::insert(hotkey, netuid, SafeFloatSerializable::from(&denominator));
        AlphaV2::<Test>::insert((&hotkey, &coldkey, netuid), SafeFloatSerializable::from(&denominator));
        TotalHotkeyAlpha::<Test>::insert(hotkey, netuid, AlphaCurrency::from(stake));

        // burn, expect error due to precision loss
        assert_ok!(
            SubtensorModule::burn_alpha(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                burn_amount,
                netuid
            )
        );

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid),
            AlphaCurrency::from(stake / 2)
        );
    });
}
