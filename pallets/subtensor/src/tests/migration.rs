#![allow(
    unused,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]

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
use substrate_fixed::types::extra::U2;
use substrate_fixed::types::{I96F32, U64F64};
use subtensor_runtime_common::{NetUidStorageIndex, TaoCurrency};

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
        //assert_eq!(SubtensorModule::<Test>::get_subnet_owner(1), );

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
fn test_migrate_network_last_registered() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entry
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_network_last_registered";

        let pallet_name = "SubtensorModule";
        let storage_name = "NetworkLastRegistered";
        let pallet_name_hash = twox_128(pallet_name.as_bytes());
        let storage_name_hash = twox_128(storage_name.as_bytes());
        let prefix = [pallet_name_hash, storage_name_hash].concat();

        let mut full_key = prefix.clone();

        let original_value: u64 = 123;
        put_raw(&full_key, &original_value.encode());

        let stored_before = get_raw(&full_key).expect("Expected RateLimit to exist");
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
        let weight = crate::migrations::migrate_rate_limiting_last_blocks::
        migrate_obsolete_rate_limiting_last_blocks_storage::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as completed"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------

        assert_eq!(
            SubtensorModule::get_network_last_lock_block(),
            original_value
        );
        assert_eq!(
            get_raw(&full_key),
            None,
            "RateLimit storage should have been cleared"
        );

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[allow(deprecated)]
#[test]
fn test_migrate_last_block_tx() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entry
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_last_tx_block";

        let test_account: U256 = U256::from(1);
        let original_value: u64 = 123;

        LastTxBlock::<Test>::insert(test_account, original_value);

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // ------------------------------
        // Step 2: Run the Migration
        // ------------------------------
        let weight = crate::migrations::migrate_rate_limiting_last_blocks::
        migrate_obsolete_rate_limiting_last_blocks_storage::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as completed"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------

        assert_eq!(
            SubtensorModule::get_last_tx_block(&test_account),
            original_value
        );
        assert!(
            !LastTxBlock::<Test>::contains_key(test_account),
            "RateLimit storage should have been cleared"
        );

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[allow(deprecated)]
#[test]
fn test_migrate_last_tx_block_childkey_take() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entry
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_last_tx_block_childkey_take";

        let test_account: U256 = U256::from(1);
        let original_value: u64 = 123;

        LastTxBlockChildKeyTake::<Test>::insert(test_account, original_value);

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // ------------------------------
        // Step 2: Run the Migration
        // ------------------------------
        let weight = crate::migrations::migrate_rate_limiting_last_blocks::
        migrate_obsolete_rate_limiting_last_blocks_storage::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as completed"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------

        assert_eq!(
            SubtensorModule::get_last_tx_block_childkey_take(&test_account),
            original_value
        );
        assert!(
            !LastTxBlockChildKeyTake::<Test>::contains_key(test_account),
            "RateLimit storage should have been cleared"
        );

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[allow(deprecated)]
#[test]
fn test_migrate_last_tx_block_delegate_take() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entry
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_last_tx_block_delegate_take";

        let test_account: U256 = U256::from(1);
        let original_value: u64 = 123;

        LastTxBlockDelegateTake::<Test>::insert(test_account, original_value);

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // ------------------------------
        // Step 2: Run the Migration
        // ------------------------------
        let weight = crate::migrations::migrate_rate_limiting_last_blocks::
        migrate_last_tx_block_delegate_take::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as completed"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------

        assert_eq!(
            SubtensorModule::get_last_tx_block_delegate_take(&test_account),
            original_value
        );
        assert!(
            !LastTxBlockDelegateTake::<Test>::contains_key(test_account),
            "RateLimit storage should have been cleared"
        );

        assert!(!weight.is_zero(), "Migration weight should be non-zero");
    });
}

#[test]
fn test_migrate_rate_limit_keys() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &[u8] = b"migrate_rate_limit_keys";
        let prefix = {
            let pallet_prefix = twox_128("SubtensorModule".as_bytes());
            let storage_prefix = twox_128("LastRateLimitedBlock".as_bytes());
            [pallet_prefix, storage_prefix].concat()
        };

        // Seed new-format entries that must survive the migration untouched.
        let new_last_account = U256::from(10);
        SubtensorModule::set_last_tx_block(&new_last_account, 555);
        let new_child_account = U256::from(11);
        SubtensorModule::set_last_tx_block_childkey(&new_child_account, 777);
        let new_delegate_account = U256::from(12);
        SubtensorModule::set_last_tx_block_delegate_take(&new_delegate_account, 888);

        // Legacy NetworkLastRegistered entry (index 1)
        let mut legacy_network_key = prefix.clone();
        legacy_network_key.push(1u8);
        sp_io::storage::set(&legacy_network_key, &111u64.encode());

        // Legacy LastTxBlock entry (index 2) for an account that already has a new-format value.
        let mut legacy_last_key = prefix.clone();
        legacy_last_key.push(2u8);
        legacy_last_key.extend_from_slice(&new_last_account.encode());
        sp_io::storage::set(&legacy_last_key, &666u64.encode());

        // Legacy LastTxBlockChildKeyTake entry (index 3)
        let legacy_child_account = U256::from(3);
        ChildKeys::<Test>::insert(
            legacy_child_account,
            NetUid::from(0),
            vec![(0u64, U256::from(99))],
        );
        let mut legacy_child_key = prefix.clone();
        legacy_child_key.push(3u8);
        legacy_child_key.extend_from_slice(&legacy_child_account.encode());
        sp_io::storage::set(&legacy_child_key, &333u64.encode());

        // Legacy LastTxBlockDelegateTake entry (index 4)
        let legacy_delegate_account = U256::from(4);
        Delegates::<Test>::insert(legacy_delegate_account, 500u16);
        let mut legacy_delegate_key = prefix.clone();
        legacy_delegate_key.push(4u8);
        legacy_delegate_key.extend_from_slice(&legacy_delegate_account.encode());
        sp_io::storage::set(&legacy_delegate_key, &444u64.encode());

        let weight = crate::migrations::migrate_rate_limit_keys::migrate_rate_limit_keys::<Test>();
        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.to_vec()),
            "Migration should be marked as executed"
        );
        assert!(!weight.is_zero(), "Migration weight should be non-zero");

        // Legacy entries were migrated and cleared.
        assert_eq!(
            SubtensorModule::get_network_last_lock_block(),
            111u64,
            "Network last lock block should match migrated value"
        );
        assert!(
            sp_io::storage::get(&legacy_network_key).is_none(),
            "Legacy network entry should be cleared"
        );

        assert_eq!(
            SubtensorModule::get_last_tx_block(&new_last_account),
            666u64,
            "LastTxBlock should reflect the merged legacy value"
        );
        assert!(
            sp_io::storage::get(&legacy_last_key).is_none(),
            "Legacy LastTxBlock entry should be cleared"
        );

        assert_eq!(
            SubtensorModule::get_last_tx_block_childkey_take(&legacy_child_account),
            333u64,
            "Child key take block should be migrated"
        );
        assert!(
            sp_io::storage::get(&legacy_child_key).is_none(),
            "Legacy child take entry should be cleared"
        );

        assert_eq!(
            SubtensorModule::get_last_tx_block_delegate_take(&legacy_delegate_account),
            444u64,
            "Delegate take block should be migrated"
        );
        assert!(
            sp_io::storage::get(&legacy_delegate_key).is_none(),
            "Legacy delegate take entry should be cleared"
        );

        // New-format entries remain untouched.
        assert_eq!(
            SubtensorModule::get_last_tx_block_childkey_take(&new_child_account),
            777u64,
            "Existing child take entry should be preserved"
        );
        assert_eq!(
            SubtensorModule::get_last_tx_block_delegate_take(&new_delegate_account),
            888u64,
            "Existing delegate take entry should be preserved"
        );
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

