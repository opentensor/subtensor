use super::*;
use frame_support::storage::IterableStorageDoubleMap;
use sp_core::Get;

impl<T: Config> Pallet<T> {
    pub fn do_swap_hotkey(
        origin: T::RuntimeOrigin,
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
    ) -> DispatchResultWithPostInfo {
        let coldkey = ensure_signed(origin)?;

        let mut weight = T::DbWeight::get().reads_writes(2, 0);
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, old_hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::HotKeySetTxRateLimitExceeded
        );

        weight.saturating_accrue(T::DbWeight::get().reads(2));

        ensure!(old_hotkey != new_hotkey, Error::<T>::NewHotKeyIsSameWithOld);
        ensure!(
            !Self::is_hotkey_registered_on_any_network(new_hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        weight
            .saturating_accrue(T::DbWeight::get().reads((TotalNetworks::<T>::get() + 1u16) as u64));

        let swap_cost = 1_000_000_000u64;
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, swap_cost),
            Error::<T>::NotEnoughBalanceToPaySwapHotKey
        );
        let actual_burn_amount = Self::remove_balance_from_coldkey_account(&coldkey, swap_cost)?;
        Self::burn_tokens(actual_burn_amount);

        Owner::<T>::remove(old_hotkey);
        Owner::<T>::insert(new_hotkey, coldkey.clone());
        weight.saturating_accrue(T::DbWeight::get().writes(2));

        if let Ok(total_hotkey_stake) = TotalHotkeyStake::<T>::try_get(old_hotkey) {
            TotalHotkeyStake::<T>::remove(old_hotkey);
            TotalHotkeyStake::<T>::insert(new_hotkey, total_hotkey_stake);

            weight.saturating_accrue(T::DbWeight::get().writes(2));
        }

        if let Ok(delegate_take) = Delegates::<T>::try_get(old_hotkey) {
            Delegates::<T>::remove(old_hotkey);
            Delegates::<T>::insert(new_hotkey, delegate_take);

            weight.saturating_accrue(T::DbWeight::get().writes(2));
        }

        if let Ok(last_tx) = LastTxBlock::<T>::try_get(old_hotkey) {
            LastTxBlock::<T>::remove(old_hotkey);
            LastTxBlock::<T>::insert(new_hotkey, last_tx);

            weight.saturating_accrue(T::DbWeight::get().writes(2));
        }

        let mut coldkey_stake: Vec<(T::AccountId, u64)> = vec![];
        for (coldkey, stake_amount) in Stake::<T>::iter_prefix(old_hotkey) {
            coldkey_stake.push((coldkey.clone(), stake_amount));
        }

        let _ = Stake::<T>::clear_prefix(old_hotkey, coldkey_stake.len() as u32, None);
        weight.saturating_accrue(T::DbWeight::get().writes(coldkey_stake.len() as u64));

        for (coldkey, stake_amount) in coldkey_stake {
            Stake::<T>::insert(new_hotkey, coldkey, stake_amount);
            weight.saturating_accrue(T::DbWeight::get().writes(1));
        }

        let mut netuid_is_member: Vec<u16> = vec![];
        for netuid in <IsNetworkMember<T> as IterableStorageDoubleMap<T::AccountId, u16, bool>>::iter_key_prefix(old_hotkey) {
            netuid_is_member.push(netuid);
        }

        let _ = IsNetworkMember::<T>::clear_prefix(old_hotkey, netuid_is_member.len() as u32, None);
        weight.saturating_accrue(T::DbWeight::get().writes(netuid_is_member.len() as u64));

        for netuid in netuid_is_member.iter() {
            IsNetworkMember::<T>::insert(new_hotkey, netuid, true);
            weight.saturating_accrue(T::DbWeight::get().writes(1));
        }

        for netuid in netuid_is_member.iter() {
            if let Ok(axon_info) = Axons::<T>::try_get(netuid, old_hotkey) {
                Axons::<T>::remove(netuid, old_hotkey);
                Axons::<T>::insert(netuid, new_hotkey, axon_info);

                weight.saturating_accrue(T::DbWeight::get().writes(2));
            }
        }

        for netuid in netuid_is_member.iter() {
            if let Ok(uid) = Uids::<T>::try_get(netuid, old_hotkey) {
                Uids::<T>::remove(netuid, old_hotkey);
                Uids::<T>::insert(netuid, new_hotkey, uid);

                weight.saturating_accrue(T::DbWeight::get().writes(2));

                Keys::<T>::insert(netuid, uid, new_hotkey);

                weight.saturating_accrue(T::DbWeight::get().writes(1));

                LoadedEmission::<T>::mutate(netuid, |emission_exists| match emission_exists {
                    Some(emissions) => {
                        if let Some(emission) = emissions.get_mut(uid as usize) {
                            let (_, se, ve) = emission;
                            *emission = (new_hotkey.clone(), *se, *ve);
                        }
                    }
                    None => {}
                });

                weight.saturating_accrue(T::DbWeight::get().writes(1));
            }
        }

        Self::set_last_tx_block(&coldkey, block);
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        Self::deposit_event(Event::HotkeySwapped {
            coldkey,
            old_hotkey: old_hotkey.clone(),
            new_hotkey: new_hotkey.clone(),
        });

        Ok(Some(weight).into())
    }
}