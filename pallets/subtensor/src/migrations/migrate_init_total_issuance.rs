use super::*;
use frame_support::pallet_prelude::{Identity, OptionQuery, Weight};
use frame_support::storage_alias;
use sp_std::vec::Vec;

// TODO: Implement comprehensive tests for this migration

/// Module containing deprecated storage format for LoadedEmission
pub mod deprecated_loaded_emission_format {
    use super::*;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

pub(crate) fn migrate_init_total_issuance<T: Config>() -> Weight {
    // Calculate the total locked tokens across all subnets
    let subnets_len = crate::SubnetLocked::<T>::iter().count() as u64;
    let total_subnet_locked: u64 =
        crate::SubnetLocked::<T>::iter().fold(0, |acc, (_, v)| acc.saturating_add(v));

    // Retrieve the total balance of all accounts
    let total_account_balances = <<T as crate::Config>::Currency as fungible::Inspect<
        <T as frame_system::Config>::AccountId,
    >>::total_issuance();

    // Get the total stake from the system
    let total_stake = crate::TotalStake::<T>::get();

    // Retrieve the previous total issuance for logging purposes
    let prev_total_issuance = crate::TotalIssuance::<T>::get();

    // Calculate the new total issuance
    let new_total_issuance = total_account_balances
        .saturating_add(total_stake)
        .saturating_add(total_subnet_locked);

    // Update the total issuance in storage
    crate::TotalIssuance::<T>::put(new_total_issuance);

    // Log the change in total issuance
    log::info!(
        "Subtensor Pallet Total Issuance Updated: previous: {:?}, new: {:?}",
        prev_total_issuance,
        new_total_issuance
    );

    // Return the weight of the operation
    // We performed subnets_len + 5 reads and 1 write
    <T as frame_system::Config>::DbWeight::get().reads_writes(subnets_len.saturating_add(5), 1)
}

pub mod initialise_total_issuance {
    use frame_support::pallet_prelude::Weight;
    use frame_support::traits::OnRuntimeUpgrade;

    use crate::*;

    pub struct Migration<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for Migration<T> {
        /// Performs the migration to initialize and update the total issuance.
        ///
        /// This function does the following:
        /// 1. Calculates the total locked tokens across all subnets
        /// 2. Retrieves the total account balances and total stake
        /// 3. Computes and updates the new total issuance
        ///
        /// Returns the weight of the migration operation.
        fn on_runtime_upgrade() -> Weight {
            super::migrate_init_total_issuance::<T>()
        }

        /// Performs post-upgrade checks to ensure the migration was successful.
        ///
        /// This function is only compiled when the "try-runtime" feature is enabled.
        #[cfg(feature = "try-runtime")]
        fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            // Verify that all accounting invariants are satisfied after the migration
            crate::Pallet::<T>::check_total_issuance()?;
            Ok(())
        }
    }
}
