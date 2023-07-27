use super::*;
use frame_support::sp_std::vec;
use sp_std::vec::Vec;
use frame_system::ensure_root;
use crate::math::checked_sum;

impl<T: Config> Pallet<T> { 

    // Register a new subnetwork as a user.
    pub fn user_add_network(
        origin: T::RuntimeOrigin,
        modality: u16,
        immunity_period: u16,
        reg_allowed: bool
    ) -> dispatch::DispatchResult {
        ensure_signed( origin )?;

        // Ensure the modality is valid.
        ensure!( Self::if_modality_is_valid( modality ), Error::<T>::InvalidModality );

        // Find next uid
        let netuid: u16 = {
            let total_networks = TotalNetworks::<T>::get();

            if total_networks < SubnetLimit::<T>::get() {
                total_networks + 1
            } else {
                Self::get_subnet_to_prune()
            }
        };

        // Clear network data
        Self::remove_network(netuid);

        // Create the subnet
        Self::init_new_network(netuid, 1000, modality);
        NetworkRegisteredAt::<T>::insert(netuid, Self::get_current_block_as_u64());

        // Emit the new network event.
        log::info!("NetworkAdded( netuid:{:?}, modality:{:?} )", netuid, modality);
        Self::deposit_event( Event::NetworkAdded( netuid, modality ) );

        // Set some configurable hyperparams for the network
        Self::set_immunity_period(netuid, immunity_period);
        Self::set_network_registration_allowed(netuid, reg_allowed);
        Self::set_max_allowed_uids(netuid, 128);
        Self::set_max_allowed_validators(netuid, 32);

        Ok(())
    }

    // ---- The implementation for the extrinsic add_network.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- Must be sudo.
    //
    // 	* 'netuid' (u16):
    // 		- The u16 network identifier.
    //
    // 	* 'tempo' ( u16 ):
    // 		- Number of blocks between epoch step.
    //
    // 	* 'modality' ( u16 ):
    // 		- Network modality specifier.
    //
    // # Event:
    // 	* NetworkAdded;
    // 		- On successfully creation of a network.
    //
    // # Raises:
    // 	* 'NetworkExist':
    // 		- Attempting to register an already existing.
    //
    // 	* 'InvalidModality':
    // 		- Attempting to register a network with an invalid modality.
    //
    // 	* 'InvalidTempo':
    // 		- Attempting to register a network with an invalid tempo.
    //
    pub fn do_add_network( 
        origin: T::RuntimeOrigin, 
        netuid: u16, 
        tempo: u16, 
        modality: u16 
    ) -> dispatch::DispatchResultWithPostInfo {

        // --- 1. Ensure this is a sudo caller.
        ensure_root( origin )?;

        // --- 2. Ensure this subnetwork does not already exist.
        ensure!( !Self::if_subnet_exist( netuid ), Error::<T>::NetworkExist );

        // --- 3. Ensure the modality is valid.
        ensure!( Self::if_modality_is_valid( modality ), Error::<T>::InvalidModality );

        // --- 4. Ensure the tempo is valid.
        ensure!( Self::if_tempo_is_valid( tempo ), Error::<T>::InvalidTempo );

        // --- 5. Initialize the network and all its parameters.
        Self::init_new_network( netuid, tempo, modality );
        
        // --- 6. Emit the new network event.
        log::info!("NetworkAdded( netuid:{:?}, modality:{:?} )", netuid, modality);
        Self::deposit_event( Event::NetworkAdded( netuid, modality ) );

        // --- 7. Ok and return.
        Ok(().into())
    }

    // ---- The implementation for the extrinsic remove_network.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- Must be sudo.
    //
    // 	* 'netuid' (u16):
    // 		- The u16 network identifier.
    //
    // # Event:
    // 	* NetworkRemoved;
    // 		- On the successfull removing of this network.
    //
    // # Raises:
    // 	* 'NetworkDoesNotExist':
    // 		- Attempting to remove a non existent network.
    //
    pub fn do_remove_network( origin: T::RuntimeOrigin, netuid: u16 ) -> dispatch::DispatchResult {

        // --- 1. Ensure the function caller it Sudo.
        ensure_root( origin )?;

        // --- 2. Ensure the network to be removed exists.
        ensure!( Self::if_subnet_exist( netuid ), Error::<T>::NetworkDoesNotExist );

        // --- 3. Explicitly erase the network and all its parameters.
        Self::remove_network( netuid );
    
        // --- 4. Emit the event.
        log::info!("NetworkRemoved( netuid:{:?} )", netuid);
        Self::deposit_event( Event::NetworkRemoved( netuid ) );

        // --- 5. Ok and return.
        Ok(())
    }

