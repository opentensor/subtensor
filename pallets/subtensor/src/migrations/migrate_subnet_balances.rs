use super::*;
use frame_support::{
    traits::{Get, fungible::Inspect},
    weights::Weight,
};

/// Performs migration to mint SubnetTAO and subnet locked funds into subnet accounts.
///
/// # Arguments
///
/// # Returns
///
/// * `Weight` - The computational weight of this operation.
///
pub fn migrate_subnet_balances<T: Config>() -> Weight {
    let migration_name = b"migrate_subnet_balances".to_vec();
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

    // Mint SubnetTAO into subnet accounts
    let mut total_minted = TaoBalance::ZERO;
    SubnetTAO::<T>::iter().for_each(|(netuid, tao)| {
        if let Some(subnet_account) = Pallet::<T>::get_subnet_account_id(netuid) {
            let credit = Pallet::<T>::mint_tao(tao);
            let _ = Pallet::<T>::spend_tao(&subnet_account, credit, tao);
            total_minted = total_minted.saturating_add(tao);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));
        }
    });

    // Mint SubnetLocked into subnet accounts
    SubnetLocked::<T>::iter().for_each(|(netuid, tao)| {
        if let Some(subnet_account) = Pallet::<T>::get_subnet_account_id(netuid) {
            let credit = Pallet::<T>::mint_tao(tao);
            let _ = Pallet::<T>::spend_tao(&subnet_account, credit, tao);
            total_minted = total_minted.saturating_add(tao);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));
        }
    });

    // mint_tao increases subtensor TotalIssuance, but this is not the intention here because 
    // SubnetTAO and SubnetLocked are already accounted in it. Reduce it back.
    TotalIssuance::<T>::mutate(|total| {
        *total = total.saturating_sub(total_minted);
    });

    // Update the total issuance in storage
    let balances_total_issuance = <T as Config>::Currency::total_issuance();
    let subtensor_total_issuance = TotalIssuance::<T>::get();
    weight = weight.saturating_add(T::DbWeight::get().reads(2));
    if balances_total_issuance != subtensor_total_issuance {
        log::warn!(
            "Balances and Subtensor total issuance still do not match: {} vs {}. Making them match now.",
            balances_total_issuance,
            subtensor_total_issuance
        );
        TotalIssuance::<T>::put(balances_total_issuance);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
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
