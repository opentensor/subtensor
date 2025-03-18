use super::*;
use crate::{Error, system::ensure_signed};

impl<T: Config> Pallet<T> {
    /// Recycles tokens from a cold/hot key pair, reducing AlphaOut on a subnet
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call (must be signed by the coldkey)
    /// * `hotkey` - The hotkey account
    /// * `amount` - The amount of tokens to recycle
    /// * `netuid` - The subnet ID from which to reduce AlphaOut
    ///
    /// # Returns
    ///
    /// * `DispatchResult` - Success or error
    pub fn do_recycle(
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
            Self::can_remove_balance_from_coldkey_account(&coldkey, amount),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // convert TAO to alpha equivalent
        let Some(alpha_amount) = Self::sim_swap_tao_for_alpha(netuid, amount) else {
            return Err(Error::<T>::InsufficientLiquidity.into());
        };

        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= alpha_amount,
            Error::<T>::NotEnoughAlphaOutToRecycle
        );

        // remove the amount from the coldkey account
        let actual_burn_amount = Self::remove_balance_from_coldkey_account(&coldkey, amount)?;

        // update related storages
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub(alpha_amount);
        });
        SubnetTAO::<T>::mutate(netuid, |v| *v = v.saturating_sub(actual_burn_amount));
        TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_sub(actual_burn_amount));

        Self::increase_rao_recycled(netuid, actual_burn_amount);

        // Deposit event
        Self::deposit_event(Event::TokensRecycled(
            coldkey,
            hotkey,
            actual_burn_amount,
            netuid,
            alpha_amount,
        ));

        Ok(())
    }

    /// Burns tokens from a cold/hot key pair without reducing AlphaOut
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call (must be signed by the coldkey)
    /// * `hotkey` - The hotkey account
    /// * `amount` - The amount of tokens to burn
    /// * `netuid` - The subnet ID
    ///
    /// # Returns
    ///
    /// * `DispatchResult` - Success or error
    pub fn do_burn(
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
            Self::can_remove_balance_from_coldkey_account(&coldkey, amount),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        let actual_burn_amount = Self::remove_balance_from_coldkey_account(&coldkey, amount)?;

        SubnetTAO::<T>::mutate(netuid, |v| *v = v.saturating_sub(actual_burn_amount));
        TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_sub(actual_burn_amount));

        Self::deposit_event(Event::TokensBurned(coldkey, hotkey, actual_burn_amount, netuid));

        Ok(())
    }
}