    // ---- The implementation for the extrinsic sudo_add_network_connect_requirement.
    // Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The caller, must be sudo.
    //
    // 	* `netuid_a` (u16):
    // 		- The network we are adding the requirment to (parent network)
    //
    // 	* `netuid_b` (u16):
    // 		- The network we the requirement refers to (child network)
    //
    // 	* `prunning_score_requirement` (u16):
    // 		- The topk percentile prunning score requirement (u16:MAX normalized.)
    //
    pub fn do_sudo_add_network_connection_requirement(
        origin: T::RuntimeOrigin, 
        netuid_a: u16,
        netuid_b: u16,
        requirement: u16
    ) -> dispatch::DispatchResult {
        ensure_root( origin )?;
        ensure!( netuid_a != netuid_b, Error::<T>::InvalidConnectionRequirement );
        ensure!( Self::if_subnet_exist( netuid_a ), Error::<T>::NetworkDoesNotExist );
        ensure!( Self::if_subnet_exist( netuid_b ), Error::<T>::NetworkDoesNotExist );
        Self::add_connection_requirement( netuid_a, netuid_b, requirement );
        log::info!("NetworkConnectionAdded( netuid_a:{:?}, netuid_b:{:?} requirement: {:?} )", netuid_a, netuid_b, requirement);
        Self::deposit_event( Event::NetworkConnectionAdded( netuid_a, netuid_b, requirement ) );
        Ok(())
    }

    // ---- The implementation for the extrinsic sudo_remove_network_connect_requirement.
    // Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The caller, must be sudo.
    //
    // 	* `netuid_a` (u16):
    // 		- The network we are removing the requirment from.
    //
    // 	* `netuid_b` (u16):
    // 		- The required network connection to remove.
    //   
    pub fn do_sudo_remove_network_connection_requirement(
        origin: T::RuntimeOrigin, 
        netuid_a: u16,
        netuid_b: u16,
    ) -> dispatch::DispatchResult {
        ensure_root( origin )?;
        ensure!( Self::if_subnet_exist( netuid_a ), Error::<T>::NetworkDoesNotExist );
        ensure!( Self::if_subnet_exist( netuid_b ), Error::<T>::NetworkDoesNotExist );
        Self::remove_connection_requirment( netuid_a, netuid_b );
        log::info!("NetworkConnectionRemoved( netuid_a:{:?}, netuid_b:{:?} )", netuid_a, netuid_b );
        Self::deposit_event( Event::NetworkConnectionRemoved( netuid_a, netuid_b ) );
        Ok(())
    }


    // ---- The implementation for the extrinsic set_emission_values.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- Must be sudo.
    //
   	// 	* `netuids` (Vec<u16>):
	// 		- A vector of network uids values. This must include all netuids.
	//
	// 	* `emission` (Vec<u64>):
	// 		- The emission values associated with passed netuids in order.
    //
    // # Event:
    // 	* NetworkRemoved;
    // 		- On the successfull removing of this network.
    //
    // # Raises:
    // 	* 'EmissionValuesDoesNotMatchNetworks':
    // 		- Attempting to remove a non existent network.
    //
    pub fn do_set_emission_values( 
        origin: T::RuntimeOrigin, 
        netuids: Vec<u16>,
        emission: Vec<u64>
    ) -> dispatch::DispatchResult {

        // --- 1. Ensure caller is sudo.
        ensure_root( origin )?;

        // --- 2. Ensure emission values match up to network uids.
        ensure!( netuids.len() == emission.len(), Error::<T>::WeightVecNotEqualSize );

        // --- 3. Ensure we are setting emission for all networks. 
        ensure!( netuids.len() == TotalNetworks::<T>::get() as usize, Error::<T>::IncorrectNetuidsLength );

        // --- 4. Ensure the passed uids contain no duplicates.
        ensure!( !Self::has_duplicate_netuids( &netuids ), Error::<T>::DuplicateUids );

        // --- 5. Ensure that the passed uids are valid for the network.
        ensure!( !Self::contains_invalid_netuids( &netuids ), Error::<T>::InvalidUid );

        // --- 6. check if sum of emission rates is equal to the block emission.
        // Be sure to check for overflow during sum.
        let emission_sum: Option<u64> = checked_sum::<u64>( &emission );
        ensure!( emission_sum.is_some(), Error::<T>::InvalidEmissionValues );
        ensure!( emission_sum.unwrap() == Self::get_block_emission(), Error::<T>::InvalidEmissionValues);

        // --- 7. Add emission values for each network
        Self::set_emission_values( &netuids, &emission );

        // --- 8. Add emission values for each network
        log::info!("EmissionValuesSet()");
        Self::deposit_event( Event::EmissionValuesSet() );

        // --- 9. Ok and return.
        Ok(())
    }

