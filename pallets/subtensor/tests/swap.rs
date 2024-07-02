#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]

use codec::Encode;
use frame_support::weights::Weight;
use frame_support::{assert_err, assert_ok};
use frame_system::Config;
mod mock;
use mock::*;
use pallet_subtensor::*;
use sp_core::U256;

#[test]
fn test_do_swap_hotkey_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let swap_cost = 1_000_000_000u64;

        // Setup initial state
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, old_hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost);

        // Perform the swap
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            &old_hotkey,
            &new_hotkey
        ));

        // Verify the swap
        assert_eq!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkey),
            coldkey
        );
        assert_ne!(
            SubtensorModule::get_owning_coldkey_for_hotkey(&old_hotkey),
            coldkey
        );

        // Verify other storage changes
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&new_hotkey),
            SubtensorModule::get_total_stake_for_hotkey(&old_hotkey)
        );
        assert_eq!(
            SubtensorModule::get_delegate(new_hotkey.encode()),
            SubtensorModule::get_delegate(old_hotkey.encode())
        );
        assert_eq!(
            SubtensorModule::get_last_tx_block(&new_hotkey),
            SubtensorModule::get_last_tx_block(&old_hotkey)
        );

        // Verify raw storage maps
        // Stake
        for (coldkey, stake_amount) in Stake::<Test>::iter_prefix(old_hotkey) {
            assert_eq!(Stake::<Test>::get(new_hotkey, coldkey), stake_amount);
        }

        let mut weight = Weight::zero();
        // UIDs
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                Uids::<Test>::get(netuid, new_hotkey),
                Uids::<Test>::get(netuid, old_hotkey)
            );
        }

        // Prometheus
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                Prometheus::<Test>::get(netuid, new_hotkey),
                Prometheus::<Test>::get(netuid, old_hotkey)
            );
        }

        // LoadedEmission
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                LoadedEmission::<Test>::get(netuid).unwrap(),
                LoadedEmission::<Test>::get(netuid).unwrap()
            );
        }

        // IsNetworkMember
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert!(IsNetworkMember::<Test>::contains_key(new_hotkey, netuid));
            assert!(!IsNetworkMember::<Test>::contains_key(old_hotkey, netuid));
        }

        // Owner
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);

        // TotalHotkeyStake
        assert_eq!(
            TotalHotkeyStake::<Test>::get(new_hotkey),
            TotalHotkeyStake::<Test>::get(old_hotkey)
        );

        // Delegates
        assert_eq!(
            Delegates::<Test>::get(new_hotkey),
            Delegates::<Test>::get(old_hotkey)
        );

        // LastTxBlock
        assert_eq!(
            LastTxBlock::<Test>::get(new_hotkey),
            LastTxBlock::<Test>::get(old_hotkey)
        );

        // Axons
        for netuid in SubtensorModule::get_netuid_is_member(&old_hotkey, &mut weight) {
            assert_eq!(
                Axons::<Test>::get(netuid, new_hotkey),
                Axons::<Test>::get(netuid, old_hotkey)
            );
        }

        // TotalHotkeyColdkeyStakesThisInterval
        assert_eq!(
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(new_hotkey, coldkey),
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(old_hotkey, coldkey)
        );
    });
}

