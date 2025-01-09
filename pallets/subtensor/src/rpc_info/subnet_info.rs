use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::storage::IterableStorageMap;
extern crate alloc;
use codec::Compact;

#[freeze_struct("fe79d58173da662a")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetInfo<T: Config> {
    netuid: Compact<u16>,
    rho: Compact<u16>,
    kappa: Compact<u16>,
    difficulty: Compact<u64>,
    immunity_period: Compact<u16>,
    max_allowed_validators: Compact<u16>,
    min_allowed_weights: Compact<u16>,
    max_weights_limit: Compact<u16>,
    scaling_law_power: Compact<u16>,
    subnetwork_n: Compact<u16>,
    max_allowed_uids: Compact<u16>,
    blocks_since_last_step: Compact<u64>,
    tempo: Compact<u16>,
    network_modality: Compact<u16>,
    network_connect: Vec<[u16; 2]>,
    emission_values: Compact<u64>,
    burn: Compact<u64>,
    owner: T::AccountId,
}

#[freeze_struct("65f931972fa13222")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetInfov2<T: Config> {
    netuid: Compact<u16>,
    rho: Compact<u16>,
    kappa: Compact<u16>,
    difficulty: Compact<u64>,
    immunity_period: Compact<u16>,
    max_allowed_validators: Compact<u16>,
    min_allowed_weights: Compact<u16>,
    max_weights_limit: Compact<u16>,
    scaling_law_power: Compact<u16>,
    subnetwork_n: Compact<u16>,
    max_allowed_uids: Compact<u16>,
    blocks_since_last_step: Compact<u64>,
    tempo: Compact<u16>,
    network_modality: Compact<u16>,
    network_connect: Vec<[u16; 2]>,
    emission_values: Compact<u64>,
    burn: Compact<u64>,
    owner: T::AccountId,
    identity: Option<SubnetIdentity>,
}

#[freeze_struct("55b472510f10e76a")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetHyperparams {
    rho: Compact<u16>,
    kappa: Compact<u16>,
    immunity_period: Compact<u16>,
    min_allowed_weights: Compact<u16>,
    max_weights_limit: Compact<u16>,
    tempo: Compact<u16>,
    min_difficulty: Compact<u64>,
    max_difficulty: Compact<u64>,
    weights_version: Compact<u64>,
    weights_rate_limit: Compact<u64>,
    adjustment_interval: Compact<u16>,
    activity_cutoff: Compact<u16>,
    pub registration_allowed: bool,
    target_regs_per_interval: Compact<u16>,
    min_burn: Compact<u64>,
    max_burn: Compact<u64>,
    bonds_moving_avg: Compact<u64>,
    max_regs_per_block: Compact<u16>,
    serving_rate_limit: Compact<u64>,
    max_validators: Compact<u16>,
    adjustment_alpha: Compact<u64>,
    difficulty: Compact<u64>,
    commit_reveal_weights_interval: Compact<u64>,
    commit_reveal_weights_enabled: bool,
    alpha_high: Compact<u16>,
    alpha_low: Compact<u16>,
    liquid_alpha_enabled: bool,
}

