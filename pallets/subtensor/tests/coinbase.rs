use crate::mock::*;
mod mock;
// use frame_support::{assert_err, assert_ok};
use sp_core::U256;

// Test the ability to hash all sorts of hotkeys.
#[test]
#[cfg(not(tarpaulin))]
fn test_hotkey_hashing() {
    new_test_ext(1).execute_with(|| {
        for i in 0..10000 {
            SubtensorModule::hash_hotkey_to_u64(&U256::from(i));
        }
    });
}

// Test drain tempo on hotkeys.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_hotkey_drain_time -- --nocapture
#[test]
#[cfg(not(tarpaulin))]
fn test_hotkey_drain_time() {
    new_test_ext(1).execute_with(|| {
        // Block 0
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(0), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(1), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(2), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(3), 0, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(4), 0, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(5), 0, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(6), 0, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(7), 0, 1));

        // Block 1
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(0), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(1), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(2), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(3), 1, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(4), 1, 1));
        assert!(!SubtensorModule::should_drain_hotkey(&U256::from(5), 1, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(6), 1, 1));
        assert!(SubtensorModule::should_drain_hotkey(&U256::from(7), 1, 1));
    });
}

// To run this test specifically, use the following command:
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_coinbase_basic -- --nocapture
#[test]
#[cfg(not(tarpaulin))]
fn test_coinbase_basic() {
    new_test_ext(1).execute_with(|| {
        // Define network ID
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let coldkey = U256::from(3);

        // Create a network with a tempo 1
        add_network(netuid, 1, 0);
        register_ok_neuron(netuid, hotkey, coldkey, 100000);
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1000);

        // Set the subnet emission value to 1.
        SubtensorModule::set_emission_values(&[netuid], vec![1]).unwrap();
        assert_eq!(SubtensorModule::get_subnet_emission_value(netuid), 1);

        // Hotkey has no pending emission
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has same stake
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 1000);

        // Subnet has no pending emission.
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // Step block
        next_block();

        // Hotkey has no pending emission
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has same stake
        assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hotkey), 1000);

        // Subnet has no pending emission of 1 ( from coinbase )
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 1);

        // Step block releases
        next_block();

        // Subnet pending has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // Hotkey pending immediately drained.
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has NEW stake
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            1000 + 2
        );

        // Set the hotkey drain time to 2 block.
        SubtensorModule::set_hotkey_emission_tempo(2);

        // Step block releases
        next_block();

        // Subnet pending increased by 1
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 1);

        // Hotkey pending not increased (still on subnet)
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has same stake
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            1000 + 2
        );

        // Step block releases
        next_block();

        // Subnet pending has been drained.
        assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);

        // Hotkey pending drained.
        assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

        // Hotkey has 2 new TAO.
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hotkey),
            1000 + 4
        );
    });
}

// Test getting and setting hotkey emission tempo
#[test]
#[cfg(not(tarpaulin))]
fn test_set_and_get_hotkey_emission_tempo() {
    new_test_ext(1).execute_with(|| {
        // Get the default hotkey emission tempo
        let default_tempo = SubtensorModule::get_hotkey_emission_tempo();
        assert_eq!(default_tempo, 0); // default is 0 in mock.rs

        // Set a new hotkey emission tempo
        let new_tempo = 5;
        SubtensorModule::set_hotkey_emission_tempo(new_tempo);

        // Get the updated hotkey emission tempo
        let updated_tempo = SubtensorModule::get_hotkey_emission_tempo();
        assert_eq!(updated_tempo, new_tempo);
    });
}

// #[test]
// #[cfg(not(tarpaulin))]
// fn test_comprehensive_coinbase() {
//     new_test_ext(1).execute_with(|| {
//         // Setup
//         let netuid_1: u16 = 1;
//         let netuid_2: u16 = 2;
//         let owner = U256::from(999);
//         let hotkey_1 = U256::from(1);
//         let hotkey_2 = U256::from(2);
//         let hotkey_3 = U256::from(3);
//         let coldkey_1 = U256::from(101);
//         let coldkey_2 = U256::from(102);
//         let coldkey_3 = U256::from(103);
//         let nominator_1 = U256::from(201);
//         let nominator_2 = U256::from(202);
//         let nominator_3 = U256::from(203);

//         // Create networks with different tempos
//         add_network(netuid_1, 2, 0); // tempo 2
//         add_network(netuid_2, 3, 0); // tempo 3
//                                      // SubtensorModule::set_subnet_owner(netuid_1, owner);
//         SubtensorModule::set_subnet_owner_cut(1000); // 10% owner cut

//         // Register neurons and set up stakes
//         register_ok_neuron(netuid_1, hotkey_1, coldkey_1, 100000);
//         register_ok_neuron(netuid_1, hotkey_2, coldkey_2, 100000);
//         register_ok_neuron(netuid_2, hotkey_3, coldkey_3, 100000);

