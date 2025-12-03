use crate::HasMigrationRun;
use crate::{Config, pallet};
use frame_support::traits::Get;
use frame_support::weights::Weight;
use scale_info::prelude::string::String;
use sp_std::collections::btree_map::BTreeMap;

pub const MAX_FEE_RATE: u16 = 1310; // 2 %

pub fn migrate_fee_rate<T: Config>() -> Weight {
    let migration_name = b"migrate_fee_rate".to_vec();

    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    // Check if the migration has already run
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

    let keypairs = pallet::FeeRate::<T>::iter().collect::<BTreeMap<_, _>>();
    weight = weight.saturating_add(T::DbWeight::get().reads(keypairs.len() as u64));

    for (netuid, rate) in keypairs {
        if rate > MAX_FEE_RATE {
            pallet::FeeRate::<T>::mutate(netuid, |rate| *rate = MAX_FEE_RATE);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
        }
    }

    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
