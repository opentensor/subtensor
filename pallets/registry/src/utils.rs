use super::*;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    pub fn get_identity_of_delegate(account: &T::AccountId) -> Option<IdentityInfo<T::MaxAdditionalFields>> {
        if let Some(value) = IdentityOf::<T>::get(account) {
            return Some(value.info);
        }

        None
    }

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
