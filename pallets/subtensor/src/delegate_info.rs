use super::*;
use frame_support::IterableStorageDoubleMap;
use frame_support::storage::IterableStorageMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;
use sp_core::hexdisplay::AsBytesRef;
use codec::Compact;


#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DelegateInfo<T: Config> {
    delegate_ss58: T::AccountId,
    take: Compact<u16>,
    nominators: Vec<(T::AccountId, Compact<u64>)>, // map of nominator_ss58 to stake amount
    owner_ss58: T::AccountId,
    registrations: Vec<Compact<u16>>, // Vec of netuid this delegate is registered on
    validator_permits: Vec<Compact<u16>>, // Vec of netuid this delegate has validator permit on
    return_per_1000: Compact<u64>, // Delegators current daily return per 1000 TAO staked
}

impl<T: Config> Pallet<T> {
    fn get_delegate_by_existing_account( delegate: AccountIdOf<T> ) -> DelegateInfo<T> {
        let mut nominators = Vec::<(T::AccountId, Compact<u64>)>::new();

        for ( nominator, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( delegate.clone() ) {
            nominators.push( ( nominator.clone(), stake.into() ) );
        }

        let registrations = Self::get_registered_networks_for_hotkey( &delegate.clone() );
        let mut validator_permits = Vec::<Compact<u16>>::new();
        let mut emissions: u128 = 0;
        
        for netuid in registrations.iter() {
            let _uid = Self::get_uid_for_net_and_hotkey( *netuid, &delegate.clone());
            if !_uid.is_ok() {
                continue; // this should never happen
            } else {
                let uid = _uid.expect("Delegate's UID should be ok");
                let validator_permit = Self::get_validator_permit_for_uid( *netuid, uid );
                if validator_permit {
                    validator_permits.push( (*netuid).into() );
                }
                
                let emission = Self::get_emission_for_uid( *netuid, uid) as u128;
                emissions += emission;
            }
        }

        let owner = Self::get_owning_coldkey_for_hotkey( &delegate.clone() );
        let take: Compact<u16> = <Delegates<T>>::get( delegate.clone() ).into();

        let total_stake = Self::get_total_stake_for_hotkey( &delegate.clone() );
        
        let emissions_per_day = emissions * 72;
        let return_per_1000 = emissions_per_day / (total_stake as u128 / 1000);

        return DelegateInfo {
            delegate_ss58: delegate.clone(),
            take,
            nominators,
            owner_ss58: owner.clone(),
            registrations: registrations.iter().map(|x| x.into()).collect(),
            validator_permits,
            return_per_1000: (return_per_1000 as u64).into(),
        };
    }


	pub fn get_delegate( delegate_account_vec: Vec<u8> ) -> Option<DelegateInfo<T>> {
        if delegate_account_vec.len() != 32 {
            return None;
        }

        let delegate: AccountIdOf<T> = T::AccountId::decode( &mut delegate_account_vec.as_bytes_ref() ).unwrap();
        // Check delegate exists
        if !<Delegates<T>>::contains_key( delegate.clone() ) {
            return None;
        }

        let delegate_info = Self::get_delegate_by_existing_account( delegate.clone() );
        return Some( delegate_info );
	}

    pub fn get_delegates() -> Vec<DelegateInfo<T>> {
        let mut delegates = Vec::<DelegateInfo<T>>::new();
        for delegate in < Delegates<T> as IterableStorageMap<T::AccountId, u16> >::iter_keys().into_iter() {
            let delegate_info = Self::get_delegate_by_existing_account( delegate.clone() );
            delegates.push( delegate_info );
        }

        return delegates;
	}
}

