#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]

use codec::Encode;
use frame_support::weights::Weight;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{Config, RawOrigin};
mod mock;
use mock::*;
use pallet_subtensor::*;
use sp_core::U256;
use sp_core::H256;


// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_owner --exact --nocapture
#[test]
fn test_swap_owner() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        Owner::<Test>::insert(&old_hotkey, &coldkey);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!Owner::<Test>::contains_key(&old_hotkey));
        assert_eq!(Owner::<Test>::get(&new_hotkey), coldkey);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_owned_hotkeys --exact --nocapture
#[test]
fn test_swap_owned_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        OwnedHotkeys::<Test>::insert(&coldkey, vec![old_hotkey]);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        let hotkeys = OwnedHotkeys::<Test>::get(&coldkey);
        assert!(!hotkeys.contains(&old_hotkey));
        assert!(hotkeys.contains(&new_hotkey));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_total_hotkey_stake --exact --nocapture
#[test]
fn test_swap_total_hotkey_stake() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        TotalHotkeyStake::<Test>::insert(&old_hotkey, 100);
        TotalHotkeyStake::<Test>::insert(&new_hotkey, 50);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!TotalHotkeyStake::<Test>::contains_key(&old_hotkey));
        assert_eq!(TotalHotkeyStake::<Test>::get(&new_hotkey), 150);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_total_hotkey_coldkey_stakes_this_interval --exact --nocapture
#[test]
fn test_swap_total_hotkey_coldkey_stakes_this_interval() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        TotalHotkeyColdkeyStakesThisInterval::<Test>::insert(&old_hotkey, &coldkey, (100, 1000));
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!TotalHotkeyColdkeyStakesThisInterval::<Test>::contains_key(&old_hotkey, &coldkey));
        assert_eq!(TotalHotkeyColdkeyStakesThisInterval::<Test>::get(&new_hotkey, &coldkey), (100, 1000));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_last_tx_block --exact --nocapture
#[test]
fn test_swap_last_tx_block() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        LastTxBlock::<Test>::insert(&old_hotkey, 1000);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!LastTxBlock::<Test>::contains_key(&old_hotkey));
        assert_eq!(LastTxBlock::<Test>::get(&new_hotkey), SubtensorModule::get_current_block_as_u64());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_last_tx_block_delegate_take --exact --nocapture
#[test]
fn test_swap_last_tx_block_delegate_take() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        pallet_subtensor::LastTxBlockDelegateTake::<Test>::insert(&old_hotkey, 1000);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!LastTxBlockDelegateTake::<Test>::contains_key(&old_hotkey));
        assert_eq!(LastTxBlockDelegateTake::<Test>::get(&new_hotkey), SubtensorModule::get_current_block_as_u64());
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_senate_members --exact --nocapture
#[test]
fn test_swap_senate_members() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Assuming there's a way to add a member to the senate
        // SenateMembers::add_member(&old_hotkey);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        // Assert that the old_hotkey is no longer a member and new_hotkey is now a member
        // assert!(!SenateMembers::is_member(&old_hotkey));
        // assert!(SenateMembers::is_member(&new_hotkey));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_delegates --exact --nocapture
#[test]
fn test_swap_delegates() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        Delegates::<Test>::insert(&old_hotkey, 100);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!Delegates::<Test>::contains_key(&old_hotkey));
        assert_eq!(Delegates::<Test>::get(&new_hotkey), 100);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_subnet_membership --exact --nocapture
#[test]
fn test_swap_subnet_membership() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let netuid = 0u16;
        let mut weight = Weight::zero();

        add_network(netuid, 0, 1);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid, true);
        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!IsNetworkMember::<Test>::contains_key(&old_hotkey, netuid));
        assert!(IsNetworkMember::<Test>::get(&new_hotkey, netuid));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_uids_and_keys --exact --nocapture
