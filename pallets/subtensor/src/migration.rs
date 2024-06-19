use super::*;
use alloc::collections::BTreeMap;
use frame_support::traits::DefensiveResult;
use frame_support::{
    pallet_prelude::{Blake2_128Concat, Identity, OptionQuery, ValueQuery},
    storage_alias,
    traits::{fungible::Inspect as _, Get, GetStorageVersion, StorageVersion},
    weights::Weight,
};
use log::info;
use sp_std::vec::Vec;

// TODO (camfairchild): TEST MIGRATION

const LOG_TARGET: &str = "loadedemissionmigration";

pub mod deprecated_loaded_emission_format {
    use super::*;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[storage_alias]
    pub(super) type LoadedEmission<T: Config> =
        StorageMap<Pallet<T>, Identity, u16, Vec<(AccountIdOf<T>, u64)>, OptionQuery>;
}

pub mod deprecated_stake_variables {
    use super::*;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    #[storage_alias] // --- MAP ( hot ) --> stake | Returns the total amount of stake under a hotkey.
    pub type TotalHotkeyStake<T: Config> =
        StorageMap<Pallet<T>, Identity, AccountIdOf<T>, u64, ValueQuery>;
    #[storage_alias] // --- MAP ( cold ) --> stake | Returns the total amount of stake under a coldkey.
    pub type TotalColdkeyStake<T: Config> =
        StorageMap<Pallet<T>, Identity, AccountIdOf<T>, u64, ValueQuery>;
    #[storage_alias] // --- DMAP ( hot, cold ) --> stake | Returns the stake under a coldkey prefixed by hotkey.
    pub type Stake<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Blake2_128Concat,
        AccountIdOf<T>,
        Identity,
        AccountIdOf<T>,
        u64,
        ValueQuery,
    >;
}

/// Performs migration to update the total issuance based on the sum of stakes and total balances.
/// This migration is applicable only if the current storage version is 5, after which it updates the storage version to 6.
///
/// # Returns
/// Weight of the migration process.
pub fn migration5_total_issuance<T: Config>(test: bool) -> Weight {
    let mut weight = T::DbWeight::get().reads(1); // Initialize migration weight

    use deprecated_stake_variables as old;

    // Grab current version
    let new_storage_version = 6;
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version past above new_storage_version.
    if onchain_version < new_storage_version {
        // Execute migration if the current storage version is 5
        if Pallet::<T>::on_chain_storage_version() == StorageVersion::new(5) || test {
            // Calculate the sum of all stake values
            let stake_sum: u64 = old::Stake::<T>::iter().fold(0, |accumulator, (_, _, stake_value)| {
                accumulator.saturating_add(stake_value)
            });
            weight = weight
                .saturating_add(T::DbWeight::get().reads_writes(old::Stake::<T>::iter().count() as u64, 0));

            // Calculate the sum of all stake values
            let locked_sum: u64 = SubnetLocked::<T>::iter()
                .fold(0, |accumulator, (_, locked_value)| {
                    accumulator.saturating_add(locked_value)
                });
            weight = weight.saturating_add(
                T::DbWeight::get().reads_writes(SubnetLocked::<T>::iter().count() as u64, 0),
            );

            // Retrieve the total balance sum
            let total_balance = T::Currency::total_issuance();
            weight = weight.saturating_add(T::DbWeight::get().reads(1));

            // Compute the total issuance value
            let total_issuance_value: u64 = stake_sum + total_balance + locked_sum;

            // Update the total issuance in storage
            TotalIssuance::<T>::put(total_issuance_value);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
        }

        // Update the storage version to 6
        StorageVersion::new(new_storage_version).put::<Pallet<T>>();
        weight = weight.saturating_add(T::DbWeight::get().writes(1));
    }

    weight // Return the computed weight of the migration process
}

