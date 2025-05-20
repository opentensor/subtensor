#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use alloc::collections::BTreeMap;
use approx::assert_abs_diff_eq;
use codec::{Decode, Encode};
use frame_support::{
    StorageDoubleMap, StorageHasher, Twox64Concat, assert_ok,
    storage::unhashed::{get, get_raw, put, put_raw},
    traits::{StorageInstance, StoredMap},
    weights::Weight,
};

use crate::migrations::migrate_storage;
use frame_system::Config;
use sp_core::{H256, U256, crypto::Ss58Codec};
use sp_io::hashing::twox_128;
use sp_runtime::traits::Zero;
use substrate_fixed::types::I96F32;
use substrate_fixed::types::extra::U2;

#[allow(clippy::arithmetic_side_effects)]
fn close(value: u64, target: u64, eps: u64) {
    assert!(
        (value as i64 - target as i64).abs() < eps as i64,
        "Assertion failed: value = {}, target = {}, eps = {}",
        value,
        target,
        eps
    )
}

#[test]
fn test_initialise_ti() {
    use frame_support::traits::OnRuntimeUpgrade;

    new_test_ext(1).execute_with(|| {
        pallet_balances::TotalIssuance::<Test>::put(1000);
        crate::SubnetTAO::<Test>::insert(1, 100);
        crate::SubnetTAO::<Test>::insert(2, 5);

        // Ensure values are NOT initialized prior to running migration
        assert!(crate::TotalIssuance::<Test>::get() == 0);
		assert!(crate::TotalStake::<Test>::get() == 0);

        crate::migrations::migrate_init_total_issuance::initialise_total_issuance::Migration::<Test>::on_runtime_upgrade();

        // Ensure values were initialized correctly
		assert!(crate::TotalStake::<Test>::get() == 105);
        assert!(
            crate::TotalIssuance::<Test>::get()
                == 105u64.saturating_add(1000)
        );
    });
}

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
        crate::migrations::migrate_transfer_ownership_to_foundation::migrate_transfer_ownership_to_foundation::<Test>(hex);

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
        crate::migrations::migrate_delete_subnet_3::migrate_delete_subnet_3::<Test>();

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
        crate::migrations::migrate_delete_subnet_21::migrate_delete_subnet_21::<Test>();

        assert!(!SubtensorModule::if_subnet_exist(21));
    })
}

#[test]
fn test_migrate_commit_reveal_2() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entries
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_commit_reveal_2_v2";

        let pallet_prefix = twox_128("SubtensorModule".as_bytes());
        let storage_prefix_interval = twox_128("WeightCommitRevealInterval".as_bytes());
        let storage_prefix_commits = twox_128("WeightCommits".as_bytes());

        let netuid: u16 = 1;
        let interval_value: u64 = 50u64;

        // Construct the full key for WeightCommitRevealInterval
        let mut interval_key = Vec::new();
        interval_key.extend_from_slice(&pallet_prefix);
        interval_key.extend_from_slice(&storage_prefix_interval);
        interval_key.extend_from_slice(&netuid.encode());

        put_raw(&interval_key, &interval_value.encode());

        let test_account: U256 = U256::from(1);

        // Construct the full key for WeightCommits (DoubleMap)
        let mut commit_key = Vec::new();
        commit_key.extend_from_slice(&pallet_prefix);
        commit_key.extend_from_slice(&storage_prefix_commits);

        // First key (netuid) hashed with Twox64Concat
        let netuid_hashed = Twox64Concat::hash(&netuid.encode());
        commit_key.extend_from_slice(&netuid_hashed);

        // Second key (account) hashed with Twox64Concat
        let account_hashed = Twox64Concat::hash(&test_account.encode());
        commit_key.extend_from_slice(&account_hashed);

        let commit_value: (H256, u64) = (H256::from_low_u64_be(42), 100);
        put_raw(&commit_key, &commit_value.encode());

        let stored_interval = get_raw(&interval_key).expect("Expected to get a value");
        assert_eq!(
            u64::decode(&mut &stored_interval[..]).expect("Failed to decode interval value"),
            interval_value
        );

        let stored_commit = get_raw(&commit_key).expect("Expected to get a value");
        assert_eq!(
            <(H256, u64)>::decode(&mut &stored_commit[..]).expect("Failed to decode commit value"),
            commit_value
        );

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // ------------------------------
        // Step 2: Run the Migration
        // ------------------------------
        let weight = crate::migrations::migrate_commit_reveal_v2::migrate_commit_reveal_2::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as run"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------
        let stored_interval_after = get_raw(&interval_key);
        assert!(
            stored_interval_after.is_none(),
            "WeightCommitRevealInterval should be cleared"
        );

        let stored_commit_after = get_raw(&commit_key);
        assert!(
            stored_commit_after.is_none(),
            "WeightCommits entry should be cleared"
        );

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

