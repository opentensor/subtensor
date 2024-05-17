use super::*;
use crate::math::*;
use sp_std::vec;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

#[derive(Default)]
pub struct EpochInstance<T: Config> {
    pub netuid: u16,
    pub neuron_count: u16,
    pub current_block: u64,
    pub last_update: Vec<u64>,
    pub active_mask: Vec<bool>,
    pub inactive_mask: Vec<bool>,
    // Outdated matrix, to be removed with DTAO
    pub outdated: Vec<Vec<bool>>,
    pub activity_cutoff: u64,
    // Block at registration vector (block when each neuron was most recently registered).
    pub block_at_registration: Vec<u64>,
    // (neuron_id, hotkey) vector, stored in Keys state, initially added in append_neuron
    pub hotkeys: Vec<(u16, T::AccountId)>,
    pub stake: Vec<I32F32>,
    pub active_stake: Vec<I32F32>,
    pub validator_permits: Vec<bool>,
    pub validator_forbids: Vec<bool>,
    pub max_allowed_validators: u16,
    pub weights: Vec<Vec<I32F32>>,
    pub preranks: Vec<I32F32>,
    // consensus majority ratio, e.g. 51%.
    pub kappa: I32F32,
    pub consensus: Vec<I32F32>,
    pub ranks: Vec<I32F32>,
    // Server trust: ratio of rank after vs. rank before.
    pub trust: Vec<I32F32>,
    pub incentive: Vec<I32F32>,
    pub bonds: Vec<Vec<I32F32>>,
    pub bonds_moving_average: I64F64,
    pub ema_bonds: Vec<Vec<I32F32>>,
    pub dividends: Vec<I32F32>,
    pub combined_emission: Vec<u64>,
    pub server_emission: Vec<u64>,
    pub validator_emission: Vec<u64>,
    pub pruning_scores: Vec<I32F32>,
    pub validator_trust: Vec<u16>,
    pub new_validator_permits: Vec<bool>,
}

pub trait InitializeEpoch {
    fn set_stake(&mut self, stake: Vec<I32F32>);
}

pub trait CalculateEpoch {
    // Calculate active and inactive masks.
    fn calc_active_inactive(&mut self);
    // Calculate validator forbids
    fn calc_validator_forbids(&mut self);
    fn calc_active_stake(&mut self);
    fn adjust_weights(&mut self);
    fn calc_consensus(&mut self);
    fn calc_ranks(&mut self);
    fn calc_trust(&mut self);
    fn calc_incentive(&mut self);
    fn calc_bonds_and_dividends(&mut self);
    fn calc_emission_and_pruning_scores(&mut self, rao_emission: u64);
    fn calc_validator_trust(&mut self);
    fn calc_new_validator_permits(&mut self);
    fn adjust_ema_bonds(&mut self);
    fn log_epoch(&self);
    fn run(&mut self, rao_emission: u64);
}

impl<T: Config> InitializeEpoch for EpochInstance<T> {
    fn set_stake(&mut self, stake: Vec<I32F32>) {
        self.stake = stake;
    }
}

impl<T: Config> CalculateEpoch for EpochInstance<T> {
    fn calc_active_inactive(&mut self) {
        self.inactive_mask = self
            .last_update
            .iter()
            .map(|updated| *updated + self.activity_cutoff < self.current_block)
            .collect();

        // Active is a logical negation of inactive.
        self.active_mask = self.inactive_mask.iter().map(|&b| !b).collect();

        // Outdated matrix, updated_ij=True if i has last updated (weights) after j has last registered.
        self.outdated = self.last_update
            .iter()
            .map(|updated| {
                self.block_at_registration
                    .iter()
                    .map(|registered| updated <= registered)
                    .collect()
            })
            .collect();
    }
    fn calc_validator_forbids(&mut self) {
        // Logical negation of validator_permits.
        self.validator_forbids = self.validator_permits.iter().map(|&b| !b).collect();
    }
    fn calc_active_stake(&mut self) {
        self.active_stake = self.stake.clone();

        // Remove inactive stake.
        inplace_mask_vector(&self.inactive_mask, &mut self.active_stake);

        // Remove non-validator stake.
        inplace_mask_vector(&self.validator_forbids, &mut self.active_stake);

        // Normalize active stake.
        inplace_normalize(&mut self.active_stake);
    }
    fn adjust_weights(&mut self) {
        // log::trace!("W: {:?}", &inst.weights );
        // Mask weights that are not from permitted validators.
        inplace_mask_rows(&self.validator_forbids, &mut self.weights);
        // log::trace!( "W (permit): {:?}", &weights );

        // Remove self-weight by masking diagonal.
        inplace_mask_diag(&mut self.weights);
        // log::trace!( "W (permit+diag): {:?}", &weights );

        // Mask outdated weights: remove weights referring to deregistered neurons.
        inplace_mask_matrix(&self.outdated, &mut self.weights);

        // Normalize remaining weights.
        inplace_row_normalize(&mut self.weights);
        // log::trace!( "W (mask+norm): {:?}", &weights );
    }

