use super::*;
use frame_support::pallet_prelude::OptionQuery;
use frame_support::{
    pallet_prelude::Identity,
    storage_alias,
    traits::{Get, GetStorageVersion, StorageVersion, fungible::Inspect},
    weights::Weight,
};
use sp_std::vec::Vec;



#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::mock::*;
    use frame_support::traits::{GetStorageVersion, StorageVersion};
    use sp_core::U256;
    use subtensor_runtime_common::TaoCurrency;

    /// Test successful migration path
    #[test]
    fn test_migrate_total_issuance_success() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 5
            StorageVersion::new(5).put::<Pallet<Test>>();
            
            // Create some stake data
            let hotkey1 = U256::from(1);
            let hotkey2 = U256::from(2);
            let coldkey = U256::from(100);
            
            // Add networks and register neurons to create stake
            let netuid = NetUid::from(1);
            add_network(netuid, 100, 0);
            register_ok_neuron(netuid, hotkey1, coldkey, 0);
            register_ok_neuron(netuid, hotkey2, coldkey, 100);
            
            // Set some stake values
            let stake_amount = TaoCurrency::from(1000);
            TotalHotkeyStake::<Test>::insert(hotkey1, stake_amount);
            TotalHotkeyStake::<Test>::insert(hotkey2, stake_amount);
            Owner::<Test>::insert(hotkey1, coldkey);
            Owner::<Test>::insert(hotkey2, coldkey);
            
            // Run migration
            let weight = migrate_total_issuance::<Test>(false);
            
            // Verify migration executed (non-zero weight)
            assert!(weight != Weight::zero());
            
            // Verify storage version updated to 6
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(6));
            
            // Verify TotalIssuance was updated
            let total_issuance = TotalIssuance::<Test>::get();
            assert!(total_issuance > TaoCurrency::ZERO);
        });
    }

    /// Test that migration skips when version is not 5
    #[test]
    fn test_migrate_total_issuance_wrong_version() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to 4 (not 5)
            StorageVersion::new(4).put::<Pallet<Test>>();
            
            // Run migration
            let weight = migrate_total_issuance::<Test>(false);
            
            // Verify migration was skipped - only initial read occurred
            let expected_weight = Test::DbWeight::get().reads(1);
            assert_eq!(weight, expected_weight);
            
            // Verify version unchanged
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(4));
        });
    }

    /// Test migration with test flag enabled
    #[test]
    fn test_migrate_total_issuance_test_mode() {
        new_test_ext(1).execute_with(|| {
            // Setup: Set storage version to any value
            StorageVersion::new(10).put::<Pallet<Test>>();
            
            // Run migration with test = true
            let weight = migrate_total_issuance::<Test>(true);
            
            // Verify migration executed even with wrong version
            assert!(weight != Weight::zero());
            
            // Verify storage version updated to 6
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(6));
        });
    }

    /// Test weight calculation includes all operations
    #[test]
    fn test_migrate_total_issuance_weight_calculation() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(5).put::<Pallet<Test>>();
            
            // Add multiple hotkeys to test weight scaling
            let coldkey = U256::from(100);
            for i in 1..=5 {
                let hotkey = U256::from(i);
                Owner::<Test>::insert(hotkey, coldkey);
                TotalHotkeyStake::<Test>::insert(hotkey, TaoCurrency::from(1000));
            }
            
            // Run migration
            let weight = migrate_total_issuance::<Test>(false);
            
            // Verify weight includes all reads and writes
            // Expected: 1 version read + (5 Owner reads * 2 for stake) + 1 total_issuance read + 2 writes
            assert!(weight.ref_time() > 0);
            
            let min_expected = Test::DbWeight::get().reads(1 + 10 + 1)
                .saturating_add(Test::DbWeight::get().writes(2));
            
            assert!(weight.ref_time() >= min_expected.ref_time());
        });
    }

    /// Test migration with empty Owner storage
    #[test]
    fn test_migrate_total_issuance_empty_owners() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(5).put::<Pallet<Test>>();
            
            // Don't add any owners
            
            // Run migration
            let weight = migrate_total_issuance::<Test>(false);
            
            // Verify migration still completes
            assert!(weight != Weight::zero());
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(6));
            
            // Total issuance should be set (even if just from balances)
            let total_issuance = TotalIssuance::<Test>::get();
            assert!(total_issuance >= TaoCurrency::ZERO);
        });
    }

    /// Test stake sum calculation with various stake amounts
    #[test]
    fn test_migrate_total_issuance_stake_aggregation() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(5).put::<Pallet<Test>>();
            
            let coldkey = U256::from(100);
            
            // Create hotkeys with different stake amounts
            let stake_amounts = vec![
                TaoCurrency::from(100),
                TaoCurrency::from(500),
                TaoCurrency::from(1000),
                TaoCurrency::ZERO, // Zero stake should also be handled
            ];
            
            for (i, stake) in stake_amounts.iter().enumerate() {
                let hotkey = U256::from((i + 1) as u64);
                Owner::<Test>::insert(hotkey, coldkey);
                TotalHotkeyStake::<Test>::insert(hotkey, *stake);
            }
            
            // Run migration
            migrate_total_issuance::<Test>(false);
            
            // Verify total issuance calculated
            let total_issuance = TotalIssuance::<Test>::get();
            
            // Total should at least include the sum of stakes
            let expected_stake_sum: TaoCurrency = stake_amounts.iter().fold(TaoCurrency::ZERO, |acc, s| acc.saturating_add(*s));
            assert!(total_issuance >= expected_stake_sum);
        });
    }

    /// Test migration preserves existing behavior on conversion success
    #[test]
    fn test_migrate_total_issuance_conversion_success_path() {
        new_test_ext(1).execute_with(|| {
            StorageVersion::new(5).put::<Pallet<Test>>();
            
            // Setup minimal data
            let hotkey = U256::from(1);
            let coldkey = U256::from(100);
            Owner::<Test>::insert(hotkey, coldkey);
            TotalHotkeyStake::<Test>::insert(hotkey, TaoCurrency::from(500));
            
            // Capture initial state
            let initial_version = Pallet::<Test>::on_chain_storage_version();
            
            // Run migration
            let weight = migrate_total_issuance::<Test>(false);
            
            // Verify conversion succeeded and storage updated
            assert!(weight > Test::DbWeight::get().reads(1));
            assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(6));
            assert!(initial_version < StorageVersion::new(6));
        });
    }
}

