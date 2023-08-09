use super::*;
use alloc::collections::BTreeMap;
use log::{info};
use frame_support::{
	traits::{Get, StorageVersion, GetStorageVersion},
	weights::Weight, storage_alias,
    pallet_prelude::{
        Identity, OptionQuery
    },
    inherent::Vec,
    storage::with_transaction,
    fail,
};

const LOG_TARGET_V2: &str = "loadedemissionmigration";

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
        info!(target: LOG_TARGET_V2, ">>> Updating the LoadedEmission to a new format {:?}", onchain_version);

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
                info!(target: LOG_TARGET_V2, "     Do migration of netuid: {:?}...", netuid); 
                
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
        StorageVersion::new(2).put::<Pallet::<T>>(); // Update to version 2 so we don't run this again.
        // One write to storage version
        weight.saturating_accrue(T::DbWeight::get().writes(1));
        
        weight
    } else {
        info!(target: LOG_TARGET_V2, "Migration to v2 already done!");
        Weight::zero()
    }
}

const LOG_TARGET_V3_RESERVED_BALANCES: &str = "reservedbalancesmigration";

fn migrate_to_v3_transaction<T: Config>(weight: Weight, reserved_balances: BTreeMap::<T::AccountId, u64>) -> DispatchResult {

    // Verify matches total coldkey stake map
    for (coldkey, stake) in reserved_balances {
        let coldkey_reserved_balance = TotalColdkeyStake::<T>::get(&coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));
        if coldkey_reserved_balance != stake {
            log::warn!(target: LOG_TARGET_V3_RESERVED_BALANCES, "Coldkey {:?} has a reserved balance of {:?} but a total coldkey stake of {:?}", coldkey, stake, coldkey_reserved_balance);
            fail!("Coldkey reserved balance does not match total coldkey stake");
        }
        
        // Verify can convert to balance
        let stake_as_balance = <T as pallet::Config>::Reserva::Balance::try_from(stake);
        
        if stake_as_balance.is_err() {
            log::warn!(target: LOG_TARGET_V3_RESERVED_BALANCES, "Coldkey {:?} has a reserved balance of {:?} which cannot be converted to a balance", coldkey, stake);
            fail!("Coldkey reserved balance cannot be converted to a balance");
        }

        // Verify can deposit into free balance
        let deposited = <T>::Currency::deposit_creating(&coldkey, stake_as_balance.unwrap());
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        let undeposited = stake_as_balance.unwrap() - deposited;
        if undeposited > <T>::Currency::Balance::zero() {
            // Failed to deposit all the stake
            log::warn!(target: LOG_TARGET_V3_RESERVED_BALANCES, "Coldkey {:?} has a stake of {:?} which could not be issued/deposited. Deposited: {:?}", coldkey, stake, deposited);

            // De-issue the stake that was just deposited
            // Won't fail because we know it was deposited
            <T>::Currency::settle(&coldkey, deposited, WithdrawReasons::TRANSFER, ExistenceRequirement::KeepAlive).unwrap(); 
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
            fail!("Failed to deposit all the stake") // Fails transaction
        } 
        
        // Reserve the stake that was just deposited
        if !<T>::Currency::can_reserve(&coldkey, stake_as_balance) {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            log::warn!(target: LOG_TARGET_V3_RESERVED_BALANCES, "Coldkey {:?} has a free balance of {:?} from the stake which cannot be reserved", coldkey, stake);
            fail!("Failed to reserve all the stake") // Fails transaction
        } else {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            if <T>::Currency::reserve(&coldkey, stake_as_balance).is_err() {
                weight.saturating_accrue(T::DbWeight::get().reads(1));

                log::warn!(target: LOG_TARGET_V3_RESERVED_BALANCES, "Coldkey {:?} has a free balance of {:?} from the stake which cannot be reserved", coldkey, stake);
                fail!("Failed to reserve all the stake") // Fails transaction
            }
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        }
    }

    // Update storage version.
    StorageVersion::new(3).put::<Pallet::<T>>(); // Update to version 3 so we don't run this again.
    // One write to storage version
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    Ok(())
}

pub fn migrate_to_v3_stake_as_reserved_balances<T: Config>() -> Weight {
     // Check storage version
    let mut weight = T::DbWeight::get().reads_writes(1, 0);

    // Grab current version
    let onchain_version =  Pallet::<T>::on_chain_storage_version();

    // Only runs if we haven't already updated version to 2.
    if onchain_version < 3 { 
        info!(target: LOG_TARGET_V3_RESERVED_BALANCES, ">>> Updating the balances to a new format {:?}", onchain_version);
        // We set the reserved balance equal to the total coldkey stake for each coldkey
        //     This will NOT effect the stake map values. READ-ONLY

        // Should be a Vec<(Hotkey, Coldkey, Stake)>ß
        let stake_map = Stake::<T>::iter().collect::<Vec<_>>();
        weight.saturating_accrue(T::DbWeight::get().reads(stake_map.len() as u64));

        let mut reserved_balances = BTreeMap::<T::AccountId, u64>::new();
        
        for (hotkey, coldkey, stake) in stake_map {
            weight.saturating_accrue(T::DbWeight::get().reads(1));

            let reserved_balance = stake;

            if !reserved_balances.contains_key(&coldkey) {
                reserved_balances.insert(coldkey.clone(), 0);
            }

            let mut coldkey_reserved_balance = reserved_balances.get_mut(&coldkey).unwrap();
            *coldkey_reserved_balance += reserved_balance;
        }

        let transactional_migration_result = with_transaction(|| migrate_to_v3_transaction(weight, reserved_balances));
        if transactional_migration_result.is_err() {
            log::error!(target: LOG_TARGET_V3_RESERVED_BALANCES, "{:?}", transactional_migration_result.unwrap_err());
        }
        
        weight
    } else {
        info!(target: LOG_TARGET_V3_RESERVED_BALANCES, "Migration to v3 already done!");
        Weight::zero()
    }
}
