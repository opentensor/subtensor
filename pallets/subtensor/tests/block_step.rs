mod mock;
use frame_support::assert_ok;
use frame_system::Config;
use mock::*;
use sp_core::U256;

#[test]
fn test_loaded_emission() {
    new_test_ext().execute_with(|| { 
        let n: u16 = 100;
        let netuid: u16 = 0;
        let tempo: u16 = 10;
        let netuids: Vec<u16> = vec![0];
        let emission:  Vec<u64> = vec![1000000000];
        add_network( netuid, tempo, 0 );
        SubtensorModule::set_max_allowed_uids( netuid, n );
        assert_ok!(SubtensorModule::do_set_emission_values(<<Test as Config>::RuntimeOrigin>::root(), netuids, emission));
        for i in 0..n {SubtensorModule::append_neuron( netuid, &U256::from(i), 0 );}
        assert!( !SubtensorModule::has_loaded_emission_tuples( netuid ) );

        // Try loading at block 0
        let block: u64 = 0;
        assert_eq!( SubtensorModule::blocks_until_next_epoch( netuid, tempo, block ), 9 );
        SubtensorModule::generate_emission( block );
        assert!( !SubtensorModule::has_loaded_emission_tuples( netuid ) );

        // Try loading at block = 9;
        let block: u64 = 9;
        assert_eq!( SubtensorModule::blocks_until_next_epoch( netuid, tempo, block ), 0 );
        SubtensorModule::generate_emission( block );
        assert!( SubtensorModule::has_loaded_emission_tuples( netuid ) );
        assert_eq!( SubtensorModule::get_loaded_emission_tuples( netuid ).len(), n as usize );

        // Try draining the emission tuples
        // None remaining because we are at epoch.
        let block: u64 = 9;
        SubtensorModule::drain_emission( block );
        assert!( !SubtensorModule::has_loaded_emission_tuples( netuid ) );

        // Generate more emission.
        SubtensorModule::generate_emission( 9 );
        assert_eq!( SubtensorModule::get_loaded_emission_tuples( netuid ).len(), n as usize );
        
        for block in 10..20 {
            let mut n_remaining: usize = 0;
            let mut n_to_drain: usize = 0;
            if SubtensorModule::has_loaded_emission_tuples( netuid ) {
                n_remaining = SubtensorModule::get_loaded_emission_tuples( netuid ).len();
                n_to_drain = SubtensorModule::tuples_to_drain_this_block( netuid, tempo, block, SubtensorModule::get_loaded_emission_tuples( netuid ).len() );
            }
            SubtensorModule::drain_emission( block ); // drain it with 9 more blocks to go 
            if SubtensorModule::has_loaded_emission_tuples( netuid ) {
                assert_eq!( SubtensorModule::get_loaded_emission_tuples( netuid ).len(), n_remaining - n_to_drain );
            }
            log::info!( "n_to_drain:{:?}", n_to_drain.clone() );
            log::info!( "SubtensorModule::get_loaded_emission_tuples( netuid ).len():{:?}", n_remaining - n_to_drain );
        }

    })
}

#[test]
fn test_tuples_to_drain_this_block(){
    new_test_ext().execute_with(|| { 
        // pub fn tuples_to_drain_this_block( netuid: u16, tempo: u16, block_number: u64, n_remaining: usize ) -> usize {
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 1, 0, 10 ), 10 ); // drain all epoch block.
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 0, 0, 10 ), 10 ); // drain all no tempo.
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 10, 0, 10 ), 2 ); // drain 10 / ( 10 / 2 ) = 2
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 20, 0, 10 ), 1 ); // drain 10 / ( 20 / 2 ) = 1
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 10, 0, 20 ), 5 ); // drain 20 / ( 9 / 2 ) = 5 
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 20, 0, 0 ), 0 );  // nothing to drain.
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 10, 1, 20 ), 5 ); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 10, 10, 20 ), 4 ); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 10, 15, 20 ), 10 ); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 10, 19, 20 ), 20 ); // drain 19 / ( 10 / 2 ) = 4
        assert_eq!( SubtensorModule::tuples_to_drain_this_block( 0, 10, 20, 20 ), 20 ); // drain 19 / ( 10 / 2 ) = 4
        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    for l in 0 .. 10 {
                        assert!( SubtensorModule::tuples_to_drain_this_block( i, j, k, l ) <= 10 ); 
                    }
                }
            }
        }
    })
}


