use super::*;
use crate::HasMigrationRun;
use codec::Decode;
use frame_support::weights::Weight;
use sp_io::hashing::twox_128;
use sp_io::storage::{clear, get};

/// Remove the deprecated OwnerHyperparamRateLimit storage item.
/// If the old value was 0 (disabled), preserve that by setting OwnerHyperparamTempos to 0.
/// Otherwise, leave the new storage at its default (2 tempos).
pub fn migrate_owner_hyperparam_rl_to_tempos<T: Config>() -> Weight {
    let migration_name = b"migrate_owner_hyperparam_rl_to_tempos".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!("Migration '{:?}' already executed. Skipping.", migration_name);
        return weight;
    }

    let pallet_name = twox_128("SubtensorModule".as_bytes());
    let storage_name = twox_128("OwnerHyperparamRateLimit".as_bytes());
    let full_key = [pallet_name, storage_name].concat();

    if let Some(value_bytes) = get(&full_key) {
        if let Ok(old_limit_blocks) = <u64 as Decode>::decode(&mut &value_bytes[..]) {
            if old_limit_blocks == 0u64 {
                // Preserve disabled state
                Pallet::<T>::set_owner_hyperparam_tempos(0);
            }
        }

        clear(&full_key);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));
    weight
}