// cargo test --package pallet-subtensor --lib -- tests::migration::test_migrate_fix_root_tao_and_alpha_in --exact --show-output
#[test]
fn test_migrate_fix_root_tao_and_alpha_in() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_fix_root_tao_and_alpha_in";

        // Set counters initially
        let initial_value = 1_000_000_000_000;
        SubnetTAO::<Test>::insert(NetUid::ROOT, TaoCurrency::from(initial_value));
        SubnetAlphaIn::<Test>::insert(NetUid::ROOT, AlphaCurrency::from(initial_value));
        SubnetAlphaOut::<Test>::insert(NetUid::ROOT, AlphaCurrency::from(initial_value));
        SubnetVolume::<Test>::insert(NetUid::ROOT, initial_value as u128);
        TotalStake::<Test>::set(TaoCurrency::from(initial_value));

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // Run the migration
        let weight =
            crate::migrations::migrate_fix_root_tao_and_alpha_in::migrate_fix_root_tao_and_alpha_in::<Test>();

        // Verify the migration ran correctly
        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as run"
        );
        assert!(!weight.is_zero(), "Migration weight should be non-zero");

        // Verify counters have changed
        assert!(SubnetTAO::<Test>::get(NetUid::ROOT) != initial_value.into());
        assert!(SubnetAlphaIn::<Test>::get(NetUid::ROOT) != initial_value.into());
        assert!(SubnetAlphaOut::<Test>::get(NetUid::ROOT) != initial_value.into());
        assert!(SubnetVolume::<Test>::get(NetUid::ROOT) != initial_value as u128);
        assert!(TotalStake::<Test>::get() != initial_value.into());
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

        CRV3WeightCommits::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            epoch,
            old_queue.clone(),
        );

        // Sanity: entry decodes under old alias
        assert_eq!(
            CRV3WeightCommits::<Test>::get(NetUidStorageIndex::from(netuid), epoch),
            old_queue
        );

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
            CRV3WeightCommits::<Test>::get(NetUidStorageIndex::from(netuid), epoch).is_empty(),
            "old queue should have been drained"
        );

        let new_q = CRV3WeightCommitsV2::<Test>::get(NetUidStorageIndex::from(netuid), epoch);
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
        assert_eq!(
            SubtensorModule::get_reveal_period(NetUid::from(netuid)),
            1u64
        );

        // Test direct storage access
        assert_eq!(RevealPeriodEpochs::<Test>::get(NetUid::from(netuid)), 1u64);
        assert!(CommitRevealWeightsEnabled::<Test>::get(NetUid::from(
            netuid
        )));
    });
}

#[test]
fn test_migrate_auto_stake_destination() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // Step 1: Simulate Old Storage Entries
        // ------------------------------
        const MIGRATION_NAME: &[u8] = b"migrate_auto_stake_destination";
		let netuids = [NetUid::ROOT, NetUid::from(1), NetUid::from(2), NetUid::from(42)];
		for netuid in &netuids {
			NetworksAdded::<Test>::insert(*netuid, true);
		}

        let pallet_prefix = twox_128("SubtensorModule".as_bytes());
        let storage_prefix = twox_128("AutoStakeDestination".as_bytes());

        // Create test accounts
        let coldkey1: U256 = U256::from(1);
        let coldkey2: U256 = U256::from(2);
        let hotkey1: U256 = U256::from(100);
        let hotkey2: U256 = U256::from(200);

        // Construct storage keys for old format (StorageMap)
        let mut key1 = Vec::new();
        key1.extend_from_slice(&pallet_prefix);
        key1.extend_from_slice(&storage_prefix);
        key1.extend_from_slice(&Blake2_128Concat::hash(&coldkey1.encode()));

        let mut key2 = Vec::new();
        key2.extend_from_slice(&pallet_prefix);
        key2.extend_from_slice(&storage_prefix);
        key2.extend_from_slice(&Blake2_128Concat::hash(&coldkey2.encode()));

        // Store old format entries
        put_raw(&key1, &hotkey1.encode());
        put_raw(&key2, &hotkey2.encode());

        // Verify old entries are stored
        assert_eq!(get_raw(&key1), Some(hotkey1.encode()));
        assert_eq!(get_raw(&key2), Some(hotkey2.encode()));

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.to_vec()),
            "Migration should not have run yet"
        );

        // ------------------------------
        // Step 2: Run the Migration
        // ------------------------------
        let weight = crate::migrations::migrate_auto_stake_destination::migrate_auto_stake_destination::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.to_vec()),
            "Migration should be marked as run"
        );

        // ------------------------------
        // Step 3: Verify Migration Effects
        // ------------------------------

        // Verify new format entries exist
		for netuid in &netuids {
			if *netuid == NetUid::ROOT {
				assert_eq!(
					AutoStakeDestination::<Test>::get(coldkey1, NetUid::ROOT),
					None
				);
				assert_eq!(
					AutoStakeDestination::<Test>::get(coldkey2, NetUid::ROOT),
					None
				);
			} else {
				assert_eq!(
					AutoStakeDestination::<Test>::get(coldkey1, *netuid),
					Some(hotkey1)
				);
				assert_eq!(
					AutoStakeDestination::<Test>::get(coldkey2, *netuid),
					Some(hotkey2)
				);

				// Verify entry for AutoStakeDestinationColdkeys
				assert_eq!(
					AutoStakeDestinationColdkeys::<Test>::get(hotkey1, *netuid),
					vec![coldkey1]
				);
				assert_eq!(
					AutoStakeDestinationColdkeys::<Test>::get(hotkey2, *netuid),
					vec![coldkey2]
				);
			}
		}

        // Verify old format entries are cleared
        assert_eq!(get_raw(&key1), None, "Old storage entry 1 should be cleared");
        assert_eq!(get_raw(&key2), None, "Old storage entry 2 should be cleared");

        // Verify weight calculation
        assert!(!weight.is_zero(), "Migration weight should be non-zero");

        // ------------------------------
        // Step 4: Test Migration Idempotency
        // ------------------------------
        let weight_second_run = crate::migrations::migrate_auto_stake_destination::migrate_auto_stake_destination::<Test>();

        // Second run should only read the migration flag
        assert_eq!(
            weight_second_run,
            <Test as Config>::DbWeight::get().reads(1),
            "Second run should only read the migration flag"
        );
    });
}

