use super::*;
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;
use codec::Compact;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct StakeInfo<T: Config> {
    hotkey: T::AccountId,
    coldkey: T::AccountId,
    stake: Compact<u64>,
}

impl<T: Config> Pallet<T> {
	fn _get_stake_info_for_coldkeys(coldkeys: Vec<T::AccountId>) -> Vec<StakeInfo<T>> {
		if coldkeys.len() == 0 {
			return Vec::new(); // No coldkeys to check
		}

		let mut stake_info: Vec<StakeInfo<T>> = Vec::new();
        for (hotkey, coldkey, stake) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter() {
			if coldkeys.contains(&coldkey) {
				stake_info.push(StakeInfo {
					hotkey,
					coldkey,
					stake: Compact(stake),
				});
        	}
		}
		return stake_info;
	}

	pub fn get_stake_info_for_coldkeys(coldkey_account_vecs: Vec<Vec<u8>>) -> Vec<StakeInfo<T>> {
		let mut coldkeys: Vec<T::AccountId> = Vec::new();
		for coldkey_account_vec in coldkey_account_vecs {
			if coldkey_account_vec.len() != 32 {
				continue; // Invalid coldkey
			}
			let coldkey: AccountIdOf<T> = T::AccountId::decode( &mut coldkey_account_vec.as_bytes_ref() ).unwrap();
			coldkeys.push(coldkey);
		}

		if coldkeys.len() == 0 {
			return Vec::new(); // Invalid coldkey
		}

		let stake_info = Self::_get_stake_info_for_coldkeys(coldkeys);

		return stake_info;
	}

	pub fn get_stake_info_for_coldkey(coldkey_account_vec: Vec<u8>) -> Vec<StakeInfo<T>> {
		if coldkey_account_vec.len() != 32 {
            return Vec::new(); // Invalid coldkey
        }

		let coldkey: AccountIdOf<T> = T::AccountId::decode( &mut coldkey_account_vec.as_bytes_ref() ).unwrap();
		let stake_info = Self::_get_stake_info_for_coldkeys(vec![coldkey]);

        return stake_info;
	}
}

