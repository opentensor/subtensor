use super::*;
use frame_support::weights::Weight;

pub fn migrate_remove_activity_cutoff<T: Config>() -> Weight {
    migrate_storage::<T>(
        "migrate_remove_activity_cutoff",
        "SubtensorModule",
        "ActivityCutoff",
    )
    .saturating_add(migrate_storage::<T>(
        "migrate_remove_min_activity_cutoff",
        "SubtensorModule",
        "MinActivityCutoff",
    ))
}
