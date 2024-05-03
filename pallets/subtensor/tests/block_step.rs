mod mock;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use sp_core::U256;
use substrate_fixed::types::I64F64;

#[macro_use]
mod helpers;

// TODO: Apparently, run_coinbase doesn't change LoadedEmission, do we need this test?
// #[test]
// fn test_loaded_emission() {
//     new_test_ext(1).execute_with(|| {
//         let n: u16 = 100;
//         let netuid: u16 = 1;
//         let tempo: u16 = 10;
//         let netuids: Vec<u16> = vec![1];
//         let emission: Vec<u64> = vec![1000000000];
//         add_network(netuid, tempo, 0);
//         SubtensorModule::set_max_allowed_uids(netuid, n);
//         SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
//         assert_ok!(SubtensorModule::set_emission_values(&netuids, emission));
//         for i in 0..n {
//             SubtensorModule::append_neuron(netuid, &U256::from(i), 0);
//         }
//         assert!(!SubtensorModule::has_loaded_emission_tuples(netuid));

//         // Try loading at block 0
//         let block: u64 = 0;
//         assert_eq!(
//             SubtensorModule::blocks_until_next_epoch(netuid, tempo, block),
//             8
//         );
//         SubtensorModule::run_coinbase(block);
//         assert!(!SubtensorModule::has_loaded_emission_tuples(netuid));

//         // Try loading at block = 9;
//         let block: u64 = 8;
//         assert_eq!(
//             SubtensorModule::blocks_until_next_epoch(netuid, tempo, block),
//             0
//         );
//         SubtensorModule::run_coinbase(block);
//         assert!(SubtensorModule::has_loaded_emission_tuples(netuid));
//         assert_eq!(
//             SubtensorModule::get_loaded_emission_tuples(netuid).len(),
//             n as usize
//         );

//         // Try draining the emission tuples
//         // None remaining because we are at epoch.
//         let block: u64 = 8;
//         SubtensorModule::drain_emission(block);
//         assert!(!SubtensorModule::has_loaded_emission_tuples(netuid));

//         // Generate more emission.
//         SubtensorModule::run_coinbase(8);
//         assert_eq!(
//             SubtensorModule::get_loaded_emission_tuples(netuid).len(),
//             n as usize
//         );

//         for block in 9..19 {
//             let mut n_remaining: usize = 0;
//             let mut n_to_drain: usize = 0;
//             if SubtensorModule::has_loaded_emission_tuples(netuid) {
//                 n_remaining = SubtensorModule::get_loaded_emission_tuples(netuid).len();
//                 n_to_drain = SubtensorModule::tuples_to_drain_this_block(
//                     netuid,
//                     tempo,
//                     block,
//                     SubtensorModule::get_loaded_emission_tuples(netuid).len(),
//                 );
//             }
//             SubtensorModule::drain_emission(block); // drain it with 9 more blocks to go
//             if SubtensorModule::has_loaded_emission_tuples(netuid) {
//                 assert_eq!(
//                     SubtensorModule::get_loaded_emission_tuples(netuid).len(),
//                     n_remaining - n_to_drain
//                 );
//             }
//             log::info!("n_to_drain:{:?}", n_to_drain.clone());
//             log::info!(
//                 "SubtensorModule::get_loaded_emission_tuples( netuid ).len():{:?}",
//                 n_remaining - n_to_drain
//             );
//         }
//     })
// }

