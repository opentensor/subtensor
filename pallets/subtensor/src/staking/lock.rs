use super::*;

impl<T: Config> Pallet<T> {
    pub fn do_lock(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        alpha_locked: u64,
    ) -> dispatch::DispatchResult {
        // Ensure the origin is valid.
        let coldkey = ensure_signed(origin)?;

        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Ensure that the hotkey account exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure the hotkey is registered on this subnet.
        ensure!(
            Self::is_hotkey_registered_on_network(netuid, &hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // Ensure the coldkey owns this hotkey.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::HotKeyNotDelegateAndSignerNotOwnHotKey
        );

        // Get the current lock in alpha.
        let current_alpha_locked: u64 = SubnetLocked::<T>::get(netuid);

        // Get current stake of the caller.
        let alpha_staked: u64 = Alpha::<T>::get((hotkey.clone(), coldkey.clone(), netuid));

        // Ensure the the lock is above zero.
        ensure!(alpha_locked > 0, Error::<T>::NotEnoughStakeToWithdraw);

        // Ensure that the caller has enough stake to unstake.
        ensure!(
            alpha_locked <= alpha_staked,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Ensure the the lock is above zero.
        ensure!(
            alpha_locked > current_alpha_locked,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // Insert the new locked
        SubnetLocked::<T>::insert(netuid, alpha_locked);

        // Get current owner of the subnet.
        let current_owner: T::AccountId = SubnetOwnerHotkey::<T>::get(netuid);
        if current_owner != hotkey {
            // Insert the new owner coldkey.
            SubnetOwner::<T>::insert(netuid, coldkey.clone());
            // Insert the new owner hotkey.
            SubnetOwnerHotkey::<T>::insert(netuid, hotkey.clone());
        }

        // Lock increased event.
        log::info!(
            "LockIncreased( coldkey:{:?}, hotkey:{:?}, netuid:{:?}, alpha_locked:{:?} )",
            coldkey.clone(),
            hotkey.clone(),
            netuid,
            alpha_locked
        );
        Self::deposit_event(Event::LockIncreased(
            coldkey.clone(),
            hotkey.clone(),
            netuid,
            alpha_locked,
        ));

        // Ok and return.
        Ok(())
    }
}
