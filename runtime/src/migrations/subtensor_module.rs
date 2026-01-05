use core::marker::PhantomData;

use frame_support::{traits::Get, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use log;
use pallet_rate_limiting::{RateLimit, RateLimitKind, RateLimitTarget};
use scale_info::prelude::string::String;
use sp_runtime::traits::SaturatedConversion;
use subtensor_runtime_common::TaoCurrency;
use subtensor_runtime_common::rate_limiting::{GROUP_REGISTER_NETWORK, GroupId};

use pallet_subtensor::{
    Config as SubtensorConfig, HasMigrationRun, NetworkLockReductionInterval,
    NetworkRegistrationStartBlock, Pallet as SubtensorPallet,
};

pub struct Migration<T: SubtensorConfig>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for Migration<T>
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            LimitScope = subtensor_runtime_common::NetUid,
            GroupId = GroupId,
        >,
{
    fn on_runtime_upgrade() -> Weight {
        migrate_network_lock_reduction_interval::<T>()
            .saturating_add(migrate_network_lock_cost_2500::<T>())
    }
}

pub fn migrate_network_lock_reduction_interval<T>() -> Weight
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            LimitScope = subtensor_runtime_common::NetUid,
            GroupId = GroupId,
        >,
{
    const FOUR_DAYS: u64 = 28_800;
    const EIGHT_DAYS: u64 = 57_600;
    const ONE_WEEK_BLOCKS: u64 = 50_400;

    let migration_name = b"migrate_network_lock_reduction_interval".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    // Skip if already executed
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{}' already run - skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    let current_block = SubtensorPallet::<T>::get_current_block_as_u64();

    // -- 1) Set new values --------------------------------------------------
    NetworkLockReductionInterval::<T>::put(EIGHT_DAYS);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    pallet_rate_limiting::Limits::<T>::insert(
        RateLimitTarget::Group(GROUP_REGISTER_NETWORK),
        RateLimit::Global(RateLimitKind::Exact(FOUR_DAYS.saturated_into())),
    );
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    SubtensorPallet::<T>::set_network_last_lock(TaoCurrency::from(1_000_000_000_000));
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Hold price at 2000 TAO until day 7, then begin linear decay
    let last_lock_block = current_block.saturating_add(ONE_WEEK_BLOCKS);

    // Allow registrations starting at day 7
    NetworkRegistrationStartBlock::<T>::put(last_lock_block);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Mirror the register-network last seen in pallet-rate-limiting.
    let last_seen_block: BlockNumberFor<T> = last_lock_block.saturated_into();
    pallet_rate_limiting::LastSeen::<T>::insert(
        RateLimitTarget::Group(GROUP_REGISTER_NETWORK),
        None::<T::UsageKey>,
        last_seen_block,
    );
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // -- 2) Mark migration done --------------------------------------------
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed.",
        String::from_utf8_lossy(&migration_name),
    );

    weight
}

