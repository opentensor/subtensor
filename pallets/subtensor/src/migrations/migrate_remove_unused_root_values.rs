use super::*;
use crate::HasMigrationRun;
use frame_support::{traits::Get, weights::Weight};
use scale_info::prelude::string::String;

pub fn migrate_remove_unused_root_values<T: Config>() -> Weight {
    let migration_name = b"migrate_remove_unused_root_values".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

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

    let netuid = 0;

    Incentive::<T>::remove(netuid);
    Dividends::<T>::remove(netuid);
    Rank::<T>::remove(netuid);
    Trust::<T>::remove(netuid);
    ValidatorTrust::<T>::remove(netuid);
    ValidatorPermit::<T>::remove(netuid);
    Consensus::<T>::remove(netuid);
    StakeWeight::<T>::remove(netuid);
    Active::<T>::remove(netuid);
    Emission::<T>::remove(netuid);
    PruningScores::<T>::remove(netuid);

    // Mark Migration as Completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(12));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