//         SubtensorModule::create_account_if_non_existent(&coldkey_1, &hotkey_1);
//         SubtensorModule::create_account_if_non_existent(&coldkey_2, &hotkey_2);
//         SubtensorModule::create_account_if_non_existent(&coldkey_3, &hotkey_3);

//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey_1, &hotkey_1, 1000);
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey_2, &hotkey_2, 2000);
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey_3, &hotkey_3, 3000);

//         // Set up nominators
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(&nominator_1, &hotkey_1, 500);
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(&nominator_2, &hotkey_2, 1000);
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(&nominator_3, &hotkey_3, 1500);

//         // Set emission values
//         SubtensorModule::set_emission_values(&[netuid_1, netuid_2], vec![100, 200]).unwrap();

//         // Set different hotkey emission tempos
//         SubtensorModule::set_hotkey_emission_tempo(4);

//         // Initial assertions
//         assert_eq!(SubtensorModule::get_subnet_emission_value(netuid_1), 100);
//         assert_eq!(SubtensorModule::get_subnet_emission_value(netuid_2), 200);
//         assert_eq!(SubtensorModule::get_pending_emission(netuid_1), 0);
//         assert_eq!(SubtensorModule::get_pending_emission(netuid_2), 0);

//         // Run for 10 blocks
//         for block in 1..=10 {
//             next_block();

//             // Check subnet emissions
//             let pending_1 = SubtensorModule::get_pending_emission(netuid_1);
//             let pending_2 = SubtensorModule::get_pending_emission(netuid_2);

//             if block % 2 == 0 {
//                 assert_eq!(pending_1, 0, "Subnet 1 should drain at block {}", block);
//             } else {
//                 assert_eq!(
//                     pending_1, 100,
//                     "Subnet 1 should accumulate at block {}",
//                     block
//                 );
//             }

//             if block % 3 == 0 {
//                 assert_eq!(pending_2, 0, "Subnet 2 should drain at block {}", block);
//             } else {
//                 assert!(
//                     pending_2 > 0 && pending_2 <= 400,
//                     "Subnet 2 should accumulate at block {}",
//                     block
//                 );
//             }

//             // Check hotkey emissions
//             for hotkey in [&hotkey_1, &hotkey_2, &hotkey_3] {
//                 let pending = SubtensorModule::get_pending_hotkey_emission(hotkey);
//                 if block % 4 == 0 {
//                     assert_eq!(
//                         pending, 0,
//                         "Hotkey {:?} should drain at block {}",
//                         hotkey, block
//                     );
//                 } else {
//                     assert!(
//                         pending >= 0,
//                         "Hotkey {:?} should have non-negative pending emission at block {}",
//                         hotkey,
//                         block
//                     );
//                 }
//             }

//             // Check stakes after each block
//             let stake_1 = SubtensorModule::get_total_stake_for_hotkey(&hotkey_1);
//             let stake_2 = SubtensorModule::get_total_stake_for_hotkey(&hotkey_2);
//             let stake_3 = SubtensorModule::get_total_stake_for_hotkey(&hotkey_3);

//             assert!(stake_1 >= 1500, "Hotkey 1 stake should not decrease");
//             assert!(stake_2 >= 3000, "Hotkey 2 stake should not decrease");
//             assert!(stake_3 >= 4500, "Hotkey 3 stake should not decrease");

//             // Check nominator stakes
//             let nom_stake_1 =
//                 SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator_1, &hotkey_1);
//             let nom_stake_2 =
//                 SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator_2, &hotkey_2);
//             let nom_stake_3 =
//                 SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator_3, &hotkey_3);

//             assert!(nom_stake_1 >= 500, "Nominator 1 stake should not decrease");
//             assert!(nom_stake_2 >= 1000, "Nominator 2 stake should not decrease");
//             assert!(nom_stake_3 >= 1500, "Nominator 3 stake should not decrease");
//         }

//         // Final assertions
//         let final_stake_1 = SubtensorModule::get_total_stake_for_hotkey(&hotkey_1);
//         let final_stake_2 = SubtensorModule::get_total_stake_for_hotkey(&hotkey_2);
//         let final_stake_3 = SubtensorModule::get_total_stake_for_hotkey(&hotkey_3);

//         assert!(final_stake_1 > 1500, "Hotkey 1 should have gained stake");
//         assert!(final_stake_2 > 3000, "Hotkey 2 should have gained stake");
//         assert!(final_stake_3 > 4500, "Hotkey 3 should have gained stake");

//         let final_nom_stake_1 =
//             SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator_1, &hotkey_1);
//         let final_nom_stake_2 =
//             SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator_2, &hotkey_2);
//         let final_nom_stake_3 =
//             SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator_3, &hotkey_3);

