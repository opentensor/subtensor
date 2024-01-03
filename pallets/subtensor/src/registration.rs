use
{
    super::
    {
        *
    },
    frame_system::
    {
        ensure_signed,
        pallet_prelude::
        {
            HeaderFor
        }
    },
    frame_support::
    {
        pallet_prelude::
        {
            DispatchResult,
            DispatchResultWithPostInfo
        },
        storage::
        {
            IterableStorageDoubleMap
        }
    },
    sp_std::
    {
        vec,
        vec::
        {
            Vec
        },
        convert::
        {
            TryInto
        }
    },
    sp_runtime::
    {
        MultiAddress
    },
    sp_io::
    {
        hashing::
        {
            keccak_256,
            sha2_256
        }
    },
    sp_core::
    {
        H256, 
        U256, 
        Get
    }
};

const LOG_TARGET: &'static str = "runtime::subtensor::registration";

impl<T: Config> Pallet<T> 
{
    // ---- The implementation for the extrinsic do_burned_registration: registering by burning TAO.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the calling coldkey.
    //             Burned registers can only be created by the coldkey.
    //
    // 	* 'netuid' (u16):
    // 		- The u16 network identifier.
    //
    // 	* 'hotkey' ( T::AccountId ):
    // 		- Hotkey to be registered to the network.
    //
    // # Event:
    // 	* NeuronRegistered;
    // 		- On successfully registereing a uid to a neuron slot on a subnetwork.
    //
    // # Raises:
    // 	* 'NetworkDoesNotExist':
    // 		- Attempting to registed to a non existent network.
    //
    // 	* 'TooManyRegistrationsThisBlock':
    // 		- This registration exceeds the total allowed on this network this block.
    //
    // 	* 'AlreadyRegistered':
    // 		- The hotkey is already registered on this network.
    //
    pub fn do_burned_registration(origin: T::RuntimeOrigin, netuid: u16, hotkey: T::AccountId) -> DispatchResult 
    {
        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_registration( coldkey:{:?} netuid:{:?} hotkey:{:?} )",
            coldkey,
            netuid,
            hotkey
        );

        // --- 2. Ensure the passed network is valid.
        {
            ensure!(
                netuid != Self::get_root_netuid(),
                Error::<T>::OperationNotPermittedonRootSubnet
            );
            ensure!(
                Self::if_subnet_exist(netuid),
                Error::<T>::NetworkDoesNotExist
            );
        }

        // --- 3. Ensure the passed network allows registrations.
        {
            ensure!(
                Self::get_network_registration_allowed(netuid),
                Error::<T>::RegistrationDisabled
            );
        }

        // --- 4. Ensure we are not exceeding the max allowed registrations per block.
        {
            ensure!(
                Self::get_registrations_this_block(netuid) < Self::get_max_registrations_per_block(netuid),
                Error::<T>::TooManyRegistrationsThisBlock
            );
        }

        // --- 4. Ensure we are not exceeding the max allowed registrations per interval.
        {
            ensure!(
                Self::get_registrations_this_interval(netuid) < Self::get_target_registrations_per_interval(netuid) * 3,
                Error::<T>::TooManyRegistrationsThisInterval
            );
        }

        // --- 5. Ensure that the key is not already registered.
        {
            ensure!(
                !Uids::<T>::contains_key(netuid, &hotkey),
                Error::<T>::AlreadyRegistered
            );
        }

        // DEPRECATED --- 6. Ensure that the key passes the registration requirement
        {
            //ensure!(
            //    Self::passes_network_connection_requirement(netuid, &hotkey),
            //    Error::<T>::DidNotPassConnectedNetworkRequirement
            //);
        }

        // --- 7. Ensure the callers coldkey has enough stake to perform the transaction.
        let registration_cost_as_balance:   <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
        {
            registration_cost_as_balance = Self::u64_to_balance(Self::get_burn_as_u64(netuid)).unwrap();

            ensure!(
                Self::can_remove_balance_from_coldkey_account(&coldkey, registration_cost_as_balance),
                Error::<T>::NotEnoughBalanceToStake
            );
        }

        // --- 8. Ensure the remove operation from the coldkey is a success.
        {
            ensure!(
                Self::remove_balance_from_coldkey_account(&coldkey, registration_cost_as_balance) == true,
                Error::<T>::BalanceWithdrawalError
            );

            // The burn occurs here.
            Self::burn_tokens(Self::get_burn_as_u64(netuid));
        }

        // --- 9. If the network account does not exist we will create it here.
        {
            Self::create_account_if_non_existent(&coldkey, &hotkey);
        }