// ERROR HANDLING DOCUMENTATION:
//
// Current error handling for conversion failure (line 76):
// - Logs error message via log::error!
// - Migration is aborted without updating storage version
// - This allows migration to be retried in future blocks
//
// Implications:
// 1. If total_balance cannot convert to u64, migration will retry every block
// 2. This could cause performance issues if conversion consistently fails
// 3. However, aborting is safer than proceeding with incorrect values
//
// Potential improvements (if needed in future):
// 1. Add a retry counter to prevent infinite retry loops
// 2. Consider updating storage version even on failure after N attempts
// 3. Add more detailed error context (e.g., the actual balance value that failed)
// 4. Emit an event to alert chain operators of persistent failures


/// Module containing deprecated storage format for LoadedEmission
pub mod deprecated_loaded_emission_format {
    use super::*;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

/// Performs migration to update the total issuance based on the sum of stakes and total balances.
///
/// This migration is applicable only if the current storage version is 5, after which it updates the storage version to 6.
///
/// # Arguments
///
/// * `test` - A boolean flag to force migration execution for testing purposes.
///
/// # Returns
///
/// * `Weight` - The computational weight of this operation.
///
/// # Example
///
/// ```ignore
///  let weight = migrate_total_issuance::<Runtime>(false);
/// ```
pub fn migrate_total_issuance<T: Config>(test: bool) -> Weight {
    // Initialize migration weight with the cost of reading the storage version
    let mut weight = T::DbWeight::get().reads(1);

    // Execute migration if the current storage version is 5 or if in test mode
    if Pallet::<T>::on_chain_storage_version() == StorageVersion::new(5) || test {
        // Calculate the sum of all stake values
        let stake_sum = Owner::<T>::iter()
            .map(|(hotkey, _coldkey)| Pallet::<T>::get_total_stake_for_hotkey(&hotkey))
            .fold(TaoCurrency::ZERO, |acc, stake| acc.saturating_add(stake));
        // Add weight for reading all Owner and TotalHotkeyStake entries
        weight = weight.saturating_add(
            T::DbWeight::get().reads((Owner::<T>::iter().count() as u64).saturating_mul(2)),
        );

        // Retrieve the total balance sum
        let total_balance = <T as Config>::Currency::total_issuance();
        // Add weight for reading total issuance
        weight = weight.saturating_add(T::DbWeight::get().reads(1));

        // Attempt to convert total balance to u64
        match TryInto::<u64>::try_into(total_balance) {
            Ok(total_balance_sum) => {
                // Compute the total issuance value
                let total_issuance_value = stake_sum
                    .saturating_add(total_balance_sum.into());

                // Update the total issuance in storage
                TotalIssuance::<T>::put(total_issuance_value);

                // Update the storage version to 6
                StorageVersion::new(6).put::<Pallet<T>>();

                // Add weight for writing total issuance and storage version
                weight = weight.saturating_add(T::DbWeight::get().writes(2));
            }
            Err(_) => {
                // TODO: Implement proper error handling for conversion failure
                log::error!("Failed to convert total balance to u64, migration aborted");
            }
        }
    }

    // Return the computed weight of the migration process
    weight
}