#[test]
fn test_swap_uids_and_keys() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let netuid = 0u16;
        let uid = 5u16;
        let mut weight = Weight::zero();

        add_network(netuid, 0, 1);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid, true);
        Uids::<Test>::insert(netuid, &old_hotkey, uid);
        Keys::<Test>::insert(netuid, uid, old_hotkey);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert_eq!(Uids::<Test>::get(netuid, &old_hotkey), None);
        assert_eq!(Uids::<Test>::get(netuid, &new_hotkey), Some(uid));
        assert_eq!(Keys::<Test>::get(netuid, uid), new_hotkey);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_prometheus --exact --nocapture
#[test]
fn test_swap_prometheus() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let netuid = 0u16;
        let prometheus_info = PrometheusInfo::default();
        let mut weight = Weight::zero();

        add_network(netuid, 0, 1);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid, true);
        Prometheus::<Test>::insert(netuid, &old_hotkey, prometheus_info.clone());

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!Prometheus::<Test>::contains_key(netuid, &old_hotkey));
        assert_eq!(Prometheus::<Test>::get(netuid, &new_hotkey), Some(prometheus_info));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_axons --exact --nocapture
#[test]
fn test_swap_axons() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let netuid = 0u16;
        let axon_info = AxonInfo::default();
        let mut weight = Weight::zero();

        add_network(netuid, 0, 1);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid, true);
        Axons::<Test>::insert(netuid, &old_hotkey, axon_info.clone());

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!Axons::<Test>::contains_key(netuid, &old_hotkey));
        assert_eq!(Axons::<Test>::get(netuid, &new_hotkey), Some(axon_info));
    });
}
     
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_weight_commits --exact --nocapture
#[test]
fn test_swap_weight_commits() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let netuid = 0u16;
        let weight_commits = (H256::from_low_u64_be(100), 200);
        let mut weight = Weight::zero();

        add_network(netuid, 0, 1);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid, true);
        WeightCommits::<Test>::insert(netuid, &old_hotkey, weight_commits.clone());

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!WeightCommits::<Test>::contains_key(netuid, &old_hotkey));
        assert_eq!(WeightCommits::<Test>::get(netuid, &new_hotkey), Some(weight_commits));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_loaded_emission --exact --nocapture
#[test]
fn test_swap_loaded_emission() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let netuid = 0u16;
        let server_emission = 1000u64;
        let validator_emission = 1000u64;
        let mut weight = Weight::zero();

        add_network(netuid, 0, 1);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid, true);
        LoadedEmission::<Test>::insert(netuid, vec![(old_hotkey, server_emission, validator_emission)]);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        let new_loaded_emission = LoadedEmission::<Test>::get(netuid);
        assert_eq!(new_loaded_emission, Some(vec![(new_hotkey, server_emission, validator_emission)]));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_stake --exact --nocapture
#[test]
fn test_swap_stake() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake_amount = 100u64;
        let mut weight = Weight::zero();

        Stake::<Test>::insert(&old_hotkey, &coldkey, stake_amount);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(!Stake::<Test>::contains_key(&old_hotkey, &coldkey));
        assert_eq!(Stake::<Test>::get(&new_hotkey, &coldkey), stake_amount);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_staking_hotkeys --exact --nocapture
#[test]
fn test_swap_staking_hotkeys() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        Stake::<Test>::insert(&old_hotkey, &coldkey, 100);
        StakingHotkeys::<Test>::insert(&coldkey, vec![old_hotkey]);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        let staking_hotkeys = StakingHotkeys::<Test>::get(&coldkey);
        assert!(!staking_hotkeys.contains(&old_hotkey));
        assert!(staking_hotkeys.contains(&new_hotkey));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_hotkey_with_multiple_coldkeys --exact --nocapture