// TODO: Should draining of emission tuples be tested?
// #[test]
// fn test_tuples_to_drain_this_block() {
//     new_test_ext(1).execute_with(|| {
//         // pub fn tuples_to_drain_this_block( netuid: u16, tempo: u16, block_number: u64, n_remaining: usize ) -> usize {
//         assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 1, 0, 10), 10); // drain all epoch block.
//         assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 0, 0, 10), 10); // drain all no tempo.
//         assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 10, 0, 10), 2); // drain 10 / ( 10 / 2 ) = 2
//         assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 20, 0, 10), 1); // drain 10 / ( 20 / 2 ) = 1
//         assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 10, 0, 20), 5); // drain 20 / ( 9 / 2 ) = 5
//         assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 20, 0, 0), 0); // nothing to drain.
//         assert_eq!(SubtensorModule::tuples_to_drain_this_block(0, 10, 1, 20), 5); // drain 19 / ( 10 / 2 ) = 4
//         assert_eq!(
//             SubtensorModule::tuples_to_drain_this_block(0, 10, 10, 20),
//             4
//         ); // drain 19 / ( 10 / 2 ) = 4
//         assert_eq!(
//             SubtensorModule::tuples_to_drain_this_block(0, 10, 15, 20),
//             10
//         ); // drain 19 / ( 10 / 2 ) = 4
//         assert_eq!(
//             SubtensorModule::tuples_to_drain_this_block(0, 10, 19, 20),
//             20
//         ); // drain 19 / ( 10 / 2 ) = 4
//         assert_eq!(
//             SubtensorModule::tuples_to_drain_this_block(0, 10, 20, 20),
//             20
//         ); // drain 19 / ( 10 / 2 ) = 4
//         for i in 0..10 {
//             for j in 0..10 {
//                 for k in 0..10 {
//                     for l in 0..10 {
//                         assert!(SubtensorModule::tuples_to_drain_this_block(i, j, k, l) <= 10);
//                     }
//                 }
//             }
//         }
//     })
// }

#[test]
fn test_blocks_until_epoch() {
    new_test_ext(1).execute_with(|| {
        // Check tempo = 0 block = * netuid = *
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 0, 0), 1000);

        // Check tempo = 1 block = * netuid = *
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, 0), 0);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, 0), 1);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, 1), 1);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, 1), 0);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, 2), 0);
        assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, 2), 1);
        for i in 0..100 {
            if i % 2 == 0 {
                assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, i), 0);
                assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, i), 1);
            } else {
                assert_eq!(SubtensorModule::blocks_until_next_epoch(0, 1, i), 1);
                assert_eq!(SubtensorModule::blocks_until_next_epoch(1, 1, i), 0);
            }
        }

        // Check general case.
        for netuid in 0..30 as u16 {
            for block in 0..30 as u64 {
                for tempo in 1..30 as u16 {
                    assert_eq!(
                        SubtensorModule::blocks_until_next_epoch(netuid, tempo, block),
                        tempo as u64 - (block + netuid as u64 + 1) % (tempo as u64 + 1)
                    );
                }
            }
        }
    });
}

// /********************************************
//     block_step::adjust_registration_terms_for_networks tests
// *********************************************/
#[test]
fn test_burn_adjustment() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );
        assert_eq!(
            SubtensorModule::get_adjustment_interval(netuid),
            adjustment_interval
        ); // Sanity check the adjustment interval.

        // Register key 1.
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2.
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        // We are over the number of regs allowed this interval.
        // Step the block and trigger the adjustment.
        step_block(1);

        // Check the adjusted burn.
        assert_eq!(SubtensorModule::get_burn_as_u64(netuid), 1500);
    });
}

