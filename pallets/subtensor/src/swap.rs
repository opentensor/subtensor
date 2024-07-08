use super::*;
use frame_support::{storage::IterableStorageDoubleMap, weights::Weight};
use sp_core::Get;

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
        ensure_signed(origin)?;

        let mut weight = T::DbWeight::get().reads(2);

        // Check if the new coldkey is already associated with any hotkeys
        ensure!(
            !Self::coldkey_has_associated_hotkeys(new_coldkey),
            Error::<T>::ColdKeyAlreadyAssociated
        );

        let block: u64 = Self::get_current_block_as_u64();

        // Swap coldkey references in storage maps
        // NOTE The order of these calls is important
        Self::swap_total_coldkey_stake(old_coldkey, new_coldkey, &mut weight);
        Self::swap_stake_for_coldkey(old_coldkey, new_coldkey, &mut weight);
        Self::swap_owner_for_coldkey(old_coldkey, new_coldkey, &mut weight);
        Self::swap_total_hotkey_coldkey_stakes_this_interval_for_coldkey(
            old_coldkey,
            new_coldkey,
            &mut weight,
        );
        Self::swap_subnet_owner_for_coldkey(old_coldkey, new_coldkey, &mut weight);
        Self::swap_owned_for_coldkey(old_coldkey, new_coldkey, &mut weight);

        // Transfer any remaining balance from old_coldkey to new_coldkey
        let remaining_balance = Self::get_coldkey_balance(old_coldkey);
        if remaining_balance > 0 {
            Self::kill_coldkey_account(old_coldkey, remaining_balance)?;
            Self::add_balance_to_coldkey_account(new_coldkey, remaining_balance);
        }

        Self::set_last_tx_block(new_coldkey, block);
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        Self::deposit_event(Event::ColdkeySwapped {
            old_coldkey: old_coldkey.clone(),
            new_coldkey: new_coldkey.clone(),
        });

        Ok(Some(weight).into())
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
                StakingHotkeys::<T>::insert(coldkey.clone(), staking_hotkeys);
                writes = writes.saturating_add(1u64); // One write for insert
            }
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
    }

    /// Swaps the owner of all hotkeys from the old coldkey to the new coldkey.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The AccountId of the old coldkey.
    /// * `new_coldkey` - The AccountId of the new coldkey.
    /// * `weight` - Mutable reference to the weight of the transaction.
    ///
    /// # Effects
    ///
    /// * Updates the owner of all hotkeys associated with the old coldkey to the new coldkey.
    /// * Updates the transaction weight.
    pub fn swap_owner_for_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        let hotkeys = OwnedHotkeys::<T>::get(old_coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 0));
        for hotkey in hotkeys.iter() {
            Owner::<T>::insert(&hotkey, new_coldkey);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
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
        Owner::<T>::iter().any(|(_, owner)| owner == *coldkey)
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

    /// Swaps the owned hotkeys for the coldkey
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
    pub fn swap_owned_for_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        // Update OwnedHotkeys map with new coldkey
        let hotkeys = OwnedHotkeys::<T>::get(old_coldkey);
        OwnedHotkeys::<T>::remove(old_coldkey);
        OwnedHotkeys::<T>::insert(new_coldkey, hotkeys);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 2));
    }
}
