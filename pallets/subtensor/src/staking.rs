use super::*;
use frame_support::{
    traits::{
        tokens::{
            fungible::{Balanced as _, Inspect as _, Mutate as _},
            Fortitude, Precision, Preservation,
        },
        Imbalance,
    },
};
use sp_core::Get;
use sp_std::vec::Vec;
use substrate_fixed::types::I64F64;
use types::SubnetType;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic become_delegate: signals that this hotkey allows delegated stake.
    ///
    /// # Args:
    /// *  'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     - The signature of the caller's coldkey.
    ///
    /// *  'hotkey' (T::AccountId):
    ///     - The hotkey we are delegating (must be owned by the coldkey.)
    ///
    /// # Event:
    /// *  DelegateAdded;
    ///     - On successfully setting a hotkey as a delegate.
    ///
    /// # Raises:
    /// *  'NotRegistered':
    ///     - The hotkey we are delegating is not registered on the network.
    ///
    /// *  'NonAssociatedColdKey':
    ///     - The hotkey we are delegating is not owned by the calling coldket.
    ///
    /// *  'TxRateLimitExceeded':
    ///     - Thrown if key has hit transaction rate limit
    ///
    pub fn do_become_delegate(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_become_delegate( origin:{:?} hotkey:{:?} )",
            coldkey,
            hotkey
        );

        // --- 2. Ensure we are delegating a known key.
        // --- 3. Ensure that the coldkey is the owner.
        Self::do_account_checks(&coldkey, &hotkey)?;

        // --- 5. Ensure we are not already a delegate
        ensure!(
            !Self::hotkey_is_delegate(&hotkey),
            Error::<T>::HotKeyAlreadyDelegate
        );

        // --- 6. Ensure we don't exceed tx rate limit
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::DelegateTxRateLimitExceeded
        );

        // --- 7. Delegate the key.
        // With introduction of DelegatesTake Delegates became just a flag.
        // Probably there is a migration needed to convert it to bool or something down the road
        Self::delegate_hotkey(&hotkey, Self::get_default_take());
                
        // Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);

        // Also, set last block for take increase rate limiting, since default take is max
        Self::set_last_tx_block_delegate_take(&coldkey, block);

        // --- 8. Emit the staking event.
        log::info!(
            "DelegateAdded( coldkey:{:?}, hotkey:{:?} )",
            coldkey,
            hotkey
        );
        Self::deposit_event(Event::DelegateAdded(
            coldkey,
            hotkey,
            Self::get_default_take(),
        ));

        // --- 9. Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic decrease_take
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     - The hotkey we are delegating (must be owned by the coldkey.)
    ///
    /// * 'netuid' (u16):
    ///     - Subnet ID to decrease take for
    /// 
    /// * 'take' (u16):
    ///     - The stake proportion that this hotkey takes from delegations for subnet ID.
    ///
    /// # Event:
    /// * TakeDecreased;
    ///     - On successfully setting a decreased take for this hotkey.
    ///
    /// # Raises:
    /// * 'NotRegistered':
    ///     - The hotkey we are delegating is not registered on the network.
    ///
    /// * 'NonAssociatedColdKey':
    ///     - The hotkey we are delegating is not owned by the calling coldket.
    ///
    /// * 'DelegateTakeTooLow':
    ///     - The delegate is setting a take which is not lower than the previous.
    ///
    pub fn do_decrease_take(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_decrease_take( origin:{:?} hotkey:{:?}, netuid:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            netuid,
            take
        );

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_account_checks(&coldkey, &hotkey)?;

        // --- 3. Ensure we are always strictly decreasing, never increasing take
        if let Ok(current_take) = DelegatesTake::<T>::try_get(&hotkey, netuid) {
            ensure!(take < current_take, Error::<T>::DelegateTakeTooLow);
        }

        // --- 3.1 Ensure take is within the min ..= InitialDefaultTake (18%) range
        let min_take = MinTake::<T>::get();
        let max_take = MaxTake::<T>::get();
        ensure!(take >= min_take, Error::<T>::DelegateTakeTooLow);
        ensure!(take <= max_take, Error::<T>::DelegateTakeTooHigh);

        // --- 4. Set the new take value.
        DelegatesTake::<T>::insert(hotkey.clone(), netuid, take);

        // --- 5. Emit the take value.
        log::info!(
            "TakeDecreased( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        Self::deposit_event(Event::TakeDecreased(coldkey, hotkey, take));

        // --- 6. Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic increase_take
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     - The hotkey we are delegating (must be owned by the coldkey.)
    ///
    /// * 'netuid' (u16):
    ///     - Subnet ID to decrease take for
    ///
    /// * 'take' (u16):
    ///     - The stake proportion that this hotkey takes from delegations for subnet ID.
    ///
    /// # Event:
    /// * TakeIncreased;
    ///     - On successfully setting a increased take for this hotkey.
    ///
    /// # Raises:
    /// * 'NotRegistered':
    ///     - The hotkey we are delegating is not registered on the network.
    ///
    /// * 'NonAssociatedColdKey':
    ///     - The hotkey we are delegating is not owned by the calling coldket.
    ///
    /// * 'TxRateLimitExceeded':
    ///     - Thrown if key has hit transaction rate limit
    ///
    /// * 'DelegateTakeTooLow':
    ///     - The delegate is setting a take which is not greater than the previous.
    ///
    pub fn do_increase_take(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_increase_take( origin:{:?} hotkey:{:?}, netuid:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            netuid,
            take
        );

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_account_checks(&coldkey, &hotkey)?;

        // --- 3. Ensure we are strinctly increasing take
        if let Ok(current_take) = DelegatesTake::<T>::try_get(&hotkey, netuid) {
            ensure!(take > current_take, Error::<T>::DelegateTakeTooLow);
        }

        // --- 4. Ensure take is within the min ..= InitialDefaultTake (18%) range
        let max_take = MaxTake::<T>::get();
        ensure!(take <= max_take, Error::<T>::DelegateTakeTooHigh);

        // --- 5. Enforce the rate limit (independently on do_add_stake rate limits)
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_delegate_take_rate_limit(
                Self::get_last_tx_block_delegate_take(&coldkey),
                block
            ),
            Error::<T>::DelegateTxRateLimitExceeded
        );

        // Set last block for rate limiting
        Self::set_last_tx_block_delegate_take(&coldkey, block);

        // --- 6. Set the new take value.
        DelegatesTake::<T>::insert(hotkey.clone(), netuid, take);

        // --- 7. Emit the take value.
        log::info!(
            "TakeIncreased( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        Self::deposit_event(Event::TakeIncreased(coldkey, hotkey, take));

        // --- 8. Ok and return.
        Ok(())
    }

    // ---- The implementation for the extrinsic add_stake: Adds stake to a hotkey account.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The associated hotkey account.
    //
    // 	* 'stake_to_be_added' (u64):
    // 		- The amount of stake to be added to the hotkey staking account.
    //
    // # Event:
    // 	* StakeAdded;
    // 		- On the successfully adding stake to a global account.
    //
    // # Raises:
    // 	* 'CouldNotConvertToBalance':
    // 		- Unable to convert the passed stake value to a balance.
    //
    // 	* 'NotEnoughBalanceToStake':
    // 		- Not enough balance on the coldkey to add onto the global account.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- The calling coldkey is not associated with this hotkey.
    //
    // 	* 'BalanceWithdrawalError':
    // 		- Errors stemming from transaction pallet.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    pub fn do_add_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        tao_to_be_added: u64,
    ) -> dispatch::DispatchResult {
        // We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_add_stake( origin:{:?} hotkey:{:?}, netuid:{:?}, stake_to_be_added:{:?} )",
            coldkey,
            hotkey,
            netuid,
            tao_to_be_added
        );

        // Ensure that the netuid exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, tao_to_be_added),
            Error::<T>::NotEnoughBalanceToStake
        );

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!(
            Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::HotKeyNotDelegateAndSignerNotOwnHotKey
        );

        // Ensure no type transition is in progress for subnet
        // TODOSDT: Only block for networks in transition (see commented below)
        ensure!(
            SubnetInTransition::<T>::iter().next().is_none(),
            Error::<T>::TemporarilyNotAllowed
        );
        // ensure!(
        //     SubnetInTransition::<T>::get(netuid).is_none(),
        //     Error::<T>::TemporarilyNotAllowed
        // );
        
        // Ensure we don't exceed stake rate limit
        let stakes_this_interval =
            Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey);
        ensure!(
            stakes_this_interval < Self::get_target_stakes_per_interval(),
            Error::<T>::StakeRateLimitExceeded
        );

        // If this is a nomination stake, check if total stake after adding will be above
        // the minimum required stake.

        // If coldkey is not owner of the hotkey, it's a nomination stake.
        if !Self::coldkey_owns_hotkey(&coldkey, &hotkey) {
            let current_stake_alpha = SubStake::<T>::get((&coldkey, &hotkey, netuid));
            let current_stake_tao = Self::estimate_dynamic_unstake(netuid, current_stake_alpha);
            let total_stake_after_add = current_stake_tao.saturating_add(tao_to_be_added);

            ensure!(
                total_stake_after_add >= NominatorMinRequiredStake::<T>::get(),
                Error::<T>::NomStakeBelowMinimumThreshold
            );
        }

        // Ensure the remove operation from the coldkey is a success.
        Self::remove_balance_from_coldkey_account(&coldkey, tao_to_be_added)
            .map_err(|_| Error::<T>::BalanceWithdrawalError)?;

        // Compute Dynamic Stake.
        let dynamic_stake = Self::compute_dynamic_stake(netuid, tao_to_be_added);

        // If we reach here, add the balance to the hotkey.
        Self::increase_subnet_token_on_coldkey_hotkey_account(&coldkey, &hotkey, netuid, dynamic_stake);
        TotalSubnetTAO::<T>::mutate(netuid, |stake| *stake = stake.saturating_add(tao_to_be_added));

        // -- 12. Set last block for rate limiting
        let block: u64 = Self::get_current_block_as_u64();
        Self::set_last_tx_block(&coldkey, block);
        Self::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey,
            &hotkey,
            stakes_this_interval + 1,
            block,
        );
                
        // --- 13. Emit the staking event.
        log::info!(
            "StakeAdded( hotkey:{:?}, netuid:{:?}, stake_to_be_added:{:?} )",
            hotkey,
            netuid,
            tao_to_be_added
        );
        Self::deposit_event(Event::StakeAdded(hotkey, netuid, tao_to_be_added));

        // --- 14. Ok and return.
        Ok(())
    }

    /// The implementation for the extrinsic remove_stake: Removes stake from a hotkey account and adds it onto a coldkey.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     - The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     - The associated hotkey account.
    ///
    /// * 'stake_to_be_added' (u64):
    ///     - The amount of stake to be added to the hotkey staking account.
    ///
    /// # Event:
    /// * StakeRemoved;
    ///     - On the successfully removing stake from the hotkey account.
    ///
    /// # Raises:
    /// * 'NotRegistered':
    ///     - Thrown if the account we are attempting to unstake from is non existent.
    ///
    /// * 'NonAssociatedColdKey':
    ///     - Thrown if the coldkey does not own the hotkey we are unstaking from.
    ///
    /// * 'NotEnoughStakeToWithdraw':
    ///     -  Thrown if there is not enough stake on the hotkey to withdwraw this amount.
    ///
    /// * 'TxRateLimitExceeded':
    ///     - Thrown if key has hit transaction rate limit
    ///
    pub fn do_remove_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        alpha_to_be_removed: u64,
    ) -> dispatch::DispatchResult {
        // We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_remove_stake( origin:{:?} netuid:{:?}, hotkey:{:?}, stake_to_be_removed:{:?} )",
            coldkey,
            hotkey,
            netuid,
            alpha_to_be_removed
        );

        // Ensure that the netuid exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!(
            Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::HotKeyNotDelegateAndSignerNotOwnHotKey
        );

        // Ensure that the stake amount to be removed is above zero.
        ensure!(alpha_to_be_removed > 0, Error::<T>::StakeToWithdrawIsZero);

        // Ensure that the hotkey has enough stake to withdraw.
        ensure!(
            Self::has_enough_stake(&coldkey, &hotkey, netuid, alpha_to_be_removed),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Ensure we don't exceed stake rate limit
        let unstakes_this_interval =
            Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey);
        ensure!(
            unstakes_this_interval < Self::get_target_stakes_per_interval(),
            Error::<T>::UnstakeRateLimitExceeded
        );

        // If this is a nomination stake, check if total stake after removing will be above
        // the minimum required stake.

        // If coldkey is not owner of the hotkey, it's a nomination stake.
        let block: u64 = Self::get_current_block_as_u64();
        if !Self::coldkey_owns_hotkey(&coldkey, &hotkey) {
            let current_stake_alpha = SubStake::<T>::get((&coldkey, &hotkey, netuid));
            let alpha_after_remove = current_stake_alpha.saturating_sub(alpha_to_be_removed);
            let total_stake_after_remove = Self::estimate_dynamic_unstake(netuid, alpha_after_remove);

            ensure!(
                total_stake_after_remove == 0 || total_stake_after_remove >= NominatorMinRequiredStake::<T>::get(),
                Error::<T>::NomStakeBelowMinimumThreshold
            );
        } else {
            // If coldkey is owner of the hotkey, then ensure that subnet lock period has expired
            let subnet_lock_period: u64 = Self::get_subnet_owner_lock_period();
            if Self::get_subnet_creator_hotkey(netuid) == hotkey {
                ensure!(
                    block - Self::get_network_registered_block(netuid) >= subnet_lock_period,
                    Error::<T>::SubnetCreatorLock
                )
            }
        }

        // Remove stake from state maps
        Self::do_remove_stake_no_checks(
            &coldkey,
            &hotkey,
            netuid,
            alpha_to_be_removed,
        );

        // Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);
        Self::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey,
            &hotkey,
            unstakes_this_interval + 1,
            block,
        );

        // Emit the unstaking event.
        log::info!(
            "StakeRemoved( hotkey:{:?}, stake_to_be_removed:{:?} )",
            hotkey,
            alpha_to_be_removed
        );
        Self::deposit_event(Event::StakeRemoved(hotkey, netuid, alpha_to_be_removed));

        // --- 11. Done and ok.
        Ok(())
    }

    /// Removes the stake assuming all checks have passed
    /// 
    pub fn do_remove_stake_no_checks(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
        alpha_to_be_removed: u64,
    ) {
        // We remove the balance from the hotkey.
        Self::decrease_subnet_token_on_coldkey_hotkey_account(
            coldkey,
            hotkey,
            netuid,
            alpha_to_be_removed,
        );

        // Compute Dynamic unstake.
        let tao_unstaked: u64 = Self::compute_dynamic_unstake(netuid, alpha_to_be_removed);
        TotalSubnetTAO::<T>::mutate(netuid, |stake| *stake = stake.saturating_sub(tao_unstaked));

        // We add the balance to the coldkey. If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(coldkey, tao_unstaked);
    }

    /// Computes the dynamic unstake amount based on the current reserves and the stake to be removed.
    /// This function is used to dynamically adjust the reserves of a subnet when unstaking occurs,
    /// ensuring that the reserve ratios are maintained according to the bonding curve defined by `k`.
    ///
    /// # Arguments
    /// * `netuid` - The unique identifier for the network (subnet) from which the stake is being removed.
    /// * `stake_to_be_removed` - The amount of stake (in terms of alpha tokens) to be removed from the subnet.
    ///
    /// # Returns
    /// * `u64` - The amount of tao tokens that will be released as a result of the unstake operation.
    ///
    /// # Details
    /// The function first checks if the subnet identified by `netuid` supports dynamic staking. If not,
    /// it simply returns the `stake_to_be_removed` as the amount of tao to be released, since no dynamic calculations are needed.
    ///
    /// For dynamic subnets, the function retrieves the current tao and alpha reserves (`tao_reserve` and `dynamic_reserve`),
    /// along with the bonding curve constant `k`. It then calculates the new alpha reserve by adding the `stake_to_be_removed`
    /// to the current alpha reserve. Using the bonding curve equation `tao_reserve = k / alpha_reserve`, it computes the new
    /// tao reserve.
    ///
    /// The difference between the old and new tao reserves gives the amount of tao that will be released. This is calculated
    /// by subtracting the new tao reserve from the old tao reserve. The function then updates the subnet's reserves in storage
    /// and decrements the outstanding alpha by the amount of stake removed.
    ///
    /// # Panics
    /// The function will panic if the new dynamic reserve calculation overflows, although this is highly unlikely due to the
    /// use of saturating arithmetic operations.
    pub fn compute_dynamic_unstake(netuid: u16, stake_to_be_removed: u64) -> u64 {
        let subnet_type = Self::get_subnet_type(netuid);

        // STAO networks do not have dynamic stake
        match subnet_type {
            SubnetType::DTAO => {
                let tao_reserve = DynamicTAOReserve::<T>::get(netuid);
                let dynamic_reserve = DynamicAlphaReserve::<T>::get(netuid);
                let k = DynamicK::<T>::get(netuid);
        
                // Calculate the new dynamic reserve after adding the stake to be removed
                let new_dynamic_reserve = dynamic_reserve.saturating_add(stake_to_be_removed);
                // Calculate the new tao reserve based on the new dynamic reserve
                let new_tao_reserve: u64 = (k / (new_dynamic_reserve as u128)) as u64;
                // Calculate the amount of tao to be pulled out based on the difference in tao reserves
                let tao = tao_reserve.saturating_sub(new_tao_reserve);
        
                // Update the reserves with the new values
                DynamicTAOReserve::<T>::insert(netuid, new_tao_reserve);
                DynamicAlphaReserve::<T>::insert(netuid, new_dynamic_reserve);
                DynamicAlphaOutstanding::<T>::mutate(netuid, |outstanding| {
                    *outstanding -= stake_to_be_removed
                }); // Decrement outstanding alpha.
        
                tao
            }
            SubnetType::STAO => stake_to_be_removed
        }
    }

    /// Returns the amount of TAO returned if stake_to_be_removed is unstaked
    /// Doesn't make any state changes
    /// 
    pub fn estimate_dynamic_unstake(netuid: u16, stake_to_be_removed: u64) -> u64 {
        let subnet_type = Self::get_subnet_type(netuid);

        // STAO networks do not have dynamic stake
        match subnet_type {
            SubnetType::DTAO => {
                let tao_reserve = DynamicTAOReserve::<T>::get(netuid);
                let dynamic_reserve = DynamicAlphaReserve::<T>::get(netuid);
                let k = DynamicK::<T>::get(netuid);
        
                // Calculate the new dynamic reserve after adding the stake to be removed
                let new_dynamic_reserve = dynamic_reserve.saturating_add(stake_to_be_removed);
                // Calculate the new tao reserve based on the new dynamic reserve
                let new_tao_reserve: u64 = (k / (new_dynamic_reserve as u128)) as u64;
                // Calculate the amount of tao to be pulled out based on the difference in tao reserves
                tao_reserve.saturating_sub(new_tao_reserve)
            }
            SubnetType::STAO => stake_to_be_removed
        }
    }

    /// Computes the dynamic stake amount based on the current reserves and the stake to be added.
    /// This function is used to dynamically adjust the reserves of a subnet when staking occurs,
    /// ensuring that the reserve ratios are maintained according to the bonding curve defined by `k`.
    ///
    /// # Arguments
    /// * `netuid` - The unique identifier for the network (subnet) where the stake is being added.
    /// * `stake_to_be_added` - The amount of stake (in terms of alpha tokens) to be added to the subnet.
    ///
    /// # Returns
    /// * `u64` - The amount of dynamic token that will be pulled out as a result of the stake operation.
    ///
    /// # Details
    /// The function first checks if the subnet identified by `netuid` supports dynamic staking. If not,
    /// it simply returns the `stake_to_be_added` as the amount of dynamic token to be pulled out, since no dynamic calculations are needed.
    ///
    /// For dynamic subnets, the function retrieves the current tao and alpha reserves (`tao_reserve` and `dynamic_reserve`),
    /// along with the bonding curve constant `k`. It then calculates the new tao reserve by adding the `stake_to_be_added`
    /// to the current tao reserve. Using the bonding curve equation `dynamic_reserve = k / tao_reserve`, it computes the new
    /// dynamic reserve.
    ///
    /// The difference between the old and new dynamic reserves gives the amount of dynamic token that will be pulled out. This is calculated
    /// by subtracting the new dynamic reserve from the old dynamic reserve. The function then updates the subnet's reserves in storage
    /// and increments the outstanding alpha by the amount of stake added.
    ///
    /// # Panics
    /// The function will panic if the new tao reserve calculation overflows, although this is highly unlikely due to the
    /// use of saturating arithmetic operations.
    pub fn compute_dynamic_stake(netuid: u16, stake_to_be_added: u64) -> u64 {
        let subnet_type = Self::get_subnet_type(netuid);

        // STAO networks do not have dynamic stake
        match subnet_type {
            SubnetType::DTAO => {
                let tao_reserve = DynamicTAOReserve::<T>::get(netuid);
                let dynamic_reserve = DynamicAlphaReserve::<T>::get(netuid);
                let k = DynamicK::<T>::get(netuid);
        
                // Calculate the new tao reserve after adding the stake
                let new_tao_reserve = tao_reserve.saturating_add(stake_to_be_added);
                // Calculate the new dynamic reserve based on the new tao reserve
                let new_dynamic_reserve: u64 = (k / (new_tao_reserve as u128)) as u64;
                // Calculate the amount of dynamic token to be pulled out based on the difference in dynamic reserves
                let dynamic_token = dynamic_reserve.saturating_sub(new_dynamic_reserve);
        
                // Update the reserves with the new values
                DynamicTAOReserve::<T>::insert(netuid, new_tao_reserve);
                DynamicAlphaReserve::<T>::insert(netuid, new_dynamic_reserve);
                DynamicAlphaOutstanding::<T>::mutate(netuid, |outstanding| *outstanding += dynamic_token); // Increment outstanding alpha.
        
                dynamic_token
            }
            SubnetType::STAO => stake_to_be_added
        }
    }

    // Returns true if the passed hotkey allow delegative staking.
    //
    pub fn hotkey_is_delegate(hotkey: &T::AccountId) -> bool {
        Delegates::<T>::contains_key(hotkey)
    }

    // Sets the hotkey as a delegate with take.
    //
    pub fn delegate_hotkey(hotkey: &T::AccountId, take: u16) {
        Delegates::<T>::insert(hotkey, take);
    }

    // Getters for Dynamic terms
    //
    pub fn get_total_stake_on_subnet(netuid: u16) -> u64 {
        TotalSubnetTAO::<T>::get(netuid)
    }
    pub fn get_tao_reserve(netuid: u16) -> u64 {
        DynamicTAOReserve::<T>::get(netuid)
    }
    pub fn set_tao_reserve(netuid: u16, amount: u64) {
        DynamicTAOReserve::<T>::insert(netuid, amount);
    }
    pub fn get_alpha_reserve(netuid: u16) -> u64 {
        DynamicAlphaReserve::<T>::get(netuid)
    }
    pub fn set_alpha_reserve(netuid: u16, amount: u64) {
        DynamicAlphaReserve::<T>::insert(netuid, amount);
    }
    pub fn get_alpha_outstanding(netuid: u16) -> u64 {
        DynamicAlphaOutstanding::<T>::get(netuid)
    }
    pub fn set_alpha_outstanding(netuid: u16, amount: u64) {
        DynamicAlphaOutstanding::<T>::insert(netuid, amount);
    }
    pub fn get_pool_k(netuid: u16) -> u128 {
        DynamicK::<T>::get(netuid)
    }
    pub fn get_alpha_issuance(netuid: u16) -> u64 {
        DynamicAlphaIssuance::<T>::get(netuid)
    }
    pub fn set_pool_k(netuid: u16, k: u128) {
        DynamicK::<T>::insert(netuid, k);
    }
    pub fn is_subnet_dynamic(netuid: u16) -> bool {
        IsDynamic::<T>::get(netuid)
    }
    pub fn set_subnet_dynamic(netuid: u16) {
        IsDynamic::<T>::insert(netuid, true)
    }

    // Returns the total amount of stake under a subnet (delegative or otherwise)
    pub fn get_total_stake_for_subnet(target_subnet: u16) -> u64 {
        SubStake::<T>::iter()
            .filter(|((_, _, subnet), _)| *subnet == target_subnet)
            .fold(0, |acc, (_, stake)| acc.saturating_add(stake))
    }

    // Returns the total amount of stake under a hotkey for a subnet (delegative or otherwise)
    //
    pub fn get_total_stake_for_hotkey_and_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {
        TotalHotkeySubStake::<T>::get(hotkey, netuid)
    }

    // Retrieves the total stakes for a given hotkey (account ID) for the current staking interval.
    pub fn get_stakes_this_interval_for_coldkey_hotkey(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
    ) -> u64 {
        // Retrieve the configured stake interval duration from storage.
        let stake_interval = StakeInterval::<T>::get();

        // Obtain the current block number as an unsigned 64-bit integer.
        let current_block = Self::get_current_block_as_u64();

        // Fetch the total stakes and the last block number when stakes were made for the hotkey.
        let (stakes, block_last_staked_at) =
            TotalHotkeyColdkeyStakesThisInterval::<T>::get(coldkey, hotkey);

        // Calculate the block number after which the stakes for the hotkey should be reset.
        let block_to_reset_after = block_last_staked_at + stake_interval;

        // If the current block number is beyond the reset point,
        // it indicates the end of the staking interval for the hotkey.
        if block_to_reset_after <= current_block {
            // Reset the stakes for this hotkey for the current interval.
            Self::set_stakes_this_interval_for_coldkey_hotkey(
                coldkey,
                hotkey,
                0,
                block_last_staked_at,
            );
            // Return 0 as the stake amount since we've just reset the stakes.
            return 0;
        }

        // If the staking interval has not yet ended, return the current stake amount.
        stakes
    }

    pub fn get_target_stakes_per_interval() -> u64 {
        TargetStakesPerInterval::<T>::get()
    }

    // Creates a cold - hot pairing account if the hotkey is not already an active account.
    //
    pub fn create_account_if_non_existent(coldkey: &T::AccountId, hotkey: &T::AccountId) {
        if !Self::hotkey_account_exists(hotkey) {
            Owner::<T>::insert(hotkey, coldkey);
        }
    }

    // Returns the coldkey owning this hotkey. This function should only be called for active accounts.
    //
    pub fn get_owning_coldkey_for_hotkey(hotkey: &T::AccountId) -> T::AccountId {
        Owner::<T>::get(hotkey)
    }

    // Returns the hotkey take
    //
    pub fn get_delegate_take(hotkey: &T::AccountId, netuid: u16) -> u16 {
        DelegatesTake::<T>::get(hotkey, netuid)
    }

    pub fn do_set_delegate_takes(
        origin: T::RuntimeOrigin,
        hotkey: &T::AccountId,
        takes: Vec<(u16, u16)>,
    ) -> dispatch::DispatchResult {
        let coldkey = ensure_signed(origin)?;
        log::trace!(
            "do_increase_take( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            takes
        );

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_account_checks(&coldkey, hotkey)?;
        let block: u64 = Self::get_current_block_as_u64();

        for (netuid, take) in takes {
            // Check if the subnet exists before setting the take.
            ensure!(
                Self::if_subnet_exist(netuid),
                Error::<T>::SubNetworkDoesNotExist
            );

            // Ensure the take does not exceed the initial default take.
            let max_take = T::InitialDefaultTake::get();
            ensure!(take <= max_take, Error::<T>::DelegateTakeTooHigh);

            // Enforce the rate limit (independently on do_add_stake rate limits)
            ensure!(
                !Self::exceeds_tx_delegate_take_rate_limit(
                    Self::get_last_tx_block_delegate_take(hotkey),
                    block
                ),
                Error::<T>::DelegateTxRateLimitExceeded
            );

            // Insert the take into the storage.
            DelegatesTake::<T>::insert(hotkey, netuid, take);
        }

        // Set last block for rate limiting after all takes are set
        Self::set_last_tx_block_delegate_take(hotkey, block);

        Ok(())
    }

    // Returns true if the hotkey account has been created.
    //
    pub fn hotkey_account_exists(hotkey: &T::AccountId) -> bool {
        Owner::<T>::contains_key(hotkey)
    }

    // Return true if the passed coldkey owns the hotkey.
    //
    pub fn coldkey_owns_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> bool {
        if Self::hotkey_account_exists(hotkey) {
            Owner::<T>::get(hotkey) == *coldkey
        } else {
            false
        }
    }

    // Returns true if the cold-hot staking account has enough balance to fufil the decrement.
    //
    pub fn has_enough_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
        decrement: u64,
    ) -> bool {
        Self::get_subnet_stake_for_coldkey_and_hotkey(coldkey, hotkey, netuid) >= decrement
    }
    // Increases the stake on the hotkey account under its owning coldkey.
    //
    pub fn increase_subnet_token_on_hotkey_account(hotkey: &T::AccountId, netuid: u16, increment: u64) {
        Self::increase_subnet_token_on_coldkey_hotkey_account(
            &Self::get_owning_coldkey_for_hotkey(hotkey),
            hotkey,
            netuid,
            increment,
        );
    }

    // Decreases the stake on the hotkey account under its owning coldkey.
    //
    pub fn decrease_subnet_token_on_hotkey_account(hotkey: &T::AccountId, netuid: u16, decrement: u64) {
        Self::decrease_subnet_token_on_coldkey_hotkey_account(
            &Self::get_owning_coldkey_for_hotkey(hotkey),
            hotkey,
            netuid,
            decrement,
        );
    }

    // Returns the subent stake under the cold - hot pairing in the staking table.
    //
    pub fn get_subnet_stake_for_coldkey_and_hotkey(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
    ) -> u64 {
        SubStake::<T>::try_get((coldkey, hotkey, netuid)).unwrap_or(0)
    }

    pub fn get_tao_per_alpha_price(netuid: u16) -> I64F64 {
        let tao_reserve: u64 = DynamicTAOReserve::<T>::get(netuid);
        let alpha_reserve: u64 = DynamicAlphaReserve::<T>::get(netuid);
        if alpha_reserve == 0 {
            I64F64::from_num(1.0)
        } else {
            I64F64::from_num(tao_reserve) / I64F64::from_num(alpha_reserve)
        }
    }

    /// Returns the stake under the cold - hot pairing in the staking table.
    ///
    /// TODO: We could probably store this total as a state variable
    pub fn get_hotkey_global_dynamic_tao(hotkey: &T::AccountId) -> u64 {
        let mut global_dynamic_tao: I64F64 = I64F64::from_num(0.0);
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        for netuid in netuids.iter() {
            if IsDynamic::<T>::get(*netuid) {
                // Computes the proportion of TAO owned by this netuid.
                let other_subnet_token: I64F64 =
                    I64F64::from_num(Self::get_total_stake_for_hotkey_and_subnet(hotkey, *netuid));
                let other_dynamic_outstanding: I64F64 =
                    I64F64::from_num(DynamicAlphaOutstanding::<T>::get(*netuid));
                let other_tao_reserve: I64F64 =
                    I64F64::from_num(DynamicTAOReserve::<T>::get(*netuid));
                let my_proportion: I64F64 = if other_dynamic_outstanding != 0 {
                    other_subnet_token / other_dynamic_outstanding
                } else {
                    I64F64::from_num(1.0)
                };
                global_dynamic_tao += my_proportion * other_tao_reserve;
            } else {
                // Computes the amount of TAO owned in the non dynamic subnet.
                let other_subnet_token_tao: u64 =
                    Self::get_total_stake_for_hotkey_and_subnet(hotkey, *netuid);
                global_dynamic_tao += I64F64::from_num(other_subnet_token_tao);
            }
        }
        global_dynamic_tao.to_num::<u64>()
    }

    /// Returns the stake under the cold - hot pairing in the staking table.
    ///
    pub fn get_nominator_global_dynamic_tao(coldkey: &T::AccountId, hotkey: &T::AccountId) -> u64 {
        let mut global_dynamic_tao: I64F64 = I64F64::from_num(0.0);
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        for netuid in netuids.iter() {
            if IsDynamic::<T>::get(*netuid) {
                // Computes the proportion of TAO owned by this netuid.
                let other_subnet_token: I64F64 = I64F64::from_num(
                    Self::get_subnet_stake_for_coldkey_and_hotkey(coldkey, hotkey, *netuid),
                );
                let other_dynamic_outstanding: I64F64 =
                    I64F64::from_num(DynamicAlphaOutstanding::<T>::get(*netuid));
                let other_tao_reserve: I64F64 =
                    I64F64::from_num(DynamicTAOReserve::<T>::get(*netuid));
                let my_proportion: I64F64 = if other_dynamic_outstanding != 0 {
                    other_subnet_token / other_dynamic_outstanding
                } else {
                    I64F64::from_num(1.0)
                };
                global_dynamic_tao += my_proportion * other_tao_reserve;
            } else {
                // Computes the amount of TAO owned in the non dynamic subnet.
                let other_subnet_token_tao: u64 =
                    Self::get_subnet_stake_for_coldkey_and_hotkey(coldkey, hotkey, *netuid);
                global_dynamic_tao += I64F64::from_num(other_subnet_token_tao);
            }
        }
        global_dynamic_tao.to_num::<u64>()
    }

    /// Increases the stake on the cold - hot pairing by increment while also incrementing other counters.
    /// This function should be called rather than set_stake under account.
    ///
    pub fn increase_subnet_token_on_coldkey_hotkey_account(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
        increment_alpha: u64,
    ) {
        if increment_alpha == 0 {
            return;
        }
        TotalHotkeySubStake::<T>::mutate(hotkey, netuid, |stake| {
            *stake = stake.saturating_add(increment_alpha);
        });
        SubStake::<T>::mutate((coldkey, hotkey, netuid), |stake| {
            *stake = stake.saturating_add(increment_alpha)
        });
        Staker::<T>::insert(hotkey, coldkey, true);
    }

    /// Decreases the stake on the cold - hot pairing by the decrement while decreasing other counters.
    ///
    pub fn decrease_subnet_token_on_coldkey_hotkey_account(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
        decrement_alpha: u64,
    ) {
        if decrement_alpha == 0 {
            return;
        }
        let existing_total_stake = TotalHotkeySubStake::<T>::get(&hotkey, netuid);
        if existing_total_stake == decrement_alpha {
            TotalHotkeySubStake::<T>::remove(hotkey, netuid);
        } else {
            TotalHotkeySubStake::<T>::insert(
                hotkey, 
                netuid, 
                existing_total_stake.saturating_sub(decrement_alpha)
            );
        }

        // Delete substake map entry if all stake is removed
        let existing_substake = SubStake::<T>::get((coldkey, hotkey, netuid));
        if existing_substake == decrement_alpha {
            SubStake::<T>::remove((coldkey, hotkey, netuid));
        } else {
            SubStake::<T>::insert(
                (coldkey, hotkey, netuid),
                existing_substake.saturating_sub(decrement_alpha),
            );
        }

        // Delete staker map entry if all stake is removed
        if SubStake::<T>::iter_prefix((&coldkey, &hotkey)).next().is_none() {
            Staker::<T>::remove(hotkey, coldkey);
        }
    }

    /// Empties the stake associated with a given coldkey-hotkey account pairing.
    /// This function retrieves the current stake for the specified coldkey-hotkey pairing.
    /// It also removes the stake entry for the hotkey-coldkey pairing and adjusts the TotalStake
    /// and TotalIssuance by subtracting the removed stake amount.
    ///
    /// Returns the amount of stake that was removed.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - A reference to the AccountId of the coldkey involved in the staking.
    /// * `hotkey` - A reference to the AccountId of the hotkey associated with the coldkey.
    pub fn empty_stake_on_coldkey_hotkey_account(coldkey: &T::AccountId, hotkey: &T::AccountId, netuid: u16) -> u64 {
        let unstaked_tao = {
            let stake = SubStake::<T>::get((&coldkey, &hotkey, netuid));
            // Determine the type of network. 
            // For STAO stake is TAO, for DTAO stake is alpha and needs to be unstaked
            match Self::get_subnet_type(netuid) {
                SubnetType::DTAO => {
                    Self::compute_dynamic_unstake(netuid, stake)
                },
                SubnetType::STAO => {
                    stake
                }
            }
        };
        
        // Clear SubStake entry
        SubStake::<T>::remove((coldkey, hotkey, netuid));

        // Clear Staker entry
        if SubStake::<T>::iter_prefix((&coldkey, &hotkey)).next().is_none() {
            Staker::<T>::remove(hotkey, coldkey);
        }

        // Reduce Total Issuance by total unstaked TAO
        TotalIssuance::<T>::mutate(|issuance| *issuance = issuance.saturating_sub(unstaked_tao));

        unstaked_tao
    }

    /// Clears the nomination for an account, if it is a nominator account and the stake is below the minimum required threshold.
    pub fn clear_small_nomination_if_required(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        stake: u64,
    ) {
        // Verify if the account is a nominator account by checking ownership of the hotkey by the coldkey.
        if !Self::coldkey_owns_hotkey(coldkey, hotkey) {
            // If the stake is below the minimum required, it's considered a small nomination and needs to be cleared.
            if stake < Self::get_nominator_min_required_stake() {
                // Remove the stake from the nominator account. (this is a more forceful unstake operation which )
                // Actually deletes the staking account.
                let cleared_stake = Self::empty_stake_on_coldkey_hotkey_account(coldkey, hotkey, netuid);
                // Add the stake to the coldkey account.
                Self::add_balance_to_coldkey_account(coldkey, cleared_stake);
            }
        }
    }

    /// Clears small nominations for all accounts.
    ///
    /// WARN: This is an O(N) operation, where N is the number of staking accounts. It should be
    /// used with caution.
    pub fn clear_small_nominations() {
        // Loop through all staking accounts to identify and clear nominations below the minimum stake.
        for ((coldkey, hotkey, netuid), stake) in SubStake::<T>::iter() {
            Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid, stake);
        }
    }

    pub fn add_balance_to_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) {
        // infallible
        let _ = T::Currency::deposit(coldkey, amount, Precision::BestEffort);
    }

    pub fn set_balance_on_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) {
        T::Currency::set_balance(coldkey, amount);
    }

    pub fn can_remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> bool {
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }

        // This bit is currently untested. @todo
        T::Currency::can_withdraw(
            coldkey,
            amount,
        )
        .into_result(false)
        .is_ok()
    }

    pub fn get_coldkey_balance(
        coldkey: &T::AccountId,
    ) -> <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance {
        T::Currency::reducible_balance(coldkey, Preservation::Expendable, Fortitude::Polite)
    }

    #[must_use = "Balance must be used to preserve total issuance of token"]
    pub fn remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> Result<u64, DispatchError> {
        if amount == 0 {
            return Ok(0);
        }

        let credit = T::Currency::withdraw(
                coldkey,
                amount,
                Precision::BestEffort,
                Preservation::Preserve,
                Fortitude::Polite,
            )
            .map_err(|_| Error::<T>::BalanceWithdrawalError)?
            .peek();

        if credit == 0 {
            return Err(Error::<T>::ZeroBalanceAfterWithdrawn.into());
        }

        Ok(credit)
    }

    pub fn unstake_all_coldkeys_from_hotkey_account(hotkey: &T::AccountId) {
        // Iterate through all coldkeys that have a stake on this hotkey account.
        let all_netuids: Vec<u16> = Self::get_all_subnet_netuids();
        for (coldkey_i, _) in
            Staker::<T>::iter_prefix(
                hotkey,
            )
        {
            for &netuid_i in all_netuids.iter() {
                // Get the subnet type
                let subnet_type = Self::get_subnet_type(netuid_i);

                // Get the stake on this uid.
                let stake_alpha_i =
                    Self::get_subnet_stake_for_coldkey_and_hotkey(&coldkey_i, hotkey, netuid_i);

                let stake_tao_i = match subnet_type {
                    SubnetType::DTAO => {
                        Self::compute_dynamic_unstake(netuid_i, stake_alpha_i)
                    },
                    SubnetType::STAO => {
                        stake_alpha_i
                    },
                };

                // Remove the stake from the coldkey - hotkey pairing.
                Self::decrease_subnet_token_on_coldkey_hotkey_account(
                    &coldkey_i, hotkey, netuid_i, stake_alpha_i
                );

                // Add the balance to the coldkey account.
                Self::add_balance_to_coldkey_account(&coldkey_i, stake_tao_i);
            }
        }
    }
}
