use super::*;
use substrate_fixed::types::U64F64;

impl<T: Config> Pallet<T> {
    /// Transfer all assets, stakes, subnet ownerships, and hotkey associations from `old_coldkey` to
    /// to `new_coldkey`.
    pub fn do_swap_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
        swap_cost: TaoCurrency,
    ) -> DispatchResult {
        ensure!(
            StakingHotkeys::<T>::get(new_coldkey).is_empty(),
            Error::<T>::ColdKeyAlreadyAssociated
        );
        ensure!(
            !Self::hotkey_account_exists(new_coldkey),
            Error::<T>::NewColdKeyIsHotkey
        );

        // Remove and recycle the swap cost from the old coldkey's account
        ensure!(
            Self::can_remove_balance_from_coldkey_account(old_coldkey, swap_cost.into()),
            Error::<T>::NotEnoughBalanceToPaySwapColdKey
        );
        let burn_amount = Self::remove_balance_from_coldkey_account(old_coldkey, swap_cost.into())?;
        Self::recycle_tao(burn_amount);

        // Swap the identity if the old coldkey has one and the new coldkey doesn't
        if IdentitiesV2::<T>::get(new_coldkey).is_none()
            && let Some(identity) = IdentitiesV2::<T>::take(old_coldkey)
        {
            IdentitiesV2::<T>::insert(new_coldkey.clone(), identity);
        }

        for netuid in Self::get_all_subnet_netuids() {
            Self::transfer_subnet_ownership(netuid, old_coldkey, new_coldkey);
            Self::transfer_auto_stake_destination(netuid, old_coldkey, new_coldkey);
            Self::transfer_coldkey_stake(netuid, old_coldkey, new_coldkey);
        }
        Self::transfer_staking_hotkeys(old_coldkey, new_coldkey);
        Self::transfer_hotkeys_ownership(old_coldkey, new_coldkey);

        // Transfer any remaining balance from old_coldkey to new_coldkey
        let remaining_balance = Self::get_coldkey_balance(old_coldkey);
        if remaining_balance > 0 {
            Self::kill_coldkey_account(old_coldkey, remaining_balance)?;
            Self::add_balance_to_coldkey_account(new_coldkey, remaining_balance);
        }

        Self::set_last_tx_block(new_coldkey, Self::get_current_block_as_u64());

        Self::deposit_event(Event::ColdkeySwapped {
            old_coldkey: old_coldkey.clone(),
            new_coldkey: new_coldkey.clone(),
            swap_cost,
        });
        Ok(())
    }

    /// Transfer the ownership of the subnet to the new coldkey if it is owned by the old coldkey.
    fn transfer_subnet_ownership(
        netuid: NetUid,
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) {
        let subnet_owner = SubnetOwner::<T>::get(netuid);
        if subnet_owner == *old_coldkey {
            SubnetOwner::<T>::insert(netuid, new_coldkey.clone());
        }
    }

    /// Transfer the auto stake destination from the old coldkey to the new coldkey if it is set.
    fn transfer_auto_stake_destination(
        netuid: NetUid,
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) {
        if let Some(old_auto_stake_hotkey) = AutoStakeDestination::<T>::get(old_coldkey, netuid) {
            AutoStakeDestination::<T>::remove(old_coldkey, netuid);
            AutoStakeDestination::<T>::insert(new_coldkey, netuid, old_auto_stake_hotkey.clone());
            AutoStakeDestinationColdkeys::<T>::mutate(old_auto_stake_hotkey, netuid, |v| {
                // Remove old/new coldkeys (avoid duplicates), then add the new one.
                v.retain(|c| *c != *old_coldkey && *c != *new_coldkey);
                v.push(new_coldkey.clone());
            });
        }
    }

    /// Transfer the stake of all staking hotkeys linked to the old coldkey to the new coldkey.
    fn transfer_coldkey_stake(
        netuid: NetUid,
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) {
        for hotkey in StakingHotkeys::<T>::get(old_coldkey) {
            // Get the stake on the old (hot,coldkey) account.
            let old_alpha: U64F64 = Alpha::<T>::get((&hotkey, old_coldkey, netuid));
            // Get the stake on the new (hot,coldkey) account.
            let new_alpha: U64F64 = Alpha::<T>::get((&hotkey, new_coldkey, netuid));
            // Add the stake to new account.
            Alpha::<T>::insert(
                (&hotkey, new_coldkey, netuid),
                new_alpha.saturating_add(old_alpha),
            );
            // Remove the value from the old account.
            Alpha::<T>::remove((&hotkey, old_coldkey, netuid));

            if new_alpha.saturating_add(old_alpha) > U64F64::from(0u64) {
                Self::transfer_root_claimed_for_new_keys(
                    netuid,
                    &hotkey,
                    &hotkey,
                    old_coldkey,
                    new_coldkey,
                );

                if netuid == NetUid::ROOT {
                    // Register new coldkey with root stake
                    Self::maybe_add_coldkey_index(new_coldkey);
                }
            }
        }
    }

    /// Transfer staking hotkeys from the old coldkey to the new coldkey.
    fn transfer_staking_hotkeys(old_coldkey: &T::AccountId, new_coldkey: &T::AccountId) {
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
    }

    /// Transfer the ownership of the hotkeys owned by the old coldkey to the new coldkey.
    fn transfer_hotkeys_ownership(old_coldkey: &T::AccountId, new_coldkey: &T::AccountId) {
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
    }
}
