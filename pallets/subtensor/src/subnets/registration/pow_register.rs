use super::*;
use sp_core::{ H256, U256};

impl<T: Config> Pallet<T> {

    /// ---- The implementation for the extrinsic do_registration.
    ///
    /// # Args:
    /// *'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     - The signature of the calling hotkey.
    ///
    /// *'netuid' (u16):
    ///     - The u16 network identifier.
    ///
    /// *'block_number' ( u64 ):
    ///     - Block hash used to prove work done.
    ///
    /// *'nonce' ( u64 ):
    ///     - Positive integer nonce used in POW.
    ///
    /// *'work' ( Vec<u8> ):
    ///     - Vector encoded bytes representing work done.
    ///
    /// *'hotkey' ( T::AccountId ):
    ///     - Hotkey to be registered to the network.
    ///
    /// *'coldkey' ( T::AccountId ):
    ///     - Associated coldkey account.
    ///
    /// # Event:
    /// *NeuronRegistered;
    ///     - On successfully registereing a uid to a neuron slot on a subnetwork.
    ///
    /// # Raises:
    /// *'SubNetworkDoesNotExist':
    ///     - Attempting to registed to a non existent network.
    ///
    /// *'TooManyRegistrationsThisBlock':
    ///     - This registration exceeds the total allowed on this network this block.
    ///
    /// *'HotKeyAlreadyRegisteredInSubNet':
    ///     - The hotkey is already registered on this network.
    ///
    /// *'InvalidWorkBlock':
    ///     - The work has been performed on a stale, future, or non existent block.
    ///
    /// *'InvalidDifficulty':
    ///     - The work does not match the difficutly.
    ///
    /// *'InvalidSeal':
    ///     - The seal is incorrect.
    ///
    pub fn do_registration(
        origin: T::RuntimeOrigin,
        netuid: u16,
        block_number: u64,
        nonce: u64,
        work: Vec<u8>,
        hotkey: T::AccountId,
        coldkey: T::AccountId,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction.
        // TODO( const ): This not be the hotkey signature or else an exterior actor can register the hotkey and potentially control it?
        let signing_origin = ensure_signed(origin)?;
        log::info!(
            "do_registration( origin:{:?} netuid:{:?} hotkey:{:?}, coldkey:{:?} )",
            signing_origin,
            netuid,
            hotkey,
            coldkey
        );

        ensure!(
            signing_origin == hotkey,
            Error::<T>::TransactorAccountShouldBeHotKey
        );

        // --- 2. Ensure the passed network is valid.
        ensure!(
            netuid != Self::get_root_netuid(),
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Ensure the passed network allows registrations.
        ensure!(
            Self::get_network_pow_registration_allowed(netuid),
            Error::<T>::SubNetRegistrationDisabled
        );

        // --- 4. Ensure we are not exceeding the max allowed registrations per block.
        ensure!(
            Self::get_registrations_this_block(netuid)
                < Self::get_max_registrations_per_block(netuid),
            Error::<T>::TooManyRegistrationsThisBlock
        );

        // --- 5. Ensure we are not exceeding the max allowed registrations per interval.
        ensure!(
            Self::get_registrations_this_interval(netuid)
                < Self::get_target_registrations_per_interval(netuid) * 3,
            Error::<T>::TooManyRegistrationsThisInterval
        );

        // --- 6. Ensure that the key is not already registered.
        ensure!(
            !Uids::<T>::contains_key(netuid, &hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // --- 7. Ensure the passed block number is valid, not in the future or too old.
        // Work must have been done within 3 blocks (stops long range attacks).
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure!(
            block_number <= current_block_number,
            Error::<T>::InvalidWorkBlock
        );
        ensure!(
            current_block_number - block_number < 3,
            Error::<T>::InvalidWorkBlock
        );

        // --- 8. Ensure the supplied work passes the difficulty.
        let difficulty: U256 = Self::get_difficulty(netuid);
        let work_hash: H256 = Self::vec_to_hash(work.clone());
        ensure!(
            Self::hash_meets_difficulty(&work_hash, difficulty),
            Error::<T>::InvalidDifficulty
        ); // Check that the work meets difficulty.

        // --- 7. Check Work is the product of the nonce, the block number, and hotkey. Add this as used work.
        let seal: H256 = Self::create_seal_hash(block_number, nonce, &hotkey);
        ensure!(seal == work_hash, Error::<T>::InvalidSeal);
        UsedWork::<T>::insert(work.clone(), current_block_number);

        // DEPRECATED --- 8. Ensure that the key passes the registration requirement
        // ensure!(
        //     Self::passes_network_connection_requirement(netuid, &hotkey),
        //     Error::<T>::DidNotPassConnectedNetworkRequirement
        // );

        // --- 9. If the network account does not exist we will create it here.
        Self::create_account_if_non_existent(&coldkey, &hotkey);

        // --- 10. Ensure that the pairing is correct.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 11. Append neuron or prune it.
        let subnetwork_uid: u16;
        let current_subnetwork_n: u16 = Self::get_subnetwork_n(netuid);

        // Possibly there is no neuron slots at all.
        ensure!(
            Self::get_max_allowed_uids(netuid) != 0,
            Error::<T>::NoNeuronIdAvailable
        );

        if current_subnetwork_n < Self::get_max_allowed_uids(netuid) {
            // --- 11.1.1 No replacement required, the uid appends the subnetwork.
            // We increment the subnetwork count here but not below.
            subnetwork_uid = current_subnetwork_n;

            // --- 11.1.2 Expand subnetwork with new account.
            Self::append_neuron(netuid, &hotkey, current_block_number);
            log::info!("add new neuron account");
        } else {
            // --- 11.1.1 Replacement required.
            // We take the neuron with the lowest pruning score here.
            subnetwork_uid = Self::get_neuron_to_prune(netuid);

            // --- 11.1.1 Replace the neuron account with the new info.
            Self::replace_neuron(netuid, subnetwork_uid, &hotkey, current_block_number);
            log::info!("prune neuron");
        }

        // --- 12. Record the registration and increment block and interval counters.
        POWRegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
        RegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
        RegistrationsThisBlock::<T>::mutate(netuid, |val| *val += 1);

        // --- 13. Deposit successful event.
        log::info!(
            "NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ",
            netuid,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(netuid, subnetwork_uid, hotkey));

        // --- 14. Ok and done.
        Ok(())
    }
}
