use super::*;
use crate::{epoch::*, math::*};
use sp_std::vec;
use substrate_fixed::types::{I32F32, I64F64};

pub struct EpochInstanceSparse<T: Config> {
    pub inner: EpochInstance<T>,
    pub weights: Vec<Vec<(u16, I32F32)>>,
    pub bonds: Vec<Vec<(u16, I32F32)>>,
    pub ema_bonds: Vec<Vec<(u16, I32F32)>>,
}

impl<T: Config> InitializeEpoch for EpochInstanceSparse<T> {
    fn set_stake(&mut self, stake: Vec<I32F32>) {
        self.inner.set_stake(stake);
    }
}

impl<T: Config> CalculateEpoch for EpochInstanceSparse<T> {
    fn calc_active_inactive(&mut self) {
        self.inner.inactive_mask = self.inner.last_update
            .iter()
            .map(|updated| *updated + self.inner.activity_cutoff < self.inner.current_block)
            .collect();

        // Active is a logical negation of inactive.
        self.inner.active_mask = self.inner.inactive_mask.iter().map(|&b| !b).collect();
    }
    fn calc_validator_forbids(&mut self) {
        self.inner.calc_validator_forbids();
    }
    fn calc_active_stake(&mut self) {
        self.inner.calc_active_stake();
    }
    fn adjust_weights(&mut self) {
        // Mask weights that are not from permitted validators.
        self.weights = mask_rows_sparse(&self.inner.validator_forbids, &self.weights);

        // Remove self-weight by masking diagonal.
        self.weights = mask_diag_sparse(&self.weights);

        // Remove weights referring to deregistered neurons.
        self.weights = vec_mask_sparse_matrix(
            &self.weights,
            &self.inner.last_update,
            &self.inner.block_at_registration,
            &|updated, registered| updated <= registered,
        );

        // Normalize remaining weights.
        inplace_row_normalize_sparse(&mut self.weights);
    }
    fn calc_consensus(&mut self) {
        // Compute preranks: r_j = SUM(i) w_ij * s_i
        self.inner.preranks = matmul_sparse(&self.weights, &self.inner.active_stake, self.inner.neuron_count);

        // Clip weights at majority consensus
        // consensus majority ratio, e.g. 51%.
        self.inner.consensus = weighted_median_col_sparse(&self.inner.active_stake, &self.weights, self.inner.neuron_count, self.inner.kappa);

        self.weights = col_clip_sparse(&self.weights, &self.inner.consensus);
    }

    fn calc_ranks(&mut self) {
        self.inner.ranks = matmul_sparse(&self.weights, &self.inner.active_stake, self.inner.neuron_count);
    }

    fn calc_trust(&mut self) {
        self.inner.calc_trust();
    }

    fn calc_incentive(&mut self) {
        self.inner.calc_incentive();
    }

    fn calc_bonds_and_dividends(&mut self) {
        // Remove bonds referring to deregistered neurons.
        self.bonds = vec_mask_sparse_matrix(
            &self.bonds,
            &self.inner.last_update,
            &self.inner.block_at_registration,
            &|updated, registered| updated <= registered,
        );

        // Normalize remaining bonds: sum_i b_ij = 1.
        inplace_col_normalize_sparse(&mut self.bonds, self.inner.neuron_count);

        // Compute bonds delta column normalized (ΔB = W◦S (outdated W masked)).
        let mut bonds_delta = row_hadamard_sparse(&self.weights, &self.inner.active_stake);

        // Normalize bonds delta (sum_i b_ij = 1).
        inplace_col_normalize_sparse(&mut bonds_delta, self.inner.neuron_count);

        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num(1) - I32F32::from_num(self.inner.bonds_moving_average);
        self.ema_bonds = mat_ema_sparse(&bonds_delta, &self.bonds, alpha);

        // Normalize EMA bonds (sum_i b_ij = 1)
        inplace_col_normalize_sparse(&mut self.ema_bonds, self.inner.neuron_count);

        // Compute dividends: d_i = SUM(j) b_ij * inc_j.
        // range: I32F32(0, 1)
        self.inner.dividends = matmul_transpose_sparse(&self.ema_bonds, &self.inner.incentive);
        inplace_normalize(&mut self.inner.dividends);
    }

