use crate::Vec;
use crate::{Config, HasMigrationRun, NetworksAdded, Pallet};
use alloc::string::String;
use codec::{Decode, Encode};
use frame_support::IterableStorageMap;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use sp_io::hashing::twox_128;
use sp_io::storage::{clear, get};

#[allow(deprecated)]
pub fn migrate_obsolete_rate_limiting_maps<T: Config>() -> Weight {
    let migration_name = b"migrate_obsolete_rate_limiting_maps".to_vec();

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

    let pallet_name = "SubtensorModule";
    let storage_name = "ServingRateLimit";
    let pallet_name_hash = twox_128(pallet_name.as_bytes());
    let storage_name_hash = twox_128(storage_name.as_bytes());
    let prefix = [pallet_name_hash, storage_name_hash].concat();

    for netuid in netuids.into_iter() {
        let mut encoded_netuid = netuid.encode();
        let mut full_key = prefix.clone();

        full_key.append(&mut encoded_netuid);

        if let Some(value_bytes) = get(&full_key) {
            if let Ok(rate_limit) = Decode::decode(&mut &value_bytes[..]) {
                Pallet::<T>::set_serving_rate_limit(netuid, rate_limit, true);
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
