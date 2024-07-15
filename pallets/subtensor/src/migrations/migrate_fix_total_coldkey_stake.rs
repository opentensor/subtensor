use super::*;
use frame_support::{
    pallet_prelude::{Identity, OptionQuery},
    storage_alias,
    traits::{Get, GetStorageVersion, StorageVersion},
    weights::Weight,
};
use sp_std::vec::Vec;

// TODO (camfairchild): TEST MIGRATION
pub mod deprecated_loaded_emission_format {
    use super::*;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

/// Migrates and fixes the total coldkey stake.
///
/// This function iterates through all staking hotkeys, calculates the total stake for each coldkey,
/// and updates the `TotalColdkeyStake` storage accordingly. The migration is only performed if the
/// on-chain storage version is 6.
///
/// # Returns
/// The weight of the migration process.
pub fn do_migrate_fix_total_coldkey_stake<T: Config>() -> Weight {
    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    // Iterate through all staking hotkeys.
    for (coldkey, hotkey_vec) in StakingHotkeys::<T>::iter() {
        // Init the zero value.
        let mut coldkey_stake_sum: u64 = 0;
        weight = weight.saturating_add(T::DbWeight::get().reads(1));

        // Calculate the total stake for the current coldkey.
        for hotkey in hotkey_vec {
            // Cant fail on retrieval.
            coldkey_stake_sum =
                coldkey_stake_sum.saturating_add(Stake::<T>::get(hotkey, coldkey.clone()));
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
        }
        // Update the `TotalColdkeyStake` storage with the calculated stake sum.
        // Cant fail on insert.
        TotalColdkeyStake::<T>::insert(coldkey.clone(), coldkey_stake_sum);
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }
    weight
}
// Public migrate function to be called by Lib.rs on upgrade.
pub fn migrate_fix_total_coldkey_stake<T: Config>() -> Weight {
    let current_storage_version: u16 = 7;
    let next_storage_version: u16 = 8;

    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    // Grab the current on-chain storage version.
    // Cant fail on retrieval.
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only run this migration on storage version 6.
    if onchain_version == current_storage_version {
        weight = weight.saturating_add(do_migrate_fix_total_coldkey_stake::<T>());
        // Cant fail on insert.
        StorageVersion::new(next_storage_version).put::<Pallet<T>>();
        weight.saturating_accrue(T::DbWeight::get().writes(1));
    }

    // Return the migration weight.
    weight
}
