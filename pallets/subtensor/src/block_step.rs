use crate::types::SubnetType;
use super::*;
use sp_core::Get;
use sp_std::vec::Vec;
use substrate_fixed::types::I110F18;
use substrate_fixed::types::I64F64;

struct SubnetBlockStepInfo {
    netuid: u16,
    subnet_type: SubnetType,
    price: I64F64,
    tao_staked: u64,
    transition_in_progress: bool,
}

impl<T: Config> Pallet<T> {
    /// Executes the necessary operations for each block.
    pub fn block_step() -> Result<(), &'static str> {
        let block_number: u64 = Self::get_current_block_as_u64();
        log::debug!("block_step for block: {:?} ", block_number);
        // --- 1. Adjust difficulties.
        Self::adjust_registration_terms_for_networks();
        // --- 2. Mint and distribute TAO.
        Self::run_coinbase(block_number);
        // Adjust Tempos every 1000 blocks
        if Self::blocks_until_next_epoch(0, 1000, block_number) == 0 {
            Self::adjust_tempos();
        }

        // Return ok.
        Ok(())
    }

    /// Adjusts the tempo for each network based on their relative prices to ensure operations
    /// are performed more frequently on networks with higher prices.
    ///
    /// This function calculates a value `bi` for each network, which represents the number of blocks
    /// that progress before an operation is performed on the network. Networks with higher prices
    /// will have operations performed more frequently. The average operation frequency across all
    /// networks is aimed to be every `K` blocks.
    pub fn adjust_tempos() {
        // Retrieve all network UIDs.
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();

        // Compute and collect prices for each dynamic subnet, excluding the root subnet.
        let mut prices: Vec<I64F64> = Vec::new();
        for netuid in netuids.iter() {
            if *netuid == Self::get_root_netuid() || !Self::is_subnet_dynamic(*netuid) {
                continue;
            }
            let price = Self::get_tao_per_alpha_price(*netuid);
            prices.push(price);
        }

        // Assuming `K` is a predefined constant representing the average desired operation interval in blocks.
        let k: I64F64 = I64F64::from_num(10); // Replace 1.0 with the actual value of `K` if available.

        // Calculate tempos using the extracted prices and netuids.
        match Self::calculate_tempos(&netuids, k, &prices) {
            Ok(tempos) => {
                // Set the calculated tempos for each network.
                for (netuid, tempo) in tempos.iter() {
                    Self::set_tempo(*netuid, *tempo);
                }
            }
            Err(e) => {
                log::error!("Failed to calculate tempos: {}", e);
            }
        }
    }

    /// Calculates the tempos for each network based on the given prices and a constant `K`.
    ///
    /// # Arguments
    /// * `netuids` - A reference to a vector of network UIDs.
    /// * `k` - The constant representing the average desired operation interval in blocks.
    /// * `prices` - A reference to a vector of prices for each network.
    ///
    /// # Returns
    /// * A result containing either a vector of tuples where each tuple contains a network UID and its corresponding tempo, or an error string if there's a mismatch in vector sizes or other issues.
    pub fn calculate_tempos(
        netuids: &[u16],
        k: I64F64,
        prices: &[I64F64],
    ) -> Result<Vec<(u16, u16)>, &'static str> {
        // Check for mismatched vector sizes
        if netuids.len() != prices.len() {
            return Err("Mismatched vector sizes: netuids and prices must have the same length.");
        }

        // Check for empty vectors
        if netuids.is_empty() || prices.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate total price to find relative frequencies
        let total_price: I64F64 = prices.iter().sum();
        if total_price == I64F64::from_num(0.0) {
            return Ok(netuids.iter().map(|&uid| (uid, 0)).collect()); // If sum of prices is zero, return zero tempos
        }

        // Calculate relative frequencies based on prices
        let relative_frequencies: Vec<I64F64> = prices
            .iter()
            .map(|&price| price / total_price) // relative frequency = price_i / total_price
            .collect();

        // Calculate total relative frequency to normalize it to K
        let total_relative_frequency: I64F64 = relative_frequencies.iter().sum();
        let normalization_factor: I64F64 = k / total_relative_frequency;

        // Calculate tempos based on normalized relative frequencies
        let min_tempo = T::MinTempo::get();
        let max_tempo = T::MaxTempo::get();
        let tempos: Vec<(u16, u16)> = netuids
            .iter()
            .zip(relative_frequencies.iter())
            .map(|(&uid, &rel_freq)| {
                let mut tempo = (normalization_factor / rel_freq).to_num::<u16>();
                if tempo < min_tempo {
                    tempo = min_tempo;
                }
                if tempo > max_tempo {
                    tempo = max_tempo;
                }
                (uid, tempo)
            })
            .collect();

        Ok(tempos)
    }
    // Helper function which returns the number of blocks remaining before we will run the epoch on this
    // network. Networks run their epoch when (block_number + netuid + 1 ) % (tempo + 1) = 0
    //
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
        tempo as u64 - (block_number + netuid as u64 + 1) % (tempo as u64 + 1)
    }

    pub fn get_subnet_type(netuid: u16) -> SubnetType {
        if Self::is_subnet_dynamic(netuid) { 
            SubnetType::DTAO 
        } else { 
            SubnetType::STAO 
        }
    }

    fn get_subnets() -> Vec<SubnetBlockStepInfo> {
        // Get all the network uids.
        Self::get_all_subnet_netuids().iter().map(|&netuid| {
            let dynamic = Self::is_subnet_dynamic(netuid);
            SubnetBlockStepInfo {
                netuid,
                subnet_type: Self::get_subnet_type(netuid),
                price: {
                    if netuid == Self::get_root_netuid() || !dynamic {
                        I64F64::from_num(0.0)
                    } else {
                        Self::get_tao_per_alpha_price(netuid)
                    }
                },
                tao_staked: TotalSubnetTAO::<T>::get(netuid),
                // TODOSDT: Only consider current subnet, not all (see commented below)
                transition_in_progress: SubnetInTransition::<T>::iter().next().is_some(),
                // transition_in_progress: SubnetInTransition::<T>::get(netuid).is_some(),
            }
        }).collect()
    }

    pub fn run_coinbase(block_number: u64) {
        // Compute and fill the prices from all subnets.
        let mut subnets = Self::get_subnets();
        let total_prices: I64F64 = subnets.iter().map(|subnet_info| subnet_info.price).sum();

        // Compute total TAO staked across all subnets
        let total_tao_staked: u64 = subnets.iter()
            .filter(|subnet| subnet.netuid != Self::get_root_netuid())
            .map(|subnet_info| subnet_info.tao_staked).sum();

        // Compute emission per subnet as [p.tao_in/sum_tao for p in pools]
        let total_block_emission = Self::get_block_emission().unwrap_or(0);
        let total_block_emission_i64f64: I64F64 = I64F64::from_num(total_block_emission);
        let mut actual_total_block_emission = 0u64;

        if total_tao_staked != 0 {
            subnets.iter_mut().for_each(|subnet_info| {
                if !subnet_info.transition_in_progress {
                    let subnet_proportion: I64F64 = if subnet_info.netuid == Self::get_root_netuid() {
                        I64F64::from_num(0)
                    } else {
                        I64F64::from_num(subnet_info.tao_staked) / I64F64::from_num(total_tao_staked)
                    };
                    let emission_i64f64 = total_block_emission_i64f64 * subnet_proportion;
                    let subnet_block_emission = emission_i64f64.to_num();
                    EmissionValues::<T>::insert(subnet_info.netuid, subnet_block_emission);
                    // Increment the amount of TAO that is waiting to be distributed through Yuma Consensus.
                    PendingEmission::<T>::mutate(subnet_info.netuid, |emission| *emission += subnet_block_emission);

                    match subnet_info.subnet_type {
                        SubnetType::DTAO => {
                            // Condition the inflation of TAO and alpha based on the sum of the prices.
                            // This keeps the market caps of ALPHA subsumed by TAO.
                            let tao_in: u64; // The total amount of TAO emitted this block into all pools.
                            let alpha_in: u64; // The amount of ALPHA emitted this block into each pool.
                            if total_prices <= I64F64::from_num(1.0) {
                                // Alpha prices are lower than 1.0, emit TAO and not ALPHA into the pools.
                                tao_in = subnet_block_emission;
                                alpha_in = 0;
                            } else {
                                // Alpha prices are greater than 1.0, emit ALPHA and not TAO into the pools.
                                tao_in = 0;
                                alpha_in = subnet_block_emission; // 10^9 rao
                            }

                            if tao_in > 0 {
                                // Increment total TAO on subnet
                                TotalSubnetTAO::<T>::mutate(subnet_info.netuid, |stake| *stake = stake.saturating_add(tao_in));

                                // Increment the pools tao reserve based on the block emission.
                                DynamicTAOReserve::<T>::mutate(subnet_info.netuid, |reserve| *reserve += tao_in);

                                actual_total_block_emission = actual_total_block_emission.saturating_add(tao_in);
                            }

                            if alpha_in > 0 {
                                // Increment the pools alpha reserve based on the alpha in emission.
                                DynamicAlphaReserve::<T>::mutate(subnet_info.netuid, |reserve| *reserve += alpha_in);

                                // Increment the total supply of alpha because we just added some to the reserve.
                                DynamicAlphaIssuance::<T>::mutate(subnet_info.netuid, |issuance| *issuance += alpha_in);
                            }
            
                            // Recalculate the Dynamic K value for the new pool.
                            DynamicK::<T>::insert(
                                subnet_info.netuid,
                                (DynamicTAOReserve::<T>::get(subnet_info.netuid) as u128)
                                    * (DynamicAlphaReserve::<T>::get(subnet_info.netuid) as u128),
                            );
                        },
                        SubnetType::STAO => {
                            if subnet_block_emission != 0 {
                                TotalSubnetTAO::<T>::mutate(subnet_info.netuid, |stake| *stake = stake.saturating_add(subnet_block_emission));
                                actual_total_block_emission = actual_total_block_emission.saturating_add(subnet_block_emission);
                            }
                        }
                    }
                }
            });

            // Increment the total amount of TAO in existence based on the total tao_in
            TotalIssuance::<T>::mutate(|issuance| *issuance = issuance.saturating_add(actual_total_block_emission));
    
            ////////////////////////////////
            // run epochs.
            subnets.iter_mut().for_each(|subnet_info| {
                // Check to see if this network has reached tempo.
                let tempo: u16 = Self::get_tempo(subnet_info.netuid);
                if Self::blocks_until_next_epoch(subnet_info.netuid, tempo, block_number) == 0 {
                    // Get the pending emission issuance to distribute for this subnet
                    let emission = PendingEmission::<T>::get(subnet_info.netuid);
                    // Drain pending emission and update dynamic pools
                    PendingEmission::<T>::insert(subnet_info.netuid, 0);
    
                    // Run the epoch mechanism and return emission tuples for hotkeys in the network in alpha.
                    let emission_tuples: Vec<(T::AccountId, u64, u64)> =
                        Self::epoch(subnet_info.netuid, emission);
    
                    // Emit the tuples through the hotkeys incrementing their alpha staking balance for this subnet
                    // as well as all nominators.
                    for (hotkey, server_amount, validator_amount) in emission_tuples.iter() {
                        Self::emit_inflation_through_hotkey_account(
                            hotkey,
                            subnet_info.netuid,
                            *server_amount,
                            *validator_amount,
                        );
                    }

                    // Increase subnet totals
                    match subnet_info.subnet_type {
                        SubnetType::DTAO => {
                            // Increment the total amount of alpha outstanding (the amount on all of the staking accounts)
                            DynamicAlphaOutstanding::<T>::mutate(subnet_info.netuid, |reserve| *reserve += emission);
                            // Also increment the total amount of alpha in total everywhere.
                            DynamicAlphaIssuance::<T>::mutate(subnet_info.netuid, |issuance| *issuance += emission);
                        },
                        SubnetType::STAO => {},
                    }
    
                    // Some other counters for accounting.
                    Self::set_blocks_since_last_step(subnet_info.netuid, 0);
                    Self::set_last_mechanism_step_block(subnet_info.netuid, block_number);
                } else {
                    Self::set_blocks_since_last_step(
                        subnet_info.netuid,
                        Self::get_blocks_since_last_step(subnet_info.netuid) + 1,
                    );
                }
            });
        }
    }

    // Distributes token inflation through the hotkey based on emission. The call ensures that the inflation
    // is distributed onto the accounts in proportion of the stake delegated minus the take. This function
    // is called after an epoch to distribute the newly minted stake according to delegation.
    //
    // Algorithm:
    //   0. Hotkey always receives server_emission completely.
    //   1. If a hotkey is a not delegate, it gets 100% of both server and validator emission. STOP.
    //   2. Delegate gets it's take, i.e. a percentage of validator_emission specific to a given subnet (netuid)
    //
    //   remaining_validator_emission is what's left. Here is how it's distributed:
    //
    //   3. If either delegate_local_stake (total amount of stake under a hotkey for a subnet) or
    //      delegate_global_dynamic_tao (total delegate stake * alpha_price) are non-zero, then
    //      for each nominator nominating this delegate do:
    //      3.a Nominator reward comes in two parts: Local and Global
    //          Local = (1 - global_stake_weight) * remaining_validator_emission
    //                  (nominator Alpha in this subnet for hotkey) / (sum of all Alpha in this subnet for hotkey)
    //          Global = global_stake_weight * remaining_validator_emission * (sum of nominator stake across all subnets) /
    //                   (sum of everybody's stake across all subnets)
    //          Global Stake Weight effectively is always 1 currently, so there is no local emission, but no matter what's
    //          the ratio is set in the future, the sum of all rewards is always going to be remaining_validator_emission.
    //
    // Questions/Comments:
    //   1. Can tao_per_alpha_price be zero if get_total_stake_for_hotkey_and_subnet is non-zero?
    //   2. TODO: Add tests for how DynamicTAOReserve and DynamicAlphaReserve are affected by staking operations
    //   3. Is it theoretically possible that lock cost gets up to about 18M TAO for a single network? Will
    //      it not overflow initial_dynamic_reserve?
    //   4. Should residual after step 3 be non-zero in any case?
    //   5. This algorithm re-purposes TotalHotkeySubStake and SubStake state variables to store Alpha (vs. TAO).
    //
    pub fn emit_inflation_through_hotkey_account(
        delegate: &T::AccountId,
        netuid: u16,
        server_emission: u64,
        validator_emission: u64,
    ) {
        // 1. Check if the hotkey is not a delegate and thus the emission is entirely owed to them.
        if !Self::hotkey_is_delegate(delegate) {
            let total_delegate_emission: u64 = server_emission + validator_emission;
            Self::increase_subnet_token_on_hotkey_account(delegate, netuid, total_delegate_emission);
            let coldkey: T::AccountId = Self::get_owning_coldkey_for_hotkey(delegate);
            let tao_server_emission: u64 = Self::compute_dynamic_unstake(netuid, server_emission);
            Self::add_balance_to_coldkey_account(
                &coldkey,
                tao_server_emission,
            );
            return;
        }
        // 2. Else the key is a delegate, first compute the delegate take from the emission.
        let take_proportion: I64F64 = I64F64::from_num(DelegatesTake::<T>::get(delegate, netuid))
            / I64F64::from_num(u16::MAX);
        let delegate_take: I64F64 = take_proportion * I64F64::from_num(validator_emission);
        let delegate_take_u64: u64 = delegate_take.to_num::<u64>();
        let remaining_validator_emission: u64 = validator_emission - delegate_take_u64;
        let mut residual: u64 = remaining_validator_emission;

        // 3. For each nominator compute its proportion of stake weight and distribute the remaining emission to them.
        let global_stake_weight: I64F64 = Self::get_global_stake_weight_float();
        let delegate_local_stake: u64 =
            Self::get_total_stake_for_hotkey_and_subnet(delegate, netuid);
        let delegate_global_dynamic_tao = Self::get_hotkey_global_dynamic_tao(delegate);
        log::debug!(
            "global_stake_weight: {:?}, delegate_local_stake: {:?}, delegate_global_stake: {:?}",
            global_stake_weight,
            delegate_local_stake,
            delegate_global_dynamic_tao
        );

        if delegate_local_stake + delegate_global_dynamic_tao != 0 {
            Staker::<T>::iter_prefix(delegate)
                .for_each(|(nominator_i, _)| {
                    // 3.a Compute the stake weight percentage for the nominatore weight.
                    let nominator_local_stake: u64 = Self::get_subnet_stake_for_coldkey_and_hotkey(
                        &nominator_i,
                        delegate,
                        netuid,
                    );
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
                        Self::get_nominator_global_dynamic_tao(&nominator_i, delegate); // Get global stake.
                    let nominator_global_emission_i: I64F64 = if delegate_global_dynamic_tao == 0 {
                        I64F64::from_num(0)
                    } else {
                        let nominator_global_percentage: I64F64 =
                            I64F64::from_num(nominator_global_stake)
                                / I64F64::from_num(delegate_global_dynamic_tao);
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
                    Self::increase_subnet_token_on_coldkey_hotkey_account(
                        &nominator_i,
                        delegate,
                        netuid,
                        nominator_emission_u64,
                    );
                });
        }

        // --- 4. Last increase final account balance of delegate after 4, since 5 will change the stake proportion of
        // the delegate and effect calculation in 4.
        let total_delegate_emission: u64 = delegate_take_u64 + residual;
        log::debug!(
            "total_delegate_emission: {:?}",
            delegate_take_u64 + server_emission
        );
        Self::increase_subnet_token_on_hotkey_account(delegate, netuid, total_delegate_emission);
        let coldkey: T::AccountId = Self::get_owning_coldkey_for_hotkey(delegate);
        let tao_server_emission: u64 = Self::compute_dynamic_unstake(netuid, server_emission);
        Self::add_balance_to_coldkey_account(
            &coldkey,
            tao_server_emission,
        );
    }

    /// Returns emission awarded to a hotkey as a function of its proportion of the total stake.
    ///
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
        proportional_emission.to_num::<u64>()
    }

    // Returns the delegated stake 'take' assigned to this key. (If exists, otherwise 0)
    //
    pub fn calculate_delegate_proportional_take(
        hotkey: &T::AccountId,
        netuid: u16,
        emission: u64,
    ) -> u64 {
        if Self::hotkey_is_delegate(hotkey) {
            let take_proportion: I64F64 = I64F64::from_num(DelegatesTake::<T>::get(hotkey, netuid))
                / I64F64::from_num(u16::MAX);
            let take_emission: I64F64 = take_proportion * I64F64::from_num(emission);
            take_emission.to_num::<u64>()
        } else {
            0
        }
    }

    /// Adjusts the network difficulties/burns of every active network. Resetting state parameters.
    ///
    pub fn adjust_registration_terms_for_networks() {
        log::debug!("adjust_registration_terms_for_networks");

        // --- 1. Iterate through each network.
        for (netuid, _) in NetworksAdded::<T>::iter() {
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
                    #[allow(clippy::comparison_chain)]
                    if pow_registrations_this_interval > burn_registrations_this_interval {
                        // A. There are too many registrations this interval and most of them are pow registrations
                        // this triggers an increase in the pow difficulty.
                        // pow_difficulty ++
                        Self::set_difficulty(
                            netuid,
                            Self::upgraded_difficulty(
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
                            Self::upgraded_burn(
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
                            Self::upgraded_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                        // pow_difficulty ++
                        Self::set_difficulty(
                            netuid,
                            Self::upgraded_difficulty(
                                netuid,
                                current_difficulty,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                    }
                } else {
                    // Not enough registrations this interval.
                    #[allow(clippy::comparison_chain)]
                    if pow_registrations_this_interval > burn_registrations_this_interval {
                        // C. There are not enough registrations this interval and most of them are pow registrations
                        // this triggers a decrease in the burn cost
                        // burn_cost --
                        Self::set_burn(
                            netuid,
                            Self::upgraded_burn(
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
                            Self::upgraded_difficulty(
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
                            Self::upgraded_burn(
                                netuid,
                                current_burn,
                                registrations_this_interval,
                                target_registrations_this_interval,
                            ),
                        );
                        // pow_difficulty --
                        Self::set_difficulty(
                            netuid,
                            Self::upgraded_difficulty(
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

    /// Calculates the upgraded difficulty by multiplying the current difficulty by the ratio ( reg_actual + reg_target / reg_target + reg_target )
    /// We use I110F18 to avoid any overflows on u64. Also min_difficulty and max_difficulty bound the range.
    ///
    pub fn upgraded_difficulty(
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
            Self::get_max_difficulty(netuid)
        } else if next_value <= I110F18::from_num(Self::get_min_difficulty(netuid)) {
            return Self::get_min_difficulty(netuid);
        } else {
            return next_value.to_num::<u64>();
        }
    }

    /// Calculates the upgraded burn by multiplying the current burn by the ratio ( reg_actual + reg_target / reg_target + reg_target )
    /// We use I110F18 to avoid any overflows on u64. Also min_burn and max_burn bound the range.
    ///
    pub fn upgraded_burn(
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
            Self::get_max_burn_as_u64(netuid)
        } else if next_value <= I110F18::from_num(Self::get_min_burn_as_u64(netuid)) {
            return Self::get_min_burn_as_u64(netuid);
        } else {
            return next_value.to_num::<u64>();
        }
    }
}
