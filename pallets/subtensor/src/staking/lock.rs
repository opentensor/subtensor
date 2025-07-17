use super::*;
use safe_math::*;
use substrate_fixed::types::{I96F32, U64F64};
use subtensor_runtime_common::NetUid;

#[freeze_struct("f92b0bb7408af4d8")]
#[derive(
    Clone, Copy, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
)]
pub struct StakeLock {
    pub alpha_locked: AlphaCurrency,
    pub start_block: u64,
    pub end_block: u64,
}

impl<T: Config> Pallet<T> {
    /// Sets the lock interval in blocks.
    ///
    /// This function updates the minimum duration for which stakes can be locked.
    ///
    /// # Arguments
    ///
    /// * `new_interval` - The new lock interval in blocks.
    ///
    /// # Events
    ///
    /// Emits a `LockIntervalSet` event with the new interval value.
    pub fn set_lock_interval_blocks(new_interval: u64) {
        // Update the lock interval storage
        LockIntervalBlocks::<T>::put(new_interval);

        // Emit an event for the new lock interval
        Self::deposit_event(Event::LockIntervalSet { new_interval });
    }

    /// Gets the current lock interval in blocks.
    ///
    /// This function retrieves the current value of the lock interval.
    ///
    /// # Returns
    ///
    /// * `u64` - The current lock interval in blocks.
    pub fn get_lock_interval_blocks() -> u64 {
        LockIntervalBlocks::<T>::get()
    }

    /// Calculates the conviction score for a specific hotkey and coldkey pair on a given subnet.
    ///
    /// This function retrieves the locked stake amount from the `Locks` storage and calculates
    /// the conviction score based on the locked amount and the lock duration.
    ///
    /// # Arguments
    ///
    /// * `hotkey` - The hotkey account ID.
    /// * `coldkey` - The coldkey account ID.
    /// * `netuid` - The subnet ID.
    ///
    /// # Returns
    ///
    /// * `AlphaCurrency` - The conviction score calculated from the locked stake.
    pub fn get_conviction_for_hotkey_and_coldkey_on_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> AlphaCurrency {
        let stake_lock = Locks::<T>::get((netuid, hotkey.clone(), coldkey.clone()));

        Self::calculate_conviction(&stake_lock, Self::get_current_block_as_u64())
    }