    fn calc_consensus(&mut self) {
        // Compute preranks: r_j = SUM(i) w_ij * s_i
        self.preranks = matmul(&self.weights, &self.active_stake);

        self.consensus = weighted_median_col(
            &self.active_stake,
            &self.weights,
            self.kappa,
        );

        // update weights
        inplace_col_clip(&mut self.weights, &self.consensus);
        // log::trace!( "W: {:?}", &weights );
    }

    fn calc_ranks(&mut self) {
        self.ranks = matmul(&self.weights, &self.active_stake);
    }

    fn calc_trust(&mut self) {
        // Compute server trust: ratio of rank after vs. rank before.
        self.trust = vecdiv(&self.ranks, &self.preranks);
        // Sets trust in range: I32F32(0, 1)
        inplace_normalize(&mut self.ranks);
    }

    fn calc_incentive(&mut self) {
        self.incentive = self.ranks.clone();
    }

    fn calc_bonds_and_dividends(&mut self) {
        // mask outdated bonds
        inplace_mask_matrix(&self.outdated, &mut self.bonds);

        // Normalize bonds delta. (sum_i b_ij = 1)
        inplace_col_normalize(&mut self.bonds);

        // Compute bonds delta column normalized (ΔB = W◦S)
        let mut bonds_delta: Vec<Vec<I32F32>> = row_hadamard(&self.weights, &self.active_stake);

        // sum_i b_ij = 1
        inplace_col_normalize(&mut bonds_delta);

        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num(1) - I32F32::from_num(self.bonds_moving_average);
        self.ema_bonds = mat_ema(&bonds_delta, &self.bonds, alpha);

        // sum_i b_ij = 1
        inplace_col_normalize(&mut self.ema_bonds);

        // Compute dividends: d_i = SUM(j) b_ij * inc_j
        self.dividends = matmul_transpose(&self.ema_bonds, &self.incentive);
        inplace_normalize(&mut self.dividends);
    }

