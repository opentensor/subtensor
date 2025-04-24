use super::*;
use frame_support::weights::Weight;

pub fn migrate_orphaned_storage_items<T: Config>() -> Weight {
    remove_last_hotkey_coldkey_emission_on_netuid::<T>()
}


pub(crate) fn remove_last_hotkey_coldkey_emission_on_netuid<T: Config>() -> Weight {
    let migration_name = "migrate_remove_last_hotkey_coldkey_emission_on_netuid";
    let pallet_name = "SubtensorModule";
    let storage_name = "LastHotkeyColdkeyEmissionOnNetuid";

    migrate_storage::<T>(migration_name, pallet_name, storage_name)
}
