use super::*;
use frame_support::storage::IterableStorageDoubleMap;
use sp_std::vec;
use subtensor_runtime_common::NetUid;

impl<T: Config> Pallet<T> {
    /// Returns the number of filled slots on a network.
    pub fn get_subnetwork_n(netuid: NetUid) -> u16 {
        SubnetworkN::<T>::get(netuid)
    }

    /// Sets value for the element at the given position if it exists.
    pub fn set_element_at<N>(vec: &mut [N], position: usize, value: N) {
        if let Some(element) = vec.get_mut(position) {
            *element = value;
        }
    }

    /// Replace the neuron under this uid.
    pub fn replace_neuron(
        netuid: NetUid,
        uid_to_replace: u16,
        new_hotkey: &T::AccountId,
        block_number: u64,
    ) {
        log::debug!(
            "replace_neuron( netuid: {netuid:?} | uid_to_replace: {uid_to_replace:?} | new_hotkey: {new_hotkey:?} ) "
        );

        // 1. Get the old hotkey under this position.
        let old_hotkey: T::AccountId = Keys::<T>::get(netuid, uid_to_replace);

        // Do not replace owner hotkey from `SubnetOwnerHotkey`
        if let Ok(sn_owner_hotkey) = SubnetOwnerHotkey::<T>::try_get(netuid) {
            if sn_owner_hotkey == old_hotkey.clone() {
                log::warn!(
                    "replace_neuron: Skipped replacement because neuron is the subnet owner hotkey. \
                    netuid: {netuid:?}, uid_to_replace: {uid_to_replace:?}, new_hotkey: {new_hotkey:?}, owner_hotkey: {sn_owner_hotkey:?}"
                );
                return;
            }
        }

        // 2. Remove previous set memberships.
        Uids::<T>::remove(netuid, old_hotkey.clone());
        AssociatedEvmAddress::<T>::remove(netuid, uid_to_replace);
        IsNetworkMember::<T>::remove(old_hotkey.clone(), netuid);
        #[allow(unknown_lints)]
        Keys::<T>::remove(netuid, uid_to_replace);

        // 3. Create new set memberships.
        Self::set_active_for_uid(netuid, uid_to_replace, true); // Set to active by default.
        Keys::<T>::insert(netuid, uid_to_replace, new_hotkey.clone()); // Make hotkey - uid association.
        Uids::<T>::insert(netuid, new_hotkey.clone(), uid_to_replace); // Make uid - hotkey association.
        BlockAtRegistration::<T>::insert(netuid, uid_to_replace, block_number); // Fill block at registration.
        IsNetworkMember::<T>::insert(new_hotkey.clone(), netuid, true); // Fill network is member.

        // 4. Clear neuron certificates
        NeuronCertificates::<T>::remove(netuid, old_hotkey.clone());

        // 5. Reset new neuron's values.
        Self::clear_neuron(netuid, uid_to_replace);

        // 5a. reset axon info for the new uid.
        Axons::<T>::remove(netuid, old_hotkey);
    }

    /// Appends the uid to the network.
    pub fn append_neuron(netuid: NetUid, new_hotkey: &T::AccountId, block_number: u64) {
        // 1. Get the next uid. This is always equal to subnetwork_n.
        let next_uid: u16 = Self::get_subnetwork_n(netuid);
        log::debug!(
            "append_neuron( netuid: {netuid:?} | next_uid: {new_hotkey:?} | new_hotkey: {next_uid:?} ) "
        );

        // 2. Get and increase the uid count.
        SubnetworkN::<T>::insert(netuid, next_uid.saturating_add(1));

        // 3. Expand Yuma Consensus with new position.
        Rank::<T>::mutate(netuid, |v| v.push(0));
        Trust::<T>::mutate(netuid, |v| v.push(0));
        Active::<T>::mutate(netuid, |v| v.push(true));
        Emission::<T>::mutate(netuid, |v| v.push(0.into()));
        Consensus::<T>::mutate(netuid, |v| v.push(0));
        Incentive::<T>::mutate(netuid, |v| v.push(0));
        Dividends::<T>::mutate(netuid, |v| v.push(0));
        LastUpdate::<T>::mutate(netuid, |v| v.push(block_number));
        PruningScores::<T>::mutate(netuid, |v| v.push(0));
        ValidatorTrust::<T>::mutate(netuid, |v| v.push(0));
        ValidatorPermit::<T>::mutate(netuid, |v| v.push(false));

        // 4. Insert new account information.
        Keys::<T>::insert(netuid, next_uid, new_hotkey.clone()); // Make hotkey - uid association.
        Uids::<T>::insert(netuid, new_hotkey.clone(), next_uid); // Make uid - hotkey association.
        BlockAtRegistration::<T>::insert(netuid, next_uid, block_number); // Fill block at registration.
        IsNetworkMember::<T>::insert(new_hotkey.clone(), netuid, true); // Fill network is member.
    }

