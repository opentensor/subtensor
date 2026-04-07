use super::*;
use crate::migrations::migrate_subnet_locked::SUBNET_LOCKED;
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
    let mut total_subnet_tao = TaoBalance::ZERO;
    SubnetTAO::<T>::iter().for_each(|(netuid, tao)| {
        if let Some(subnet_account) = Pallet::<T>::get_subnet_account_id(netuid) {
            let credit = Pallet::<T>::mint_tao(tao);
            let _ = Pallet::<T>::spend_tao(&subnet_account, credit, tao);
            total_subnet_tao = total_subnet_tao.saturating_add(tao);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));
        }
    });

    // Mint SubnetLocked into subnet accounts
    // Currently (3.3.13) we still burn TAO less initial pool value (which is min lock cost) on 
    // registrations and record full lock cost (including initial pool TAO) in SubnetLocked. 
    // Initial pool TAO is accounted for both in SubnetLocked and SubnetTAO. The double-accounted 
    // initial pool TAO equals NetworkMinLockCost, which is 1 TAO per subnet. It is only 
    // double-accounted in situation when owner emission is zero and subnet is dissolved, which is 
    // only known in the future and is uncertain currently. To make accounting accurate and certain, 
    // we stay with pessimistic approach and rather avoid minting more than 21M TAO. This means that 
    // subnet accounts will be credited SubnetLocked amount less initial pool TAO, but the 
    // TotalIssuance recorded will be increased by the full SubnetLocked amount.
    let mut total_subnet_locked = TaoBalance::ZERO;
    SubnetLocked::<T>::iter().for_each(|(netuid, tao)| {
        if let Some(subnet_account) = Pallet::<T>::get_subnet_account_id(netuid) {
            let initial_pool_tao = NetworkMinLockCost::<T>::get();
            let tao_lock = tao.saturating_sub(initial_pool_tao);
            let credit = Pallet::<T>::mint_tao(tao_lock);
            let _ = Pallet::<T>::spend_tao(&subnet_account, credit, tao_lock);
            total_subnet_locked = total_subnet_locked.saturating_add(tao_lock);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));
        }
    });

    // In rao release (v2.0.0) the lock was burned (TotalIssuance reduction), in the subsequent 
    // migration migrate_restore_subnet_locked we restored locks into SubnetLocked, but did not 
    // increase the TotalIssuance back. Now, in order to restore the TotalIssuance correctly and 
    // account for TAO burned / unburned in locks, increase subtensor pallet TotalIssuance by 
    // total of SubnetLocked update in migrate_restore_subnet_locked.
    let total_locked_adjustment = SUBNET_LOCKED
        .iter()
        .fold(0u64, |acc, (_, value)| acc.saturating_add(*value));

    // mint_tao increases subtensor TotalIssuance, but this is not the intention here because 
    // SubnetTAO and SubnetLocked are already accounted in it. Reduce it back.
    TotalIssuance::<T>::mutate(|total| {
        *total = total.saturating_sub(total_subnet_tao)
            .saturating_sub(total_locked_adjustment.into())
            // Adjust for migrate_subnet_locked unburn
            .saturating_add(total_subnet_locked);
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
        log::warn!("  total_locked_adjustment = {}", total_locked_adjustment);
        log::warn!("  balances_total_issuance = {}", balances_total_issuance);
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
