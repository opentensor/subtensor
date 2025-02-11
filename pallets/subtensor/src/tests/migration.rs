#![allow(unused, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]

use super::mock::*;
use crate::*;
use codec::{Decode, Encode};
use frame_support::{
    assert_ok,
    storage::unhashed::{get, get_raw, put, put_raw},
    traits::{StorageInstance, StoredMap},
    weights::Weight,
    StorageHasher, Twox64Concat,
};
use frame_system::Config;
use sp_core::{crypto::Ss58Codec, H256, U256};
use sp_io::hashing::twox_128;
use sp_runtime::traits::Zero;
use substrate_fixed::types::extra::U2;
use substrate_fixed::types::I96F32;

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
        crate::SubnetLocked::<Test>::insert(1, 100);
        crate::SubnetLocked::<Test>::insert(2, 5);
        pallet_balances::TotalIssuance::<Test>::put(1000);
        crate::TotalStake::<Test>::put(25);

        // Ensure values are NOT initialized prior to running migration
        assert!(crate::TotalIssuance::<Test>::get() == 0);

        crate::migrations::migrate_init_total_issuance::initialise_total_issuance::Migration::<Test>::on_runtime_upgrade();

        // Ensure values were initialized correctly
        assert!(
            crate::TotalIssuance::<Test>::get()
                == 105u64.saturating_add(1000).saturating_add(25)
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

fn run_migration_and_check(migration_name: &'static str) -> frame_support::weights::Weight {
    // Execute the migration and store its weight
    let weight: frame_support::weights::Weight =
        crate::migrations::migrate_fix_total_coldkey_stake::migrate_fix_total_coldkey_stake::<Test>(
        );

    // Check if the migration has been marked as completed
    assert!(HasMigrationRun::<Test>::get(
        migration_name.as_bytes().to_vec()
    ));

    // Return the weight of the executed migration
    weight
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

// SKIP_WASM_BUILD=1 RUST_LOG=debug cargo test --package pallet-subtensor --lib -- tests::migration::test_migrate_rao --exact --show-output --nocapture
#[test]
fn test_migrate_rao() {
    new_test_ext(1).execute_with(|| {
        // Setup initial state
        let netuid_0: u16 = 0;
        let netuid_1: u16 = 1;
        let netuid_2: u16 = 2;
        let netuid_3: u16 = 3;
        let hotkey1 = U256::from(1);
        let hotkey2 = U256::from(2);
        let coldkey1 = U256::from(3);
        let coldkey2 = U256::from(4);
        let coldkey3 = U256::from(5);
        let stake_amount: u64 = 1_000_000_000;
        let lock_amount: u64 = 500;
        NetworkMinLockCost::<Test>::set(500);

        // Add networks root and alpha
        add_network(netuid_0, 1, 0);
        add_network(netuid_1, 1, 0);
        add_network(netuid_2, 1, 0);
        add_network(netuid_3, 1, 0);

        // Set subnet lock
        SubnetLocked::<Test>::insert(netuid_1, lock_amount);

        // Add some initial stake
        EmissionValues::<Test>::insert(netuid_1, 1_000_000_000);
        EmissionValues::<Test>::insert(netuid_2, 2_000_000_000);
        EmissionValues::<Test>::insert(netuid_3, 3_000_000_000);

        Owner::<Test>::insert(hotkey1, coldkey1);
        Owner::<Test>::insert(hotkey2, coldkey2);
        Stake::<Test>::insert(hotkey1, coldkey1, stake_amount);
        Stake::<Test>::insert(hotkey1, coldkey2, stake_amount);
        Stake::<Test>::insert(hotkey2, coldkey2, stake_amount);
        Stake::<Test>::insert(hotkey2, coldkey3, stake_amount);

        // Verify initial conditions
        assert_eq!(SubnetTAO::<Test>::get(netuid_0), 0);
        assert_eq!(SubnetTAO::<Test>::get(netuid_1), 0);
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid_0), 0);
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid_1), 0);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid_0), 0);
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid_1), 0);
        assert_eq!(TotalHotkeyShares::<Test>::get(hotkey1, netuid_0), 0);
        assert_eq!(TotalHotkeyShares::<Test>::get(hotkey1, netuid_1), 0);
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey1, netuid_0), 0);
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey2, netuid_1), 0);

        // Run migration
        crate::migrations::migrate_rao::migrate_rao::<Test>();

        // Verify root subnet (netuid 0) state after migration
        assert_eq!(SubnetTAO::<Test>::get(netuid_0), 4 * stake_amount); // Root has everything
        assert_eq!(SubnetTAO::<Test>::get(netuid_1), 1_000_000_000); // Always 1000000000
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid_0), 1_000_000_000); // Always 1_000_000_000
        assert_eq!(SubnetAlphaIn::<Test>::get(netuid_1), 1_000_000_000); // Always 1_000_000_000
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid_0), 4 * stake_amount); // Root has everything.
        assert_eq!(SubnetAlphaOut::<Test>::get(netuid_1), 0); // No stake outstanding.

        // Assert share information for hotkey1 on netuid_0
        assert_eq!(
            TotalHotkeyShares::<Test>::get(hotkey1, netuid_0),
            2 * stake_amount
        ); // Shares
           // Assert no shares for hotkey1 on netuid_1
        assert_eq!(TotalHotkeyShares::<Test>::get(hotkey1, netuid_1), 0); // No shares
                                                                          // Assert alpha for hotkey1 on netuid_0
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey1, netuid_0),
            2 * stake_amount
        ); // Alpha
           // Assert no alpha for hotkey1 on netuid_1
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey1, netuid_1), 0); // No alpha.
                                                                         // Assert share information for hotkey2 on netuid_0
        assert_eq!(
            TotalHotkeyShares::<Test>::get(hotkey2, netuid_0),
            2 * stake_amount
        ); // Shares
           // Assert no shares for hotkey2 on netuid_1
        assert_eq!(TotalHotkeyShares::<Test>::get(hotkey2, netuid_1), 0); // No shares
                                                                          // Assert alpha for hotkey2 on netuid_0
        assert_eq!(
            TotalHotkeyAlpha::<Test>::get(hotkey2, netuid_0),
            2 * stake_amount
        ); // Alpha
           // Assert no alpha for hotkey2 on netuid_1
        assert_eq!(TotalHotkeyAlpha::<Test>::get(hotkey2, netuid_1), 0); // No alpha.

        // Assert stake balances for hotkey1 and coldkey1 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid_0
            ),
            stake_amount
        );
        // Assert stake balances for hotkey1 and coldkey2 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey2, netuid_0
            ),
            stake_amount
        );
        // Assert stake balances for hotkey2 and coldkey2 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2, &coldkey2, netuid_0
            ),
            stake_amount
        );
        // Assert stake balances for hotkey2 and coldkey3 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey2, &coldkey3, netuid_0
            ),
            stake_amount
        );
        // Assert total stake for hotkey1 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey1, netuid_0),
            2 * stake_amount
        );
        // Assert total stake for hotkey2 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey2, netuid_0),
            2 * stake_amount
        );
        // Increase stake for hotkey1 and coldkey1 on netuid_0
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey1,
            netuid_0,
            stake_amount,
        );
        // Assert updated stake for hotkey1 and coldkey1 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid_0
            ),
            2 * stake_amount
        );
        // Assert updated total stake for hotkey1 on netuid_0
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey1, netuid_0),
            3 * stake_amount
        );
        // Increase stake for hotkey1 and coldkey1 on netuid_1
        SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey1,
            &coldkey1,
            netuid_1,
            stake_amount,
        );
        // Assert updated stake for hotkey1 and coldkey1 on netuid_1
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey1, &coldkey1, netuid_1
            ),
            stake_amount
        );
        // Assert updated total stake for hotkey1 on netuid_1
        assert_eq!(
            SubtensorModule::get_stake_for_hotkey_on_subnet(&hotkey1, netuid_1),
            stake_amount
        );

        // Run the coinbase
        let emission: u64 = 1_000_000_000;
        SubtensorModule::run_coinbase(I96F32::from_num(emission));
        close(
            SubnetTaoInEmission::<Test>::get(netuid_1),
            emission / 6,
            100,
        );
        close(
            SubnetTaoInEmission::<Test>::get(netuid_2),
            2 * (emission / 6),
            100,
        );
        close(
            SubnetTaoInEmission::<Test>::get(netuid_3),
            3 * (emission / 6),
            100,
        );
    });
}

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