// Leaving in for reference. Will remove later.
// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::migration::test_migrate_rao --exact --show-output --nocapture
// #[test]
// fn test_migrate_rao() {
//     new_test_ext(1).execute_with(|| {
//         // Setup initial state
//         let netuid_0: u16 = 0;
//         let netuid_1: u16 = 1;
//         let netuid_2: u16 = 2;
//         let netuid_3: u16 = 3;
//         let hotkey1 = U256::from(1);
//         let hotkey2 = U256::from(2);
//         let coldkey1 = U256::from(3);
//         let coldkey2 = U256::from(4);
//         let coldkey3 = U256::from(5);
//         let stake_amount: u64 = 1_000_000_000;
//         let lock_amount: u64 = 500;
//         NetworkMinLockCost::<Test>::set(500);

//         // Add networks root and alpha
//         add_network(netuid_0, 1, 0);
//         add_network(netuid_1, 1, 0);
//         add_network(netuid_2, 1, 0);
//         add_network(netuid_3, 1, 0);

//         // Set subnet lock
//         SubnetLocked::<Test>::insert(netuid_1, lock_amount);

//         // Add some initial stake
//         EmissionValues::<Test>::insert(netuid_1, 1_000_000_000);
//         EmissionValues::<Test>::insert(netuid_2, 2_000_000_000);
//         EmissionValues::<Test>::insert(netuid_3, 3_000_000_000);

//         Owner::<Test>::insert(hotkey1, coldkey1);
//         Owner::<Test>::insert(hotkey2, coldkey2);
//         Stake::<Test>::insert(hotkey1, coldkey1, stake_amount);
//         Stake::<Test>::insert(hotkey1, coldkey2, stake_amount);
//         Stake::<Test>::insert(hotkey2, coldkey2, stake_amount);
//         Stake::<Test>::insert(hotkey2, coldkey3, stake_amount);

//         // Verify initial conditions
//         assert_eq!(SubnetTAO::<Test>::get(netuid_0), 0);
//         assert_eq!(SubnetTAO::<Test>::get(netuid_1), 0);
//         assert_eq!(SubnetAlphaOut::<Test>::get(netuid_0), 0);
//         assert_eq!(SubnetAlphaOut::<Test>::get(netuid_1), 0);
//         assert_eq!(SubnetAlphaIn::<Test>::get(netuid_0), 0);
//         assert_eq!(SubnetAlphaIn::<Test>::get(netuid_1), 0);
//         assert_eq!(TotalHotkeyShares::<Test>::get(hotkey1, netuid_0), 0);
//         assert_eq!(TotalHotkeyShares::<Test>::get(hotkey1, netuid_1), 0);
//         assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey1, netuid_0), 0);
//         assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey2, netuid_1), 0);

//         // Run migration
//         crate::migrations::migrate_rao::migrate_rao::<Test>();

