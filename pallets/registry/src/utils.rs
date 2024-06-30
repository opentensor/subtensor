use super::*;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    pub fn set_identity_of(account: &T::AccountId, reg: Registration<BalanceOf<T>, T::MaxAdditionalFields>) {
        IdentityOf::<T>::insert(account, reg);
    }

    pub fn get_identity_of(account: &T::AccountId) -> Option<IdentityInfo<T::MaxAdditionalFields>> {
        if let Some(value) = IdentityOf::<T>::get(account) {
            return Some(value.info);
        }

        None
    }

    pub fn get_all_identities() -> Vec<IdentityInfo<T::MaxAdditionalFields>> {
        let mut identities = Vec::<IdentityInfo<T::MaxAdditionalFields>>::new();
        for id in IdentityOf::<T>::iter_keys() {
            let delegate_id = Self::get_identity_of(&id);

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