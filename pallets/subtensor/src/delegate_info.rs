use super::*;
use alloc::collections::BTreeMap;
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};
use sp_core::{hexdisplay::AsBytesRef, Get};
use substrate_fixed::types::U64F64;
use sp_std::vec::Vec;

extern crate alloc;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DelegateInfo<T: Config> {
    delegate_ss58: T::AccountId,
    take: Vec<(Compact<u16>, Compact<u16>)>,
    nominators: Vec<(T::AccountId, Compact<u64>)>, // map of nominator_ss58 to stake amount
    owner_ss58: T::AccountId,
    registrations: Vec<Compact<u16>>, // Vec of netuid this delegate is registered on
    validator_permits: Vec<Compact<u16>>, // Vec of netuid this delegate has validator permit on
    return_per_1000: Compact<u64>, // Delegators current daily return per 1000 TAO staked minus take fee
    total_daily_return: Compact<u64>, // Delegators current daily return
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DelegateInfoLight<T: Config> {
    delegate_ss58: T::AccountId,
    owner_ss58: T::AccountId,
    take: u16, // take as number if it is default for all subnets or u16::MAX if it is custom
    owner_stake: Compact<u64>,
    total_stake: Compact<u64>,
    validator_permits: Vec<Compact<u16>>, // Vec of netuid this delegate has validator permit on
    return_per_1000: Compact<u64>, // Delegators current daily return per 1000 TAO staked minus take fee
    total_daily_return: Compact<u64>, // Delegators current daily return
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubStakeElement<T: Config> {
    hotkey: T::AccountId,
    coldkey: T::AccountId,
    netuid: Compact<u16>,
    stake: Compact<u64>,
}

impl<T: Config> Pallet<T> {
    /// Returns all `SubStakeElement` instances associated with a given hotkey.
    ///
    /// This function takes a hotkey's bytes representation, decodes it to the `AccountId` type,
    /// and then iterates through all the coldkeys that have staked on this hotkey across all
    /// subnetworks (netuids). For each coldkey, it retrieves the stake amount and constructs
    /// a `SubStakeElement` instance which is then added to the response vector.
    ///
    /// # Arguments
    ///
    /// * `hotkey_bytes` - A byte vector representing the hotkey for which to retrieve the `SubStakeElement` instances.
    ///
    /// # Returns
    ///
    /// A vector of `SubStakeElement<T>` instances representing all the stakes associated with the given hotkey.
    ///
    /// # Panics
    ///
    /// This function will panic if the hotkey cannot be decoded into an `AccountId`.
    ///
    pub fn get_substake_for_hotkey(hotkey_bytes: Vec<u8>) -> Vec<SubStakeElement<T>> {
        if hotkey_bytes.len() != 32 {
            return Vec::new();
        }
        let hotkey: AccountIdOf<T> =
            T::AccountId::decode(&mut hotkey_bytes.as_bytes_ref()).unwrap();
        let coldkey = Self::get_owning_coldkey_for_hotkey(&hotkey);

        SubStake::<T>::iter_prefix((&coldkey, &hotkey))
            .map(|(netuid, stake)| {
                SubStakeElement {
                    hotkey: hotkey.clone(),
                    coldkey: coldkey.clone(),
                    netuid: Compact(netuid),
                    stake: Compact(stake),
                }
            }).collect()
    }

    /// Returns all `SubStakeElement` instances associated with a given coldkey.
    ///
    /// This function takes a coldkey's bytes representation, decodes it to the `AccountId` type,
    /// and then iterates through all the hotkeys that have staked on this coldkey across all
    /// subnetworks (netuids). For each hotkey, it retrieves the stake amount and constructs
    /// a `SubStakeElement` instance which is then added to the response vector.
    ///
    /// # Arguments
    ///
    /// * `coldkey_bytes` - A byte vector representing the coldkey for which to retrieve the `SubStakeElement` instances.
    ///
    /// # Returns
    ///
    /// A vector of `SubStakeElement<T>` instances representing all the stakes associated with the given coldkey.
    ///
    /// # Panics
    ///
    /// This function will panic if the coldkey cannot be decoded into an `AccountId`.
    ///
    pub fn get_substake_for_coldkey(coldkey_bytes: Vec<u8>) -> Vec<SubStakeElement<T>> {
        if coldkey_bytes.len() != 32 {
            return Vec::new();
        }
        let coldkey: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_bytes.as_slice()).expect("Coldkey decoding failed");
        SubStake::<T>::iter_prefix((&coldkey,)).map(|((hotkey, nid), stake)|{
            SubStakeElement {
                hotkey,
                coldkey: coldkey.clone(),
                netuid: Compact(nid),
                stake: Compact(stake),
            }
        }).collect()
    }

    /// Returns all `SubStakeElement` instances associated with a given netuid.
    ///
    /// This function iterates through all the stakes in the `SubStake` storage, filtering
    /// those that match the provided netuid. For each matching stake, it constructs a
    /// `SubStakeElement` instance and adds it to the response vector.
    ///
    /// # Arguments
    ///
    /// * `netuid` - A 16-bit unsigned integer representing the netuid for which to retrieve the `SubStakeElement` instances.
    ///
    /// # Returns
    ///
    /// A vector of `SubStakeElement<T>` instances representing all the stakes associated with the given netuid.
    ///
    pub fn get_substake_for_netuid(netuid: u16) -> Vec<SubStakeElement<T>> {
        SubStake::<T>::iter().filter(|((_, _, nid), stake)| {
            *nid == netuid && *stake != 0
        }).map(|((coldkey, hotkey, nid), stake)|{
            SubStakeElement {
                hotkey,
                coldkey,
                netuid: Compact(nid),
                stake: Compact(stake),
            }
        }).collect()
    }

    /// Returns Global Dynamic TAO balance for a hotkey.
    ///
    /// This function retrieves GDT of a hotkey.
    ///
    /// # Arguments
    ///
    /// * `hotkey_bytes` - A byte vector representing the hotkey for which to retrieve the `SubStakeElement` instances.
    ///
    /// # Returns
    ///
    /// u64 representing the GDT of the hotkey
    ///
    pub fn get_total_stake_for_hotkey(hotkey_bytes: Vec<u8>) -> u64 {
        let account_id: AccountIdOf<T> =
            T::AccountId::decode(&mut hotkey_bytes.as_slice()).expect("Hotkey decoding failed");
        Self::get_hotkey_global_dynamic_tao(&account_id)
    }

    /// Returns Global Dynamic TAO balance for a coldkey.
    ///
    /// This function iterates through all hotkeys associated with the coldkey and adds
    /// GDT for each hotkey to the result
    ///
    /// # Arguments
    ///
    /// * `coldkey_bytes` - A byte vector representing the hotkey for which to retrieve the `SubStakeElement` instances.
    ///
    /// # Returns
    ///
    /// u64 representing the GDT of the coldkey
    ///
    pub fn get_total_stake_for_coldkey(coldkey_bytes: Vec<u8>) -> u64 {
        let account_id: AccountIdOf<T> =
            T::AccountId::decode(&mut coldkey_bytes.as_slice()).expect("Coldkey decoding failed");

        // O(1) complexity on number of coldkeys in storage
        SubStake::<T>::iter_prefix((account_id,)).map(|((_hotkey, netuid), stake)| {
            Self::estimate_dynamic_unstake(netuid, stake)
        }).sum()
    }

    fn get_delegate_by_existing_account(delegate: &AccountIdOf<T>) -> DelegateInfo<T> {
        let all_netuids: Vec<u16> = Self::get_all_subnet_netuids();
        let nominators = 
        Staker::<T>::iter_key_prefix(delegate).map(|nominator| {
            let mut total_staked_to_delegate_i: u64 = 0;
            for netuid_i in all_netuids.iter() {
                total_staked_to_delegate_i +=
                    Self::get_subnet_stake_for_coldkey_and_hotkey(&nominator, delegate, *netuid_i);
            }
            (nominator, total_staked_to_delegate_i)
        }).filter(|(_nominator, total_staked_to_delegate)| *total_staked_to_delegate != 0)
        .map(|(nominator, total_staked_to_delegate_i)| (nominator, Compact(total_staked_to_delegate_i)))
        .collect();
        let registrations = Self::get_registered_networks_for_hotkey(delegate);
        let mut validator_permits = Vec::<Compact<u16>>::new();
        let mut emissions_per_day: U64F64 = U64F64::from_num(0);

        for netuid in registrations.iter() {
            let _uid = Self::get_uid_for_net_and_hotkey(*netuid, delegate);
            if _uid.is_err() {
                continue; // this should never happen
            } else {
                let uid = _uid.expect("Delegate's UID should be ok");
                let validator_permit = Self::get_validator_permit_for_uid(*netuid, uid);
                if validator_permit {
                    validator_permits.push((*netuid).into());
                }

                let emission: U64F64 = Self::get_emission_for_uid(*netuid, uid).into();
                let tempo: U64F64 = Self::get_tempo(*netuid).into();
                let epochs_per_day: U64F64 = U64F64::from_num(7200) / tempo;
                emissions_per_day += emission * epochs_per_day;
            }
        }

        let owner = Self::get_owning_coldkey_for_hotkey(delegate);

        // Create a vector of tuples (netuid, take). If a take is not set in DelegatesTake, use default value
        let take = NetworksAdded::<T>::iter()
            .filter(|(_, added)| *added)
            .map(|(netuid, _)| {
                (
                    Compact(netuid),
                    Compact(
                        if let Ok(take) = DelegatesTake::<T>::try_get(delegate, netuid) {
                            take
                        } else {
                            <DefaultDefaultTake<T>>::get()
                        },
                    ),
                )
            })
            .collect();

        let total_stake: U64F64 = Self::get_hotkey_global_dynamic_tao(delegate).into();

        let mut return_per_1000: U64F64 = U64F64::from_num(0);

        if total_stake > U64F64::from_num(0) {
            return_per_1000 = (emissions_per_day * U64F64::from_num(0.82))
                / (total_stake / U64F64::from_num(1000));
        }

        DelegateInfo {
            delegate_ss58: delegate.clone(),
            take,
            nominators,
            owner_ss58: owner.clone(),
            registrations: registrations.iter().map(|x| x.into()).collect(),
            validator_permits,
            return_per_1000: U64F64::to_num::<u64>(return_per_1000).into(),
            total_daily_return: U64F64::to_num::<u64>(emissions_per_day).into(),
        }
    }

    fn get_delegate_by_existing_account_light_by_netuid(delegate: &AccountIdOf<T>, netuid: u16) -> DelegateInfoLight<T> {
        let owner = Self::get_owning_coldkey_for_hotkey(delegate);
        let take = if DelegatesTake::<T>::iter_prefix(delegate).next().is_some() { u16::MAX } else { <DefaultDefaultTake<T>>::get()};
        let owner_stake: u64 = Self::get_subnet_stake_for_coldkey_and_hotkey(&owner, delegate, netuid);
        let total_stake: u64 = Self::get_total_stake_for_hotkey_and_subnet(delegate, netuid);
        let validator_permits = Vec::<Compact<u16>>::new();
        let return_per_1000: U64F64 = U64F64::from_num(0);
        let total_daily_return: U64F64 = U64F64::from_num(0);
        DelegateInfoLight {
            delegate_ss58: delegate.clone(),
            owner_ss58: owner,
            take,
            owner_stake: owner_stake.into(),
            total_stake: total_stake.into(),
            validator_permits,
            return_per_1000: U64F64::to_num::<u64>(return_per_1000).into(),
            total_daily_return: U64F64::to_num::<u64>(total_daily_return).into()
        }
    }

    fn get_delegate_by_existing_account_light(delegate: &AccountIdOf<T>) -> DelegateInfoLight<T> {
        let mut validator_permits = Vec::<Compact<u16>>::new();
        let registrations = Self::get_registered_networks_for_hotkey(delegate);

        let mut emissions_per_day: U64F64 = U64F64::from_num(0);
        for netuid in registrations.iter() {
            let _uid = Self::get_uid_for_net_and_hotkey(*netuid, delegate);
            if _uid.is_err() {
                continue; // this should never happen
            } else {
                let uid = _uid.expect("Delegate's UID should be ok");
                let validator_permit = Self::get_validator_permit_for_uid(*netuid, uid);
                if validator_permit {
                    validator_permits.push((*netuid).into());
                }

                let emission: U64F64 = Self::get_emission_for_uid(*netuid, uid).into();
                let tempo: U64F64 = Self::get_tempo(*netuid).into();
                let epochs_per_day: U64F64 = U64F64::from_num(7200) / tempo;
                emissions_per_day += emission * epochs_per_day;
            }
        }

        let owner = Self::get_owning_coldkey_for_hotkey(delegate);

        // Create a vector of tuples (netuid, take). If a take is not set in DelegatesTake, use default value
        let take = if DelegatesTake::<T>::iter_prefix(delegate).next().is_some() {
            // None
            u16::MAX
        } else {
            // Some(<DefaultDefaultTake<T>>::get())
            <DefaultDefaultTake<T>>::get()
        };

        let total_stake: U64F64 = Self::get_hotkey_global_dynamic_tao(delegate).into();
        let owner_stake = Self::get_nominator_global_dynamic_tao(&owner, delegate);

        let mut return_per_1000: U64F64 = U64F64::from_num(0);

        if total_stake > U64F64::from_num(0) {
            return_per_1000 = (emissions_per_day * U64F64::from_num(0.82))
                / (total_stake / U64F64::from_num(1000));
        }

        DelegateInfoLight {
            delegate_ss58: delegate.clone(),
            owner_ss58: owner,
            take,
            owner_stake: owner_stake.into(),
            total_stake: total_stake.to_num::<u64>().into(),
            validator_permits,
            return_per_1000: U64F64::to_num::<u64>(return_per_1000).into(),
            total_daily_return: U64F64::to_num::<u64>(emissions_per_day).into(),
        }
    }
    
    pub fn get_delegate(delegate_account_vec: Vec<u8>) -> Option<DelegateInfo<T>> {
        if delegate_account_vec.len() != 32 {
            return None;
        }

        let delegate: AccountIdOf<T> =
            T::AccountId::decode(&mut delegate_account_vec.as_bytes_ref()).ok()?;
        // Check delegate exists
        if DelegatesTake::<T>::iter_prefix(&delegate).next().is_none() {
            return None;
        }

        let delegate_info = Self::get_delegate_by_existing_account(&delegate);
        Some(delegate_info)
    }

    /// get all delegates info from storage
    ///
    pub fn get_delegates() -> Vec<DelegateInfo<T>> {
        // Get all hotkeys registered on the netuid
        Self::get_all_subnet_netuids().iter()
            .flat_map(|netuid| {
                Uids::<T>::iter_prefix(netuid)
                .map(|(delegate, _)| Self::get_delegate_by_existing_account(&delegate))
            }).collect()
    }

    /// get all delegates' total stake from storage
    ///
    pub fn get_delegates_light() -> Vec<DelegateInfoLight<T>> {
        // Get all hotkeys registered on all subnets
        Self::get_all_subnet_netuids().iter()
            .flat_map(|netuid| {
                Uids::<T>::iter_prefix(netuid)
                    .map(|(delegate, _)| Self::get_delegate_by_existing_account_light(&delegate))
            }).collect()
    }

    /// get all delegates for a subnet
    ///
    /// * `netuid` - Subnet ID to find all registered delegates
    /// 
    pub fn get_delegates_by_netuid_light(netuid: u16) -> Vec<DelegateInfoLight<T>> {
        // Get all hotkeys registered on the netuid
        Uids::<T>::iter_prefix(netuid)
            .map(|(delegate, _)| Self::get_delegate_by_existing_account_light_by_netuid(&delegate, netuid))
            .collect()
    }
    
    /// get all delegates' light info from storage
    ///
    /// * `netuid` - Subnet ID to find all delegates total stakes for
    /// 
    pub fn get_all_delegates_total_stake() -> Vec<(T::AccountId, Compact<u64>)> {
        // Get all hotkeys registered on all subnets
        Self::get_all_subnet_netuids().iter()
            .flat_map(|netuid| {
                Uids::<T>::iter_prefix(netuid).map(|(delegate, _)| 
                    (delegate.clone(), Self::get_hotkey_global_dynamic_tao(&delegate).into())
                )
            }).collect()
    }

    /// get all delegate info and staked token amount for a given delegatee account
    ///
    /// * `coldkey_account_vec` - Coldkey account to find all delegations made by it
    /// 
    pub fn get_delegated(coldkey_account_vec: Vec<u8>) -> Vec<(DelegateInfo<T>, Compact<u64>)> {
        let Ok(coldkey) = T::AccountId::decode(&mut coldkey_account_vec.as_bytes_ref()) else {
            return Vec::new(); // No delegates for invalid account
        };

        let mut hotkey_stakes: BTreeMap<<T as frame_system::Config>::AccountId, u64> = BTreeMap::new();
        SubStake::<T>::iter_prefix((&coldkey,)).for_each(|((hotkey, _), stake)| {
            hotkey_stakes.entry(hotkey).and_modify(|s| *s += stake).or_insert(stake);
        });

        hotkey_stakes.iter()
            .filter(|(_, &total_staked_to_delegate)| total_staked_to_delegate != 0)
            .map(|(delegate_id, &total_delegate_stake)| {
                (
                    Self::get_delegate_by_existing_account(delegate_id),
                    Compact(total_delegate_stake),
                )
            })
            .collect()
    }
}
