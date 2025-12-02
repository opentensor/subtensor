#![allow(
    unused,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

use approx::assert_abs_diff_eq;
use codec::Encode;
use frame_support::dispatch::{DispatchInfo, GetDispatchInfo};
use frame_support::error::BadOrigin;
use frame_support::traits::OnInitialize;
use frame_support::traits::schedule::DispatchTime;
use frame_support::traits::schedule::v3::Named as ScheduleNamed;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{Config, RawOrigin};
use sp_core::{Get, H256, U256};
use sp_runtime::traits::{DispatchInfoOf, DispatchTransaction, TransactionExtension};
use sp_runtime::{DispatchError, traits::TxBaseImplication};
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, SubnetInfo, TaoCurrency};
use subtensor_swap_interface::{SwapEngine, SwapHandler};

use super::mock;
use super::mock::*;
use crate::transaction_extension::SubtensorTransactionExtension;
use crate::*;
use crate::{Call, ColdkeySwapScheduleDuration, Error};

#[test]
fn test_announce_coldkey_swap_with_no_announcement_works() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);

        assert_eq!(ColdkeySwapAnnouncements::<Test>::iter().count(), 0);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who.clone()),
            new_coldkey,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who.clone(), (now, new_coldkey))]
        );
        assert_eq!(
            last_event(),
            RuntimeEvent::SubtensorModule(Event::ColdkeySwapAnnounced {
                who,
                new_coldkey,
                block_number: now,
            })
        );
    });
}

#[test]
fn test_announce_coldkey_swap_with_existing_announcement_past_delay_works() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_2 = U256::from(3);

        assert_eq!(ColdkeySwapAnnouncements::<Test>::iter().count(), 0);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who.clone()),
            new_coldkey,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who.clone(), (now, new_coldkey))]
        );

        let delay = ColdkeySwapScheduleDuration::<Test>::get() + 1;
        System::run_to_block::<AllPalletsWithSystem>(now + delay);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who.clone()),
            new_coldkey_2,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who.clone(), (now, new_coldkey_2))]
        );
    });
}

#[test]
fn test_announce_coldkey_swap_with_existing_announcement_not_past_delay_fails() {
    new_test_ext(1).execute_with(|| {
        let who = U256::from(1);
        let new_coldkey = U256::from(2);
        let new_coldkey_2 = U256::from(3);

        assert_eq!(ColdkeySwapAnnouncements::<Test>::iter().count(), 0);

        assert_ok!(SubtensorModule::announce_coldkey_swap(
            RuntimeOrigin::signed(who.clone()),
            new_coldkey,
        ));

        let now = System::block_number();
        assert_eq!(
            ColdkeySwapAnnouncements::<Test>::iter().collect::<Vec<_>>(),
            vec![(who.clone(), (now, new_coldkey))]
        );

        let unmet_delay = ColdkeySwapScheduleDuration::<Test>::get();
        System::run_to_block::<AllPalletsWithSystem>(now + unmet_delay);

        assert_noop!(
            SubtensorModule::announce_coldkey_swap(
                RuntimeOrigin::signed(who.clone()),
                new_coldkey_2,
            ),
            Error::<Test>::ColdkeySwapReannouncedTooEarly
        );
    });
}

#[test]
fn test_announce_coldkey_swap_with_bad_origin_fails() {
    new_test_ext(1).execute_with(|| {
        let new_coldkey = U256::from(1);

        assert_noop!(
            SubtensorModule::announce_coldkey_swap(RuntimeOrigin::none(), new_coldkey),
            BadOrigin
        );

        assert_noop!(
            SubtensorModule::announce_coldkey_swap(RuntimeOrigin::root(), new_coldkey),
            BadOrigin
        );
    });
}

#[test]
fn test_swap_subnet_owner() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let netuid = NetUid::from(1u16);

        add_network(netuid, 1, 0);
        SubnetOwner::<Test>::insert(netuid, old_coldkey);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(SubnetOwner::<Test>::get(netuid), new_coldkey);
    });
}

#[test]
fn test_swap_total_coldkey_stake() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let other_coldkey = U256::from(3);
        let hotkey = U256::from(4);
        let other_hotkey = U256::from(5);
        let stake = DefaultMinStake::<Test>::get().to_u64() * 10;

        let netuid = NetUid::from(1u16);
        add_network(netuid, 1, 0);
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, stake * 2 + 1_000);
        register_ok_neuron(netuid, hotkey, old_coldkey, 1001000);
        register_ok_neuron(netuid, other_hotkey, other_coldkey, 1001000);

        let reserve = stake * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey,
            netuid,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            other_hotkey,
            netuid,
            stake.into()
        ));
        let total_stake_before_swap = SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            total_stake_before_swap
        );
    });
}

#[test]
fn test_swap_staking_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        StakingHotkeys::<Test>::insert(old_coldkey, vec![hotkey]);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert!(StakingHotkeys::<Test>::get(old_coldkey).is_empty());
        assert_eq!(StakingHotkeys::<Test>::get(new_coldkey), vec![hotkey]);
    });
}

#[test]
fn test_swap_hotkey_owners() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);

        Owner::<Test>::insert(hotkey, old_coldkey);
        OwnedHotkeys::<Test>::insert(old_coldkey, vec![hotkey]);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(Owner::<Test>::get(hotkey), new_coldkey);
        assert!(OwnedHotkeys::<Test>::get(old_coldkey).is_empty());
        assert_eq!(OwnedHotkeys::<Test>::get(new_coldkey), vec![hotkey]);
    });
}

#[test]
fn test_transfer_remaining_balance() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let balance = 100;

        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, balance);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(SubtensorModule::get_coldkey_balance(&old_coldkey), 0);
        assert_eq!(SubtensorModule::get_coldkey_balance(&new_coldkey), balance);
    });
}

#[test]
fn test_swap_with_no_stake() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            TaoCurrency::ZERO
        );
    });
}

