use super::*;
use frame_support::{
    storage_alias,
    traits::{Get, GetStorageVersion, StorageVersion},
    weights::Weight,
};
use log::{info, warn};
use sp_std::vec::Vec;
use subtensor_runtime_common::NetUid;

/// Constant for logging purposes
const LOG_TARGET: &str = "loadedemissionmigration";
const LOG_TARGET_1: &str = "fixtotalstakestorage";

/// Module containing deprecated storage format
pub mod deprecated_loaded_emission_format {
    use super::*;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

/// Migrates the LoadedEmission storage to a new format
///
/// # Arguments
///
/// * `T` - The runtime configuration trait
///
/// # Returns
///
/// * `Weight` - The computational weight of this operation
///
/// # Example
///
/// ```ignore
/// let weight = migrate_to_v1_separate_emission::<Runtime>();
/// ```
pub fn migrate_to_v1_separate_emission<T: Config>() -> Weight {
    use deprecated_loaded_emission_format as old;

    // Initialize weight counter
    let mut weight = T::DbWeight::get().reads_writes(1, 0);

    // Get current on-chain storage version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only proceed if current version is less than 1
    if onchain_version < 1 {
        info!(
            target: LOG_TARGET,
            ">>> Updating the LoadedEmission to a new format {onchain_version:?}"
        );

        // Collect all network IDs (netuids) from old LoadedEmission storage
        let curr_loaded_emission: Vec<u16> = old::LoadedEmission::<T>::iter_keys().collect();

        // Remove any undecodable entries
        for netuid in curr_loaded_emission {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            if old::LoadedEmission::<T>::try_get(netuid).is_err() {
                weight.saturating_accrue(T::DbWeight::get().writes(1));
                old::LoadedEmission::<T>::remove(netuid);
                warn!("Was unable to decode old loaded_emission for netuid {netuid}");
            }
        }

        // Translate old storage values to new format
        LoadedEmission::<T>::translate::<Vec<(AccountIdOf<T>, u64)>, _>(
            |netuid: NetUid,
             netuid_emissions: Vec<(AccountIdOf<T>, u64)>|
             -> Option<Vec<(AccountIdOf<T>, u64, u64)>> {
                info!(target: LOG_TARGET, "     Do migration of netuid: {netuid:?}...");

                // Convert old format (server, validator_emission) to new format (server, server_emission, validator_emission)
                // Assume all loaded emission is validator emissions
                let new_netuid_emissions = netuid_emissions
                    .into_iter()
                    .map(|(server, validator_emission)| (server, 0_u64, validator_emission))
                    .collect();

                // Update weight for read and write operations
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

                Some(new_netuid_emissions)
            },
        );

        // Update storage version to 1
        StorageVersion::new(1).put::<Pallet<T>>();
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET_1, "Migration to v1 already completed!");
        Weight::zero()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mock::*;
    use frame_support::traits::{GetStorageVersion, StorageVersion};
    use sp_core::U256;

    /// Test successful migration from old to new emission format
    #[test]
    fn test_migrate_to_v1_separate_emission_success() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 0 (old version)
            StorageVersion::new(0).put::<Pallet<Test>>();
            
            // Create test data in old format: Vec<(AccountId, validator_emission)>
            let netuid = NetUid::from(1);
            let server1 = U256::from(100);
            let server2 = U256::from(200);
            let old_emissions = vec![
                (server1, 1000_u64),
                (server2, 2000_u64),
            ];
            
            // Insert old format data using deprecated storage
            deprecated_loaded_emission_format::LoadedEmission::<Test>::insert(
                netuid.into(),
                old_emissions.clone(),
            );
            
            // Run migration
            let weight = migrate_to_v1_separate_emission::<Test>();
            
            // Verify migration executed (non-zero weight)
            assert!(weight != Weight::zero());
            
            // Verify storage version updated to 1
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(1));
            
            // Verify data translated to new format: Vec<(AccountId, server_emission, validator_emission)>
            let new_emissions = LoadedEmission::<Test>::get(netuid).unwrap();
            assert_eq!(new_emissions.len(), 2);
            
