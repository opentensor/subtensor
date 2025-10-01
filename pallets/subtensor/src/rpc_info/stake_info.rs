extern crate alloc;

use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::SwapExt;

use super::*;

#[freeze_struct("28269be895d7b5ba")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct StakeInfo<AccountId: TypeInfo + Encode + Decode> {
    hotkey: AccountId,
    coldkey: AccountId,
    netuid: Compact<NetUid>,
    stake: Compact<AlphaCurrency>,
    locked: Compact<u64>,
    emission: Compact<AlphaCurrency>,
    tao_emission: Compact<TaoCurrency>,
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
        let netuids = Self::get_all_subnet_netuids();
        let mut stake_info: Vec<(T::AccountId, Vec<StakeInfo<T::AccountId>>)> = Vec::new();
        for coldkey_i in coldkeys.clone().iter() {
            // Get all hotkeys associated with this coldkey.
            let staking_hotkeys = StakingHotkeys::<T>::get(coldkey_i.clone());
            let mut stake_info_for_coldkey: Vec<StakeInfo<T::AccountId>> = Vec::new();
            for netuid_i in netuids.clone().iter() {
                for hotkey_i in staking_hotkeys.clone().iter() {
                    let alpha = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
                        hotkey_i, coldkey_i, *netuid_i,
                    );
                    if alpha.is_zero() {
                        continue;
                    }
                    let emission = AlphaDividendsPerSubnet::<T>::get(*netuid_i, &hotkey_i);
                    let tao_emission = TaoDividendsPerSubnet::<T>::get(*netuid_i, &hotkey_i);
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
        netuid: NetUid,
    ) -> Option<StakeInfo<T::AccountId>> {
        let alpha = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey_account,
            &coldkey_account,
            netuid,
        );
        let emission = AlphaDividendsPerSubnet::<T>::get(netuid, &hotkey_account);
        let tao_emission = TaoDividendsPerSubnet::<T>::get(netuid, &hotkey_account);
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
        origin: Option<(T::AccountId, NetUid)>,
        _origin_coldkey_account: T::AccountId,
        destination: Option<(T::AccountId, NetUid)>,
        _destination_coldkey_account: T::AccountId,
        amount: u64,
    ) -> u64 {
        if destination == origin {
            0_u64
        } else {
            let netuid = destination.or(origin).map(|v| v.1).unwrap_or_default();
            T::SwapExt::approx_fee_amount(netuid.into(), TaoCurrency::from(amount)).to_u64()
        }
    }
}
