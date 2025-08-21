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
use frame_support::dispatch::Pays;
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::weights::Weight;
use safe_math::*;
use sp_core::Get;
use substrate_fixed::types::I64F64;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::SwapHandler;

impl<T: Config> Pallet<T> {
    /// Fetches the total count of root network validators
    ///
    /// This function retrieves the total number of root network validators.
    ///
    /// # Returns:
    /// * 'u16': The total number of root network validators
    ///
    pub fn get_num_root_validators() -> u16 {
        Self::get_subnetwork_n(NetUid::ROOT)
    }

    /// Fetches the max validators count of root network.
    ///
    /// This function retrieves the max validators count of root network.
    ///
    /// # Returns:
    /// * 'u16': The max validators count of root network.
    ///
    pub fn get_max_root_validators() -> u16 {
        Self::get_max_allowed_uids(NetUid::ROOT)
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
    pub fn contains_invalid_root_uids(netuids: &[NetUid]) -> bool {
        for netuid in netuids {
            if !Self::if_subnet_exist(*netuid) {
                log::debug!("contains_invalid_root_uids: netuid {netuid:?} does not exist");
                return true;
            }
        }
        false
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
        let current_block_number: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::if_subnet_exist(NetUid::ROOT),
            Error::<T>::RootNetworkDoesNotExist
        );

        // --- 1. Ensure that the call originates from a signed source and retrieve the caller's account ID (coldkey).
        let coldkey = ensure_signed(origin)?;
        log::debug!("do_root_register( coldkey: {coldkey:?}, hotkey: {hotkey:?} )");

        // --- 2. Ensure that the number of registrations in this block doesn't exceed the allowed limit.
        ensure!(
            Self::get_registrations_this_block(NetUid::ROOT)
                < Self::get_max_registrations_per_block(NetUid::ROOT),
            Error::<T>::TooManyRegistrationsThisBlock
        );

        // --- 3. Ensure that the number of registrations in this interval doesn't exceed thrice the target limit.
        ensure!(
            Self::get_registrations_this_interval(NetUid::ROOT)
                < Self::get_target_registrations_per_interval(NetUid::ROOT).saturating_mul(3),
            Error::<T>::TooManyRegistrationsThisInterval
        );

        // --- 4. Check if the hotkey is already registered. If so, error out.
        ensure!(
            !Uids::<T>::contains_key(NetUid::ROOT, &hotkey),
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
            Self::append_neuron(NetUid::ROOT, &hotkey, current_block_number);
            log::debug!("add new neuron: {hotkey:?} on uid {subnetwork_uid:?}");
        } else {
            // --- 13.1.1 The network is full. Perform replacement.
            // Find the neuron with the lowest stake value to replace.
            let mut lowest_stake = AlphaCurrency::MAX;
            let mut lowest_uid: u16 = 0;

            // Iterate over all keys in the root network to find the neuron with the lowest stake.
            for (uid_i, hotkey_i) in Keys::<T>::iter_prefix(NetUid::ROOT) {
                let stake_i = Self::get_stake_for_hotkey_on_subnet(&hotkey_i, NetUid::ROOT);
                if stake_i < lowest_stake {
                    lowest_stake = stake_i;
                    lowest_uid = uid_i;
                }
            }
            subnetwork_uid = lowest_uid;
            let replaced_hotkey: T::AccountId =
                Self::get_hotkey_for_net_and_uid(NetUid::ROOT, subnetwork_uid)?;

            // --- 13.1.2 The new account has a higher stake than the one being replaced.
            ensure!(
                lowest_stake < Self::get_stake_for_hotkey_on_subnet(&hotkey, NetUid::ROOT),
                Error::<T>::StakeTooLowForRoot
            );

            // --- 13.1.3 The new account has a higher stake than the one being replaced.
            // Replace the neuron account with new information.
            Self::replace_neuron(NetUid::ROOT, lowest_uid, &hotkey, current_block_number);

            log::debug!(
                "replace neuron: {replaced_hotkey:?} with {hotkey:?} on uid {subnetwork_uid:?}"
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
        RegistrationsThisInterval::<T>::mutate(NetUid::ROOT, |val| *val += 1);
        #[allow(clippy::arithmetic_side_effects)]
        // note this RA + clippy false positive is a known substrate issue
        RegistrationsThisBlock::<T>::mutate(NetUid::ROOT, |val| *val += 1);

        // --- 16. Log and announce the successful registration.
        log::debug!(
            "RootRegistered(netuid:{:?} uid:{:?} hotkey:{:?})",
            NetUid::ROOT,
            subnetwork_uid,
            hotkey
        );
        Self::deposit_event(Event::NeuronRegistered(
            NetUid::ROOT,
            subnetwork_uid,
            hotkey,
        ));

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
        ensure!(
            Self::if_subnet_exist(NetUid::ROOT),
            Error::<T>::RootNetworkDoesNotExist
        );

        // --- 1. Ensure that the call originates from a signed source and retrieve the caller's account ID (coldkey).
        let coldkey = ensure_signed(origin)?;
        log::debug!("do_root_register( coldkey: {coldkey:?}, hotkey: {hotkey:?} )");

        // --- 2. Check if the hotkey is already registered to the root network. If not, error out.
        ensure!(
            Uids::<T>::contains_key(NetUid::ROOT, &hotkey),
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
        log::debug!("SenateAdjusted(old_hotkey:{replaced:?} hotkey:{hotkey:?})");
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
        // --- 1. Check the hotkey is registered in the root network.
        ensure!(
            Uids::<T>::contains_key(NetUid::ROOT, hotkey),
            Error::<T>::HotKeyNotRegisteredInSubNet
        );

        // --- 2. Verify the hotkey is NOT already a member of the Senate.
        ensure!(
            !T::SenateMembers::is_member(hotkey),
            Error::<T>::HotKeyAlreadyRegisteredInSubNet
        );

        // --- 3. Grab the hotkey's stake.
        let current_stake = Self::get_stake_for_hotkey_on_subnet(hotkey, NetUid::ROOT);

        // Add the hotkey to the Senate.
        // If we're full, we'll swap out the lowest stake member.
        let members = T::SenateMembers::members();
        let last: Option<&T::AccountId> = None;
        if (members.len() as u32) == T::SenateMembers::max_members() {
            let mut sorted_members = members.clone();
            sorted_members.sort_by(|a, b| {
                let a_stake = Self::get_stake_for_hotkey_on_subnet(a, NetUid::ROOT);
                let b_stake = Self::get_stake_for_hotkey_on_subnet(b, NetUid::ROOT);

                b_stake.cmp(&a_stake)
            });

            if let Some(last) = sorted_members.last() {
                let last_stake = Self::get_stake_for_hotkey_on_subnet(last, NetUid::ROOT);

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
    pub fn do_dissolve_network(netuid: NetUid) -> dispatch::DispatchResult {
        // 1. --- The network exists?
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // 2. --- Perform the cleanup before removing the network.
        T::SwapInterface::dissolve_all_liquidity_providers(netuid)?;
        Self::destroy_alpha_in_out_stakes(netuid)?;

        // 3. --- Remove the network
        Self::remove_network(netuid);

        // 4. --- Emit the NetworkRemoved event
        log::debug!("NetworkRemoved( netuid:{netuid:?} )");
        Self::deposit_event(Event::NetworkRemoved(netuid));

        Ok(())
    }

    pub fn remove_network(netuid: NetUid) {
        // --- 1. Get the owner and remove from SubnetOwner.
        let owner_coldkey: T::AccountId = SubnetOwner::<T>::get(netuid);
        SubnetOwner::<T>::remove(netuid);

        // --- 2. Remove network count.
        SubnetworkN::<T>::remove(netuid);

        // --- 3. Remove network modality storage.
        NetworkModality::<T>::remove(netuid);

        // --- 4. Remove netuid from added networks.
        NetworksAdded::<T>::remove(netuid);

        // --- 5. Decrement the network counter.
        TotalNetworks::<T>::mutate(|n: &mut u16| *n = n.saturating_sub(1));

        // --- 6. Remove various network-related storages.
        NetworkRegisteredAt::<T>::remove(netuid);

        // --- 7. Remove incentive mechanism memory.
        let _ = Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        let keys = Keys::<T>::iter_prefix(netuid).collect::<Vec<_>>();
        let _ = Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Bonds::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 8. Remove the weights for this subnet itself.
        let _ = Weights::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 9. Also zero out any weights *in the root network* that point to this netuid.
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<NetUid, u16, Vec<(u16, u16)>>>::iter_prefix(
                NetUid::ROOT,
            )
        {
            let mut modified_weights = weights_i.clone();
            for (subnet_id, weight) in modified_weights.iter_mut() {
                // If the root network had a weight pointing to this netuid, set it to 0
                if subnet_id == &u16::from(netuid) {
                    *weight = 0;
                }
            }
            Weights::<T>::insert(NetUid::ROOT, uid_i, modified_weights);
        }

        // --- 10. Remove network-related parameters and data.
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

        for (_uid, key) in keys {
            IsNetworkMember::<T>::remove(key, netuid);
        }
        Tempo::<T>::remove(netuid);
        Kappa::<T>::remove(netuid);
        Difficulty::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        ActivityCutoff::<T>::remove(netuid);
        MaxWeightsLimit::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        RegistrationsThisInterval::<T>::remove(netuid);
        POWRegistrationsThisInterval::<T>::remove(netuid);
        BurnRegistrationsThisInterval::<T>::remove(netuid);
        SubnetTAO::<T>::remove(netuid);
        SubnetAlphaInEmission::<T>::remove(netuid);
        SubnetAlphaOutEmission::<T>::remove(netuid);
        SubnetTaoInEmission::<T>::remove(netuid);
        SubnetVolume::<T>::remove(netuid);
        SubnetMovingPrice::<T>::remove(netuid);

        // --- 12. Add the balance back to the owner.
        SubnetOwner::<T>::remove(netuid);

        // --- 13. Remove subnet identity if it exists.
        if SubnetIdentitiesV3::<T>::contains_key(netuid) {
            SubnetIdentitiesV3::<T>::remove(netuid);
            Self::deposit_event(Event::SubnetIdentityRemoved(netuid));
        }

        // --- Log final removal.
        log::debug!(
            "remove_network: netuid={netuid}, owner={owner_coldkey:?} removed successfully"
        );
    }

    #[allow(clippy::arithmetic_side_effects)]
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
    pub fn get_network_lock_cost() -> TaoCurrency {
        let last_lock = Self::get_network_last_lock();
        let min_lock = Self::get_network_min_lock();
        let last_lock_block = Self::get_network_last_lock_block();
        let current_block = Self::get_current_block_as_u64();
        let lock_reduction_interval = Self::get_lock_reduction_interval();
        let mult: TaoCurrency = if last_lock_block == 0 { 1 } else { 2 }.into();

        let mut lock_cost = last_lock.saturating_mul(mult).saturating_sub(
            last_lock
                .to_u64()
                .safe_div(lock_reduction_interval)
                .saturating_mul(current_block.saturating_sub(last_lock_block))
                .into(),
        );

        if lock_cost < min_lock {
            lock_cost = min_lock;
        }

        log::debug!(
            "last_lock: {last_lock:?}, min_lock: {min_lock:?}, last_lock_block: {last_lock_block:?}, lock_reduction_interval: {lock_reduction_interval:?}, current_block: {current_block:?}, mult: {mult:?} lock_cost: {lock_cost:?}"
        );

        lock_cost
    }

    pub fn get_network_registered_block(netuid: NetUid) -> u64 {
        NetworkRegisteredAt::<T>::get(netuid)
    }
    pub fn get_network_immunity_period() -> u64 {
        NetworkImmunityPeriod::<T>::get()
    }
    pub fn set_network_immunity_period(net_immunity_period: u64) {
        NetworkImmunityPeriod::<T>::set(net_immunity_period);
        Self::deposit_event(Event::NetworkImmunityPeriodSet(net_immunity_period));
    }
    pub fn set_network_min_lock(net_min_lock: TaoCurrency) {
        NetworkMinLockCost::<T>::set(net_min_lock);
        Self::deposit_event(Event::NetworkMinLockCostSet(net_min_lock));
    }
    pub fn get_network_min_lock() -> TaoCurrency {
        NetworkMinLockCost::<T>::get()
    }
    pub fn set_network_last_lock(net_last_lock: TaoCurrency) {
        NetworkLastLockCost::<T>::set(net_last_lock);
    }
    pub fn get_network_last_lock() -> TaoCurrency {
        NetworkLastLockCost::<T>::get()
    }
    pub fn get_network_last_lock_block() -> u64 {
        NetworkLastRegistered::<T>::get()
    }
    pub fn set_network_last_lock_block(block: u64) {
        NetworkLastRegistered::<T>::set(block);
    }
    pub fn set_lock_reduction_interval(interval: u64) {
        NetworkLockReductionInterval::<T>::set(interval);
        Self::deposit_event(Event::NetworkLockCostReductionIntervalSet(interval));
    }
    pub fn get_lock_reduction_interval() -> u64 {
        let interval: I64F64 =
            I64F64::saturating_from_num(NetworkLockReductionInterval::<T>::get());
        let block_emission: I64F64 = I64F64::saturating_from_num(
            Self::get_block_emission()
                .unwrap_or(1_000_000_000.into())
                .to_u64(),
        );
        let halving: I64F64 = block_emission
            .checked_div(I64F64::saturating_from_num(1_000_000_000))
            .unwrap_or(I64F64::saturating_from_num(0.0));
        let halved_interval: I64F64 = interval.saturating_mul(halving);
        halved_interval.saturating_to_num::<u64>()
    }
    pub fn get_rate_limited_last_block(rate_limit_key: &RateLimitKey) -> u64 {
        LastRateLimitedBlock::<T>::get(rate_limit_key)
    }
    pub fn set_rate_limited_last_block(rate_limit_key: &RateLimitKey, block: u64) {
        LastRateLimitedBlock::<T>::set(rate_limit_key, block);
    }

    pub fn get_network_to_prune() -> Option<NetUid> {
        let current_block: u64 = Self::get_current_block_as_u64();

        let mut candidate_netuid: Option<NetUid> = None;
        let mut candidate_price: U96F32 = U96F32::saturating_from_num(u128::MAX);
        let mut candidate_timestamp: u64 = u64::MAX;

        for (netuid, added) in NetworksAdded::<T>::iter() {
            if !added || netuid == NetUid::ROOT {
                continue;
            }
            if !Self::if_subnet_exist(netuid) {
                continue;
            }

            let registered_at = NetworkRegisteredAt::<T>::get(netuid);

            // Skip immune networks.
            if current_block < registered_at.saturating_add(Self::get_network_immunity_period()) {
                continue;
            }

            let price: U96F32 = T::SwapInterface::current_alpha_price(netuid.into());

            // If tie on price, earliest registration wins.
            if price < candidate_price
                || (price == candidate_price && registered_at < candidate_timestamp)
            {
                candidate_netuid = Some(netuid);
                candidate_price = price;
                candidate_timestamp = registered_at;
            }
        }

        candidate_netuid
    }
}