#[test]
fn test_swap_with_multiple_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);

        OwnedHotkeys::<Test>::insert(old_coldkey, vec![hotkey1, hotkey2]);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert!(OwnedHotkeys::<Test>::get(old_coldkey).is_empty());
        assert_eq!(
            OwnedHotkeys::<Test>::get(new_coldkey),
            vec![hotkey1, hotkey2]
        );
    });
}

#[test]
fn test_swap_with_multiple_subnets() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        add_network(netuid1, 1, 0);
        add_network(netuid2, 1, 0);
        SubnetOwner::<Test>::insert(netuid1, old_coldkey);
        SubnetOwner::<Test>::insert(netuid2, old_coldkey);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(SubnetOwner::<Test>::get(netuid1), new_coldkey);
        assert_eq!(SubnetOwner::<Test>::get(netuid2), new_coldkey);
    });
}

// TODO
#[test]
fn test_swap_with_zero_balance() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        println!(
            "old_coldkey balance: {}",
            Balances::free_balance(old_coldkey)
        );
        println!(
            "new_coldkey balance: {}",
            Balances::free_balance(new_coldkey)
        );

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        println!(
            "old_coldkey balance: {}",
            Balances::free_balance(old_coldkey)
        );
        println!(
            "new_coldkey balance: {}",
            Balances::free_balance(new_coldkey)
        );

        assert_eq!(Balances::free_balance(old_coldkey), 0);
        assert_eq!(Balances::free_balance(new_coldkey), 0);
    });
}

#[test]
fn test_swap_with_max_values() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let old_coldkey2 = U256::from(3);
        let new_coldkey2 = U256::from(4);
        let hotkey = U256::from(5);
        let hotkey2 = U256::from(6);
        let other_coldkey = U256::from(7);
        let netuid = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let stake = 10_000;
        let max_stake = 21_000_000_000_000_000; // 21 Million TAO; max possible balance.

        // Add a network
        add_network(netuid, 1, 0);
        add_network(netuid2, 1, 0);

        // Register hotkey on each subnet.
        // hotkey2 is owned by other_coldkey.
        register_ok_neuron(netuid, hotkey, old_coldkey, 1001000);
        register_ok_neuron(netuid2, hotkey2, other_coldkey, 1001000);

        // Give balance to old_coldkey and old_coldkey2.
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, max_stake + 1_000);
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey2, max_stake + 1_000);

        let reserve = max_stake * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());
        mock::setup_reserves(netuid2, reserve.into(), reserve.into());

        // Stake to hotkey on each subnet.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey,
            netuid,
            max_stake.into()
        ));
        let expected_stake1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &old_coldkey,
            netuid,
        );

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey2),
            hotkey2,
            netuid2,
            max_stake.into()
        ));
        let expected_stake2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey2,
            netuid2,
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey2, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));
        assert_ok!(SubtensorModule::do_swap_coldkey(
            &old_coldkey2,
            &new_coldkey2
        ));

        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            expected_stake1.to_u64().into(),
            epsilon = TaoCurrency::from(expected_stake1.to_u64()) / 1000.into()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey2),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey2),
            expected_stake2.to_u64().into(),
            epsilon = TaoCurrency::from(expected_stake2.to_u64()) / 1000.into()
        );
    });
}

#[test]
fn test_swap_with_non_existent_new_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let stake = DefaultMinStake::<Test>::get().to_u64() * 10;
        let netuid = NetUid::from(1);

        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey, old_coldkey, 1001000);
        // Give old coldkey some balance.
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, stake + 1_000);

        let reserve = stake * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Stake to hotkey.
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey,
            netuid,
            stake.into()
        ));
        let expected_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &old_coldkey,
            netuid,
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey));

        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );

        let actual_stake = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &new_coldkey,
            netuid,
        );
        assert_abs_diff_eq!(
            actual_stake,
            expected_stake,
            epsilon = expected_stake / 1000.into()
        );
    });
}

#[test]
fn test_swap_with_max_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let max_hotkeys = 1000;
        let hotkeys: Vec<U256> = (0..max_hotkeys).map(U256::from).collect();

        OwnedHotkeys::<Test>::insert(old_coldkey, hotkeys.clone());

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey));

        assert!(OwnedHotkeys::<Test>::get(old_coldkey).is_empty());
        assert_eq!(OwnedHotkeys::<Test>::get(new_coldkey), hotkeys);
    });
}

#[test]
fn test_swap_effect_on_delegated_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_coldkey = U256::from(1001);
        let subnet_owner_hotkey = U256::from(1002);
        let netuid = add_dynamic_network(&subnet_owner_hotkey, &subnet_owner_coldkey);

        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let delegator = U256::from(3);
        let hotkey = U256::from(4);
        let stake = 100_000_000_000;

        StakingHotkeys::<Test>::insert(old_coldkey, vec![hotkey]);
        StakingHotkeys::<Test>::insert(delegator, vec![hotkey]);
        SubtensorModule::create_account_if_non_existent(&old_coldkey, &hotkey);
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, stake);
        SubtensorModule::add_balance_to_coldkey_account(&delegator, stake);

        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(old_coldkey),
            hotkey,
            netuid,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            RuntimeOrigin::signed(delegator),
            hotkey,
            netuid,
            stake.into()
        ));
        let coldkey_stake_before = SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);
        let delegator_stake_before = SubtensorModule::get_total_stake_for_coldkey(&delegator);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            coldkey_stake_before,
            epsilon = 500.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&delegator),
            delegator_stake_before,
            epsilon = 500.into()
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO,
            epsilon = 500.into()
        );
    });
}

