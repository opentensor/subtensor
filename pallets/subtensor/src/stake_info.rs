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
    pub stake: Compact<u64>,
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

            for ((hotkey, coldkey, _netuid), stake) in <SubStake<T>>::iter() {
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

    /// This function is used to retrieve the stake associated with a coldkey on a specific subnet.
    /// It iterates over the `SubStake` storage map and returns the stake information for the UI.
    ///
    /// # Arguments:
    /// * `coldkey_account_vec`: Vec<u8> - The vector representing the coldkey account.
    /// * `netuid`: u16 - The unique identifier of the network.
    pub fn get_subnet_stake_info_for_coldkey(
        coldkey_account_vec: Vec<u8>,
        netuid: u16,
    ) -> Vec<SubnetStakeInfo<T>> {
        if coldkey_account_vec.len() != 32 {
            return Vec::new(); // Invalid coldkey
        }

        let coldkey: T::AccountId = T::AccountId::decode(&mut &coldkey_account_vec[..])
            .expect("Failed to decode AccountId");

        // Filter `SubStake` storage map for entries matching the coldkey and netuid.
        let mut subnet_stake_info: Vec<SubnetStakeInfo<T>> = Vec::new();
        for ((hotkey, coldkey_iter, subnet), stake) in SubStake::<T>::iter() {
            if coldkey == coldkey_iter && netuid == subnet {
                subnet_stake_info.push(SubnetStakeInfo {
                    hotkey,
                    netuid,
                    stake: Compact(stake),
                });
            }
        }

        subnet_stake_info
    }

    /// This function is used to get the stake that a vector of coldkeys holds on the subnet.
    /// It iterates over the `SubStake` storage map and returns the stake mapped to the UI.
    ///
    /// # Args:
    /// * 'coldkey_account_vecs': Vec<Vec<u8>>:
    ///     - The vector of coldkey account vectors.
    /// * 'netuid': u16:
    ///     - The network uid.
    ///
    /// # Returns:
    /// A vector of tuples, each containing a `T::AccountId` (coldkey) and a vector of `SubnetStakeInfo<T>`.
    pub fn get_subnet_stake_info_for_coldkeys(
        coldkey_account_vecs: Vec<Vec<u8>>,
        netuid: u16,
    ) -> Vec<(T::AccountId, Vec<SubnetStakeInfo<T>>)> {
        let mut results: Vec<(T::AccountId, Vec<SubnetStakeInfo<T>>)> = Vec::new();

        for coldkey_account_vec in coldkey_account_vecs {
            if coldkey_account_vec.len() != 32 {
                continue; // Skip invalid coldkey
            }

            let coldkey: T::AccountId = T::AccountId::decode(&mut &coldkey_account_vec[..])
                .expect("Failed to decode AccountId");

            // Filter `SubStake` storage map for entries matching the coldkey and netuid.
            let mut subnet_stake_info: Vec<SubnetStakeInfo<T>> = Vec::new();
            for ((hotkey, coldkey_iter, subnet), stake) in SubStake::<T>::iter() {
                if coldkey == coldkey_iter && netuid == subnet {
                    subnet_stake_info.push(SubnetStakeInfo {
                        hotkey,
                        netuid,
                        stake: Compact(stake), // Wrap the stake in Compact
                    });
                }
            }

            if !subnet_stake_info.is_empty() {
                results.push((coldkey, subnet_stake_info));
            }
        }

        results
    }

    /// This function returns the total amount of stake on a subnet.
    /// It returns a number, which is the sum of the stakes on the subnet identified by the subnet's UID.
    ///
    /// # Args:
    /// * 'netuid': u16:
    ///     - The network uid.
    ///
    /// # Returns:
    /// The total stake as a `Compact<u64>`.
    pub fn get_total_subnet_stake(netuid: u16) -> Compact<u64> {
        // Initialize a variable to hold the sum of stakes.
        let mut total_stake: u64 = 0;

        // Filter `SubStake` storage map for entries matching the netuid and sum their stakes.
        for ((_, _, subnet), stake) in SubStake::<T>::iter() {
            if netuid == subnet {
                total_stake += stake; // Assuming stake is of type u64
            }
        }

        // Return the total stake wrapped in Compact.
        Compact(total_stake)
    }

    /// This function is used to get all the stake information for a given coldkey across all subnets.
    /// It iterates over the `SubStake` storage map and returns a vector of all stakes associated with the coldkey.
    ///
    /// # Args:
    /// * 'coldkey_account_vec': Vec<u8>:
    ///     - The coldkey account vector.
    ///
    /// # Returns:
    /// A vector of tuples, each containing a hotkey (`T::AccountId`), netuid (`u16`), and stake amount (`Compact<u64>`).
    pub fn get_all_stake_info_for_coldkey(
        coldkey_account_vec: Vec<u8>,
    ) -> Vec<(T::AccountId, u16, Compact<u64>)> {
        if coldkey_account_vec.len() != 32 {
            return Vec::new(); // Invalid coldkey, return empty vector
        }

        let coldkey: T::AccountId = T::AccountId::decode(&mut &coldkey_account_vec[..])
            .expect("Failed to decode AccountId");

        // Initialize a vector to hold all stake information.
        let mut all_stake_info: Vec<(T::AccountId, u16, Compact<u64>)> = Vec::new();

        // Iterate over `SubStake` storage map for entries matching the coldkey and collect their information.
        for ((hotkey, coldkey_iter, netuid), stake) in SubStake::<T>::iter() {
            if coldkey == coldkey_iter {
                all_stake_info.push((hotkey, netuid, Compact(stake))); // Assuming stake is of type u64
            }
        }

        // Return the vector of all stake information.
        all_stake_info
    }
}