        // --- 10. Ensure that the pairing is correct.
        {
            ensure!(
                Self::coldkey_owns_hotkey(&coldkey, &hotkey),
                Error::<T>::NonAssociatedColdKey
            );
        }

        // --- 11. Append neuron or prune it.
        let subnetwork_uid: u16;
        {
            let current_subnetwork_n: u16 = Self::get_subnetwork_n(netuid);
            {
                // Possibly there is no neuron slots at all.
                ensure!(
                    Self::get_max_allowed_uids(netuid) != 0,
                    Error::<T>::NetworkDoesNotExist
                );
            }

            let current_block_number: u64 = Self::get_current_block_as_u64();
            if current_subnetwork_n < Self::get_max_allowed_uids(netuid) 
            {
                // --- 12.1.1 No replacement required, the uid appends the subnetwork.
                // We increment the subnetwork count here but not below.
                subnetwork_uid = current_subnetwork_n;

                // --- 12.1.2 Expand subnetwork with new account.
                Self::append_neuron(netuid, &hotkey, current_block_number);
                log::info!("add new neuron account");
            } 
            else 
            {
                // --- 13.1.1 Replacement required.
                // We take the neuron with the lowest pruning score here.
                subnetwork_uid = Self::get_neuron_to_prune(netuid);

                // --- 13.1.1 Replace the neuron account with the new info.
                Self::replace_neuron(netuid, subnetwork_uid, &hotkey, current_block_number);
                log::info!("prune neuron");
            }
        }

        // --- 14. Record the registration and increment block and interval counters.
        {
            BurnRegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
            RegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
            RegistrationsThisBlock::<T>::mutate(netuid, |val| *val += 1);
            Self::increase_rao_recycled(netuid, Self::get_burn_as_u64(netuid));
        }

        // --- 15. Deposit successful event.
        {
            log::info!(
                "NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ",
                netuid,
                subnetwork_uid,
                hotkey
            );

            Self::deposit_event(Event::NeuronRegistered(netuid, subnetwork_uid, hotkey));
        }

