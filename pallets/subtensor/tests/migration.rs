mod mock;


use mock::*;
use sp_core::U256;


#[test]
fn test_migration_fix_total_stake_maps() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let ck1 = U256::from(1);
        let ck2 = U256::from(2);
        let ck3 = U256::from(3);

        let hk1 = U256::from(1 + 100);
        let hk2 = U256::from(2 + 100);

        let mut total_stake_amount = 0;

        // Give each coldkey some stake in the maps
        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&ck1, &hk1, netuid, 100);
        total_stake_amount += 100;

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&ck2, &hk1, netuid, 10_101);
        total_stake_amount += 10_101;

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(&ck3, &hk2, netuid, 100_000_000);
        total_stake_amount += 100_000_000;

        SubtensorModule::increase_stake_on_coldkey_hotkey_account(
            &ck1,
            &hk2,
            netuid,
            1_123_000_000,
        );
        total_stake_amount += 1_123_000_000;

        // Check that the total stake is correct
        assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);

        // Check that the total coldkey stake is correct
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck1),
            100 + 1_123_000_000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck3),
            100_000_000
        );

        // Check that the total hotkey stake is correct
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk1),
            100 + 10_101
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk2),
            100_000_000 + 1_123_000_000
        );

        // Mess up the total coldkey stake
        pallet_subtensor::TotalColdkeyStake::<Test>::insert(ck1, 0);
        // Verify that the total coldkey stake is now 0 for ck1
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck1), 0);

        // Mess up the total stake
        pallet_subtensor::TotalStake::<Test>::put(123_456_789);
        // Verify that the total stake is now wrong
        assert_ne!(SubtensorModule::get_total_stake(), total_stake_amount);

        // Run the migration to fix the total stake maps
        pallet_subtensor::migration::migrate_to_v2_fixed_total_stake::<Test>();

        // Verify that the total stake is now correct
        assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);
        // Verify that the total coldkey stake is now correct for each coldkey
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck1),
            100 + 1_123_000_000
        );
        assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
        assert_eq!(
            SubtensorModule::get_total_stake_for_coldkey(&ck3),
            100_000_000
        );

        // Verify that the total hotkey stake is STILL correct for each hotkey
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk1),
            100 + 10_101
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey(&hk2),
            100_000_000 + 1_123_000_000
        );
    })
}

#[test]
fn test_migration_transfer_nets_to_foundation() {
    new_test_ext().execute_with(|| {
        // Create subnet 1
        add_network(1, 1, 0);
        // Create subnet 11
        add_network(11, 1, 0);

        log::info!("{:?}", SubtensorModule::get_subnet_owner(1));
        //assert_eq!(SubtensorModule::<T>::get_subnet_owner(1), );

        // Run the migration to transfer ownership
        let hex =
            hex_literal::hex!["feabaafee293d3b76dae304e2f9d885f77d2b17adab9e17e921b321eccd61c77"];
        pallet_subtensor::migration::migrate_transfer_ownership_to_foundation::<Test>(hex);

        log::info!("new owner: {:?}", SubtensorModule::get_subnet_owner(1));
    })
}

#[test]
fn test_migration_delete_subnet_3() {
    new_test_ext().execute_with(|| {
        // Create subnet 3
        add_network(3, 1, 0);
        assert_eq!(SubtensorModule::if_subnet_exist(3), true);

        // Run the migration to transfer ownership
        pallet_subtensor::migration::migrate_delete_subnet_3::<Test>();

        assert_eq!(SubtensorModule::if_subnet_exist(3), false);
    })
}

#[test]
fn test_migration_delete_subnet_21() {
    new_test_ext().execute_with(|| {
        // Create subnet 21
        add_network(21, 1, 0);
        assert_eq!(SubtensorModule::if_subnet_exist(21), true);

        // Run the migration to transfer ownership
        pallet_subtensor::migration::migrate_delete_subnet_21::<Test>();

        assert_eq!(SubtensorModule::if_subnet_exist(21), false);
    })
}

#[test]
fn test_migration_stake_to_substake() {
    new_test_ext().execute_with(|| {
        // We need to create the root network for this test
        let root: u16 = 0;
        let netuid: u16 = 1;
        let tempo: u16 = 13;
        let hotkey1 = U256::from(1);
        let coldkey1 = U256::from(100);
        let stake_amount1 = 1000u64;

        let hotkey2 = U256::from(2);
        let coldkey2 = U256::from(200);
        let stake_amount2 = 2000u64;

        //add root network
        add_network(root, tempo, 0);
        //add subnet 1
        add_network(netuid, tempo, 0);

        SubtensorModule::add_balance_to_coldkey_account(&coldkey1, stake_amount1);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey2, stake_amount2);

        // Register neuron 1
        register_ok_neuron(netuid, hotkey1, coldkey1, 0);
        // Register neuron 2
        register_ok_neuron(netuid, hotkey2, coldkey2, 0);

        // Due to the way update stake work , we need to isolate just adding stake to the
        // Stake StorageMap. We therefore need to manipulate the Stake StorageMap directly.
        set_stake_value(coldkey1, hotkey1, stake_amount1);
        assert_eq!(
            pallet_subtensor::Stake::<Test>::get(coldkey1, hotkey1),
            stake_amount1
        );

        set_stake_value(coldkey2, hotkey2, stake_amount2);
        assert_eq!(
            pallet_subtensor::Stake::<Test>::get(coldkey2, hotkey2),
            stake_amount2
        );

        assert_eq!(
            pallet_subtensor::SubStake::<Test>::get((&hotkey1, &coldkey1, &0u16)),
            0
        );
        assert_eq!(
            pallet_subtensor::SubStake::<Test>::get((&hotkey2, &coldkey2, &0u16)),
            0
        );
        // Run the migration
        pallet_subtensor::migration::migrate_stake_to_substake::<Test>();

        // Verify that Stake entries have been migrated to SubStake
        assert_eq!(
            pallet_subtensor::SubStake::<Test>::get((&hotkey1, &coldkey1, &0u16)),
            stake_amount1
        );
        assert_eq!(
            pallet_subtensor::SubStake::<Test>::get((&hotkey2, &coldkey2, &0u16)),
            stake_amount2
        );

        // Verify TotalHotkeySubStake has been updated
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey1, 0),
            stake_amount1
        );
        assert_eq!(
            SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey2, 0),
            stake_amount2
        );
    });
}

// Helper function to set a value in the Stake StorageMap
fn set_stake_value(coldkey: U256, hotkey: U256, stake_amount: u64) {
    pallet_subtensor::Stake::<Test>::insert(coldkey, hotkey, stake_amount);
}
