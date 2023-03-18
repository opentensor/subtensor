use super::*;
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use alloc::vec::Vec;
use codec::Compact;

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct NeuronInfo<T: Config> {
    hotkey: T::AccountId,
    coldkey: T::AccountId,
    uid: Compact<u16>,
    netuid: Compact<u16>,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(T::AccountId, Compact<u64>)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: Compact<u16>,
    emission: Compact<u64>,
    incentive: Compact<u16>,
    consensus: Compact<u16>,
    trust: Compact<u16>,
    validator_trust: Compact<u16>,
    dividends: Compact<u16>,
    last_update: Compact<u64>,
    validator_permit: bool,
    weights: Vec<(Compact<u16>, Compact<u16>)>, // Vec of (uid, weight)
    bonds: Vec<(Compact<u16>, Compact<u16>)>, // Vec of (uid, bond)
    pruning_score: Compact<u16>,
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct NeuronInfoLite<T: Config> {
    hotkey: T::AccountId,
    coldkey: T::AccountId,
    uid: Compact<u16>,
    netuid: Compact<u16>,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(T::AccountId, Compact<u64>)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: Compact<u16>,
    emission: Compact<u64>,
    incentive: Compact<u16>,
    consensus: Compact<u16>,
    trust: Compact<u16>,
    validator_trust: Compact<u16>,
    dividends: Compact<u16>,
    last_update: Compact<u64>,
    validator_permit: bool,
    // has no weights or bonds
    pruning_score: Compact<u16>,
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

        let weights = <Weights<T>>::get(netuid, uid).iter()
            .filter_map(|(i, w)| if *w > 0 { Some((i.into(), w.into())) } else { None })
            .collect::<Vec<(Compact<u16>, Compact<u16>)>>();
        
        let bonds = <Bonds<T>>::get(netuid, uid).iter()
            .filter_map(|(i, b)| if *b > 0 { Some((i.into(), b.into())) } else { None })
            .collect::<Vec<(Compact<u16>, Compact<u16>)>>();
        
        let stake: Vec<(T::AccountId, Compact<u64>)> = < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( hotkey.clone() )
            .map(|(coldkey, stake)| (coldkey, stake.into()))
            .collect();

        let neuron = NeuronInfo {
            hotkey: hotkey.clone(),
            coldkey: coldkey.clone(),
            uid: uid.into(),
            netuid: netuid.into(),
            active,
            axon_info,
            prometheus_info,
            stake,
            rank: rank.into(),
            emission: emission.into(),
            incentive: incentive.into(),
            consensus: consensus.into(),
            trust: trust.into(),
            validator_trust: validator_trust.into(),
            dividends: dividends.into(),
            last_update: last_update.into(),
            validator_permit,
            weights,
            bonds,
            pruning_score: pruning_score.into()
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

    fn get_neuron_lite_subnet_exists(netuid: u16, uid: u16) -> Option<NeuronInfoLite<T>> {
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

        let stake: Vec<(T::AccountId, Compact<u64>)> = < Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64> >::iter_prefix( hotkey.clone() )
            .map(|(coldkey, stake)| (coldkey, stake.into()))
            .collect();

        let neuron = NeuronInfoLite {
            hotkey: hotkey.clone(),
            coldkey: coldkey.clone(),
            uid: uid.into(),
            netuid: netuid.into(),
            active,
            axon_info,
            prometheus_info,
            stake,
            rank: rank.into(),
            emission: emission.into(),
            incentive: incentive.into(),
            consensus: consensus.into(),
            trust: trust.into(),
            validator_trust: validator_trust.into(),
            dividends: dividends.into(),
            last_update: last_update.into(),
            validator_permit,
            pruning_score: pruning_score.into()
        };
        
        return Some(neuron);
    }    

    pub fn get_neurons_lite(netuid: u16) -> Vec<NeuronInfoLite<T>> {
         if !Self::if_subnet_exist(netuid) {
            return Vec::new();
        }

        let mut neurons: Vec<NeuronInfoLite<T>> = Vec::new();
        let n = Self::get_subnetwork_n(netuid);
        for uid in 0..n {
            let uid = uid;

            let _neuron = Self::get_neuron_lite_subnet_exists(netuid, uid);
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

    pub fn get_neuron_lite(netuid: u16, uid: u16) -> Option<NeuronInfoLite<T>> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        let neuron = Self::get_neuron_lite_subnet_exists(netuid, uid);
        neuron
   }
}

