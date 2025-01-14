use super::*;
use alloc::string::String;
use frame_support::{traits::Get, weights::Weight};
use log;

pub fn migrate_fix_is_network_member<T: Config>() -> Weight {
    let migration_name = b"migrate_fix_is_network_member".to_vec();

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

    weight = do_fix_is_network_member::<T>(weight);

    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed. Storage version set to 7.",
        String::from_utf8_lossy(&migration_name)
    );

    // Return the migration weight.
    weight
}

fn do_fix_is_network_member<T: Config>(weight: Weight) -> Weight {
    // Clear the IsNetworkMember storage
    let mut curr = IsNetworkMember::<T>::clear(U32::MAX, None);
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(curr.loops, curr.unique));
    while curr.maybe_cursor.is_some() {
        // Clear until empty
        curr = IsNetworkMember::<T>::clear(U32::MAX, curr.maybe_cursor);
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(curr.loops, curr.unique));
    }
    // Repopulate the IsNetworkMember storage using the Keys map
    let netuids = Subtensor::<T>::get_all_subnet_netuids();
    for netuid in netuids {
        for key in Keys::<T>::iter_prefix(netuid) {
            IsNetworkMember::<T>::insert(key, netuid, true);
        }
    }

    weight
}
