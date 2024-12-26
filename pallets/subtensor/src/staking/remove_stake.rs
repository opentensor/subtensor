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

        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, netuid: {:?}, alpha_unstaked:{:?} )",
            coldkey,
            hotkey,
            netuid,
            alpha_unstaked
        );

        // 1. Ensure that the stake amount to be removed is above zero.
        ensure!(alpha_unstaked > 0, Error::<T>::StakeToWithdrawIsZero);

        // 2. Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // 3. Ensure that the hotkey account exists this is only possible through registration.
        ensure!( Self::hotkey_account_exists(&hotkey), Error::<T>::HotKeyAccountNotExists );

        // 4. Ensure that the hotkey has enough stake to withdraw.
        ensure!(
            Self::has_enough_stake_on_subnet(&hotkey, &coldkey, netuid, alpha_unstaked),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // 5. Swap the alpba to tao and update counters for this subnet.
        let tao_unstaked: u64 =Self::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_unstaked);

        // 6. We add the balance to the coldkey. If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked);

        // 7. If the stake is below the minimum, we clear the nomination from storage.
        Self::clear_small_nomination_if_required( &hotkey, &coldkey, netuid );

        // 8. Done and ok.
        Ok(())
    }
}