#[test]
fn test_blocks_until_epoch(){
    new_test_ext().execute_with(|| { 

        // Check tempo = 0 block = * netuid = *
        assert_eq!( SubtensorModule::blocks_until_next_epoch( 0, 0, 0 ), 1000 ); 

        // Check tempo = 1 block = * netuid = *
        assert_eq!( SubtensorModule::blocks_until_next_epoch( 0, 1, 0 ),  0 ); 
        assert_eq!( SubtensorModule::blocks_until_next_epoch( 1, 1, 0 ),  1 ); 
        assert_eq!( SubtensorModule::blocks_until_next_epoch( 0, 1, 1 ),  1 ); 
        assert_eq!( SubtensorModule::blocks_until_next_epoch( 1, 1, 1 ),  0 ); 
        assert_eq!( SubtensorModule::blocks_until_next_epoch( 0, 1, 2 ),  0 ); 
        assert_eq!( SubtensorModule::blocks_until_next_epoch( 1, 1, 2 ),  1 ); 
        for i in 0..100 { 
            if i % 2 == 0 {
                assert_eq!( SubtensorModule::blocks_until_next_epoch( 0, 1, i ),  0 ); 
                assert_eq!( SubtensorModule::blocks_until_next_epoch( 1, 1, i ),  1 ); 
            } else {
                assert_eq!( SubtensorModule::blocks_until_next_epoch( 0, 1, i ),  1 ); 
                assert_eq!( SubtensorModule::blocks_until_next_epoch( 1, 1, i ),  0 ); 
            }
        } 

        // Check general case.
        for netuid in 0..30 as u16 { 
            for block in 0..30 as u64 {
                for tempo in 1..30 as u16 {
                    assert_eq!( SubtensorModule::blocks_until_next_epoch( netuid, tempo, block ), tempo as u64 - ( block + netuid as u64 + 1 ) % ( tempo as u64  + 1 ) ); 
                }
            }
        } 


    });
}


/********************************************
	block_step::adjust_registration_terms_for_networks tests
*********************************************/

#[test]
fn test_burn_adjustment() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval = 1;
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// Register key 1.
		let hotkey_account_id_1 = U256::from(1);
		let coldkey_account_id_1 = U256::from(1);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_1, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1), netuid,  hotkey_account_id_1));

		// Register key 2.
		let hotkey_account_id_2 =U256::from(2);
		let coldkey_account_id_2 = U256::from(2);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_2, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2), netuid,  hotkey_account_id_2));

		// We are over the number of regs allowed this interval.
		// Step the block and trigger the adjustment.
		step_block( 1 );

		// Check the adjusted burn.
		assert_eq!(SubtensorModule::get_burn_as_u64(netuid), 1500);
	});
}

