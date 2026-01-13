use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use codec::Compact;
use rate_limiting_interface::RateLimitingInterface;
use sp_runtime::SaturatedConversion;
use subtensor_runtime_common::{AlphaCurrency, MechId, NetUid, NetUidStorageIndex, rate_limiting};

#[freeze_struct("9e5a291e7e71482d")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct NeuronInfo<AccountId: TypeInfo + Encode + Decode> {
    hotkey: AccountId,
    coldkey: AccountId,
    uid: Compact<u16>,
    netuid: Compact<NetUid>,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(AccountId, Compact<AlphaCurrency>)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: Compact<u16>,
    emission: Compact<AlphaCurrency>,
    incentive: Compact<u16>,
    consensus: Compact<u16>,
    trust: Compact<u16>,
    validator_trust: Compact<u16>,
    dividends: Compact<u16>,
    last_update: Compact<u64>,
    validator_permit: bool,
    weights: Vec<(Compact<u16>, Compact<u16>)>, // Vec of (uid, weight)
    bonds: Vec<(Compact<u16>, Compact<u16>)>,   // Vec of (uid, bond)
    pruning_score: Compact<u16>,
}

#[freeze_struct("b9fdff7fc6e023c7")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct NeuronInfoLite<AccountId: TypeInfo + Encode + Decode> {
    hotkey: AccountId,
    coldkey: AccountId,
    uid: Compact<u16>,
    netuid: Compact<NetUid>,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(AccountId, Compact<AlphaCurrency>)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: Compact<u16>,
    emission: Compact<AlphaCurrency>,
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
    pub fn get_neurons(netuid: NetUid) -> Vec<NeuronInfo<T::AccountId>> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new();
        }

        let mut neurons = Vec::new();
        let n = Self::get_subnetwork_n(netuid);
        for uid in 0..n {
            let neuron = match Self::get_neuron_subnet_exists(netuid, uid) {
                Some(n) => n,
                None => break, // No more neurons
            };

            neurons.push(neuron);
        }
        neurons
    }

    fn get_neuron_subnet_exists(netuid: NetUid, uid: u16) -> Option<NeuronInfo<T::AccountId>> {
        let hotkey = match Self::get_hotkey_for_net_and_uid(netuid, uid) {
            Ok(h) => h,
            Err(_) => return None,
        };

        let axon_info = Self::get_axon_info(netuid, &hotkey.clone());

        let prometheus_info = Self::get_prometheus_info(netuid, &hotkey.clone());

        let coldkey = Owner::<T>::get(hotkey.clone()).clone();

        let active = Self::get_active_for_uid(netuid, uid);
        let rank = Self::get_rank_for_uid(netuid, uid);
        let emission = Self::get_emission_for_uid(netuid, uid);
        let incentive = Self::get_incentive_for_uid(netuid.into(), uid);
        let consensus = Self::get_consensus_for_uid(netuid, uid);
        let trust = Self::get_trust_for_uid(netuid, uid);
        let validator_trust = Self::get_validator_trust_for_uid(netuid, uid);
        let dividends = Self::get_dividends_for_uid(netuid, uid);
        let pruning_score = Self::get_pruning_score_for_uid(netuid, uid);
        let usage = Self::weights_rl_usage_key_for_uid(netuid, MechId::from(0u8), uid);
        let last_update =
            T::RateLimiting::last_seen(rate_limiting::GROUP_WEIGHTS_SUBNET, Some(usage))
                .map(|block| block.saturated_into::<u64>())
                .unwrap_or(0);
        let validator_permit = Self::get_validator_permit_for_uid(netuid, uid);

        let weights = Weights::<T>::get(NetUidStorageIndex::from(netuid), uid)
            .into_iter()
            .filter_map(|(i, w)| {
                if w > 0 {
                    Some((i.into(), w.into()))
                } else {
                    None
                }
            })
            .collect::<Vec<(Compact<u16>, Compact<u16>)>>();

        let bonds = Bonds::<T>::get(NetUidStorageIndex::from(netuid), uid)
            .iter()
            .filter_map(|(i, b)| {
                if *b > 0 {
                    Some((i.into(), b.into()))
                } else {
                    None
                }
            })
            .collect::<Vec<(Compact<u16>, Compact<u16>)>>();
        let stake: Vec<(T::AccountId, Compact<AlphaCurrency>)> = vec![(
            coldkey.clone(),
            Self::get_stake_for_hotkey_on_subnet(&hotkey, netuid).into(),
        )];
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
            pruning_score: pruning_score.into(),
        };

        Some(neuron)
    }

    pub fn get_neuron(netuid: NetUid, uid: u16) -> Option<NeuronInfo<T::AccountId>> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        Self::get_neuron_subnet_exists(netuid, uid)
    }

    fn get_neuron_lite_subnet_exists(
        netuid: NetUid,
        uid: u16,
    ) -> Option<NeuronInfoLite<T::AccountId>> {
        let hotkey = match Self::get_hotkey_for_net_and_uid(netuid, uid) {
            Ok(h) => h,
            Err(_) => return None,
        };

        let axon_info = Self::get_axon_info(netuid, &hotkey.clone());

        let prometheus_info = Self::get_prometheus_info(netuid, &hotkey.clone());

        let coldkey = Owner::<T>::get(hotkey.clone()).clone();

        let active = Self::get_active_for_uid(netuid, uid);
        let rank = Self::get_rank_for_uid(netuid, uid);
        let emission = Self::get_emission_for_uid(netuid, uid);
        let incentive = Self::get_incentive_for_uid(netuid.into(), uid);
        let consensus = Self::get_consensus_for_uid(netuid, uid);
        let trust = Self::get_trust_for_uid(netuid, uid);
        let validator_trust = Self::get_validator_trust_for_uid(netuid, uid);
        let dividends = Self::get_dividends_for_uid(netuid, uid);
        let pruning_score = Self::get_pruning_score_for_uid(netuid, uid);
        let usage = Self::weights_rl_usage_key_for_uid(netuid, MechId::from(0u8), uid);
        let last_update =
            T::RateLimiting::last_seen(rate_limiting::GROUP_WEIGHTS_SUBNET, Some(usage))
                .map(|block| block.saturated_into::<u64>())
                .unwrap_or(0);
        let validator_permit = Self::get_validator_permit_for_uid(netuid, uid);

        let stake: Vec<(T::AccountId, Compact<AlphaCurrency>)> = vec![(
            coldkey.clone(),
            Self::get_stake_for_hotkey_on_subnet(&hotkey, netuid).into(),
        )];

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
            pruning_score: pruning_score.into(),
        };

        Some(neuron)
    }

    pub fn get_neurons_lite(netuid: NetUid) -> Vec<NeuronInfoLite<T::AccountId>> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new();
        }

        let mut neurons: Vec<NeuronInfoLite<T::AccountId>> = Vec::new();
        let n = Self::get_subnetwork_n(netuid);
        for uid in 0..n {
            let neuron = match Self::get_neuron_lite_subnet_exists(netuid, uid) {
                Some(n) => n,
                None => break, // No more neurons
            };

            neurons.push(neuron);
        }
        neurons
    }

    pub fn get_neuron_lite(netuid: NetUid, uid: u16) -> Option<NeuronInfoLite<T::AccountId>> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        Self::get_neuron_lite_subnet_exists(netuid, uid)
    }
}
