use super::*;
// use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    /// Moves stake from one hotkey to another across subnets.
    ///
    /// # Arguments
    /// * `origin` - The origin of the transaction, which must be signed by the `origin_hotkey`.
    /// * `origin_hotkey` - The account ID of the hotkey from which the stake is being moved.
    /// * `destination_hotkey` - The account ID of the hotkey to which the stake is being moved.
    /// * `origin_netuid` - The network ID of the origin subnet.
    /// * `destination_netuid` - The network ID of the destination subnet.
    ///
    /// # Returns
    /// * `DispatchResult` - Indicates the success or failure of the operation.
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The origin is not signed by the `origin_hotkey`.
    /// * Either the origin or destination subnet does not exist.
    /// * The `origin_hotkey` or `destination_hotkey` does not exist.
    /// * There are locked funds that cannot be moved across subnets.
    ///
    /// # Events
    /// Emits a `StakeMoved` event upon successful completion of the stake movement.
    pub fn do_move_stake(
        origin: T::RuntimeOrigin,
        origin_hotkey: T::AccountId,
        destination_hotkey: T::AccountId,
        origin_netuid: u16,
        destination_netuid: u16,
        alpha_amount: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. Check that the origin is signed by the origin_hotkey.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Check that the subnet exists.
        ensure!(
            Self::if_subnet_exist(origin_netuid),
            Error::<T>::SubnetNotExists
        );
        ensure!(
            Self::if_subnet_exist(destination_netuid),
            Error::<T>::SubnetNotExists
        );

        // --- 3. Check that the origin_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&origin_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 4. Check that the destination_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&destination_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 6. Get the current alpha stake for the origin hotkey-coldkey pair in the origin subnet
        let origin_alpha = Alpha::<T>::get((origin_hotkey.clone(), coldkey.clone(), origin_netuid));
        ensure!(
            alpha_amount <= origin_alpha,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Ensure we don't exceed stake rate limit for destination key
        // let stakes_this_interval =
        //     Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &destination_hotkey);
        // ensure!(
        //     stakes_this_interval < Self::get_target_stakes_per_interval(),
        //     Error::<T>::StakeRateLimitExceeded
        // ); (DEPRECATED)

        // Ensure we don't exceed stake rate limit for origin key
        // let unstakes_this_interval =
        //     Self::get_stakes_this_interval_for_coldkey_hotkey(&coldkey, &origin_hotkey);
        // ensure!(
        //     unstakes_this_interval < Self::get_target_stakes_per_interval(),
        //     Error::<T>::UnstakeRateLimitExceeded
        // ); (DEPRECATED)

        // --- 7. Unstake the amount of alpha from the origin subnet, converting it to TAO
        let origin_tao = Self::unstake_from_subnet(
            &origin_hotkey.clone(),
            &coldkey.clone(),
            origin_netuid,
            alpha_amount,
        );

        // --- 8. Stake the resulting TAO into the destination subnet for the destination hotkey
        Self::stake_into_subnet(
            &destination_hotkey.clone(),
            &coldkey.clone(),
            destination_netuid,
            origin_tao,
        );

        // Set last block for rate limiting
        // let current_block = Self::get_current_block_as_u64();
        // Self::set_last_tx_block(&coldkey, current_block);
        // Self::set_stakes_this_interval_for_coldkey_hotkey(
        //     &coldkey,
        //     &destination_hotkey,
        //     stakes_this_interval.saturating_add(1),
        //     current_block,
        // );
        // Self::set_stakes_this_interval_for_coldkey_hotkey(
        //     &coldkey,
        //     &origin_hotkey,
        //     unstakes_this_interval.saturating_add(1),
        //     current_block,
        // ); (DEPRECATED)

        // Set the last time the stake increased for nominator drain protection.
        // if alpha_amount > 0 {
        //     LastAddStakeIncrease::<T>::insert(&destination_hotkey, &coldkey, current_block);
        // } DEPRECATED

        // --- 10. Log the event.
        log::info!(
            "StakeMoved( coldkey:{:?}, origin_hotkey:{:?}, origin_netuid:{:?}, destination_hotkey:{:?}, destination_netuid:{:?} )",
            coldkey.clone(),
            origin_hotkey.clone(),
            origin_netuid,
            destination_hotkey.clone(),
            destination_netuid
        );
        Self::deposit_event(Event::StakeMoved(
            coldkey,
            origin_hotkey,
            origin_netuid,
            destination_hotkey,
            destination_netuid,
        ));

        // -- 11. Ok and return.
        Ok(())
    }
}
