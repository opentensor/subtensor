use super::*;
use frame_support::ensure;
use frame_system::ensure_signed;
use sp_core::{H160, ecdsa::Signature, hashing::keccak_256};
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
        github_repo: Vec<u8>,
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
        let identity = ChainIdentityOfV2 {
            name,
            url,
            github_repo,
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
        IdentitiesV2::<T>::insert(coldkey.clone(), identity.clone());

        // Log the identity set event
        log::debug!("ChainIdentitySet( coldkey:{:?} ) ", coldkey.clone());

        // Emit an event to notify that an identity has been set
        Self::deposit_event(Event::ChainIdentitySet(coldkey.clone()));

        // Return Ok to indicate successful execution
        Ok(())
    }

    /// Sets the identity for a subnet.
    ///
    /// This function allows the owner of a subnet to set or update the identity information associated with the subnet.
    /// It verifies that the caller is the owner of the specified subnet, validates the provided identity information,
    /// and then stores it in the blockchain state.
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call, which should be a signed extrinsic.
    /// * `netuid` - The unique identifier for the subnet.
    /// * `subnet_name` - The name of the subnet to be associated with the identity.
    /// * `github_repo` - The GitHub repository URL associated with the subnet identity.
    /// * `subnet_contact` - Contact information for the subnet.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the subnet identity is successfully set, otherwise returns an error.
    pub fn do_set_subnet_identity(
        origin: T::RuntimeOrigin,
        netuid: u16,
        subnet_name: Vec<u8>,
        github_repo: Vec<u8>,
        subnet_contact: Vec<u8>,
        subnet_url: Vec<u8>,
        discord: Vec<u8>,
        description: Vec<u8>,
        additional: Vec<u8>,
    ) -> dispatch::DispatchResult {
        // Ensure the call is signed and get the signer's (coldkey) account
        let coldkey = ensure_signed(origin)?;

        // Ensure that the coldkey owns the subnet
        ensure!(
            Self::get_subnet_owner(netuid) == coldkey,
            Error::<T>::NotSubnetOwner
        );

        // Create the identity struct with the provided information
        let identity: SubnetIdentityOfV2 = SubnetIdentityOfV2 {
            subnet_name,
            github_repo,
            subnet_contact,
            subnet_url,
            discord,
            description,
            additional,
        };

        // Validate the created identity
        ensure!(
            Self::is_valid_subnet_identity(&identity),
            Error::<T>::InvalidIdentity
        );

        // Store the validated identity in the blockchain state
        SubnetIdentitiesV2::<T>::insert(netuid, identity.clone());

        // Log the identity set event
        log::debug!("SubnetIdentitySet( netuid:{:?} ) ", netuid);

        // Emit an event to notify that an identity has been set
        Self::deposit_event(Event::SubnetIdentitySet(netuid));

        // Return Ok to indicate successful execution
        Ok(())
    }

    /// Associate an EVM key with a hotkey.
    ///
    /// This function accepts a Signature, which is a signed message containing the hotkey concatenated with
    /// the hashed block number. It will then attempt to recover the EVM key from the signature and compare it
    /// with the `evm_key` parameter, and ensures that they match.
    ///
    /// The EVM key is expected to sign the message according to this formula to produce the signature:
    /// ```text
    /// keccak_256(hotkey ++ keccak_256(block_number))
    /// ```
    ///
    /// # Arguments
    ///
    /// * `origin` - The origin of the call, which should be the coldkey that owns the hotkey.
    /// * `netuid` - The unique identifier for the subnet that the hotkey belongs to.
    /// * `hotkey` - The hotkey associated with the `origin` coldkey.
    /// * `evm_key` - The EVM address to associate with the `hotkey`.
    /// * `block_number` - The block number used in the `signature`.
    /// * `signature` - A signed message by the `evm_key` containing the `hotkey` and the hashed `block_number`.
    pub fn do_associate_evm_key(
        origin: T::RuntimeOrigin,
        netuid: u16,
        hotkey: T::AccountId,
        evm_key: H160,
        block_number: u64,
        signature: Signature,
    ) -> dispatch::DispatchResult {
        let coldkey = ensure_signed(origin)?;

        ensure!(
            Self::get_owning_coldkey_for_hotkey(&hotkey) == coldkey,
            Error::<T>::NonAssociatedColdKey
        );

        let uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey)?;

        let mut message = [0u8; 64];
        let block_hash = keccak_256(block_number.encode().as_ref());
        message[..32].copy_from_slice(&hotkey.encode()[..]);
        message[32..].copy_from_slice(block_hash.as_ref());
        let public = signature
            .recover_prehashed(&keccak_256(message.as_ref()))
            .ok_or(Error::<T>::UnableToRecoverPublicKey)?;
        let secp_pubkey = libsecp256k1::PublicKey::parse_compressed(&public.0)
            .map_err(|_| Error::<T>::UnableToRecoverPublicKey)?;
        let uncompressed = secp_pubkey.serialize();
        let hashed_evm_key = H160::from_slice(&keccak_256(&uncompressed[1..])[12..]);

        ensure!(
            evm_key == hashed_evm_key,
            Error::<T>::InvalidRecoveredPublicKey
        );

        let current_block_number = Self::get_current_block_as_u64();

        AssociatedEvmAddress::<T>::insert(netuid, uid, (evm_key, current_block_number));

        Self::deposit_event(Event::EvmKeyAssociated {
            netuid,
            hotkey,
            evm_key,
            block_associated: current_block_number,
        });

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
    pub fn is_valid_identity(identity: &ChainIdentityOfV2) -> bool {
        let total_length = identity
            .name
            .len()
            .saturating_add(identity.url.len())
            .saturating_add(identity.image.len())
            .saturating_add(identity.discord.len())
            .saturating_add(identity.description.len())
            .saturating_add(identity.additional.len());

        let max_length: usize = 256_usize
            .saturating_add(256)
            .saturating_add(256)
            .saturating_add(1024)
            .saturating_add(256)
            .saturating_add(1024)
            .saturating_add(1024);

        total_length <= max_length
            && identity.name.len() <= 256
            && identity.url.len() <= 256
            && identity.github_repo.len() <= 256
            && identity.image.len() <= 1024
            && identity.discord.len() <= 256
            && identity.description.len() <= 1024
            && identity.additional.len() <= 1024
    }

    /// Validates the given SubnetIdentityOf struct.
    ///
    /// This function checks if the total length of all fields in the SubnetIdentityOf struct
    /// is less than or equal to 2304 bytes, and if each individual field is also
    /// within its respective maximum byte limit.
    ///
    /// # Arguments
    ///
    /// * `identity` - A reference to the SubnetIdentityOf struct to be validated.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the SubnetIdentity is valid, false otherwise.
    pub fn is_valid_subnet_identity(identity: &SubnetIdentityOfV2) -> bool {
        let total_length = identity
            .subnet_name
            .len()
            .saturating_add(identity.github_repo.len())
            .saturating_add(identity.subnet_contact.len());

        let max_length: usize = 256_usize
            .saturating_add(1024)
            .saturating_add(1024)
            .saturating_add(1024)
            .saturating_add(256)
            .saturating_add(1024)
            .saturating_add(1024);

        total_length <= max_length
            && identity.subnet_name.len() <= 256
            && identity.github_repo.len() <= 1024
            && identity.subnet_contact.len() <= 1024
            && identity.subnet_url.len() <= 1024
            && identity.discord.len() <= 256
            && identity.description.len() <= 1024
            && identity.additional.len() <= 1024
    }
}
