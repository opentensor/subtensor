use super::*;
use crate::epoch::math::*;
use safe_math::*;
use substrate_fixed::types::{I96F32, U64F64};
use subtensor_runtime_common::NetUid;

#[freeze_struct("79fc7facda47d89")]
#[derive(
    Clone, Copy, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
)]
pub struct StakeLock {
    pub alpha_locked: u64,
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
    /// * `u64` - The conviction score calculated from the locked stake.
    pub fn get_conviction_for_hotkey_and_coldkey_on_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> u64 {
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
        alpha_locked: u64,
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
        ensure!(alpha_locked > 0, Error::<T>::NotEnoughStakeToWithdraw);

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
    /// # Arguments
    /// * `update_period` - How frequently this call is made (for EMA calculation)
    ///
    /// # Effects
    /// - When the update condition is met, it calls `update_subnet_owner` for each subnet,
    ///   potentially changing the owner of each subnet based on conviction scores.
    pub fn update_stake_locks(update_period: u64) {
        let current_block = Self::get_current_block_as_u64();
        let update_interval = 7200 * 15; // Approx 15 days.
        if current_block % (update_interval * 2) == 0 {
            for netuid in Self::get_all_subnet_netuids() {
                Self::update_subnet_owner(netuid, update_period);
            }
        }
    }

    fn get_conviction_ema(
        netuid: NetUid,
        update_period: u64,
        conviction: u64,
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
    ) -> u64 {
        let one = U64F64::saturating_from_num(1.0);
        let mut smoothing_factor = U64F64::saturating_from_num(update_period)
            .safe_div(U64F64::saturating_from_num(LockIntervalBlocks::<T>::get()));
        if smoothing_factor > one {
            smoothing_factor = one;
        }

        let old_ema =
            U64F64::saturating_from_num(ConvictionEma::<T>::get((netuid, hotkey, coldkey)));

        old_ema
            .saturating_mul(one.saturating_sub(smoothing_factor))
            .saturating_add(
                smoothing_factor.saturating_mul(U64F64::saturating_from_num(conviction)),
            )
            .saturating_to_num::<u64>()
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
        let updated_owner_conviction_ema = Self::get_conviction_ema(
            netuid,
            update_period,
            updated_owner_conviction,
            &owner_hotkey,
            &owner_coldkey,
        );
        let mut new_owner_coldkey = owner_coldkey.clone();
        let mut new_owner_hotkey = owner_hotkey.clone();
        let mut owner_updated = false;

        for ((hotkey, coldkey), stake_lock) in Locks::<T>::iter_prefix((netuid,)) {
            // Update EMAs. The update value depends on the update_period so that even if we change how
            // frequently we update EMAs, the EMA curve doesn't change (except getting less or more
            // accurate)

            let new_conviction = Self::calculate_conviction(&stake_lock, current_block);
            let new_ema =
                Self::get_conviction_ema(netuid, update_period, new_conviction, &hotkey, &coldkey);
            ConvictionEma::<T>::insert((netuid, hotkey.clone(), coldkey.clone()), new_ema);

            if new_ema > updated_owner_conviction_ema {
                new_owner_coldkey = coldkey;
                new_owner_hotkey = hotkey;
                owner_updated = true;
            }
        }

        // TODO: Is this needed?
        // Implement a minimum conviction threshold for becoming a subnet owner
        // let min_conviction_threshold = I96F32::from_num(1000); // Example threshold, adjust as needed
        // if max_total_conviction < min_conviction_threshold {
        //     return;
        // }

        // Set the subnet owner to the coldkey of the hotkey with highest conviction
        if owner_updated {
            SubnetOwner::<T>::insert(netuid, new_owner_coldkey.clone());
            SubnetOwnerHotkey::<T>::insert(netuid, new_owner_hotkey.clone());
        }
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
    /// The conviction score is calculated using the following formula:
    /// score = lock_amount * (1 - e^(-lock_duration / (lock_interval_blocks )))
    ///
    /// Where:
    /// - lock_duration is in blocks
    ///
    pub fn calculate_conviction(lock: &StakeLock, current_block: u64) -> u64 {
        let lock_duration = lock.end_block.saturating_sub(current_block);
        let lock_interval_blocks = Self::get_lock_interval_blocks();
        let time_factor =
            -I96F32::from_num(lock_duration).saturating_div(I96F32::from_num(lock_interval_blocks));
        let exp_term = I96F32::from_num(1) - exp_safe_f96(I96F32::from_num(time_factor));
        let conviction_score = I96F32::from_num(lock.alpha_locked).saturating_mul(exp_term);

        conviction_score.to_num::<u64>()
    }
}
