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
        // Fetch the owner once: missing entry => hotkey doesn't exist; mismatch => not owned.
        let owner = Owner::<T>::try_get(hotkey).map_err(|_| Error::<T>::HotKeyAccountNotExists)?;
        ensure!(&owner == coldkey, Error::<T>::NonAssociatedColdKey);

        // Ensure the hotkey is not registered on any subnet.
        ensure!(
            !Self::is_hotkey_registered_on_any_network(hotkey),
            Error::<T>::HotkeyIsStillRegistered
        );

        // Ensure the hotkey has no outstanding stake from any coldkey.
        // Check both the legacy `Alpha` and the new `AlphaV2` storages directly so the
        // iterator short-circuits on the first entry instead of materializing both maps.
        ensure!(
            Alpha::<T>::iter_prefix((hotkey,)).next().is_none()
                && AlphaV2::<T>::iter_prefix((hotkey,)).next().is_none(),
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

        // Clean up AutoStakeDestination references via the inverse index so we only
        // touch subnets where this hotkey was actually an auto-stake destination
        // (instead of scanning every subnet).
        let auto_stake_entries: Vec<(NetUid, Vec<T::AccountId>)> =
            AutoStakeDestinationColdkeys::<T>::iter_prefix(hotkey).collect();
        for (netuid, coldkeys) in auto_stake_entries {
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
