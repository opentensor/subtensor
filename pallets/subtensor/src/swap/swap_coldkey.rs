use super::*;
use frame_support::traits::fungible::Mutate;
use frame_support::traits::tokens::Preservation;
use frame_support::weights::Weight;
use sp_core::Get;

impl<T: Config> Pallet<T> {
  

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
        new_coldkey: &T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let old_coldkey = ensure_signed(origin)?;
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

        // Calculate and charge the swap fee
        let swap_cost = Self::get_key_swap_cost();
        log::debug!("Coldkey swap cost: {:?}", swap_cost);

        ensure!(
            Self::can_remove_balance_from_coldkey_account(&old_coldkey, swap_cost),
            Error::<T>::NotEnoughBalanceToPaySwapColdKey
        );
        let actual_burn_amount =
            Self::remove_balance_from_coldkey_account(&old_coldkey, swap_cost)?;
        Self::burn_tokens(actual_burn_amount);

        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // Actually do the swap.
        weight = weight.saturating_add(
            Self::perform_swap_coldkey(&old_coldkey, new_coldkey)
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
            // Attempt to transfer the entire total balance to new_coldkey.
            T::Currency::transfer(
                old_coldkey,
                new_coldkey,
                total_balance,
                Preservation::Expendable,
            )?;
        }

        Ok(weight)
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

    /// Swaps the stake associated with a coldkey from the old coldkey to the new coldkey.
    ///
    /// # Arguments
    ///
    /// * `old_coldkey` - The AccountId of the old coldkey.
    /// * `new_coldkey` - The AccountId of the new coldkey.
    /// * `weight` - Mutable reference to the weight of the transaction.
    ///
    /// # Effects
    ///
    /// * Transfers all stakes from the old coldkey to the new coldkey.
    /// * Updates the ownership of hotkeys.
    /// * Updates the total stake for both old and new coldkeys.
    /// * Updates the transaction weight.
    ///

    pub fn swap_stake_for_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        weight: &mut Weight,
    ) {
        // Retrieve the list of hotkeys owned by the old coldkey
        let old_owned_hotkeys: Vec<T::AccountId> = OwnedHotkeys::<T>::get(old_coldkey);

        // Initialize the total transferred stake to zero
        let mut total_transferred_stake: u64 = 0u64;

        // Log the total stake of old and new coldkeys before the swap
        log::info!(
            "Before swap - Old coldkey total stake: {}",
            TotalColdkeyStake::<T>::get(old_coldkey)
        );
        log::info!(
            "Before swap - New coldkey total stake: {}",
            TotalColdkeyStake::<T>::get(new_coldkey)
        );

        // Iterate over each hotkey owned by the old coldkey
        for hotkey in old_owned_hotkeys.iter() {
            // Retrieve and remove the stake associated with the hotkey and old coldkey
            let stake: u64 = Stake::<T>::take(hotkey, old_coldkey);
            log::info!("Transferring stake for hotkey {:?}: {}", hotkey, stake);
            if stake > 0 {
                // Insert the stake for the hotkey and new coldkey
                let old_stake = Stake::<T>::get(hotkey, new_coldkey);
                Stake::<T>::insert(hotkey, new_coldkey, stake.saturating_add(old_stake));
                total_transferred_stake = total_transferred_stake.saturating_add(stake);

                // Update the owner of the hotkey to the new coldkey
                Owner::<T>::insert(hotkey, new_coldkey);

                // Update the transaction weight
                weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));
            }
        }
        log::info!(
            "Starting transfer of delegated stakes for old coldkey: {:?}",
            old_coldkey
        );

        for staking_hotkey in StakingHotkeys::<T>::get(old_coldkey) {
            log::info!("Processing staking hotkey: {:?}", staking_hotkey);
            if Stake::<T>::contains_key(staking_hotkey.clone(), old_coldkey) {
                let hotkey = &staking_hotkey;
                // Retrieve and remove the stake associated with the hotkey and old coldkey
                let stake: u64 = Stake::<T>::get(hotkey, old_coldkey);
                Stake::<T>::remove(hotkey, old_coldkey);
                log::info!(
                    "Transferring delegated stake for hotkey {:?}: {}",
                    hotkey,
                    stake
                );
                if stake > 0 {
                    // Insert the stake for the hotkey and new coldkey
                    let old_stake = Stake::<T>::get(hotkey, new_coldkey);
                    Stake::<T>::insert(hotkey, new_coldkey, stake.saturating_add(old_stake));
                    total_transferred_stake = total_transferred_stake.saturating_add(stake);
                    log::info!(
                        "Updated stake for hotkey {:?} under new coldkey {:?}: {}",
                        hotkey,
                        new_coldkey,
                        stake.saturating_add(old_stake)
                    );

                    // Update the transaction weight
                    weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 1));
                }
            } else {
                log::info!(
                    "No stake found for staking hotkey {:?} under old coldkey {:?}",
                    staking_hotkey,
                    old_coldkey
                );
                weight.saturating_accrue(T::DbWeight::get().reads(1));
            }
        }

        log::info!(
            "Completed transfer of delegated stakes for old coldkey: {:?}",
            old_coldkey
        );

        // Log the total transferred stake
        log::info!("Total transferred stake: {}", total_transferred_stake);

        // Update the total stake for both old and new coldkeys if any stake was transferred
        if total_transferred_stake > 0 {
            let old_coldkey_stake: u64 = TotalColdkeyStake::<T>::take(old_coldkey); // Remove it here.
            let new_coldkey_stake: u64 = TotalColdkeyStake::<T>::get(new_coldkey);

            TotalColdkeyStake::<T>::insert(old_coldkey, 0);
            TotalColdkeyStake::<T>::insert(
                new_coldkey,
                new_coldkey_stake.saturating_add(old_coldkey_stake),
            );

            log::info!("Updated old coldkey stake from {} to 0", old_coldkey_stake);
            log::info!(
                "Updated new coldkey stake from {} to {}",
                new_coldkey_stake,
                new_coldkey_stake.saturating_add(old_coldkey_stake)
            );

            // Update the transaction weight
            weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));
        }

        // Update the list of owned hotkeys for both old and new coldkeys

        let mut new_owned_hotkeys = OwnedHotkeys::<T>::get(new_coldkey);
        for hotkey in old_owned_hotkeys {
            if !new_owned_hotkeys.contains(&hotkey) {
                new_owned_hotkeys.push(hotkey);
            }
        }

        OwnedHotkeys::<T>::insert(new_coldkey, new_owned_hotkeys);
        OwnedHotkeys::<T>::remove(old_coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 2));

        // Update the staking hotkeys for both old and new coldkeys
        let staking_hotkeys: Vec<T::AccountId> = StakingHotkeys::<T>::get(old_coldkey);

        let mut existing_staking_hotkeys = StakingHotkeys::<T>::get(new_coldkey);
        for hotkey in staking_hotkeys {
            if !existing_staking_hotkeys.contains(&hotkey) {
                existing_staking_hotkeys.push(hotkey);
            }
        }

        StakingHotkeys::<T>::remove(old_coldkey);
        StakingHotkeys::<T>::insert(new_coldkey, existing_staking_hotkeys);
        weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

        // Log the total stake of old and new coldkeys after the swap
        log::info!(
            "After swap - Old coldkey total stake: {}",
            TotalColdkeyStake::<T>::get(old_coldkey)
        );
        log::info!(
            "After swap - New coldkey total stake: {}",
            TotalColdkeyStake::<T>::get(new_coldkey)
        );
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
