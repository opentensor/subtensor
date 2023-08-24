use super::*;
use log::{info};
use frame_support::{
    inherent::Vec,
    pallet_prelude::{Identity, OptionQuery},
    storage_alias,
    traits::{Get, GetStorageVersion, StorageVersion},
    weights::Weight,
};

// TODO (camfairchild): TEST MIGRATION

const LOG_TARGET: &str = "loadedemissionmigration";

pub mod deprecated_loaded_emission_format {
    use super::*;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

pub fn migrate_create_root_network<T: Config>() -> Weight {
    let root_netuid: u16 = 0;

    // Check if root network already exists.
    if NetworksAdded::<T>::get(root_netuid) {
        return Weight::zero();
    }
    // Build the root network if not exists.
    SubnetworkN::<T>::insert(root_netuid, 0);
    NetworksAdded::<T>::insert(root_netuid, true);
    Tempo::<T>::insert(root_netuid, 100);
    NetworkModality::<T>::insert(root_netuid, 0);
    TotalNetworks::<T>::mutate(|n| *n += 1);

    // Fill the root network params.
    MaxAllowedUids::<T>::insert(root_netuid, T::SenateMembers::max_members() as u16);
    NetworkRegistrationAllowed::<T>::insert(root_netuid, true);
    MaxAllowedValidators::<T>::insert(root_netuid, T::SenateMembers::max_members() as u16);
    MinAllowedWeights::<T>::insert(root_netuid, 0);
    MaxWeightsLimit::<T>::insert(root_netuid, u16::MAX);
    TargetRegistrationsPerInterval::<T>::insert(root_netuid, 1);

    // Empty senate.
    for hotkey_i in T::SenateMembers::members().iter() {
        T::TriumvirateInterface::remove_votes(&hotkey_i);
        T::SenateMembers::remove_member(&hotkey_i);
    }
    // Return zero weight.
    Weight::zero()
}

pub fn migrate_to_v1_separate_emission<T: Config>() -> Weight {
    use deprecated_loaded_emission_format as old;
    // Check storage version
    let mut weight = T::DbWeight::get().reads_writes(1, 0);

    // Grab current version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version to 1.
    if onchain_version < 1 {
        info!(target: LOG_TARGET, ">>> Updating the LoadedEmission to a new format {:?}", onchain_version);

        // We transform the storage values from the old into the new format.

        // Start by removing any undecodable entries.
        let curr_loaded_emission: Vec<u16> = old::LoadedEmission::<T>::iter_keys().collect();
        for netuid in curr_loaded_emission {
            // Iterates over the netuids
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            if let Err(_) = old::LoadedEmission::<T>::try_get(netuid) {
                weight.saturating_accrue(T::DbWeight::get().writes(1));
                old::LoadedEmission::<T>::remove(netuid);
                log::warn!(
                    "Was unable to decode old loaded_emisssion for netuid {}",
                    netuid
                );
            }
        }

        // Translate the old storage values into the new format.
        LoadedEmission::<T>::translate::<Vec<(AccountIdOf<T>, u64)>, _>(
            |netuid: u16,
             netuid_emissions: Vec<(AccountIdOf<T>, u64)>|
             -> Option<Vec<(AccountIdOf<T>, u64, u64)>> {
                info!(target: LOG_TARGET, "     Do migration of netuid: {:?}...", netuid);

                // We will assume all loaded emission is validator emissions,
                //      so this will get distributed over delegatees (nominators), if there are any
                //      This will NOT effect any servers that are not (also) a delegate validator. 
                // server_emission will be 0 for any alread loaded emission.
                
                let mut new_netuid_emissions = Vec::new();
                for (server, validator_emission) in netuid_emissions {
                    new_netuid_emissions.push((server, 0 as u64, validator_emission));
                }

                // One read (old) and write (new) per netuid
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

                Some(new_netuid_emissions)
            },
        );

        // Update storage version.
        StorageVersion::new(1).put::<Pallet::<T>>(); // Update to version 2 so we don't run this again.
        // One write to storage version
        weight.saturating_accrue(T::DbWeight::get().writes(1));
        
        weight
    } else {
        info!(target: LOG_TARGET_1, "Migration to v2 already done!");
        Weight::zero()
    }
}

const LOG_TARGET_1: &str = "fixtotalstakestorage";

pub fn migrate_to_v2_fixed_total_stake<T: Config>() -> Weight {
	let new_storage_version = 2;

     // Check storage version
    let mut weight = T::DbWeight::get().reads(1);

    // Grab current version
    let onchain_version =  Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version past above new_storage_version.
    if onchain_version < new_storage_version {
        info!(target: LOG_TARGET_1, ">>> Fixing the TotalStake and TotalColdkeyStake storage {:?}", onchain_version);

		// Stake and TotalHotkeyStake are known to be accurate
		// TotalColdkeyStake is known to be inaccurate
		// TotalStake is known to be inaccurate

		TotalStake::<T>::put(0); // Set to 0
		weight.saturating_accrue(T::DbWeight::get().writes(1));

		// We iterate over TotalColdkeyStake keys and set them to 0
		let total_coldkey_stake_keys = TotalColdkeyStake::<T>::iter_keys().collect::<Vec<_>>();
		for coldkey in total_coldkey_stake_keys {
			weight.saturating_accrue(T::DbWeight::get().reads(1));
			TotalColdkeyStake::<T>::insert(coldkey, 0); // Set to 0
			weight.saturating_accrue(T::DbWeight::get().writes(1));
		}

		// Now we iterate over the entire stake map, and sum each coldkey stake
		//   We also track TotalStake
		for (_, coldkey, stake) in Stake::<T>::iter() {
			weight.saturating_accrue(T::DbWeight::get().reads(1));
			// Get the current coldkey stake
			let mut total_coldkey_stake = TotalColdkeyStake::<T>::get(coldkey.clone());
			weight.saturating_accrue(T::DbWeight::get().reads(1));
			// Add the stake to the coldkey stake
			total_coldkey_stake = total_coldkey_stake.saturating_add(stake);
			// Update the coldkey stake
			TotalColdkeyStake::<T>::insert(coldkey, total_coldkey_stake);
			weight.saturating_accrue(T::DbWeight::get().writes(1));

			// Get the current total stake
			let mut total_stake = TotalStake::<T>::get();
			weight.saturating_accrue(T::DbWeight::get().reads(1));
			// Add the stake to the total stake
			total_stake = total_stake.saturating_add(stake);
			// Update the total stake
			TotalStake::<T>::put(total_stake);
			weight.saturating_accrue(T::DbWeight::get().writes(1));
		}

		// Now both TotalStake and TotalColdkeyStake are accurate

        // Update storage version.
        StorageVersion::new(new_storage_version).put::<Pallet::<T>>(); // Update to version so we don't run this again.
        // One write to storage version
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET_1, "Migration to v2 already done!");
        Weight::zero()
    }
}