#[test]
fn test_burn_adjustment_with_moving_average() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 1;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );
        // Set alpha here.
        SubtensorModule::set_adjustment_alpha(netuid, u64::MAX / 2);

        // Register key 1.
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2.
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        // We are over the number of regs allowed this interval.
        // Step the block and trigger the adjustment.
        step_block(1);

        // Check the adjusted burn.
        // 0.5 * 1000 + 0.5 * 1500 = 1250
        assert_eq!(SubtensorModule::get_burn_as_u64(netuid), 1250);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_a() {
    // Test case A of the difficulty and burn adjustment algorithm.
    // ====================
    // There are too many registrations this interval and most of them are pow registrations
    // this triggers an increase in the pow difficulty.
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 1;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, start_diff);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a burn registration.
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2. This is a POW registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            0,
            &hotkey_account_id_2,
        );
        let result0 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            curr_block_num,
            nonce0,
            work0,
            hotkey_account_id_2,
            coldkey_account_id_2,
        );
        assert_ok!(result0);

        // Register key 3. This is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_3,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result1);

        // We are over the number of regs allowed this interval.
        // Most of them are POW registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted POW difficulty has INCREASED.
        //   and the burn has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert_eq!(adjusted_burn, burn_cost);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff > start_diff);
        assert_eq!(adjusted_diff, 20_000);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_b() {
    // Test case B of the difficulty and burn adjustment algorithm.
    // ====================
    // There are too many registrations this interval and most of them are burn registrations
    // this triggers an increase in the burn cost.
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 1;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1.
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2.
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        // Register key 3. This one is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            0,
            &hotkey_account_id_3,
        );
        let result = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce,
            work,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result);

        // We are over the number of regs allowed this interval.
        // Most of them are burn registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted burn has INCREASED.
        //   and the difficulty has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn > burn_cost);
        assert_eq!(adjusted_burn, 2_000);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert_eq!(adjusted_diff, start_diff);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_c() {
    // Test case C of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and most of them are POW registrations
    // this triggers a decrease in the burn cost
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 4; // Needs registrations < 4 to trigger
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a BURN registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2. This is a POW registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            0,
            &hotkey_account_id_2,
        );
        let result0 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            curr_block_num,
            nonce0,
            work0,
            hotkey_account_id_2,
            coldkey_account_id_2,
        );
        assert_ok!(result0);

        // Register key 3. This is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_3,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result1);

        // We are UNDER the number of regs allowed this interval.
        // Most of them are POW registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted burn has DECREASED.
        //   and the difficulty has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn < burn_cost);
        assert_eq!(adjusted_burn, 875);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert_eq!(adjusted_diff, start_diff);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_d() {
    // Test case D of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and most of them are BURN registrations
    // this triggers a decrease in the POW difficulty
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval = 4; // Needs registrations < 4 to trigger
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, 1);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a BURN registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_1, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            hotkey_account_id_1
        ));

        // Register key 2. This is a BURN registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        // Register key 3. This is a POW registration
        let hotkey_account_id_3 = U256::from(3);
        let coldkey_account_id_3 = U256::from(3);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_3,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_3,
            coldkey_account_id_3,
        );
        assert_ok!(result1);

        // We are UNDER the number of regs allowed this interval.
        // Most of them are BURN registrations (2 out of 3)
        // Step the block and trigger the adjustment.
        step_block(1);
        curr_block_num += 1;

        // Check the adjusted POW difficulty has DECREASED.
        //   and the burn has not changed.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert_eq!(adjusted_burn, burn_cost);

        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff < start_diff);
        assert_eq!(adjusted_diff, 8750);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_e() {
    // Test case E of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and nobody registered either POW or BURN
    // this triggers a decrease in the BURN cost and POW difficulty
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval: u16 = 3;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, 1);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a POW registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_1,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_1,
            coldkey_account_id_1,
        );
        assert_ok!(result1);

        // Register key 2. This is a BURN registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        step_block(1);
        curr_block_num += 1;

        // We are UNDER the number of regs allowed this interval.
        // And the number of regs of each type is equal

        // Check the adjusted BURN has DECREASED.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn < burn_cost);
        assert_eq!(adjusted_burn, 833);

        // Check the adjusted POW difficulty has DECREASED.
        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff < start_diff);
        assert_eq!(adjusted_diff, 8_333);
    });
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_f() {
    // Test case F of the difficulty and burn adjustment algorithm.
    // ====================
    // There are too many registrations this interval and the pow and burn registrations are equal
    // this triggers an increase in the burn cost and pow difficulty
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval: u16 = 1;
        let start_diff: u64 = 10_000;
        let mut curr_block_num = 0;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, start_diff);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // Register key 1. This is a POW registration
        let hotkey_account_id_1 = U256::from(1);
        let coldkey_account_id_1 = U256::from(1);
        let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number(
            netuid,
            curr_block_num,
            11231312312,
            &hotkey_account_id_1,
        );
        let result1 = SubtensorModule::register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
            netuid,
            curr_block_num,
            nonce1,
            work1,
            hotkey_account_id_1,
            coldkey_account_id_1,
        );
        assert_ok!(result1);

        // Register key 2. This is a BURN registration
        let hotkey_account_id_2 = U256::from(2);
        let coldkey_account_id_2 = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id_2, 10000);
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
            netuid,
            hotkey_account_id_2
        ));

        step_block(1);
        curr_block_num += 1;
        // We are OVER the number of regs allowed this interval.
        // And the number of regs of each type is equal

        // Check the adjusted BURN has INCREASED.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn > burn_cost);
        assert_eq!(adjusted_burn, 1_500);

        // Check the adjusted POW difficulty has INCREASED.
        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff > start_diff);
        assert_eq!(adjusted_diff, 15_000);
    });
}

