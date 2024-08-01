use super::*;
use crate::epoch::math::*;
use alloc::collections::BTreeMap;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    pub fn do_lock(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        duration: u64,
        alpha_locked: u64,
    ) -> dispatch::DispatchResult {
        // Ensure the origin is valid.
        let coldkey = ensure_signed(origin)?;

        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Ensure that the hotkey account exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure the hotkey is registered on this subnet.
        ensure!(
            Self::is_hotkey_registered_on_network(netuid, &hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // Ensure the the lock is above zero.
        ensure!(alpha_locked > 0, Error::<T>::NotEnoughStakeToWithdraw);

        // Get the lockers current stake.
        let current_alpha_stake = Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);

        // Ensure that the caller has enough stake to unstake.
        ensure!(
            alpha_locked <= current_alpha_stake,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Get the current block.
        let current_block = Self::get_current_block_as_u64();

        // Set the new lock.
        Locks::<T>::insert(
            (netuid, hotkey.clone(), coldkey.clone()), 
            (alpha_locked, current_block, current_block.saturating_add(duration) )
        );

        // Lock increased event.
        log::info!(
            "LockIncreased( coldkey:{:?}, hotkey:{:?}, netuid:{:?}, alpha_locked:{:?} )",
            coldkey.clone(),
            hotkey.clone(),
            netuid,
            alpha_locked
        );
        Self::deposit_event(Event::LockIncreased(
            coldkey.clone(),
            hotkey.clone(),
            netuid,
            alpha_locked,
        ));

        // Ok and return.
        Ok(())
    }

    /// Calculates the maximum allowed unstakable amount for a given lock.
    ///
    /// This function determines how much of a locked stake can be unstaked based on
    /// the time elapsed since the lock was created. It uses an exponential decay
    /// function to gradually increase the unstakable amount over time.
    ///
    /// # Arguments
    ///
    /// * `alpha_locked` - The total amount of stake that was locked.
    /// * `start_block` - The block number when the lock was created.
    /// * `current_block` - The current block number.
    ///
    /// # Returns
    ///
    /// * `u64` - The maximum amount that can be unstaked at the current block.
    pub fn calculate_max_allowed_unstakable(alpha_locked: u64, start_block: u64, current_block: u64) -> u64 {
        let lock_interval_blocks = 7200 * 30 * 6; // Approximately half a year.
        
        let exponent = -I96F32::from_num(current_block.saturating_sub(start_block))
            .checked_div(I96F32::from_num(lock_interval_blocks))
            .unwrap_or(I96F32::from_num(0));
        
        let unlockable_fraction = I96F32::from_num(1) - exp_safe_F96( exponent.checked_neg().unwrap_or(I96F32::from_num(0)));
        
        I96F32::from_num(alpha_locked)
            .checked_mul(unlockable_fraction)
            .unwrap_or(I96F32::from_num(0))
            .to_num::<u64>()
    }

    /// Distributes the owner payment among hotkeys based on their conviction scores.
    ///
    /// This function calculates the conviction scores for all locked hotkeys in a subnet,
    /// and then distributes the payment proportionally based on these scores.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID of the subnet.
    /// * `amount` - The total amount of payment to distribute.
    ///
    /// # Effects
    ///
    /// * Calculates conviction scores for all locked hotkeys in the subnet.
    /// * Distributes the payment proportionally based on conviction scores.
    /// * Adds the distributed share to each hotkey's balance.
    /// * Emits an `OwnerPaymentDistributed` event for each distribution.
    pub fn distribute_owner_cut(netuid: u16, amount: u64) -> u64 {
        // Get the current block number
        let current_block = Self::get_current_block_as_u64();

        // Initialize variables to track total conviction and individual hotkey convictions
        let mut total_conviction: u64 = 0;
        let mut hotkey_convictions: BTreeMap<T::AccountId, u64> = BTreeMap::new();

        // Calculate total conviction and individual hotkey convictions
        for ((iter_netuid, hotkey, _), (lock_amount, _, end_block)) in Locks::<T>::iter() {
            if iter_netuid != netuid { continue; }
            // Calculate conviction for each lock
            let conviction = Self::calculate_conviction(lock_amount, end_block, current_block);
            // Add conviction to the hotkey's total
            *hotkey_convictions.entry(hotkey).or_default() += conviction;
            // Add to the total conviction
            total_conviction = total_conviction.saturating_add(conviction);
        }

        // If there's no conviction, return the full amount
        if total_conviction == 0 {
            return amount;
        }

        // Initialize variable to track remaining amount to distribute
        let mut remaining_amount = amount;

        // Distribute the owner cut based on conviction scores
        for (hotkey, conviction) in hotkey_convictions.iter() {
            // Calculate the share for this hotkey based on its conviction
            let share = u64::from(amount)
                .saturating_mul(u64::from(*conviction))
                .checked_div(u64::from(total_conviction))
                .unwrap_or(0) as u64;

            // Get the coldkey associated with this hotkey
            let owner_coldkey = Self::get_owning_coldkey_for_hotkey(&hotkey);

            // Emit the calculated share into the subnet for this hotkey
            Self::emit_into_subnet(&hotkey, &owner_coldkey, netuid, share);
            
            // Add the share to the lock.
            if Locks::<T>::contains_key((netuid, hotkey.clone(), owner_coldkey.clone())) {
                let (current_lock, start_block, end_block) = Locks::<T>::get((netuid, hotkey.clone(), owner_coldkey.clone()));
                let new_lock = current_lock.saturating_add(share);
                Locks::<T>::insert(
                (netuid, hotkey.clone(), owner_coldkey.clone()),
                    (new_lock, start_block, end_block)
                );
            }

            // Subtract the distributed share from the remaining amount
            remaining_amount = remaining_amount.saturating_sub(share);
        }

        // Return any undistributed amount
        remaining_amount
    }


    /// Updates the owners of all subnets periodically.
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
    pub fn update_all_subnet_owners() {
        let current_block = Self::get_current_block_as_u64();
        let update_interval = 7200 * 15;
        if current_block % (update_interval * 2) == 0 {
            for netuid in Self::get_all_subnet_netuids() {
                Self::update_subnet_owner(netuid);
            }
        }
    }

    /// Determines the subnet owner based on the highest conviction score.
    ///
    /// This function calculates the conviction score for each hotkey in the subnet,
    /// considering the lock amount and duration. The hotkey with the highest total
    /// conviction score becomes the subnet owner.
    ///
    /// # Arguments
    /// * `netuid` - The network ID of the subnet
    ///
    /// # Effects
    /// * Updates the SubnetOwner storage item with the coldkey of the highest conviction hotkey
    pub fn update_subnet_owner(netuid: u16) {
        let mut max_total_conviction: I96F32 = I96F32::from_num(0.0);
        let mut max_conviction_hotkey = None;
        let mut hotkey_convictions = BTreeMap::new();
        let current_block = Self::get_current_block_as_u64();

        // Iterate through all locks in the subnet
        for ((iter_netuid, iter_hotkey, _), (lock_amount, _, end_block)) in Locks::<T>::iter() {
            // Skip if the subnet does not match.
            if iter_netuid != netuid { continue; }
    
            // Calculate conviction score based on lock amount and duration
            let conviction_score = I96F32::from_num(Self::calculate_conviction(lock_amount, end_block, current_block));

            // Accumulate conviction scores for each hotkey
            let total_conviction = hotkey_convictions.entry(iter_hotkey.clone()).or_insert(I96F32::from_num(0));
            *total_conviction = total_conviction.saturating_add(conviction_score);

            // Update max conviction if current hotkey has higher total conviction
            if *total_conviction > max_total_conviction {
                max_total_conviction = *total_conviction;
                max_conviction_hotkey = Some(iter_hotkey.clone());
            }
        }

        // Set the total subnet Conviction.
        SubnetLocked::<T>::insert(netuid, max_total_conviction.to_num::<u64>());

        // Set the subnet owner to the coldkey of the hotkey with highest conviction
        if let Some(hotkey) = max_conviction_hotkey {
            let owning_coldkey = Self::get_owning_coldkey_for_hotkey(&hotkey);
            SubnetOwner::<T>::insert(netuid, owning_coldkey);
        }
    }

    /// Calculates the conviction score for a locked stake.
    ///
    /// This function computes a conviction score based on the amount of locked stake and the
    /// remaining lock duration. The score increases with both the lock amount and duration,
    /// but with diminishing returns for longer lock periods.
    ///
    /// # Arguments
    ///
    /// * `lock_amount` - The amount of stake locked, as a u64.
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
    /// score = lock_amount * (1 - e^(-lock_duration / (365 * 24 * 60 * 60)))
    ///
    /// Where:
    /// - lock_duration is in blocks
    /// - The denominator converts days to blocks (assuming 1 block per second)
    /// - e is the mathematical constant (base of natural logarithm)
    pub fn calculate_conviction(lock_amount: u64, end_block: u64, current_block: u64) -> u64 {
        let lock_duration = end_block.saturating_sub(current_block);
        let time_factor = -I96F32::from_num(lock_duration).saturating_div(I96F32::from_num(365 * 24 * 60 * 60)); // Convert days to blocks
        let exp_term = I96F32::from_num(1) - I96F32::from_num(time_factor);
        let conviction_score = I96F32::from_num(lock_amount).saturating_mul(exp_term);
        conviction_score.to_num::<u64>()
    }
}
