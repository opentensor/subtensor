use super::*;
use frame_support::weights::Weight;
use rate_limiting_interface::RateLimitingInterface;
use sp_core::Get;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{
    Currency, MechId, NetUid,
    rate_limiting::{self, RateLimitUsageKey},
};

impl<T: Config> Pallet<T> {
    /// Swaps the hotkey of a coldkey account.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the transaction, and also the coldkey account.
    /// * `old_hotkey` - The old hotkey to be swapped.
    /// * `new_hotkey` - The new hotkey to replace the old one.
    /// * `netuid` - The hotkey swap in a subnet or all subnets.
    ///
    /// # Returns
    ///
    /// * `DispatchResultWithPostInfo` - The result of the dispatch.
    ///
    /// # Errors
    ///
    /// * `NonAssociatedColdKey` - If the coldkey does not own the old hotkey.
    /// * `NewHotKeyIsSameWithOld` - If the new hotkey is the same as the old hotkey.
    /// * `HotKeyAlreadyRegisteredInSubNet` - If the new hotkey is already registered in the subnet.
    /// * `NotEnoughBalanceToPaySwapHotKey` - If there is not enough balance to pay for the swap.
    pub fn do_swap_hotkey(
        origin: T::RuntimeOrigin,
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid: Option<NetUid>,
    ) -> DispatchResultWithPostInfo {
        // 1. Ensure the origin is signed and get the coldkey
        let coldkey = ensure_signed(origin)?;

        // 2. Ensure the coldkey owns the old hotkey
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, old_hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // 3. Initialize the weight for this operation
        let mut weight = T::DbWeight::get().reads(2);

        // 4. Ensure the new hotkey is different from the old one
        ensure!(old_hotkey != new_hotkey, Error::<T>::NewHotKeyIsSameWithOld);

        // 7. Ensure the new hotkey is not already registered on any network
        ensure!(
            !Self::is_hotkey_registered_on_any_network(new_hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // 8. Swap last-seen
        let last_tx_block = T::RateLimiting::last_seen(
            rate_limiting::GROUP_SWAP_KEYS,
            Some(RateLimitUsageKey::Account(old_hotkey.clone())),
        );
        T::RateLimiting::set_last_seen(
            rate_limiting::GROUP_SWAP_KEYS,
            Some(RateLimitUsageKey::Account(new_hotkey.clone())),
            last_tx_block,
        );
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 10. Swap LastTxBlockChildKeyTake
        let last_tx_block_child_key_take: u64 = Self::get_last_tx_block_childkey_take(old_hotkey);
        Self::set_last_tx_block_childkey(new_hotkey, last_tx_block_child_key_take);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 11. fork for swap hotkey on a specific subnet case after do the common check
        if let Some(netuid) = netuid {
            return Self::swap_hotkey_on_subnet(&coldkey, old_hotkey, new_hotkey, netuid, weight);
        };

        // Start to do everything for swap hotkey on all subnets case
        // 12. Get the cost for swapping the key
        let swap_cost = Self::get_key_swap_cost();
        log::debug!("Swap cost: {swap_cost:?}");

        // 13. Ensure the coldkey has enough balance to pay for the swap
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, swap_cost.into()),
            Error::<T>::NotEnoughBalanceToPaySwapHotKey
        );

        weight.saturating_accrue(T::DbWeight::get().reads_writes(3, 0));

        // 14. Remove the swap cost from the coldkey's account
        let actual_recycle_amount =
            Self::remove_balance_from_coldkey_account(&coldkey, swap_cost.into())?;

        // 18. Recycle the tokens
        Self::recycle_tao(actual_recycle_amount);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 2));

        // 19. Perform the hotkey swap
        Self::perform_hotkey_swap_on_all_subnets(old_hotkey, new_hotkey, &coldkey, &mut weight)?;

        // 21. Emit an event for the hotkey swap
        Self::deposit_event(Event::HotkeySwapped {
            coldkey,
            old_hotkey: old_hotkey.clone(),
            new_hotkey: new_hotkey.clone(),
        });

        // 22. Return the weight of the operation
        Ok(Some(weight).into())
    }

    /// Performs the hotkey swap operation, transferring all associated data and state from the old hotkey to the new hotkey.
    ///
    /// This function executes a series of steps to ensure a complete transfer of all relevant information:
    /// 1. Swaps the owner of the hotkey.
    /// 2. Updates the list of owned hotkeys for the coldkey.
    /// 3. Transfers the total hotkey stake.
    /// 4. Moves all stake-related data for the interval.
    /// 5. Updates the last transaction block for the new hotkey.
    /// 6. Transfers the delegate take information.
    /// 7. Updates delegate information.
    /// 8. For each subnet:
    ///    - Updates network membership status.
    ///    - Transfers UID and key information.
    ///    - Moves Prometheus data.
    ///    - Updates axon information.
    ///    - Transfers weight commits.
    ///    - Updates loaded emission data.
    /// 9. Transfers all stake information, including updating staking hotkeys for each coldkey.
    ///
    /// Throughout the process, the function accumulates the computational weight of operations performed.
    ///
    /// # Arguments
    /// * `old_hotkey` - The AccountId of the current hotkey to be replaced.
    /// * `new_hotkey` - The AccountId of the new hotkey to replace the old one.
    /// * `coldkey` - The AccountId of the coldkey that owns both hotkeys.
    /// * `weight` - A mutable reference to the Weight, updated as operations are performed.
    ///
    /// # Returns
    /// * `DispatchResult` - Ok(()) if the swap was successful, or an error if any operation failed.
    ///
    /// # Note
    /// This function performs extensive storage reads and writes, which can be computationally expensive.
    /// The accumulated weight should be carefully considered in the context of block limits.
    pub fn perform_hotkey_swap_on_all_subnets(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        weight: &mut Weight,
    ) -> DispatchResult {
        // 1. keep the old hotkey alpha values for the case where hotkey staked by multiple coldkeys.
        let old_alpha_values: Vec<((T::AccountId, NetUid), U64F64)> =
            Alpha::<T>::iter_prefix((old_hotkey,)).collect();
        weight.saturating_accrue(T::DbWeight::get().reads(old_alpha_values.len() as u64));

        // 2. Swap owner.
        // Owner( hotkey ) -> coldkey -- the coldkey that owns the hotkey.
        Owner::<T>::remove(old_hotkey);
        Owner::<T>::insert(new_hotkey, coldkey.clone());
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 3. Swap OwnedHotkeys.
        // OwnedHotkeys( coldkey ) -> Vec<hotkey> -- the hotkeys that the coldkey owns.
        let mut hotkeys = OwnedHotkeys::<T>::get(coldkey);
        // Add the new key if needed.
        if !hotkeys.contains(new_hotkey) {
            hotkeys.push(new_hotkey.clone());
        }

        // 4. Remove the old key.
        hotkeys.retain(|hk| *hk != *old_hotkey);
        OwnedHotkeys::<T>::insert(coldkey, hotkeys);

        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 5. execute the hotkey swap on all subnets
        for netuid in Self::get_all_subnet_netuids() {
            Self::perform_hotkey_swap_on_one_subnet(old_hotkey, new_hotkey, weight, netuid)?;
        }

        // 6. Swap LastTxBlock
        // LastTxBlock( hotkey ) --> u64 -- the last transaction block for the hotkey.
        T::RateLimiting::set_last_seen(
            rate_limiting::GROUP_SWAP_KEYS,
            Some(RateLimitUsageKey::Account(old_hotkey.clone())),
            None,
        );
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        // 7. Swap LastTxBlockDelegateTake
        // LastTxBlockDelegateTake( hotkey ) --> u64 -- the last transaction block for the hotkey delegate take.
        Self::remove_last_tx_block_delegate_take(old_hotkey);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

        // 8. Swap LastTxBlockChildKeyTake
        // LastTxBlockChildKeyTake( hotkey ) --> u64 -- the last transaction block for the hotkey child key take.
        Self::remove_last_tx_block_childkey(old_hotkey);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

        // 9. Swap delegates.
        // Delegates( hotkey ) -> take value -- the hotkey delegate take value.
        if Delegates::<T>::contains_key(old_hotkey) {
            let old_delegate_take = Delegates::<T>::get(old_hotkey);
            Delegates::<T>::remove(old_hotkey);
            Delegates::<T>::insert(new_hotkey, old_delegate_take);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));
        }

        // 10. Alpha already update in perform_hotkey_swap_on_one_subnet
        // Update the StakingHotkeys for the case where hotkey staked by multiple coldkeys.
        for ((coldkey, _netuid), _alpha) in old_alpha_values {
            // Swap StakingHotkeys.
            // StakingHotkeys( coldkey ) --> Vec<hotkey> -- the hotkeys that the coldkey stakes.
            let mut staking_hotkeys = StakingHotkeys::<T>::get(&coldkey);
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            if staking_hotkeys.contains(old_hotkey) {
                staking_hotkeys.retain(|hk| *hk != *old_hotkey && *hk != *new_hotkey);
                if !staking_hotkeys.contains(new_hotkey) {
                    staking_hotkeys.push(new_hotkey.clone());
                }
                StakingHotkeys::<T>::insert(&coldkey, staking_hotkeys);
                weight.saturating_accrue(T::DbWeight::get().writes(1));
            }
        }

        // Return successful after swapping all the relevant terms.
        Ok(())
    }

    fn swap_hotkey_on_subnet(
        coldkey: &T::AccountId,
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        netuid: NetUid,
        init_weight: Weight,
    ) -> DispatchResultWithPostInfo {
        // 1. Ensure coldkey not swap hotkey too frequently
        let mut weight: Weight = init_weight;
        let block: u64 = Self::get_current_block_as_u64();
        let hotkey_swap_interval = T::HotkeySwapOnSubnetInterval::get();
        let last_hotkey_swap_block = LastHotkeySwapOnNetuid::<T>::get(netuid, coldkey);

        // NOTE: This subnet interval gate is legacy swap-keys rate-limiting group behavior and
        // remains in pallet-subtensor; it is not migrated into pallet-rate-limiting because that
        // system supports only a single span per target.
        ensure!(
            last_hotkey_swap_block.saturating_add(hotkey_swap_interval) < block,
            Error::<T>::HotKeySwapOnSubnetIntervalNotPassed
        );
        weight.saturating_accrue(T::DbWeight::get().reads_writes(3, 0));

        // 2. Ensure the hotkey not registered on the network before.
        ensure!(
            !Self::is_hotkey_registered_on_specific_network(new_hotkey, netuid),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 0));

        // 3. Get the cost for swapping the key on the subnet
        let swap_cost = T::KeySwapOnSubnetCost::get();
        log::debug!("Swap cost in subnet {netuid:?}: {swap_cost:?}");
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 0));

        // 4. Ensure the coldkey has enough balance to pay for the swap
        ensure!(
            Self::can_remove_balance_from_coldkey_account(coldkey, swap_cost),
            Error::<T>::NotEnoughBalanceToPaySwapHotKey
        );

        // 5. Remove the swap cost from the coldkey's account
        let actual_recycle_amount = Self::remove_balance_from_coldkey_account(coldkey, swap_cost)?;
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 0));

        // 6. Recycle the tokens
        Self::recycle_tao(actual_recycle_amount);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 7. Swap owner.
        // Owner( hotkey ) -> coldkey -- the coldkey that owns the hotkey.
        // Owner::<T>::remove(old_hotkey);
        Owner::<T>::insert(new_hotkey, coldkey.clone());
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 8. Swap OwnedHotkeys.
        // OwnedHotkeys( coldkey ) -> Vec<hotkey> -- the hotkeys that the coldkey owns.
        let mut hotkeys = OwnedHotkeys::<T>::get(coldkey);
        // Add the new key if needed.
        if !hotkeys.contains(new_hotkey) {
            hotkeys.push(new_hotkey.clone());
            OwnedHotkeys::<T>::insert(coldkey, hotkeys);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));
        }

        // 9. Perform the hotkey swap
        Self::perform_hotkey_swap_on_one_subnet(old_hotkey, new_hotkey, &mut weight, netuid)?;

        // 10. Update the last transaction block for the coldkey
        LastHotkeySwapOnNetuid::<T>::insert(netuid, coldkey, block);
        weight.saturating_accrue(T::DbWeight::get().writes(2));

        // 11. Emit an event for the hotkey swap
        Self::deposit_event(Event::HotkeySwappedOnSubnet {
            coldkey: coldkey.clone(),
            old_hotkey: old_hotkey.clone(),
            new_hotkey: new_hotkey.clone(),
            netuid,
        });

        Ok(Some(weight).into())
    }

    // do hotkey swap public part for both swap all subnets and just swap one subnet
    pub fn perform_hotkey_swap_on_one_subnet(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        weight: &mut Weight,
        netuid: NetUid,
    ) -> DispatchResult {
        // 1. Swap total hotkey alpha for all subnets it exists on.
        // TotalHotkeyAlpha( hotkey, netuid ) -> alpha -- the total alpha that the hotkey has on a specific subnet.
        let alpha = TotalHotkeyAlpha::<T>::take(old_hotkey, netuid);

        TotalHotkeyAlpha::<T>::mutate(new_hotkey, netuid, |value| {
            *value = value.saturating_add(alpha)
        });
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 2. Swap total hotkey shares on all subnets it exists on.
        // TotalHotkeyShares( hotkey, netuid ) -> alpha -- the total alpha that the hotkey has on a specific subnet.
        let share = TotalHotkeyShares::<T>::take(old_hotkey, netuid);
        // TotalHotkeyAlpha::<T>::remove(old_hotkey, netuid);
        TotalHotkeyShares::<T>::mutate(new_hotkey, netuid, |value| {
            *value = value.saturating_add(share)
        });
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // 3. Swap all subnet specific info.

        // 3.1 Remove the previous hotkey and insert the new hotkey from membership.
        // IsNetworkMember( hotkey, netuid ) -> bool -- is the hotkey a subnet member.
        let is_network_member: bool = IsNetworkMember::<T>::get(old_hotkey, netuid);
        IsNetworkMember::<T>::remove(old_hotkey, netuid);
        IsNetworkMember::<T>::insert(new_hotkey, netuid, is_network_member);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

        // 3.2 Swap Uids + Keys.
        // Keys( netuid, hotkey ) -> uid -- the uid the hotkey has in the network if it is a member.
        // Uids( netuid, hotkey ) -> uid -- the uids that the hotkey has.
        if is_network_member {
            // 3.2.1 Swap the UIDS
            if let Ok(old_uid) = Uids::<T>::try_get(netuid, old_hotkey) {
                Uids::<T>::remove(netuid, old_hotkey);
                Uids::<T>::insert(netuid, new_hotkey, old_uid);
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

                // 3.2.2 Swap the keys.
                Keys::<T>::insert(netuid, old_uid, new_hotkey.clone());
                weight.saturating_accrue(T::DbWeight::get().reads_writes(0, 1));
            }
        }

        // 3.3 Swap Prometheus.
        // Prometheus( netuid, hotkey ) -> prometheus -- the prometheus data that a hotkey has in the network.
        if is_network_member
            && let Ok(old_prometheus_info) = Prometheus::<T>::try_get(netuid, old_hotkey)
        {
            Prometheus::<T>::remove(netuid, old_hotkey);
            Prometheus::<T>::insert(netuid, new_hotkey, old_prometheus_info);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }

        // 3.4. Swap axons.
        // Axons( netuid, hotkey ) -> axon -- the axon that the hotkey has.
        if is_network_member && let Ok(old_axon_info) = Axons::<T>::try_get(netuid, old_hotkey) {
            Axons::<T>::remove(netuid, old_hotkey);
            Axons::<T>::insert(netuid, new_hotkey, old_axon_info);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }

        // 3.5 Swap WeightCommits
        // WeightCommits( hotkey ) --> Vec<u64> -- the weight commits for the hotkey.
        if is_network_member {
            for mecid in 0..MechanismCountCurrent::<T>::get(netuid).into() {
                let netuid_index = Self::get_mechanism_storage_index(netuid, MechId::from(mecid));
                if let Ok(old_weight_commits) =
                    WeightCommits::<T>::try_get(netuid_index, old_hotkey)
                {
                    WeightCommits::<T>::remove(netuid_index, old_hotkey);
                    WeightCommits::<T>::insert(netuid_index, new_hotkey, old_weight_commits);
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
                }
            }
        }

        // 3.6. Swap the subnet loaded emission.
        // LoadedEmission( netuid ) --> Vec<(hotkey, u64)> -- the loaded emission for the subnet.
        if is_network_member && let Some(mut old_loaded_emission) = LoadedEmission::<T>::get(netuid)
        {
            for emission in old_loaded_emission.iter_mut() {
                if emission.0 == *old_hotkey {
                    emission.0 = new_hotkey.clone();
                }
            }
            LoadedEmission::<T>::remove(netuid);
            LoadedEmission::<T>::insert(netuid, old_loaded_emission);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }

        // 3.7. Swap neuron TLS certificates.
        // NeuronCertificates( netuid, hotkey ) -> Vec<u8> -- the neuron certificate for the hotkey.
        if is_network_member
            && let Ok(old_neuron_certificates) =
                NeuronCertificates::<T>::try_get(netuid, old_hotkey)
        {
            NeuronCertificates::<T>::remove(netuid, old_hotkey);
            NeuronCertificates::<T>::insert(netuid, new_hotkey, old_neuron_certificates);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
        }
        // 4. Swap ChildKeys.
        // 5. Swap ParentKeys.
        // 6. Swap PendingChildKeys.
        Self::parent_child_swap_hotkey(old_hotkey, new_hotkey, netuid, weight)?;

        // Also check for others with our hotkey as a child
        for (hotkey, (children, cool_down_block)) in PendingChildKeys::<T>::iter_prefix(netuid) {
            weight.saturating_accrue(T::DbWeight::get().reads(1));

            if let Some(potential_idx) =
                children.iter().position(|(_, child)| *child == *old_hotkey)
            {
                let mut new_children = children.clone();
                let entry_to_remove = new_children.remove(potential_idx);
                new_children.push((entry_to_remove.0, new_hotkey.clone())); // Keep the proportion.

                PendingChildKeys::<T>::remove(netuid, hotkey.clone());
                PendingChildKeys::<T>::insert(netuid, hotkey, (new_children, cool_down_block));
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));
            }
        }

        // 6.4 Swap AutoStakeDestination
        if let Ok(old_auto_stake_coldkeys) =
            AutoStakeDestinationColdkeys::<T>::try_get(old_hotkey, netuid)
        {
            // Move the vector from old hotkey to new hotkey.
            for coldkey in &old_auto_stake_coldkeys {
                AutoStakeDestination::<T>::insert(coldkey, netuid, new_hotkey);
            }
            AutoStakeDestinationColdkeys::<T>::remove(old_hotkey, netuid);
            AutoStakeDestinationColdkeys::<T>::insert(new_hotkey, netuid, old_auto_stake_coldkeys);
        }

        // 7. Swap SubnetOwnerHotkey
        // SubnetOwnerHotkey( netuid ) --> hotkey -- the hotkey that is the owner of the subnet.
        if let Ok(old_subnet_owner_hotkey) = SubnetOwnerHotkey::<T>::try_get(netuid) {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            if old_subnet_owner_hotkey == *old_hotkey {
                SubnetOwnerHotkey::<T>::insert(netuid, new_hotkey);
                weight.saturating_accrue(T::DbWeight::get().writes(1));
            }
        }

        // 8. Swap dividend records
        // 8.1 Swap TotalHotkeyAlphaLastEpoch
        let old_alpha = TotalHotkeyAlphaLastEpoch::<T>::take(old_hotkey, netuid);
        let new_total_hotkey_alpha = TotalHotkeyAlphaLastEpoch::<T>::get(new_hotkey, netuid);
        TotalHotkeyAlphaLastEpoch::<T>::insert(
            new_hotkey,
            netuid,
            old_alpha.saturating_add(new_total_hotkey_alpha),
        );
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // 8.2 Swap AlphaDividendsPerSubnet
        let old_hotkey_alpha_dividends = AlphaDividendsPerSubnet::<T>::get(netuid, old_hotkey);
        let new_hotkey_alpha_dividends = AlphaDividendsPerSubnet::<T>::get(netuid, new_hotkey);
        AlphaDividendsPerSubnet::<T>::remove(netuid, old_hotkey);
        AlphaDividendsPerSubnet::<T>::insert(
            netuid,
            new_hotkey,
            old_hotkey_alpha_dividends.saturating_add(new_hotkey_alpha_dividends),
        );
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // 8.3 Swap TaoDividendsPerSubnet
        // Tao dividends were removed

        // 8.4 Swap VotingPower
        // VotingPower( netuid, hotkey ) --> u64 -- the voting power EMA for the hotkey.
        Self::swap_voting_power_for_hotkey(old_hotkey, new_hotkey, netuid);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

        // 9. Swap Alpha
        // Alpha( hotkey, coldkey, netuid ) -> alpha
        let old_alpha_values: Vec<((T::AccountId, NetUid), U64F64)> =
            Alpha::<T>::iter_prefix((old_hotkey,)).collect();
        weight.saturating_accrue(T::DbWeight::get().reads(old_alpha_values.len() as u64));
        weight.saturating_accrue(T::DbWeight::get().writes(old_alpha_values.len() as u64));

        // 9.1. Transfer root claimable

        Self::transfer_root_claimable_for_new_hotkey(old_hotkey, new_hotkey);

        // 9.2.  Insert the new alpha values.
        for ((coldkey, netuid_alpha), alpha) in old_alpha_values {
            if netuid == netuid_alpha {
                Self::transfer_root_claimed_for_new_keys(
                    netuid, old_hotkey, new_hotkey, &coldkey, &coldkey,
                );

                let new_alpha = Alpha::<T>::take((new_hotkey, &coldkey, netuid));
                Alpha::<T>::remove((old_hotkey, &coldkey, netuid));
                Alpha::<T>::insert(
                    (new_hotkey, &coldkey, netuid),
                    alpha.saturating_add(new_alpha),
                );
                weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

                // Swap StakingHotkeys.
                // StakingHotkeys( coldkey ) --> Vec<hotkey> -- the hotkeys that the coldkey stakes.
                let mut staking_hotkeys = StakingHotkeys::<T>::get(&coldkey);
                weight.saturating_accrue(T::DbWeight::get().reads(1));
                if staking_hotkeys.contains(old_hotkey) && !staking_hotkeys.contains(new_hotkey) {
                    staking_hotkeys.push(new_hotkey.clone());
                    StakingHotkeys::<T>::insert(&coldkey, staking_hotkeys);
                    weight.saturating_accrue(T::DbWeight::get().writes(1));
                }
            }
        }

        Ok(())
    }
}
