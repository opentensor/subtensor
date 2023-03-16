use super::*;
use crate::math::*;
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct NeuronInfo<T: Config> {
    hotkey: T::AccountId,
    coldkey: T::AccountId,
    uid: u16,
    netuid: u16,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(T::AccountId, u64)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: u16,
    emission: u64,
    incentive: u16,
    consensus: u16,
    trust: u16,
    validator_trust: u16,
    dividends: u16,
    last_update: u64,
    validator_permit: bool,
    weights: Vec<(u16, u16)>, // Vec of (uid, weight)
    bonds: Vec<(u16, u16)>, // Vec of (uid, bond)
    pruning_score: u16
}

impl<T: Config> Pallet<T> {
	pub fn get_neurons(netuid: u16) -> Vec<NeuronInfo<T>> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new();
        }

        let mut neurons = Vec::new();
        let n = Self::get_subnetwork_n(netuid);
        for uid in 0..n {
            let uid = uid;
            let netuid = netuid;

            let _neuron = Self::get_neuron_subnet_exists(netuid, uid);
            let neuron;
            if _neuron.is_none() {
                break; // No more neurons
            } else {
                // No error, hotkey was registered
                neuron = _neuron.expect("Neuron should exist");
            }

            neurons.push( neuron );
        }
        neurons
	}

    fn get_neuron_subnet_exists(netuid: u16, uid: u16) -> Option<NeuronInfo<T>> {
        let _hotkey = Self::get_hotkey_for_net_and_uid(netuid, uid);
        let hotkey;
        if _hotkey.is_err() {
            return None;
        } else {
            // No error, hotkey was registered
            hotkey = _hotkey.expect("Hotkey should exist");
        }

        let axon_info = Self::get_axon_info( netuid, &hotkey.clone() );

        let prometheus_info = Self::get_prometheus_info( netuid, &hotkey.clone() );

        
        let coldkey = Owner::<T>::get( hotkey.clone() ).clone();
        
        let active = Self::get_active_for_uid( netuid, uid as u16 );
        let rank = Self::get_rank_for_uid( netuid, uid as u16 );
        let emission = Self::get_emission_for_uid( netuid, uid as u16 );
        let incentive = Self::get_incentive_for_uid( netuid, uid as u16 );
        let consensus = Self::get_consensus_for_uid( netuid, uid as u16 );
        let trust = Self::get_trust_for_uid( netuid, uid as u16 );
        let validator_trust = Self::get_validator_trust_for_uid( netuid, uid as u16 );
        let dividends = Self::get_dividends_for_uid( netuid, uid as u16 );
        let pruning_score = Self::get_pruning_score_for_uid( netuid, uid as u16 );
        let last_update = Self::get_last_update_for_uid( netuid, uid as u16 );
        let validator_permit = Self::get_validator_permit_for_uid( netuid, uid as u16 );

        let weights = Self::get_weights(netuid)[uid as usize].iter()
            .enumerate()
            .map(|(i, w)| (i as u16, fixed_proportion_to_u16(*w)))
            .filter(|(_, b)| *b > 0)
            .collect::<Vec<(u16, u16)>>();
        
        let bonds = Self::get_bonds(netuid)[uid as usize].iter()
            .enumerate()
            .map(|(i, b)| (i as u16, fixed_proportion_to_u16(*b)))
            .filter(|(_, b)| *b > 0)
            .collect::<Vec<(u16, u16)>>();
        
        let mut stakes = Vec::<(T::AccountId, u64)>::new();
        for ( coldkey, stake ) in < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( hotkey.clone() ) {
            stakes.push( (coldkey.clone(), stake) );
        }

        let stake = stakes;

        let neuron = NeuronInfo {
            hotkey: hotkey.clone(),
            coldkey: coldkey.clone(),
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

    pub fn get_neuron(netuid: u16, uid: u16) -> Option<NeuronInfo<T>> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        let neuron = Self::get_neuron_subnet_exists(netuid, uid);
        neuron
	}
}

