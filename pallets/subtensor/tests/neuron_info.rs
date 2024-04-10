mod mock;
use codec::Compact;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use sp_core::U256;

#[test]
fn test_get_neuron_none() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let uid: u16 = 42;

        let neuron = SubtensorModule::get_neuron(netuid, uid);
        assert_eq!(neuron, None);
    });
}

#[test]
#[cfg(not(tarpaulin))]
fn test_get_neuron_some() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 2;
        let modality: u16 = 2;

        let uid: u16 = 0;
        let hotkey0 = U256::from(0);
        let coldkey0 = U256::from(0);

        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey0, coldkey0, 39420842);

        let neuron = SubtensorModule::get_neuron(netuid, uid);
        assert_ne!(neuron, None);
    });
}

/* @TODO: Add more neurons to list */
#[test]
fn test_get_neurons_list() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 2;
        let modality: u16 = 2;

        add_network(netuid, tempo, modality);

        let _uid: u16 = 42;

        let neuron_count = 1;
        for index in 0..neuron_count {
            let hotkey = U256::from(0 + index);
            let coldkey = U256::from(0 + index);
            let nonce: u64 = 39420842 + index;
            register_ok_neuron(netuid, hotkey, coldkey, nonce);
        }

        let neurons = SubtensorModule::get_neurons(netuid);
        log::info!("neurons: {:?}", neurons);
        assert_eq!(neurons.len(), neuron_count as usize);
    });
}

#[test]
fn test_get_neurons_empty() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;

        let neuron_count = 0;
        let neurons = SubtensorModule::get_neurons(netuid);
        log::info!("neurons: {:?}", neurons);
        assert_eq!(neurons.len(), neuron_count as usize);
    });
}

#[test]
fn test_get_neuron_subnet_staking_info() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 2;
        let modality: u16 = 2;

        let uid: u16 = 0;
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(12);
        let stake_amount: u64 = 1;

        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey0, coldkey0, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, stake_amount + 5);

        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            stake_amount,
        ));

        let neuron = SubtensorModule::get_neuron_lite(netuid, uid);
        log::info!("neuron: {:?}", neuron);
        assert_eq!(
            neuron.unwrap().stake,
            vec![(coldkey0, Compact(stake_amount))]
        );
    });
}

#[test]
fn test_get_neuron_subnet_staking_info_multiple() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;

        let tempo: u16 = 2;
        let modality: u16 = 2;

        add_network(netuid, tempo, modality);

        let stake_amounts: [u64; 5] = [1, 2, 3, 4, 5];
        let mut expected_stakes = Vec::new();

        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_target_registrations_per_interval(netuid, 10);

        for (index, &stake_amount) in stake_amounts.iter().enumerate() {
            let _uid: u16 = index as u16;
            let hotkey = U256::from(index as u64);
            let coldkey = U256::from((index + 10) as u64);

            register_ok_neuron(netuid, hotkey, coldkey, 39420842 + index as u64);
            // Adding more because of existential deposit
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, stake_amount + 5);

            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                stake_amount,
            ));

            expected_stakes.push((coldkey, Compact(stake_amount)));
            step_block(1);
        }
        log::info!("expected_stakes: {:?}", expected_stakes);
        // Retrieve and assert for each neuron
        for (index, &(ref coldkey, ref stake)) in expected_stakes.iter().enumerate() {
            let uid: u16 = index as u16;
            let neuron =
                SubtensorModule::get_neuron_lite(netuid, uid).expect("Neuron should exist");

            assert!(
                neuron.stake.contains(&(coldkey.clone(), stake.clone())),
                "Stake for uid {} does not match expected value",
                uid
            );
        }
    });
}

