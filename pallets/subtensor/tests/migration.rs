mod mock;
// use frame_support::assert_ok;
// use frame_system::Config;
use mock::*;
// use sp_core::U256;

// #[test]
// // To run this test with cargo, use the following command:
// // cargo test --package pallet-subtensor --test migration test_migration5_total_issuance
// fn test_migration5_total_issuance() {
//     new_test_ext(1).execute_with(|| {
//         // Run the migration to check total issuance.
//         let test: bool = true;

//         assert_eq!(SubtensorModule::get_total_issuance(), 0);
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(test);
//         assert_eq!(SubtensorModule::get_total_issuance(), 0);

//         SubtensorModule::add_balance_to_coldkey_account(&U256::from(1), 10000);
//         assert_eq!(SubtensorModule::get_total_issuance(), 0);
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(test);
//         assert_eq!(SubtensorModule::get_total_issuance(), 10000);

//         SubtensorModule::increase_subnet_token_on_coldkey_hotkey_account(
//             &U256::from(1),
//             &U256::from(1),
//             1,
//             30000,
//         );
//         assert_eq!(SubtensorModule::get_total_issuance(), 10000);
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(test);
//         assert_eq!(SubtensorModule::get_total_issuance(), 10000 + 30000);
//     })
// }

// #[test]
// // To run this test with cargo, use the following command:
// // cargo test --package pallet-subtensor --test migration test_total_issuance_global
// fn test_total_issuance_global() {
//     new_test_ext(0).execute_with(|| {
//         // Initialize network unique identifier and keys for testing.
//         let netuid: u16 = 1; // Network unique identifier set to 1 for testing.
//         let coldkey = U256::from(0); // Coldkey initialized to 0, representing an account's public key for non-transactional operations.
//         let hotkey = U256::from(0); // Hotkey initialized to 0, representing an account's public key for transactional operations.
//         let owner: U256 = U256::from(0);

//         let lockcost: u64 = SubtensorModule::get_network_lock_cost();
//         SubtensorModule::add_balance_to_coldkey_account(&owner, lockcost); // Add a balance of lockcost to the coldkey account.

//         // Pallet balances issuance increases accordingly
//         assert_eq!(lockcost, PalletBalances::total_issuance());

//         assert_eq!(SubtensorModule::get_total_issuance(), 0); // initial is zero.
//         assert_ok!(SubtensorModule::register_network(
//             <<Test as Config>::RuntimeOrigin>::signed(owner),
//             hotkey
//         ));

//         // We register by withdrawing, balances total issuance goes back to one ED
//         assert_eq!(ExistentialDeposit::get(), PalletBalances::total_issuance());

//         SubtensorModule::set_max_allowed_uids(netuid, 2); // Set the maximum allowed neuron count for the network to 2.
//         assert_eq!(SubtensorModule::get_total_issuance(), 0); // initial is zero.
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Pick up lock.
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             lockcost + PalletBalances::total_issuance()
//         );
//         assert!(SubtensorModule::if_subnet_exist(netuid));

//         // Test the migration's effect on total issuance after adding balance to a coldkey account.
//         let account_balance: u64 = 20000;
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             lockcost + ExistentialDeposit::get()
//         ); // Ensure the total issuance starts at 0 before the migration.
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey, account_balance);
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Execute the migration to update total issuance.
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             account_balance + lockcost + ExistentialDeposit::get()
//         );

//         // Test the effect of burning on total issuance.
//         let coldkey2 = U256::from(1);
//         let hotkey2 = U256::from(1);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey2, account_balance);

//         let burn_cost: u64 = 10_000;
//         SubtensorModule::set_burn(netuid, burn_cost); // Set the burn amount to 10_000 for the network.
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             account_balance + lockcost + ExistentialDeposit::get()
//         ); // Confirm the total issuance remains 20000 before burning.
//         let neuron_count_before_burning = SubtensorModule::get_subnetwork_n(netuid);
//         assert_ok!(SubtensorModule::burned_register(
//             <<Test as Config>::RuntimeOrigin>::signed(hotkey2),
//             netuid,
//             hotkey2
//         )); // Execute the burn operation, reducing the total issuance.
//         let neuron_count_after_burning = SubtensorModule::get_subnetwork_n(netuid);
//         assert_eq!(neuron_count_after_burning - neuron_count_before_burning, 1); // Ensure the subnetwork count increases by 1 after burning
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             account_balance + lockcost - burn_cost + ExistentialDeposit::get()
//         ); // Verify the total issuance is reduced to 10000 after burning.
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Execute the migration to update total issuance.
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             2 * account_balance + lockcost - burn_cost + ExistentialDeposit::get()
//         ); // Verify the total issuance is updated to 10000 nothing changes

//         // Test staking functionality and its effect on total issuance.
//         let new_stake: u64 = 10000;
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             2 * account_balance + lockcost - burn_cost + ExistentialDeposit::get()
//         ); // Same
//         SubtensorModule::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, 1, new_stake); // Stake an additional 10000 to the coldkey-hotkey account. This is i
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             2 * account_balance + lockcost - burn_cost + ExistentialDeposit::get()
//         ); // Same
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Fix issuance
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             2 * account_balance + lockcost - burn_cost + new_stake + ExistentialDeposit::get()
//         ); // New

