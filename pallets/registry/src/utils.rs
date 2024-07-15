use super::*;
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
    pub fn get_identity_of_delegate(account: &T::AccountId) -> Option<IdentityInfo<T::MaxAdditionalFields>> {
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
            return Some(identities)
        }
        
        None
    }
}
