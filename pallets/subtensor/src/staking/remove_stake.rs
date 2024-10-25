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
        netuid: u16,
        alpha_unstaked: u64,
    ) -> dispatch::DispatchResult {
        // We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, alpha_unstaked:{:?} )",
            coldkey,
            hotkey,
            alpha_unstaked
        );

        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Ensure that the hotkey account exists this is only possible through registration.
        // ensure!(
        //     Self::hotkey_account_exists(&hotkey),
        //     Error::<T>::HotKeyAccountNotExists
        // );

        // Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        // DEPRECATED.
        // ensure!(
        //     Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
        //     Error::<T>::HotKeyNotDelegateAndSignerNotOwnHotKey
        // );

        // Ensure that the stake amount to be removed is above zero.
        ensure!(alpha_unstaked > 0, Error::<T>::StakeToWithdrawIsZero);

        // Ensure that the hotkey has enough stake to withdraw.
        ensure!(
            Self::has_enough_stake_on_subnet(&hotkey, &coldkey, netuid, alpha_unstaked),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Ensure we don't exceed stake rate limit
        let unstakes_this_interval =
            Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey);
        ensure!(
            unstakes_this_interval < Self::get_target_stakes_per_interval(),
            Error::<T>::UnstakeRateLimitExceeded
        );

        // Ensure we can unstake this with locks.
        let total_stake: u64 =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        // Ensure we are not unstaking more than allowed
        let current_block = Self::get_current_block_as_u64();
        log::debug!("here");

        if Locks::<T>::contains_key((netuid, hotkey.clone(), coldkey.clone())) {
            // Retrieve the lock information for the given netuid, hotkey, and coldkey
            let (alpha_locked, _start_block, end_block) =
                Locks::<T>::get((netuid, hotkey.clone(), coldkey.clone()));
            let conviction = Self::calculate_conviction(alpha_locked, end_block, current_block);
            let stake_after_unstake = total_stake.saturating_sub(alpha_unstaked);
            // Ensure the requested unstake amount is not more than what's allowed
            ensure!(
                stake_after_unstake >= conviction,
                Error::<T>::NotEnoughStakeToWithdraw
            );
            // If conviction is 0, remove the lock
            if conviction == 0 {
                Locks::<T>::remove((netuid, hotkey.clone(), coldkey.clone()));
            }
        }

        // Convert and unstake from the subnet.
        let tao_unstaked: u64 =
            Self::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_unstaked);

        // We add the balance to the coldkey.  If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked);

        // If the stake is below the minimum, we clear the nomination from storage.
        // This only applies to nominator stakes.
        // If the coldkey does not own the hotkey, it's a nominator stake.
        let new_stake = Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid, new_stake);

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
            "StakeRemoved( hotkey:{:?}, tao_unstaked:{:?} )",
            hotkey.clone(),
            tao_unstaked
        );

        // Done and ok.
        Ok(())
    }
}
