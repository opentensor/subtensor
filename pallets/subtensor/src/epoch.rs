use super::*;
use crate::math::*;
use frame_support::sp_std::vec;
use frame_support::storage::IterableStorageDoubleMap;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

impl<T: Config> Pallet<T> {
    // Calculates reward consensus and returns the emissions for uids/hotkeys in a given `netuid`.
    // (Dense version used only for testing purposes.)
    pub fn epoch_dense(netuid: u16, rao_emission: u64) -> Vec<(T::AccountId, u64, u64)> {
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n(netuid);
        log::trace!("n:\n{:?}\n", n);

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::trace!("current_block:\n{:?}\n", current_block);

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff(netuid) as u64;
        log::trace!("activity_cutoff:\n{:?}\n", activity_cutoff);

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update(netuid);
        log::trace!("Last update:\n{:?}\n", &last_update);

        // Inactive mask.
        let inactive: Vec<bool> = last_update
            .iter()
            .map(|updated| *updated + activity_cutoff < current_block)
            .collect();
        log::trace!("Inactive:\n{:?}\n", inactive.clone());

        // Logical negation of inactive.
        let active: Vec<bool> = inactive.iter().map(|&b| !b).collect();

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration(netuid);
        log::trace!("Block at registration:\n{:?}\n", &block_at_registration);

        // Outdated matrix, updated_ij=True if i has last updated (weights) after j has last registered.
        let outdated: Vec<Vec<bool>> = last_update
            .iter()
            .map(|updated| {
                block_at_registration
                    .iter()
                    .map(|registered| updated <= registered)
                    .collect()
            })
            .collect();
        log::trace!("Outdated:\n{:?}\n", &outdated);

        // ===========
        // == Stake ==
        // ===========

        let mut hotkeys: Vec<(u16, T::AccountId)> = vec![];
        for (uid_i, hotkey) in
            <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid)
        {
            hotkeys.push((uid_i, hotkey));
        }
        log::trace!("hotkeys: {:?}", &hotkeys);

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        for (uid_i, hotkey) in hotkeys.iter() {
            stake_64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
        }
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        log::trace!("S:\n{:?}\n", &stake);

        // =======================
        // == Validator permits ==
        // =======================

        // Get validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permit(netuid);
        log::trace!("validator_permits: {:?}", validator_permits);

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators(netuid);
        log::trace!("max_allowed_validators: {:?}", max_allowed_validators);

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk(&stake, max_allowed_validators as usize);
        log::trace!("new_validator_permits: {:?}", new_validator_permits);

        // ==================
        // == Active Stake ==
        // ==================

        let mut active_stake: Vec<I32F32> = stake.clone();

        // Remove inactive stake.
        inplace_mask_vector(&inactive, &mut active_stake);

        // Remove non-validator stake.
        inplace_mask_vector(&validator_forbids, &mut active_stake);

        // Normalize active stake.
        inplace_normalize(&mut active_stake);
        log::trace!("S:\n{:?}\n", &active_stake);

        // =============
        // == Weights ==
        // =============

        // Access network weights row unnormalized.
        let mut weights: Vec<Vec<I32F32>> = Self::get_weights(netuid);
        log::trace!("W:\n{:?}\n", &weights);

        // Mask weights that are not from permitted validators.
        inplace_mask_rows(&validator_forbids, &mut weights);
        // log::trace!( "W (permit): {:?}", &weights );

        // Remove self-weight by masking diagonal.
        inplace_mask_diag(&mut weights);
        // log::trace!( "W (permit+diag):\n{:?}\n", &weights );

        // Mask outdated weights: remove weights referring to deregistered neurons.
        inplace_mask_matrix(&outdated, &mut weights);
        // log::trace!( "W (permit+diag+outdate):\n{:?}\n", &weights );

        // Normalize remaining weights.
        inplace_row_normalize(&mut weights);
        // log::trace!( "W (mask+norm):\n{:?}\n", &weights );

