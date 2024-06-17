use super::*;
use frame_support::{
    pallet_prelude::{Decode, Encode},
};
extern crate alloc;
use codec::Compact;
use sp_std::vec::Vec;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DynamicPoolInfo {
    pub subnet_stake: Compact<u64>,
    pub alpha_issuance: Compact<u64>,
    pub alpha_outstanding: Compact<u64>,
    pub alpha_reserve: Compact<u64>,
    pub tao_reserve: Compact<u64>,
    pub k: Compact<u128>,
    pub price: Compact<u128>,
    pub netuid: Compact<u16>,
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DynamicPoolInfoV2 {
    pub netuid: Compact<u16>,
    pub alpha_issuance: Compact<u64>,
    pub alpha_outstanding: Compact<u64>,
    pub alpha_reserve: Compact<u64>,
    pub tao_reserve: Compact<u64>,
}

impl<T: Config> Pallet<T> {
    pub fn get_dynamic_pool_info_v2(netuid: u16) -> Option<DynamicPoolInfoV2> {
        if !Self::is_subnet_dynamic(netuid) || !Self::if_subnet_exist(netuid) {
            return None;
        }

        let alpha_issuance: u64 = Self::get_alpha_issuance(netuid);
        let alpha_outstanding: u64 = Self::get_alpha_outstanding(netuid);
        let alpha_reserve: u64 = Self::get_alpha_reserve(netuid);
        let tao_reserve: u64 = Self::get_tao_reserve(netuid);

        // Return the dynamic pool info.
        Some(DynamicPoolInfoV2 {
            netuid: netuid.into(),
            alpha_issuance: Compact(alpha_issuance),
            alpha_outstanding: Compact(alpha_outstanding),
            alpha_reserve: Compact(alpha_reserve),
            tao_reserve: Compact(tao_reserve),
        })
    }

    pub fn get_all_dynamic_pool_infos_v2() -> Vec<DynamicPoolInfoV2> {
        Self::get_all_subnet_netuids()
            .iter()
            .map(|&netuid| Self::get_dynamic_pool_info_v2(netuid))
            .filter(|info| info.is_some())
            .map(|info| info.unwrap())
            .collect()
    }
}
