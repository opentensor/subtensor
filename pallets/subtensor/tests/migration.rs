mod mock;
use mock::*;
use sp_core::U256;

#[test]
fn test_migration_fix_total_stake_maps_wrong_entries() {
    new_test_ext().execute_with(|| {
		let ck1 = U256::from(1);
		let ck2 = U256::from(2);
		let ck3 = U256::from(3);

		let hk1 = U256::from(1 + 100);
		let hk2 = U256::from(2 + 100);

		let mut total_stake_amount = 0;

		// Give each coldkey some stake in the maps
		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck1,
			&hk1,
			100
		);
		total_stake_amount += 100;

		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck2,
			&hk1,
			10_101
		);
		total_stake_amount += 10_101;

		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck3,
			&hk2,
			100_000_000
		);
		total_stake_amount += 100_000_000;

		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck1,
			&hk2,
			1_123_000_000
		);
		total_stake_amount += 1_123_000_000;

		// Check that the total stake is correct
		assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);

		// Check that the total coldkey stake is correct
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck1), 100 + 1_123_000_000);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck3), 100_000_000);

		// Check that the total hotkey stake is correct
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk1), 100 + 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk2), 100_000_000 + 1_123_000_000);

		// Mess up the total coldkey stake
		pallet_subtensor::TotalColdkeyStake::<Test>::insert(ck1, 0);
		// Verify that the total coldkey stake is now 0 for ck1
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck1), 0);

		// Mess up the total stake
		pallet_subtensor::TotalStake::<Test>::put(123_456_789);
		// Verify that the total stake is now wrong
		assert_ne!(SubtensorModule::get_total_stake(), total_stake_amount);

		// Mess up the total hotkey stake
		pallet_subtensor::TotalHotkeyStake::<Test>::insert(hk1, 0);
		// Verify that the total hotkey stake is now wrong
		assert_ne!(SubtensorModule::get_total_stake_for_hotkey(&hk1), 0);

		// Run the migration to fix the total stake maps
		pallet_subtensor::migration::migrate_to_v2_fixed_total_stake::<Test>();

		// Verify that the total stake is now correct
		assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);
		// Verify that the total coldkey stake is now correct for each coldkey
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck1), 100 + 1_123_000_000);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck3), 100_000_000);

		// Verify that the total hotkey stake is now correct for each hotkey
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk1), 100 + 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk2), 100_000_000 + 1_123_000_000);

		// Verify that the Stake map has no extra entries
		assert_eq!(pallet_subtensor::Stake::<Test>::iter().count(), 4); // 4 entries total
		assert_eq!(pallet_subtensor::Stake::<Test>::iter_key_prefix(hk1).count(), 2); // 2 stake entries for hk1
		assert_eq!(pallet_subtensor::Stake::<Test>::iter_key_prefix(hk2).count(), 2); // 2 stake entries for hk2
    })
}

#[test]
fn test_migration_fix_total_stake_maps_extra_entries() {
    new_test_ext().execute_with(|| {
		let ck1 = U256::from(1);
		let ck2 = U256::from(2);
		let ck3 = U256::from(3);

		let hk1 = U256::from(1 + 100);
		let hk2 = U256::from(2 + 100);

		let mut total_stake_amount = 0;

		// Give each coldkey some stake in the maps
		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck1,
			&hk1,
			100
		);
		total_stake_amount += 100;

		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck2,
			&hk1,
			10_101
		);
		total_stake_amount += 10_101;

		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck3,
			&hk2,
			100_000_000
		);
		total_stake_amount += 100_000_000;

		SubtensorModule::increase_stake_on_coldkey_hotkey_account(
			&ck1,
			&hk2,
			1_123_000_000
		);
		total_stake_amount += 1_123_000_000;

		// Check that the total stake is correct
		assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);

		// Check that the total coldkey stake is correct
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck1), 100 + 1_123_000_000);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck3), 100_000_000);

		// Check that the total hotkey stake is correct
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk1), 100 + 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk2), 100_000_000 + 1_123_000_000);

		// Mess up the total coldkey stake by adding a new entry for a new coldkey
		let ck4 = U256::from(4);
		pallet_subtensor::TotalColdkeyStake::<Test>::insert(ck4, 123_456_000);
		// Verify that the total coldkey stake is present for ck4
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck4), 123_456_000);

		// Mess up the total hotkey stake by adding a new entry for a new hotkey
		let hk3 = U256::from(3 + 100);
		pallet_subtensor::TotalHotkeyStake::<Test>::insert(hk3, 123_456_000);
		// Verify that the total hotkey stake is now present for hk3
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk3), 123_456_000);

		// Run the migration to fix the total stake maps
		pallet_subtensor::migration::migrate_to_v2_fixed_total_stake::<Test>();

		// Verify that the total stake is now correct
		assert_eq!(SubtensorModule::get_total_stake(), total_stake_amount);
		// Verify that the total coldkey stake is now correct for each coldkey
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck1), 100 + 1_123_000_000);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck2), 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck3), 100_000_000);
		// Verify new coldkey 4 has no stake
		assert_eq!(SubtensorModule::get_total_stake_for_coldkey(&ck4), 0);

		// Verify that the total hotkey stake is now correct for each hotkey
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk1), 100 + 10_101);
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk2), 100_000_000 + 1_123_000_000);
		// Verify new hotkey 3 has no stake
		assert_eq!(SubtensorModule::get_total_stake_for_hotkey(&hk3), 0);

		// Verify that the Stake map has no extra entries
		assert_eq!(pallet_subtensor::Stake::<Test>::iter().count(), 4); // 4 entries total
		assert_eq!(pallet_subtensor::Stake::<Test>::iter_key_prefix(hk1).count(), 2); // 2 stake entries for hk1
		assert_eq!(pallet_subtensor::Stake::<Test>::iter_key_prefix(hk2).count(), 2); // 2 stake entries for hk2
    })
}
