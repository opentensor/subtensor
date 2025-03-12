use super::*;
use safe_math::*;
use sp_core::Get;
use substrate_fixed::types::{I96F32, U64F64};

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
            None,
            None,
            false,
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
    pub fn toggle_transfer(netuid: u16, toggle: bool) -> dispatch::DispatchResult {
        TransferToggle::<T>::insert(netuid, toggle);
        log::debug!(
            "TransferToggle( netuid: {:?}, toggle: {:?} ) ",
            netuid,
            toggle
        );
        Self::deposit_event(Event::TransferToggle(netuid, toggle));
        Ok(())
    }
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
            None,
            None,
            true,
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
            None,
            None,
            false,
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

    /// Swaps a specified amount of stake for the same `(coldkey, hotkey)` pair from one subnet
    /// (`origin_netuid`) to another (`destination_netuid`).
    ///
    /// # Arguments
    /// * `origin` - The origin of the transaction, which must be signed by the coldkey that owns the hotkey.
    /// * `hotkey` - The hotkey whose stake is being swapped.
    /// * `origin_netuid` - The subnet ID from which stake is removed.
    /// * `destination_netuid` - The subnet ID to which stake is added.
    /// * `alpha_amount` - The amount of stake to swap.
    /// * `limit_price` - The limit price.
    /// * `allow_partial` - Allow partial execution
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
    pub fn do_swap_stake_limit(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        origin_netuid: u16,
        destination_netuid: u16,
        alpha_amount: u64,
        limit_price: u64,
        allow_partial: bool,
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
            Some(limit_price),
            Some(allow_partial),
            false,
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

    // If limit_price is None, this is a regular operation, otherwise, it is slippage-protected
    // by setting limit price between origin_netuid and destination_netuid token
    fn transition_stake_internal(
        origin_coldkey: &T::AccountId,
        destination_coldkey: &T::AccountId,
        origin_hotkey: &T::AccountId,
        destination_hotkey: &T::AccountId,
        origin_netuid: u16,
        destination_netuid: u16,
        alpha_amount: u64,
        maybe_limit_price: Option<u64>,
        maybe_allow_partial: Option<bool>,
        check_transfer_toggle: bool,
    ) -> Result<u64, Error<T>> {
        // Calculate the maximum amount that can be executed
        let max_amount = if let Some(limit_price) = maybe_limit_price {
            Self::get_max_amount_move(origin_netuid, destination_netuid, limit_price)
        } else {
            alpha_amount
        };

        // Validate user input
        Self::validate_stake_transition(
            origin_coldkey,
            destination_coldkey,
            origin_hotkey,
            destination_hotkey,
            origin_netuid,
            destination_netuid,
            alpha_amount,
            max_amount,
            maybe_allow_partial,
            check_transfer_toggle,
        )?;

        // Calculate the amount that should be moved in this operation
        let move_amount = if alpha_amount < max_amount {
            alpha_amount
        } else {
            max_amount
        };

        // Unstake from the origin subnet, returning TAO (or a 1:1 equivalent).
        let fee = Self::calculate_staking_fee(
            origin_netuid,
            origin_hotkey,
            I96F32::saturating_from_num(alpha_amount),
        )
        .safe_div(2);

        let tao_unstaked = Self::unstake_from_subnet(
            origin_hotkey,
            origin_coldkey,
            origin_netuid,
            move_amount,
            fee,
        );

        // Stake the unstaked amount into the destination.
        // Because of the fee, the tao_unstaked may be too low if initial stake is low. In that case,
        // do not restake.
        if tao_unstaked >= DefaultMinStake::<T>::get().saturating_add(fee) {
            // If the coldkey is not the owner, make the hotkey a delegate.
            if Self::get_owning_coldkey_for_hotkey(destination_hotkey) != *destination_coldkey {
                Self::maybe_become_delegate(destination_hotkey);
            }

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

    /// Returns the maximum amount of origin netuid Alpha that can be executed before we cross
    /// limit_price.
    ///
    /// ```ignore
    /// The TAO we get from unstaking is
    ///     unstaked_tao = subnet_tao(1) - alpha_in(1) * subnet_tao(1) / (alpha_in(1) + unstaked_alpha)
    ///
    /// The Alpha we get from staking is
    ///     moved_alpha = alpha_in(2) - alpha_in(2) * subnet_tao(2) / (subnet_tao(2) + unstaked_tao)
    ///
    /// The resulting swap price that shall be compared to limit_price is moved_alpha / unstaked_alpha
    ///
    /// With a known limit_price parameter x = unstaked_alpha can be found using the formula:
    ///
    ///     alpha_in(2) * subnet_tao(1) - limit_price * alpha_in(1) * subnet_tao(2)
    /// x = -----------------------------------------------------------------------
    ///              limit_price * (subnet_tao(1) + subnet_tao(2))
    /// ```
    ///
    /// In the corner case when SubnetTAO(2) == SubnetTAO(1), no slippage is going to occur.
    ///
    pub fn get_max_amount_move(
        origin_netuid: u16,
        destination_netuid: u16,
        limit_price: u64,
    ) -> u64 {
        let tao: U64F64 = U64F64::saturating_from_num(1_000_000_000);

        // Corner case: both subnet IDs are root or stao
        // There's no slippage for root or stable subnets, so slippage is always 0.
        // The price always stays at 1.0, return 0 if price is expected to raise.
        if ((origin_netuid == Self::get_root_netuid())
            || (SubnetMechanism::<T>::get(origin_netuid)) == 0)
            && ((destination_netuid == Self::get_root_netuid())
                || (SubnetMechanism::<T>::get(destination_netuid)) == 0)
        {
            if limit_price > tao.saturating_to_num::<u64>() {
                return 0;
            } else {
                return u64::MAX;
            }
        }

        // Corner case: Origin is root or stable, destination is dynamic
        // Same as adding stake with limit price
        if ((origin_netuid == Self::get_root_netuid())
            || (SubnetMechanism::<T>::get(origin_netuid)) == 0)
            && ((SubnetMechanism::<T>::get(destination_netuid)) == 1)
        {
            if limit_price == 0 {
                return u64::MAX;
            } else {
                // The destination price is reverted because the limit_price is origin_price / destination_price
                let destination_subnet_price = tao
                    .safe_div(U64F64::saturating_from_num(limit_price))
                    .saturating_mul(tao)
                    .saturating_to_num::<u64>();
                return Self::get_max_amount_add(destination_netuid, destination_subnet_price);
            }
        }

        // Corner case: Origin is dynamic, destination is root or stable
        // Same as removing stake with limit price
        if ((destination_netuid == Self::get_root_netuid())
            || (SubnetMechanism::<T>::get(destination_netuid)) == 0)
            && ((SubnetMechanism::<T>::get(origin_netuid)) == 1)
        {
            return Self::get_max_amount_remove(origin_netuid, limit_price);
        }

        // Corner case: SubnetTAO for any of two subnets is zero
        let subnet_tao_1 = SubnetTAO::<T>::get(origin_netuid);
        let subnet_tao_2 = SubnetTAO::<T>::get(destination_netuid);
        if (subnet_tao_1 == 0) || (subnet_tao_2 == 0) {
            return 0;
        }
        let subnet_tao_1_float: U64F64 = U64F64::saturating_from_num(subnet_tao_1);
        let subnet_tao_2_float: U64F64 = U64F64::saturating_from_num(subnet_tao_2);

        // Corner case: SubnetAlphaIn for any of two subnets is zero
        let alpha_in_1 = SubnetAlphaIn::<T>::get(origin_netuid);
        let alpha_in_2 = SubnetAlphaIn::<T>::get(destination_netuid);
        if (alpha_in_1 == 0) || (alpha_in_2 == 0) {
            return 0;
        }
        let alpha_in_1_float: U64F64 = U64F64::saturating_from_num(alpha_in_1);
        let alpha_in_2_float: U64F64 = U64F64::saturating_from_num(alpha_in_2);

        // Corner case: limit_price > current_price (price of origin (as a base) relative
        // to destination (as a quote) cannot increase with moving)
        // The alpha price is never zero at this point because of the checks above.
        // Excluding this corner case guarantees that main case nominator is non-negative
        let limit_price_float: U64F64 = U64F64::saturating_from_num(limit_price)
            .checked_div(U64F64::saturating_from_num(1_000_000_000))
            .unwrap_or(U64F64::saturating_from_num(0));
        let current_price = Self::get_alpha_price(origin_netuid)
            .safe_div(Self::get_alpha_price(destination_netuid));
        if limit_price_float > current_price {
            return 0;
        }

        // Corner case: limit_price is zero
        if limit_price == 0 {
            return u64::MAX;
        }

        // Main case
        // Nominator is positive
        // Denominator is positive
        // Perform calculation in a non-overflowing order
        let tao_sum: U64F64 =
            U64F64::saturating_from_num(subnet_tao_2_float.saturating_add(subnet_tao_1_float));
        let t1_over_sum: U64F64 = subnet_tao_1_float.safe_div(tao_sum);
        let t2_over_sum: U64F64 = subnet_tao_2_float.safe_div(tao_sum);

        alpha_in_2_float
            .saturating_mul(t1_over_sum)
            .safe_div(limit_price_float)
            .saturating_sub(alpha_in_1_float.saturating_mul(t2_over_sum))
            .saturating_to_num::<u64>()
    }
}
