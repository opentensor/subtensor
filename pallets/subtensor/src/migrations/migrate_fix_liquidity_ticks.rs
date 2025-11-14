use super::*;
use alloc::string::String;
use subtensor_swap_interface::SwapHandler;

/// Migrate and fix LP ticks that saturated
pub fn migrate_fix_liquidity_ticks<T: Config>() -> Weight {
    let migration_name = b"migrate_fix_liquidity_ticks".to_vec();
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

    ////////////////////////////////////////////////////////
    // Actual migration

    T::SwapInterface::migrate_fix_liquidity_ticks(&mut weight);
    T::SwapInterface::migrate_fix_current_liquidity(&mut weight);

    // Fix protocol liquidity for all subnets
    let netuids: Vec<NetUid> = NetworksAdded::<T>::iter()
        .map(|(netuid, _)| netuid)
        .collect();
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(netuids.len() as u64, 0));

    for netuid in netuids.into_iter() {
        T::SwapInterface::migrate_fix_protocol_liquidity(netuid, &mut weight);
    }

    ////////////////////////////////////////////////////////

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );
    weight
}
