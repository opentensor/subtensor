
impl<T: Config> Pallet<T> 
{
    // Fetches the total count of subnet validators (those that set weights.)
    //
    // This function retrieves the total number of subnet validators.
    //
    // # Returns:
    // * 'u16': The total number of validators
    //
    pub fn get_max_subnets() -> u16 
    {
        return SubnetLimit::<T>::get();
    }

    pub fn set_max_subnets(limit: u16)
    {
        SubnetLimit::<T>::put(limit);

        Self::deposit_event(Event::SubnetLimitSet(limit));
    }

    pub fn set_tempo(netuid: u16, tempo: u16) 
    {
        Tempo::<T>::insert(netuid, tempo);

        Self::deposit_event(Event::TempoSet(netuid, tempo));
    }

    pub fn set_last_adjustment_block(netuid: u16, last_adjustment_block: u64) 
    {
        LastAdjustmentBlock::<T>::insert(netuid, last_adjustment_block);
    }

    pub fn set_blocks_since_last_step(netuid: u16, blocks_since_last_step: u64) 
    {
        BlocksSinceLastStep::<T>::insert(netuid, blocks_since_last_step);
    }

    pub fn set_registrations_this_block(netuid: u16, registrations_this_block: u16) 
    {
        RegistrationsThisBlock::<T>::insert(netuid, registrations_this_block);
    }

    pub fn set_last_mechanism_step_block(netuid: u16, last_mechanism_step_block: u64) 
    {
        LastMechansimStepBlock::<T>::insert(netuid, last_mechanism_step_block);
    }

    pub fn set_registrations_this_interval(netuid: u16, registrations_this_interval: u16) 
    {
        RegistrationsThisInterval::<T>::insert(netuid, registrations_this_interval);
    }

    pub fn set_pow_registrations_this_interval(netuid: u16, pow_registrations_this_interval: u16) 
    {
        POWRegistrationsThisInterval::<T>::insert(netuid, pow_registrations_this_interval);
    }

    pub fn set_burn_registrations_this_interval(netuid: u16, burn_registrations_this_interval: u16) 
    {
        BurnRegistrationsThisInterval::<T>::insert(netuid, burn_registrations_this_interval);
    }

    pub fn get_rank(netuid: u16) -> Vec<u16> 
    {
        return Rank::<T>::get(netuid);
    }

    pub fn get_trust(netuid: u16) -> Vec<u16> 
    {
        return Trust::<T>::get(netuid);
    }

    pub fn get_active(netuid: u16) -> Vec<bool> 
    {
        return Active::<T>::get(netuid);
    }

    pub fn get_emission(netuid: u16) -> Vec<u64> 
    {
        return Emission::<T>::get(netuid);
    }

    pub fn get_consensus(netuid: u16) -> Vec<u16> 
    {
        return Consensus::<T>::get(netuid);
    }

    pub fn get_incentive(netuid: u16) -> Vec<u16> 
    {
        return Incentive::<T>::get(netuid);
    }

    pub fn get_dividends(netuid: u16) -> Vec<u16> 
    {
        return Dividends::<T>::get(netuid);
    }

    pub fn get_last_update(netuid: u16) -> Vec<u64> 
    {
        return LastUpdate::<T>::get(netuid);
    }

    pub fn get_pruning_score(netuid: u16) -> Vec<u16> 
    {
        return PruningScores::<T>::get(netuid);
    }

    pub fn get_validator_trust(netuid: u16) -> Vec<u16> 
    {
        ValidatorTrust::<T>::get(netuid)
    }

    pub fn get_validator_permit(netuid: u16) -> Vec<bool> 
    {
        return ValidatorPermit::<T>::get(netuid);
    }

    pub fn get_tempo(netuid: u16) -> u16 
    {
        return Tempo::<T>::get(netuid);
    }

    pub fn get_emission_value(netuid: u16) -> u64 
    {
        return EmissionValues::<T>::get(netuid);
    }

    pub fn get_pending_emission(netuid: u16) -> u64 
    {
        return PendingEmission::<T>::get(netuid);
    }

    pub fn get_last_adjustment_block(netuid: u16) -> u64 
    {
        return LastAdjustmentBlock::<T>::get(netuid)
    }