    // Initializes a new subnetwork under netuid with all hyperparameters.
    //
    pub fn init_new_network_ex( 
        netuid: u16,
        tempo: u16,
        modality: u16,
        serving_limit: u64,
        weights_limit: u64,
        weights_version: u64,
        weights_min: u16,
        weights_max: u16,
        min_burn: u64,
        max_burn: u64,
        bonds_moving_avg: u64,
        max_allowed_validators: u16,
        max_allowed_uids: u16,
        adjustment_interval: u16,
        adjustment_alpha: u64,
        target_reg_per_interval: u16,
        max_reg_per_block: u16,
        activity_cutoff: u16,
        rho: u16,
        kappa: u16,
        validator_prune_len: u64,
        scaling_law_power: u16,
        immunity_period: u16,
        reg_allowed: bool
    ) {
        Self::init_new_network( netuid, tempo, modality );

        Self::set_serving_rate_limit(netuid, serving_limit);
        Self::set_weights_set_rate_limit(netuid, weights_limit);
        Self::set_weights_version_key(netuid, weights_version);
        Self::set_min_allowed_weights(netuid, weights_min);
        Self::set_max_weight_limit(netuid, weights_max);
        
        Self::set_min_burn(netuid, min_burn);
        Self::set_max_burn(netuid, max_burn);

        Self::set_bonds_moving_average(netuid, bonds_moving_avg);

        Self::set_max_allowed_validators(netuid, max_allowed_validators);
        Self::set_max_allowed_uids(netuid, max_allowed_uids);

        Self::set_adjustment_interval(netuid, adjustment_interval);
        Self::set_adjustment_alpha(netuid, adjustment_alpha);

        Self::set_target_registrations_per_interval(netuid, target_reg_per_interval);
        Self::set_max_registrations_per_block(netuid, max_reg_per_block);

        Self::set_activity_cutoff(netuid, activity_cutoff);

        Self::set_rho(netuid, rho);
        Self::set_kappa(netuid, kappa);

        Self::set_validator_prune_len(netuid, validator_prune_len);

        Self::set_scaling_law_power(netuid, scaling_law_power);

        Self::set_immunity_period(netuid, immunity_period);

        Self::set_network_registration_allowed(netuid, reg_allowed);
    }

    // Initializes a new subnetwork under netuid with parameters.
    //
    pub fn init_new_network( netuid:u16, tempo:u16, modality:u16 ){

        // --- 1. Set network to 0 size.
        SubnetworkN::<T>::insert( netuid, 0 );

        // --- 2. Set this network uid to alive.
        NetworksAdded::<T>::insert( netuid, true );
        
        // --- 3. Fill tempo memory item.
        Tempo::<T>::insert( netuid, tempo );

        // --- 4 Fill modality item.
        NetworkModality::<T>::insert( netuid, modality );

        // --- 5. Increase total network count.
        TotalNetworks::<T>::mutate( |n| *n += 1 );

        // --- 6. Set all default values **explicitly**.
        Self::set_default_values_for_all_parameters( netuid );
    }

    // Removes the network (netuid) and all of its parameters.
    //
    pub fn remove_network( netuid:u16 ) {

        // --- 1. Remove network count.
        SubnetworkN::<T>::remove( netuid );

        // --- 2. Remove network modality storage.
        NetworkModality::<T>::remove( netuid );

        // --- 3. Remove netuid from added networks.
        NetworksAdded::<T>::remove( netuid );

        // --- 4. Erase all memory associated with the network.
        Self::erase_all_network_data( netuid );

        // --- 5. Decrement the network counter.
        TotalNetworks::<T>::mutate(|val| *val -= 1);
    }


