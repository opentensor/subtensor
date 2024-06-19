use crate::mock::*;
mod mock;
use sp_core::U256;

// To run this test specifically, use the following command:
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test coinbase test_coinbase_basic -- --nocapture
#[test]
#[cfg(not(tarpaulin))]
fn test_coinbase_basic() {
    new_test_ext(1).execute_with(|| {
        // Define network ID
        let netuid: u16 = 1;
        let tempo: u16 = 1;
        let hotkey = U256::from(1);
        let coldkey = U256::from(3);

        // Create a network with a tempo 1
        add_network( netuid, 1, 0 );
        register_ok_neuron( netuid, hotkey, coldkey, 100000 );
        SubtensorModule::create_account_if_non_existent(&coldkey, &hotkey);
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1000);

        // Set the subnet emission value to 1.
        SubtensorModule::set_emission_values(&[netuid], vec![1]).unwrap();
        assert_eq!( SubtensorModule::get_subnet_emission_value( netuid ), 1 );

        // Hotkey has no pending emission
        assert_eq!( SubtensorModule::get_pending_hotkey_emission( &hotkey ), 0 );

        // Hotkey has same stake
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey ), 1000 + 0 );

        // Subnet has no pending emission. 
        assert_eq!( SubtensorModule::get_pending_emission( netuid ), 0 );

        // Step block
        next_block();

        // Hotkey has no pending emission
        assert_eq!( SubtensorModule::get_pending_hotkey_emission( &hotkey ), 0 );

        // Hotkey has same stake
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey ), 1000 + 0 );

        // Subnet has no pending emission of 1 ( from coinbase )
        assert_eq!( SubtensorModule::get_pending_emission( netuid ), 1 );

        // Step block releases
        next_block();

        // Hotkey pending immediately drained.
        assert_eq!( SubtensorModule::get_pending_hotkey_emission( &hotkey ), 0 );

        // Hotkey has NEW stake
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey ), 1000 + 2 );

        // Subnet pending has been drained.
        assert_eq!( SubtensorModule::get_pending_emission( netuid ), 0 );

        // Set the hotkey drain time to 2 block.
        SubtensorModule::set_hotkey_emission_tempo( 2 );

        // Step block releases
        next_block();

        // Hotkey pending not yet increased. 
        assert_eq!( SubtensorModule::get_pending_hotkey_emission( &hotkey ), 0 );

        // Hotkey has same stake
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey ), 1000 + 2 );

        // Subnet pending has been drained.
        assert_eq!( SubtensorModule::get_pending_emission( netuid ), 1 );

        // Step block releases
        next_block();

        // Hotkey pending increases by 2 NOT yet drained.
        assert_eq!( SubtensorModule::get_pending_hotkey_emission( &hotkey ), 2 );

        // Hotkey has same stake.
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey ), 1000 + 2 );

        // Subnet pending has been drained.
        assert_eq!( SubtensorModule::get_pending_emission( netuid ), 0 );

        // Step block releases
        next_block();

        // Hotkey pending increases by 2 NOT yet drained.
        assert_eq!( SubtensorModule::get_pending_hotkey_emission( &hotkey ), 0 );

        // Hotkey stake increased by another 2 TAO drained from above.
        assert_eq!( SubtensorModule::get_total_stake_for_hotkey( &hotkey ), 1000 + 4 );

        // Subnet pending increaed by 1.
        assert_eq!( SubtensorModule::get_pending_emission( netuid ), 1 );

    });
}
