use super::*;
use crate::HasMigrationRun;
use frame_support::{storage_alias, traits::Get, weights::Weight};
use scale_info::prelude::string::String;
use substrate_fixed::types::U64F64;

pub mod deprecated_swap_maps {
    use super::*;

    #[storage_alias]
    pub type AlphaSqrtPrice<T: Config> =
        StorageMap<Pallet<T>, Twox64Concat, NetUid, U64F64, ValueQuery>;

    /// TAO reservoir for scraps of protocol claimed fees.
    #[storage_alias]
    pub type ScrapReservoirTao<T: Config> =
        StorageMap<Pallet<T>, Twox64Concat, NetUid, TaoCurrency, ValueQuery>;

    /// Alpha reservoir for scraps of protocol claimed fees.
    #[storage_alias]
    pub type ScrapReservoirAlpha<T: Config> =
        StorageMap<Pallet<T>, Twox64Concat, NetUid, AlphaCurrency, ValueQuery>;
}

pub fn migrate_swapv3_to_balancer<T: Config>() -> Weight {
    let migration_name = BoundedVec::truncate_from(b"migrate_swapv3_to_balancer".to_vec());
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
        String::from_utf8_lossy(&migration_name),
    );

    // ------------------------------
    // Step 1: Clear Map entries
    // ------------------------------
    remove_prefix::<T>("Swap", "AlphaSqrtPrice", &mut weight);
    remove_prefix::<T>("Swap", "CurrentTick", &mut weight);
    remove_prefix::<T>("Swap", "FeeGlobalTao", &mut weight);
    remove_prefix::<T>("Swap", "FeeGlobalAlpha", &mut weight);
    // Scrap reservoirs can be just cleaned because they are already included in reserves
    remove_prefix::<T>("Swap", "ScrapReservoirTao", &mut weight);
    remove_prefix::<T>("Swap", "ScrapReservoirAlpha", &mut weight);
    remove_prefix::<T>("Swap", "Ticks", &mut weight);
    remove_prefix::<T>("Swap", "TickIndexBitmapWords", &mut weight);
    remove_prefix::<T>("Swap", "SwapV3Initialized", &mut weight);
    remove_prefix::<T>("Swap", "CurrentLiquidity", &mut weight);
    remove_prefix::<T>("Swap", "Positions", &mut weight);

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
