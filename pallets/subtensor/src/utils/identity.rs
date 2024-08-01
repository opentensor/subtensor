use super::*;
use frame_support::ensure;
use frame_system::ensure_signed;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    /// Sets the identity for a coldkey.
    ///
    /// This function allows a user to set or update their identity information associated with their coldkey.
    /// It checks if the caller has at least one registered hotkey, validates the provided identity information,
    /// and then stores it in the blockchain state.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call, which should be a signed extrinsic.
    /// * `name` - The name to be associated with the identity.
    /// * `url` - A URL associated with the identity.
    /// * `image` - An image URL or identifier for the identity.
    /// * `discord` - Discord information for the identity.
    /// * `description` - A description of the identity.
    /// * `additional` - Any additional information for the identity.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the identity is successfully set, otherwise returns an error.
    pub fn do_set_identity(
        origin: T::RuntimeOrigin,
        name: Vec<u8>,
        url: Vec<u8>,
        image: Vec<u8>,
        discord: Vec<u8>,
        description: Vec<u8>,
        additional: Vec<u8>,
    ) -> dispatch::DispatchResult {
        // Ensure the call is signed and get the signer's (coldkey) account
        let coldkey = ensure_signed(origin)?;

        // Retrieve all hotkeys associated with this coldkey
        let hotkeys: Vec<T::AccountId> = OwnedHotkeys::<T>::get(coldkey.clone());

        // Ensure that at least one of the associated hotkeys is registered on any network
        ensure!(
            hotkeys
                .iter()
                .any(|hotkey| Self::is_hotkey_registered_on_any_network(hotkey)),
            Error::<T>::HotKeyNotRegisteredInNetwork
        );

        // Create the identity struct with the provided information
        let identity = ChainIdentityOf {
            name,
            url,
            image,
            discord,
            description,
            additional,
        };

        // Validate the created identity
        ensure!(
            Self::is_valid_identity(&identity),
            Error::<T>::InvalidIdentity
        );

        // Store the validated identity in the blockchain state
        Identities::<T>::insert(coldkey.clone(), identity.clone());

        // Log the identity set event
        log::info!("ChainIdentitySet( coldkey:{:?} ) ", coldkey.clone());

        // Emit an event to notify that an identity has been set
        Self::deposit_event(Event::ChainIdentitySet(coldkey.clone()));

        // Return Ok to indicate successful execution
        Ok(())
    }

    /// Validates the given ChainIdentityOf struct.
    ///
    /// This function checks if the total length of all fields in the ChainIdentityOf struct
    /// is less than or equal to 512 bytes, and if each individual field is also
    /// less than or equal to 512 bytes.
    ///
    /// # Arguments
    ///
    /// * `identity` - A reference to the ChainIdentityOf struct to be validated.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the Identity is valid, false otherwise.
    pub fn is_valid_identity(identity: &ChainIdentityOf) -> bool {
        let total_length = identity
            .name
            .len()
            .saturating_add(identity.url.len())
            .saturating_add(identity.image.len())
            .saturating_add(identity.discord.len())
            .saturating_add(identity.description.len())
            .saturating_add(identity.additional.len());

        total_length <= 256 + 256 + 1024 + 256 + 1024 + 1024
            && identity.name.len() <= 256
            && identity.url.len() <= 256
            && identity.image.len() <= 1024
            && identity.discord.len() <= 256
            && identity.description.len() <= 1024
            && identity.additional.len() <= 1024
    }

    /// Swaps the hotkey of a delegate identity from an old account ID to a new account ID.
    ///
    /// # Parameters
    /// - `old_hotkey`: A reference to the current account ID (old hotkey) of the delegate identity.
    /// - `new_hotkey`: A reference to the new account ID (new hotkey) to be assigned to the delegate identity.
    ///
    /// # Returns
    /// - `Result<(), SwapError>`: Returns `Ok(())` if the swap is successful. Returns `Err(SwapError)` otherwise.
    pub fn swap_delegate_identity_coldkey(
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) -> DispatchResult {
        // Attempt to remove the identity associated with the old hotkey.
        let identity: ChainIdentity =
            Identities::<T>::take(old_coldkey).ok_or(Error::<T>::OldColdkeyNotFound)?;

        // Ensure the new hotkey is not already in use.
        if Identities::<T>::contains_key(new_coldkey) {
            // Reinsert the identity back with the old hotkey to maintain consistency.
            Identities::<T>::insert(old_coldkey, identity);
            return Err(Error::<T>::NewColdkeyInUse.into());
        }

        // Insert the identity with the new hotkey.
        Identities::<T>::insert(new_coldkey, identity);
        Ok(())
    }
}
