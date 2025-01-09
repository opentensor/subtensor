use super::*;

impl<T: Config> Pallet<T> {
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
        log::debug!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, stake_to_be_removed:{:?} )",
            coldkey,
            hotkey,
            stake_to_be_removed
        );

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Owner::<T>::contains_key(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!(
            Delegates::<T>::contains_key(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::HotKeyNotDelegateAndSignerNotOwnHotKey
        );

        // Ensure that the stake amount to be removed is above zero.
        ensure!(stake_to_be_removed > 0, Error::<T>::StakeToWithdrawIsZero);

        // Ensure that the hotkey has enough stake to withdraw.
        ensure!(
            Stake::<T>::get(&hotkey, &coldkey) >= stake_to_be_removed,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        Self::try_increase_staking_counter(&coldkey, &hotkey)?;

        // We remove the balance from the hotkey.
        Self::decrease_stake_on_coldkey_hotkey_account(&coldkey, &hotkey, stake_to_be_removed);

        // Track this removal in the stake delta.
        StakeDeltaSinceLastEmissionDrain::<T>::mutate(&hotkey, &coldkey, |stake_delta| {
            *stake_delta = stake_delta.saturating_sub_unsigned(stake_to_be_removed as u128);
        });

        // We add the balance to the coldkey.  If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_removed);

        // If the stake is below the minimum, we clear the nomination from storage.
        // This only applies to nominator stakes.
        // If the coldkey does not own the hotkey, it's a nominator stake.
        let new_stake = Stake::<T>::get(&hotkey, &coldkey);
        Self::clear_small_nomination_if_required(&hotkey, &coldkey, new_stake);

        // Check if stake lowered below MinStake and remove Pending children if it did
        if TotalHotkeyStake::<T>::get(&hotkey) < StakeThreshold::<T>::get() {
            Self::get_all_subnet_netuids().iter().for_each(|netuid| {
                PendingChildKeys::<T>::remove(netuid, &hotkey);
            })
        }

        // Set last block for rate limiting
        let block = Self::get_current_block_as_u64();
        LastTxBlock::<T>::insert(&coldkey, block);

        // Emit the unstaking event.
        log::debug!(
            "StakeRemoved( hotkey:{:?}, stake_to_be_removed:{:?} )",
            hotkey,
            stake_to_be_removed
        );
        Self::deposit_event(Event::StakeRemoved(hotkey, stake_to_be_removed));

        // Done and ok.
        Ok(())
    }
}
