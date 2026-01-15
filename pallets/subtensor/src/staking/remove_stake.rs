use super::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::{Order, SwapHandler};

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
    /// * 'alpha_unstaked' (Alpha):
    ///     -  The amount of stake to be removed from the staking account.
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
        alpha_unstaked: AlphaCurrency,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_remove_stake( origin:{coldkey:?} hotkey:{hotkey:?}, netuid: {netuid:?}, alpha_unstaked:{alpha_unstaked:?} )"
        );

        Self::ensure_subtoken_enabled(netuid)?;

        // 1.1. Cap the alpha_unstaked at available Alpha because user might be paying transaxtion fees
        // in Alpha and their total is already reduced by now.
        let alpha_available =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let alpha_unstaked = alpha_unstaked.min(alpha_available);

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
        let tao_unstaked = Self::unstake_from_subnet(
            &hotkey,
            &coldkey,
            netuid,
            alpha_unstaked,
            T::SwapInterface::min_price(),
            false,
        )?;

        // 4. We add the balance to the coldkey. If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked.into());

        // 5. If the stake is below the minimum, we clear the nomination from storage.
        Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid);

        // 6. Check if stake lowered below MinStake and remove Pending children if it did
        if Self::get_total_stake_for_hotkey(&hotkey) < StakeThreshold::<T>::get().into() {
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
        log::debug!("do_unstake_all( origin:{coldkey:?} hotkey:{hotkey:?} )");

        // 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // 3. Get all netuids.
        let netuids = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {netuids:?}");

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

            if !alpha_unstaked.is_zero() {
                // Swap the alpha to tao and update counters for this subnet.
                let tao_unstaked = Self::unstake_from_subnet(
                    &hotkey,
                    &coldkey,
                    netuid,
                    alpha_unstaked,
                    T::SwapInterface::min_price(),
                    false,
                )?;

                // Add the balance to the coldkey. If the above fails we will not credit this coldkey.
                Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked.into());

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
        log::debug!("do_unstake_all( origin:{coldkey:?} hotkey:{hotkey:?} )");

        // 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // 3. Get all netuids.
        let netuids = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {netuids:?}");

        // 4. Iterate through all subnets and remove stake.
        let mut total_tao_unstaked = TaoCurrency::ZERO;
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

                if !alpha_unstaked.is_zero() {
                    // Swap the alpha to tao and update counters for this subnet.
                    let tao_unstaked = Self::unstake_from_subnet(
                        &hotkey,
                        &coldkey,
                        netuid,
                        alpha_unstaked,
                        T::SwapInterface::min_price(),
                        false,
                    )?;

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
            T::SwapInterface::max_price(),
            false, // no limit for Root subnet
            false,
        )?;

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
        alpha_unstaked: AlphaCurrency,
        limit_price: TaoCurrency,
        allow_partial: bool,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_remove_stake( origin:{coldkey:?} hotkey:{hotkey:?}, netuid: {netuid:?}, alpha_unstaked:{alpha_unstaked:?} )"
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
        let tao_unstaked = Self::unstake_from_subnet(
            &hotkey,
            &coldkey,
            netuid,
            possible_alpha,
            limit_price,
            false,
        )?;

        // 5. We add the balance to the coldkey. If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked.into());

        // 6. If the stake is below the minimum, we clear the nomination from storage.
        Self::clear_small_nomination_if_required(&hotkey, &coldkey, netuid);

        // 7. Check if stake lowered below MinStake and remove Pending children if it did
        if Self::get_total_stake_for_hotkey(&hotkey) < StakeThreshold::<T>::get().into() {
            Self::get_all_subnet_netuids().iter().for_each(|netuid| {
                PendingChildKeys::<T>::remove(netuid, &hotkey);
            })
        }

        // Done and ok.
        Ok(())
    }

    // Returns the maximum amount of RAO that can be executed with price limit
    pub fn get_max_amount_remove(
        netuid: NetUid,
        limit_price: TaoCurrency,
    ) -> Result<AlphaCurrency, DispatchError> {
        // Corner case: root and stao
        // There's no slippage for root or stable subnets, so if limit price is 1e9 rao or
        // lower, then max_amount equals u64::MAX, otherwise it is 0.
        if netuid.is_root() || SubnetMechanism::<T>::get(netuid) == 0 {
            if limit_price <= 1_000_000_000.into() {
                return Ok(AlphaCurrency::MAX);
            } else {
                return Err(Error::<T>::ZeroMaxStakeAmount.into());
            }
        }

        // Use reverting swap to estimate max limit amount
        let order = GetTaoForAlpha::<T>::with_amount(u64::MAX);
        let result = T::SwapInterface::swap(netuid.into(), order, limit_price.into(), false, true)
            .map(|r| r.amount_paid_in.saturating_add(r.fee_paid))?;

        if !result.is_zero() {
            Ok(result)
        } else {
            Err(Error::<T>::ZeroMaxStakeAmount.into())
        }
    }

    pub fn do_remove_stake_full_limit(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: NetUid,
        limit_price: Option<TaoCurrency>,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin.clone())?;

        let alpha_unstaked =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        if let Some(limit_price) = limit_price {
            Self::do_remove_stake_limit(origin, hotkey, netuid, alpha_unstaked, limit_price, false)
        } else {
            Self::do_remove_stake(origin, hotkey, netuid, alpha_unstaked)
        }
    }

    pub fn destroy_alpha_in_out_stakes(netuid: NetUid) -> DispatchResult {
        // 1) Ensure the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // 2) Enumerate all α entries on this subnet to build distribution weights and cleanup lists.
        //    - collect keys to remove,
        //    - per (hot,cold) α VALUE (not shares) with fallback to raw share if pool uninitialized,
        //    - track hotkeys to clear pool totals.
        let mut keys_to_remove: Vec<(T::AccountId, T::AccountId)> = Vec::new();
        let mut hotkeys_seen: Vec<T::AccountId> = Vec::new();
        let mut stakers: Vec<(T::AccountId, T::AccountId, u128)> = Vec::new();
        let mut total_alpha_value_u128: u128 = 0;

        let hotkeys_in_subnet: Vec<T::AccountId> = TotalHotkeyAlpha::<T>::iter()
            .filter(|(_, this_netuid, _)| *this_netuid == netuid)
            .map(|(hot, _, _)| hot.clone())
            .collect::<Vec<_>>();

        for hot in hotkeys_in_subnet.iter() {
            for ((cold, this_netuid), share_u64f64) in Alpha::<T>::iter_prefix((hot,)) {
                if this_netuid != netuid {
                    continue;
                }
                keys_to_remove.push((hot.clone(), cold.clone()));
                if !hotkeys_seen.contains(hot) {
                    hotkeys_seen.push(hot.clone());
                }

                // Primary: actual α value via share pool.
                let pool = Self::get_alpha_share_pool(hot.clone(), netuid);
                let actual_val_u64 = pool.try_get_value(&cold).unwrap_or(0);

                // Fallback: if pool uninitialized, treat raw Alpha share as value.
                let val_u64 = if actual_val_u64 == 0 {
                    share_u64f64.saturating_to_num::<u64>()
                } else {
                    actual_val_u64
                };

                if val_u64 > 0 {
                    let val_u128 = val_u64 as u128;
                    total_alpha_value_u128 = total_alpha_value_u128.saturating_add(val_u128);
                    stakers.push((hot.clone(), cold, val_u128));
                }
            }
        }

        // 5) Determine the TAO pot and pre-adjust accounting to avoid double counting.
        let pot_tao: TaoCurrency = SubnetTAO::<T>::get(netuid);
        let pot_u64: u64 = pot_tao.into();

        if pot_u64 > 0 {
            SubnetTAO::<T>::remove(netuid);
            TotalStake::<T>::mutate(|total| *total = total.saturating_sub(pot_tao));
        }

        // 6) Pro‑rata distribution of the pot by α value (largest‑remainder),
        //    **credited directly to each staker's COLDKEY free balance**.
        if pot_u64 > 0 && total_alpha_value_u128 > 0 && !stakers.is_empty() {
            struct Portion<A, C> {
                _hot: A,
                cold: C,
                share: u64, // TAO to credit to coldkey balance
                rem: u128,  // remainder for largest‑remainder method
            }

            let pot_u128: u128 = pot_u64 as u128;
            let mut portions: Vec<Portion<_, _>> = Vec::with_capacity(stakers.len());
            let mut distributed: u128 = 0;

            for (hot, cold, alpha_val) in &stakers {
                let prod: u128 = pot_u128.saturating_mul(*alpha_val);
                let share_u128: u128 = prod.checked_div(total_alpha_value_u128).unwrap_or_default();
                let share_u64: u64 = share_u128.min(u128::from(u64::MAX)) as u64;
                distributed = distributed.saturating_add(u128::from(share_u64));

                let rem: u128 = prod.checked_rem(total_alpha_value_u128).unwrap_or_default();
                portions.push(Portion {
                    _hot: hot.clone(),
                    cold: cold.clone(),
                    share: share_u64,
                    rem,
                });
            }

            let leftover: u128 = pot_u128.saturating_sub(distributed);
            if leftover > 0 {
                portions.sort_by(|a, b| b.rem.cmp(&a.rem));
                let give: usize = core::cmp::min(leftover, portions.len() as u128) as usize;
                for p in portions.iter_mut().take(give) {
                    p.share = p.share.saturating_add(1);
                }
            }

            // Credit each share directly to coldkey free balance.
            for p in portions {
                if p.share > 0 {
                    Self::add_balance_to_coldkey_account(&p.cold, p.share);
                }
            }
        }

        // 7) Destroy all α-in/α-out state for this subnet.
        // 7.a) Remove every (hot, cold, netuid) α entry.
        for (hot, cold) in keys_to_remove {
            Alpha::<T>::remove((hot, cold, netuid));
        }
        // 7.b) Clear share‑pool totals for each hotkey on this subnet.
        for hot in hotkeys_seen {
            TotalHotkeyAlpha::<T>::remove(&hot, netuid);
            TotalHotkeyShares::<T>::remove(&hot, netuid);
        }
        // 7.c) Remove α‑in/α‑out counters (fully destroyed).
        SubnetAlphaIn::<T>::remove(netuid);
        SubnetAlphaInProvided::<T>::remove(netuid);
        SubnetAlphaOut::<T>::remove(netuid);

        // Clear the locked balance on the subnet.
        Self::set_subnet_locked_balance(netuid, TaoCurrency::ZERO);

        // 8) Finalize lock handling: refunds are no longer issued.

        Ok(())
    }
}
