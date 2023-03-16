use super::*;
use frame_support::IterableStorageDoubleMap;
use serde::{Serialize, Deserialize};
use frame_support::storage::IterableStorageMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;
use sp_core::hexdisplay::AsBytesRef;


#[derive(Decode, Encode, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DelegateInfo {
    delegate_ss58: DeAccountId,
    take: u16,
    nominators: Vec<(DeAccountId, u64)>, // map of nominator_ss58 to stake amount
    owner_ss58: DeAccountId,
    registrations: Vec<u16>, // vec of subnets this delegate is registered on
    validator_permits: Vec<u16>, // vec of subnets this delegate has validator permits for
    return_per_1000: u64, // return of delegator's stake per 1000 TAO staked over 24h -> 7200 blocks
}

impl<T: Config> Pallet<T> {
	pub fn get_delegate( delegate_account_vec: Vec<u8> ) -> Option<DelegateInfo> {
        if delegate_account_vec.len() != 32 {
            return None;
        }

        let delegate: AccountIdOf<T> = T::AccountId::decode( &mut delegate_account_vec.as_bytes_ref() ).unwrap();
        
        let mut nominators = Vec::<(DeAccountId, u64)>::new();

        for ( nominator, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( delegate.clone() ) {
            nominators.push( ( nominator.clone().encode().into(), stake ) );
        }

        let registrations = Self::get_registered_networks_for_hotkey( &delegate.clone() );
        let mut validator_permits = Vec::<u16>::new();
        let mut emissions: u128 = 0;
        
        for netuid in registrations.iter() {
            let _uid = Self::get_uid_for_net_and_hotkey( *netuid, &delegate.clone());
            if !_uid.is_ok() {
                continue; // this should never happen
            } else {
                let uid = _uid.expect("Delegate's UID should be ok");
                let validator_permit = Self::get_validator_permit_for_uid( *netuid, uid );
                if validator_permit {
                    validator_permits.push( *netuid );
                }
                
                let emission = Self::get_emission_for_uid( *netuid, uid) as u128;
                emissions += emission;
            }
        }
            

        let owner = Self::get_owning_coldkey_for_hotkey( &delegate.clone() );
        let take = <Delegates<T>>::get( delegate.clone() );

        let total_stake = Self::get_total_stake_for_hotkey( &delegate.clone() );
        
        let emissions_per_day = emissions * 72;
        let return_per_1000 = emissions_per_day / (total_stake as u128 / 1000);

        return Some( DelegateInfo {
            delegate_ss58: delegate.clone().encode().into(),
            take,
            nominators,
            owner_ss58: owner.clone().encode().into(),
            registrations,
            validator_permits,
            return_per_1000: return_per_1000 as u64,
        });
	}



    pub fn get_delegates() -> Vec<DelegateInfo> {
        let mut delegates = Vec::<DelegateInfo>::new();
        for delegate in < Delegates<T> as IterableStorageMap<T::AccountId, u16> >::iter_keys() {
            
            let delegate_info = Self::get_delegate( delegate.clone().encode() );
            if delegate_info.is_none() {
                continue;
            } else {
                delegates.push( delegate_info.expect("DelegateInfo was None after check") );
            }
        }

        return delegates;
	}
}