    pub fn get_blocks_since_last_step(netuid: u16) -> u64 
    {
        return BlocksSinceLastStep::<T>::get(netuid)
    }

    pub fn get_difficulty(netuid: u16) -> U256
    {
        return U256::from(Self::get_difficulty_as_u64(netuid));
    }

    pub fn get_registrations_this_block(netuid: u16) -> u16 
    {
        return RegistrationsThisBlock::<T>::get(netuid);
    }

    pub fn get_last_mechanism_step_block(netuid: u16) -> u64 
    {
        return LastMechansimStepBlock::<T>::get(netuid);
    }

    pub fn get_registrations_this_interval(netuid: u16) -> u16 
    {
        return RegistrationsThisInterval::<T>::get(netuid);
    }

    pub fn get_pow_registrations_this_interval(netuid: u16) -> u16 
    {
        return POWRegistrationsThisInterval::<T>::get(netuid);
    }

    pub fn get_burn_registrations_this_interval(netuid: u16) -> u16 
    {
        return BurnRegistrationsThisInterval::<T>::get(netuid);
    }

    pub fn get_neuron_block_at_registration(netuid: u16, neuron_uid: u16) -> u64 
    {
        return BlockAtRegistration::<T>::get(netuid, neuron_uid);
    }

    pub fn get_adjustment_interval(netuid: u16) -> u16 
    {
        return AdjustmentInterval::<T>::get(netuid);
    }
    
    pub fn set_adjustment_interval(netuid: u16, adjustment_interval: u16) 
    {
        AdjustmentInterval::<T>::insert(netuid, adjustment_interval);

        Self::deposit_event(Event::AdjustmentIntervalSet(netuid, adjustment_interval));
    }

    pub fn get_adjustment_alpha(netuid: u16) -> u64 
    {
        return AdjustmentAlpha::<T>::get(netuid);
    }

    pub fn set_adjustment_alpha(netuid: u16, adjustment_alpha: u64) 
    {
        AdjustmentAlpha::<T>::insert(netuid, adjustment_alpha);

        Self::deposit_event(Event::AdjustmentAlphaSet(netuid, adjustment_alpha));
    }

    pub fn get_validator_prune_len(netuid: u16) -> u64 
    {
        return ValidatorPruneLen::<T>::get(netuid);
    }

    pub fn set_validator_prune_len(netuid: u16, validator_prune_len: u64) 
    {
        ValidatorPruneLen::<T>::insert(netuid, validator_prune_len);

        Self::deposit_event(Event::ValidatorPruneLenSet(netuid, validator_prune_len));
    }

    pub fn get_scaling_law_power(netuid: u16) -> u16 
    {
        return ScalingLawPower::<T>::get(netuid);
    }

    pub fn set_scaling_law_power(netuid: u16, scaling_law_power: u16) 
    {
        ScalingLawPower::<T>::insert(netuid, scaling_law_power);

        Self::deposit_event(Event::ScalingLawPowerSet(netuid, scaling_law_power));
    }

    pub fn get_immunity_period(netuid: u16) -> u16 
    {
        return ImmunityPeriod::<T>::get(netuid);
    }

    pub fn set_immunity_period(netuid: u16, immunity_period: u16) 
    {
        ImmunityPeriod::<T>::insert(netuid, immunity_period);

        Self::deposit_event(Event::ImmunityPeriodSet(netuid, immunity_period));
    }

    pub fn get_min_difficulty(netuid: u16) -> u64 
    {
        return MinDifficulty::<T>::get(netuid);
    }

    pub fn set_min_difficulty(netuid: u16, min_difficulty: u64) 
    {
        MinDifficulty::<T>::insert(netuid, min_difficulty);

        Self::deposit_event(Event::MinDifficultySet(netuid, min_difficulty));
    }

    pub fn get_max_difficulty(netuid: u16) -> u64 
    {
        return MaxDifficulty::<T>::get(netuid);
    }

    pub fn set_max_difficulty(netuid: u16, max_difficulty: u64) 
    {
        MaxDifficulty::<T>::insert(netuid, max_difficulty);

        Self::deposit_event(Event::MaxDifficultySet(netuid, max_difficulty));
    }

    pub fn get_kappa(netuid: u16) -> u16 
    {
        return Kappa::<T>::get(netuid);
    }

