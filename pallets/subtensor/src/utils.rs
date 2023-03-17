
use super::*;
use frame_support::inherent::Vec;
use sp_core::U256;
use frame_support::pallet_prelude::DispatchResult;
use crate::system::ensure_root;

impl<T: Config> Pallet<T> {

    // ========================
	// ==== Global Setters ====
	// ========================
    pub fn set_tempo( netuid: u16, tempo: u16 ) { Tempo::<T>::insert( netuid, tempo ); }
    pub fn set_last_adjustment_block( netuid: u16, last_adjustment_block: u64 ) { LastAdjustmentBlock::<T>::insert( netuid, last_adjustment_block ); }
    pub fn set_blocks_since_last_step( netuid: u16, blocks_since_last_step: u64 ) { BlocksSinceLastStep::<T>::insert( netuid, blocks_since_last_step ); }
    pub fn set_registrations_this_block( netuid: u16, registrations_this_block: u16 ) { RegistrationsThisBlock::<T>::insert(netuid, registrations_this_block); }
    pub fn set_last_mechanism_step_block( netuid: u16, last_mechanism_step_block: u64 ) { LastMechansimStepBlock::<T>::insert(netuid, last_mechanism_step_block); }
    pub fn set_registrations_this_interval( netuid: u16, registrations_this_interval: u16 ) { RegistrationsThisInterval::<T>::insert(netuid, registrations_this_interval); }
    pub fn set_pow_registrations_this_interval( netuid: u16, pow_registrations_this_interval: u16 ) { POWRegistrationsThisInterval::<T>::insert(netuid, pow_registrations_this_interval); }
    pub fn set_burn_registrations_this_interval( netuid: u16, burn_registrations_this_interval: u16 ) { BurnRegistrationsThisInterval::<T>::insert(netuid, burn_registrations_this_interval); }

    // ========================
	// ==== Global Getters ====
	// ========================
    pub fn get_total_issuance() -> u64 { TotalIssuance::<T>::get() }
    pub fn get_block_emission() -> u64 { BlockEmission::<T>::get() }
    pub fn get_current_block_as_u64( ) -> u64 { TryInto::try_into( <frame_system::Pallet<T>>::block_number() ).ok().expect("blockchain will not exceed 2^64 blocks; QED.") }

    // ==============================
	// ==== YumaConsensus params ====
	// ==============================
    pub fn get_rank( netuid:u16 ) -> Vec<u16> { Rank::<T>::get( netuid ) }
    pub fn get_trust( netuid:u16 ) -> Vec<u16> { Trust::<T>::get( netuid ) }
    pub fn get_active( netuid:u16 ) -> Vec<bool> { Active::<T>::get( netuid ) }
    pub fn get_emission( netuid:u16 ) -> Vec<u64> { Emission::<T>::get( netuid ) }
    pub fn get_consensus( netuid:u16 ) -> Vec<u16> { Consensus::<T>::get( netuid ) }
    pub fn get_incentive( netuid:u16 ) -> Vec<u16> { Incentive::<T>::get( netuid ) }
    pub fn get_dividends( netuid:u16 ) -> Vec<u16> { Dividends::<T>::get( netuid ) }
    pub fn get_last_update( netuid:u16 ) -> Vec<u64> { LastUpdate::<T>::get( netuid ) }
    pub fn get_pruning_score( netuid:u16 ) -> Vec<u16> { PruningScores::<T>::get( netuid ) }
    pub fn get_validator_trust( netuid:u16 ) -> Vec<u16> { ValidatorTrust::<T>::get( netuid ) }
    pub fn get_validator_permit( netuid:u16 ) -> Vec<bool> { ValidatorPermit::<T>::get( netuid ) }

