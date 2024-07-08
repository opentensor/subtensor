use super::*;
use crate::MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP;
use frame_support::traits::fungible::Mutate;
use frame_support::traits::tokens::Preservation;
use frame_support::{storage::IterableStorageDoubleMap, weights::Weight};
use sp_core::{Get, U256};

impl<T: Config> Pallet<T> {
    /// Swaps the hotkey of a coldkey account.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the transaction, and also the coldkey account.
    /// * `old_hotkey` - The old hotkey to be swapped.
    /// * `new_hotkey` - The new hotkey to replace the old one.
    ///
    /// # Returns
    ///
    /// * `DispatchResultWithPostInfo` - The result of the dispatch.
    ///
    /// # Errors
    ///
    /// * `NonAssociatedColdKey` - If the coldkey does not own the old hotkey.
    /// * `HotKeySetTxRateLimitExceeded` - If the transaction rate limit is exceeded.
    /// * `NewHotKeyIsSameWithOld` - If the new hotkey is the same as the old hotkey.
    /// * `HotKeyAlreadyRegisteredInSubNet` - If the new hotkey is already registered in the subnet.
    /// * `NotEnoughBalanceToPaySwapHotKey` - If there is not enough balance to pay for the swap.
    pub fn do_swap_hotkey(
        origin: T::RuntimeOrigin,
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let coldkey = ensure_signed(origin)?;
        ensure!(
            !Self::coldkey_in_arbitration(&coldkey),
            Error::<T>::ColdkeyIsInArbitration
        );

        let mut weight = T::DbWeight::get().reads(2);

        ensure!(old_hotkey != new_hotkey, Error::<T>::NewHotKeyIsSameWithOld);
        ensure!(
            !Self::is_hotkey_registered_on_any_network(new_hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 0));
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, old_hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::HotKeySetTxRateLimitExceeded
        );

        weight.saturating_accrue(
            T::DbWeight::get().reads((TotalNetworks::<T>::get().saturating_add(1u16)) as u64),
        );

        Self::swap_owner(old_hotkey, new_hotkey, &coldkey, &mut weight);
        Self::swap_total_hotkey_stake(old_hotkey, new_hotkey, &mut weight);
        Self::swap_delegates(old_hotkey, new_hotkey, &mut weight);
        Self::swap_stake(old_hotkey, new_hotkey, &mut weight);

        // Store the value of is_network_member for the old key
        let netuid_is_member: Vec<u16> = Self::get_netuid_is_member(old_hotkey, &mut weight);

        Self::swap_is_network_member(old_hotkey, new_hotkey, &netuid_is_member, &mut weight);
        Self::swap_axons(old_hotkey, new_hotkey, &netuid_is_member, &mut weight);
        Self::swap_keys(old_hotkey, new_hotkey, &netuid_is_member, &mut weight);
        Self::swap_loaded_emission(old_hotkey, new_hotkey, &netuid_is_member, &mut weight);
        Self::swap_uids(old_hotkey, new_hotkey, &netuid_is_member, &mut weight);
        Self::swap_prometheus(old_hotkey, new_hotkey, &netuid_is_member, &mut weight);

        Self::swap_total_hotkey_coldkey_stakes_this_interval(old_hotkey, new_hotkey, &mut weight);

        Self::set_last_tx_block(&coldkey, block);
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        Self::deposit_event(Event::HotkeySwapped {
            coldkey,
            old_hotkey: old_hotkey.clone(),
            new_hotkey: new_hotkey.clone(),
        });