    pub fn set_kappa(netuid: u16, kappa: u16)
    {
        Kappa::<T>::insert(netuid, kappa);

        Self::deposit_event(Event::KappaSet(netuid, kappa));
    }

    pub fn get_rho(netuid: u16) -> u16 
    {
        return Rho::<T>::get(netuid);
    }

    pub fn set_rho(netuid: u16, rho: u16) 
    {
        Rho::<T>::insert(netuid, rho);
    }

    pub fn get_activity_cutoff(netuid: u16) -> u16 
    {
        return ActivityCutoff::<T>::get(netuid);
    }

    pub fn set_activity_cutoff(netuid: u16, activity_cutoff: u16) 
    {
        ActivityCutoff::<T>::insert(netuid, activity_cutoff);
        Self::deposit_event(Event::ActivityCutoffSet(netuid, activity_cutoff));
    }

    pub fn get_subnet_owner(netuid: u16) -> T::AccountId 
    {
        return SubnetOwner::<T>::get(netuid);
    }

    pub fn get_subnet_owner_cut() -> u16 
    {
        return SubnetOwnerCut::<T>::get();
    }

    pub fn set_subnet_owner_cut(subnet_owner_cut: u16) 
    {
        SubnetOwnerCut::<T>::set(subnet_owner_cut);

        Self::deposit_event(Event::SubnetOwnerCutSet(subnet_owner_cut));
    }

    pub fn get_burn_as_u64(netuid: u16) -> u64 
    {
        return Burn::<T>::get(netuid);
    }

    pub fn set_burn(netuid: u16, burn: u64) 
    {
        Burn::<T>::insert(netuid, burn);
    }

    pub fn get_min_burn_as_u64(netuid: u16) -> u64 
    {
        return MinBurn::<T>::get(netuid);
    }

    pub fn set_min_burn(netuid: u16, min_burn: u64) 
    {
        MinBurn::<T>::insert(netuid, min_burn);

        Self::deposit_event(Event::MinBurnSet(netuid, min_burn));
    }

    pub fn get_max_burn_as_u64(netuid: u16) -> u64 
    {
        return MaxBurn::<T>::get(netuid);
    }

    pub fn set_max_burn(netuid: u16, max_burn: u64) 
    {
        MaxBurn::<T>::insert(netuid, max_burn);

        Self::deposit_event(Event::MaxBurnSet(netuid, max_burn));
    }

    pub fn get_difficulty_as_u64(netuid: u16) -> u64 
    {
        return Difficulty::<T>::get(netuid);
    }
    
    pub fn set_difficulty(netuid: u16, difficulty: u64) 
    {
        Difficulty::<T>::insert(netuid, difficulty);

        Self::deposit_event(Event::DifficultySet(netuid, difficulty));
    }

    pub fn get_max_allowed_validators(netuid: u16) -> u16 
    {
        return MaxAllowedValidators::<T>::get(netuid);
    }

    pub fn set_max_allowed_validators(netuid: u16, max_allowed_validators: u16) 
    {
        MaxAllowedValidators::<T>::insert(netuid, max_allowed_validators);

        Self::deposit_event(Event::MaxAllowedValidatorsSet(netuid, max_allowed_validators));
    }

    pub fn get_bonds_moving_average(netuid: u16) -> u64 
    {
        return BondsMovingAverage::<T>::get(netuid);
    }

    pub fn set_bonds_moving_average(netuid: u16, bonds_moving_average: u64) 
    {
        BondsMovingAverage::<T>::insert(netuid, bonds_moving_average);

        Self::deposit_event(Event::BondsMovingAverageSet(netuid, bonds_moving_average));
    }

    pub fn set_total_issuance(total_issuance: u64) 
    {
        TotalIssuance::<T>::put(total_issuance);
    }

    pub fn do_set_total_issuance(origin: T::RuntimeOrigin, total_issuance: u64) -> DispatchResult 
    {
        ensure_root(origin)?;
        TotalIssuance::<T>::put(total_issuance);

        return Ok(());
    }

    pub fn get_rao_recycled(netuid: u16) -> u64 
    {
        return RAORecycledForRegistration::<T>::get(netuid);
    }

