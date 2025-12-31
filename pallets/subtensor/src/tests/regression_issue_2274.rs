use frame_support::{assert_ok, traits::Currency};
use sp_core::U256;
use subtensor_runtime_common::{AlphaCurrency, BalanceOps, Currency as CurrencyT};

use super::mock::*;
use crate::*;

#[test]
fn test_burn_extrinsic_updates_subnet_alpha_out() {
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

        // add stake to coldkey-hotkey pair
        let stake = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        // get initial total issuance and alpha out
        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // amount to burn
        let burn_amount = AlphaCurrency::from(stake / 2);

        // burn
        assert_ok!(SubtensorModule::burn_alpha(
            RuntimeOrigin::signed(coldkey),
            hotkey,
            burn_amount,
            netuid
        ));

        let final_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // Assert that SubnetAlphaOut is reduced
        assert!(final_net_alpha < initial_net_alpha, "SubnetAlphaOut should be reduced after burn");
        assert_eq!(final_net_alpha, initial_net_alpha - burn_amount);
    });
}

#[test]
fn test_balance_ops_decrease_stake_consistency() {
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

        // add stake to coldkey-hotkey pair
        let stake = 200_000;
        increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake.into(), netuid);

        let initial_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // Scenario: Decrease more stake than available using BalanceOps
        let too_much = AlphaCurrency::from(stake * 2);

        // Using BalanceOps::decrease_stake directly
        let result = <SubtensorModule as BalanceOps<U256>>::decrease_stake(
            &coldkey, &hotkey, netuid, too_much
        );

        // Result should be 0 because implementation returns 0 if insufficient
        assert_eq!(result, Ok(0.into()));

        let final_net_alpha = SubnetAlphaOut::<Test>::get(netuid);

        // We expect SubnetAlphaOut to remain SAME if 0 stake was removed.
        assert_eq!(final_net_alpha, initial_net_alpha, "SubnetAlphaOut should NOT change if 0 stake removed");
    });
}
