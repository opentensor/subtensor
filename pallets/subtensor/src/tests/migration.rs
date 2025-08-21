#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use alloc::collections::BTreeMap;
use approx::assert_abs_diff_eq;
use codec::{Decode, Encode};
use frame_support::{
    StorageHasher, Twox64Concat, assert_ok,
    storage::unhashed::{get, get_raw, put, put_raw},
    storage_alias,
    traits::{StorageInstance, StoredMap},
    weights::Weight,
};

use crate::migrations::migrate_storage;
use frame_system::Config;
use pallet_drand::types::RoundNumber;
use scale_info::prelude::collections::VecDeque;
use sp_core::{H256, U256, crypto::Ss58Codec};
use sp_io::hashing::twox_128;
use sp_runtime::traits::Zero;
use substrate_fixed::types::I96F32;
use substrate_fixed::types::extra::U2;
use subtensor_runtime_common::TaoCurrency;

#[allow(clippy::arithmetic_side_effects)]
fn close(value: u64, target: u64, eps: u64) {
    assert!(
        (value as i64 - target as i64).abs() < eps as i64,
        "Assertion failed: value = {value}, target = {target}, eps = {eps}"
    )
}

#[test]
fn test_initialise_ti() {
    use frame_support::traits::OnRuntimeUpgrade;

    new_test_ext(1).execute_with(|| {
        pallet_balances::TotalIssuance::<Test>::put(1000);
        crate::SubnetTAO::<Test>::insert(NetUid::from(1), TaoCurrency::from(100));
        crate::SubnetTAO::<Test>::insert(NetUid::from(2), TaoCurrency::from(5));

        // Ensure values are NOT initialized prior to running migration
        assert!(crate::TotalIssuance::<Test>::get().is_zero());
		assert!(crate::TotalStake::<Test>::get().is_zero());

        crate::migrations::migrate_init_total_issuance::initialise_total_issuance::Migration::<Test>::on_runtime_upgrade();

        // Ensure values were initialized correctly
		assert_eq!(crate::TotalStake::<Test>::get(), TaoCurrency::from(105));
        assert_eq!(
            crate::TotalIssuance::<Test>::get(), TaoCurrency::from(105 + 1000)
        );
    });
}

#[test]
fn test_migration_transfer_nets_to_foundation() {
    new_test_ext(1).execute_with(|| {
        // Create subnet 1
        add_network(1.into(), 1, 0);
        // Create subnet 11
        add_network(11.into(), 1, 0);

        log::info!("{:?}", SubtensorModule::get_subnet_owner(1.into()));
        //assert_eq!(SubtensorModule::<T>::get_subnet_owner(1), );

        // Run the migration to transfer ownership
        let hex =
            hex_literal::hex!["feabaafee293d3b76dae304e2f9d885f77d2b17adab9e17e921b321eccd61c77"];
        crate::migrations::migrate_transfer_ownership_to_foundation::migrate_transfer_ownership_to_foundation::<Test>(hex);

        log::info!("new owner: {:?}", SubtensorModule::get_subnet_owner(1.into()));
    })
}

#[test]
fn test_migration_delete_subnet_3() {
    new_test_ext(1).execute_with(|| {
        // Create subnet 3
        add_network(3.into(), 1, 0);
        assert!(SubtensorModule::if_subnet_exist(3.into()));

        // Run the migration to transfer ownership
        crate::migrations::migrate_delete_subnet_3::migrate_delete_subnet_3::<Test>();

        assert!(!SubtensorModule::if_subnet_exist(3.into()));
    })
}

