
use super::*;
use frame_support::storage::{IterableStorageDoubleMap, IterableStorageMap};
use sp_runtime::Saturating;
use sp_std::vec;
use substrate_fixed::{
    types::{I96F32},
};

impl<T: Config> Pallet<T> {

    /// Facilitates user registration of a new subnetwork.
    ///
    /// # Args:
    /// * 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    ///
    /// # Event:
    /// * 'NetworkAdded': Emitted when a new network is successfully added.
    ///
    /// # Raises:
    /// * 'TxRateLimitExceeded': If the rate limit for network registration is exceeded.
    /// * 'NotEnoughBalanceToStake': If there isn't enough balance to stake for network registration.
    /// * 'BalanceWithdrawalError': If an error occurs during balance withdrawal for network registration.
    ///
    pub fn user_add_network(origin: T::RuntimeOrigin, hotkey: &T::AccountId, mechid: u16) -> dispatch::DispatchResult {
        // --- 0. Ensure the caller is a signed user.
        let coldkey = ensure_signed(origin)?;

        // --- 0.1. Ensure the mechanism exists.
        ensure!( Self::mechanism_exists( mechid ), Error::<T>::MechanismDoesNotExist );

        // --- 1. Rate limit for network registrations.
        let current_block = Self::get_current_block_as_u64();
        let last_lock_block = Self::get_network_last_lock_block();
        ensure!(
            current_block.saturating_sub(last_lock_block) >= NetworkRateLimit::<T>::get(),
            Error::<T>::NetworkTxRateLimitExceeded
        );

        // --- 2. Calculate and lock the required tokens.
        let lock_amount: u64 = Self::get_network_lock_cost();
        log::debug!("network lock_amount: {:?}", lock_amount);
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, lock_amount),
            Error::<T>::NotEnoughBalanceToStake
        );

        // --- 4. Determine the netuid to register.
        let netuid_to_register: u16 = {
            log::debug!(
                "subnet count: {:?}\nmax subnets: {:?}",
                Self::get_num_subnets(),
                Self::get_max_subnets()
            );
            if Self::get_num_subnets().saturating_sub(1) < Self::get_max_subnets() {
                // We subtract one because we don't want root subnet to count towards total
                let mut next_available_netuid = 0;
                loop {
                    next_available_netuid.saturating_inc();
                    if !Self::if_subnet_exist(next_available_netuid) {
                        log::debug!("got subnet id: {:?}", next_available_netuid);
                        break next_available_netuid;
                    }
                }
            } else {
                let netuid_to_prune = Self::get_subnet_to_prune();
                ensure!(netuid_to_prune > 0, Error::<T>::AllNetworksInImmunity);

                Self::remove_network(netuid_to_prune);
                log::debug!("remove_network: {:?}", netuid_to_prune,);
                Self::deposit_event(Event::NetworkRemoved(netuid_to_prune));
                netuid_to_prune
            }
        };

        // --- 5. Perform the lock operation.
        let actual_tao_lock_amount = Self::remove_balance_from_coldkey_account(&coldkey, lock_amount)?;
        log::debug!("actual_tao_lock_amount: {:?}", actual_tao_lock_amount);

        // Self::set_subnet_locked_balance(netuid_to_register, actual_tao_lock_amount);
        Self::set_network_last_lock(actual_tao_lock_amount);

        // --- 6. Set initial and custom parameters for the network.
        Self::init_new_network(netuid_to_register, 360);
        log::debug!("init_new_network: {:?}", netuid_to_register);

        // --- 7. Set netuid storage.
        let current_block_number: u64 = Self::get_current_block_as_u64();
        log::debug!("Current block number: {:?}", current_block_number);

        Self::append_neuron( netuid_to_register, hotkey, current_block_number );
        log::debug!("Appended neuron for netuid {:?}, hotkey: {:?}", netuid_to_register, hotkey);

        NetworkLastRegistered::<T>::set(current_block_number);
        log::debug!("NetworkLastRegistered set to: {:?}", current_block_number);

        NetworkRegisteredAt::<T>::insert(netuid_to_register, current_block_number);
        log::debug!("NetworkRegisteredAt for netuid {:?} set to: {:?}", netuid_to_register, current_block_number);

        SubnetOwner::<T>::insert(netuid_to_register, coldkey.clone());
        log::debug!("SubnetOwner for netuid {:?} set to: {:?}", netuid_to_register, coldkey);

        SubnetMechanism::<T>::insert( netuid_to_register, mechid );
        log::debug!("SubnetMechanism for netuid {:?} set to: {:?}", netuid_to_register, mechid);

        // --- 7.1. Compute the stake operation based on the mechanism.
        let _ = Self::stake_into_subnet( hotkey, &coldkey, netuid_to_register, actual_tao_lock_amount );

        // --- 8. Emit the NetworkAdded event.
        log::info!(
            "NetworkAdded( netuid:{:?}, mechanism:{:?} )",
            netuid_to_register,
            mechid
        );
        Self::deposit_event(Event::NetworkAdded(netuid_to_register, 0));

        // --- 9. Return success.
        Ok(())
    }

    /// Facilitates the removal of a user's subnetwork.
    ///
    /// # Args:
    /// * 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    /// * 'netuid': ('u16'): The unique identifier of the network to be removed.
    ///
    /// # Event:
    /// * 'NetworkRemoved': Emitted when a network is successfully removed.
    ///
    /// # Raises:
    /// * 'SubNetworkDoesNotExist': If the specified network does not exist.
    /// * 'NotSubnetOwner': If the caller does not own the specified subnet.
    ///
    pub fn user_remove_network(origin: T::RuntimeOrigin, netuid: u16) -> dispatch::DispatchResult {
        // --- 1. Ensure the function caller is a signed user.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Ensure this subnet exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Ensure the caller owns this subnet.
        ensure!(
            SubnetOwner::<T>::get(netuid) == coldkey,
            Error::<T>::NotSubnetOwner
        );

        // --- 4. Explicitly erase the network and all its parameters.
        Self::remove_network(netuid);

        // --- 5. Emit the NetworkRemoved event.
        log::info!("NetworkRemoved( netuid:{:?} )", netuid);
        Self::deposit_event(Event::NetworkRemoved(netuid));

        // --- 6. Return success.
        Ok(())
    }

    /// Sets initial and custom parameters for a new network.
    pub fn init_new_network(netuid: u16, tempo: u16) {
        // --- 1. Set network to 0 size.
        SubnetworkN::<T>::insert(netuid, 0);

        // --- 2. Set this network uid to alive.
        NetworksAdded::<T>::insert(netuid, true);

        // --- 3. Fill tempo memory item.
        Tempo::<T>::insert(netuid, tempo);

        // --- 4 Fill modality item.
        NetworkModality::<T>::insert(netuid, 0);

        // --- 5. Increase total network count.
        TotalNetworks::<T>::mutate(|n| *n = n.saturating_add(1));

        // --- 6. Set all default values **explicitly**.
        Self::set_network_registration_allowed(netuid, true);
        Self::set_max_allowed_uids(netuid, 256);
        Self::set_max_allowed_validators(netuid, 64);
        Self::set_min_allowed_weights(netuid, 1);
        Self::set_max_weight_limit(netuid, u16::MAX);
        Self::set_adjustment_interval(netuid, 360);
        Self::set_target_registrations_per_interval(netuid, 1);
        Self::set_adjustment_alpha(netuid, 17_893_341_751_498_265_066); // 18_446_744_073_709_551_615 * 0.97 = 17_893_341_751_498_265_066
        Self::set_immunity_period(netuid, 5000);
        Self::set_min_burn(netuid, 1);
        Self::set_min_difficulty(netuid, u64::MAX);
        Self::set_max_difficulty(netuid, u64::MAX);

        // Make network parameters explicit.
        if !Tempo::<T>::contains_key(netuid) {
            Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));
        }
        if !Kappa::<T>::contains_key(netuid) {
            Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));
        }
        if !Difficulty::<T>::contains_key(netuid) {
            Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));
        }
        if !MaxAllowedUids::<T>::contains_key(netuid) {
            MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));
        }
        if !ImmunityPeriod::<T>::contains_key(netuid) {
            ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));
        }
        if !ActivityCutoff::<T>::contains_key(netuid) {
            ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));
        }
        if !EmissionValues::<T>::contains_key(netuid) {
            EmissionValues::<T>::insert(netuid, EmissionValues::<T>::get(netuid));
        }
        if !MaxWeightsLimit::<T>::contains_key(netuid) {
            MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));
        }
        if !MinAllowedWeights::<T>::contains_key(netuid) {
            MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid));
        }
        if !RegistrationsThisInterval::<T>::contains_key(netuid) {
            RegistrationsThisInterval::<T>::insert(
                netuid,
                RegistrationsThisInterval::<T>::get(netuid),
            );
        }
        if !POWRegistrationsThisInterval::<T>::contains_key(netuid) {
            POWRegistrationsThisInterval::<T>::insert(
                netuid,
                POWRegistrationsThisInterval::<T>::get(netuid),
            );
        }
        if !BurnRegistrationsThisInterval::<T>::contains_key(netuid) {
            BurnRegistrationsThisInterval::<T>::insert(
                netuid,
                BurnRegistrationsThisInterval::<T>::get(netuid),
            );
        }
    }

    /// Removes a network (identified by netuid) and all associated parameters.
    ///
    /// This function is responsible for cleaning up all the data associated with a network.
    /// It ensures that all the storage values related to the network are removed, and any
    /// reserved balance is returned to the network owner.
    ///
    /// # Args:
    ///  * 'netuid': ('u16'): The unique identifier of the network to be removed.
    ///
    /// # Note:
    /// This function does not emit any events, nor does it raise any errors. It silently
    /// returns if any internal checks fail.
    ///
    pub fn remove_network(netuid: u16) {
        // --- 1. Return balance to subnet owner.
        let owner_coldkey = SubnetOwner::<T>::get(netuid);
        let reserved_amount = Self::get_subnet_locked_balance(netuid);

        // --- 2. Remove network count.
        SubnetworkN::<T>::remove(netuid);

        // --- 3. Remove network modality storage.
        NetworkModality::<T>::remove(netuid);

        // --- 4. Remove netuid from added networks.
        NetworksAdded::<T>::remove(netuid);

        // --- 6. Decrement the network counter.
        TotalNetworks::<T>::mutate(|n| *n = n.saturating_sub(1));

        // --- 7. Remove various network-related storages.
        NetworkRegisteredAt::<T>::remove(netuid);

        // --- 8. Remove incentive mechanism memory.
        let _ = Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Bonds::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 8. Removes the weights for this subnet (do not remove).
        let _ = Weights::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 9. Iterate over stored weights and fill the matrix.
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(
                Self::get_root_netuid(),
            )
        {
            // Create a new vector to hold modified weights.
            let mut modified_weights = weights_i.clone();
            // Iterate over each weight entry to potentially update it.
            for (subnet_id, weight) in modified_weights.iter_mut() {
                if subnet_id == &netuid {
                    // If the condition matches, modify the weight
                    *weight = 0; // Set weight to 0 for the matching subnet_id.
                }
            }
            Weights::<T>::insert(Self::get_root_netuid(), uid_i, modified_weights);
        }

        // --- 10. Remove various network-related parameters.
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

        // --- 11. Erase network parameters.
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

        // --- 12. Add the balance back to the owner.
        Self::add_balance_to_coldkey_account(&owner_coldkey, reserved_amount);
        Self::set_subnet_locked_balance(netuid, 0);
        SubnetOwner::<T>::remove(netuid);
    }


    /// This function is used to determine which subnet to prune when the total number of networks has reached the limit.
    /// It iterates over all the networks and finds the oldest subnet with the minimum emission value that is not in the immunity period.
    ///
    /// # Returns:
    /// * 'u16':
    ///     - The uid of the network to be pruned.
    ///
    pub fn get_subnet_to_prune() -> u16 {
        let mut netuids: Vec<u16> = vec![];
        let current_block = Self::get_current_block_as_u64();

        // Even if we don't have a root subnet, this still works
        for netuid in NetworksAdded::<T>::iter_keys_from(NetworksAdded::<T>::hashed_key_for(0)) {
            if current_block.saturating_sub(Self::get_network_registered_block(netuid))
                < Self::get_network_immunity_period()
            {
                continue;
            }

            // This iterator seems to return them in order anyways, so no need to sort by key
            netuids.push(netuid);
        }

        // Now we sort by emission, and then by subnet creation time.
        netuids.sort_by(|a, b| {
            use sp_std::cmp::Ordering;

            match Self::get_emission_value(*b).cmp(&Self::get_emission_value(*a)) {
                Ordering::Equal => {
                    if Self::get_network_registered_block(*b)
                        < Self::get_network_registered_block(*a)
                    {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                }
                v => v,
            }
        });

        log::info!("Netuids Order: {:?}", netuids);

        match netuids.last() {
            Some(netuid) => *netuid,
            None => 0,
        }
    }


}