#[test]
fn test_do_swap_hotkey_ok_robust() {
    new_test_ext(1).execute_with(|| {
        let num_subnets: u16 = 10;
        let tempo: u16 = 13;
        let swap_cost = 1_000_000_000u64;

        // Create 10 sets of keys
        let mut old_hotkeys = vec![];
        let mut new_hotkeys = vec![];
        let mut coldkeys = vec![];

        for i in 0..10 {
            old_hotkeys.push(U256::from(i * 2 + 1));
            new_hotkeys.push(U256::from(i * 2 + 2));
            coldkeys.push(U256::from(i * 2 + 11));
        }

        // Setup initial state
        for netuid in 1..=num_subnets {
            add_network(netuid, tempo, 0);
            SubtensorModule::set_max_registrations_per_block(netuid, 20);
            SubtensorModule::set_target_registrations_per_interval(netuid, 1000);
            log::info!(
                "Registrations this interval for netuid {:?} is {:?}",
                netuid,
                SubtensorModule::get_target_registrations_per_interval(netuid)
            );
            for i in 0..10 {
                register_ok_neuron(netuid, old_hotkeys[i], coldkeys[i], 0);
            }
        }

        // Add balance to coldkeys for swap cost
        for coldkey in coldkeys.iter().take(10) {
            SubtensorModule::add_balance_to_coldkey_account(coldkey, swap_cost);
        }

        // Perform the swaps for only two hotkeys
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkeys[0]),
            &old_hotkeys[0],
            &new_hotkeys[0]
        ));
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkeys[1]),
            &old_hotkeys[1],
            &new_hotkeys[1]
        ));

        // Verify the swaps
        for netuid in 1..=num_subnets {
            for i in 0..10 {
                if i == 0 || i == 1 {
                    assert_eq!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkeys[i]),
                        coldkeys[i]
                    );
                    assert_ne!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&old_hotkeys[i]),
                        coldkeys[i]
                    );

                    // Verify other storage changes
                    assert_eq!(
                        SubtensorModule::get_total_stake_for_hotkey(&new_hotkeys[i]),
                        SubtensorModule::get_total_stake_for_hotkey(&old_hotkeys[i])
                    );

                    assert_eq!(
                        SubtensorModule::get_delegate(new_hotkeys[i].encode()),
                        SubtensorModule::get_delegate(old_hotkeys[i].encode())
                    );

                    assert_eq!(
                        SubtensorModule::get_last_tx_block(&new_hotkeys[i]),
                        SubtensorModule::get_last_tx_block(&old_hotkeys[i])
                    );

                    // Verify raw storage maps
                    // Stake
                    for (coldkey, stake_amount) in Stake::<Test>::iter_prefix(old_hotkeys[i]) {
                        assert_eq!(Stake::<Test>::get(new_hotkeys[i], coldkey), stake_amount);
                    }

                    let mut weight = Weight::zero();
                    // UIDs
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert_eq!(
                            Uids::<Test>::get(netuid, new_hotkeys[i]),
                            Uids::<Test>::get(netuid, old_hotkeys[i])
                        );
                    }

                    // Prometheus
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert_eq!(
                            Prometheus::<Test>::get(netuid, new_hotkeys[i]),
                            Prometheus::<Test>::get(netuid, old_hotkeys[i])
                        );
                    }

                    // LoadedEmission
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert_eq!(
                            LoadedEmission::<Test>::get(netuid).unwrap(),
                            LoadedEmission::<Test>::get(netuid).unwrap()
                        );
                    }

                    // IsNetworkMember
                    for netuid in
                        SubtensorModule::get_netuid_is_member(&old_hotkeys[i], &mut weight)
                    {
                        assert!(IsNetworkMember::<Test>::contains_key(
                            new_hotkeys[i],
                            netuid
                        ));
                        assert!(!IsNetworkMember::<Test>::contains_key(
                            old_hotkeys[i],
                            netuid
                        ));
                    }

                    // Owner
                    assert_eq!(Owner::<Test>::get(new_hotkeys[i]), coldkeys[i]);

                    // Keys
                    for (uid, hotkey) in Keys::<Test>::iter_prefix(netuid) {
                        if hotkey == old_hotkeys[i] {
                            assert_eq!(Keys::<Test>::get(netuid, uid), new_hotkeys[i]);
                        }
                    }
                } else {
                    // Ensure other hotkeys remain unchanged
                    assert_eq!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&old_hotkeys[i]),
                        coldkeys[i]
                    );
                    assert_ne!(
                        SubtensorModule::get_owning_coldkey_for_hotkey(&new_hotkeys[i]),
                        coldkeys[i]
                    );
                }
            }
        }
    });
}

#[test]
fn test_swap_hotkey_tx_rate_limit_exceeded() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let old_hotkey = U256::from(1);
        let new_hotkey_1 = U256::from(2);
        let new_hotkey_2 = U256::from(4);
        let coldkey = U256::from(3);
        let swap_cost = 1_000_000_000u64 * 2;

        let tx_rate_limit = 1;

        // Get the current transaction rate limit
        let current_tx_rate_limit = SubtensorModule::get_tx_rate_limit();
        log::info!("current_tx_rate_limit: {:?}", current_tx_rate_limit);

        // Set the transaction rate limit
        SubtensorModule::set_tx_rate_limit(tx_rate_limit);
        // assert the rate limit is set to 1000 blocks
        assert_eq!(SubtensorModule::get_tx_rate_limit(), tx_rate_limit);

        // Setup initial state
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, old_hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, swap_cost);

        // Perform the first swap
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            &old_hotkey,
            &new_hotkey_1
        ));

        // Attempt to perform another swap immediately, which should fail due to rate limit
        assert_err!(
            SubtensorModule::do_swap_hotkey(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                &new_hotkey_1,
                &new_hotkey_2
            ),
            Error::<Test>::HotKeySetTxRateLimitExceeded
        );

        // move in time past the rate limit
        step_block(1001);
        assert_ok!(SubtensorModule::do_swap_hotkey(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            &new_hotkey_1,
            &new_hotkey_2
        ));
    });
}

