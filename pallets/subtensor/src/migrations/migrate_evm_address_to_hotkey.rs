use super::*;
use frame_support::{
    pallet_prelude::{OptionQuery, Twox64Concat},
    storage_alias,
    traits::Get,
    weights::Weight,
};
use log::info;
use sp_core::H160;

const LOG_TARGET: &str = "migrate_evm_address_to_hotkey";

/// Module containing deprecated storage format for AssociatedEvmAddress
pub mod deprecated_evm_address_format {
    use super::*;

    #[storage_alias]
    pub(super) type AssociatedEvmAddress<T: Config> =
        StorageDoubleMap<Pallet<T>, Twox64Concat, u16, Twox64Concat, u16, (H160, u64), OptionQuery>;
}

/// Migrate AssociatedEvmAddress from (netuid, uid) -> (evm_address, block) to (netuid, hotkey) -> (evm_address, block)
pub fn migrate_evm_address_to_hotkey<T: Config>() -> Weight {
    let mut weight = T::DbWeight::get().reads(1);
    let migration_name = "Migrate AssociatedEvmAddress from UID to Hotkey";
    let migration_name_bytes = migration_name.as_bytes().to_vec();

    // Check if migration has already run
    if HasMigrationRun::<T>::get(&migration_name_bytes) {
        info!(
            target: LOG_TARGET,
            "Migration '{}' has already run. Skipping.",
            migration_name
        );
        return Weight::zero();
    }

    info!(target: LOG_TARGET, ">>> Starting Migration: {}", migration_name);

    let mut migrated_count: u64 = 0;
    let mut orphaned_count: u64 = 0;
    let mut storage_reads: u64 = 0;
    let mut storage_writes: u64 = 0;

    // Create a vector to store the old entries
    let mut old_entries = Vec::new();

    // Read all old entries
    deprecated_evm_address_format::AssociatedEvmAddress::<T>::iter().for_each(
        |(netuid, uid, (evm_address, block))| {
            old_entries.push((netuid, uid, evm_address, block));
            storage_reads = storage_reads.saturating_add(1);
        },
    );

    weight = weight.saturating_add(T::DbWeight::get().reads(old_entries.len() as u64));

    // Clear the old storage
    let _ = deprecated_evm_address_format::AssociatedEvmAddress::<T>::clear(u32::MAX, None);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Migrate each entry
    for (netuid, uid, evm_address, block) in old_entries {
        // Look up the hotkey for this uid on this subnet
        if let Ok(hotkey) = Keys::<T>::try_get(netuid, uid) {
            storage_reads = storage_reads.saturating_add(1);

            // Store with the new format using hotkey
            AssociatedEvmAddress::<T>::insert(netuid, &hotkey, (evm_address, block));
            storage_writes = storage_writes.saturating_add(1);
            migrated_count = migrated_count.saturating_add(1);

            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

            info!(
                target: LOG_TARGET,
                "Migrated EVM address {} from UID {} to hotkey {:?} on subnet {}",
                evm_address, uid, hotkey, netuid
            );
        } else {
            // No hotkey found for this UID - the neuron may have been deregistered
            orphaned_count = orphaned_count.saturating_add(1);
            weight = weight.saturating_add(T::DbWeight::get().reads(1));

            info!(
                target: LOG_TARGET,
                "WARNING: Orphaned EVM address {} for UID {} on subnet {} - no hotkey found",
                evm_address, uid, netuid
            );
        }
    }

    // Mark migration as complete
    HasMigrationRun::<T>::insert(&migration_name_bytes, true);
    storage_writes = storage_writes.saturating_add(1);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    info!(
        target: LOG_TARGET,
        "Migration {} finished. Migrated: {}, Orphaned: {}, Storage reads: {}, Storage writes: {}",
        migration_name, migrated_count, orphaned_count, storage_reads, storage_writes
    );

    weight
}
