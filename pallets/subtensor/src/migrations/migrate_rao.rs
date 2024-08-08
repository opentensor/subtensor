// use super::*;
// use frame_support::IterableStorageMap;
// use alloc::string::String;
// use frame_support::{traits::Get, weights::Weight};
// use log;

// pub fn migrate_rao<T: Config>() -> Weight {
//     let migration_name = b"migrate_rao".to_vec();

//     // Initialize the weight with one read operation.
//     let mut weight = T::DbWeight::get().reads(1);

//     // Check if the migration has already run
//     if HasMigrationRun::<T>::get(&migration_name) {
//         log::info!(
//             "Migration '{:?}' has already run. Skipping.",
//             migration_name
//         );
//         return weight;
//     }
//     log::info!(
//         "Running migration '{}'",
//         String::from_utf8_lossy(&migration_name)
//     );

//     let netuids: Vec<u16> = <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter().map(|(netuid, _)| netuid).collect();
//     weight = weight.saturating_add(T::DbWeight::get().reads_writes(netuids.len() as u64, 0));

//     // Set the mechanism to 0 (stable).
//     for netuid in netuids {
//         // Set all subnets to Stable.
//         SubnetMechanism::<T>::insert(netuid, 0);
//         weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

//         // Set the owner hotkey.
//         // FIX ina  hUGE WAY.
//         if netuid != 0 && SubnetOwner::<T>::contains_key( netuid ) {
//             // Owning coldkey
//             let owner_coldkey = SubnetOwner::<T>::get(netuid);
//             // Get the previous lock
//             let locked_tao = SubnetLocked::<T>::get(netuid);
//             // Set the lock to the new owner.
//             Locks::<T>::insert((netuid, owner_coldkey.clone(), owner_coldkey.clone()), (locked_tao, 0, Self::get_lock_interval_blocks()));
//             // 1 read and 1 write.
//             weight = weight.saturating_add(T::DbWeight::get().reads_writes(3, 1));
//         }

//     }

//     // Set all subnet stake to root.
//     // potentialls reset total stake.
//     // potentially create new StakingColdkeys map.
//     Stake::<T>::iter().for_each(|(hotkey, coldkey, stake)| {
//         // Increase SubnetTAO
//         SubnetTAO::<T>::mutate(netuid, |total| { *total = total.saturating_add(stake); });
//         // INcrease SubnetAlphaOut
//         SubnetAlphaOut::<T>::mutate(netuid, |total| { *total = total.saturating_add(stake); });
//         // Set all the stake on root 0 subnet.
//         Alpha::<T>::mutate( (hotkey.clone(), coldkey.clone(), 0), |root| *root = stake);
//         // Set the total stake on the coldkey
//         TotalColdkeyAlpha::<T>::mutate(coldkey.clone(), 0, |total| *total = stake);
//         // Set the total stake on the hotkey
//         TotalHotkeyAlpha::<T>::mutate(hotkey.clone(), 0, |total| *total = stake);
//         // 3 reads and 3 writes.
//         weight = weight.saturating_add(T::DbWeight::get().reads_writes(3, 3));
//     });

//     // Mark the migration as completed
//     HasMigrationRun::<T>::insert(&migration_name, true);
//     weight = weight.saturating_add(T::DbWeight::get().writes(1));

//     log::info!(
//         "Migration '{:?}' completed. Storage version set to 7.",
//         String::from_utf8_lossy(&migration_name)
//     );

//     // Return the migration weight.
//     weight
// }
