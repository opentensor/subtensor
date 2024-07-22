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
        // Ensure the origin is signed and get the coldkey
        let coldkey = ensure_signed(origin)?;

        // Check if the coldkey is in arbitration
        ensure!(
            !Self::coldkey_in_arbitration(&coldkey),
            Error::<T>::ColdkeyIsInArbitration
        );

        // Initialize the weight for this operation
        let mut weight = T::DbWeight::get().reads(2);

        // Ensure the new hotkey is different from the old one
        ensure!(old_hotkey != new_hotkey, Error::<T>::NewHotKeyIsSameWithOld);
        // Ensure the new hotkey is not already registered on any network
        ensure!(
            !Self::is_hotkey_registered_on_any_network(new_hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // Update the weight for the checks above
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 0));
        // Ensure the coldkey owns the old hotkey
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, old_hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // Get the current block number
        let block: u64 = Self::get_current_block_as_u64();
        // Ensure the transaction rate limit is not exceeded
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::HotKeySetTxRateLimitExceeded
        );

        // Update the weight for reading the total networks
        weight.saturating_accrue(
            T::DbWeight::get().reads((TotalNetworks::<T>::get().saturating_add(1u16)) as u64),
        );

        // Get the cost for swapping the key
        let swap_cost = Self::get_key_swap_cost();
        log::debug!("Swap cost: {:?}", swap_cost);

        // Ensure the coldkey has enough balance to pay for the swap
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, swap_cost),
            Error::<T>::NotEnoughBalanceToPaySwapHotKey
        );
        // Remove the swap cost from the coldkey's account
        let actual_burn_amount = Self::remove_balance_from_coldkey_account(&coldkey, swap_cost)?;
        // Burn the tokens
        Self::burn_tokens(actual_burn_amount);

        // Perform the hotkey swap
        Self::perform_hotkey_swap(old_hotkey, new_hotkey, &coldkey, &mut weight);

        // Update the last transaction block for the coldkey
        Self::set_last_tx_block(&coldkey, block);
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        // Emit an event for the hotkey swap
        Self::deposit_event(Event::HotkeySwapped {
            coldkey,
            old_hotkey: old_hotkey.clone(),
            new_hotkey: new_hotkey.clone(),
        });

        // Return the weight of the operation
        Ok(Some(weight).into())
    }

    pub fn perform_hotkey_swap( old_hotkey: &T::AccountId, new_hotkey: &T::AccountId, coldkey: &T::AccountId, weight: &mut Weight ) -> DispatchResult {

        // 1. Swap owner.
        // Owner( hotkey ) -> coldkey -- the coldkey that owns the hotkey.
        Owner::<T>::remove(old_hotkey);
        Owner::<T>::insert(new_hotkey, coldkey.clone());
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 2. Swap OwnedHotkeys.
        // OwnedHotkeys( coldkey ) -> Vec<hotkey> -- the hotkeys that the coldkey owns.
        let mut hotkeys = OwnedHotkeys::<T>::get(coldkey);
        // Add the new key if needed.
        if !hotkeys.contains(new_hotkey) {
            hotkeys.push(new_hotkey.clone());
        } 
        // Remove the old key.
        hotkeys.retain(|hk| *hk != *old_hotkey);
        OwnedHotkeys::<T>::insert(coldkey, hotkeys);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 3. Swap total hotkey stake.
        // TotalHotkeyStake( hotkey ) -> stake -- the total stake that the hotkey has across all delegates.
        let old_total_hotkey_stake = TotalHotkeyStake::<T>::get( old_hotkey ); // Get the old total hotkey stake.
        let new_total_hotkey_stake = TotalHotkeyStake::<T>::get( new_hotkey ); // Get the new total hotkey stake.
        TotalHotkeyStake::<T>::remove( old_hotkey ); // Remove the old total hotkey stake.
        TotalHotkeyStake::<T>::insert( new_hotkey, old_total_hotkey_stake.saturating_add( new_total_hotkey_stake ) ); // Insert the new total hotkey stake via the addition.
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // Swap total hotkey stakes.
        // TotalHotkeyColdkeyStakesThisInterval( hotkey ) --> (u64: stakes, u64: block_number)
        let stake_tuples: Vec<(T::AccountId, (u64, u64))> = TotalHotkeyColdkeyStakesThisInterval::<T>::iter_prefix(old_hotkey).collect();
        for (coldkey, stake_tup) in stake_tuples {
            // NOTE: You could use this to increase your allowed stake operations but this would cost.
            TotalHotkeyColdkeyStakesThisInterval::<T>::insert(new_hotkey, &coldkey, stake_tup);
            TotalHotkeyColdkeyStakesThisInterval::<T>::remove(old_hotkey, &coldkey);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }

        // Swap LastTxBlock
        // LastTxBlock( hotkey ) --> u64 -- the last transaction block for the hotkey.
        let old_last_tx_block: u64 = LastTxBlock::<T>::get( old_hotkey );
        LastTxBlock::<T>::remove( old_hotkey );
        LastTxBlock::<T>::insert( new_hotkey, Self::get_current_block_as_u64() );
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

        // Swap LastTxBlockDelegateTake
        // LastTxBlockDelegateTake( hotkey ) --> u64 -- the last transaction block for the hotkey delegate take.
        LastTxBlockDelegateTake::<T>::remove( old_hotkey );
        LastTxBlockDelegateTake::<T>::insert( new_hotkey, Self::get_current_block_as_u64() );
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

        // Swap Senate members.
        // Senate( hotkey ) --> ?
        if T::SenateMembers::is_member(old_hotkey) {
            T::SenateMembers::swap_member(old_hotkey, new_hotkey).map_err(|e| e.error)?;
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }

        // 4. Swap delegates.
        // Delegates( hotkey ) -> take value -- the hotkey delegate take value.
        let old_delegate_take = Delegates::<T>::get( old_hotkey );
        Delegates::<T>::remove( old_hotkey ); // Remove the old delegate take.
        Delegates::<T>::insert( new_hotkey, old_delegate_take ); // Insert the new delegate take.
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

        // Swap all subnet specific info.
        let all_netuids: Vec<u16> = Self::get_all_subnet_netuids();
        for netuid in all_netuids { 
            // 7.1 Remove the previous hotkey and insert the new hotkey from membership.
            // IsNetworkMember( hotkey, netuid ) -> bool -- is the hotkey a subnet member.
            let is_network_member: bool = IsNetworkMember::<T>::get( old_hotkey, netuid );
            IsNetworkMember::<T>::remove( old_hotkey, netuid );
            IsNetworkMember::<T>::insert( new_hotkey, netuid, is_network_member );
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

            // 7.2 Swap Uids + Keys.
            // Keys( netuid, hotkey ) -> uid -- the uid the hotkey has in the network if it is a member.
            // Uids( netuid, hotkey ) -> uid -- the uids that the hotkey has.
            if is_network_member {
                // 7.2.1 Swap the UIDS
                if let Ok(old_uid) = Uids::<T>::try_get(netuid, old_hotkey) {
                    Uids::<T>::remove(netuid, old_hotkey);
                    Uids::<T>::insert(netuid, new_hotkey, old_uid);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

                    // 7.2.2 Swap the keys.
                    Keys::<T>::insert(netuid, old_uid, new_hotkey.clone());
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
                }
            }

            // 7.3 Swap Prometheus.
            // Prometheus( netuid, hotkey ) -> prometheus -- the prometheus data that a hotkey has in the network.
            if is_network_member {
                if let Ok(old_prometheus_info) = Prometheus::<T>::try_get(netuid, old_hotkey) {
                    Prometheus::<T>::remove(netuid, old_hotkey);
                    Prometheus::<T>::insert(netuid, new_hotkey, old_prometheus_info);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
                }
            }

            // 7.4. Swap axons.
            // Axons( netuid, hotkey ) -> axon -- the axon that the hotkey has.
            if is_network_member {
                if let Ok(old_axon_info) = Axons::<T>::try_get(netuid, old_hotkey) {
                    Axons::<T>::remove(netuid, old_hotkey);
                    Axons::<T>::insert(netuid, new_hotkey, old_axon_info);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
                }
            }

            // 7.5 Swap WeightCommits     
            // WeightCommits( hotkey ) --> Vec<u64> -- the weight commits for the hotkey.
            if is_network_member {
                if let Ok(old_weight_commits) = WeightCommits::<T>::try_get(netuid, old_hotkey) {
                    WeightCommits::<T>::remove(netuid, old_hotkey);
                    WeightCommits::<T>::insert(netuid, new_hotkey, old_weight_commits);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
                }
            }

            // 7.5. Swap the subnet loaded emission.
            // LoadedEmission( netuid ) --> Vec<(hotkey, u64)> -- the loaded emission for the subnet.
            if is_network_member {
                if let Some(mut old_loaded_emission) = LoadedEmission::<T>::get(netuid) {
                    for emission in old_loaded_emission.iter_mut() {
                        if emission.0 == *old_hotkey {
                            emission.0 = new_hotkey.clone();
                        }
                    }
                    LoadedEmission::<T>::remove(netuid);
                    LoadedEmission::<T>::insert(netuid, old_loaded_emission);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
                }
            }

        }

        // Swap Stake.
        // Stake( hotkey, coldkey ) -> stake -- the stake that the hotkey controls on behalf of the coldkey.
        let stakes: Vec<(T::AccountId, u64)> = Stake::<T>::iter_prefix(old_hotkey).collect();
        // Clear the entire old prefix here.
        let _ = Stake::<T>::clear_prefix( old_hotkey, stakes.len() as u32, None );
        // Iterate over all the staking rows and insert them into the new hotkey.
        for (coldkey, old_stake_amount) in stakes {
            weight.saturating_accrue(T::DbWeight::get().reads(1));

            // Swap Stake value
            // Stake( hotkey, coldkey ) -> stake -- the stake that the hotkey controls on behalf of the coldkey.
            // Get the new stake value.
            let new_stake_value: u64 = Stake::<T>::get(new_hotkey, &coldkey);
            // Insert the new stake value.
            Stake::<T>::insert(new_hotkey, &coldkey, new_stake_value.saturating_add(old_stake_amount));
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

            // Swap StakingHotkeys.
            // StakingHotkeys( coldkey ) --> Vec<hotkey> -- the hotkeys that the coldkey stakes.
            let mut staking_hotkeys = StakingHotkeys::<T>::get(&coldkey);
            staking_hotkeys.retain(|hk| *hk != *old_hotkey && *hk != *new_hotkey);
            staking_hotkeys.push(new_hotkey.clone());
            StakingHotkeys::<T>::insert(coldkey.clone(), staking_hotkeys);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        }

        Ok(())
    }

    pub fn swap_senate_member(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        weight: &mut Weight,
    ) -> DispatchResult {
        weight.saturating_accrue(T::DbWeight::get().reads(1));
        if T::SenateMembers::is_member(old_hotkey) {
            T::SenateMembers::swap_member(old_hotkey, new_hotkey).map_err(|e| e.error)?;
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }
        Ok(())
    }
}
