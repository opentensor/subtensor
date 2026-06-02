#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use frame_support::{assert_ok, weights::Weight};
use sp_core::U256;
use subtensor_runtime_common::TaoBalance;
use subtensor_swap_interface::SwapHandler;

#[test]
fn test_destroy_alpha_in_out_stakes_get_total_alpha_value() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(1001);
        let owner_hot = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);

        // Add some stake to have alpha value
        let stake_tao: u64 = 1000;
        setup_reserves(netuid, (stake_tao * 1_000_000).into(), (stake_tao * 10_000_000).into());
        let amount: TaoBalance = (stake_tao).into();
        assert_ok!(SubtensorModule::create_account_if_non_existent(&owner_cold, &owner_hot));
        add_balance_to_coldkey_account(&owner_cold, amount);
        // Stake into subnet to create some alpha
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hot,
            &owner_cold,
            netuid,
            amount,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        ));

        // Now test the function
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = frame_support::weights::WeightMeter::with_limit(w);
        let result = SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(netuid, &mut weight_meter);
        assert!(result, "destroy_alpha_in_out_stakes_get_total_alpha_value should return true when there is alpha to process");
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_settle_stakes() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(1001);
        let owner_hot = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);

        // Add some stake to have alpha value
        let stake_tao: u64 = 1000;
        setup_reserves(netuid, (stake_tao * 1_000_000).into(), (stake_tao * 10_000_000).into());
        let amount: TaoBalance = (stake_tao).into();
        assert_ok!(SubtensorModule::create_account_if_non_existent(&owner_cold, &owner_hot));
        add_balance_to_coldkey_account(&owner_cold, amount);
        // Stake into subnet to create some alpha
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hot,
            &owner_cold,
            netuid,
            amount,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        ));

        // First, we need to get the total alpha value (simulate the previous step)
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(netuid, &mut weight_meter);
        // Now test the settle_stakes function
        let mut weight_meter2 = frame_support::weights::WeightMeter::with_limit(w);
        let result = SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(netuid, &mut weight_meter2);
        assert!(result, "destroy_alpha_in_out_stakes_settle_stakes should return true when there is alpha to settle");
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_clean_alpha() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(1001);
        let owner_hot = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);

        // Add some stake to have alpha value
        let stake_tao: u64 = 1000;
        setup_reserves(netuid, (stake_tao * 1_000_000).into(), (stake_tao * 10_000_000).into());
        let amount: TaoBalance = (stake_tao).into();
        assert_ok!(SubtensorModule::create_account_if_non_existent(&owner_cold, &owner_hot));
        add_balance_to_coldkey_account(&owner_cold, amount);
        // Stake into subnet to create some alpha
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hot,
            &owner_cold,
            netuid,
            amount,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        ));

        // Simulate the previous two steps: get total alpha and settle stakes
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(netuid, &mut weight_meter);
        let mut weight_meter2 = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(netuid, &mut weight_meter2);
        // Now test the clean_alpha function
        let mut weight_meter3 = frame_support::weights::WeightMeter::with_limit(w);
        let result = SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha(netuid, &mut weight_meter3);
        assert!(result, "destroy_alpha_in_out_stakes_clean_alpha should return true when there is alpha to clean");
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_clear_hotkey_totals() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(1001);
        let owner_hot = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);

        // Add some stake to have alpha value and hotkey totals
        let stake_tao: u64 = 1000;
        setup_reserves(netuid, (stake_tao * 1_000_000).into(), (stake_tao * 10_000_000).into());
        let amount: TaoBalance = (stake_tao).into();
        assert_ok!(SubtensorModule::create_account_if_non_existent(&owner_cold, &owner_hot));
        add_balance_to_coldkey_account(&owner_cold, amount);
        // Stake into subnet to create some alpha and hotkey totals
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hot,
            &owner_cold,
            netuid,
            amount,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        ));

        // Simulate the previous three steps
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(netuid, &mut weight_meter);
        let mut weight_meter2 = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(netuid, &mut weight_meter2);
        let mut weight_meter3 = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha(netuid, &mut weight_meter3);
        // Now test the clear_hotkey_totals function
        let mut weight_meter4 = frame_support::weights::WeightMeter::with_limit(w);
        let result = SubtensorModule::destroy_alpha_in_out_stakes_clear_hotkey_totals(netuid, &mut weight_meter4);
        assert!(result, "destroy_alpha_in_out_stakes_clear_hotkey_totals should return true when there are hotkey totals to clear");
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes_clear_locks() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(1001);
        let owner_hot = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);

        // Add some stake to have alpha value and create locks
        let stake_tao: u64 = 1000;
        setup_reserves(netuid, (stake_tao * 1_000_000).into(), (stake_tao * 10_000_000).into());
        let amount: TaoBalance = (stake_tao).into();
        assert_ok!(SubtensorModule::create_account_if_non_existent(&owner_cold, &owner_hot));
        add_balance_to_coldkey_account(&owner_cold, amount);
        // Stake into subnet to create some alpha and locks
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hot,
            &owner_cold,
            netuid,
            amount,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        ));

        // Simulate the previous four steps
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_get_total_alpha_value(netuid, &mut weight_meter);
        let mut weight_meter2 = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_settle_stakes(netuid, &mut weight_meter2);
        let mut weight_meter3 = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_clean_alpha(netuid, &mut weight_meter3);
        let mut weight_meter4 = frame_support::weights::WeightMeter::with_limit(w);
        let _ = SubtensorModule::destroy_alpha_in_out_stakes_clear_hotkey_totals(netuid, &mut weight_meter4);
        // Now test the clear_locks function
        let mut weight_meter5 = frame_support::weights::WeightMeter::with_limit(w);
        let result = SubtensorModule::destroy_alpha_in_out_stakes_clear_locks(netuid, &mut weight_meter5);
        assert!(result, "destroy_alpha_in_out_stakes_clear_locks should return true when there are locks to clear");
    });
}

#[test]
fn test_destroy_alpha_in_out_stakes() {
    new_test_ext(0).execute_with(|| {
        let owner_cold = U256::from(1001);
        let owner_hot = U256::from(1002);
        let netuid = add_dynamic_network(&owner_hot, &owner_cold);

        // Add some stake to have alpha value and create locks, etc.
        let stake_tao: u64 = 1000;
        setup_reserves(netuid, (stake_tao * 1_000_000).into(), (stake_tao * 10_000_000).into());
        let amount: TaoBalance = (stake_tao).into();
        assert_ok!(SubtensorModule::create_account_if_non_existent(&owner_cold, &owner_hot));
        add_balance_to_coldkey_account(&owner_cold, amount);
        // Stake into subnet to create some alpha and locks
        assert_ok!(SubtensorModule::stake_into_subnet(
            &owner_hot,
            &owner_cold,
            netuid,
            amount,
            <Test as Config>::SwapInterface::max_price(),
            false,
            false,
        ));

        // Now test the main destroy function (which should call all the steps internally)
        let w = Weight::from_parts(u64::MAX, u64::MAX);
        let mut weight_meter = frame_support::weights::WeightMeter::with_limit(w);
        DissolvedSubnetTotalAlphaValue::<Test>::set(Some(0)); 
        DissolvedSubnetDistributedTao::<Test>::set(Some(0));
        let result = SubtensorModule::destroy_alpha_in_out_stakes(netuid, &mut weight_meter);
        assert!(result, "destroy_alpha_in_out_stakes should return true when it successfully processes the netuid");
    });
}
