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

    /// Resets the trust, emission, consensus, incentive, dividends, bonds, and weights of 
    /// the neuron to default
    pub fn clear_neuron(netuid: NetUid, neuron_uid: u16) {
        let neuron_index: usize = neuron_uid.into();
        Emission::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0.into()));
        Trust::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0));
        Consensus::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0));
        for subid in 0..SubsubnetCountCurrent::<T>::get(netuid).into() {
            let netuid_index = Self::get_subsubnet_storage_index(netuid, subid.into());
            Incentive::<T>::mutate(netuid_index, |v| Self::set_element_at(v, neuron_index, 0));
            Bonds::<T>::remove(netuid_index, neuron_uid); // Remove bonds for Validator.

            // Clear weights set BY the neuron_uid
            Weights::<T>::remove(netuid_index, neuron_uid);

            // Set weights FOR the neuron_uid to 0
            let all_uids: Vec<u16> = Weights::<T>::iter_key_prefix(netuid_index).collect();
            for uid in all_uids {
                Weights::<T>::mutate(netuid_index, uid, |weight_vec: &mut Vec<(u16, u16)>| {
                    for (weight_uid, w) in weight_vec.iter_mut() {
                        if *weight_uid == neuron_uid {
                            *w = 0;
                        }
                    }
                });
            }
        }
        Dividends::<T>::mutate(netuid, |v| Self::set_element_at(v, neuron_index, 0));
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
        for subid in 0..SubsubnetCountCurrent::<T>::get(netuid).into() {
            let netuid_index = Self::get_subsubnet_storage_index(netuid, subid.into());
            Incentive::<T>::mutate(netuid_index, |v| v.push(0));
            LastUpdate::<T>::mutate(netuid_index, |v| v.push(block_number));
        }
        Dividends::<T>::mutate(netuid, |v| v.push(0));
        PruningScores::<T>::mutate(netuid, |v| v.push(0));
        ValidatorTrust::<T>::mutate(netuid, |v| v.push(0));
        ValidatorPermit::<T>::mutate(netuid, |v| v.push(false));

        // 4. Insert new account information.
        Keys::<T>::insert(netuid, next_uid, new_hotkey.clone()); // Make hotkey - uid association.
        Uids::<T>::insert(netuid, new_hotkey.clone(), next_uid); // Make uid - hotkey association.
        BlockAtRegistration::<T>::insert(netuid, next_uid, block_number); // Fill block at registration.
        IsNetworkMember::<T>::insert(new_hotkey.clone(), netuid, true); // Fill network is member.
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