//         // Verify root subnet (netuid 0) state after migration
//         assert_eq!(SubnetTAO::<Test>::get(netuid_0), 4 * stake_amount); // Root has everything
//         assert_eq!(SubnetTAO::<Test>::get(netuid_1), 1_000_000_000); // Always 1000000000
//         assert_eq!(SubnetAlphaIn::<Test>::get(netuid_0), 1_000_000_000); // Always 1_000_000_000
//         assert_eq!(SubnetAlphaIn::<Test>::get(netuid_1), 1_000_000_000); // Always 1_000_000_000
//         assert_eq!(SubnetAlphaOut::<Test>::get(netuid_0), 4 * stake_amount); // Root has everything.
//         assert_eq!(SubnetAlphaOut::<Test>::get(netuid_1), 0); // No stake outstanding.

//         // Assert share information for hotkey1 on netuid_0
//         assert_eq!(
//             TotalHotkeyShares::<Test>::get(hotkey1, netuid_0),
//             2 * stake_amount
//         ); // Shares
//         // Assert no shares for hotkey1 on netuid_1
//         assert_eq!(TotalHotkeyShares::<Test>::get(hotkey1, netuid_1), 0); // No shares
//         // Assert alpha for hotkey1 on netuid_0
//         assert_eq!(
//             TotalHotkeyAlpha::<Test>::get(hotkey1, netuid_0),
//             2 * stake_amount
//         ); // Alpha
//         // Assert no alpha for hotkey1 on netuid_1
//         assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey1, netuid_1), 0); // No alpha.
//         // Assert share information for hotkey2 on netuid_0
//         assert_eq!(
//             TotalHotkeyShares::<Test>::get(hotkey2, netuid_0),
//             2 * stake_amount
//         ); // Shares
//         // Assert no shares for hotkey2 on netuid_1
//         assert_eq!(TotalHotkeyShares::<Test>::get(hotkey2, netuid_1), 0); // No shares
//         // Assert alpha for hotkey2 on netuid_0
//         assert_eq!(
//             TotalHotkeyAlpha::<Test>::get(hotkey2, netuid_0),
//             2 * stake_amount
//         ); // Alpha
//         // Assert no alpha for hotkey2 on netuid_1
//         assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey2, netuid_1), 0); // No alpha.

//         // Assert stake balances for hotkey1 and coldkey1 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey1, &coldkey1, netuid_0
//             ),
//             stake_amount
//         );
//         // Assert stake balances for hotkey1 and coldkey2 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey1, &coldkey2, netuid_0
//             ),
//             stake_amount
//         );
//         // Assert stake balances for hotkey2 and coldkey2 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey2, &coldkey2, netuid_0
//             ),
//             stake_amount
//         );
//         // Assert stake balances for hotkey2 and coldkey3 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey2, &coldkey3, netuid_0
//             ),
//             stake_amount
//         );
//         // Assert total stake for hotkey1 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey1, netuid_0),
//             2 * stake_amount
//         );
//         // Assert total stake for hotkey2 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey2, netuid_0),
//             2 * stake_amount
//         );
//         // Increase stake for hotkey1 and coldkey1 on netuid_0
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &hotkey1,
//             &coldkey1,
//             netuid_0,
//             stake_amount,
//         );
//         // Assert updated stake for hotkey1 and coldkey1 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey1, &coldkey1, netuid_0
//             ),
//             2 * stake_amount
//         );
//         // Assert updated total stake for hotkey1 on netuid_0
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey1, netuid_0),
//             3 * stake_amount
//         );
//         // Increase stake for hotkey1 and coldkey1 on netuid_1
//         SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
//             &hotkey1,
//             &coldkey1,
//             netuid_1,
//             stake_amount,
//         );
//         // Assert updated stake for hotkey1 and coldkey1 on netuid_1
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
//                 &hotkey1, &coldkey1, netuid_1
//             ),
//             stake_amount
//         );
//         // Assert updated total stake for hotkey1 on netuid_1
//         assert_eq!(
//             SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey1, netuid_1),
//             stake_amount
//         );