#[test]
#[allow(unused_assignments)]
fn test_burn_adjustment_case_a() {
	// Test case A of the difficulty and burn adjustment algorithm.
	// ====================
	// There are too many registrations this interval and most of them are pow registrations
    // this triggers an increase in the pow difficulty.
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval = 1;
		let start_diff: u64 = 10_000;
		let mut curr_block_num = 0;
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_difficulty(netuid, start_diff);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// Register key 1. This is a burn registration.
		let hotkey_account_id_1 = U256::from(1);
		let coldkey_account_id_1 = U256::from(1);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_1, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1), netuid,  hotkey_account_id_1));

		// Register key 2. This is a POW registration
		let hotkey_account_id_2 =U256::from(2);
		let coldkey_account_id_2 = U256::from(2);
		let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 0, &hotkey_account_id_2);
		let result0 = SubtensorModule::register(
			<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
			netuid,
			curr_block_num,
			nonce0, work0,
			hotkey_account_id_2, 
			coldkey_account_id_2
		);
		assert_ok!(result0);

		// Register key 3. This is a POW registration
		let hotkey_account_id_3 =U256::from(3);
		let coldkey_account_id_3 = U256::from(3);
		let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 11231312312, &hotkey_account_id_3);
		let result1 = SubtensorModule::register(
			<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
			netuid,
			curr_block_num,
			nonce1, work1,
			hotkey_account_id_3, 
			coldkey_account_id_3
		);
		assert_ok!(result1);

		// We are over the number of regs allowed this interval.
		// Most of them are POW registrations (2 out of 3)
		// Step the block and trigger the adjustment.
		step_block( 1 );
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
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval = 1;
		let start_diff: u64 = 10_000;
		let mut curr_block_num = 0;
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_difficulty(netuid, start_diff);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// Register key 1.
		let hotkey_account_id_1 = U256::from(1);
		let coldkey_account_id_1 = U256::from(1);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_1, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1), netuid,  hotkey_account_id_1));

		// Register key 2.
		let hotkey_account_id_2 =U256::from(2);
		let coldkey_account_id_2 = U256::from(2);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_2, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2), netuid,  hotkey_account_id_2));

		// Register key 3. This one is a POW registration
		let hotkey_account_id_3 = U256::from(3);
		let coldkey_account_id_3 = U256::from(3);
		let (nonce, work): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 0, &hotkey_account_id_3);
		let result = SubtensorModule::register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3), netuid, curr_block_num, nonce, work, hotkey_account_id_3, coldkey_account_id_3);
		assert_ok!(result);

		// We are over the number of regs allowed this interval.
		// Most of them are burn registrations (2 out of 3)
		// Step the block and trigger the adjustment.
		step_block( 1 );
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
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval = 4; // Needs registrations < 4 to trigger
		let start_diff: u64 = 10_000;
		let mut curr_block_num = 0;
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_difficulty(netuid, start_diff);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// Register key 1. This is a BURN registration
		let hotkey_account_id_1 = U256::from(1);
		let coldkey_account_id_1 = U256::from(1);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_1, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1), netuid,  hotkey_account_id_1));

		// Register key 2. This is a POW registration
		let hotkey_account_id_2 =U256::from(2);
		let coldkey_account_id_2 = U256::from(2);
		let (nonce0, work0): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 0, &hotkey_account_id_2);
		let result0 = SubtensorModule::register(
			<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2),
			netuid,
			curr_block_num,
			nonce0, work0,
			hotkey_account_id_2, 
			coldkey_account_id_2
		);
		assert_ok!(result0);

		// Register key 3. This is a POW registration
		let hotkey_account_id_3 =U256::from(3);
		let coldkey_account_id_3 = U256::from(3);
		let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 11231312312, &hotkey_account_id_3);
		let result1 = SubtensorModule::register(
			<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
			netuid,
			curr_block_num,
			nonce1, work1,
			hotkey_account_id_3, 
			coldkey_account_id_3
		);
		assert_ok!(result1);

		// We are UNDER the number of regs allowed this interval.
		// Most of them are POW registrations (2 out of 3)
		// Step the block and trigger the adjustment.
		step_block( 1 );
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
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval = 4; // Needs registrations < 4 to trigger
		let start_diff: u64 = 10_000;
		let mut curr_block_num = 0;
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_difficulty(netuid, start_diff);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// Register key 1. This is a BURN registration
		let hotkey_account_id_1 = U256::from(1);
		let coldkey_account_id_1 = U256::from(1);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_1, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1), netuid,  hotkey_account_id_1));
		
		// Register key 2. This is a BURN registration
		let hotkey_account_id_2 = U256::from(2);
		let coldkey_account_id_2 = U256::from(2);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_2, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2), netuid,  hotkey_account_id_2));

		// Register key 3. This is a POW registration
		let hotkey_account_id_3 =U256::from(3);
		let coldkey_account_id_3 = U256::from(3);
		let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 11231312312, &hotkey_account_id_3);
		let result1 = SubtensorModule::register(
			<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_3),
			netuid,
			curr_block_num,
			nonce1, work1,
			hotkey_account_id_3, 
			coldkey_account_id_3
		);
		assert_ok!(result1);

		// We are UNDER the number of regs allowed this interval.
		// Most of them are BURN registrations (2 out of 3)
		// Step the block and trigger the adjustment.
		step_block( 1 );
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
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval: u16 = 3;
		let start_diff: u64 = 10_000;
		let mut curr_block_num = 0;
		SubtensorModule::set_max_registrations_per_block( netuid, 10 );
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_difficulty(netuid, start_diff);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// Register key 1. This is a POW registration
		let hotkey_account_id_1 =U256::from(1);
		let coldkey_account_id_1 = U256::from(1);
		let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 11231312312, &hotkey_account_id_1);
		let result1 = SubtensorModule::register(
			<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
			netuid,
			curr_block_num,
			nonce1, work1,
			hotkey_account_id_1, 
			coldkey_account_id_1
		);
		assert_ok!(result1);

		// Register key 2. This is a BURN registration
		let hotkey_account_id_2 = U256::from(2);
		let coldkey_account_id_2 = U256::from(2);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_2, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2), netuid,  hotkey_account_id_2));


		step_block( 1 );
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
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval: u16 = 1;
		let start_diff: u64 = 10_000;
		let mut curr_block_num = 0;
		SubtensorModule::set_max_registrations_per_block( netuid, 10 );
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_difficulty(netuid, start_diff);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// Register key 1. This is a POW registration
		let hotkey_account_id_1 =U256::from(1);
		let coldkey_account_id_1 = U256::from(1);
		let (nonce1, work1): (u64, Vec<u8>) = SubtensorModule::create_work_for_block_number( netuid, curr_block_num, 11231312312, &hotkey_account_id_1);
		let result1 = SubtensorModule::register(
			<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_1),
			netuid,
			curr_block_num,
			nonce1, work1,
			hotkey_account_id_1, 
			coldkey_account_id_1
		);
		assert_ok!(result1);

		// Register key 2. This is a BURN registration
		let hotkey_account_id_2 = U256::from(2);
		let coldkey_account_id_2 = U256::from(2);
		SubtensorModule::add_balance_to_coldkey_account( &coldkey_account_id_2, 10000 );
		assert_ok!(SubtensorModule::burned_register(<<Test as Config>::RuntimeOrigin>::signed(hotkey_account_id_2), netuid,  hotkey_account_id_2));

		step_block( 1 );
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
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let burn_cost:u64 = 1000;
		let adjustment_interval = 1;
		let target_registrations_per_interval: u16 = 1;
		let start_diff: u64 = 10_000;
		SubtensorModule::set_max_registrations_per_block( netuid, 10 );
		SubtensorModule::set_burn( netuid, burn_cost);
		SubtensorModule::set_difficulty(netuid, start_diff);
		SubtensorModule::set_adjustment_interval( netuid, adjustment_interval ); 
		SubtensorModule::set_target_registrations_per_interval( netuid, target_registrations_per_interval);
		add_network(netuid, tempo, 0);

		// No registrations this interval of any kind.
		step_block( 1 );

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