    fn calc_emission_and_pruning_scores(&mut self, rao_emission: u64) {
        self.inner.calc_emission_and_pruning_scores(rao_emission);
    }

    fn calc_validator_trust(&mut self) {
        // Calculate updated validator trust
        self.inner.validator_trust = row_sum_sparse(&self.weights)
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect();
    }

    fn calc_new_validator_permits(&mut self) {
        self.inner.calc_new_validator_permits();
    }

    fn adjust_ema_bonds(&mut self) {
        inplace_col_max_upscale_sparse(&mut self.ema_bonds, self.inner.neuron_count);
    }

    fn log_epoch(&self) {
        log::trace!("n: {:?}", self.inner.neuron_count);
        log::trace!("current_block: {:?}", self.inner.current_block);
        log::trace!("Last update: {:?}", &self.inner.last_update);
        log::trace!("activity_cutoff: {:?}", self.inner.activity_cutoff);
        log::trace!("Block at registration: {:?}", &self.inner.block_at_registration);
        log::trace!("Inactive: {:?}", &self.inner.inactive_mask);
        log::trace!("hotkeys: {:?}", &self.inner.hotkeys);
        log::trace!("S:\n{:?}\n", &self.inner.stake);
        log::trace!("validator_permits: {:?}", self.inner.validator_permits);
        log::trace!("max_allowed_validators: {:?}", self.inner.max_allowed_validators);
        log::trace!("S:\n{:?}\n", &self.inner.active_stake);
        // log::trace!( "R (before): {:?}", &preranks );
        log::trace!("C: {:?}", &self.inner.consensus);
        // log::trace!( "R (after): {:?}", &ranks );
        log::trace!("T: {:?}", &self.inner.trust);
        log::trace!("I (=R): {:?}", &self.inner.incentive);
        // log::trace!( "B (outdatedmask): {:?}", &bonds );
        // log::trace!( "B (mask+norm): {:?}", &bonds );
        // log::trace!( "ΔB: {:?}", &bonds_delta );
        // log::trace!( "ΔB (norm): {:?}", &bonds_delta );
        // log::trace!( "emaB: {:?}", &ema_bonds );
        log::trace!("D: {:?}", &self.inner.dividends);
        log::trace!("SE: {:?}", &self.inner.server_emission);
        log::trace!("VE: {:?}", &self.inner.validator_emission);
        log::trace!("CE: {:?}", &self.inner.combined_emission);
        log::trace!("P: {:?}", &self.inner.pruning_scores);
        log::trace!("Tv: {:?}", &self.inner.validator_trust);
        log::trace!("Tv: {:?}", &self.inner.validator_trust);
        log::trace!("new_validator_permits: {:?}", self.inner.new_validator_permits);
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
    pub fn epoch(netuid: u16, rao_emission: u64) -> Vec<(T::AccountId, u64, u64)> {
        let mut inst = Self::init_epoch_instance_sparse(netuid);

        // Perform all epoch calculations
        inst.run(rao_emission);

        // Persist and return emissions
        Self::persist_epoch_updates_sparse(netuid, inst)
    }

    pub fn init_epoch_instance_sparse(netuid: u16) -> EpochInstanceSparse<T> {
        // =================================================
        // == Initialize epoch instance with state values ==
        // =================================================

        let neuron_count = Self::get_subnetwork_n(netuid);
        let mut inst: EpochInstanceSparse<T> = EpochInstanceSparse {
            inner: EpochInstance {
                netuid: netuid,
                neuron_count: neuron_count,
                current_block: Self::get_current_block_as_u64(),
                last_update: Self::get_last_update(netuid),
                activity_cutoff: Self::get_activity_cutoff(netuid) as u64,
                block_at_registration: Self::get_block_at_registration(netuid),
                hotkeys: Keys::<T>::iter_prefix(netuid).collect(),
                validator_permits: Self::get_validator_permit(netuid),
                max_allowed_validators: Self::get_max_allowed_validators(netuid),
                kappa: Self::get_float_kappa(netuid),
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
                weights: Vec::new(),
                bonds: Vec::new(),
            },
            weights: Self::get_weights_sparse(netuid, neuron_count),
            bonds: Self::get_bonds_sparse(netuid, neuron_count),
            ema_bonds: Vec::new(),
        };
        inst.set_stake(Self::get_stakes(netuid, &inst.inner.hotkeys, neuron_count));
        inst
    }

    pub fn persist_epoch_updates_sparse(netuid: u16, inst: EpochInstanceSparse<T>) -> Vec<(T::AccountId, u64, u64)> {
        // ============================
        // == Persist in chain state ==
        // ============================

        inst.inner.new_validator_permits
            .iter()
            .zip(inst.inner.validator_permits)
            .zip(inst.ema_bonds)
            .enumerate()
            .for_each(|(i, ((new_permit, validator_permit), ema_bond))| {
                // Set bonds only if uid retains validator permit, otherwise clear bonds.
                if *new_permit {
                    let new_bonds_row: Vec<(u16, u16)> = ema_bond
                        .iter()
                        .map(|(j, value)| (*j, fixed_proportion_to_u16(*value)))
                        .collect();
                    Bonds::<T>::insert(netuid, i as u16, new_bonds_row);
                } else if validator_permit {
                    // Only overwrite the intersection.
                    let new_empty_bonds_row: Vec<(u16, u16)> = vec![];
                    Bonds::<T>::insert(netuid, i as u16, new_empty_bonds_row);
                }
            });

        Active::<T>::insert(netuid, inst.inner.active_mask);
        Emission::<T>::insert(netuid, inst.inner.combined_emission);
        Rank::<T>::insert(
            netuid,
            inst.inner.ranks
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Trust::<T>::insert(
            netuid,
            inst.inner.trust
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Consensus::<T>::insert(
            netuid,
            inst.inner.consensus
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Incentive::<T>::insert(
            netuid,
            inst.inner.incentive
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        Dividends::<T>::insert(
            netuid,
            inst.inner.dividends
                .iter()
                .map(|xi| fixed_proportion_to_u16(*xi))
                .collect::<Vec<u16>>(),
        );
        PruningScores::<T>::insert(netuid, vec_max_upscale_to_u16(&inst.inner.pruning_scores));
        ValidatorPermit::<T>::insert(netuid, inst.inner.new_validator_permits);
        ValidatorTrust::<T>::insert(netuid, inst.inner.validator_trust);

        // Return emission tuples ( hotkeys, server_emission, validator_emission )
        inst.inner.hotkeys
            .iter()
            .map(|(uid, hotkey): &(u16, T::AccountId)| {
                (
                    hotkey.clone(),
                    inst.inner.server_emission[*uid as usize],
                    inst.inner.validator_emission[*uid as usize],
                )
            })
            .collect()
    }

    // Output unnormalized sparse weights, input weights are assumed to be row max-upscaled in u16.
    #[allow(clippy::indexing_slicing)]
    pub fn get_weights_sparse(netuid: u16, neuron_count: u16) -> Vec<Vec<(u16, I32F32)>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
                .filter(|(uid_i, _)| *uid_i < n as u16)
        {
            for (uid_j, weight_ij) in weights_i.iter().filter(|(uid_j, _)| *uid_j < n as u16) {
                weights
                    .get_mut(uid_i as usize)
                    .expect("uid_i is filtered to be less than n; qed")
                    .push((*uid_j, I32F32::from_num(*weight_ij)));
            }
        }
        weights
    }

    /// Output unnormalized sparse bonds, input bonds are assumed to be column max-upscaled in u16.
    pub fn get_bonds_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
        for (uid_i, bonds_vec) in
            <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
                .filter(|(uid_i, _)| *uid_i < n as u16)
        {
            for (uid_j, bonds_ij) in bonds_vec {
                bonds
                    .get_mut(uid_i as usize)
                    .expect("uid_i is filtered to be less than n; qed")
                    .push((uid_j, I32F32::from_num(bonds_ij)));
            }
        }
        bonds
    }
}