#[test]
fn test_migrate_crv3_v2_to_timelocked() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // 0. Constants / helpers
        // ------------------------------
        const MIG_NAME: &[u8] = b"crv3_v2_to_timelocked_v1";
        let netuid = NetUid::from(99);
        let epoch: u64 = 7;

        // ------------------------------
        // 1. Simulate OLD storage (4‑tuple; V2 layout)
        // ------------------------------
        let who: U256 = U256::from(0xdeadbeef_u64);
        let commit_block: u64 = 12345;
        let ciphertext: BoundedVec<u8, ConstU32<MAX_CRV3_COMMIT_SIZE_BYTES>> =
            vec![1u8, 2, 3].try_into().unwrap();
        let round: RoundNumber = 9;

        let old_queue: VecDeque<_> =
            VecDeque::from(vec![(who, commit_block, ciphertext.clone(), round)]);

        // Insert under the deprecated alias
        CRV3WeightCommitsV2::<Test>::insert(
            NetUidStorageIndex::from(netuid),
            epoch,
            old_queue.clone(),
        );

        // Sanity: entry decodes under old alias
        assert_eq!(
            CRV3WeightCommitsV2::<Test>::get(NetUidStorageIndex::from(netuid), epoch),
            old_queue,
            "pre-migration: old queue should be present"
        );

        // Destination should be empty pre-migration
        assert!(
            TimelockedWeightCommits::<Test>::get(NetUidStorageIndex::from(netuid), epoch)
                .is_empty(),
            "pre-migration: destination should be empty"
        );

        assert!(
            !HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag should be false before run"
        );

        // ------------------------------
        // 2. Run migration
        // ------------------------------
        let w = crate::migrations::migrate_crv3_v2_to_timelocked::migrate_crv3_v2_to_timelocked::<
            Test,
        >();
        assert!(!w.is_zero(), "weight must be non-zero");

        // ------------------------------
        // 3. Verify results
        // ------------------------------
        assert!(
            HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag not set"
        );

        // Old storage must be empty (drained)
        assert!(
            CRV3WeightCommitsV2::<Test>::get(NetUidStorageIndex::from(netuid), epoch).is_empty(),
            "old queue should have been drained"
        );

        // New storage must match exactly
        let new_q = TimelockedWeightCommits::<Test>::get(NetUidStorageIndex::from(netuid), epoch);
        assert_eq!(
            new_q, old_queue,
            "migrated queue must exactly match the old queue"
        );

        // Verify the front element matches what we inserted
        let (who2, commit_block2, cipher2, round2) = new_q.front().cloned().unwrap();
        assert_eq!(who2, who);
        assert_eq!(commit_block2, commit_block);
        assert_eq!(cipher2, ciphertext);
        assert_eq!(round2, round);
    });
}

#[test]
fn test_migrate_remove_network_modality() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // 0. Constants / helpers
        // ------------------------------
        const MIGRATION_NAME: &str = "migrate_remove_network_modality";

        // Create multiple networks to test
        let netuids: [NetUid; 3] = [1.into(), 2.into(), 3.into()];
        for netuid in netuids.iter() {
            add_network(*netuid, 1, 0);
        }

        // Set initial storage version to 7 (below target)
        StorageVersion::new(7).put::<Pallet<Test>>();
        assert_eq!(
            Pallet::<Test>::on_chain_storage_version(),
            StorageVersion::new(7)
        );

        // ------------------------------
        // 1. Simulate NetworkModality entries using deprecated storage alias
        // ------------------------------
        // We need to manually create storage entries that would exist for NetworkModality
        // Since NetworkModality was a StorageMap<_, Identity, NetUid, u16>, we simulate this
        let pallet_prefix = twox_128("SubtensorModule".as_bytes());
        let storage_prefix = twox_128("NetworkModality".as_bytes());

        // Create NetworkModality entries for each network
        for (i, netuid) in netuids.iter().enumerate() {
            let mut key = Vec::new();
            key.extend_from_slice(&pallet_prefix);
            key.extend_from_slice(&storage_prefix);
            // Identity encoding for netuid
            key.extend_from_slice(&netuid.encode());

            let modality_value: u16 = (i as u16) + 1; // Different values for testing
            put_raw(&key, &modality_value.encode());

            // Verify the entry was created
            let stored_value = get_raw(&key).expect("NetworkModality entry should exist");
            assert_eq!(
                u16::decode(&mut &stored_value[..]).expect("Failed to decode modality"),
                modality_value
            );
        }

        assert!(
            !HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should not have run yet"
        );

        // ------------------------------
        // 2. Run migration
        // ------------------------------
        let weight =
            crate::migrations::migrate_remove_network_modality::migrate_remove_network_modality::<
                Test,
            >();

        // ------------------------------
        // 3. Verify migration effects
        // ------------------------------
        assert!(
            HasMigrationRun::<Test>::get(MIGRATION_NAME.as_bytes().to_vec()),
            "Migration should be marked as run"
        );

        // Verify weight is non-zero
        assert!(!weight.is_zero(), "Migration weight should be non-zero");

        // Verify weight calculation: 1 read (version check) + 1 read (total networks) + N writes (removal) + 1 write (version update)
        let expected_weight = <Test as Config>::DbWeight::get().reads(2)
            + <Test as Config>::DbWeight::get().writes(netuids.len() as u64 + 1);
        assert_eq!(
            weight, expected_weight,
            "Weight calculation should be correct"
        );
    });
}

