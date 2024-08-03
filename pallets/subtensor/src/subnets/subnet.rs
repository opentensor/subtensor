use super::*;
use frame_support::IterableStorageMap;

impl<T: Config> Pallet<T> {
    /// Fetches the total count of subnets.
    ///
    /// This function retrieves the total number of subnets present on the chain.
    ///
    /// # Returns:
    /// * 'u16': The total number of subnets.
    ///
    pub fn get_num_subnets() -> u16 {
        TotalNetworks::<T>::get()
    }

    /// Fetches the max number of subnet
    ///
    /// This function retrieves the max number of subnet.
    ///
    /// # Returns:
    /// * 'u16': The max number of subnet
    ///
    pub fn get_max_subnets() -> u16 {
        SubnetLimit::<T>::get()
    }

    /// Sets the max number of subnet
    ///
    /// This function sets the max number of subnet.
    ///
    pub fn set_max_subnets(limit: u16) {
        SubnetLimit::<T>::put(limit);
        Self::deposit_event(Event::SubnetLimitSet(limit));
    }

    /// Returns true if the subnetwork exists.
    ///
    /// This function checks if a subnetwork with the given UID exists.
    ///
    /// # Returns:
    /// * 'bool': Whether the subnet exists.
    ///
    pub fn if_subnet_exist(netuid: u16) -> bool {
        NetworksAdded::<T>::get(netuid)
    }

    /// Returns a list of subnet netuid equal to total networks.
    ///
    ///
    /// This iterates through all the networks and returns a list of netuids.
    ///
    /// # Returns:
    /// * 'Vec<u16>': Netuids of all subnets.
    ///
    pub fn get_all_subnet_netuids() -> Vec<u16> {
        <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter()
            .map(|(netuid, _)| netuid)
            .collect()
    }

    /// Finds the next available mechanism ID.
    ///
    /// This function iterates through possible mechanism IDs starting from 0
    /// until it finds an ID that is not currently in use.
    ///
    /// # Returns
    /// * `u16` - The next available mechanism ID.
    pub fn get_next_netuid() -> u16 {
        let mut next_netuid = 0;
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        loop {
            if !netuids.contains(&next_netuid) {
                break next_netuid;
            }
            next_netuid = next_netuid.saturating_add(1);
        }
    }

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
    pub fn user_add_network(
        origin: T::RuntimeOrigin,
        hotkey: &T::AccountId,
        mechid: u16,
    ) -> DispatchResult {
        // --- 0. Ensure the caller is a signed user.
        let coldkey = ensure_signed(origin)?;

        // Ensure this is a new hotkey cannot be owned by anyone else.
        if Self::hotkey_account_exists(hotkey) {
            ensure!(
                Self::coldkey_owns_hotkey(&coldkey, hotkey),
                Error::<T>::NonAssociatedColdKey
            );
        }
        // --- 0.1. Ensure the mechanism exists either Stable or Dynamic.
        ensure!(
            mechid == 0 || mechid == 1,
            Error::<T>::MechanismDoesNotExist
        );

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
        let netuid_to_register: u16 = Self::get_next_netuid();

        // --- 5. Perform the lock operation.
        let actual_tao_lock_amount =
            Self::remove_balance_from_coldkey_account(&coldkey, lock_amount)?;
        log::debug!("actual_tao_lock_amount: {:?}", actual_tao_lock_amount);

        // Self::set_subnet_locked_balance(netuid_to_register, actual_tao_lock_amount);
        Self::set_network_last_lock(actual_tao_lock_amount);

        // --- 6. Set initial and custom parameters for the network.
        Self::init_new_network(netuid_to_register, 360);
        log::debug!("init_new_network: {:?}", netuid_to_register);

        // --- 7. Add the caller to the neuron set.
        Self::create_account_if_non_existent(&coldkey, hotkey);
        Self::append_neuron(netuid_to_register, hotkey, current_block);
        log::debug!(
            "Appended neuron for netuid {:?}, hotkey: {:?}",
            netuid_to_register,
            hotkey
        );

        // --- 9. Set the mechanism.
        SubnetMechanism::<T>::insert(netuid_to_register, mechid);
        log::debug!(
            "SubnetMechanism for netuid {:?} set to: {:?}",
            netuid_to_register,
            mechid
        );

        // --- 10 Set the creation terms.
        NetworkLastRegistered::<T>::set(current_block);
        NetworkRegisteredAt::<T>::insert(netuid_to_register, current_block);

        // Init the pool.
        let tao_init: u64 = 1_000_000_000; // 1 TAO to init the pool.
        SubnetTAO::<T>::insert(netuid_to_register, tao_init); // add the TAO to the pool.
        SubnetAlphaIn::<T>::insert(netuid_to_register, actual_tao_lock_amount); // Set the alpha in based on the lock.
        let alpha_locked: u64 = Self::stake_into_subnet(
            hotkey,
            &coldkey,
            netuid_to_register,
            actual_tao_lock_amount.saturating_sub(tao_init),
        ); 

        // --- 8. Set the owner.
        // Set the coldkey as the current owner
        SubnetOwner::<T>::insert(netuid_to_register, coldkey.clone()); 
        // Set the hotkey as the lock.
        Locks::<T>::insert(
            (netuid_to_register, coldkey.clone(), hotkey.clone()), 
            (alpha_locked, current_block, current_block.saturating_add(Self::get_lock_interval_blocks()))
        );

        // --- 12. Emit the NetworkAdded event.
        log::info!(
            "NetworkAdded( netuid:{:?}, mechanism:{:?} )",
            netuid_to_register,
            mechid
        );
        Self::deposit_event(Event::NetworkAdded(netuid_to_register, 0));

        // --- 9. Return success.
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
}
