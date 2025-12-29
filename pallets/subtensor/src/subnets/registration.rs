use super::*;
use sp_core::{H256, U256};
use sp_io::hashing::{keccak_256, sha2_256};
use sp_runtime::Saturating;
use subtensor_runtime_common::{Currency, NetUid};
use subtensor_swap_interface::SwapHandler;
use system::pallet_prelude::BlockNumberFor;

const LOG_TARGET: &str = "runtime::subtensor::registration";

impl<T: Config> Pallet<T> {
    pub fn register_neuron(netuid: NetUid, hotkey: &T::AccountId) -> Result<u16, DispatchError> {
        let block_number: u64 = Self::get_current_block_as_u64();
        let current_subnetwork_n: u16 = Self::get_subnetwork_n(netuid);

        if current_subnetwork_n < Self::get_max_allowed_uids(netuid) {
            // No replacement required, the uid appends the subnetwork.
            let neuron_uid = current_subnetwork_n;

            // Expand subnetwork with new account.
            Self::append_neuron(netuid, hotkey, block_number);
            log::debug!("add new neuron account");

            Ok(neuron_uid)
        } else {
            match Self::get_neuron_to_prune(netuid) {
                Some(uid_to_replace) => {
                    Self::replace_neuron(netuid, uid_to_replace, hotkey, block_number);
                    log::debug!("prune neuron");
                    Ok(uid_to_replace)
                }
                None => Err(Error::<T>::NoNeuronIdAvailable.into()),
            }
        }
    }

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
    /// * 'MechanismDoesNotExist':
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
        netuid: NetUid,
        hotkey: T::AccountId,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::debug!("do_registration( coldkey:{coldkey:?} netuid:{netuid:?} hotkey:{hotkey:?} )");

