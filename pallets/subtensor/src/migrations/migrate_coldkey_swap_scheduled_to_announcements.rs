use super::*;
use crate::AccountIdOf;
use frame_support::{pallet_prelude::Blake2_128Concat, traits::Get, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::prelude::string::String;
use sp_io::storage::clear;
use sp_runtime::traits::Hash;

pub mod deprecated {
    use super::*;
    use frame_support::storage_alias;

    #[storage_alias]
    pub type ColdkeySwapScheduleDuration<T: Config> =
        StorageValue<Pallet<T>, BlockNumberFor<T>, OptionQuery>;

    #[storage_alias]
    pub type ColdkeySwapRescheduleDuration<T: Config> =
        StorageValue<Pallet<T>, BlockNumberFor<T>, OptionQuery>;

    #[storage_alias]
    pub type ColdkeySwapScheduled<T: Config> = StorageMap<
        Pallet<T>,
        Blake2_128Concat,
        AccountIdOf<T>,
        (BlockNumberFor<T>, AccountIdOf<T>),
        OptionQuery,
    >;
}

pub fn migrate_coldkey_swap_scheduled_to_announcements<T: Config>() -> Weight {
    let migration_name = b"migrate_coldkey_swap_scheduled_to_announcements".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    // Remove ColdkeySwapScheduleDuration and ColdkeySwapRescheduleDuration
    let pallet_name = twox_128(b"SubtensorModule");
    let storage_name1 = twox_128(b"ColdkeySwapScheduleDuration");
    let storage_name2 = twox_128(b"ColdkeySwapRescheduleDuration");
    clear(&[pallet_name, storage_name1].concat());
    clear(&[pallet_name, storage_name2].concat());
    weight.saturating_accrue(T::DbWeight::get().writes(2));

    // Migrate the ColdkeySwapScheduled entries to ColdkeySwapAnnouncements entries
    let now = <frame_system::Pallet<T>>::block_number();
    let scheduled = deprecated::ColdkeySwapScheduled::<T>::iter();
    let delay = ColdkeySwapAnnouncementDelay::<T>::get();

    for (who, (when, new_coldkey)) in scheduled {
        // Only migrate the scheduled coldkey swaps that are in the future
        if when > now {
            let coldkey_hash = <T as frame_system::Config>::Hashing::hash_of(&new_coldkey);
            // The announcement should be at the scheduled time - delay to be able to call
            // the swap_coldkey_announced call at the old scheduled time
            ColdkeySwapAnnouncements::<T>::insert(who, (when - delay, coldkey_hash));
            weight.saturating_accrue(T::DbWeight::get().writes(1));
        }
        weight.saturating_accrue(T::DbWeight::get().reads(1));
    }

    let results = deprecated::ColdkeySwapScheduled::<T>::clear(u32::MAX, None);
    weight.saturating_accrue(
        T::DbWeight::get().reads_writes(results.loops as u64, results.backend as u64),
    );

    // ------------------------------
    // Step 2: Mark Migration as Completed
    // ------------------------------

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