        // ================================
        // == Consensus, Validator Trust ==
        // ================================

        // Compute preranks: r_j = SUM(i) w_ij * s_i
        let preranks: Vec<I32F32> = matmul(&weights, &active_stake);

        // Clip weights at majority consensus
        let kappa: I32F32 = Self::get_float_kappa(netuid); // consensus majority ratio, e.g. 51%.
        let consensus: Vec<I32F32> = weighted_median_col(&active_stake, &weights, kappa);
        inplace_col_clip(&mut weights, &consensus);
        let validator_trust: Vec<I32F32> = row_sum(&weights);

        // ====================================
        // == Ranks, Server Trust, Incentive ==
        // ====================================

        // Compute ranks: r_j = SUM(i) w_ij * s_i
        let mut ranks: Vec<I32F32> = matmul(&weights, &active_stake);

        // Compute server trust: ratio of rank after vs. rank before.
        let trust: Vec<I32F32> = vecdiv(&ranks, &preranks);

        inplace_normalize(&mut ranks);
        let incentive: Vec<I32F32> = ranks.clone();
        log::trace!("I:\n{:?}\n", &incentive);

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds.
        let mut bonds: Vec<Vec<I32F32>> = Self::get_bonds(netuid);
        inplace_mask_matrix(&outdated, &mut bonds); // mask outdated bonds
        inplace_col_normalize(&mut bonds); // sum_i b_ij = 1
                                           // log::trace!( "B:\n{:?}\n", &bonds );

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<I32F32>> = row_hadamard(&weights, &active_stake); // ΔB = W◦S
        inplace_col_normalize(&mut bonds_delta); // sum_i b_ij = 1
                                                 // log::trace!( "ΔB:\n{:?}\n", &bonds_delta );

        // Compute bonds moving average.
        let bonds_moving_average: I64F64 =
            I64F64::from_num(Self::get_bonds_moving_average(netuid)) / I64F64::from_num(1_000_000);
        let alpha: I32F32 = I32F32::from_num(1) - I32F32::from_num(bonds_moving_average);
        let mut ema_bonds: Vec<Vec<I32F32>> = mat_ema(&bonds_delta, &bonds, alpha);
        inplace_col_normalize(&mut ema_bonds); // sum_i b_ij = 1
                                               // log::trace!( "emaB:\n{:?}\n", &ema_bonds );

        // Compute dividends: d_i = SUM(j) b_ij * inc_j
        let mut dividends: Vec<I32F32> = matmul_transpose(&ema_bonds, &incentive);
        inplace_normalize(&mut dividends);
        log::trace!("D:\n{:?}\n", &dividends);

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute emission scores.

        // Compute normalized emission scores. range: I32F32(0, 1)
        // Compute normalized emission scores. range: I32F32(0, 1)
        let combined_emission: Vec<I32F32> = incentive
            .iter()
            .zip(dividends.clone())
            .map(|(ii, di)| ii + di)
            .collect();
        let emission_sum: I32F32 = combined_emission.iter().sum();

        let mut normalized_server_emission: Vec<I32F32> = incentive.clone(); // Servers get incentive.
        let mut normalized_validator_emission: Vec<I32F32> = dividends.clone(); // Validators get dividends.
        let mut normalized_combined_emission: Vec<I32F32> = combined_emission.clone();
        // Normalize on the sum of incentive + dividends.
        inplace_normalize_using_sum(&mut normalized_server_emission, emission_sum);
        inplace_normalize_using_sum(&mut normalized_validator_emission, emission_sum);
        inplace_normalize(&mut normalized_combined_emission);