//         // Run the coinbase
//         let emission: u64 = 1_000_000_000;
//         SubtensorModule::run_coinbase(I96F32::from_num(emission));
//         close(
//             SubnetTaoInEmission::<Test>::get(netuid_1),
//             emission / 6,
//             100,
//         );
//         close(
//             SubnetTaoInEmission::<Test>::get(netuid_2),
//             2 * (emission / 6),
//             100,
//         );
//         close(
//             SubnetTaoInEmission::<Test>::get(netuid_3),
//             3 * (emission / 6),
//             100,
//         );
//     });
// }

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::migration::test_migrate_subnet_volume --exact --show-output
#[test]
fn test_migrate_subnet_volume() {
    new_test_ext(1).execute_with(|| {
        // Setup initial state
        let netuid_1: u16 = 1;
        add_network(netuid_1, 1, 0);

        // SubnetValue for netuid 1 key
        let old_key: [u8; 34] = hex_literal::hex!(
            "658faa385070e074c85bf6b568cf05553c3226e141696000b4b239c65bc2b2b40100"
        );

        // Old value in u64 format
        let old_value: u64 = 123_456_789_000_u64;
        put::<u64>(&old_key, &old_value); // Store as u64

        // Ensure it is stored as `u64`
        assert_eq!(get::<u64>(&old_key), Some(old_value));

        // Run migration
        crate::migrations::migrate_subnet_volume::migrate_subnet_volume::<Test>();

        // Verify the value is now stored as `u128`
        let new_value: Option<u128> = get(&old_key);
        let new_value_as_subnet_volume = SubnetVolume::<Test>::get(netuid_1);
        assert_eq!(new_value, Some(old_value as u128));
        assert_eq!(new_value_as_subnet_volume, old_value as u128);

        // Ensure migration does not break when running twice
        let weight_second_run =
            crate::migrations::migrate_subnet_volume::migrate_subnet_volume::<Test>();

        // Verify the value is still stored as `u128`
        let new_value: Option<u128> = get(&old_key);
        assert_eq!(new_value, Some(old_value as u128));
    });
}

#[test]
fn test_migrate_set_first_emission_block_number() {
    new_test_ext(1).execute_with(|| {
    let netuids: [u16; 3] = [1, 2, 3];
    let block_number = 100;
    for netuid in netuids.iter() {
        add_network(*netuid, 1, 0);
    }
    run_to_block(block_number);
    let weight = crate::migrations::migrate_set_first_emission_block_number::migrate_set_first_emission_block_number::<Test>();

    let expected_weight: Weight = <Test as Config>::DbWeight::get().reads(3) + <Test as Config>::DbWeight::get().writes(netuids.len() as u64);
    assert_eq!(weight, expected_weight);

    assert_eq!(FirstEmissionBlockNumber::<Test>::get(0), None);
    for netuid in netuids.iter() {
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), Some(block_number));
    }
});
}

#[test]
fn test_migrate_set_subtoken_enable() {
    new_test_ext(1).execute_with(|| {
        let netuids: [u16; 3] = [1, 2, 3];
        let block_number = 100;
        for netuid in netuids.iter() {
            add_network(*netuid, 1, 0);
        }

        let new_netuid = 4;
        add_network_without_emission_block(new_netuid, 1, 0);

        let weight =
            crate::migrations::migrate_set_subtoken_enabled::migrate_set_subtoken_enabled::<Test>();

        let expected_weight: Weight = <Test as Config>::DbWeight::get().reads(1)
            + <Test as Config>::DbWeight::get().writes(netuids.len() as u64 + 2);
        assert_eq!(weight, expected_weight);

        for netuid in netuids.iter() {
            assert!(SubtokenEnabled::<Test>::get(netuid));
        }
        assert!(!SubtokenEnabled::<Test>::get(new_netuid));
    });
}

