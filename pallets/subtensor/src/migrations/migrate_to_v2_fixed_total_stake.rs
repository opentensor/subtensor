use super::*;
use frame_support::{
    storage_alias,
    traits::{Get, GetStorageVersion},
    weights::Weight,
};
use log::info;
use sp_std::vec::Vec;

/// Constant for logging purposes
const LOG_TARGET: &str = "fix_total_stake_storage";

/// Module containing deprecated storage format
pub mod deprecated_loaded_emission_format {
    use super::*;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

/// Migrates the storage to fix TotalStake and TotalColdkeyStake
///
/// This function performs the following steps:
/// 1. Resets TotalStake to 0
/// 2. Resets all TotalColdkeyStake entries to 0
/// 3. Recalculates TotalStake and TotalColdkeyStake based on the Stake map
///
/// # Arguments
///
/// * `T` - The Config trait of the pallet
///
/// # Returns
///
/// * `Weight` - The computational weight of this operation
///
/// # Example
///
/// ```ignore
/// let weight = migrate_to_v2_fixed_total_stake::<T>();
/// ```
pub fn migrate_to_v2_fixed_total_stake<T: Config>() -> Weight {
    let new_storage_version = 2;

    // Initialize weight counter
    let weight = T::DbWeight::get().reads(1);

    // Get current on-chain storage version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only proceed if current version is less than the new version
    if onchain_version < new_storage_version {
        info!(
            target: LOG_TARGET,
            "Fixing the TotalStake and TotalColdkeyStake storage. Current version: {onchain_version:?}"
        );

        // TODO: Fix or remove migration
        // // Reset TotalStake to 0
        // TotalStake::<T>::put(0);
        // weight.saturating_accrue(T::DbWeight::get().writes(1));

        // // Reset all TotalColdkeyStake entries to 0
        // let total_coldkey_stake_keys = TotalColdkeyStake::<T>::iter_keys().collect::<Vec<_>>();
        // for coldkey in total_coldkey_stake_keys {
        //     weight.saturating_accrue(T::DbWeight::get().reads(1));
        //     TotalColdkeyStake::<T>::insert(coldkey, 0);
        //     weight.saturating_accrue(T::DbWeight::get().writes(1));
        // }

        // // Recalculate TotalStake and TotalColdkeyStake based on the Stake map
        // for (_, coldkey, stake) in Stake::<T>::iter() {
        //     weight.saturating_accrue(T::DbWeight::get().reads(1));

        //     // Update TotalColdkeyStake
        //     let mut total_coldkey_stake = TotalColdkeyStake::<T>::get(coldkey.clone());
        //     weight.saturating_accrue(T::DbWeight::get().reads(1));
        //     total_coldkey_stake = total_coldkey_stake.saturating_add(stake);
        //     TotalColdkeyStake::<T>::insert(coldkey, total_coldkey_stake);
        //     weight.saturating_accrue(T::DbWeight::get().writes(1));

        //     // Update TotalStake
        //     let mut total_stake = TotalStake::<T>::get();
        //     weight.saturating_accrue(T::DbWeight::get().reads(1));
        //     total_stake = total_stake.saturating_add(stake);
        //     TotalStake::<T>::put(total_stake);
        //     weight.saturating_accrue(T::DbWeight::get().writes(1));
        // }

        // // Update storage version to prevent re-running this migration
        // StorageVersion::new(new_storage_version).put::<Pallet<T>>();
        // weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET, "Migration to v2 already completed");
        Weight::zero()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mock::*;
    use frame_support::traits::{GetStorageVersion, StorageVersion};

    /// Test that migration correctly skips when version check fails
    #[test]
    fn test_migrate_to_v2_fixed_total_stake_version_check() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 2 or higher (already migrated)
            StorageVersion::new(2).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_to_v2_fixed_total_stake::<Test>();
            
            // Verify migration was skipped (zero weight)
            assert_eq!(weight, Weight::zero());
            
            // Verify version unchanged
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(2));
        });
    }

    /// Test that migration skips when version is exactly 2
    #[test]
    fn test_migrate_to_v2_fixed_total_stake_exact_version() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to exactly 2
            StorageVersion::new(2).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_to_v2_fixed_total_stake::<Test>();
            
            // Verify migration skipped
            assert_eq!(weight, Weight::zero());
        });
    }

    /// Test migration behavior with version 1 (should trigger check but logic is disabled)
    #[test]
    fn test_migrate_to_v2_fixed_total_stake_version_1_disabled_migration() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 1 (should trigger migration check)
            StorageVersion::new(1).put::<Pallet<Test>>();
            
            // Run migration - note the actual migration logic is commented out (TODO line 58)
            let weight = migrate_to_v2_fixed_total_stake::<Test>();
            
            // Currently returns only the read weight since migration logic is disabled
            let expected_weight = Test::DbWeight::get().reads(1);
            assert_eq!(weight, expected_weight);
            
            // Version is NOT updated because migration logic is disabled
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(1));
        });
    }

    /// Test weight calculation when migration is skipped
    #[test]
    fn test_migrate_to_v2_fixed_total_stake_skip_weight() {
        new_test_ext(1).execute_with(|| {
            // Setup version that causes skip
            StorageVersion::new(3).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_to_v2_fixed_total_stake::<Test>();
            
            // Should return zero weight for skipped migration
            assert_eq!(weight, Weight::zero());
            assert_eq!(weight.ref_time(), 0);
        });
    }

    // NOTE: The following tests would be relevant if the migration logic is re-enabled
    // Currently, the migration implementation is commented out (see line 58: TODO: Fix or remove migration)
    // If re-enabled, these test scenarios should be implemented:
    
    // TODO: If migration is re-enabled, add test for TotalStake reset and recalculation
    // #[test]
    // fn test_migrate_to_v2_fixed_total_stake_total_stake_recalculation() { ... }
    
    // TODO: If migration is re-enabled, add test for TotalColdkeyStake reset and recalculation  
    // #[test]
    // fn test_migrate_to_v2_fixed_total_stake_coldkey_stake_recalculation() { ... }
    
    // TODO: If migration is re-enabled, add test for arithmetic overflow protection
    // #[test]
    // fn test_migrate_to_v2_fixed_total_stake_overflow_protection() { ... }
    
    // TODO: If migration is re-enabled, add test for storage iteration weight calculation
    // #[test]
    // fn test_migrate_to_v2_fixed_total_stake_iteration_weight() { ... }
}

// MIGRATION STATUS DOCUMENTATION:
// This migration function is currently DISABLED (see line 58).
// The entire migration logic for resetting and recalculating TotalStake and TotalColdkeyStake
// is commented out. This appears to be intentional based on the TODO comment.
//
// Decision needed:
// 1. If migration should be removed entirely: Delete this file and remove from mod.rs
// 2. If migration should be fixed and re-enabled: Uncomment the logic, add proper error
//    handling with saturating arithmetic, and implement the additional test cases noted above
// 3. If migration should remain disabled: Document why and when it might be re-enabled