    // ==================================
	// ==== YumaConsensus UID params ====
	// ==================================
    pub fn set_last_update_for_uid( netuid:u16, uid: u16, last_update: u64 ) { 
        let mut updated_last_update_vec = Self::get_last_update( netuid ); 
        if (uid as usize) < updated_last_update_vec.len() { 
            updated_last_update_vec[uid as usize] = last_update;
            LastUpdate::<T>::insert( netuid, updated_last_update_vec );
        }  
    }
    pub fn set_active_for_uid( netuid:u16, uid: u16, active: bool ) { 
        let mut updated_active_vec = Self::get_active( netuid ); 
        if (uid as usize) < updated_active_vec.len() { 
            updated_active_vec[uid as usize] = active;
            Active::<T>::insert( netuid, updated_active_vec );
        }  
    }
    pub fn set_pruning_score_for_uid( netuid:u16, uid: u16, pruning_score: u16 ) {
        log::info!("netuid = {:?}", netuid);
        log::info!("SubnetworkN::<T>::get( netuid ) = {:?}", SubnetworkN::<T>::get( netuid ) );
        log::info!("uid = {:?}", uid );
        assert!( uid < SubnetworkN::<T>::get( netuid ) );
        PruningScores::<T>::mutate( netuid, |v| v[uid as usize] = pruning_score );
    }
    pub fn set_validator_permit_for_uid( netuid:u16, uid: u16, validator_permit: bool ) { 
        let mut updated_validator_permit = Self::get_validator_permit( netuid ); 
        if (uid as usize) < updated_validator_permit.len() { 
            updated_validator_permit[uid as usize] = validator_permit;
            ValidatorPermit::<T>::insert( netuid, updated_validator_permit );
        }  
    }

    pub fn get_rank_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Rank::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_trust_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Trust::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_emission_for_uid( netuid:u16, uid: u16) -> u64 {let vec =  Emission::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_active_for_uid( netuid:u16, uid: u16) -> bool { let vec = Active::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return false } }
    pub fn get_consensus_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Consensus::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_incentive_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Incentive::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_dividends_for_uid( netuid:u16, uid: u16) -> u16 { let vec = Dividends::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_last_update_for_uid( netuid:u16, uid: u16) -> u64 { let vec = LastUpdate::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_pruning_score_for_uid( netuid:u16, uid: u16) -> u16 { let vec = PruningScores::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return u16::MAX } }
    pub fn get_validator_trust_for_uid( netuid:u16, uid: u16) -> u16 { let vec = ValidatorTrust::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return 0 } }
    pub fn get_validator_permit_for_uid( netuid:u16, uid: u16) -> bool { let vec = ValidatorPermit::<T>::get( netuid ); if (uid as usize) < vec.len() { return vec[uid as usize] } else{ return false } }

    // ============================
	// ==== Subnetwork Getters ====
	// ============================
    pub fn get_tempo( netuid:u16 ) -> u16{ Tempo::<T>::get( netuid ) }
    pub fn get_emission_value( netuid: u16 ) -> u64 { EmissionValues::<T>::get( netuid ) }
    pub fn get_pending_emission( netuid:u16 ) -> u64{ PendingEmission::<T>::get( netuid ) }
    pub fn get_last_adjustment_block( netuid: u16) -> u64 { LastAdjustmentBlock::<T>::get( netuid ) }
    pub fn get_blocks_since_last_step(netuid:u16 ) -> u64 { BlocksSinceLastStep::<T>::get( netuid ) }
    pub fn get_difficulty( netuid: u16 ) -> U256 { U256::from( Self::get_difficulty_as_u64( netuid ) ) }    
    pub fn get_registrations_this_block( netuid:u16 ) -> u16 { RegistrationsThisBlock::<T>::get( netuid ) }
    pub fn get_last_mechanism_step_block( netuid: u16 ) -> u64 { LastMechansimStepBlock::<T>::get( netuid ) }
    pub fn get_registrations_this_interval( netuid: u16 ) -> u16 { RegistrationsThisInterval::<T>::get( netuid ) } 
    pub fn get_pow_registrations_this_interval( netuid: u16 ) -> u16 { POWRegistrationsThisInterval::<T>::get( netuid ) } 
    pub fn get_burn_registrations_this_interval( netuid: u16 ) -> u16 { BurnRegistrationsThisInterval::<T>::get( netuid ) } 
    pub fn get_neuron_block_at_registration( netuid: u16, neuron_uid: u16 ) -> u64 { BlockAtRegistration::<T>::get( netuid, neuron_uid )}

    // ========================
	// ==== Rate Limiting =====
	// ========================
	pub fn get_last_tx_block( key: &T::AccountId ) -> u64 { LastTxBlock::<T>::get( key ) }
	pub fn exceeds_tx_rate_limit( prev_tx_block: u64, current_block: u64 ) -> bool {
        let rate_limit: u64 = Self::get_tx_rate_limit();
		if rate_limit == 0 || prev_tx_block == 0 {
			return false;
		}

        return current_block - prev_tx_block <= rate_limit;
    }