        // --- 2. Ensure the passed network is valid.
        ensure!(
            !netuid.is_root(),
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

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

        // --- 5. Ensure we are not exceeding the max allowed registrations per interval.
        ensure!(
            Self::get_registrations_this_interval(netuid)
                < Self::get_target_registrations_per_interval(netuid).saturating_mul(3),
            Error::<T>::TooManyRegistrationsThisInterval
        );

        // --- 6. Ensure that the key is not already registered.
        ensure!(
            !Uids::<T>::contains_key(netuid, &hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // --- 7. Ensure the callers coldkey has enough stake to perform the transaction.
        let registration_cost = Self::get_burn(netuid);
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, registration_cost.into()),
            Error::<T>::NotEnoughBalanceToStake
        );

        // If the network account does not exist we will create it here.
        Self::create_account_if_non_existent(&coldkey, &hotkey);

        // --- 8. Ensure that the pairing is correct.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 9. Possibly there are no neuron slots at all.
        ensure!(
            Self::get_max_allowed_uids(netuid) != 0,
            Error::<T>::NoNeuronIdAvailable
        );

        // --- 10. If replacement is needed, ensure a safe prune candidate exists.
        let current_n = Self::get_subnetwork_n(netuid);
        let max_n = Self::get_max_allowed_uids(netuid);
        if current_n >= max_n {
            ensure!(
                Self::get_neuron_to_prune(netuid).is_some(),
                Error::<T>::NoNeuronIdAvailable
            );
        }

        // --- 11. Ensure the remove operation from the coldkey is a success.
        let actual_burn_amount =
            Self::remove_balance_from_coldkey_account(&coldkey, registration_cost.into())?;

        // Tokens are swapped and then burned.
        let burned_alpha = Self::swap_tao_for_alpha(
            netuid,
            actual_burn_amount,
            T::SwapInterface::max_price(),
            false,
        )?
        .amount_paid_out;
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub(burned_alpha.into())
        });

        // Actually perform the registration.
        let neuron_uid: u16 = Self::register_neuron(netuid, &hotkey)?;

        // --- 12. Record the registration and increment block and interval counters.
        BurnRegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisBlock::<T>::mutate(netuid, |val| val.saturating_inc());
        Self::increase_rao_recycled(netuid, Self::get_burn(netuid).into());

        // --- 13. Deposit successful events.
        log::debug!("NeuronRegistered( netuid:{netuid:?} uid:{neuron_uid:?} hotkey:{hotkey:?}  ) ");
        Self::deposit_event(Event::NeuronRegistered(
            netuid,
            neuron_uid,
            hotkey,
            actual_burn_amount.into(),
            burned_alpha.into(),
        ));

        // --- 14. Ok and done.
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
    /// *'MechanismDoesNotExist':
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
        netuid: NetUid,
        block_number: u64,
        nonce: u64,
        work: Vec<u8>,
        hotkey: T::AccountId,
        coldkey: T::AccountId,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction.
        let signing_origin = ensure_signed(origin)?;
        log::debug!(
            "do_registration( origin:{signing_origin:?} netuid:{netuid:?} hotkey:{hotkey:?}, coldkey:{coldkey:?} )"
        );

        ensure!(
            signing_origin == hotkey,
            Error::<T>::TransactorAccountShouldBeHotKey
        );

        // --- 2. Ensure the passed network is valid.
        ensure!(
            !netuid.is_root(),
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

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
                < Self::get_target_registrations_per_interval(netuid).saturating_mul(3),
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
        let difficulty: U256 = Self::get_difficulty(netuid);
        let work_hash: H256 = Self::vec_to_hash(work.clone());
        ensure!(
            Self::hash_meets_difficulty(&work_hash, difficulty),
            Error::<T>::InvalidDifficulty
        ); // Check that the work meets difficulty.

        // --- 9. Check Work is the product of the nonce, the block number, and hotkey. Add this as used work.
        let seal: H256 = Self::create_seal_hash(block_number, nonce, &hotkey);
        ensure!(seal == work_hash, Error::<T>::InvalidSeal);
        UsedWork::<T>::insert(work.clone(), current_block_number);

        // --- 10. If the network account does not exist we will create it here.
        Self::create_account_if_non_existent(&coldkey, &hotkey);

        // --- 11. Ensure that the pairing is correct.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 12. Possibly there is no neuron slots at all.
        ensure!(
            Self::get_max_allowed_uids(netuid) != 0,
            Error::<T>::NoNeuronIdAvailable
        );

        // --- 13. If replacement is needed, ensure a safe prune candidate exists.
        let current_n = Self::get_subnetwork_n(netuid);
        let max_n = Self::get_max_allowed_uids(netuid);
        if current_n >= max_n {
            ensure!(
                Self::get_neuron_to_prune(netuid).is_some(),
                Error::<T>::NoNeuronIdAvailable
            );
        }

        // Actually perform the registration.
        let neuron_uid: u16 = Self::register_neuron(netuid, &hotkey)?;

        // --- 14. Record the registration and increment block and interval counters.
        POWRegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisInterval::<T>::mutate(netuid, |val| val.saturating_inc());
        RegistrationsThisBlock::<T>::mutate(netuid, |val| val.saturating_inc());

        // --- 15. Deposit successful event.
        log::debug!("NeuronRegistered( netuid:{netuid:?} uid:{neuron_uid:?} hotkey:{hotkey:?}  ) ");
        Self::deposit_event(Event::NeuronRegistered(netuid, neuron_uid, hotkey, 0u64.into(), 0u64.into()));

        // --- 16. Ok and done.
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
        log::debug!("do_faucet( coldkey:{coldkey:?} )");

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
        let difficulty: U256 = U256::from(1_000_000); // Base faucet difficulty.
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
        Self::increase_issuance(100_000_000_000.into()); // We are creating tokens here from the coinbase.

        Self::add_balance_to_coldkey_account(&coldkey, balance_to_add);

        // --- 6. Deposit successful event.
        log::debug!("Faucet( coldkey:{coldkey:?} amount:{balance_to_add:?} ) ");
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

    fn get_immune_owner_hotkeys(netuid: NetUid, coldkey: &T::AccountId) -> Vec<T::AccountId> {
        Self::get_immune_owner_tuples(netuid, coldkey)
            .into_iter()
            .map(|(_, hk)| hk)
            .collect()
    }

    pub fn get_immune_owner_uids(netuid: NetUid, coldkey: &T::AccountId) -> Vec<u16> {
        Self::get_immune_owner_tuples(netuid, coldkey)
            .into_iter()
            .map(|(uid, _)| uid)
            .collect()
    }

    fn get_immune_owner_tuples(netuid: NetUid, coldkey: &T::AccountId) -> Vec<(u16, T::AccountId)> {
        // Gather (block, uid, hotkey) only for hotkeys that have a UID and a registration block.
        let mut triples: Vec<(u64, u16, T::AccountId)> = OwnedHotkeys::<T>::get(coldkey)
            .into_iter()
            .filter_map(|hotkey| {
                // Uids must exist, filter_map ignores hotkeys without UID
                Uids::<T>::get(netuid, &hotkey).map(|uid| {
                    let block = BlockAtRegistration::<T>::get(netuid, uid);
                    (block, uid, hotkey)
                })
            })
            .collect();

        // Sort by BlockAtRegistration (ascending), then by uid (ascending)
        // Recent registration is priority so that we can let older keys expire (get non-immune)
        triples.sort_by(|(b1, u1, _), (b2, u2, _)| b1.cmp(b2).then(u1.cmp(u2)));

        // Keep first ImmuneOwnerUidsLimit
        let limit = ImmuneOwnerUidsLimit::<T>::get(netuid).into();
        if triples.len() > limit {
            triples.truncate(limit);
        }

        // Project to uid/hotkey tuple
        let mut immune_tuples: Vec<(u16, T::AccountId)> =
            triples.into_iter().map(|(_, uid, hk)| (uid, hk)).collect();

        // Insert subnet owner hotkey in the beginning of the list if valid and not
        // already present
        if let Ok(owner_hk) = SubnetOwnerHotkey::<T>::try_get(netuid)
            && let Some(owner_uid) = Uids::<T>::get(netuid, &owner_hk)
            && !immune_tuples.contains(&(owner_uid, owner_hk.clone()))
        {
            immune_tuples.insert(0, (owner_uid, owner_hk.clone()));
            if immune_tuples.len() > limit {
                immune_tuples.truncate(limit);
            }
        }

        immune_tuples
    }

    /// Determine which neuron to prune.
    pub fn get_neuron_to_prune(netuid: NetUid) -> Option<u16> {
        let n = Self::get_subnetwork_n(netuid);
        if n == 0 {
            return None;
        }

        let owner_ck = SubnetOwner::<T>::get(netuid);
        let immortal_hotkeys = Self::get_immune_owner_hotkeys(netuid, &owner_ck);
        let emissions: Vec<AlphaCurrency> = Emission::<T>::get(netuid);

        // Single pass:
        // - count current non‑immortal & non‑immune UIDs,
        // - track best non‑immune and best immune candidates separately.
        let mut free_count: u16 = 0;

        // (emission, reg_block, uid)
        let mut best_non_immune: Option<(AlphaCurrency, u64, u16)> = None;
        let mut best_immune: Option<(AlphaCurrency, u64, u16)> = None;

        for uid in 0..n {
            let hk = match Self::get_hotkey_for_net_and_uid(netuid, uid) {
                Ok(h) => h,
                Err(_) => continue,
            };

            // Skip owner‑immortal hotkeys entirely.
            if immortal_hotkeys.contains(&hk) {
                continue;
            }

            let is_immune = Self::get_neuron_is_immune(netuid, uid);
            let emission = emissions
                .get(uid as usize)
                .cloned()
                .unwrap_or(AlphaCurrency::ZERO);
            let reg_block = Self::get_neuron_block_at_registration(netuid, uid);

            // Helper to decide if (e, b, u) beats the current best.
            let consider = |best: &mut Option<(AlphaCurrency, u64, u16)>| match best {
                None => *best = Some((emission, reg_block, uid)),
                Some((be, bb, bu)) => {
                    let better = if emission != *be {
                        emission < *be
                    } else if reg_block != *bb {
                        reg_block < *bb
                    } else {
                        uid < *bu
                    };
                    if better {
                        *best = Some((emission, reg_block, uid));
                    }
                }
            };

            if is_immune {
                consider(&mut best_immune);
            } else {
                free_count = free_count.saturating_add(1);
                consider(&mut best_non_immune);
            }
        }

        // No candidates left after filtering out owner‑immortal hotkeys.
        if best_non_immune.is_none() && best_immune.is_none() {
            return None;
        }

        // Safety floor for non‑immortal & non‑immune UIDs.
        let min_free: u16 = Self::get_min_non_immune_uids(netuid);
        let can_prune_non_immune = free_count > min_free;

        // Prefer non‑immune if allowed; otherwise fall back to immune.
        if can_prune_non_immune && let Some((_, _, uid)) = best_non_immune {
            return Some(uid);
        }
        best_immune.map(|(_, _, uid)| uid)
    }

    /// Determine whether the given hash satisfies the given difficulty.
    /// The test is done by multiplying the two together. If the product
    /// overflows the bounds of U256, then the product (and thus the hash)
    /// was too high.
    pub fn hash_meets_difficulty(hash: &H256, difficulty: U256) -> bool {
        let bytes: &[u8] = hash.as_bytes();
        let num_hash: U256 = U256::from_little_endian(bytes);
        let (value, overflowed) = num_hash.overflowing_mul(difficulty);

        log::trace!(
            target: LOG_TARGET,
            "Difficulty: hash: {hash:?}, hash_bytes: {bytes:?}, hash_as_num: {num_hash:?}, difficulty: {difficulty:?}, value: {value:?} overflowed: {overflowed:?}"
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
            "block_number: {block_number:?}, vec_hash: {vec_hash:?}, real_hash: {real_hash:?}"
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
            "\n hotkey:{hotkey:?} \nblock_number: {block_number_u64:?}, \nnonce_u64: {nonce_u64:?}, \nblock_hash: {block_hash_at_number:?}, \nfull_bytes: {full_bytes:?}, \nsha256_seal_hash_vec: {sha256_seal_hash_vec:?},  \nkeccak_256_seal_hash_vec: {keccak_256_seal_hash_vec:?}, \nseal_hash: {seal_hash:?}"
        );

        seal_hash
    }

    /// Helper function for creating nonce and work.
    pub fn create_work_for_block_number(
        netuid: NetUid,
        block_number: u64,
        start_nonce: u64,
        hotkey: &T::AccountId,
    ) -> (u64, Vec<u8>) {
        let difficulty: U256 = Self::get_difficulty(netuid);
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
