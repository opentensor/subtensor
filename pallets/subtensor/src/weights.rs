use super::*;
use frame_support::sp_std::vec;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {


    /// ---- The implementation for the extrinsic set_weights.
    ///
    /// # Args:
    /// 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    /// 		- The signature of the calling hotkey.
    ///
    /// 	* 'netuid' (u16):
    /// 		- The u16 network identifier.
    ///
    /// 	* 'uids' ( Vec<u16> ):
    /// 		- The uids of the weights to be set on the chain.
    ///
    /// 	* 'values' ( Vec<u16> ):
    /// 		- The values of the weights to set on the chain.
    ///
    /// 	* 'version_key' ( u64 ):
    /// 		- The network version key.
    ///
    /// # Event:
    /// 	* WeightsSet;
    /// 		- On successfully setting the weights on chain.
    ///
    /// # Raises:
    /// 	* 'NetworkDoesNotExist':
    /// 		- Attempting to set weights on a non-existent network.
    ///
    /// 	* 'NotRegistered':
    /// 		- Attempting to set weights from a non registered account.
    ///
    /// 	* 'IncorrectNetworkVersionKey':
    /// 		- Attempting to set weights without having an up-to-date version_key.
    ///
    /// 	* 'SettingWeightsTooFast':
    /// 		- Attempting to set weights faster than the weights_set_rate_limit.
    ///
    /// 	* 'NoValidatorPermit':
    /// 		- Attempting to set non-self weights without a validator permit.
    ///
    /// 	* 'WeightVecNotEqualSize':
    /// 		- Attempting to set weights with uids not of same length.
    ///
    /// 	* 'DuplicateUids':
    /// 		- Attempting to set weights with duplicate uids.
    /// 
    ///     * 'TooManyUids':
    /// 		- Attempting to set weights above the max allowed uids.
    ///
    /// 	* 'InvalidUid':
    /// 		- Attempting to set weights with invalid uids.
    ///
    /// 	* 'NotSettingEnoughWeights':
    /// 		- Attempting to set weights with fewer weights than min.
    ///
    /// 	* 'MaxWeightExceeded':
    /// 		- Attempting to set weights with max value exceeding limit.
    ///
    pub fn do_set_weights( origin: T::RuntimeOrigin, netuid: u16, uids: Vec<u16>, values: Vec<u16>, version_key:u64 ) -> dispatch::DispatchResult{

        // --- 1. Check the caller's signature. This is the hotkey of a registered account.
        let hotkey = ensure_signed( origin )?;
        log::info!("do_set_weights( origin:{:?} netuid:{:?}, uids:{:?}, values:{:?})", hotkey, netuid, uids, values );

        // --- 2. Check that the length of uid list and value list are equal for this network.
        ensure!( Self::uids_match_values( &uids, &values ), Error::<T>::WeightVecNotEqualSize );

        // --- 3. Check to see if this is a valid network.
        ensure!( Self::if_subnet_exist( netuid ), Error::<T>::NetworkDoesNotExist );
        
        // --- 4. Check to see if the number of uids is within the max allowed uids for this network.
        ensure!( Self::check_len_uids_within_allowed( netuid, &uids ), Error::<T>::TooManyUids);

        // --- 5. Check to see if the hotkey is registered to the passed network.
        ensure!( Self::is_hotkey_registered_on_network( netuid, &hotkey ), Error::<T>::NotRegistered );

        // --- 6. Ensure version_key is up-to-date.
        ensure!( Self::check_version_key( netuid, version_key ), Error::<T>::IncorrectNetworkVersionKey );

        // --- 7. Get the neuron uid of associated hotkey on network netuid.
        let neuron_uid;
        match Self::get_uid_for_net_and_hotkey( netuid, &hotkey ) { Ok(k) => neuron_uid = k, Err(e) => panic!("Error: {:?}", e) } 

        // --- 8. Ensure the uid is not setting weights faster than the weights_set_rate_limit.
        let current_block: u64 = Self::get_current_block_as_u64();
        ensure!( Self::check_rate_limit( netuid, neuron_uid, current_block ), Error::<T>::SettingWeightsTooFast );

        // --- 9. Check that the neuron uid is an allowed validator permitted to set non-self weights.
        ensure!( Self::check_validator_permit( netuid, neuron_uid, &uids, &values ), Error::<T>::NoValidatorPermit );

        // --- 10. Ensure the passed uids contain no duplicates.
        ensure!( !Self::has_duplicate_uids( &uids ), Error::<T>::DuplicateUids );

        // --- 11. Ensure that the passed uids are valid for the network.
        ensure!( !Self::contains_invalid_uids( netuid, &uids ), Error::<T>::InvalidUid );

        // --- 12. Ensure that the weights have the required length.
        ensure!( Self::check_length( netuid, neuron_uid, &uids, &values ), Error::<T>::NotSettingEnoughWeights );

        // --- 13. Normalize the weights.
        let normalized_values = Self::normalize_weights( values );

        // --- 14. Ensure the weights are max weight limited 
        ensure!( Self::max_weight_limited( netuid, neuron_uid, &uids, &normalized_values ), Error::<T>::MaxWeightExceeded );

        // --- 15. Zip weights for sinking to storage map.
        let mut zipped_weights: Vec<( u16, u16 )> = vec![];
        for ( uid, val ) in uids.iter().zip(normalized_values.iter()) { zipped_weights.push((*uid, *val)) }

        // --- 16. Set weights under netuid, uid double map entry.
        Weights::<T>::insert( netuid, neuron_uid, zipped_weights );

        // --- 17. Set the activity for the weights on this network.
        Self::set_last_update_for_uid( netuid, neuron_uid, current_block );

        // --- 18. Emit the tracking event.
        log::info!("WeightsSet( netuid:{:?}, neuron_uid:{:?} )", netuid, neuron_uid );
        Self::deposit_event( Event::WeightsSet( netuid, neuron_uid ) );

        // --- 19. Return ok.
        Ok(())
    }

    /// ==========================
	/// ==== Helper functions ====
	/// ==========================
    
    /// Returns true if version_key is up-to-date.
    ///
    pub fn check_version_key( netuid: u16, version_key: u64) -> bool {
        let network_version_key: u64 = WeightsVersionKey::<T>::get( netuid );
        log::info!("check_version_key( network_version_key:{:?}, version_key:{:?} )", network_version_key, version_key );
        return network_version_key == 0 || version_key == network_version_key;
    }

    /// Checks if the neuron has set weights within the weights_set_rate_limit.
    ///
    pub fn check_rate_limit( netuid: u16, neuron_uid: u16, current_block: u64 ) -> bool {
        if Self::is_uid_exist_on_network( netuid, neuron_uid ){ 
            // --- 1. Ensure that the diff between current and last_set weights is greater than limit.
            let last_set_weights: u64 = Self::get_last_update_for_uid( netuid, neuron_uid );
            if last_set_weights == 0 { return true; } // (Storage default) Never set weights.
            return current_block - last_set_weights >= Self::get_weights_set_rate_limit( netuid );
        }
        // --- 3. Non registered peers cant pass.
        return false;
    }

    /// Checks for any invalid uids on this network.
    pub fn contains_invalid_uids( netuid: u16, uids: &Vec<u16> ) -> bool {
        for uid in uids {
            if !Self::is_uid_exist_on_network( netuid, *uid ) {
                return true;
            }
        }
        return false;
    }

    /// Returns true if the passed uids have the same length of the passed values.
    fn uids_match_values(uids: &Vec<u16>, values: &Vec<u16>) -> bool {
        return uids.len() == values.len();
    }

    /// Returns true if the items contain duplicates.
    fn has_duplicate_uids(items: &Vec<u16>) -> bool {
        let mut parsed: Vec<u16> = Vec::new();
        for item in items {
            if parsed.contains(&item) { return true; }
            parsed.push(item.clone());
        }
        return false;
    }

    /// Returns True if setting self-weight or has validator permit.
    pub fn check_validator_permit( netuid: u16, uid: u16, uids: &Vec<u16>, weights: &Vec<u16> ) -> bool {
        // Check self weight. Allowed to set single value for self weight.
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }
        // Check if uid has validator permit.
        Self::get_validator_permit_for_uid( netuid, uid )
    }

    /// Returns True if the uids and weights are have a valid length for uid on network.
    pub fn check_length( netuid: u16, uid: u16, uids: &Vec<u16>, weights: &Vec<u16> ) -> bool {
        let min_allowed_length: usize = Self::get_min_allowed_weights(netuid) as usize;

        // Check self weight. Allowed to set single value for self weight.
        if Self::is_self_weight(uid, uids, weights) {
            return true;
        }
        // Check if number of weights exceeds min.
        if weights.len() >= min_allowed_length {
            return true;
        }
        // To few weights.
        return false;
    }

    /// Implace normalizes the passed positive integer weights so that they sum to u16 max value.
    pub fn normalize_weights(mut weights: Vec<u16>) -> Vec<u16> {
        let sum: u64 = weights.iter().map(|x| *x as u64).sum();
        if sum == 0 { return weights; }
        weights.iter_mut().for_each(|x| { *x = (*x as u64 * u16::max_value() as u64 / sum) as u16; });
        return weights;
    }

    /// Returns False if the weights exceed the max_weight_limit for this network.
    pub fn max_weight_limited( netuid: u16, uid: u16, uids: &Vec<u16>, weights: &Vec<u16> ) -> bool {

        // Allow self weights to exceed max weight limit.
        if Self::is_self_weight( uid, uids, weights ) { return true; }

        // If the max weight limit it u16 max, return true.
        let max_weight_limit: u16 = Self::get_max_weight_limit( netuid );
        if max_weight_limit == u16::MAX { return true; }
    
        // Check if the weights max value is less than or equal to the limit.
        let max: u16 = *weights.iter().max().unwrap();
        if max <= max_weight_limit { return true; }
        
        // The check has failed.
        return false;
    }

    /// Returns true if the uids and weights correspond to a self weight on the uid.
    pub fn is_self_weight( uid: u16, uids: &Vec<u16>, weights: &Vec<u16> ) -> bool {
        if weights.len() != 1 { return false; }
        if uid != uids[0] { return false; } 
        return true;
    }

    /// Returns False is the number of uids exceeds the allowed number of uids for this network.
    pub fn check_len_uids_within_allowed( netuid: u16, uids: &Vec<u16> ) -> bool {
        let subnetwork_n: u16 = Self::get_subnetwork_n( netuid );
        // we should expect at most subnetwork_n uids.
        return uids.len() <= subnetwork_n as usize;
    }
    
}