impl<T: Config> Pallet<T> {
    pub fn get_subnet_info(netuid: u16) -> Option<SubnetInfo<T>> {
        if !NetworksAdded::<T>::get(netuid) {
            return None;
        }

        let rho = Rho::<T>::get(netuid);
        let kappa = Kappa::<T>::get(netuid);
        let difficulty: Compact<u64> = Difficulty::<T>::get(netuid).into();
        let immunity_period = ImmunityPeriod::<T>::get(netuid);
        let max_allowed_validators = MaxAllowedValidators::<T>::get(netuid);
        let min_allowed_weights = MinAllowedWeights::<T>::get(netuid);
        let max_weights_limit = MaxWeightsLimit::<T>::get(netuid);
        let scaling_law_power = ScalingLawPower::<T>::get(netuid);
        let subnetwork_n = SubnetworkN::<T>::get(netuid);
        let max_allowed_uids = MaxAllowedUids::<T>::get(netuid);
        let blocks_since_last_step = BlocksSinceLastStep::<T>::get(netuid);
        let tempo = Tempo::<T>::get(netuid);
        let network_modality = <NetworkModality<T>>::get(netuid);
        let emission_values = EmissionValues::<T>::get(netuid);
        let burn: Compact<u64> = Burn::<T>::get(netuid).into();
        // DEPRECATED
        let network_connect: Vec<[u16; 2]> = Vec::<[u16; 2]>::new();
        // DEPRECATED for ( _netuid_, con_req) in < NetworkConnect<T> as IterableStorageDoubleMap<u16, u16, u16> >::iter_prefix(netuid) {
        //     network_connect.push([_netuid_, con_req]);
        // }

        Some(SubnetInfo {
            rho: rho.into(),
            kappa: kappa.into(),
            difficulty,
            immunity_period: immunity_period.into(),
            netuid: netuid.into(),
            max_allowed_validators: max_allowed_validators.into(),
            min_allowed_weights: min_allowed_weights.into(),
            max_weights_limit: max_weights_limit.into(),
            scaling_law_power: scaling_law_power.into(),
            subnetwork_n: subnetwork_n.into(),
            max_allowed_uids: max_allowed_uids.into(),
            blocks_since_last_step: blocks_since_last_step.into(),
            tempo: tempo.into(),
            network_modality: network_modality.into(),
            network_connect,
            emission_values: emission_values.into(),
            burn,
            owner: SubnetOwner::<T>::get(netuid),
        })
    }

    pub fn get_subnets_info() -> Vec<Option<SubnetInfo<T>>> {
        let mut subnet_netuids = Vec::<u16>::new();
        let mut max_netuid: u16 = 0;
        for (netuid, added) in <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter() {
            if added {
                subnet_netuids.push(netuid);
                if netuid > max_netuid {
                    max_netuid = netuid;
                }
            }
        }

        let mut subnets_info = Vec::<Option<SubnetInfo<T>>>::new();
        for netuid_ in 0..=max_netuid {
            if subnet_netuids.contains(&netuid_) {
                subnets_info.push(Self::get_subnet_info(netuid_));
            }
        }

        subnets_info
    }

