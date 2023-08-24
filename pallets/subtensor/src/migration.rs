use super::*;
use frame_support::{
    inherent::Vec,
    pallet_prelude::{Identity, OptionQuery},
    storage_alias,
    traits::{Get, GetStorageVersion, StorageVersion},
    weights::Weight,
};
use log::info;

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
    // Get the root network uid.
    let root_netuid: u16 = 0;

    // Check if root network already exists.
    if NetworksAdded::<T>::get(root_netuid) {
        return Weight::zero();
    }

    // Set the root network as added.
    NetworksAdded::<T>::insert(root_netuid, true);

    // Increment the number of total networks.
    TotalNetworks::<T>::mutate(|n| *n += 1);

    // Set the number of validators to 1.
    SubnetworkN::<T>::insert(root_netuid, 0);

    // Set the maximum number to the number of senate members.
    MaxAllowedUids::<T>::insert(root_netuid, T::SenateMembers::max_members() as u16);

    // Set the maximum number to the number of validators to all members.
    MaxAllowedValidators::<T>::insert(root_netuid, T::SenateMembers::max_members() as u16);

    // Set the min allowed weights to zero, no weights restrictions.
    MinAllowedWeights::<T>::insert(root_netuid, 0);

    // Set the max weight limit to infitiy, no weight restrictions.
    MaxWeightsLimit::<T>::insert(root_netuid, u16::MAX);

    // Add default root tempo.
    Tempo::<T>::insert(root_netuid, 100);

    // Set the root network as open.
    NetworkRegistrationAllowed::<T>::insert(root_netuid, true);

    // Set target registrations for validators as 1 per block.
    TargetRegistrationsPerInterval::<T>::insert(root_netuid, 1);

    // Empty senate members entirely, they will be filled by by registrations
    // on the subnet.
    for hotkey_i in T::SenateMembers::members().iter() {
        T::TriumvirateInterface::remove_votes(&hotkey_i);
        T::SenateMembers::remove_member(&hotkey_i);
    }

    // Return zero weight.
    Weight::zero()
}

pub fn migrate_to_v2_separate_emission<T: Config>() -> Weight {
    use deprecated_loaded_emission_format as old;
    // Check storage version
    let mut weight = T::DbWeight::get().reads_writes(1, 0);

    // Grab current version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

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
        StorageVersion::new(1).put::<Pallet<T>>(); // Update to version 2 so we don't run this again.
                                                   // One write to storage version
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET, "Migration to v2 already done!");
        Weight::zero()
    }
}