// TODO
#[test]
fn test_swap_concurrent_modifications() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let netuid = NetUid::from(1);
        let initial_stake = 1_000_000_000_000;
        let additional_stake = 500_000_000_000;

        let reserve = (initial_stake + additional_stake) * 1000;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Setup initial state
        add_network(netuid, 1, 1);
        SubtensorModule::add_balance_to_coldkey_account(
            &new_coldkey,
            initial_stake + additional_stake + 1_000_000,
        );
        register_ok_neuron(netuid, hotkey, new_coldkey, 1001000);

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(new_coldkey),
            hotkey,
            netuid,
            initial_stake.into()
        ));

        // Verify initial stake
        let stake_before_swap = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &new_coldkey,
            netuid,
        );

        // Wait some blocks
        step_block(10);

        // Simulate concurrent stake addition
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(new_coldkey),
            hotkey,
            netuid,
            additional_stake.into()
        ));

        let stake_with_additional = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            &new_coldkey,
            netuid,
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &new_coldkey,
                netuid
            ),
            stake_with_additional
        );
        assert!(stake_with_additional > stake_before_swap);
        assert!(!Alpha::<Test>::contains_key((hotkey, old_coldkey, netuid)));
    });
}

#[test]
fn test_swap_with_invalid_subnet_ownership() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let netuid = NetUid::from(1u16);

        SubnetOwner::<Test>::insert(netuid, old_coldkey);

        // Simulate an invalid state where the subnet owner doesn't match the old_coldkey
        SubnetOwner::<Test>::insert(netuid, U256::from(3));

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // The swap should not affect the mismatched subnet ownership
        assert_eq!(SubnetOwner::<Test>::get(netuid), U256::from(3));
    });
}

#[test]
fn test_do_swap_coldkey_success() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);
        let netuid = NetUid::from(1u16);
        let stake_amount1 = DefaultMinStake::<Test>::get().to_u64() * 10;
        let stake_amount2 = DefaultMinStake::<Test>::get().to_u64() * 20;
        let swap_cost = SubtensorModule::get_key_swap_cost();
        let free_balance_old = 12345 + swap_cost.to_u64();

        let reserve = (stake_amount1 + stake_amount2) * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Setup initial state
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey1, old_coldkey, 0);
        register_ok_neuron(netuid, hotkey2, old_coldkey, 0);

        // Add balance to old coldkey
        SubtensorModule::add_balance_to_coldkey_account(
            &old_coldkey,
            stake_amount1 + stake_amount2 + free_balance_old,
        );

        // Log initial state
        log::info!(
            "Initial total stake: {}",
            SubtensorModule::get_total_stake()
        );
        log::info!(
            "Initial old coldkey stake: {}",
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey)
        );
        log::info!(
            "Initial new coldkey stake: {}",
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey)
        );

        // Add stake to the neurons
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey1,
            netuid,
            stake_amount1.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey2,
            netuid,
            stake_amount2.into()
        ));

        // Insert an Identity
        let name: Vec<u8> = b"The fourth Coolest Identity".to_vec();
        let identity: ChainIdentityV2 = ChainIdentityV2 {
            name: name.clone(),
            url: vec![],
            github_repo: vec![],
            image: vec![],
            discord: vec![],
            description: vec![],
            additional: vec![],
        };

        IdentitiesV2::<Test>::insert(old_coldkey, identity.clone());

        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_some());
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_none());

        // Log state after adding stake
        log::info!(
            "Total stake after adding: {}",
            SubtensorModule::get_total_stake()
        );
        log::info!(
            "Old coldkey stake after adding: {}",
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey)
        );
        log::info!(
            "New coldkey stake after adding: {}",
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey)
        );

        // Record total stake before swap
        let total_stake_before_swap = SubtensorModule::get_total_stake();

        let hk1_alpha = Alpha::<Test>::get((hotkey1, old_coldkey, netuid));
        let hk2_alpha = Alpha::<Test>::get((hotkey2, old_coldkey, netuid));
        let total_ck_stake = SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Log state after swap
        log::info!(
            "Total stake after swap: {}",
            SubtensorModule::get_total_stake()
        );
        log::info!(
            "Old coldkey stake after swap: {}",
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey)
        );
        log::info!(
            "New coldkey stake after swap: {}",
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey)
        );

        // Verify the swap
        assert_eq!(Owner::<Test>::get(hotkey1), new_coldkey);
        assert_eq!(Owner::<Test>::get(hotkey2), new_coldkey);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            total_ck_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey1, new_coldkey, netuid)),
            hk1_alpha
        );
        assert_eq!(
            Alpha::<Test>::get((hotkey2, new_coldkey, netuid)),
            hk2_alpha
        );
        assert!(!Alpha::<Test>::contains_key((hotkey1, old_coldkey, netuid)));
        assert!(!Alpha::<Test>::contains_key((hotkey2, old_coldkey, netuid)));

        // Verify OwnedHotkeys
        let new_owned_hotkeys = OwnedHotkeys::<Test>::get(new_coldkey);
        assert!(new_owned_hotkeys.contains(&hotkey1));
        assert!(new_owned_hotkeys.contains(&hotkey2));
        assert_eq!(new_owned_hotkeys.len(), 2);
        assert!(!OwnedHotkeys::<Test>::contains_key(old_coldkey));

        // Verify balance transfer
        assert_eq!(
            SubtensorModule::get_coldkey_balance(&new_coldkey),
            free_balance_old - swap_cost.to_u64()
        );
        assert_eq!(SubtensorModule::get_coldkey_balance(&old_coldkey), 0);

        // Verify total stake remains unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            total_stake_before_swap,
            "Total stake changed unexpectedly"
        );

        // Verify identities were swapped
        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_none());
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_some());
        assert_eq!(
            IdentitiesV2::<Test>::get(new_coldkey).expect("Expected an Identity"),
            identity
        );

        // Verify event emission
        System::assert_last_event(
            Event::ColdkeySwapped {
                old_coldkey,
                new_coldkey,
                swap_cost,
            }
            .into(),
        );
    });
}

