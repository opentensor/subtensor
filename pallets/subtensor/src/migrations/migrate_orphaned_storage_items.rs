use super::*;
use frame_support::weights::Weight;

pub fn migrate_orphaned_storage_items<T: Config>() -> Weight {
    remove_last_hotkey_coldkey_emission_on_netuid::<T>()
        .saturating_add(remove_subnet_alpha_emission_sell::<T>())
        .saturating_add(remove_neurons_to_prune_at_next_epoch::<T>())
        .saturating_add(remove_total_stake_at_dynamic::<T>())
        .saturating_add(remove_subnet_name::<T>())
        .saturating_add(remove_network_min_allowed_uids::<T>())
        .saturating_add(remove_dynamic_block::<T>())
}

pub(crate) fn remove_last_hotkey_coldkey_emission_on_netuid<T: Config>() -> Weight {
    let migration_name = "migrate_remove_last_hotkey_coldkey_emission_on_netuid";
    let pallet_name = "SubtensorModule";
    let storage_name = "LastHotkeyColdkeyEmissionOnNetuid";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}

pub(crate) fn remove_subnet_alpha_emission_sell<T: Config>() -> Weight {
    let migration_name = "migrate_remove_subnet_alpha_emission_sell";
    let pallet_name = "SubtensorModule";
    let storage_name = "SubnetAlphaEmissionSell";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}

pub(crate) fn remove_neurons_to_prune_at_next_epoch<T: Config>() -> Weight {
    let migration_name = "migrate_remove_neurons_to_prune_at_next_epoch";
    let pallet_name = "SubtensorModule";
    let storage_name = "NeuronsToPruneAtNextEpoch";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}

pub(crate) fn remove_total_stake_at_dynamic<T: Config>() -> Weight {
    let migration_name = "migrate_remove_total_stake_at_dynamic";
    let pallet_name = "SubtensorModule";
    let storage_name = "TotalStakeAtDynamic";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}

pub(crate) fn remove_subnet_name<T: Config>() -> Weight {
    let migration_name = "migrate_remove_subnet_name";
    let pallet_name = "SubtensorModule";
    let storage_name = "SubnetName";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}

pub(crate) fn remove_network_min_allowed_uids<T: Config>() -> Weight {
    let migration_name = "migrate_remove_network_min_allowed_uids";
    let pallet_name = "SubtensorModule";
    let storage_name = "NetworkMinAllowedUids";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}

pub(crate) fn remove_dynamic_block<T: Config>() -> Weight {
    let migration_name = "migrate_remove_dynamic_block";
    let pallet_name = "SubtensorModule";
    let storage_name = "DynamicBlock";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}
