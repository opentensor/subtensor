use super::*;
use frame_support::weights::Weight;
use log;
use scale_info::prelude::{string::String, vec::Vec};

pub fn migrate_subnet_identities_to_v3<T: Config>() -> Weight {
    use frame_support::traits::Get;
    let migration_name = b"migrate_subnet_identities_to_v3".to_vec();

    // Start counting weight
    let mut weight = T::DbWeight::get().reads(1);

    // Check if we already ran this migration
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        target: "runtime",
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );
    // -----------------------------
    // 1) Migrate Subnet Identities
    // -----------------------------
    let old_subnet_identities = SubnetIdentitiesV2::<T>::iter().collect::<Vec<_>>();
    for (netuid, old_subnet_identity) in old_subnet_identities.clone() {
        // check for existing SubnetIdentitiesV3 entry, skip if found
        if SubnetIdentitiesV3::<T>::contains_key(netuid) {
            continue;
        }
        let new_subnet_identity = SubnetIdentityV3 {
            subnet_name: old_subnet_identity.subnet_name,
            github_repo: old_subnet_identity.github_repo,
            subnet_contact: old_subnet_identity.subnet_contact,
            subnet_url: Vec::new(),
            discord: Vec::new(),
            description: Vec::new(),
            logo_url: Vec::new(),
            additional: Vec::new(),
        };

        // Insert into the new storage map
        SubnetIdentitiesV3::<T>::insert(netuid, &new_subnet_identity);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));

        SubnetIdentitiesV2::<T>::remove(netuid);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }
    weight = weight.saturating_add(T::DbWeight::get().reads(old_subnet_identities.len() as u64));

    // -----------------------------
    // Mark the migration as done
    // -----------------------------
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