#[test]
fn test_migrate_remove_network_modality_already_run() {
    new_test_ext(1).execute_with(|| {
        const MIGRATION_NAME: &str = "migrate_remove_network_modality";

        // Mark migration as already run
        HasMigrationRun::<Test>::insert(MIGRATION_NAME.as_bytes().to_vec(), true);

        // Set storage version to 8 (target version)
        StorageVersion::new(8).put::<Pallet<Test>>();
        assert_eq!(
            Pallet::<Test>::on_chain_storage_version(),
            StorageVersion::new(8)
        );

        // Run migration
        let weight =
            crate::migrations::migrate_remove_network_modality::migrate_remove_network_modality::<
                Test,
            >();

        // Should only have read weight for checking migration status
        let expected_weight = <Test as Config>::DbWeight::get().reads(1);
        assert_eq!(
            weight, expected_weight,
            "Second run should only read the migration flag"
        );

        // Verify migration is still marked as run
        assert!(HasMigrationRun::<Test>::get(
            MIGRATION_NAME.as_bytes().to_vec()
        ));
    });
}

#[test]
fn test_migrate_subnet_limit_to_default() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // 0. Constants / helpers
        // ------------------------------
        const MIG_NAME: &[u8] = b"subnet_limit_to_default";

        // Compute a non-default value safely
        let default: u16 = DefaultSubnetLimit::<Test>::get();
        let not_default: u16 = default.wrapping_add(1);

        // ------------------------------
        // 1. Pre-state: ensure a non-default value is stored
        // ------------------------------
        SubnetLimit::<Test>::put(not_default);
        assert_eq!(
            SubnetLimit::<Test>::get(),
            not_default,
            "precondition failed: SubnetLimit should be non-default before migration"
        );

        assert!(
            !HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag should be false before run"
        );

        // ------------------------------
        // 2. Run migration
        // ------------------------------
        let w = crate::migrations::migrate_subnet_limit_to_default::migrate_subnet_limit_to_default::<Test>();
        assert!(!w.is_zero(), "weight must be non-zero");

        // ------------------------------
        // 3. Verify results
        // ------------------------------
        assert!(
            HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag not set"
        );

        assert_eq!(
            SubnetLimit::<Test>::get(),
            default,
            "SubnetLimit should be reset to the configured default"
        );
    });
}

#[test]
fn test_migrate_network_lock_reduction_interval_and_decay() {
    new_test_ext(0).execute_with(|| {
        const FOUR_DAYS: u64 = 28_800;
        const EIGHT_DAYS: u64 = 57_600;
        const ONE_WEEK_BLOCKS: u64 = 50_400;

        // ── pre ──────────────────────────────────────────────────────────────
        assert!(
            !HasMigrationRun::<Test>::get(b"migrate_network_lock_reduction_interval".to_vec()),
            "HasMigrationRun should be false before migration"
        );

        // ensure current_block > 0
        step_block(1);
        let current_block_before = Pallet::<Test>::get_current_block_as_u64();

        // ── run migration ────────────────────────────────────────────────────
        let weight = crate::migrations::migrate_network_lock_reduction_interval::migrate_network_lock_reduction_interval::<Test>();
        assert!(!weight.is_zero(), "migration weight should be > 0");

        // ── params & flags ───────────────────────────────────────────────────
        assert_eq!(NetworkLockReductionInterval::<Test>::get(), EIGHT_DAYS);
        assert_eq!(NetworkRateLimit::<Test>::get(), FOUR_DAYS);
        assert_eq!(
            Pallet::<Test>::get_network_last_lock(),
            1_000_000_000_000u64.into(), // 1000 TAO in rao
            "last_lock should be 1_000_000_000_000 rao"
        );

        // last_lock_block should be set one week in the future
        let last_lock_block = Pallet::<Test>::get_network_last_lock_block();
        let expected_block = current_block_before + ONE_WEEK_BLOCKS;
        assert_eq!(
            last_lock_block,
            expected_block,
            "last_lock_block should be current + ONE_WEEK_BLOCKS"
        );

        // registration start block should match the same future block
        assert_eq!(
            NetworkRegistrationStartBlock::<Test>::get(),
            expected_block,
            "NetworkRegistrationStartBlock should equal last_lock_block"
        );

        // lock cost should be 2000 TAO immediately after migration
        let lock_cost_now = Pallet::<Test>::get_network_lock_cost();
        assert_eq!(
            lock_cost_now,
            2_000_000_000_000u64.into(),
            "lock cost should be 2000 TAO right after migration"
        );

        assert!(
            HasMigrationRun::<Test>::get(b"migrate_network_lock_reduction_interval".to_vec()),
            "HasMigrationRun should be true after migration"
        );
    });
}