pub fn migrate_transfer_ownership_to_foundation<T: Config>(coldkey: [u8; 32]) -> Weight {
    let new_storage_version = 3;

    // Setup migration weight
    let mut weight = T::DbWeight::get().reads(1);

    // Grab current version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version past above new_storage_version.
    if onchain_version < new_storage_version {
        info!(
            target: LOG_TARGET_1,
            ">>> Migrating subnet 1 and 11 to foundation control {:?}", onchain_version
        );

        // We have to decode this using a byte slice as we don't have crypto-std
        let coldkey_account: <T as frame_system::Config>::AccountId =
            <T as frame_system::Config>::AccountId::decode(&mut &coldkey[..])
                .expect("coldkey is 32-byte array; qed");
        info!("Foundation coldkey: {:?}", coldkey_account);

        let current_block = Pallet::<T>::get_current_block_as_u64();
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        // Migrate ownership and set creation time as now
        SubnetOwner::<T>::insert(1, coldkey_account.clone());
        SubnetOwner::<T>::insert(11, coldkey_account);

        // We are setting the NetworkRegisteredAt storage to a future block to extend the immunity period to 2 weeks
        NetworkRegisteredAt::<T>::insert(1, current_block.saturating_add(13 * 7200));
        NetworkRegisteredAt::<T>::insert(11, current_block);

        weight.saturating_accrue(T::DbWeight::get().writes(4));

        // Update storage version.
        StorageVersion::new(new_storage_version).put::<Pallet<T>>(); // Update to version so we don't run this again.
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET_1, "Migration to v3 already done!");
        Weight::zero()
    }
}

pub fn migrate_create_root_network<T: Config>() -> Weight {
    // Get the root network uid.
    let root_netuid: u16 = 0;

    // Setup migration weight
    let mut weight = T::DbWeight::get().reads(1);

    // Check if root network already exists.
    if NetworksAdded::<T>::get(root_netuid) {
        // Since we read from the database once to determine this
        return weight;
    }

    // Set the root network as added.
    NetworksAdded::<T>::insert(root_netuid, true);

    // Increment the number of total networks.
    TotalNetworks::<T>::mutate(|n| *n += 1);

    // Set the maximum number to the number of senate members.
    MaxAllowedUids::<T>::insert(root_netuid, 64);

    // Set the maximum number to the number of validators to all members.
    MaxAllowedValidators::<T>::insert(root_netuid, 64);

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

    // Set weight setting rate limit to 1 day
    //WeightsSetRateLimit::<T>::insert(root_netuid, 7200);

    // Add our weights for writing to database
    weight.saturating_accrue(T::DbWeight::get().writes(8));

    // Empty senate members entirely, they will be filled by by registrations
    // on the subnet.
    for hotkey_i in T::SenateMembers::members().iter() {
        T::TriumvirateInterface::remove_votes(hotkey_i).defensive_ok();
        T::SenateMembers::remove_member(hotkey_i).defensive_ok();

        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));
    }

    weight
}

pub fn migrate_delete_subnet_3<T: Config>() -> Weight {
    let new_storage_version = 5;

    // Setup migration weight
    let mut weight = T::DbWeight::get().reads(1);

    // Grab current version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version past above new_storage_version.
    if onchain_version < new_storage_version && Pallet::<T>::if_subnet_exist(3) {
        info!(
            target: LOG_TARGET_1,
            ">>> Removing subnet 3 {:?}", onchain_version
        );

        let netuid = 3;

        // We do this all manually as we don't want to call code related to giving subnet owner back their locked token cost.
        // --- 2. Remove network count.
        SubnetworkN::<T>::remove(netuid);

        // --- 3. Remove network modality storage.
        NetworkModality::<T>::remove(netuid);

        // --- 4. Remove netuid from added networks.
        NetworksAdded::<T>::remove(netuid);

        // --- 6. Decrement the network counter.
        TotalNetworks::<T>::mutate(|n| *n -= 1);

        // --- 7. Remove various network-related storages.
        NetworkRegisteredAt::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(5));

        // --- 8. Remove incentive mechanism memory.
        let _ = Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Bonds::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Weights::<T>::clear_prefix(netuid, u32::MAX, None);

        weight.saturating_accrue(T::DbWeight::get().writes(4));

        // --- 9. Remove various network-related parameters.
        Rank::<T>::remove(netuid);
        Trust::<T>::remove(netuid);
        Active::<T>::remove(netuid);
        Emission::<T>::remove(netuid);
        Incentive::<T>::remove(netuid);
        Consensus::<T>::remove(netuid);
        Dividends::<T>::remove(netuid);
        PruningScores::<T>::remove(netuid);
        LastUpdate::<T>::remove(netuid);
        ValidatorPermit::<T>::remove(netuid);
        ValidatorTrust::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(11));

        // --- 10. Erase network parameters.
        Tempo::<T>::remove(netuid);
        Kappa::<T>::remove(netuid);
        Difficulty::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        ActivityCutoff::<T>::remove(netuid);
        EmissionValues::<T>::remove(netuid);
        MaxWeightsLimit::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        RegistrationsThisInterval::<T>::remove(netuid);
        POWRegistrationsThisInterval::<T>::remove(netuid);
        BurnRegistrationsThisInterval::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(12));

        // Update storage version.
        StorageVersion::new(new_storage_version).put::<Pallet<T>>(); // Update version so we don't run this again.
                                                                     // One write to storage version
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET_1, "Migration to v3 already done!");
        Weight::zero()
    }
}

