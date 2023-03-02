use super::*;
use serde::{Serialize, Deserialize};
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;

#[derive(Decode, Encode, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct NeuronInfo {
    hotkey: DeAccountId,
    coldkey: DeAccountId,
    uid: u16,
    netuid: u16,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(DeAccountId, u64)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: u16,
    emission: u64,
    incentive: u16,
    consensus: u16,
    weight_consensus: u16,
    trust: u16,
    validator_trust: u16,
    dividends: u16,
    last_update: u64,
    validator_permit: bool,
    weights: Vec<(u16, u16)>, // map of uid to weight
    bonds: Vec<(u16, u16)>, // map of uid to bond
    pruning_score: u16
}

impl<T: Config> Pallet<T> {
	pub fn get_neurons(netuid: u16) -> Vec<NeuronInfo> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new();
        }

        let mut neurons = Vec::new();
        let n = SubnetworkN::<T>::get( netuid ); 
        for uid in 0..n {
            let uid = uid;
            let netuid = netuid;

            let hotkey = Keys::<T>::get( netuid, uid as u16 ).clone();

            let axon_ = Axons::<T>::get( hotkey.clone() );
            let axon_info;
            if axon_.is_some() {
                axon_info = axon_.unwrap();
            } else {
                axon_info = AxonInfo::default();
            }

            let promo_ = Prometheus::<T>::get( hotkey.clone() );
            let prometheus_info;
            if promo_.is_some() {
                prometheus_info = promo_.unwrap();
            } else {
                prometheus_info = PrometheusInfo::default();
            }

            
            let coldkey = Owner::<T>::get( hotkey.clone() ).clone();
            
            // TODO: replace with last_update check if we remove Active storage
            let active = Self::get_active_for_uid( netuid, uid as u16 );
            let rank = Self::get_rank_for_uid( netuid, uid as u16 );
            let emission = Self::get_emission_for_uid( netuid, uid as u16 );
            let incentive = Self::get_incentive_for_uid( netuid, uid as u16 );
            let consensus = Self::get_consensus_for_uid( netuid, uid as u16 );
            let weight_consensus = Self::get_weight_consensus_for_uid( netuid, uid as u16 );
            let trust = Self::get_trust_for_uid( netuid, uid as u16 );
            let validator_trust = Self::get_validator_trust_for_uid( netuid, uid as u16 );
            let dividends = Self::get_dividends_for_uid( netuid, uid as u16 );
            let pruning_score = Self::get_pruning_score_for_uid( netuid, uid as u16 );
            let last_update = Self::get_last_update_for_uid( netuid, uid as u16 );
            let validator_permit = Self::get_validator_permit_for_uid( netuid, uid as u16 );

            let weights = Weights::<T>::get( netuid, uid as u16 );
            let bonds = Bonds::<T>::get( netuid, uid as u16 );
            
            let mut stakes = Vec::<(DeAccountId, u64)>::new();
            for ( coldkey, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( hotkey.clone() ) {
                stakes.push( (coldkey.clone().encode().into(), stake) );
            }

            let stake = stakes;

            let neuron = NeuronInfo {
                hotkey: hotkey.clone().encode().into(),
                coldkey: coldkey.clone().encode().into(),
                uid,
                netuid,
                active,
                axon_info,
                prometheus_info,
                stake,
                rank,
                emission,
                incentive,
                consensus,
                weight_consensus,
                trust,
                validator_trust,
                dividends,
                last_update,
                validator_permit,
                weights,
                bonds,
                pruning_score
            };
            
            neurons.push( neuron );
        }
        neurons
	}

    pub fn get_neuron(netuid: u16, uid: u16) -> Option<NeuronInfo> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }


        let hotkey = Keys::<T>::get( netuid, uid as u16 ).clone();

        let axon_ = Axons::<T>::get( hotkey.clone() );
        let axon_info;
        if axon_.is_some() {
            axon_info = axon_.unwrap();
        } else {
            axon_info = AxonInfo::default();
        }

        let promo_ = Prometheus::<T>::get( hotkey.clone() );
        let prometheus_info;
        if promo_.is_some() {
            prometheus_info = promo_.unwrap();
        } else {
            prometheus_info = PrometheusInfo::default();
        }

        let coldkey = Owner::<T>::get( hotkey.clone() ).clone();        

        let active = Self::get_active_for_uid( netuid, uid as u16 );
        let rank = Self::get_rank_for_uid( netuid, uid as u16 );
        let emission = Self::get_emission_for_uid( netuid, uid as u16 );
        let incentive = Self::get_incentive_for_uid( netuid, uid as u16 );
        let consensus = Self::get_consensus_for_uid( netuid, uid as u16 );
        let weight_consensus = Self::get_weight_consensus_for_uid( netuid, uid as u16 );
        let trust = Self::get_trust_for_uid( netuid, uid as u16 );
        let validator_trust = Self::get_validator_trust_for_uid( netuid, uid as u16 );
        let dividends = Self::get_dividends_for_uid( netuid, uid as u16 );
        let pruning_score = Self::get_pruning_score_for_uid( netuid, uid as u16 );
        let last_update = Self::get_last_update_for_uid( netuid, uid as u16 );
        let validator_permit = Self::get_validator_permit_for_uid( netuid, uid as u16 );

        let weights = Weights::<T>::get( netuid, uid as u16 );
        let bonds = Bonds::<T>::get( netuid, uid as u16 );
        
        let mut stakes = Vec::<(DeAccountId, u64)>::new();
        for ( coldkey, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( hotkey.clone() ) {
            stakes.push( (coldkey.clone().encode().into(), stake) );
        }

        let stake = stakes;

        let neuron = NeuronInfo {
            hotkey: hotkey.clone().encode().into(),
            coldkey: coldkey.clone().encode().into(),
            uid,
            netuid,
            active,
            axon_info,
            prometheus_info,
            stake,
            rank,
            emission,
            incentive,
            consensus,
            weight_consensus,
            trust,
            validator_trust,
            dividends,
            last_update,
            validator_permit,
            weights,
            bonds,
            pruning_score
        };
        
        return Some(neuron);
	}
}