#[test]
fn test_swap_hotkey_with_multiple_coldkeys() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);
        let mut weight = Weight::zero();

        Stake::<Test>::insert(&old_hotkey, &coldkey1, 100);
        Stake::<Test>::insert(&old_hotkey, &coldkey2, 200);
        StakingHotkeys::<Test>::insert(&coldkey1, vec![old_hotkey]);
        StakingHotkeys::<Test>::insert(&coldkey2, vec![old_hotkey]);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey1, &mut weight));

        assert_eq!(Stake::<Test>::get(&new_hotkey, &coldkey1), 100);
        assert_eq!(Stake::<Test>::get(&new_hotkey, &coldkey2), 200);
        assert!(StakingHotkeys::<Test>::get(&coldkey1).contains(&new_hotkey));
        assert!(StakingHotkeys::<Test>::get(&coldkey2).contains(&new_hotkey));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_hotkey_with_existing_stake --exact --nocapture
#[test]
fn test_swap_hotkey_with_existing_stake() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        Stake::<Test>::insert(&old_hotkey, &coldkey, 100);
        Stake::<Test>::insert(&new_hotkey, &coldkey, 50);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert_eq!(Stake::<Test>::get(&new_hotkey, &coldkey), 150);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_hotkey_with_multiple_subnets --exact --nocapture
#[test]
fn test_swap_hotkey_with_multiple_subnets() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let netuid1 = 0;
        let netuid2 = 1;
        let mut weight = Weight::zero();

        add_network(netuid1, 0, 1);
        add_network(netuid2, 0, 1);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid1, true);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid2, true);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        assert!(IsNetworkMember::<Test>::get(&new_hotkey, netuid1));
        assert!(IsNetworkMember::<Test>::get(&new_hotkey, netuid2));
        assert!(!IsNetworkMember::<Test>::get(&old_hotkey, netuid1));
        assert!(!IsNetworkMember::<Test>::get(&old_hotkey, netuid2));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_staking_hotkeys_multiple_coldkeys --exact --nocapture
#[test]
fn test_swap_staking_hotkeys_multiple_coldkeys() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);
        let mut weight = Weight::zero();

        // Set up initial state
        Stake::<Test>::insert(&old_hotkey, &coldkey1, 100);
        Stake::<Test>::insert(&old_hotkey, &coldkey2, 200);
        StakingHotkeys::<Test>::insert(&coldkey1, vec![old_hotkey]);
        StakingHotkeys::<Test>::insert(&coldkey2, vec![old_hotkey, U256::from(5)]);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey1, &mut weight));

        // Check if new_hotkey replaced old_hotkey in StakingHotkeys
        assert!(StakingHotkeys::<Test>::get(&coldkey1).contains(&new_hotkey));
        assert!(!StakingHotkeys::<Test>::get(&coldkey1).contains(&old_hotkey));

        // Check if new_hotkey replaced old_hotkey for coldkey2 as well
        assert!(StakingHotkeys::<Test>::get(&coldkey2).contains(&new_hotkey));
        assert!(!StakingHotkeys::<Test>::get(&coldkey2).contains(&old_hotkey));
        assert!(StakingHotkeys::<Test>::get(&coldkey2).contains(&U256::from(5))); // Other hotkeys should remain
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_hotkey_with_no_stake --exact --nocapture
#[test]
fn test_swap_hotkey_with_no_stake() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Set up initial state with no stake
        Owner::<Test>::insert(&old_hotkey, &coldkey);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey, &mut weight));

        // Check if ownership transferred
        assert!(!Owner::<Test>::contains_key(&old_hotkey));
        assert_eq!(Owner::<Test>::get(&new_hotkey), coldkey);

        // Ensure no unexpected changes in Stake
        assert!(!Stake::<Test>::contains_key(&old_hotkey, &coldkey));
        assert!(!Stake::<Test>::contains_key(&new_hotkey, &coldkey));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_hotkey_with_multiple_coldkeys_and_subnets --exact --nocapture