#[test]
fn test_migrate_remove_zero_total_hotkey_alpha() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_remove_zero_total_hotkey_alpha";
        let netuid = 1u16;

        let hotkey_zero = U256::from(100u64);
        let hotkey_nonzero = U256::from(101u64);

        // Insert one zero-alpha entry and one non-zero entry
        TotalHotkeyAlpha::<Test>::insert(hotkey_zero, netuid, 0u64);
        TotalHotkeyAlpha::<Test>::insert(hotkey_nonzero, netuid, 123u64);

        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey_zero, netuid), 0u64);
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey_nonzero, netuid), 123u64);

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet."
        );

        let weight = crate::migrations::migrate_remove_zero_total_hotkey_alpha::migrate_remove_zero_total_hotkey_alpha::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as run."
        );

        assert!(
            !TotalHotkeyAlpha::<Test>::contains_key(hotkey_zero, netuid),
            "Zero-alpha entry should have been removed."
        );

        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey_nonzero, netuid), 123u64);

        assert!(
            !weight.is_zero(),
            "Migration weight should be non-zero."
        );
    });
}

#[test]
fn test_migrate_revealed_commitments() {
    new_test_ext(1).execute_with(|| {
        // --------------------------------
        // Step 1: Simulate Old Storage Entries
        // --------------------------------
        const MIGRATION_NAME: &str = "migrate_revealed_commitments_v2";

        // Pallet prefix == twox_128("Commitments")
        let pallet_prefix = twox_128("Commitments".as_bytes());
        // Storage item prefix == twox_128("RevealedCommitments")
        let storage_prefix = twox_128("RevealedCommitments".as_bytes());

        // Example keys for the DoubleMap:
        //   Key1 (netuid) uses Identity (no hash)
        //   Key2 (account) uses Twox64Concat
        let netuid: u16 = 123;
        let account_id: u64 = 999; // Or however your test `AccountId` is represented

        // Construct the full storage key for `RevealedCommitments(netuid, account_id)`
        let mut storage_key = Vec::new();
        storage_key.extend_from_slice(&pallet_prefix);
        storage_key.extend_from_slice(&storage_prefix);

        // Identity for netuid => no hashing, just raw encode
        storage_key.extend_from_slice(&netuid.encode());

        // Twox64Concat for account
        let account_hashed = Twox64Concat::hash(&account_id.encode());
        storage_key.extend_from_slice(&account_hashed);

        // Simulate an old value we might have stored:
        // For example, the old type was `RevealedData<Balance, ...>`
        // We'll just store a random encoded value for demonstration
        let old_value = (vec![1, 2, 3, 4], 42u64);
        put_raw(&storage_key, &old_value.encode());

        // Confirm the storage value is set
        let stored_value = get_raw(&storage_key).expect("Expected to get a value");
        let decoded_value = <(Vec<u8>, u64)>::decode(&mut &stored_value[..])
            .expect("Failed to decode the old revealed commitments");
        assert_eq!(decoded_value, old_value);

        // Also confirm that the migration has NOT run yet
        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // --------------------------------
        // Step 2: Run the Migration
        // --------------------------------
        let weight = crate::migrations::migrate_upgrade_revealed_commitments::migrate_upgrade_revealed_commitments::<Test>();

        // Migration should be marked as run
        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should now be marked as run"
        );

        // --------------------------------
        // Step 3: Verify Migration Effects
        // --------------------------------
        // The old key/value should be removed
        let stored_value_after = get_raw(&storage_key);
        assert!(
            stored_value_after.is_none(),
            "Old storage entry should be cleared"
        );

        // Weight returned should be > 0 (some cost was incurred clearing storage)
        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[test]
fn test_migrate_remove_total_hotkey_coldkey_stakes_this_interval() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_remove_total_hotkey_coldkey_stakes_this_interval";

        let pallet_name = twox_128(b"SubtensorModule");
        let storage_name = twox_128(b"TotalHotkeyColdkeyStakesThisInterval");
        let prefix = [pallet_name, storage_name].concat();

        // Set up 200 000 entries to be deleted.
        for i in 0..200_000{
            let hotkey = U256::from(i as u64);
            let coldkey = U256::from(i as u64);
            let key = [prefix.clone(), hotkey.encode(), coldkey.encode()].concat();
            let value = (100 + i, 200 + i);
            put_raw(&key, &value.encode());
        }

        assert!(frame_support::storage::unhashed::contains_prefixed_key(&prefix), "Entries should exist before migration.");
        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet."
        );

        // Run migration
        let weight = crate::migrations::migrate_remove_total_hotkey_coldkey_stakes_this_interval::migrate_remove_total_hotkey_coldkey_stakes_this_interval::<Test>();

        assert!(!frame_support::storage::unhashed::contains_prefixed_key(&prefix), "All entries should have been removed.");
        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as run."
        );
        assert!(!weight.is_zero(),"Migration weight should be non-zero.");
    });
}
fn test_migrate_remove_last_hotkey_coldkey_emission_on_netuid() {
    const MIGRATION_NAME: &str = "migrate_remove_last_hotkey_coldkey_emission_on_netuid";
    let pallet_name = "SubtensorModule";
    let storage_name = "LastHotkeyColdkeyEmissionOnNetuid";
    let migration =  crate::migrations::migrate_orphaned_storage_items::remove_last_hotkey_coldkey_emission_on_netuid::<Test>;

    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );
}
#[test]
fn test_migrate_remove_subnet_alpha_emission_sell() {
    const MIGRATION_NAME: &str = "migrate_remove_subnet_alpha_emission_sell";
    let pallet_name = "SubtensorModule";
    let storage_name = "SubnetAlphaEmissionSell";
    let migration =
        crate::migrations::migrate_orphaned_storage_items::remove_subnet_alpha_emission_sell::<Test>;

    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );
}

