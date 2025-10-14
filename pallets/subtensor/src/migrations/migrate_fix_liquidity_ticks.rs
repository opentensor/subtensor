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

    // Fix reserves for all subnets
    let netuids: Vec<NetUid> = NetworksAdded::<T>::iter()
        .map(|(netuid, _)| netuid)
        .collect();
    weight = weight.saturating_add(T::DbWeight::get().reads_writes(netuids.len() as u64, 0));

    for netuid in netuids.into_iter() {
        let (tao_reserve, alpha_reserve) = T::SwapInterface::migrate_get_implied_reserves(netuid, &mut weight);
        SubnetTaoProvided::<T>::insert(netuid, TaoCurrency::from(0));
        SubnetTAO::<T>::insert(netuid, tao_reserve);
        SubnetAlphaInProvided::<T>::insert(netuid, AlphaCurrency::from(0));
        SubnetAlphaIn::<T>::insert(netuid, alpha_reserve);
        weight = weight.saturating_add(T::DbWeight::get().writes(4));
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