#[test]
fn test_migration_delete_subnet_21() {
    new_test_ext(1).execute_with(|| {
        // Create subnet 21
        add_network(21.into(), 1, 0);
        assert!(SubtensorModule::if_subnet_exist(21.into()));

        // Run the migration to transfer ownership
        crate::migrations::migrate_delete_subnet_21::migrate_delete_subnet_21::<Test>();

        assert!(!SubtensorModule::if_subnet_exist(21.into()));
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

        let netuid = NetUid::from(1);
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
        let netuid_1 = NetUid::from(1);
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
    let netuids: [NetUid; 3] = [1.into(), 2.into(), 3.into()];
    let block_number = 100;
    for netuid in netuids.iter() {
        add_network(*netuid, 1, 0);
    }
    run_to_block(block_number);
    let weight = crate::migrations::migrate_set_first_emission_block_number::migrate_set_first_emission_block_number::<Test>();

    let expected_weight: Weight = <Test as Config>::DbWeight::get().reads(3) + <Test as Config>::DbWeight::get().writes(netuids.len() as u64);
    assert_eq!(weight, expected_weight);

    assert_eq!(FirstEmissionBlockNumber::<Test>::get(NetUid::ROOT), None);
    for netuid in netuids.iter() {
        assert_eq!(FirstEmissionBlockNumber::<Test>::get(netuid), Some(block_number));
    }
});
}

#[test]
fn test_migrate_set_subtoken_enable() {
    new_test_ext(1).execute_with(|| {
        let netuids: [NetUid; 3] = [1.into(), 2.into(), 3.into()];
        let block_number = 100;
        for netuid in netuids.iter() {
            add_network(*netuid, 1, 0);
        }

        let new_netuid = NetUid::from(4);
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
        let netuid = NetUid::from(1u16);

        let hotkey_zero = U256::from(100u64);
        let hotkey_nonzero = U256::from(101u64);

        // Insert one zero-alpha entry and one non-zero entry
        TotalHotkeyAlpha::<Test>::insert(hotkey_zero, netuid, AlphaCurrency::ZERO);
        TotalHotkeyAlpha::<Test>::insert(hotkey_nonzero, netuid, AlphaCurrency::from(123));

        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey_zero, netuid), AlphaCurrency::ZERO);
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey_nonzero, netuid), AlphaCurrency::from(123));

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

        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey_nonzero, netuid), AlphaCurrency::from(123));

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
        let netuid = NetUid::from(123);
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
fn test_migrate_fix_root_subnet_tao() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_fix_root_subnet_tao";

        let mut expected_total_stake = 0;
        // Seed some hotkeys with some fake stake.
        for i in 0..100_000 {
            Owner::<Test>::insert(U256::from(U256::from(i)), U256::from(i + 1_000_000));
            let stake = i + 1_000_000;
            TotalHotkeyAlpha::<Test>::insert(
                U256::from(U256::from(i)),
                NetUid::ROOT,
                AlphaCurrency::from(stake),
            );
            expected_total_stake += stake;
        }

        assert_eq!(SubnetTAO::<Test>::get(NetUid::ROOT), TaoCurrency::ZERO);
        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // Run the migration
        let weight =
            crate::migrations::migrate_fix_root_subnet_tao::migrate_fix_root_subnet_tao::<Test>();

        // Verify the migration ran correctly
        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as run"
        );
        assert!(!weight.is_zero(), "Migration weight should be non-zero");
        assert_eq!(
            SubnetTAO::<Test>::get(NetUid::ROOT),
            expected_total_stake.into()
        );
    });
}

