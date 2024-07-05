use super::*;
use frame_support::{
    storage::IterableStorageDoubleMap,
    traits::{
        tokens::{
            fungible::{Balanced as _, Inspect as _, Mutate as _},
            Fortitude, Precision, Preservation,
        },
        Imbalance,
    }, weights::Weight,
};
use num_traits::Zero;
use sp_core::Get;

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
    /// *  'take' (u16):
    ///     - The stake proportion that this hotkey takes from delegations.
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
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signuture.
        let coldkey = ensure_signed(origin)?;
        ensure!(!Self::coldkey_is_locked(&coldkey), Error::<T>::ColdkeyIsInArbitration);
        log::info!(
            "do_become_delegate( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );

        // --- 2. Ensure we are delegating an known key.
        // --- 3. Ensure that the coldkey is the owner.
        Self::do_take_checks(&coldkey, &hotkey)?;

        // --- 4. Ensure we are not already a delegate (dont allow changing delegate take.)
        ensure!(
            !Self::hotkey_is_delegate(&hotkey),
            Error::<T>::HotKeyAlreadyDelegate
        );

        // --- 5. Ensure we don't exceed tx rate limit
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::DelegateTxRateLimitExceeded
        );

        // --- 5.1 Ensure take is within the min ..= InitialDefaultTake (18%) range
        let min_take = MinTake::<T>::get();
        let max_take = MaxTake::<T>::get();
        ensure!(take >= min_take, Error::<T>::DelegateTakeTooLow);
        ensure!(take <= max_take, Error::<T>::DelegateTakeTooHigh);

        // --- 6. Delegate the key.
        Self::delegate_hotkey(&hotkey, take);

        // Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);
        Self::set_last_tx_block_delegate_take(&coldkey, block);

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

    /// ---- The implementation for the extrinsic decrease_take
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     - The hotkey we are delegating (must be owned by the coldkey.)
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
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_decrease_take( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        ensure!(!Self::coldkey_is_locked(&coldkey), Error::<T>::ColdkeyIsInArbitration);       

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_take_checks(&coldkey, &hotkey)?;

        // --- 3. Ensure we are always strictly decreasing, never increasing take
        if let Ok(current_take) = Delegates::<T>::try_get(&hotkey) {
            ensure!(take < current_take, Error::<T>::DelegateTakeTooLow);
        }

        // --- 3.1 Ensure take is within the min ..= InitialDefaultTake (18%) range
        let min_take = MinTake::<T>::get();
        ensure!(take >= min_take, Error::<T>::DelegateTakeTooLow);

        // --- 4. Set the new take value.
        Delegates::<T>::insert(hotkey.clone(), take);

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
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_increase_take( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        ensure!(!Self::coldkey_is_locked(&coldkey), Error::<T>::ColdkeyIsInArbitration);       

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_take_checks(&coldkey, &hotkey)?;

        // --- 3. Ensure we are strinctly increasing take
        if let Ok(current_take) = Delegates::<T>::try_get(&hotkey) {
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
        Delegates::<T>::insert(hotkey.clone(), take);

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

    /// ---- The implementation for the extrinsic add_stake: Adds stake to a hotkey account.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     -  The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     -  The associated hotkey account.
    ///
    /// * 'stake_to_be_added' (u64):
    ///     -  The amount of stake to be added to the hotkey staking account.
    ///
    /// # Event:
    /// * StakeAdded;
    ///     -  On the successfully adding stake to a global account.
    ///
    /// # Raises:
    /// * 'NotEnoughBalanceToStake':
    ///     -  Not enough balance on the coldkey to add onto the global account.
    ///
    /// * 'NonAssociatedColdKey':
    ///     -  The calling coldkey is not associated with this hotkey.
    ///
    /// * 'BalanceWithdrawalError':
    ///     -  Errors stemming from transaction pallet.
    ///
    /// * 'TxRateLimitExceeded':
    ///     -  Thrown if key has hit transaction rate limit
    ///
    pub fn do_add_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        stake_to_be_added: u64,
    ) -> dispatch::DispatchResult {
        // We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_add_stake( origin:{:?} hotkey:{:?}, stake_to_be_added:{:?} )",
            coldkey,
            hotkey,
            stake_to_be_added
        );
        ensure!(!Self::coldkey_is_locked(&coldkey), Error::<T>::ColdkeyIsInArbitration);       

        // Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, stake_to_be_added),
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
            let total_stake_after_add =
                Stake::<T>::get(&hotkey, &coldkey).saturating_add(stake_to_be_added);

            ensure!(
                total_stake_after_add >= NominatorMinRequiredStake::<T>::get(),
                Error::<T>::NomStakeBelowMinimumThreshold
            );
        }

        // Ensure the remove operation from the coldkey is a success.
        let actual_amount_to_stake =
            Self::remove_balance_from_coldkey_account(&coldkey, stake_to_be_added)?;

        // If we reach here, add the balance to the hotkey.
        Self::increase_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, actual_amount_to_stake);

        // Set last block for rate limiting
        let block: u64 = Self::get_current_block_as_u64();
        Self::set_last_tx_block(&coldkey, block);

        // Emit the staking event.
        Self::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey,
            &hotkey,
            stakes_this_interval.saturating_add(1),
            block,
        );
        log::info!(
            "StakeAdded( hotkey:{:?}, stake_to_be_added:{:?} )",
            hotkey,
            actual_amount_to_stake
        );
        Self::deposit_event(Event::StakeAdded(hotkey, actual_amount_to_stake));

        // Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic remove_stake: Removes stake from a hotkey account and adds it onto a coldkey.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     -  The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     -  The associated hotkey account.
    ///
    /// * 'stake_to_be_added' (u64):
    ///     -  The amount of stake to be added to the hotkey staking account.
    ///
    /// # Event:
    /// * StakeRemoved;
    ///     -  On the successfully removing stake from the hotkey account.
    ///
    /// # Raises:
    /// * 'NotRegistered':
    ///     -  Thrown if the account we are attempting to unstake from is non existent.
    ///
    /// * 'NonAssociatedColdKey':
    ///     -  Thrown if the coldkey does not own the hotkey we are unstaking from.
    ///
    /// * 'NotEnoughStakeToWithdraw':
    ///     -  Thrown if there is not enough stake on the hotkey to withdwraw this amount.
    ///
    /// * 'TxRateLimitExceeded':
    ///     -  Thrown if key has hit transaction rate limit
    ///
    pub fn do_remove_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        stake_to_be_removed: u64,
    ) -> dispatch::DispatchResult {
        // We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, stake_to_be_removed:{:?} )",
            coldkey,
            hotkey,
            stake_to_be_removed
        );
        ensure!(!Self::coldkey_is_locked(&coldkey), Error::<T>::ColdkeyIsInArbitration);       

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
        ensure!(stake_to_be_removed > 0, Error::<T>::StakeToWithdrawIsZero);

        // Ensure that the hotkey has enough stake to withdraw.
        ensure!(
            Self::has_enough_stake(&coldkey, &hotkey, stake_to_be_removed),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Ensure we don't exceed stake rate limit
        let unstakes_this_interval =
            Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey);
        ensure!(
            unstakes_this_interval < Self::get_target_stakes_per_interval(),
            Error::<T>::UnstakeRateLimitExceeded
        );

        // We remove the balance from the hotkey.
        Self::decrease_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_to_be_removed);

        // We add the balance to the coldkey.  If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_removed);

        // If the stake is below the minimum, we clear the nomination from storage.
        // This only applies to nominator stakes.
        // If the coldkey does not own the hotkey, it's a nominator stake.
        let new_stake = Self::get_stake_for_coldkey_and_hotkey(&coldkey, &hotkey);
        Self::clear_small_nomination_if_required(&hotkey, &coldkey, new_stake);

        // Set last block for rate limiting
        let block: u64 = Self::get_current_block_as_u64();
        Self::set_last_tx_block(&coldkey, block);

        // Emit the unstaking event.
        Self::set_stakes_this_interval_for_coldkey_hotkey(
            &coldkey,
            &hotkey,
            unstakes_this_interval.saturating_add(1),
            block,
        );
        log::info!(
            "StakeRemoved( hotkey:{:?}, stake_to_be_removed:{:?} )",
            hotkey,
            stake_to_be_removed
        );
        Self::deposit_event(Event::StakeRemoved(hotkey, stake_to_be_removed));

        // Done and ok.
        Ok(())
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

    // Returns the total amount of stake in the staking table.
    //
    pub fn get_total_stake() -> u64 {
        TotalStake::<T>::get()
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
        TotalHotkeyStake::<T>::get(hotkey)
    }

    // Returns the total amount of stake held by the coldkey (delegative or otherwise)
    //
    pub fn get_total_stake_for_coldkey(coldkey: &T::AccountId) -> u64 {
        TotalColdkeyStake::<T>::get(coldkey)
    }

    // Returns the stake under the cold - hot pairing in the staking table.
    //
    pub fn get_stake_for_coldkey_and_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> u64 {
        Stake::<T>::get(hotkey, coldkey)
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
        let block_to_reset_after = block_last_staked_at.saturating_add(stake_interval);

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
            Stake::<T>::insert(hotkey, coldkey, 0);
            Owner::<T>::insert(hotkey, coldkey);

            // Update OwnedHotkeys map
            let mut hotkeys = OwnedHotkeys::<T>::get(coldkey);
            if !hotkeys.contains(hotkey) {
                hotkeys.push(hotkey.clone());
                OwnedHotkeys::<T>::insert(coldkey, hotkeys);
            }

            // Update StakingHotkeys map
            let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
            if !staking_hotkeys.contains(hotkey) {
                staking_hotkeys.push(hotkey.clone());
                StakingHotkeys::<T>::insert(coldkey, staking_hotkeys);
            }
        }
    }

    // Returns the coldkey owning this hotkey. This function should only be called for active accounts.
    //
    pub fn get_owning_coldkey_for_hotkey(hotkey: &T::AccountId) -> T::AccountId {
        Owner::<T>::get(hotkey)
    }

    // Returns the hotkey take
    //
    pub fn get_hotkey_take(hotkey: &T::AccountId) -> u16 {
        Delegates::<T>::get(hotkey)
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
    pub fn has_enough_stake(coldkey: &T::AccountId, hotkey: &T::AccountId, decrement: u64) -> bool {
        Self::get_stake_for_coldkey_and_hotkey(coldkey, hotkey) >= decrement
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

        // Update StakingHotkeys map
        let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
        if !staking_hotkeys.contains(hotkey) {
            staking_hotkeys.push(hotkey.clone());
            StakingHotkeys::<T>::insert(coldkey, staking_hotkeys);
        }
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

        // TODO: Tech debt: Remove StakingHotkeys entry if stake goes to 0
    }

    /// Empties the stake associated with a given coldkey-hotkey account pairing.
    /// This function retrieves the current stake for the specified coldkey-hotkey pairing,
    /// then subtracts this stake amount from both the TotalColdkeyStake and TotalHotkeyStake.
    /// It also removes the stake entry for the hotkey-coldkey pairing and adjusts the TotalStake
    /// and TotalIssuance by subtracting the removed stake amount.
    ///
    /// Returns the amount of stake that was removed.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - A reference to the AccountId of the coldkey involved in the staking.
    /// * `hotkey` - A reference to the AccountId of the hotkey associated with the coldkey.
    pub fn empty_stake_on_coldkey_hotkey_account(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
    ) -> u64 {
        let current_stake: u64 = Stake::<T>::get(hotkey, coldkey);
        TotalColdkeyStake::<T>::mutate(coldkey, |old| *old = old.saturating_sub(current_stake));
        TotalHotkeyStake::<T>::mutate(hotkey, |stake| *stake = stake.saturating_sub(current_stake));
        Stake::<T>::remove(hotkey, coldkey);
        TotalStake::<T>::mutate(|stake| *stake = stake.saturating_sub(current_stake));
        TotalIssuance::<T>::mutate(|issuance| *issuance = issuance.saturating_sub(current_stake));

        // Update StakingHotkeys map
        let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
        staking_hotkeys.retain(|h| h != hotkey);
        StakingHotkeys::<T>::insert(coldkey, staking_hotkeys);

        current_stake
    }

    /// Clears the nomination for an account, if it is a nominator account and the stake is below the minimum required threshold.
    pub fn clear_small_nomination_if_required(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        stake: u64,
    ) {
        // Verify if the account is a nominator account by checking ownership of the hotkey by the coldkey.
        if !Self::coldkey_owns_hotkey(coldkey, hotkey) {
            // If the stake is below the minimum required, it's considered a small nomination and needs to be cleared.
            if stake < Self::get_nominator_min_required_stake() {
                // Remove the stake from the nominator account. (this is a more forceful unstake operation which )
                // Actually deletes the staking account.
                let cleared_stake = Self::empty_stake_on_coldkey_hotkey_account(coldkey, hotkey);
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
        for (hotkey, coldkey, stake) in Stake::<T>::iter() {
            Self::clear_small_nomination_if_required(&hotkey, &coldkey, stake);
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

        T::Currency::can_withdraw(coldkey, amount)
            .into_result(false)
            .is_ok()
    }

    pub fn get_coldkey_balance(
        coldkey: &T::AccountId,
    ) -> <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance
    {
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

    pub fn kill_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> Result<u64, DispatchError> {
        if amount == 0 {
            return Ok(0);
        }

        let credit = T::Currency::withdraw(
            coldkey,
            amount,
            Precision::Exact,
            Preservation::Expendable,
            Fortitude::Force,
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
        for (delegate_coldkey_i, stake_i) in
            <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(
                hotkey,
            )
        {
            // Remove the stake from the coldkey - hotkey pairing.
            Self::decrease_stake_on_coldkey_hotkey_account(&delegate_coldkey_i, hotkey, stake_i);

            // Add the balance to the coldkey account.
            Self::add_balance_to_coldkey_account(&delegate_coldkey_i, stake_i);
        }
    }

    /// Unstakes all tokens associated with a hotkey and transfers them to a new coldkey.
    ///
    /// This function performs the following operations:
    /// 1. Verifies that the hotkey exists and is owned by the current coldkey.
    /// 2. Ensures that the new coldkey is different from the current one.
    /// 3. Unstakes all balance if there's any stake.
    /// 4. Transfers the entire balance of the hotkey to the new coldkey.
    /// 5. Verifies the success of the transfer and handles partial transfers if necessary.
    ///
    /// # Arguments
    ///
    /// * `current_coldkey` - The AccountId of the current coldkey.
    /// * `new_coldkey` - The AccountId of the new coldkey to receive the unstaked tokens.
    ///
    /// # Returns
    ///
    /// Returns a `DispatchResult` indicating success or failure of the operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The hotkey account does not exist.
    /// * The current coldkey does not own the hotkey.
    /// * The new coldkey is the same as the current coldkey.
    /// * There is no balance to transfer.
    /// * The transfer fails or is only partially successful.
    ///
    /// # Events
    ///
    /// Emits an `AllBalanceUnstakedAndTransferredToNewColdkey` event upon successful execution.
    /// Emits a `PartialBalanceTransferredToNewColdkey` event if only a partial transfer is successful.
    ///
    pub fn do_unstake_all_and_transfer_to_new_coldkey(
        current_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) -> DispatchResult {

        // Get the current wallets to drain to.
        let mut coldkeys_to_drain_to: Vec<T::AccountId> = ColdkeysToDrainTo::<T>::get( current_coldkey );

        // Check if the new coldkey is already in the drain wallets list
        ensure!(
            !coldkeys_to_drain_to.contains( new_coldkey ),
            Error::<T>::DuplicateColdkey
        );
    
        // Add the wallet to the drain wallets. 
        if coldkeys_to_drain_to.len() == 0 as usize || coldkeys_to_drain_to.len() == 1 as usize {

            // Extend the wallet to drain to.
            coldkeys_to_drain_to.push(new_coldkey.clone());

            // Push the change.
            ColdkeysToDrainTo::<T>::insert( current_coldkey, coldkeys_to_drain_to.clone() );
        } else {
            return Err(Error::<T>::ColdkeyIsInArbitration.into());
        }

        // If this is the first time we have seen this key we will put the drain period to be in 1 week.
        if coldkeys_to_drain_to.len() == 0 as usize {

            // Get the current block.
            let current_block: u64 = Self::get_current_block_as_u64();

            // Next arbitrage period
            let next_arbitrage_period: u64 = current_block + 7200 * 7;

            // First time seeing this key lets push the drain moment to 1 week in the future.
            let mut next_period_coldkeys_to_drain: Vec<T::AccountId> = ColdkeysToDrainOnBlock::<T>::get( next_arbitrage_period );

            // Add this new coldkey to these coldkeys
            // Sanity Check.
            if !next_period_coldkeys_to_drain.contains(new_coldkey) {
                next_period_coldkeys_to_drain.push(new_coldkey.clone());
            }

            // Set the new coldkeys to drain here.
            ColdkeysToDrainOnBlock::<T>::insert( next_arbitrage_period, next_period_coldkeys_to_drain );
        }

        // Return true.
        Ok(())
    }


    // Chain opens, this is the only allowed transaction.
    // 1. Someone calls drain with key C1, DestA, DestA is added to ColdToBeDrained and C1 is given a drain block in 7 days.
    // 2. Someone else calls drain with key C1, DestB, DestB is added to ColdToBeDrained and C1 already has a drain block, not updated.
    // 3. Someone calls drain key with C1, DestC. Dest C is not added to the ColdToBeDrained. No-op/
    // 4. 7200 * 7 blocks progress.
    // 5. ColdkeysToDrainOnBlock( block ) returns [ C1 ]
    // 6. ColdToBeDrained( C1 ) returns [ DestA, DestB ] and is Drained
    // 7. len([ DestA, DestB ]) == 2, set the drain block to be 7 days in the future.
    // 8. Someone calls drain with key C1, DestA, DestD is added to ColdToBeDrained and C1 is given a drain block in 7 days.
    // 9. 7200 * 7 blocks progress.
    // 10. ColdkeysToDrainOnBlock( block ) returns [ C1 ]
    // 11. ColdToBeDrained( C1 ) returns [ DestD ] and is Drained
    // 12. len([ DestD ]) == 1, call_drain( C1, DestD )

    pub fn drain_all_pending_coldkeys() -> Weight {
        let mut weight = frame_support::weights::Weight::from_parts(0, 0);

        // Get the block number
        let current_block: u64 = Self::get_current_block_as_u64();

        // Get the coldkeys to drain here.
        let source_coldkeys: Vec<T::AccountId> = ColdkeysToDrainOnBlock::<T>::get( current_block );
        ColdkeysToDrainOnBlock::<T>::remove( current_block );
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        // Iterate over all keys in Drain and call drain_to_pending_coldkeys for each
        for coldkey_i in source_coldkeys.iter() {

            // Get the wallets to drain to for this coldkey.
            let destinations_coldkeys: Vec<T::AccountId> = ColdkeysToDrainTo::<T>::get( coldkey_i );
            ColdkeysToDrainTo::<T>::remove( &coldkey_i );
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

            // If the wallets to drain is > 1, we extend the period.
            if destinations_coldkeys.len() > 1 {

                // Next arbitrage period
                let next_arbitrage_period: u64 = current_block + 7200 * 30;

                // Get the coldkeys to drain at the next arbitrage period.
                let mut next_period_coldkeys_to_drain: Vec<T::AccountId> = ColdkeysToDrainOnBlock::<T>::get( next_arbitrage_period );
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

                // Add this new coldkey to these coldkeys
                // Sanity Check.
                if !next_period_coldkeys_to_drain.contains(coldkey_i) {
                    next_period_coldkeys_to_drain.push(coldkey_i.clone());
                }

                // Set the new coldkeys to drain here.
                ColdkeysToDrainOnBlock::<T>::insert( next_arbitrage_period, next_period_coldkeys_to_drain );
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

            } else if destinations_coldkeys.len() == 1 {
                // ONLY 1 wallet: Get the wallet to drain to.
                let wallet_to_drain_to = &destinations_coldkeys[0];

                // Perform the drain.
                weight = weight.saturating_add(
                    Self::drain_from_coldkey_a_to_coldkey_b( &coldkey_i, wallet_to_drain_to )
                );
            }
        }

        weight
    }


    pub fn drain_from_coldkey_a_to_coldkey_b( coldkey_a: &T::AccountId, coldkey_b: &T::AccountId ) -> Weight {
        let mut weight = frame_support::weights::Weight::from_parts(0, 0);

        // Get the hotkeys associated with coldkey_a.
        let coldkey_a_hotkeys: Vec<T::AccountId> = StakingHotkeys::<T>::get( &coldkey_a );
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

        // Iterate over all the hotkeys associated with this coldkey
        for hotkey_i in coldkey_a_hotkeys.iter() {

            // Get the current stake from coldkey_a to hotkey_i.
            let all_current_stake_i: u64 = Self::get_stake_for_coldkey_and_hotkey( &coldkey_a, &hotkey_i );
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

            // We remove the balance from the hotkey acount equal to all of it.
            Self::decrease_stake_on_coldkey_hotkey_account( &coldkey_a, &hotkey_i, all_current_stake_i );
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 4));

            // We add the balance to the coldkey. If the above fails we will not credit this coldkey.
            Self::add_balance_to_coldkey_account( &coldkey_a, all_current_stake_i );
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(2, 2));
        }

        // Get the total balance here.
        let total_balance = Self::get_coldkey_balance( &coldkey_a );
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

        if !total_balance.is_zero() {
            // Attempt to transfer the entire total balance to coldkey_b.
            let _ = T::Currency::transfer(
                coldkey_a,
                coldkey_b,
                total_balance,
                Preservation::Expendable,
            );
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        weight
    }

    pub fn coldkey_is_locked( coldkey: &T::AccountId ) -> bool {
        ColdkeysToDrainTo::<T>::contains_key( coldkey )
    }
}
