use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use codec::Compact;
use sp_core::hexdisplay::AsBytesRef;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct StakeInfo<T: Config> {
    hotkey: T::AccountId,
    coldkey: T::AccountId,
    stake: Compact<u64>,
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetStakeInfo<T: Config> {
    hotkey: T::AccountId,
    netuid: u16,
    stake: Compact<u64>,
}

impl<T: Config> Pallet<T> {
    fn _get_stake_info_for_coldkeys(
        coldkeys: Vec<T::AccountId>,
    ) -> Vec<(T::AccountId, Vec<StakeInfo<T>>)> {
        if coldkeys.len() == 0 {
            return Vec::new(); // No coldkeys to check
        }

        let mut stake_info: Vec<(T::AccountId, Vec<StakeInfo<T>>)> = Vec::new();
        for coldkey_ in coldkeys {
            let mut stake_info_for_coldkey: Vec<StakeInfo<T>> = Vec::new();

            for ((hotkey, coldkey, netuid), stake) in <SubStake<T>>::iter() {
                if coldkey == coldkey_ {
                    stake_info_for_coldkey.push(StakeInfo {
                        hotkey,
                        coldkey,
                        stake: stake.into(),
                    });
                }
            }

            stake_info.push((coldkey_, stake_info_for_coldkey));
        }

        return stake_info;
    }

    pub fn get_stake_info_for_coldkeys(
        coldkey_account_vecs: Vec<Vec<u8>>,
    ) -> Vec<(T::AccountId, Vec<StakeInfo<T>>)> {
        let mut coldkeys: Vec<T::AccountId> = Vec::new();
        for coldkey_account_vec in coldkey_account_vecs {
            if coldkey_account_vec.len() != 32 {
                continue; // Invalid coldkey
            }
            let coldkey: AccountIdOf<T> =
                T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).unwrap();
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

        let coldkey: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).unwrap();
        let stake_info = Self::_get_stake_info_for_coldkeys(vec![coldkey]);

        if stake_info.len() == 0 {
            return Vec::new(); // Invalid coldkey
        } else {
            return stake_info.get(0).unwrap().1.clone();
        }
    }

    fn _get_stake_info_for_coldkeys_subnet(
        coldkeys: Vec<T::AccountId>,
    ) -> Vec<(T::AccountId, Vec<SubnetStakeInfo<T>>)> {
        if coldkeys.is_empty() {
            return Vec::new();
        }

        let mut subnet_stake_info: Vec<(T::AccountId, Vec<SubnetStakeInfo<T>>)> = Vec::new();
        for coldkey in coldkeys {
            let mut stake_info_for_coldkey: Vec<SubnetStakeInfo<T>> = Vec::new();

            // Iterate over SubStake storage
            for ((hotkey, coldkey_iter, netuid), stake) in <SubStake<T>>::iter() {
                if coldkey == coldkey_iter {
                    // Construct SubnetStakeInfo for each matching entry
                    stake_info_for_coldkey.push(SubnetStakeInfo {
                        hotkey,
                        netuid,
                        stake: Compact(stake), // Assuming stake is of type u64
                    });
                }
            }

            if !stake_info_for_coldkey.is_empty() {
                subnet_stake_info.push((coldkey, stake_info_for_coldkey));
            }
        }

        subnet_stake_info
    }

    pub fn get_stake_info_for_coldkey_subnet(
        coldkey_account_vec: Vec<u8>,
    ) -> Vec<SubnetStakeInfo<T>> {
        if coldkey_account_vec.len() != 32 {
            return Vec::new(); // Invalid coldkey
        }

        let coldkey: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).unwrap();
        let subnet_stake_info = Self::_get_stake_info_for_coldkeys_subnet(vec![coldkey]);

        if subnet_stake_info.len() == 0 {
            return Vec::new(); // Invalid coldkey
        } else {
            // TODO: Should remove panic here
            return subnet_stake_info.get(0).unwrap().1.clone();
        }
    }

    pub fn get_stake_info_for_coldkeys_subnet(
        coldkey_account_vecs: Vec<Vec<u8>>,
    ) -> Vec<(T::AccountId, Vec<SubnetStakeInfo<T>>)> {
        let mut coldkeys: Vec<T::AccountId> = Vec::new();
        for coldkey_account_vec in coldkey_account_vecs {
            if coldkey_account_vec.len() != 32 {
                continue; // Invalid coldkey
            }
            let coldkey: AccountIdOf<T> =
                T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()).unwrap();
            coldkeys.push(coldkey);
        }

        if coldkeys.len() == 0 {
            return Vec::new(); // Invalid coldkey
        }

        let subnet_stake_info = Self::_get_stake_info_for_coldkeys_subnet(coldkeys);

        return subnet_stake_info;
    }
}
