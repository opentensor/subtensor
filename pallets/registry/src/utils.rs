use super::*;
use sp_runtime::DispatchResult;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    /// Retrieves the identity information of a given delegate account.
    ///
    /// # Parameters
    /// - `account`: A reference to the account ID of the delegate.
    ///
    /// # Returns
    /// - `Option<IdentityInfo<T::MaxAdditionalFields>>`: An `Option` containing the identity information of the delegate
    /// if it exists, otherwise `None`.
    pub fn get_identity_of_delegate(
        account: &T::AccountId,
    ) -> Option<IdentityInfo<T::MaxAdditionalFields>> {
        if let Some(value) = IdentityOf::<T>::get(account) {
            return Some(value.info);
        }

        None
    }

    /// Retrieves the identity information of all delegates.
    ///
    /// # Returns
    /// - `Option<Vec<IdentityInfo<T::MaxAdditionalFields>>>`: An `Option` containing a vector of identity information
    /// of all delegates if any exist, otherwise `None`.
    pub fn get_delegate_identitities() -> Option<Vec<IdentityInfo<T::MaxAdditionalFields>>> {
        let mut identities = Vec::<IdentityInfo<T::MaxAdditionalFields>>::new();
        for id in IdentityOf::<T>::iter_keys() {
            let identity_info = Self::get_identity_of_delegate(&id);

            match identity_info {
                Some(identity) => {
                    identities.push(identity);
                }
                None => continue,
            }
        }

        if identities.len() > 0 {
            return Some(identities);
        }

        None
    }

    /// Sets the identity information for a given delegate account with provided values.
    ///
    /// # Parameters
    /// - `account`: A reference to the account ID of the delegate.
    /// - `info`: The identity information to set for the delegate.
    ///
    /// # Returns
    /// - `DispatchResult`: Returns `Ok(())` if the identity is set successfully, otherwise returns
    /// a `DispatchError`.
    ///
    /// # Errors
    /// - `IdentityAlreadyExists`: Returned if the delegate already has an identity set.
    pub fn set_identity_for_delegate(
        account: &T::AccountId,
        info: IdentityInfo<T::MaxAdditionalFields>,
    ) -> DispatchResult {
        if IdentityOf::<T>::contains_key(account) {
            return Err(<Error<T>>::IdentityAlreadyExists.into());
        }

        let reg: Registration<BalanceOf<T>, T::MaxAdditionalFields> = Registration {
            deposit: Zero::zero(),
            info,
        };

        IdentityOf::<T>::insert(account, reg);
        Ok(()) // Identity set successfully
    }

    /// Swaps the hotkey of a delegate identity from an old account ID to a new account ID.
    ///
    /// # Parameters
    /// - `old_hotkey`: A reference to the current account ID (old hotkey) of the delegate identity.
    /// - `new_hotkey`: A reference to the new account ID (new hotkey) to be assigned to the delegate identity.
    ///
    /// # Returns
    /// - `Result<(), SwapError>`: Returns `Ok(())` if the swap is successful. Returns `Err(SwapError)` otherwise.
    pub fn swap_delegate_identity_hotkey(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
    ) -> DispatchResult {
        // Check if the old hotkey exists in the identity map.
        if let Some(identity) = IdentityOf::<T>::take(old_hotkey) {
            // Check if the new hotkey is already in use.
            if IdentityOf::<T>::contains_key(new_hotkey) {
                // Reinsert the old hotkey back into the identity map to maintain consistency.
                IdentityOf::<T>::insert(old_hotkey, identity);
                return Err(Error::<T>::NewHotkeyInUse.into()); // New hotkey is already in use.
            }
            IdentityOf::<T>::insert(new_hotkey, identity);
            return Ok(());
        }

        Err(Error::<T>::OldHotkeyNotFound.into()) // Old hotkey does not exist in Identities.
    }
}