#[test]
fn test_swap_stake_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey1 = U256::from(3);
        let hotkey2 = U256::from(4);
        let stake_amount1 = DefaultMinStake::<Test>::get().to_u64() * 10;
        let stake_amount2 = DefaultMinStake::<Test>::get().to_u64() * 20;
        let stake_amount3 = DefaultMinStake::<Test>::get().to_u64() * 30;

        // Setup initial state
        // Add a network
        let netuid = NetUid::from(1u16);
        add_network(netuid, 1, 0);

        // Register hotkeys
        register_ok_neuron(netuid, hotkey1, old_coldkey, 0);
        register_ok_neuron(netuid, hotkey2, old_coldkey, 0);
        // Give some balance to old coldkey
        SubtensorModule::add_balance_to_coldkey_account(
            &old_coldkey,
            stake_amount1 + stake_amount2 + 1_000_000,
        );

        let reserve = (stake_amount1 + stake_amount2 + stake_amount3) * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Stake to hotkeys
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey1,
            netuid,
            stake_amount1.into()
        ));
        let expected_stake_alpha1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &old_coldkey,
            netuid,
        );

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey2,
            netuid,
            stake_amount2.into()
        ));
        let expected_stake_alpha2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey,
            netuid,
        );

        // Insert existing for same hotkey1
        // give new coldkey some balance
        SubtensorModule::add_balance_to_coldkey_account(&new_coldkey, stake_amount3 + 1_000_000);
        // Stake to hotkey1
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(new_coldkey),
            hotkey1,
            netuid,
            stake_amount3.into()
        ));
        let expected_stake_alpha3 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &new_coldkey,
            netuid,
        );

        // Record initial values
        let initial_total_issuance = SubtensorModule::get_total_issuance();
        let initial_total_stake = SubtensorModule::get_total_stake();
        let initial_total_stake_for_old_coldkey =
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);
        let initial_total_stake_for_new_coldkey =
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey);
        let initial_total_hotkey1_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey1);
        let initial_total_hotkey2_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey2);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Verify stake is additive, not replaced
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            initial_total_stake_for_old_coldkey + initial_total_stake_for_new_coldkey,
            epsilon = 2.into()
        );

        // Verify ownership transfer
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2]
        );
        assert_eq!(SubtensorModule::get_owned_hotkeys(&old_coldkey), vec![]);

        // Verify stake transfer
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &new_coldkey,
                netuid
            ),
            expected_stake_alpha1 + expected_stake_alpha3
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2,
                &new_coldkey,
                netuid
            ),
            expected_stake_alpha2
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &old_coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2,
                &old_coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );

        // Verify TotalHotkeyStake remains unchanged
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey1),
            initial_total_hotkey1_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey2),
            initial_total_hotkey2_stake
        );

        // Verify total stake and issuance remain unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            initial_total_stake,
            "Total stake changed unexpectedly"
        );
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_total_issuance,
            "Total issuance changed unexpectedly"
        );
    });
}

#[test]
fn test_swap_staking_hotkeys_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let other_coldkey = U256::from(3);
        let hotkey1 = U256::from(4);
        let hotkey2 = U256::from(5);
        let stake_amount1 = DefaultMinStake::<Test>::get().to_u64() * 10;
        let stake_amount2 = DefaultMinStake::<Test>::get().to_u64() * 20;

        // Setup initial state
        // Add a network
        let netuid = NetUid::from(1u16);
        add_network(netuid, 1, 0);
        // Give some balance to old coldkey
        SubtensorModule::add_balance_to_coldkey_account(
            &old_coldkey,
            stake_amount1 + stake_amount2 + 1_000_000,
        ); // Register hotkeys
        register_ok_neuron(netuid, hotkey1, old_coldkey, 0);
        register_ok_neuron(netuid, hotkey2, other_coldkey, 0);

        let reserve = (stake_amount1 + stake_amount2) * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Stake to hotkeys
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey1,
            netuid,
            stake_amount1.into()
        ));
        let expected_stake_alpha1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &old_coldkey,
            netuid,
        );

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey2,
            netuid,
            stake_amount2.into()
        ));
        let expected_stake_alpha2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey,
            netuid,
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Verify StakingHotkeys transfer
        assert_eq!(
            StakingHotkeys::<Test>::get(new_coldkey),
            vec![hotkey1, hotkey2]
        );
        assert_eq!(StakingHotkeys::<Test>::get(old_coldkey), vec![]);
    });
}

#[test]
fn test_swap_delegated_stake_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let other_coldkey = U256::from(3);
        let hotkey1 = U256::from(4);
        let hotkey2 = U256::from(5);
        let stake_amount1 = DefaultMinStake::<Test>::get().to_u64() * 10;
        let stake_amount2 = DefaultMinStake::<Test>::get().to_u64() * 20;
        let netuid = NetUid::from(1);

        // Setup initial state
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey1, other_coldkey, 0);
        register_ok_neuron(netuid, hotkey2, other_coldkey, 0);

        let reserve = (stake_amount1 + stake_amount2) * 10;
        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        // Notice hotkey1 and hotkey2 are Owned by other_coldkey
        // old_coldkey and new_coldkey therefore delegates stake to them
        // === Give old_coldkey some balance ===
        SubtensorModule::add_balance_to_coldkey_account(
            &old_coldkey,
            stake_amount1 + stake_amount2 + 1_000_000,
        );

        // === Stake to hotkeys ===
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey1,
            netuid,
            stake_amount1.into()
        ));
        let expected_stake_alpha1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &old_coldkey,
            netuid,
        );

        let (expected_stake_alpha2, fee) = mock::swap_tao_to_alpha(netuid, stake_amount2.into());
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey2,
            netuid,
            stake_amount2.into()
        ));
        let expected_stake_alpha2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey,
            netuid,
        );
        let fee = (expected_stake_alpha2.to_u64() as f64 * 0.003) as u64;

        // Record initial values
        let initial_total_issuance = SubtensorModule::get_total_issuance();
        let initial_total_stake = SubtensorModule::get_total_stake();
        let coldkey_stake = SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);
        let stake_coldkey_hotkey1 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &old_coldkey,
            netuid,
        );
        let stake_coldkey_hotkey2 = SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey2,
            &old_coldkey,
            netuid,
        );
        let total_hotkey1_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey1);
        let total_hotkey2_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey2);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Verify stake transfer
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &new_coldkey,
                netuid
            ),
            expected_stake_alpha1
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2,
                &new_coldkey,
                netuid
            ),
            expected_stake_alpha2
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1,
                &old_coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2,
                &old_coldkey,
                netuid
            ),
            AlphaCurrency::ZERO
        );

        // Verify TotalColdkeyStake
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            coldkey_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO
        );

        // Verify TotalHotkeyStake remains unchanged
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey1),
            total_hotkey1_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey2),
            total_hotkey2_stake
        );

        // Verify total stake and issuance remain unchanged
        assert_eq!(
            SubtensorModule::get_total_stake(),
            initial_total_stake,
            "Total stake changed unexpectedly"
        );
        assert_eq!(
            SubtensorModule::get_total_issuance(),
            initial_total_issuance,
            "Total issuance changed unexpectedly"
        );
    });
}