#[test]
fn test_swap_hotkey_with_multiple_coldkeys_and_subnets() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);
        let netuid1 = 0;
        let netuid2 = 1;
        let mut weight = Weight::zero();

        // Set up initial state
        add_network(netuid1, 0, 1);
        add_network(netuid2, 0, 1);
        Owner::<Test>::insert(&old_hotkey, &coldkey1);
        Stake::<Test>::insert(&old_hotkey, &coldkey1, 100);
        Stake::<Test>::insert(&old_hotkey, &coldkey2, 200);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid1, true);
        IsNetworkMember::<Test>::insert(&old_hotkey, netuid2, true);
        TotalHotkeyStake::<Test>::insert(&old_hotkey, 300);

        assert_ok!(SubtensorModule::perform_hotkey_swap(&old_hotkey, &new_hotkey, &coldkey1, &mut weight));

        // Check ownership transfer
        assert!(!Owner::<Test>::contains_key(&old_hotkey));
        assert_eq!(Owner::<Test>::get(&new_hotkey), coldkey1);

        // Check stake transfer
        assert_eq!(Stake::<Test>::get(&new_hotkey, &coldkey1), 100);
        assert_eq!(Stake::<Test>::get(&new_hotkey, &coldkey2), 200);
        assert!(!Stake::<Test>::contains_key(&old_hotkey, &coldkey1));
        assert!(!Stake::<Test>::contains_key(&old_hotkey, &coldkey2));

        // Check subnet membership transfer
        assert!(IsNetworkMember::<Test>::get(&new_hotkey, netuid1));
        assert!(IsNetworkMember::<Test>::get(&new_hotkey, netuid2));
        assert!(!IsNetworkMember::<Test>::get(&old_hotkey, netuid1));
        assert!(!IsNetworkMember::<Test>::get(&old_hotkey, netuid2));

        // Check total stake transfer
        assert_eq!(TotalHotkeyStake::<Test>::get(&new_hotkey), 300);
        assert!(!TotalHotkeyStake::<Test>::contains_key(&old_hotkey));
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test swap_hotkey -- test_swap_hotkey_error_cases --exact --nocapture
#[test]
fn test_swap_hotkey_error_cases() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let wrong_coldkey = U256::from(4);

        // Set up initial state
        Owner::<Test>::insert(&old_hotkey, &coldkey);
        TotalNetworks::<Test>::put(1);
        LastTxBlock::<Test>::insert(&coldkey, 0);

        // Test not enough balance
        let swap_cost = SubtensorModule::get_key_swap_cost();
        assert_noop!(
            SubtensorModule::do_swap_hotkey(RuntimeOrigin::signed(coldkey), &old_hotkey, &new_hotkey),
            Error::<Test>::NotEnoughBalanceToPaySwapHotKey
        );

        let initial_balance = SubtensorModule::get_key_swap_cost() + 1000;
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_balance);

        // Test new hotkey same as old
        assert_noop!(
            SubtensorModule::do_swap_hotkey(RuntimeOrigin::signed(coldkey), &old_hotkey, &old_hotkey),
            Error::<Test>::NewHotKeyIsSameWithOld
        );

        // Test new hotkey already registered
        IsNetworkMember::<Test>::insert(&new_hotkey, 0, true);
        assert_noop!(
            SubtensorModule::do_swap_hotkey(RuntimeOrigin::signed(coldkey), &old_hotkey, &new_hotkey),
            Error::<Test>::HotKeyAlreadyRegisteredInSubNet
        );
        IsNetworkMember::<Test>::remove(&new_hotkey, 0);

        // Test non-associated coldkey
        assert_noop!(
            SubtensorModule::do_swap_hotkey(RuntimeOrigin::signed(wrong_coldkey), &old_hotkey, &new_hotkey),
            Error::<Test>::NonAssociatedColdKey
        );


        // Run the successful swap
        assert_ok!(SubtensorModule::do_swap_hotkey(RuntimeOrigin::signed(coldkey), &old_hotkey, &new_hotkey));

        // Check balance after swap
        assert_eq!(Balances::free_balance(&coldkey), initial_balance - swap_cost);
    });
}
