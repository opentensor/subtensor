use super::*;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic increase_take
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the caller's coldkey.
    ///
    /// * 'hotkey' (T::AccountId):
    ///     - The hotkey we are delegating (must be owned by the coldkey.)
    ///
    /// * 'take' (u16):
    ///     - The stake proportion that this hotkey takes from delegations for subnet ID.
    ///
    /// # Event:
    /// * TakeIncreased;
    ///     - On successfully setting a increased take for this hotkey.
    ///
    /// # Raises:
    /// * 'NotRegistered':
    ///     - The hotkey we are delegating is not registered on the network.
    ///
    /// * 'NonAssociatedColdKey':
    ///     - The hotkey we are delegating is not owned by the calling coldket.
    ///
    /// * 'TxRateLimitExceeded':
    ///     - Thrown if key has hit transaction rate limit
    ///
    /// * 'DelegateTakeTooLow':
    ///     - The delegate is setting a take which is not greater than the previous.
    ///
    pub fn do_increase_take(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::debug!(
            "do_increase_take( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_take_checks(&coldkey, &hotkey)?;

        // --- 3. Ensure we are strinctly increasing take
        if let Ok(current_take) = Delegates::<T>::try_get(&hotkey) {
            ensure!(take > current_take, Error::<T>::DelegateTakeTooLow);
        }

        // --- 4. Ensure take is within the min ..= InitialDefaultDelegateTake (18%) range
        let max_take = MaxDelegateTake::<T>::get();
        ensure!(take <= max_take, Error::<T>::DelegateTakeTooHigh);

        // --- 5. Enforce the rate limit (independently on do_add_stake rate limits)
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_delegate_take_rate_limit(
                LastTxBlockDelegateTake::<T>::get(&coldkey),
                block
            ),
            Error::<T>::DelegateTxRateLimitExceeded
        );

        // Set last block for rate limiting
        LastTxBlockDelegateTake::<T>::insert(&coldkey, block);

        // --- 6. Set the new take value.
        Delegates::<T>::insert(hotkey.clone(), take);

        // --- 7. Emit the take value.
        log::debug!(
            "TakeIncreased( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        Self::deposit_event(Event::TakeIncreased(coldkey, hotkey, take));

        // --- 8. Ok and return.
        Ok(())
    }
}
