use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
use sp_std::ops::Neg;
extern crate alloc;
use codec::Compact;
use substrate_fixed::types::I96F32;

#[freeze_struct("5cfb3c84c3af3116")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct StakeInfo<AccountId: TypeInfo + Encode + Decode> {
    hotkey: AccountId,
    coldkey: AccountId,
    netuid: Compact<u16>,
    stake: Compact<u64>,
    locked: Compact<u64>,
    emission: Compact<u64>,
    tao_emission: Compact<u64>,
    drain: Compact<u64>,
    is_registered: bool,
}

impl<T: Config> Pallet<T> {
    fn _get_stake_info_for_coldkeys(
        coldkeys: Vec<T::AccountId>,
    ) -> Vec<(T::AccountId, Vec<StakeInfo<T::AccountId>>)> {
        if coldkeys.is_empty() {
            return Vec::new(); // No coldkeys to check
        }
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        let mut stake_info: Vec<(T::AccountId, Vec<StakeInfo<T::AccountId>>)> = Vec::new();
        for coldkey_i in coldkeys.clone().iter() {
            // Get all hotkeys associated with this coldkey.
            let staking_hotkeys = StakingHotkeys::<T>::get(coldkey_i.clone());
            let mut stake_info_for_coldkey: Vec<StakeInfo<T::AccountId>> = Vec::new();
            for netuid_i in netuids.clone().iter() {
                for hotkey_i in staking_hotkeys.clone().iter() {
                    let alpha: u64 = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
                        hotkey_i, coldkey_i, *netuid_i,
                    );
                    if alpha == 0 {
                        continue;
                    }
                    let emission: u64 = AlphaDividendsPerSubnet::<T>::get(*netuid_i, &hotkey_i);
                    let tao_emission: u64 = TaoDividendsPerSubnet::<T>::get(*netuid_i, &hotkey_i);
                    let is_registered: bool =
                        Self::is_hotkey_registered_on_network(*netuid_i, hotkey_i);
                    stake_info_for_coldkey.push(StakeInfo {
                        hotkey: hotkey_i.clone(),
                        coldkey: coldkey_i.clone(),
                        netuid: (*netuid_i).into(),
                        stake: alpha.into(),
                        locked: 0.into(),
                        emission: emission.into(),
                        tao_emission: tao_emission.into(),
                        drain: 0.into(),
                        is_registered,
                    });
                }
            }
            stake_info.push((coldkey_i.clone(), stake_info_for_coldkey));
        }
        stake_info
    }

    pub fn get_stake_info_for_coldkeys(
        coldkey_accounts: Vec<T::AccountId>,
    ) -> Vec<(T::AccountId, Vec<StakeInfo<T::AccountId>>)> {
        if coldkey_accounts.is_empty() {
            return Vec::new(); // Empty coldkeys
        }

        Self::_get_stake_info_for_coldkeys(coldkey_accounts)
    }

    pub fn get_stake_info_for_coldkey(
        coldkey_account: T::AccountId,
    ) -> Vec<StakeInfo<T::AccountId>> {
        let stake_info = Self::_get_stake_info_for_coldkeys(vec![coldkey_account]);

        if stake_info.is_empty() {
            Vec::new() // Invalid coldkey
        } else {
            let Some(first) = stake_info.first() else {
                return Vec::new();
            };

            first.1.clone()
        }
    }

    pub fn get_stake_info_for_hotkey_coldkey_netuid(
        hotkey_account: T::AccountId,
        coldkey_account: T::AccountId,
        netuid: u16,
    ) -> Option<StakeInfo<T::AccountId>> {
        let alpha: u64 = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account,
            &coldkey_account,
            netuid,
        );
        let emission: u64 = AlphaDividendsPerSubnet::<T>::get(netuid, &hotkey_account);
        let tao_emission: u64 = TaoDividendsPerSubnet::<T>::get(netuid, &hotkey_account);
        let is_registered: bool = Self::is_hotkey_registered_on_network(netuid, &hotkey_account);

        Some(StakeInfo {
            hotkey: hotkey_account,
            coldkey: coldkey_account,
            netuid: (netuid).into(),
            stake: alpha.into(),
            locked: 0.into(),
            emission: emission.into(),
            tao_emission: tao_emission.into(),
            drain: 0.into(),
            is_registered,
        })
    }

    pub fn get_stake_fee(
        origin_hotkey_account: Option<T::AccountId>,
        _origin_coldkey_account: T::AccountId,
        _destination_hotkey_account: Option<T::AccountId>,
        _destination_coldkey_account: T::AccountId,
        origin_netuid: Option<u16>,
        _destination_netuid: Option<u16>,
        amount: i64,
    ) -> u64 {
        if amount >= 0 || origin_netuid.is_none() {
            // Adding stake
            DefaultStakingFee::<T>::get()
        } else {
            match (origin_netuid, origin_hotkey_account) {
                (Some(origin_netuid), Some(origin_hotkey_account)) => {
                    // Calculate fee for unstake (negative amount)
                    Self::calculate_staking_fee(
                        origin_netuid,
                        &origin_hotkey_account,
                        I96F32::saturating_from_num(amount.neg()),
                    )
                }
                _ => DefaultStakingFee::<T>::get(),
            }
        }
    }
}
