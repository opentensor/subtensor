use crate::Pallet as Subtensor;
use crate::{Config, HasMigrationRun};
use alloc::string::String;
use frame_support::{traits::Get, weights::Weight};

// List of cleared maps for root netuid:
// - SubnetworkN
// - Tempo
// - ActivityCutoff
// - Bonds
// - Keys
// - MaxAllowedValidators
// - SubnetOwnerHotkey
// - Kappa
// - BondsPenalty
// - Yuma3On
// - BlockAtRegistration
// - Rank
// - Trust
// - Active
// - Emission
// - Consensus
// - Incentive
// - Dividends
// - LastUpdate
// - PruningScores
// - ValidatorTrust
// - ValidatorPermit
// - StakeWeight
pub fn migrate_clear_root_epoch_values<T: Config>() -> Weight {
    let migration_name = b"migrate_clear_root_epoch_values".to_vec();
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

    // Clear root epoch values
    let root_netuid = Subtensor::<T>::get_root_netuid();

    crate::SubnetworkN::<T>::remove(root_netuid);
    crate::Tempo::<T>::remove(root_netuid);
    crate::ActivityCutoff::<T>::remove(root_netuid);
    crate::MaxAllowedValidators::<T>::remove(root_netuid);
    crate::SubnetOwnerHotkey::<T>::remove(root_netuid);

    crate::Kappa::<T>::remove(root_netuid);
    crate::BondsPenalty::<T>::remove(root_netuid);
    crate::Yuma3On::<T>::remove(root_netuid);
    crate::Rank::<T>::remove(root_netuid);
    crate::Trust::<T>::remove(root_netuid);

    crate::Active::<T>::remove(root_netuid);
    crate::Emission::<T>::remove(root_netuid);
    crate::Consensus::<T>::remove(root_netuid);
    crate::Incentive::<T>::remove(root_netuid);
    crate::Dividends::<T>::remove(root_netuid);

    crate::LastUpdate::<T>::remove(root_netuid);
    crate::PruningScores::<T>::remove(root_netuid);
    crate::ValidatorTrust::<T>::remove(root_netuid);
    crate::ValidatorPermit::<T>::remove(root_netuid);
    crate::StakeWeight::<T>::remove(root_netuid);

    let total_simple_removals = 20u64;

    let mut total_db_operations = total_simple_removals;

    let bonds_removal_res = crate::Bonds::<T>::clear_prefix(root_netuid, u32::MAX, None);
    let keys_removal_res = crate::Keys::<T>::clear_prefix(root_netuid, u32::MAX, None);
    let regblocks_removal_res =
        crate::BlockAtRegistration::<T>::clear_prefix(root_netuid, u32::MAX, None);

    total_db_operations = total_db_operations.saturating_add(bonds_removal_res.backend.into());
    total_db_operations = total_db_operations.saturating_add(keys_removal_res.backend.into());
    total_db_operations = total_db_operations.saturating_add(regblocks_removal_res.backend.into());
    weight = weight.saturating_add(T::DbWeight::get().writes(total_db_operations));

    // Mark Migration as Completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
