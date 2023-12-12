use
{
};

impl<T: Config> Pallet<T> 
{
    // ==================================
    // ==== YumaConsensus UID params ====
    // ==================================
    pub fn set_last_update_for_uid(netuid: u16, uid: u16, last_update: u64) 
    {
        let mut updated_last_update_vec: Vec<u64> = Self::get_last_update(netuid);
        if (uid as usize) < updated_last_update_vec.len() 
        {
            updated_last_update_vec[uid as usize] = last_update;
            LastUpdate::<T>::insert(netuid, updated_last_update_vec);
        }
    }

    pub fn set_active_for_uid(netuid: u16, uid: u16, active: bool) 
    {
        let mut updated_active_vec: Vec<bool> = Self::get_active(netuid);
        if (uid as usize) < updated_active_vec.len() 
        {
            updated_active_vec[uid as usize] = active;
            Active::<T>::insert(netuid, updated_active_vec);
        }
    }

    pub fn set_pruning_score_for_uid(netuid: u16, uid: u16, pruning_score: u16) 
    {
        log::info!("netuid = {:?}", netuid);
        log::info!("SubnetworkN::<T>::get( netuid ) = {:?}", SubnetworkN::<T>::get(netuid));
        log::info!("uid = {:?}", uid);

        assert!(uid < SubnetworkN::<T>::get(netuid));

        PruningScores::<T>::mutate(netuid, |v| v[uid as usize] = pruning_score);
    }

    pub fn set_validator_permit_for_uid(netuid: u16, uid: u16, validator_permit: bool) 
    {
        let mut updated_validator_permit: Vec<bool> = Self::get_validator_permit(netuid);
        if (uid as usize) < updated_validator_permit.len() 
        {
            updated_validator_permit[uid as usize] = validator_permit;
            ValidatorPermit::<T>::insert(netuid, updated_validator_permit);
        }
    }

    pub fn get_rank_for_uid(netuid: u16, uid: u16) -> u16 
    {
        return Rank::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u16);
    }

    pub fn get_trust_for_uid(netuid: u16, uid: u16) -> u16 
    {
        return Trust::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u16);
    }

    pub fn get_emission_for_uid(netuid: u16, uid: u16) -> u64 
    {
        return Emission::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u64);
    }

    pub fn get_active_for_uid(netuid: u16, uid: u16) -> bool 
    {
        return Active::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(false);
    }

    pub fn get_consensus_for_uid(netuid: u16, uid: u16) -> u16
    {
        return Consensus::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u16);
    }

    pub fn get_incentive_for_uid(netuid: u16, uid: u16) -> u16 
    {
        return Incentive::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u16);
    }

    pub fn get_dividends_for_uid(netuid: u16, uid: u16) -> u16 
    {
        return Dividends::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u16);
    }

    pub fn get_last_update_for_uid(netuid: u16, uid: u16) -> u64 
    {
        return LastUpdate::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u64);
    }

    pub fn get_pruning_score_for_uid(netuid: u16, uid: u16) -> u16 
    {
        return PruningScores::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(u16::MAX);
    }

    pub fn get_validator_trust_for_uid(netuid: u16, uid: u16) -> u16 
    {
        return ValidatorTrust::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(0u16);
    }

    pub fn get_validator_permit_for_uid(netuid: u16, uid: u16) -> bool 
    {
        return ValidatorPermit::<T>::get(netuid).into_iter().nth(uid as usize).unwrap_or(false);
    }
}