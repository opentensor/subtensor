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
    owner_ss58: DeAccountId
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

        let owner = <Owner<T>>::get( delegate.clone() );

        return Some( DelegateInfo {
            delegate_ss58: delegate.clone().encode().into(),
            take: <Delegates<T>>::get( delegate.clone() ),
            nominators,
            owner_ss58: owner.clone().encode().into()
        });
	}

    pub fn get_delegates() -> Vec<DelegateInfo> {
        let mut delegates = Vec::<DelegateInfo>::new();
        for ( delegate, take ) in < Delegates<T> as IterableStorageMap<T::AccountId, u16> >::iter() {
            let mut nominators = Vec::<(DeAccountId, u64)>::new();
            for ( nominator, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( delegate.clone() ) {
                nominators.push( ( nominator.clone().encode().into(), stake ) );
            }

            let owner = <Owner<T>>::get( delegate.clone() );

            delegates.push( DelegateInfo {
                delegate_ss58: delegate.clone().encode().into(),
                take,
                nominators,
                owner_ss58: owner.clone().encode().into()
            });
        }

        return delegates;
	}
}

