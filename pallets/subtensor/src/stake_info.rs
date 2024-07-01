use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use crate::types::TensorBytes;
use codec::Compact;
use sp_core::hexdisplay::AsBytesRef;
use sp_std::vec;
use sp_std::vec::Vec;

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
    // Made public so we can access it during our tests.
    pub stake: Compact<u64>,
}

impl<T: Config> Pallet<T> {
    fn _get_stake_info_for_coldkeys(
        coldkeys: Vec<T::AccountId>,
    ) -> Vec<(T::AccountId, Vec<StakeInfo<T>>)> {
        if coldkeys.is_empty() {
            return Vec::new(); // No coldkeys to check
        }

        let mut stake_info: Vec<(T::AccountId, Vec<StakeInfo<T>>)> = Vec::new();
        for coldkey_ in coldkeys {
            let mut stake_info_for_coldkey: Vec<StakeInfo<T>> = Vec::new();

            for ((coldkey, hotkey, _netuid), stake) in <SubStake<T>>::iter() {
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

        stake_info
    }

    /// This function is used to retrieve the stake associated with a vector of coldkeys .
    /// It iterates over the `Stake` storage map and returns the stake information for the UI.
    ///
    /// # Arguments:
    /// * `coldkey_account_bytes`: Vec<TensorBytes> - The TensorBytes representing the coldkey account.
    pub fn get_stake_info_for_coldkeys(
        coldkey_account_bytes_vec: Vec<TensorBytes>,
    ) -> Vec<(T::AccountId, Vec<StakeInfo<T>>)> {
        let mut coldkeys: Vec<T::AccountId> = Vec::new();
        for coldkey_account_bytes in coldkey_account_bytes_vec {
            if coldkey_account_bytes.as_ref().len() != 32 {
                continue; // Invalid coldkey
            }
            let coldkey: AccountIdOf<T> =
                T::AccountId::decode(&mut coldkey_account_bytes.as_bytes_ref()).unwrap();
            coldkeys.push(coldkey);
        }

        if coldkeys.is_empty() {
            return Vec::new(); // Invalid coldkey
        }

        Self::_get_stake_info_for_coldkeys(coldkeys)
    }

    /// This function is used to retrieve the all the stake associated with a coldkey
    /// It iterates over the `Stake` storage map and returns the stake information for the UI.
    ///
    /// # Arguments:
    /// * `coldkey_account_bytes`: TensorBytes - The TensorBytes representing the coldkey account.
    pub fn get_stake_info_for_coldkey(coldkey_account_bytes: TensorBytes) -> Vec<StakeInfo<T>> {
        if coldkey_account_bytes.as_ref().len() != 32 {
            return Vec::new(); // Invalid coldkey
        }

        let coldkey: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_account_bytes.as_bytes_ref()).unwrap();
        let stake_info = Self::_get_stake_info_for_coldkeys(vec![coldkey]);

        if stake_info.is_empty() {
            // Invalid coldkey
            Vec::new()
        } else {
            stake_info.first().unwrap().1.clone()
        }
    }

    /// This function is used to retrieve the stake associated with a coldkey on a specific subnet.
    /// It iterates over the `SubStake` storage map and returns the stake information for the UI.
    ///
    /// # Arguments:
    /// * `coldkey_account_bytes`: TensorBytes - The TensorBytes representing the coldkey account.
    /// * `netuid`: u16 - The unique identifier of the network.
    pub fn get_subnet_stake_info_for_coldkey(
        coldkey_account_bytes: TensorBytes,
        netuid: u16,
    ) -> Vec<SubnetStakeInfo<T>> {
        if coldkey_account_bytes.as_ref().len() != 32 {
            return Vec::new(); // Invalid coldkey
        }

        let coldkey: T::AccountId = T::AccountId::decode(&mut coldkey_account_bytes.as_bytes_ref())
            .expect("Failed to decode AccountId");

        // Filter `SubStake` storage map for entries matching the coldkey and netuid.
        let mut subnet_stake_info: Vec<SubnetStakeInfo<T>> = Vec::new();
        for ((coldkey_iter, hotkey, subnet), stake) in SubStake::<T>::iter() {
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
    /// * 'coldkey_account_byte_vecs': Vec<TensorBytes>:
    ///     - The vector of coldkey account TensorBytes.
    /// * 'netuid': u16:
    ///     - The network uid.
    ///
    /// # Returns:
    /// A vector of tuples, each containing a `T::AccountId` (coldkey) and a vector of `SubnetStakeInfo<T>`.
    pub fn get_subnet_stake_info_for_coldkeys(
        coldkey_account_byte_vecs: Vec<TensorBytes>,
        netuid: u16,
    ) -> Vec<(T::AccountId, Vec<SubnetStakeInfo<T>>)> {
        let mut results: Vec<(T::AccountId, Vec<SubnetStakeInfo<T>>)> = Vec::new();

        for coldkey_account_vec in coldkey_account_byte_vecs {
            if coldkey_account_vec.as_ref().len() != 32 {
                continue; // Skip invalid coldkey
            }

            let coldkey: T::AccountId =
                T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref())
                    .expect("Failed to decode AccountId");

            // Filter `SubStake` storage map for entries matching the coldkey and netuid.
            let mut subnet_stake_info: Vec<SubnetStakeInfo<T>> = Vec::new();
            for ((coldkey_iter, hotkey, subnet), stake) in SubStake::<T>::iter() {
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
        // Return the total stake wrapped in Compact.
        Compact(TotalSubnetTAO::<T>::get(netuid))
    }

    /// This function is used to get all the stake information for a given coldkey across all subnets.
    /// It iterates over the `SubStake` storage map and returns a vector of all stakes associated with the coldkey.
    ///
    /// # Args:
    /// * 'coldkey_account_bytes': TensorBytes:
    ///     - TensorBytes representation of the coldkey.
    ///
    /// # Returns:
    /// A vector of tuples, each containing a hotkey (`T::AccountId`), netuid (`u16`), and stake amount (`Compact<u64>`).
    pub fn get_all_stake_info_for_coldkey(
        coldkey_account_bytes: TensorBytes,
    ) -> Vec<(T::AccountId, u16, Compact<u64>)> {
        if coldkey_account_bytes.as_ref().len() != 32 {
            return Vec::new(); // Invalid coldkey, return empty vector
        }

        let coldkey: T::AccountId = T::AccountId::decode(&mut coldkey_account_bytes.as_bytes_ref())
            .expect("Failed to decode AccountId");

        // Initialize a vector to hold all stake information.
        let mut all_stake_info: Vec<(T::AccountId, u16, Compact<u64>)> = Vec::new();

        // Iterate over `SubStake` storage map for entries matching the coldkey and collect their information.
        // If stake != 0
        for ((coldkey_iter, hotkey, netuid), stake) in SubStake::<T>::iter() {
            // if coldkey == coldkey_iter {
            //     all_stake_info.push((hotkey, netuid, Compact(stake)));
            // }
            if coldkey == coldkey_iter && stake != 0 {
                all_stake_info.push((hotkey, netuid, Compact(stake)));
            }
        }

        // Return the vector of all stake information.
        all_stake_info
    }

    /// This function is used to retrieve all the subnet stake info associated with a coldkey across all subnets.
    /// It iterates over the `SubStake` storage map and returns the stake information for the UI.
    ///
    /// # Arguments:
    /// * `coldkey_account_bytes`: TensorBytes - The TensorBytes representing the coldkey account.
    pub fn get_all_subnet_stake_info_for_coldkey(
        coldkey_account_bytes: TensorBytes,
    ) -> Vec<SubnetStakeInfo<T>> {
        if coldkey_account_bytes.as_ref().len() != 32 {
            return Vec::new(); // Invalid coldkey
        }

        let coldkey: T::AccountId = T::AccountId::decode(&mut coldkey_account_bytes.as_bytes_ref())
            .expect("Failed to decode AccountId");

        // Filter `SubStake` storage map for entries matching the coldkey across all subnets.
        let mut all_subnet_stake_info: Vec<SubnetStakeInfo<T>> = Vec::new();
        for ((coldkey_iter, hotkey, netuid), stake) in SubStake::<T>::iter() {
            if coldkey == coldkey_iter {
                all_subnet_stake_info.push(SubnetStakeInfo {
                    hotkey,
                    netuid,
                    stake: Compact(stake),
                });
            }
        }

        all_subnet_stake_info
    }

    /// This function returns the total stake for each subnet.
    /// It iterates over the `SubStake` storage map and calculates the sum of stakes for each subnet.
    ///
    /// # Returns:
    /// A vector of tuples, each containing the subnet UID (`u16`) and the total stake (`Compact<u64>`) for that subnet.
    pub fn get_total_stake_for_each_subnet() -> Vec<(u16, Compact<u64>)> {
        // Initialize a vector to store the total stake for each subnet.
        let mut subnet_stakes: Vec<(u16, u64)> = Vec::new();

        // Iterate over the `SubStake` storage map and calculate the total stake for each subnet.
        for ((_, _, subnet), stake) in SubStake::<T>::iter() {
            // Check if the subnet already exists in the vector.
            if let Some(index) = subnet_stakes.iter().position(|(s, _)| *s == subnet) {
                // If the subnet exists, update its total stake.
                subnet_stakes[index].1 += stake;
            } else {
                // If the subnet doesn't exist, add a new entry to the vector.
                subnet_stakes.push((subnet, stake));
            }
        }

        // Convert the vector of tuples to the desired output format.
        let total_stakes: Vec<(u16, Compact<u64>)> = subnet_stakes
            .into_iter()
            .map(|(subnet, total_stake)| (subnet, Compact(total_stake)))
            .collect();

        total_stakes
    }
}
