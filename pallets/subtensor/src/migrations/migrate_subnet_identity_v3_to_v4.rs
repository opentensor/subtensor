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

    // Translate V3-encoded entries to V4 in place within SubnetIdentitiesV3.
    SubnetIdentitiesV3::<T>::translate::<SubnetIdentityV3, _>(|_netuid, v3| {
        migrated_count += 1;
        Some(SubnetIdentityOf {
            subnet_name: v3.subnet_name,
            github_repo: v3.github_repo,
            subnet_contact: v3.subnet_contact,
            subnet_url: v3.subnet_url,
            discord: v3.discord,
            description: v3.description,
            logo_url: v3.logo_url,
            additional: v3.additional,
            agent_docs_url: vec![],
        })
    });

    weight = weight.saturating_add(T::DbWeight::get().reads_writes(migrated_count, migrated_count));

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully. Migrated {migrated_count:?} subnet identities.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}