pub fn migrate_delete_subnet_21<T: Config>() -> Weight {
    let new_storage_version = 4;

    // Setup migration weight
    let mut weight = T::DbWeight::get().reads(1);

    // Grab current version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version past above new_storage_version.
    if onchain_version < new_storage_version && Pallet::<T>::if_subnet_exist(21) {
        info!(
            target: LOG_TARGET_1,
            ">>> Removing subnet 21 {:?}", onchain_version
        );

        let netuid = 21;

        // We do this all manually as we don't want to call code related to giving subnet owner back their locked token cost.
        // --- 2. Remove network count.
        SubnetworkN::<T>::remove(netuid);

        // --- 3. Remove network modality storage.
        NetworkModality::<T>::remove(netuid);

        // --- 4. Remove netuid from added networks.
        NetworksAdded::<T>::remove(netuid);

        // --- 6. Decrement the network counter.
        TotalNetworks::<T>::mutate(|n| *n -= 1);

        // --- 7. Remove various network-related storages.
        NetworkRegisteredAt::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(5));

        // --- 8. Remove incentive mechanism memory.
        let _ = Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Bonds::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Weights::<T>::clear_prefix(netuid, u32::MAX, None);

        weight.saturating_accrue(T::DbWeight::get().writes(4));

        // --- 9. Remove various network-related parameters.
        Rank::<T>::remove(netuid);
        Trust::<T>::remove(netuid);
        Active::<T>::remove(netuid);
        Emission::<T>::remove(netuid);
        Incentive::<T>::remove(netuid);
        Consensus::<T>::remove(netuid);
        Dividends::<T>::remove(netuid);
        PruningScores::<T>::remove(netuid);
        LastUpdate::<T>::remove(netuid);
        ValidatorPermit::<T>::remove(netuid);
        ValidatorTrust::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(11));

        // --- 10. Erase network parameters.
        Tempo::<T>::remove(netuid);
        Kappa::<T>::remove(netuid);
        Difficulty::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        ActivityCutoff::<T>::remove(netuid);
        EmissionValues::<T>::remove(netuid);
        MaxWeightsLimit::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        RegistrationsThisInterval::<T>::remove(netuid);
        POWRegistrationsThisInterval::<T>::remove(netuid);
        BurnRegistrationsThisInterval::<T>::remove(netuid);

        weight.saturating_accrue(T::DbWeight::get().writes(12));

        // Update storage version.
        StorageVersion::new(new_storage_version).put::<Pallet<T>>(); // Update version so we don't run this again.
                                                                     // One write to storage version
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        weight
    } else {
        info!(target: LOG_TARGET_1, "Migration to v4 already done!");
        Weight::zero()
    }
}

