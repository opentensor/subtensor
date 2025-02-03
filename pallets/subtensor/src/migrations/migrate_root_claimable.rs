use super::*;
use alloc::string::String;
use frame_support::{traits::Get, weights::Weight};
use log;

fn do_migration_<T: Config>() -> Weight {
    let root_netuid: u16 = Pallet::<T>::get_root_netuid();
    let curr_num_coldkeys = NumColdkeys::<T>::get();
    let mut weight = T::DbWeight::get().reads(2);

    if curr_num_coldkeys > 0 {
        log::info!("Num Coldkeys: {}", curr_num_coldkeys);
        log::error!("This is unexpected, num coldkeys is greater than 0");
        return weight; // Don't run the rest of the migration.
    }

    let mut index = 0;

    // Get all the coldkeys that have stake.
    for (coldkey, _hotkeys) in StakingHotkeys::<T>::iter() {
        let mut has_root_stake = false;
        for hotkey in _hotkeys {
            let root_stake = Alpha::<T>::get((hotkey, coldkey, root_netuid));
            weight = weight.saturating_add(T::DbWeight::get().reads(1));

            if root_stake > 0 {
                has_root_stake = true;
                break;
            }
        }

        if !has_root_stake {
            continue;
        }

        weight = weight.saturating_add(T::DbWeight::get().reads(1));
        if !StakingColdkeys::<T>::contains_key(&coldkey) {
            StakingColdkeys::<T>::insert(coldkey.clone(), index);
            ColdkeysIndex::<T>::insert(index, coldkey.clone());
            index = index.saturating_add(1);
            weight = weight.saturating_add(T::DbWeight::get().writes(2));
        }
    }

    NumColdkeys::<T>::set(index);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    weight
}

pub fn migrate_root_claimable<T: Config>() -> Weight {
    let migration_name = b"migrate_root_claimable".to_vec();

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

    weight = weight.saturating_add(do_migration_::<T>());

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