#[test]
fn test_migrate_remove_neurons_to_prune_at_next_epoch() {
    const MIGRATION_NAME: &str = "migrate_remove_neurons_to_prune_at_next_epoch";
    let pallet_name = "SubtensorModule";
    let storage_name = "NeuronsToPruneAtNextEpoch";
    let migration =
        crate::migrations::migrate_orphaned_storage_items::remove_neurons_to_prune_at_next_epoch::<
            Test,
        >;

    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );
}

#[test]
fn test_migrate_remove_total_stake_at_dynamic() {
    const MIGRATION_NAME: &str = "migrate_remove_total_stake_at_dynamic";
    let pallet_name = "SubtensorModule";
    let storage_name = "TotalStakeAtDynamic";
    let migration =
        crate::migrations::migrate_orphaned_storage_items::remove_total_stake_at_dynamic::<Test>;

    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );
}

#[test]
fn test_migrate_remove_subnet_name() {
    const MIGRATION_NAME: &str = "migrate_remove_subnet_name";
    let pallet_name = "SubtensorModule";
    let storage_name = "SubnetName";
    let migration = crate::migrations::migrate_orphaned_storage_items::remove_subnet_name::<Test>;

    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );
}

#[test]
fn test_migrate_remove_network_min_allowed_uids() {
    const MIGRATION_NAME: &str = "migrate_remove_network_min_allowed_uids";
    let pallet_name = "SubtensorModule";
    let storage_name = "NetworkMinAllowedUids";
    let migration =
        crate::migrations::migrate_orphaned_storage_items::remove_network_min_allowed_uids::<Test>;

    test_remove_storage_item(MIGRATION_NAME, pallet_name, storage_name, migration, 1);
}

#[test]
fn test_migrate_remove_dynamic_block() {
    const MIGRATION_NAME: &str = "migrate_remove_dynamic_block";
    let pallet_name = "SubtensorModule";
    let storage_name = "DynamicBlock";
    let migration = crate::migrations::migrate_orphaned_storage_items::remove_dynamic_block::<Test>;

    test_remove_storage_item(MIGRATION_NAME, pallet_name, storage_name, migration, 1);
}

