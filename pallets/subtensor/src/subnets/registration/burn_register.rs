use super::*;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic do_burned_registration: registering by burning TAO.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     - The signature of the calling coldkey.
    ///       Burned registers can only be created by the coldkey.
    ///
    /// * 'netuid' (u16):
    ///     - The u16 network identifier.
    ///
    /// * 'hotkey' ( T::AccountId ):
    ///     - Hotkey to be registered to the network.
    ///
    /// # Event:
    /// * NeuronRegistered;
    ///     - On successfully registereing a uid to a neuron slot on a subnetwork.
    ///
    /// # Raises:
    /// * 'SubNetworkDoesNotExist':
    ///     - Attempting to registed to a non existent network.
    ///
    /// * 'TooManyRegistrationsThisBlock':
    ///     - This registration exceeds the total allowed on this network this block.
    ///
    /// * 'HotKeyAlreadyRegisteredInSubNet':
    ///     - The hotkey is already registered on this network.
    ///
    pub fn do_burned_registration(
        origin: T::RuntimeOrigin,
        netuid: u16,
        hotkey: T::AccountId,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_registration( coldkey:{:?} netuid:{:?} hotkey:{:?} )",
            coldkey,
            netuid,
            hotkey
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
            Self::get_network_registration_allowed(netuid),
            Error::<T>::SubNetRegistrationDisabled
        );

        // --- 4. Ensure we are not exceeding the max allowed registrations per block.
        ensure!(
            Self::get_registrations_this_block(netuid)
                < Self::get_max_registrations_per_block(netuid),
            Error::<T>::TooManyRegistrationsThisBlock
        );

        // --- 4. Ensure we are not exceeding the max allowed registrations per interval.
        ensure!(
            Self::get_registrations_this_interval(netuid)
                < Self::get_target_registrations_per_interval(netuid) * 3,
            Error::<T>::TooManyRegistrationsThisInterval
        );

        // --- 4. Ensure that the key is not already registered.
        ensure!(
            !Uids::<T>::contains_key(netuid, &hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // DEPRECATED --- 6. Ensure that the key passes the registration requirement
        // ensure!(
        //     Self::passes_network_connection_requirement(netuid, &hotkey),
        //     Error::<T>::DidNotPassConnectedNetworkRequirement
        // );

        // --- 7. Ensure the callers coldkey has enough stake to perform the transaction.
        let current_block_number: u64 = Self::get_current_block_as_u64();
        let registration_cost = Self::get_burn_as_u64(netuid);
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, registration_cost),
            Error::<T>::NotEnoughBalanceToStake
        );

        // --- 8. Ensure the remove operation from the coldkey is a success.
        let actual_burn_amount =
            Self::remove_balance_from_coldkey_account(&coldkey, registration_cost)?;

        // The burn occurs here.
        Self::burn_tokens(actual_burn_amount);

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
            // --- 12.1.1 No replacement required, the uid appends the subnetwork.
            // We increment the subnetwork count here but not below.
            subnetwork_uid = current_subnetwork_n;

            // --- 12.1.2 Expand subnetwork with new account.
            Self::append_neuron(netuid, &hotkey, current_block_number);
            log::info!("add new neuron account");
        } else {
            // --- 13.1.1 Replacement required.
            // We take the neuron with the lowest pruning score here.
            subnetwork_uid = Self::get_neuron_to_prune(netuid);

            // --- 13.1.1 Replace the neuron account with the new info.
            Self::replace_neuron(netuid, subnetwork_uid, &hotkey, current_block_number);
            log::info!("prune neuron");
        }

        // --- 14. Record the registration and increment block and interval counters.
        BurnRegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
        RegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
        RegistrationsThisBlock::<T>::mutate(netuid, |val| *val += 1);
        Self::increase_rao_recycled(netuid, Self::get_burn_as_u64(netuid));

        // --- 15. Deposit successful event.
        log::info!(
            "NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ",
            netuid,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(netuid, subnetwork_uid, hotkey));

        // --- 16. Ok and done.
        Ok(())
    }
}