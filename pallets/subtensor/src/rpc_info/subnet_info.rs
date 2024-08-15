use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
use frame_support::storage::IterableStorageMap;
extern crate alloc;
use codec::Compact;

// #[freeze_struct("fe79d58173da662a")]
// #[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
// pub struct SubnetInfo<T: Config> {
//     netuid: Compact<u16>,
//     rho: Compact<u16>,
//     kappa: Compact<u16>,
//     difficulty: Compact<u64>,
//     immunity_period: Compact<u16>,
//     max_allowed_validators: Compact<u16>,
//     min_allowed_weights: Compact<u16>,
//     max_weights_limit: Compact<u16>,
//     scaling_law_power: Compact<u16>,
//     subnetwork_n: Compact<u16>,
//     max_allowed_uids: Compact<u16>,
//     blocks_since_last_step: Compact<u64>,
//     tempo: Compact<u16>,
//     network_modality: Compact<u16>,
//     network_connect: Vec<[u16; 2]>,
//     emission_values: Compact<u64>,
//     burn: Compact<u64>,
//     owner: T::AccountId,
// }

#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug)]
pub struct SubnetInfo<T: Config> {

    netuid: Compact<u16>,
    owner: T::AccountId,
    total_locked: Compact<u64>,
    owner_locked: Compact<u64>,

    subnetwork_n: Compact<u16>,
    max_allowed_uids: Compact<u16>,

    tempo: Compact<u16>,
    blocks_since_last_step: Compact<u64>,

    emission: Compact<u64>,
    tao_in: Compact<u64>,
    alpha_in: Compact<u64>,
    alpha_out: Compact<u64>,

    rho: Compact<u16>,
    kappa: Compact<u16>,

    difficulty: Compact<u64>,
    min_difficulty: Compact<u64>,
    max_difficulty: Compact<u64>,

    burn: Compact<u64>,
    min_burn: Compact<u64>,
    max_burn: Compact<u64>,
    adjustment_alpha: Compact<u64>,

    immunity_period: Compact<u16>,
    adjustment_interval: Compact<u16>,
    activity_cutoff: Compact<u16>,
    pub registration_allowed: bool,
    target_regs_per_interval: Compact<u16>,
    max_regs_per_block: Compact<u16>,
    serving_rate_limit: Compact<u64>,

    min_allowed_weights: Compact<u16>,
    max_weights_limit: Compact<u16>,
    weights_version: Compact<u64>,
    weights_rate_limit: Compact<u64>,
    max_validators: Compact<u16>,

    bonds_moving_avg: Compact<u64>,
    scaling_law_power: Compact<u16>,
    commit_reveal_weights_interval: Compact<u64>,
    commit_reveal_weights_enabled: bool,
    alpha_high: Compact<u16>,
    alpha_low: Compact<u16>,
    liquid_alpha_enabled: bool,
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
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

        let (alpha_low, alpha_high): (u16, u16) = Self::get_alpha_values(netuid);
        Some(SubnetInfo {
            netuid: netuid.into(),
            owner: Self::get_subnet_owner(netuid),
            total_locked: SubnetLocked::<T>::get(netuid).into(),
            owner_locked: LargestLocked::<T>::get(netuid).into(),

            subnetwork_n: Self::get_subnetwork_n(netuid).into(),
            max_allowed_uids: Self::get_max_allowed_uids(netuid).into(),

            tempo: Self::get_tempo(netuid).into(),
            blocks_since_last_step: Self::get_blocks_since_last_step(netuid).into(),

            emission: Self::get_emission_value(netuid).into(),
            tao_in: SubnetTAO::<T>::get(netuid).into(),
            alpha_in: SubnetAlphaIn::<T>::get(netuid).into(),
            alpha_out: SubnetAlphaOut::<T>::get(netuid).into(),

            rho: Self::get_rho(netuid).into(),
            kappa: Self::get_kappa(netuid).into(),

            difficulty: Self::get_difficulty_as_u64(netuid).into(),
            min_difficulty: Self::get_min_difficulty(netuid).into(),
            max_difficulty: Self::get_max_difficulty(netuid).into(),

            burn: Self::get_burn_as_u64(netuid).into(),
            min_burn: Self::get_min_burn_as_u64(netuid).into(),
            max_burn: Self::get_max_burn_as_u64(netuid).into(),
            adjustment_alpha: Self::get_adjustment_alpha(netuid).into(),

            immunity_period: Self::get_immunity_period(netuid).into(),
            adjustment_interval: Self::get_adjustment_interval(netuid).into(),
            activity_cutoff: Self::get_activity_cutoff(netuid).into(),
            registration_allowed: Self::get_network_registration_allowed(netuid).into(),
            target_regs_per_interval: Self::get_target_registrations_per_interval(netuid).into(),
            max_regs_per_block: Self::get_max_registrations_per_block(netuid).into(),
            serving_rate_limit: Self::get_serving_rate_limit(netuid).into(),

            min_allowed_weights: Self::get_min_allowed_weights(netuid).into(),
            max_weights_limit: Self::get_max_weight_limit(netuid).into(),
            weights_version: Self::get_weights_version_key(netuid).into(),
            weights_rate_limit: Self::get_weights_set_rate_limit(netuid).into(),
            max_validators: Self::get_max_allowed_validators(netuid).into(),

            bonds_moving_avg: Self::get_bonds_moving_average(netuid).into(),
            commit_reveal_weights_interval: Self::get_commit_reveal_weights_interval(netuid).into(),
            commit_reveal_weights_enabled: Self::get_commit_reveal_weights_enabled(netuid),
            scaling_law_power: Self::get_scaling_law_power(netuid).into(),
            alpha_high: alpha_high.into(),
            alpha_low: alpha_low.into(),
            liquid_alpha_enabled: Self::get_liquid_alpha_enabled(netuid),
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

    pub fn get_subnet_hyperparams(netuid: u16) -> Option<SubnetHyperparams> {
        if !Self::if_subnet_exist(netuid) {
            return None;
        }

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
        let liquid_alpha_enabled = Self::get_liquid_alpha_enabled(netuid);
        let (alpha_low, alpha_high): (u16, u16) = Self::get_alpha_values(netuid);

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
            commit_reveal_weights_interval: commit_reveal_weights_interval.into(),
            commit_reveal_weights_enabled,
            alpha_high: alpha_high.into(),
            alpha_low: alpha_low.into(),
            liquid_alpha_enabled,
        })
    }
}
