use super::*;
use frame_support::storage::IterableStorageDoubleMap;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use sp_std::{cmp, vec};
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
            "append_neuron( netuid: {netuid:?} | next_uid: {next_uid:?} | new_hotkey: {new_hotkey:?} ) "
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

    /// Clears (sets to default) the neuron map values fot a neuron when it is
    /// removed from the subnet
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
        ensure!(
            max_n >= MinAllowedUids::<T>::get(netuid),
            Error::<T>::InvalidValue
        );
        ensure!(
            max_n <= MaxAllowedUids::<T>::get(netuid),
            Error::<T>::InvalidValue
        );

        MaxAllowedUids::<T>::insert(netuid, max_n);

        let owner = SubnetOwner::<T>::get(netuid);
        let owner_uids = BTreeSet::from_iter(Self::get_immune_owner_uids(netuid, &owner));
        let current_n = Self::get_subnetwork_n(netuid);

        if current_n > max_n {
            // Get all emissions with their UIDs and sort by emission (descending)
            // This ensures we keep the highest emitters and remove the lowest ones
            let mut emissions = Emission::<T>::get(netuid)
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>();
            emissions.sort_by_key(|(_, emission)| cmp::Reverse(*emission));

            // Remove uids from the end (lowest emitters) until we reach the new maximum
            let mut removed_uids = BTreeSet::new();
            let mut uids_left_to_process = current_n;

            // Iterate from the end (lowest emitters) to the beginning
            for i in (0..current_n).rev() {
                if uids_left_to_process == max_n {
                    break; // We've reached the target number of UIDs
                }

                if let Some((uid, _)) = emissions.get(i as usize).cloned() {
                    // Skip subnet owner's or temporally immune uids
                    if owner_uids.contains(&(uid as u16))
                        || Self::get_neuron_is_immune(netuid, uid as u16)
                    {
                        continue;
                    }

                    // Remove hotkey related storage items if hotkey exists
                    if let Ok(hotkey) = Keys::<T>::try_get(netuid, uid as u16) {
                        Uids::<T>::remove(netuid, &hotkey);
                        IsNetworkMember::<T>::remove(&hotkey, netuid);
                        LastHotkeyEmissionOnNetuid::<T>::remove(&hotkey, netuid);
                        AlphaDividendsPerSubnet::<T>::remove(netuid, &hotkey);
                        TaoDividendsPerSubnet::<T>::remove(netuid, &hotkey);
                        Axons::<T>::remove(netuid, &hotkey);
                        NeuronCertificates::<T>::remove(netuid, &hotkey);
                        Prometheus::<T>::remove(netuid, &hotkey);
                    }

                    // Remove all storage items associated with this uid
                    Keys::<T>::remove(netuid, uid as u16);
                    BlockAtRegistration::<T>::remove(netuid, uid as u16);
                    Weights::<T>::remove(netuid, uid as u16);
                    Bonds::<T>::remove(netuid, uid as u16);

                    // Remove from emissions array and track as removed
                    emissions.remove(i.into());
                    removed_uids.insert(uid);
                    uids_left_to_process = uids_left_to_process.saturating_sub(1);
                }
            }

            // Sort remaining emissions by uid to compress uids to the left
            // This ensures consecutive uid indices in the final arrays
            emissions.sort_by_key(|(uid, _)| *uid);

            // Extract the final uids and emissions after trimming and sorting
            let (trimmed_uids, trimmed_emissions): (Vec<usize>, Vec<AlphaCurrency>) =
                emissions.into_iter().unzip();

            // Get all current arrays from storage
            let ranks = Rank::<T>::get(netuid);
            let trust = Trust::<T>::get(netuid);
            let active = Active::<T>::get(netuid);
            let consensus = Consensus::<T>::get(netuid);
            let incentive = Incentive::<T>::get(netuid);
            let dividends = Dividends::<T>::get(netuid);
            let lastupdate = LastUpdate::<T>::get(netuid);
            let pruning_scores = PruningScores::<T>::get(netuid);
            let vtrust = ValidatorTrust::<T>::get(netuid);
            let vpermit = ValidatorPermit::<T>::get(netuid);
            let stake_weight = StakeWeight::<T>::get(netuid);

            // Create trimmed arrays by extracting values for kept uids only
            // Pre-allocate vectors with exact capacity for efficiency
            let mut trimmed_ranks = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_trust = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_active = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_consensus = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_incentive = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_dividends = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_lastupdate = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_pruning_scores = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_vtrust = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_vpermit = Vec::with_capacity(trimmed_uids.len());
            let mut trimmed_stake_weight = Vec::with_capacity(trimmed_uids.len());

            // Single iteration to extract values for all kept uids
            for &old_uid in &trimmed_uids {
                trimmed_ranks.push(ranks.get(old_uid).cloned().unwrap_or_default());
                trimmed_trust.push(trust.get(old_uid).cloned().unwrap_or_default());
                trimmed_active.push(active.get(old_uid).cloned().unwrap_or_default());
                trimmed_consensus.push(consensus.get(old_uid).cloned().unwrap_or_default());
                trimmed_incentive.push(incentive.get(old_uid).cloned().unwrap_or_default());
                trimmed_dividends.push(dividends.get(old_uid).cloned().unwrap_or_default());
                trimmed_lastupdate.push(lastupdate.get(old_uid).cloned().unwrap_or_default());
                trimmed_pruning_scores
                    .push(pruning_scores.get(old_uid).cloned().unwrap_or_default());
                trimmed_vtrust.push(vtrust.get(old_uid).cloned().unwrap_or_default());
                trimmed_vpermit.push(vpermit.get(old_uid).cloned().unwrap_or_default());
                trimmed_stake_weight.push(stake_weight.get(old_uid).cloned().unwrap_or_default());
            }

            // Update storage with trimmed arrays
            Emission::<T>::insert(netuid, trimmed_emissions);
            Rank::<T>::insert(netuid, trimmed_ranks);
            Trust::<T>::insert(netuid, trimmed_trust);
            Active::<T>::insert(netuid, trimmed_active);
            Consensus::<T>::insert(netuid, trimmed_consensus);
            Incentive::<T>::insert(netuid, trimmed_incentive);
            Dividends::<T>::insert(netuid, trimmed_dividends);
            LastUpdate::<T>::insert(netuid, trimmed_lastupdate);
            PruningScores::<T>::insert(netuid, trimmed_pruning_scores);
            ValidatorTrust::<T>::insert(netuid, trimmed_vtrust);
            ValidatorPermit::<T>::insert(netuid, trimmed_vpermit);
            StakeWeight::<T>::insert(netuid, trimmed_stake_weight);

            // Create mapping from old uid to new compressed uid
            // This is needed to update connections (weights and bonds) with correct uid references
            let old_to_new_uid: BTreeMap<usize, usize> = trimmed_uids
                .iter()
                .enumerate()
                .map(|(new_uid, &old_uid)| (old_uid, new_uid))
                .collect();

            // Update connections (weights and bonds) for each kept uid
            // This involves three operations per uid:
            // 1. Swap the uid storage to the new compressed position
            // 2. Update all connections to reference the new compressed uids
            // 3. Clear the connections to the trimmed uids
            for (old_uid, new_uid) in &old_to_new_uid {
                // Swap uid specific storage items to new compressed positions
                Keys::<T>::swap(netuid, *old_uid as u16, netuid, *new_uid as u16);
                BlockAtRegistration::<T>::swap(netuid, *old_uid as u16, netuid, *new_uid as u16);

                // Swap to new position and remap all target uids
                Weights::<T>::swap(netuid, *old_uid as u16, netuid, *new_uid as u16);
                Weights::<T>::mutate(netuid, *new_uid as u16, |weights| {
                    weights.retain_mut(|(target_uid, _weight)| {
                        if let Some(new_target_uid) = old_to_new_uid.get(&(*target_uid as usize)) {
                            *target_uid = *new_target_uid as u16;
                            true
                        } else {
                            false
                        }
                    })
                });

                // Swap to new position and remap all target uids
                Bonds::<T>::swap(netuid, *old_uid as u16, netuid, *new_uid as u16);
                Bonds::<T>::mutate(netuid, *new_uid as u16, |bonds| {
                    bonds.retain_mut(|(target_uid, _bond)| {
                        if let Some(new_target_uid) = old_to_new_uid.get(&(*target_uid as usize)) {
                            *target_uid = *new_target_uid as u16;
                            true
                        } else {
                            false
                        }
                    })
                });
            }

            // Update the subnet's uid count to reflect the new maximum
            SubnetworkN::<T>::insert(netuid, max_n);
        }

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