#[test]
fn test_migrate_subnet_symbols() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_subnet_symbols";

        // Create 100 subnets
        for i in 0..100 {
            add_network(i.into(), 1, 0);
        }

        // Shift some symbols
        TokenSymbol::<Test>::insert(
            NetUid::from(21),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(142)),
        );
        TokenSymbol::<Test>::insert(
            NetUid::from(42),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(184)),
        );
        TokenSymbol::<Test>::insert(
            NetUid::from(83),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(242)),
        );
        TokenSymbol::<Test>::insert(
            NetUid::from(99),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(284)),
        );

        // Run the migration
        let weight = crate::migrations::migrate_subnet_symbols::migrate_subnet_symbols::<Test>();

        // Check that the symbols have been corrected
        assert_eq!(
            TokenSymbol::<Test>::get(NetUid::from(21)),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(21))
        );
        assert_eq!(
            TokenSymbol::<Test>::get(NetUid::from(42)),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(42))
        );
        assert_eq!(
            TokenSymbol::<Test>::get(NetUid::from(83)),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(83))
        );
        assert_eq!(
            TokenSymbol::<Test>::get(NetUid::from(99)),
            SubtensorModule::get_symbol_for_subnet(NetUid::from(99))
        );

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[test]
fn test_migrate_set_registration_enable() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_set_registration_enable";

        // Create 3 subnets
        let netuids: [NetUid; 3] = [1.into(), 2.into(), 3.into()];
        for netuid in netuids.iter() {
            add_network(*netuid, 1, 0);
            // Set registration to false to simulate the need for migration
            SubtensorModule::set_network_registration_allowed(*netuid, false);
            SubtensorModule::set_network_pow_registration_allowed(*netuid, false);
        }

        // Sanity check: registration is disabled before migration
        for netuid in netuids.iter() {
            assert!(!SubtensorModule::get_network_registration_allowed(*netuid));
            assert!(!SubtensorModule::get_network_pow_registration_allowed(
                *netuid
            ));
        }

        // Run the migration
        let weight =
            crate::migrations::migrate_set_registration_enable::migrate_set_registration_enable::<
                Test,
            >();

        // After migration, regular registration should be enabled for all subnets except root
        for netuid in netuids.iter() {
            assert!(SubtensorModule::get_network_registration_allowed(*netuid));
            assert!(!SubtensorModule::get_network_pow_registration_allowed(
                *netuid
            ));
        }

        // Migration should be marked as run
        assert!(HasMigrationRun::<Test>::get(
            MIGRATION_NAME.as_bytes().to_vec()
        ));

        // Weight should be non-zero
        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[test]
fn test_migrate_set_nominator_min_stake() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_set_nominator_min_stake";

        let min_nomination_initial = 100_000_000;
        let min_nomination_migrated = 10_000_000;
        NominatorMinRequiredStake::<Test>::set(min_nomination_initial);

        assert_eq!(
            NominatorMinRequiredStake::<Test>::get(),
            min_nomination_initial
        );
        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // Run the migration
        let weight =
            crate::migrations::migrate_set_nominator_min_stake::migrate_set_nominator_min_stake::<
                Test,
            >();

        // Verify the migration ran correctly
        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as run"
        );
        assert!(!weight.is_zero(), "Migration weight should be non-zero");
        assert_eq!(
            NominatorMinRequiredStake::<Test>::get(),
            min_nomination_migrated
        );
    });
}

#[test]
fn test_migrate_crv3_commits_add_block() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // 0. Constants / helpers
        // ------------------------------
        const MIG_NAME: &[u8] = b"crv3_commits_add_block_v1";
        let netuid = NetUid::from(99);
        let epoch: u64 = 7;
        let tempo: u16 = 360;

        // ------------------------------
        // 1. Create a network so helper can compute first‑block
        // ------------------------------
        add_network(netuid, tempo, 0);

        // ------------------------------
        // 2. Simulate OLD storage (3‑tuple)
        // ------------------------------
        let who: U256 = U256::from(0xdeadbeef_u64);
        let ciphertext: BoundedVec<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>> =
            vec![1u8, 2, 3].try_into().unwrap();
        let round: RoundNumber = 42;

        let old_queue: VecDeque<_> = VecDeque::from(vec![(who, ciphertext.clone(), round)]);

        CRV3WeightCommits::<Test>::insert(netuid, epoch, old_queue.clone());

        // Sanity: entry decodes under old alias
        assert_eq!(CRV3WeightCommits::<Test>::get(netuid, epoch), old_queue);

        assert!(
            !HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag should be false before run"
        );

        // ------------------------------
        // 3. Run migration
        // ------------------------------
        let w = crate::migrations::migrate_crv3_commits_add_block::migrate_crv3_commits_add_block::<
            Test,
        >();
        assert!(!w.is_zero(), "weight must be non-zero");

        // ------------------------------
        // 4. Verify results
        // ------------------------------
        assert!(
            HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag not set"
        );

        // Old storage must be empty (drained)
        assert!(
            CRV3WeightCommits::<Test>::get(netuid, epoch).is_empty(),
            "old queue should have been drained"
        );

        let new_q = CRV3WeightCommitsV2::<Test>::get(netuid, epoch);
        assert_eq!(new_q.len(), 1, "exactly one migrated element expected");

        let (who2, commit_block, cipher2, round2) = new_q.front().cloned().unwrap();
        assert_eq!(who2, who);
        assert_eq!(cipher2, ciphertext);
        assert_eq!(round2, round);

        let expected_block = Pallet::<Test>::get_first_block_of_epoch(netuid, epoch);
        assert_eq!(
            commit_block, expected_block,
            "commit_block should equal first block of epoch key"
        );
    });
}

