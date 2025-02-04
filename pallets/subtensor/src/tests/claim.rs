#![allow(clippy::unwrap_used)]
#![allow(clippy::arithmetic_side_effects)]

use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use frame_system::RawOrigin;

use super::mock::*;
use crate::*;
use approx::assert_abs_diff_eq;
use frame_support::dispatch::{DispatchClass, DispatchInfo, GetDispatchInfo, Pays};
use frame_support::sp_runtime::DispatchError;
use sp_core::{Get, U256};
use substrate_fixed::types::{U64F64};

// cargo test --package pallet-subtensor --lib -- tests::claim::test_increase_claimable_ok --exact --show-output
#[test]
fn test_increase_claimable_ok() {
    new_test_ext(1).execute_with(|| {
        let amount = 1_000_000; // 500k min + 500k fee => 500k stake
        let (netuid, _coldkey, hotkey) = setup_network_with_stake(amount);
        SubtensorModule::increase_root_claimable_for_hotkey_and_subnet(&hotkey, netuid, 500_000);

        assert_eq!(
            RootClaimable::<Test>::get(&hotkey, netuid),
            U64F64::from_num(1.0)
        );
    });
}

// cargo test --package pallet-subtensor --lib -- tests::claim::test_increase_claimable_overflow --exact --show-output
#[test]
fn test_increase_claimable_overflow() {
    new_test_ext(1).execute_with(|| {
        let amount = 1_000_000;
        let (netuid, _coldkey, hotkey) = setup_network_with_stake(amount);
        SubtensorModule::increase_root_claimable_for_hotkey_and_subnet(&hotkey, netuid, u64::MAX);
        let actual_stake = SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid);
        let expected_claimable_rate = U64F64::from_num(u64::MAX) / U64F64::from_num(actual_stake);

        assert_abs_diff_eq!(
            RootClaimable::<Test>::get(&hotkey, netuid),
            expected_claimable_rate,
            epsilon = 1_000
        );
    });
}


fn setup_network_with_stake(tao_stake: u64) -> (u16, U256, U256) {
    let subnet_owner_coldkey = U256::from(1001);
    let subnet_owner_hotkey = U256::from(1002);
    let coldkey_account_id = U256::from(1);
    let hotkey_account_id = U256::from(2);
    let fee = DefaultStakingFee::<Test>::get();
    let netuid: u16 = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);
    register_ok_neuron(netuid, hotkey_account_id, coldkey_account_id, 192213123);

    SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, tao_stake + fee + ExistentialDeposit::get());

    assert_ok!(SubtensorModule::add_stake(
        RuntimeOrigin::signed(coldkey_account_id),
        hotkey_account_id,
        netuid,
        tao_stake
    ));

    (netuid, coldkey_account_id, hotkey_account_id)
}