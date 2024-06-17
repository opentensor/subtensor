use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use codec::Compact;
use sp_std::vec::Vec;
use crate::dynamic_pool_info::DynamicPoolInfoV2;

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
    pub emission_values: Compact<u64>,
    burn: Compact<u64>,
    owner: T::AccountId,
}

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
}

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetInfoV2<T: Config> {
    netuid: Compact<u16>,
    max_allowed_validators: Compact<u16>,
    scaling_law_power: Compact<u16>,
    subnetwork_n: Compact<u16>,
    max_allowed_uids: Compact<u16>,
    blocks_since_last_step: Compact<u64>,
    network_modality: Compact<u16>,
    emission_values: Compact<u64>,
    burn: Compact<u64>,
    owner: T::AccountId,
    tao_locked: Compact<u64>,
    hyperparameters: SubnetHyperparams,
    dynamic_pool: Option<DynamicPoolInfoV2>,
}

impl<T: Config> Pallet<T> {
    pub fn get_subnet_info_v2(netuid: u16) -> Option<SubnetInfoV2<T>> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        let max_allowed_validators = Self::get_max_allowed_validators(netuid);
        let scaling_law_power = Self::get_scaling_law_power(netuid);
        let subnetwork_n = Self::get_subnetwork_n(netuid);
        let max_allowed_uids = Self::get_max_allowed_uids(netuid);
        let blocks_since_last_step = Self::get_blocks_since_last_step(netuid);
        let network_modality = <NetworkModality<T>>::get(netuid);
        let emission_values = Self::get_emission_value(netuid);
        let burn: Compact<u64> = Self::get_burn_as_u64(netuid).into();

        Some(SubnetInfoV2 {
            netuid: netuid.into(),
            max_allowed_validators: max_allowed_validators.into(),
            scaling_law_power: scaling_law_power.into(),
            subnetwork_n: subnetwork_n.into(),
            max_allowed_uids: max_allowed_uids.into(),
            blocks_since_last_step: blocks_since_last_step.into(),
            network_modality: network_modality.into(),
            emission_values: emission_values.into(),
            burn,
            owner: Self::get_subnet_owner(netuid),
            tao_locked: Self::get_total_stake_on_subnet(netuid).into(),
            hyperparameters: Self::get_subnet_hyperparams(netuid),
            dynamic_pool: Self::get_dynamic_pool_info_v2(netuid),
        })
    }

    pub fn get_subnets_info_v2() -> Vec<SubnetInfoV2<T>> {
        Self::get_all_subnet_netuids()
            .iter()
            .map(|&netuid| Self::get_subnet_info_v2(netuid))
            .filter(|info| info.is_some())
            .map(|info| info.unwrap())
            .collect()
    }

    pub fn get_subnet_hyperparams(netuid: u16) -> SubnetHyperparams {
        let rho = Self::get_rho(netuid);
        let kappa = Self::get_kappa(netuid);
        let immunity_period = Self::get_immunity_period(netuid);
        let min_allowed_weights = Self::get_min_allowed_weights(netuid);
        let max_weights_limit = Self::get_max_weight_limit(netuid);
        let tempo = Self::get_tempo(netuid);
        let min_difficulty = Self::get_min_difficulty(netuid);
        let max_difficulty = Self::get_max_difficulty(netuid);
        let weights_version = Self::get_weights_version_key(netuid);
        let weights_rate_limit = Self::get_weights_set_rate_limit(netuid);
        let adjustment_interval = Self::get_adjustment_interval(netuid);
        let activity_cutoff = Self::get_activity_cutoff(netuid);
        let registration_allowed = Self::get_network_registration_allowed(netuid);
        let target_regs_per_interval = Self::get_target_registrations_per_interval(netuid);
        let min_burn = Self::get_min_burn_as_u64(netuid);
        let max_burn = Self::get_max_burn_as_u64(netuid);
        let bonds_moving_avg = Self::get_bonds_moving_average(netuid);
        let max_regs_per_block = Self::get_max_registrations_per_block(netuid);
        let serving_rate_limit = Self::get_serving_rate_limit(netuid);
        let max_validators = Self::get_max_allowed_validators(netuid);
        let adjustment_alpha = Self::get_adjustment_alpha(netuid);
        let difficulty = Self::get_difficulty_as_u64(netuid);
        let commit_reveal_weights_interval = Self::get_commit_reveal_weights_interval(netuid);
        let commit_reveal_weights_enabled = Self::get_commit_reveal_weights_enabled(netuid);

        SubnetHyperparams {
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
            commit_reveal_weights_interval: commit_reveal_weights_interval.into(),
            commit_reveal_weights_enabled,
        }
    }

    pub fn get_subnet_limit() -> u16 {
        SubnetLimit::<T>::get()
    }
}
