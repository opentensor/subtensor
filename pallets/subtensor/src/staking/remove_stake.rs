use super::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::NetUid;

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
    /// * 'netuid' (u16):
    ///     - Subnetwork UID
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
        netuid: NetUid,
        alpha_unstaked: u64,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, netuid: {:?}, alpha_unstaked:{:?} )",
            coldkey,
            hotkey,
            netuid,
            alpha_unstaked
        );

        Self::ensure_subtoken_enabled(netuid)?;

        // 2. Validate the user input
        Self::validate_remove_stake(
            &coldkey,
            &hotkey,
            netuid,
            alpha_unstaked,
            alpha_unstaked,
            false,
        )?;

        // 3. Swap the alpba to tao and update counters for this subnet.
        let fee = Self::calculate_staking_fee(
            Some((&hotkey, netuid)),
            &coldkey,
            None,
            &coldkey,
            U96F32::saturating_from_num(alpha_unstaked),
        );
        let tao_unstaked: u64 =
            Self::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_unstaked, fee);

        // 4. We add the balance to the coldkey. If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked);

        // 5. If the stake is below the minimum, we clear the nomination from storage.
        Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid);

        // 6. Check if stake lowered below MinStake and remove Pending children if it did
        if Self::get_total_stake_for_hotkey(&hotkey) < StakeThreshold::<T>::get() {
            Self::get_all_subnet_netuids().iter().for_each(|netuid| {
                PendingChildKeys::<T>::remove(netuid, &hotkey);
            })
        }

        // Done and ok.
        Ok(())
    }

    /// ---- The implementation for the extrinsic unstake_all: Removes all stake from a hotkey account across all subnets and adds it onto a coldkey.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     -  The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     -  The associated hotkey account.
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
    ///     -  Thrown if there is not enough stake on the hotkey to withdraw this amount.
    ///
    /// * 'TxRateLimitExceeded':
    ///     -  Thrown if key has hit transaction rate limit
    ///
    pub fn do_unstake_all(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!("do_unstake_all( origin:{:?} hotkey:{:?} )", coldkey, hotkey);

        // 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // 3. Get all netuids.
        let netuids = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", netuids);

        // 4. Iterate through all subnets and remove stake.
        for netuid in netuids.into_iter() {
            if !SubtokenEnabled::<T>::get(netuid) {
                continue;
            }
            // Ensure that the hotkey has enough stake to withdraw.
            let alpha_unstaked =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

            if Self::validate_remove_stake(
                &coldkey,
                &hotkey,
                netuid,
                alpha_unstaked,
                alpha_unstaked,
                false,
            )
            .is_err()
            {
                // Don't unstake from this netuid
                continue;
            }

            let fee = Self::calculate_staking_fee(
                Some((&hotkey, netuid)),
                &coldkey,
                None,
                &coldkey,
                U96F32::saturating_from_num(alpha_unstaked),
            );

            if alpha_unstaked > 0 {
                // Swap the alpha to tao and update counters for this subnet.
                let tao_unstaked: u64 =
                    Self::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_unstaked, fee);

                // Add the balance to the coldkey. If the above fails we will not credit this coldkey.
                Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked);

                // If the stake is below the minimum, we clear the nomination from storage.
                Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid);
            }
        }

        // 5. Done and ok.
        Ok(())
    }

    /// ---- The implementation for the extrinsic unstake_all: Removes all stake from a hotkey account across all subnets and adds it onto a coldkey.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     -  The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     -  The associated hotkey account.
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
    ///     -  Thrown if there is not enough stake on the hotkey to withdraw this amount.
    ///
    /// * 'TxRateLimitExceeded':
    ///     -  Thrown if key has hit transaction rate limit
    ///
    pub fn do_unstake_all_alpha(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!("do_unstake_all( origin:{:?} hotkey:{:?} )", coldkey, hotkey);

        // 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // 3. Get all netuids.
        let netuids = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", netuids);

        // 4. Iterate through all subnets and remove stake.
        let mut total_tao_unstaked: u64 = 0;
        for netuid in netuids.into_iter() {
            if !SubtokenEnabled::<T>::get(netuid) {
                continue;
            }
            // If not Root network.
            if !netuid.is_root() {
                // Ensure that the hotkey has enough stake to withdraw.
                let alpha_unstaked =
                    Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

                if Self::validate_remove_stake(
                    &coldkey,
                    &hotkey,
                    netuid,
                    alpha_unstaked,
                    alpha_unstaked,
                    false,
                )
                .is_err()
                {
                    // Don't unstake from this netuid
                    continue;
                }

                let fee = Self::calculate_staking_fee(
                    Some((&hotkey, netuid)),
                    &coldkey,
                    None,
                    &coldkey,
                    U96F32::saturating_from_num(alpha_unstaked),
                );

                if alpha_unstaked > 0 {
                    // Swap the alpha to tao and update counters for this subnet.
                    let tao_unstaked =
                        Self::unstake_from_subnet(&hotkey, &coldkey, netuid, alpha_unstaked, fee);

                    // Increment total
                    total_tao_unstaked = total_tao_unstaked.saturating_add(tao_unstaked);

                    // If the stake is below the minimum, we clear the nomination from storage.
                    Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid);
                }
            }
        }

        // Stake into root.
        Self::stake_into_subnet(
            &hotkey,
            &coldkey,
            NetUid::ROOT,
            total_tao_unstaked,
            0, // no fee for restaking
        );

        // 5. Done and ok.
        Ok(())
    }

    /// ---- The implementation for the extrinsic remove_stake_limit: Removes stake from
    /// a hotkey on a subnet with a price limit.
    ///
    /// In case if slippage occurs and the price shall move beyond the limit
    /// price, the staking order may execute only partially or not execute
    /// at all.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>Origin):
    ///     - The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     - The associated hotkey account.
    ///
    /// * 'netuid' (u16):
    ///     - Subnetwork UID
    ///
    /// * 'amount_unstaked' (u64):
    ///     - The amount of stake to be added to the hotkey staking account.
    ///
    ///  * 'limit_price' (u64):
    ///     - The limit price expressed in units of RAO per one Alpha.
    ///
    ///  * 'allow_partial' (bool):
    ///     - Allows partial execution of the amount. If set to false, this becomes
    ///       fill or kill type or order.
    ///
    /// # Event:
    /// * StakeRemoved;
    ///     - On the successfully removing stake from the hotkey account.
    ///
    /// # Raises:
    /// * 'NotRegistered':
    ///     - Thrown if the account we are attempting to unstake from is non existent.
    ///
    /// * 'NonAssociatedColdKey':
    ///     - Thrown if the coldkey does not own the hotkey we are unstaking from.
    ///
    /// * 'NotEnoughStakeToWithdraw':
    ///     - Thrown if there is not enough stake on the hotkey to withdwraw this amount.
    ///
    pub fn do_remove_stake_limit(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: NetUid,
        alpha_unstaked: u64,
        limit_price: u64,
        allow_partial: bool,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, netuid: {:?}, alpha_unstaked:{:?} )",
            coldkey,
            hotkey,
            netuid,
            alpha_unstaked
        );

        // 2. Calculate the maximum amount that can be executed with price limit
        let max_amount = Self::get_max_amount_remove(netuid, limit_price)?;
        let mut possible_alpha = alpha_unstaked;
        if possible_alpha > max_amount {
            possible_alpha = max_amount;
        }

        // 3. Validate the user input
        Self::validate_remove_stake(
            &coldkey,
            &hotkey,
            netuid,
            alpha_unstaked,
            max_amount,
            allow_partial,
        )?;

        // 4. Swap the alpha to tao and update counters for this subnet.
        let fee = Self::calculate_staking_fee(
            Some((&hotkey, netuid)),
            &coldkey,
            None,
            &coldkey,
            U96F32::saturating_from_num(alpha_unstaked),
        );
        let tao_unstaked =
            Self::unstake_from_subnet(&hotkey, &coldkey, netuid, possible_alpha, fee);

        // 5. We add the balance to the coldkey. If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked);

        // 6. If the stake is below the minimum, we clear the nomination from storage.
        Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid);

        // 7. Check if stake lowered below MinStake and remove Pending children if it did
        if Self::get_total_stake_for_hotkey(&hotkey) < StakeThreshold::<T>::get() {
            Self::get_all_subnet_netuids().iter().for_each(|netuid| {
                PendingChildKeys::<T>::remove(netuid, &hotkey);
            })
        }

        // Done and ok.
        Ok(())
    }

    // Returns the maximum amount of RAO that can be executed with price limit
    pub fn get_max_amount_remove(netuid: NetUid, limit_price: u64) -> Result<u64, Error<T>> {
        // Corner case: root and stao
        // There's no slippage for root or stable subnets, so if limit price is 1e9 rao or
        // lower, then max_amount equals u64::MAX, otherwise it is 0.
        if netuid.is_root() || SubnetMechanism::<T>::get(netuid) == 0 {
            if limit_price <= 1_000_000_000 {
                return Ok(u64::MAX);
            } else {
                return Err(Error::ZeroMaxStakeAmount);
            }
        }

        // Corner case: SubnetAlphaIn is zero. Staking can't happen, so max amount is zero.
        let alpha_in = SubnetAlphaIn::<T>::get(netuid);
        if alpha_in == 0 {
            return Err(Error::ZeroMaxStakeAmount);
        }
        let alpha_in_u128 = alpha_in as u128;

        // Corner case: SubnetTAO is zero. Staking can't happen, so max amount is zero.
        let tao_reserve = SubnetTAO::<T>::get(netuid);
        if tao_reserve == 0 {
            return Err(Error::ZeroMaxStakeAmount);
        }
        let tao_reserve_u128 = tao_reserve as u128;

        // Corner case: limit_price == 0 (because there's division by limit price)
        // => can sell all
        if limit_price == 0 {
            return Ok(u64::MAX);
        }

        // Corner case: limit_price >= current_price (price cannot increase with unstaking)
        // No overflows: alpha_price * tao <= u64::MAX * u64::MAX
        // Alpha price is U96F32 size, but it is calculated as u64/u64, so it never uses all 96 bits.
        let limit_price_u128 = limit_price as u128;
        let tao = 1_000_000_000_u128;
        if limit_price_u128
            >= tao_reserve_u128
                .saturating_mul(tao)
                .checked_div(alpha_in_u128)
                .unwrap_or(0)
        {
            return Err(Error::ZeroMaxStakeAmount);
        }

        // Main case: SubnetTAO / limit_price - SubnetAlphaIn
        // Non overflowing calculation: tao_reserve * tao <= u64::MAX * u64::MAX <= u128::MAX
        // May overflow result, then it will be capped at u64::MAX, which is OK because that matches Alpha u64 size.
        let result = tao_reserve_u128
            .saturating_mul(tao)
            .checked_div(limit_price_u128)
            .unwrap_or(0)
            .saturating_sub(alpha_in_u128);

        if result < u64::MAX as u128 {
            if result == 0 {
                return Err(Error::ZeroMaxStakeAmount);
            }

            Ok(result as u64)
        } else {
            Ok(u64::MAX)
        }
    }

    pub fn do_remove_stake_full_limit(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: NetUid,
        limit_price: u64,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin.clone())?;

        let alpha_unstaked =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        Self::do_remove_stake_limit(origin, hotkey, netuid, alpha_unstaked, limit_price, false)
    }
}
