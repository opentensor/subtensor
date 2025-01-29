use super::*;
use safe_math::*;
use sp_core::Get;
use substrate_fixed::types::U96F32;

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
        netuid: u16,
        alpha_unstaked: u64,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, netuid: {:?}, alpha_unstaked:{:?} )",
            coldkey,
            hotkey,
            netuid,
            alpha_unstaked
        );

        // 2. Validate the user input
        Self::validate_remove_stake(&coldkey, &hotkey, netuid, alpha_unstaked)?;

        // 3. Swap the alpba to tao and update counters for this subnet.
        let fee = DefaultStakingFee::<T>::get();
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
        let fee = DefaultStakingFee::<T>::get();

        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!("do_unstake_all( origin:{:?} hotkey:{:?} )", coldkey, hotkey);

        // 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // 3. Get all netuids.
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", netuids);

        // 4. Iterate through all subnets and remove stake.
        for netuid in netuids.iter() {
            // Ensure that the hotkey has enough stake to withdraw.
            let alpha_unstaked =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, *netuid);
            if alpha_unstaked > 0 {
                // Swap the alpha to tao and update counters for this subnet.
                let tao_unstaked: u64 =
                    Self::unstake_from_subnet(&hotkey, &coldkey, *netuid, alpha_unstaked, fee);

                // Add the balance to the coldkey. If the above fails we will not credit this coldkey.
                Self::add_balance_to_coldkey_account(&coldkey, tao_unstaked);

                // If the stake is below the minimum, we clear the nomination from storage.
                Self::clear_small_nomination_if_required(&hotkey, &coldkey, *netuid);
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
        let fee = DefaultStakingFee::<T>::get();

        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!("do_unstake_all( origin:{:?} hotkey:{:?} )", coldkey, hotkey);

        // 2. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // 3. Get all netuids.
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", netuids);

        // 4. Iterate through all subnets and remove stake.
        let mut total_tao_unstaked: u64 = 0;
        for netuid in netuids.iter() {
            // If not Root network.
            if *netuid != Self::get_root_netuid() {
                // Ensure that the hotkey has enough stake to withdraw.
                let alpha_unstaked =
                    Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, *netuid);
                if alpha_unstaked > 0 {
                    // Swap the alpha to tao and update counters for this subnet.
                    let tao_unstaked: u64 =
                        Self::unstake_from_subnet(&hotkey, &coldkey, *netuid, alpha_unstaked, fee);

                    // Increment total
                    total_tao_unstaked = total_tao_unstaked.saturating_add(tao_unstaked);

                    // If the stake is below the minimum, we clear the nomination from storage.
                    Self::clear_small_nomination_if_required(&hotkey, &coldkey, *netuid);
                }
            }
        }

        // Stake into root.
        Self::stake_into_subnet(
            &hotkey,
            &coldkey,
            Self::get_root_netuid(),
            total_tao_unstaked,
            0, // no fee for restaking
        );

        // 5. Done and ok.
        Ok(())
    }

    pub fn do_remove_stake_limit(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        alpha_unstaked: u64,
        limit_price: u64,
    ) -> dispatch::DispatchResult {
        // 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_remove_stake( origin:{:?} hotkey:{:?}, netuid: {:?}, alpha_unstaked:{:?} )",
            coldkey,
            hotkey,
            netuid,
            alpha_unstaked
        );

        // 2. Validate the user input
        Self::validate_remove_stake(&coldkey, &hotkey, netuid, alpha_unstaked)?;

        // 3. Calcaulate the maximum amount that can be executed with price limit
        let max_amount = Self::get_max_amount_remove(netuid, limit_price);
        let mut possible_alpha = alpha_unstaked;
        if possible_alpha > max_amount {
            possible_alpha = max_amount;
        }

        // 4. Swap the alpba to tao and update counters for this subnet.
        let fee = DefaultStakingFee::<T>::get();
        let tao_unstaked: u64 =
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
    pub fn get_max_amount_remove(netuid: u16, limit_price: u64) -> u64 {
        // Corner case: root and stao
        // There's no slippage for root or stable subnets, so if limit price is 1e9 rao or
        // higher, then max_amount equals u64::MAX, otherwise it is 0.
        if (netuid == Self::get_root_netuid()) || (SubnetMechanism::<T>::get(netuid)) == 0 {
            if limit_price <= 1_000_000_000 {
                return u64::MAX;
            } else {
                return 0;
            }
        }

        // Corner case: SubnetAlphaIn is zero. Staking can't happen, so max amount is zero.
        let alpha_in = SubnetAlphaIn::<T>::get(netuid);
        if alpha_in == 0 {
            return 0;
        }
        let alpha_in_float: U96F32 = U96F32::saturating_from_num(alpha_in);

        // Corner case: SubnetTAO is zero. Staking can't happen, so max amount is zero.
        let tao_reserve = SubnetTAO::<T>::get(netuid);
        if tao_reserve == 0 {
            return 0;
        }
        let tao_reserve_float: U96F32 = U96F32::saturating_from_num(tao_reserve);

        // Corner case: limit_price == 0 (because there's division by limit price)
        // => can sell all
        if limit_price == 0 {
            return u64::MAX;
        }

        // Corner case: limit_price > current_price (price cannot increase with unstaking)
        let limit_price_float: U96F32 = U96F32::saturating_from_num(limit_price)
            .checked_div(U96F32::saturating_from_num(1_000_000_000))
            .unwrap_or(U96F32::saturating_from_num(0));
        if limit_price_float > Self::get_alpha_price(netuid) {
            return 0;
        }

        // Main case: return SQRT(SubnetTAO * SubnetAlphaIn / limit_price) - SubnetAlphaIn
        // This is the positive solution of quare equation for finding Alpha amount from
        // limit_price.
        let zero: U96F32 = U96F32::saturating_from_num(0.0);
        let epsilon: U96F32 = U96F32::saturating_from_num(0.1);
        let sqrt: U96F32 = checked_sqrt(tao_reserve_float, epsilon)
            .unwrap_or(zero)
            .saturating_mul(
                checked_sqrt(alpha_in_float.safe_div(limit_price_float), epsilon).unwrap_or(zero),
            );

        sqrt.saturating_sub(U96F32::saturating_from_num(alpha_in_float))
            .saturating_to_num::<u64>()
    }
}