#[test]
fn test_get_neuron_stake_based_on_netuid() {
    new_test_ext(1).execute_with(|| {
        let netuid_root: u16 = 0; // Root network
        let netuid_sub: u16 = 1; // Subnetwork

        let uid_root: u16 = 0;
        let uid_sub: u16 = 1;

        let hotkey_root = U256::from(0);
        let coldkey_root = U256::from(0);
        let stake_amount_root: u64 = 100;

        let hotkey_sub = U256::from(1);
        let coldkey_sub = U256::from(1);
        let stake_amount_sub: u64 = 200;

        // Setup for root network
        add_network(netuid_root, 2, 2);
        add_network(netuid_sub, 2, 2);
        register_ok_neuron(netuid_sub, hotkey_root, coldkey_root, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_root, stake_amount_root);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_root),
            hotkey_root,
            stake_amount_root,
        ));

        step_block(1);

        // Setup for subnetwork
        // add_network(netuid_sub, 2, 2);
        register_ok_neuron(netuid_sub, hotkey_sub, coldkey_sub, 39420843);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_sub, stake_amount_sub);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_sub),
            hotkey_sub,
            netuid_sub,
            stake_amount_sub,
        ));

        // Test for main network
        let neuron_main = SubtensorModule::get_neuron(netuid_sub, uid_root)
            .expect("Neuron should exist for main network");
        assert_eq!(
            neuron_main.stake.len(),
            1,
            "Main network should have 1 stake entry"
        );
        // assert_eq!(
        //     neuron_main.stake[0].1 .0, stake_amount_root,
        //     "Stake amount for main network does not match"
        // );

        // Test for subnetwork
        let neuron_sub = SubtensorModule::get_neuron(netuid_sub, uid_sub)
            .expect("Neuron should exist for subnetwork");
        assert_eq!(
            neuron_sub.stake.len(),
            1,
            "Subnetwork should have 1 stake entry"
        );
        assert_eq!(
            neuron_sub.stake[0].1 .0,
            // Need to account for existential deposit
            stake_amount_sub - 1,
            "Stake amount for subnetwork does not match"
        );
    });
}

#[test]
fn test_adding_substake_affects_only_targeted_neuron() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 2;
        let modality: u16 = 2;

        // Setup the network and neurons
        add_network(netuid, tempo, modality);
        let neuron_count = 5;
        let total_stake: u64 = neuron_count as u64 * 1000;
        let initial_stake: u64 = 1000;

        SubtensorModule::set_target_stakes_per_interval(10000);
        SubtensorModule::set_max_registrations_per_block(netuid, neuron_count);
        SubtensorModule::set_target_registrations_per_interval(netuid, neuron_count);

        // Register neurons and add initial stake
        for i in 0..neuron_count {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i);
            register_ok_neuron(netuid, hotkey, coldkey, 39420842 + i as u64);
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, total_stake);
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                initial_stake,
            ));
        }

        // Add sub-stake to the first neuron
        let target_neuron_index: u16 = 0;
        let additional_stake: u64 = 500;
        let target_hotkey = U256::from(target_neuron_index);
        let target_coldkey = U256::from(target_neuron_index);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(target_coldkey),
            target_hotkey,
            netuid,
            additional_stake,
        ));

        // Check that only the targeted neuron's stake has increased
        for i in 0..neuron_count {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i);
            let expected_stake = if i == target_neuron_index {
                initial_stake + additional_stake
            } else {
                initial_stake
            };
            let neuron_stake =
                SubtensorModule::get_subnet_stake_for_coldkey_and_hotkey(&coldkey, &hotkey, netuid);
            assert_eq!(
                neuron_stake, expected_stake,
                "Neuron {} stake does not match expected value. Expected: {}, Got: {}",
                i, expected_stake, neuron_stake
            );
        }
    });
}

