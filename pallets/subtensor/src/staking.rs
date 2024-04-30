use super::*;
use frame_support::storage::IterableStorageDoubleMap;

impl<T: Config> Pallet<T> {
    // ---- The implementation for the extrinsic become_delegate: signals that this hotkey allows delegated stake.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The hotkey we are delegating (must be owned by the coldkey.)
    //
    // 	* 'take' (u16):
    // 		- The stake proportion that this hotkey takes from delegations.
    //
    // # Event:
    // 	* DelegateAdded;
    // 		- On successfully setting a hotkey as a delegate.
    //
    // # Raises:
    // 	* 'NotRegistered':
    // 		- The hotkey we are delegating is not registered on the network.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- The hotkey we are delegating is not owned by the calling coldket.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    pub fn do_become_delegate(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signuture.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_become_delegate( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );

        // --- 2. Ensure we are delegating an known key.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::NotRegistered
        );

        // --- 3. Ensure that the coldkey is the owner.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 4. Ensure we are not already a delegate (dont allow changing delegate take.)
        ensure!(
            !Self::hotkey_is_delegate(&hotkey),
            Error::<T>::AlreadyDelegate
        );

        // --- 5. Ensure we don't exceed tx rate limit
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::TxRateLimitExceeded
        );

        // --- 6. Delegate the key.
        Self::delegate_hotkey(&hotkey, take);

        // Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);

        // --- 7. Emit the staking event.
        log::info!(
            "DelegateAdded( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        Self::deposit_event(Event::DelegateAdded(coldkey, hotkey, take));

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
        stake_to_be_added: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_add_stake( origin:{:?} hotkey:{:?}, stake_to_be_added:{:?} )",
            coldkey,
            hotkey,
            stake_to_be_added
        );

        // --- 2. We convert the stake u64 into a balancer.
        let stake_as_balance = Self::u64_to_balance(stake_to_be_added);
        ensure!(
            stake_as_balance.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );

        // --- 3. Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()),
            Error::<T>::NotEnoughBalanceToStake
        );

        // --- 4. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::NotRegistered
        );

        // --- 5. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!(
            Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 6. Ensure we don't exceed stake rate limit
        let stakes_this_interval =
            Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey);
        ensure!(
            stakes_this_interval < Self::get_target_stakes_per_interval(),
            Error::<T>::StakeRateLimitExceeded
        );

        // --- 7. If this is a nomination stake, check if total stake after adding will be above
        // the minimum required stake.

        // If coldkey is not owner of the hotkey, it's a nomination stake.
        if !Self::coldkey_owns_hotkey(&coldkey, &hotkey) {
            let total_stake_after_add =
                Stake::<T>::get(&hotkey, &coldkey).saturating_add(stake_to_be_added);

            ensure!(
                total_stake_after_add >= NominatorMinRequiredStake::<T>::get(),
                Error::<T>::NomStakeBelowMinimumThreshold
            );
        }

        // --- 8. Ensure the remove operation from the coldkey is a success.
        ensure!(
            Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true,
            Error::<T>::BalanceWithdrawalError
        );

        // --- 9. If we reach here, add the balance to the hotkey.
        Self::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_to_be_added);

        // Set last block for rate limiting
        let block: u64 = Self::get_current_block_as_u64();
        Self::set_last_tx_block(&coldkey, block);

        // --- 9. Emit the staking event.
        Self::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey,
            &hotkey,
            stakes_this_interval + 1,
            block,
        );
        log::info!(
            "StakeAdded( hotkey:{:?}, stake_to_be_added:{:?} )",
            hotkey,
            stake_to_be_added
        );
        Self::deposit_event(Event::StakeAdded(hotkey, stake_to_be_added));

        // --- 10. Ok and return.
        Ok(())
    }

    // ---- The implementation for the extrinsic remove_stake: Removes stake from a hotkey account and adds it onto a coldkey.
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
    // 	* StakeRemoved;
    // 		- On the successfully removing stake from the hotkey account.
    //
    // # Raises:
    // 	* 'NotRegistered':
    // 		- Thrown if the account we are attempting to unstake from is non existent.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- Thrown if the coldkey does not own the hotkey we are unstaking from.
    //
    // 	* 'NotEnoughStaketoWithdraw':
    // 		- Thrown if there is not enough stake on the hotkey to withdwraw this amount.
    //
    // 	* 'CouldNotConvertToBalance':
    // 		- Thrown if we could not convert this amount to a balance.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    //
    pub fn do_remove_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        stake_to_be_removed: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;

        // If this action would reduce a nomination stake below the minimum stake, then unstake all
        // funds for the account.
        let stake_to_be_removed = {
            let attempted_stake_after_remove =
                Stake::<T>::get(&hotkey, &coldkey).saturating_sub(stake_to_be_removed);

            if !Self::coldkey_owns_hotkey(&coldkey, &hotkey) // check if nomination stake
                && attempted_stake_after_remove > 0
                && attempted_stake_after_remove < NominatorMinRequiredStake::<T>::get()
            {
                log::info!(
                    "User provided 'stake_to_be_removed' ({}) would reduce stake below minimum threshold, unstaking entire user stake instead.",
                    stake_to_be_removed
                );
                Stake::<T>::get(&hotkey, &coldkey)
            } else {
                stake_to_be_removed
            }
        };

        log::info!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, stake_to_be_removed:{:?} )",
            coldkey,
            hotkey,
            stake_to_be_removed
        );

        // --- 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::NotRegistered
        );

        // --- 3. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!(
            Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- Ensure that the stake amount to be removed is above zero.
        ensure!(
            stake_to_be_removed > 0,
            Error::<T>::NotEnoughStaketoWithdraw
        );

        // --- 4. Ensure that the hotkey has enough stake to withdraw.
        ensure!(
            Self::has_enough_stake(&coldkey, &hotkey, stake_to_be_removed),
            Error::<T>::NotEnoughStaketoWithdraw
        );

        // --- 5. Ensure that we can conver this u64 to a balance.
        let stake_to_be_added_as_currency = Self::u64_to_balance(stake_to_be_removed);
        ensure!(
            stake_to_be_added_as_currency.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );

        // --- 6. Ensure we don't exceed stake rate limit
        let unstakes_this_interval =
            Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey);
        ensure!(
            unstakes_this_interval < Self::get_target_stakes_per_interval(),
            Error::<T>::UnstakeRateLimitExceeded
        );

        // --- 7. We remove the balance from the hotkey.
        Self::decrease_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_to_be_removed);

        // --- 8. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_added_as_currency.unwrap());

        // Set last block for rate limiting
        let block: u64 = Self::get_current_block_as_u64();
        Self::set_last_tx_block(&coldkey, block);

        // --- 9. Emit the unstaking event.
        Self::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey,
            &hotkey,
            unstakes_this_interval + 1,
            block,
        );
        log::info!(
            "StakeRemoved( hotkey:{:?}, stake_to_be_removed:{:?} )",
            hotkey,
            stake_to_be_removed
        );
        Self::deposit_event(Event::StakeRemoved(hotkey, stake_to_be_removed));

        // --- 10. Done and ok.
        Ok(())
    }

    // Returns true if the passed hotkey allow delegative staking.
    //
    pub fn hotkey_is_delegate(hotkey: &T::AccountId) -> bool {
        return Delegates::<T>::contains_key(hotkey);
    }

    // Sets the hotkey as a delegate with take.
    //
    pub fn delegate_hotkey(hotkey: &T::AccountId, take: u16) {
        Delegates::<T>::insert(hotkey, take);
    }

    // Returns the total amount of stake in the staking table.
    //
    pub fn get_total_stake() -> u64 {
        return TotalStake::<T>::get();
    }

    // Increases the total amount of stake by the passed amount.
    //
    pub fn increase_total_stake(increment: u64) {
        TotalStake::<T>::put(Self::get_total_stake().saturating_add(increment));
    }

    // Decreases the total amount of stake by the passed amount.
    //
    pub fn decrease_total_stake(decrement: u64) {
        TotalStake::<T>::put(Self::get_total_stake().saturating_sub(decrement));
    }

    // Returns the total amount of stake under a hotkey (delegative or otherwise)
    //
    pub fn get_total_stake_for_hotkey(hotkey: &T::AccountId) -> u64 {
        return TotalHotkeyStake::<T>::get(hotkey);
    }

    // Returns the total amount of stake held by the coldkey (delegative or otherwise)
    //
    pub fn get_total_stake_for_coldkey(coldkey: &T::AccountId) -> u64 {
        return TotalColdkeyStake::<T>::get(coldkey);
    }

    // Returns the stake under the cold - hot pairing in the staking table.
    //
    pub fn get_stake_for_coldkey_and_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> u64 {
        return Stake::<T>::get(hotkey, coldkey);
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
        return TargetStakesPerInterval::<T>::get();
    }

    // Creates a cold - hot pairing account if the hotkey is not already an active account.
    //
    pub fn create_account_if_non_existent(coldkey: &T::AccountId, hotkey: &T::AccountId) {
        if !Self::hotkey_account_exists(hotkey) {
            Stake::<T>::insert(hotkey, coldkey, 0);
            Owner::<T>::insert(hotkey, coldkey);
        }
    }

    // Returns the coldkey owning this hotkey. This function should only be called for active accounts.
    //
    pub fn get_owning_coldkey_for_hotkey(hotkey: &T::AccountId) -> T::AccountId {
        return Owner::<T>::get(hotkey);
    }

    // Returns true if the hotkey account has been created.
    //
    pub fn hotkey_account_exists(hotkey: &T::AccountId) -> bool {
        return Owner::<T>::contains_key(hotkey);
    }

    // Return true if the passed coldkey owns the hotkey.
    //
    pub fn coldkey_owns_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> bool {
        if Self::hotkey_account_exists(hotkey) {
            return Owner::<T>::get(hotkey) == *coldkey;
        } else {
            return false;
        }
    }

    // Returns true if the cold-hot staking account has enough balance to fufil the decrement.
    //
    pub fn has_enough_stake(coldkey: &T::AccountId, hotkey: &T::AccountId, decrement: u64) -> bool {
        return Self::get_stake_for_coldkey_and_hotkey(coldkey, hotkey) >= decrement;
    }

    // Increases the stake on the hotkey account under its owning coldkey.
    //
    pub fn increase_stake_on_hotkey_account(hotkey: &T::AccountId, increment: u64) {
        Self::increase_stake_on_coldkey_hotkey_account(
            &Self::get_owning_coldkey_for_hotkey(hotkey),
            hotkey,
            increment,
        );
    }

    // Decreases the stake on the hotkey account under its owning coldkey.
    //
    pub fn decrease_stake_on_hotkey_account(hotkey: &T::AccountId, decrement: u64) {
        Self::decrease_stake_on_coldkey_hotkey_account(
            &Self::get_owning_coldkey_for_hotkey(hotkey),
            hotkey,
            decrement,
        );
    }

    // Increases the stake on the cold - hot pairing by increment while also incrementing other counters.
    // This function should be called rather than set_stake under account.
    //
    pub fn increase_stake_on_coldkey_hotkey_account(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        increment: u64,
    ) {
        TotalColdkeyStake::<T>::insert(
            coldkey,
            TotalColdkeyStake::<T>::get(coldkey).saturating_add(increment),
        );
        TotalHotkeyStake::<T>::insert(
            hotkey,
            TotalHotkeyStake::<T>::get(hotkey).saturating_add(increment),
        );
        Stake::<T>::insert(
            hotkey,
            coldkey,
            Stake::<T>::get(hotkey, coldkey).saturating_add(increment),
        );
        TotalStake::<T>::put(TotalStake::<T>::get().saturating_add(increment));
    }

    // Decreases the stake on the cold - hot pairing by the decrement while decreasing other counters.
    //
    pub fn decrease_stake_on_coldkey_hotkey_account(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        decrement: u64,
    ) {
        TotalColdkeyStake::<T>::mutate(coldkey, |old| *old = old.saturating_sub(decrement));
        TotalHotkeyStake::<T>::insert(
            hotkey,
            TotalHotkeyStake::<T>::get(hotkey).saturating_sub(decrement),
        );
        Stake::<T>::insert(
            hotkey,
            coldkey,
            Stake::<T>::get(hotkey, coldkey).saturating_sub(decrement),
        );
        TotalStake::<T>::put(TotalStake::<T>::get().saturating_sub(decrement));
    }

    /// Empties the stake associated with a given coldkey-hotkey account pairing.
    /// This function retrieves the current stake for the specified coldkey-hotkey pairing,
    /// then subtracts this stake amount from both the TotalColdkeyStake and TotalHotkeyStake.
    /// It also removes the stake entry for the hotkey-coldkey pairing and adjusts the TotalStake
    /// and TotalIssuance by subtracting the removed stake amount.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - A reference to the AccountId of the coldkey involved in the staking.
    /// * `hotkey` - A reference to the AccountId of the hotkey associated with the coldkey.
    pub fn empty_stake_on_coldkey_hotkey_account(coldkey: &T::AccountId, hotkey: &T::AccountId) {
        let current_stake: u64 = Stake::<T>::get(hotkey, coldkey);
        TotalColdkeyStake::<T>::mutate(coldkey, |old| *old = old.saturating_sub(current_stake));
        TotalHotkeyStake::<T>::mutate(hotkey, |stake| *stake = stake.saturating_sub(current_stake));
        Stake::<T>::remove(hotkey, coldkey);
        TotalStake::<T>::mutate(|stake| *stake = stake.saturating_sub(current_stake));
        TotalIssuance::<T>::mutate(|issuance| *issuance = issuance.saturating_sub(current_stake));
    }

    /// Clears the nomination for an account, if it is a nominator account and the stake is below the minimum required threshold.
    pub fn clear_small_nomination_if_required(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        stake: u64,
    ) {
        // Verify if the account is a nominator account by checking ownership of the hotkey by the coldkey.
        if !Self::coldkey_owns_hotkey(&coldkey, &hotkey) {
            // If the stake is below the minimum required, it's considered a small nomination and needs to be cleared.
            if stake < Self::get_nominator_min_required_stake() {
                // Remove the stake from the nominator account. (this is a more forceful unstake operation which )
                // Actually deletes the staking account.
                Self::empty_stake_on_coldkey_hotkey_account(&coldkey, &hotkey);
                // Convert the removed stake back to balance and add it to the coldkey account.
                let stake_as_balance = Self::u64_to_balance(stake);
                Self::add_balance_to_coldkey_account(&coldkey, stake_as_balance.unwrap());
            }
        }
    }

    /// Clears small nominations for all accounts.
    ///
    /// WARN: This is an O(N) operation, where N is the number of staking accounts. It should be
    /// used with caution.
    pub fn clear_small_nominations() {
        // Loop through all staking accounts to identify and clear nominations below the minimum stake.
        for (hotkey, coldkey, stake) in Stake::<T>::iter() {
            Self::clear_small_nomination_if_required(&hotkey, &coldkey, stake);
        }
    }

    pub fn u64_to_balance(
        input: u64,
    ) -> Option<
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
    > {
        input.try_into().ok()
    }

    pub fn add_balance_to_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance,
    ) {
        T::Currency::deposit_creating(&coldkey, amount); // Infallibe
    }

    pub fn set_balance_on_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance,
    ) {
        T::Currency::make_free_balance_be(&coldkey, amount);
    }

    pub fn can_remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance,
    ) -> bool {
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }

        // This bit is currently untested. @todo
        let new_potential_balance = current_balance - amount;
        let can_withdraw = T::Currency::ensure_can_withdraw(
            &coldkey,
            amount,
            WithdrawReasons::except(WithdrawReasons::TIP),
            new_potential_balance,
        )
        .is_ok();
        can_withdraw
    }

    pub fn get_coldkey_balance(
        coldkey: &T::AccountId,
    ) -> <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance {
        return T::Currency::free_balance(&coldkey);
    }

    pub fn remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance,
    ) -> bool {
        return match T::Currency::withdraw(
            &coldkey,
            amount,
            WithdrawReasons::except(WithdrawReasons::TIP),
            ExistenceRequirement::KeepAlive,
        ) {
            Ok(_result) => true,
            Err(_error) => false,
        };
    }

    pub fn unstake_all_coldkeys_from_hotkey_account(hotkey: &T::AccountId) {
        // Iterate through all coldkeys that have a stake on this hotkey account.
        for (delegate_coldkey_i, stake_i) in
            <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(
                hotkey,
            )
        {
            // Convert to balance and add to the coldkey account.
            let stake_i_as_balance = Self::u64_to_balance(stake_i);
            if stake_i_as_balance.is_none() {
                continue; // Don't unstake if we can't convert to balance.
            } else {
                // Stake is successfully converted to balance.

                // Remove the stake from the coldkey - hotkey pairing.
                Self::decrease_stake_on_coldkey_hotkey_account(
                    &delegate_coldkey_i,
                    hotkey,
                    stake_i,
                );

                // Add the balance to the coldkey account.
                Self::add_balance_to_coldkey_account(
                    &delegate_coldkey_i,
                    stake_i_as_balance.unwrap(),
                );
            }
        }
    }
}