#[test]
fn test_migrate_disable_commit_reveal() {
    const MIG_NAME: &[u8] = b"disable_commit_reveal_v1";
    let netuids = [NetUid::from(1), NetUid::from(2), NetUid::from(42)];

    // ---------------------------------------------------------------------
    // 1. build initial state ─ all nets enabled
    // ---------------------------------------------------------------------
    new_test_ext(1).execute_with(|| {
        for (i, netuid) in netuids.iter().enumerate() {
            add_network(*netuid, 5u16 + i as u16, 0);
            CommitRevealWeightsEnabled::<Test>::insert(*netuid, true);
        }
        assert!(
            !HasMigrationRun::<Test>::get(MIG_NAME),
            "migration flag should be unset before run"
        );

        // -----------------------------------------------------------------
        // 2. run migration
        // -----------------------------------------------------------------
        let w = crate::migrations::migrate_disable_commit_reveal::migrate_disable_commit_reveal::<
            Test,
        >();

        assert!(
            HasMigrationRun::<Test>::get(MIG_NAME),
            "migration flag not set"
        );

        // -----------------------------------------------------------------
        // 3. verify every netuid is now disabled and only one value exists
        // -----------------------------------------------------------------
        for netuid in netuids {
            assert!(
                !CommitRevealWeightsEnabled::<Test>::get(netuid),
                "commit-reveal should be disabled for netuid {netuid}"
            );
        }

        // There should be no stray keys
        let collected: Vec<_> = CommitRevealWeightsEnabled::<Test>::iter().collect();
        assert_eq!(collected.len(), netuids.len(), "unexpected key count");
        for (k, v) in collected {
            assert!(!v, "found an enabled flag after migration for netuid {k}");
        }

        // -----------------------------------------------------------------
        // 4. running again should be a no-op
        // -----------------------------------------------------------------
        let w2 = crate::migrations::migrate_disable_commit_reveal::migrate_disable_commit_reveal::<
            Test,
        >();
        assert_eq!(
            w2,
            <Test as Config>::DbWeight::get().reads(1),
            "second run should read the flag and do nothing else"
        );
    });
}

