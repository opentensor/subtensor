use super::*;
use sp_core::Get;

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
        let origin_alpha = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
            &origin_hotkey,
            &coldkey,
            origin_netuid,
        );
        ensure!(
            alpha_amount <= origin_alpha,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // --- 7. Unstake the amount of alpha from the origin subnet, converting it to TAO
        let fee = DefaultMinStake::<T>::get().saturating_div(2); // fee is half of min stake because it is applied twice
        let origin_tao = Self::unstake_from_subnet(
            &origin_hotkey.clone(),
            &coldkey.clone(),
            origin_netuid,
            alpha_amount,
            fee,
        );

        // Ensure origin_tao is at least DefaultMinStake
        ensure!(
            origin_tao >= DefaultMinStake::<T>::get(),
            Error::<T>::AmountTooLow
        );

        // --- 8. Stake the resulting TAO into the destination subnet for the destination hotkey
        Self::stake_into_subnet(
            &destination_hotkey.clone(),
            &coldkey.clone(),
            destination_netuid,
            origin_tao,
            fee,
        );

        // --- 9. Log the event.
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
            origin_tao.saturating_sub(fee),
        ));

        // -- 10. Ok and return.
        Ok(())
    }

    /// Transfers stake from one coldkey to another, optionally moving from one subnet to another,
    /// while keeping the same hotkey.
    ///
    /// # Arguments
    /// * `origin` - The origin of the transaction, which must be signed by the `origin_coldkey`.
    /// * `destination_coldkey` - The account ID of the coldkey to which the stake is being transferred.
    /// * `hotkey` - The account ID of the hotkey associated with this stake.
    /// * `origin_netuid` - The network ID (subnet) from which the stake is being transferred.
    /// * `destination_netuid` - The network ID (subnet) to which the stake is being transferred.
    /// * `alpha_amount` - The amount of stake to transfer.
    ///
    /// # Returns
    /// * `DispatchResult` - Indicates success or failure.
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The transaction is not signed by the `origin_coldkey`.
    /// * The subnet (`origin_netuid` or `destination_netuid`) does not exist.
    /// * The `hotkey` does not exist.
    /// * The `(origin_coldkey, hotkey, origin_netuid)` does not have enough stake for `alpha_amount`.
    /// * The amount to be transferred is below the minimum stake requirement.
    /// * There is a failure in staking or unstaking logic.
    ///
    /// # Events
    /// Emits a `StakeTransferred` event upon successful completion of the transfer.
    pub fn do_transfer_stake(
        origin: T::RuntimeOrigin,
        destination_coldkey: T::AccountId,
        hotkey: T::AccountId,
        origin_netuid: u16,
        destination_netuid: u16,
        alpha_amount: u64,
    ) -> dispatch::DispatchResult {
        // 1. Ensure the extrinsic is signed by the origin_coldkey.
        let coldkey = ensure_signed(origin)?;

        // 2. Ensure both subnets exist.
        ensure!(
            Self::if_subnet_exist(origin_netuid),
            Error::<T>::SubnetNotExists
        );
        ensure!(
            Self::if_subnet_exist(destination_netuid),
            Error::<T>::SubnetNotExists
        );

        // 3. Check that the hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // 4. Check that the signed coldkey actually owns the given hotkey.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // 5. Get current stake.
        let origin_alpha =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, origin_netuid);
        ensure!(
            alpha_amount <= origin_alpha,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // 6. Unstake from the origin coldkey; this returns an amount of TAO.
        let fee = DefaultMinStake::<T>::get().saturating_div(2);
        let origin_tao =
            Self::unstake_from_subnet(&hotkey, &coldkey, origin_netuid, alpha_amount, fee);

        // 7. Ensure the returned TAO meets a minimum stake requirement (if required).
        ensure!(
            origin_tao >= DefaultMinStake::<T>::get(),
            Error::<T>::AmountTooLow
        );

        // 8. Stake the TAO into `(destination_coldkey, hotkey)` on the destination subnet.
        //    Create the account if it does not exist.
        Self::stake_into_subnet(
            &hotkey,
            &destination_coldkey,
            destination_netuid,
            origin_tao,
            fee,
        );

        // 9. Emit an event for logging/monitoring.
        log::info!(
            "StakeTransferred(origin_coldkey: {:?}, destination_coldkey: {:?}, hotkey: {:?}, origin_netuid: {:?}, destination_netuid: {:?}, amount: {:?})",
            coldkey,
            destination_coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            origin_tao
        );
        Self::deposit_event(Event::StakeTransferred(
            coldkey,
            destination_coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            origin_tao,
        ));

        // 10. Return success.
        Ok(())
    }
}
