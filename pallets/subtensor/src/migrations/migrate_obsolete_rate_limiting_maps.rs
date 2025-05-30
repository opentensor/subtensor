use crate::Vec;
use crate::{Config, HasMigrationRun, NetworksAdded, Pallet};
use alloc::string::String;
use codec::{Decode, Encode};
use frame_support::IterableStorageMap;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use sp_io::hashing::twox_128;
use sp_io::storage::{clear, get};

pub fn migrate_obsolete_rate_limiting_maps<T: Config>() -> Weight {
    migrate_serving_rate_limits::<T>()
        .saturating_add(migrate_tx_rate_limits::<T>())
        .saturating_add(migrate_set_weights_rate_limits::<T>())
        .saturating_add(migrate_network_rate_limits::<T>())
}

pub fn migrate_network_rate_limits<T: Config>() -> Weight {
    let migration_name = b"migrate_network_rate_limits".to_vec();
    let pallet_name = "SubtensorModule";
    let storage_name = "NetworkRateLimit";

    migrate_value::<T, _>(migration_name, pallet_name, storage_name, |limit| {
        Pallet::<T>::set_network_rate_limit(limit, true);
    })
}
pub fn migrate_tx_rate_limits<T: Config>() -> Weight {
    let migration_name = b"migrate_tx_rate_limits".to_vec();
    let pallet_name = "SubtensorModule";
    let storage_name = "TxRateLimit";

    migrate_value::<T, _>(migration_name, pallet_name, storage_name, |limit| {
        Pallet::<T>::set_tx_rate_limit(limit, true);
    })
}

pub fn migrate_serving_rate_limits<T: Config>() -> Weight {
    let migration_name = b"migrate_serving_rate_limits".to_vec();
    let pallet_name = "SubtensorModule";
    let storage_name = "ServingRateLimit";

    migrate_limit_map_netuids::<T, _, _>(
        migration_name,
        pallet_name,
        storage_name,
        |netuid| netuid.encode(),
        |netuid, limit| {
            Pallet::<T>::set_serving_rate_limit(netuid, limit, true);
        },
    )
}

pub fn migrate_set_weights_rate_limits<T: Config>() -> Weight {
    let migration_name = b"migrate_set_weights_rate_limits".to_vec();
    let pallet_name = "SubtensorModule";
    let storage_name = "WeightsSetRateLimit";

    migrate_limit_map_netuids::<T, _, _>(
        migration_name,
        pallet_name,
        storage_name,
        |netuid| netuid.encode(),
        |netuid, limit| {
            Pallet::<T>::set_weights_set_rate_limit(netuid, limit, true);
        },
    )
}

fn migrate_limit_map_netuids<T, KeyFunction, SetValueFunction>(
    migration_name: Vec<u8>,
    pallet_name: &str,
    storage_name: &str,
    key: KeyFunction,
    set_value: SetValueFunction,
) -> Weight
where
    T: Config,
    KeyFunction: Fn(u16 /*netuid*/) -> Vec<u8>,
    SetValueFunction: Fn(u16 /*netuid*/, u64 /*limit in blocks*/),
{
    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    // Check if the migration has already run
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            migration_name
        );
        return weight;
    }
    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    let netuids: Vec<u16> = <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter()
        .map(|(netuid, _)| netuid)
        .collect();
    weight = weight.saturating_add(T::DbWeight::get().reads(netuids.len() as u64));

    let pallet_name_hash = twox_128(pallet_name.as_bytes());
    let storage_name_hash = twox_128(storage_name.as_bytes());
    let prefix = [pallet_name_hash, storage_name_hash].concat();

    for netuid in netuids.into_iter() {
        let mut full_key = prefix.clone();
        let mut key = key(netuid);

        full_key.append(&mut key);

        if let Some(value_bytes) = get(&full_key) {
            if let Ok(rate_limit) = Decode::decode(&mut &value_bytes[..]) {
                set_value(netuid, rate_limit);
            }

            clear(&full_key);
        }

        weight = weight.saturating_add(T::DbWeight::get().writes(2));
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
    }

    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed.",
        String::from_utf8_lossy(&migration_name)
    );

    // Return the migration weight.
    weight
}

fn migrate_value<T, SetValueFunction>(
    migration_name: Vec<u8>,
    pallet_name: &str,
    storage_name: &str,
    set_value: SetValueFunction,
) -> Weight
where
    T: Config,
    SetValueFunction: Fn(u64 /*limit in blocks*/),
{
    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    // Check if the migration has already run
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            migration_name
        );
        return weight;
    }
    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    let pallet_name_hash = twox_128(pallet_name.as_bytes());
    let storage_name_hash = twox_128(storage_name.as_bytes());
    let full_key = [pallet_name_hash, storage_name_hash].concat();

    if let Some(value_bytes) = get(&full_key) {
        if let Ok(rate_limit) = Decode::decode(&mut &value_bytes[..]) {
            set_value(rate_limit);
        }

        clear(&full_key);
    }

    weight = weight.saturating_add(T::DbWeight::get().writes(2));
    weight = weight.saturating_add(T::DbWeight::get().reads(1));

    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed.",
        String::from_utf8_lossy(&migration_name)
    );

    // Return the migration weight.
    weight
}
