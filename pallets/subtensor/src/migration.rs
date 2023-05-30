use super::*;
use log::{info};
use frame_support::{
	traits::{Get, StorageVersion, GetStorageVersion},
	weights::Weight, storage_alias,
    pallet_prelude::{
        Identity, OptionQuery
    },
    inherent::Vec
};

// TODO (camfairchild): TEST MIGRATION

const LOG_TARGET: &str = "loadedemissionmigration";

pub mod deprecated_loaded_emission_format {
    use super::*;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[storage_alias]
    pub(super) type LoadedEmission<T:Config> = StorageMap< Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery >;
}

pub fn migrate_to_v2_separate_emission<T: Config>() -> Weight {
    use deprecated_loaded_emission_format as old;
     // Check storage version
    let mut weight = T::DbWeight::get().reads_writes(1, 0);

    // Grab current version
    let onchain_version =  Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version to 2.
    if onchain_version < 2 { 
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
				log::warn!("Was unable to decode old loaded_emisssion for netuid {}", netuid);
            }
        }

        // Translate the old storage values into the new format.
        LoadedEmission::<T>::translate::<Vec<(AccountIdOf<T>, u64)>, _>(
            |netuid: u16, netuid_emissions: Vec<(AccountIdOf<T>, u64)>| -> Option<Vec<(AccountIdOf<T>, u64, u64)>> {
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
            }
        );

        // Update storage version.
        StorageVersion::new(1).put::<Pallet::<T>>(); // Update to version 2 so we don't run this again.
        // One write to storage version
        weight.saturating_accrue(T::DbWeight::get().writes(1));
        
        weight
    } else {
        info!(target: LOG_TARGET, "Migration to v2 already done!");
        Weight::zero()
    }
}