#[test]
fn test_do_swap_hotkey_err_not_owner() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let not_owner_coldkey = U256::from(4);
        let swap_cost = 1_000_000_000u64;

        // Setup initial state
        add_network(netuid, tempo, 0);
        register_ok_neuron(netuid, old_hotkey, coldkey, 0);
        SubtensorModule::add_balance_to_coldkey_account(&not_owner_coldkey, swap_cost);

        // Attempt the swap with a non-owner coldkey
        assert_err!(
            SubtensorModule::do_swap_hotkey(
                <<Test as Config>::RuntimeOrigin>::signed(not_owner_coldkey),
                &old_hotkey,
                &new_hotkey
            ),
            Error::<Test>::NonAssociatedColdKey
        );
    });
}

#[test]
fn test_swap_owner_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Initialize Owner for old_hotkey
        Owner::<Test>::insert(old_hotkey, coldkey);

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the swap
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);
        assert!(!Owner::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_owner_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Ensure old_hotkey does not exist
        assert!(!Owner::<Test>::contains_key(old_hotkey));

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the swap
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);
        assert!(!Owner::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_owner_new_hotkey_already_exists() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let another_coldkey = U256::from(4);
        let mut weight = Weight::zero();

        // Initialize Owner for old_hotkey and new_hotkey
        Owner::<Test>::insert(old_hotkey, coldkey);
        Owner::<Test>::insert(new_hotkey, another_coldkey);

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the swap
        assert_eq!(Owner::<Test>::get(new_hotkey), coldkey);
        assert!(!Owner::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_owner_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let mut weight = Weight::zero();

        // Initialize Owner for old_hotkey
        Owner::<Test>::insert(old_hotkey, coldkey);

        // Perform the swap
        SubtensorModule::swap_owner(&old_hotkey, &new_hotkey, &coldkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_total_hotkey_stake_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let total_stake = 1000u64;
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyStake for old_hotkey
        TotalHotkeyStake::<Test>::insert(old_hotkey, total_stake);

        // Perform the swap
        SubtensorModule::swap_total_hotkey_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the swap
        assert_eq!(TotalHotkeyStake::<Test>::get(new_hotkey), total_stake);
        assert!(!TotalHotkeyStake::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_total_hotkey_stake_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let mut weight = Weight::zero();

        // Ensure old_hotkey does not exist
        assert!(!TotalHotkeyStake::<Test>::contains_key(old_hotkey));

        // Perform the swap
        SubtensorModule::swap_total_hotkey_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify that new_hotkey does not have a stake
        assert!(!TotalHotkeyStake::<Test>::contains_key(new_hotkey));
    });
}

#[test]
fn test_swap_total_hotkey_stake_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let total_stake = 1000u64;
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyStake for old_hotkey
        TotalHotkeyStake::<Test>::insert(old_hotkey, total_stake);

        // Perform the swap
        SubtensorModule::swap_total_hotkey_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_delegates_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let delegate_take = 10u16;
        let mut weight = Weight::zero();

        // Initialize Delegates for old_hotkey
        Delegates::<Test>::insert(old_hotkey, delegate_take);

        // Perform the swap
        SubtensorModule::swap_delegates(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the swap
        assert_eq!(Delegates::<Test>::get(new_hotkey), delegate_take);
        assert!(!Delegates::<Test>::contains_key(old_hotkey));
    });
}

#[test]
fn test_swap_delegates_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let mut weight = Weight::zero();

        // Ensure old_hotkey does not exist
        assert!(!Delegates::<Test>::contains_key(old_hotkey));

        // Perform the swap
        SubtensorModule::swap_delegates(&old_hotkey, &new_hotkey, &mut weight);

        // Verify that new_hotkey does not have a delegate
        assert!(!Delegates::<Test>::contains_key(new_hotkey));
    });
}

#[test]
fn test_swap_delegates_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let delegate_take = 10u16;
        let mut weight = Weight::zero();

        // Initialize Delegates for old_hotkey
        Delegates::<Test>::insert(old_hotkey, delegate_take);

        // Perform the swap
        SubtensorModule::swap_delegates(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_stake_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake_amount = 1000u64;
        let mut weight = Weight::zero();

        // Initialize Stake for old_hotkey
        Stake::<Test>::insert(old_hotkey, coldkey, stake_amount);

        // Perform the swap
        SubtensorModule::swap_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the swap
        assert_eq!(Stake::<Test>::get(new_hotkey, coldkey), stake_amount);
        assert!(!Stake::<Test>::contains_key(old_hotkey, coldkey));
    });
}

