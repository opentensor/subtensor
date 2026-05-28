use alloc::string::String;
use frame_support::{traits::Get, weights::Weight};

use crate::{Config, HasMigrationRun, Lock, Pallet as Subtensor, staking::lock::ConvictionModel};

const MIGRATION_NAME: &[u8] = b"migrate_populate_locking_coldkeys";

pub fn migrate_populate_locking_coldkeys<T: Config>() -> Weight {
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(MIGRATION_NAME) {
        log::info!(
            "Migration '{}' already executed - skipping",
            String::from_utf8_lossy(MIGRATION_NAME)
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(MIGRATION_NAME)
    );

    let now = Subtensor::<T>::get_current_block_as_u64();
    let unlock_rate = crate::UnlockRate::<T>::get();
    let maturity_rate = crate::MaturityRate::<T>::get();
    let mut scanned_count = 0u64;
    let mut indexed_count = 0u64;
    let mut removed_count = 0u64;
    let mut locks_to_remove = sp_std::vec::Vec::new();

    for ((coldkey, netuid, hotkey), lock) in Lock::<T>::iter() {
        scanned_count = scanned_count.saturating_add(1);
        let rolled = ConvictionModel::roll_forward_lock(
            lock,
            now,
            unlock_rate,
            maturity_rate,
            Subtensor::<T>::is_subnet_owner_hotkey(netuid, &hotkey),
            Subtensor::<T>::is_perpetual_lock(&coldkey, netuid),
        );

        if !rolled.is_zero() {
            Subtensor::<T>::add_locking_coldkey(&hotkey, netuid, &coldkey);
            indexed_count = indexed_count.saturating_add(1);
        } else {
            locks_to_remove.push((coldkey, netuid, hotkey));
        }
    }

    for (coldkey, netuid, hotkey) in locks_to_remove {
        Lock::<T>::remove((coldkey, netuid, hotkey));
        removed_count = removed_count.saturating_add(1);
    }

    weight = weight.saturating_add(T::DbWeight::get().reads(scanned_count));
    weight = weight
        .saturating_add(T::DbWeight::get().writes(indexed_count.saturating_add(removed_count)));

    HasMigrationRun::<T>::insert(MIGRATION_NAME, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{}' completed. scanned_entries={}, indexed_entries={}, removed_zero_entries={}",
        String::from_utf8_lossy(MIGRATION_NAME),
        scanned_count,
        indexed_count,
        removed_count
    );

    weight
}
