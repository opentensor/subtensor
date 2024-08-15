use super::*;
extern crate alloc;
use codec::Compact;
use frame_support::pallet_prelude::{Decode, Encode};

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct DynamicInfo<T: Config>{
    tempo: Compact<u16>,
    last_step: Compact<u64>,
    owner: T::AccountId,
    emission: Compact<u64>,
    alpha_in: Compact<u64>,
    alpha_out: Compact<u64>,
    tao_in: Compact<u64>,
    total_locked: Compact<u64>,
    owner_locked: Compact<u64>,
    netuid: Compact<u16>,
}

impl<T: Config> Pallet<T> {
    pub fn get_dynamic_info(netuid: u16) -> Option<DynamicInfo<T>> {
        if !Self::if_subnet_exist(netuid) {return None;}
        Some(DynamicInfo {
            tempo: Tempo::<T>::get( netuid ).into(),
            last_step: LastMechansimStepBlock::<T>::get( netuid ).into(),
            owner: SubnetOwner::<T>::get( netuid ).into(),
            emission: EmissionValues::<T>::get( netuid ).into(),
            alpha_in: SubnetAlphaIn::<T>::get( netuid ).into(),
            alpha_out: SubnetAlphaOut::<T>::get( netuid ).into(),
            tao_in: SubnetTAO::<T>::get( netuid ).into(),
            total_locked: SubnetLocked::<T>::get( netuid ).into(),
            owner_locked: LargestLocked::<T>::get( netuid ).into(),
            netuid: netuid.into(),
        })
    }
    pub fn get_all_dynamic_info() -> Vec<Option<DynamicInfo<T>>> {
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        let mut dynamic_info = Vec::<Option<DynamicInfo<T>>>::new();
        for netuid in netuids.clone().iter() {
            dynamic_info.push(Self::get_dynamic_info(*netuid));
        }
        dynamic_info
    }
}