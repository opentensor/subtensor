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
        coldkey: T::AccountId,
        origin_hotkey: T::AccountId,
        destination_hotkey: T::AccountId,
        origin_netuid: u16,
        destination_netuid: u16,
        alpha_amount: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. Check that the subnet exists.
        ensure!(
            Self::if_subnet_exist(origin_netuid),
            Error::<T>::SubnetNotExists
        );
        ensure!(
            Self::if_subnet_exist(destination_netuid),
            Error::<T>::SubnetNotExists
        );

        // --- 2. Check that the origin_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&origin_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 3. Check that the destination_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&destination_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 4. Get the current alpha stake for the origin hotkey-coldkey pair in the origin subnet
        let origin_alpha = Alpha::<T>::get((origin_hotkey.clone(), origin_netuid, coldkey.clone()));
        ensure!(
            alpha_amount <= origin_alpha,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        Self::try_increase_staking_counter(&coldkey, &destination_hotkey)?;
        Self::try_increase_staking_counter(&coldkey, &origin_hotkey)?;

        // --- 5. Unstake the amount of alpha from the origin subnet, converting it to TAO
        let origin_tao = Self::unstake_from_subnet(
            &origin_hotkey.clone(),
            &coldkey.clone(),
            origin_netuid,
            alpha_amount,
        );

        // --- 6. Stake the resulting TAO into the destination subnet for the destination hotkey
        Self::stake_into_subnet(
            &destination_hotkey.clone(),
            &coldkey.clone(),
            destination_netuid,
            origin_tao,
        );

        // Set last block for rate limiting
        let current_block = Self::get_current_block_as_u64();
        Self::set_last_tx_block(&coldkey, current_block);

        // Adjust the stake deltas
        if alpha_amount > 0 {
            StakeDeltaSinceLastEmissionDrain::<T>::mutate(
                (origin_hotkey.clone(), origin_netuid, coldkey.clone()),
                |alpha| alpha.saturating_sub_unsigned(alpha_amount as u128),
            );
            StakeDeltaSinceLastEmissionDrain::<T>::mutate(
                (
                    destination_hotkey.clone(),
                    destination_netuid,
                    coldkey.clone(),
                ),
                |alpha| alpha.saturating_add_unsigned(alpha_amount as u128),
            );
        }

        // --- 7. Log the event.
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

        // -- 8. Ok and return.
        Ok(())
    }

    pub fn do_remove_all_stake(
        coldkey: T::AccountId,
        hotkey: T::AccountId,
        netuids: Vec<u16>,
    ) -> dispatch::DispatchResult {
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Check that all subnets exist
        ensure!(
            netuids.iter().all(|netuid| Self::if_subnet_exist(*netuid)),
            Error::<T>::SubnetNotExists
        );

        // If no subnets are specified, remove all stake from all subnets
        let netuids = if netuids.is_empty() {
            Self::get_all_subnet_netuids()
        } else {
            netuids
        };

        Self::try_increase_staking_counter(&coldkey, &hotkey)?;

        // Set last block for rate limiting
        let current_block = Self::get_current_block_as_u64();
        Self::set_last_tx_block(&coldkey, current_block);

        for netuid in netuids {
            // Check alpha on each netuid
            let alpha = Alpha::<T>::get((hotkey.clone(), netuid, coldkey.clone()));
            if alpha > 0 {
                // Unstake the alpha
                Self::unstake_from_subnet(&hotkey.clone(), &coldkey.clone(), netuid, alpha);
            }
        }

        Ok(())
    }
}
