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
        let amount_tao_staked = Self::remove_balance_from_coldkey_account(&coldkey, stake_to_be_added)?;

        // Compute the stake operation based on the mechanism.
        let mechid: u16 = SubnetMechanism::<T>::get( netuid );
        let alpha_amount_staked: u64;
        if mechid == 2 { // STAO
            // Compute dynamic stake.
            let total_subnet_tao: u64 = SubnetTAO::<T>::get( netuid );
            let total_mechanism_tao: u64 = Self::get_total_mechanism_tao( SubnetMechanism::<T>::get( netuid ) );
            alpha_amount_staked = amount_tao_staked * ((total_mechanism_tao + amount_tao_staked) / (total_subnet_tao + amount_tao_staked));
        } else { // ROOT and other.
            alpha_amount_staked = amount_tao_staked;
        }
        // Increment counters.
        SubnetAlpha::<T>::insert(
            netuid,
            SubnetAlpha::<T>::get(netuid).saturating_add(alpha_amount_staked),
        );
        SubnetTAO::<T>::insert(
            netuid,
            SubnetTAO::<T>::get(netuid).saturating_add( amount_tao_staked ),
        );
        // TotalColdkeyStake::<T>::insert(
        //     coldkey,
        //     TotalColdkeyStake::<T>::get(coldkey).saturating_add( amount_tao_staked ),
        // );
        // TotalHotkeyStake::<T>::insert(
        //     hotkey,
        //     TotalHotkeyStake::<T>::get(hotkey).saturating_add( amount_tao_staked ),
        // );
        Stake::<T>::insert(
            &hotkey,
            &coldkey,
            Stake::<T>::get( &hotkey, &coldkey ).saturating_add( amount_tao_staked ),
        );
        TotalHotkeyAlpha::<T>::insert(
            &hotkey,
            netuid,
            TotalHotkeyAlpha::<T>::get( &hotkey, netuid ).saturating_add( alpha_amount_staked ),
        );
        Alpha::<T>::insert(
            (&hotkey, &coldkey, netuid),
            Alpha::<T>::get((&hotkey, &coldkey, netuid)).saturating_add( alpha_amount_staked ),
        );
        // Update Staking hotkeys map.
        let mut staking_hotkeys = StakingHotkeys::<T>::get(&coldkey);
        if !staking_hotkeys.contains(&hotkey) {
            staking_hotkeys.push(hotkey.clone());
            StakingHotkeys::<T>::insert(&coldkey, staking_hotkeys);
        }

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
            hotkey.clone(),
            alpha_amount_staked
        );
        Self::deposit_event(Event::StakeAdded(hotkey.clone(), alpha_amount_staked));

        // Ok and return.
        Ok(())
    }
}
