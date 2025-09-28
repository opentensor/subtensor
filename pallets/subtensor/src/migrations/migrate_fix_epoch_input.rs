use super::*;
use frame_support::{
    pallet_prelude::{Identity, OptionQuery},
    storage_alias,
    traits::Get,
    weights::Weight,
};
use log::info;
use sp_std::vec::Vec;

const LOG_TARGET_1: &str = "migrate_fix_epoch_input";

/// Remove duplicate instances when one hotkey appears more than onces in Keys map
pub fn deduplicate_hotkeys<T: Config>() -> Weight {
    let mut weight = T::DbWeight::get().reads(0);
    // Iterate over Keys map, detect duplicate hotkey
    // and build the list of uids that need to be removed

    // For each uid to be removed swap all neuron maps with the highest uid and 
    // deregister the neuron (to prevent gaps in uid numbers in the subnet)


    weight
}

/// Fix any known inconsistensies in epoch input data
/// This migration should execute regularly
pub fn migrate_fix_epoch_input<T: Config>() -> Weight {
    // Setup migration weight
    let mut weight = T::DbWeight::get().reads(1);
    let migration_name = "Fix epoch input data";

    info!(target: LOG_TARGET_1, ">>> Starting Migration: {migration_name}");

    // // Iterate through all Owner entries
    // Owner::<T>::iter().for_each(|(hotkey, coldkey)| {
    //     storage_reads = storage_reads.saturating_add(1); // Read from Owner storage
    //     let mut hotkeys = OwnedHotkeys::<T>::get(&coldkey);
    //     storage_reads = storage_reads.saturating_add(1); // Read from OwnedHotkeys storage

    //     // Add the hotkey if it's not already in the vector
    //     if !hotkeys.contains(&hotkey) {
    //         hotkeys.push(hotkey);
    //         keys_touched = keys_touched.saturating_add(1);

    //         // Update longest hotkey vector info
    //         if longest_hotkey_vector < hotkeys.len() {
    //             longest_hotkey_vector = hotkeys.len();
    //             longest_coldkey = Some(coldkey.clone());
    //         }

    //         // Update the OwnedHotkeys storage
    //         OwnedHotkeys::<T>::insert(&coldkey, hotkeys);
    //         storage_writes = storage_writes.saturating_add(1); // Write to OwnedHotkeys storage
    //     }

    //     // Accrue weight for reads and writes
    //     weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 1));
    // });

    // Log migration results
    // info!(
    //     target: LOG_TARGET_1,
    //     "Migration {migration_name} finished. Keys touched: {keys_touched}, Longest hotkey vector: {longest_hotkey_vector}, Storage reads: {storage_reads}, Storage writes: {storage_writes}"
    // );
    // if let Some(c) = longest_coldkey {
    //     info!(target: LOG_TARGET_1, "Longest hotkey vector is controlled by: {c:?}");
    // }

    weight
}