#[test]
fn test_burn_adjustment_case_e_zero_registrations() {
    // Test case E of the difficulty and burn adjustment algorithm.
    // ====================
    // There are not enough registrations this interval and nobody registered either POW or BURN
    // this triggers a decrease in the BURN cost and POW difficulty

    // BUT there are zero registrations this interval.
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let burn_cost: u64 = 1000;
        let adjustment_interval = 1;
        let target_registrations_per_interval: u16 = 1;
        let start_diff: u64 = 10_000;
        add_network(netuid, tempo, 0);
        SubtensorModule::set_max_registrations_per_block(netuid, 10);
        SubtensorModule::set_burn(netuid, burn_cost);
        SubtensorModule::set_difficulty(netuid, start_diff);
        SubtensorModule::set_min_difficulty(netuid, 1);
        SubtensorModule::set_adjustment_interval(netuid, adjustment_interval);
        SubtensorModule::set_adjustment_alpha(netuid, 58000); // Set to old value.
        SubtensorModule::set_target_registrations_per_interval(
            netuid,
            target_registrations_per_interval,
        );

        // No registrations this interval of any kind.
        step_block(1);

        // We are UNDER the number of regs allowed this interval.
        // And the number of regs of each type is equal

        // Check the adjusted BURN has DECREASED.
        let adjusted_burn = SubtensorModule::get_burn_as_u64(netuid);
        assert!(adjusted_burn < burn_cost);
        assert_eq!(adjusted_burn, 500);

        // Check the adjusted POW difficulty has DECREASED.
        let adjusted_diff = SubtensorModule::get_difficulty_as_u64(netuid);
        assert!(adjusted_diff < start_diff);
        assert_eq!(adjusted_diff, 5_000);
    });
}

