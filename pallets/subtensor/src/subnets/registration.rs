use super::*;
use sp_core::{H256, U256};
use sp_io::hashing::{keccak_256, sha2_256};
use sp_runtime::Saturating;
use system::pallet_prelude::BlockNumberFor;

const LOG_TARGET: &str = "runtime::subtensor::registration";

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
        log::debug!(
            "do_registration( coldkey:{:?} netuid:{:?} hotkey:{:?} )",
            coldkey,
            netuid,
            hotkey
        );

        // --- 2. Ensure the passed network is valid.
        ensure!(
            netuid != Self::ROOT_NETUID,
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );
        ensure!(
            NetworksAdded::<T>::get(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Ensure the passed network allows registrations.
        ensure!(
            NetworkRegistrationAllowed::<T>::get(netuid),
            Error::<T>::SubNetRegistrationDisabled
        );

        // --- 4. Ensure we are not exceeding the max allowed registrations per block.
        ensure!(
            RegistrationsThisBlock::<T>::get(netuid) < MaxRegistrationsPerBlock::<T>::get(netuid),
            Error::<T>::TooManyRegistrationsThisBlock
        );

        // --- 4. Ensure we are not exceeding the max allowed registrations per interval.
        ensure!(
            RegistrationsThisInterval::<T>::get(netuid)
                < TargetRegistrationsPerInterval::<T>::get(netuid).saturating_mul(3),
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
        let registration_cost = Burn::<T>::get(netuid);
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
        let current_subnetwork_n: u16 = SubnetworkN::<T>::get(netuid);

        // Possibly there is no neuron slots at all.
        ensure!(
            MaxAllowedUids::<T>::get(netuid) != 0,
            Error::<T>::NoNeuronIdAvailable
        );

        if current_subnetwork_n < MaxAllowedUids::<T>::get(netuid) {
            // --- 12.1.1 No replacement required, the uid appends the subnetwork.
            // We increment the subnetwork count here but not below.
            subnetwork_uid = current_subnetwork_n;

            // --- 12.1.2 Expand subnetwork with new account.
            Self::append_neuron(netuid, &hotkey, current_block_number);
            log::debug!("add new neuron account");
        } else {
            // --- 13.1.1 Replacement required.
            // We take the neuron with the lowest pruning score here.
            subnetwork_uid = Self::get_neuron_to_prune(netuid);

            // --- 13.1.1 Replace the neuron account with the new info.
            Self::replace_neuron(netuid, subnetwork_uid, &hotkey, current_block_number);
            log::debug!("prune neuron");
        }

        // --- 14. Record the registration and increment block and interval counters.
        BurnRegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisBlock::<T>::mutate(netuid, |val| val.saturating_inc());
        Self::increase_rao_recycled(netuid, Burn::<T>::get(netuid));

        // --- 15. Deposit successful event.
        log::debug!(
            "NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ",
            netuid,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(netuid, subnetwork_uid, hotkey));

        // --- 16. Ok and done.
        Ok(())
    }

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
        log::debug!(
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
            netuid != Self::ROOT_NETUID,
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );
        ensure!(
            NetworksAdded::<T>::get(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Ensure the passed network allows registrations.
        ensure!(
            NetworkPowRegistrationAllowed::<T>::get(netuid),
            Error::<T>::SubNetRegistrationDisabled
        );

        // --- 4. Ensure we are not exceeding the max allowed registrations per block.
        ensure!(
            RegistrationsThisBlock::<T>::get(netuid) < MaxRegistrationsPerBlock::<T>::get(netuid),
            Error::<T>::TooManyRegistrationsThisBlock
        );

        // --- 5. Ensure we are not exceeding the max allowed registrations per interval.
        ensure!(
            RegistrationsThisInterval::<T>::get(netuid)
                < TargetRegistrationsPerInterval::<T>::get(netuid).saturating_mul(3),
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
            current_block_number.saturating_sub(block_number) < 3,
            Error::<T>::InvalidWorkBlock
        );

        // --- 8. Ensure the supplied work passes the difficulty.
        let difficulty = Difficulty::<T>::get(netuid).into();
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
        let current_subnetwork_n: u16 = SubnetworkN::<T>::get(netuid);

        // Possibly there is no neuron slots at all.
        ensure!(
            MaxAllowedUids::<T>::get(netuid) != 0,
            Error::<T>::NoNeuronIdAvailable
        );

        if current_subnetwork_n < MaxAllowedUids::<T>::get(netuid) {
            // --- 11.1.1 No replacement required, the uid appends the subnetwork.
            // We increment the subnetwork count here but not below.
            subnetwork_uid = current_subnetwork_n;

            // --- 11.1.2 Expand subnetwork with new account.
            Self::append_neuron(netuid, &hotkey, current_block_number);
            log::debug!("add new neuron account");
        } else {
            // --- 11.1.1 Replacement required.
            // We take the neuron with the lowest pruning score here.
            subnetwork_uid = Self::get_neuron_to_prune(netuid);

            // --- 11.1.1 Replace the neuron account with the new info.
            Self::replace_neuron(netuid, subnetwork_uid, &hotkey, current_block_number);
            log::debug!("prune neuron");
        }

        // --- 12. Record the registration and increment block and interval counters.
        POWRegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisBlock::<T>::mutate(netuid, |val| val.saturating_inc());

        // --- 13. Deposit successful event.
        log::debug!(
            "NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ",
            netuid,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(netuid, subnetwork_uid, hotkey));

        // --- 14. Ok and done.
        Ok(())
    }

    pub fn do_faucet(
        origin: T::RuntimeOrigin,
        block_number: u64,
        nonce: u64,
        work: Vec<u8>,
    ) -> DispatchResult {
        // --- 0. Ensure the faucet is enabled.
        // ensure!(AllowFaucet::<T>::get(), Error::<T>::FaucetDisabled);

        // --- 1. Check that the caller has signed the transaction.
        let coldkey = ensure_signed(origin)?;
        log::debug!("do_faucet( coldkey:{:?} )", coldkey);

        // --- 2. Ensure the passed block number is valid, not in the future or too old.
        // Work must have been done within 3 blocks (stops long range attacks).
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure!(
            block_number <= current_block_number,
            Error::<T>::InvalidWorkBlock
        );
        ensure!(
            current_block_number.saturating_sub(block_number) < 3,
            Error::<T>::InvalidWorkBlock
        );

        // --- 3. Ensure the supplied work passes the difficulty.
        let difficulty: U256 = if !cfg!(feature = "fast-blocks") {
            U256::from(1_000_000) // Base faucet difficulty.
        } else {
            U256::from(100) // Lowered for fast blocks
        };

        let work_hash: H256 = Self::vec_to_hash(work.clone());
        ensure!(
            Self::hash_meets_difficulty(&work_hash, difficulty),
            Error::<T>::InvalidDifficulty
        ); // Check that the work meets difficulty.

        // --- 4. Check Work is the product of the nonce, the block number, and hotkey. Add this as used work.
        let seal: H256 = Self::create_seal_hash(block_number, nonce, &coldkey);
        ensure!(seal == work_hash, Error::<T>::InvalidSeal);
        UsedWork::<T>::insert(work.clone(), current_block_number);

        // --- 5. Add Balance via faucet.
        let balance_to_add: u64 = 1_000_000_000_000;
        Self::coinbase(100_000_000_000); // We are creating tokens here from the coinbase.

        Self::add_balance_to_coldkey_account(&coldkey, balance_to_add);

        // --- 6. Deposit successful event.
        log::debug!(
            "Faucet( coldkey:{:?} amount:{:?} ) ",
            coldkey,
            balance_to_add
        );
        Self::deposit_event(Event::Faucet(coldkey, balance_to_add));

        // --- 7. Ok and done.
        Ok(())
    }

    pub fn vec_to_hash(vec_hash: Vec<u8>) -> H256 {
        let de_ref_hash = &vec_hash; // b: &Vec<u8>
        let de_de_ref_hash: &[u8] = de_ref_hash; // c: &[u8]
        let real_hash: H256 = H256::from_slice(de_de_ref_hash);
        real_hash
    }

    /// Determine which peer to prune from the network by finding the element with the lowest pruning score out of
    /// immunity period. If there is a tie for lowest pruning score, the neuron registered earliest is pruned.
    /// If all neurons are in immunity period, the neuron with the lowest pruning score is pruned. If there is a tie for
    /// the lowest pruning score, the immune neuron registered earliest is pruned.
    /// Ties for earliest registration are broken by the neuron with the lowest uid.
    pub fn get_neuron_to_prune(netuid: u16) -> u16 {
        let mut min_score: u16 = u16::MAX;
        let mut min_score_in_immunity: u16 = u16::MAX;
        let mut earliest_registration: u64 = u64::MAX;
        let mut earliest_registration_in_immunity: u64 = u64::MAX;
        let mut uid_to_prune: u16 = 0;
        let mut uid_to_prune_in_immunity: u16 = 0;

        // This boolean is used instead of checking if min_score == u16::MAX, to avoid the case
        // where all non-immune neurons have pruning score u16::MAX
        // This may be unlikely in practice.
        let mut found_non_immune = false;

        let neurons_n = SubnetworkN::<T>::get(netuid);
        if neurons_n == 0 {
            return 0; // If there are no neurons in this network.
        }

        for neuron_uid in 0..neurons_n {
            let pruning_score: u16 = Self::get_pruning_score_for_uid(netuid, neuron_uid);
            let block_at_registration: u64 = BlockAtRegistration::<T>::get(netuid, neuron_uid);
            let is_immune = Self::get_neuron_is_immune(netuid, neuron_uid);

            if is_immune {
                // if the immune neuron has a lower pruning score than the minimum for immune neurons,
                // or, if the pruning scores are equal and the immune neuron was registered earlier than the current minimum for immune neurons,
                // then update the minimum pruning score and the uid to prune for immune neurons
                if pruning_score < min_score_in_immunity
                    || (pruning_score == min_score_in_immunity
                        && block_at_registration < earliest_registration_in_immunity)
                {
                    min_score_in_immunity = pruning_score;
                    earliest_registration_in_immunity = block_at_registration;
                    uid_to_prune_in_immunity = neuron_uid;
                }
            } else {
                found_non_immune = true;
                // if the non-immune neuron has a lower pruning score than the minimum for non-immune neurons,
                // or, if the pruning scores are equal and the non-immune neuron was registered earlier than the current minimum for non-immune neurons,
                // then update the minimum pruning score and the uid to prune for non-immune neurons
                if pruning_score < min_score
                    || (pruning_score == min_score && block_at_registration < earliest_registration)
                {
                    min_score = pruning_score;
                    earliest_registration = block_at_registration;
                    uid_to_prune = neuron_uid;
                }
            }
        }

        if found_non_immune {
            Self::set_pruning_score_for_uid(netuid, uid_to_prune, u16::MAX);
            uid_to_prune
        } else {
            Self::set_pruning_score_for_uid(netuid, uid_to_prune_in_immunity, u16::MAX);
            uid_to_prune_in_immunity
        }
    }

    /// Determine whether the given hash satisfies the given difficulty.
    /// The test is done by multiplying the two together. If the product
    /// overflows the bounds of U256, then the product (and thus the hash)
    /// was too high.
    pub fn hash_meets_difficulty(hash: &H256, difficulty: U256) -> bool {
        let bytes: &[u8] = hash.as_bytes();
        let num_hash: U256 = U256::from(bytes);
        let (value, overflowed) = num_hash.overflowing_mul(difficulty);

        log::trace!(
            target: LOG_TARGET,
            "Difficulty: hash: {:?}, hash_bytes: {:?}, hash_as_num: {:?}, difficulty: {:?}, value: {:?} overflowed: {:?}",
            hash,
            bytes,
            num_hash,
            difficulty,
            value,
            overflowed
        );
        !overflowed
    }

    pub fn get_block_hash_from_u64(block_number: u64) -> H256 {
        let block_number: BlockNumberFor<T> = TryInto::<BlockNumberFor<T>>::try_into(block_number)
            .ok()
            .expect("convert u64 to block number.");
        let block_hash_at_number: <T as frame_system::Config>::Hash =
            system::Pallet::<T>::block_hash(block_number);
        let vec_hash: Vec<u8> = block_hash_at_number.as_ref().to_vec();
        let deref_vec_hash: &[u8] = &vec_hash; // c: &[u8]
        let real_hash: H256 = H256::from_slice(deref_vec_hash);

        log::trace!(
            target: LOG_TARGET,
            "block_number: {:?}, vec_hash: {:?}, real_hash: {:?}",
            block_number,
            vec_hash,
            real_hash
        );

        real_hash
    }

    pub fn hash_to_vec(hash: H256) -> Vec<u8> {
        let hash_as_bytes: &[u8] = hash.as_bytes();
        let hash_as_vec: Vec<u8> = hash_as_bytes.to_vec();
        hash_as_vec
    }

    pub fn hash_block_and_hotkey(block_hash_bytes: &[u8; 32], hotkey: &T::AccountId) -> H256 {
        let binding = hotkey.encode();
        // Safe because Substrate guarantees that all AccountId types are at least 32 bytes
        let (hotkey_bytes, _) = binding.split_at(32);
        let mut full_bytes = [0u8; 64];
        let (first_half, second_half) = full_bytes.split_at_mut(32);
        first_half.copy_from_slice(block_hash_bytes);
        second_half.copy_from_slice(hotkey_bytes);
        let keccak_256_seal_hash_vec: [u8; 32] = keccak_256(&full_bytes[..]);

        H256::from_slice(&keccak_256_seal_hash_vec)
    }

    pub fn hash_hotkey_to_u64(hotkey: &T::AccountId) -> u64 {
        let binding = hotkey.encode();
        let (hotkey_bytes, _) = binding.split_at(32);
        let mut full_bytes = [0u8; 64];
        // Copy the hotkey_bytes into the first half of full_bytes
        full_bytes[..32].copy_from_slice(hotkey_bytes);
        let keccak_256_seal_hash_vec: [u8; 32] = keccak_256(&full_bytes[..]);
        let hash_u64: u64 = u64::from_le_bytes(
            keccak_256_seal_hash_vec[0..8]
                .try_into()
                .unwrap_or_default(),
        );
        hash_u64
    }

    pub fn create_seal_hash(block_number_u64: u64, nonce_u64: u64, hotkey: &T::AccountId) -> H256 {
        let nonce = nonce_u64.to_le_bytes();
        let block_hash_at_number: H256 = Self::get_block_hash_from_u64(block_number_u64);
        let block_hash_bytes: &[u8; 32] = block_hash_at_number.as_fixed_bytes();
        let binding = Self::hash_block_and_hotkey(block_hash_bytes, hotkey);
        let block_and_hotkey_hash_bytes: &[u8; 32] = binding.as_fixed_bytes();

        let mut full_bytes = [0u8; 40];
        let (first_chunk, second_chunk) = full_bytes.split_at_mut(8);
        first_chunk.copy_from_slice(&nonce);
        second_chunk.copy_from_slice(block_and_hotkey_hash_bytes);
        let sha256_seal_hash_vec: [u8; 32] = sha2_256(&full_bytes[..]);
        let keccak_256_seal_hash_vec: [u8; 32] = keccak_256(&sha256_seal_hash_vec);
        let seal_hash: H256 = H256::from_slice(&keccak_256_seal_hash_vec);

        log::trace!(
			"\n hotkey:{:?} \nblock_number: {:?}, \nnonce_u64: {:?}, \nblock_hash: {:?}, \nfull_bytes: {:?}, \nsha256_seal_hash_vec: {:?},  \nkeccak_256_seal_hash_vec: {:?}, \nseal_hash: {:?}",
			hotkey,
            block_number_u64,
			nonce_u64,
			block_hash_at_number,
			full_bytes,
			sha256_seal_hash_vec,
            keccak_256_seal_hash_vec,
			seal_hash
		);

        seal_hash
    }

    /// Helper function for creating nonce and work.
    pub fn create_work_for_block_number(
        netuid: u16,
        block_number: u64,
        start_nonce: u64,
        hotkey: &T::AccountId,
    ) -> (u64, Vec<u8>) {
        let difficulty = Difficulty::<T>::get(netuid).into();
        let mut nonce: u64 = start_nonce;
        let mut work: H256 = Self::create_seal_hash(block_number, nonce, hotkey);
        while !Self::hash_meets_difficulty(&work, difficulty) {
            nonce.saturating_inc();
            work = Self::create_seal_hash(block_number, nonce, hotkey);
        }
        let vec_work: Vec<u8> = Self::hash_to_vec(work);
        (nonce, vec_work)
    }
}
