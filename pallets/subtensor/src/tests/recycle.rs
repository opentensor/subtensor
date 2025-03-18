use frame_support::{assert_noop, assert_ok, traits::Currency};
use sp_core::U256;

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
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake, netuid);

        // get initial total issuance and alpha out
        let initial_balance = Balances::free_balance(coldkey);
        let initial_alpha = SubnetAlphaOut::<Test>::get(netuid);
        let initial_net_tao = SubnetTAO::<Test>::get(netuid);
        // preset total issuance
        TotalIssuance::<Test>::put(initial_balance + stake);
        let initial_issuance = TotalIssuance::<Test>::get();

        // amount to recycle
        let recycle_amount = stake / 2;

        // recycle
        assert_ok!(SubtensorModule::recycle(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            recycle_amount,
            netuid
        ));

        assert!(Balances::free_balance(coldkey) < initial_balance);
        assert!(SubnetAlphaOut::<Test>::get(netuid) < initial_alpha);
        assert!(SubnetTAO::<Test>::get(netuid) < initial_net_tao);
        assert!(TotalIssuance::<Test>::get() < initial_issuance);

        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::TokensRecycled(..))
            )
        }));
    });
}

#[test]
fn test_burn_success() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);

        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        let stake = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake, netuid);

        let initial_balance = Balances::free_balance(&coldkey);
        let initial_alpha = SubnetAlphaOut::<Test>::get(netuid);
        let initial_net_tao = SubnetTAO::<Test>::get(netuid);
        // preset total issuance
        TotalIssuance::<Test>::put(initial_balance + stake);
        let initial_issuance = TotalIssuance::<Test>::get();

        let burn = stake / 2;
        assert_ok!(SubtensorModule::burn(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            burn,
            netuid
        ));

        assert!(Balances::free_balance(coldkey) < initial_balance);
        assert!(SubnetAlphaOut::<Test>::get(netuid) == initial_alpha);
        assert!(SubnetTAO::<Test>::get(netuid) < initial_net_tao);
        assert!(TotalIssuance::<Test>::get() < initial_issuance);
        assert!(System::events().iter().any(|e| {
            matches!(
                &e.event,
                RuntimeEvent::SubtensorModule(Event::TokensBurned(..))
            )
        }));
    });
}

#[test]
fn test_recycle_errors() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let wrong_coldkey = U256::from(3);

        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        let stake_amount = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_amount, netuid);

        assert_noop!(
            SubtensorModule::recycle(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                100_000,
                99 // non-existent subnet
            ),
            Error::<Test>::SubNetworkDoesNotExist
        );

        assert_noop!(
            SubtensorModule::recycle(
                RuntimeOrigin::signed(wrong_coldkey),
                hotkey,
                100_000,
                netuid
            ),
            Error::<Test>::NonAssociatedColdKey
        );

        assert_noop!(
            SubtensorModule::recycle(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                10_000_000_000, // too much
                netuid
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );

        // Set AlphaOut to 0 to cause InsufficientLiquidity
        SubnetAlphaIn::<Test>::insert(netuid, 1_000_000); // Ensure there's enough alphaIn
        SubnetAlphaOut::<Test>::insert(netuid, 0); // But no alphaOut

        assert_noop!(
            SubtensorModule::recycle(RuntimeOrigin::signed(coldkey), hotkey, 100_000, netuid),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

#[test]
fn test_burn_errors() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let wrong_coldkey = U256::from(3);

        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let initial_balance = 1_000_000_000;
        Balances::make_free_balance_be(&coldkey, initial_balance);

        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        register_ok_neuron(netuid, hotkey, coldkey, 0);

        let stake_amount = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_amount, netuid);

        assert_noop!(
            SubtensorModule::burn(
                RuntimeOrigin::signed(wrong_coldkey),
                hotkey,
                100_000,
                netuid
            ),
            Error::<Test>::NonAssociatedColdKey
        );

        assert_noop!(
            SubtensorModule::burn(
                RuntimeOrigin::signed(coldkey),
                hotkey,
                10_000_000_000, // too much
                netuid
            ),
            Error::<Test>::NotEnoughStakeToWithdraw
        );
    });
}
