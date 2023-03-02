use super::*;
use frame_support::IterableStorageDoubleMap;
use serde::{Serialize, Deserialize};
use frame_support::storage::IterableStorageMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;

#[derive(Decode, Encode, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields)]
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

        let rho = <Rho<T>>::get(netuid);
        let kappa = <Kappa<T>>::get(netuid);
        let difficulty = <Difficulty<T>>::get(netuid);
        let immunity_period = <ImmunityPeriod<T>>::get(netuid);
        let validator_batch_size = <ValidatorBatchSize<T>>::get(netuid);
        let validator_sequence_length = <ValidatorSequenceLength<T>>::get(netuid);
        let validator_epochs_per_reset = <ValidatorEpochsPerReset<T>>::get(netuid);
        let validator_epoch_length = <ValidatorEpochLen<T>>::get(netuid);
        let max_allowed_validators = <MaxAllowedValidators<T>>::get(netuid);
        let min_allowed_weights = <MinAllowedWeights<T>>::get(netuid);
        let max_weights_limit = <MaxWeightsLimit<T>>::get(netuid);
        let scaling_law_power = <ScalingLawPower<T>>::get(netuid);
        let synergy_scaling_law_power = <SynergyScalingLawPower<T>>::get(netuid);
        let subnetwork_n = <SubnetworkN<T>>::get(netuid);
        let max_allowed_uids = <MaxAllowedUids <T>>::get(netuid);
        let blocks_since_last_step = <BlocksSinceLastStep <T>>::get(netuid);
        let tempo = <Tempo <T>>::get(netuid);
        let network_modality = <NetworkModality <T>>::get(netuid);
        let emission_values = <EmissionValues <T>>::get(netuid);


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

