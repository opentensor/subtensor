// The MIT License (MIT)
// Copyright © 2023 Yuma Rao

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
// documentation files (the “Software”), to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
// and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or substantial portions of
// the Software.

// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use super::*;
use crate::math::*;
use frame_support::dispatch::Pays;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use sp_std::vec;
use substrate_fixed::{
    transcendental::log2,
    types::I96F32,
};

impl<T: Config> Pallet<T> {
    /// Retrieves a boolean true is subnet emissions are determined by
    /// subnet specific staking.
    ///
    /// # Returns:
    /// * 'bool': Whether subnet emissions are determined by subnet specific staking.
    ///
    pub fn subnet_staking_on() -> bool {
        SubnetStakingOn::<T>::get()
    }
    pub fn set_subnet_staking(subnet_staking: bool) {
        SubnetStakingOn::<T>::put(subnet_staking);
    }

    /// Retrieves the unique identifier (UID) for the root network.
    ///
    /// The root network is a special case and has a fixed UID of 0.
    ///
    /// # Returns:
    /// * 'u16': The UID for the root network.
    ///
    pub fn get_root_netuid() -> u16 {
        0
    }

    /// Fetches the total count of subnets.
    ///
    /// This function retrieves the total number of subnets present on the chain.
    ///
    /// # Returns:
    /// * 'u16': The total number of subnets.
    ///
    pub fn get_num_subnets() -> u16 {
        TotalNetworks::<T>::get()
    }

    /// Fetches the max number of subnet
    ///
    /// This function retrieves the max number of subnet.
    ///
    /// # Returns:
    /// * 'u16': The max number of subnet
    ///
    pub fn get_max_subnets() -> u16 {
        SubnetLimit::<T>::get()
    }

    /// Sets the max number of subnet
    ///
    /// This function sets the max number of subnet.
    ///
    pub fn set_max_subnets(limit: u16) {
        SubnetLimit::<T>::put(limit);
        Self::deposit_event(Event::SubnetLimitSet(limit));
    }

    /// Fetches the total count of root network validators
    ///
    /// This function retrieves the total number of root network validators.
    ///
    /// # Returns:
    /// * 'u16': The total number of root network validators
    ///
    pub fn get_num_root_validators() -> u16 {
        Self::get_subnetwork_n(Self::get_root_netuid())
    }

    /// Fetches the max validators count of root network.
    ///
    /// This function retrieves the max validators count of root network.
    ///
    /// # Returns:
    /// * 'u16': The max validators count of root network.
    ///
    pub fn get_max_root_validators() -> u16 {
        Self::get_max_allowed_uids(Self::get_root_netuid())
    }

    /// Returns true if the subnetwork exists.
    ///
    /// This function checks if a subnetwork with the given UID exists.
    ///
    /// # Returns:
    /// * 'bool': Whether the subnet exists.
    ///
    pub fn if_subnet_exist(netuid: u16) -> bool {
        NetworksAdded::<T>::get(netuid)
    }

    /// Returns a list of subnet netuid equal to total networks.
    ///
    ///
    /// This iterates through all the networks and returns a list of netuids.
    ///
    /// # Returns:
    /// * 'Vec<u16>': Netuids of all subnets.
    ///
    pub fn get_all_subnet_netuids() -> Vec<u16> {
        NetworksAdded::<T>::iter()
            .map(|(netuid, _)| netuid)
            .collect()
    }