#[allow(clippy::arithmetic_side_effects)]
fn test_remove_storage_item<F: FnOnce() -> Weight>(
    migration_name: &'static str,
    pallet_name: &'static str,
    storage_name: &'static str,
    migration: F,
    test_entries_number: i32,
) {
    new_test_ext(1).execute_with(|| {
        let pallet_name = twox_128(pallet_name.as_bytes());
        let storage_name = twox_128(storage_name.as_bytes());
        let prefix = [pallet_name, storage_name].concat();

        // Set up entries to be deleted.
        for i in 0..test_entries_number {
            let hotkey = U256::from(i as u64);
            let coldkey = U256::from(i as u64);
            let key = [prefix.clone(), hotkey.encode(), coldkey.encode()].concat();
            let value = (100 + i, 200 + i);
            put_raw(&key, &value.encode());
        }

        assert!(
            frame_support::storage::unhashed::contains_prefixed_key(&prefix),
            "Entries should exist before migration."
        );
        assert!(
            !HasMigrationRun::<Test>::get(migration_name.as_bytes().to_vec()),
            "Migration should not have run yet."
        );

        // Run migration
        let weight = migration();

        assert!(
            !frame_support::storage::unhashed::contains_prefixed_key(&prefix),
            "All entries should have been removed."
        );
        assert!(
            HasMigrationRun::<Test>::get(migration_name.as_bytes().to_vec()),
            "Migration should be marked as run."
        );
        assert!(!weight.is_zero(), "Migration weight should be non-zero.");
    });
}