// To run this test with logging and Rust backtrace enabled, and to see all output (stdout/stderr) without capturing by the test runner, use:
// RUST_BACKTRACE=1 cargo test --package pallet-subtensor --test block_step test_subnet_staking_emission -- --nocapture
#[test]
fn test_subnet_staking_emission() {
    new_test_ext(1).execute_with(|| {
        let delegate = U256::from(1);
        let nominator1 = U256::from(2);
        let nominator2 = U256::from(3);
        SubtensorModule::set_target_stakes_per_interval(20);
        add_network(1, 1, 0);
        add_network(2, 1, 0);
        add_network(3, 1, 0);
        assert_eq!(SubtensorModule::get_num_subnets(), 3);
        SubtensorModule::add_balance_to_coldkey_account(&delegate, 100000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator1, 100000);
        SubtensorModule::add_balance_to_coldkey_account(&nominator2, 100000);
        register_ok_neuron(1, delegate, delegate, 124124);
        register_ok_neuron(2, delegate, delegate, 124124);
        register_ok_neuron(3, delegate, delegate, 124124);
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate),
            delegate,
            1,
            10000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate),
            delegate,
            2,
            1000
        ));
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(delegate),
            delegate,
            3,
            100
        ));
        SubtensorModule::get_subnet_staking_emission_values(0).unwrap();
        assert_eq!(SubtensorModule::get_subnet_emission_value(1), 900_900_900); // (10000 / (100 + 1000 + 10000)) * 1000000000 ~= 900900900
        assert_eq!(SubtensorModule::get_subnet_emission_value(2), 90_090_090); // (1000 / (100 + 1000 + 10000)) * 1000000000 ~= 90,090,090
        assert_eq!(SubtensorModule::get_subnet_emission_value(3), 9_009_009); // (100 / (100 + 1000 + 10000)) * 1000000000 ~= 9,009,009
        assert_eq!(900_900_900 + 90_090_090 + 9_009_009, 999_999_999);
    });
}



#[test]
fn test_run_coinbase_price_greater_than_1() {
    new_test_ext(1).execute_with(|| {
        // Create subnet with price 4
        let netuid: u16 = 1;
        setup_dynamic_network(netuid, 1u16, 1u16);
        add_dynamic_stake(netuid, 1u16, 1u16, 100_000_000_000u64);
        assert_eq!(SubtensorModule::get_tao_per_alpha_price(netuid), 4.0);

        // Make some TAO
        SubtensorModule::coinbase(100);
        let total_issuance = SubtensorModule::get_total_issuance();
        let block_emission = SubtensorModule::get_block_emission().unwrap();
        assert_eq!(total_issuance, 100);
        assert_eq!(block_emission > 0, true);

        // Check that running run_coinbase behaves correctly
        let tao_reserve_before = SubtensorModule::get_tao_reserve(netuid);
        log::info!("Tao reserve before: {:?}", tao_reserve_before);
        let alpha_reserve_before = SubtensorModule::get_alpha_reserve(netuid);
        log::info!("Alpha reserve before: {:?}", alpha_reserve_before);
        let pending_alpha_before = SubtensorModule::get_alpha_pending_emission(netuid);
        log::info!("Pending alpha before: {:?}", pending_alpha_before);
        SubtensorModule::run_coinbase(1);
        let tao_reserve_after = SubtensorModule::get_tao_reserve(netuid);
        log::info!("Tao reserve after: {:?}", tao_reserve_after);
        let alpha_reserve_after = SubtensorModule::get_alpha_reserve(netuid);
        log::info!("Alpha reserve after: {:?}", alpha_reserve_after);
        let pending_alpha_after = SubtensorModule::get_alpha_pending_emission(netuid);
        log::info!("Pending alpha after: {:?}", pending_alpha_after);
        log::info!("Tao emissions: {:?}", SubtensorModule::get_subnet_emission_value(netuid));


        assert_eq!(tao_reserve_after == tao_reserve_before, true);
        assert_eq!(alpha_reserve_after > alpha_reserve_before, true);
        assert_eq!(pending_alpha_after > pending_alpha_before, true);
    })
}