    // ========================
	// ==== Sudo calls ========
	// ========================
    pub fn get_default_take() -> u16 { DefaultTake::<T>::get() }
    pub fn set_default_take( default_take: u16 ) { DefaultTake::<T>::put( default_take ) }
    pub fn do_sudo_set_default_take( origin: T::RuntimeOrigin, default_take: u16 ) -> DispatchResult { 
        ensure_root( origin )?;
        Self::set_default_take( default_take );
        log::info!("DefaultTakeSet( default_take: {:?} ) ", default_take);
        Self::deposit_event( Event::DefaultTakeSet( default_take ) );
        Ok(()) 
    }

	// Configure tx rate limiting
	pub fn get_tx_rate_limit() -> u64 { TxRateLimit::<T>::get() }
    pub fn set_tx_rate_limit( tx_rate_limit: u64 ) { TxRateLimit::<T>::put( tx_rate_limit ) }
    pub fn do_sudo_set_tx_rate_limit( origin: T::RuntimeOrigin, tx_rate_limit: u64 ) -> DispatchResult { 
        ensure_root( origin )?;
        Self::set_tx_rate_limit( tx_rate_limit );
        log::info!("TxRateLimitSet( tx_rate_limit: {:?} ) ", tx_rate_limit );
        Self::deposit_event( Event::TxRateLimitSet( tx_rate_limit ) );
        Ok(()) 
    }

    pub fn get_serving_rate_limit( netuid: u16 ) -> u64 { ServingRateLimit::<T>::get(netuid) }
    pub fn set_serving_rate_limit( netuid: u16, serving_rate_limit: u64 ) { ServingRateLimit::<T>::insert( netuid, serving_rate_limit ) }
    pub fn do_sudo_set_serving_rate_limit( origin: T::RuntimeOrigin, netuid: u16, serving_rate_limit: u64 ) -> DispatchResult { 
        ensure_root( origin )?;
        Self::set_serving_rate_limit( netuid, serving_rate_limit );
        log::info!("ServingRateLimitSet( serving_rate_limit: {:?} ) ", serving_rate_limit );
        Self::deposit_event( Event::ServingRateLimitSet( netuid, serving_rate_limit ) );
        Ok(()) 
    }

    pub fn get_min_difficulty( netuid: u16) -> u64 { MinDifficulty::<T>::get( netuid ) }
    pub fn set_min_difficulty( netuid: u16, min_difficulty: u64 ) { MinDifficulty::<T>::insert( netuid, min_difficulty ); }
    pub fn do_sudo_set_min_difficulty( origin: T::RuntimeOrigin, netuid: u16, min_difficulty: u64 ) -> DispatchResult { 
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_min_difficulty( netuid, min_difficulty );
        log::info!("MinDifficultySet( netuid: {:?} min_difficulty: {:?} ) ", netuid, min_difficulty);
        Self::deposit_event( Event::MinDifficultySet( netuid, min_difficulty) );
        Ok(()) 
    }

    pub fn get_max_difficulty( netuid: u16) -> u64 { MaxDifficulty::<T>::get( netuid ) }
    pub fn set_max_difficulty( netuid: u16, max_difficulty: u64 ) { MaxDifficulty::<T>::insert( netuid, max_difficulty ); }
    pub fn do_sudo_set_max_difficulty( origin: T::RuntimeOrigin, netuid: u16, max_difficulty: u64 ) -> DispatchResult { 
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_max_difficulty( netuid, max_difficulty );
        log::info!("MaxDifficultySet( netuid: {:?} max_difficulty: {:?} ) ", netuid, max_difficulty);
        Self::deposit_event( Event::MaxDifficultySet( netuid, max_difficulty) );
        Ok(()) 
    }

