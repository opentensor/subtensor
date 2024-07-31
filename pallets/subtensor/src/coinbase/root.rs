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
// use crate::epoch::math::*;
use frame_support::dispatch::Pays;
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use sp_std::vec;
use substrate_fixed::types::I64F64;

impl<T: Config> Pallet<T> {
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

    /// Sets the emission values for each netuid
    ///
    pub fn set_emission_values(netuids: &[u16], emission: Vec<u64>) -> Result<(), &'static str> {
        log::debug!(
            "set_emission_values: netuids: {:?} emission:{:?}",
            netuids,
            emission
        );

        // Be careful this function can fail.
        if Self::contains_invalid_root_uids(netuids) {
            log::error!("set_emission_values: contains_invalid_root_uids");
            return Err("Invalid netuids");
        }
        if netuids.len() != emission.len() {
            log::error!("set_emission_values: netuids.len() != emission.len()");
            return Err("netuids and emission must have the same length");
        }
        for (netuid_i, emission_i) in netuids.iter().zip(emission) {
            log::debug!("set netuid:{:?} emission:{:?}", netuid_i, emission_i);
            EmissionValues::<T>::insert(*netuid_i, emission_i);
        }
        Ok(())
    }

    /// Retrieves weight matrix associated with the root network.
    ///  Weights represent the preferences for each subnetwork.
    ///
    /// # Returns:
    /// A 2D vector ('Vec<Vec<I32F32>>') where each entry [i][j] represents the weight of subnetwork
    /// 'j' with according to the preferences of key. Validator 'i' within the root network.
    ///
    pub fn get_root_weights() -> Vec<Vec<I64F64>> {
        // --- 0. The number of validators on the root network.
        let n: usize = Self::get_num_root_validators() as usize;

        // --- 1 The number of subnets to validate.
        log::debug!("subnet size before cast: {:?}", Self::get_num_subnets());
        let k: usize = Self::get_num_subnets() as usize;
        log::debug!("n: {:?} k: {:?}", n, k);

        // --- 2. Initialize a 2D vector with zeros to store the weights. The dimensions are determined
        // by `n` (number of validators) and `k` (total number of subnets).
        let mut weights: Vec<Vec<I64F64>> = vec![vec![I64F64::from_num(0.0); k]; n];
        log::debug!("weights:\n{:?}\n", weights);

        let subnet_list = Self::get_all_subnet_netuids();

        // --- 3. Iterate over stored weights and fill the matrix.
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(
                Self::get_root_netuid(),
            )
        {
            // --- 4. Iterate over each weight entry in `weights_i` to update the corresponding value in the
            // initialized `weights` 2D vector. Here, `uid_j` represents a subnet, and `weight_ij` is the
            // weight of `uid_i` with respect to `uid_j`.
            for (netuid, weight_ij) in &weights_i {
                let idx = uid_i as usize;
                if let Some(weight) = weights.get_mut(idx) {
                    if let Some((w, _)) = weight
                        .iter_mut()
                        .zip(&subnet_list)
                        .find(|(_, subnet)| *subnet == netuid)
                    {
                        *w = I64F64::from_num(*weight_ij);
                    }
                }
            }
        }

        // --- 5. Return the filled weights matrix.
        weights
    }

    /// Sets the network rate limit and emit the `NetworkRateLimitSet` event
    ///
    pub fn set_network_rate_limit(limit: u64) {
        NetworkRateLimit::<T>::set(limit);
        Self::deposit_event(Event::NetworkRateLimitSet(limit));
    }

    /// Checks if registrations are allowed for a given subnet.
    ///
    /// This function retrieves the subnet hyperparameters for the specified subnet and checks the `registration_allowed` flag.
    /// If the subnet doesn't exist or doesn't have hyperparameters defined, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if registrations are allowed for the subnet, `false` otherwise.
    pub fn is_registration_allowed(netuid: u16) -> bool {
        Self::get_subnet_hyperparams(netuid)
            .map(|params| params.registration_allowed)
            .unwrap_or(false)
    }

    /// Computes and sets emission values for the root network which determine the emission for all subnets.
    ///
    /// This function is responsible for calculating emission based on network weights, stake values,
    /// and registered hotkeys.
    ///
    /// DEPRECATED.
    // pub fn root_epoch(block_number: u64) -> Result<(), &'static str> {
    //     // --- 0. The unique ID associated with the root network.
    //     let root_netuid: u16 = Self::get_root_netuid();

    //     // --- 1. Check if we should update the emission values based on blocks since emission was last set.
    //     let blocks_until_next_epoch: u64 =
    //         Self::blocks_until_next_epoch(root_netuid, Self::get_tempo(root_netuid), block_number);
    //     if blocks_until_next_epoch != 0 {
    //         // Not the block to update emission values.
    //         log::debug!("blocks_until_next_epoch: {:?}", blocks_until_next_epoch);
    //         return Err("");
    //     }

    //     // --- 2. Retrieves the number of root validators on subnets.
    //     let n: u16 = Self::get_num_root_validators();
    //     log::debug!("n:\n{:?}\n", n);
    //     if n == 0 {
    //         // No validators.
    //         return Err("No validators to validate emission values.");
    //     }

    //     // --- 3. Obtains the number of registered subnets.
    //     let k: u16 = Self::get_all_subnet_netuids().len() as u16;
    //     log::debug!("k:\n{:?}\n", k);
    //     if k == 0 {
    //         // No networks to validate.
    //         return Err("No networks to validate emission values.");
    //     }

    //     // --- 4. Determines the total block emission across all the subnetworks. This is the
    //     // value which will be distributed based on the computation below.
    //     let block_emission: I64F64 = I64F64::from_num(Self::get_block_emission()?);
    //     log::debug!("block_emission:\n{:?}\n", block_emission);

    //     // --- 5. A collection of all registered hotkeys on the root network. Hotkeys
    //     // pairs with network UIDs and stake values.
    //     let mut hotkeys: Vec<(u16, T::AccountId)> = vec![];
    //     for (uid_i, hotkey) in
    //         <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(root_netuid)
    //     {
    //         hotkeys.push((uid_i, hotkey));
    //     }
    //     log::debug!("hotkeys:\n{:?}\n", hotkeys);

    //     // --- 6. Retrieves and stores the stake value associated with each hotkey on the root network.
    //     // Stakes are stored in a 64-bit fixed point representation for precise calculations.
    //     let mut stake_i64: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
    //     for ((_, hotkey), stake) in hotkeys.iter().zip(&mut stake_i64) {
    //         *stake = I64F64::from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, netuid));
    //     }
    //     inplace_normalize_64(&mut stake_i64);
    //     log::debug!("S:\n{:?}\n", &stake_i64);

    //     // --- 7. Retrieves the network weights in a 2D Vector format. Weights have shape
    //     // n x k where is n is the number of registered peers and k is the number of subnets.
    //     let mut weights: Vec<Vec<I64F64>> = Self::get_root_weights();
    //     log::debug!("W:\n{:?}\n", &weights);

    //     // Normalize weights.
    //     inplace_row_normalize_64(&mut weights);
    //     log::debug!("W(norm):\n{:?}\n", &weights);

    //     // --- 8. Calculates the rank of networks. Rank is a product of weights and stakes.
    //     // Ranks will have shape k, a score for each subnet.
    //     let ranks: Vec<I64F64> = matmul_64(&weights, &stake_i64);
    //     log::debug!("R:\n{:?}\n", &ranks);

    //     // --- 9. Calculates the trust of networks. Trust is a sum of all stake with weights > 0.
    //     // Trust will have shape k, a score for each subnet.
    //     log::debug!("Subnets:\n{:?}\n", Self::get_all_subnet_netuids());
    //     log::debug!("N Subnets:\n{:?}\n", Self::get_num_subnets());

    //     let total_networks = Self::get_num_subnets();
    //     let mut trust = vec![I64F64::from_num(0); total_networks as usize];
    //     let mut total_stake: I64F64 = I64F64::from_num(0);
    //     for (weights, hotkey_stake) in weights.iter().zip(stake_i64) {
    //         total_stake = total_stake.saturating_add(hotkey_stake);
    //         for (weight, trust_score) in weights.iter().zip(&mut trust) {
    //             if *weight > 0 {
    //                 *trust_score = trust_score.saturating_add(hotkey_stake);
    //             }
    //         }
    //     }

    //     log::debug!("T_before normalization:\n{:?}\n", &trust);
    //     log::debug!("Total_stake:\n{:?}\n", &total_stake);

    //     if total_stake == 0 {
    //         return Err("No stake on network");
    //     }

    //     for trust_score in trust.iter_mut() {
    //         if let Some(quotient) = trust_score.checked_div(total_stake) {
    //             *trust_score = quotient;
    //         }
    //     }

    //     // --- 10. Calculates the consensus of networks. Consensus is a sigmoid normalization of the trust scores.
    //     // Consensus will have shape k, a score for each subnet.
    //     log::debug!("T:\n{:?}\n", &trust);
    //     let one = I64F64::from_num(1);
    //     let mut consensus = vec![I64F64::from_num(0); total_networks as usize];
    //     for (trust_score, consensus_i) in trust.iter_mut().zip(&mut consensus) {
    //         let shifted_trust =
    //             trust_score.saturating_sub(I64F64::from_num(Self::get_float_kappa(0))); // Range( -kappa, 1 - kappa )
    //         let temperatured_trust =
    //             shifted_trust.saturating_mul(I64F64::from_num(Self::get_rho(0))); // Range( -rho * kappa, rho ( 1 - kappa ) )
    //         let exponentiated_trust: I64F64 =
    //             substrate_fixed::transcendental::exp(temperatured_trust.saturating_neg())
    //                 .expect("temperatured_trust is on range( -rho * kappa, rho ( 1 - kappa ) )");

    //         *consensus_i = one.saturating_div(one.saturating_add(exponentiated_trust));
    //     }

    //     log::debug!("C:\n{:?}\n", &consensus);
    //     let mut weighted_emission = vec![I64F64::from_num(0); total_networks as usize];
    //     for ((emission, consensus_i), rank) in
    //         weighted_emission.iter_mut().zip(&consensus).zip(&ranks)
    //     {
    //         *emission = consensus_i.saturating_mul(*rank);
    //     }
    //     inplace_normalize_64(&mut weighted_emission);
    //     log::debug!("Ei64:\n{:?}\n", &weighted_emission);

    //     // -- 11. Converts the normalized 64-bit fixed point rank values to u64 for the final emission calculation.
    //     let emission_as_tao: Vec<I64F64> = weighted_emission
    //         .iter()
    //         .map(|v: &I64F64| v.saturating_mul(block_emission))
    //         .collect();

    //     // --- 12. Converts the normalized 64-bit fixed point rank values to u64 for the final emission calculation.
    //     let emission_u64: Vec<u64> = vec_fixed64_to_u64(emission_as_tao);
    //     log::debug!("Eu64:\n{:?}\n", &emission_u64);

    //     // --- 13. Set the emission values for each subnet directly.
    //     let netuids: Vec<u16> = Self::get_all_subnet_netuids();
    //     log::debug!("netuids: {:?} values: {:?}", netuids, emission_u64);

    //     Self::set_emission_values(&netuids, emission_u64)
    // }

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
                < Self::get_target_registrations_per_interval(root_netuid).saturating_mul(3),
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
            let mut lowest_stake: u64 = u64::MAX;
            let mut lowest_uid: u16 = 0;

            // Iterate over all keys in the root network to find the neuron with the lowest stake.
            for (uid_i, hotkey_i) in
                <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(
                    root_netuid,
                )
            {
                let stake_i: u64 = Self::get_global_for_hotkey(&hotkey_i);
                if stake_i < lowest_stake {
                    lowest_stake = stake_i;
                    lowest_uid = uid_i;
                }
            }
            subnetwork_uid = lowest_uid;
            let replaced_hotkey: T::AccountId =
                Self::get_hotkey_for_net_and_uid(root_netuid, subnetwork_uid)?;

            // --- 13.1.2 The new account has a higher stake than the one being replaced.
            ensure!(
                lowest_stake < Self::get_global_for_hotkey(&hotkey),
                Error::<T>::StakeTooLowForRoot
            );

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

        // --- 13. Join the Senate if eligible.
        // Returns the replaced member, if any.
        let _ = Self::join_senate_if_eligible(&hotkey)?;

        // --- 14. Force all members on root to become a delegate.
        if !Self::hotkey_is_delegate(&hotkey) {
            Self::delegate_hotkey(&hotkey, 11_796); // 18% cut defaulted.
        }

        // --- 15. Update the registration counters for both the block and interval.
        #[allow(clippy::arithmetic_side_effects)]
        // note this RA + clippy false positive is a known substrate issue
        RegistrationsThisInterval::<T>::mutate(root_netuid, |val| *val += 1);
        #[allow(clippy::arithmetic_side_effects)]
        // note this RA + clippy false positive is a known substrate issue
        RegistrationsThisBlock::<T>::mutate(root_netuid, |val| *val += 1);

        // --- 16. Log and announce the successful registration.
        log::info!(
            "RootRegistered(netuid:{:?} uid:{:?} hotkey:{:?})",
            root_netuid,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(root_netuid, subnetwork_uid, hotkey));

        // --- 17. Finish and return success.
        Ok(())
    }

    // Checks if a hotkey should be a member of the Senate, and if so, adds them.
    //
    // This function is responsible for adding a hotkey to the Senate if they meet the requirements.
    // The root key with the least stake is pruned in the event of a filled membership.
    //
    // # Arguments:
    // * 'origin': Represents the origin of the call.
    // * 'hotkey': The hotkey that the user wants to register to the root network.
    //
    // # Returns:
    // * 'DispatchResult': A result type indicating success or failure of the registration.
    //
    pub fn do_adjust_senate(origin: T::RuntimeOrigin, hotkey: T::AccountId) -> DispatchResult {
        // --- 0. Get the unique identifier (UID) for the root network.
        let root_netuid: u16 = Self::get_root_netuid();
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

        // --- 2. Check if the hotkey is already registered to the root network. If not, error out.
        ensure!(
            Uids::<T>::contains_key(root_netuid, &hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // --- 3. Create a network account for the user if it doesn't exist.
        Self::create_account_if_non_existent(&coldkey, &hotkey);

        // --- 4. Join the Senate if eligible.
        // Returns the replaced member, if any.
        let replaced = Self::join_senate_if_eligible(&hotkey)?;

        if replaced.is_none() {
            // Not eligible to join the Senate, or no replacement needed.
            // Check if the hotkey is *now* a member of the Senate.
            // Otherwise, error out.
            ensure!(
                T::SenateMembers::is_member(&hotkey),
                Error::<T>::StakeTooLowForRoot, // Had less stake than the lowest stake incumbent.
            );
        }

        // --- 5. Log and announce the successful Senate adjustment.
        log::info!(
            "SenateAdjusted(old_hotkey:{:?} hotkey:{:?})",
            replaced,
            hotkey
        );
        Self::deposit_event(Event::SenateAdjusted {
            old_member: replaced.cloned(),
            new_member: hotkey,
        });

        // --- 6. Finish and return success.
        Ok(())
    }

    // Checks if a hotkey should be a member of the Senate, and if so, adds them.
    //
    // # Arguments:
    // * 'hotkey': The hotkey that the user wants to register to the root network.
    //
    // # Returns:
    // * 'Result<Option<&T::AccountId>, Error<T>>': A result containing the replaced member, if any.
    //
    fn join_senate_if_eligible(hotkey: &T::AccountId) -> Result<Option<&T::AccountId>, Error<T>> {
        // Get the root network UID.
        let root_netuid: u16 = Self::get_root_netuid();

        // --- 1. Check the hotkey is registered in the root network.
        ensure!(
            Uids::<T>::contains_key(root_netuid, hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // --- 2. Verify the hotkey is NOT already a member of the Senate.
        ensure!(
            !T::SenateMembers::is_member(hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // --- 3. Grab the hotkey's stake.
        let current_stake = Self::get_global_for_hotkey(hotkey);

        // Add the hotkey to the Senate.
        // If we're full, we'll swap out the lowest stake member.
        let members = T::SenateMembers::members();
        let last: Option<&T::AccountId> = None;
        if (members.len() as u32) == T::SenateMembers::max_members() {
            let mut sorted_members = members.clone();
            sorted_members.sort_by(|a, b| {
                let a_stake = Self::get_global_for_hotkey(a);
                let b_stake = Self::get_global_for_hotkey(b);

                b_stake.cmp(&a_stake)
            });

            if let Some(last) = sorted_members.last() {
                let last_stake = Self::get_global_for_hotkey(last);

                if last_stake < current_stake {
                    // Swap the member with the lowest stake.
                    T::SenateMembers::swap_member(last, hotkey)
                        .map_err(|_| Error::<T>::CouldNotJoinSenate)?;
                }
            }
        } else {
            T::SenateMembers::add_member(hotkey).map_err(|_| Error::<T>::CouldNotJoinSenate)?;
        }

        // Return the swapped out member, if any.
        Ok(last)
    }

    // DEPRECATED.
    // pub fn do_set_root_weights(
    //     origin: T::RuntimeOrigin,
    //     netuid: u16,
    //     hotkey: T::AccountId,
    //     uids: Vec<u16>,
    //     values: Vec<u16>,
    //     version_key: u64,
    // ) -> dispatch::DispatchResult {
    //     // Check the caller's signature. This is the coldkey of a registered account.
    //     let coldkey = ensure_signed(origin)?;
    //     log::info!(
    //         "do_set_root_weights( origin:{:?} netuid:{:?}, uids:{:?}, values:{:?})",
    //         coldkey,
    //         netuid,
    //         uids,
    //         values
    //     );

    //     // Check the hotkey account exists.
    //     ensure!(
    //         Self::hotkey_account_exists(&hotkey),
    //         Error::<T>::HotKeyAccountNotExists
    //     );

    //     // Check that the signer coldkey owns the hotkey
    //     ensure!(
    //         Self::get_owning_coldkey_for_hotkey(&hotkey) == coldkey,
    //         Error::<T>::NonAssociatedColdKey
    //     );

    //     // Check to see if this is a valid network.
    //     ensure!(
    //         Self::if_subnet_exist(netuid),
    //         Error::<T>::SubNetworkDoesNotExist
    //     );

    //     // Check that this is the root network.
    //     ensure!(netuid == Self::get_root_netuid(), Error::<T>::NotRootSubnet);

    //     // Check that the length of uid list and value list are equal for this network.
    //     ensure!(
    //         Self::uids_match_values(&uids, &values),
    //         Error::<T>::WeightVecNotEqualSize
    //     );

    //     // Check to see if the number of uids is within the max allowed uids for this network.
    //     // For the root network this number is the number of subnets.
    //     ensure!(
    //         !Self::contains_invalid_root_uids(&uids),
    //         Error::<T>::UidVecContainInvalidOne
    //     );

    //     // Check to see if the hotkey is registered to the passed network.
    //     ensure!(
    //         Self::is_hotkey_registered_on_network(netuid, &hotkey),
    //         Error::<T>::HotKeyNotRegisteredInSubNet
    //     );

    //     // Check to see if the hotkey has enough stake to set weights.
    //     ensure!(
    //         Self::get_total_alpha(&hotkey) >= Self::get_weights_min_stake(),
    //         Error::<T>::NotEnoughStakeToSetWeights
    //     );

    //     // Ensure version_key is up-to-date.
    //     ensure!(
    //         Self::check_version_key(netuid, version_key),
    //         Error::<T>::IncorrectWeightVersionKey
    //     );

    //     // Get the neuron uid of associated hotkey on network netuid.
    //     let neuron_uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey)?;

    //     // Ensure the uid is not setting weights faster than the weights_set_rate_limit.
    //     let current_block: u64 = Self::get_current_block_as_u64();
    //     ensure!(
    //         Self::check_rate_limit(netuid, neuron_uid, current_block),
    //         Error::<T>::SettingWeightsTooFast
    //     );

    //     // Ensure the passed uids contain no duplicates.
    //     ensure!(!Self::has_duplicate_uids(&uids), Error::<T>::DuplicateUids);

    //     // Ensure that the weights have the required length.
    //     ensure!(
    //         Self::check_length(netuid, neuron_uid, &uids, &values),
    //         Error::<T>::WeightVecLengthIsLow
    //     );

    //     // Max-upscale the weights.
    //     let max_upscaled_weights: Vec<u16> = vec_u16_max_upscale_to_u16(&values);

    //     // Ensure the weights are max weight limited
    //     ensure!(
    //         Self::max_weight_limited(netuid, neuron_uid, &uids, &max_upscaled_weights),
    //         Error::<T>::MaxWeightExceeded
    //     );

    //     // Zip weights for sinking to storage map.
    //     let mut zipped_weights: Vec<(u16, u16)> = vec![];
    //     for (uid, val) in uids.iter().zip(max_upscaled_weights.iter()) {
    //         zipped_weights.push((*uid, *val))
    //     }

    //     // Set weights under netuid, uid double map entry.
    //     Weights::<T>::insert(netuid, neuron_uid, zipped_weights);

    //     // Set the activity for the weights on this network.
    //     Self::set_last_update_for_uid(netuid, neuron_uid, current_block);

    //     // Emit the tracking event.
    //     log::info!(
    //         "RootWeightsSet( netuid:{:?}, neuron_uid:{:?} )",
    //         netuid,
    //         neuron_uid
    //     );
    //     Self::deposit_event(Event::WeightsSet(netuid, neuron_uid));

    //     // Return ok.
    //     Ok(())
    // }

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
}