    fn calc_emission_and_pruning_scores(&mut self, rao_emission: u64) {
        // Compute normalized emission scores. range: I32F32(0, 1)
        let combined_emission_i32f32: Vec<I32F32> = self
            .incentive
            .iter()
            .zip(self.dividends.clone())
            .map(|(ii, di)| ii + di)
            .collect();
        let emission_sum: I32F32 = combined_emission_i32f32.iter().sum();

        let mut normalized_server_emission: Vec<I32F32> = self.incentive.clone(); // Servers get incentive.
        let mut normalized_validator_emission: Vec<I32F32> = self.dividends.clone(); // Validators get dividends.
        let mut normalized_combined_emission: Vec<I32F32> = combined_emission_i32f32.clone();

        // Normalize on the sum of incentive + dividends.
        inplace_normalize_using_sum(&mut normalized_server_emission, emission_sum);
        inplace_normalize_using_sum(&mut normalized_validator_emission, emission_sum);
        inplace_normalize(&mut normalized_combined_emission);

        // If emission is zero, replace emission with normalized stake.
        if emission_sum == I32F32::from(0) {
            // no weights set | outdated weights | self_weights
            if is_zero(&self.active_stake) {
                // no active stake
                normalized_validator_emission = self.stake.clone(); // do not mask inactive, assumes stake is normalized
                normalized_combined_emission = self.stake.clone();
            } else {
                normalized_validator_emission = self.active_stake.clone(); // emission proportional to inactive-masked normalized stake
                normalized_combined_emission = self.active_stake.clone();
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num(rao_emission);

        self.server_emission = normalized_server_emission
            .iter()
            .map(|se: &I32F32| (I96F32::from_num(*se) * float_rao_emission).to_num::<u64>())
            .collect();
        self.validator_emission = normalized_validator_emission
            .iter()
            .map(|ve: &I32F32| (I96F32::from_num(*ve) * float_rao_emission).to_num::<u64>())
            .collect();

        // Only used to track emission in storage.
        self.combined_emission = normalized_combined_emission
            .iter()
            .map(|ce: &I32F32| (I96F32::from_num(*ce) * float_rao_emission).to_num::<u64>())
            .collect();

        // Set pruning scores using combined emission scores.
        self.pruning_scores = normalized_combined_emission.clone();

        log::trace!("nSE: {:?}", &normalized_server_emission);
        log::trace!("nVE: {:?}", &normalized_validator_emission);
        log::trace!("nCE: {:?}", &normalized_combined_emission);
    }

    fn calc_validator_trust(&mut self) {
        // Calculate updated validator trust
        self.validator_trust = row_sum(&self.weights)
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect();
    }

    fn calc_new_validator_permits(&mut self) {
        // Column max-upscale EMA bonds for storage: max_i w_ij = 1.
        // Get new validator permits.
        self.new_validator_permits = is_topk(&self.stake, self.max_allowed_validators as usize);
    }

    fn adjust_ema_bonds(&mut self) {
        inplace_col_max_upscale(&mut self.ema_bonds);
    }

    fn log_epoch(&self) {
        log::trace!("n: {:?}", self.neuron_count);
        log::trace!("current_block: {:?}", self.current_block);
        log::trace!("Last update: {:?}", &self.last_update);
        log::trace!("activity_cutoff: {:?}", self.activity_cutoff);
        log::trace!("Block at registration: {:?}", &self.block_at_registration);
        log::trace!("Inactive: {:?}", &self.inactive_mask);
        log::trace!("hotkeys: {:?}", &self.hotkeys);
        log::trace!("S:\n{:?}\n", &self.stake);
        log::trace!("validator_permits: {:?}", self.validator_permits);
        log::trace!("max_allowed_validators: {:?}", self.max_allowed_validators);
        log::trace!("S:\n{:?}\n", &self.active_stake);
        // log::trace!( "R (before): {:?}", &preranks );
        log::trace!("C: {:?}", &self.consensus);
        // log::trace!( "R (after): {:?}", &ranks );
        log::trace!("T: {:?}", &self.trust);
        log::trace!("I (=R): {:?}", &self.incentive);
        // log::trace!( "B (outdatedmask): {:?}", &bonds );
        // log::trace!( "B (mask+norm): {:?}", &bonds );
        // log::trace!( "ΔB: {:?}", &bonds_delta );
        // log::trace!( "ΔB (norm): {:?}", &bonds_delta );
        // log::trace!( "emaB: {:?}", &ema_bonds );
        log::trace!("D: {:?}", &self.dividends);
        log::trace!("SE: {:?}", &self.server_emission);
        log::trace!("VE: {:?}", &self.validator_emission);
        log::trace!("CE: {:?}", &self.combined_emission);
        log::trace!("P: {:?}", &self.pruning_scores);
        log::trace!("Tv: {:?}", &self.validator_trust);
        log::trace!("Tv: {:?}", &self.validator_trust);
        log::trace!("new_validator_permits: {:?}", self.new_validator_permits);
    }

    fn run(&mut self, rao_emission: u64) {
        // Perform all epoch calculations
        self.calc_active_inactive();
        self.calc_validator_forbids();
        self.calc_active_stake();
        self.adjust_weights();
        self.calc_consensus();
        self.calc_ranks();
        self.calc_trust();
        self.calc_incentive();
        self.calc_bonds_and_dividends();
        self.calc_emission_and_pruning_scores(rao_emission);
        self.calc_validator_trust();
        self.calc_new_validator_permits();
        self.adjust_ema_bonds();
        self.log_epoch();
    }
}

impl<T: Config> Pallet<T> {
    /// Calculates reward consensus values, then updates rank, trust, consensus, incentive, dividend, pruning_score, emission and bonds, and
    /// returns the emissions for uids/hotkeys in a given `netuid`.
    /// (Dense version used only for testing purposes.)
    ///
    /// # Args:
    ///  * 'netuid': ( u16 ):
    ///     - The network to distribute the emission onto.
    ///
    ///  * 'rao_emission': ( u64 ):
    ///     - The total emission for the epoch.
    ///
    ///  * 'debug' ( bool ):
    ///     - Print debugging outputs.
    ///
    #[allow(clippy::indexing_slicing)]
    pub fn epoch_dense(netuid: u16, rao_emission: u64) -> Vec<(T::AccountId, u64, u64)> {
        let mut inst = Self::init_epoch_instance(netuid);

        // Perform all epoch calculations
        inst.run(rao_emission); 

        // Persist and return emissions
        Self::persist_epoch_updates(netuid, inst)
    }

    pub fn init_epoch_instance(netuid: u16) -> EpochInstance<T> {
        // =================================================
        // == Initialize epoch instance with state values ==
        // =================================================

        let neuron_count = Self::get_subnetwork_n(netuid);
        let mut inst: EpochInstance<T> = EpochInstance {
            netuid: netuid,
            neuron_count: neuron_count,
            current_block: Self::get_current_block_as_u64(),
            last_update: Self::get_last_update(netuid),
            activity_cutoff: Self::get_activity_cutoff(netuid) as u64,
            block_at_registration: Self::get_block_at_registration(netuid),
            hotkeys: Keys::<T>::iter_prefix(netuid).collect(),
            validator_permits: Self::get_validator_permit(netuid),
            max_allowed_validators: Self::get_max_allowed_validators(netuid),
            weights: Self::get_weights(netuid),
            kappa: Self::get_float_kappa(netuid),
            bonds: Self::get_bonds(netuid),
            bonds_moving_average: I64F64::from_num(Self::get_bonds_moving_average(netuid)) 
                / I64F64::from_num(1_000_000),

            active_mask: Vec::new(),
            inactive_mask: Vec::new(),
            outdated: Vec::new(),
            stake: Vec::new(),
            active_stake: Vec::new(),
            validator_forbids: Vec::new(),
            preranks: Vec::new(),
            consensus: Vec::new(),
            ranks: Vec::new(),
            trust: Vec::new(),
            incentive: Vec::new(),
            ema_bonds: Vec::new(),
            dividends: Vec::new(),
            combined_emission: Vec::new(),
            server_emission: Vec::new(),
            validator_emission: Vec::new(),
            pruning_scores: Vec::new(),
            validator_trust: Vec::new(),
            new_validator_permits: Vec::new(),
        };
        inst.set_stake(Self::get_stakes(netuid, &inst.hotkeys, neuron_count));
        inst
    }

    pub fn persist_epoch_updates(netuid: u16, inst: EpochInstance<T>) -> Vec<(T::AccountId, u64, u64)> {
        // ============================
        // == Persist in chain state ==
        // ============================

        inst.new_validator_permits
            .iter()
            .zip(inst.validator_permits)
            .zip(inst.ema_bonds)
            .enumerate()
            .for_each(|(i, ((new_permit, validator_permit), ema_bond))| {
                // Set bonds only if uid retains validator permit, otherwise clear bonds.
                if *new_permit {
                    let new_bonds_row: Vec<(u16, u16)> = (0..inst.neuron_count)
                        .zip(vec_fixed_proportions_to_u16(ema_bond.clone()))
                        .collect();
                    Bonds::<T>::insert(netuid, i as u16, new_bonds_row);
                } else if validator_permit {
                    // Only overwrite the intersection.
                    let new_empty_bonds_row: Vec<(u16, u16)> = vec![];
                    Bonds::<T>::insert(netuid, i as u16, new_empty_bonds_row);
                }
            });

        Active::<T>::insert(netuid, inst.active_mask);
        Emission::<T>::insert(netuid, inst.combined_emission);
        Rank::<T>::insert(
            netuid,
            inst.ranks
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Trust::<T>::insert(
            netuid,
            inst.trust
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Consensus::<T>::insert(
            netuid,
            inst.consensus
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Incentive::<T>::insert(
            netuid,
            inst.incentive
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Dividends::<T>::insert(
            netuid,
            inst.dividends
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        PruningScores::<T>::insert(netuid, vec_max_upscale_to_u16(&inst.pruning_scores));
        ValidatorPermit::<T>::insert(netuid, inst.new_validator_permits);
        ValidatorTrust::<T>::insert(netuid, inst.validator_trust);

        // Return emission tuples ( hotkeys, server_emission, validator_emission )
        inst.hotkeys
            .iter()
            .map(|(uid, hotkey): &(u16, T::AccountId)| {
                (
                    hotkey.clone(),
                    inst.server_emission[*uid as usize],
                    inst.validator_emission[*uid as usize],
                )
            })
            .collect()
    }

    pub fn get_stakes(_netuid: u16, hotkeys: &Vec<(u16, T::AccountId)>, neuron_count: u16) -> Vec<I32F32> {
        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); neuron_count as usize];
        hotkeys.iter().for_each(|(uid_i, hotkey)| {
            stake_64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
        });
        inplace_normalize_64(&mut stake_64);
        vec_fixed64_to_fixed32(stake_64)
    }

    pub fn get_float_rho(netuid: u16) -> I32F32 {
        I32F32::from_num(Self::get_rho(netuid))
    }
    pub fn get_float_kappa(netuid: u16) -> I32F32 {
        I32F32::from_num(Self::get_kappa(netuid)) / I32F32::from_num(u16::MAX)
    }

    pub fn get_normalized_stake(netuid: u16) -> Vec<I32F32> {
        let n = Self::get_subnetwork_n(netuid);
        let mut stake_64: Vec<I64F64> = (0..n)
            .map(|neuron_uid| {
                I64F64::from_num(Self::get_stake_for_uid_and_subnetwork(netuid, neuron_uid))
            })
            .collect();
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        stake
    }

    pub fn get_block_at_registration(netuid: u16) -> Vec<u64> {
        let n = Self::get_subnetwork_n(netuid);
        let block_at_registration: Vec<u64> = (0..n)
            .map(|neuron_uid| {
                if Keys::<T>::contains_key(netuid, neuron_uid) {
                    Self::get_neuron_block_at_registration(netuid, neuron_uid)
                } else {
                    0
                }
            })
            .collect();
        block_at_registration
    }

    // // Output unnormalized sparse weights, input weights are assumed to be row max-upscaled in u16.
    // #[allow(clippy::indexing_slicing)]
    // pub fn get_weights_sparse(netuid: u16, neuron_count: u16) -> Vec<Vec<(u16, I32F32)>> {
    //     let mut weights: Vec<Vec<(u16, I32F32)>> = vec![vec![]; neuron_count as usize];
    //     Weights::<T>::iter_prefix(netuid)
    //         .filter(|(uid_i, _)| *uid_i < neuron_count as u16)
    //         .for_each(|(uid_i, weights_i)| {
    //             weights[uid_i as usize] = weights_i
    //                 .iter()
    //                 .filter(|(uid_j, _)| *uid_j < neuron_count)
    //                 .map(|(uid_j, weight_ij)| (*uid_j, I32F32::from_num(*weight_ij)))
    //                 .collect();
    //         });
    //     weights
    // }

    /// Output unnormalized weights in [n, n] matrix, input weights are assumed to be row max-upscaled in u16.
    #[allow(clippy::indexing_slicing)]
    pub fn get_weights(netuid: u16) -> Vec<Vec<I32F32>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut weights: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.0); n]; n];
        Weights::<T>::iter_prefix(netuid).for_each(|(uid_i, weights_i)| {
            weights_i.iter().for_each(|(uid_j, weight_ij)| {
                weights[uid_i as usize][*uid_j as usize] = I32F32::from_num(*weight_ij);
            });
        });
        weights
    }

    // /// Output unnormalized sparse bonds, input bonds are assumed to be column max-upscaled in u16.
    // #[allow(clippy::indexing_slicing)]
    // pub fn get_bonds_sparse(netuid: u16, neuron_count: u16) -> Vec<Vec<(u16, I32F32)>> {
    //     let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![vec![]; neuron_count as usize];
    //     Bonds::<T>::iter_prefix(netuid).for_each(|(uid_i, bonds_i)| {
    //         bonds[uid_i as usize] = bonds_i
    //             .iter()
    //             .map(|(uid_j, bonds_ij)| (*uid_j, I32F32::from_num(*bonds_ij)))
    //             .collect();
    //     });
    //     bonds
    // }

    /// Output unnormalized bonds in [n, n] matrix, input bonds are assumed to be column max-upscaled in u16.
    #[allow(clippy::indexing_slicing)]
    pub fn get_bonds(netuid: u16) -> Vec<Vec<I32F32>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut bonds: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.0); n]; n];
        Bonds::<T>::iter_prefix(netuid).for_each(|(uid_i, bonds_i)| {
            bonds_i.iter().for_each(|(uid_j, bonds_ij)| {
                bonds[uid_i as usize][*uid_j as usize] = I32F32::from_num(*bonds_ij);
            });
        });
        bonds
    }
}
