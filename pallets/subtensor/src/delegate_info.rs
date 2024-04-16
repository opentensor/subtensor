use alloc::collections::BTreeMap;
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};
use sp_core::hexdisplay::AsBytesRef;
use substrate_fixed::types::U64F64;
use super::*;

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

impl<T: Config> Pallet<T> {
    fn get_delegate_by_existing_account(delegate: AccountIdOf<T>) -> DelegateInfo<T> {

        let mut nominators = Vec::<(T::AccountId, Compact<u64>)>::new();
        for (nominator, _) in Stake::<T>::iter_prefix( delegate.clone() ) {
            let mut total_staked_to_delegate_i: u64 = 0;
            for netuid_i in 0..(TotalNetworks::<T>::get()+1) {
                total_staked_to_delegate_i += Self::get_subnet_stake_for_coldkey_and_hotkey( &nominator, &delegate, netuid_i );
            }
            if total_staked_to_delegate_i == 0 { continue; }
            nominators.push((nominator.clone(), total_staked_to_delegate_i.into()));
        }
        let registrations = Self::get_registered_networks_for_hotkey(&delegate.clone());
        let mut validator_permits = Vec::<Compact<u16>>::new();
        let mut emissions_per_day: U64F64 = U64F64::from_num(0);

        for netuid in registrations.iter() {
            let _uid = Self::get_uid_for_net_and_hotkey(*netuid, &delegate.clone());
            if !_uid.is_ok() {
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

        let owner = Self::get_owning_coldkey_for_hotkey(&delegate.clone());
        let take = <Delegates<T>>::iter_prefix(&delegate)
            .map(|(netuid, take)| (Compact(netuid), Compact(take))).collect();

        let total_stake: U64F64 = Self::get_total_stake_for_hotkey(&delegate.clone()).into();

        let mut return_per_1000: U64F64 = U64F64::from_num(0);

        if total_stake > U64F64::from_num(0) {
            return_per_1000 = (emissions_per_day * U64F64::from_num(0.82))
                / (total_stake / U64F64::from_num(1000));
        }

        return DelegateInfo {
            delegate_ss58: delegate.clone(),
            take,
            nominators,
            owner_ss58: owner.clone(),
            registrations: registrations.iter().map(|x| x.into()).collect(),
            validator_permits,
            return_per_1000: U64F64::to_num::<u64>(return_per_1000).into(),
            total_daily_return: U64F64::to_num::<u64>(emissions_per_day).into(),
        };
    }

    pub fn get_delegate(delegate_account_vec: Vec<u8>) -> Option<DelegateInfo<T>> {
        if delegate_account_vec.len() != 32 {
            return None;
        }

        let delegate: AccountIdOf<T> =
            T::AccountId::decode(&mut delegate_account_vec.as_bytes_ref()).unwrap();
        // Check delegate exists
        if <Delegates<T>>::iter_prefix(&delegate).next().is_none() {
            return None;
        }

        let delegate_info = Self::get_delegate_by_existing_account(delegate.clone());
        return Some(delegate_info);
    }

    pub fn get_delegates() -> Vec<DelegateInfo<T>> {
        let mut unique_delegates = BTreeMap::new();
        <Delegates<T>>::iter()
            .filter(|(delegate_id, _netuid, _take)| {
                let delegate_as_vec = delegate_id.encode();
                let handled = unique_delegates.contains_key(&delegate_as_vec);
                unique_delegates.insert(delegate_as_vec, ());
                !handled
            })
            .map(|(delegate_id, _, _)| {
                Self::get_delegate_by_existing_account(delegate_id)
            })
            .collect()
    }

    pub fn get_delegated(delegatee_account_vec: Vec<u8>) -> Vec<(DelegateInfo<T>, Compact<u64>)> {
        if delegatee_account_vec.len() != 32 {
            return Vec::new(); // No delegates for invalid account
        }

        let delegatee: AccountIdOf<T> =
            T::AccountId::decode(&mut delegatee_account_vec.as_bytes_ref()).unwrap();

        let mut unique_delegates = BTreeMap::new();
        <Delegates<T>>::iter()
            .filter(|(delegate_id, _netuid, _take)| {
                let delegate_as_vec = delegate_id.encode();
                let handled = unique_delegates.contains_key(&delegate_as_vec);
                unique_delegates.insert(delegate_as_vec, ());
                !handled
            })
            .map(|(delegate_id, _, _)| {
                let mut total_staked_to_delegate_i: u64 = 0;
                for netuid_i in 0..=TotalNetworks::<T>::get() {
                    total_staked_to_delegate_i += Self::get_subnet_stake_for_coldkey_and_hotkey( &delegatee, &delegate_id, netuid_i );
                }
                (delegate_id, Compact(total_staked_to_delegate_i))
            })
            .filter(|(_, Compact(total_staked_to_delegate_i))| {
                *total_staked_to_delegate_i != 0
            })
            .map(|(delegate_id, total_delegate_stake)| {
                (Self::get_delegate_by_existing_account(delegate_id), total_delegate_stake)
            })
            .collect()
    }

    pub fn get_delegate_limit() -> u32 {
        DelegateLimit::<T>::get()
    }
}
