mod mock;
use mock::*;
use sp_core::U256;
// use pallet_subtensor::*;

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_create_run_blocks_with_one_coldkey_and_hotkey --exact --nocapture
#[test]
fn test_create_dtao_and_stao_subnets() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000_000);
        let netuid_a = register_network(coldkey, hotkey, 0); // Mechanism = 0 is STAO
        let netuid_b = register_network(coldkey, hotkey, 1); // Mechanism = 1 is STAO
        let netuid_c = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        let netuid_d = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        pallet_subtensor::Tempo::<Test>::insert(netuid_a, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_b, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_c, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_d, 1);
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_a) ), 100_000_000_000); // 100k lock
        assert_eq!(pallet_subtensor::SubnetTAO::<Test>::get( netuid_a ), 100_000_000_000); // 200k lock
        assert_eq!(pallet_subtensor::SubnetAlpha::<Test>::get( netuid_a ), 100_000_000_000); // 100k --> 100k tao -> alpha
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_b) ), 200_000_000_000); // 200k lock
        assert_eq!(pallet_subtensor::SubnetTAO::<Test>::get( netuid_b ), 200_000_000_000); // 200k lock
        assert_eq!(pallet_subtensor::SubnetAlpha::<Test>::get( netuid_b ), 200_000_000_000); // 200k --> 200k tao -> alpha
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_c) ), 400_000_000_000); // 400k lock.
        assert_eq!(pallet_subtensor::SubnetTAO::<Test>::get( netuid_c ), 400_000_000_000); // 400k lock
        assert_eq!(pallet_subtensor::SubnetAlpha::<Test>::get( netuid_c ), 400_000_000_000); // 400k --> 400k tao -> alpha
        assert_eq!(pallet_subtensor::Alpha::<Test>::get( (hotkey, coldkey, netuid_d) ), 1_200_000_000_000); // 800k lock --> 1200k alpha.
        assert_eq!(pallet_subtensor::SubnetTAO::<Test>::get( netuid_d ), 800_000_000_000); // 800k lock
        assert_eq!(pallet_subtensor::SubnetAlpha::<Test>::get( netuid_d ), 1_200_000_000_000); // 800k --> 1200k tao -> alpha
        assert_eq!(SubtensorModule::get_global_for_hotkey( &hotkey ), 1_500_000_000_000); // 100k + 200K + 400K + 800K (these are the lock costs.)
        assert_eq!(SubtensorModule::get_global_for_hotkey_on_subnet( &hotkey, netuid_a ), 100_000_000_000); // 100k lock
        assert_eq!(SubtensorModule::get_global_for_hotkey_on_subnet( &hotkey, netuid_b ), 200_000_000_000); // 200k lock
        assert_eq!(SubtensorModule::get_global_for_hotkey_on_subnet( &hotkey, netuid_c ), 400_000_000_000); // 400k lock
        assert_eq!(SubtensorModule::get_global_for_hotkey_on_subnet( &hotkey, netuid_d ), 800_000_000_000); // 800k lock
        assert_eq!(SubtensorModule::get_global_for_hotkey_and_coldkey( &hotkey, &coldkey ), 1_500_000_000_000); // 100k + 200K + 400K + 800K (these are the lock costs.)
    });
}


// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_run_steps --exact --nocapture
#[test]
fn test_run_steps() {
    new_test_ext(1).execute_with(|| {
        let coldkey = U256::from(1);
        let hotkey = U256::from(2);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey, 100_000_000_000_000);
        let netuid_a = register_network(coldkey, hotkey, 0); // Mechanism = 0 is STAO
        let netuid_b = register_network(coldkey, hotkey, 1); // Mechanism = 1 is STAO
        let netuid_c = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        let netuid_d = register_network(coldkey, hotkey, 2); // Mechanism = 2 is DTAO
        pallet_subtensor::Tempo::<Test>::insert(netuid_a, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_b, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_c, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_d, 1);
        step_block( 4 );
    });
}


// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_run_steps_different_coldkeys --exact --nocapture
#[test]
fn test_run_steps_different_coldkeys() {
    new_test_ext(1).execute_with(|| {
        let hotkey = U256::from(1);
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(2), 100_000_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(3), 100_000_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(4), 100_000_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(5), 100_000_000_000_000);
        let netuid_a = register_network( U256::from(2), hotkey, 0); // Mechanism = 0 is STAO
        let netuid_b = register_network( U256::from(3), hotkey, 1); // Mechanism = 1 is STAO
        let netuid_c = register_network( U256::from(4), hotkey, 2); // Mechanism = 2 is DTAO
        let netuid_d = register_network( U256::from(5), hotkey, 2); // Mechanism = 2 is DTAO
        pallet_subtensor::Tempo::<Test>::insert(netuid_a, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_b, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_c, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_d, 1);
        step_block( 4 );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_run_steps_different_cold_different_hot --exact --nocapture
#[test]
fn test_run_steps_different_cold_different_hot() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(2), 100_000_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(3), 100_000_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(4), 100_000_000_000_000);
        SubtensorModule::add_balance_to_coldkey_account(&U256::from(5), 100_000_000_000_000);
        let netuid_a = register_network( U256::from(2), U256::from(6), 0); // Mechanism = 0 is STAO
        let netuid_b = register_network( U256::from(3), U256::from(7), 1); // Mechanism = 1 is STAO
        let netuid_c = register_network( U256::from(4), U256::from(8), 2); // Mechanism = 2 is DTAO
        let netuid_d = register_network( U256::from(5), U256::from(9), 2); // Mechanism = 2 is DTAO
        pallet_subtensor::Tempo::<Test>::insert(netuid_a, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_b, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_c, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_d, 1);
        step_block( 4 );
    });
}

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --test alpha -- test_run_steps_same_cold_different_hot --exact --nocapture
#[test]
fn test_run_steps_same_cold_different_hot() {
    new_test_ext(1).execute_with(|| {
        SubtensorModule::add_balance_to_coldkey_account( &U256::from(1), 100_000_000_000_000 );
        let netuid_a = register_network( U256::from(1), U256::from(6), 0); // Mechanism = 0 is STAO
        let netuid_b = register_network( U256::from(1), U256::from(7), 1); // Mechanism = 1 is STAO
        let netuid_c = register_network( U256::from(1), U256::from(8), 2); // Mechanism = 2 is DTAO
        let netuid_d = register_network( U256::from(1), U256::from(9), 2); // Mechanism = 2 is DTAO
        pallet_subtensor::Tempo::<Test>::insert(netuid_a, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_b, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_c, 1);
        pallet_subtensor::Tempo::<Test>::insert(netuid_d, 1);
        step_block( 4 );
    });
}