    /// Appends the uid to the network.
    pub fn clear_neuron(netuid: NetUid, neuron_uid: u16) {
        let neuron_index: usize = neuron_uid.into();
        Emission::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0.into()));
        Trust::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0));
        Consensus::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0));
        Incentive::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0));
        Dividends::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0));
        Bonds::<T>::remove(netuid, neuron_uid); // Remove bonds for Validator.
    }

    pub fn trim_to_max_allowed_uids(netuid: NetUid, max_n: u16) -> DispatchResult {
        // Reasonable limits
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );
        ensure!(max_n > 16, Error::<T>::InvalidValue);
        ensure!(
            max_n <= Self::get_max_allowed_uids(netuid),
            Error::<T>::InvalidValue
        );

        // Set the value.
        MaxAllowedUids::<T>::insert(netuid, max_n);

        // Check if we need to trim.
        let current_n: u16 = Self::get_subnetwork_n(netuid);

        // We need to trim, get rid of values between max_n and current_n.
        if current_n > max_n {
            let ranks: Vec<u16> = Rank::<T>::get(netuid);
            let trimmed_ranks: Vec<u16> = ranks.into_iter().take(max_n as usize).collect();
            Rank::<T>::insert(netuid, trimmed_ranks);

            let trust: Vec<u16> = Trust::<T>::get(netuid);
            let trimmed_trust: Vec<u16> = trust.into_iter().take(max_n as usize).collect();
            Trust::<T>::insert(netuid, trimmed_trust);

            let active: Vec<bool> = Active::<T>::get(netuid);
            let trimmed_active: Vec<bool> = active.into_iter().take(max_n as usize).collect();
            Active::<T>::insert(netuid, trimmed_active);

            let emission: Vec<AlphaCurrency> = Emission::<T>::get(netuid);
            let trimmed_emission: Vec<AlphaCurrency> =
                emission.into_iter().take(max_n as usize).collect();
            Emission::<T>::insert(netuid, trimmed_emission);

            let consensus: Vec<u16> = Consensus::<T>::get(netuid);
            let trimmed_consensus: Vec<u16> = consensus.into_iter().take(max_n as usize).collect();
            Consensus::<T>::insert(netuid, trimmed_consensus);

            let incentive: Vec<u16> = Incentive::<T>::get(netuid);
            let trimmed_incentive: Vec<u16> = incentive.into_iter().take(max_n as usize).collect();
            Incentive::<T>::insert(netuid, trimmed_incentive);

            let dividends: Vec<u16> = Dividends::<T>::get(netuid);
            let trimmed_dividends: Vec<u16> = dividends.into_iter().take(max_n as usize).collect();
            Dividends::<T>::insert(netuid, trimmed_dividends);

            let lastupdate: Vec<u64> = LastUpdate::<T>::get(netuid);
            let trimmed_lastupdate: Vec<u64> =
                lastupdate.into_iter().take(max_n as usize).collect();
            LastUpdate::<T>::insert(netuid, trimmed_lastupdate);

            let pruning_scores: Vec<u16> = PruningScores::<T>::get(netuid);
            let trimmed_pruning_scores: Vec<u16> =
                pruning_scores.into_iter().take(max_n as usize).collect();
            PruningScores::<T>::insert(netuid, trimmed_pruning_scores);

            let vtrust: Vec<u16> = ValidatorTrust::<T>::get(netuid);
            let trimmed_vtrust: Vec<u16> = vtrust.into_iter().take(max_n as usize).collect();
            ValidatorTrust::<T>::insert(netuid, trimmed_vtrust);

            let vpermit: Vec<bool> = ValidatorPermit::<T>::get(netuid);
            let trimmed_vpermit: Vec<bool> = vpermit.into_iter().take(max_n as usize).collect();
            ValidatorPermit::<T>::insert(netuid, trimmed_vpermit);

            let stake_weight: Vec<u16> = StakeWeight::<T>::get(netuid);
            let trimmed_stake_weight: Vec<u16> =
                stake_weight.into_iter().take(max_n as usize).collect();
            StakeWeight::<T>::insert(netuid, trimmed_stake_weight);

            // Trim UIDs and Keys by removing entries with UID >= max_n (since UIDs are 0-indexed)
            // UIDs range from 0 to current_n-1, so we remove UIDs from max_n to current_n-1
            for uid in max_n..current_n {
                if let Ok(hotkey) = Keys::<T>::try_get(netuid, uid) {
                    Uids::<T>::remove(netuid, &hotkey);
                    // Remove IsNetworkMember association for the hotkey
                    IsNetworkMember::<T>::remove(&hotkey, netuid);
                    // Remove last hotkey emission for the hotkey
                    LastHotkeyEmissionOnNetuid::<T>::remove(&hotkey, netuid);
                    // Remove alpha dividends for the hotkey
                    AlphaDividendsPerSubnet::<T>::remove(netuid, &hotkey);
                    // Remove tao dividends for the hotkey
                    TaoDividendsPerSubnet::<T>::remove(netuid, &hotkey);
                }
                #[allow(unknown_lints)]
                Keys::<T>::remove(netuid, uid);
                // Remove block at registration for the uid
                BlockAtRegistration::<T>::remove(netuid, uid);
            }

            // Trim weights and bonds for removed UIDs
            for uid in max_n..current_n {
                Weights::<T>::remove(netuid, uid);
                Bonds::<T>::remove(netuid, uid);
            }

            // Trim axons, certificates, and prometheus info for removed hotkeys
            for uid in max_n..current_n {
                if let Ok(hotkey) = Keys::<T>::try_get(netuid, uid) {
                    Axons::<T>::remove(netuid, &hotkey);
                    NeuronCertificates::<T>::remove(netuid, &hotkey);
                    Prometheus::<T>::remove(netuid, &hotkey);
                }
            }

            // Trim weight and bond connections to removed UIDs for remaining neurons
            // UIDs 0 to max_n-1 are kept, so we iterate through these valid UIDs
            for uid in 0..max_n {
                Weights::<T>::mutate(netuid, uid, |weights| {
                    weights.retain(|(target_uid, _)| *target_uid < max_n);
                });
                Bonds::<T>::mutate(netuid, uid, |bonds| {
                    bonds.retain(|(target_uid, _)| *target_uid < max_n);
                });
            }

            // Update the subnetwork size
            SubnetworkN::<T>::insert(netuid, max_n);
        }

        // --- Ok and done.
        Ok(())
    }

    /// Returns true if the uid is set on the network.
    ///
    pub fn is_uid_exist_on_network(netuid: NetUid, uid: u16) -> bool {
        Keys::<T>::contains_key(netuid, uid)
    }

    /// Returns true if the hotkey holds a slot on the network.
    ///
    pub fn is_hotkey_registered_on_network(netuid: NetUid, hotkey: &T::AccountId) -> bool {
        Uids::<T>::contains_key(netuid, hotkey)
    }

    /// Returs the hotkey under the network uid as a Result. Ok if the uid is taken.
    ///
    pub fn get_hotkey_for_net_and_uid(
        netuid: NetUid,
        neuron_uid: u16,
    ) -> Result<T::AccountId, DispatchError> {
        Keys::<T>::try_get(netuid, neuron_uid)
            .map_err(|_err| Error::<T>::HotKeyNotRegisteredInSubNet.into())
    }

    /// Returns the uid of the hotkey in the network as a Result. Ok if the hotkey has a slot.
    ///
    pub fn get_uid_for_net_and_hotkey(
        netuid: NetUid,
        hotkey: &T::AccountId,
    ) -> Result<u16, DispatchError> {
        Uids::<T>::try_get(netuid, hotkey)
            .map_err(|_err| Error::<T>::HotKeyNotRegisteredInSubNet.into())
    }

    /// Returns the stake of the uid on network or 0 if it doesnt exist.
    ///
    pub fn get_stake_for_uid_and_subnetwork(netuid: NetUid, neuron_uid: u16) -> AlphaCurrency {
        if let Ok(hotkey) = Self::get_hotkey_for_net_and_uid(netuid, neuron_uid) {
            Self::get_stake_for_hotkey_on_subnet(&hotkey, netuid)
        } else {
            AlphaCurrency::ZERO
        }
    }

    /// Return a list of all networks a hotkey is registered on.
    ///
    pub fn get_registered_networks_for_hotkey(hotkey: &T::AccountId) -> Vec<NetUid> {
        let mut all_networks: Vec<NetUid> = vec![];
        for (network, is_registered) in <IsNetworkMember<T> as IterableStorageDoubleMap<
            T::AccountId,
            NetUid,
            bool,
        >>::iter_prefix(hotkey)
        {
            if is_registered {
                all_networks.push(network)
            }
        }
        all_networks
    }

    /// Return true if a hotkey is registered on any network.
    ///
    pub fn is_hotkey_registered_on_any_network(hotkey: &T::AccountId) -> bool {
        for (_, is_registered) in <IsNetworkMember<T> as IterableStorageDoubleMap<
            T::AccountId,
            NetUid,
            bool,
        >>::iter_prefix(hotkey)
        {
            if is_registered {
                return true;
            }
        }
        false
    }

    /// Return true if a hotkey is registered on specific network.
    ///
    pub fn is_hotkey_registered_on_specific_network(hotkey: &T::AccountId, netuid: NetUid) -> bool {
        IsNetworkMember::<T>::contains_key(hotkey, netuid)
    }
}
