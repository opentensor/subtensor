use super::*;
// use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    #[allow(clippy::arithmetic_side_effects)]
    /// Moves stake from one hotkey to another across subnets.
    ///
    /// # Arguments
    /// * `origin` - The origin of the transaction, which must be signed by the `origin_hotkey`.
    /// * `origin_hotkey` - The account ID of the hotkey from which the stake is being moved.
    /// * `destination_hotkey` - The account ID of the hotkey to which the stake is being moved.
    /// * `origin_netuid` - The network ID of the origin subnet.
    /// * `amount_moved` - The amount unstaked from the origin network.
    /// * `netuid_amount_vec` - The distribution of unstaked TAO, to different network and the propotions
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
        amount_moved: Option<u64>,
        netuid_amount_vec: Vec<(u16, u64)>,
    ) -> dispatch::DispatchResult {
        // --- 1. Check that the origin is signed by the origin_hotkey.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Check that the original subnet exists.
        ensure!(
            Self::if_subnet_exist(origin_netuid),
            Error::<T>::SubnetNotExists
        );

        // --- 3. Check destination netuid and get the sum of amount
        let mut unique_netuid = Vec::new();
        let mut tatal_moved = 0_u64;

        for (netuid, amount) in netuid_amount_vec.iter() {
            ensure!(Self::if_subnet_exist(*netuid), Error::<T>::SubnetNotExists);
            ensure!(!unique_netuid.contains(netuid), Error::<T>::DuplicateNetuid);
            unique_netuid.push(*netuid);
            tatal_moved = tatal_moved.saturating_add(*amount);
        }

        // --- 4. Check that the total moved amount
        ensure!(tatal_moved > 0, Error::<T>::TotalMovedAmountIsZero);

        // --- 5. Check that the origin_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&origin_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 6. Check that the destination_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&destination_hotkey.clone()),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 8. Get the current alpha stake for the origin hotkey-coldkey pair in the origin subnet
        // or use amount_moved
        let origin_alpha = Alpha::<T>::get((origin_hotkey.clone(), coldkey.clone(), origin_netuid));

        let move_alpha = match amount_moved {
            Some(amount) if amount <= origin_alpha => amount,
            _ => origin_alpha,
        };

        ensure!(move_alpha > 0, Error::<T>::MoveAmountCanNotBeZero);

        // -- 7. If move just in the same network, swap the lock. otherwise, return error if lock existed
        if unique_netuid.len() == 1 && unique_netuid.first() == Some(&origin_netuid) {
            if Locks::<T>::contains_key((origin_netuid, origin_hotkey.clone(), coldkey.clone())) {
                // swap lock just in the same network
                let lock_data =
                    Locks::<T>::take((origin_netuid, origin_hotkey.clone(), coldkey.clone()));

                Locks::<T>::insert(
                    (origin_netuid, destination_hotkey.clone(), coldkey.clone()),
                    lock_data,
                );
            }
        } else {
            ensure!(
                !Locks::<T>::contains_key((origin_netuid, origin_hotkey.clone(), coldkey.clone())),
                Error::<T>::MovedStakeIsLocked
            );
        }

        // --- 9. Unstake the full amount of alpha from the origin subnet, converting it to TAO
        let origin_tao = Self::unstake_from_subnet(
            &origin_hotkey.clone(),
            &coldkey.clone(),
            origin_netuid,
            move_alpha,
        );

        // --- 10. Stake the resulting TAO into the destination subnet for the destination hotkey
        for (netuid, amount) in netuid_amount_vec.iter() {
            // --- 11. Calculate the added tao for each netuid according to the proportion.
            let added_tao = origin_tao
                .saturating_mul(*amount)
                .saturating_div(tatal_moved);

            // --- 12 add stake to destination netuid
            Self::stake_into_subnet(
                &destination_hotkey.clone(),
                &coldkey.clone(),
                *netuid,
                added_tao,
            );

            // --- 14. Log the event.
            log::info!(
				"StakeMoved( coldkey:{:?}, origin_hotkey:{:?}, origin_netuid:{:?}, destination_hotkey:{:?}, destination_netuid:{:?} )",
				coldkey.clone(),
				origin_hotkey.clone(),
				origin_netuid,
				destination_hotkey.clone(),
				*netuid
			);

            // --- 15. Emit the StakeMoved event.
            Self::deposit_event(Event::StakeMoved(
                coldkey.clone(),
                origin_hotkey.clone(),
                origin_netuid,
                destination_hotkey.clone(),
                *netuid,
                added_tao,
            ));
        }

        // -- 16. Ok and return.
        Ok(())
    }
}
