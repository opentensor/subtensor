use super::*;
use safe_math::*;
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
        // Check that the origin is signed by the origin_hotkey.
        let coldkey = ensure_signed(origin)?;

        // Validate input and move stake
        let tao_moved = Self::transition_stake_internal(
            &coldkey,
            &coldkey,
            &origin_hotkey,
            &destination_hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount,
        )?;

        // Log the event.
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
            tao_moved,
        ));

        // Ok and return.
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
        // Ensure the extrinsic is signed by the origin_coldkey.
        let coldkey = ensure_signed(origin)?;

        // Validate input and move stake
        let tao_moved = Self::transition_stake_internal(
            &coldkey,
            &destination_coldkey,
            &hotkey,
            &hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount,
        )?;

        // 9. Emit an event for logging/monitoring.
        log::info!(
            "StakeTransferred(origin_coldkey: {:?}, destination_coldkey: {:?}, hotkey: {:?}, origin_netuid: {:?}, destination_netuid: {:?}, amount: {:?})",
            coldkey,
            destination_coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            tao_moved
        );
        Self::deposit_event(Event::StakeTransferred(
            coldkey,
            destination_coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            tao_moved,
        ));

        // 10. Return success.
        Ok(())
    }

    /// Swaps a specified amount of stake for the same `(coldkey, hotkey)` pair from one subnet
    /// (`origin_netuid`) to another (`destination_netuid`).
    ///
    /// # Arguments
    /// * `origin` - The origin of the transaction, which must be signed by the coldkey that owns the hotkey.
    /// * `hotkey` - The hotkey whose stake is being swapped.
    /// * `origin_netuid` - The subnet ID from which stake is removed.
    /// * `destination_netuid` - The subnet ID to which stake is added.
    /// * `alpha_amount` - The amount of stake to swap.
    ///
    /// # Returns
    /// * `DispatchResult` - Indicates success or failure.
    ///
    /// # Errors
    /// This function returns an error if:
    /// * The origin is not signed by the correct coldkey (i.e., not associated with `hotkey`).
    /// * Either the `origin_netuid` or the `destination_netuid` does not exist.
    /// * The specified `hotkey` does not exist.
    /// * The `(coldkey, hotkey, origin_netuid)` does not have enough stake (`alpha_amount`).
    /// * The unstaked amount is below `DefaultMinStake`.
    ///
    /// # Events
    /// Emits a `StakeSwapped` event upon successful completion.
    pub fn do_swap_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        origin_netuid: u16,
        destination_netuid: u16,
        alpha_amount: u64,
    ) -> dispatch::DispatchResult {
        // Ensure the extrinsic is signed by the coldkey.
        let coldkey = ensure_signed(origin)?;

        // Validate input and move stake
        let tao_moved = Self::transition_stake_internal(
            &coldkey,
            &coldkey,
            &hotkey,
            &hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount,
        )?;

        // Emit an event for logging.
        log::info!(
            "StakeSwapped(coldkey: {:?}, hotkey: {:?}, origin_netuid: {:?}, destination_netuid: {:?}, amount: {:?})",
            coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            tao_moved
        );
        Self::deposit_event(Event::StakeSwapped(
            coldkey,
            hotkey,
            origin_netuid,
            destination_netuid,
            tao_moved,
        ));

        // 6. Return success.
        Ok(())
    }

    fn transition_stake_internal(
        origin_coldkey: &T::AccountId,
        destination_coldkey: &T::AccountId,
        origin_hotkey: &T::AccountId,
        destination_hotkey: &T::AccountId,
        origin_netuid: u16,
        destination_netuid: u16,
        alpha_amount: u64,
    ) -> Result<u64, Error<T>> {
        // Validate user input
        Self::validate_stake_transition(
            origin_coldkey,
            destination_coldkey,
            origin_hotkey,
            destination_hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount,
        )?;

        // Unstake from the origin subnet, returning TAO (or a 1:1 equivalent).
        let fee = DefaultStakingFee::<T>::get().safe_div(2);
        let tao_unstaked = Self::unstake_from_subnet(
            origin_hotkey,
            origin_coldkey,
            origin_netuid,
            alpha_amount,
            fee,
        );

        // Stake the unstaked amount into the destination.
        // Because of the fee, the tao_unstaked may be too low if initial stake is low. In that case,
        // do not restake.
        if tao_unstaked >= DefaultMinStake::<T>::get().saturating_add(fee) {
            Self::stake_into_subnet(
                destination_hotkey,
                destination_coldkey,
                destination_netuid,
                tao_unstaked,
                fee,
            );
        }

        Ok(tao_unstaked.saturating_sub(fee))
    }
}
