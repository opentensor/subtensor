use super::*;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic become_delegate: signals that this hotkey allows delegated stake.
    ///
    /// # Args:
    /// *  'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     - The signature of the caller's coldkey.
    ///
    /// *  'hotkey' (T::AccountId):
    ///     - The hotkey we are delegating (must be owned by the coldkey.)
    ///
    /// *  'take' (u16):
    ///     - The stake proportion that this hotkey takes from delegations.
    ///
    /// # Event:
    /// *  DelegateAdded;
    ///     - On successfully setting a hotkey as a delegate.
    ///
    /// # Raises:
    /// *  'NotRegistered':
    ///     - The hotkey we are delegating is not registered on the network.
    ///
    /// *  'NonAssociatedColdKey':
    ///     - The hotkey we are delegating is not owned by the calling coldket.
    ///
    /// *  'TxRateLimitExceeded':
    ///     - Thrown if key has hit transaction rate limit
    ///
    pub fn do_become_delegate(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signuture.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_become_delegate( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );

        // --- 2. Ensure we are delegating an known key.
        // --- 3. Ensure that the coldkey is the owner.
        Self::do_take_checks(&coldkey, &hotkey)?;

        // --- 4. Ensure we are not already a delegate (dont allow changing delegate take.)
        ensure!(
            !Delegates::<T>::contains_key(&hotkey),
            Error::<T>::HotKeyAlreadyDelegate
        );

        // --- 5. Ensure we don't exceed tx rate limit
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(LastTxBlock::<T>::get(&coldkey), block),
            Error::<T>::DelegateTxRateLimitExceeded
        );

        // --- 5.1 Ensure take is within the min ..= InitialDefaultDelegateTake (18%) range
        let min_take = MinDelegateTake::<T>::get();
        let max_take = MaxDelegateTake::<T>::get();
        ensure!(take >= min_take, Error::<T>::DelegateTakeTooLow);
        ensure!(take <= max_take, Error::<T>::DelegateTakeTooHigh);

        // --- 6. Delegate the key.
        Delegates::<T>::insert(&hotkey, take);

        // Set last block for rate limiting
        LastTxBlock::<T>::insert(&coldkey, block);
        LastTxBlockDelegateTake::<T>::insert(&coldkey, block);

        // --- 7. Emit the staking event.
        log::debug!(
            "DelegateAdded( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        Self::deposit_event(Event::DelegateAdded(coldkey, hotkey, take));

        // --- 8. Ok and return.
        Ok(())
    }
}