#[test]
fn test_migrate_restore_subnet_locked_65_128() {
    use sp_runtime::traits::SaturatedConversion;
    new_test_ext(0).execute_with(|| {
        let name = b"migrate_restore_subnet_locked".to_vec();
        assert!(
            !HasMigrationRun::<Test>::get(name.clone()),
            "HasMigrationRun should be false before migration"
        );

        // Expected snapshot for netuids 65..128.
        const EXPECTED: &[(u16, u64)] = &[
            (65, 37_274_536_408),
            (66, 65_230_444_016),
            (67, 114_153_284_032),
            (68, 199_768_252_064),
            (69, 349_594_445_728),
            (70, 349_412_366_216),
            (71, 213_408_488_702),
            (72, 191_341_473_067),
            (73, 246_711_333_592),
            (74, 291_874_466_228),
            (75, 247_485_227_056),
            (76, 291_241_991_316),
            (77, 303_154_601_714),
            (78, 287_407_417_932),
            (79, 254_935_051_664),
            (80, 255_413_055_349),
            (81, 249_790_431_509),
            (82, 261_343_249_180),
            (83, 261_361_408_796),
            (84, 201_938_003_214),
            (85, 264_805_234_604),
            (86, 223_171_973_880),
            (87, 180_397_358_280),
            (88, 270_596_039_760),
            (89, 286_399_608_951),
            (90, 267_684_201_301),
            (91, 284_637_542_762),
            (92, 288_373_410_868),
            (93, 290_836_604_849),
            (94, 270_861_792_144),
            (95, 210_595_055_304),
            (96, 315_263_727_200),
            (97, 158_244_884_792),
            (98, 168_102_223_900),
            (99, 252_153_339_800),
            (100, 378_230_014_000),
            (101, 205_977_765_866),
            (102, 149_434_017_849),
            (103, 135_476_471_008),
            (104, 147_970_415_680),
            (105, 122_003_668_139),
            (106, 133_585_556_570),
            (107, 200_137_144_216),
            (108, 106_767_623_816),
            (109, 124_280_483_748),
            (110, 186_420_726_696),
            (111, 249_855_564_892),
            (112, 196_761_272_984),
            (113, 147_120_048_727),
            (114, 84_021_895_534),
            (115, 98_002_215_656),
            (116, 89_944_262_256),
            (117, 107_183_582_952),
            (118, 110_644_724_664),
            (119, 99_380_483_902),
            (120, 138_829_019_156),
            (121, 111_988_743_976),
            (122, 130_264_686_152),
            (123, 118_034_291_488),
            (124, 79_312_501_676),
            (125, 43_214_310_704),
            (126, 64_755_449_962),
            (127, 97_101_698_382),
            (128, 145_645_807_991),
        ];

        // Run migration
        let weight =
            crate::migrations::migrate_subnet_locked::migrate_restore_subnet_locked::<Test>();
        assert!(!weight.is_zero(), "migration weight should be > 0");

        // Read back storage as (u16 -> u64)
        let actual: BTreeMap<u16, u64> = SubnetLocked::<Test>::iter()
            .map(|(k, v)| (k.saturated_into::<u16>(), u64::from(v)))
            .collect();

        let expected: BTreeMap<u16, u64> = EXPECTED.iter().copied().collect();

        // 1) exact content
        assert_eq!(
            actual, expected,
            "SubnetLocked map mismatch for 65..128 snapshot"
        );

        // 2) count and total
        let expected_len = expected.len();
        let expected_sum: u128 = expected.values().map(|v| *v as u128).sum();

        let count_after = actual.len();
        let sum_after: u128 = actual.values().map(|v| *v as u128).sum();

        assert_eq!(count_after, expected_len, "entry count mismatch");
        assert_eq!(sum_after, expected_sum, "total RAO sum mismatch");

        // 3) migration flag set
        assert!(
            HasMigrationRun::<Test>::get(name.clone()),
            "HasMigrationRun should be true after migration"
        );

        // 4) idempotence
        let before = actual.clone();
        let _again =
            crate::migrations::migrate_subnet_locked::migrate_restore_subnet_locked::<Test>();
        let after: BTreeMap<u16, u64> = SubnetLocked::<Test>::iter()
            .map(|(k, v)| (k.saturated_into::<u16>(), u64::from(v)))
            .collect();
        assert_eq!(
            before, after,
            "re-running the migration should not change storage"
        );
    });
}

#[test]
fn test_migrate_network_lock_cost_2500_sets_price_and_decay() {
    new_test_ext(0).execute_with(|| {
        // ── constants ───────────────────────────────────────────────────────
        const RAO_PER_TAO: u64 = 1_000_000_000;
        const TARGET_COST_TAO: u64 = 2_500;
        const TARGET_COST_RAO: u64 = TARGET_COST_TAO * RAO_PER_TAO;
        const NEW_LAST_LOCK_RAO: u64 = (TARGET_COST_TAO / 2) * RAO_PER_TAO;

        let migration_key = b"migrate_network_lock_cost_2500".to_vec();

        // ── pre ──────────────────────────────────────────────────────────────
        assert!(
            !HasMigrationRun::<Test>::get(migration_key.clone()),
            "HasMigrationRun should be false before migration"
        );

        // Ensure current_block > 0 so mult == 2 in get_network_lock_cost()
        step_block(1);
        let current_block_before = Pallet::<Test>::get_current_block_as_u64();

        // Snapshot interval to ensure migration doesn't change it
        let interval_before = NetworkLockReductionInterval::<Test>::get();

        // ── run migration ────────────────────────────────────────────────────
        let weight = crate::migrations::migrate_network_lock_cost_2500::migrate_network_lock_cost_2500::<Test>();
        assert!(!weight.is_zero(), "migration weight should be > 0");

        // ── asserts: params & flags ─────────────────────────────────────────
        assert_eq!(
            Pallet::<Test>::get_network_last_lock(),
            NEW_LAST_LOCK_RAO.into(),
            "last_lock should be set to 1,250 TAO (in rao)"
        );
        assert_eq!(
            Pallet::<Test>::get_network_last_lock_block(),
            current_block_before,
            "last_lock_block should be set to the current block"
        );

        // Lock cost should be exactly 2,500 TAO immediately after migration
        let lock_cost_now = Pallet::<Test>::get_network_lock_cost();
        assert_eq!(
            lock_cost_now,
            TARGET_COST_RAO.into(),
            "lock cost should be 2,500 TAO right after migration"
        );

        // Interval should be unchanged by this migration
        assert_eq!(
            NetworkLockReductionInterval::<Test>::get(),
            interval_before,
            "lock reduction interval should not be modified by this migration"
        );

        assert!(
            HasMigrationRun::<Test>::get(migration_key.clone()),
            "HasMigrationRun should be true after migration"
        );

        // ── decay check (1 block later) ─────────────────────────────────────
        // Expected: cost = max(min_lock, 2*L - floor(L / eff_interval) * delta_blocks)
        let eff_interval = Pallet::<Test>::get_lock_reduction_interval();
        let per_block_decrement: u64 = if eff_interval == 0 {
            0
        } else {
            NEW_LAST_LOCK_RAO / eff_interval
        };

        let min_lock_rao: u64 = Pallet::<Test>::get_network_min_lock().to_u64();

        step_block(1);
        let expected_after_1: u64 =
            core::cmp::max(min_lock_rao, TARGET_COST_RAO - per_block_decrement);
        let lock_cost_after_1 = Pallet::<Test>::get_network_lock_cost();
        assert_eq!(
            lock_cost_after_1,
            expected_after_1.into(),
            "lock cost should decay by one per-block step after 1 block"
        );

        // ── idempotency: running the migration again should do nothing ──────
        let last_lock_before_rerun = Pallet::<Test>::get_network_last_lock();
        let last_lock_block_before_rerun = Pallet::<Test>::get_network_last_lock_block();
        let cost_before_rerun = Pallet::<Test>::get_network_lock_cost();

        let _weight2 = crate::migrations::migrate_network_lock_cost_2500::migrate_network_lock_cost_2500::<Test>();

        assert!(
            HasMigrationRun::<Test>::get(migration_key.clone()),
            "HasMigrationRun remains true on second run"
        );
        assert_eq!(
            Pallet::<Test>::get_network_last_lock(),
            last_lock_before_rerun,
            "second run should not modify last_lock"
        );
        assert_eq!(
            Pallet::<Test>::get_network_last_lock_block(),
            last_lock_block_before_rerun,
            "second run should not modify last_lock_block"
        );
        assert_eq!(
            Pallet::<Test>::get_network_lock_cost(),
            cost_before_rerun,
            "second run should not change current lock cost"
        );
    });
}

