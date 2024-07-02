use super::*;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    pub fn get_identity_of_delegate(account: &T::AccountId) -> Option<IdentityInfo<T::MaxAdditionalFields>> {
        if let Some(value) = IdentityOf::<T>::get(account) {
            return Some(value.info);
        }

        None
    }

    pub fn get_delegate_identitites() -> Vec<IdentityInfo<T::MaxAdditionalFields>> {
        let mut identities = Vec::<IdentityInfo<T::MaxAdditionalFields>>::new();
        for id in IdentityOf::<T>::iter_keys() {
            let delegate_id = Self::get_identity_of_delegate(&id);

            match delegate_id {
                Some(identity) => {
                    identities.push(identity);
                }
                None => continue,
            }
        }

        identities
    }
}