    pub fn set_rao_recycled(netuid: u16, rao_recycled: u64) 
    {
        RAORecycledForRegistration::<T>::insert(netuid, rao_recycled);
        Self::deposit_event(Event::RAORecycledForRegistrationSet(netuid, rao_recycled));
    }

    pub fn increase_rao_recycled(netuid: u16, inc_rao_recycled: u64) 
    {
        Self::set_rao_recycled(netuid, Self::get_rao_recycled(netuid).saturating_add(inc_rao_recycled));
    }

    pub fn get_network_registered_block(netuid: u16) -> u64 
    {
        NetworkRegisteredAt::<T>::get(netuid)
    }

    pub fn get_network_immunity_period() -> u64 
    {
        NetworkImmunityPeriod::<T>::get()
    }

    pub fn set_network_immunity_period(net_immunity_period: u64) 
    {
        NetworkImmunityPeriod::<T>::set(net_immunity_period);

        Self::deposit_event(Event::NetworkImmunityPeriodSet(net_immunity_period));
    }

    pub fn set_network_min_lock(net_min_lock: u64) 
    {
        NetworkMinLockCost::<T>::set(net_min_lock);

        Self::deposit_event(Event::NetworkMinLockCostSet(net_min_lock));
    }

    pub fn get_network_min_lock() -> u64 
    {
        NetworkMinLockCost::<T>::get()
    }

    pub fn set_network_last_lock(net_last_lock: u64) 
    {
        NetworkLastLockCost::<T>::set(net_last_lock);
    }

    pub fn get_network_last_lock() -> u64 
    {
        NetworkLastLockCost::<T>::get()
    }

    pub fn get_network_last_lock_block() -> u64 
    {
        NetworkLastRegistered::<T>::get()
    }

    pub fn set_lock_reduction_interval(interval: u64) 
    {
        NetworkLockReductionInterval::<T>::set(interval);

        Self::deposit_event(Event::NetworkLockCostReductionIntervalSet(interval));
    }

    pub fn get_lock_reduction_interval() -> u64 
    {
        NetworkLockReductionInterval::<T>::get()
    }

    pub fn is_subnet_owner(address: &T::AccountId) -> bool 
    {
        SubnetOwner::<T>::iter_values().any(|owner| *address == owner)
    }
    
    // Returns true if the subnetwork exists.
    //
    // This function checks if a subnetwork with the given UID exists.
    //
    // # Returns:
    // * 'bool': Whether the subnet exists.
    //
    pub fn if_subnet_exist(netuid: u16) -> bool 
    {
        return NetworksAdded::<T>::get(netuid);
    }


    // Sets initial and custom parameters for a new network.
    pub fn init_new_network(netuid: u16, tempo: u16) 
    {
        // --- 1. Set network to 0 size.
        {
            SubnetworkN::<T>::insert(netuid, 0);
        }

        // --- 2. Set this network uid to alive.
        {
            NetworksAdded::<T>::insert(netuid, true);
        }

        // --- 3. Fill tempo memory item.
        {
            Tempo::<T>::insert(netuid, tempo);
        }

        // --- 4 Fill modality item.
        {
            NetworkModality::<T>::insert(netuid, 0);
        }

        // --- 5. Increase total network count.
        {
            TotalNetworks::<T>::mutate(|n| *n += 1);
        }

        // --- 6. Set all default values **explicitly**.
        {
            Self::set_network_registration_allowed(netuid, NetworkRegistrationAllowed::<T>::get(netuid));
            Self::set_max_allowed_uids(netuid, MaxAllowedUids::<T>::get(netuid));
            Self::set_max_allowed_validators(netuid, MaxAllowedValidators::<T>::get(netuid));
            Self::set_min_allowed_weights(netuid, MinAllowedWeights::<T>::get(netuid));
            Self::set_max_weight_limit(netuid, MaxWeightLimit::<T>::get(netuid));
            Self::set_adjustment_interval(netuid, AdjustmentInterval::<T>::get(netuid));
            Self::set_target_registrations_per_interval(netuid, TargetRegistrationsPerInterval::<T>::get(netuid));
            Self::set_adjustment_alpha(netuid, AdjustmentAlpha::<T>::get(netuid));
            Self::set_immunity_period(netuid, ImmunityPeriod::<T>::get(netuid));
            Self::set_min_burn(netuid, MinBurn::<T>::get(netuid));
            Self::set_min_difficulty(netuid, MinDifficulty::<T>::get(netuid));
            Self::set_max_difficulty(netuid, MaxDifficulty::<T>::get(netuid));
        }

        // 7. Make network parameters explicit.
        {
            if !Tempo::<T>::contains_key(netuid) 
            {
                Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));
            }