#[test]
fn test_migrate_kappa_map_to_default() {
    new_test_ext(1).execute_with(|| {
        // ------------------------------
        // 0. Constants / helpers
        // ------------------------------
        const MIG_NAME: &[u8] = b"kappa_map_to_default";
        let default: u16 = DefaultKappa::<Test>::get();

        let not_default: u16 = if default == u16::MAX {
            default - 1
        } else {
            default + 1
        };

        // ------------------------------
        // 1. Pre-state: seed using the correct key type (NetUid)
        // ------------------------------
        let n0: NetUid = 0u16.into();
        let n1: NetUid = 1u16.into();
        let n2: NetUid = 42u16.into();

        Kappa::<Test>::insert(n0, not_default);
        Kappa::<Test>::insert(n1, default);
        Kappa::<Test>::insert(n2, not_default);

        assert_eq!(
            Kappa::<Test>::get(n0),
            not_default,
            "precondition failed: Kappa[n0] should be non-default before migration"
        );
        assert_eq!(
            Kappa::<Test>::get(n1),
            default,
            "precondition failed: Kappa[n1] should be default before migration"
        );
        assert_eq!(
            Kappa::<Test>::get(n2),
            not_default,
            "precondition failed: Kappa[n2] should be non-default before migration"
        );

        assert!(
            !HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag should be false before run"
        );

        // ------------------------------
        // 2. Run migration
        // ------------------------------
        let w =
            crate::migrations::migrate_kappa_map_to_default::migrate_kappa_map_to_default::<Test>();
        assert!(!w.is_zero(), "weight must be non-zero");

        // ------------------------------
        // 3. Verify results
        // ------------------------------
        assert!(
            HasMigrationRun::<Test>::get(MIG_NAME.to_vec()),
            "migration flag not set"
        );

        assert_eq!(
            Kappa::<Test>::get(n0),
            default,
            "Kappa[n0] should be reset to the configured default"
        );
        assert_eq!(
            Kappa::<Test>::get(n1),
            default,
            "Kappa[n1] should remain at the configured default"
        );
        assert_eq!(
            Kappa::<Test>::get(n2),
            default,
            "Kappa[n2] should be reset to the configured default"
        );
    });
}

#[test]
fn test_migrate_remove_tao_dividends() {
    const MIGRATION_NAME: &str = "migrate_remove_tao_dividends";
    let pallet_name = "SubtensorModule";
    let storage_name = "TaoDividendsPerSubnet";
    let migration =
        crate::migrations::migrate_remove_tao_dividends::migrate_remove_tao_dividends::<Test>;

    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );

    let storage_name = "PendingAlphaSwapped";
    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );

    let storage_name = "PendingRootDivs";
    test_remove_storage_item(
        MIGRATION_NAME,
        pallet_name,
        storage_name,
        migration,
        200_000,
    );
}