    /// Locks a specified amount of stake for a given duration on a subnet.
    ///
    /// This function allows a user to lock their stake, increasing their conviction score.
    /// The locked stake cannot be withdrawn until the lock period expires, and the new lock
    /// must not decrease the current conviction score.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call, must be signed by the coldkey.
    /// * `hotkey` - The hotkey associated with the stake to be locked.
    /// * `netuid` - The ID of the subnet where the stake is locked.
    /// * `duration` - The duration (in blocks) for which the stake will be locked.
    /// * `alpha_locked` - The amount of stake to be locked.
    ///
    /// # Returns
    ///
    /// * `DispatchResult` - The result of the lock operation.
    ///
    /// # Errors
    ///
    /// * `SubnetNotExists` - If the specified subnet does not exist.
    /// * `HotKeyAccountNotExists` - If the hotkey account does not exist.
    /// * `HotKeyNotRegisteredInSubNet` - If the hotkey is not registered on the specified subnet.
    /// * `NotEnoughStakeToWithdraw` - If the user doesn't have enough stake to lock, or if the new lock would decrease the current conviction.
    ///
    /// # Events
    ///
    /// * `LockIncreased` - Emitted when the lock is successfully increased.
    ///
    /// # TODO
    ///
    /// * Consider implementing a maximum lock duration to prevent excessively long locks.
    /// * Implement a mechanism to partially unlock stakes as the lock period progresses.
    /// * Add more granular error handling for different failure scenarios.
    pub fn do_lock(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: NetUid,
        duration: u64,
        alpha_locked: AlphaCurrency,
    ) -> dispatch::DispatchResult {
        // Step 1: Validate inputs and check conditions
        // Ensure the origin is valid.
        let coldkey = ensure_signed(origin)?;

        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Ensure that the hotkey account exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure the the lock is above zero.
        ensure!(alpha_locked > 0.into(), Error::<T>::NotEnoughStakeToWithdraw);

        // Get the lockers current stake.
        let current_alpha_stake =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        // Ensure that the caller has enough stake to lock.
        ensure!(
            alpha_locked <= current_alpha_stake,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Step 2: Calculate and compare convictions
        // Get the current block.
        let current_block = Self::get_current_block_as_u64();
        let new_end_block = current_block.saturating_add(duration);

        // Check that we are not decreasing the current conviction.
        if Locks::<T>::contains_key((netuid, hotkey.clone(), coldkey.clone())) {
            // Get the current lock.
            let stake_lock = Locks::<T>::get((netuid, &hotkey, &coldkey));

            // Calculate the current conviction.
            let current_conviction = Self::calculate_conviction(&stake_lock, current_block);

            // Calculate the new conviction.
            let new_conviction = Self::calculate_conviction(
                &StakeLock {
                    alpha_locked,
                    start_block: current_block,
                    end_block: new_end_block,
                },
                current_block,
            );

            // Ensure the new lock does not decrease the current conviction
            ensure!(
                new_conviction >= current_conviction,
                Error::<T>::NotEnoughStakeToWithdraw
            );
        }

        // Step 3: Set the new lock
        Locks::<T>::insert(
            (netuid, hotkey.clone(), coldkey.clone()),
            StakeLock {
                alpha_locked,
                start_block: current_block,
                end_block: current_block.saturating_add(duration),
            },
        );

        // Step 4: Emit event and return
        // Lock increased event.
        log::info!(
            "LockIncreased( coldkey:{:?}, hotkey:{:?}, netuid:{:?}, alpha_locked:{:?} )",
            coldkey.clone(),
            hotkey.clone(),
            netuid,
            alpha_locked
        );
        Self::deposit_event(Event::LockIncreased {
            coldkey: coldkey.clone(),
            hotkey: hotkey.clone(),
            netuid,
            alpha_locked,
        });

        // Ok and return.
        Ok(())
    }

    /// Updates the stake lock EMAs and owners of all subnets periodically.
    ///
    /// This function checks if it's time to update subnet owners based on the current block number
    /// and a predefined update interval. If the condition is met, it iterates through all subnet
    /// network IDs and calls the `update_subnet_owner` function for each subnet.
    ///
    /// # Details
    /// - The update interval is set to 7200 * 15 blocks (approximately 15 days, assuming 7200 blocks per day).
    /// - The update is triggered every two intervals (30 days) when the current block number is divisible by twice the update interval.
    ///
    /// # Effects
    /// - When the update condition is met, it calls `update_subnet_owner` for each subnet,
    ///   potentially changing the owner of each subnet based on conviction scores.
    pub fn update_stake_locks(current_block: u64) {
        let update_interval = 216_000; // Approx 30 days.
        if current_block.checked_rem(update_interval).unwrap_or(1) == 0 {
            for netuid in Self::get_all_subnet_netuids() {
                Self::update_subnet_owner(netuid, update_interval);
            }
        }
    }

    /// Calculates the exponentially moving average (EMA) of conviction for a given (hotkey, coldkey) pair in a subnet.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The identifier of the subnet.
    /// * `update_period` - The number of blocks since the last update. Used to compute the smoothing factor.
    /// * `conviction` - The current conviction value to blend into the EMA.
    /// * `hotkey` - The hotkey account associated with the stake lock.
    /// * `coldkey` - The coldkey account associated with the stake lock.
    ///
    /// # Returns
    ///
    /// Returns the updated conviction EMA as a `u64`.
    ///
    /// # Description
    ///
    /// This function uses the formula:
    ///
    /// ```text
    /// new_ema = old_ema * (1 - alpha) + conviction * alpha
    /// where alpha = update_period / lock_interval
    /// ```
    ///
    /// - If `alpha` (smoothing factor) exceeds `1.0`, it is capped at `1.0`.
    /// - `ConvictionEma` is retrieved from storage for the key `(netuid, hotkey, coldkey)`.
    /// - Floating point arithmetic is performed using `U64F64` fixed-point type to maintain precision.
    ///
    /// # Notes
    ///
    /// - This function is pure: it does not mutate any storage.
    /// - Use the result to update `ConvictionEma` if necessary.
    /// - Zero LockIntervalBLocks will result in constant EMA
    ///
    pub fn get_conviction_ema(
        netuid: NetUid,
        update_period: u64,
        conviction: AlphaCurrency,
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
    ) -> AlphaCurrency {
        let one = U64F64::saturating_from_num(1.0);
        let zero = U64F64::saturating_from_num(1.0);
        let lock_interval_blocks = U64F64::saturating_from_num(Self::get_lock_interval_blocks());
        let mut smoothing_factor =
            U64F64::saturating_from_num(update_period).safe_div_or(lock_interval_blocks, zero);
        if smoothing_factor > one {
            smoothing_factor = one;
        }

        let old_ema =
            U64F64::saturating_from_num(ConvictionEma::<T>::get((netuid, hotkey, coldkey)));

        AlphaCurrency::from(old_ema
            .saturating_mul(one.saturating_sub(smoothing_factor))
            .saturating_add(
                smoothing_factor.saturating_mul(U64F64::saturating_from_num(conviction)),
            )
            .saturating_to_num::<u64>())
    }

    /// Determines the subnet owner based on the highest conviction score.
    ///
    /// This function calculates the conviction score for each hotkey in the subnet,
    /// considering the lock amount and duration. The hotkey with the highest total
    /// conviction score becomes the subnet owner.
    ///
    /// # Arguments
    /// * `netuid` - The network ID of the subnet
    /// * `update_period` - How frequently this call is made (for EMA calculation)
    ///
    /// # Effects
    /// * Updates the SubnetOwner storage item with the coldkey of the highest conviction hotkey
    pub fn update_subnet_owner(netuid: NetUid, update_period: u64) {
        let current_block = Self::get_current_block_as_u64();

        // Get the updated current owner's conviction first
        let owner_coldkey = SubnetOwner::<T>::get(netuid);
        let owner_hotkey = SubnetOwnerHotkey::<T>::get(netuid);
        let owner_lock = Locks::<T>::get((netuid, owner_hotkey.clone(), owner_coldkey.clone()));
        let updated_owner_conviction = Self::calculate_conviction(&owner_lock, current_block);
        let mut updated_owner_conviction_ema = Self::get_conviction_ema(
            netuid,
            update_period,
            updated_owner_conviction,
            &owner_hotkey,
            &owner_coldkey,
        );
        let mut new_owner_coldkey = owner_coldkey.clone();
        let mut new_owner_hotkey = owner_hotkey.clone();
        let mut owner_updated = false;
        let mut total_conviction = AlphaCurrency::from(0);

        for ((hotkey, coldkey), stake_lock) in Locks::<T>::iter_prefix((netuid,)) {
            // Update EMAs. The update value depends on the update_period so that even if we change how
            // frequently we update EMAs, the EMA curve doesn't change (except getting less or more
            // accurate)

            let new_conviction = Self::calculate_conviction(&stake_lock, current_block);
            let new_ema =
                Self::get_conviction_ema(netuid, update_period, new_conviction, &hotkey, &coldkey);
            ConvictionEma::<T>::insert((netuid, hotkey.clone(), coldkey.clone()), new_ema);
            total_conviction = total_conviction.saturating_add(new_conviction);

            // In case of a tie, lower value coldkey wins
            if (new_ema > updated_owner_conviction_ema)
                || (new_ema == updated_owner_conviction_ema && coldkey < new_owner_coldkey)
            {
                new_owner_coldkey = coldkey;
                new_owner_hotkey = hotkey;
                updated_owner_conviction_ema = new_ema;
                owner_updated = true;
            }
        }

        // Implement a minimum conviction threshold for becoming a subnet owner
        let min_conviction_threshold = AlphaCurrency::from(1000); // TODO: adjust as needed
        if total_conviction < min_conviction_threshold {
            owner_updated = false;
        }

        // Set the subnet owner to the coldkey of the hotkey with highest conviction
        if owner_updated {
            SubnetOwner::<T>::insert(netuid, new_owner_coldkey.clone());
            SubnetOwnerHotkey::<T>::insert(netuid, new_owner_hotkey.clone());
        }

        // Update subnet locked
        SubnetLocked::<T>::insert(netuid, total_conviction);
    }

    /// Calculates the conviction score for a locked stake.
    ///
    /// This function computes a conviction score based on the amount of locked stake, the time
    /// this lock existed (since start_block) and the remaining lock duration. The score increases
    /// with both the lock amount and duration, but with diminishing returns for longer lock
    /// periods.
    ///
    /// # Arguments
    ///
    /// * `lock_amount` - The amount of stake locked, as a u64.
    /// * `start_block` - The block number when the lock was set, as a u64.
    /// * `end_block` - The block number when the lock expires, as a u64.
    /// * `current_block` - The current block number, as a u64.
    ///
    /// # Returns
    ///
    /// * A u64 representing the calculated conviction score.
    ///
    /// # Formula
    ///
    /// The conviction score is linear of blocks, starting with 100% locked at start_block and going
    /// down to 0% locked at the end_block:
    ///
    ///   conviction = alpha_locked * (ax + b),
    ///
    /// where a = 1 / (start_block - end_block)
    ///       b = end_block / (end_block - start_block)
    ///       x is current block
    ///
    pub fn calculate_conviction(lock: &StakeLock, current_block: u64) -> AlphaCurrency {
        // Handle corner cases first (with 100% precision)
        if current_block < lock.start_block {
            return 0.into();
        } else if current_block == lock.start_block {
            return lock.alpha_locked;
        } else if current_block >= lock.end_block {
            return 0.into();
        }

        // Handle the cases between start and end
        let lock_duration =
            I96F32::saturating_from_num(lock.end_block.saturating_sub(lock.start_block));
        let minus_one = I96F32::saturating_from_num(-1);
        let a = minus_one.safe_div(lock_duration);
        let b = I96F32::saturating_from_num(lock.end_block).safe_div(lock_duration);
        let x = I96F32::saturating_from_num(current_block);
        let locked_alpha_fixed = I96F32::saturating_from_num(lock.alpha_locked);
        let conviction_score =
            locked_alpha_fixed.saturating_mul(a.saturating_mul(x).saturating_add(b));

        conviction_score.saturating_to_num::<u64>()
    }

    pub fn check_locks_on_stake_reduction(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
        alpha_unstaked: u64,
    ) -> dispatch::DispatchResult {
        if Locks::<T>::contains_key((netuid, &hotkey, &coldkey)) {
            let total_stake =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid);
            let current_block = Self::get_current_block_as_u64();
            // Retrieve the lock information for the given netuid, hotkey, and coldkey
            let stake_lock = Locks::<T>::get((netuid, hotkey.clone(), coldkey.clone()));
            let conviction = Self::calculate_conviction(&stake_lock, current_block);

            let stake_after_unstake = total_stake.saturating_sub(alpha_unstaked);
            // Ensure the requested unstake amount is not more than what's allowed
            ensure!(
                stake_after_unstake >= conviction,
                Error::<T>::NotEnoughStakeToWithdraw
            );
            // If conviction is 0, remove the lock
            if conviction == 0 {
                Locks::<T>::remove((netuid, hotkey.clone(), coldkey.clone()));
            }
        }

        Ok(())
    }
}
