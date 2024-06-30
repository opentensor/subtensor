
use super::*;
use crate::epoch::math::*;
use sp_std::vec;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic set_weights.
    ///
    /// # Args:
    ///  * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///    - The signature of the calling hotkey.
    ///
    ///  * 'netuid' (u16):
    ///    - The u16 network identifier.
    ///
    ///  * 'uids' ( Vec<u16> ):
    ///    - The uids of the weights to be set on the chain.
    ///
    ///  * 'values' ( Vec<u16> ):
    ///    - The values of the weights to set on the chain.
    ///
    ///  * 'version_key' ( u64 ):
    ///    - The network version key.
    ///
    /// # Event:
    ///  * WeightsSet;
    ///    - On successfully setting the weights on chain.
    ///
    /// # Raises:
    ///  * 'SubNetworkDoesNotExist':
    ///    - Attempting to set weights on a non-existent network.
    ///
    ///  * 'NotRegistered':
    ///    - Attempting to set weights from a non registered account.
    ///
    ///  * 'IncorrectWeightVersionKey':
    ///    - Attempting to set weights without having an up-to-date version_key.
    ///
    ///  * 'SettingWeightsTooFast':
    ///    - Attempting to set weights faster than the weights_set_rate_limit.
    ///
    ///  * 'NeuronNoValidatorPermit':
    ///    - Attempting to set non-self weights without a validator permit.
    ///
    ///  * 'WeightVecNotEqualSize':
    ///    - Attempting to set weights with uids not of same length.
    ///
    ///  * 'DuplicateUids':
    ///    - Attempting to set weights with duplicate uids.
    ///
    /// * 'UidsLengthExceedUidsInSubNet':
    ///    - Attempting to set weights above the max allowed uids.
    ///
    /// * 'UidVecContainInvalidOne':
    ///    - Attempting to set weights with invalid uids.
    ///
    /// * 'WeightVecLengthIsLow':
    ///    - Attempting to set weights with fewer weights than min.
    ///
    /// * 'MaxWeightExceeded':
    ///    - Attempting to set weights with max value exceeding limit.
    ///
    pub fn do_set_weights(
        origin: T::RuntimeOrigin,
        netuid: u16,
        uids: Vec<u16>,
        values: Vec<u16>,
        version_key: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. Check the caller's signature. This is the hotkey of a registered account.
        let hotkey = ensure_signed(origin)?;
        log::info!(
            "do_set_weights( origin:{:?} netuid:{:?}, uids:{:?}, values:{:?})",
            hotkey,
            netuid,
            uids,
            values
        );

        // --- Check that the netuid is not the root network.
        ensure!(
            netuid != Self::get_root_netuid(),
            Error::<T>::CanNotSetRootNetworkWeights
        );

        // --- 2. Check that the length of uid list and value list are equal for this network.
        ensure!(
            Self::uids_match_values(&uids, &values),
            Error::<T>::WeightVecNotEqualSize
        );

        // --- 3. Check to see if this is a valid network.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 4. Check to see if the number of uids is within the max allowed uids for this network.
        ensure!(
            Self::check_len_uids_within_allowed(netuid, &uids),
            Error::<T>::UidsLengthExceedUidsInSubNet
        );

        // --- 5. Check to see if the hotkey is registered to the passed network.
        ensure!(
            Self::is_hotkey_registered_on_network(netuid, &hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // --- 6. Check to see if the hotkey has enought stake to set weights.
        ensure!(
            Self::get_total_stake_for_hotkey(&hotkey) >= Self::get_weights_min_stake(),
            Error::<T>::NotEnoughStakeToSetWeights
        );

        // --- 7. Ensure version_key is up-to-date.
        ensure!(
            Self::check_version_key(netuid, version_key),
            Error::<T>::IncorrectWeightVersionKey
        );

        // --- 9. Ensure the uid is not setting weights faster than the weights_set_rate_limit.
        let neuron_uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey)?;
        let current_block: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::check_rate_limit(netuid, neuron_uid, current_block),
            Error::<T>::SettingWeightsTooFast
        );

        // --- 10. Check that the neuron uid is an allowed validator permitted to set non-self weights.
        ensure!(
            Self::check_validator_permit(netuid, neuron_uid, &uids, &values),
            Error::<T>::NeuronNoValidatorPermit
        );

        // --- 11. Ensure the passed uids contain no duplicates.
        ensure!(!Self::has_duplicate_uids(&uids), Error::<T>::DuplicateUids);

        // --- 12. Ensure that the passed uids are valid for the network.
        ensure!(
            !Self::contains_invalid_uids(netuid, &uids),
            Error::<T>::UidVecContainInvalidOne
        );

        // --- 13. Ensure that the weights have the required length.
        ensure!(
            Self::check_length(netuid, neuron_uid, &uids, &values),
            Error::<T>::WeightVecLengthIsLow
        );

        // --- 14. Max-upscale the weights.
        let max_upscaled_weights: Vec<u16> = vec_u16_max_upscale_to_u16(&values);

        // --- 15. Ensure the weights are max weight limited
        ensure!(
            Self::max_weight_limited(netuid, neuron_uid, &uids, &max_upscaled_weights),
            Error::<T>::MaxWeightExceeded
        );

        // --- 16. Zip weights for sinking to storage map.
        let mut zipped_weights: Vec<(u16, u16)> = vec![];
        for (uid, val) in uids.iter().zip(max_upscaled_weights.iter()) {
            zipped_weights.push((*uid, *val))
        }

        // --- 17. Set weights under netuid, uid double map entry.
        Weights::<T>::insert(netuid, neuron_uid, zipped_weights);

        // --- 18. Set the activity for the weights on this network.
        Self::set_last_update_for_uid(netuid, neuron_uid, current_block);

        // --- 19. Emit the tracking event.
        log::info!(
            "WeightsSet( netuid:{:?}, neuron_uid:{:?} )",
            netuid,
            neuron_uid
        );
        Self::deposit_event(Event::WeightsSet(netuid, neuron_uid));

        // --- 20. Return ok.
        Ok(())
    }
}
