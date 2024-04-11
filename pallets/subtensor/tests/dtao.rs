use crate::mock::*;
use frame_support::assert_ok;
use frame_system::Config;
use frame_system::{EventRecord, Phase};
use substrate_fixed::types::I64F64;
use sp_core::U256;
mod mock;

// To run just the tests in this file, use the following command:
// Use the following command to run the tests in this file with verbose logging:
// RUST_LOG=debug cargo test -p pallet-subtensor --test dtao

#[test]
fn test_add_subnet_stake_ok_no_emission() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let hotkey = U256::from(0);
        let coldkey = U256::from(1);
        
        SubtensorModule::add_balance_to_coldkey_account( &coldkey, 100_000_000_000 ); // 100 TAO.
        // Check
        // -- that the lock cost is 100 TAO.
        // -- that the balance is 100 TAO.
        // -- that the root pool is empty.
        // -- that the root alpha pool is empty.
        // -- that the root price is 1.0.
        // -- that the root has zero k value.
        assert_eq!( SubtensorModule::get_network_lock_cost(), 100_000_000_000 ); // 100 TAO.
        assert_eq!( SubtensorModule::get_coldkey_balance( &coldkey ), 100_000_000_000 ); // 0 TAO.
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey_and_subnet( &hotkey, 0), 0 ); // 1 subnets * 100 TAO lock cost.
        assert_eq!( SubtensorModule::get_total_stake_for_subnet( 0 ), 0 );
        assert_eq!( SubtensorModule::get_tao_per_alpha_price(0), 1.0 );
        assert_eq!( SubtensorModule::get_tao_reserve(0), 0 );
        assert_eq!( SubtensorModule::get_alpha_reserve(0), 0 );
        assert_eq!( SubtensorModule::get_pool_k(0), 0 );
        assert_eq!( SubtensorModule::is_subnet_dynamic(0), false );

        // Register a network with this coldkey + hotkey for a lock cost of 1 TAO.
        step_block(1);
        assert_ok!( SubtensorModule::register_network( <<Test as Config>::RuntimeOrigin>::signed(coldkey), hotkey ));

        // Check: 
        // -- that the lock cost is now doubled.
        // -- that the lock cost has been withdrawn from the balance.
        // -- that the owner of the new subnet is the coldkey.
        // -- that the new network as someone registered.
        // -- that the registered key is the hotkey.
        // -- that the hotkey is owned by the owning coldkey.
        // -- that the hotkey has stake on the new network equal to the lock cost. Alpha/TAO price of 1 to 1.
        // -- that the total stake per subnet is 100 TAO.
        // -- that the new alpha/tao price is 1.0.
        // -- that the tao reserve is 100 TAO.
        // -- that the alpha reserve is 100 ALPHA
        // -- that the k factor is 100 TAO * 100 ALPHA.
        // -- that the new network is dynamic
        assert_eq!( SubtensorModule::get_network_lock_cost(), 200_000_000_000 ); // 200 TAO.
        assert_eq!( SubtensorModule::get_coldkey_balance( &coldkey ), 0 ); // 0 TAO.
        assert_eq!( SubtensorModule::get_subnet_owner( 1 ), coldkey );
        assert_eq!( SubtensorModule::get_subnetwork_n( 1 ), 1 );
        assert_eq!( SubtensorModule::get_hotkey_for_net_and_uid( 1, 0 ).unwrap(), hotkey );
        assert_eq!( SubtensorModule::get_owning_coldkey_for_hotkey( &hotkey ), coldkey );
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey_and_subnet( &hotkey, 1), 100_000_000_000 ); // 1 subnets * 100 TAO lock cost.
        assert_eq!( SubtensorModule::get_total_stake_for_subnet( 1 ), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_tao_per_alpha_price(1), 1.0 );
        assert_eq!( SubtensorModule::get_tao_reserve(1), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(1), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_pool_k(1), 100_000_000_000 * 100_000_000_000 );
        assert_eq!( SubtensorModule::is_subnet_dynamic(1), true );

        // Register a new network
        assert_eq!( SubtensorModule::get_network_lock_cost(), 200_000_000_000 ); // 100 TAO.
        SubtensorModule::add_balance_to_coldkey_account( &coldkey, 200_000_000_000 ); // 100 TAO.
        assert_ok!( SubtensorModule::register_network( <<Test as Config>::RuntimeOrigin>::signed(coldkey), hotkey ));

         // Check: 
        // -- that the lock cost is now doubled.
        // -- that the lock cost has been withdrawn from the balance.
        // -- that the owner of the new subnet is the coldkey.
        // -- that the new network as someone registered.
        // -- that the registered key is the hotkey.
        // -- that the hotkey is owned by the owning coldkey.
        // -- that the hotkey has stake on the new network equal to the lock cost. Alpha/TAO price of 1 to 1.
        // -- that the total stake per subnet 2 is 400 TAO.
        // -- that the new alpha/tao price is 0.5.
        // -- that the tao reserve is 200 TAO.
        // -- that the alpha reserve is 400 ALPHA
        // -- that the k factor is 200 TAO * 400 ALPHA.
        // -- that the new network is dynamic
        assert_eq!( SubtensorModule::get_network_lock_cost(), 400_000_000_000 ); // 4 TAO.
        assert_eq!( SubtensorModule::get_coldkey_balance( &coldkey ), 0 ); // 0 TAO.
        assert_eq!( SubtensorModule::get_subnet_owner( 2 ), coldkey );
        assert_eq!( SubtensorModule::get_subnetwork_n( 2 ), 1 );
        assert_eq!( SubtensorModule::get_hotkey_for_net_and_uid( 2, 0 ).unwrap(), hotkey );
        assert_eq!( SubtensorModule::get_owning_coldkey_for_hotkey( &hotkey ), coldkey );
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey_and_subnet( &hotkey, 2), 400_000_000_000 ); // 2 subnets * 2 TAO lock cost.
        assert_eq!( SubtensorModule::get_total_stake_for_subnet( 2 ), 400_000_000_000 );
        assert_eq!( SubtensorModule::get_tao_per_alpha_price(2), 0.5 );
        assert_eq!( SubtensorModule::get_tao_reserve(2), 200_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(2), 400_000_000_000 );
        assert_eq!( SubtensorModule::get_pool_k(2), 200_000_000_000 * 400_000_000_000 );
        assert_eq!( SubtensorModule::is_subnet_dynamic(2), true );


        // Let's remove all of our stake from subnet 2.
        // Check:
        // -- that the balance is initially 0
        // -- that the unstake event is ok.
        // -- that the balance is 100 TAO. Given the slippage.
        // -- that the price per alpha has changed to 0.125
        // -- that the tao reserve is 100 TAO.
        // -- that the alpha reserve is 800 ALPHA
        // -- that the k factor is 100 TAO * 400 ALPHA. (unchanged)
        assert_eq!(Balances::free_balance(coldkey), 0 );
        assert_ok!(SubtensorModule::remove_subnet_stake(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey),
            hotkey,
            2,
            400_000_000_000
        ));
        assert_eq!( Balances::free_balance(coldkey), 100_000_000_000);
        assert_eq!( SubtensorModule::get_tao_per_alpha_price(2), 0.125 );
        assert_eq!( SubtensorModule::get_tao_reserve(2), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(2), 800_000_000_000 );
        assert_eq!( SubtensorModule::get_pool_k(2), 200_000_000_000 * 400_000_000_000 );

        // Let's run a block step.
        // Check
        // -- that the pending emission for the 2 subnets is correct
        // -- that the pending alpha emission of the 2 subnets is correct.
        assert_eq!( SubtensorModule::get_alpha_pending_emission(1), 0 );
        assert_eq!( SubtensorModule::get_alpha_pending_emission(2), 0 );
        assert_eq!( SubtensorModule::get_tao_per_alpha_price(1), 1.0 );
        assert_eq!( SubtensorModule::get_tao_per_alpha_price(2), 0.125 );
        step_block(1);
        assert_eq!( SubtensorModule::get_tao_reserve(1), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_tao_reserve(2), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(1), 101_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(2), 801_000_000_000 );
        run_to_block(10);
        assert_eq!( SubtensorModule::get_tao_reserve(1), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_tao_reserve(2), 100_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(1), 109_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(2), 809_000_000_000 );
        run_to_block(30);
        assert_eq!( SubtensorModule::get_tao_reserve(1), 112_269_348_487 );
        assert_eq!( SubtensorModule::get_tao_reserve(2), 101_730_651_499 );
        assert_eq!( SubtensorModule::get_alpha_reserve(1), 129_000_000_000 );
        assert_eq!( SubtensorModule::get_alpha_reserve(2), 829_000_000_000 );

        for _ in 0..100 {
            step_block(1);
            log::info!("S1: {}, S2: {}", SubtensorModule::get_tao_per_alpha_price(1), SubtensorModule::get_tao_per_alpha_price(2));
        }

    });
}