#[test]
fn test_swap_stake_old_hotkey_not_exist() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake_amount = 1000u64;
        let mut weight = Weight::zero();

        // Initialize Stake for old_hotkey
        Stake::<Test>::insert(old_hotkey, coldkey, stake_amount);

        // Ensure old_hotkey has a stake
        assert!(Stake::<Test>::contains_key(old_hotkey, coldkey));

        // Perform the swap
        SubtensorModule::swap_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify that new_hotkey has the stake and old_hotkey does not
        assert!(Stake::<Test>::contains_key(new_hotkey, coldkey));
        assert!(!Stake::<Test>::contains_key(old_hotkey, coldkey));
    });
}

#[test]
fn test_swap_stake_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake_amount = 1000u64;
        let mut weight = Weight::zero();

        // Initialize Stake for old_hotkey
        Stake::<Test>::insert(old_hotkey, coldkey, stake_amount);

        // Perform the swap
        SubtensorModule::swap_stake(&old_hotkey, &new_hotkey, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_is_network_member_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let mut weight = Weight::zero();

        // Initialize IsNetworkMember for old_hotkey
        for netuid in &netuid_is_member {
            IsNetworkMember::<Test>::insert(old_hotkey, netuid, true);
        }

        // Perform the swap
        SubtensorModule::swap_is_network_member(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the swap
        for netuid in &netuid_is_member {
            assert!(IsNetworkMember::<Test>::contains_key(new_hotkey, netuid));
            assert!(!IsNetworkMember::<Test>::contains_key(old_hotkey, netuid));
        }
    });
}

#[test]
fn test_swap_is_network_member_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let mut weight = Weight::zero();

        // Initialize IsNetworkMember for old_hotkey
        for netuid in &netuid_is_member {
            IsNetworkMember::<Test>::insert(old_hotkey, netuid, true);
        }

        // Perform the swap
        SubtensorModule::swap_is_network_member(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_axons_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let axon_info = AxonInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
            protocol: 1,
            placeholder1: 0,
            placeholder2: 0,
        };
        let mut weight = Weight::zero();

        // Initialize Axons for old_hotkey
        for netuid in &netuid_is_member {
            Axons::<Test>::insert(netuid, old_hotkey, axon_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_axons(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            assert_eq!(Axons::<Test>::get(netuid, new_hotkey).unwrap(), axon_info);
            assert!(!Axons::<Test>::contains_key(netuid, old_hotkey));
        }
    });
}

#[test]
fn test_swap_axons_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let axon_info = AxonInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
            protocol: 1,
            placeholder1: 0,
            placeholder2: 0,
        };
        let mut weight = Weight::zero();

        // Initialize Axons for old_hotkey
        for netuid in &netuid_is_member {
            Axons::<Test>::insert(netuid, old_hotkey, axon_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_axons(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = netuid_is_member.len() as u64
            * <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_keys_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Keys for old_hotkey
        for netuid in &netuid_is_member {
            log::info!("Inserting old_hotkey:{:?} netuid:{:?}", old_hotkey, netuid);
            Keys::<Test>::insert(*netuid, uid, old_hotkey);
        }

        // Perform the swap
        SubtensorModule::swap_keys(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            log::info!(
                "neutuid, uid, hotkey: {:?}, {:?}, {:?}",
                netuid,
                uid,
                new_hotkey
            );
            assert_eq!(Keys::<Test>::get(netuid, uid), new_hotkey);
        }
    });
}

#[test]
fn test_swap_keys_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Keys for old_hotkey
        for netuid in &netuid_is_member {
            Keys::<Test>::insert(*netuid, uid, old_hotkey);
        }

        // Perform the swap
        SubtensorModule::swap_keys(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_loaded_emission_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let se = 100u64;
        let ve = 200u64;
        let mut weight = Weight::zero();

        // Initialize LoadedEmission for old_hotkey
        for netuid in &netuid_is_member {
            LoadedEmission::<Test>::mutate(netuid, |emission_exists| {
                if let Some(emissions) = emission_exists {
                    emissions.push((old_hotkey, se, ve));
                } else {
                    *emission_exists = Some(vec![(old_hotkey, se, ve)]);
                }
            });
        }

        // Perform the swap
        SubtensorModule::swap_loaded_emission(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the swap
        for netuid in &netuid_is_member {
            let emissions = LoadedEmission::<Test>::get(netuid).unwrap();
            assert!(emissions.iter().any(|(hk, _, _)| hk == &new_hotkey));
            assert!(!emissions.iter().any(|(hk, _, _)| hk == &old_hotkey));
        }
    });
}