pub fn migrate_to_v1_separate_emission<T: Config>() -> Weight {
    use deprecated_loaded_emission_format as old;
    // Check storage version
    let mut weight = T::DbWeight::get().reads_writes(1, 0);

    // Grab current version
    let onchain_version = Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version to 1.
    if onchain_version < 1 {
        info!(
            target: LOG_TARGET,
            ">>> Updating the LoadedEmission to a new format {:?}", onchain_version
        );

        // We transform the storage values from the old into the new format.

        // Start by removing any undecodable entries.
        let curr_loaded_emission: Vec<u16> = old::LoadedEmission::<T>::iter_keys().collect();
        for netuid in curr_loaded_emission {
            // Iterates over the netuids
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            if old::LoadedEmission::<T>::try_get(netuid).is_err() {
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
                info!(
                    target: LOG_TARGET,
                    "     Do migration of netuid: {:?}...", netuid
                );

                // We will assume all loaded emission is validator emissions,
                // so this will get distributed over delegatees (nominators), if there are any
                // This will NOT effect any servers that are not (also) a delegate validator.
                // server_emission will be 0 for any alread loaded emission.

                let mut new_netuid_emissions = Vec::new();
                for (server, validator_emission) in netuid_emissions {
                    new_netuid_emissions.push((server, 0_u64, validator_emission));
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
        info!(target: LOG_TARGET_1, "Migration to v2 already done!");
        Weight::zero()
    }
}

const LOG_TARGET_1: &str = "fixtotalstakestorage";

pub fn migrate_stake_to_substake<T: Config>() -> Weight {
    let new_storage_version = 7;
    let mut weight = T::DbWeight::get().reads_writes(1, 1);

    use deprecated_stake_variables as old;

    let onchain_version = Pallet::<T>::on_chain_storage_version();
    log::info!("Current on-chain storage version: {:?}", onchain_version); // Debug print
    if onchain_version < new_storage_version {
        log::info!("Starting migration from Stake to SubStake."); // Debug print
        let mut counter = 0;
        old::Stake::<T>::iter().for_each(|(coldkey, hotkey, stake)| {
            if stake > 0 {
                // Ensure we're only migrating non-zero stakes
                // Insert into SubStake with netuid set to 0 for all entries
                SubStake::<T>::insert((&coldkey, &hotkey, &0u16), stake);
                // Accrue read and write weights
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
                counter += 1;
            }
        });
        log::info!("Inserted {} entries into SubStake", counter);

        // Assuming TotalHotkeySubStake needs to be updated similarly
        let mut total_stakes: BTreeMap<T::AccountId, u64> = BTreeMap::new();
        let mut total_subnet_stakes: BTreeMap<u16, u64> = BTreeMap::new();
        SubStake::<T>::iter().for_each(|((coldkey, hotkey, netuid), stake)| {
            *total_stakes.entry(hotkey.clone()).or_insert(0) += stake;
            *total_subnet_stakes.entry(netuid).or_insert(0) += stake;
            if stake > 0 {
                Staker::<T>::insert(&coldkey, &hotkey, true);
                weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
            }
        });

        for (hotkey, total_stake) in total_stakes.iter() {
            TotalHotkeySubStake::<T>::insert(hotkey, &0u16, *total_stake);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
        }
        log::info!(
            "Inserted {} entries into TotalHotkeySubStake",
            total_stakes.len()
        );

        // For STAO the total stake is the same thing as TotalSubnetTAO for DTAO, so
        // we are using this map for both STAO and DTAO.
        for (netuid, total_stake) in total_subnet_stakes.iter() {
            TotalSubnetTAO::<T>::insert(netuid, total_stake);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
        }
        log::info!(
            "Inserted {} entries into TotalSubnetTAO",
            total_subnet_stakes.len()
        );

        // Remove the old `TotalStake` type.
        frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
            "SubtensorModule".as_bytes(),
            "TotalStake".as_bytes(),
        ));

        // Update the storage version to indicate this migration has been completed
        log::info!(
            "Migration completed, updating storage version to: {:?}",
            new_storage_version
        ); // Debug print
        StorageVersion::new(new_storage_version).put::<Pallet<T>>();
        weight += T::DbWeight::get().writes(1);
    } else {
        log::info!("Migration to fill SubStake from Stake already done!"); // Debug print
    }

    log::info!("Final weight: {:?}", weight); // Debug print
    weight
}

pub fn migrate_remove_deprecated_stake_variables<T: Config>() -> Weight {
    let new_storage_version = 8;
    let mut weight = T::DbWeight::get().reads_writes(1, 1);

    use deprecated_stake_variables as old;

    let onchain_version = Pallet::<T>::on_chain_storage_version();
    log::info!("Current on-chain storage version: {:?}", onchain_version); // Debug print
    if onchain_version < new_storage_version {
        log::info!("Starting migration: Remove TotalColdkeyStake and TotalHotkeyStake."); // Debug print
        old::TotalHotkeyStake::<T>::iter().for_each(|(hotkey, _)| {
            old::TotalHotkeyStake::<T>::remove(hotkey);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        });

        old::TotalColdkeyStake::<T>::iter().for_each(|(hotkey, _)| {
            old::TotalColdkeyStake::<T>::remove(hotkey);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        });

        // Update the storage version to indicate this migration has been completed
        log::info!(
            "Migration completed, updating storage version to: {:?}",
            new_storage_version
        ); // Debug print
        StorageVersion::new(new_storage_version).put::<Pallet<T>>();
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        // Remove Stake values
        // old::Stake::<T>::translate(|_, _, _| {
        //     weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
        //     None
        // });
    } else {
        log::info!("Migration to remove deprecated storage variables already done!");
        // Debug print
    }

    log::info!("Final weight: {:?}", weight); // Debug print
    weight
}

pub fn migrate_populate_subnet_creator<T: Config>() -> Weight {
    let new_storage_version = 9;
    let mut weight = T::DbWeight::get().reads_writes(1, 1);

    let onchain_version = Pallet::<T>::on_chain_storage_version();
    log::info!("Current on-chain storage version: {:?}", onchain_version);
    if onchain_version < new_storage_version {
        log::info!("Starting migration: Populate subnet creator.");
        SubnetOwner::<T>::iter().for_each(|(netuid, owner)| {
            SubnetCreator::<T>::insert(netuid, owner);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        });
        StorageVersion::new(new_storage_version).put::<Pallet<T>>();
    } else {
        log::info!("Migration to populate subnet creator already done!");
    }

    log::info!("Final weight: {:?}", weight);
    weight
}

pub fn migrate_subnet_locked_to_owner_substake<T: Config>() -> Weight {
    let new_storage_version = 10;
    let migration_name = "Add SubnetLocked to owner SubStake and TotalSubnetTAO";
    let mut weight = T::DbWeight::get().reads_writes(1, 1);

    let onchain_version = Pallet::<T>::on_chain_storage_version();
    log::info!("Current on-chain storage version: {:?}", onchain_version);
    if onchain_version < new_storage_version {
        log::info!("Starting migration: {}.", migration_name);

        SubnetLocked::<T>::iter().for_each(|(netuid, lock)| {
            TotalSubnetTAO::<T>::mutate(netuid, |balance| *balance = balance.saturating_add(lock));
            let coldkey = SubnetOwner::<T>::get(netuid);
            let hotkey = SubnetCreator::<T>::get(netuid);

            // Same as calling increase_subnet_token_on_coldkey_hotkey_account
            TotalHotkeySubStake::<T>::mutate(&hotkey, netuid, |stake| {
                *stake = stake.saturating_add(lock);
            });
            SubStake::<T>::mutate((&coldkey, &hotkey, netuid), |stake| {
                *stake = stake.saturating_add(lock)
            });
            Staker::<T>::insert(hotkey, coldkey, true);            

            weight.saturating_accrue(T::DbWeight::get().reads_writes(3, 4));
        });

        StorageVersion::new(new_storage_version).put::<Pallet<T>>();
    } else {
        log::info!("Migration already done: {}", migration_name);
    }

    log::info!("Final weight: {:?}", weight);
    weight
}