#[test]
fn test_migrate_remove_commitments_rate_limit() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entry
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_remove_commitments_rate_limit";

        // Build the raw storage key: twox128("Commitments") ++ twox128("RateLimit")
        let pallet_prefix = twox_128("Commitments".as_bytes());
        let storage_prefix = twox_128("RateLimit".as_bytes());

        let mut key = Vec::new();
        key.extend_from_slice(&pallet_prefix);
        key.extend_from_slice(&storage_prefix);

        let original_value: u64 = 123;
        put_raw(&key, &original_value.encode());

        let stored_before = get_raw(&key).expect("Expected RateLimit to exist");
        assert_eq!(
            u64::decode(&mut &stored_before[..]).expect("Failed to decode RateLimit"),
            original_value
        );

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // ------------------------------
        // Step 2: Run the Migration
        // ------------------------------
        let weight = crate::migrations::migrate_remove_commitments_rate_limit::
            migrate_remove_commitments_rate_limit::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as completed"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------
        assert!(
            get_raw(&key).is_none(),
            "RateLimit storage should have been cleared"
        );

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[test]
fn test_migrate_clear_root_epoch_values() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entry
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_clear_root_epoch_values";

        let root_netuid = Pallet::<Test>::get_root_netuid();

        SubnetworkN::<Test>::insert(root_netuid, 0);
        Tempo::<Test>::insert(root_netuid, 0);
        ActivityCutoff::<Test>::insert(root_netuid, 0);
        MaxAllowedValidators::<Test>::insert(root_netuid, 0);
        SubnetOwnerHotkey::<Test>::insert(root_netuid, U256::from(1));

        Kappa::<Test>::insert(root_netuid, 0);
        BondsPenalty::<Test>::insert(root_netuid, 0);
        Yuma3On::<Test>::insert(root_netuid, false);
        Rank::<Test>::insert(root_netuid, Vec::<u16>::new());
        Trust::<Test>::insert(root_netuid, Vec::<u16>::new());

        Active::<Test>::insert(root_netuid, Vec::<bool>::new());
        Emission::<Test>::insert(root_netuid, Vec::<u64>::new());
        Consensus::<Test>::insert(root_netuid, Vec::<u16>::new());
        Incentive::<Test>::insert(root_netuid, Vec::<u16>::new());
        Dividends::<Test>::insert(root_netuid, Vec::<u16>::new());

        LastUpdate::<Test>::insert(root_netuid, Vec::<u64>::new());
        PruningScores::<Test>::insert(root_netuid, Vec::<u16>::new());
        ValidatorTrust::<Test>::insert(root_netuid, Vec::<u16>::new());
        ValidatorPermit::<Test>::insert(root_netuid, Vec::<bool>::new());
        StakeWeight::<Test>::insert(root_netuid, Vec::<u16>::new());

        Bonds::<Test>::insert(root_netuid, root_netuid, Vec::<(u16, u16)>::new());
        Keys::<Test>::insert(root_netuid, root_netuid, U256::from(1));
        BlockAtRegistration::<Test>::insert(root_netuid, root_netuid, 0);

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        assert!(SubnetworkN::<Test>::contains_key(root_netuid));
        assert!(Tempo::<Test>::contains_key(root_netuid));
        assert!(ActivityCutoff::<Test>::contains_key(root_netuid));
        assert!(MaxAllowedValidators::<Test>::contains_key(root_netuid));
        assert!(SubnetOwnerHotkey::<Test>::contains_key(root_netuid));

        assert!(Kappa::<Test>::contains_key(root_netuid));
        assert!(BondsPenalty::<Test>::contains_key(root_netuid));
        assert!(Yuma3On::<Test>::contains_key(root_netuid));
        assert!(Rank::<Test>::contains_key(root_netuid));
        assert!(Trust::<Test>::contains_key(root_netuid));

        assert!(Active::<Test>::contains_key(root_netuid));
        assert!(Emission::<Test>::contains_key(root_netuid));
        assert!(Consensus::<Test>::contains_key(root_netuid));
        assert!(Incentive::<Test>::contains_key(root_netuid));
        assert!(Dividends::<Test>::contains_key(root_netuid));

        assert!(LastUpdate::<Test>::contains_key(root_netuid));
        assert!(PruningScores::<Test>::contains_key(root_netuid));
        assert!(ValidatorTrust::<Test>::contains_key(root_netuid));
        assert!(ValidatorPermit::<Test>::contains_key(root_netuid));
        assert!(StakeWeight::<Test>::contains_key(root_netuid));

        assert!(Bonds::<Test>::contains_prefix(root_netuid));
        assert!(Keys::<Test>::contains_prefix(root_netuid));
        assert!(BlockAtRegistration::<Test>::contains_prefix(root_netuid));

        // ------------------------------
        // Step 2: Run the Migration
        // ------------------------------
        let weight =
            crate::migrations::migrate_clear_root_epoch_values::migrate_clear_root_epoch_values::<
                Test,
            >();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as completed"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------
        assert!(!SubnetworkN::<Test>::contains_key(root_netuid));
        assert!(!Tempo::<Test>::contains_key(root_netuid));
        assert!(!ActivityCutoff::<Test>::contains_key(root_netuid));
        assert!(!MaxAllowedValidators::<Test>::contains_key(root_netuid));
        assert!(!SubnetOwnerHotkey::<Test>::contains_key(root_netuid));

        assert!(!Kappa::<Test>::contains_key(root_netuid));
        assert!(!BondsPenalty::<Test>::contains_key(root_netuid));
        assert!(!Yuma3On::<Test>::contains_key(root_netuid));
        assert!(!Rank::<Test>::contains_key(root_netuid));
        assert!(!Trust::<Test>::contains_key(root_netuid));

        assert!(!Active::<Test>::contains_key(root_netuid));
        assert!(!Emission::<Test>::contains_key(root_netuid));
        assert!(!Consensus::<Test>::contains_key(root_netuid));
        assert!(!Incentive::<Test>::contains_key(root_netuid));
        assert!(!Dividends::<Test>::contains_key(root_netuid));

        assert!(!LastUpdate::<Test>::contains_key(root_netuid));
        assert!(!PruningScores::<Test>::contains_key(root_netuid));
        assert!(!ValidatorTrust::<Test>::contains_key(root_netuid));
        assert!(!ValidatorPermit::<Test>::contains_key(root_netuid));
        assert!(!StakeWeight::<Test>::contains_key(root_netuid));

        assert!(!Bonds::<Test>::contains_prefix(root_netuid));
        assert!(!Keys::<Test>::contains_prefix(root_netuid));
        assert!(!BlockAtRegistration::<Test>::contains_prefix(root_netuid));

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}
