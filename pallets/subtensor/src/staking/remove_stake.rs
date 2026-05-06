use super::*;
use frame_support::weights::WeightMeter;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};
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
        origin: OriginFor<T>,
        hotkey: T::AccountId,
        netuid: NetUid,
        alpha_unstaked: AlphaBalance,
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
        Self::unstake_from_subnet(
            &hotkey,
            &coldkey,
            &coldkey,
            netuid,
            alpha_unstaked,
            T::SwapInterface::min_price(),
            false,
        )?;

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
    pub fn do_unstake_all(origin: OriginFor<T>, hotkey: T::AccountId) -> dispatch::DispatchResult {
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
                Self::unstake_from_subnet(
                    &hotkey,
                    &coldkey,
                    &coldkey,
                    netuid,
                    alpha_unstaked,
                    T::SwapInterface::min_price(),
                    false,
                )?;

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
        origin: OriginFor<T>,
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
        let mut total_tao_unstaked = TaoBalance::ZERO;
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
        origin: OriginFor<T>,
        hotkey: T::AccountId,
        netuid: NetUid,
        alpha_unstaked: AlphaBalance,
        limit_price: TaoBalance,
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
        Self::unstake_from_subnet(
            &hotkey,
            &coldkey,
            &coldkey,
            netuid,
            possible_alpha,
            limit_price,
            false,
        )?;

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

    // Returns the maximum amount of RAO that can be executed with price limit
    pub fn get_max_amount_remove(
        netuid: NetUid,
        limit_price: TaoBalance,
    ) -> Result<AlphaBalance, DispatchError> {
        // Corner case: root and stao
        // There's no slippage for root or stable subnets, so if limit price is 1e9 rao or
        // lower, then max_amount equals u64::MAX, otherwise it is 0.
        if netuid.is_root() || SubnetMechanism::<T>::get(netuid) == 0 {
            if limit_price <= 1_000_000_000.into() {
                return Ok(AlphaBalance::MAX);
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
        origin: OriginFor<T>,
        hotkey: T::AccountId,
        netuid: NetUid,
        limit_price: Option<TaoBalance>,
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

    pub fn destroy_alpha_in_out_stakes(netuid: NetUid, remaining_weight: Weight) -> (Weight, bool) {
        // 1) Initialize the weight meter from the remaining weight.
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);

        LoopRemovePrefixWithWeightMeter!(
            weight_meter,
            T::DbWeight::get().writes(1),
            HotkeyLock<T>,
            netuid
        );

        // 2) Owner / lock cost.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
        let owner_coldkey: T::AccountId = SubnetOwner::<T>::get(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
        let lock_cost: TaoBalance = Self::get_subnet_locked_balance(netuid);

        // Determine if this subnet is eligible for a lock refund (legacy).
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
        let reg_at: u64 = NetworkRegisteredAt::<T>::get(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));

        let start_block: u64 = NetworkRegistrationStartBlock::<T>::get();
        let should_refund_owner: bool = reg_at < start_block;

        // 3) Compute owner's received emission in TAO at current price (ONLY if we may refund).
        // We:
        //      - get the current alpha issuance,
        //      - apply owner fraction to get owner α,
        //      - price that α using a *simulated* AMM swap.
        let mut owner_emission_tao = TaoBalance::ZERO;
        if should_refund_owner && !lock_cost.is_zero() {
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
            let total_emitted_alpha_u128: u128 = Self::get_alpha_issuance(netuid).to_u64() as u128;

            if total_emitted_alpha_u128 > 0 {
                WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(1));
                let owner_fraction: U96F32 = Self::get_float_subnet_owner_cut();
                let owner_alpha_u64 = U96F32::from_num(total_emitted_alpha_u128)
                    .saturating_mul(owner_fraction)
                    .floor()
                    .saturating_to_num::<u64>();

                owner_emission_tao = if owner_alpha_u64 > 0 {
                    // Need max 3 reads for current_alpha_price
                    WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads(3));
                    let cur_price: U96F32 = T::SwapInterface::current_alpha_price(netuid.into());
                    let val_u64 = U96F32::from_num(owner_alpha_u64)
                        .saturating_mul(cur_price)
                        .floor()
                        .saturating_to_num::<u64>();
                    val_u64.into()
                } else {
                    TaoBalance::ZERO
                };
            }
        }

        // 7.c) Remove α‑in/α‑out counters (fully destroyed).
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetAlphaIn::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetAlphaInProvided::<T>::remove(netuid);
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        SubnetAlphaOut::<T>::remove(netuid);

        // Clear the locked balance on the subnet.
        WeightMeterWrapper!(weight_meter, T::DbWeight::get().writes(1));
        Self::set_subnet_locked_balance(netuid, TaoBalance::ZERO);

        // 8) Finalize lock handling:
        //    - Legacy subnets (registered before NetworkRegistrationStartBlock) receive:
        //        refund = max(0, lock_cost(τ) − owner_received_emission_in_τ).
        //    - New subnets: no refund.
        let refund: TaoBalance = if should_refund_owner {
            lock_cost.saturating_sub(owner_emission_tao)
        } else {
            TaoBalance::ZERO
        };

        if !refund.is_zero() {
            WeightMeterWrapper!(weight_meter, T::DbWeight::get().reads_writes(1, 1));
            let _ = Self::transfer_tao_from_subnet(netuid, &owner_coldkey, refund);
        }

        (weight_meter.consumed(), true)
    }

    pub fn destroy_alpha_in_out_stakes_get_total_alpha_value(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        let mut read_all = true;
        let mut total_alpha_value_u128: u128 = 0;

        let iter = match LastKeptRawKey::<T>::get() {
            Some(key) => {
                if let Some(value) = DissolvedSubnetTotalAlphaValue::<T>::get() {
                    total_alpha_value_u128 = value;
                } else {
                    log::warn!("DissolvedSubnetTotalAlphaValue not set");
                }
                TotalHotkeyAlpha::<T>::iter_from(key)
            }
            None => TotalHotkeyAlpha::<T>::iter(),
        };

        for (hot, this_netuid, _) in iter {
            // let mut coldkeys: Vec<T::AccountId> = Vec::new();
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                    &hot,
                    this_netuid,
                )));
                break;
            }
            weight_meter.consume(r);

            if this_netuid != netuid {
                continue;
            }

            let mut iterate_all = true;
            for (cold, this_netuid, share_u64f64) in Self::alpha_iter_single_prefix(&hot) {
                if !weight_meter.can_consume(r) {
                    iterate_all = false;
                    LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                        &hot,
                        this_netuid,
                    )));
                    break;
                }
                weight_meter.consume(r);

                if this_netuid != netuid {
                    continue;
                }

                // Primary: actual α value via share pool.
                let pool = Self::get_alpha_share_pool(hot.clone(), netuid);
                let actual_val_u64 = pool.try_get_value(&cold).unwrap_or(0);

                // Fallback: if pool uninitialized, treat raw Alpha share as value.
                let val_u64 = if actual_val_u64 == 0 {
                    u64::from(share_u64f64)
                } else {
                    actual_val_u64
                };

                if val_u64 > 0 {
                    let val_u128 = val_u64 as u128;
                    total_alpha_value_u128 = total_alpha_value_u128.saturating_add(val_u128);
                }
            }

            if !iterate_all {
                read_all = false;
                break;
            }
        }

        //always update the status no matter if there is weight left or not
        DissolvedSubnetTotalAlphaValue::<T>::set(Some(total_alpha_value_u128));

        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        (weight_meter.consumed(), read_all)
    }

    pub fn destroy_alpha_in_out_stakes_settle_stakes(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        let mut read_all = true;

        let mut stakers: Vec<(T::AccountId, T::AccountId, u128)> = Vec::new();
        let total_alpha_value_u128: u128 = match DissolvedSubnetTotalAlphaValue::<T>::get() {
            Some(value) => value,
            None => {
                log::warn!("DissolvedSubnetTotalAlphaValue not set");
                return (weight_meter.consumed(), false);
            }
        };
        let mut settled_alpha_value_u128 =
            DissolvedSubnetSettledAlphaValue::<T>::get().unwrap_or(0);

        let mut hotkeys_in_subnet: Vec<T::AccountId> = Vec::new();

        let iter = match LastKeptRawKey::<T>::get() {
            Some(key) => TotalHotkeyAlpha::<T>::iter_from(key),
            None => TotalHotkeyAlpha::<T>::iter(),
        };

        for (hot, this_netuid, _) in iter {
            // let mut coldkeys: Vec<T::AccountId> = Vec::new();
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                    &hot,
                    this_netuid,
                )));
                break;
            }
            weight_meter.consume(r);

            if this_netuid != netuid {
                continue;
            }
            hotkeys_in_subnet.push(hot.clone());

            let mut iterate_all = true;

            for (cold, this_netuid, share_u64f64) in Self::alpha_iter_single_prefix(&hot) {
                if !weight_meter.can_consume(r.saturating_mul(2_u64)) {
                    iterate_all = false;
                    LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                        &hot,
                        this_netuid,
                    )));
                    break;
                }

                weight_meter.consume(r.saturating_mul(2_u64));

                if this_netuid != netuid {
                    continue;
                }

                // Primary: actual α value via share pool.
                let pool = Self::get_alpha_share_pool(hot.clone(), netuid);
                let actual_val_u64 = pool.try_get_value(&cold).unwrap_or(0);

                // Fallback: if pool uninitialized, treat raw Alpha share as value.
                let val_u64 = if actual_val_u64 == 0 {
                    u64::from(share_u64f64)
                } else {
                    actual_val_u64
                };

                if val_u64 > 0 {
                    // reserve the weight for the add_balance_to_coldkey_account function call later
                    if !weight_meter.can_consume(w) {
                        iterate_all = false;
                        LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                            &hot,
                            this_netuid,
                        )));
                        break;
                    }
                    weight_meter.consume(r.saturating_mul(2_u64));
                    let val_u128 = val_u64 as u128;
                    stakers.push((hot.clone(), cold, val_u128));
                }
            }

            if !iterate_all {
                read_all = false;
                break;
            }
        }

        // total TAO in the subnet pool
        let pot_tao: TaoBalance = SubnetTAO::<T>::get(netuid);
        let pot_u64: u64 = pot_tao.into();
        struct Portion<A, C> {
            _hot: A,
            cold: C,
            share: u64, // TAO to credit to coldkey balance
            rem: u128,  // remainder for largest‑remainder method
        }
        let mut portions: Vec<Portion<_, _>> = Vec::with_capacity(stakers.len());

        // 6) Pro‑rata distribution of the pot by α value (largest‑remainder),
        //    **credited directly to each staker's COLDKEY free balance**.
        if pot_u64 > 0 && total_alpha_value_u128 > 0 && !stakers.is_empty() {
            let pot_u128: u128 = pot_u64 as u128;

            let mut distributed: u128 = 0;
            let mut total_rem: u128 = 0;

            for (hot, cold, alpha_val) in &stakers {
                let prod: u128 = pot_u128.saturating_mul(*alpha_val);
                let share_u128: u128 = prod.checked_div(total_alpha_value_u128).unwrap_or_default();
                let share_u64: u64 = share_u128.min(u128::from(u64::MAX)) as u64;
                distributed = distributed.saturating_add(u128::from(share_u64));

                let rem: u128 = prod.checked_rem(total_alpha_value_u128).unwrap_or_default();
                total_rem = total_rem.saturating_add(rem);
                portions.push(Portion {
                    _hot: hot.clone(),
                    cold: cold.clone(),
                    share: share_u64,
                    rem,
                });
            }

            settled_alpha_value_u128 = settled_alpha_value_u128.saturating_add(distributed);

            let leftover: u128 = total_rem
                .checked_div(total_alpha_value_u128)
                .unwrap_or_default();
            if leftover > 0 {
                settled_alpha_value_u128 = settled_alpha_value_u128.saturating_add(leftover);
                portions.sort_by(|a, b| b.rem.cmp(&a.rem));
                let give: usize = core::cmp::min(leftover, portions.len() as u128) as usize;
                for p in portions.iter_mut().take(give) {
                    p.share = p.share.saturating_add(1);
                }
            }

            portions = portions
                .into_iter()
                .filter(|p| p.share > 0)
                .collect::<Vec<_>>();

            // Credit each share directly to coldkey free balance.
            for p in portions {
                if p.share > 0 {
                    // Cannot fail the whole transaction if this transfer fails
                    let _ = Self::transfer_tao_from_subnet(netuid, &p.cold, p.share.into());
                }
            }
        }

        // ignore the weight for handling the final operation, we must set the correct status for the next run
        if read_all {
            if settled_alpha_value_u128 < total_alpha_value_u128 {
                let final_leftover: u128 = total_alpha_value_u128
                    .saturating_sub(settled_alpha_value_u128)
                    .checked_div(total_alpha_value_u128)
                    .unwrap_or_default();

                if final_leftover > 0 {
                    let owner = SubnetOwner::<T>::get(netuid);
                    let _ = Self::transfer_tao_from_subnet(netuid, &owner, final_leftover.into());
                }

                DissolvedSubnetSettledAlphaValue::<T>::set(Some(settled_alpha_value_u128));
            } else {
                DissolvedSubnetSettledAlphaValue::<T>::set(None);
            }

            DissolvedSubnetTotalAlphaValue::<T>::set(None);
            LastKeptRawKey::<T>::set(None);
            DissolvedSubnetSettledAlphaValue::<T>::set(None);
            if pot_u64 > 0 {
                SubnetTAO::<T>::remove(netuid);
                TotalStake::<T>::mutate(|total| *total = total.saturating_sub(pot_tao));
            }
        } else {
            DissolvedSubnetSettledAlphaValue::<T>::set(Some(settled_alpha_value_u128));
        }

        (weight_meter.consumed(), read_all)
    }

    pub fn destroy_alpha_in_out_stakes_clean_alpha(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        let mut read_all = true;
        //    - track hotkeys to clear pool totals.

        let iter = match LastKeptRawKey::<T>::get() {
            Some(key) => TotalHotkeyAlpha::<T>::iter_from(key),
            None => TotalHotkeyAlpha::<T>::iter(),
        };

        for (hot, this_netuid, _) in iter {
            let mut coldkeys: Vec<T::AccountId> = Vec::new();
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                    &hot,
                    this_netuid,
                )));
                break;
            }
            weight_meter.consume(r);

            if this_netuid != netuid {
                continue;
            }

            let mut iterate_all = true;
            for (cold, this_netuid, _) in Self::alpha_iter_single_prefix(&hot) {
                if !weight_meter.can_consume(r) {
                    read_all = false;
                    LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                        &hot,
                        this_netuid,
                    )));
                    iterate_all = false;
                    break;
                }
                weight_meter.consume(r);
                if this_netuid != netuid {
                    continue;
                }
                coldkeys.push(cold.clone());
            }

            if !iterate_all {
                read_all = false;
                break;
            }

            let weight_for_all_remove = w.saturating_mul(coldkeys.len() as u64);

            if !weight_meter.can_consume(weight_for_all_remove) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(
                    &hot,
                    this_netuid,
                )));
                break;
            }
            weight_meter.consume(weight_for_all_remove);

            for cold in coldkeys {
                Alpha::<T>::remove((&hot, &cold, netuid));
                AlphaV2::<T>::remove((&hot, &cold, netuid));
            }
        }

        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        (weight_meter.consumed(), read_all)
    }

    pub fn destroy_alpha_in_out_stakes_clear_hotkey_totals(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        let mut read_all = true;
        let mut hotkeys_to_remove: Vec<T::AccountId> = Vec::new();

        let iter = match LastKeptRawKey::<T>::get() {
            Some(key) => TotalHotkeyAlpha::<T>::iter_from(key),
            None => TotalHotkeyAlpha::<T>::iter(),
        };

        // get all hotkeys in the subnet
        for (hotkey, nu, _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(&hotkey, nu)));
                break;
            }
            weight_meter.consume(r);
            if nu != netuid {
                continue;
            }

            let weight_for_all_remove = w.saturating_mul(3_u64);
            if !weight_meter.can_consume(weight_for_all_remove) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(TotalHotkeyAlpha::<T>::hashed_key_for(&hotkey, nu)));
                break;
            }
            weight_meter.consume(weight_for_all_remove);

            hotkeys_to_remove.push(hotkey.clone());
        }

        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        for hotkey in hotkeys_to_remove {
            TotalHotkeyAlpha::<T>::remove(&hotkey, netuid);
            TotalHotkeyShares::<T>::remove(&hotkey, netuid);
            TotalHotkeySharesV2::<T>::remove(&hotkey, netuid);
        }

        (weight_meter.consumed(), read_all)
    }

    pub fn destroy_alpha_in_out_stakes_clear_locks(
        netuid: NetUid,
        remaining_weight: Weight,
    ) -> (Weight, bool) {
        let r = T::DbWeight::get().reads(1);
        let w = T::DbWeight::get().writes(1);
        let mut keys_to_remove: Vec<(T::AccountId, T::AccountId)> = Vec::new();
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        let mut read_all = true;

        let iter = match LastKeptRawKey::<T>::get() {
            Some(key) => Lock::<T>::iter_from(key),
            None => Lock::<T>::iter(),
        };

        for ((coldkey, this_netuid, hotkey), _) in iter {
            if !weight_meter.can_consume(r) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(Lock::<T>::hashed_key_for((
                    &coldkey,
                    this_netuid,
                    &hotkey,
                ))));
                break;
            }
            weight_meter.consume(r);

            if this_netuid != netuid {
                continue;
            }

            if !weight_meter.can_consume(w) {
                read_all = false;
                LastKeptRawKey::<T>::set(Some(Lock::<T>::hashed_key_for((
                    &coldkey,
                    this_netuid,
                    &hotkey,
                ))));
                break;
            }
            weight_meter.consume(w);

            keys_to_remove.push((coldkey, hotkey));
        }

        for (coldkey, hotkey) in keys_to_remove {
            Lock::<T>::remove((coldkey, netuid, hotkey));
        }

        if read_all {
            LastKeptRawKey::<T>::set(None);
        }

        (weight_meter.consumed(), read_all)
    }
}
