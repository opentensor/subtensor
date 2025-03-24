use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
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
        coldkeys: Vec<<T as frame_system::Config>::AccountId>,
    ) -> Vec<(<T as frame_system::Config>::AccountId, Vec<StakeInfo<<T as frame_system::Config>::AccountId>>)> {
        if coldkeys.is_empty() {
            return Vec::new(); // No coldkeys to check
        }
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        let mut stake_info: Vec<(<T as frame_system::Config>::AccountId, Vec<StakeInfo<<T as frame_system::Config>::AccountId>>)> = Vec::new();
        for coldkey_i in coldkeys.clone().iter() {
            // Get all hotkeys associated with this coldkey.
            let staking_hotkeys = StakingHotkeys::<T>::get(coldkey_i.clone());
            let mut stake_info_for_coldkey: Vec<StakeInfo<<T as frame_system::Config>::AccountId>> = Vec::new();
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
        coldkey_accounts: Vec<<T as frame_system::Config>::AccountId>,
    ) -> Vec<(<T as frame_system::Config>::AccountId, Vec<StakeInfo<<T as frame_system::Config>::AccountId>>)> {
        if coldkey_accounts.is_empty() {
            return Vec::new(); // Empty coldkeys
        }

        Self::_get_stake_info_for_coldkeys(coldkey_accounts)
    }

    pub fn get_stake_info_for_coldkey(
        coldkey_account: <T as frame_system::Config>::AccountId,
    ) -> Vec<StakeInfo<<T as frame_system::Config>::AccountId>> {
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
        hotkey_account: <T as frame_system::Config>::AccountId,
        coldkey_account: <T as frame_system::Config>::AccountId,
        netuid: u16,
    ) -> Option<StakeInfo<<T as frame_system::Config>::AccountId>> {
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
        origin: Option<(<T as frame_system::Config>::AccountId, u16)>,
        origin_coldkey_account: <T as frame_system::Config>::AccountId,
        destination: Option<(<T as frame_system::Config>::AccountId, u16)>,
        destination_coldkey_account: <T as frame_system::Config>::AccountId,
        amount: u64,
    ) -> u64 {
        let origin_: Option<(&<T as frame_system::Config>::AccountId, u16)> =
            if let Some((ref origin_hotkey, origin_netuid)) = origin {
                Some((origin_hotkey, origin_netuid))
            } else {
                None
            };

        let destination_ = if let Some((ref destination_hotkey, destination_netuid)) = destination {
            Some((destination_hotkey, destination_netuid))
        } else {
            None
        };

        Self::calculate_staking_fee(
            origin_,
            &origin_coldkey_account,
            destination_,
            &destination_coldkey_account,
            I96F32::saturating_from_num(amount),
        )
    }
}