#[test]
fn test_swap_subnet_owner_for_coldkey() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);

        // Initialize SubnetOwner for old_coldkey
        add_network(netuid1, 13, 0);
        add_network(netuid2, 14, 0);
        SubnetOwner::<Test>::insert(netuid1, old_coldkey);
        SubnetOwner::<Test>::insert(netuid2, old_coldkey);

        // Set up TotalNetworks
        TotalNetworks::<Test>::put(3);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Verify the swap
        assert_eq!(SubnetOwner::<Test>::get(netuid1), new_coldkey);
        assert_eq!(SubnetOwner::<Test>::get(netuid2), new_coldkey);
    });
}

#[test]
fn test_do_swap_coldkey_with_subnet_ownership() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let netuid = NetUid::from(1u16);
        let stake_amount = 1000;
        let swap_cost = SubtensorModule::get_key_swap_cost().to_u64();

        // Setup initial state
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, old_coldkey, 0);

        // Set TotalNetworks because swap relies on it
        crate::TotalNetworks::<Test>::set(1);

        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, stake_amount + swap_cost);
        SubnetOwner::<Test>::insert(netuid, old_coldkey);

        // Populate OwnedHotkeys map
        OwnedHotkeys::<Test>::insert(old_coldkey, vec![hotkey]);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Verify subnet ownership transfer
        assert_eq!(SubnetOwner::<Test>::get(netuid), new_coldkey);
    });
}

#[test]
fn test_coldkey_has_associated_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        let netuid = NetUid::from(1u16);

        // Setup initial state
        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 1000);
    });
}

#[test]
fn test_coldkey_swap_total() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let nominator1 = U256::from(2);
        let nominator2 = U256::from(3);
        let nominator3 = U256::from(4);
        let delegate1 = U256::from(5);
        let delegate2 = U256::from(6);
        let delegate3 = U256::from(7);
        let hotkey1 = U256::from(2);
        let hotkey2 = U256::from(3);
        let hotkey3 = U256::from(4);
        let netuid1 = NetUid::from(1);
        let netuid2 = NetUid::from(2);
        let netuid3 = NetUid::from(3);
        let stake = DefaultMinStake::<Test>::get().to_u64() * 10;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake * 6);
        SubtensorModule::add_balance_to_coldkey_account(&delegate1, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&delegate2, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&delegate3, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&nominator1, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&nominator2, stake * 2);
        SubtensorModule::add_balance_to_coldkey_account(&nominator3, stake * 2);

        let reserve = stake * 10;
        mock::setup_reserves(netuid1, reserve.into(), reserve.into());
        mock::setup_reserves(netuid2, reserve.into(), reserve.into());
        mock::setup_reserves(netuid3, reserve.into(), reserve.into());

        // Setup initial state
        add_network(netuid1, 13, 0);
        add_network(netuid2, 14, 0);
        add_network(netuid3, 15, 0);
        register_ok_neuron(netuid1, hotkey1, coldkey, 0);
        register_ok_neuron(netuid2, hotkey2, coldkey, 0);
        register_ok_neuron(netuid3, hotkey3, coldkey, 0);
        register_ok_neuron(netuid1, delegate1, delegate1, 0);
        register_ok_neuron(netuid2, delegate2, delegate2, 0);
        register_ok_neuron(netuid3, delegate3, delegate3, 0);
        Delegates::<Test>::insert(hotkey1, u16::MAX / 10);
        Delegates::<Test>::insert(hotkey2, u16::MAX / 10);
        Delegates::<Test>::insert(hotkey3, u16::MAX / 10);
        Delegates::<Test>::insert(delegate1, u16::MAX / 10);
        Delegates::<Test>::insert(delegate2, u16::MAX / 10);
        Delegates::<Test>::insert(delegate3, u16::MAX / 10);

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey3,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate1),
            hotkey1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate2),
            hotkey2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate3),
            hotkey3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate1),
            delegate1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate2),
            delegate2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate3),
            delegate3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator1),
            hotkey1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator2),
            hotkey2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator3),
            hotkey3,
            netuid1,
            stake.into()
        ));

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator1),
            delegate1,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator2),
            delegate2,
            netuid1,
            stake.into()
        ));
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(nominator3),
            delegate3,
            netuid1,
            stake.into()
        ));

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&coldkey),
            vec![hotkey1, hotkey2, hotkey3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&coldkey),
            vec![hotkey1, hotkey2, hotkey3, delegate1, delegate2, delegate3]
        );
        let ck_stake = SubtensorModule::get_total_stake_for_coldkey(&coldkey);
        let hk1_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey1);
        let hk2_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey2);
        let hk3_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey3);
        let d1_stake = SubtensorModule::get_total_stake_for_hotkey(&delegate1);
        let d2_stake = SubtensorModule::get_total_stake_for_hotkey(&delegate2);
        let d3_stake = SubtensorModule::get_total_stake_for_hotkey(&delegate3);

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate1),
            vec![delegate1]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate2),
            vec![delegate2]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate3),
            vec![delegate3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate1),
            vec![delegate1, hotkey1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate2),
            vec![delegate2, hotkey2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate3),
            vec![delegate3, hotkey3]
        );

        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator1), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator2), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator3), vec![]);

        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator1),
            vec![hotkey1, delegate1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator2),
            vec![hotkey2, delegate2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator3),
            vec![hotkey3, delegate3]
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost.to_u64());

        // Perform the swap
        let new_coldkey = U256::from(1100);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&coldkey),
            ck_stake
        );
        assert_ok!(SubtensorModule::do_swap_coldkey(&coldkey, &new_coldkey,));
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            ck_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&coldkey),
            TaoCurrency::ZERO
        );

        // Check everything is swapped.
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2, hotkey3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&new_coldkey),
            vec![hotkey1, hotkey2, hotkey3, delegate1, delegate2, delegate3]
        );
        // Shouldn't change.
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey1),
            hk1_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey2),
            hk2_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey3),
            hk3_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate1),
            d1_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate2),
            d2_stake
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate3),
            d3_stake
        );

        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate1),
            vec![delegate1]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate2),
            vec![delegate2]
        );
        assert_eq!(
            SubtensorModule::get_owned_hotkeys(&delegate3),
            vec![delegate3]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate1),
            vec![delegate1, hotkey1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate2),
            vec![delegate2, hotkey2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&delegate3),
            vec![delegate3, hotkey3]
        );

        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator1), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator2), vec![]);
        assert_eq!(SubtensorModule::get_owned_hotkeys(&nominator3), vec![]);

        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator1),
            vec![hotkey1, delegate1]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator2),
            vec![hotkey2, delegate2]
        );
        assert_eq!(
            SubtensorModule::get_all_staked_hotkeys(&nominator3),
            vec![hotkey3, delegate3]
        );
    });
}

