use super::*;
use frame_support::IterableStorageDoubleMap;
use frame_support::storage::IterableStorageMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;
use sp_core::hexdisplay::AsBytesRef;


#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DelegateInfo<T: Config> {
    delegate_ss58: T::AccountId,
    take: u16,
    nominators: Vec<(T::AccountId, u64)>, // map of nominator_ss58 to stake amount
    owner_ss58: T::AccountId
}

impl<T: Config> Pallet<T> {
	pub fn get_delegate( delegate_account_vec: Vec<u8> ) -> Option<DelegateInfo<T>> {
        if delegate_account_vec.len() != 32 {
            return None;
        }

        let delegate: AccountIdOf<T> = T::AccountId::decode( &mut delegate_account_vec.as_bytes_ref() ).unwrap();

        let mut nominators = Vec::<(T::AccountId, u64)>::new();

        for ( nominator, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( delegate.clone() ) {
            nominators.push( ( nominator.clone(), stake ) );
        }

        let owner = <Owner<T>>::get( delegate.clone() );

        return Some( DelegateInfo {
            delegate_ss58: delegate.clone(),
            take: <Delegates<T>>::get( delegate.clone() ),
            nominators,
            owner_ss58: owner.clone()
        });
	}

    pub fn get_delegates() -> Vec<DelegateInfo<T>> {
        let mut delegates = Vec::<DelegateInfo<T>>::new();
        for ( delegate, take ) in < Delegates<T> as IterableStorageMap<T::AccountId, u16> >::iter() {
            let mut nominators = Vec::<(T::AccountId, u64)>::new();
            for ( nominator, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( delegate.clone() ) {
                nominators.push( ( nominator.clone(), stake ) );
            }

            let owner = <Owner<T>>::get( delegate.clone() );

            delegates.push( DelegateInfo {
                delegate_ss58: delegate.clone(),
                take,
                nominators,
                owner_ss58: owner.clone()
            });
        }

        return delegates;
	}
}

