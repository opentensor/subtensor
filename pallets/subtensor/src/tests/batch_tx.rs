use codec::Compact;
use frame_support::{assert_ok, traits::Currency};
use frame_system::Config;
use sp_core::{H256, U256};
use sp_runtime::{
    traits::{BlakeTwo256, Hash},
    DispatchError,
};

use crate::{Error, Event};

use super::mock::*;

#[test]
fn test_batch_txs() {
    let alice = U256::from(0);
    let bob = U256::from(1);
    let charlie = U256::from(2);
    let initial_balances = vec![
        (alice, 8_000_000_000),
        (bob, 1_000_000_000),
        (charlie, 1_000_000_000),
    ];
    test_ext_with_balances(initial_balances).execute_with(|| {
        assert_ok!(Utility::batch(
            <<Test as Config>::RuntimeOrigin>::signed(alice),
            vec![
                RuntimeCall::Balances(BalanceCall::transfer_allow_death {
                    dest: bob,
                    value: 1_000_000_000
                }),
                RuntimeCall::Balances(BalanceCall::transfer_allow_death {
                    dest: charlie,
                    value: 1_000_000_000
                })
            ]
        ));
        assert_eq!(Balances::total_balance(&alice), 6_000_000_000);
        assert_eq!(Balances::total_balance(&bob), 2_000_000_000);
        assert_eq!(Balances::total_balance(&charlie), 2_000_000_000);
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --test batch_tx -- test_batch_set_weights --exact --nocapture
#[test]
fn test_batch_set_weights() {
    // Verify the batch set weights call works
    new_test_ext(1).execute_with(|| {
        let netuid_0: u16 = 1;
        let netuid_1: u16 = 2;
        let netuid_2: u16 = 3;

        // Create 3 networks
        add_network(netuid_0, 1, 0);
        add_network(netuid_1, 2, 0);
        add_network(netuid_2, 3, 0);

        let hotkey: U256 = U256::from(2);
        let spare_hk: U256 = U256::from(3);

        let coldkey: U256 = U256::from(101);
        let spare_ck = U256::from(102);

        let stake_to_give_child = 109_999;

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake_to_give_child + 10);

        // Register both hotkeys on each network
        register_ok_neuron(netuid_0, hotkey, coldkey, 1);
        register_ok_neuron(netuid_0, spare_hk, spare_ck, 1);

        register_ok_neuron(netuid_1, hotkey, coldkey, 1);
        register_ok_neuron(netuid_1, spare_hk, spare_ck, 1);

        register_ok_neuron(netuid_2, hotkey, coldkey, 1);
        register_ok_neuron(netuid_2, spare_hk, spare_ck, 1);

        // Increase stake on hotkey setting the weights
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey,
            &hotkey,
            stake_to_give_child,
        );

        // Set the rate limit to 0 for all networks
        SubtensorModule::set_weights_set_rate_limit(netuid_0, 0);
        SubtensorModule::set_weights_set_rate_limit(netuid_1, 0);
        SubtensorModule::set_weights_set_rate_limit(netuid_2, 0);

        // Has stake and no parent
        step_block(7200 + 1);

        // Set weights on the other hotkey and Use maximum value for u16
        let weights: Vec<(Compact<u16>, Compact<u16>)> = vec![(Compact(1), Compact(u16::MAX))];
        let version_key_0: Compact<u64> = SubtensorModule::get_weights_version_key(netuid_0).into();
        let version_key_1: Compact<u64> = SubtensorModule::get_weights_version_key(netuid_1).into();
        let version_key_2: Compact<u64> = SubtensorModule::get_weights_version_key(netuid_2).into();

        // Set the min stake very high
        SubtensorModule::set_stake_threshold(stake_to_give_child * 5);

        // Check the key has less stake than required
        assert!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid_0)
                < SubtensorModule::get_stake_threshold()
        );

        let netuids_vec: Vec<Compact<u16>> =
            vec![netuid_0.into(), netuid_1.into(), netuid_2.into()];

        // Check the batch succeeds (force set weights)
        assert_ok!(SubtensorModule::batch_set_weights(
            RuntimeOrigin::signed(hotkey),
            netuids_vec.clone(),
            vec![weights.clone(), weights.clone(), weights.clone()], // One per network
            vec![version_key_0, version_key_1, version_key_2]
        ));

        // Check the events are emitted, three errors about not enough stake
        // Also events for batch completed with errors and batch complete with errors
        assert!(System::events().iter().any(|event| matches!(
            event.event.clone(),
            RuntimeEvent::SubtensorModule(Event::BatchWeightsCompleted { .. })
        )));
        assert!(System::events().iter().any(|event| matches!(
            event.event.clone(),
            RuntimeEvent::SubtensorModule(Event::BatchCompletedWithErrors { .. })
        )));

        let expected_err: DispatchError = Error::<Test>::NotEnoughStakeToSetWeights.into();

        assert_eq!(
            System::events()
                .iter()
                .filter(|event| match event.event {
                    RuntimeEvent::SubtensorModule(Event::BatchWeightItemFailed(err)) =>
                        err == expected_err,
                    _ => false,
                })
                .collect::<Vec<_>>()
                .len(),
            3, // Three not enough stake errors
            "{:?}",
            System::events()
        );

        // Reset the events
        System::reset_events();

        assert!(!SubtensorModule::check_weights_min_stake(&hotkey, netuid_0));

        // Set a minimum stake to set weights
        SubtensorModule::set_stake_threshold(stake_to_give_child - 5);

        // Check if the stake for the hotkey is above
        assert!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid_0)
                >= SubtensorModule::get_stake_threshold()
        );

        // Try with enough stake
        assert_ok!(SubtensorModule::batch_set_weights(
            RuntimeOrigin::signed(hotkey),
            netuids_vec.clone(),
            vec![weights.clone(), weights.clone(), weights.clone()],
            vec![version_key_0, version_key_1, version_key_2]
        ));

        assert!(SubtensorModule::check_weights_min_stake(&hotkey, netuid_0));

        // Check the events are emitted, no errors
        assert!(System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchWeightsCompleted { .. })
        )));

        // No errors
        assert!(!System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchCompletedWithErrors { .. })
        )));
        assert!(!System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchWeightItemFailed { .. })
        )));

        // Reset events
        System::reset_events();

        // Test again, but with only one failure, different reason
        // Set version key higher for just one network
        SubtensorModule::set_weights_version_key(netuid_2, u64::from(version_key_2) + 1_u64);
        // Verify the version key is *not* correct
        assert!(!SubtensorModule::check_version_key(
            netuid_2,
            version_key_2.into()
        ));
        assert_ok!(SubtensorModule::batch_set_weights(
            RuntimeOrigin::signed(hotkey),
            netuids_vec.clone(),
            vec![weights.clone(), weights.clone(), weights.clone()],
            vec![version_key_0, version_key_1, version_key_2] // Version key 2 is not correct
        ));

        // Check the events are emitted, one error
        assert!(System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchWeightsCompleted { .. })
        )));
        assert!(System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchCompletedWithErrors { .. })
        )));

        // Only one error
        assert_eq!(
            System::events()
                .iter()
                .filter(|event| matches!(
                    event.event,
                    RuntimeEvent::SubtensorModule(Event::BatchWeightItemFailed(..))
                ))
                .collect::<Vec<_>>()
                .len(),
            1
        );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --test batch_tx -- test_batch_commit_weights --exact --nocapture