    pub fn get_weights_version_key( netuid: u16) -> u64 { WeightsVersionKey::<T>::get( netuid ) }
    pub fn set_weights_version_key( netuid: u16, weights_version_key: u64 ) { WeightsVersionKey::<T>::insert( netuid, weights_version_key ); }
    pub fn do_sudo_set_weights_version_key( origin: T::RuntimeOrigin, netuid: u16, weights_version_key: u64 ) -> DispatchResult { 
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_weights_version_key( netuid, weights_version_key );
        log::info!("WeightsVersionKeySet( netuid: {:?} weights_version_key: {:?} ) ", netuid, weights_version_key);
        Self::deposit_event( Event::WeightsVersionKeySet( netuid, weights_version_key) );
        Ok(()) 
    }

    pub fn get_weights_set_rate_limit( netuid: u16) -> u64 { WeightsSetRateLimit::<T>::get( netuid ) }
    pub fn set_weights_set_rate_limit( netuid: u16, weights_set_rate_limit: u64 ) { WeightsSetRateLimit::<T>::insert( netuid, weights_set_rate_limit ); }
    pub fn do_sudo_set_weights_set_rate_limit( origin: T::RuntimeOrigin, netuid: u16, weights_set_rate_limit: u64 ) -> DispatchResult { 
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_weights_set_rate_limit( netuid, weights_set_rate_limit );
        log::info!("WeightsSetRateLimitSet( netuid: {:?} weights_set_rate_limit: {:?} ) ", netuid, weights_set_rate_limit);
        Self::deposit_event( Event::WeightsSetRateLimitSet( netuid, weights_set_rate_limit) );
        Ok(()) 
    }

    pub fn get_adjustment_interval( netuid: u16) -> u16 { AdjustmentInterval::<T>::get( netuid ) }
    pub fn set_adjustment_interval( netuid: u16, adjustment_interval: u16 ) { AdjustmentInterval::<T>::insert( netuid, adjustment_interval ); }
    pub fn do_sudo_set_adjustment_interval( origin: T::RuntimeOrigin, netuid: u16, adjustment_interval: u16 ) -> DispatchResult { 
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_adjustment_interval( netuid, adjustment_interval );
        log::info!("AdjustmentIntervalSet( netuid: {:?} adjustment_interval: {:?} ) ", netuid, adjustment_interval);
        Self::deposit_event( Event::AdjustmentIntervalSet( netuid, adjustment_interval) );
        Ok(()) 
    }