#[test]
fn test_coldkey_delegations() {
    new_test_ext(1).execute_with(|| {
        let new_coldkey = U256::from(0);
        let owner = U256::from(1);
        let coldkey = U256::from(4);
        let delegate = U256::from(2);
        let netuid = NetUid::from(0); // Stake to 0
        let netuid2 = NetUid::from(1); // Stake to 1
        let stake = DefaultMinStake::<Test>::get().to_u64() * 10;
        let reserve = stake * 1000;

        mock::setup_reserves(netuid, reserve.into(), reserve.into());
        mock::setup_reserves(netuid2, reserve.into(), reserve.into());

        add_network(netuid, 13, 0); // root
        add_network(netuid2, 13, 0);

        assert_ok!(SubtensorModule::root_register(
            <<Test as Config>::RuntimeOrigin>::signed(owner),
            delegate
        )); // register on root
        register_ok_neuron(netuid2, delegate, owner, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake * 10);

        // since the reserves are equal and we stake the same amount to both networks, we can reuse
        // this values for different networks. but you should take it into account in case of tests
        // changes
        let (expected_stake, fee) = mock::swap_tao_to_alpha(netuid, stake.into());

        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate,
            netuid,
            stake.into()
        ));

        // Add stake to netuid2
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            delegate,
            netuid2,
            stake.into()
        ));

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost.to_u64());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&coldkey, &new_coldkey,));

        // Verify stake was moved for the delegate
        let approx_total_stake = TaoCurrency::from(stake * 2 - fee * 2);
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&delegate),
            approx_total_stake,
            epsilon = approx_total_stake / 100.into()
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&coldkey),
            TaoCurrency::ZERO
        );
        assert_abs_diff_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            approx_total_stake,
            epsilon = approx_total_stake / 100.into()
        );
        assert_eq!(
            expected_stake,
            Alpha::<Test>::get((delegate, new_coldkey, netuid))
                .to_num::<u64>()
                .into(),
        );
        assert_eq!(Alpha::<Test>::get((delegate, coldkey, netuid)), 0);

        assert_eq!(
            expected_stake,
            Alpha::<Test>::get((delegate, new_coldkey, netuid2))
                .to_num::<u64>()
                .into()
        );
        assert_eq!(Alpha::<Test>::get((delegate, coldkey, netuid2)), 0);
    });
}

#[test]
fn test_schedule_swap_coldkey_execution() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let netuid = NetUid::from(1u16);
        let stake_amount = DefaultMinStake::<Test>::get().to_u64() * 10;
        let reserve = stake_amount * 10;

        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        add_network(netuid, 13, 0);
        register_ok_neuron(netuid, hotkey, old_coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, 1000000000000000);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            hotkey,
            netuid,
            stake_amount.into()
        ));

        // Check initial ownership
        assert_eq!(
            Owner::<Test>::get(hotkey),
            old_coldkey,
            "Initial ownership check failed"
        );

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());

        // Schedule the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Get the scheduled execution block
        let current_block = System::block_number();
        let execution_block = current_block + ColdkeySwapScheduleDuration::<Test>::get();

        System::run_to_block::<AllPalletsWithSystem>(execution_block - 1);

        let stake_before_swap = SubtensorModule::get_total_stake_for_coldkey(&old_coldkey);

        run_to_block(execution_block);

        // Run on_initialize for the execution block
        <SubtensorModule as OnInitialize<BlockNumber>>::on_initialize(execution_block);

        // Also run Scheduler's on_initialize
        <pallet_scheduler::Pallet<Test> as OnInitialize<BlockNumber>>::on_initialize(
            execution_block,
        );

        // Check if the swap has occurred
        let new_owner = Owner::<Test>::get(hotkey);
        assert_eq!(
            new_owner, new_coldkey,
            "Ownership was not updated as expected"
        );

        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&new_coldkey),
            stake_before_swap,
            "Stake was not transferred to new coldkey"
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&old_coldkey),
            TaoCurrency::ZERO,
            "Old coldkey still has stake"
        );

        // Check for the SwapExecuted event
        System::assert_has_event(
            Event::ColdkeySwapped {
                old_coldkey,
                new_coldkey,
                swap_cost,
            }
            .into(),
        );
    });
}