    // Explicitly sets all network parameters to their default values.
    // Note: this is required because, although there are defaults, they are not explicitly set until this call.
    //
    pub fn set_default_values_for_all_parameters(netuid: u16){
        // Make network parameters explicit.
        if !Tempo::<T>::contains_key( netuid ) { Tempo::<T>::insert( netuid, Tempo::<T>::get( netuid ));}
        if !Kappa::<T>::contains_key( netuid ) { Kappa::<T>::insert( netuid, Kappa::<T>::get( netuid ));}
        if !Difficulty::<T>::contains_key( netuid ) { Difficulty::<T>::insert( netuid, Difficulty::<T>::get( netuid ));}
        if !MaxAllowedUids::<T>::contains_key( netuid ) { MaxAllowedUids::<T>::insert( netuid, MaxAllowedUids::<T>::get( netuid ));}
        if !ImmunityPeriod::<T>::contains_key( netuid ) { ImmunityPeriod::<T>::insert( netuid, ImmunityPeriod::<T>::get( netuid ));}
        if !ActivityCutoff::<T>::contains_key( netuid ) { ActivityCutoff::<T>::insert( netuid, ActivityCutoff::<T>::get( netuid ));}
        if !EmissionValues::<T>::contains_key( netuid ) { EmissionValues::<T>::insert( netuid, EmissionValues::<T>::get( netuid ));}   
        if !MaxWeightsLimit::<T>::contains_key( netuid ) { MaxWeightsLimit::<T>::insert( netuid, MaxWeightsLimit::<T>::get( netuid ));}
        if !MinAllowedWeights::<T>::contains_key( netuid ) { MinAllowedWeights::<T>::insert( netuid, MinAllowedWeights::<T>::get( netuid )); }
        if !RegistrationsThisInterval::<T>::contains_key( netuid ) { RegistrationsThisInterval::<T>::insert( netuid, RegistrationsThisInterval::<T>::get( netuid ));}
        if !POWRegistrationsThisInterval::<T>::contains_key( netuid ) { POWRegistrationsThisInterval::<T>::insert( netuid, POWRegistrationsThisInterval::<T>::get( netuid ));}
        if !BurnRegistrationsThisInterval::<T>::contains_key( netuid ) { BurnRegistrationsThisInterval::<T>::insert( netuid, BurnRegistrationsThisInterval::<T>::get( netuid ));}
    }

    // Explicitly erases all data associated with this network.
    //
    pub fn erase_all_network_data(netuid: u16){

        // --- 1. Remove incentive mechanism memory.
        let _ = Uids::<T>::clear_prefix( netuid, u32::max_value(), None );
        let _ = Keys::<T>::clear_prefix( netuid, u32::max_value(), None );
        let _ = Bonds::<T>::clear_prefix( netuid, u32::max_value(), None );
        let _ = Weights::<T>::clear_prefix( netuid, u32::max_value(), None );

        Rank::<T>::remove( netuid );
        Trust::<T>::remove( netuid );
        Active::<T>::remove( netuid );
        Emission::<T>::remove( netuid );
        Incentive::<T>::remove( netuid );
        Consensus::<T>::remove( netuid );
        Dividends::<T>::remove( netuid );
        PruningScores::<T>::remove( netuid );
        LastUpdate::<T>::remove( netuid );
        ValidatorPermit::<T>::remove( netuid );
        ValidatorTrust::<T>::remove( netuid );

        // --- 2. Erase network parameters.
        Tempo::<T>::remove( netuid );
        Kappa::<T>::remove( netuid );
        Difficulty::<T>::remove( netuid );
        MaxAllowedUids::<T>::remove( netuid );
        ImmunityPeriod::<T>::remove( netuid );
        ActivityCutoff::<T>::remove( netuid );
        EmissionValues::<T>::remove( netuid );
        MaxWeightsLimit::<T>::remove( netuid );
        MinAllowedWeights::<T>::remove( netuid );
        RegistrationsThisInterval::<T>::remove( netuid );
        POWRegistrationsThisInterval::<T>::remove( netuid );
        BurnRegistrationsThisInterval::<T>::remove( netuid );
    }


    // --- Returns true if a network connection exists.
    //
    pub fn network_connection_requirement_exists( netuid_a: u16, netuid_b: u16 ) -> bool {
        NetworkConnect::<T>::contains_key( netuid_a, netuid_b )
    }

    // --- Returns the network connection requirment between net A and net B.
    //
    pub fn get_network_connection_requirement( netuid_a: u16, netuid_b: u16 ) -> u16 {
        if Self::network_connection_requirement_exists( netuid_a, netuid_b ){
            return NetworkConnect::<T>::get( netuid_a, netuid_b ).unwrap();
        } else {
            // Should never occur.
            return u16::MAX;
        }
    }