#[test]
fn test_run_coinbase_price_less_than_1() {
    new_test_ext(1).execute_with(|| {
        // Create subnet with price 0.64 by unstaking 25 TAO
        let netuid: u16 = 1;
        setup_dynamic_network(netuid, 1u16, 1u16);
        remove_dynamic_stake(netuid, 1u16, 1u16, 25_000_000_000u64);
        assert_i64f64_approx_eq!(SubtensorModule::get_tao_per_alpha_price(netuid), 0.64);

        // Make some TAO
        SubtensorModule::coinbase(100);
        let total_issuance = SubtensorModule::get_total_issuance();
        let block_emission = SubtensorModule::get_block_emission().unwrap();
        assert_eq!(total_issuance, 100);
        assert_eq!(block_emission > 0, true);

        // Check that running run_coinbase behaves correctly
        let tao_reserve_before = SubtensorModule::get_tao_reserve(netuid);
        let alpha_reserve_before = SubtensorModule::get_alpha_reserve(netuid);
        let pending_alpha_before = SubtensorModule::get_alpha_pending_emission(netuid);
        SubtensorModule::run_coinbase(1);
        let tao_reserve_after = SubtensorModule::get_tao_reserve(netuid);
        let alpha_reserve_after = SubtensorModule::get_alpha_reserve(netuid);
        let pending_alpha_after = SubtensorModule::get_alpha_pending_emission(netuid);
        log::info!("Subnet emissions: {:?}", SubtensorModule::get_subnet_emission_value(netuid));
        log::info!("Subnet emissions from Subnet Info: {:?}", SubtensorModule::get_subnet_info(netuid).unwrap().emission_values);

        assert_eq!(tao_reserve_after > tao_reserve_before, true);
        assert_eq!(alpha_reserve_after, alpha_reserve_before);
        assert_eq!(pending_alpha_after > pending_alpha_before, true);
    })
}

#[test]
fn test_10_subnet_take_basic_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = 1;
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);

        // Create networks.
        let lock_cost_1 = SubtensorModule::get_network_lock_cost();
        setup_dynamic_network(netuid1, 3u16, 1u16);
        SubtensorModule::add_balance_to_coldkey_account( &coldkey0, 1000_000_000_000 );
        SubtensorModule::add_balance_to_coldkey_account( &coldkey1, 1000_000_000_000 );
        SubtensorModule::add_balance_to_coldkey_account( &hotkey0, 1000_000_000_000 );

        // The tests below assume lock costs of LC1 = 100
        assert_eq!(lock_cost_1, 100_000_000_000);

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: LC1     (100)
        //
        // DynamicTAOReserve (get_tao_reserve) assertions
        //   Subnet 1: 100
        //
        // DynamicAlphaReserve (get_alpha_reserve) assertions
        //   Subnet 1: 100
        //
        // DynamicAlphaOutstanding (get_alpha_outstading) assertions
        //   Subnet 1: 100
        //
        assert_substake_eq!(&coldkey0, &hotkey0, netuid1, 100_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid1), 100_000_000_000);

        // Coldkey / hotkey 0 become a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));

        // Coldkey / hotkey 0 sets the take on subnet 1 to 10%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid1,
            u16::MAX / 10
        ));

        // Nominate 100 from coldkey/hotkey 1 to hotkey0 on subnet 1
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid1,
            100_000_000_000
        ));

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: 100
        //             cold1, hot0: 50
        //
        // DynamicTAOReserve (get_tao_reserve) assertions
        //   Subnet 1: 200
        //
        // DynamicAlphaReserve (get_alpha_reserve) assertions
        //   Subnet 1: 50
        //
        // DynamicAlphaOutstanding (get_alpha_outstading) assertions
        //   Subnet 1: 150
        //
        assert_substake_eq!(&coldkey0, &hotkey0, netuid1, 100_000_000_000);
        assert_substake_eq!(&coldkey1, &hotkey0, netuid1, 50_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid1), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid1), 50_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid1), 150_000_000_000);

        // Emission
        //
        // Emit inflation through run_coinbase
        // We will emit 0 server emission (which should go in-full to the owner of the hotkey).
        // We will emit 200 validator emission, which should be distributed in-part to the nominators.
        //
        let emission = 200_000_000_000;
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid1, 0, emission);

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: 350 - 110 = 240
        //             cold1, hot0: 110
        //
        assert_substake_approx_eq!(&coldkey0, &hotkey0, netuid1, 240.);
        assert_substake_approx_eq!(&coldkey1, &hotkey0, netuid1, 110.);
    });
}

