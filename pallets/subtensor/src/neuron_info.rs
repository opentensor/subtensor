use
{
    super::
    {
        *
    },
    frame_support::
    {
        storage::
        {
            IterableStorageDoubleMap
        },
        pallet_prelude::
        {
            Decode,
            Encode
        }
    },
    sp_std::
    {
        vec::
        {
            Vec
        }
    },
    codec::
    {
        Compact
    }
};

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct NeuronInfo<T: Config> 
{
    hotkey:             T::AccountId,
    coldkey:            T::AccountId,
    uid:                Compact<u16>,
    netuid:             Compact<u16>,
    active:             bool,
    axon_info:          AxonInfo,
    prometheus_info:    PrometheusInfo,
    stake:              Vec<(T::AccountId, Compact<u64>)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank:               Compact<u16>,
    emission:           Compact<u64>,
    incentive:          Compact<u16>,
    consensus:          Compact<u16>,
    trust:              Compact<u16>,
    validator_trust:    Compact<u16>,
    dividends:          Compact<u16>,
    last_update:        Compact<u64>,
    validator_permit:   bool,
    weights:            Vec<(Compact<u16>, Compact<u16>)>, // Vec of (uid, weight)
    bonds:              Vec<(Compact<u16>, Compact<u16>)>, // Vec of (uid, bond)
    pruning_score:      Compact<u16>
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct NeuronInfoLite<T: Config> 
{
    hotkey:             T::AccountId,
    coldkey:            T::AccountId,
    uid:                Compact<u16>,
    netuid:             Compact<u16>,
    active:             bool,
    axon_info:          AxonInfo,
    prometheus_info:    PrometheusInfo,
    stake:              Vec<(T::AccountId, Compact<u64>)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank:               Compact<u16>,
    emission:           Compact<u64>,
    incentive:          Compact<u16>,
    consensus:          Compact<u16>,
    trust:              Compact<u16>,
    validator_trust:    Compact<u16>,
    dividends:          Compact<u16>,
    last_update:        Compact<u64>,
    validator_permit:   bool,
    // has no weights or bonds
    pruning_score:      Compact<u16>
}

impl<T: Config> Pallet<T> {
	pub fn get_neurons(netuid: u16) -> Vec<NeuronInfo<T>> 
    {
        if !Self::if_subnet_exist(netuid) 
        {
            return Vec::new();
        }

        let mut neurons:    Vec<NeuronInfo<T>>  = Vec::new();
        let n:              u16                 = Self::get_subnetwork_n(netuid);
        for uid in 0..n 
        {
            let neuron = Self::get_neuron_subnet_exists(netuid, uid);
            if neuron.is_none() // No more neurons
            {
                break; 
            } 

            neurons.push(neuron.expect("Neuron should exist"));
        }

        return neurons;
	}

    fn get_neuron_subnet_exists(netuid: u16, uid: u16) -> Option<NeuronInfo<T>> 
    {
        let hotkey = Self::get_hotkey_for_net_and_uid(netuid, uid);
        if hotkey.is_err()
        {
            return None;
        }

        let hotkey:             T::AccountId    = hotkey.expect("Hotkey should exist");
        let axon_info:          AxonInfo        = Self::get_axon_info( netuid, &hotkey.clone() );
        let prometheus_info:    PrometheusInfo  = Self::get_prometheus_info( netuid, &hotkey.clone() );
        let coldkey:            T::AccountId    = Owner::<T>::get( hotkey.clone() ).clone();
        let active:             bool            = Self::get_active_for_uid( netuid, uid as u16 );
        let rank:               u16             = Self::get_rank_for_uid( netuid, uid as u16 );
        let emission:           u64             = Self::get_emission_for_uid( netuid, uid as u16 );
        let incentive:          u16             = Self::get_incentive_for_uid( netuid, uid as u16 );
        let consensus:          u16             = Self::get_consensus_for_uid( netuid, uid as u16 );
        let trust:              u16             = Self::get_trust_for_uid( netuid, uid as u16 );
        let validator_trust:    u16             = Self::get_validator_trust_for_uid( netuid, uid as u16 );
        let dividends:          u16             = Self::get_dividends_for_uid( netuid, uid as u16 );
        let pruning_score:      u16             = Self::get_pruning_score_for_uid( netuid, uid as u16 );
        let last_update:        u64             = Self::get_last_update_for_uid( netuid, uid as u16 );
        let validator_permit:   bool            = Self::get_validator_permit_for_uid( netuid, uid as u16 );

        let weights: Vec<(Compact<u16>, Compact<u16>)>  = <Weights<T>>::get(netuid, uid).iter()
            .filter_map(|(i, w)| if *w > 0 { Some((i.into(), w.into())) } else { None })
            .collect::<Vec<(Compact<u16>, Compact<u16>)>>();
        
        let bonds: Vec<(Compact<u16>, Compact<u16>)>    = <Bonds<T>>::get(netuid, uid).iter()
            .filter_map(|(i, b)| if *b > 0 { Some((i.into(), b.into())) } else { None })
            .collect::<Vec<(Compact<u16>, Compact<u16>)>>();
        
        let stake: Vec<(T::AccountId, Compact<u64>)>    = <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(hotkey.clone())
            .map(|(coldkey, stake)| (coldkey, stake.into()))
            .collect();

        return Some(NeuronInfo 
        {
            hotkey:             hotkey.clone(),
            coldkey:            coldkey.clone(),
            uid:                uid.into(),
            netuid:             netuid.into(),
            active,
            axon_info,
            prometheus_info,
            stake,
            rank:               rank.into(),
            emission:           emission.into(),
            incentive:          incentive.into(),
            consensus:          consensus.into(),
            trust:              trust.into(),
            validator_trust:    validator_trust.into(),
            dividends:          dividends.into(),
            last_update:        last_update.into(),
            validator_permit,
            weights,
            bonds,
            pruning_score:      pruning_score.into()
        });
    }

    pub fn get_neuron(netuid: u16, uid: u16) -> Option<NeuronInfo<T>> 
    {
        if !Self::if_subnet_exist(netuid) 
        {
            return None;
        }

        return Self::get_neuron_subnet_exists(netuid, uid);
	}

    fn get_neuron_lite_subnet_exists(netuid: u16, uid: u16) -> Option<NeuronInfoLite<T>> {
        let hotkey = Self::get_hotkey_for_net_and_uid(netuid, uid);
        if hotkey.is_err() 
        {
            return None;
        }

        let hotkey:             T::AccountId    = hotkey.expect("Hotkey should exist");
        let axon_info:          AxonInfo        = Self::get_axon_info( netuid, &hotkey.clone() );
        let prometheus_info:    PrometheusInfo  = Self::get_prometheus_info( netuid, &hotkey.clone() );
        let coldkey:            T::AccountId    = Owner::<T>::get( hotkey.clone() ).clone();
        let active:             bool            = Self::get_active_for_uid( netuid, uid as u16 );
        let rank:               u16             = Self::get_rank_for_uid( netuid, uid as u16 );
        let emission:           u64             = Self::get_emission_for_uid( netuid, uid as u16 );
        let incentive:          u16             = Self::get_incentive_for_uid( netuid, uid as u16 );
        let consensus:          u16             = Self::get_consensus_for_uid( netuid, uid as u16 );
        let trust:              u16             = Self::get_trust_for_uid( netuid, uid as u16 );
        let validator_trust:    u16             = Self::get_validator_trust_for_uid( netuid, uid as u16 );
        let dividends:          u16             = Self::get_dividends_for_uid( netuid, uid as u16 );
        let pruning_score:      u16             = Self::get_pruning_score_for_uid( netuid, uid as u16 );
        let last_update:        u64             = Self::get_last_update_for_uid( netuid, uid as u16 );
        let validator_permit:   bool            = Self::get_validator_permit_for_uid( netuid, uid as u16 );

        /*let stake: Vec<(T::AccountId, Compact<u64>)> = <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(hotkey.clone())
            .map(|(coldkey, stake)| (coldkey, stake.into()))
            .collect();*/

        return Some(NeuronInfoLite
        {
            hotkey:             hotkey.clone(),
            coldkey:            coldkey.clone(),
            uid:                uid.into(),
            netuid:             netuid.into(),
            active,
            axon_info,
            prometheus_info,
            stake:              vec![],
            rank:               rank.into(),
            emission:           emission.into(),
            incentive:          incentive.into(),
            consensus:          consensus.into(),
            trust:              trust.into(),
            validator_trust:    validator_trust.into(),
            dividends:          dividends.into(),
            last_update:        last_update.into(),
            validator_permit,
            pruning_score:      pruning_score.into()
        });
    }    

    pub fn get_neurons_lite(netuid: u16) -> Vec<NeuronInfoLite<T>> 
    {
        if !Self::if_subnet_exist(netuid) 
        {
            return Vec::new();
        }

        let mut neurons:    Vec<NeuronInfoLite<T>>  = Vec::new();
        let n:              u16                     = Self::get_subnetwork_n(netuid);
        for uid in 0..n 
        {
            let neuron = Self::get_neuron_lite_subnet_exists(netuid, uid);
            if neuron.is_none() 
            {
                break; // No more neurons
            } 

            neurons.push(neuron.expect("Neuron should exist"));
        }
        
        return neurons;
    }

    pub fn get_neuron_lite(netuid: u16, uid: u16) -> Option<NeuronInfoLite<T>> 
    {
        if !Self::if_subnet_exist(netuid) 
        {
            return None;
        }

        return Self::get_neuron_lite_subnet_exists(netuid, uid);
   }
}

