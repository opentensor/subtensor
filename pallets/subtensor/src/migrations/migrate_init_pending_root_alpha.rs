use super::*;
use alloc::{collections::BTreeMap, string::String};
use frame_support::{traits::Get, weights::Weight};
use substrate_fixed::types::I96F32;

/// Migration to initialize PendingRootAlpha storage based on RootClaimed storage.
/// This aggregates all RootClaimed values across all netuids and coldkeys for each hotkey.
pub fn migrate_init_pending_root_alpha<T: Config>() -> Weight {
    let migration_name = b"migrate_init_pending_root_alpha".to_vec();
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

    let mut root_claimable_alpha_map: BTreeMap<T::AccountId, u128> = BTreeMap::new();
    for (hotkey, root_claimable) in RootClaimable::<T>::iter() {
        let total = root_claimable
            .iter()
            .map(|(netuid, claimable_rate)| {
                let stake = Pallet::<T>::get_stake_for_hotkey_on_subnet(&hotkey, *netuid);
                let value = claimable_rate.saturating_mul(I96F32::saturating_from_num(stake));
                value.saturating_to_num::<u128>()
            })
            .fold(0_u128, |acc, x| acc.saturating_add(x));

        root_claimable_alpha_map.insert(hotkey, total);
    }

    // Aggregate RootClaimed values by hotkey
    // Key: hotkey, Value: sum of all RootClaimed values for that hotkey
    let mut root_claimed_alpha_map: BTreeMap<T::AccountId, u128> = BTreeMap::new();

    // Iterate over all RootClaimed entries: (netuid, hotkey, coldkey) -> claimed_value
    for ((_netuid, hotkey, _coldkey), claimed_value) in RootClaimed::<T>::iter() {
        // Aggregate the claimed value for this hotkey
        root_claimed_alpha_map
            .entry(hotkey.clone())
            .and_modify(|total| *total = total.saturating_add(claimed_value))
            .or_insert(claimed_value);

        // Account for read operation
        weight = weight.saturating_add(T::DbWeight::get().reads(1));
    }

    // Set PendingRootAlpha for each hotkey
    let mut migrated_count = 0u64;
    for (hotkey, claimable) in root_claimable_alpha_map {
        let claimed = root_claimed_alpha_map.get(&hotkey).unwrap_or(&0);
        // Calculate pending: claimable - claimed
        let pending = claimable.saturating_sub(*claimed);
        // Convert u128 to u64, saturating at u64::MAX if overflow
        // let pending_u64 = pending.min(u64::MAX as u128) as u64;
        // let alpha_currency: AlphaCurrency = pending_u64.into();
        PendingRootAlpha::<T>::insert(&hotkey, pending);
        migrated_count = migrated_count.saturating_add(1);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    log::info!(
        "Migration '{}' completed successfully. Initialized PendingRootAlpha for {} hotkeys.",
        String::from_utf8_lossy(&migration_name),
        migrated_count
    );

    // Mark the migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    weight
}