#[test]
fn test_20_subnet_take_basic_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = 1;
        let hotkey0 = U256::from(1);
        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);

        // Create networks.
        let lock_cost_1 = SubtensorModule::get_network_lock_cost();
        setup_dynamic_network(netuid1, 3u16, 1u16);
        SubtensorModule::add_balance_to_coldkey_account( &coldkey0, 1000_000_000_000 );
        SubtensorModule::add_balance_to_coldkey_account( &coldkey1, 1000_000_000_000 );
        SubtensorModule::add_balance_to_coldkey_account( &hotkey0, 1000_000_000_000 );

        // The tests below assume lock costs of LC1 = 100
        assert_eq!(lock_cost_1, 100_000_000_000);

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: LC1     (100)
        //
        // DynamicTAOReserve (get_tao_reserve) assertions
        //   Subnet 1: 100
        //
        // DynamicAlphaReserve (get_alpha_reserve) assertions
        //   Subnet 1: 100
        //
        // DynamicAlphaOutstanding (get_alpha_outstading) assertions
        //   Subnet 1: 100
        //
        assert_substake_eq!(&coldkey0, &hotkey0, netuid1, 100_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid1), 100_000_000_000);

        // Coldkey / hotkey 0 become a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));

        // Coldkey / hotkey 0 sets the take on subnet 1 to 20%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid1,
            u16::MAX / 5
        ));

        // Nominate 100 from coldkey/hotkey 1 to hotkey0 on subnet 1
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid1,
            100_000_000_000
        ));

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: 100
        //             cold1, hot0: 50
        //
        // DynamicTAOReserve (get_tao_reserve) assertions
        //   Subnet 1: 200
        //
        // DynamicAlphaReserve (get_alpha_reserve) assertions
        //   Subnet 1: 50
        //
        // DynamicAlphaOutstanding (get_alpha_outstading) assertions
        //   Subnet 1: 150
        //
        assert_substake_eq!(&coldkey0, &hotkey0, netuid1, 100_000_000_000);
        assert_substake_eq!(&coldkey1, &hotkey0, netuid1, 50_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid1), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid1), 50_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid1), 150_000_000_000);

        // Emission
        //
        // Emit inflation through run_coinbase
        // We will emit 0 server emission (which should go in-full to the owner of the hotkey).
        // We will emit 200 validator emission, which should be distributed in-part to the nominators.
        //
        let emission = 200_000_000_000;
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid1, 0, emission);

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: 350 - 103.3333 ~ 246.67
        //             cold1, hot0: 103.3333
        //
        assert_substake_approx_eq!(&coldkey0, &hotkey0, netuid1, 246.67);
        assert_substake_approx_eq!(&coldkey1, &hotkey0, netuid1, 103.33);
    });
}

