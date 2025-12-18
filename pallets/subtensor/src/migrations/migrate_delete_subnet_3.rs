use super::*;
use frame_support::{
    storage_alias,
    traits::{Get, GetStorageVersion, StorageVersion},
    weights::Weight,
};
use log::info;
use sp_std::vec::Vec;
use subtensor_runtime_common::{NetUid, NetUidStorageIndex};

/// Constant for logging purposes
const LOG_TARGET: &str = "migrate_delete_subnet_3";

/// Module containing deprecated storage format
pub mod deprecated_loaded_emission_format {
    use super::*;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

/// Migrates the storage to delete subnet 3
///
/// This function performs the following steps:
/// 1. Checks if the migration is necessary
/// 2. Removes all storage related to subnet 3
/// 3. Updates the storage version
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
/// let weight = migrate_delete_subnet_3::<T>();
/// ```
pub fn migrate_delete_subnet_3<T: Config>() -> Weight {
    let new_storage_version = 5;

    // Initialize weight counter
    let mut weight = T::DbWeight::get().reads(1);

    // Get current on-chain storage version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only proceed if current version is less than the new version and subnet 3 exists
    if onchain_version < new_storage_version && Pallet::<T>::if_subnet_exist(3.into()) {
        info!(
            target: LOG_TARGET,
            "Removing subnet 3. Current version: {onchain_version:?}"
        );

        let netuid = NetUid::from(3);

        // Remove network count
        SubnetworkN::<T>::remove(netuid);

        // Remove netuid from added networks
        NetworksAdded::<T>::remove(netuid);

        // Decrement the network counter
        TotalNetworks::<T>::mutate(|n| *n = n.saturating_sub(1));

        // Remove network registration time
        NetworkRegisteredAt::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(5));

        // Remove incentive mechanism memory
        let _ = Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Bonds::<T>::clear_prefix(NetUidStorageIndex::from(netuid), u32::MAX, None);
        let _ = Weights::<T>::clear_prefix(NetUidStorageIndex::from(netuid), u32::MAX, None);

        weight.saturating_accrue(T::DbWeight::get().writes(4));

        // Remove various network-related parameters
        Rank::<T>::remove(netuid);
        Trust::<T>::remove(netuid);
        Active::<T>::remove(netuid);
        Emission::<T>::remove(netuid);
        Incentive::<T>::remove(NetUidStorageIndex::from(netuid));
        Consensus::<T>::remove(netuid);
        Dividends::<T>::remove(netuid);
        PruningScores::<T>::remove(netuid);
        LastUpdate::<T>::remove(NetUidStorageIndex::from(netuid));
        ValidatorPermit::<T>::remove(netuid);
        ValidatorTrust::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(11));

