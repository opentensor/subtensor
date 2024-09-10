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

        // --- 2. Check that the subnet exists.
        ensure!(
            Self::if_subnet_exist(origin_netuid),
            Error::<T>::SubnetNotExists
        );

        let mut unique_netuid = Vec::new();
        let mut tatal_moved = 0_u64;

        for (netuid, amount) in netuid_amount_vec.iter() {
            ensure!(Self::if_subnet_exist(*netuid), Error::<T>::SubnetNotExists);
            ensure!(*netuid != origin_netuid, Error::<T>::DuplicateChild);
            ensure!(!unique_netuid.contains(netuid), Error::<T>::DuplicateChild);
            unique_netuid.push(*netuid);
            tatal_moved = tatal_moved.saturating_add(*amount);
        }

        ensure!(tatal_moved > 0, Error::<T>::HotKeyAccountNotExists);

        // --- 3. Check that the origin_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&origin_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 4. Check that the destination_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&destination_hotkey.clone()),
            Error::<T>::HotKeyAccountNotExists
        );

        // -- 5. Ensure we are not moving locked funds across subnets.
        // if origin_netuid != destination_netuid {
        // You cannot move locked funds across subnets.
        ensure!(
            !Locks::<T>::contains_key((origin_netuid, origin_hotkey.clone(), coldkey.clone())),
            Error::<T>::NotEnoughStakeToWithdraw
        );
        // }

        // --- 6. Get the current alpha stake for the origin hotkey-coldkey pair in the origin subnet
        // or use amount_moved
        let origin_alpha = Alpha::<T>::get((origin_hotkey.clone(), coldkey.clone(), origin_netuid));

        let move_alpha = match amount_moved {
            Some(amount) if amount <= origin_alpha => amount,
            _ => origin_alpha,
        };

        ensure!(move_alpha > 0, Error::<T>::MoveAmountCanNotBeZero);

        // --- 7. Unstake the full amount of alpha from the origin subnet, converting it to TAO
        let origin_tao = Self::unstake_from_subnet(
            &origin_hotkey.clone(),
            &coldkey.clone(),
            origin_netuid,
            move_alpha,
        );

        // --- 8. Stake the resulting TAO into the destination subnet for the destination hotkey
        for (netuid, amount) in netuid_amount_vec.iter() {
            let added_tao = origin_tao
                .saturating_mul(*amount)
                .saturating_div(tatal_moved);

            Self::stake_into_subnet(
                &destination_hotkey.clone(),
                &coldkey.clone(),
                *netuid,
                added_tao,
            );

            // --- 9. Swap the locks.
            if Locks::<T>::contains_key((origin_netuid, &origin_hotkey.clone(), coldkey.clone())) {
                let lock_data =
                    Locks::<T>::take((origin_netuid, origin_hotkey.clone(), coldkey.clone()));
                Locks::<T>::insert(
                    (netuid, destination_hotkey.clone(), coldkey.clone()),
                    lock_data,
                );
            }

            // --- 10. Log the event.
            log::info!(
				"StakeMoved( coldkey:{:?}, origin_hotkey:{:?}, origin_netuid:{:?}, destination_hotkey:{:?}, destination_netuid:{:?} )",
				coldkey.clone(),
				origin_hotkey.clone(),
				origin_netuid,
				destination_hotkey.clone(),
				*netuid
			);
            Self::deposit_event(Event::StakeMoved(
                coldkey.clone(),
                origin_hotkey.clone(),
                origin_netuid.clone(),
                destination_hotkey.clone(),
                *netuid,
            ));
        }

        // -- 11. Ok and return.
        Ok(())
    }
}