    pub fn get_validator_exclude_quantile( netuid: u16 ) -> u16 { ValidatorExcludeQuantile::<T>::get( netuid ) }
    pub fn set_validator_exclude_quantile( netuid: u16, validator_exclude_quantile: u16 ) { ValidatorExcludeQuantile::<T>::insert( netuid, validator_exclude_quantile ); }
    pub fn do_sudo_set_validator_exclude_quantile( origin:T::RuntimeOrigin, netuid: u16, validator_exclude_quantile: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );
        ensure!( validator_exclude_quantile <= 100, Error::<T>::StorageValueOutOfRange ); // The quantile must be between 0 and 100 => 0% and 100%
        Self::set_validator_exclude_quantile( netuid, validator_exclude_quantile );
        log::info!("ValidatorExcludeQuantileSet( netuid: {:?} validator_exclude_quantile: {:?} ) ", netuid, validator_exclude_quantile);
        Self::deposit_event( Event::ValidatorExcludeQuantileSet( netuid, validator_exclude_quantile ));
        Ok(())
    }

    pub fn get_validator_prune_len( netuid: u16 ) -> u64 { ValidatorPruneLen::<T>::get( netuid ) }
    pub fn set_validator_prune_len( netuid: u16, validator_prune_len: u64 ) { ValidatorPruneLen::<T>::insert( netuid, validator_prune_len ); }
    pub fn do_sudo_set_validator_prune_len( origin:T::RuntimeOrigin, netuid: u16, validator_prune_len: u64 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );
        Self::set_validator_prune_len(netuid, validator_prune_len);
        log::info!("ValidatorPruneLenSet( netuid: {:?} validator_prune_len: {:?} ) ", netuid, validator_prune_len);
		Self::deposit_event( Event::ValidatorPruneLenSet( netuid, validator_prune_len ));
		Ok(())
    }

    pub fn get_validator_logits_divergence( netuid: u16 ) -> u16 { ValidatorLogitsDivergence::<T>::get( netuid ) }
    pub fn set_validator_logits_divergence( netuid: u16, validator_logits_divergence: u16 ) { ValidatorLogitsDivergence::<T>::insert( netuid, validator_logits_divergence ); }
    pub fn do_sudo_set_validator_logits_divergence( origin:T::RuntimeOrigin, netuid: u16, validator_logits_divergence: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );
        Self::set_validator_logits_divergence(netuid, validator_logits_divergence);
        log::info!("ValidatorLogitsDivergenceSet( netuid: {:?} validator_logits_divergence: {:?} ) ", netuid, validator_logits_divergence);
        Self::deposit_event( Event::ValidatorLogitsDivergenceSet( netuid, validator_logits_divergence ));
        Ok(())
    }

    pub fn get_scaling_law_power( netuid: u16 ) -> u16 { ScalingLawPower::<T>::get( netuid ) }
    pub fn set_scaling_law_power( netuid: u16, scaling_law_power: u16 ) { ScalingLawPower::<T>::insert( netuid, scaling_law_power ); }
    pub fn do_sudo_set_scaling_law_power( origin:T::RuntimeOrigin, netuid: u16, scaling_law_power: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );
        ensure!( scaling_law_power <= 100, Error::<T>::StorageValueOutOfRange ); // The scaling law power must be between 0 and 100 => 0% and 100%
        Self::set_scaling_law_power( netuid, scaling_law_power );
        log::info!("ScalingLawPowerSet( netuid: {:?} scaling_law_power: {:?} ) ", netuid, scaling_law_power);
        Self::deposit_event( Event::ScalingLawPowerSet( netuid, scaling_law_power ));
        Ok(())
    }

    pub fn get_synergy_scaling_law_power( netuid: u16 ) -> u16 { SynergyScalingLawPower::<T>::get( netuid ) }
    pub fn set_synergy_scaling_law_power( netuid: u16, synergy_scaling_law_power: u16 ) { SynergyScalingLawPower::<T>::insert( netuid, synergy_scaling_law_power ); }
    pub fn do_sudo_set_synergy_scaling_law_power( origin:T::RuntimeOrigin, netuid: u16, synergy_scaling_law_power: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );
        ensure!( synergy_scaling_law_power <= 100, Error::<T>::StorageValueOutOfRange ); // The synergy scaling law power must be between 0 and 100 => 0% and 100%
        Self::set_synergy_scaling_law_power( netuid, synergy_scaling_law_power );
        log::info!("SynergyScalingLawPowerSet( netuid: {:?} synergy_scaling_law_power: {:?} ) ", netuid, synergy_scaling_law_power);
        Self::deposit_event( Event::SynergyScalingLawPowerSet( netuid, synergy_scaling_law_power ));
        Ok(())
    }

    pub fn get_max_weight_limit( netuid: u16) -> u16 { MaxWeightsLimit::<T>::get( netuid ) }    
    pub fn set_max_weight_limit( netuid: u16, max_weight_limit: u16 ) { MaxWeightsLimit::<T>::insert( netuid, max_weight_limit ); }
    pub fn do_sudo_set_max_weight_limit( origin:T::RuntimeOrigin, netuid: u16, max_weight_limit: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );
        Self::set_max_weight_limit( netuid, max_weight_limit );
        log::info!("MaxWeightLimitSet( netuid: {:?} max_weight_limit: {:?} ) ", netuid, max_weight_limit);
        Self::deposit_event( Event::MaxWeightLimitSet( netuid, max_weight_limit ) );
        Ok(())
    }

    pub fn get_immunity_period(netuid: u16 ) -> u16 { ImmunityPeriod::<T>::get( netuid ) }
    pub fn set_immunity_period( netuid: u16, immunity_period: u16 ) { ImmunityPeriod::<T>::insert( netuid, immunity_period ); }
    pub fn do_sudo_set_immunity_period( origin:T::RuntimeOrigin, netuid: u16, immunity_period: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_immunity_period( netuid, immunity_period );
        log::info!("ImmunityPeriodSet( netuid: {:?} immunity_period: {:?} ) ", netuid, immunity_period);
        Self::deposit_event(Event::ImmunityPeriodSet(netuid, immunity_period));
        Ok(())
    }

    pub fn get_validator_epochs_per_reset( netuid: u16 )-> u16 { ValidatorEpochsPerReset::<T>::get( netuid ) }
    pub fn set_validator_epochs_per_reset( netuid: u16, validator_epochs_per_reset: u16 ) { ValidatorEpochsPerReset::<T>::insert( netuid, validator_epochs_per_reset ); }
    pub fn do_sudo_set_validator_epochs_per_reset( origin:T::RuntimeOrigin, netuid: u16, validator_epochs_per_reset: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_validator_epochs_per_reset( netuid, validator_epochs_per_reset );
        log::info!("ValidatorEpochPerResetSet( netuid: {:?} validator_epochs_per_reset: {:?} ) ", netuid, validator_epochs_per_reset );
        Self::deposit_event(Event::ValidatorEpochPerResetSet(netuid, validator_epochs_per_reset));
        Ok(())
    }

    pub fn get_validator_sequence_length( netuid: u16 )-> u16 {ValidatorSequenceLength::<T>::get( netuid ) }
    pub fn set_validator_sequence_length( netuid: u16, validator_sequence_length: u16 ) { ValidatorSequenceLength::<T>::insert( netuid, validator_sequence_length ); }
    pub fn do_sudo_set_validator_sequence_length( origin:T::RuntimeOrigin, netuid: u16, validator_sequence_length: u16 ) -> DispatchResult {
        ensure_root( origin )?; 
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        ValidatorSequenceLength::<T>::insert( netuid, validator_sequence_length );
        log::info!("ValidatorSequenceLengthSet( netuid: {:?} validator_sequence_length: {:?} ) ", netuid, validator_sequence_length );
        Self::deposit_event(Event::ValidatorSequenceLengthSet(netuid, validator_sequence_length));
        Ok(())
    }


    pub fn get_validator_epoch_length( netuid: u16 )-> u16 {ValidatorEpochLen::<T>::get( netuid ) }
    pub fn set_validator_epoch_length( netuid: u16, validator_epoch_length: u16 ) { ValidatorEpochLen::<T>::insert( netuid, validator_epoch_length ); }
    pub fn do_sudo_set_validator_epoch_length( origin:T::RuntimeOrigin, netuid: u16, validator_epoch_length: u16 ) -> DispatchResult {
        ensure_root( origin )?; 
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        ValidatorEpochLen::<T>::insert( netuid, validator_epoch_length );
        log::info!("ValidatorEpochLengthSet( netuid: {:?} validator_epoch_length: {:?} ) ", netuid, validator_epoch_length );
        Self::deposit_event(Event::ValidatorEpochLengthSet(netuid, validator_epoch_length));
        Ok(())
    }

    pub fn get_validator_batch_size( netuid: u16 ) -> u16 { ValidatorBatchSize::<T>::get( netuid ) }
    pub fn set_validator_batch_size( netuid: u16, validator_batch_size: u16 ) { ValidatorBatchSize::<T>::insert( netuid, validator_batch_size ); }
    pub fn do_sudo_set_validator_batch_size( origin:T::RuntimeOrigin, netuid: u16, validator_batch_size: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_validator_batch_size( netuid, validator_batch_size );
        log::info!("ValidatorBatchSizeSet( netuid: {:?} validator_batch_size: {:?} ) ", netuid, validator_batch_size);
        Self::deposit_event(Event::ValidatorBatchSizeSet(netuid, validator_batch_size));
        Ok(())
    }
            
    pub fn get_min_allowed_weights( netuid:u16 ) -> u16 { MinAllowedWeights::<T>::get( netuid ) }
    pub fn set_min_allowed_weights( netuid: u16, min_allowed_weights: u16 ) { MinAllowedWeights::<T>::insert( netuid, min_allowed_weights ); }
    pub fn do_sudo_set_min_allowed_weights( origin:T::RuntimeOrigin, netuid: u16, min_allowed_weights: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_min_allowed_weights( netuid, min_allowed_weights );
        log::info!("MinAllowedWeightSet( netuid: {:?} min_allowed_weights: {:?} ) ", netuid, min_allowed_weights);
        Self::deposit_event( Event::MinAllowedWeightSet( netuid, min_allowed_weights) );
        Ok(())
    }

    pub fn get_max_allowed_uids( netuid: u16 ) -> u16  { MaxAllowedUids::<T>::get( netuid ) }
    pub fn set_max_allowed_uids(netuid: u16, max_allowed: u16) { MaxAllowedUids::<T>::insert( netuid, max_allowed ); }
    pub fn do_sudo_set_max_allowed_uids( origin:T::RuntimeOrigin, netuid: u16, max_allowed_uids: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist );
        ensure!(Self::get_max_allowed_uids(netuid)< max_allowed_uids, Error::<T>::MaxAllowedUIdsNotAllowed);
        Self::set_max_allowed_uids( netuid, max_allowed_uids );
        log::info!("MaxAllowedUidsSet( netuid: {:?} max_allowed_uids: {:?} ) ", netuid, max_allowed_uids);
        Self::deposit_event( Event::MaxAllowedUidsSet( netuid, max_allowed_uids) );
        Ok(())
    }

    pub fn get_kappa( netuid: u16 ) -> u16  { Kappa::<T>::get( netuid ) }
    pub fn set_kappa( netuid: u16, kappa: u16 ) { Kappa::<T>::insert( netuid, kappa ); }
    pub fn do_sudo_set_kappa( origin:T::RuntimeOrigin, netuid: u16, kappa: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_kappa( netuid, kappa );
        log::info!("KappaSet( netuid: {:?} kappa: {:?} ) ", netuid, kappa );
        Self::deposit_event( Event::KappaSet( netuid, kappa) );
        Ok(())
    }
            
    pub fn get_rho( netuid: u16 ) -> u16  { Rho::<T>::get( netuid ) }
    pub fn set_rho( netuid: u16, rho: u16 ) { Rho::<T>::insert( netuid, rho ); }
    pub fn do_sudo_set_rho( origin:T::RuntimeOrigin, netuid: u16, rho: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_rho( netuid, rho );
        log::info!("RhoSet( netuid: {:?} rho: {:?} ) ", netuid, rho );
        Self::deposit_event( Event::RhoSet( netuid, rho ) );
        Ok(())
    }
            
    pub fn get_activity_cutoff( netuid: u16 ) -> u16  { ActivityCutoff::<T>::get( netuid ) }
    pub fn set_activity_cutoff( netuid: u16, activity_cutoff: u16 ) { ActivityCutoff::<T>::insert( netuid, activity_cutoff ); }
    pub fn do_sudo_set_activity_cutoff( origin:T::RuntimeOrigin, netuid: u16, activity_cutoff: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_activity_cutoff( netuid, activity_cutoff );
        log::info!("ActivityCutoffSet( netuid: {:?} activity_cutoff: {:?} ) ", netuid, activity_cutoff);
        Self::deposit_event( Event::ActivityCutoffSet( netuid, activity_cutoff) );
        Ok(())
    }
            
    pub fn get_target_registrations_per_interval( netuid: u16 ) -> u16 { TargetRegistrationsPerInterval::<T>::get( netuid ) }
    pub fn set_target_registrations_per_interval( netuid: u16, target_registrations_per_interval: u16 ) { TargetRegistrationsPerInterval::<T>::insert( netuid, target_registrations_per_interval ); }
    pub fn do_sudo_set_target_registrations_per_interval( origin:T::RuntimeOrigin, netuid: u16, target_registrations_per_interval: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_target_registrations_per_interval( netuid, target_registrations_per_interval );
        log::info!("RegistrationPerIntervalSet( netuid: {:?} target_registrations_per_interval: {:?} ) ", netuid, target_registrations_per_interval );
        Self::deposit_event( Event::RegistrationPerIntervalSet( netuid, target_registrations_per_interval) );
        Ok(())
    }

    pub fn get_burn_as_u64( netuid: u16 ) -> u64  { Burn::<T>::get( netuid ) }
    pub fn set_burn( netuid: u16, burn: u64 ) { Burn::<T>::insert( netuid, burn ); }
    pub fn do_sudo_set_burn( origin:T::RuntimeOrigin, netuid: u16, burn: u64 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_burn( netuid, burn );
        log::info!("BurnSet( netuid: {:?} burn: {:?} ) ", netuid, burn );
        Self::deposit_event( Event::BurnSet( netuid, burn ) );
        Ok(())
    }

    pub fn get_min_burn_as_u64( netuid: u16 ) -> u64  { MinBurn::<T>::get( netuid ) }
    pub fn set_min_burn( netuid: u16, min_burn: u64 ) { MinBurn::<T>::insert( netuid, min_burn ); }
    pub fn do_sudo_set_min_burn( origin:T::RuntimeOrigin, netuid: u16, min_burn: u64 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_min_burn( netuid, min_burn );
        log::info!("MinBurnSet( netuid: {:?} min_burn: {:?} ) ", netuid, min_burn );
        Self::deposit_event( Event::MinBurnSet( netuid, min_burn ) );
        Ok(())
    }

    pub fn get_max_burn_as_u64( netuid: u16 ) -> u64  { MaxBurn::<T>::get( netuid ) }
    pub fn set_max_burn( netuid: u16, max_burn: u64 ) { MaxBurn::<T>::insert( netuid, max_burn ); }
    pub fn do_sudo_set_max_burn( origin:T::RuntimeOrigin, netuid: u16, max_burn: u64 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_max_burn( netuid, max_burn );
        log::info!("MaxBurnSet( netuid: {:?} max_burn: {:?} ) ", netuid, max_burn );
        Self::deposit_event( Event::MaxBurnSet( netuid, max_burn ) );
        Ok(())
    }
    
    pub fn get_difficulty_as_u64( netuid: u16 ) -> u64  { Difficulty::<T>::get( netuid ) }
    pub fn set_difficulty( netuid: u16, difficulty: u64 ) { Difficulty::<T>::insert( netuid, difficulty ); }
    pub fn do_sudo_set_difficulty( origin:T::RuntimeOrigin, netuid: u16, difficulty: u64 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_difficulty( netuid, difficulty );
        log::info!("DifficultySet( netuid: {:?} difficulty: {:?} ) ", netuid, difficulty );
        Self::deposit_event( Event::DifficultySet( netuid, difficulty ) );
        Ok(())
    }
            
    pub fn get_max_allowed_validators( netuid: u16 ) -> u16  { MaxAllowedValidators::<T>::get( netuid ) }
    pub fn set_max_allowed_validators( netuid: u16, max_allowed_validators: u16 ) { MaxAllowedValidators::<T>::insert( netuid, max_allowed_validators ); }
    pub fn do_sudo_set_max_allowed_validators( origin:T::RuntimeOrigin, netuid: u16, max_allowed_validators: u16 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_max_allowed_validators( netuid, max_allowed_validators );
        log::info!("MaxAllowedValidatorsSet( netuid: {:?} max_allowed_validators: {:?} ) ", netuid, max_allowed_validators );
        Self::deposit_event( Event::MaxAllowedValidatorsSet( netuid, max_allowed_validators ) );
        Ok(())
    }

    pub fn get_bonds_moving_average( netuid: u16 ) -> u64 { BondsMovingAverage::<T>::get( netuid ) }
    pub fn set_bonds_moving_average( netuid: u16, bonds_moving_average: u64 ) { BondsMovingAverage::<T>::insert( netuid, bonds_moving_average ); }
    pub fn do_sudo_set_bonds_moving_average( origin:T::RuntimeOrigin, netuid: u16, bonds_moving_average: u64 ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_bonds_moving_average( netuid, bonds_moving_average );
        log::info!("BondsMovingAverageSet( netuid: {:?} bonds_moving_average: {:?} ) ", netuid, bonds_moving_average );
        Self::deposit_event( Event::BondsMovingAverageSet( netuid, bonds_moving_average ) );
        Ok(())
    }

    pub fn get_max_registrations_per_block( netuid: u16 ) -> u16 { MaxRegistrationsPerBlock::<T>::get( netuid ) }
    pub fn set_max_registrations_per_block( netuid: u16, max_registrations_per_block: u16 ) { MaxRegistrationsPerBlock::<T>::insert( netuid, max_registrations_per_block ); }
    pub fn do_sudo_set_max_registrations_per_block(
        origin: T::RuntimeOrigin, 
        netuid: u16, 
        max_registrations_per_block: u16
    ) -> DispatchResult {
        ensure_root( origin )?;
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::NetworkDoesNotExist);
        Self::set_max_registrations_per_block( netuid, max_registrations_per_block );
        log::info!("MaxRegistrationsPerBlock( netuid: {:?} max_registrations_per_block: {:?} ) ", netuid, max_registrations_per_block );
        Self::deposit_event( Event::MaxRegistrationsPerBlockSet( netuid, max_registrations_per_block) );
        Ok(())
    }

}