#[test]
fn test_coldkey_swap_delegate_identity_updated() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        let netuid = NetUid::from(1);
        let burn_cost = TaoCurrency::from(10);
        let tempo = 1;

        SubtensorModule::set_burn(netuid, burn_cost);
        add_network(netuid, tempo, 0);

        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, 100_000_000_000);
        mock::setup_reserves(netuid, 1_000_000_000_000.into(), 1_000_000_000_000.into());

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            netuid,
            old_coldkey
        ));

        let name: Vec<u8> = b"The Third Coolest Identity".to_vec();
        let identity: ChainIdentityV2 = ChainIdentityV2 {
            name: name.clone(),
            url: vec![],
            image: vec![],
            github_repo: vec![],
            discord: vec![],
            description: vec![],
            additional: vec![],
        };

        IdentitiesV2::<Test>::insert(old_coldkey, identity.clone());

        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_some());
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_none());

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_none());
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_some());
        assert_eq!(
            IdentitiesV2::<Test>::get(new_coldkey).expect("Expected an Identity"),
            identity
        );
    });
}

#[test]
fn test_coldkey_swap_no_identity_no_changes() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        let netuid = NetUid::from(1);
        let burn_cost = TaoCurrency::from(10);
        let tempo = 1;

        SubtensorModule::set_burn(netuid, burn_cost);
        add_network(netuid, tempo, 0);

        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, 100_000_000_000);
        mock::setup_reserves(netuid, 1_000_000_000_000.into(), 1_000_000_000_000.into());

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            netuid,
            old_coldkey
        ));

        // Ensure the old coldkey does not have an identity before the swap
        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_none());

        // Perform the coldkey swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Ensure no identities have been changed
        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_none());
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_none());
    });
}

#[test]
fn test_coldkey_swap_no_identity_no_changes_newcoldkey_exists() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(3);
        let new_coldkey = U256::from(4);

        let netuid = NetUid::from(1);
        let burn_cost = TaoCurrency::from(10);
        let tempo = 1;

        SubtensorModule::set_burn(netuid, burn_cost);
        add_network(netuid, tempo, 0);
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, 100_000_000_000);
        mock::setup_reserves(netuid, 1_000_000_000_000.into(), 1_000_000_000_000.into());

        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(old_coldkey),
            netuid,
            old_coldkey
        ));

        let name: Vec<u8> = b"The Coolest Identity".to_vec();
        let identity: ChainIdentityV2 = ChainIdentityV2 {
            name: name.clone(),
            url: vec![],
            github_repo: vec![],
            image: vec![],
            discord: vec![],
            description: vec![],
            additional: vec![],
        };

        IdentitiesV2::<Test>::insert(new_coldkey, identity.clone());
        // Ensure the new coldkey does have an identity before the swap
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_some());
        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_none());

        // Perform the coldkey swap
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        // Ensure no identities have been changed
        assert!(IdentitiesV2::<Test>::get(old_coldkey).is_none());
        assert!(IdentitiesV2::<Test>::get(new_coldkey).is_some());
    });
}

#[test]
fn test_cant_schedule_swap_without_enough_to_burn() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(3);
        let new_coldkey = U256::from(4);
        let hotkey = U256::from(5);

        assert_noop!(
            SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey),
            Error::<Test>::NotEnoughBalanceToPaySwapColdKey
        );
    });
}

