use super::*;
use crate::math::*;
use frame_support::sp_std::vec;
use frame_support::inherent::Vec;
use substrate_fixed::types::{I32F32, I64F64, I96F32};
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {

    /// Calculates reward consensus and returns the emissions for uids/hotkeys in a given `netuid`.
    /// (Dense version used only for testing purposes.)
    pub fn epoch_dense( netuid: u16, rao_emission: u64 ) -> Vec<(T::AccountId, u64)> {
  
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n( netuid );
        log::trace!( "n:\n{:?}\n", n );

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::trace!( "current_block:\n{:?}\n", current_block );

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff( netuid ) as u64;
        log::trace!( "activity_cutoff:\n{:?}\n", activity_cutoff );

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update( netuid );
        log::trace!( "Last update:\n{:?}\n", last_update.clone() );

        // Inactive mask.
        let inactive: Vec<bool> = last_update.iter().map(| updated | *updated + activity_cutoff < current_block ).collect();
        log::trace!( "Inactive:\n{:?}\n", inactive.clone() );

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration( netuid );
        log::trace!( "Block at registration:\n{:?}\n", block_at_registration.clone() );

        // Outdated matrix, updated_ij=True if i has last updated (weights) after j has last registered.
        let outdated: Vec<Vec<bool>> = last_update.iter().map(| updated | block_at_registration.iter().map(| registered | updated <= registered ).collect() ).collect();
        log::trace!( "Outdated:\n{:?}\n", outdated.clone() );

        // ===========
        // == Stake ==
        // ===========
        let mut hotkeys: Vec<(u16, T::AccountId)> = vec![];
        for ( uid_i, hotkey ) in < Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId >>::iter_prefix( netuid ) {
            hotkeys.push( (uid_i, hotkey) ); 
        }
        log::trace!( "hotkeys: {:?}", hotkeys.clone() );

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![ I64F64::from_num(0.0); n as usize ];
        for (uid_i, hotkey) in hotkeys.iter() {
            stake_64[ *uid_i as usize ] = I64F64::from_num( Self::get_total_stake_for_hotkey( hotkey ) );
        }
        inplace_normalize_64( &mut stake_64 );
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32( stake_64 );
        log::trace!( "S:\n{:?}\n", stake.clone() );

        // Remove inactive stake.
        let mut active_stake: Vec<I32F32> = stake.clone();
        inplace_mask_vector( &inactive, &mut active_stake );

        // Normalize active stake.
        inplace_normalize( &mut active_stake );
        log::trace!( "S:\n{:?}\n", active_stake.clone() );

        // =======================
        // == Validator permits ==
        // =======================

        // Get validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permit( netuid );
        log::trace!( "validator_permits: {:?}", validator_permits );

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators( netuid );
        log::trace!( "max_allowed_validators: {:?}", max_allowed_validators );

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk( &stake, max_allowed_validators as usize );
        log::trace!( "new_validator_permits: {:?}", new_validator_permits );

        // =============
        // == Weights ==
        // =============

        // Access network weights row normalized.
        let mut weights: Vec<Vec<I32F32>> = Self::get_weights( netuid );
        log::trace!( "W:\n{:?}\n", weights.clone() );

        // Mask weights that are not from permitted validators.
        inplace_mask_rows( &validator_forbids, &mut weights );
        // log::trace!( "W (permit): {:?}", weights.clone() );

        // Remove self-weight by masking diagonal.
        inplace_mask_diag( &mut weights );
        // log::trace!( "W (permit+diag):\n{:?}\n", weights.clone() );

        // Mask outdated weights: remove weights referring to deregistered neurons.
        inplace_mask_matrix( &outdated, &mut weights );
        // log::trace!( "W (permit+diag+outdate):\n{:?}\n", weights.clone() );

        // Normalize remaining weights.
        inplace_row_normalize( &mut weights );
        // log::trace!( "W (mask+norm):\n{:?}\n", weights.clone() );

        // ==========================================
        // == Weight consensus and Validator trust ==
        // ==========================================

        let mut weight_cuts: u16 = Self::get_weight_cuts( netuid ); // number of weight cuts.
        if weight_cuts > 3 { weight_cuts = 3; } // limit weight cuts given computational expense of each cut.
        let mut weight_consensus: Vec<I32F32> = vec![ I32F32::from_num(0); n as usize ]; // weight consensus.
        for i in 0..=weight_cuts {
            // Calculate specific vote share per weight assigned: [validator] -> [server] -> vote_share.
            let mut vote_share: Vec<Vec<I32F32>> = row_hadamard( &weights, &active_stake ); // ΔB = W◦S
            inplace_col_normalize( &mut vote_share ); // sum_i b_ij = 1

            // Calculate the weight consensus for each uid.
            let weight_votes: Vec<Vec<I32F32>> = hadamard( &vote_share, &weights );
            weight_consensus = col_sum( &weight_votes );

            // Cut the weights above consensus.
            if i < weight_cuts {
                inplace_col_clip( &mut weights, &weight_consensus );
            } // else do not cut in last iteration, which is only to get final weight_consensus.
        }
        let validator_trust: Vec<I32F32> = row_sum( &weights );

        // ========================================
        // == Ranks, Trust, Consensus, Incentive ==
        // ========================================

        // Compute ranks: r_j = SUM(i) w_ij * s_i
        let mut ranks: Vec<I32F32> = matmul( &weights, &active_stake );
        inplace_normalize( &mut ranks );
        log::trace!( "R:\n{:?}\n", ranks.clone() );

        // Compute thresholded weights.
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.1 ) / I32F32::from_num( n + 1 );
        let clipped_weights: Vec<Vec<I32F32>> = clip( &weights, threshold, upper, lower );
        // log::trace!( "tW:\n{:?}\n", clipped_weights.clone() );

        // Compute trust scores: t_j = SUM(i) w_ij * s_i
        let trust: Vec<I32F32> = matmul( &clipped_weights, &active_stake );
        log::trace!( "T:\n{:?}\n", trust.clone() );

        // Compute consensus.
        let rho: I32F32 = Self::get_float_rho( netuid );
        let kappa: I32F32 = Self::get_float_kappa( netuid );
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, rho, kappa)).collect();
        log::trace!( "C:\n{:?}\n", consensus.clone() );

        // Compute incentive.
        let mut incentive: Vec<I32F32> = ranks.iter().zip( consensus.clone() ).map( |(ri, ci)| ri * ci ).collect();
        inplace_normalize( &mut incentive );
        log::trace!( "I:\n{:?}\n", incentive.clone() );

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds column normalized.
        let mut bonds: Vec<Vec<I32F32>> = Self::get_bonds( netuid );
        inplace_mask_matrix( &outdated, &mut bonds );  // mask outdated bonds
        inplace_col_normalize( &mut bonds ); // sum_i b_ij = 1
        // log::trace!( "B:\n{:?}\n", bonds.clone() );        

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<I32F32>> = row_hadamard( &weights, &active_stake ); // ΔB = W◦S
        inplace_col_normalize( &mut bonds_delta ); // sum_i b_ij = 1
        // log::trace!( "ΔB:\n{:?}\n", bonds_delta.clone() );
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.1 );
        let mut ema_bonds: Vec<Vec<I32F32>> = mat_ema( &bonds_delta, &bonds, alpha );
        inplace_col_normalize( &mut ema_bonds ); // sum_i b_ij = 1
        // log::trace!( "emaB:\n{:?}\n", ema_bonds.clone() );

        // Compute dividends: d_i = SUM(j) b_ij * inc_j
        let mut dividends: Vec<I32F32> = matmul_transpose( &ema_bonds, &incentive );
        inplace_normalize( &mut dividends );
        log::trace!( "D:\n{:?}\n", dividends.clone() );

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute emission scores.
        let mut normalized_emission: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut normalized_emission );
        
        // If emission is zero, replace emission with normalized stake.
        if is_zero( &normalized_emission ) { // no weights set | outdated weights | self_weights
            if is_zero( &active_stake ) { // no active stake
                normalized_emission = stake.clone(); // do not mask inactive, assumes stake is normalized
            }
            else {
                normalized_emission = active_stake.clone(); // emission proportional to inactive-masked normalized stake
            }
        }

        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num( rao_emission );
        let emission: Vec<I96F32> = normalized_emission.iter().map( |e: &I32F32| I96F32::from_num( *e ) * float_rao_emission ).collect();
        let emission: Vec<u64> = emission.iter().map( |e: &I96F32| e.to_num::<u64>() ).collect();
        log::trace!( "E: {:?}", emission.clone() );

        // Set pruning scores.
        let pruning_scores: Vec<I32F32> = normalized_emission.clone();
        log::trace!( "P: {:?}", pruning_scores.clone() );

        // ===================
        // == Value storage ==
        // ===================
        let cloned_emission: Vec<u64> = emission.clone();
        let cloned_ranks: Vec<u16> = ranks.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_trust: Vec<u16> = trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_consensus: Vec<u16> = consensus.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_incentive: Vec<u16> = incentive.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_dividends: Vec<u16> = dividends.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_prunning_socres: Vec<u16> = pruning_scores.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_validator_trust: Vec<u16> = validator_trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_weight_consensus: Vec<u16> = validator_trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        Emission::<T>::insert( netuid, cloned_emission );
        Rank::<T>::insert( netuid, cloned_ranks);
        Trust::<T>::insert( netuid, cloned_trust);
        Consensus::<T>::insert( netuid, cloned_consensus );
        Incentive::<T>::insert( netuid, cloned_incentive );
        Dividends::<T>::insert( netuid, cloned_dividends );
        PruningScores::<T>::insert( netuid, cloned_prunning_socres );
        ValidatorTrust::<T>::insert( netuid, cloned_validator_trust );
        ValidatorPermit::<T>::insert( netuid, new_validator_permits.clone() );
        WeightConsensus::<T>::insert( netuid, cloned_weight_consensus);

        for i in 0..n {
            // Set bonds only if uid retains validator permit, otherwise clear bonds.
            if new_validator_permits[i as usize] {
                let new_bonds_row: Vec<(u16,u16)> = (0..n).zip( vec_fixed_proportions_to_u16( ema_bonds[i as usize].clone() ) ).collect();
                Bonds::<T>::insert( netuid, i, new_bonds_row );
            }
            else if validator_permits[ i as usize ] {
                // Only overwrite the intersection.
                let new_empty_bonds_row: Vec<(u16,u16)> = vec![];
                Bonds::<T>::insert( netuid, i, new_empty_bonds_row );
            }
        }


        let mut result: Vec<(T::AccountId, u64)> = vec![]; 
        for ( uid_i, hotkey ) in hotkeys.iter() {
            result.push( ( hotkey.clone(), emission[ *uid_i as usize ] ) );
        }
        result

    }

    /// Calculates reward consensus values, then updates rank, trust, consensus, incentive, dividend, pruning_score, emission and bonds, and 
    /// returns the emissions for uids/hotkeys in a given `netuid`.
    ///
    /// # Args:
    /// 	* 'netuid': ( u16 ):
    ///         - The network to distribute the emission onto.
    /// 		
    /// 	* 'rao_emission': ( u64 ):
    ///         - The total emission for the epoch.
    ///
    /// 	* 'debug' ( bool ):
    /// 		- Print debugging outputs.
    ///    
    pub fn epoch( netuid: u16, rao_emission: u64 ) -> Vec<(T::AccountId, u64)> {
        // Get subnetwork size.
        let n: u16 = Self::get_subnetwork_n( netuid );
        log::trace!( "n: {:?}", n );

        // ======================
        // == Active & updated ==
        // ======================

        // Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::trace!( "current_block: {:?}", current_block );

        // Get activity cutoff.
        let activity_cutoff: u64 = Self::get_activity_cutoff( netuid ) as u64;
        log::trace!( "activity_cutoff: {:?}", activity_cutoff );

        // Last update vector.
        let last_update: Vec<u64> = Self::get_last_update( netuid );
        log::trace!( "Last update: {:?}", last_update.clone() );

        // Inactive mask.
        let inactive: Vec<bool> = last_update.iter().map(| updated | *updated + activity_cutoff < current_block ).collect();
        log::trace!( "Inactive: {:?}", inactive.clone() );

        // Block at registration vector (block when each neuron was most recently registered).
        let block_at_registration: Vec<u64> = Self::get_block_at_registration( netuid );
        log::trace!( "Block at registration: {:?}", block_at_registration.clone() );

        // ===========
        // == Stake ==
        // ===========

        let mut hotkeys: Vec<(u16, T::AccountId)> = vec![];
        for ( uid_i, hotkey ) in < Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId >>::iter_prefix( netuid ) {
            hotkeys.push( (uid_i, hotkey) ); 
        }
        log::trace!( "hotkeys: {:?}", hotkeys.clone() );

        // Access network stake as normalized vector.
        let mut stake_64: Vec<I64F64> = vec![ I64F64::from_num(0.0); n as usize ];
        for (uid_i, hotkey) in hotkeys.iter() {
            stake_64[ *uid_i as usize ] = I64F64::from_num( Self::get_total_stake_for_hotkey( hotkey ) );
        }
        inplace_normalize_64( &mut stake_64 );
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32( stake_64 );
        // range: I32F32(0, 1)
        log::trace!( "S: {:?}", stake.clone() );

        // Remove inactive stake.
        let mut active_stake: Vec<I32F32> = stake.clone();
        inplace_mask_vector( &inactive, &mut active_stake );
        log::trace!( "S (mask): {:?}", active_stake.clone() );

        // Normalize active stake.
        inplace_normalize( &mut active_stake );
        log::trace!( "S (mask+norm): {:?}", active_stake.clone() );

        // =======================
        // == Validator permits ==
        // =======================

        // Get current validator permits.
        let validator_permits: Vec<bool> = Self::get_validator_permit( netuid );
        log::trace!( "validator_permits: {:?}", validator_permits );

        // Logical negation of validator_permits.
        let validator_forbids: Vec<bool> = validator_permits.iter().map(|&b| !b).collect();

        // Get max allowed validators.
        let max_allowed_validators: u16 = Self::get_max_allowed_validators( netuid );
        log::trace!( "max_allowed_validators: {:?}", max_allowed_validators );

        // Get new validator permits.
        let new_validator_permits: Vec<bool> = is_topk( &stake, max_allowed_validators as usize );
        log::trace!( "new_validator_permits: {:?}", new_validator_permits );

        // =============
        // == Weights ==
        // =============

        // Access network weights row normalized.
        let mut weights: Vec<Vec<(u16, I32F32)>> = Self::get_weights_sparse( netuid );
        // log::trace!( "W: {:?}", weights.clone() );

        // Mask weights that are not from permitted validators.
        weights = mask_rows_sparse( &validator_forbids, &weights );
        // log::trace!( "W (permit): {:?}", weights.clone() );

        // Remove self-weight by masking diagonal.
        weights = mask_diag_sparse( &weights );
        // log::trace!( "W (permit+diag): {:?}", weights.clone() );

        // Remove weights referring to deregistered neurons.
        weights = vec_mask_sparse_matrix( &weights, &last_update, &block_at_registration, &| updated, registered | updated <= registered );
        // log::trace!( "W (permit+diag+outdate): {:?}", weights.clone() );

        // Normalize remaining weights.
        inplace_row_normalize_sparse( &mut weights );
        // log::trace!( "W (mask+norm): {:?}", weights.clone() );

        // ==========================================
        // == Weight consensus and Validator trust ==
        // ==========================================

        let mut weight_cuts: u16 = Self::get_weight_cuts( netuid ); // number of weight cuts.
        if weight_cuts > 3 { weight_cuts = 3; } // limit weight cuts given computational expense of each cut.
        let mut weight_consensus: Vec<I32F32> = vec![ I32F32::from_num(0); n as usize ]; // weight consensus.
        for i in 0..=weight_cuts {
            // Calculate specific vote share per weight assigned: [validator] -> [server] -> vote_share.
            let mut vote_share: Vec<Vec<(u16, I32F32)>> = row_hadamard_sparse( &weights, &active_stake ); // ΔB = W◦S
            inplace_col_normalize_sparse( &mut vote_share, n ); // sum_i b_ij = 1

            // Calculate the weight consensus for each uid.
            let weight_votes: Vec<Vec<(u16, I32F32)>> = hadamard_sparse( &vote_share, &weights, n );
            weight_consensus = col_sum_sparse( &weight_votes, n );

            // Cut the weights above consensus.
            if i < weight_cuts {
                weights = col_clip_sparse( &weights, &weight_consensus );
            } // else do not cut in last iteration, which is only to get final weight_consensus.
        }
        let validator_trust: Vec<I32F32> = row_sum_sparse( &weights );

        // ========================================
        // == Ranks, Trust, Consensus, Incentive ==
        // ========================================

        // Compute ranks: r_j = SUM(i) w_ij * s_i.
        // range: I32F32(0, 1)
        let mut ranks: Vec<I32F32> = matmul_sparse( &weights, &active_stake, n );
        inplace_normalize( &mut ranks );
        log::trace!( "R: {:?}", ranks.clone() );

        // Compute thresholded weights.
        // range: I32F32(0, 1)
        let upper: I32F32 = I32F32::from_num( 1.0 );
        let lower: I32F32 = I32F32::from_num( 0.0 );
        let threshold: I32F32 = I32F32::from_num( 0.1 ) / I32F32::from_num( n + 1 );
        let clipped_weights: Vec<Vec<(u16, I32F32)>> = clip_sparse( &weights, threshold, upper, lower );
        // log::trace!( "W (threshold): {:?}", clipped_weights.clone() );

        // Compute trust scores: t_j = SUM(i) w_ij * s_i
        // range: I32F32(0, 1)
        let trust: Vec<I32F32> = matmul_sparse( &clipped_weights, &active_stake, n );
        log::trace!( "T: {:?}", trust.clone() );

        // Compute consensus.
        // range: I32F32(0, 1)
        let rho: I32F32 = Self::get_float_rho( netuid );
        let kappa: I32F32 = Self::get_float_kappa( netuid );
        let consensus: Vec<I32F32> = trust.iter().map(|t: &I32F32| sigmoid_safe(*t, rho, kappa)).collect();
        log::trace!( "C: {:?}", consensus.clone() );

        // Compute incentive.
        // range: I32F32(0, 1)
        let mut incentive: Vec<I32F32> = ranks.iter().zip( consensus.clone() ).map( |(ri, ci)| ri * ci ).collect();
        inplace_normalize( &mut incentive );
        log::trace!( "I: {:?}", incentive.clone() );

        // =========================
        // == Bonds and Dividends ==
        // =========================

        // Access network bonds column normalized.
        let mut bonds: Vec<Vec<(u16, I32F32)>> = Self::get_bonds_sparse( netuid );
        // log::trace!( "B: {:?}", bonds.clone() );
        
        // Remove bonds referring to deregistered neurons.
        bonds = vec_mask_sparse_matrix( &bonds, &last_update, &block_at_registration, &| updated, registered | updated <= registered );
        // log::trace!( "B (outdatedmask): {:?}", bonds.clone() );

        // Normalize remaining bonds: sum_i b_ij = 1.
        inplace_col_normalize_sparse( &mut bonds, n );
        // log::trace!( "B (mask+norm): {:?}", bonds.clone() );        

        // Compute bonds delta column normalized.
        let mut bonds_delta: Vec<Vec<(u16, I32F32)>> = row_hadamard_sparse( &weights, &active_stake ); // ΔB = W◦S (outdated W masked)
        // log::trace!( "ΔB: {:?}", bonds_delta.clone() );

        // Normalize bonds delta.
        inplace_col_normalize_sparse( &mut bonds_delta, n ); // sum_i b_ij = 1
        // log::trace!( "ΔB (norm): {:?}", bonds_delta.clone() );
    
        // Compute bonds moving average.
        let alpha: I32F32 = I32F32::from_num( 0.1 );
        let mut ema_bonds: Vec<Vec<(u16, I32F32)>> = mat_ema_sparse( &bonds_delta, &bonds, alpha );

        // Normalize EMA bonds.
        inplace_col_normalize_sparse( &mut ema_bonds, n ); // sum_i b_ij = 1
        // log::trace!( "emaB: {:?}", ema_bonds.clone() );

        // Compute dividends: d_i = SUM(j) b_ij * inc_j.
        // range: I32F32(0, 1)
        let mut dividends: Vec<I32F32> = matmul_transpose_sparse( &ema_bonds, &incentive );
        inplace_normalize( &mut dividends );
        log::trace!( "D: {:?}", dividends.clone() );

        // =================================
        // == Emission and Pruning scores ==
        // =================================

        // Compute normalized emission scores. range: I32F32(0, 1)
        let mut normalized_emission: Vec<I32F32> = incentive.iter().zip( dividends.clone() ).map( |(ii, di)| ii + di ).collect();
        inplace_normalize( &mut normalized_emission );

        // If emission is zero, replace emission with normalized stake.
        if is_zero( &normalized_emission ) { // no weights set | outdated weights | self_weights
            if is_zero( &active_stake ) { // no active stake
                normalized_emission = stake.clone(); // do not mask inactive, assumes stake is normalized
            }
            else {
                normalized_emission = active_stake.clone(); // emission proportional to inactive-masked normalized stake
            }
        }
        
        // Compute rao based emission scores. range: I96F32(0, rao_emission)
        let float_rao_emission: I96F32 = I96F32::from_num( rao_emission );
        let emission: Vec<I96F32> = normalized_emission.iter().map( |e: &I32F32| I96F32::from_num( *e ) * float_rao_emission ).collect();
        let emission: Vec<u64> = emission.iter().map( |e: &I96F32| e.to_num::<u64>() ).collect();
        log::trace!( "nE: {:?}", normalized_emission.clone() );
        log::trace!( "E: {:?}", emission.clone() );

        // Set pruning scores.
        let pruning_scores: Vec<I32F32> = normalized_emission.clone();
        log::trace!( "P: {:?}", pruning_scores.clone() );

        // ===================
        // == Value storage ==
        // ===================
        let cloned_emission: Vec<u64> = emission.clone();
        let cloned_ranks: Vec<u16> = ranks.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_trust: Vec<u16> = trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_consensus: Vec<u16> = consensus.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_incentive: Vec<u16> = incentive.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_dividends: Vec<u16> = dividends.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_prunning_socres: Vec<u16> = pruning_scores.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_validator_trust: Vec<u16> = validator_trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        let cloned_weight_consensus: Vec<u16> = validator_trust.iter().map(|xi| fixed_proportion_to_u16(*xi)).collect::<Vec<u16>>();
        Emission::<T>::insert( netuid, cloned_emission );
        Rank::<T>::insert( netuid, cloned_ranks);
        Trust::<T>::insert( netuid, cloned_trust);
        Consensus::<T>::insert( netuid, cloned_consensus );
        Incentive::<T>::insert( netuid, cloned_incentive );
        Dividends::<T>::insert( netuid, cloned_dividends );
        PruningScores::<T>::insert( netuid, cloned_prunning_socres );
        ValidatorTrust::<T>::insert( netuid, cloned_validator_trust );
        ValidatorPermit::<T>::insert( netuid, new_validator_permits.clone() );
        WeightConsensus::<T>::insert( netuid, cloned_weight_consensus);

        for i in 0..n {
            // Set bonds only if uid retains validator permit, otherwise clear bonds.
            if new_validator_permits[i as usize] {
                let new_bonds_row: Vec<(u16,u16)> = ema_bonds[i as usize].iter().map( |(j, value)| (*j, fixed_proportion_to_u16(*value))).collect();
                Bonds::<T>::insert( netuid, i, new_bonds_row );
            }
            else if validator_permits[ i as usize ] {
                // Only overwrite the intersection.
                let new_empty_bonds_row: Vec<(u16,u16)> = vec![];
                Bonds::<T>::insert( netuid, i, new_empty_bonds_row );
            }
        }

        // Emission tuples ( hotkeys, u64 emission)
        let mut result: Vec<(T::AccountId, u64)> = vec![]; 
        for ( uid_i, hotkey ) in hotkeys.iter() {
            result.push( ( hotkey.clone(), emission[ *uid_i as usize ] ) );
        }
        result
    }

    pub fn get_float_rho( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_rho( netuid ) )  }
    pub fn get_float_kappa( netuid:u16 ) -> I32F32 { I32F32::from_num( Self::get_kappa( netuid )  ) / I32F32::from_num( u16::MAX ) }

    pub fn get_normalized_stake( netuid:u16 ) -> Vec<I32F32> {
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut stake_64: Vec<I64F64> = vec![ I64F64::from_num(0.0); n ]; 
        for neuron_uid in 0..n {
            stake_64[neuron_uid] = I64F64::from_num( Self::get_stake_for_uid_and_subnetwork( netuid, neuron_uid as u16 ) );
        }
        inplace_normalize_64( &mut stake_64 );
        let stake: Vec<I32F32> = vec_fixed64_to_fixed32( stake_64 );
        stake
    }

    pub fn get_block_at_registration( netuid:u16 ) -> Vec<u64> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize;
        let mut block_at_registration: Vec<u64> = vec![ 0; n ];
        for neuron_uid in 0..n {
            if Keys::<T>::contains_key( netuid, neuron_uid as u16 ){
                block_at_registration[ neuron_uid ] = Self::get_neuron_block_at_registration( netuid, neuron_uid as u16 );
            }
        }
        block_at_registration
    }

    pub fn get_weights_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ].push( ( *uid_j, u16_proportion_to_fixed( *weight_ij ) ));
            }
        }
        weights
    } 

    pub fn get_weights( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut weights: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, weights_i ) in < Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, weight_ij) in weights_i.iter() { 
                weights [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed(  *weight_ij );
            }
        }
        weights
    }

    pub fn get_bonds_sparse( netuid:u16 ) -> Vec<Vec<(u16, I32F32)>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<(u16, I32F32)>> = vec![ vec![]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ].push( ( *uid_j, u16_proportion_to_fixed( *bonds_ij ) ));
            }
        }
        bonds
    } 

    pub fn get_bonds( netuid:u16 ) -> Vec<Vec<I32F32>> { 
        let n: usize = Self::get_subnetwork_n( netuid ) as usize; 
        let mut bonds: Vec<Vec<I32F32>> = vec![ vec![ I32F32::from_num(0.0); n ]; n ]; 
        for ( uid_i, bonds_i ) in < Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)> >>::iter_prefix( netuid ) {
            for (uid_j, bonds_ij) in bonds_i.iter() { 
                bonds [ uid_i as usize ] [ *uid_j as usize ] = u16_proportion_to_fixed( *bonds_ij );
            }
        }
        bonds
    }
}
