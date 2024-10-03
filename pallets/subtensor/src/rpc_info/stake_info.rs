use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use codec::Compact;

#[freeze_struct("4f16c654467bc8b6")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct StakeInfo<AccountId: TypeInfo + Encode + Decode> {
    hotkey: AccountId,
    coldkey: AccountId,
    netuid: Compact<u16>,
    stake: Compact<u64>,
    locked: Compact<u64>,
    emission: Compact<u64>,
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
                    let alpha: u64 =
                        Alpha::<T>::get((hotkey_i.clone(), coldkey_i.clone(), netuid_i));
                    let emission: u64 = LastHotkeyColdkeyEmissionOnNetuid::<T>::get((
                        hotkey_i.clone(),
                        coldkey_i.clone(),
                        *netuid_i,
                    ));
                    let drain: u64 = LastHotkeyEmissionDrain::<T>::get(hotkey_i.clone());
                    let conviction: u64 = Self::get_conviction_for_hotkey_and_coldkey_on_subnet(
                        hotkey_i, coldkey_i, *netuid_i,
                    );
                    let is_registered: bool =
                        Self::is_hotkey_registered_on_network(*netuid_i, &hotkey_i);
                    stake_info_for_coldkey.push(StakeInfo {
                        hotkey: hotkey_i.clone(),
                        coldkey: coldkey_i.clone(),
                        netuid: (*netuid_i).into(),
                        stake: alpha.into(),
                        locked: conviction.into(),
                        emission: emission.into(),
                        drain: drain.into(),
                        is_registered: is_registered,
                    });
                }
            }
            stake_info.push((coldkey_i.clone(), stake_info_for_coldkey));
        }
        stake_info
    }

    pub fn get_stake_info_for_coldkeys(
        coldkeys: Vec<T::AccountId>,
    ) -> Vec<(T::AccountId, Vec<StakeInfo<T::AccountId>>)> {
        if coldkeys.is_empty() {
            return Vec::new(); // Invalid coldkey
        }

        Self::_get_stake_info_for_coldkeys(coldkeys)
    }

    pub fn get_stake_info_for_coldkey(coldkey: T::AccountId) -> Vec<StakeInfo<T::AccountId>> {
        let stake_info = Self::_get_stake_info_for_coldkeys(vec![coldkey]);

        if stake_info.is_empty() {
            Vec::new() // Invalid coldkey
        } else {
            let Some(first) = stake_info.first() else {
                return Vec::new();
            };

            first.1.clone()
        }
    }
}
