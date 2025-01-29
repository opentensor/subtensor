use super::*;
use safe_math::*;
use sp_core::Get;
use substrate_fixed::types::U96F32;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic add_stake: Adds stake to a hotkey account.
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
    /// * StakeAdded;
    ///     -  On the successfully adding stake to a global account.
    ///
    /// # Raises:
    /// * 'NotEnoughBalanceToStake':
    ///     -  Not enough balance on the coldkey to add onto the global account.
    ///
    /// * 'NonAssociatedColdKey':
    ///     -  The calling coldkey is not associated with this hotkey.
    ///
    /// * 'BalanceWithdrawalError':
    ///     -  Errors stemming from transaction pallet.
    ///
    /// * 'TxRateLimitExceeded':
    ///     -  Thrown if key has hit transaction rate limit
    ///
    pub fn do_add_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        stake_to_be_added: u64,
    ) -> dispatch::DispatchResult {
        // 1. We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_add_stake( origin:{:?} hotkey:{:?}, netuid:{:?}, stake_to_be_added:{:?} )",
            coldkey,
            hotkey,
            netuid,
            stake_to_be_added
        );

        // 2. Validate user input
        Self::validate_add_stake(
            &coldkey,
            &hotkey,
            netuid,
            stake_to_be_added,
            stake_to_be_added,
            false,
        )?;

        // 3. Ensure the remove operation from the coldkey is a success.
        let tao_staked: u64 =
            Self::remove_balance_from_coldkey_account(&coldkey, stake_to_be_added)?;

        // 4. Swap the stake into alpha on the subnet and increase counters.
        // Emit the staking event.
        let fee = DefaultStakingFee::<T>::get();
        Self::stake_into_subnet(&hotkey, &coldkey, netuid, tao_staked, fee);

        // Ok and return.
        Ok(())
    }

    /// ---- The implementation for the extrinsic add_stake_limit: Adds stake to a hotkey
    /// account on a subnet with price limit.
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
    ///  * 'limit_price' (u64):
    ///     - The limit price expressed in units of RAO per one Alpha.
    ///
    ///  * 'allow_partial' (bool):
    ///     - Allows partial execution of the amount. If set to false, this becomes
    ///       fill or kill type or order.
    ///
    /// # Event:
    /// * StakeAdded;
    ///     -  On the successfully adding stake to a global account.
    ///
    /// # Raises:
    /// * 'NotEnoughBalanceToStake':
    ///     -  Not enough balance on the coldkey to add onto the global account.
    ///
    /// * 'NonAssociatedColdKey':
    ///     -  The calling coldkey is not associated with this hotkey.
    ///
    /// * 'BalanceWithdrawalError':
    ///     -  Errors stemming from transaction pallet.
    ///
    /// * 'TxRateLimitExceeded':
    ///     -  Thrown if key has hit transaction rate limit
    ///
    pub fn do_add_stake_limit(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        stake_to_be_added: u64,
        limit_price: u64,
        allow_partial: bool,
    ) -> dispatch::DispatchResult {
        // 1. We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_add_stake( origin:{:?} hotkey:{:?}, netuid:{:?}, stake_to_be_added:{:?} )",
            coldkey,
            hotkey,
            netuid,
            stake_to_be_added
        );

        // 2. Calcaulate the maximum amount that can be executed with price limit
        let max_amount = Self::get_max_amount_add(netuid, limit_price);
        let mut possible_stake = stake_to_be_added;
        if possible_stake > max_amount {
            possible_stake = max_amount;
        }

        // 3. Validate user input
        Self::validate_add_stake(
            &coldkey,
            &hotkey,
            netuid,
            stake_to_be_added,
            max_amount,
            allow_partial,
        )?;

        // 4. Ensure the remove operation from the coldkey is a success.
        let tao_staked: u64 = Self::remove_balance_from_coldkey_account(&coldkey, possible_stake)?;

        // 5. Swap the stake into alpha on the subnet and increase counters.
        // Emit the staking event.
        let fee = DefaultStakingFee::<T>::get();
        Self::stake_into_subnet(&hotkey, &coldkey, netuid, tao_staked, fee);

        // Ok and return.
        Ok(())
    }

    // Returns the maximum amount of RAO that can be executed with price limit
    pub fn get_max_amount_add(netuid: u16, limit_price: u64) -> u64 {
        // Corner case: root and stao
        // There's no slippage for root or stable subnets, so if limit price is 1e9 rao or
        // higher, then max_amount equals u64::MAX, otherwise it is 0.
        if (netuid == Self::get_root_netuid()) || (SubnetMechanism::<T>::get(netuid)) == 0 {
            if limit_price >= 1_000_000_000 {
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

        // Corner case: limit_price < current_price (price cannot decrease with staking)
        let limit_price_float: U96F32 = U96F32::saturating_from_num(limit_price)
            .checked_div(U96F32::saturating_from_num(1_000_000_000))
            .unwrap_or(U96F32::saturating_from_num(0));
        if limit_price_float < Self::get_alpha_price(netuid) {
            return 0;
        }

        // Main case: return SQRT(limit_price * SubnetTAO * SubnetAlphaIn) - SubnetTAO
        // This is the positive solution of quare equation for finding additional TAO from
        // limit_price.
        let zero: U96F32 = U96F32::saturating_from_num(0.0);
        let epsilon: U96F32 = U96F32::saturating_from_num(0.1);
        let sqrt: U96F32 =
            checked_sqrt(limit_price_float.saturating_mul(tao_reserve_float), epsilon)
                .unwrap_or(zero)
                .saturating_mul(checked_sqrt(alpha_in_float, epsilon).unwrap_or(zero));

        sqrt.saturating_sub(U96F32::saturating_from_num(tao_reserve_float))
            .saturating_to_num::<u64>()
    }
}
