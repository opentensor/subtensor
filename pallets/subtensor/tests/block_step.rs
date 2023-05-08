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