//         assert!(
//             final_nom_stake_1 > 500,
//             "Nominator 1 should have gained stake"
//         );
//         assert!(
//             final_nom_stake_2 > 1000,
//             "Nominator 2 should have gained stake"
//         );
//         assert!(
//             final_nom_stake_3 > 1500,
//             "Nominator 3 should have gained stake"
//         );

//         // let final_owner_balance = SubtensorModule::get_stake_for_coldkey_and_hotkey(&owner);
//         // assert!(final_owner_balance > 0, "Owner should have received emissions");

//         // // Error assertions
//         // assert_err!(
//         //     SubtensorModule::set_emission_values(&[999], vec![100]),
//         //     Error::<Test>::InvalidNetworkId
//         // );

//         // assert_err!(
//         //     SubtensorModule::set_hotkey_emission_tempo(0),
//         //     Error::<Test>::InvalidHotkeyEmissionTempo
//         // );

//         // // Test parent-child relationship
//         // SubtensorModule::set_delegate(&hotkey_2, &hotkey_1, 5000); // 50% delegation
//         // next_block();
//         // next_block();

//         let parent_emission = SubtensorModule::get_pending_hotkey_emission(&hotkey_1);
//         let child_emission = SubtensorModule::get_pending_hotkey_emission(&hotkey_2);
//         assert!(
//             parent_emission > 0,
//             "Parent should receive emissions from child"
//         );
//         assert!(
//             child_emission > 0,
//             "Child should still receive some emissions"
//         );

//         // Test with zero emission
//         SubtensorModule::set_emission_values(&[netuid_1], vec![0]).unwrap();
//         for _ in 0..5 {
//             next_block();
//         }
//         assert_eq!(
//             SubtensorModule::get_pending_emission(netuid_1),
//             0,
//             "No emission should accumulate with zero emission value"
//         );
//         // Test with very large emission
//         SubtensorModule::set_emission_values(&[netuid_2], vec![u64::MAX]).unwrap();
//         next_block();
//         let large_pending = SubtensorModule::get_pending_emission(netuid_2);
//         assert!(
//             large_pending > 0,
//             "Large emission should result in non-zero pending emission"
//         );
//         assert!(
//             large_pending <= u64::MAX,
//             "Pending emission should not overflow"
//         );
//     });
// }

// #[test]
// fn test_comprehensive_coinbase() {
//     new_test_ext().execute_with(|| {
//         // Setup
//         let netuid = 1;
//         let hotkey = AccountId::from([1u8; 32]);
//         let coldkey = AccountId::from([2u8; 32]);
//         let nominator = AccountId::from([3u8; 32]);
//         let child = AccountId::from([4u8; 32]);
//         let parent = AccountId::from([5u8; 32]);

//         // Create network
//         assert_ok!(SubtensorModule::add_network(Origin::root(), netuid, 10, 0));

//         // Register neurons
//         assert_ok!(SubtensorModule::register(Origin::signed(hotkey.clone()), netuid, 100000));
//         assert_ok!(SubtensorModule::register(Origin::signed(child.clone()), netuid, 100000));
//         assert_ok!(SubtensorModule::register(Origin::signed(parent.clone()), netuid, 100000));

//         // Set up stakes
//         assert_ok!(SubtensorModule::add_stake(Origin::signed(coldkey.clone()), hotkey.clone(), 1000));
//         assert_ok!(SubtensorModule::add_stake(Origin::signed(nominator.clone()), hotkey.clone(), 500));
//         assert_ok!(SubtensorModule::add_stake(Origin::signed(child.clone()), child.clone(), 500));
//         assert_ok!(SubtensorModule::add_stake(Origin::signed(parent.clone()), parent.clone(), 2000));

//         // Set up child and parent relationships
//         assert_ok!(SubtensorModule::set_child(Origin::signed(hotkey.clone()), child.clone(), netuid, 5000));
//         assert_ok!(SubtensorModule::set_parent(Origin::signed(hotkey.clone()), parent.clone(), netuid, 5000));

//         // Set emission value
//         assert_ok!(SubtensorModule::set_emission_values(Origin::root(), vec![(netuid, 1000)]));

//         // Set hotkey emission tempo
//         SubtensorModule::set_hotkey_emission_tempo(5);

//         // Initial assertions
//         assert_eq!(SubtensorModule::get_emission_value(netuid), 1000);
//         assert_eq!(SubtensorModule::get_pending_emission(netuid), 0);
//         assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0);

//         // Run coinbase and advance blocks
//         for i in 1..=20 {
//             SubtensorModule::run_coinbase();

//             // Check subnet emission accumulation
//             let pending_emission = SubtensorModule::get_pending_emission(netuid);
//             if i % 10 == 0 {
//                 assert_eq!(pending_emission, 0, "Subnet emission should be drained at block {}", i);
//             } else {
//                 assert_eq!(pending_emission, 1000 * (i % 10), "Incorrect pending emission at block {}", i);
//             }