    // --- Adds a network b connection requirement to network a. 
    //
    pub fn add_connection_requirement( netuid_a: u16, netuid_b: u16, requirement: u16 ) {
        NetworkConnect::<T>::insert( netuid_a, netuid_b, requirement );
    }

    // --- Removes the network b connection requirement from network a. 
    //
    pub fn remove_connection_requirment( netuid_a: u16, netuid_b: u16) {
        if Self::network_connection_requirement_exists(netuid_a, netuid_b) { NetworkConnect::<T>::remove( netuid_a, netuid_b ); }
    }

    // Returns true if the items contain duplicates.
    //
    fn has_duplicate_netuids( netuids: &Vec<u16> ) -> bool {
        let mut parsed: Vec<u16> = Vec::new();
        for item in netuids {
            if parsed.contains(&item) { return true; }
            parsed.push(item.clone());
        }
        return false;
    }

    // Checks for any invalid netuids on this network.
    //
    pub fn contains_invalid_netuids( netuids: &Vec<u16> ) -> bool {
        for netuid in netuids {
            if !Self::if_subnet_exist( *netuid ) {
                return true;
            }
        }
        return false;
    }

    // Set emission values for the passed networks. 
    //
    pub fn set_emission_values( netuids: &Vec<u16>, emission: &Vec<u64> ){
        for (i, netuid_i) in netuids.iter().enumerate() {
            Self::set_emission_for_network( *netuid_i, emission[i] ); 
        }
    }

    // Set the emission on a single network.
    //
    pub fn set_emission_for_network( netuid: u16, emission: u64 ){
        EmissionValues::<T>::insert( netuid, emission );
    }

    // Returns true if the subnetwork exists.
    //
    pub fn if_subnet_exist( netuid: u16 ) -> bool{
        return NetworksAdded::<T>::get( netuid );
    }

	// Returns true if the subnetwork allows registration.
    //
    pub fn if_subnet_allows_registration( netuid: u16 ) -> bool{
        return NetworkRegistrationAllowed::<T>::get( netuid );
    }

    // Returns true if the passed modality is allowed.
    //
    pub fn if_modality_is_valid( modality: u16 ) -> bool{
        let allowed_values: Vec<u16> = vec![0, 1, 2];
        return allowed_values.contains( &modality );
    } 

    // Returns true if the passed tempo is allowed.
    //
    pub fn if_tempo_is_valid(tempo: u16) -> bool {
        tempo < u16::MAX
    }
    
    pub fn get_network_registered_block( netuid: u16 ) -> u64 {
        NetworkRegisteredAt::<T>::get( netuid )
    }
    
    pub fn get_network_immunity_period() -> u64 {
        NetworkImmunityPeriod::<T>::get()
    }

    pub fn get_subnet_to_prune() -> u16 {
        let mut min_score = u64::MAX;
        let mut min_score_in_immunity_period = u64::MAX;
        let mut uid_with_min_score = 0;
        let mut uid_with_min_score_in_immunity_period: u16 =  0;

        for netuid in 0..TotalNetworks::<T>::get() {
            let pruning_score: u64 = Self::get_emission_value( netuid );
            let block_at_registration: u64 = Self::get_network_registered_block( netuid );
            let current_block :u64 = Self::get_current_block_as_u64();
            let immunity_period: u64 = Self::get_network_immunity_period();
            if min_score == pruning_score {
                if current_block - block_at_registration <  immunity_period { //neuron is in immunity period
                    if min_score_in_immunity_period > pruning_score {
                        min_score_in_immunity_period = pruning_score; 
                        uid_with_min_score_in_immunity_period = netuid;
                    }
                }
                else {
                    min_score = pruning_score; 
                    uid_with_min_score = netuid;
                }
            }
            // Find min pruning score.
            else if min_score > pruning_score { 
                if current_block - block_at_registration <  immunity_period { // network is in immunity period
                    if min_score_in_immunity_period > pruning_score {
                         min_score_in_immunity_period = pruning_score; 
                        uid_with_min_score_in_immunity_period = netuid;
                    }
                }
                else {
                    min_score = pruning_score; 
                    uid_with_min_score = netuid;
                }
            }
        }
        if min_score == u64::MAX { // all networks are in immunity period
            return uid_with_min_score_in_immunity_period;
        }
        else {
            return uid_with_min_score;
        }
    }
}