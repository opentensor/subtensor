use super::*;
// use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
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
        netuid: u16,
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

        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, stake_to_be_added),
            Error::<T>::NotEnoughBalanceToStake
        );

        // Ensure that the hotkey account exists this is only possible through registration.
        // Remove this requirement.
        if !Self::hotkey_account_exists(&hotkey) {
            Self::create_account_if_non_existent(&coldkey, &hotkey);
        }
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

        // Ensure we don't exceed stake rate limit
        // DEPRECATED
        let stakes_this_interval =
            Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &hotkey);
        ensure!(
            stakes_this_interval < Self::get_target_stakes_per_interval(),
            Error::<T>::StakeRateLimitExceeded
        );

        // Set the last time the stake increased for nominator drain protection.
        LastAddStakeIncrease::<T>::insert(&hotkey, &coldkey, Self::get_current_block_as_u64());

        // If coldkey is not owner of the hotkey, it's a nomination stake.
        if !Self::coldkey_owns_hotkey(&coldkey, &hotkey) {
            let total_stake_after_add: u64 =
                Alpha::<T>::get((&hotkey, &coldkey, netuid)).saturating_add(stake_to_be_added);
            ensure!(
                total_stake_after_add >= NominatorMinRequiredStake::<T>::get(),
                Error::<T>::NomStakeBelowMinimumThreshold
            );
        }

        // Ensure the remove operation from the coldkey is a success.
        let tao_staked: u64 =
            Self::remove_balance_from_coldkey_account(&coldkey, stake_to_be_added)?;

        // Convert and stake to alpha on the subnet.
        let alpha_staked: u64 = Self::stake_into_subnet(&hotkey, &coldkey, netuid, tao_staked);

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
            "StakeAdded( hotkey:{:?}, alpha_staked:{:?} )",
            hotkey.clone(),
            alpha_staked
        );

        // Ok and return.
        Ok(())
    }
}
