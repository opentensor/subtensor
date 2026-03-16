use super::*;
use crate::HasMigrationRun;
use frame_support::{traits::Get, weights::Weight};
use scale_info::prelude::string::String;

pub fn migrate_subnet_identity_v3_to_v4<T: Config>() -> Weight {
    let migration_name = b"migrate_subnet_identity_v3_to_v4".to_vec();
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

    let mut migrated_count: u64 = 0;

    for (netuid, v3_identity) in SubnetIdentitiesV3::<T>::iter() {
        let v4_identity = SubnetIdentityOfV4 {
            subnet_name: v3_identity.subnet_name,
            github_repo: v3_identity.github_repo,
            subnet_contact: v3_identity.subnet_contact,
            subnet_url: v3_identity.subnet_url,
            discord: v3_identity.discord,
            description: v3_identity.description,
            logo_url: v3_identity.logo_url,
            additional: v3_identity.additional,
            agent_docs_url: vec![],
        };
        SubnetIdentitiesV4::<T>::insert(netuid, v4_identity);
        migrated_count += 1;
    }

    weight = weight.saturating_add(T::DbWeight::get().reads(migrated_count));
    weight = weight.saturating_add(T::DbWeight::get().writes(migrated_count));

    // Remove old V3 entries
    remove_prefix::<T>("SubtensorModule", "SubnetIdentitiesV3", &mut weight);

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully. Migrated {migrated_count:?} subnet identities.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