        // Erase network parameters
        Tempo::<T>::remove(netuid);
        Kappa::<T>::remove(netuid);
        Difficulty::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        ActivityCutoff::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        RegistrationsThisInterval::<T>::remove(netuid);
        POWRegistrationsThisInterval::<T>::remove(netuid);
        BurnRegistrationsThisInterval::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(10));

        // Update storage version
        StorageVersion::new(new_storage_version).put::<Pallet<T>>();
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET, "Migration to v5 already completed or subnet 3 doesn't exist");
        Weight::zero()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mock::*;
    use frame_support::traits::{GetStorageVersion, StorageVersion};
    use sp_core::U256;

    /// Test that migration runs successfully when conditions are met
    #[test]
    fn test_migrate_delete_subnet_3_success() {
        new_test_ext(1).execute_with(|| {
            // Setup: Create subnet 3
            let netuid = NetUid::from(3);
            add_network(netuid, 100, 0);
            
            // Register some neurons to populate storage
            let hotkey = U256::from(1);
            let coldkey = U256::from(2);
            register_ok_neuron(netuid, hotkey, coldkey, 0);
            
            // Verify subnet exists before migration
            assert!(Pallet::<Test>::if_subnet_exist(netuid));
            assert_eq!(TotalNetworks::<Test>::get(), 1);
            
            // Set storage version to 4 (less than new version 5)
            StorageVersion::new(4).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_delete_subnet_3::<Test>();
            
            // Verify migration executed
            assert!(weight != Weight::zero());
            
            // Verify subnet 3 is removed
            assert!(!Pallet::<Test>::if_subnet_exist(netuid));
            assert_eq!(TotalNetworks::<Test>::get(), 0);
            
            // Verify storage version updated
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(5));
            
            // Verify network registration data removed
            assert!(!NetworksAdded::<Test>::contains_key(netuid));
            assert!(!NetworkRegisteredAt::<Test>::contains_key(netuid));
        });
    }

    /// Test that migration skips when already completed (version check)
    #[test]
    fn test_migrate_delete_subnet_3_already_migrated() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 5 or higher
            StorageVersion::new(5).put::<Pallet<Test>>();
            
            // Create subnet 3 that should NOT be deleted
            let netuid = NetUid::from(3);
            add_network(netuid, 100, 0);
            
            // Run migration
            let weight = migrate_delete_subnet_3::<Test>();
            
            // Verify migration was skipped (zero weight)
            assert_eq!(weight, Weight::zero());
            
            // Verify subnet 3 still exists
            assert!(Pallet::<Test>::if_subnet_exist(netuid));
        });
    }

    /// Test that migration skips when subnet 3 doesn't exist
    #[test]
    fn test_migrate_delete_subnet_3_subnet_not_exist() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 4 but don't create subnet 3
            StorageVersion::new(4).put::<Pallet<Test>>();
            
            // Verify subnet 3 doesn't exist
            assert!(!Pallet::<Test>::if_subnet_exist(NetUid::from(3)));
            
            // Run migration
            let weight = migrate_delete_subnet_3::<Test>();
            
            // Verify migration was skipped
            assert_eq!(weight, Weight::zero());
        });
    }

    /// Test that all relevant storage items for subnet 3 are removed
    #[test]
    fn test_migrate_delete_subnet_3_storage_cleanup() {
        new_test_ext(1).execute_with(|| {
            // Setup: Create subnet 3 with full storage
            let netuid = NetUid::from(3);
            add_network(netuid, 100, 0);
            
            let hotkey = U256::from(1);
            let coldkey = U256::from(2);
            register_ok_neuron(netuid, hotkey, coldkey, 0);
            
            // Manually set additional storage items that should be cleaned up
            Tempo::<Test>::insert(netuid, 100);
            Kappa::<Test>::insert(netuid, 100);
            Difficulty::<Test>::insert(netuid, 10000);
            MaxAllowedUids::<Test>::insert(netuid, 100);
            ImmunityPeriod::<Test>::insert(netuid, 100);
            ActivityCutoff::<Test>::insert(netuid, 100);
            
            // Set storage version
            StorageVersion::new(4).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_delete_subnet_3::<Test>();
            
            // Verify migration executed
            assert!(weight != Weight::zero());
            
            // Verify all storage items removed
            assert!(!SubnetworkN::<Test>::contains_key(netuid));
            assert!(!NetworksAdded::<Test>::contains_key(netuid));
            assert!(!NetworkRegisteredAt::<Test>::contains_key(netuid));
            assert!(!Rank::<Test>::contains_key(netuid));
            assert!(!Trust::<Test>::contains_key(netuid));
            assert!(!Active::<Test>::contains_key(netuid));
            assert!(!Emission::<Test>::contains_key(netuid));
            assert!(!Consensus::<Test>::contains_key(netuid));
            assert!(!Dividends::<Test>::contains_key(netuid));
            assert!(!PruningScores::<Test>::contains_key(netuid));
            assert!(!ValidatorPermit::<Test>::contains_key(netuid));
            assert!(!ValidatorTrust::<Test>::contains_key(netuid));
            assert!(!Tempo::<Test>::contains_key(netuid));
            assert!(!Kappa::<Test>::contains_key(netuid));
            assert!(!Difficulty::<Test>::contains_key(netuid));
            assert!(!MaxAllowedUids::<Test>::contains_key(netuid));
            assert!(!ImmunityPeriod::<Test>::contains_key(netuid));
            assert!(!ActivityCutoff::<Test>::contains_key(netuid));
            assert!(!MinAllowedWeights::<Test>::contains_key(netuid));
            assert!(!RegistrationsThisInterval::<Test>::contains_key(netuid));
            assert!(!POWRegistrationsThisInterval::<Test>::contains_key(netuid));
            assert!(!BurnRegistrationsThisInterval::<Test>::contains_key(netuid));
        });
    }

    /// Test that weight calculation is accurate for migration
    #[test]
    fn test_migrate_delete_subnet_3_weight_calculation() {
        new_test_ext(1).execute_with(|| {
            // Setup
            let netuid = NetUid::from(3);
            add_network(netuid, 100, 0);
            StorageVersion::new(4).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_delete_subnet_3::<Test>();
            
            // Verify weight is non-zero and includes all expected operations
            // Weight should include:
            // - 1 read (version check)
            // - 5 writes (initial removals)
            // - 4 writes (prefix clears)
            // - 11 writes (network parameters)
            // - 10 writes (erase parameters)
            // - 1 write (storage version update)
            // Total: 1 read + 31 writes minimum
            assert!(weight.ref_time() > 0);
            
            let expected_min_weight = Test::DbWeight::get().reads(1)
                .saturating_add(Test::DbWeight::get().writes(31));
            
            assert!(weight.ref_time() >= expected_min_weight.ref_time());
        });
    }

    /// Test migration preserves other subnets
    #[test]
    fn test_migrate_delete_subnet_3_preserves_other_subnets() {
        new_test_ext(1).execute_with(|| {
            // Setup: Create subnet 1, 2, and 3
            add_network(NetUid::from(1), 100, 0);
            add_network(NetUid::from(2), 100, 0);
            add_network(NetUid::from(3), 100, 0);
            
            assert_eq!(TotalNetworks::<Test>::get(), 3);
            
            StorageVersion::new(4).put::<Pallet<Test>>();
            
            // Run migration
            migrate_delete_subnet_3::<Test>();
            
            // Verify only subnet 3 removed
            assert!(Pallet::<Test>::if_subnet_exist(NetUid::from(1)));
            assert!(Pallet::<Test>::if_subnet_exist(NetUid::from(2)));
            assert!(!Pallet::<Test>::if_subnet_exist(NetUid::from(3)));
            assert_eq!(TotalNetworks::<Test>::get(), 2);
        });
    }
}
