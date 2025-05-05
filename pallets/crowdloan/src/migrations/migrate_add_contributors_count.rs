use alloc::string::String;
use frame_support::{BoundedVec, traits::Get, weights::Weight};

use crate::*;

pub fn migrate_add_contributors_count<T: Config>() -> Weight {
    let migration_name = BoundedVec::truncate_from(b"migrate_add_contributors_count".to_vec());
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

    // Get all crowdloans, there is not so many at the moment so we are safe.
    let crowdloan_ids = Crowdloans::<T>::iter_keys().collect::<Vec<_>>();
    weight = weight.saturating_add(T::DbWeight::get().reads(crowdloan_ids.len() as u64));

    for crowdloan_id in crowdloan_ids {
        let contributions = Contributions::<T>::iter_key_prefix(crowdloan_id)
            .collect::<Vec<_>>()
            .len();
        weight = weight.saturating_add(T::DbWeight::get().reads(contributions as u64));

        ContributorsCount::<T>::insert(crowdloan_id, contributions as u32);
    }

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}

#[cfg(test)]
mod tests {
    use crate::mock::{Test, TestState};

    use super::*;
    use sp_core::U256;

    #[test]
    fn test_migrate_add_contributors_count_works() {
        TestState::default().build_and_execute(|| {
            Crowdloans::<Test>::insert(
                0,
                CrowdloanInfo {
                    creator: U256::from(1),
                    deposit: 100,
                    min_contribution: 10,
                    cap: 1000,
                    end: 100,
                    call: None,
                    finalized: false,
                    raised: 0,
                    funds_account: U256::from(2),
                    target_address: None,
                },
            );

            Contributions::<Test>::insert(0, U256::from(1), 100);
            Contributions::<Test>::insert(0, U256::from(2), 100);
            Contributions::<Test>::insert(0, U256::from(3), 100);

            assert_eq!(ContributorsCount::<Test>::get(0), None);
            assert_eq!(
                HasMigrationRun::<Test>::get(BoundedVec::truncate_from(
                    b"migrate_add_contributors_count".to_vec()
                )),
                false
            );

            migrate_add_contributors_count::<Test>();

            assert_eq!(ContributorsCount::<Test>::get(0), Some(3));
            assert_eq!(
                HasMigrationRun::<Test>::get(BoundedVec::truncate_from(
                    b"migrate_add_contributors_count".to_vec()
                )),
                true
            );
        });
    }
}