// add_network( netuid1, tempo1, 0 );
// add_network( netuid2, tempo2, 0 );

// // // Lets step a block. There if no emission because we have not set an emission vector.
// assert_eq!( SubtensorModule::get_pending_emission( netuid0 ), 0 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid1 ), 0 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid2 ), 0 );
// step_block(1);
// assert_eq!( SubtensorModule::get_pending_emission( netuid0 ), 0 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid1 ), 0 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid2 ), 0 );

// // Lets set the block emission for this network. It will get all the emission.
// let netuids: Vec<u16> = vec![ 0, 1, 2];
// let emission: Vec<u64> = vec![ 333_333_333, 333_333_333, 333_333_334  ];
// assert_ok!( SubtensorModule::sudo_set_emission_values(<<Test as Config>::RuntimeOrigin>::root(), netuids, emission) );

// // Run a forward block. All emission ends up in pending.
// assert_eq!( SubtensorModule::get_emission_value( netuid0 ), 333_333_333 );
// assert_eq!( SubtensorModule::get_emission_value( netuid1 ), 333_333_333 );
// assert_eq!( SubtensorModule::get_emission_value( netuid2 ), 333_333_334 );
// step_block(1);
// assert_eq!( SubtensorModule::get_pending_emission( netuid0 ), 333_333_333 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid1 ), 333_333_333 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid2 ), 333_333_334 );