#[test]
fn test_adding_substake_affects_only_targeted_neuron_with_get_neurons_lite() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 2;
        let modality: u16 = 2;

        log::info!("Setting up the network and neurons");
        add_network(netuid, tempo, modality);
        let neuron_count = 5;
        let initial_stake: u64 = 1000;

        SubtensorModule::set_target_stakes_per_interval(10000);
        SubtensorModule::set_max_registrations_per_block(netuid, neuron_count);
        SubtensorModule::set_target_registrations_per_interval(netuid, neuron_count);

        // Register neurons and add initial stake
        for i in 0..neuron_count {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i);
            log::info!(
                "Registering neuron {} with hotkey {:?} and coldkey {:?}",
                i,
                hotkey,
                coldkey
            );
            register_ok_neuron(netuid, hotkey, coldkey, 0 as u64);
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake * 5);
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                initial_stake,
            ));
        }

        // Add sub-stake to the targeted neuron
        let target_neuron_index: u16 = 2;
        let additional_stake: u64 = 500;
        log::info!("Adding additional stake to neuron {}", target_neuron_index);
        let target_hotkey = U256::from(target_neuron_index);
        let target_coldkey = U256::from(target_neuron_index);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(target_coldkey),
            target_hotkey,
            netuid,
            additional_stake,
        ));

        // Retrieve all neurons using get_neurons_lite and check stakes
        let neurons_lite = SubtensorModule::get_neurons_lite(netuid);
        log::info!(
            "Retrieved {} neurons using get_neurons_lite",
            neurons_lite.len()
        );
        assert_eq!(
            neurons_lite.len(),
            neuron_count as usize,
            "There should be {} neurons",
            neuron_count
        );


        // Check that only the targeted neuron's stake has increased
    for neuron in neurons_lite.into_iter() {
    // Find the stake for the neuron based on its identifier (assuming the identifier is the first element in the tuple)
    let neuron_stake = neuron.stake.iter().find(|(id, _)| *id == neuron.hotkey).expect("Neuron stake not found");

    let expected_stake = if neuron.hotkey == U256::from(target_neuron_index) {
        Compact(initial_stake + additional_stake)
    } else {
        Compact(initial_stake)
    };
    log::info!("Stake in all neurons: {:?}", neuron.stake);
    log::info!("Neurons: {:?}", neuron);
    log::info!("Neurons UID: {:?}", neuron.uid);
    log::info!("Checking stake for neuron with hotkey {:?}: Expected: {:?}, Got: {:?}", neuron.hotkey, expected_stake, neuron_stake.1);
    assert_eq!(
        neuron_stake.1, expected_stake,
        "Stake does not match expected value for neuron with hotkey {:?}. Expected: {:?}, Got: {:?}",
        neuron.hotkey, expected_stake, neuron_stake.1
    );
}
    });
}

#[test]
fn test_adding_substake_affects_only_targeted_neuron_with_get_neuron_lite() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 2;
        let modality: u16 = 2;

        log::info!("Setting up the network and neurons");
        add_network(netuid, tempo, modality);
        let neuron_count = 5;
        let initial_stake: u64 = 1000;

        SubtensorModule::set_target_stakes_per_interval(10000);
        SubtensorModule::set_max_registrations_per_block(netuid, neuron_count);
        SubtensorModule::set_target_registrations_per_interval(netuid, neuron_count);

        // Append neurons and add initial stake
        for i in 0..neuron_count {
            let hotkey = U256::from(i);
            let coldkey = U256::from(i);
            log::info!(
                "Appending neuron {} with hotkey {:?} and coldkey {:?}",
                i,
                hotkey,
                coldkey
            );
            register_ok_neuron(netuid, hotkey, coldkey, 0 as u64);
            SubtensorModule::add_balance_to_coldkey_account(&coldkey, initial_stake * 5);
            assert_ok!(SubtensorModule::add_subnet_stake(
                <<Test as Config>::RuntimeOrigin>::signed(coldkey),
                hotkey,
                netuid,
                initial_stake,
            ));
        }

        // Add sub-stake to the targeted neuron
        let target_neuron_index: u16 = 0;
        let additional_stake: u64 = 500;
        let target_hotkey = U256::from(target_neuron_index);
        let target_coldkey = U256::from(target_neuron_index);
        log::info!("Adding additional stake to neuron {}", target_neuron_index);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(target_coldkey),
            target_hotkey,
            netuid,
            additional_stake,
        ));

        // Retrieve and check all neurons to ensure only the targeted neuron's stake has increased
        for i in 0..neuron_count {
            let neuron_index = i as u16;
            if let Some(neuron_lite) = SubtensorModule::get_neuron_lite(netuid, neuron_index) {
                let neuron_hotkey = U256::from(i);
                let found_stake_tuple = neuron_lite
                    .stake
                    .iter()
                    .find(|(hotkey, _)| *hotkey == neuron_hotkey);
                if let Some((_, stake)) = found_stake_tuple {
                    let stake_value: u64 = stake.0; // Assuming `Compact` is a wrapper around the value.
                    let expected_stake = if neuron_index == target_neuron_index {
                        initial_stake + additional_stake
                    } else {
                        initial_stake
                    };
                    log::info!(
                        "Checking stake for neuron {}: Expected: {}, Got: {}",
                        i,
                        expected_stake,
                        stake_value
                    );
                    assert_eq!(
                        stake_value, expected_stake,
                        "Stake does not match expected value for neuron {}. Expected: {}, Got: {}",
                        i, expected_stake, stake_value
                    );
                } else {
                    panic!("Stake for neuron with hotkey {:?} not found", neuron_hotkey);
                }
            } else {
                panic!("Neuron with index {} not found", neuron_index);
            }
        }
    });
}