#[test]
fn test_batch_commit_weights() {
    // Verify the batch set weights call works
    new_test_ext(1).execute_with(|| {
        let netuid_0: u16 = 1;
        let netuid_1: u16 = 2;
        let netuid_2: u16 = 3;

        // Create 3 networks
        add_network(netuid_0, 1, 0);
        add_network(netuid_1, 1, 0);
        add_network(netuid_2, 1, 0);

        let hotkey: U256 = U256::from(2);
        let spare_hk: U256 = U256::from(3);

        let coldkey: U256 = U256::from(101);
        let spare_ck = U256::from(102);

        let stake_to_give_child = 109_999;

        SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake_to_give_child + 10);

        // Register both hotkeys on each network
        register_ok_neuron(netuid_0, hotkey, coldkey, 1);
        register_ok_neuron(netuid_0, spare_hk, spare_ck, 1);

        register_ok_neuron(netuid_1, hotkey, coldkey, 1);
        register_ok_neuron(netuid_1, spare_hk, spare_ck, 1);

        register_ok_neuron(netuid_2, hotkey, coldkey, 1);
        register_ok_neuron(netuid_2, spare_hk, spare_ck, 1);

        // Increase stake on hotkey setting the weights
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &coldkey,
            &hotkey,
            stake_to_give_child,
        );

        // Set the rate limit to 0 for all networks
        SubtensorModule::set_weights_set_rate_limit(netuid_0, 0);
        SubtensorModule::set_weights_set_rate_limit(netuid_1, 0);
        SubtensorModule::set_weights_set_rate_limit(netuid_2, 0);

        // Disable commit reveal for all networks (pre-emptively)
        SubtensorModule::set_commit_reveal_weights_enabled(netuid_0, false);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid_1, false);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid_2, false);

        // Has stake and no parent
        step_block(7200 + 1);

        let hash: H256 = BlakeTwo256::hash_of(&vec![1, 2, 3]);

        let netuids_vec: Vec<Compact<u16>> =
            vec![netuid_0.into(), netuid_1.into(), netuid_2.into()];

        // Check the batch succeeds (force commit weights)
        assert_ok!(SubtensorModule::batch_commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuids_vec.clone(),
            vec![hash, hash, hash], // One per network
        ));

        // Check the events are emitted, three errors about commit reveal disabled
        // Also events for batch completed with errors and batch complete with errors
        assert!(System::events().iter().any(|event| matches!(
            event.event.clone(),
            RuntimeEvent::SubtensorModule(Event::BatchWeightsCompleted { .. })
        )));
        assert!(System::events().iter().any(|event| matches!(
            event.event.clone(),
            RuntimeEvent::SubtensorModule(Event::BatchCompletedWithErrors { .. })
        )));

        let expected_err: DispatchError = Error::<Test>::CommitRevealDisabled.into();

        assert_eq!(
            System::events()
                .iter()
                .filter(|event| match event.event {
                    RuntimeEvent::SubtensorModule(Event::BatchWeightItemFailed(err)) =>
                        err == expected_err,
                    _ => false,
                })
                .collect::<Vec<_>>()
                .len(),
            3 // Three commit reveal disabled errors
        );

        // Reset the events
        System::reset_events();

        // Enable commit reveal for all networks
        SubtensorModule::set_commit_reveal_weights_enabled(netuid_0, true);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid_1, true);
        SubtensorModule::set_commit_reveal_weights_enabled(netuid_2, true);

        // Set a minimum stake to set weights
        SubtensorModule::set_stake_threshold(stake_to_give_child - 5);

        // Check if the stake for the hotkey is above
        assert!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey, netuid_0)
                >= SubtensorModule::get_stake_threshold()
        );

        // Try with commit reveal enabled
        assert_ok!(SubtensorModule::batch_commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuids_vec.clone(),
            vec![hash, hash, hash]
        ));

        assert!(SubtensorModule::check_weights_min_stake(&hotkey, netuid_0));

        // Check the events are emitted, no errors
        assert!(System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchWeightsCompleted { .. })
        )));

        // No errors
        assert!(!System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchCompletedWithErrors { .. })
        )));
        assert!(!System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchWeightItemFailed { .. })
        )));

        // Reset events
        System::reset_events();

        // Test again, but with only one failure, different reason
        // Disable commit reveal for one network
        SubtensorModule::set_commit_reveal_weights_enabled(netuid_2, false);
        assert_ok!(SubtensorModule::batch_commit_weights(
            RuntimeOrigin::signed(hotkey),
            netuids_vec.clone(),
            vec![hash, hash, hash]
        ));

        // Check the events are emitted, one error
        assert!(System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchWeightsCompleted { .. })
        )));
        assert!(System::events().iter().any(|event| matches!(
            event.event,
            RuntimeEvent::SubtensorModule(Event::BatchCompletedWithErrors { .. })
        )));

        // Only one error
        assert_eq!(
            System::events()
                .iter()
                .filter(|event| matches!(
                    event.event,
                    RuntimeEvent::SubtensorModule(Event::BatchWeightItemFailed(..))
                ))
                .collect::<Vec<_>>()
                .len(),
            1
        );
    });
}