#[test]
fn test_swap_loaded_emission_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        // let uid = 42u64;
        let se = 100u64;
        let ve = 200u64;
        let mut weight = Weight::zero();

        // Initialize LoadedEmission for old_hotkey
        for netuid in &netuid_is_member {
            LoadedEmission::<Test>::mutate(netuid, |emission_exists| {
                if let Some(emissions) = emission_exists {
                    emissions.push((old_hotkey, se, ve));
                } else {
                    *emission_exists = Some(vec![(old_hotkey, se, ve)]);
                }
            });
        }

        // Perform the swap
        SubtensorModule::swap_loaded_emission(
            &old_hotkey,
            &new_hotkey,
            &netuid_is_member,
            &mut weight,
        );

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_uids_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Uids for old_hotkey
        for netuid in &netuid_is_member {
            Uids::<Test>::insert(netuid, old_hotkey, uid);
        }

        // Perform the swap
        SubtensorModule::swap_uids(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            assert_eq!(Uids::<Test>::get(netuid, new_hotkey).unwrap(), uid);
            assert!(!Uids::<Test>::contains_key(netuid, old_hotkey));
        }
    });
}

#[test]
fn test_swap_uids_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let uid = 42u16;
        let mut weight = Weight::zero();

        // Initialize Uids for old_hotkey
        for netuid in &netuid_is_member {
            Uids::<Test>::insert(netuid, old_hotkey, uid);
        }

        // Perform the swap
        SubtensorModule::swap_uids(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(4);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_prometheus_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let prometheus_info = PrometheusInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
        };
        let mut weight = Weight::zero();

        // Initialize Prometheus for old_hotkey
        for netuid in &netuid_is_member {
            Prometheus::<Test>::insert(netuid, old_hotkey, prometheus_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_prometheus(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the swap
        for netuid in &netuid_is_member {
            assert_eq!(
                Prometheus::<Test>::get(netuid, new_hotkey).unwrap(),
                prometheus_info
            );
            assert!(!Prometheus::<Test>::contains_key(netuid, old_hotkey));
        }
    });
}

#[test]
fn test_swap_prometheus_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let netuid_is_member = vec![1u16, 2u16];
        let prometheus_info = PrometheusInfo {
            block: 100,
            version: 1,
            ip: 0x1234567890abcdef,
            port: 8080,
            ip_type: 4,
        };
        let mut weight = Weight::zero();

        // Initialize Prometheus for old_hotkey
        for netuid in &netuid_is_member {
            Prometheus::<Test>::insert(netuid, old_hotkey, prometheus_info.clone());
        }

        // Perform the swap
        SubtensorModule::swap_prometheus(&old_hotkey, &new_hotkey, &netuid_is_member, &mut weight);

        // Verify the weight update
        let expected_weight = netuid_is_member.len() as u64
            * <Test as frame_system::Config>::DbWeight::get().reads_writes(1, 2);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_swap_total_hotkey_coldkey_stakes_this_interval_success() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake = (1000u64, 42u64); // Example tuple value
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyColdkeyStakesThisInterval for old_hotkey
        TotalHotkeyColdkeyStakesThisInterval::<Test>::insert(old_hotkey, coldkey, stake);

        // Perform the swap
        SubtensorModule::swap_total_hotkey_coldkey_stakes_this_interval(
            &old_hotkey,
            &new_hotkey,
            &mut weight,
        );

        // Verify the swap
        assert_eq!(
            TotalHotkeyColdkeyStakesThisInterval::<Test>::get(new_hotkey, coldkey),
            stake
        );
        assert!(!TotalHotkeyColdkeyStakesThisInterval::<Test>::contains_key(
            old_hotkey, coldkey
        ));
    });
}

#[test]
fn test_swap_total_hotkey_coldkey_stakes_this_interval_weight_update() {
    new_test_ext(1).execute_with(|| {
        let old_hotkey = U256::from(1);
        let new_hotkey = U256::from(2);
        let coldkey = U256::from(3);
        let stake = (1000u64, 42u64);
        let mut weight = Weight::zero();

        // Initialize TotalHotkeyColdkeyStakesThisInterval for old_hotkey
        TotalHotkeyColdkeyStakesThisInterval::<Test>::insert(old_hotkey, coldkey, stake);

        // Perform the swap

        SubtensorModule::swap_total_hotkey_coldkey_stakes_this_interval(
            &old_hotkey,
            &new_hotkey,
            &mut weight,
        );

        // Verify the weight update
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().writes(2);
        assert_eq!(weight, expected_weight);
    });
}