pub fn migrate_network_lock_cost_2500<T>() -> Weight
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            LimitScope = subtensor_runtime_common::NetUid,
            GroupId = GroupId,
        >,
{
    const RAO_PER_TAO: u64 = 1_000_000_000;
    const TARGET_COST_TAO: u64 = 2_500;
    const NEW_LAST_LOCK_RAO: u64 = (TARGET_COST_TAO / 2) * RAO_PER_TAO; // 1,250 TAO

    let migration_name = b"migrate_network_lock_cost_2500".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    // Skip if already executed
    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            target: "runtime",
            "Migration '{}' already run - skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    // Use the current block; ensure it's non-zero so mult == 2 in get_network_lock_cost()
    let current_block = SubtensorPallet::<T>::get_current_block_as_u64();
    let block_to_set = if current_block == 0 { 1 } else { current_block };

    // Set last_lock so that price = 2 * last_lock = 2,500 TAO at this block
    SubtensorPallet::<T>::set_network_last_lock(TaoCurrency::from(NEW_LAST_LOCK_RAO));
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Mirror the register-network last seen in pallet-rate-limiting.
    let last_seen_block: BlockNumberFor<T> = block_to_set.saturated_into();
    pallet_rate_limiting::LastSeen::<T>::insert(
        RateLimitTarget::Group(GROUP_REGISTER_NETWORK),
        None::<T::UsageKey>,
        last_seen_block,
    );
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    // Mark migration done
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed. lock_cost set to 2,500 TAO at block {}.",
        String::from_utf8_lossy(&migration_name),
        block_to_set
    );

    weight
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use frame_support::pallet_prelude::Zero;
    use frame_system::pallet_prelude::BlockNumberFor;
    use pallet_rate_limiting::{RateLimit, RateLimitKind, RateLimitTarget};
    use sp_io::TestExternalities;
    use sp_runtime::traits::SaturatedConversion;
    use subtensor_runtime_common::Currency;
    use subtensor_runtime_common::rate_limiting::GROUP_REGISTER_NETWORK;

    use super::*;
    use crate::{BuildStorage, Runtime, System};

    fn new_test_ext() -> TestExternalities {
        sp_tracing::try_init_simple();
        let mut ext: TestExternalities = crate::RuntimeGenesisConfig::default()
            .build_storage()
            .expect("runtime storage")
            .into();
        ext.execute_with(|| System::set_block_number(1u64.saturated_into()));
        ext
    }

    fn step_block(blocks: u64) {
        let next = System::block_number().saturating_add(blocks.saturated_into());
        System::set_block_number(next);
    }

    fn register_network_last_seen() -> Option<BlockNumberFor<Runtime>> {
        pallet_rate_limiting::LastSeen::<Runtime>::get(
            RateLimitTarget::Group(GROUP_REGISTER_NETWORK),
            None::<<Runtime as pallet_rate_limiting::Config>::UsageKey>,
        )
    }

    #[test]
    fn test_migrate_network_lock_reduction_interval_and_decay() {
        new_test_ext().execute_with(|| {
            const FOUR_DAYS: u64 = 28_800;
            const EIGHT_DAYS: u64 = 57_600;
            const ONE_WEEK_BLOCKS: u64 = 50_400;

            // -- pre --------------------------------------------------------------
            assert!(
                !HasMigrationRun::<Runtime>::get(
                    b"migrate_network_lock_reduction_interval".to_vec()
                ),
                "HasMigrationRun should be false before migration"
            );

            // ensure current_block > 0
            step_block(1);
            let current_block_before = SubtensorPallet::<Runtime>::get_current_block_as_u64();

            // -- run migration ---------------------------------------------------
            let weight = migrate_network_lock_reduction_interval::<Runtime>();
            assert!(!weight.is_zero(), "migration weight should be > 0");

            // -- params & flags --------------------------------------------------
            assert_eq!(NetworkLockReductionInterval::<Runtime>::get(), EIGHT_DAYS);
            assert_eq!(
                pallet_rate_limiting::Limits::<Runtime>::get(RateLimitTarget::Group(
                    GROUP_REGISTER_NETWORK
                )),
                Some(RateLimit::Global(RateLimitKind::Exact(
                    FOUR_DAYS.saturated_into()
                )))
            );
            assert_eq!(
                SubtensorPallet::<Runtime>::get_network_last_lock(),
                1_000_000_000_000u64.into(), // 1000 TAO in rao
                "last_lock should be 1_000_000_000_000 rao"
            );

            // last_lock_block should be set one week in the future
            let last_lock_block = register_network_last_seen().expect("last seen entry");
            let expected_block = current_block_before + ONE_WEEK_BLOCKS;
            assert_eq!(
                last_lock_block,
                expected_block.saturated_into::<BlockNumberFor<Runtime>>(),
                "last_lock_block should be current + ONE_WEEK_BLOCKS"
            );

            // registration start block should match the same future block
            assert_eq!(
                NetworkRegistrationStartBlock::<Runtime>::get(),
                expected_block,
                "NetworkRegistrationStartBlock should equal last_lock_block"
            );

            // lock cost should be 2000 TAO immediately after migration
            let lock_cost_now = SubtensorPallet::<Runtime>::get_network_lock_cost();
            assert_eq!(
                lock_cost_now,
                2_000_000_000_000u64.into(),
                "lock cost should be 2000 TAO right after migration"
            );

            assert!(
                HasMigrationRun::<Runtime>::get(
                    b"migrate_network_lock_reduction_interval".to_vec()
                ),
                "HasMigrationRun should be true after migration"
            );
        });
    }

    #[test]
    fn test_migrate_network_lock_cost_2500_sets_price_and_decay() {
        new_test_ext().execute_with(|| {
            // -- constants -------------------------------------------------------
            const RAO_PER_TAO: u64 = 1_000_000_000;
            const TARGET_COST_TAO: u64 = 2_500;
            const TARGET_COST_RAO: u64 = TARGET_COST_TAO * RAO_PER_TAO;
            const NEW_LAST_LOCK_RAO: u64 = (TARGET_COST_TAO / 2) * RAO_PER_TAO;

            let migration_key = b"migrate_network_lock_cost_2500".to_vec();

            // -- pre --------------------------------------------------------------
            assert!(
                !HasMigrationRun::<Runtime>::get(migration_key.clone()),
                "HasMigrationRun should be false before migration"
            );

            // Ensure current_block > 0 so mult == 2 in get_network_lock_cost()
            step_block(1);
            let current_block_before = SubtensorPallet::<Runtime>::get_current_block_as_u64();

            // Snapshot interval to ensure migration doesn't change it
            let interval_before = NetworkLockReductionInterval::<Runtime>::get();

            // -- run migration ---------------------------------------------------
            let weight = migrate_network_lock_cost_2500::<Runtime>();
            assert!(!weight.is_zero(), "migration weight should be > 0");

            // -- asserts: params & flags -----------------------------------------
            assert_eq!(
                SubtensorPallet::<Runtime>::get_network_last_lock(),
                NEW_LAST_LOCK_RAO.into(),
                "last_lock should be set to 1,250 TAO (in rao)"
            );
            assert_eq!(
                register_network_last_seen().expect("last seen entry"),
                current_block_before.saturated_into::<BlockNumberFor<Runtime>>(),
                "last_lock_block should be set to the current block"
            );

            // Lock cost should be exactly 2,500 TAO immediately after migration
            let lock_cost_now = SubtensorPallet::<Runtime>::get_network_lock_cost();
            assert_eq!(
                lock_cost_now,
                TARGET_COST_RAO.into(),
                "lock cost should be 2,500 TAO right after migration"
            );

            // Interval should be unchanged by this migration
            assert_eq!(
                NetworkLockReductionInterval::<Runtime>::get(),
                interval_before,
                "lock reduction interval should not be modified by this migration"
            );

            assert!(
                HasMigrationRun::<Runtime>::get(migration_key.clone()),
                "HasMigrationRun should be true after migration"
            );

            // -- decay check (1 block later) -------------------------------------
            // Expected: cost = max(min_lock, 2*L - floor(L / eff_interval) * delta_blocks)
            let eff_interval = SubtensorPallet::<Runtime>::get_lock_reduction_interval();
            let per_block_decrement: u64 = if eff_interval == 0 {
                0
            } else {
                NEW_LAST_LOCK_RAO / eff_interval
            };

            let min_lock_rao: u64 = SubtensorPallet::<Runtime>::get_network_min_lock().to_u64();

            step_block(1);
            let expected_after_1: u64 =
                core::cmp::max(min_lock_rao, TARGET_COST_RAO - per_block_decrement);
            let lock_cost_after_1 = SubtensorPallet::<Runtime>::get_network_lock_cost();
            assert_eq!(
                lock_cost_after_1,
                expected_after_1.into(),
                "lock cost should decay by one per-block step after 1 block"
            );

            // -- idempotency: running the migration again should do nothing ------
            let last_lock_before_rerun = SubtensorPallet::<Runtime>::get_network_last_lock();
            let last_lock_block_before_rerun =
                register_network_last_seen().expect("last seen entry");
            let cost_before_rerun = SubtensorPallet::<Runtime>::get_network_lock_cost();

            let _weight2 = migrate_network_lock_cost_2500::<Runtime>();

            assert!(
                HasMigrationRun::<Runtime>::get(migration_key.clone()),
                "HasMigrationRun remains true on second run"
            );
            assert_eq!(
                SubtensorPallet::<Runtime>::get_network_last_lock(),
                last_lock_before_rerun,
                "second run should not modify last_lock"
            );
            assert_eq!(
                register_network_last_seen().expect("last seen entry"),
                last_lock_block_before_rerun,
                "second run should not modify last_lock_block"
            );
            assert_eq!(
                SubtensorPallet::<Runtime>::get_network_lock_cost(),
                cost_before_rerun,
                "second run should not change current lock cost"
            );
        });
    }
}