//         // Set emission values for the network and verify.
//         let emission: u64 = 1_000_000_000;
//         SubtensorModule::set_tempo(netuid, 1);
//         set_emission_values(netuid, emission);
//         assert_eq!(SubtensorModule::get_emission_value(netuid), emission); // Verify the emission value is set correctly for the network.
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             2 * account_balance + lockcost - burn_cost + new_stake + ExistentialDeposit::get()
//         );
//         run_to_block(2); // Advance to block number 2 to trigger the emission through the subnet.
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             2 * account_balance + lockcost - burn_cost
//                 + new_stake
//                 + emission
//                 + ExistentialDeposit::get()
//         ); // Verify the total issuance reflects the staked amount and emission value that has been put through the epoch.
//         pallet_subtensor::migration::migration5_total_issuance::<Test>(true); // Test migration does not change amount.
//         assert_eq!(
//             SubtensorModule::get_total_issuance(),
//             2 * account_balance + lockcost - burn_cost
//                 + new_stake
//                 + emission
//                 + ExistentialDeposit::get()
//         ); // Verify the total issuance reflects the staked amount and emission value that has been put through the epoch.
//     })
// }

#[test]
fn test_migration_transfer_nets_to_foundation() {
    new_test_ext(1).execute_with(|| {
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
    new_test_ext(1).execute_with(|| {
        // Create subnet 3
        add_network(3, 1, 0);
        assert!(SubtensorModule::if_subnet_exist(3));

        // Run the migration to transfer ownership
        pallet_subtensor::migration::migrate_delete_subnet_3::<Test>();

        assert!(!SubtensorModule::if_subnet_exist(3));
    })
}

#[test]
fn test_migration_delete_subnet_21() {
    new_test_ext(1).execute_with(|| {
        // Create subnet 21
        add_network(21, 1, 0);
        assert!(SubtensorModule::if_subnet_exist(21));

        // Run the migration to transfer ownership
        pallet_subtensor::migration::migrate_delete_subnet_21::<Test>();

        assert!(!SubtensorModule::if_subnet_exist(21));
    })
}

// #[test]
// fn test_migration_stake_to_substake() {
//     new_test_ext(1).execute_with(|| {
//         // We need to create the root network for this test
//         let root: u16 = 0;
//         let netuid: u16 = 1;
//         let tempo: u16 = 13;
//         let hotkey1 = U256::from(1);
//         let coldkey1 = U256::from(100);
//         let stake_amount1 = 1000u64;

//         let hotkey2 = U256::from(2);
//         let coldkey2 = U256::from(200);
//         let stake_amount2 = 2000u64;

//         //add root network
//         add_network(root, tempo, 0);
//         //add subnet 1
//         add_network(netuid, tempo, 0);

//         SubtensorModule::add_balance_to_coldkey_account(&coldkey1, stake_amount1);
//         SubtensorModule::add_balance_to_coldkey_account(&coldkey2, stake_amount2);

//         // Register neuron 1
//         register_ok_neuron(netuid, hotkey1, coldkey1, 0);
//         // Register neuron 2
//         register_ok_neuron(netuid, hotkey2, coldkey2, 0);

//         // Due to the way update stake work , we need to isolate just adding stake to the
//         // Stake StorageMap. We therefore need to manipulate the Stake StorageMap directly.
//         set_stake_value(coldkey1, hotkey1, stake_amount1);
//         assert_eq!(
//             pallet_subtensor::Stake::<Test>::get(coldkey1, hotkey1),
//             stake_amount1
//         );

//         set_stake_value(coldkey2, hotkey2, stake_amount2);
//         assert_eq!(
//             pallet_subtensor::Stake::<Test>::get(coldkey2, hotkey2),
//             stake_amount2
//         );

//         assert_eq!(
//             pallet_subtensor::SubStake::<Test>::get((&coldkey1, &hotkey1, &0u16)),
//             0
//         );
//         assert_eq!(
//             pallet_subtensor::SubStake::<Test>::get((&coldkey2, &hotkey2, &0u16)),
//             0
//         );
//         // Run the migration
//         pallet_subtensor::migration::migrate_stake_to_substake::<Test>();

//         // Verify that Stake entries have been migrated to SubStake
//         assert_eq!(
//             pallet_subtensor::SubStake::<Test>::get((&coldkey1, &hotkey1, &0u16)),
//             stake_amount1
//         );
//         assert_eq!(
//             pallet_subtensor::SubStake::<Test>::get((&coldkey2, &hotkey2, &0u16)),
//             stake_amount2
//         );

//         // Verify TotalHotkeySubStake has been updated
//         assert_eq!(
//             SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey1, 0),
//             stake_amount1
//         );
//         assert_eq!(
//             SubtensorModule::get_total_stake_for_hotkey_and_subnet(&hotkey2, 0),
//             stake_amount2
//         );
//     });
// }

// // Helper function to set a value in the Stake StorageMap
// fn set_stake_value(coldkey: U256, hotkey: U256, stake_amount: u64) {
//     pallet_subtensor::Stake::<Test>::insert(coldkey, hotkey, stake_amount);
// }
