use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::storage::IterableStorageMap;
use frame_support::IterableStorageDoubleMap;
use substrate_fixed::types::U64F64;
extern crate alloc;
use codec::Compact;
use sp_core::hexdisplay::AsBytesRef;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DelegateInfo<T: Config> {
    delegate_ss58: T::AccountId,
    take: Compact<u16>,
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

        for (nominator, stake) in
            <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(
                delegate.clone(),
            )
        {
            if stake == 0 {
                continue;
            }
            // Only add nominators with stake
            nominators.push((nominator.clone(), stake.into()));
        }

        let registrations = Self::get_registered_networks_for_hotkey(&delegate.clone());
        let mut validator_permits = Vec::<Compact<u16>>::new();
        let mut emissions_per_day: U64F64 = U64F64::from_num(0);

        for netuid in registrations.iter() {
            let _uid = Self::get_uid_for_net_and_hotkey(*netuid, &delegate.clone());
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

        let owner = Self::get_owning_coldkey_for_hotkey(&delegate.clone());
        let take: Compact<u16> = <Delegates<T>>::get(delegate.clone()).into();

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
            T::AccountId::decode(&mut delegate_account_vec.as_bytes_ref()).ok()?;
        // Check delegate exists
        if !<Delegates<T>>::contains_key(delegate.clone()) {
            return None;
        }

        let delegate_info = Self::get_delegate_by_existing_account(delegate.clone());
        Some(delegate_info)
    }

    pub fn get_delegates() -> Vec<DelegateInfo<T>> {
        let mut delegates = Vec::<DelegateInfo<T>>::new();
        for delegate in <Delegates<T> as IterableStorageMap<T::AccountId, u16>>::iter_keys() {
            let delegate_info = Self::get_delegate_by_existing_account(delegate.clone());
            delegates.push(delegate_info);
        }

        delegates
    }

    pub fn get_delegated(delegatee_account_vec: Vec<u8>) -> Vec<(DelegateInfo<T>, Compact<u64>)> {
        let Ok(delegatee) = T::AccountId::decode(&mut delegatee_account_vec.as_bytes_ref()) else {
            return Vec::new(); // No delegates for invalid account
        };

        let mut delegates: Vec<(DelegateInfo<T>, Compact<u64>)> = Vec::new();
        for delegate in <Delegates<T> as IterableStorageMap<T::AccountId, u16>>::iter_keys() {
            let staked_to_this_delegatee =
                Self::get_stake_for_coldkey_and_hotkey(&delegatee.clone(), &delegate.clone());
            if staked_to_this_delegatee == 0 {
                continue; // No stake to this delegate
            }
            // Staked to this delegate, so add to list
            let delegate_info = Self::get_delegate_by_existing_account(delegate.clone());
            delegates.push((delegate_info, staked_to_this_delegatee.into()));
        }

        delegates
    }
}