// // Run two more blocks and emission accrues for all networks.
// step_block(1);
// assert_eq!( SubtensorModule::get_pending_emission( netuid0 ), 666_666_666 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid1 ), 666_666_666 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid2 ), 666_666_668 );

// step_block(1);
// assert_eq!( SubtensorModule::get_pending_emission( netuid0 ), 999_999_999 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid1 ), 999_999_999 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid2 ), 1_000_000_002 );

// // Create keys.
// let hotkey0: u64 = 0;
// let coldkey0: u64 = 0;

// // Register 1 neuron to each network starting emission.
// register_ok_neuron( netuid0, hotkey0, coldkey0, 39420842 );
// register_ok_neuron( netuid1, hotkey0, coldkey0, 12412392 );
// register_ok_neuron( netuid2, hotkey0, coldkey0, 21813123 );

// // Run the block.
// step_block(1);
// assert_eq!( SubtensorModule::get_pending_emission( netuid0 ), 1_333_333_332 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid1 ), 1_333_333_332 );
// assert_eq!( SubtensorModule::get_pending_emission( netuid2 ), 1_333_333_336 );


// #[test]
// fn test_nakamoto(){
//     new_test_ext().execute_with(|| { 

//         // Create nakamoto.
//         let n: u16 = 10;
//         let netuid: u16 = 0;
//         let tempo: u16 = 100;
//         add_network( netuid, tempo, 0 );
//         let netuids: Vec<u16> = vec![ 0 ];
//         let emission: Vec<u64> = vec![ 1_000_000_000 ];
//         assert_ok!( SubtensorModule::sudo_set_emission_values(<<Test as Config>::RuntimeOrigin>::root(), netuids, emission) );

//         // Increase network size to 4096
//         SubtensorModule::set_max_allowed_uids( netuid, n );
//         SubtensorModule::set_max_registrations_per_block( netuid, n * 2 );

//         // Register neurons.
//         for i in 0..n as u64 {
//             log::trace!( "Register:\n{:?}\n", i );
//             register_ok_neuron( netuid, i, i, i * 1_000_000_000 + i * 1_000_000 );
//             assert_eq!( SubtensorModule::get_hotkey_for_net_and_uid( netuid, i as u16 ).unwrap(), i );
//         }

//         // Register the next batch to replace the older ones.
//         for i in 0..n as u64 {
//             log::trace!( "Register:\n{:?}\n", i );
//             register_ok_neuron( netuid, i + n as u64, i + n as u64 , i * 2_200_100_500 + i * 2_000_000 + 124124 );
//             assert_eq!( SubtensorModule::get_hotkey_for_net_and_uid( netuid, i as u16 ).unwrap(), i + n as u64 );
//         }

//     });
// }