#[test]
fn test_coldkey_in_swap_schedule_prevents_funds_usage() {
    // Testing the signed extension validate function
    // correctly filters transactions that attempt to use funds
    // while a coldkey swap is scheduled.

    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let version_key: u64 = 0;
        let coldkey = U256::from(0);
        let new_coldkey = U256::from(1);
        let hotkey: U256 = U256::from(2); // Add the hotkey field
        assert_ne!(hotkey, coldkey); // Ensure hotkey is NOT the same as coldkey !!!

        let stake = 100_000_000_000;
        let reserve = stake * 100;

        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        let who = coldkey; // The coldkey signs this transaction

        // Disallowed transactions are
        // - add_stake
        // - add_stake_limit
        // - swap_stake
        // - swap_stake_limit
        // - move_stake
        // - transfer_stake
        // - balances.transfer_all
        // - balances.transfer_allow_death
        // - balances.transfer_keep_alive

        // Allowed transactions are:
        // - remove_stake
        // - remove_stake_limit
        // others...

        // Create netuid
        add_network(netuid, 1, 0);
        // Register the hotkey
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);

        SubtensorModule::add_balance_to_coldkey_account(&who, u64::MAX);

        // Set the minimum stake to 0.
        SubtensorModule::set_stake_threshold(0);
        // Add stake to the hotkey
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(who),
            hotkey,
            netuid,
            stake.into()
        ));

        // Schedule the coldkey for a swap
        assert_ok!(SubtensorModule::announce_coldkey_swap(
            <Test as frame_system::Config>::RuntimeOrigin::signed(who),
            new_coldkey,
        ));

        assert!(ColdkeySwapAnnouncements::<Test>::contains_key(who));

        // Try each call

        // Add stake
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake {
            hotkey,
            netuid,
            amount_staked: stake.into(),
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        // Should fail
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Add stake limit
        let call = RuntimeCall::SubtensorModule(SubtensorCall::add_stake_limit {
            hotkey,
            netuid,
            amount_staked: stake.into(),
            limit_price: stake.into(),
            allow_partial: false,
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Swap stake
        let call = RuntimeCall::SubtensorModule(SubtensorCall::swap_stake {
            hotkey,
            origin_netuid: netuid,
            destination_netuid: netuid,
            alpha_amount: stake.into(),
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Swap stake limit
        let call = RuntimeCall::SubtensorModule(SubtensorCall::swap_stake_limit {
            hotkey,
            origin_netuid: netuid,
            destination_netuid: netuid,
            alpha_amount: stake.into(),
            limit_price: stake.into(),
            allow_partial: false,
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Move stake
        let call = RuntimeCall::SubtensorModule(SubtensorCall::move_stake {
            origin_hotkey: hotkey,
            destination_hotkey: hotkey,
            origin_netuid: netuid,
            destination_netuid: netuid,
            alpha_amount: stake.into(),
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Transfer stake
        let call = RuntimeCall::SubtensorModule(SubtensorCall::transfer_stake {
            destination_coldkey: new_coldkey,
            hotkey,
            origin_netuid: netuid,
            destination_netuid: netuid,
            alpha_amount: stake.into(),
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Transfer all
        let call = RuntimeCall::Balances(BalancesCall::transfer_all {
            dest: new_coldkey,
            keep_alive: false,
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Transfer keep alive
        let call = RuntimeCall::Balances(BalancesCall::transfer_keep_alive {
            dest: new_coldkey,
            value: 100_000_000_000,
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Transfer allow death
        let call = RuntimeCall::Balances(BalancesCall::transfer_allow_death {
            dest: new_coldkey,
            value: 100_000_000_000,
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Burned register
        let call = RuntimeCall::SubtensorModule(SubtensorCall::burned_register { netuid, hotkey });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        remove_stake_rate_limit_for_tests(&hotkey, &coldkey, netuid);

        // Remove stake
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake {
            hotkey,
            netuid,
            amount_unstaked: (DefaultMinStake::<Test>::get().to_u64() * 2).into(),
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Remove stake limit
        let call = RuntimeCall::SubtensorModule(SubtensorCall::remove_stake_limit {
            hotkey,
            netuid,
            amount_unstaked: (DefaultMinStake::<Test>::get().to_u64() * 2).into(),
            limit_price: 123456789.into(), // should be low enough
            allow_partial: true,
        });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0),
            CustomTransactionError::ColdkeySwapAnnounced
        );

        // Announce coldkey swap should succeed
        let call =
            RuntimeCall::SubtensorModule(SubtensorCall::announce_coldkey_swap { new_coldkey });
        let info = call.get_dispatch_info();
        let ext = SubtensorTransactionExtension::<Test>::new();
        assert_ok!(ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0));
    });
}

#[test]
fn test_announced_coldkey_swap_prevents_critical_calls() {
    // Testing the signed extension validate function
    // correctly filters transactions that are critical
    // while a coldkey swap is announced.

    new_test_ext(0).execute_with(|| {
        let netuid = NetUid::from(1);
        let version_key: u64 = 0;
        let coldkey = U256::from(0);
        let new_coldkey = U256::from(1);
        let hotkey: U256 = U256::from(2); // Add the hotkey field
        assert_ne!(hotkey, coldkey); // Ensure hotkey is NOT the same as coldkey !!!
        let stake = 100_000_000_000;
        let reserve = stake * 10;

        mock::setup_reserves(netuid, reserve.into(), reserve.into());

        let who = coldkey; // The coldkey signs this transaction

        // Disallowed transactions are
        // - dissolve_network

        // Create netuid
        add_network(netuid, 1, 0);
        // Register the hotkey
        SubtensorModule::append_neuron(netuid, &hotkey, 0);
        crate::Owner::<Test>::insert(hotkey, coldkey);

        SubtensorModule::add_balance_to_coldkey_account(&who, u64::MAX);

        // Set the minimum stake to 0.
        SubtensorModule::set_stake_threshold(0);
        // Add stake to the hotkey
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(who),
            hotkey,
            netuid,
            stake.into()
        ));

        // Schedule the coldkey for a swap
        assert_ok!(SubtensorModule::announce_coldkey_swap(
            <Test as frame_system::Config>::RuntimeOrigin::signed(who),
            new_coldkey,
        ));

        assert!(ColdkeySwapAnnouncements::<Test>::contains_key(who));

        // Try each call

        // Dissolve network
        let ext = SubtensorTransactionExtension::<Test>::new();
        let call =
            RuntimeCall::SubtensorModule(SubtensorCall::dissolve_network { netuid, coldkey });
        let info = call.get_dispatch_info();

        assert_noop!(
            ext.dispatch_transaction(RuntimeOrigin::signed(who).into(), call, &info, 0, 0,),
            CustomTransactionError::ColdkeySwapAnnounced
        );
    });
}

#[test]
fn test_swap_auto_stake_destination_coldkeys() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);
        let hotkey = U256::from(3);
        let netuid = NetUid::from(1u16);
        let coldkeys = vec![U256::from(4), U256::from(5), old_coldkey];

        add_network(netuid, 1, 0);
        AutoStakeDestinationColdkeys::<Test>::insert(hotkey, netuid, coldkeys.clone());
        AutoStakeDestination::<Test>::insert(old_coldkey, netuid, hotkey);

        let swap_cost = SubtensorModule::get_key_swap_cost();
        SubtensorModule::add_balance_to_coldkey_account(&old_coldkey, swap_cost.to_u64());
        assert_ok!(SubtensorModule::do_swap_coldkey(&old_coldkey, &new_coldkey,));

        let new_coldkeys = AutoStakeDestinationColdkeys::<Test>::get(hotkey, netuid);
        assert!(new_coldkeys.contains(&new_coldkey));
        assert!(!new_coldkeys.contains(&old_coldkey));
        assert_eq!(
            AutoStakeDestination::<Test>::try_get(old_coldkey, netuid),
            Err(())
        );
        assert_eq!(
            AutoStakeDestination::<Test>::try_get(new_coldkey, netuid),
            Ok(hotkey)
        );
    });
}

#[test]
#[allow(deprecated)]
fn test_swap_coldkey_deprecated() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        assert_noop!(
            SubtensorModule::swap_coldkey(
                <<Test as Config>::RuntimeOrigin>::root(),
                old_coldkey,
                new_coldkey,
                TaoCurrency::MAX
            ),
            Error::<Test>::Deprecated
        );
    });
}

#[test]
#[allow(deprecated)]
fn test_schedule_swap_coldkey_deprecated() {
    new_test_ext(1).execute_with(|| {
        let old_coldkey = U256::from(1);
        let new_coldkey = U256::from(2);

        assert_noop!(
            SubtensorModule::schedule_swap_coldkey(
                <<Test as Config>::RuntimeOrigin>::root(),
                new_coldkey,
            ),
            Error::<Test>::Deprecated
        );
    });
}
