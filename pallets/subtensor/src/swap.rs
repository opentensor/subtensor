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

        let swap_cost = Self::get_hotkey_swap_cost();
        log::debug!("Swap cost: {:?}", swap_cost);

        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, swap_cost),
            Error::<T>::NotEnoughBalanceToPaySwapHotKey
        );
        let actual_burn_amount = Self::remove_balance_from_coldkey_account(&coldkey, swap_cost)?;
        Self::burn_tokens(actual_burn_amount);

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
        }

        // Clear the prefix for the old hotkey after transferring all stakes
        let _ = Stake::<T>::clear_prefix(old_hotkey, stake_count, None);
        writes = writes.saturating_add(1); // One write for insert; // One write for clear_prefix

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
}