fn do_setup_unactive_sn() -> (Vec<NetUid>, Vec<NetUid>) {
    // Register some subnets
    let netuid0 = add_dynamic_network_without_emission_block(&U256::from(0), &U256::from(0));
    let netuid1 = add_dynamic_network_without_emission_block(&U256::from(1), &U256::from(1));
    let netuid2 = add_dynamic_network_without_emission_block(&U256::from(2), &U256::from(2));
    let inactive_netuids = vec![netuid0, netuid1, netuid2];
    // Add active subnets
    let netuid3 = add_dynamic_network_without_emission_block(&U256::from(3), &U256::from(3));
    let netuid4 = add_dynamic_network_without_emission_block(&U256::from(4), &U256::from(4));
    let netuid5 = add_dynamic_network_without_emission_block(&U256::from(5), &U256::from(5));
    let active_netuids = vec![netuid3, netuid4, netuid5];
    let netuids: Vec<NetUid> = inactive_netuids
        .iter()
        .chain(active_netuids.iter())
        .copied()
        .collect();

    let initial_tao = Pallet::<Test>::get_network_min_lock();
    let initial_alpha: AlphaCurrency = initial_tao.to_u64().into();

    const EXTRA_POOL_TAO: u64 = 123_123_u64;
    const EXTRA_POOL_ALPHA: u64 = 123_123_u64;

    // Add stake to the subnet pools
    for netuid in &netuids {
        let extra_for_pool = TaoCurrency::from(EXTRA_POOL_TAO);
        let stake_in_pool = TaoCurrency::from(
            u64::from(initial_tao)
                .checked_add(EXTRA_POOL_TAO)
                .expect("initial_tao + extra_for_pool overflow"),
        );
        SubnetTAO::<Test>::insert(netuid, stake_in_pool);
        TotalStake::<Test>::mutate(|total_stake| {
            let updated_total = u64::from(*total_stake)
                .checked_add(EXTRA_POOL_TAO)
                .expect("total stake overflow");
            *total_stake = updated_total.into();
        });
        TotalIssuance::<Test>::mutate(|total_issuance| {
            let updated_total = u64::from(*total_issuance)
                .checked_add(EXTRA_POOL_TAO)
                .expect("total issuance overflow");
            *total_issuance = updated_total.into();
        });

        let subnet_alpha_in = AlphaCurrency::from(
            u64::from(initial_alpha)
                .checked_add(EXTRA_POOL_ALPHA)
                .expect("initial alpha + extra alpha overflow"),
        );
        SubnetAlphaIn::<Test>::insert(netuid, subnet_alpha_in);
        SubnetAlphaOut::<Test>::insert(netuid, AlphaCurrency::from(EXTRA_POOL_ALPHA));
        SubnetVolume::<Test>::insert(netuid, 123123_u128);

        // Try registering on the subnet to simulate a real network
        // give balance to the coldkey
        let coldkey_account_id = U256::from(1111);
        let hotkey_account_id = U256::from(1111);
        let burn_cost = SubtensorModule::get_burn(*netuid);
        SubtensorModule::add_balance_to_coldkey_account(&coldkey_account_id, burn_cost.into());
        TotalIssuance::<Test>::mutate(|total_issuance| {
            let updated_total = u64::from(*total_issuance)
                .checked_add(u64::from(burn_cost))
                .expect("total issuance overflow (burn)");
            *total_issuance = updated_total.into();
        });

        // register the neuron
        assert_ok!(SubtensorModule::burned_register(
            <<Test as Config>::RuntimeOrigin>::signed(coldkey_account_id),
            *netuid,
            hotkey_account_id
        ));
    }

    for netuid in &active_netuids {
        // Set the FirstEmissionBlockNumber for the active subnet
        FirstEmissionBlockNumber::<Test>::insert(netuid, 100);
        // Also set SubtokenEnabled to true
        SubtokenEnabled::<Test>::insert(netuid, true);
    }

    let alpha_amt = AlphaCurrency::from(123123_u64);
    // Create some Stake entries
    for netuid in &netuids {
        for hotkey in 0..10 {
            let hk = U256::from(hotkey);
            TotalHotkeyAlpha::<Test>::insert(hk, netuid, alpha_amt);
            TotalHotkeyShares::<Test>::insert(hk, netuid, U64F64::from(123123_u64));
            TotalHotkeyAlphaLastEpoch::<Test>::insert(hk, netuid, alpha_amt);

            RootClaimable::<Test>::mutate(hk, |claimable| {
                claimable.insert(*netuid, I96F32::from(alpha_amt.to_u64()));
            });
            for coldkey in 0..10 {
                let ck = U256::from(coldkey);
                Alpha::<Test>::insert((hk, ck, netuid), U64F64::from(123_u64));
                RootClaimed::<Test>::insert((netuid, hk, ck), 222_u128);
            }
        }
    }
    // Add some pending emissions
    let alpha_em_amt = AlphaCurrency::from(355555_u64);
    for netuid in &netuids {
        PendingServerEmission::<Test>::insert(netuid, alpha_em_amt);
        PendingValidatorEmission::<Test>::insert(netuid, alpha_em_amt);
        PendingRootAlphaDivs::<Test>::insert(netuid, alpha_em_amt);
        PendingOwnerCut::<Test>::insert(netuid, alpha_em_amt);

        SubnetTaoInEmission::<Test>::insert(netuid, TaoCurrency::from(12345678_u64));
        SubnetAlphaInEmission::<Test>::insert(netuid, AlphaCurrency::from(12345678_u64));
        SubnetAlphaOutEmission::<Test>::insert(netuid, AlphaCurrency::from(12345678_u64));
    }

    (active_netuids, inactive_netuids)
}

#[test]
fn test_migrate_reset_unactive_sn_get_unactive_netuids() {
    new_test_ext(1).execute_with(|| {
        let (active_netuids, inactive_netuids) = do_setup_unactive_sn();

        let initial_tao = Pallet::<Test>::get_network_min_lock();
        let initial_alpha: AlphaCurrency = initial_tao.to_u64().into();

        let (unactive_netuids, w) =
            crate::migrations::migrate_reset_unactive_sn::get_unactive_sn_netuids::<Test>(
                initial_alpha,
            );
        // Make sure ALL the inactive subnets are in the unactive netuids
        assert!(
            inactive_netuids
                .iter()
                .all(|netuid| unactive_netuids.contains(netuid))
        );
        // Make sure the active subnets are not in the unactive netuids
        assert!(
            active_netuids
                .iter()
                .all(|netuid| !unactive_netuids.contains(netuid))
        );
    });
}