#[test]
fn test_two_subnets_take_ok() {
    new_test_ext(1).execute_with(|| {
        let netuid1 = 1;
        let netuid2 = 2;
        let hotkey0 = U256::from(1);
        let hotkey1 = U256::from(2);
        let coldkey0 = U256::from(3);
        let coldkey1 = U256::from(4);

        // Create networks.
        let lock_cost_1 = SubtensorModule::get_network_lock_cost();
        setup_dynamic_network(netuid1, 3u16, 1u16);
        let lock_cost_2 = SubtensorModule::get_network_lock_cost();
        setup_dynamic_network(netuid2, 3u16, 2u16);
        SubtensorModule::add_balance_to_coldkey_account( &coldkey0, 1000_000_000_000 );
        SubtensorModule::add_balance_to_coldkey_account( &coldkey1, 1000_000_000_000 );
        SubtensorModule::add_balance_to_coldkey_account( &hotkey0, 1000_000_000_000 );
        SubtensorModule::add_balance_to_coldkey_account( &hotkey1, 1000_000_000_000 );

        // The tests below assume lock costs of LC1 = LC2 = 100
        assert_eq!(lock_cost_1, 100_000_000_000);
        assert_eq!(lock_cost_2, 100_000_000_000);

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: LC1     (100)
        //
        // DynamicTAOReserve (get_tao_reserve) assertions
        //   Subnet 1: 100
        //   Subnet 2: 100
        //
        // DynamicAlphaReserve (get_alpha_reserve) assertions
        //   Subnet 1: 100
        //   Subnet 2: 200
        //
        // DynamicAlphaOutstanding (get_alpha_outstading) assertions
        //   Subnet 1: 100
        //   Subnet 2: 200
        //
        assert_substake_eq!(&coldkey0, &hotkey0, netuid1, 100_000_000_000);
        assert_substake_eq!(&coldkey0, &hotkey1, netuid2, 200_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid1), 100_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid2), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid2), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid2), 200_000_000_000);

        // Hotkey 0 becomes a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0
        ));

        // Hotkey 1 becomes a delegate
        assert_ok!(SubtensorModule::do_become_delegate(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1
        ));

        // Hotkey 0 sets the take on subnet 1 to 10%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey0,
            netuid1,
            u16::MAX / 10
        ));

        // Hotkey 1 sets the take on subnet 2 to 20%
        assert_ok!(SubtensorModule::do_decrease_take(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey0),
            hotkey1,
            netuid2,
            u16::MAX / 5
        ));

        // Nominate 100 from coldkey1 to hotkey0 on subnet 1
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey0,
            netuid1,
            100_000_000_000
        ));

        // Nominate 100 from coldkey1 to hotkey1 on subnet 2
        assert_ok!(SubtensorModule::add_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey1),
            hotkey1,
            netuid2,
            100_000_000_000
        ));

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: 100
        //             cold1, hot0: 50
        //   Subnet 2, cold0, hot1: 200
        //             cold1, hot1: 100
        //
        // DynamicTAOReserve (get_tao_reserve) assertions
        //   Subnet 1: 200
        //
        // DynamicAlphaReserve (get_alpha_reserve) assertions
        //   Subnet 1: 50
        //
        // DynamicAlphaOutstanding (get_alpha_outstading) assertions
        //   Subnet 1: 150
        //
        assert_substake_eq!(&coldkey0, &hotkey0, netuid1, 100_000_000_000);
        assert_substake_eq!(&coldkey1, &hotkey0, netuid1, 50_000_000_000);
        assert_substake_eq!(&coldkey0, &hotkey1, netuid2, 200_000_000_000);
        assert_substake_eq!(&coldkey1, &hotkey1, netuid2, 100_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid1), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid1), 50_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid1), 150_000_000_000);
        assert_eq!(SubtensorModule::get_tao_reserve(netuid2), 200_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_reserve(netuid2), 100_000_000_000);
        assert_eq!(SubtensorModule::get_alpha_outstanding(netuid2), 300_000_000_000);

        // Emission
        //
        // Emit inflation through run_coinbase
        // We will emit 0 server emission (which should go in-full to the owner of the hotkey).
        // We will emit 100 validator emission through each of hotkeys, which should be
        // distributed in-part to the nominators.
        //
        let emission = 100_000_000_000;
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey0, netuid1, 0, emission);
        SubtensorModule::emit_inflation_through_hotkey_account(&hotkey1, netuid2, 0, emission);

        // SubStake (Alpha balance)
        //   Subnet 1, cold0, hot0: 170
        //             cold1, hot0: 80
        //   Subnet 2, cold0, hot1: 273.34
        //             cold1, hot1: 126.67
        //
        assert_substake_approx_eq!(&coldkey0, &hotkey0, netuid1, 170.);
        assert_substake_approx_eq!(&coldkey1, &hotkey0, netuid1, 80.);
        assert_substake_approx_eq!(&coldkey0, &hotkey1, netuid2, 273.33);
        assert_substake_approx_eq!(&coldkey1, &hotkey1, netuid2, 126.67);
    });
}