            // Old validator emissions should be preserved, server emissions should be 0
            assert_eq!(new_emissions[0], (server1, 0_u64, 1000_u64));
            assert_eq!(new_emissions[1], (server2, 0_u64, 2000_u64));
        });
    }

    /// Test that migration skips when already completed
    #[test]
    fn test_migrate_to_v1_separate_emission_already_migrated() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 1 or higher
            StorageVersion::new(1).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_to_v1_separate_emission::<Test>();
            
            // Verify migration was skipped (zero weight)
            assert_eq!(weight, Weight::zero());
            
            // Verify version unchanged
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(1));
        });
    }

    /// Test handling of multiple netuids with old format data
    #[test]
    fn test_migrate_to_v1_separate_emission_multiple_netuids() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(0).put::<Pallet<Test>>();
            
            // Setup multiple netuids with old format data
            for netuid_val in 1..=3 {
                let netuid = NetUid::from(netuid_val);
                let server = U256::from(netuid_val as u64 * 100);
                let old_emissions = vec![(server, netuid_val as u64 * 1000)];
                
                deprecated_loaded_emission_format::LoadedEmission::<Test>::insert(
                    netuid.into(),
                    old_emissions,
                );
            }
            
            // Run migration
            let weight = migrate_to_v1_separate_emission::<Test>();
            
            // Verify migration executed
            assert!(weight != Weight::zero());
            
            // Verify all netuids migrated correctly
            for netuid_val in 1..=3 {
                let netuid = NetUid::from(netuid_val);
                let new_emissions = LoadedEmission::<Test>::get(netuid).unwrap();
                assert_eq!(new_emissions.len(), 1);
                
                let expected_server = U256::from(netuid_val as u64 * 100);
                let expected_validator_emission = netuid_val as u64 * 1000;
                assert_eq!(new_emissions[0], (expected_server, 0_u64, expected_validator_emission));
            }
        });
    }

    /// Test handling of empty emissions data
    #[test]
    fn test_migrate_to_v1_separate_emission_empty_data() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(0).put::<Pallet<Test>>();
            
            // Setup netuid with empty emissions
            let netuid = NetUid::from(1);
            let empty_emissions: Vec<(U256, u64)> = vec![];
            
            deprecated_loaded_emission_format::LoadedEmission::<Test>::insert(
                netuid.into(),
                empty_emissions,
            );
            
            // Run migration
            let weight = migrate_to_v1_separate_emission::<Test>();
            
            // Verify migration executed
            assert!(weight != Weight::zero());
            
            // Verify empty data handled correctly
            let new_emissions = LoadedEmission::<Test>::get(netuid).unwrap();
            assert_eq!(new_emissions.len(), 0);
        });
    }

    /// Test weight calculation includes all operations
    #[test]
    fn test_migrate_to_v1_separate_emission_weight_calculation() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(0).put::<Pallet<Test>>();
            
            // Setup test data
            let netuid = NetUid::from(1);
            let server = U256::from(100);
            let old_emissions = vec![(server, 1000_u64)];
            
            deprecated_loaded_emission_format::LoadedEmission::<Test>::insert(
                netuid.into(),
                old_emissions,
            );
            
            // Run migration
            let weight = migrate_to_v1_separate_emission::<Test>();
            
            // Verify weight includes:
            // - Initial version read
            // - Read old emission data
            // - Write new emission data
            // - Write storage version
            assert!(weight.ref_time() > 0);
            
            let expected_min_weight = Test::DbWeight::get().reads(2)
                .saturating_add(Test::DbWeight::get().writes(2));
            
            assert!(weight.ref_time() >= expected_min_weight.ref_time());
        });
    }

    /// Test that old format states are handled correctly
    #[test]
    fn test_migrate_to_v1_separate_emission_preserves_validator_emissions() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(0).put::<Pallet<Test>>();
            
            // Setup with various validator emission values
            let netuid = NetUid::from(1);
            let test_cases = vec![
                (U256::from(1), 0_u64),           // Zero emission
                (U256::from(2), 1_u64),           // Minimal emission
                (U256::from(3), u64::MAX / 2),    // Large emission
            ];
            
            deprecated_loaded_emission_format::LoadedEmission::<Test>::insert(
                netuid.into(),
                test_cases.clone(),
            );
            
            // Run migration
            migrate_to_v1_separate_emission::<Test>();
            
            // Verify all values preserved correctly
            let new_emissions = LoadedEmission::<Test>::get(netuid).unwrap();
            assert_eq!(new_emissions.len(), test_cases.len());
            
            for (idx, (server, validator_emission)) in test_cases.iter().enumerate() {
                assert_eq!(new_emissions[idx], (*server, 0_u64, *validator_emission));
            }
        });
    }

    /// Test migration with no old data present
    #[test]
    fn test_migrate_to_v1_separate_emission_no_old_data() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(0).put::<Pallet<Test>>();
            
            // Don't insert any old data
            
            // Run migration
            let weight = migrate_to_v1_separate_emission::<Test>();
            
            // Verify migration still completes and updates version
            assert!(weight != Weight::zero());
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(1));
        });
    }
}