        Ok(Some(weight).into())
    }

    /// Swaps the coldkey associated with a set of hotkeys from an old coldkey to a new coldkey.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call, which must be signed by the old coldkey.
    /// * `old_coldkey` - The account ID of the old coldkey.
    /// * `new_coldkey` - The account ID of the new coldkey.
    ///
    /// # Returns
    ///
    /// Returns a `DispatchResultWithPostInfo` indicating success or failure, along with the weight consumed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The caller is not the old coldkey.
    /// - The new coldkey is the same as the old coldkey.
    /// - The new coldkey is already associated with other hotkeys.
    /// - The transaction rate limit for coldkey swaps has been exceeded.
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
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let coldkey_performing_swap = ensure_signed(origin)?;
        ensure!(
            !Self::coldkey_in_arbitration(&coldkey_performing_swap),
            Error::<T>::ColdkeyIsInArbitration
        );

        let mut weight: Weight = T::DbWeight::get().reads(2);

        // Check that the coldkey is a new key (does not exist elsewhere.)
        ensure!(
            !Self::coldkey_has_associated_hotkeys(new_coldkey),
            Error::<T>::ColdKeyAlreadyAssociated
        );
        // Check that the new coldkey is not a hotkey.
        ensure!(
            !Self::hotkey_account_exists(new_coldkey),
            Error::<T>::ColdKeyAlreadyAssociated
        );

        // Actually do the swap.
        weight = weight.saturating_add(
            Self::perform_swap_coldkey(old_coldkey, new_coldkey)
                .map_err(|_| Error::<T>::ColdkeySwapError)?,
        );

        Self::set_last_tx_block(new_coldkey, Self::get_current_block_as_u64());
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        Self::deposit_event(Event::ColdkeySwapped {
            old_coldkey: old_coldkey.clone(),
            new_coldkey: new_coldkey.clone(),
        });

        Ok(Some(weight).into())
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
    /// This function will return an error if:
    /// - The old coldkey is the same as the new coldkey.
    /// - The new coldkey is already in the list of destination coldkeys.
    /// - There are already 2 destination coldkeys for the old coldkey.
    /// - The old coldkey doesn't have the minimum required TAO balance.
    /// - The proof of work is invalid or doesn't meet the required difficulty.
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

        // Check minimum amount of TAO (1 TAO)
        ensure!(
            Self::get_coldkey_balance(old_coldkey) >= MIN_BALANCE_TO_PERFORM_COLDKEY_SWAP,
            Error::<T>::InsufficientBalanceToPerformColdkeySwap
        );

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

        // If the destinations keys are empty or have size 1 then we will add the new coldkey to the list
        if destination_coldkeys.is_empty() || destination_coldkeys.len() == 1_usize {
            destination_coldkeys.push(new_coldkey.clone());
            ColdkeySwapDestinations::<T>::insert(old_coldkey.clone(), destination_coldkeys.clone());
        } else {
            return Err(Error::<T>::ColdkeyIsInArbitration.into());
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
    fn calculate_pow_difficulty(swap_attempts: u32) -> U256 {
        let base_difficulty: U256 = U256::from(10_000_000); // Base difficulty
        base_difficulty * U256::from(2).pow(U256::from(swap_attempts))
    }

    /// Arbitrates coldkeys that are scheduled to be swapped on this block.
    ///
    /// This function retrieves the list of coldkeys scheduled to be swapped on the current block,
    /// and processes each coldkey by either extending the arbitration period or performing the swap
    /// to the new coldkey.
    ///
    /// # Returns
    ///
    /// * `Weight` - The total weight consumed by the operation.
    pub fn swap_coldkeys_this_block() -> Result<Weight, &'static str> {
        let mut weight = frame_support::weights::Weight::from_parts(0, 0);

        // Get the block number
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!("Swapping coldkeys for block: {:?}", current_block);

        // Get the coldkeys to swap here and then remove them.
        let source_coldkeys: Vec<T::AccountId> = ColdkeysToSwapAtBlock::<T>::get(current_block);
        ColdkeysToSwapAtBlock::<T>::remove(current_block);
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        // Iterate over all keys in swap and call perform_swap_coldkey for each
        for coldkey_i in source_coldkeys.iter() {
            // Get the wallets to swap to for this coldkey.
            let destinations_coldkeys: Vec<T::AccountId> =
                ColdkeySwapDestinations::<T>::get(coldkey_i);
            ColdkeySwapDestinations::<T>::remove(&coldkey_i);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

            // If the wallets to swap is > 1 we bump the arbitration period.
            if destinations_coldkeys.len() > 1 {
                // Set the arbitration period to u64::MAX until we have a senate vote
                ColdkeyArbitrationBlock::<T>::insert(coldkey_i.clone(), u64::MAX);

                Self::deposit_event(Event::ArbitrationPeriodExtended {
                    coldkey: coldkey_i.clone(),
                });
            } else if let Some(new_coldkey) = destinations_coldkeys.first() {
                // ONLY 1 wallet: Get the wallet to swap to.
                // Perform the swap.
                if Self::perform_swap_coldkey(coldkey_i, new_coldkey)
                    .map(|w| weight = weight.saturating_add(w))
                    .is_err()
                {
                    return Err("Failed to perform coldkey swap");
                }
            }
        }

        Ok(weight)
    }

    pub fn perform_swap_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) -> Result<Weight, &'static str> {
        log::info!(
            "Performing swap for coldkey: {:?} to {:?}",
            old_coldkey,
            new_coldkey
        );
        // Init the weight.
        let mut weight = frame_support::weights::Weight::from_parts(0, 0);

        // Swap coldkey references in storage maps
        // NOTE The order of these calls is important
        Self::swap_total_coldkey_stake(old_coldkey, new_coldkey, &mut weight);
        Self::swap_stake_for_coldkey(old_coldkey, new_coldkey, &mut weight);
        Self::swap_total_hotkey_coldkey_stakes_this_interval_for_coldkey(
            old_coldkey,
            new_coldkey,
            &mut weight,
        );
        Self::swap_subnet_owner_for_coldkey(old_coldkey, new_coldkey, &mut weight);

        // Transfer any remaining balance from old_coldkey to new_coldkey
        let remaining_balance = Self::get_coldkey_balance(old_coldkey);
        if remaining_balance > 0 {
            if let Err(e) = Self::kill_coldkey_account(old_coldkey, remaining_balance) {
                return Err(e.into());
            }
            Self::add_balance_to_coldkey_account(new_coldkey, remaining_balance);
        }

        // Swap the coldkey.
        let total_balance: u64 = Self::get_coldkey_balance(old_coldkey);
        if total_balance > 0 {
            // Attempt to transfer the entire total balance to coldkeyB.
            if let Err(e) = T::Currency::transfer(
                old_coldkey,
                new_coldkey,
                total_balance,
                Preservation::Expendable,
            ) {
                return Err(e.into());
            }
        }

        Ok(weight)
    }

    /// Retrieves the network membership status for a given hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The hotkey to check for network membership.
    ///
    /// # Returns
    ///
    /// * `Vec<u16>` - A vector of network IDs where the hotkey is a member.
    pub fn get_netuid_is_member(old_hotkey: &T::AccountId, weight: &mut Weight) -> Vec<u16> {
        let netuid_is_member: Vec<u16> =
            <IsNetworkMember<T> as IterableStorageDoubleMap<_, _, _>>::iter_prefix(old_hotkey)
                .map(|(netuid, _)| netuid)
                .collect();
        weight.saturating_accrue(T::DbWeight::get().reads(netuid_is_member.len() as u64));
        netuid_is_member
    }

    /// Swaps the owner of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `coldkey` - The coldkey owning the hotkey.
    /// * `weight` - The weight of the transaction.
    ///
    pub fn swap_owner(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        Owner::<T>::remove(old_hotkey);
        Owner::<T>::insert(new_hotkey, coldkey.clone());

        // Update OwnedHotkeys map
        let mut hotkeys = OwnedHotkeys::<T>::get(coldkey);
        if !hotkeys.contains(new_hotkey) {
            hotkeys.push(new_hotkey.clone());
        }
        hotkeys.retain(|hk| *hk != *old_hotkey);
        OwnedHotkeys::<T>::insert(coldkey, hotkeys);

        weight.saturating_accrue(T::DbWeight::get().writes(2));
    }

    /// Swaps the total stake of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `weight` - The weight of the transaction.
    ///
    /// # Weight Calculation
    ///
    /// * Reads: 1 if the old hotkey exists, otherwise 1 for the failed read.
    /// * Writes: 2 if the old hotkey exists (one for removal and one for insertion).
    pub fn swap_total_hotkey_stake(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        if let Ok(total_hotkey_stake) = TotalHotkeyStake::<T>::try_get(old_hotkey) {
            TotalHotkeyStake::<T>::remove(old_hotkey);
            TotalHotkeyStake::<T>::insert(new_hotkey, total_hotkey_stake);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        } else {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
        }
    }

    /// Swaps the delegates of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `weight` - The weight of the transaction.
    ///
    /// # Weight Calculation
    ///
    /// * Reads: 1 if the old hotkey exists, otherwise 1 for the failed read.
    /// * Writes: 2 if the old hotkey exists (one for removal and one for insertion).
    pub fn swap_delegates(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        if let Ok(delegate_take) = Delegates::<T>::try_get(old_hotkey) {
            Delegates::<T>::remove(old_hotkey);
            Delegates::<T>::insert(new_hotkey, delegate_take);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        } else {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
        }
    }

    /// Swaps the stake of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `weight` - The weight of the transaction.
    pub fn swap_stake(old_hotkey: &T::AccountId, new_hotkey: &T::AccountId, weight: &mut Weight) {
        let mut writes: u64 = 0;
        let stakes: Vec<(T::AccountId, u64)> = Stake::<T>::iter_prefix(old_hotkey).collect();
        let stake_count = stakes.len() as u32;

        for (coldkey, stake_amount) in stakes {
            Stake::<T>::insert(new_hotkey, &coldkey, stake_amount);
            writes = writes.saturating_add(1u64); // One write for insert

            // Update StakingHotkeys map
            let mut staking_hotkeys = StakingHotkeys::<T>::get(&coldkey);
            if !staking_hotkeys.contains(new_hotkey) {
                staking_hotkeys.push(new_hotkey.clone());
                writes = writes.saturating_add(1u64); // One write for insert
            }
            if let Some(pos) = staking_hotkeys.iter().position(|x| x == old_hotkey) {
                staking_hotkeys.remove(pos);
                writes = writes.saturating_add(1u64); // One write for remove
            }
            StakingHotkeys::<T>::insert(coldkey.clone(), staking_hotkeys);
            writes = writes.saturating_add(1u64); // One write for insert
        }

        // Clear the prefix for the old hotkey after transferring all stakes
        let _ = Stake::<T>::clear_prefix(old_hotkey, stake_count, None);
        writes = writes.saturating_add(1); // One write for insert; // One write for clear_prefix

        // TODO: Remove all entries for old hotkey from StakingHotkeys map

        weight.saturating_accrue(T::DbWeight::get().writes(writes));
    }

    /// Swaps the network membership status of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `netuid_is_member` - A vector of network IDs where the hotkey is a member.
    /// * `weight` - The weight of the transaction.
    pub fn swap_is_network_member(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid_is_member: &[u16],
        weight: &mut Weight,
    ) {
        let _ = IsNetworkMember::<T>::clear_prefix(old_hotkey, netuid_is_member.len() as u32, None);
        weight.saturating_accrue(T::DbWeight::get().writes(netuid_is_member.len() as u64));
        for netuid in netuid_is_member.iter() {
            IsNetworkMember::<T>::insert(new_hotkey, netuid, true);
            weight.saturating_accrue(T::DbWeight::get().writes(1));
        }
    }

    /// Swaps the axons of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `netuid_is_member` - A vector of network IDs where the hotkey is a member.
    /// * `weight` - The weight of the transaction.
    ///
    /// # Weight Calculation
    ///
    /// * Reads: 1 for each network ID if the old hotkey exists in that network.
    /// * Writes: 2 for each network ID if the old hotkey exists in that network (one for removal and one for insertion).
    pub fn swap_axons(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid_is_member: &[u16],
        weight: &mut Weight,
    ) {
        for netuid in netuid_is_member.iter() {
            if let Ok(axon_info) = Axons::<T>::try_get(netuid, old_hotkey) {
                Axons::<T>::remove(netuid, old_hotkey);
                Axons::<T>::insert(netuid, new_hotkey, axon_info);
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
            } else {
                weight.saturating_accrue(T::DbWeight::get().reads(1));
            }
        }
    }

    /// Swaps the references in the keys storage map of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `netuid_is_member` - A vector of network IDs where the hotkey is a member.
    /// * `weight` - The weight of the transaction.
    pub fn swap_keys(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid_is_member: &[u16],
        weight: &mut Weight,
    ) {
        let mut writes: u64 = 0;
        for netuid in netuid_is_member {
            let keys: Vec<(u16, T::AccountId)> = Keys::<T>::iter_prefix(netuid).collect();
            for (uid, key) in keys {
                if key == *old_hotkey {
                    log::info!("old hotkey found: {:?}", old_hotkey);
                    Keys::<T>::insert(netuid, uid, new_hotkey.clone());
                }
                writes = writes.saturating_add(2u64);
            }
        }
        log::info!("writes: {:?}", writes);
        weight.saturating_accrue(T::DbWeight::get().writes(writes));
    }

    /// Swaps the loaded emission of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `netuid_is_member` - A vector of network IDs where the hotkey is a member.
    /// * `weight` - The weight of the transaction.
    ///
    pub fn swap_loaded_emission(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid_is_member: &[u16],
        weight: &mut Weight,
    ) {
        for netuid in netuid_is_member {
            if let Some(mut emissions) = LoadedEmission::<T>::get(netuid) {
                for emission in emissions.iter_mut() {
                    if emission.0 == *old_hotkey {
                        emission.0 = new_hotkey.clone();
                    }
                }
                LoadedEmission::<T>::insert(netuid, emissions);
            }
        }
        weight.saturating_accrue(T::DbWeight::get().writes(netuid_is_member.len() as u64));
    }

    /// Swaps the UIDs of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `netuid_is_member` - A vector of network IDs where the hotkey is a member.
    /// * `weight` - The weight of the transaction.
    ///
    pub fn swap_uids(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid_is_member: &[u16],
        weight: &mut Weight,
    ) {
        for netuid in netuid_is_member.iter() {
            if let Ok(uid) = Uids::<T>::try_get(netuid, old_hotkey) {
                Uids::<T>::remove(netuid, old_hotkey);
                Uids::<T>::insert(netuid, new_hotkey, uid);
                weight.saturating_accrue(T::DbWeight::get().writes(2));
            }
        }
    }

    /// Swaps the Prometheus data of the hotkey.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `netuid_is_member` - A vector of network IDs where the hotkey is a member.
    /// * `weight` - The weight of the transaction.
    ///
    /// # Weight Calculation
    ///
    /// * Reads: 1 for each network ID if the old hotkey exists in that network.
    /// * Writes: 2 for each network ID if the old hotkey exists in that network (one for removal and one for insertion).
    pub fn swap_prometheus(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid_is_member: &[u16],
        weight: &mut Weight,
    ) {
        for netuid in netuid_is_member.iter() {
            if let Ok(prometheus_info) = Prometheus::<T>::try_get(netuid, old_hotkey) {
                Prometheus::<T>::remove(netuid, old_hotkey);
                Prometheus::<T>::insert(netuid, new_hotkey, prometheus_info);
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
            } else {
                weight.saturating_accrue(T::DbWeight::get().reads(1));
            }
        }
    }

    /// Swaps the total hotkey-coldkey stakes for the current interval.
    ///
    /// # Arguments
    ///
    /// * `old_hotkey` - The old hotkey.
    /// * `new_hotkey` - The new hotkey.
    /// * `weight` - The weight of the transaction.
    ///
    pub fn swap_total_hotkey_coldkey_stakes_this_interval(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        let stakes: Vec<(T::AccountId, (u64, u64))> =
            TotalHotkeyColdkeyStakesThisInterval::<T>::iter_prefix(old_hotkey).collect();
        log::info!("Stakes to swap: {:?}", stakes);
        for (coldkey, stake) in stakes {
            log::info!(
                "Swapping stake for coldkey: {:?}, stake: {:?}",
                coldkey,
                stake
            );
            TotalHotkeyColdkeyStakesThisInterval::<T>::insert(new_hotkey, &coldkey, stake);
            TotalHotkeyColdkeyStakesThisInterval::<T>::remove(old_hotkey, &coldkey);
            weight.saturating_accrue(T::DbWeight::get().writes(2)); // One write for insert and one for remove
        }
    }

    /// Swaps the total stake associated with a coldkey from the old coldkey to the new coldkey.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The AccountId of the old coldkey.
    /// * `new_coldkey` - The AccountId of the new coldkey.
    /// * `weight` - Mutable reference to the weight of the transaction.
    ///
    /// # Effects
    ///
    /// * Removes the total stake from the old coldkey.
    /// * Inserts the total stake for the new coldkey.
    /// * Updates the transaction weight.
    pub fn swap_total_coldkey_stake(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        let stake = TotalColdkeyStake::<T>::get(old_coldkey);
        TotalColdkeyStake::<T>::remove(old_coldkey);
        TotalColdkeyStake::<T>::insert(new_coldkey, stake);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
    }

    /// Swaps all stakes associated with a coldkey from the old coldkey to the new coldkey.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The AccountId of the old coldkey.
    /// * `new_coldkey` - The AccountId of the new coldkey.
    /// * `weight` - Mutable reference to the weight of the transaction.
    ///
    /// # Effects
    ///
    /// * Removes all stakes associated with the old coldkey.
    /// * Inserts all stakes for the new coldkey.
    /// * Updates the transaction weight.
    pub fn swap_stake_for_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        // Swap the owners.
        let old_owned_hotkeys = OwnedHotkeys::<T>::get(old_coldkey);
        for owned_key in old_owned_hotkeys.clone().iter() {
            Owner::<T>::insert(owned_key, new_coldkey);
            // Find all hotkeys for this coldkey
            let hotkeys = OwnedHotkeys::<T>::get(old_coldkey);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 0));
            for hotkey in hotkeys.iter() {
                let stake = Stake::<T>::get(&hotkey, old_coldkey);
                Stake::<T>::remove(&hotkey, old_coldkey);
                Stake::<T>::insert(&hotkey, new_coldkey, stake);

                // Update StakingHotkeys map
                let staking_hotkeys = StakingHotkeys::<T>::get(old_coldkey);
                StakingHotkeys::<T>::insert(new_coldkey.clone(), staking_hotkeys);

                weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 3));
            }
            OwnedHotkeys::<T>::remove(old_coldkey.clone());
            OwnedHotkeys::<T>::insert(new_coldkey.clone(), old_owned_hotkeys.clone());

            // Swap all the keys the coldkey is staking too.
            let staking_hotkeys = StakingHotkeys::<T>::get(old_coldkey);
            StakingHotkeys::<T>::remove(old_coldkey.clone());
            for hotkey in staking_hotkeys.iter() {
                // Remove the previous stake and re-insert it.
                let stake = Stake::<T>::get(hotkey, old_coldkey);
                Stake::<T>::remove(hotkey, old_coldkey);
                Stake::<T>::insert(hotkey, new_coldkey, stake);
                weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 3));
            }
            // Add the new staking keys value.
            StakingHotkeys::<T>::insert(new_coldkey.clone(), staking_hotkeys.clone());
        }
    }

    /// Swaps the total hotkey-coldkey stakes for the current interval from the old coldkey to the new coldkey.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The AccountId of the old coldkey.
    /// * `new_coldkey` - The AccountId of the new coldkey.
    /// * `weight` - Mutable reference to the weight of the transaction.
    ///
    /// # Effects
    ///
    /// * Removes all total hotkey-coldkey stakes for the current interval associated with the old coldkey.
    /// * Inserts all total hotkey-coldkey stakes for the current interval for the new coldkey.
    /// * Updates the transaction weight.
    pub fn swap_total_hotkey_coldkey_stakes_this_interval_for_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 0));
        for hotkey in OwnedHotkeys::<T>::get(old_coldkey).iter() {
            let (stake, block) =
                TotalHotkeyColdkeyStakesThisInterval::<T>::get(&hotkey, old_coldkey);
            TotalHotkeyColdkeyStakesThisInterval::<T>::remove(&hotkey, old_coldkey);
            TotalHotkeyColdkeyStakesThisInterval::<T>::insert(&hotkey, new_coldkey, (stake, block));
            weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));
        }
    }

    /// Checks if a coldkey has any associated hotkeys.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - The AccountId of the coldkey to check.
    ///
    /// # Returns
    ///
    /// * `bool` - True if the coldkey has any associated hotkeys, false otherwise.
    pub fn coldkey_has_associated_hotkeys(coldkey: &T::AccountId) -> bool {
        !StakingHotkeys::<T>::get(coldkey).is_empty()
    }

    /// Swaps the subnet owner from the old coldkey to the new coldkey for all networks where the old coldkey is the owner.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The AccountId of the old coldkey.
    /// * `new_coldkey` - The AccountId of the new coldkey.
    /// * `weight` - Mutable reference to the weight of the transaction.
    ///
    /// # Effects
    ///
    /// * Updates the subnet owner to the new coldkey for all networks where the old coldkey was the owner.
    /// * Updates the transaction weight.
    pub fn swap_subnet_owner_for_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        for netuid in 0..=TotalNetworks::<T>::get() {
            let subnet_owner = SubnetOwner::<T>::get(netuid);
            if subnet_owner == *old_coldkey {
                SubnetOwner::<T>::insert(netuid, new_coldkey.clone());
                weight.saturating_accrue(T::DbWeight::get().writes(1));
            }
        }
        weight.saturating_accrue(T::DbWeight::get().reads(TotalNetworks::<T>::get() as u64));
    }
}
