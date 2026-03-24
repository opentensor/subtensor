use super::*;

impl<T: Config> Pallet<T> {
    pub fn do_try_associate_hotkey(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
    ) -> DispatchResult {
        // Ensure the hotkey is not already associated with a coldkey
        Self::create_account_if_non_existent(coldkey, hotkey);

        Ok(())
    }

    pub fn do_disassociate_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> DispatchResult {
        // Ensure the hotkey exists.
        ensure!(
            Self::hotkey_account_exists(hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure the coldkey owns the hotkey.
        ensure!(
            Self::coldkey_owns_hotkey(coldkey, hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // Ensure the hotkey is not registered on any subnet.
        ensure!(
            !Self::is_hotkey_registered_on_any_network(hotkey),
            Error::<T>::HotkeyIsStillRegistered
        );

        // Ensure the hotkey has no outstanding stake from any coldkey.
        ensure!(
            Self::alpha_iter_single_prefix(hotkey).next().is_none(),
            Error::<T>::HotkeyHasOutstandingStake
        );

        // Remove Owner entry.
        Owner::<T>::remove(hotkey);

        // Remove hotkey from OwnedHotkeys.
        let mut owned = OwnedHotkeys::<T>::get(coldkey);
        owned.retain(|h| h != hotkey);
        if owned.is_empty() {
            OwnedHotkeys::<T>::remove(coldkey);
        } else {
            OwnedHotkeys::<T>::insert(coldkey, owned);
        }

        // Remove hotkey from StakingHotkeys.
        let mut staking = StakingHotkeys::<T>::get(coldkey);
        staking.retain(|h| h != hotkey);
        if staking.is_empty() {
            StakingHotkeys::<T>::remove(coldkey);
        } else {
            StakingHotkeys::<T>::insert(coldkey, staking);
        }

        // Remove Delegates entry if present.
        Delegates::<T>::remove(hotkey);

        // Clean up AutoStakeDestination references.
        // Other coldkeys may have set this hotkey as their auto-stake destination.
        for netuid in Self::get_all_subnet_netuids() {
            let coldkeys = AutoStakeDestinationColdkeys::<T>::get(hotkey, netuid);
            for ck in &coldkeys {
                AutoStakeDestination::<T>::remove(ck, netuid);
            }
            AutoStakeDestinationColdkeys::<T>::remove(hotkey, netuid);
        }

        Self::deposit_event(Event::HotkeyDisassociated {
            coldkey: coldkey.clone(),
            hotkey: hotkey.clone(),
        });

        Ok(())
    }
}
