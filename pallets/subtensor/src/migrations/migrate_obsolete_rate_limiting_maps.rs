use crate::Vec;
use crate::{Config, HasMigrationRun, NetworksAdded, Pallet};
use alloc::string::String;
use frame_support::IterableStorageMap;
use frame_support::traits::Get;
use frame_support::weights::Weight;

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

    for netuid in netuids.into_iter() {
        let rate_limit = crate::ServingRateLimit::<T>::get(netuid);
        Pallet::<T>::set_serving_rate_limit(netuid, rate_limit, true);
        crate::ServingRateLimit::<T>::remove(netuid);

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
