use alloc::string::String;
use frame_support::{BoundedVec, traits::Get, weights::Weight};

use crate::*;

pub fn migrate_add_contributors_count<T: Config>() -> Weight {
    let migration_name =
        BoundedVec::truncate_from(b"migrate_crowdloan_contributors_count".to_vec());
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

    // Get all crowdloans, there is not so many at the moment so we are safe.
    let crowdloan_ids = Crowdloans::<T>::iter_keys().collect::<Vec<_>>();
    weight = weight.saturating_add(T::DbWeight::get().reads(crowdloan_ids.len() as u64));

    for crowdloan_id in crowdloan_ids {
        let contributions = Contributions::<T>::iter_key_prefix(crowdloan_id)
            .collect::<Vec<_>>()
            .len();
        weight = weight.saturating_add(T::DbWeight::get().reads(contributions as u64));

        ContributorsCount::<T>::insert(crowdloan_id, contributions as u32);
    }

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
