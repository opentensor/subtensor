use super::*;
use frame_support::IterableStorageDoubleMap;
use frame_support::storage::IterableStorageMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetInfo {
    netuid: u16,
    rho: u16,
    kappa: u16,
    difficulty: u64,
    immunity_period: u16,
    validator_batch_size: u16,
    validator_sequence_length: u16,
    validator_epochs_per_reset: u16,
    validator_epoch_length: u16,
    max_allowed_validators: u16,
    min_allowed_weights: u16,
    max_weights_limit: u16,
    scaling_law_power: u16,
    synergy_scaling_law_power: u16,
    subnetwork_n: u16,
    max_allowed_uids: u16,
    blocks_since_last_step: u64,
    tempo: u16,
    network_modality: u16,
    network_connect: Vec<[u16; 2]>,
    emission_values: u64
}

impl<T: Config> Pallet<T> {
	pub fn get_subnet_info(netuid: u16) -> Option<SubnetInfo> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        let rho = Self::get_rho(netuid);
        let kappa = Self::get_kappa(netuid);
        let difficulty = Self::get_difficulty_as_u64(netuid);
        let immunity_period = Self::get_immunity_period(netuid);
        let validator_batch_size = Self::get_validator_batch_size(netuid);
        let validator_sequence_length = Self::get_validator_sequence_length(netuid);
        let validator_epochs_per_reset = Self::get_validator_epochs_per_reset(netuid);
        let validator_epoch_length = Self::get_validator_epoch_length(netuid);
        let max_allowed_validators = Self::get_max_allowed_validators(netuid);
        let min_allowed_weights = Self::get_min_allowed_weights(netuid);
        let max_weights_limit = Self::get_max_weight_limit(netuid);
        let scaling_law_power = Self::get_scaling_law_power(netuid);
        let synergy_scaling_law_power = Self::get_synergy_scaling_law_power(netuid);
        let subnetwork_n = Self::get_subnetwork_n(netuid);
        let max_allowed_uids = Self::get_max_allowed_uids(netuid);
        let blocks_since_last_step = Self::get_blocks_since_last_step(netuid);
        let tempo = Self::get_tempo(netuid);
        let network_modality = <NetworkModality <T>>::get(netuid);
        let emission_values = Self::get_emission_value(netuid);


        let mut network_connect: Vec<[u16; 2]> = Vec::<[u16; 2]>::new();

        for ( _netuid_, con_req) in < NetworkConnect<T> as IterableStorageDoubleMap<u16, u16, u16> >::iter_prefix(netuid) {
            network_connect.push([_netuid_, con_req]);
        }

        return Some(SubnetInfo {
            rho,
            kappa,
            difficulty,
            immunity_period,
            netuid,
            validator_batch_size,
            validator_sequence_length,
            validator_epochs_per_reset,
            validator_epoch_length,
            max_allowed_validators,
            min_allowed_weights,
            max_weights_limit,
            scaling_law_power,
            synergy_scaling_law_power,
            subnetwork_n,
            max_allowed_uids,
            blocks_since_last_step,
            tempo,
            network_modality,
            network_connect,
            emission_values
        })
	}

    pub fn get_subnets_info() -> Vec<Option<SubnetInfo>> {
        let mut subnet_netuids = Vec::<u16>::new();
        let mut max_netuid: u16 = 0;
        for ( netuid, added ) in < NetworksAdded<T> as IterableStorageMap<u16, bool> >::iter() {
            if added {
                subnet_netuids.push(netuid);
                if netuid > max_netuid {
                    max_netuid = netuid;
                }
            }
        }

        let mut subnets_info = Vec::<Option<SubnetInfo>>::new();
        for netuid_ in 0..(max_netuid + 1) {
            if subnet_netuids.contains(&netuid_) {
                subnets_info.push(Self::get_subnet_info(netuid_));
            }
        }

        return subnets_info;
	}
}

