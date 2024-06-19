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
