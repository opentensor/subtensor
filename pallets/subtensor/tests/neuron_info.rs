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
            let hotkey = U256::from(index);
            let coldkey = U256::from(index);
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
        let stake_amount = 1000;
        let stake_weight = u16::MAX as u64;

        add_network(netuid, tempo, modality);
        register_ok_neuron(netuid, hotkey0, coldkey0, 39420842);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey0, stake_amount);

        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid,
            stake_amount,
        ));

        step_block(tempo);

        let neuron = SubtensorModule::get_neuron_lite(netuid, uid);
        log::info!("neuron: {:?}", neuron);
        assert_eq!(
            neuron.unwrap().stake,
            vec![(coldkey0, Compact(stake_weight))]
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

        let stake_amounts: [u64; 5] = [1000, 2000, 3000, 4000, 5000];
        let total_stake = 15000;

        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_target_registrations_per_interval(netuid, 10);

        let expected_stakes: Vec<(U256, Compact<u64>)> = stake_amounts
            .iter()
            .enumerate()
            .map(|(index, &stake_amount)| {
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
                let stake_weight =
                    (u16::MAX as f32 * stake_amount as f32 / total_stake as f32) as u64;

                (coldkey, Compact(stake_weight))
            })
            .collect();
        log::info!("expected_stakes: {:?}", expected_stakes);

        step_block(2);

        // Retrieve and assert for each neuron
        expected_stakes.iter().enumerate().for_each(
            |(index, &(expected_coldkey, Compact(expected_stake_weight)))| {
                let uid: u16 = index as u16;
                let neuron =
                    SubtensorModule::get_neuron_lite(netuid, uid).expect("Neuron should exist");

                let (coldkey, Compact(stake_weight)) = neuron.stake[0];

                assert_eq!(expected_coldkey, coldkey,);
                // Divide by 10 to mask rounding errors
                assert_eq!(expected_stake_weight / 10, stake_weight / 10,);
            },
        );
    });
}

#[test]
fn test_get_neuron_stake_based_on_netuid() {
    new_test_ext(1).execute_with(|| {
        let netuid_root: u16 = 0; // Root network
        let netuid_sub: u16 = 1; // Subnetwork
        let tempo = 2;

        let uid_0: u16 = 0;

        let hotkey_root = U256::from(0);
        let coldkey_root = U256::from(0);
        let stake_amount_root: u64 = 1000;

        let hotkey_sub = U256::from(1);
        let coldkey_sub = U256::from(1);
        let stake_amount_sub: u64 = 2000;

        // Setup for root network
        add_network(netuid_root, tempo, 2);
        SubtensorModule::create_account_if_non_existent(&coldkey_root, &hotkey_root);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_root, stake_amount_root);
        assert_ok!(SubtensorModule::add_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_root),
            hotkey_root,
            stake_amount_root,
        ));

        // Setup for subnetwork
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_sub, stake_amount_sub);
        add_network(netuid_sub, tempo, 2);
        register_ok_neuron(netuid_sub, hotkey_sub, coldkey_sub, 39420843);
        // assert_ok!(SubtensorModule::add_subnet_stake(
        //     <<Test as Config>::RuntimeOrigin>::signed(coldkey_sub),
        //     hotkey_sub,
        //     netuid_sub,
        //     stake_amount_sub,
        // ));

        // Test for main network
        let neuron_main = SubtensorModule::get_neuron(netuid_sub, uid_0)
            .expect("Neuron should exist for main network");
        assert_eq!(
            neuron_main.stake.len(),
            1,
            "Main network should have 1 stake entry"
        );

        // Test for subnetwork
        let neuron_sub = SubtensorModule::get_neuron(netuid_sub, uid_0)
            .expect("Neuron should exist for subnetwork");
        assert_eq!(
            neuron_sub.stake.len(),
            1,
            "Subnetwork should have 1 stake entry"
        );

        step_block(tempo);
        let total_stake = (stake_amount_sub + stake_amount_root) as f32;

        let (_, Compact(stake_weight)) = neuron_sub.stake[0];
        let expected_stake_weight = (stake_amount_sub as f32 / total_stake) as u64;
        assert_eq!(
            expected_stake_weight, stake_weight,
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
            register_ok_neuron(netuid, hotkey, coldkey, 0);
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

        // Cause epoch to run so that it sets StakeWeight
        step_block(tempo);

        // Retrieve all neurons using get_neurons_lite and check stakes
        let total_stake = (neuron_count as u64 * initial_stake + additional_stake) as f32;
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
            let (_, Compact(neuron_stake)) = neuron.stake.iter().find(|(id, _)| *id == neuron.hotkey).expect("Neuron stake not found");

            let expected_stake_weight = (if neuron.hotkey == U256::from(target_neuron_index) {
                (initial_stake + additional_stake) as f32 / total_stake
            } else {
                initial_stake as f32 / total_stake
            } * (u16::MAX as f32)) as u64;
            log::info!("Stake in all neurons: {:?}", neuron.stake);
            log::info!("Neurons: {:?}", neuron);
            log::info!("Neurons UID: {:?}", neuron.uid);
            log::info!("Checking stake for neuron with hotkey {:?}: Expected: {:?}, Got: {:?}", neuron.hotkey, expected_stake_weight, neuron_stake);
            assert_eq!(
                *neuron_stake, expected_stake_weight,
                "Stake does not match expected value for neuron with hotkey {:?}. Expected: {:?}, Got: {:?}",
                neuron.hotkey, expected_stake_weight, *neuron_stake
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
            register_ok_neuron(netuid, hotkey, coldkey, 0);
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

        // Cause epoch to run so that it sets StakeWeight
        step_block(tempo);

        // Retrieve and check all neurons to ensure only the targeted neuron's stake has increased
        let total_stake = (neuron_count as u64 * initial_stake + additional_stake) as f32;
        for i in 0..neuron_count {
            let neuron_index = i;
            if let Some(neuron_lite) = SubtensorModule::get_neuron_lite(netuid, neuron_index) {
                let neuron_hotkey = U256::from(i);
                let found_stake_tuple = neuron_lite
                    .stake
                    .iter()
                    .find(|(hotkey, _)| *hotkey == neuron_hotkey);
                if let Some((_, Compact(stake_weight))) = found_stake_tuple {
                    let expected_stake_weight = (if neuron_index == target_neuron_index {
                        (initial_stake + additional_stake) as f32 / total_stake
                    } else {
                        initial_stake as f32 / total_stake
                    } * (u16::MAX as f32)) as u64;
                    log::info!(
                        "Checking stake for neuron {}: Expected: {}, Got: {}",
                        i,
                        expected_stake_weight,
                        stake_weight
                    );
                    assert_eq!(
                        *stake_weight, expected_stake_weight,
                        "Stake does not match expected value for neuron {}. Expected: {}, Got: {}",
                        i, expected_stake_weight, *stake_weight
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