        // --- 16. Ok and done.
        return Ok(());
    }

    // ---- The implementation for the extrinsic do_registration.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the calling hotkey.
    //
    // 	* 'netuid' (u16):
    // 		- The u16 network identifier.
    //
    // 	* 'block_number' ( u64 ):
    // 		- Block hash used to prove work done.
    //
    // 	* 'nonce' ( u64 ):
    // 		- Positive integer nonce used in POW.
    //
    // 	* 'work' ( Vec<u8> ):
    // 		- Vector encoded bytes representing work done.
    //
    // 	* 'hotkey' ( T::AccountId ):
    // 		- Hotkey to be registered to the network.
    //
    // 	* 'coldkey' ( T::AccountId ):
    // 		- Associated coldkey account.
    //
    // # Event:
    // 	* NeuronRegistered;
    // 		- On successfully registereing a uid to a neuron slot on a subnetwork.
    //
    // # Raises:
    // 	* 'NetworkDoesNotExist':
    // 		- Attempting to registed to a non existent network.
    //
    // 	* 'TooManyRegistrationsThisBlock':
    // 		- This registration exceeds the total allowed on this network this block.
    //
    // 	* 'AlreadyRegistered':
    // 		- The hotkey is already registered on this network.
    //
    // 	* 'InvalidWorkBlock':
    // 		- The work has been performed on a stale, future, or non existent block.
    //
    // 	* 'InvalidDifficulty':
    // 		- The work does not match the difficutly.
    //
    // 	* 'InvalidSeal':
    // 		- The seal is incorrect.
    //
    pub fn do_registration(origin: T::RuntimeOrigin, netuid: u16, block_number: u64, nonce: u64, work: Vec<u8>, hotkey: T::AccountId, coldkey: T::AccountId) 
        -> DispatchResult
    {
        // --- 1. Check that the caller has signed the transaction.
        {
            // TODO( const ): This not be the hotkey signature or else an exterior actor can register the hotkey and potentially control it?
            let signing_origin = ensure_signed(origin)?;
            log::info!(
                "do_registration( origin:{:?} netuid:{:?} hotkey:{:?}, coldkey:{:?} )",
                signing_origin,
                netuid,
                hotkey,
                coldkey
            );

            ensure!(signing_origin == hotkey, Error::<T>::HotkeyOriginMismatch);
        }

        // --- 2. Ensure the passed network is valid.
        {
            ensure!(
                netuid != Self::get_root_netuid(),
                Error::<T>::OperationNotPermittedonRootSubnet
            );

            ensure!(
                Self::if_subnet_exist(netuid),
                Error::<T>::NetworkDoesNotExist
            );
        }

        // --- 3. Ensure the passed network allows registrations.
        {
            ensure!(
                Self::get_network_pow_registration_allowed(netuid),
                Error::<T>::RegistrationDisabled
            );
        }

        // --- 4. Ensure we are not exceeding the max allowed registrations per block.
        {
            ensure!(
                Self::get_registrations_this_block(netuid) < Self::get_max_registrations_per_block(netuid),
                Error::<T>::TooManyRegistrationsThisBlock
            );
        }

        // --- 5. Ensure we are not exceeding the max allowed registrations per interval.
        {
            ensure!(
                Self::get_registrations_this_interval(netuid) < Self::get_target_registrations_per_interval(netuid) * 3,
                Error::<T>::TooManyRegistrationsThisInterval
            );
        }

        // --- 6. Ensure that the key is not already registered.
        {
            ensure!(
                !Uids::<T>::contains_key(netuid, &hotkey),
                Error::<T>::AlreadyRegistered
            );
        }

        // --- 7. Ensure the passed block number is valid, not in the future or too old.
        let current_block_number: u64 = Self::get_current_block_as_u64();
        {
            // Work must have been done within 3 blocks (stops long range attacks).
            ensure!(
                block_number <= current_block_number,
                Error::<T>::InvalidWorkBlock
            );

            ensure!(
                current_block_number - block_number < 3,
                Error::<T>::InvalidWorkBlock
            );
        }

        // --- 8. Ensure the supplied work passes the difficulty.
        let work_hash: H256;
        {
            let difficulty: U256    = Self::get_difficulty(netuid);
            work_hash               = H256::from_slice(&(work.clone()));

            ensure!(
                Self::hash_meets_difficulty(&work_hash, difficulty),
                Error::<T>::InvalidDifficulty
            ); // Check that the work meets difficulty.
        }

        // --- 9. Check Work is the product of the nonce, the block number, and hotkey. Add this as used work.
        {
            let seal:       H256 = Self::create_seal_hash(block_number, nonce, &hotkey);
            ensure!(
                seal == work_hash, 
                Error::<T>::InvalidSeal
            );

            UsedWork::<T>::insert(&work.clone(), current_block_number);
        }

        // DEPRECATED --- 10. Ensure that the key passes the registration requirement
        {
            // ensure!(
            //     Self::passes_network_connection_requirement(netuid, &hotkey),
            //     Error::<T>::DidNotPassConnectedNetworkRequirement
            // );
        }

        // --- 11. If the network account does not exist we will create it here.
        {
            Self::create_account_if_non_existent(&coldkey, &hotkey);
        }

        // --- 12. Ensure that the pairing is correct.#
        {
            ensure!(
                Self::coldkey_owns_hotkey(&coldkey, &hotkey),
                Error::<T>::NonAssociatedColdKey
            );
        }

        // --- 13. Append neuron or prune it.
        let subnetwork_uid: u16;
        {
            let current_subnetwork_n: u16 = Self::get_subnetwork_n(netuid);

            // Possibly there is no neuron slots at all.
            ensure!(
                Self::get_max_allowed_uids(netuid) != 0,
                Error::<T>::NetworkDoesNotExist
            );

            if current_subnetwork_n < Self::get_max_allowed_uids(netuid) 
            {
                // --- 13.1.1 No replacement required, the uid appends the subnetwork.
                // We increment the subnetwork count here but not below.
                subnetwork_uid = current_subnetwork_n;

                // --- 13.1.2 Expand subnetwork with new account.
                Self::append_neuron(netuid, &hotkey, current_block_number);
                log::info!("add new neuron account");
            } 
            else 
            {
                // --- 13.1.1 Replacement required.
                // We take the neuron with the lowest pruning score here.
                subnetwork_uid = Self::get_neuron_to_prune(netuid);

                // --- 13.1.2 Replace the neuron account with the new info.
                Self::replace_neuron(netuid, subnetwork_uid, &hotkey, current_block_number);
                log::info!("prune neuron");
            }
        }

        // --- 12. Record the registration and increment block and interval counters.
        {
            POWRegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
            RegistrationsThisInterval::<T>::mutate(netuid, |val| *val += 1);
            RegistrationsThisBlock::<T>::mutate(netuid, |val| *val += 1);
        }

        // --- 13. Deposit successful event.
        {
            log::info!(
                "NeuronRegistered( netuid:{:?} uid:{:?} hotkey:{:?}  ) ",
                netuid,
                subnetwork_uid,
                hotkey
            );

            Self::deposit_event(Event::NeuronRegistered(netuid, subnetwork_uid, hotkey));
        }

        // --- 14. Ok and done.
        return Ok(());
    }

    pub fn do_faucet(origin: T::RuntimeOrigin, block_number: u64, nonce: u64, work: Vec<u8>) -> DispatchResult 
    {
        // --- 0. Ensure the faucet is enabled.
        {
            // ensure!(AllowFaucet::<T>::get(), Error::<T>::FaucetDisabled);
        }

        // --- 1. Check that the caller has signed the transaction.
        let coldkey: T::AccountId;
        {
            coldkey = ensure_signed(origin)?;

            log::info!("do_faucet( coldkey:{:?} )", coldkey);
        }

        // --- 2. Ensure the passed block number is valid, not in the future or too old.
        // Work must have been done within 3 blocks (stops long range attacks).
        let current_block_number: u64;
        {
            current_block_number = Self::get_current_block_as_u64();

            ensure!(
                block_number <= current_block_number,
                Error::<T>::InvalidWorkBlock
            );

            ensure!(
                current_block_number - block_number < 3,
                Error::<T>::InvalidWorkBlock
            );
        }

        // --- 3. Ensure the supplied work passes the difficulty.
        let work_hash: H256;
        {
            let difficulty: U256    = U256::from(1_000_000); // Base faucet difficulty.
            work_hash               = H256::from_slice(&work);

            ensure!(
                Self::hash_meets_difficulty(&work_hash, difficulty),
                Error::<T>::InvalidDifficulty
            ); // Check that the work meets difficulty.
        }

        // --- 4. Check Work is the product of the nonce, the block number, and hotkey. Add this as used work.
        {
            let seal: H256 = Self::create_seal_hash(block_number, nonce, &coldkey);
            ensure!(
                seal == work_hash,
                Error::<T>::InvalidSeal
            );

            UsedWork::<T>::insert(&work.clone(), current_block_number);
        }

        // --- 5. Add Balance via faucet.
        let balance_to_add: u64;
        {
            balance_to_add = 100_000_000_000;
            Self::add_balance_to_coldkey_account(
                &coldkey, 
                Self::u64_to_balance(balance_to_add).unwrap()
            );

            TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add(balance_to_add));
        }

        // --- 6. Deposit successful event.
        {
            log::info!(
                "Faucet( coldkey:{:?} amount:{:?} ) ",
                coldkey,
                balance_to_add
            );

            Self::deposit_event(Event::Faucet(coldkey, balance_to_add));
        }

        // --- 7. Ok and done.
        return Ok(());
    }

    // Determine which peer to prune from the network by finding the element with the lowest pruning score out of
    // immunity period. If all neurons are in immunity period, return node with lowest prunning score.
    // This function will always return an element to prune.
    pub fn get_neuron_to_prune(netuid: u16) -> u16 
    {
        let mut min_score:                              u16 = u16::MAX;
        let mut min_score_in_immunity_period:           u16 = u16::MAX;
        let mut uid_with_min_score:                     u16 = 0;
        let mut uid_with_min_score_in_immunity_period:  u16 = 0;
        if Self::get_subnetwork_n(netuid) == 0  // If there are no neurons in this network.
        {
            return 0;
        }

        for neuron_uid_i in 0..Self::get_subnetwork_n(netuid) 
        {
            let pruning_score:          u16 = Self::get_pruning_score_for_uid(netuid, neuron_uid_i);
            let block_at_registration:  u64 = Self::get_neuron_block_at_registration(netuid, neuron_uid_i);
            let current_block:          u64 = Self::get_current_block_as_u64();
            let immunity_period:        u64 = Self::get_immunity_period(netuid) as u64;
            if min_score == pruning_score 
            {
                if current_block - block_at_registration < immunity_period 
                {
                    //neuron is in immunity period
                    if min_score_in_immunity_period > pruning_score 
                    {
                        min_score_in_immunity_period            = pruning_score;
                        uid_with_min_score_in_immunity_period   = neuron_uid_i;
                    }
                } 
                else 
                {
                    min_score           = pruning_score;
                    uid_with_min_score  = neuron_uid_i;
                }
            }
            else if min_score > pruning_score // Find min pruning score.
            {
                if current_block - block_at_registration < immunity_period 
                {
                    //neuron is in immunity period
                    if min_score_in_immunity_period > pruning_score 
                    {
                        min_score_in_immunity_period            = pruning_score;
                        uid_with_min_score_in_immunity_period   = neuron_uid_i;
                    }
                } 
                else 
                {
                    min_score           = pruning_score;
                    uid_with_min_score  = neuron_uid_i;
                }
            }
        }

        if min_score == u16::MAX 
        {
            //all neuorns are in immunity period
            Self::set_pruning_score_for_uid(
                netuid,
                uid_with_min_score_in_immunity_period,
                u16::MAX,
            );

            return uid_with_min_score_in_immunity_period;
        }

        Self::set_pruning_score_for_uid(netuid, uid_with_min_score, u16::MAX);

        return uid_with_min_score;
    }

    // Determine whether the given hash satisfies the given difficulty.
    // The test is done by multiplying the two together. If the product
    // overflows the bounds of U256, then the product (and thus the hash)
    // was too high.
    pub fn hash_meets_difficulty(hash: &H256, difficulty: U256) -> bool 
    {
        let bytes:      &[u8]   = &hash.as_bytes();
        let num_hash:   U256    = U256::from(bytes);
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

        return !(overflowed);
    }

    pub fn get_block_hash_from_u64(block_number: u64) -> H256 
    {
        let block_number: <HeaderFor<T> as sp_runtime::traits::Header>::Number = TryInto::<<HeaderFor<T> as sp_runtime::traits::Header>::Number>::try_into(block_number)
            .ok()
            .expect("convert u64 to block number.");

        let block_hash_at_number:   <T as frame_system::Config>::Hash   = system::Pallet::<T>::block_hash(block_number);
        let vec_hash:               Vec<u8>                             = block_hash_at_number.as_ref().into_iter().cloned().collect();
        let real_hash:              H256                                = H256::from_slice(&vec_hash);

        log::trace!(
            target: LOG_TARGET,
            "block_number: {:?}, vec_hash: {:?}, real_hash: {:?}",
            block_number,
            vec_hash,
            real_hash
        );

        return real_hash;
    }

    pub fn hash_to_vec(hash: H256) -> Vec<u8> 
    {
        return Vec::from(hash.as_bytes());
    }

    pub fn hash_block_and_hotkey(block_hash_bytes: &[u8], hotkey: &T::AccountId) -> H256 
    {
        // Get the public key from the account id.
        let hotkey_pubkey:              MultiAddress<T::AccountId, ()>  = MultiAddress::Id(hotkey.clone());
        let binding:                    Vec<u8>                         = hotkey_pubkey.encode();

        // Skip extra 0th byte.
        let hotkey_bytes:               &[u8]                           = binding[1..].as_ref();
                                                                            // 0-31 = block_hash_bytes, 32-63 = hotkey_bytes
                                                                            
        let full_bytes                                                  = [&block_hash_bytes[0..31], &hotkey_bytes[0..31]].concat();
        let keccak_256_seal_hash_vec:   [u8; 32]                        = keccak_256(full_bytes.as_slice());
        let seal_hash:                  H256                            = H256::from_slice(&keccak_256_seal_hash_vec);

        return seal_hash;
    }

    pub fn create_seal_hash(block_number_u64: u64, nonce_u64: u64, hotkey: &T::AccountId) -> H256 
    {
        let nonce:                          U256        = U256::from(nonce_u64);
        let block_hash_at_number:           H256        = Self::get_block_hash_from_u64(block_number_u64);
        let block_hash_bytes:               &[u8]       = block_hash_at_number.as_bytes();
        let binding:                        H256        = Self::hash_block_and_hotkey(block_hash_bytes, hotkey);
        let block_and_hotkey_hash_bytes:    &[u8]       = binding.as_ref();

        let nonce_bytes:                    &[u8]       = &[
                                                            nonce.byte(0), nonce.byte(1), nonce.byte(2), nonce.byte(3), 
                                                            nonce.byte(4), nonce.byte(5), nonce.byte(6), nonce.byte(7)
                                                        ];

        let full_bytes                                  = [
                                                            &nonce_bytes[0..7],
                                                            &block_and_hotkey_hash_bytes[0..31]
                                                        ].concat();

        let sha256_seal_hash_vec:           [u8; 32]    = sha2_256(full_bytes.as_slice());
        let keccak_256_seal_hash_vec:       [u8; 32]    = keccak_256(&sha256_seal_hash_vec);
        let seal_hash:                      H256        = H256::from_slice(&keccak_256_seal_hash_vec);

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

        return seal_hash;
    }

    // Helper function for creating nonce and work.
    pub fn create_work_for_block_number(netuid: u16, block_number: u64, start_nonce: u64, hotkey: &T::AccountId) -> (u64, Vec<u8>) 
    {
        let difficulty: U256    = Self::get_difficulty(netuid);
        let mut nonce:  u64     = start_nonce;
        let mut work:   H256    = Self::create_seal_hash(block_number, nonce, &hotkey);
        while !Self::hash_meets_difficulty(&work, difficulty) 
        {
            nonce += 1;

            work = Self::create_seal_hash(block_number, nonce, &hotkey);
        }

        return (nonce, Self::hash_to_vec(work));
    }

    pub fn do_swap_hotkey(origin: T::RuntimeOrigin, old_hotkey: &T::AccountId, new_hotkey: &T::AccountId) -> DispatchResultWithPostInfo 
    {
        let coldkey: T::AccountId = ensure_signed(origin)?;
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, old_hotkey), 
            Error::<T>::NonAssociatedColdKey
        );


        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::TxRateLimitExceeded
        );

        let mut weight = T::DbWeight::get().reads_writes(2, 0);
        weight.saturating_accrue(T::DbWeight::get().reads(2));

        ensure!(
            old_hotkey != new_hotkey,
            Error::<T>::AlreadyRegistered
        );

        ensure!(
            !Self::is_hotkey_registered_on_any_network(new_hotkey), 
            Error::<T>::AlreadyRegistered
        );  

        weight.saturating_accrue(T::DbWeight::get().reads((TotalNetworks::<T>::get() + 1u16) as u64));

        let swap_cost = 1_000_000_000u64;
        let swap_cost_as_balance = Self::u64_to_balance(swap_cost).unwrap();

        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, swap_cost_as_balance),
            Error::<T>::NotEnoughBalance
        );

        ensure!(
            Self::remove_balance_from_coldkey_account(&coldkey, swap_cost_as_balance) == true,
            Error::<T>::BalanceWithdrawalError
        );

        Self::burn_tokens(swap_cost);

        Owner::<T>::remove(old_hotkey);
        Owner::<T>::insert(new_hotkey, coldkey.clone());
        weight.saturating_accrue(T::DbWeight::get().writes(2));

        if let Ok(total_hotkey_stake) = TotalHotkeyStake::<T>::try_get(old_hotkey) 
        {
            TotalHotkeyStake::<T>::remove(old_hotkey);
            TotalHotkeyStake::<T>::insert(new_hotkey, total_hotkey_stake);

            weight.saturating_accrue(T::DbWeight::get().writes(2));
        }

        if let Ok(delegate_take) = Delegates::<T>::try_get(old_hotkey) 
        {
            Delegates::<T>::remove(old_hotkey);
            Delegates::<T>::insert(new_hotkey, delegate_take);

            weight.saturating_accrue(T::DbWeight::get().writes(2));
        }

        if let Ok(last_tx) = LastTxBlock::<T>::try_get(old_hotkey) 
        {
            LastTxBlock::<T>::remove(old_hotkey);
            LastTxBlock::<T>::insert(new_hotkey, last_tx);

            weight.saturating_accrue(T::DbWeight::get().writes(2));
        }

        let mut coldkey_stake: Vec<(T::AccountId, u64)> = vec![];
        for (coldkey, stake_amount) in Stake::<T>::iter_prefix(old_hotkey) 
        {
            coldkey_stake.push((coldkey.clone(), stake_amount));
        }

        let _ = Stake::<T>::clear_prefix(old_hotkey, coldkey_stake.len() as u32, None);
        weight.saturating_accrue(T::DbWeight::get().writes(coldkey_stake.len() as u64));

        for (coldkey, stake_amount) in coldkey_stake 
        {
            Stake::<T>::insert(new_hotkey, coldkey, stake_amount);
            weight.saturating_accrue(T::DbWeight::get().writes(1));
        }

        let mut netuid_is_member: Vec<u16> = vec![];
        for netuid in <IsNetworkMember<T> as IterableStorageDoubleMap<T::AccountId, u16, bool>>::iter_key_prefix(old_hotkey) 
        {
            netuid_is_member.push(netuid);
        }

        let _ = IsNetworkMember::<T>::clear_prefix(old_hotkey, netuid_is_member.len() as u32, None);
        weight.saturating_accrue(T::DbWeight::get().writes(netuid_is_member.len() as u64));

        for netuid in netuid_is_member.iter() 
        {
            IsNetworkMember::<T>::insert(new_hotkey, netuid, true);
            weight.saturating_accrue(T::DbWeight::get().writes(1));
        }

        for netuid in netuid_is_member.iter() 
        {
            if let Ok(axon_info) = Axons::<T>::try_get(netuid, old_hotkey) 
            {
                Axons::<T>::remove(netuid, old_hotkey);
                Axons::<T>::insert(netuid, new_hotkey, axon_info);

                weight.saturating_accrue(T::DbWeight::get().writes(2));
            }
        }

        for netuid in netuid_is_member.iter() 
        {
            if let Ok(uid) = Uids::<T>::try_get(netuid, old_hotkey) 
            {
                Uids::<T>::remove(netuid, old_hotkey);
                Uids::<T>::insert(netuid, new_hotkey, uid);

                weight.saturating_accrue(T::DbWeight::get().writes(2));

                Keys::<T>::insert(netuid, uid, new_hotkey);

                weight.saturating_accrue(T::DbWeight::get().writes(1));

                LoadedEmission::<T>::mutate(netuid, |emission_exists| {
                    match emission_exists 
                    {
                        Some(emissions) => 
                        {
                            if let Some(emission) = emissions.get_mut(uid as usize) 
                            {
                                let (_, se, ve) = emission;

                                *emission = (new_hotkey.clone(), *se, *ve);
                            }
                        }

                        None => {}
                    }
                });

                weight.saturating_accrue(T::DbWeight::get().writes(1));
            }
        }

        Self::set_last_tx_block(&coldkey, block);
        weight.saturating_accrue(T::DbWeight::get().writes(1));

        Self::deposit_event(Event::HotkeySwapped{coldkey, old_hotkey: old_hotkey.clone(), new_hotkey: new_hotkey.clone()});

        return Ok(Some(weight).into());
    }

    pub fn get_network_registration_allowed(netuid: u16) -> bool 
    {
        return NetworkRegistrationAllowed::<T>::get(netuid);
    }

    pub fn set_network_registration_allowed(netuid: u16, registration_allowed: bool) 
    {
        NetworkRegistrationAllowed::<T>::insert(netuid, registration_allowed);

        Self::deposit_event(Event::RegistrationAllowed(netuid, registration_allowed));
    }

    pub fn get_network_pow_registration_allowed(netuid: u16) -> bool 
    {
        return NetworkPowRegistrationAllowed::<T>::get(netuid);
    }

    pub fn set_network_pow_registration_allowed(netuid: u16, registration_allowed: bool) 
    {
        NetworkPowRegistrationAllowed::<T>::insert(netuid, registration_allowed);

        Self::deposit_event(Event::PowRegistrationAllowed(netuid, registration_allowed));
    }

    pub fn get_target_registrations_per_interval(netuid: u16) -> u16 
    {
        return TargetRegistrationsPerInterval::<T>::get(netuid);
    }

    pub fn set_target_registrations_per_interval(netuid: u16, target_registrations_per_interval: u16) 
    {
        TargetRegistrationsPerInterval::<T>::insert(netuid, target_registrations_per_interval);

        Self::deposit_event(Event::RegistrationPerIntervalSet(netuid, target_registrations_per_interval));
    }

    pub fn get_max_registrations_per_block(netuid: u16) -> u16 
    {
        return MaxRegistrationsPerBlock::<T>::get(netuid);
    }

    pub fn set_max_registrations_per_block(netuid: u16, max_registrations_per_block: u16) 
    {
        MaxRegistrationsPerBlock::<T>::insert(netuid, max_registrations_per_block);

        Self::deposit_event(Event::MaxRegistrationsPerBlockSet(netuid, max_registrations_per_block));
    }

        // Registers a user's hotkey to the root network.
    //
    // This function is responsible for registering the hotkey of a user.
    // The root key with the least stake if pruned in the event of a filled network.
    //
    // # Arguments:
    // * 'origin': Represents the origin of the call.
    // * 'hotkey': The hotkey that the user wants to register to the root network.
    //
    // # Returns:
    // * 'DispatchResult': A result type indicating success or failure of the registration.
    //
    pub fn do_root_register(origin: T::RuntimeOrigin, hotkey: T::AccountId) -> DispatchResult 
    {
        // --- 0. Get the unique identifier (UID) for the root network.
        let root_netuid:            u16;
        let current_block_number:   u64;
        {
            root_netuid             = Self::get_root_netuid();
            current_block_number    = Self::get_current_block_as_u64();

            ensure!(
                Self::if_subnet_exist(root_netuid),
                Error::<T>::NetworkDoesNotExist
            );
        }

        // --- 1. Ensure that the call originates from a signed source and retrieve the caller's account ID (coldkey).
        let coldkey: T::AccountId;
        {
            coldkey = ensure_signed(origin)?;

            log::info!(
                "do_root_register( coldkey: {:?}, hotkey: {:?} )",
                coldkey,
                hotkey
            );
        }

        // --- 2. Ensure that the number of registrations in this block doesn't exceed the allowed limit.
        {
            ensure!(
                Self::get_registrations_this_block(root_netuid) < Self::get_max_registrations_per_block(root_netuid),
                Error::<T>::TooManyRegistrationsThisBlock
            );
        }

        // --- 3. Ensure that the number of registrations in this interval doesn't exceed thrice the target limit.
        {
            ensure!(
                Self::get_registrations_this_interval(root_netuid) < Self::get_target_registrations_per_interval(root_netuid) * 3,
                Error::<T>::TooManyRegistrationsThisInterval
            );
        }

        // --- 4. Check if the hotkey is already registered. If so, error out.
        {
            ensure!(
                !Uids::<T>::contains_key(root_netuid, &hotkey),
                Error::<T>::AlreadyRegistered
            );
        }

        // --- 6. Create a network account for the user if it doesn't exist.
        {
            Self::create_account_if_non_existent(&coldkey, &hotkey);
        }

        // --- 7. Fetch the current size of the subnetwork.
        // Declare a variable to hold the root UID.
        let subnetwork_uid:                 u16;
        let current_num_root_validators:    u16;
        {
            current_num_root_validators = Self::get_num_root_validators();        
        }

        // --- 8. Check if the root net is below its allowed size.
        // max allowed is senate size.
        if current_num_root_validators < Self::get_max_root_validators() 
        {
            // --- 8.1.1 We can append to the subnetwork as it's not full.
            subnetwork_uid = current_num_root_validators;

            // --- 8.1.2 Add the new account and make them a member of the Senate.
            Self::append_neuron(root_netuid, &hotkey, current_block_number);
            log::info!("add new neuron: {:?} on uid {:?}", hotkey, subnetwork_uid);
        } 
        else 
        {
            // --- 9.1.1 The network is full. Perform replacement.
            // Find the neuron with the lowest stake value to replace.
            let mut lowest_stake: u64 = u64::MAX;
            let mut lowest_uid: u16 = 0;

            // Iterate over all keys in the root network to find the neuron with the lowest stake.
            for (uid_i, hotkey_i) in <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(root_netuid)
            {
                let stake_i: u64 = Self::get_total_stake_for_hotkey(&hotkey_i);
                if stake_i < lowest_stake 
                {
                    lowest_stake = stake_i;
                    lowest_uid = uid_i;
                }
            }
            subnetwork_uid = lowest_uid;
            let replaced_hotkey: T::AccountId = Self::get_hotkey_for_net_and_uid(root_netuid, subnetwork_uid).unwrap();

            // --- 9.1.2 The new account has a higher stake than the one being replaced.
            ensure!(
                lowest_stake < Self::get_total_stake_for_hotkey(&hotkey),
                Error::<T>::StakeTooLowForRoot
            );

            // --- 9.1.3 The new account has a higher stake than the one being replaced.
            // Replace the neuron account with new information.
            Self::replace_neuron(root_netuid, lowest_uid, &hotkey, current_block_number);

            log::info!(
                "replace neuron: {:?} with {:?} on uid {:?}",
                replaced_hotkey,
                hotkey,
                subnetwork_uid
            );
        }

        // --- 10. Force all members on root to become a delegate.
        {
            if !Self::hotkey_is_delegate(&hotkey) 
            {
                Self::delegate_hotkey(&hotkey, 11_796); // 18% cut defaulted.
            }
        }

        // --- 11. Update the registration counters for both the block and interval.
        {
            RegistrationsThisInterval::<T>::mutate(root_netuid, |val| *val += 1);
            RegistrationsThisBlock::<T>::mutate(root_netuid, |val| *val += 1);
        }

        // --- 12. Log and announce the successful registration.
        {
            log::info!(
                "RootRegistered(netuid:{:?} uid:{:?} hotkey:{:?})",
                root_netuid,
                subnetwork_uid,
                hotkey
            );
            
            Self::deposit_event(Event::NeuronRegistered(root_netuid, subnetwork_uid, hotkey));
        }

        // --- 16. Finish and return success.
        return Ok(());
    }
}