            if !Kappa::<T>::contains_key(netuid) 
            {
                Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));
            }

            if !Difficulty::<T>::contains_key(netuid) 
            {
                Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));
            }

            if !MaxAllowedUids::<T>::contains_key(netuid) 
            {
                MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));
            }

            if !ImmunityPeriod::<T>::contains_key(netuid) 
            {
                ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));
            }

            if !ActivityCutoff::<T>::contains_key(netuid) 
            {
                ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));
            }

            if !EmissionValues::<T>::contains_key(netuid) 
            {
                EmissionValues::<T>::insert(netuid, EmissionValues::<T>::get(netuid));
            }

            if !MaxWeightsLimit::<T>::contains_key(netuid) 
            {
                MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));
            }
            
            if !MinAllowedWeights::<T>::contains_key(netuid) 
            {
                MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid));
            }

            if !RegistrationsThisInterval::<T>::contains_key(netuid) 
            {
                RegistrationsThisInterval::<T>::insert(
                    netuid,
                    RegistrationsThisInterval::<T>::get(netuid),
                );
            }

            if !POWRegistrationsThisInterval::<T>::contains_key(netuid) 
            {
                POWRegistrationsThisInterval::<T>::insert(
                    netuid,
                    POWRegistrationsThisInterval::<T>::get(netuid),
                );
            }

            if !BurnRegistrationsThisInterval::<T>::contains_key(netuid) 
            {
                BurnRegistrationsThisInterval::<T>::insert(
                    netuid,
                    BurnRegistrationsThisInterval::<T>::get(netuid),
                );
            }
        }
    }

    // Removes a network (identified by netuid) and all associated parameters.
    //
    // This function is responsible for cleaning up all the data associated with a network.
    // It ensures that all the storage values related to the network are removed, and any
    // reserved balance is returned to the network owner.
    //
    // # Args:
    // 	* 'netuid': ('u16'): The unique identifier of the network to be removed.
    //
    // # Note:
    // This function does not emit any events, nor does it raise any errors. It silently
    // returns if any internal checks fail.
    //
    pub fn remove_network(netuid: u16) 
    {
        // --- 1. Return balance to subnet owner.
        let owner_coldkey: T::AccountId;
        let reserved_amount_as_bal;
        {
            owner_coldkey = SubnetOwner::<T>::get(netuid);
            let reserved_amount = Self::get_subnet_locked_balance(netuid);

            // Ensure that we can convert this u64 to a balance.
            reserved_amount_as_bal = Self::u64_to_balance(reserved_amount);
            if !reserved_amount_as_bal.is_some() 
            {
                return;
            }
        }

        // --- 2. Remove network count.
        {
            SubnetworkN::<T>::remove(netuid);
        }

        // --- 3. Remove network modality storage.
        {
            NetworkModality::<T>::remove(netuid);
        }

        // --- 4. Remove netuid from added networks.
        {
            NetworksAdded::<T>::remove(netuid);
        }

        // --- 6. Decrement the network counter.
        {
            TotalNetworks::<T>::mutate(|n| *n -= 1);
        }

        // --- 7. Remove various network-related storages.
        {
            NetworkRegisteredAt::<T>::remove(netuid);
        }

        // --- 8. Remove incentive mechanism memory.
        {
            let _ = Uids::<T>::clear_prefix(netuid, u32::max_value(), None);
            let _ = Keys::<T>::clear_prefix(netuid, u32::max_value(), None);
            let _ = Bonds::<T>::clear_prefix(netuid, u32::max_value(), None);
            let _ = Weights::<T>::clear_prefix(netuid, u32::max_value(), None);
        }

        // --- 9. Remove various network-related parameters.
        {
            Rank::<T>::remove(netuid);
            Trust::<T>::remove(netuid);
            Active::<T>::remove(netuid);
            Emission::<T>::remove(netuid);
            Incentive::<T>::remove(netuid);
            Consensus::<T>::remove(netuid);
            Dividends::<T>::remove(netuid);
            PruningScores::<T>::remove(netuid);
            LastUpdate::<T>::remove(netuid);
            ValidatorPermit::<T>::remove(netuid);
            ValidatorTrust::<T>::remove(netuid);
        }

        // --- 10. Erase network parameters.
        {
            Tempo::<T>::remove(netuid);
            Kappa::<T>::remove(netuid);
            Difficulty::<T>::remove(netuid);
            MaxAllowedUids::<T>::remove(netuid);
            ImmunityPeriod::<T>::remove(netuid);
            ActivityCutoff::<T>::remove(netuid);
            EmissionValues::<T>::remove(netuid);
            MaxWeightsLimit::<T>::remove(netuid);
            MinAllowedWeights::<T>::remove(netuid);
            RegistrationsThisInterval::<T>::remove(netuid);
            POWRegistrationsThisInterval::<T>::remove(netuid);
            BurnRegistrationsThisInterval::<T>::remove(netuid);
        }

        // --- 11. Add the balance back to the owner.
        {
            Self::add_balance_to_coldkey_account(&owner_coldkey, reserved_amount_as_bal.unwrap());
            Self::set_subnet_locked_balance(netuid, 0);

            SubnetOwner::<T>::remove(netuid);
        }
    }

    pub fn get_float_rho(netuid: u16) -> I32F32 
    { 
        return I32F32::from_num(Self::get_rho(netuid));
    }

    pub fn get_float_kappa(netuid: u16) -> I32F32 
    { 
        return I32F32::from_num(Self::get_kappa(netuid)) / I32F32::from_num(u16::MAX);
    }

    pub fn get_normalized_stake(netuid: u16) -> Vec<I32F32> 
    {
        let n:              usize       = Self::get_subnetwork_n(netuid) as usize; 
        let mut stake_64:   Vec<I64F64> = vec![I64F64::from_num(0.0); n]; 
        for neuron_uid in 0..n 
        {
            stake_64[neuron_uid] = I64F64::from_num(Self::get_stake_for_uid_and_subnetwork(netuid, neuron_uid as u16));
        }

        inplace_normalize_64(&mut stake_64);
        
        return vec_fixed64_to_fixed32(stake_64);
    }

    pub fn get_block_at_registration(netuid: u16) -> Vec<u64> 
    { 
        let n:                          usize       = Self::get_subnetwork_n(netuid) as usize;
        let mut block_at_registration:  Vec<u64>    = vec![0; n];
        for neuron_uid in 0..n 
        {
            if Keys::<T>::contains_key(netuid, neuron_uid as u16)
            {
                block_at_registration[neuron_uid] = Self::get_neuron_block_at_registration(netuid, neuron_uid as u16);
            }
        }

        return block_at_registration;
    }

    // Output unnormalized sparse bonds, input bonds are assumed to be column max-upscaled in u16.
    pub fn get_bonds_sparse(netuid: u16) -> Vec<Vec<(u16, I32F32)>> 
    { 
        let n:          usize                   = Self::get_subnetwork_n(netuid) as usize; 
        let mut bonds:  Vec<Vec<(u16, I32F32)>> = vec![vec![]; n]; 
        for (uid_i, bonds_i) in <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid)
        {
            for (uid_j, bonds_ij) in bonds_i.iter() 
            { 
                bonds[uid_i as usize].push((
                    *uid_j, 
                    I32F32::from_num(*bonds_ij) 
                ));
            }
        }

        return bonds;
    } 

    // Output unnormalized bonds in [n, n] matrix, input bonds are assumed to be column max-upscaled in u16.
    pub fn get_bonds(netuid: u16) -> Vec<Vec<I32F32>> 
    { 
        let n:          usize               = Self::get_subnetwork_n(netuid) as usize; 
        let mut bonds:  Vec<Vec<I32F32>>    = vec![vec![ I32F32::from_num(0.0); n]; n]; 
        for (uid_i, bonds_i) in <Bonds<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(netuid) 
        {
            for (uid_j, bonds_ij) in bonds_i.iter() 
            {
                bonds[uid_i as usize][*uid_j as usize] = I32F32::from_num(*bonds_ij);
            }
        }
        
        return bonds;
    }
}