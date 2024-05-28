// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// Copyright (C) 2024 Opentensor Technologies Inc.

// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::*;
use sp_io::hashing::twox_128;

use frame_support::{
    traits::{
        Get, GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion,
        STORAGE_VERSION_STORAGE_KEY_POSTFIX,
    },
    weights::Weight,
};

const NEW_STORAGE_VERSION: u16 = 5;
const OLD_PALLET_NAME: &str = "triumvirate";
const TARGET: &str = "runtime::collective";

/// Migrate the entire storage of this pallet to a new prefix.
///
/// This new prefix must be the same as the one set in construct_runtime. For safety, use
/// `PalletInfo` to get it, as:
/// `<Runtime as frame_system::Config>::PalletInfo::name::<CollectivePallet>`.
///
/// The migration will look into the storage version in order not to trigger a migration on an up
/// to date storage. Thus the on chain storage version must be less than `NEW_STORAGE_VERSION` in order to trigger the
/// migration.
pub fn migrate<T: frame_system::Config, P: GetStorageVersion + PalletInfoAccess, N: AsRef<str>>(
    old_pallet_name: N,
) -> Weight {
    let old_pallet_name = old_pallet_name.as_ref();
    let new_pallet_name = <P as PalletInfoAccess>::name();

    if new_pallet_name == old_pallet_name {
        log::info!(
            target: TARGET,
            "New pallet name is equal to the old pallet name. No migration needs to be done.",
        );
        return Weight::zero();
    }

    let on_chain_storage_version = <P as GetStorageVersion>::on_chain_storage_version();
    log::info!(
        target: TARGET,
        "Running migration to v{:?} for collective with storage version {:?}",
        NEW_STORAGE_VERSION,
        on_chain_storage_version,
    );

    if on_chain_storage_version < NEW_STORAGE_VERSION {
        frame_support::storage::migration::move_pallet(
            old_pallet_name.as_bytes(),
            new_pallet_name.as_bytes(),
        );
        log_migration("migration", old_pallet_name, new_pallet_name);

        // Update the storage version.
        StorageVersion::new(NEW_STORAGE_VERSION).put::<P>();
        <T as frame_system::Config>::BlockWeights::get().max_block
    } else {
        log::warn!(
            target: TARGET,
            "Attempted to apply migration to v{:?} but failed because storage version is {:?}",
            NEW_STORAGE_VERSION,
            on_chain_storage_version,
        );
        Weight::zero()
    }
}

/// Some checks prior to migration. This can be linked to
/// [`frame_support::traits::OnRuntimeUpgrade::pre_upgrade`] for further testing.
///
/// Panics if anything goes wrong.
pub fn pre_migrate<P: GetStorageVersion + PalletInfoAccess, N: AsRef<str>>(old_pallet_name: N) {
    let old_pallet_name = old_pallet_name.as_ref();
    let new_pallet_name = <P as PalletInfoAccess>::name();
    log_migration("pre-migration", old_pallet_name, new_pallet_name);

    if new_pallet_name == old_pallet_name {
        return;
    }

    let new_pallet_prefix = twox_128(new_pallet_name.as_bytes());
    let storage_version_key = twox_128(STORAGE_VERSION_STORAGE_KEY_POSTFIX);

    let mut new_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
        new_pallet_prefix.to_vec(),
        new_pallet_prefix.to_vec(),
        |key| Ok(key.to_vec()),
    );

    // Ensure nothing except the storage_version_key is stored in the new prefix.
    assert!(new_pallet_prefix_iter.all(|key| key == storage_version_key));

    assert!(<P as GetStorageVersion>::on_chain_storage_version() < NEW_STORAGE_VERSION);
}

/// Some checks for after migration. This can be linked to
/// [`frame_support::traits::OnRuntimeUpgrade::post_upgrade`] for further testing.
///
/// Panics if anything goes wrong.
pub fn post_migrate<P: GetStorageVersion + PalletInfoAccess, N: AsRef<str>>(old_pallet_name: N) {
    let old_pallet_name = old_pallet_name.as_ref();
    let new_pallet_name = <P as PalletInfoAccess>::name();
    log_migration("post-migration", old_pallet_name, new_pallet_name);

    if new_pallet_name == old_pallet_name {
        return;
    }

    // Assert that nothing remains at the old prefix.
    let old_pallet_prefix = twox_128(old_pallet_name.as_bytes());
    let old_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
        old_pallet_prefix.to_vec(),
        old_pallet_prefix.to_vec(),
        |_| Ok(()),
    );
    assert_eq!(old_pallet_prefix_iter.count(), 0);

    // NOTE: storage_version_key is already in the new prefix.
    let new_pallet_prefix = twox_128(new_pallet_name.as_bytes());
    let new_pallet_prefix_iter = frame_support::storage::KeyPrefixIterator::new(
        new_pallet_prefix.to_vec(),
        new_pallet_prefix.to_vec(),
        |_| Ok(()),
    );
    assert!(new_pallet_prefix_iter.count() >= 1);

    assert_eq!(
        <P as GetStorageVersion>::on_chain_storage_version(),
        NEW_STORAGE_VERSION
    );
}

fn log_migration(stage: &str, old_pallet_name: &str, new_pallet_name: &str) {
    log::info!(
        target: TARGET,
        "{}, prefix: '{}' ==> '{}'",
        stage,
        old_pallet_name,
        new_pallet_name,
    );
}

pub struct MigrateName<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateName<T> {
    /// Run pre-migrate
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
        use std::vec;

        pre_migrate(OLD_PALLET_NAME);
        Ok(vec![]) // No pre-upgrade info to save.
    }

    /// Run migration
    fn on_runtime_upgrade() -> Weight {
        // Skip migration if it already took place.
        if migration_already_occured::<T>() {
            log::warn!(
            target: TARGET,
            "Migration already completed and can be removed.",
            );
            return <T as frame_system::Config>::DbWeight::get().reads_writes(0u64, 0u64);
        }

        log::info!(target: TARGET, "Migration not yet completed. Running migration...");

        migrate::<T, Pallet<T>, &str>(OLD_PALLET_NAME);

        // R/W not important for solo chain.
        <T as frame_system::Config>::DbWeight::get().reads_writes(0u64, 0u64)
    }

    // Run post-migrate
    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        log::info!(target: TARGET, "Running post-upgrade...");

        post_migrate::<T, &str>(OLD_PALLET_NAME);

        Ok(())
    }
}

fn migration_already_occured<T: Config>() -> bool {
    // Check if the storage version is already set.
    Pallet::<T>::on_chain_storage_version() >= StorageVersion::new(NEW_STORAGE_VERSION)
}