#[test]
fn test_migrate_reset_unactive_sn() {
    new_test_ext(1).execute_with(|| {
        let (active_netuids, inactive_netuids) = do_setup_unactive_sn();

        let initial_tao = Pallet::<Test>::get_network_min_lock();
        let initial_alpha: AlphaCurrency = initial_tao.to_u64().into();

        // Run the migration
        let w = crate::migrations::migrate_reset_unactive_sn::migrate_reset_unactive_sn::<Test>();
        assert!(!w.is_zero(), "weight must be non-zero");

        // Verify the results
        for netuid in &inactive_netuids {
            let actual_tao_lock_amount = SubnetLocked::<Test>::get(*netuid);
            let actual_tao_lock_amount_less_pool_tao = if (actual_tao_lock_amount < initial_tao) {
                TaoCurrency::ZERO
            } else {
                actual_tao_lock_amount - initial_tao
            };
            assert_eq!(
                PendingServerEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_eq!(
                PendingValidatorEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_eq!(
                PendingRootAlphaDivs::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_eq!(
                // not modified
                RAORecycledForRegistration::<Test>::get(netuid),
                actual_tao_lock_amount_less_pool_tao
            );
            assert!(pallet_subtensor_swap::AlphaSqrtPrice::<Test>::contains_key(
                *netuid
            ));
            assert_eq!(PendingOwnerCut::<Test>::get(netuid), AlphaCurrency::ZERO);
            assert_ne!(SubnetTAO::<Test>::get(netuid), initial_tao);
            assert_ne!(SubnetAlphaIn::<Test>::get(netuid), initial_alpha);
            assert_ne!(SubnetAlphaOut::<Test>::get(netuid), AlphaCurrency::ZERO);
            assert_eq!(SubnetTaoInEmission::<Test>::get(netuid), TaoCurrency::ZERO);
            assert_eq!(
                SubnetAlphaInEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_eq!(
                SubnetAlphaOutEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_ne!(SubnetVolume::<Test>::get(netuid), 0u128);
            for hotkey in 0..10 {
                let hk = U256::from(hotkey);
                assert_ne!(
                    TotalHotkeyAlpha::<Test>::get(hk, netuid),
                    AlphaCurrency::ZERO
                );
                assert_ne!(
                    TotalHotkeyShares::<Test>::get(hk, netuid),
                    U64F64::from_num(0.0)
                );
                assert_ne!(
                    TotalHotkeyAlphaLastEpoch::<Test>::get(hk, netuid),
                    AlphaCurrency::ZERO
                );
                assert_ne!(RootClaimable::<Test>::get(hk).get(netuid), None);
                for coldkey in 0..10 {
                    let ck = U256::from(coldkey);
                    assert_ne!(Alpha::<Test>::get((hk, ck, netuid)), U64F64::from_num(0.0));
                    assert_ne!(RootClaimed::<Test>::get((netuid, hk, ck)), 0u128);
                }
            }

            // Don't touch SubnetLocked
            assert_ne!(SubnetLocked::<Test>::get(netuid), TaoCurrency::ZERO);
        }

        // !!! Make sure the active subnets were not reset
        for netuid in &active_netuids {
            let actual_tao_lock_amount = SubnetLocked::<Test>::get(*netuid);
            let actual_tao_lock_amount_less_pool_tao = actual_tao_lock_amount - initial_tao;
            assert_ne!(
                PendingServerEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_ne!(
                PendingValidatorEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_ne!(
                PendingRootAlphaDivs::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_eq!(
                // not modified
                RAORecycledForRegistration::<Test>::get(netuid),
                actual_tao_lock_amount_less_pool_tao
            );
            assert_ne!(SubnetTaoInEmission::<Test>::get(netuid), TaoCurrency::ZERO);
            assert_ne!(
                SubnetAlphaInEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert_ne!(
                SubnetAlphaOutEmission::<Test>::get(netuid),
                AlphaCurrency::ZERO
            );
            assert!(pallet_subtensor_swap::AlphaSqrtPrice::<Test>::contains_key(
                *netuid
            ));
            assert_ne!(PendingOwnerCut::<Test>::get(netuid), AlphaCurrency::ZERO);
            assert_ne!(SubnetTAO::<Test>::get(netuid), initial_tao);
            assert_ne!(SubnetAlphaIn::<Test>::get(netuid), initial_alpha);
            assert_ne!(SubnetAlphaOut::<Test>::get(netuid), AlphaCurrency::ZERO);
            assert_ne!(SubnetVolume::<Test>::get(netuid), 0u128);
            for hotkey in 0..10 {
                let hk = U256::from(hotkey);
                assert_ne!(
                    TotalHotkeyAlpha::<Test>::get(hk, netuid),
                    AlphaCurrency::ZERO
                );
                assert_ne!(
                    TotalHotkeyShares::<Test>::get(hk, netuid),
                    U64F64::from_num(0.0)
                );
                assert_ne!(
                    TotalHotkeyAlphaLastEpoch::<Test>::get(hk, netuid),
                    AlphaCurrency::ZERO
                );
                assert!(RootClaimable::<Test>::get(hk).contains_key(netuid));
                for coldkey in 0..10 {
                    let ck = U256::from(coldkey);
                    assert_ne!(Alpha::<Test>::get((hk, ck, netuid)), U64F64::from_num(0.0));
                    assert_ne!(RootClaimed::<Test>::get((netuid, hk, ck)), 0u128);
                }
            }
            // Don't touch SubnetLocked
            assert_ne!(SubnetLocked::<Test>::get(netuid), TaoCurrency::ZERO);
        }
    });
}

#[test]
fn test_migrate_reset_unactive_sn_idempotence() {
    new_test_ext(1).execute_with(|| {
        let (active_netuids, inactive_netuids) = do_setup_unactive_sn();
        let netuids = inactive_netuids
            .iter()
            .chain(active_netuids.iter())
            .copied()
            .collect::<Vec<_>>();

        // Run the migration
        let w = crate::migrations::migrate_reset_unactive_sn::migrate_reset_unactive_sn::<Test>();
        assert!(!w.is_zero(), "weight must be non-zero");

        let mut subnet_tao_before = BTreeMap::new();
        for netuid in &netuids {
            subnet_tao_before.insert(netuid, SubnetTAO::<Test>::get(netuid));
        }
        let total_stake_before = TotalStake::<Test>::get();
        let total_issuance_before = TotalIssuance::<Test>::get();

        // Run total issuance migration
        crate::migrations::migrate_init_total_issuance::migrate_init_total_issuance::<Test>();

        // Verify that none of the values are different
        for netuid in &netuids {
            assert_eq!(
                SubnetTAO::<Test>::get(netuid),
                *subnet_tao_before.get(netuid).unwrap_or(&TaoCurrency::ZERO)
            );
        }
        assert_eq!(TotalStake::<Test>::get(), total_stake_before);
        assert_eq!(TotalIssuance::<Test>::get(), total_issuance_before);
    });
}
