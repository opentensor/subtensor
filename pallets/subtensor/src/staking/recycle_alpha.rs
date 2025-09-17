use super::*;
use crate::{Error, system::ensure_signed};
use subtensor_runtime_common::{AlphaCurrency, NetUid};

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
        amount: AlphaCurrency,
        netuid: NetUid,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        ensure!(
            !netuid.is_root(),
            Error::<T>::CannotBurnOrRecycleOnRootSubnet
        );

        Self::ensure_subtoken_enabled(netuid)?;

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure that the hotkey has enough stake to withdraw.
        // Cap the amount at available Alpha because user might be paying transaxtion fees
        // in Alpha and their total is already reduced by now.
        let alpha_available =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let amount = amount.min(alpha_available);

        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= amount,
            Error::<T>::InsufficientLiquidity
        );

        // Deduct from the coldkey's stake.
        let actual_alpha_decrease = Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey, &coldkey, netuid, amount,
        );

        // Recycle means we should decrease the alpha issuance tracker.
        Self::recycle_subnet_alpha(netuid, amount);

        Self::deposit_event(Event::AlphaRecycled(
            coldkey,
            hotkey,
            actual_alpha_decrease,
            netuid,
        ));

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
        amount: AlphaCurrency,
        netuid: NetUid,
    ) -> DispatchResult {
        let coldkey = ensure_signed(origin)?;

        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        ensure!(
            !netuid.is_root(),
            Error::<T>::CannotBurnOrRecycleOnRootSubnet
        );

        Self::ensure_subtoken_enabled(netuid)?;

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure that the hotkey has enough stake to withdraw.
        // Cap the amount at available Alpha because user might be paying transaxtion fees
        // in Alpha and their total is already reduced by now.
        let alpha_available =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &coldkey, netuid);
        let amount = amount.min(alpha_available);

        ensure!(
            SubnetAlphaOut::<T>::get(netuid) >= amount,
            Error::<T>::InsufficientLiquidity
        );

        // Deduct from the coldkey's stake.
        let actual_alpha_decrease = Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey, &coldkey, netuid, amount,
        );

        Self::burn_subnet_alpha(netuid, amount);

        // Deposit event
        Self::deposit_event(Event::AlphaBurned(
            coldkey,
            hotkey,
            actual_alpha_decrease,
            netuid,
        ));

        Ok(())
    }
}