    /// Calculates the block emission based on the total issuance.
    ///
    /// This function computes the block emission by applying a logarithmic function
    /// to the total issuance of the network. The formula used takes into account
    /// the current total issuance and adjusts the emission rate accordingly to ensure
    /// a smooth issuance curve. The emission rate decreases as the total issuance increases,
    /// following a logarithmic decay.
    ///
    /// # Returns
    /// * 'Result<u64, &'static str>': The calculated block emission rate or error.
    ///
    pub fn get_block_emission() -> Result<u64, &'static str> {
        // Convert the total issuance to a fixed-point number for calculation.
        Self::get_block_emission_for_issuance(Self::get_total_issuance())
    }

    /// Returns the block emission for an issuance value.
    pub fn get_block_emission_for_issuance(issuance: u64) -> Result<u64, &'static str> {
        // Convert issuance to a float for calculations below.
        let total_issuance: I96F32 = I96F32::from_num(issuance);
        // Check to prevent division by zero when the total supply is reached
        // and creating an issuance greater than the total supply.
        if total_issuance >= I96F32::from_num(TotalSupply::<T>::get()) {
            return Ok(0);
        }
        // Calculate the logarithmic residual of the issuance against half the total supply.
        let residual: I96F32 = log2(
            I96F32::from_num(1.0)
                / (I96F32::from_num(1.0)
                    - total_issuance
                        / (I96F32::from_num(2.0) * I96F32::from_num(10_500_000_000_000_000.0))),
        )
        .map_err(|_| "Logarithm calculation failed")?;
        // Floor the residual to smooth out the emission rate.
        let floored_residual: I96F32 = residual.floor();
        // Calculate the final emission rate using the floored residual.
        // Convert floored_residual to an integer
        let floored_residual_int: u64 = floored_residual.to_num::<u64>();
        // Multiply 2.0 by itself floored_residual times to calculate the power of 2.
        let mut multiplier: I96F32 = I96F32::from_num(1.0);
        for _ in 0..floored_residual_int {
            multiplier *= I96F32::from_num(2.0);
        }
        let block_emission_percentage: I96F32 = I96F32::from_num(1.0) / multiplier;
        // Calculate the actual emission based on the emission rate
        let block_emission: I96F32 =
            block_emission_percentage * I96F32::from_num(DefaultBlockEmission::<T>::get());
        // Convert to u64
        let block_emission_u64: u64 = block_emission.to_num::<u64>();
        if BlockEmission::<T>::get() != block_emission_u64 {
            BlockEmission::<T>::put(block_emission_u64);
        }
        Ok(block_emission_u64)
    }

    /// Checks for any UIDs in the given list that are either equal to the root netuid or exceed the total number of subnets.
    ///
    /// It's important to check for invalid UIDs to ensure data integrity and avoid referencing nonexistent subnets.
    ///
    /// # Arguments:
    /// * 'uids': A reference to a vector of UIDs to check.
    ///
    /// # Returns:
    /// * 'bool': 'true' if any of the UIDs are invalid, 'false' otherwise.
    ///
    pub fn contains_invalid_root_uids(netuids: &[u16]) -> bool {
        for netuid in netuids {
            if !Self::if_subnet_exist(*netuid) {
                log::debug!(
                    "contains_invalid_root_uids: netuid {:?} does not exist",
                    netuid
                );
                return true;
            }
        }
        false
    }

    /// Returns the network rate limit
    ///
    pub fn get_network_rate_limit() -> u64 {
        NetworkRateLimit::<T>::get()
    }

    /// Sets the network rate limit and emit the `NetworkRateLimitSet` event
    ///
    pub fn set_network_rate_limit(limit: u64) {
        NetworkRateLimit::<T>::set(limit);
        Self::deposit_event(Event::NetworkRateLimitSet(limit));
    }

    /// Registers a user's hotkey to the root network.
    ///
    /// This function is responsible for registering the hotkey of a user.
    /// The root key with the least stake if pruned in the event of a filled network.
    ///
    /// # Arguments:
    /// * 'origin': Represents the origin of the call.
    /// * 'hotkey': The hotkey that the user wants to register to the root network.
    ///
    /// # Returns:
    /// * 'DispatchResult': A result type indicating success or failure of the registration.
    ///
    pub fn do_root_register(origin: T::RuntimeOrigin, hotkey: T::AccountId) -> DispatchResult {
        // --- 0. Get the unique identifier (UID) for the root network.
        let root_netuid: u16 = Self::get_root_netuid();
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::if_subnet_exist(root_netuid),
            Error::<T>::RootNetworkDoesNotExist
        );

        // --- 1. Ensure that the call originates from a signed source and retrieve the caller's account ID (coldkey).
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_root_register( coldkey: {:?}, hotkey: {:?} )",
            coldkey,
            hotkey
        );

        // --- 2. Ensure that the number of registrations in this block doesn't exceed the allowed limit.
        ensure!(
            Self::get_registrations_this_block(root_netuid)
                < Self::get_max_registrations_per_block(root_netuid),
            Error::<T>::TooManyRegistrationsThisBlock
        );

        // --- 3. Ensure that the number of registrations in this interval doesn't exceed thrice the target limit.
        ensure!(
            Self::get_registrations_this_interval(root_netuid)
                < Self::get_target_registrations_per_interval(root_netuid) * 3,
            Error::<T>::TooManyRegistrationsThisInterval
        );

        // --- 4. Check if the hotkey is already registered. If so, error out.
        ensure!(
            !Uids::<T>::contains_key(root_netuid, &hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // --- 6. Create a network account for the user if it doesn't exist.
        Self::create_account_if_non_existent(&coldkey, &hotkey);

        // --- 7. Fetch the current size of the subnetwork.
        let current_num_root_validators: u16 = Self::get_num_root_validators();

        // Declare a variable to hold the root UID.
        let subnetwork_uid: u16;

        // GDT of hotkey
        let hotkey_gdt = Self::get_hotkey_global_dynamic_tao(&hotkey);

        // --- 8. Check if the root net is below its allowed size.
        // max allowed is senate size.
        if current_num_root_validators < Self::get_max_root_validators() {
            // --- 12.1.1 We can append to the subnetwork as it's not full.
            subnetwork_uid = current_num_root_validators;

            // --- 12.1.2 Add the new account and make them a member of the Senate.
            Self::append_neuron(root_netuid, &hotkey, current_block_number);
            log::info!("add new neuron: {:?} on uid {:?}", hotkey, subnetwork_uid);
        } else {
            // --- 13.1.1 The network is full. Perform replacement.
            // Find the neuron with the lowest stake value to replace.
            // Iterate over all keys in the root network to find the neuron with the lowest stake.
            let (lowest_stake, lowest_uid) = Keys::<T>::iter_prefix(root_netuid).fold(
                (u64::MAX, 0),
                |(lowest_stake, lowest_uid), (uid_i, hotkey_i)| {
                    let stake_i: u64 = Self::get_hotkey_global_dynamic_tao(&hotkey_i);
                    if stake_i < lowest_stake {
                        (stake_i, uid_i)
                    } else {
                        (lowest_stake, lowest_uid)
                    }
                },
            );
            subnetwork_uid = lowest_uid;
            let replaced_hotkey: T::AccountId =
                Self::get_hotkey_for_net_and_uid(root_netuid, subnetwork_uid)?;

            // --- 13.1.2 The new account has a higher stake than the one being replaced.
            ensure!(lowest_stake < hotkey_gdt, Error::<T>::StakeTooLowForRoot);

            // --- 13.1.3 The new account has a higher stake than the one being replaced.
            // Replace the neuron account with new information.
            Self::replace_neuron(root_netuid, lowest_uid, &hotkey, current_block_number);

            log::info!(
                "replace neuron: {:?} with {:?} on uid {:?}",
                replaced_hotkey,
                hotkey,
                subnetwork_uid
            );
        }

        let current_stake = hotkey_gdt;
        // If we're full, we'll swap out the lowest stake member.
        let members = T::SenateMembers::members();
        if (members.len() as u32) == T::SenateMembers::max_members() {
            let mut sorted_members = members.clone();
            sorted_members.sort_by(|a, b| {
                let a_stake = Self::get_hotkey_global_dynamic_tao(a);
                let b_stake = Self::get_hotkey_global_dynamic_tao(b);

                b_stake.cmp(&a_stake)
            });

            if let Some(last) = sorted_members.last() {
                let last_stake = Self::get_hotkey_global_dynamic_tao(last);

                if last_stake < current_stake {
                    T::SenateMembers::swap_member(last, &hotkey).map_err(|e| e.error)?;
                    T::TriumvirateInterface::remove_votes(&last)?;
                }
            }
        } else {
            T::SenateMembers::add_member(&hotkey).map_err(|e| e.error)?;
        }

        // --- 13. Force all members on root to become a delegate.
        if !Self::hotkey_is_delegate(&hotkey) {
            Self::delegate_hotkey(&hotkey, 11_796); // 18% cut defaulted.
        }

        // --- 14. Update the registration counters for both the block and interval.
        RegistrationsThisInterval::<T>::mutate(root_netuid, |val| *val += 1);
        RegistrationsThisBlock::<T>::mutate(root_netuid, |val| *val += 1);

        // --- 15. Log and announce the successful registration.
        log::info!(
            "RootRegistered(netuid:{:?} uid:{:?} hotkey:{:?})",
            root_netuid,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(root_netuid, subnetwork_uid, hotkey));

        // --- 16. Finish and return success.
        Ok(())
    }

    pub fn do_set_root_weights(
        origin: T::RuntimeOrigin,
        netuid: u16,
        hotkey: T::AccountId,
        uids: Vec<u16>,
        values: Vec<u16>,
        version_key: u64,
    ) -> dispatch::DispatchResult {
        // Check the caller's signature. This is the coldkey of a registered account.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_set_root_weights( origin:{:?} netuid:{:?}, uids:{:?}, values:{:?})",
            coldkey,
            netuid,
            uids,
            values
        );

        // Check the hotkey account exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Check that the signer coldkey owns the hotkey
        ensure!(
            Self::get_owning_coldkey_for_hotkey(&hotkey) == coldkey,
            Error::<T>::NonAssociatedColdKey
        );

        // Check to see if this is a valid network.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // Check that this is the root network.
        ensure!(netuid == Self::get_root_netuid(), Error::<T>::NotRootSubnet);

        // Check that the length of uid list and value list are equal for this network.
        ensure!(
            Self::uids_match_values(&uids, &values),
            Error::<T>::WeightVecNotEqualSize
        );

        // Check to see if the number of uids is within the max allowed uids for this network.
        // For the root network this number is the number of subnets.
        ensure!(
            !Self::contains_invalid_root_uids(&uids),
            Error::<T>::UidVecContainInvalidOne
        );

        // Check to see if the hotkey is registered to the passed network.
        ensure!(
            Self::is_hotkey_registered_on_network(netuid, &hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // Check to see if the hotkey has enough stake to set weights.
        ensure!(
            Self::get_hotkey_global_dynamic_tao(&hotkey) >= Self::get_weights_min_stake(),
            Error::<T>::NotEnoughStakeToSetWeights
        );

        // Ensure version_key is up-to-date.
        ensure!(
            Self::check_version_key(netuid, version_key),
            Error::<T>::IncorrectWeightVersionKey
        );

        // Get the neuron uid of associated hotkey on network netuid.
        let neuron_uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey)?;

        // Ensure the uid is not setting weights faster than the weights_set_rate_limit.
        let current_block: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::check_rate_limit(netuid, neuron_uid, current_block),
            Error::<T>::SettingWeightsTooFast
        );

        // Ensure the passed uids contain no duplicates.
        ensure!(!Self::has_duplicate_uids(&uids), Error::<T>::DuplicateUids);

        // Ensure that the weights have the required length.
        ensure!(
            Self::check_length(netuid, neuron_uid, &uids, &values),
            Error::<T>::WeightVecLengthIsLow
        );

        // Max-upscale the weights.
        let max_upscaled_weights: Vec<u16> = vec_u16_max_upscale_to_u16(&values);

        // Ensure the weights are max weight limited
        ensure!(
            Self::max_weight_limited(netuid, neuron_uid, &uids, &max_upscaled_weights),
            Error::<T>::MaxWeightExceeded
        );

        // Zip weights for sinking to storage map.
        let mut zipped_weights: Vec<(u16, u16)> = vec![];
        for (uid, val) in uids.iter().zip(max_upscaled_weights.iter()) {
            zipped_weights.push((*uid, *val))
        }

        // Set weights under netuid, uid double map entry.
        Weights::<T>::insert(netuid, neuron_uid, zipped_weights);

        // Set the activity for the weights on this network.
        Self::set_last_update_for_uid(netuid, neuron_uid, current_block);

        // Emit the tracking event.
        log::info!(
            "RootWeightsSet( netuid:{:?}, neuron_uid:{:?} )",
            netuid,
            neuron_uid
        );
        Self::deposit_event(Event::WeightsSet(netuid, neuron_uid));

        // Return ok.
        Ok(())
    }

    pub fn do_vote_root(
        origin: T::RuntimeOrigin,
        hotkey: &T::AccountId,
        proposal: T::Hash,
        index: u32,
        approve: bool,
    ) -> DispatchResultWithPostInfo {
        // --- 1. Ensure that the caller has signed with their coldkey.
        let coldkey = ensure_signed(origin.clone())?;

        // --- 2. Ensure that the calling coldkey owns the associated hotkey.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 3. Ensure that the calling hotkey is a member of the senate.
        ensure!(
            T::SenateMembers::is_member(hotkey),
            Error::<T>::NotSenateMember
        );

        // --- 4. Detects first vote of the member in the motion
        let is_account_voting_first_time =
            T::TriumvirateInterface::add_vote(hotkey, proposal, index, approve)?;

        // --- 5. Calculate extrinsic weight
        let members = T::SenateMembers::members();
        let member_count = members.len() as u32;
        let vote_weight = Weight::from_parts(20_528_275, 4980)
            .saturating_add(Weight::from_parts(48_856, 0).saturating_mul(member_count.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(Weight::from_parts(0, 128).saturating_mul(member_count.into()));

        Ok((
            Some(vote_weight),
            if is_account_voting_first_time {
                Pays::No
            } else {
                Pays::Yes
            },
        )
            .into())
    }

    /// Facilitates user registration of a new subnetwork.
    ///
    /// # Args:
    /// * 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    ///
    /// # Event:
    /// * 'NetworkAdded': Emitted when a new network is successfully added.
    ///
    /// # Raises:
    /// * 'TxRateLimitExceeded': If the rate limit for network registration is exceeded.
    /// * 'NotEnoughBalanceToStake': If there isn't enough balance to stake for network registration.
    /// * 'BalanceWithdrawalError': If an error occurs during balance withdrawal for network registration.
    ///
    pub fn user_add_network(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
    ) -> dispatch::DispatchResult {
        // --- 0. Ensure the caller is a signed user.
        let coldkey = ensure_signed(origin)?;

        // --- 1. Ensure that the hotkey is not owned by another key.
        if Owner::<T>::contains_key(&hotkey) {
            ensure!(
                Self::coldkey_owns_hotkey(&coldkey, &hotkey),
                Error::<T>::NonAssociatedColdKey
            );
        }

        // --- 2. Check rate limit for network registrations.
        let current_block = Self::get_current_block_as_u64();
        let last_lock_block = Self::get_network_last_lock_block();
        ensure!(
            current_block.saturating_sub(last_lock_block) >= NetworkRateLimit::<T>::get(),
            Error::<T>::NetworkTxRateLimitExceeded
        );

        // --- 3. Calculate and lock the required tokens to register a network.
        let lock_amount: u64 = Self::get_network_lock_cost();
        log::debug!("network lock_amount: {:?}", lock_amount);
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, lock_amount),
            Error::<T>::NotEnoughBalanceToStake
        );

        // --- 5. Determine the netuid to register by iterating through netuids to find next lowest netuid.
        let netuid_to_register: u16 = {
            let mut next_available_netuid = 0;
            loop {
                next_available_netuid += 1;
                if !Self::if_subnet_exist(next_available_netuid) {
                    break next_available_netuid;
                }
            }
        };

        // --- 5. Perform the lock operation.
        let actual_lock_amount = Self::remove_balance_from_coldkey_account(&coldkey, lock_amount)?;
        Self::set_subnet_locked_balance(netuid_to_register, actual_lock_amount);
        Self::set_network_last_lock(actual_lock_amount);

        // --- 6. Create a new network and set initial and custom parameters for the network.
        Self::init_new_network(netuid_to_register, 360);
        let current_block_number: u64 = Self::get_current_block_as_u64();
        NetworkLastRegistered::<T>::set(current_block_number);
        NetworkRegisteredAt::<T>::insert(netuid_to_register, current_block_number);
        log::debug!("init_new_network: {:?}", netuid_to_register,);

        // --- 7. Set Subnet owner to the coldkey.
        SubnetOwner::<T>::insert(netuid_to_register, coldkey.clone()); // Set the owner (which can change.)
        SubnetCreator::<T>::insert(netuid_to_register, hotkey.clone()); // Set the creator hotkey (which is forever.)

        // --- 8. Instantiate initial token supply based on lock cost.
        let initial_tao_reserve: u64 = lock_amount as u64;
        let initial_dynamic_reserve: u64 = lock_amount * Self::get_num_subnets() as u64;
        let initial_dynamic_outstanding: u64 = lock_amount * Self::get_num_subnets() as u64;
        let initial_dynamic_k: u128 =
            (initial_tao_reserve as u128) * (initial_dynamic_reserve as u128);

        DynamicTAOReserve::<T>::insert(netuid_to_register, initial_tao_reserve);
        DynamicAlphaReserve::<T>::insert(netuid_to_register, initial_dynamic_reserve);
        DynamicAlphaOutstanding::<T>::insert(netuid_to_register, initial_dynamic_outstanding);
        DynamicK::<T>::insert(netuid_to_register, initial_dynamic_k);
        IsDynamic::<T>::insert(netuid_to_register, true); // Turn on dynamic staking.

        // --- 9. Register the owner to the network and expand size.
        Self::create_account_if_non_existent(&coldkey, &hotkey);
        Self::append_neuron(netuid_to_register, &hotkey, current_block_number);

        // --- 10. Distribute initial supply of tokens to the owners.
        Self::increase_subnet_token_on_coldkey_hotkey_account(
            &coldkey,
            &hotkey,
            netuid_to_register,
            initial_dynamic_outstanding,
        );

        // --- 8. Emit the NetworkAdded event.
        log::info!(
            "NetworkAdded( netuid:{:?}, modality:{:?} )",
            netuid_to_register,
            0
        );
        Self::deposit_event(Event::NetworkAdded(netuid_to_register, 0));

        // --- 9. Return success.
        Ok(())
    }

    /// Facilitates the removal of a user's subnetwork.
    ///
    /// # Args:
    /// * 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    /// * 'netuid': ('u16'): The unique identifier of the network to be removed.
    ///
    /// # Event:
    /// * 'NetworkRemoved': Emitted when a network is successfully removed.
    ///
    /// # Raises:
    /// * 'SubNetworkDoesNotExist': If the specified network does not exist.
    /// * 'NotSubnetOwner': If the caller does not own the specified subnet.
    ///
    pub fn user_remove_network(origin: T::RuntimeOrigin, netuid: u16) -> dispatch::DispatchResult {
        // --- 1. Ensure the function caller is a signed user.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Ensure this subnet exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Ensure the caller owns this subnet.
        ensure!(
            SubnetOwner::<T>::get(netuid) == coldkey,
            Error::<T>::NotSubnetOwner
        );

        // --- 4. Explicitly erase the network and all its parameters.
        Self::remove_network(netuid);

        // --- 5. Emit the NetworkRemoved event.
        log::info!("NetworkRemoved( netuid:{:?} )", netuid);
        Self::deposit_event(Event::NetworkRemoved(netuid));

        // --- 6. Return success.
        Ok(())
    }

    /// Sets initial and custom parameters for a new network.
    pub fn init_new_network(netuid: u16, tempo: u16) {
        // --- 1. Set network to 0 size.
        SubnetworkN::<T>::insert(netuid, 0);

        // --- 2. Set this network uid to alive.
        NetworksAdded::<T>::insert(netuid, true);

        // --- 3. Fill tempo memory item.
        Tempo::<T>::insert(netuid, tempo);

        // --- 4 Fill modality item.
        NetworkModality::<T>::insert(netuid, 0);

        // --- 5. Increase total network count.
        TotalNetworks::<T>::mutate(|n| *n += 1);

        // --- 6. Set all default values **explicitly**.
        Self::set_network_registration_allowed(netuid, true);
        Self::set_max_allowed_uids(netuid, 256);
        Self::set_max_allowed_validators(netuid, 64);
        Self::set_min_allowed_weights(netuid, 1);
        Self::set_max_weight_limit(netuid, u16::MAX);
        Self::set_adjustment_interval(netuid, 360);
        Self::set_target_registrations_per_interval(netuid, 1);
        Self::set_adjustment_alpha(netuid, 17_893_341_751_498_265_066); // 18_446_744_073_709_551_615 * 0.97 = 17_893_341_751_498_265_066
        Self::set_immunity_period(netuid, 5000);
        Self::set_min_burn(netuid, 1);
        Self::set_min_difficulty(netuid, u64::MAX);
        Self::set_max_difficulty(netuid, u64::MAX);

        // Make network parameters explicit.
        if !Tempo::<T>::contains_key(netuid) {
            Tempo::<T>::insert(netuid, Tempo::<T>::get(netuid));
        }
        if !Kappa::<T>::contains_key(netuid) {
            Kappa::<T>::insert(netuid, Kappa::<T>::get(netuid));
        }
        if !Difficulty::<T>::contains_key(netuid) {
            Difficulty::<T>::insert(netuid, Difficulty::<T>::get(netuid));
        }
        if !MaxAllowedUids::<T>::contains_key(netuid) {
            MaxAllowedUids::<T>::insert(netuid, MaxAllowedUids::<T>::get(netuid));
        }
        if !ImmunityPeriod::<T>::contains_key(netuid) {
            ImmunityPeriod::<T>::insert(netuid, ImmunityPeriod::<T>::get(netuid));
        }
        if !ActivityCutoff::<T>::contains_key(netuid) {
            ActivityCutoff::<T>::insert(netuid, ActivityCutoff::<T>::get(netuid));
        }
        if !EmissionValues::<T>::contains_key(netuid) {
            EmissionValues::<T>::insert(netuid, DefaultEmissionValues::<T>::get());
        }
        if !MaxWeightsLimit::<T>::contains_key(netuid) {
            MaxWeightsLimit::<T>::insert(netuid, MaxWeightsLimit::<T>::get(netuid));
        }
        if !MinAllowedWeights::<T>::contains_key(netuid) {
            MinAllowedWeights::<T>::insert(netuid, MinAllowedWeights::<T>::get(netuid));
        }
        if !RegistrationsThisInterval::<T>::contains_key(netuid) {
            RegistrationsThisInterval::<T>::insert(
                netuid,
                RegistrationsThisInterval::<T>::get(netuid),
            );
        }
        if !POWRegistrationsThisInterval::<T>::contains_key(netuid) {
            POWRegistrationsThisInterval::<T>::insert(
                netuid,
                POWRegistrationsThisInterval::<T>::get(netuid),
            );
        }
        if !BurnRegistrationsThisInterval::<T>::contains_key(netuid) {
            BurnRegistrationsThisInterval::<T>::insert(
                netuid,
                BurnRegistrationsThisInterval::<T>::get(netuid),
            );
        }
    }

    /// Removes a network (identified by netuid) and all associated parameters.
    ///
    /// This function is responsible for cleaning up all the data associated with a network.
    /// It ensures that all the storage values related to the network are removed, and any
    /// reserved balance is returned to the network owner.
    ///
    /// # Args:
    ///  * 'netuid': ('u16'): The unique identifier of the network to be removed.
    ///
    /// # Note:
    /// This function does not emit any events, nor does it raise any errors. It silently
    /// returns if any internal checks fail.
    ///
    pub fn remove_network(netuid: u16) {
        // --- 1. Return balance to subnet owner.
        let owner_coldkey = SubnetOwner::<T>::get(netuid);
        let reserved_amount = Self::get_subnet_locked_balance(netuid);

        // --- 2. Remove network count.
        SubnetworkN::<T>::remove(netuid);

        // --- 3. Remove network modality storage.
        NetworkModality::<T>::remove(netuid);

        // --- 4. Remove netuid from added networks.
        NetworksAdded::<T>::remove(netuid);

        // --- 6. Decrement the network counter.
        TotalNetworks::<T>::mutate(|n| *n -= 1);

        // --- 7. Remove various network-related storages.
        NetworkRegisteredAt::<T>::remove(netuid);

        // --- 8. Remove incentive mechanism memory.
        let _ = Uids::<T>::clear_prefix(netuid, u32::max_value(), None);
        let _ = Keys::<T>::clear_prefix(netuid, u32::max_value(), None);
        let _ = Bonds::<T>::clear_prefix(netuid, u32::max_value(), None);

        // --- 8. Removes the weights for this subnet (do not remove).
        let _ = Weights::<T>::clear_prefix(netuid, u32::max_value(), None);

        // --- 9. Iterate over stored weights and fill the matrix.
        for (uid_i, weights_i) in
            Weights::<T>::iter_prefix(
                Self::get_root_netuid(),
            )
        {
            // Create a new vector to hold modified weights.
            let mut modified_weights = weights_i.clone();
            // Iterate over each weight entry to potentially update it.
            for (subnet_id, weight) in modified_weights.iter_mut() {
                if subnet_id == &netuid {
                    // If the condition matches, modify the weight
                    *weight = 0; // Set weight to 0 for the matching subnet_id.
                }
            }
            Weights::<T>::insert(Self::get_root_netuid(), uid_i, modified_weights);
        }

        // --- 10. Remove various network-related parameters.
        Rank::<T>::remove(netuid);
        Trust::<T>::remove(netuid);
        Active::<T>::remove(netuid);
        Emission::<T>::remove(netuid);
        Incentive::<T>::remove(netuid);
        Consensus::<T>::remove(netuid);
        Dividends::<T>::remove(netuid);
        PruningScores::<T>::remove(netuid);
        LastUpdate::<T>::remove(netuid);
        ValidatorPermit::<T>::remove(netuid);
        ValidatorTrust::<T>::remove(netuid);

        // --- 11. Erase network parameters.
        Tempo::<T>::remove(netuid);
        Kappa::<T>::remove(netuid);
        Difficulty::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        ActivityCutoff::<T>::remove(netuid);
        EmissionValues::<T>::remove(netuid);
        MaxWeightsLimit::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        RegistrationsThisInterval::<T>::remove(netuid);
        POWRegistrationsThisInterval::<T>::remove(netuid);
        BurnRegistrationsThisInterval::<T>::remove(netuid);

        // --- 12. Add the balance back to the owner.
        Self::add_balance_to_coldkey_account(&owner_coldkey, reserved_amount);
        Self::set_subnet_locked_balance(netuid, 0);
        SubnetOwner::<T>::remove(netuid);
    }

    /// This function calculates the lock cost for a network based on the last lock amount, minimum lock cost, last lock block, and current block.
    /// The lock cost is calculated using the formula:
    /// lock_cost = (last_lock * mult) - (last_lock / lock_reduction_interval) * (current_block - last_lock_block)
    /// where:
    /// - last_lock is the last lock amount for the network
    /// - mult is the multiplier which increases lock cost each time a registration occurs
    /// - last_lock_block is the block number at which the last lock occurred
    /// - lock_reduction_interval the number of blocks before the lock returns to previous value.
    /// - current_block is the current block number
    /// - DAYS is the number of blocks in a day
    /// - min_lock is the minimum lock cost for the network
    ///
    /// If the calculated lock cost is less than the minimum lock cost, the minimum lock cost is returned.
    ///
    /// # Returns:
    ///  * 'u64':
    ///     - The lock cost for the network.
    ///
    pub fn get_network_lock_cost() -> u64 {
        let last_lock = Self::get_network_last_lock();
        let min_lock = Self::get_network_min_lock();
        let last_lock_block = Self::get_network_last_lock_block();
        let current_block = Self::get_current_block_as_u64();
        let lock_reduction_interval = Self::get_lock_reduction_interval();
        let mult = if last_lock_block == 0 { 1 } else { 2 };

        let mut lock_cost = last_lock.saturating_mul(mult).saturating_sub(
            last_lock
                .saturating_div(lock_reduction_interval)
                .saturating_mul(current_block.saturating_sub(last_lock_block)),
        );

        if lock_cost < min_lock {
            lock_cost = min_lock;
        }

        log::debug!( "last_lock: {:?}, min_lock: {:?}, last_lock_block: {:?}, lock_reduction_interval: {:?}, current_block: {:?}, mult: {:?} lock_cost: {:?}",
        last_lock, min_lock, last_lock_block, lock_reduction_interval, current_block, mult, lock_cost);

        lock_cost
    }

    pub fn get_network_registered_block(netuid: u16) -> u64 {
        NetworkRegisteredAt::<T>::get(netuid)
    }
    pub fn get_network_immunity_period() -> u64 {
        NetworkImmunityPeriod::<T>::get()
    }
    pub fn set_network_immunity_period(net_immunity_period: u64) {
        NetworkImmunityPeriod::<T>::set(net_immunity_period);
        Self::deposit_event(Event::NetworkImmunityPeriodSet(net_immunity_period));
    }
    pub fn set_network_min_lock(net_min_lock: u64) {
        NetworkMinLockCost::<T>::set(net_min_lock);
        Self::deposit_event(Event::NetworkMinLockCostSet(net_min_lock));
    }
    pub fn get_network_min_lock() -> u64 {
        NetworkMinLockCost::<T>::get()
    }
    pub fn set_network_last_lock(net_last_lock: u64) {
        NetworkLastLockCost::<T>::set(net_last_lock);
    }
    pub fn get_network_last_lock() -> u64 {
        NetworkLastLockCost::<T>::get()
    }
    pub fn get_network_last_lock_block() -> u64 {
        NetworkLastRegistered::<T>::get()
    }
    pub fn set_lock_reduction_interval(interval: u64) {
        NetworkLockReductionInterval::<T>::set(interval);
        Self::deposit_event(Event::NetworkLockCostReductionIntervalSet(interval));
    }
    pub fn get_lock_reduction_interval() -> u64 {
        NetworkLockReductionInterval::<T>::get()
    }
}
