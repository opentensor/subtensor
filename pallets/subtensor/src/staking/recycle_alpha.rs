use super::*;
use crate::{Error, system::ensure_signed};

impl<T: Config> Pallet<T> {
    /// Recycles alpha from a cold/hot key pair, reducing AlphaOut on a subnet
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call (must be signed by the coldkey)
    /// * `hotkey` - The hotkey account
    /// * `amount` - The amount of alpha to recycle
    /// * `netuid` - The subnet ID from which to reduce AlphaOut
    ///
    /// # Returns
    ///
    /// * `DispatchResult` - Success or error
    pub(crate) fn do_recycle_alpha(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        amount: u64,
        netuid: u16,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;

        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        ensure!(
            TotalHotkeyAlpha::<T>::get(&hotkey, netuid) >= amount,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= amount,
            Error::<T>::InsufficientLiquidity
        );

        if TotalHotkeyAlpha::<T>::mutate(&hotkey, netuid, |v| {
            *v = v.saturating_sub(amount);
            *v
        }) == 0
        {
            TotalHotkeyAlpha::<T>::remove(&hotkey, netuid);
        }

        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub(amount);
        });

        Self::deposit_event(Event::AlphaRecycled(coldkey, hotkey, amount, netuid));

        Ok(())
    }

    /// Burns alpha from a cold/hot key pair without reducing AlphaOut
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call (must be signed by the coldkey)
    /// * `hotkey` - The hotkey account
    /// * `amount` - The "up to" amount of alpha to burn
    /// * `netuid` - The subnet ID
    ///
    /// # Returns
    ///
    /// * `DispatchResult` - Success or error
    pub(crate) fn do_burn_alpha(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        amount: u64,
        netuid: u16,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;

        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        ensure!(
            TotalHotkeyAlpha::<T>::get(&hotkey, netuid) >= amount,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= amount,
            Error::<T>::InsufficientLiquidity
        );

        if TotalHotkeyAlpha::<T>::mutate(&hotkey, netuid, |v| {
            *v = v.saturating_sub(amount);
            *v
        }) == 0
        {
            TotalHotkeyAlpha::<T>::remove(&hotkey, netuid);
        }

        // Deposit event
        Self::deposit_event(Event::AlphaBurned(coldkey, hotkey, amount, netuid));

        Ok(())
    }
}