        // If emission is zero, replace emission with normalized stake.
        if emission_sum == I32F32::from(0) {
            // no weights set | outdated weights | self_weights
            if is_zero(&active_stake) {
                // no active stake
                normalized_validator_emission = stake.clone(); // do not mask inactive, assumes stake is normalized
                normalized_combined_emission = stake.clone();
            } else {
                normalized_validator_emission = active_stake.clone(); // emission proportional to inactive-masked normalized stake
                normalized_combined_emission = active_stake.clone();
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num(rao_emission);

        let server_emission: Vec<I96F32> = normalized_server_emission
            .iter()
            .map(|se: &I32F32| I96F32::from_num(*se) * float_rao_emission)
            .collect();
        let server_emission: Vec<u64> = server_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        let validator_emission: Vec<I96F32> = normalized_validator_emission
            .iter()
            .map(|ve: &I32F32| I96F32::from_num(*ve) * float_rao_emission)
            .collect();
        let validator_emission: Vec<u64> = validator_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        // Used only to track combined emission in the storage.
        let combined_emission: Vec<I96F32> = normalized_combined_emission
            .iter()
            .map(|ce: &I32F32| I96F32::from_num(*ce) * float_rao_emission)
            .collect();
        let combined_emission: Vec<u64> = combined_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        log::trace!("nSE: {:?}", &normalized_server_emission);
        log::trace!("SE: {:?}", &server_emission);
        log::trace!("nVE: {:?}", &normalized_validator_emission);
        log::trace!("VE: {:?}", &validator_emission);
        log::trace!("nCE: {:?}", &normalized_combined_emission);
        log::trace!("CE: {:?}", &combined_emission);

        // Set pruning scores using combined emission scores.
        let pruning_scores: Vec<I32F32> = normalized_combined_emission.clone();
        log::trace!("P: {:?}", &pruning_scores);

        // ===================
        // == Value storage ==
        // ===================
        let cloned_emission: Vec<u64> = combined_emission.clone();
        let cloned_ranks: Vec<u16> = ranks
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_trust: Vec<u16> = trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_consensus: Vec<u16> = consensus
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_incentive: Vec<u16> = incentive
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_dividends: Vec<u16> = dividends
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_pruning_scores: Vec<u16> = vec_max_upscale_to_u16(&pruning_scores);
        let cloned_validator_trust: Vec<u16> = validator_trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        Active::<T>::insert(netuid, active.clone());
        Emission::<T>::insert(netuid, cloned_emission);
        Rank::<T>::insert(netuid, cloned_ranks);
        Trust::<T>::insert(netuid, cloned_trust);
        Consensus::<T>::insert(netuid, cloned_consensus);
        Incentive::<T>::insert(netuid, cloned_incentive);
        Dividends::<T>::insert(netuid, cloned_dividends);
        PruningScores::<T>::insert(netuid, cloned_pruning_scores);
        ValidatorTrust::<T>::insert(netuid, cloned_validator_trust);
        ValidatorPermit::<T>::insert(netuid, new_validator_permits.clone());

        // Column max-upscale EMA bonds for storage: max_i w_ij = 1.
        inplace_col_max_upscale(&mut ema_bonds);
        for i in 0..n {
            // Set bonds only if uid retains validator permit, otherwise clear bonds.
            if new_validator_permits[i as usize] {
                let new_bonds_row: Vec<(u16, u16)> = (0..n)
                    .zip(vec_fixed_proportions_to_u16(ema_bonds[i as usize].clone()))
                    .collect();
                Bonds::<T>::insert(netuid, i, new_bonds_row);
            } else if validator_permits[i as usize] {
                // Only overwrite the intersection.
                let new_empty_bonds_row: Vec<(u16, u16)> = vec![];
                Bonds::<T>::insert(netuid, i, new_empty_bonds_row);
            }
        }

        let mut result: Vec<(T::AccountId, u64, u64)> = vec![];
        for (uid_i, hotkey) in hotkeys.iter() {
            result.push((
                hotkey.clone(),
                server_emission[*uid_i as usize],
                validator_emission[*uid_i as usize],
            ));
        }
        result
    }

    // Calculates reward consensus values, then updates rank, trust, consensus, incentive, dividend, pruning_score, emission and bonds, and
    // returns the emissions for uids/hotkeys in a given `netuid`.
    //
    // # Args:
    // 	* 'netuid': ( u16 ):
    //         - The network to distribute the emission onto.
    //
    // 	* 'rao_emission': ( u64 ):
    //         - The total emission for the epoch.
    //
    // 	* 'debug' ( bool ):
    // 		- Print debugging outputs.
    //
    pub fn epoch(netuid: u16, rao_emission: u64) -> Vec<(T::AccountId, u64, u64)> {
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n(netuid);
        log::trace!("n: {:?}", n);

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::trace!("current_block: {:?}", current_block);

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff(netuid) as u64;
        log::trace!("activity_cutoff: {:?}", activity_cutoff);

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update(netuid);
        log::trace!("Last update: {:?}", &last_update);

        // Inactive mask.
        let inactive: Vec<bool> = last_update
            .iter()
            .map(|updated| *updated + activity_cutoff < current_block)
            .collect();
        log::trace!("Inactive: {:?}", inactive.clone());

        // Logical negation of inactive.
        let active: Vec<bool> = inactive.iter().map(|&b| !b).collect();

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration(netuid);
        log::trace!("Block at registration: {:?}", &block_at_registration);

        // ===========
        // == Stake ==
        // ===========

        let mut hotkeys: Vec<(u16, T::AccountId)> = vec![];
        for (uid_i, hotkey) in
            <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid)
        {
            hotkeys.push((uid_i, hotkey));
        }
        log::trace!("hotkeys: {:?}", &hotkeys);

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        for (uid_i, hotkey) in hotkeys.iter() {
            stake_64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
        }
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        // range: I32F32(0, 1)
        log::trace!("S: {:?}", &stake);

        // =======================
        // == Validator permits ==
        // =======================

        // Get current validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permit(netuid);
        log::trace!("validator_permits: {:?}", validator_permits);

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators(netuid);
        log::trace!("max_allowed_validators: {:?}", max_allowed_validators);

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk(&stake, max_allowed_validators as usize);
        log::trace!("new_validator_permits: {:?}", new_validator_permits);

        // ==================
        // == Active Stake ==
        // ==================

        let mut active_stake: Vec<I32F32> = stake.clone();

        // Remove inactive stake.
        inplace_mask_vector(&inactive, &mut active_stake);

        // Remove non-validator stake.
        inplace_mask_vector(&validator_forbids, &mut active_stake);

        // Normalize active stake.
        inplace_normalize(&mut active_stake);
        log::trace!("S:\n{:?}\n", &active_stake);

        // =============
        // == Weights ==
        // =============

        // Access network weights row unnormalized.
        let mut weights: Vec<Vec<(u16, I32F32)>> = Self::get_weights_sparse(netuid);
        // log::trace!( "W: {:?}", &weights );

        // Mask weights that are not from permitted validators.
        weights = mask_rows_sparse(&validator_forbids, &weights);
        // log::trace!( "W (permit): {:?}", &weights );

        // Remove self-weight by masking diagonal.
        weights = mask_diag_sparse(&weights);
        // log::trace!( "W (permit+diag): {:?}", &weights );

        // Remove weights referring to deregistered neurons.
        weights = vec_mask_sparse_matrix(
            &weights,
            &last_update,
            &block_at_registration,
            &|updated, registered| updated <= registered,
        );
        // log::trace!( "W (permit+diag+outdate): {:?}", &weights );

        // Normalize remaining weights.
        inplace_row_normalize_sparse(&mut weights);
        // log::trace!( "W (mask+norm): {:?}", &weights );

        // ================================
        // == Consensus, Validator Trust ==
        // ================================

        // Compute preranks: r_j = SUM(i) w_ij * s_i
        let preranks: Vec<I32F32> = matmul_sparse(&weights, &active_stake, n);
        // log::trace!( "R (before): {:?}", &preranks );

        // Clip weights at majority consensus
        let kappa: I32F32 = Self::get_float_kappa(netuid); // consensus majority ratio, e.g. 51%.
        let consensus: Vec<I32F32> = weighted_median_col_sparse(&active_stake, &weights, n, kappa);
        log::trace!("C: {:?}", &consensus);

        weights = col_clip_sparse(&weights, &consensus);
        // log::trace!( "W: {:?}", &weights );

        let validator_trust: Vec<I32F32> = row_sum_sparse(&weights);
        log::trace!("Tv: {:?}", &validator_trust);

        // =============================
        // == Ranks, Trust, Incentive ==
        // =============================

        // Compute ranks: r_j = SUM(i) w_ij * s_i.
        let mut ranks: Vec<I32F32> = matmul_sparse(&weights, &active_stake, n);
        // log::trace!( "R (after): {:?}", &ranks );

        // Compute server trust: ratio of rank after vs. rank before.
        let trust: Vec<I32F32> = vecdiv(&ranks, &preranks); // range: I32F32(0, 1)
        log::trace!("T: {:?}", &trust);

        inplace_normalize(&mut ranks); // range: I32F32(0, 1)
        let incentive: Vec<I32F32> = ranks.clone();
        log::trace!("I (=R): {:?}", &incentive);

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds.
        let mut bonds: Vec<Vec<(u16, I32F32)>> = Self::get_bonds_sparse(netuid);
        // log::trace!( "B: {:?}", &bonds );

        // Remove bonds referring to deregistered neurons.
        bonds = vec_mask_sparse_matrix(
            &bonds,
            &last_update,
            &block_at_registration,
            &|updated, registered| updated <= registered,
        );
        // log::trace!( "B (outdatedmask): {:?}", &bonds );

        // Normalize remaining bonds: sum_i b_ij = 1.
        inplace_col_normalize_sparse(&mut bonds, n);
        // log::trace!( "B (mask+norm): {:?}", &bonds );

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<(u16, I32F32)>> = row_hadamard_sparse(&weights, &active_stake); // ΔB = W◦S (outdated W masked)
                                                                                                     // log::trace!( "ΔB: {:?}", &bonds_delta );

        // Normalize bonds delta.
        inplace_col_normalize_sparse(&mut bonds_delta, n); // sum_i b_ij = 1
                                                           // log::trace!( "ΔB (norm): {:?}", &bonds_delta );

        // Compute bonds moving average.
        let bonds_moving_average: I64F64 =
            I64F64::from_num(Self::get_bonds_moving_average(netuid)) / I64F64::from_num(1_000_000);
        let alpha: I32F32 = I32F32::from_num(1) - I32F32::from_num(bonds_moving_average);
        let mut ema_bonds: Vec<Vec<(u16, I32F32)>> = mat_ema_sparse(&bonds_delta, &bonds, alpha);

        // Normalize EMA bonds.
        inplace_col_normalize_sparse(&mut ema_bonds, n); // sum_i b_ij = 1
                                                         // log::trace!( "emaB: {:?}", &ema_bonds );

        // Compute dividends: d_i = SUM(j) b_ij * inc_j.
        // range: I32F32(0, 1)
        let mut dividends: Vec<I32F32> = matmul_transpose_sparse(&ema_bonds, &incentive);
        inplace_normalize(&mut dividends);
        log::trace!("D: {:?}", &dividends);

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute normalized emission scores. range: I32F32(0, 1)
        let combined_emission: Vec<I32F32> = incentive
            .iter()
            .zip(dividends.clone())
            .map(|(ii, di)| ii + di)
            .collect();
        let emission_sum: I32F32 = combined_emission.iter().sum();

        let mut normalized_server_emission: Vec<I32F32> = incentive.clone(); // Servers get incentive.
        let mut normalized_validator_emission: Vec<I32F32> = dividends.clone(); // Validators get dividends.
        let mut normalized_combined_emission: Vec<I32F32> = combined_emission.clone();
        // Normalize on the sum of incentive + dividends.
        inplace_normalize_using_sum(&mut normalized_server_emission, emission_sum);
        inplace_normalize_using_sum(&mut normalized_validator_emission, emission_sum);
        inplace_normalize(&mut normalized_combined_emission);

        // If emission is zero, replace emission with normalized stake.
        if emission_sum == I32F32::from(0) {
            // no weights set | outdated weights | self_weights
            if is_zero(&active_stake) {
                // no active stake
                normalized_validator_emission = stake.clone(); // do not mask inactive, assumes stake is normalized
                normalized_combined_emission = stake.clone();
            } else {
                normalized_validator_emission = active_stake.clone(); // emission proportional to inactive-masked normalized stake
                normalized_combined_emission = active_stake.clone();
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num(rao_emission);

        let server_emission: Vec<I96F32> = normalized_server_emission
            .iter()
            .map(|se: &I32F32| I96F32::from_num(*se) * float_rao_emission)
            .collect();
        let server_emission: Vec<u64> = server_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        let validator_emission: Vec<I96F32> = normalized_validator_emission
            .iter()
            .map(|ve: &I32F32| I96F32::from_num(*ve) * float_rao_emission)
            .collect();
        let validator_emission: Vec<u64> = validator_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        // Only used to track emission in storage.
        let combined_emission: Vec<I96F32> = normalized_combined_emission
            .iter()
            .map(|ce: &I32F32| I96F32::from_num(*ce) * float_rao_emission)
            .collect();
        let combined_emission: Vec<u64> = combined_emission
            .iter()
            .map(|e: &I96F32| e.to_num::<u64>())
            .collect();

        log::trace!("nSE: {:?}", &normalized_server_emission);
        log::trace!("SE: {:?}", &server_emission);
        log::trace!("nVE: {:?}", &normalized_validator_emission);
        log::trace!("VE: {:?}", &validator_emission);
        log::trace!("nCE: {:?}", &normalized_combined_emission);
        log::trace!("CE: {:?}", &combined_emission);

        // Set pruning scores using combined emission scores.
        let pruning_scores: Vec<I32F32> = normalized_combined_emission.clone();
        log::trace!("P: {:?}", &pruning_scores);

        // ===================
        // == Value storage ==
        // ===================
        let cloned_emission: Vec<u64> = combined_emission.clone();
        let cloned_ranks: Vec<u16> = ranks
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_trust: Vec<u16> = trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_consensus: Vec<u16> = consensus
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_incentive: Vec<u16> = incentive
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_dividends: Vec<u16> = dividends
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        let cloned_pruning_scores: Vec<u16> = vec_max_upscale_to_u16(&pruning_scores);
        let cloned_validator_trust: Vec<u16> = validator_trust
            .iter()
            .map(|xi| fixed_proportion_to_u16(*xi))
            .collect::<Vec<u16>>();
        Active::<T>::insert(netuid, active.clone());
        Emission::<T>::insert(netuid, cloned_emission);
        Rank::<T>::insert(netuid, cloned_ranks);
        Trust::<T>::insert(netuid, cloned_trust);
        Consensus::<T>::insert(netuid, cloned_consensus);
        Incentive::<T>::insert(netuid, cloned_incentive);
        Dividends::<T>::insert(netuid, cloned_dividends);
        PruningScores::<T>::insert(netuid, cloned_pruning_scores);
        ValidatorTrust::<T>::insert(netuid, cloned_validator_trust);
        ValidatorPermit::<T>::insert(netuid, new_validator_permits.clone());

        // Column max-upscale EMA bonds for storage: max_i w_ij = 1.
        inplace_col_max_upscale_sparse(&mut ema_bonds, n);
        for i in 0..n {
            // Set bonds only if uid retains validator permit, otherwise clear bonds.
            if new_validator_permits[i as usize] {
                let new_bonds_row: Vec<(u16, u16)> = ema_bonds[i as usize]
                    .iter()
                    .map(|(j, value)| (*j, fixed_proportion_to_u16(*value)))
                    .collect();
                Bonds::<T>::insert(netuid, i, new_bonds_row);
            } else if validator_permits[i as usize] {
                // Only overwrite the intersection.
                let new_empty_bonds_row: Vec<(u16, u16)> = vec![];
                Bonds::<T>::insert(netuid, i, new_empty_bonds_row);
            }
        }

        // Emission tuples ( hotkeys, server_emission, validator_emission )
        let mut result: Vec<(T::AccountId, u64, u64)> = vec![];
        for (uid_i, hotkey) in hotkeys.iter() {
            result.push((
                hotkey.clone(),
                server_emission[*uid_i as usize],
                validator_emission[*uid_i as usize],
            ));
        }
        result
    }

    pub fn get_float_rho(netuid: u16) -> I32F32 {
        I32F32::from_num(Self::get_rho(netuid))
    }
    pub fn get_float_kappa(netuid: u16) -> I32F32 {
        I32F32::from_num(Self::get_kappa(netuid)) / I32F32::from_num(u16::MAX)
    }

    pub fn get_normalized_stake(netuid: u16) -> Vec<I32F32> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut stake_64: Vec<I64F64> = vec![I64F64::from_num(0.0); n];
        for neuron_uid in 0..n {
            stake_64[neuron_uid] = I64F64::from_num(Self::get_stake_for_uid_and_subnetwork(
                netuid,
                neuron_uid as u16,
            ));
        }
        inplace_normalize_64(&mut stake_64);
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32(stake_64);
        stake
    }

    pub fn get_block_at_registration(netuid: u16) -> Vec<u64> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut block_at_registration: Vec<u64> = vec![0; n];
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key(netuid, neuron_uid as u16) {
                block_at_registration[neuron_uid] =
                    Self::get_neuron_block_at_registration(netuid, neuron_uid as u16);
            }
        }
        block_at_registration
    }

    // Output unnormalized sparse weights, input weights are assumed to be row max-upscaled in u16.
    pub fn get_weights_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
        {
            if uid_i >= n as u16 {
                continue;
            }
            for (uid_j, weight_ij) in weights_i.iter() {
                if *uid_j >= n as u16 {
                    continue;
                }
                weights[uid_i as usize].push((*uid_j, I32F32::from_num(*weight_ij)));
            }
        }
        weights
    }

    // Output unnormalized weights in [n, n] matrix, input weights are assumed to be row max-upscaled in u16.
    pub fn get_weights(netuid: u16) -> Vec<Vec<I32F32>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut weights: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.0); n]; n];
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
        {
            for (uid_j, weight_ij) in weights_i.iter() {
                weights[uid_i as usize][*uid_j as usize] = I32F32::from_num(*weight_ij);
            }
        }
        weights
    }

    // Output unnormalized sparse bonds, input bonds are assumed to be column max-upscaled in u16.
    pub fn get_bonds_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![vec![]; n];
        for (uid_i, bonds_i) in
            <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
        {
            for (uid_j, bonds_ij) in bonds_i.iter() {
                bonds[uid_i as usize].push((*uid_j, I32F32::from_num(*bonds_ij)));
            }
        }
        bonds
    }

    // Output unnormalized bonds in [n, n] matrix, input bonds are assumed to be column max-upscaled in u16.
    pub fn get_bonds(netuid: u16) -> Vec<Vec<I32F32>> {
        let n: usize = Self::get_subnetwork_n(netuid) as usize;
        let mut bonds: Vec<Vec<I32F32>> = vec![vec![I32F32::from_num(0.0); n]; n];
        for (uid_i, bonds_i) in
            <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
        {
            for (uid_j, bonds_ij) in bonds_i.iter() {
                bonds[uid_i as usize][*uid_j as usize] = I32F32::from_num(*bonds_ij);
            }
        }
        bonds
    }
}