    pub fn get_subnet_info_v2(netuid: u16) -> Option<SubnetInfov2<T>> {
        if !NetworksAdded::<T>::get(netuid) {
            return None;
        }

        let rho = Rho::<T>::get(netuid);
        let kappa = Kappa::<T>::get(netuid);
        let difficulty: Compact<u64> = Difficulty::<T>::get(netuid).into();
        let immunity_period = ImmunityPeriod::<T>::get(netuid);
        let max_allowed_validators = MaxAllowedValidators::<T>::get(netuid);
        let min_allowed_weights = MinAllowedWeights::<T>::get(netuid);
        let max_weights_limit = MaxWeightsLimit::<T>::get(netuid);
        let scaling_law_power = ScalingLawPower::<T>::get(netuid);
        let subnetwork_n = SubnetworkN::<T>::get(netuid);
        let max_allowed_uids = MaxAllowedUids::<T>::get(netuid);
        let blocks_since_last_step = BlocksSinceLastStep::<T>::get(netuid);
        let tempo = Tempo::<T>::get(netuid);
        let network_modality = <NetworkModality<T>>::get(netuid);
        let emission_values = EmissionValues::<T>::get(netuid);
        let burn: Compact<u64> = Burn::<T>::get(netuid).into();
        let identity: Option<SubnetIdentity> = SubnetIdentities::<T>::get(netuid);

        // DEPRECATED
        let network_connect: Vec<[u16; 2]> = Vec::<[u16; 2]>::new();
        // DEPRECATED for ( _netuid_, con_req) in < NetworkConnect<T> as IterableStorageDoubleMap<u16, u16, u16> >::iter_prefix(netuid) {
        //     network_connect.push([_netuid_, con_req]);
        // }

        Some(SubnetInfov2 {
            rho: rho.into(),
            kappa: kappa.into(),
            difficulty,
            immunity_period: immunity_period.into(),
            netuid: netuid.into(),
            max_allowed_validators: max_allowed_validators.into(),
            min_allowed_weights: min_allowed_weights.into(),
            max_weights_limit: max_weights_limit.into(),
            scaling_law_power: scaling_law_power.into(),
            subnetwork_n: subnetwork_n.into(),
            max_allowed_uids: max_allowed_uids.into(),
            blocks_since_last_step: blocks_since_last_step.into(),
            tempo: tempo.into(),
            network_modality: network_modality.into(),
            network_connect,
            emission_values: emission_values.into(),
            burn,
            owner: SubnetOwner::<T>::get(netuid),
            identity,
        })
    }
    pub fn get_subnets_info_v2() -> Vec<Option<SubnetInfo<T>>> {
        let mut subnet_netuids = Vec::<u16>::new();
        let mut max_netuid: u16 = 0;
        for (netuid, added) in <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter() {
            if added {
                subnet_netuids.push(netuid);
                if netuid > max_netuid {
                    max_netuid = netuid;
                }
            }
        }

        let mut subnets_info = Vec::<Option<SubnetInfo<T>>>::new();
        for netuid_ in 0..=max_netuid {
            if subnet_netuids.contains(&netuid_) {
                subnets_info.push(Self::get_subnet_info(netuid_));
            }
        }

        subnets_info
    }
    pub fn get_subnet_hyperparams(netuid: u16) -> Option<SubnetHyperparams> {
        if !NetworksAdded::<T>::get(netuid) {
            return None;
        }

        let rho = Rho::<T>::get(netuid);
        let kappa = Kappa::<T>::get(netuid);
        let immunity_period = ImmunityPeriod::<T>::get(netuid);
        let min_allowed_weights = MinAllowedWeights::<T>::get(netuid);
        let max_weights_limit = MaxWeightsLimit::<T>::get(netuid);
        let tempo = Tempo::<T>::get(netuid);
        let min_difficulty = MinDifficulty::<T>::get(netuid);
        let max_difficulty = MaxDifficulty::<T>::get(netuid);
        let weights_version = WeightsVersionKey::<T>::get(netuid);
        let weights_rate_limit = WeightsSetRateLimit::<T>::get(netuid);
        let adjustment_interval = AdjustmentInterval::<T>::get(netuid);
        let activity_cutoff = ActivityCutoff::<T>::get(netuid);
        let registration_allowed = NetworkRegistrationAllowed::<T>::get(netuid);
        let target_regs_per_interval = TargetRegistrationsPerInterval::<T>::get(netuid);
        let min_burn = MinBurn::<T>::get(netuid);
        let max_burn = MaxBurn::<T>::get(netuid);
        let bonds_moving_avg = BondsMovingAverage::<T>::get(netuid);
        let max_regs_per_block = MaxRegistrationsPerBlock::<T>::get(netuid);
        let serving_rate_limit = ServingRateLimit::<T>::get(netuid);
        let max_validators = MaxAllowedValidators::<T>::get(netuid);
        let adjustment_alpha = AdjustmentAlpha::<T>::get(netuid);
        let difficulty = Difficulty::<T>::get(netuid);
        let commit_reveal_periods = RevealPeriodEpochs::<T>::get(netuid);
        let commit_reveal_weights_enabled = CommitRevealWeightsEnabled::<T>::get(netuid);
        let liquid_alpha_enabled = LiquidAlphaOn::<T>::get(netuid);
        let (alpha_low, alpha_high): (u16, u16) = AlphaValues::<T>::get(netuid);

        Some(SubnetHyperparams {
            rho: rho.into(),
            kappa: kappa.into(),
            immunity_period: immunity_period.into(),
            min_allowed_weights: min_allowed_weights.into(),
            max_weights_limit: max_weights_limit.into(),
            tempo: tempo.into(),
            min_difficulty: min_difficulty.into(),
            max_difficulty: max_difficulty.into(),
            weights_version: weights_version.into(),
            weights_rate_limit: weights_rate_limit.into(),
            adjustment_interval: adjustment_interval.into(),
            activity_cutoff: activity_cutoff.into(),
            registration_allowed,
            target_regs_per_interval: target_regs_per_interval.into(),
            min_burn: min_burn.into(),
            max_burn: max_burn.into(),
            bonds_moving_avg: bonds_moving_avg.into(),
            max_regs_per_block: max_regs_per_block.into(),
            serving_rate_limit: serving_rate_limit.into(),
            max_validators: max_validators.into(),
            adjustment_alpha: adjustment_alpha.into(),
            difficulty: difficulty.into(),
            commit_reveal_weights_interval: commit_reveal_periods.into(),
            commit_reveal_weights_enabled,
            alpha_high: alpha_high.into(),
            alpha_low: alpha_low.into(),
            liquid_alpha_enabled,
        })
    }
}