#[test]
fn test_migrate_commit_reveal_settings() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_commit_reveal_settings";

        // Set up some networks first
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        
        // Add networks to simulate existing networks
        add_network(netuid1.into(), 1, 0);
        add_network(netuid2.into(), 1, 0);

        // Ensure the storage items use default values initially (but aren't explicitly set)
        // Since these are ValueQuery storage items, they return defaults even when not set
        assert_eq!(RevealPeriodEpochs::<Test>::get(NetUid::from(netuid1)), 1u64);
        assert_eq!(RevealPeriodEpochs::<Test>::get(NetUid::from(netuid2)), 1u64);
        assert!(CommitRevealWeightsEnabled::<Test>::get(NetUid::from(netuid1)));
        assert!(CommitRevealWeightsEnabled::<Test>::get(NetUid::from(netuid2)));

        // Check migration hasn't run
        assert!(!HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()));

        // Run migration
        let weight = crate::migrations::migrate_commit_reveal_settings::migrate_commit_reveal_settings::<Test>();

        // Check migration has been marked as run
        assert!(HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()));

        // Verify RevealPeriodEpochs was set correctly
        assert_eq!(RevealPeriodEpochs::<Test>::get(NetUid::from(netuid1)), 1u64);
        assert_eq!(RevealPeriodEpochs::<Test>::get(NetUid::from(netuid2)), 1u64);

        // Verify CommitRevealWeightsEnabled was set correctly
        assert!(CommitRevealWeightsEnabled::<Test>::get(NetUid::from(netuid1)));
        assert!(CommitRevealWeightsEnabled::<Test>::get(NetUid::from(netuid2)));

        // Check that weight calculation is correct
        // 1 read for migration check + 2 reads for network iteration + 2 * 2 writes for storage + 1 write for migration flag
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads(1 + 2) + <Test as frame_system::Config>::DbWeight::get().writes(2 * 2 + 1);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_migrate_commit_reveal_settings_already_run() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_commit_reveal_settings";
        
        // Mark migration as already run
        HasMigrationRun::<Test>::insert(MIGRATION_NAME.as_bytes().to_vec(), true);

        // Run migration
        let weight = crate::migrations::migrate_commit_reveal_settings::migrate_commit_reveal_settings::<Test>();

        // Should only have read weight for checking migration status
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads(1);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_migrate_commit_reveal_settings_no_networks() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_commit_reveal_settings";

        // Check migration hasn't run
        assert!(!HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()));

        // Run migration
        let weight = crate::migrations::migrate_commit_reveal_settings::migrate_commit_reveal_settings::<Test>();

        // Check migration has been marked as run
        assert!(HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()));

        // Check that weight calculation is correct (no networks, so no additional reads/writes)
        // 1 read for migration check + 0 reads for networks + 0 writes for storage + 1 write for migration flag
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads(1) + <Test as frame_system::Config>::DbWeight::get().writes(1);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_migrate_commit_reveal_settings_multiple_networks() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_commit_reveal_settings";

        // Set up multiple networks
        let netuids = vec![1u16, 2u16, 3u16, 10u16, 42u16];
        
        for netuid in &netuids {
            add_network((*netuid).into(), 1, 0);
        }

        // Run migration
        let weight = crate::migrations::migrate_commit_reveal_settings::migrate_commit_reveal_settings::<Test>();

        // Verify all networks have correct settings
        for netuid in &netuids {
            assert_eq!(RevealPeriodEpochs::<Test>::get(NetUid::from(*netuid)), 1u64);
            assert!(CommitRevealWeightsEnabled::<Test>::get(NetUid::from(*netuid)));
        }

        // Check migration has been marked as run
        assert!(HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()));

        // Check that weight calculation is correct
        let network_count = netuids.len() as u64;
        let expected_weight = <Test as frame_system::Config>::DbWeight::get().reads(1 + network_count) 
            + <Test as frame_system::Config>::DbWeight::get().writes(network_count * 2 + 1);
        assert_eq!(weight, expected_weight);
    });
}

#[test]
fn test_migrate_commit_reveal_settings_values_access() {
    new_test_ext(1).execute_with(|| {
        let netuid: u16 = 1;
        add_network(netuid.into(), 1, 0);

        // Run migration
        crate::migrations::migrate_commit_reveal_settings::migrate_commit_reveal_settings::<Test>();

        // Test that we can access the values using the pallet functions
        assert_eq!(SubtensorModule::get_reveal_period(NetUid::from(netuid)), 1u64);
        
        // Test direct storage access
        assert_eq!(RevealPeriodEpochs::<Test>::get(NetUid::from(netuid)), 1u64);
        assert!(CommitRevealWeightsEnabled::<Test>::get(NetUid::from(netuid)));
    });
}