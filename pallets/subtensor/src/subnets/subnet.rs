use super::*;
use sp_core::Get;
use subtensor_runtime_common::{NetUid, TaoCurrency};

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

    /// Returns true if the subnetwork exists.
    ///
    /// This function checks if a subnetwork with the given UID exists.
    ///
    /// # Returns:
    /// * 'bool': Whether the subnet exists.
    ///
    pub fn if_subnet_exist(netuid: NetUid) -> bool {
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
    pub fn get_all_subnet_netuids() -> Vec<NetUid> {
        NetworksAdded::<T>::iter()
            .map(|(netuid, _)| netuid)
            .collect()
    }

    /// Returns the mechanism id for a subnet.
    ///
    ///
    /// This checks the Mechanism map for the value, defaults to 0.
    ///
    /// # Args:
    /// * 'u16': The subnet netuid
    ///
    /// # Returns:
    /// * 'u16': The subnet mechanism
    ///
    pub fn get_subnet_mechanism(netuid: NetUid) -> u16 {
        SubnetMechanism::<T>::get(netuid)
    }

    /// Finds the next available mechanism ID.
    ///
    /// This function iterates through possible mechanism IDs starting from 0
    /// until it finds an ID that is not currently in use.
    ///
    /// # Returns
    /// * `u16` - The next available mechanism ID.
    pub fn get_next_netuid() -> NetUid {
        let mut next_netuid = NetUid::from(1); // do not allow creation of root
        let netuids = Self::get_all_subnet_netuids();
        loop {
            if !netuids.contains(&next_netuid) {
                break next_netuid;
            }
            next_netuid = next_netuid.next();
        }
    }

    /// Sets the network rate limit and emit the `NetworkRateLimitSet` event
    ///
    pub fn set_network_rate_limit(limit: u64) {
        NetworkRateLimit::<T>::set(limit);
        Self::deposit_event(Event::NetworkRateLimitSet(limit));
    }

    /// Checks if registrations are allowed for a given subnet.
    ///
    /// This function retrieves the subnet hyperparameters for the specified subnet and checks the
    /// `registration_allowed` flag. If the subnet doesn't exist or doesn't have hyperparameters
    /// defined, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if registrations are allowed for the subnet, `false` otherwise.
    pub fn is_registration_allowed(netuid: NetUid) -> bool {
        Self::get_subnet_hyperparams(netuid)
            .map(|params| params.registration_allowed)
            .unwrap_or(false)
    }

    /// Facilitates user registration of a new subnetwork.
    ///
    /// ### Args
    /// * **`origin`** – `T::RuntimeOrigin` &nbsp;Must be **signed** by the coldkey.  
    /// * **`hotkey`** – `&T::AccountId` &nbsp;First neuron of the new subnet.  
    /// * **`mechid`** – `u16` &nbsp;Only the dynamic mechanism (`1`) is currently supported.  
    /// * **`identity`** – `Option<SubnetIdentityOfV3>` &nbsp;Optional metadata for the subnet.
    ///
    /// ### Events
    /// * `NetworkAdded(netuid, mechid)` – always.  
    /// * `SubnetIdentitySet(netuid)`   – when a custom identity is supplied.  
    /// * `NetworkRemoved(netuid)`      – when a subnet is pruned to make room.
    ///
    /// ### Errors
    /// * `NonAssociatedColdKey`            – `hotkey` already belongs to another coldkey.  
    /// * `MechanismDoesNotExist`           – unsupported `mechid`.  
    /// * `NetworkTxRateLimitExceeded`      – caller hit the register-network rate limit.  
    /// * `SubnetLimitReached`              – limit hit **and** no eligible subnet to prune.  
    /// * `CannotAffordLockCost`            – caller lacks the lock cost.  
    /// * `BalanceWithdrawalError`          – failed to lock balance.  
    /// * `InvalidIdentity`                 – supplied `identity` failed validation.
    ///
    pub fn do_register_network(
        origin: T::RuntimeOrigin,
        hotkey: &T::AccountId,
        mechid: u16,
        identity: Option<SubnetIdentityOfV3>,
    ) -> DispatchResult {
        // --- 1. Ensure the caller is a signed user.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Ensure the hotkey does not exist or is owned by the coldkey.
        ensure!(
            !Self::hotkey_account_exists(hotkey) || Self::coldkey_owns_hotkey(&coldkey, hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 3. Ensure the mechanism is Dynamic.
        ensure!(mechid == 1, Error::<T>::MechanismDoesNotExist);

        // --- 4. Rate limit for network registrations.
        let current_block = Self::get_current_block_as_u64();
        let last_lock_block = Self::get_network_last_lock_block();
        ensure!(
            current_block.saturating_sub(last_lock_block) >= NetworkRateLimit::<T>::get(),
            Error::<T>::NetworkTxRateLimitExceeded
        );

        // --- 5. Check if we need to prune a subnet (if at SubnetLimit).
        //         But do not prune yet; we only do it after all checks pass.
        let subnet_limit = Self::get_max_subnets();
        let current_count: u16 = NetworksAdded::<T>::iter()
            .filter(|(netuid, added)| *added && *netuid != NetUid::ROOT)
            .count() as u16;

        let mut recycle_netuid: Option<NetUid> = None;
        if current_count >= subnet_limit {
            if let Some(netuid) = Self::get_network_to_prune() {
                recycle_netuid = Some(netuid);
            } else {
                return Err(Error::<T>::SubnetLimitReached.into());
            }
        }

        // --- 6. Calculate and lock the required tokens.
        let lock_amount = Self::get_network_lock_cost();
        log::debug!("network lock_amount: {lock_amount:?}");
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, lock_amount.into()),
            Error::<T>::CannotAffordLockCost
        );

        // --- 7. Perform the lock operation.
        let actual_tao_lock_amount =
            Self::remove_balance_from_coldkey_account(&coldkey, lock_amount.into())?;
        log::debug!("actual_tao_lock_amount: {actual_tao_lock_amount:?}");

        // --- 8. Set the lock amount for use to determine pricing.
        Self::set_network_last_lock(actual_tao_lock_amount);

        // --- 9. If we identified a subnet to prune, do it now.
        if let Some(prune_netuid) = recycle_netuid {
            Self::do_dissolve_network(prune_netuid)?;
        }

        // --- 10. Determine netuid to register. If we pruned a subnet, reuse that netuid.
        let netuid_to_register: NetUid = match recycle_netuid {
            Some(prune_netuid) => prune_netuid,
            None => Self::get_next_netuid(),
        };

        // --- 11. Set initial and custom parameters for the network.
        let default_tempo = DefaultTempo::<T>::get();
        Self::init_new_network(netuid_to_register, default_tempo);
        log::debug!("init_new_network: {netuid_to_register:?}");

        // --- 12. Add the caller to the neuron set.
        Self::create_account_if_non_existent(&coldkey, hotkey);
        Self::append_neuron(netuid_to_register, hotkey, current_block);
        log::debug!("Appended neuron for netuid {netuid_to_register:?}, hotkey: {hotkey:?}");

        // --- 13. Set the mechanism.
        SubnetMechanism::<T>::insert(netuid_to_register, mechid);
        log::debug!("SubnetMechanism for netuid {netuid_to_register:?} set to: {mechid:?}");

        // --- 14. Set the creation terms.
        NetworkLastRegistered::<T>::set(current_block);
        NetworkRegisteredAt::<T>::insert(netuid_to_register, current_block);

        // --- 15. Set the symbol.
        let symbol = Self::get_next_available_symbol(netuid_to_register);
        TokenSymbol::<T>::insert(netuid_to_register, symbol);

        // The initial TAO is the locked amount
        // Put initial TAO from lock into subnet TAO and produce numerically equal amount of Alpha.
        let pool_initial_tao: TaoCurrency = Self::get_network_min_lock();
        let pool_initial_alpha: AlphaCurrency = pool_initial_tao.to_u64().into();
        let actual_tao_lock_amount_less_pool_tao =
            actual_tao_lock_amount.saturating_sub(pool_initial_tao);

        // Core pool + ownership
        SubnetTAO::<T>::insert(netuid_to_register, pool_initial_tao);
        SubnetAlphaIn::<T>::insert(netuid_to_register, pool_initial_alpha);
        SubnetOwner::<T>::insert(netuid_to_register, coldkey.clone());
        SubnetOwnerHotkey::<T>::insert(netuid_to_register, hotkey.clone());
        TransferToggle::<T>::insert(netuid_to_register, true);
        SubnetLocked::<T>::insert(netuid_to_register, pool_initial_tao);
        LargestLocked::<T>::insert(netuid_to_register, pool_initial_tao.to_u64());
        SubnetTaoProvided::<T>::insert(netuid_to_register, TaoCurrency::ZERO);
        SubnetAlphaInProvided::<T>::insert(netuid_to_register, AlphaCurrency::from(0));
        SubnetAlphaOut::<T>::insert(netuid_to_register, AlphaCurrency::from(0));
        SubnetVolume::<T>::insert(netuid_to_register, 0u128);
        RAORecycledForRegistration::<T>::insert(
            netuid_to_register,
            actual_tao_lock_amount_less_pool_tao,
        );

        if actual_tao_lock_amount_less_pool_tao > TaoCurrency::ZERO {
            Self::burn_tokens(actual_tao_lock_amount_less_pool_tao);
        }

        if actual_tao_lock_amount > TaoCurrency::ZERO && pool_initial_tao > TaoCurrency::ZERO {
            // Record in TotalStake the initial TAO in the pool.
            Self::increase_total_stake(pool_initial_tao);
        }

        // --- 17. Add the identity if it exists
        if let Some(identity_value) = identity {
            ensure!(
                Self::is_valid_subnet_identity(&identity_value),
                Error::<T>::InvalidIdentity
            );

            SubnetIdentitiesV3::<T>::insert(netuid_to_register, identity_value);
            Self::deposit_event(Event::SubnetIdentitySet(netuid_to_register));
        }

        // --- 18. Emit the NetworkAdded event.
        log::info!("NetworkAdded( netuid:{netuid_to_register:?}, mechanism:{mechid:?} )");
        Self::deposit_event(Event::NetworkAdded(netuid_to_register, mechid));

        // --- 19. Return success.
        Ok(())
    }

    /// Sets initial and custom parameters for a new network.
    pub fn init_new_network(netuid: NetUid, tempo: u16) {
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
        Self::set_min_difficulty(netuid, u64::MAX);
        Self::set_max_difficulty(netuid, u64::MAX);
        Self::set_commit_reveal_weights_enabled(netuid, true);
        Self::set_yuma3_enabled(netuid, true);

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

    /// Execute the start call for a subnet.
    ///
    /// This function is used to trigger the start call process for a subnet identified by `netuid`.
    /// It ensures that the subnet exists, the caller is the subnet owner,
    /// and the last emission block number has not been set yet.
    /// It then sets the last emission block number to the current block number.
    ///
    /// # Parameters
    ///
    /// * `origin`: The origin of the call, which is used to ensure the caller is the subnet owner.
    /// * `netuid`: The unique identifier of the subnet for which the start call process is being initiated.
    ///
    /// # Raises
    ///
    /// * `Error::<T>::SubNetworkDoesNotExist`: If the subnet does not exist.
    /// * `DispatchError::BadOrigin`: If the caller is not the subnet owner.
    /// * `Error::<T>::FirstEmissionBlockNumberAlreadySet`: If the last emission block number has already been set.
    ///
    /// # Returns
    ///
    /// * `DispatchResult`: A result indicating the success or failure of the operation.
    pub fn do_start_call(origin: T::RuntimeOrigin, netuid: NetUid) -> DispatchResult {
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );
        Self::ensure_subnet_owner(origin, netuid)?;
        ensure!(
            FirstEmissionBlockNumber::<T>::get(netuid).is_none(),
            Error::<T>::FirstEmissionBlockNumberAlreadySet
        );

        let registration_block_number = NetworkRegisteredAt::<T>::get(netuid);
        let current_block_number = Self::get_current_block_as_u64();

        ensure!(
            current_block_number
                >= registration_block_number.saturating_add(T::DurationOfStartCall::get()),
            Error::<T>::NeedWaitingMoreBlocksToStarCall
        );
        let next_block_number = current_block_number.saturating_add(1);

        FirstEmissionBlockNumber::<T>::insert(netuid, next_block_number);
        SubtokenEnabled::<T>::insert(netuid, true);
        Self::deposit_event(Event::FirstEmissionBlockNumberSet(
            netuid,
            next_block_number,
        ));
        Ok(())
    }

    /// Sets or updates the hotkey account associated with the owner of a specific subnet.
    ///
    /// This function allows either the root origin or the current subnet owner to set or update
    /// the hotkey for a given subnet. The subnet must already exist. To prevent abuse, the call is
    /// rate-limited to once per configured interval (default: one week) per subnet.
    ///
    /// # Parameters
    /// - `origin`: The dispatch origin of the call. Must be either root or the current owner of the subnet.
    /// - `netuid`: The unique identifier of the subnet whose owner hotkey is being set.
    /// - `hotkey`: The new hotkey account to associate with the subnet owner.
    ///
    /// # Returns
    /// - `DispatchResult`: Returns `Ok(())` if the hotkey was successfully set, or an appropriate error otherwise.
    ///
    /// # Errors
    /// - `Error::SubnetNotExists`: If the specified subnet does not exist.
    /// - `Error::TxRateLimitExceeded`: If the function is called more frequently than the allowed rate limit.
    ///
    /// # Access Control
    /// Only callable by:
    /// - Root origin, or
    /// - The coldkey account that owns the subnet.
    ///
    /// # Storage
    /// - Updates [`SubnetOwnerHotkey`] for the given `netuid`.
    /// - Reads and updates [`LastRateLimitedBlock`] for rate-limiting.
    /// - Reads [`DefaultSetSNOwnerHotkeyRateLimit`] to determine the interval between allowed updates.
    ///
    /// # Rate Limiting
    /// This function is rate-limited to one call per subnet per interval (e.g., one week).
    pub fn do_set_sn_owner_hotkey(
        origin: T::RuntimeOrigin,
        netuid: NetUid,
        hotkey: &T::AccountId,
    ) -> DispatchResult {
        // Ensure the caller is either root or subnet owner.
        Self::ensure_subnet_owner_or_root(origin, netuid)?;

        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Rate limit: 1 call per week
        ensure!(
            Self::passes_rate_limit_on_subnet(
                &TransactionType::SetSNOwnerHotkey,
                hotkey, // ignored
                netuid, // Specific to a subnet.
            ),
            Error::<T>::TxRateLimitExceeded
        );

        // Set last transaction block
        let current_block = Self::get_current_block_as_u64();
        Self::set_last_transaction_block_on_subnet(
            hotkey,
            netuid,
            &TransactionType::SetSNOwnerHotkey,
            current_block,
        );

        // Insert/update the hotkey
        SubnetOwnerHotkey::<T>::insert(netuid, hotkey);

        // Return success.
        Ok(())
    }

    pub fn is_valid_subnet_for_emission(netuid: NetUid) -> bool {
        FirstEmissionBlockNumber::<T>::get(netuid).is_some()
    }
}
