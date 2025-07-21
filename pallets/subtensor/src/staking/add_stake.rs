use substrate_fixed::types::I96F32;
use subtensor_runtime_common::NetUid;
use subtensor_swap_interface::{OrderType, SwapHandler};

use super::*;

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
    /// * 'netuid' (u16):
    ///     - Subnetwork UID
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
        netuid: NetUid,
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

        Self::ensure_subtoken_enabled(netuid)?;

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
        let tao_staked: I96F32 =
            Self::remove_balance_from_coldkey_account(&coldkey, stake_to_be_added)?.into();

        // 4. Swap the stake into alpha on the subnet and increase counters.
        // Emit the staking event.
        Self::stake_into_subnet(
            &hotkey,
            &coldkey,
            netuid,
            tao_staked.saturating_to_num::<u64>(),
            T::SwapInterface::max_price(),
            true,
        )?;

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
    /// * 'netuid' (u16):
    ///     - Subnetwork UID
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
        netuid: NetUid,
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

        // 2. Calculate the maximum amount that can be executed with price limit
        let max_amount = Self::get_max_amount_add(netuid, limit_price)?;
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

        // 4. If the coldkey is not the owner, make the hotkey a delegate.
        if Self::get_owning_coldkey_for_hotkey(&hotkey) != coldkey {
            Self::maybe_become_delegate(&hotkey);
        }

        // 5. Ensure the remove operation from the coldkey is a success.
        let tao_staked: u64 = Self::remove_balance_from_coldkey_account(&coldkey, possible_stake)?;

        // 6. Swap the stake into alpha on the subnet and increase counters.
        // Emit the staking event.
        Self::stake_into_subnet(&hotkey, &coldkey, netuid, tao_staked, limit_price, true)?;

        // Ok and return.
        Ok(())
    }

    // Returns the maximum amount of RAO that can be executed with price limit
    pub fn get_max_amount_add(netuid: NetUid, limit_price: u64) -> Result<u64, Error<T>> {
        // Corner case: root and stao
        // There's no slippage for root or stable subnets, so if limit price is 1e9 rao or
        // higher, then max_amount equals u64::MAX, otherwise it is 0.
        if netuid.is_root() || SubnetMechanism::<T>::get(netuid) == 0 {
            if limit_price >= 1_000_000_000 {
                return Ok(u64::MAX);
            } else {
                return Err(Error::ZeroMaxStakeAmount);
            }
        }

        // Use reverting swap to estimate max limit amount
        let result = T::SwapInterface::swap(
            netuid.into(),
            OrderType::Buy,
            u64::MAX,
            limit_price,
            false,
            true,
        )
        .map(|r| r.amount_paid_in.saturating_add(r.fee_paid))
        .map_err(|_| Error::ZeroMaxStakeAmount)?;

        if result != 0 {
            Ok(result)
        } else {
            Err(Error::ZeroMaxStakeAmount)
        }
    }

    pub fn do_add_stake_aggregate(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: NetUid,
        stake_to_be_added: u64,
    ) -> dispatch::DispatchResult {
        // We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;

        // Consider the weight from on_finalize
        if cfg!(feature = "runtime-benchmarks") && !cfg!(test) {
            Self::do_add_stake(
                crate::dispatch::RawOrigin::Signed(coldkey.clone()).into(),
                hotkey.clone(),
                netuid,
                stake_to_be_added,
            )?;
        }

        // Save the staking job for the on_finalize
        let stake_job = StakeJob::AddStake {
            hotkey,
            coldkey,
            netuid,
            stake_to_be_added,
        };

        let stake_job_id = NextStakeJobId::<T>::get();
        let current_blocknumber = <frame_system::Pallet<T>>::block_number();

        StakeJobs::<T>::insert(current_blocknumber, stake_job_id, stake_job);
        NextStakeJobId::<T>::set(stake_job_id.saturating_add(1));

        Ok(())
    }

    pub fn do_add_stake_limit_aggregate(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: NetUid,
        stake_to_be_added: u64,
        limit_price: u64,
        allow_partial: bool,
    ) -> dispatch::DispatchResult {
        let coldkey = ensure_signed(origin)?;

        if cfg!(feature = "runtime-benchmarks") && !cfg!(test) {
            Self::do_add_stake_limit(
                crate::dispatch::RawOrigin::Signed(coldkey.clone()).into(),
                hotkey.clone(),
                netuid,
                stake_to_be_added,
                limit_price,
                allow_partial,
            )?;
        }

        let stake_job = StakeJob::AddStakeLimit {
            hotkey,
            coldkey,
            netuid,
            stake_to_be_added,
            limit_price,
            allow_partial,
        };

        let stake_job_id = NextStakeJobId::<T>::get();
        let current_blocknumber = <frame_system::Pallet<T>>::block_number();

        StakeJobs::<T>::insert(current_blocknumber, stake_job_id, stake_job);
        NextStakeJobId::<T>::set(stake_job_id.saturating_add(1));

        Ok(())
    }
}
