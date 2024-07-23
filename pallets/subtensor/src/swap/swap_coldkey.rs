use super::*;
use crate::MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP;
use frame_support::weights::Weight;
use sp_core::{Get, U256};

impl<T: Config> Pallet<T> {
    /// Swaps the coldkey associated with a set of hotkeys from an old coldkey to a new coldkey.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call, which must be signed by the old coldkey.
    /// * `new_coldkey` - The account ID of the new coldkey.
    ///
    /// # Returns
    ///
    /// Returns a `DispatchResultWithPostInfo` indicating success or failure, along with the weight consumed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The caller is not a valid signed origin.
    /// - The old coldkey (caller) is in arbitration.
    /// - The new coldkey is already associated with other hotkeys or is a hotkey itself.
    /// - There's not enough balance to pay for the swap.
    ///
    /// # Events
    ///
    /// Emits a `ColdkeySwapped` event when successful.
    ///
    /// # Weight
    ///
    /// Weight is tracked and updated throughout the function execution.
    pub fn do_swap_coldkey(
        origin: T::RuntimeOrigin,
        new_coldkey: &T::AccountId,
    ) -> DispatchResultWithPostInfo {
        // 1. Ensure the origin is signed and get the old coldkey
        let old_coldkey = ensure_signed(origin)?;

        // 2. Check if the old coldkey is in arbitration
        ensure!(
            !Self::coldkey_in_arbitration(&old_coldkey),
            Error::<T>::ColdkeyIsInArbitration
        );

        // 3. Initialize the weight for this operation
        let mut weight: Weight = T::DbWeight::get().reads(2);

        // 4. Ensure the new coldkey is not associated with any hotkeys
        ensure!(
            StakingHotkeys::<T>::get(new_coldkey).is_empty(),
            Error::<T>::ColdKeyAlreadyAssociated
        );

        // 5. Ensure the new coldkey is not a hotkey
        ensure!(
            !Self::hotkey_account_exists(new_coldkey),
            Error::<T>::ColdKeyAlreadyAssociated
        );

        // 6. Calculate the swap cost and ensure sufficient balance
        let swap_cost = Self::get_key_swap_cost();
        log::debug!("Coldkey swap cost: {:?}", swap_cost);
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&old_coldkey, swap_cost),
            Error::<T>::NotEnoughBalanceToPaySwapColdKey
        );

        // 7. Remove and burn the swap cost from the old coldkey's account
        let actual_burn_amount =
            Self::remove_balance_from_coldkey_account(&old_coldkey, swap_cost)?;
        Self::burn_tokens(actual_burn_amount);

        // 8. Update the weight for the balance operations
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 9. Perform the actual coldkey swap
        let _ = Self::perform_swap_coldkey(&old_coldkey, new_coldkey, &mut weight);

        // 10. Update the last transaction block for the new coldkey
        Self::set_last_tx_block(new_coldkey, Self::get_current_block_as_u64());
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        // 11. Emit the ColdkeySwapped event
        Self::deposit_event(Event::ColdkeySwapped {
            old_coldkey: old_coldkey.clone(),
            new_coldkey: new_coldkey.clone(),
        });

        // 12. Return the result with the updated weight
        Ok(Some(weight).into())
    }

    /// Performs the actual coldkey swap operation, transferring all associated data and balances from the old coldkey to the new coldkey.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The account ID of the old coldkey.
    /// * `new_coldkey` - The account ID of the new coldkey.
    /// * `weight` - A mutable reference to the current transaction weight.
    ///
    /// # Returns
    ///
    /// Returns a `DispatchResult` indicating success or failure of the operation.
    ///
    /// # Steps
    ///
    /// 1. Swap TotalHotkeyColdkeyStakesThisInterval:
    ///    - For each hotkey owned by the old coldkey, transfer its stake and block data to the new coldkey.
    ///
    /// 2. Swap subnet ownership:
    ///    - For each subnet, if the old coldkey is the owner, transfer ownership to the new coldkey.
    ///
    /// 3. Swap Stakes:
    ///    - For each hotkey staking for the old coldkey, transfer its stake to the new coldkey.
    ///
    /// 4. Swap total coldkey stake:
    ///    - Transfer the total stake from the old coldkey to the new coldkey.
    ///
    /// 5. Swap StakingHotkeys:
    ///    - Transfer the list of staking hotkeys from the old coldkey to the new coldkey.
    ///
    /// 6. Swap hotkey owners:
    ///    - For each hotkey owned by the old coldkey, transfer ownership to the new coldkey.
    ///    - Update the list of owned hotkeys for both old and new coldkeys.
    ///
    /// 7. Transfer remaining balance:
    ///    - Transfer any remaining balance from the old coldkey to the new coldkey.
    ///
    /// Throughout the process, the function updates the transaction weight to reflect the operations performed.
    ///
    /// # Notes
    ///
    /// This function is a critical part of the coldkey swap process and should be called only after all necessary checks and validations have been performed.
    pub fn perform_swap_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) -> DispatchResult {
        // 1. Swap TotalHotkeyColdkeyStakesThisInterval
        // TotalHotkeyColdkeyStakesThisInterval: MAP ( hotkey, coldkey ) --> ( stake, block ) | Stake of the hotkey for the coldkey.
        for hotkey in OwnedHotkeys::<T>::get(old_coldkey).iter() {
            let (stake, block) =
                TotalHotkeyColdkeyStakesThisInterval::<T>::get(&hotkey, old_coldkey);
            TotalHotkeyColdkeyStakesThisInterval::<T>::remove(&hotkey, old_coldkey);
            TotalHotkeyColdkeyStakesThisInterval::<T>::insert(&hotkey, new_coldkey, (stake, block));
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }

        // 2. Swap subnet owner.
        // SubnetOwner: MAP ( netuid ) --> (coldkey) | Owner of the subnet.
        for netuid in Self::get_all_subnet_netuids() {
            let subnet_owner = SubnetOwner::<T>::get(netuid);
            if subnet_owner == *old_coldkey {
                SubnetOwner::<T>::insert(netuid, new_coldkey.clone());
            }
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        }

        // 3. Swap Stake.
        // Stake: MAP ( hotkey, coldkey ) --> u64 | Stake of the hotkey for the coldkey.
        for hotkey in StakingHotkeys::<T>::get(old_coldkey) {
            // Get the stake on the old (hot,coldkey) account.
            let old_stake: u64 = Stake::<T>::get(&hotkey, old_coldkey);
            // Get the stake on the new (hot,coldkey) account.
            let new_stake: u64 = Stake::<T>::get(&hotkey, new_coldkey);
            // Add the stake to new account.
            Stake::<T>::insert(&hotkey, new_coldkey, new_stake.saturating_add(old_stake));
            // Remove the value from the old account.
            Stake::<T>::remove(&hotkey, old_coldkey);
            // Add the weight for the read and write.
            weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));
        }

        // 4. Swap total coldkey stake.
        // TotalColdkeyStake: MAP ( coldkey ) --> u64 | Total stake of the coldkey.
        let old_coldkey_stake: u64 = TotalColdkeyStake::<T>::get(old_coldkey);
        // Get the stake of the new coldkey.
        let new_coldkey_stake: u64 = TotalColdkeyStake::<T>::get(new_coldkey);
        // Remove the value from the old account.
        TotalColdkeyStake::<T>::insert(old_coldkey, 0);
        // Add the stake to new account.
        TotalColdkeyStake::<T>::insert(
            new_coldkey,
            new_coldkey_stake.saturating_add(old_coldkey_stake),
        );
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // 5. Swap StakingHotkeys.
        // StakingHotkeys: MAP ( coldkey ) --> Vec<hotkeys> | Hotkeys staking for the coldkey.
        let old_staking_hotkeys: Vec<T::AccountId> = StakingHotkeys::<T>::get(old_coldkey);
        let mut new_staking_hotkeys: Vec<T::AccountId> = StakingHotkeys::<T>::get(new_coldkey);
        for hotkey in old_staking_hotkeys {
            // If the hotkey is not already in the new coldkey, add it.
            if !new_staking_hotkeys.contains(&hotkey) {
                new_staking_hotkeys.push(hotkey);
            }
        }
        StakingHotkeys::<T>::remove(old_coldkey);
        StakingHotkeys::<T>::insert(new_coldkey, new_staking_hotkeys);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // 6. Swap hotkey owners.
        // Owner: MAP ( hotkey ) --> coldkey | Owner of the hotkey.
        // OwnedHotkeys: MAP ( coldkey ) --> Vec<hotkeys> | Hotkeys owned by the coldkey.
        let old_owned_hotkeys: Vec<T::AccountId> = OwnedHotkeys::<T>::get(old_coldkey);
        let mut new_owned_hotkeys: Vec<T::AccountId> = OwnedHotkeys::<T>::get(new_coldkey);
        for owned_hotkey in old_owned_hotkeys.iter() {
            // Remove the hotkey from the old coldkey.
            Owner::<T>::remove(owned_hotkey);
            // Add the hotkey to the new coldkey.
            Owner::<T>::insert(owned_hotkey, new_coldkey.clone());
            // Addd the owned hotkey to the new set of owned hotkeys.
            if !new_owned_hotkeys.contains(owned_hotkey) {
                new_owned_hotkeys.push(owned_hotkey.clone());
            }
        }
        OwnedHotkeys::<T>::remove(old_coldkey);
        OwnedHotkeys::<T>::insert(new_coldkey, new_owned_hotkeys);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // 7. Transfer remaining balance.
        // Balance: MAP ( coldkey ) --> u64 | Balance of the coldkey.
        // Transfer any remaining balance from old_coldkey to new_coldkey
        let remaining_balance = Self::get_coldkey_balance(old_coldkey);
        if remaining_balance > 0 {
            Self::kill_coldkey_account(old_coldkey, remaining_balance)?;
            Self::add_balance_to_coldkey_account(new_coldkey, remaining_balance);
        }
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // Return ok.
        Ok(())
    }

    /// Checks if a coldkey is currently in arbitration.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - The account ID of the coldkey to check.
    ///
    /// # Returns
    ///
    /// * `bool` - True if the coldkey is in arbitration, false otherwise.
    ///
    /// # Notes
    ///
    /// This function compares the arbitration block number of the coldkey with the current block number.
    pub fn coldkey_in_arbitration(coldkey: &T::AccountId) -> bool {
        ColdkeyArbitrationBlock::<T>::get(coldkey) > Self::get_current_block_as_u64()
    }

    /// Returns the remaining arbitration period for a given coldkey.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - The account ID of the coldkey to check.
    ///
    /// # Returns
    ///
    /// * `u64` - The remaining arbitration period in blocks.
    ///
    ///
    /// # Notes
    ///
    /// This function calculates the remaining arbitration period by subtracting the current block number
    /// from the arbitration block number of the coldkey.
    pub fn get_remaining_arbitration_period(coldkey: &T::AccountId) -> u64 {
        let current_block: u64 = Self::get_current_block_as_u64();
        let arbitration_block: u64 = ColdkeyArbitrationBlock::<T>::get(coldkey);
        if arbitration_block > current_block {
            arbitration_block.saturating_sub(current_block)
        } else {
            0
        }
    }

    pub fn meets_min_allowed_coldkey_balance(coldkey: &T::AccountId) -> bool {
        let all_staked_keys: Vec<T::AccountId> = StakingHotkeys::<T>::get(coldkey);
        let mut total_staking_balance: u64 = 0;
        for hotkey in all_staked_keys {
            total_staking_balance = total_staking_balance
                .saturating_add(Self::get_stake_for_coldkey_and_hotkey(coldkey, &hotkey));
        }
        total_staking_balance =
            total_staking_balance.saturating_add(Self::get_coldkey_balance(coldkey));
        total_staking_balance >= MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP
    }

    /// Schedules a coldkey swap to a new coldkey with arbitration.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The account ID of the old coldkey.
    /// * `new_coldkey` - The account ID of the new coldkey.
    /// * `work` - The proof of work submitted by the caller.
    /// * `block_number` - The block number at which the work was performed.
    /// * `nonce` - The nonce used for the proof of work.
    ///
    /// # Returns
    ///
    /// * `DispatchResult` - The result of the dispatch.
    ///
    /// # Errors
    ///

    /// - `SameColdkey`: The old coldkey is the same as the new coldkey.
    /// - `DuplicateColdkey`: The new coldkey is already in the list of destination coldkeys.
    /// - `MaxColdkeyDestinationsReached`: There are already the maximum allowed destination coldkeys for the old coldkey.
    /// - `InsufficientBalanceToPerformColdkeySwap`: The old coldkey doesn't have the minimum required TAO balance.
    /// - `InvalidDifficulty`: The proof of work is invalid or doesn't meet the required difficulty.
    ///
    /// # Notes
    ///
    /// This function ensures that the new coldkey is not already in the list of destination coldkeys.
    /// It also checks for a minimum TAO balance and verifies the proof of work.
    /// The difficulty of the proof of work increases exponentially with each subsequent call.
    pub fn do_schedule_coldkey_swap(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        work: Vec<u8>,
        block_number: u64,
        nonce: u64,
    ) -> DispatchResult {
        ensure!(old_coldkey != new_coldkey, Error::<T>::SameColdkey);

        // Check if the old_coldkey is a subnet owner for any network
        let is_subnet_owner = (0..=TotalNetworks::<T>::get())
            .any(|netuid| SubnetOwner::<T>::get(netuid) == *old_coldkey);

        // Check if the old_coldkey has more than 500 TAO delegated
        let total_delegated = Self::get_total_delegated_stake(old_coldkey);
        let has_sufficient_delegation = total_delegated > 500_000_000_000; // 500 TAO in RAO

        // Only check the minimum balance if the old_coldkey is not a subnet owner
        // and doesn't have sufficient delegation
        if !(is_subnet_owner || has_sufficient_delegation) {
            ensure!(
                Self::meets_min_allowed_coldkey_balance(old_coldkey),
                Error::<T>::InsufficientBalanceToPerformColdkeySwap
            );
        }

        // Get current destination coldkeys
        let mut destination_coldkeys: Vec<T::AccountId> =
            ColdkeySwapDestinations::<T>::get(old_coldkey.clone());

        // Calculate difficulty based on the number of existing destination coldkeys
        let difficulty = Self::calculate_pow_difficulty(destination_coldkeys.len() as u32);
        let work_hash = Self::vec_to_hash(work.clone());
        ensure!(
            Self::hash_meets_difficulty(&work_hash, difficulty),
            Error::<T>::InvalidDifficulty
        );

        // Verify work is the product of the nonce, the block number, and coldkey
        let seal = Self::create_seal_hash(block_number, nonce, old_coldkey);
        ensure!(seal == work_hash, Error::<T>::InvalidSeal);

        // Check if the new coldkey is already in the swap wallets list
        ensure!(
            !destination_coldkeys.contains(new_coldkey),
            Error::<T>::DuplicateColdkey
        );

        // If the destinations keys are empty or have less than the maximum allowed, we will add the new coldkey to the list
        const MAX_COLDKEY_DESTINATIONS: usize = 10;

        if destination_coldkeys.len() < MAX_COLDKEY_DESTINATIONS {
            destination_coldkeys.push(new_coldkey.clone());
            ColdkeySwapDestinations::<T>::insert(old_coldkey.clone(), destination_coldkeys.clone());
        } else {
            return Err(Error::<T>::MaxColdkeyDestinationsReached.into());
        }

        // It is the first time we have seen this key
        if destination_coldkeys.len() == 1_usize {
            // Set the arbitration block for this coldkey
            let arbitration_block: u64 =
                Self::get_current_block_as_u64().saturating_add(ArbitrationPeriod::<T>::get());
            ColdkeyArbitrationBlock::<T>::insert(old_coldkey.clone(), arbitration_block);

            // Update the list of coldkeys to arbitrate on this block
            let mut key_to_arbitrate_on_this_block: Vec<T::AccountId> =
                ColdkeysToSwapAtBlock::<T>::get(arbitration_block);
            if !key_to_arbitrate_on_this_block.contains(old_coldkey) {
                key_to_arbitrate_on_this_block.push(old_coldkey.clone());
            }
            ColdkeysToSwapAtBlock::<T>::insert(arbitration_block, key_to_arbitrate_on_this_block);
        }

        // Emit an event indicating that a coldkey swap has been scheduled
        Self::deposit_event(Event::ColdkeySwapScheduled {
            old_coldkey: old_coldkey.clone(),
            new_coldkey: new_coldkey.clone(),
            arbitration_block: ColdkeyArbitrationBlock::<T>::get(old_coldkey),
        });

        Ok(())
    }

    /// Calculate the proof of work difficulty based on the number of swap attempts
    #[allow(clippy::arithmetic_side_effects)]
    pub fn calculate_pow_difficulty(swap_attempts: u32) -> U256 {
        let base_difficulty: U256 = U256::from(BaseDifficulty::<T>::get()); // Base difficulty
        base_difficulty.saturating_mul(U256::from(2).pow(U256::from(swap_attempts)))
    }

    /// Arbitrates coldkeys that are scheduled to be swapped on this block.
    ///
    /// This function retrieves the list of coldkeys scheduled to be swapped on the current block,
    /// and processes each coldkey by either extending the arbitration period or performing the swap
    /// to the new coldkey.
    ///
    /// # Returns
    ///
    /// * `Weight` - The total weight consumed by this operation
    pub fn swap_coldkeys_this_block(_weight_limit: &Weight) -> Result<Weight, &'static str> {
        let mut weight_used = frame_support::weights::Weight::from_parts(0, 0);

        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!("Swapping coldkeys for block: {:?}", current_block);

        let source_coldkeys: Vec<T::AccountId> = ColdkeysToSwapAtBlock::<T>::get(current_block);
        ColdkeysToSwapAtBlock::<T>::remove(current_block);
        weight_used = weight_used.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        let mut keys_swapped = 0u64;
        for coldkey_i in source_coldkeys.iter() {
            // TODO: need a sane way to terminate early without locking users in.
            // we should update the swap time
            // if weight_used.ref_time() > weight_limit.ref_time() {
            //     log::warn!("Could not finish swapping all coldkeys this block due to weight limit, breaking after swapping {} keys.", keys_swapped);
            //     break;
            // }

            let destinations_coldkeys: Vec<T::AccountId> =
                ColdkeySwapDestinations::<T>::get(coldkey_i);
            weight_used = weight_used.saturating_add(T::DbWeight::get().reads(1));

            if destinations_coldkeys.len() > 1 {
                // Do not remove ColdkeySwapDestinations if there are multiple destinations
                ColdkeyArbitrationBlock::<T>::insert(coldkey_i.clone(), u64::MAX);
                Self::deposit_event(Event::ArbitrationPeriodExtended {
                    coldkey: coldkey_i.clone(),
                });
            } else if let Some(new_coldkey) = destinations_coldkeys.first() {
                // Only remove ColdkeySwapDestinations if there's a single destination
                ColdkeySwapDestinations::<T>::remove(&coldkey_i);
                weight_used = weight_used.saturating_add(T::DbWeight::get().writes(1));
                keys_swapped = keys_swapped.saturating_add(1);
                let _ = Self::perform_swap_coldkey(coldkey_i, new_coldkey, &mut weight_used);
            }
        }

        Ok(weight_used)
    }

}