//             // Check if epoch should run
//             assert_eq!(SubtensorModule::should_run_epoch(netuid, i as u64), i % 10 == 0);

//             // Check hotkey emission accumulation and draining
//             let hotkey_emission = SubtensorModule::get_pending_hotkey_emission(&hotkey);
//             if i % 5 == 0 {
//                 assert_eq!(hotkey_emission, 0, "Hotkey emission should be drained at block {}", i);
//             } else {
//                 assert!(hotkey_emission > 0, "Hotkey should have pending emission at block {}", i);
//             }

//             // Check if hotkey should be drained
//             assert_eq!(SubtensorModule::should_drain_hotkey(&hotkey, i as u64, 5), i % 5 == 0);

//             run_to_block(i + 1);
//         }

//         // Final stake checks
//         let hotkey_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey);
//         let child_stake = SubtensorModule::get_total_stake_for_hotkey(&child);
//         let parent_stake = SubtensorModule::get_total_stake_for_hotkey(&parent);
//         let nominator_stake = SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator, &hotkey);

//         assert!(hotkey_stake > 1000, "Hotkey stake should have increased");
//         assert!(child_stake > 500, "Child stake should have increased");
//         assert!(parent_stake > 2000, "Parent stake should have increased");
//         assert!(nominator_stake > 500, "Nominator stake should have increased");

//         // Check stake distribution
//         assert!(hotkey_stake > child_stake, "Hotkey should have more stake than child");
//         assert!(hotkey_stake > parent_stake - 2000, "Hotkey should have gained more stake than parent");

//         // Check get_stake_with_children_and_parents
//         let total_stake = SubtensorModule::get_stake_with_children_and_parents(&hotkey, netuid);
//         assert!(total_stake > hotkey_stake, "Total stake should be higher than hotkey's own stake");

//         // Check root_epoch
//         assert_ok!(SubtensorModule::root_epoch(21));
//         assert_eq!(SubtensorModule::get_emission_value(netuid), 1000);

//         // Run epoch manually and check results
//         let hotkey_emissions = SubtensorModule::epoch(netuid, 1000);
//         assert!(hotkey_emissions.iter().any(|(h, _, _)| h == &hotkey), "Hotkey should receive emission");

//         // Final coinbase run
//         SubtensorModule::run_coinbase();
//         assert!(SubtensorModule::get_pending_hotkey_emission(&hotkey) > 0, "Hotkey should have pending emission after final coinbase");
//         // Drain hotkey emission manually
//         let initial_total_issuance = SubtensorModule::get_total_issuance();
//         let drained_emission = SubtensorModule::get_pending_hotkey_emission(&hotkey);
//         let total_new_tao = SubtensorModule::drain_hotkey_emission(&hotkey, drained_emission, 21);

//         assert_eq!(SubtensorModule::get_pending_hotkey_emission(&hotkey), 0, "Pending emission should be zero after draining");
//         assert_eq!(total_new_tao, drained_emission, "Total new TAO should match drained emission");
//         assert_eq!(SubtensorModule::get_total_issuance(), initial_total_issuance + total_new_tao, "Total issuance should increase by the drained amount");

//         // Check final stakes after manual drain
//         let final_hotkey_stake = SubtensorModule::get_total_stake_for_hotkey(&hotkey);
//         let final_nominator_stake = SubtensorModule::get_stake_for_coldkey_and_hotkey(&nominator, &hotkey);

//         assert!(final_hotkey_stake > hotkey_stake, "Hotkey stake should increase after manual drain");
//         assert!(final_nominator_stake > nominator_stake, "Nominator stake should increase after manual drain");

//         // Test with zero emission
//         assert_ok!(SubtensorModule::set_emission_values(Origin::root(), vec![(netuid, 0)]));
//         SubtensorModule::run_coinbase();
//         assert_eq!(SubtensorModule::get_pending_emission(netuid), 0, "No emission should accumulate with zero emission value");

//         // Test with maximum emission
//         assert_ok!(SubtensorModule::set_emission_values(Origin::root(), vec![(netuid, u64::MAX)]));
//         SubtensorModule::run_coinbase();
//         let max_pending = SubtensorModule::get_pending_emission(netuid);
//         assert!(max_pending > 0, "Large emission should result in non-zero pending emission");
//         assert!(max_pending <= u64::MAX, "Pending emission should not overflow");

//         // Error cases
//         assert_err!(
//             SubtensorModule::set_emission_values(Origin::root(), vec![(999, 100)]),
//             Error::<Test>::NetworkDoesNotExist
//         );

//         assert_err!(
//             SubtensorModule::set_hotkey_emission_tempo(0),
//             Error::<Test>::InvalidHotkeyEmissionTempo
//         );
//     });
// }
