use super::*;
use frame_support::storage::IterableStorageDoubleMap;
use frame_support::storage::IterableStorageMap;
use substrate_fixed::types::I110F18;
use substrate_fixed::types::I64F64;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    /// Executes the necessary operations for each block.
    pub fn block_step() -> Result<(), &'static str> {
        let block_number: u64 = Self::get_current_block_as_u64();
        log::debug!("block_step for block: {:?} ", block_number);
        // --- 1. Adjust difficulties.
        Self::adjust_registration_terms_for_networks();
        // --- 2. Calculate per-subnet emissions
        match Self::root_epoch(block_number) {
            Ok(_) => (),
            Err(e) => {
                log::trace!("Error while running root epoch: {:?}", e);
            }
        }
        // --- 3. Drains emission tuples ( hotkey, amount ).
        Self::drain_emission(block_number);
        // --- 4. Generates emission tuples from epoch functions.
        Self::generate_emission(block_number);
        // Return ok.
        Ok(())
    }

    /// Calculates the number of blocks until the next epoch for a given network.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network identifier.
    /// * `tempo` - The tempo of the network.
    /// * `block_number` - The current block number.
    ///
    /// # Returns
    ///
    /// The number of blocks until the next epoch.
    pub fn blocks_until_next_epoch(netuid: u16, tempo: u16, block_number: u64) -> u64 {
        // tempo | netuid | # first epoch block
        //   1        0               0
        //   1        1               1
        //   2        0               1
        //   2        1               0
        //   100      0              99
        //   100      1              98
        // Special case: tempo = 0, the network never runs.
        if tempo == 0 {
            return 1000;
        }
        return tempo as u64 - (block_number + netuid as u64 + 1) % (tempo as u64 + 1);
    }

    // Helper function returns the number of tuples to drain on a particular step based on
    // the remaining tuples to sink and the block number
    //
    pub fn tuples_to_drain_this_block(
        netuid: u16,
        tempo: u16,
        block_number: u64,
        n_remaining: usize,
    ) -> usize {
        let blocks_until_epoch: u64 = Self::blocks_until_next_epoch(netuid, tempo, block_number);
        if blocks_until_epoch / 2 == 0 {
            return n_remaining;
        } // drain all.
        if tempo / 2 == 0 {
            return n_remaining;
        } // drain all
        if n_remaining == 0 {
            return 0;
        } // nothing to drain at all.
          // Else return enough tuples to drain all within half the epoch length.
        let to_sink_via_tempo: usize = n_remaining / (tempo as usize / 2);
        let to_sink_via_blocks_until_epoch: usize = n_remaining / (blocks_until_epoch as usize / 2);
        if to_sink_via_tempo > to_sink_via_blocks_until_epoch {
            return to_sink_via_tempo;
        } else {
            return to_sink_via_blocks_until_epoch;
        }
    }

    pub fn has_loaded_emission_tuples(netuid: u16) -> bool {
        LoadedEmission::<T>::contains_key(netuid)
    }
    pub fn get_loaded_emission_tuples(netuid: u16) -> Vec<(T::AccountId, u64, u64)> {
        LoadedEmission::<T>::get(netuid).unwrap()
    }

    // Reads from the loaded emission storage which contains lists of pending emission tuples ( hotkey, amount )
    // and distributes small chunks of them at a time.
    //
    pub fn drain_emission(_: u64) {
        // --- 1. We iterate across each network.
        for (netuid, _) in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {
            if !Self::has_loaded_emission_tuples(netuid) {
                continue;
            } // There are no tuples to emit.
            let tuples_to_drain: Vec<(T::AccountId, u64, u64)> =
                Self::get_loaded_emission_tuples(netuid);
            let mut total_emitted: u64 = 0;
            for (hotkey, server_amount, validator_amount) in tuples_to_drain.iter() {
                Self::emit_inflation_through_hotkey_account(
                    &hotkey,
                    netuid,
                    *server_amount,
                    *validator_amount,
                );
                total_emitted += *server_amount + *validator_amount;
            }
            LoadedEmission::<T>::remove(netuid);
            TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add(total_emitted));
        }
    }

    /// Generates emission for each network and adds it to the pending emission storage.
    /// If a network has reached its tempo, it runs the epoch mechanism and generates emission tuples.
    ///
    /// # Arguments
    ///
    /// * `block_number` - The current block number.
    ///
    /// # Sequence Diagram
    ///
    /// ```mermaid
    /// sequenceDiagram
    ///     participant BlockStep
    ///     participant Tempo
    ///     participant SubnetOwner
    ///     participant PendingEmission
    ///     participant LoadedEmission
    ///
    ///     BlockStep->>Tempo: Iterate over each network (netuid, tempo)
    ///     loop For each network
    ///         alt If netuid is root network
    ///             BlockStep->>BlockStep: Skip (root emission is burned)
    ///         else
    ///             BlockStep->>BlockStep: Get subnet emission value
    ///             BlockStep->>SubnetOwner: Check if network has an owner
    ///             alt If network has an owner
    ///                 BlockStep->>BlockStep: Calculate owner's cut
    ///                 BlockStep->>BlockStep: Subtract owner's cut from remaining emission
    ///                 BlockStep->>SubnetOwner: Add owner's cut to coldkey account balance
    ///                 BlockStep->>BlockStep: Create new tokens from coinbase (owner's cut)
    ///             end
    ///             BlockStep->>PendingEmission: Add remaining emission to pending emission
    ///             BlockStep->>BlockStep: Check if network has reached tempo
    ///             alt If network has not reached tempo
    ///                 BlockStep->>BlockStep: Increment blocks_since_last_step counter
    ///             else
    ///                 BlockStep->>PendingEmission: Retrieve pending emission (emission_to_drain)
    ///                 BlockStep->>PendingEmission: Set pending emission to zero
    ///                 BlockStep->>BlockStep: Run epoch mechanism (generate emission tuples)
    ///                 BlockStep->>BlockStep: Calculate total emission from tuples
    ///                 alt If total emission exceeds allowed emission_to_drain
    ///                     BlockStep->>BlockStep: Skip to next iteration
    ///                 else
    ///                     BlockStep->>LoadedEmission: Concatenate new emission tuples with existing loaded emission
    ///                     BlockStep->>BlockStep: Reset blocks_since_last_step counter to zero
    ///                     BlockStep->>BlockStep: Update last_mechanism_step_block with current block number
    ///                 end
    ///             end
    ///         end
    ///     end
    /// ```
    ///
    /// # Description
    ///
    /// The `generate_emission` function is responsible for generating emission for each network and adding it to the pending emission storage. It iterates over each network using the `Tempo` storage map, which maps a network ID (`netuid`) to its tempo value.
    ///
    /// For each network, the function performs the following steps:
    ///
    /// 1. If the network is the root network, it skips the emission generation since the root network's emission is burned.
    ///
    /// 2. For non-root networks, it retrieves the subnet emission value using `get_subnet_emission_value(netuid)`.
    ///
    /// 3. It checks if the network has an owner by calling `SubnetOwner::<T>::contains_key(netuid)`. If the network has an owner, it calculates the owner's cut of the emission, subtracts it from the remaining emission, adds the owner's cut to their coldkey account balance, and creates new tokens from the coinbase using the owner's cut amount.
    ///
    /// 4. The remaining emission (after the owner's cut, if applicable) is added to the network's pending emission using `PendingEmission::<T>::mutate()`.
    ///
    /// 5. It checks if the network has reached its tempo by calling `blocks_until_next_epoch(netuid, tempo, block_number)`.
    ///
    /// 6. If the network has not reached its tempo, it increments the `blocks_since_last_step` counter for the network using `set_blocks_since_last_step()` and continues to the next iteration.
    ///
    /// 7. If the network has reached its tempo, it retrieves the pending emission for
    ///    the network from `PendingEmission::<T>::get(netuid)` and stores it in `emission_to_drain`.
    ///    It then sets the pending emission for the network to zero using
    ///    `PendingEmission::<T>::insert(netuid, 0)`.
    ///
    /// 8. The function runs the epoch mechanism for the network by calling `epoch(netuid, emission_to_drain)`,
    ///    which returns a vector of emission tuples `(account_id, validator_emission, server_emission)`.
    ///
    /// 9. It calculates the total emission by summing the validator and server emissions from the emission tuples.
    ///    If the total emission exceeds the allowed `emission_to_drain`, it skips to the next iteration.
    ///
    /// 10. If the total emission is within the allowed limit, it concatenates the new emission tuples with any
    ///     existing loaded emission tuples for the network using `LoadedEmission::<T>::insert()`.
    ///
    /// 11. Finally, it resets the `blocks_since_last_step` counter for the network to zero using
    ///     `set_blocks_since_last_step(netuid, 0)` and updates the `last_mechanism_step_block` for the network
    ///     with the current `block_number` using `set_last_mechanism_step_block(netuid, block_number)`.
    ///
    /// The `generate_emission` function plays a crucial role in the emission generation process for each network.
    /// It ensures that emission is generated based on the network's tempo, distributes the emission to the network
    /// owner (if applicable), and adds the remaining emission to the network's pending emission. If a network has
    /// reached its tempo, it triggers the epoch mechanism to generate emission tuples and updates the loaded emission
    /// for the network.
    pub fn generate_emission(block_number: u64) {
        // --- 1. Iterate across each network and add pending emission into stash.
        for (netuid, tempo) in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {
            // Skip the root network.
            if netuid == Self::get_root_netuid() {
                // Root emission is burned.
                continue;
            }

            // --- 2. Queue the emission due to this network.
            let new_queued_emission: u64 = Self::get_subnet_emission_value(netuid);
            log::debug!(
                "generate_emission for netuid: {:?} with tempo: {:?} and emission: {:?}",
                netuid,
                tempo,
                new_queued_emission,
            );

            let subnet_has_owner = SubnetOwner::<T>::contains_key(netuid);
            let mut remaining = I96F32::from_num(new_queued_emission);
            if subnet_has_owner {
                let cut = remaining
                    .saturating_mul(I96F32::from_num(Self::get_subnet_owner_cut()))
                    .saturating_div(I96F32::from_num(u16::MAX));

                remaining = remaining.saturating_sub(cut);

                Self::add_balance_to_coldkey_account(
                    &Self::get_subnet_owner(netuid),
                    Self::u64_to_balance(cut.to_num::<u64>()).unwrap(),
                );

                // We are creating tokens here from the coinbase.
                Self::coinbase(cut.to_num::<u64>());
            }
            // --- 5. Add remaining amount to the network's pending emission.
            PendingEmission::<T>::mutate(netuid, |queued| *queued += remaining.to_num::<u64>());
            log::debug!(
                "netuid_i: {:?} queued_emission: +{:?} ",
                netuid,
                new_queued_emission
            );

            // --- 6. Check to see if this network has reached tempo.
            if Self::blocks_until_next_epoch(netuid, tempo, block_number) != 0 {
                // --- 3.1 No epoch, increase blocks since last step and continue,
                Self::set_blocks_since_last_step(
                    netuid,
                    Self::get_blocks_since_last_step(netuid) + 1,
                );
                continue;
            }

            // --- 7 This network is at tempo and we are running its epoch.
            // First drain the queued emission.
            let emission_to_drain: u64 = PendingEmission::<T>::get(netuid);
            PendingEmission::<T>::insert(netuid, 0);

            // --- 8. Run the epoch mechanism and return emission tuples for hotkeys in the network.
            let emission_tuples_this_block: Vec<(T::AccountId, u64, u64)> =
                Self::epoch(netuid, emission_to_drain);
            log::debug!(
                "netuid_i: {:?} emission_to_drain: {:?} ",
                netuid,
                emission_to_drain
            );

            // --- 9. Check that the emission does not exceed the allowed total.
            let emission_sum: u128 = emission_tuples_this_block
                .iter()
                .map(|(_account_id, ve, se)| *ve as u128 + *se as u128)
                .sum();
            if emission_sum > emission_to_drain as u128 {
                continue;
            } // Saftey check.

            // --- 10. Sink the emission tuples onto the already loaded.
            let mut concat_emission_tuples: Vec<(T::AccountId, u64, u64)> =
                emission_tuples_this_block.clone();
            if Self::has_loaded_emission_tuples(netuid) {
                // 10.a We already have loaded emission tuples, so we concat the new ones.
                let mut current_emission_tuples: Vec<(T::AccountId, u64, u64)> =
                    Self::get_loaded_emission_tuples(netuid);
                concat_emission_tuples.append(&mut current_emission_tuples);
            }
            LoadedEmission::<T>::insert(netuid, concat_emission_tuples);

            // --- 11 Set counters.
            Self::set_blocks_since_last_step(netuid, 0);
            Self::set_last_mechanism_step_block(netuid, block_number);
        }
    }

    /// Distributes token inflation through the hotkey based on emission.
    ///
    /// The function ensures that the inflation is distributed onto the accounts in proportion to the stake
    /// delegated minus the take. This function is called after an epoch to distribute the newly minted stake
    /// according to delegation.
    ///
    /// # Arguments
    ///
    /// * `hotkey` - The account ID of the hotkey.
    /// * `server_emission` - The amount of emission allocated for the server.
    /// * `validator_emission` - The amount of emission allocated for the validator.
    ///
    /// # Sequence Diagram
    ///
    /// ```mermaid
    /// sequenceDiagram
    ///     participant Function
    ///     participant Hotkey
    ///     participant Stakers
    ///
    ///     Function->>Hotkey: Check if hotkey is a delegate
    ///     alt If hotkey is not a delegate
    ///         Function->>Hotkey: Increase stake on hotkey account by server_emission + validator_emission
    ///     else If hotkey is a delegate
    ///         Function->>Function: Calculate delegate's proportional take from validator_emission
    ///         Function->>Function: Calculate remaining validator_emission after subtracting delegate_take
    ///         loop For each staker and their stake
    ///             Function->>Function: Calculate proportional emission based on staker's stake
    ///             Function->>Stakers: Increase stake on staker's account by stake_proportion
    ///             Function->>Function: Subtract stake_proportion from remaining_validator_emission
    ///         end
    ///         Function->>Hotkey: Increase stake on hotkey account by delegate_take + remaining_validator_emission
    ///         Function->>Hotkey: Increase stake on hotkey account by server_emission
    ///     end
    /// ```
    ///
    /// # Description
    ///
    /// The `emit_inflation_through_hotkey_account` function distributes the `validator_emission` and `server_emission`
    /// to the stakers and the delegate based on their stake proportions. Here's how it works:
    ///
    /// 1. It first checks if the `hotkey` is a delegate by calling `Self::hotkey_is_delegate(hotkey)`. If the `hotkey`
    ///    is not a delegate, it simply increases the stake on the `hotkey` account by the sum of `server_emission` and
    ///    `validator_emission` using `Self::increase_stake_on_hotkey_account(hotkey, server_emission + validator_emission)`
    ///    and returns.
    ///
    /// 2. If the `hotkey` is a delegate, the function proceeds to distribute the `validator_emission` and `server_emission`
    ///    separately.
    ///
    /// 3. It retrieves the total stake for the `hotkey` by calling `Self::get_total_stake_for_hotkey(hotkey)` and stores
    ///    it in `total_hotkey_stake`.
    ///
    /// 4. It calculates the delegate's proportional take from the `validator_emission` by calling
    ///    `Self::calculate_delegate_proportional_take(hotkey, validator_emission)` and stores it in `delegate_take`.
    ///    This represents the portion of the `validator_emission` that the delegate keeps for themselves.
    ///
    /// 5. It calculates the remaining `validator_emission` after subtracting the `delegate_take` and stores it in
    ///    `validator_emission_minus_take` and `remaining_validator_emission`.
    ///
    /// 6. It iterates over the stakers (cold keys) and their corresponding stakes for the given `hotkey` using
    ///    `<Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(hotkey)`.
    ///
    /// 7. For each staker (`owning_coldkey_i`) and their stake (`stake_i`), it calculates the proportional emission
    ///    based on their stake using `Self::calculate_stake_proportional_emission(stake_i, total_hotkey_stake, validator_emission_minus_take)`
    ///    and stores it in `stake_proportion`.
    ///
    ///
    /// 8. It increases the stake on the staker's account (`owning_coldkey_i`) for the given `hotkey` by the `stake_proportion`
    ///    using `Self::increase_stake_on_coldkey_hotkey_account(&owning_coldkey_i, hotkey, stake_proportion)`.
    ///
    /// 9. It subtracts the `stake_proportion` from the `remaining_validator_emission` to keep track of the remaining
    ///    emission to be distributed.
    ///
    /// 10. After iterating over all the stakers, it increases the stake on the `hotkey` account by the sum of `delegate_take`
    ///     and `remaining_validator_emission` using `Self::increase_stake_on_hotkey_account(hotkey, delegate_take + remaining_validator_emission)`.
    ///     This step is performed after the iteration to avoid affecting the stake proportions during the calculation.
    ///
    /// 11. Finally, it increases the stake on the `hotkey` account by the `server_emission` using
    ///     `Self::increase_stake_on_hotkey_account(hotkey, server_emission)`. The `server_emission` is distributed
    ///     entirely to the delegate (hotkey) owner.
    ///
    /// In summary, this function distributes the `validator_emission` and `server_emission` to the stakers and the delegate
    /// based on their stake proportions. The delegate receives a proportional take from the `validator_emission` and the
    /// entire `server_emission`. The remaining `validator_emission` is distributed among the stakers based on their stake
    /// proportions.
    pub fn emit_inflation_through_hotkey_account(
        delegate: &T::AccountId,
        netuid: u16,
        server_emission: u64,
        validator_emission: u64,
    ) {
        // 1. Check if the hotkey is not a delegate and thus the emission is entirely owed to them.
        if !Self::hotkey_is_delegate(delegate) {
            let total_delegate_emission: u64 = server_emission + validator_emission;
            Self::increase_stake_on_hotkey_account(delegate, netuid, total_delegate_emission);
            return;
        }
        // 2. Else the key is a delegate, first compute the delegate take from the emission.
        let take_proportion: I64F64 =
            I64F64::from_num(Delegates::<T>::get(delegate)) / I64F64::from_num(u16::MAX);
        let delegate_take: I64F64 = take_proportion * I64F64::from_num(validator_emission);
        let delegate_take_u64: u64 = delegate_take.to_num::<u64>();
        let remaining_validator_emission: u64 = validator_emission - delegate_take_u64;
        let mut residual: u64 = remaining_validator_emission;

        // 3. For each nominator compute its proportion of stake weight and distribute the remaining emission to them.
        let global_stake_weight: I64F64 = Self::get_global_stake_weight_float();
        let delegate_local_stake: u64 =
            Self::get_total_stake_for_hotkey_and_subnet(delegate, netuid);
        let delegate_global_stake: u64 = Self::get_total_stake_for_hotkey(delegate);
        log::debug!(
            "global_stake_weight: {:?}, delegate_local_stake: {:?}, delegate_global_stake: {:?}",
            global_stake_weight,
            delegate_local_stake,
            delegate_global_stake
        );

        if delegate_local_stake + delegate_global_stake != 0 {
            for (nominator_i, _) in <Stake<T> as IterableStorageDoubleMap<
                T::AccountId,
                T::AccountId,
                u64,
            >>::iter_prefix(delegate)
            {
                // 3.a Compute the stake weight percentage for the nominatore weight.
                let nominator_local_stake: u64 =
                    Self::get_subnet_stake_for_coldkey_and_hotkey(&nominator_i, delegate, netuid);
                let nominator_local_emission_i: I64F64 = if delegate_local_stake == 0 {
                    I64F64::from_num(0)
                } else {
                    let nominator_local_percentage: I64F64 =
                        I64F64::from_num(nominator_local_stake)
                            / I64F64::from_num(delegate_local_stake);
                    nominator_local_percentage
                        * I64F64::from_num(remaining_validator_emission)
                        * (I64F64::from_num(1.0) - global_stake_weight)
                };
                log::debug!(
                    "nominator_local_emission_i: {:?}",
                    nominator_local_emission_i
                );

                let nominator_global_stake: u64 =
                    Self::get_total_stake_for_hotkey_and_coldkey(delegate, &nominator_i);
                let nominator_global_emission_i: I64F64 = if delegate_global_stake == 0 {
                    I64F64::from_num(0)
                } else {
                    let nominator_global_percentage: I64F64 =
                        I64F64::from_num(nominator_global_stake)
                            / I64F64::from_num(delegate_global_stake);
                    nominator_global_percentage
                        * I64F64::from_num(remaining_validator_emission)
                        * global_stake_weight
                };
                log::debug!(
                    "nominator_global_emission_i: {:?}",
                    nominator_global_emission_i
                );
                let nominator_emission_u64: u64 =
                    (nominator_global_emission_i + nominator_local_emission_i).to_num::<u64>();

                // 3.b Increase the stake of the nominator.
                log::debug!(
                    "nominator: {:?}, global_emission: {:?}, local_emission: {:?}",
                    nominator_i,
                    nominator_global_emission_i,
                    nominator_local_emission_i
                );
                residual -= nominator_emission_u64;
                Self::increase_stake_on_coldkey_hotkey_account(
                    &nominator_i,
                    delegate,
                    netuid,
                    nominator_emission_u64,
                );
            }
        }

        // --- 5. Last increase final account balance of delegate after 4, since 5 will change the stake proportion of
        // the delegate and effect calculation in 4.
        let total_delegate_emission: u64 = delegate_take_u64 + server_emission + residual;
        log::debug!(
            "total_delegate_emission: {:?}",
            delegate_take_u64 + server_emission
        );
        Self::increase_stake_on_hotkey_account(delegate, netuid, total_delegate_emission);
    }

    // Returns emission awarded to a hotkey as a function of its proportion of the total stake.
    //
    pub fn calculate_stake_proportional_emission(
        stake: u64,
        total_stake: u64,
        emission: u64,
    ) -> u64 {
        if total_stake == 0 {
            return 0;
        };
        let stake_proportion: I64F64 = I64F64::from_num(stake) / I64F64::from_num(total_stake);
        let proportional_emission: I64F64 = I64F64::from_num(emission) * stake_proportion;
        return proportional_emission.to_num::<u64>();
    }

    // Returns the delegated stake 'take' assigned to this key. (If exists, otherwise 0)
    //
    pub fn calculate_delegate_proportional_take(hotkey: &T::AccountId, emission: u64) -> u64 {
        if Self::hotkey_is_delegate(hotkey) {
            let take_proportion: I64F64 =
                I64F64::from_num(Delegates::<T>::get(hotkey)) / I64F64::from_num(u16::MAX);
            let take_emission: I64F64 = take_proportion * I64F64::from_num(emission);
            return take_emission.to_num::<u64>();
        } else {
            return 0;
        }
    }

    // Adjusts the network difficulties/burns of every active network. Resetting state parameters.
    //
    pub fn adjust_registration_terms_for_networks() {
        log::debug!("adjust_registration_terms_for_networks");

        // --- 1. Iterate through each network.
        for (netuid, _) in <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter() {
            // --- 2. Pull counters for network difficulty.
            let last_adjustment_block: u64 = Self::get_last_adjustment_block(netuid);
            let adjustment_interval: u16 = Self::get_adjustment_interval(netuid);
            let current_block: u64 = Self::get_current_block_as_u64();
            log::debug!("netuid: {:?} last_adjustment_block: {:?} adjustment_interval: {:?} current_block: {:?}", 
                netuid,
                last_adjustment_block,
                adjustment_interval,
                current_block
            );

            // --- 3. Check if we are at the adjustment interval for this network.
            // If so, we need to adjust the registration difficulty based on target and actual registrations.
            if (current_block - last_adjustment_block) >= adjustment_interval as u64 {
                log::debug!("interval reached.");

                // --- 4. Get the current counters for this network w.r.t burn and difficulty values.
                let current_burn: u64 = Self::get_burn_as_u64(netuid);
                let current_difficulty: u64 = Self::get_difficulty_as_u64(netuid);
                let registrations_this_interval: u16 =
                    Self::get_registrations_this_interval(netuid);
                let pow_registrations_this_interval: u16 =
                    Self::get_pow_registrations_this_interval(netuid);
                let burn_registrations_this_interval: u16 =
                    Self::get_burn_registrations_this_interval(netuid);
                let target_registrations_this_interval: u16 =
                    Self::get_target_registrations_per_interval(netuid);
                // --- 5. Adjust burn + pow
                // There are six cases to consider. A, B, C, D, E, F
                if registrations_this_interval > target_registrations_this_interval {
                    if pow_registrations_this_interval > burn_registrations_this_interval {
                        // A. There are too many registrations this interval and most of them are pow registrations
                        // this triggers an increase in the pow difficulty.
                        // pow_difficulty ++
                        Self::set_difficulty(
                            netuid,
                            Self::adjust_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else if pow_registrations_this_interval < burn_registrations_this_interval {
                        // B. There are too many registrations this interval and most of them are burn registrations
                        // this triggers an increase in the burn cost.
                        // burn_cost ++
                        Self::set_burn(
                            netuid,
                            Self::adjust_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else {
                        // F. There are too many registrations this interval and the pow and burn registrations are equal
                        // this triggers an increase in the burn cost and pow difficulty
                        // burn_cost ++
                        Self::set_burn(
                            netuid,
                            Self::adjust_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                        // pow_difficulty ++
                        Self::set_difficulty(
                            netuid,
                            Self::adjust_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    }
                } else {
                    // Not enough registrations this interval.
                    if pow_registrations_this_interval > burn_registrations_this_interval {
                        // C. There are not enough registrations this interval and most of them are pow registrations
                        // this triggers a decrease in the burn cost
                        // burn_cost --
                        Self::set_burn(
                            netuid,
                            Self::adjust_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else if pow_registrations_this_interval < burn_registrations_this_interval {
                        // D. There are not enough registrations this interval and most of them are burn registrations
                        // this triggers a decrease in the pow difficulty
                        // pow_difficulty --
                        Self::set_difficulty(
                            netuid,
                            Self::adjust_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    } else {
                        // E. There are not enough registrations this interval and the pow and burn registrations are equal
                        // this triggers a decrease in the burn cost and pow difficulty
                        // burn_cost --
                        Self::set_burn(
                            netuid,
                            Self::adjust_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                        // pow_difficulty --
                        Self::set_difficulty(
                            netuid,
                            Self::adjust_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    }
                }

                // --- 6. Drain all counters for this network for this interval.
                Self::set_last_adjustment_block(netuid, current_block);
                Self::set_registrations_this_interval(netuid, 0);
                Self::set_pow_registrations_this_interval(netuid, 0);
                Self::set_burn_registrations_this_interval(netuid, 0);
            } else {
                log::debug!("interval not reached.");
            }

            // --- 7. Drain block registrations for each network. Needed for registration rate limits.
            Self::set_registrations_this_block(netuid, 0);
        }
    }

    // Performs the difficulty adjustment by multiplying the current difficulty by the ratio ( reg_actual + reg_target / reg_target * reg_target )
    // We use I110F18 to avoid any overflows on u64. Also min_difficulty and max_difficulty bound the range.
    //
    pub fn adjust_difficulty(
        netuid: u16,
        current_difficulty: u64,
        registrations_this_interval: u16,
        target_registrations_per_interval: u16,
    ) -> u64 {
        let updated_difficulty: I110F18 = I110F18::from_num(current_difficulty)
            * I110F18::from_num(registrations_this_interval + target_registrations_per_interval)
            / I110F18::from_num(
                target_registrations_per_interval + target_registrations_per_interval,
            );
        let alpha: I110F18 =
            I110F18::from_num(Self::get_adjustment_alpha(netuid)) / I110F18::from_num(u64::MAX);
        let next_value: I110F18 = alpha * I110F18::from_num(current_difficulty)
            + (I110F18::from_num(1.0) - alpha) * updated_difficulty;
        if next_value >= I110F18::from_num(Self::get_max_difficulty(netuid)) {
            return Self::get_max_difficulty(netuid);
        } else if next_value <= I110F18::from_num(Self::get_min_difficulty(netuid)) {
            return Self::get_min_difficulty(netuid);
        } else {
            return next_value.to_num::<u64>();
        }
    }

    // Performs the burn adjustment by multiplying the current difficulty by the ratio ( reg_actual + reg_target / reg_target * reg_target )
    // We use I110F18 to avoid any overflows on u64. Also min_burn and max_burn bound the range.
    //
    pub fn adjust_burn(
        netuid: u16,
        current_burn: u64,
        registrations_this_interval: u16,
        target_registrations_per_interval: u16,
    ) -> u64 {
        let updated_burn: I110F18 = I110F18::from_num(current_burn)
            * I110F18::from_num(registrations_this_interval + target_registrations_per_interval)
            / I110F18::from_num(
                target_registrations_per_interval + target_registrations_per_interval,
            );
        let alpha: I110F18 =
            I110F18::from_num(Self::get_adjustment_alpha(netuid)) / I110F18::from_num(u64::MAX);
        let next_value: I110F18 = alpha * I110F18::from_num(current_burn)
            + (I110F18::from_num(1.0) - alpha) * updated_burn;
        if next_value >= I110F18::from_num(Self::get_max_burn_as_u64(netuid)) {
            return Self::get_max_burn_as_u64(netuid);
        } else if next_value <= I110F18::from_num(Self::get_min_burn_as_u64(netuid)) {
            return Self::get_min_burn_as_u64(netuid);
        } else {
            return next_value.to_num::<u64>();
        }
    }
}
