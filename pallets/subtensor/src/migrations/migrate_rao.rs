use super::*;
use frame_support::IterableStorageMap;
use alloc::string::String;
use frame_support::{traits::Get, weights::Weight};
use log;

pub fn migrate_rao<T: Config>() -> Weight {
    let migration_name = b"migrate_rao".to_vec();

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

    let netuids: Vec<u16> = <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter().map(|(netuid, _)| netuid).collect();
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(netuids.len() as u64, 0));

    // Migrate all TAO to root.
    Stake::<T>::iter().for_each(|(hotkey, coldkey, stake)| {
        // Increase SubnetTAO on root.
        SubnetTAO::<T>::mutate( 0, |total| { *total = total.saturating_add(stake); });
        // Increase SubnetAlphaOut on root.
        SubnetAlphaOut::<T>::mutate( 0, |total| { *total = total.saturating_add(stake); });
        // Increase SubnetAlphaIn on root.
        SubnetAlphaIn::<T>::mutate( 0, |total| { *total = total.saturating_add(stake); });
        // Set all the stake on root 0 subnet.
        Alpha::<T>::mutate( (hotkey.clone(), coldkey.clone(), 0), |total| { *total = total.saturating_add(stake) } );
        // Set the total stake on the coldkey
        TotalColdkeyAlpha::<T>::mutate(coldkey.clone(), 0, |total| { *total = total.saturating_add(stake) } );
        // Set the total stake on the hotkey
        TotalHotkeyAlpha::<T>::mutate(hotkey.clone(), 0, |total| { *total = total.saturating_add(stake) } );
        // 3 reads and 3 writes.
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(3, 3));
    });

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
