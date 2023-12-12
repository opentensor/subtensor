use
{
};

impl<T: Config> Pallet<T> 
{
    // Facilitates user registration of a new subnetwork.
    //
    // # Args:
    // 	* 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    //
    // # Event:
    // 	* 'NetworkAdded': Emitted when a new network is successfully added.
    //
    // # Raises:
    // 	* 'TxRateLimitExceeded': If the rate limit for network registration is exceeded.
    // 	* 'NotEnoughBalanceToStake': If there isn't enough balance to stake for network registration.
    // 	* 'BalanceWithdrawalError': If an error occurs during balance withdrawal for network registration.
    //
    pub fn user_add_network(origin: T::RuntimeOrigin) -> dispatch::DispatchResult 
    {
        // --- 0. Ensure the caller is a signed user.
        let coldkey: T::AccountId;
        {
            coldkey = ensure_signed(origin)?;
        } 

        // --- 1. Rate limit for network registrations.
        {
            let current_block:      u64 = Self::get_current_block_as_u64();
            let last_lock_block:    u64 = Self::get_network_last_lock_block();

            ensure!(
                current_block - last_lock_block >= Self::get_network_rate_limit(), 
                Error::<T>::TxRateLimitExceeded
            );
        }

        // --- 2. Calculate and lock the required tokens.
        let lock_as_balance;
        let lock_amount: u64;
        {
            lock_amount = Self::get_network_lock_cost();
            lock_as_balance = Self::u64_to_balance(lock_amount);

            log::debug!("network lock_amount: {:?}", lock_amount);

            ensure!(
                lock_as_balance.is_some(),
                Error::<T>::CouldNotConvertToBalance
            );

            ensure!(
                Self::can_remove_balance_from_coldkey_account(&coldkey, lock_as_balance.unwrap()),
                Error::<T>::NotEnoughBalanceToStake
            );
        }

        // --- 4. Determine the netuid to register.
        let netuid_to_register: u16 = 
        {
            log::debug!("subnet count: {:?}\nmax subnets: {:?}", Self::get_num_subnets(), Self::get_max_subnets());

            if Self::get_num_subnets().saturating_sub(1) < Self::get_max_subnets() // We subtract one because we don't want root subnet to count towards total
            { 
                let mut next_available_netuid = 0;
                loop 
                {
                    next_available_netuid += 1;

                    if !Self::if_subnet_exist(next_available_netuid) 
                    {
                        log::debug!("got subnet id: {:?}", next_available_netuid);

                        break next_available_netuid;
                    }
                }
            }
            else 
            {
                let netuid_to_prune = Self::get_subnet_to_prune();
                ensure!(netuid_to_prune > 0, Error::<T>::AllNetworksInImmunity);

                Self::remove_network(netuid_to_prune);
                log::debug!("remove_network: {:?}", netuid_to_prune);

                netuid_to_prune
            }
        };

        // --- 5. Perform the lock operation.
        {
            ensure!(
                Self::remove_balance_from_coldkey_account(&coldkey, lock_as_balance.unwrap()) == true,
                Error::<T>::BalanceWithdrawalError
            );

            Self::set_subnet_locked_balance(netuid_to_register, lock_amount);
            Self::set_network_last_lock(lock_amount);
        }

        // --- 6. Set initial and custom parameters for the network.
        {
            Self::init_new_network(netuid_to_register, 360);

            log::debug!("init_new_network: {:?}", netuid_to_register);
        }

        // --- 7. Set netuid storage.
        {
            let current_block_number: u64 = Self::get_current_block_as_u64();
            
            NetworkLastRegistered::<T>::set(current_block_number);
            NetworkRegisteredAt::<T>::insert(netuid_to_register, current_block_number);
            SubnetOwner::<T>::insert(netuid_to_register, coldkey);
        }

        // --- 8. Emit the NetworkAdded event.
        {
            log::info!(
                "NetworkAdded( netuid:{:?}, modality:{:?} )",
                netuid_to_register,
                0
            );
            
            Self::deposit_event(Event::NetworkAdded(netuid_to_register, 0));
        }

        // --- 9. Return success.
        return Ok(());
    }

    // Facilitates the removal of a user's subnetwork.
    //
    // # Args:
    // 	* 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    //     * 'netuid': ('u16'): The unique identifier of the network to be removed.
    //
    // # Event:
    // 	* 'NetworkRemoved': Emitted when a network is successfully removed.
    //
    // # Raises:
    // 	* 'NetworkDoesNotExist': If the specified network does not exist.
    // 	* 'NotSubnetOwner': If the caller does not own the specified subnet.
    //
    pub fn user_remove_network(origin: T::RuntimeOrigin, netuid: u16) -> dispatch::DispatchResult 
    {
        // --- 1. Ensure the function caller is a signed user.
        let coldkey: T::AccountId;
        {
            coldkey = ensure_signed(origin)?;
        }

        // --- 2. Ensure this subnet exists.
        {
            ensure!(
                Self::if_subnet_exist(netuid),
                Error::<T>::NetworkDoesNotExist
            );
        }

        // --- 3. Ensure the caller owns this subnet.
        {
            ensure!(
                SubnetOwner::<T>::get(netuid) == coldkey,
                Error::<T>::NotSubnetOwner
            );
        }

        // --- 4. Explicitly erase the network and all its parameters.
        {
            Self::remove_network(netuid);
        }

        // --- 5. Emit the NetworkRemoved event.
        {
            log::info!("NetworkRemoved( netuid:{:?} )", netuid);

            Self::deposit_event(Event::NetworkRemoved(netuid));
        }

        // --- 6. Return success.
        return Ok(());
    }
}