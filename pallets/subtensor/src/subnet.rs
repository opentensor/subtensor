use
{

};

impl<T: Config> Pallet<T> 
{
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

    pub fn is_subnet_owner(address: &T::AccountId) -> bool 
    {
        SubnetOwner::<T>::iter_values().any(|owner| *address == owner)
    }
}