use super::*;
use substrate_fixed::types::I96F32;
use alloc::collections::BTreeMap;

impl<T: Config> Pallet<T> {

        
    /// The `coinbase` function performs a four-part emission distribution process involving
    /// subnets, epochs, hotkeys, and nominators.
    // It is divided into several steps, each handling a specific part of the distribution:

    // Step 1: Compute the block-wise emission for each subnet.
    // This involves calculating how much (TAO) should be emitted into each subnet using the
    // root epoch function.

    // Step 2: Accumulate the subnet block emission.
    // After calculating the block-wise emission, these values are accumulated to keep track
    // of how much each subnet should emit before the next distribution phase. This accumulation
    // is a running total that gets updated each block.

    // Step 3: Distribute the accumulated emissions through epochs.
    // Subnets periodically distribute their accumulated emissions to hotkeys (active validators/miners)
    // in the network on a `tempo` --- the time between epochs. This step runs Yuma consensus to
    // determine how emissions are split among hotkeys based on their contributions and roles.
    // The accumulation of hotkey emissions is done through the `accumulate_hotkey_emission` function.
    // The function splits the rewards for a hotkey amongst itself and its `parents`. The parents are
    // the hotkeys that are delegating their stake to the hotkey.

    // Step 4: Further distribute emissions from hotkeys to nominators.
    // Finally, the emissions received by hotkeys are further distributed to their nominators,
    // who are stakeholders that support the hotkeys.
    pub fn run_coinbase() {
        // --- 0. Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!("Current block: {:?}", current_block);

        // --- 1. Get all netuids.
        let subnets: Vec<u16> = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", subnets);

        // --- 2. Get the current coinbase emission.
        let block_emission: I96F32 = I96F32::from_num( Self::get_block_emission().unwrap_or(0) );
        log::debug!("Block emission: {:?}", block_emission);

        // --- 3. Total subnet TAO.
        let total_subnet_tao: I96F32 = subnets.iter().fold(I96F32::from_num(0), |acc, netuid| {
            acc + I96F32::from_num(SubnetTAO::<T>::get(*netuid))
        });
        log::debug!("Total subnet TAO: {:?}", total_subnet_tao);

        // --- 4. Compute EmissionValues per subnet.
        // This loop calculates the emission for each subnet based on its mechanism and proportion of TAO.
        // For each subnet s in a mechanism m:
        // 1. Calculate subnet's proportion of mechanism TAO: P_s = T_s / T_m
        // 2. Calculate subnet's TAO emission: E_s = P_s * E_m
        // 3. Convert TAO emission to alpha emission: E_α = tao_to_alpha(E_s)
        // 4. Update total issuance: I_new = I_old + E_s
        // 5. Update subnet TAO: T_s_new = T_s_old + E_s
        // 6. Update subnet alpha: A_s_new = A_s_old + E_α
        // 7. Accumulate pending emission: P_e_new = P_e_old + E_α
        for netuid in subnets.clone().iter() {
            // 1. Get subnet mechanism ID
            let mechid: u16 = SubnetMechanism::<T>::get(*netuid);
            // 4. Get subnet TAO (T_s)
            let subnet_tao: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(*netuid));
            // 5. Calculate subnet's proportion of mechanism TAO: P_s = T_s / T_m
            let subnet_proportion: I96F32 = subnet_tao.checked_div(total_subnet_tao).unwrap_or(I96F32::from_num(0));
            // 6. Calculate subnet's TAO emission: E_s = P_s * E_m
            let tao_emission: u64 = subnet_proportion.checked_mul(block_emission).unwrap_or(I96F32::from_num(0)).to_num::<u64>();
            // 7. Store the block emission for this subnet
            EmissionValues::<T>::insert(*netuid, tao_emission);
            // Add the TAO into the subnet immediatetly.
            SubnetTAO::<T>::mutate(*netuid, |total| { *total = total.saturating_add(tao_emission) });
            // Switch on dynamic or Stable.
            if mechid == 1 {
                // Dynamic.
                // Add the SubnetAlpha directly into the pool immediately.
                SubnetAlphaIn::<T>::mutate(*netuid, |total| { *total = total.saturating_add(block_emission.to_num::<u64>())});
                // Set the pending emission directly as alpha always block emission total
                PendingEmission::<T>::mutate(netuid, |total| { *total = total.saturating_add(block_emission.to_num::<u64>())});
            } else {
                // Set the pending emission as tao emission.
                PendingEmission::<T>::mutate(netuid, |total| { *total = total.saturating_add(tao_emission)});
            }
        }
        log::debug!("Emission per subnet: {:?}", EmissionValues::<T>::iter().collect::<Vec<_>>());
        log::debug!("Pending Emission per subnet: {:?}", PendingEmission::<T>::iter().collect::<Vec<_>>());

        // --- 5. Drain the accumulated subnet emissions, pass them through the epoch().
        // Before accumulating on the hotkeys the function redistributes the emission towards hotkey parents.
        // subnet_emission --> epoch() --> hotkey_emission --> (hotkey + parent hotkeys)
        let mut hotkey_emission_limit: u64 = 0;
        let mut hotkey_emission_tuples: Vec<(T::AccountId, u16, u64)> = vec![];
        for netuid in subnets.clone().iter() {
            // --- 5.1 Check to see if the subnet should run its epoch.
            if Self::should_run_epoch(*netuid, current_block) {
                // --- 5.2 Drain the subnet emission.
                let subnet_emission: u64 = PendingEmission::<T>::get(*netuid);
                PendingEmission::<T>::insert(*netuid, 0);
                log::debug!(
                    "Drained subnet emission for netuid {:?}: {:?}",
                    *netuid,
                    subnet_emission
                );

                // --- 5.3 Set last step counter.
                Self::set_blocks_since_last_step(*netuid, 0);
                Self::set_last_mechanism_step_block(*netuid, current_block);

                // 5.3 Give 18% cut to the owner immediately as a lock.
                let owner_hotkey: T::AccountId = SubnetOwnerHotkey::<T>::get(*netuid);
                let owner_coldkey: T::AccountId = SubnetOwner::<T>::get(*netuid);
                let owner_cut: u64 = subnet_emission.saturating_mul(18).saturating_div(100);
                Self::emit_into_subnet(&owner_hotkey, &owner_coldkey, *netuid, owner_cut);

                // Decrement the emission by the owner cut.
                let remaining_emission: u64 = subnet_emission.saturating_sub(owner_cut);

                // 5.3 Pass emission through epoch() --> hotkey emission.
                let mut hotkey_emission: Vec<(T::AccountId, u64, u64)> = Self::epoch(*netuid, remaining_emission);
                log::debug!(
                    "Hotkey emission results for netuid {:?}: {:?}",
                    *netuid,
                    hotkey_emission
                );

                // 5.4 Accumulate the tuples on hotkeys:
                for (hotkey, mining_emission, validator_emission) in hotkey_emission {
                    // Distribute the emission on the hotkey and parent hotkeys appending new vectors to hotkey_emission_tuples.
                    Self::source_hotkey_emission(
                        &hotkey,
                        *netuid,
                        validator_emission, // Amount received from validating
                        mining_emission,    // Amount recieved from mining.
                        &mut hotkey_emission_tuples
                    );
                    hotkey_emission_limit += validator_emission + mining_emission;
                    log::debug!("Accumulated emissions on hotkey {:?} for netuid {:?}: mining {:?}, validator {:?}", hotkey, *netuid, mining_emission, validator_emission);
                }
            } else {
                log::debug!("Tempo not reached for subnet: {:?}", *netuid);
            }
        }

        // Finally apply the emission tuples;
        log::debug!("Hotkey Emission tuples: {:?}", hotkey_emission_tuples);
        let total_hotkey_emitted: u64 = hotkey_emission_tuples.iter().map(|(_, _, amount)| amount).sum();
        assert!(total_hotkey_emitted <= hotkey_emission_limit, "total_hotkey_emitted: ({}) > hotkey_emission_limit: ({})", total_hotkey_emitted, hotkey_emission_limit);
        Self::accumulate_hotkey_emission( &mut hotkey_emission_tuples );

        // --- 6. Drain the accumulated hotkey emissions through to the nominators.
        // The hotkey takes a proportion of the emission, the remainder is drained through to the nominators.
        // We keep track of the last stake increase event for accounting purposes.
        // hotkeys --> nominators.
        let mut nominator_emission_limit: u64 = 0;
        let mut nominator_emission: Vec<(T::AccountId, T::AccountId, u16, u64)> = vec![];
        let emission_tempo: u64 = Self::get_hotkey_emission_tempo();
        for (hotkey, netuid_i, hotkey_emission) in PendingdHotkeyEmissionOnNetuid::<T>::iter() {
            if Self::should_drain_hotkey(&hotkey, current_block, emission_tempo) {
                log::debug!("Draining hotkey {:?} on netuid {:?} on block {:?}: {:?}", hotkey, netuid_i, current_block, hotkey_emission);
                // Remove the hotkey emission from the pending emissions.
                PendingdHotkeyEmissionOnNetuid::<T>::remove( &hotkey, netuid_i );
                // Drain the hotkey emission.
                Self::source_nominator_emission( &hotkey, netuid_i, hotkey_emission, current_block, &mut nominator_emission );
                nominator_emission_limit += hotkey_emission;
            }
        }
        // Update drain blocks.
        for (hotkey, _, _) in PendingdHotkeyEmissionOnNetuid::<T>::iter() {
            if Self::should_drain_hotkey(&hotkey, current_block, emission_tempo) {
                LastHotkeyEmissionDrain::<T>::insert(hotkey, current_block);
            }
        }
        // Finally apply the emission tuples;
        log::debug!("Emission tuples: {:?}", nominator_emission);
        let total_nominator_emitted: u64 = nominator_emission.iter().map(|(_, _, _, amount)| amount).sum();
        assert!(total_nominator_emitted <= nominator_emission_limit, "total_nominator_emitted: ({}) > emission_limit: ({})", total_nominator_emitted, nominator_emission_limit);
        Self::accumulate_nominator_emission( &mut nominator_emission );
    }

    /// Accumulates the mining and validator emissions on a hotkey and distributes the validator emission among its parents.
    ///
    /// This function is responsible for accumulating the mining and validator emissions associated with a hotkey onto a hotkey.
    /// It first calculates the total stake of the hotkey, considering the stakes contributed by its parents and reduced by its children.
    /// It then retrieves the list of parents of the hotkey and distributes the validator emission proportionally based on the stake contributed by each parent.
    /// The remaining validator emission, after distribution to the parents, along with the mining emission, is then added to the hotkey's own accumulated emission.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey for which emissions are being calculated.
    /// * `netuid` - The unique identifier of the network to which the hotkey belongs.
    /// * `mining_emission` - The amount of mining emission allocated to the hotkey.
    /// * `validator_emission` - The amount of validator emission allocated to the hotkey.
    ///
    pub fn source_hotkey_emission(
        hotkey: &T::AccountId,
        netuid: u16,
        validating_emission: u64,
        mining_emission: u64,
        hotkey_emission_tuples: &mut Vec<(T::AccountId, u16, u64)>,
    ) {
        // Append the take emission tuple.
        let validating_emission: I96F32 = I96F32::from_num(validating_emission);
        let take_proportion: I96F32 = I96F32::from_num(Delegates::<T>::get(hotkey)).saturating_div(I96F32::from_num(u16::MAX));
        let hotkey_take: I96F32 = take_proportion * validating_emission;

        // Distribute to parents.
        let dynamic_weight: I96F32 = Self::get_dynamic_weight();
        let alpha_weight: I96F32 = I96F32::from_num( 1.0 ) - dynamic_weight;

        let mut remainder: I96F32 = validating_emission - hotkey_take;
        let parnet_emission: I96F32 = validating_emission - hotkey_take;
        let hotkey_alpha: I96F32 = I96F32::from_num( Self::get_stake_for_hotkey_on_subnet( hotkey, netuid ) );
        let hotkey_dynamic: I96F32 = I96F32::from_num( Self::get_global_for_hotkey( hotkey ) );

        // Iterate over parents.
        for (proportion, parent) in Self::get_parents(hotkey, netuid) {
            // Proportion from parent.
            let parent_proportion: I96F32 = I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));
            let parent_dynamic: I96F32 = I96F32::from_num( Self::get_global_for_hotkey( &parent ) );
            let parent_alpha: I96F32 = I96F32::from_num( Self::get_stake_for_hotkey_on_subnet( &parent, netuid ) );

            // Compute dynamic proportion due.
            let parent_alpha_emission: I96F32 = alpha_weight * parnet_emission * (parent_alpha * parent_proportion).checked_div(hotkey_alpha).unwrap_or(I96F32::from_num(0.0));
            let parent_dynamic_emission: I96F32 = dynamic_weight * parnet_emission * (parent_dynamic * parent_proportion).checked_div(hotkey_dynamic).unwrap_or(I96F32::from_num(0.0));
            let total_parent_emission: I96F32 = parent_alpha_emission + parent_dynamic_emission;
            hotkey_emission_tuples.push((parent, netuid, total_parent_emission.to_num::<u64>()));

            // Decrement the remainder.
            remainder -= total_parent_emission;
        }

        // Append the hotkey take, remainder and mining emission
        hotkey_emission_tuples.push((hotkey.clone(), netuid, hotkey_take.to_num::<u64>() + remainder.to_num::<u64>() + mining_emission ));
    }

    //. --- 4. Drains the accumulated hotkey emission through to the nominators. The hotkey takes a proportion of the emission.
    /// The remainder is drained through to the nominators keeping track of the last stake increase event to ensure that the hotkey does not
    /// gain more emission than it's stake since the last drain.
    /// hotkeys --> nominators.
    ///
    /// 1. It resets the accumulated emissions for the hotkey to zero.
    /// 4. It calculates the total stake for the hotkey and determines the hotkey's own take from the emissions based on its delegation status.
    /// 5. It then calculates the remaining emissions after the hotkey's take and distributes this remaining amount proportionally among the hotkey's nominators.
    /// 6. Each nominator's share of the emissions is added to their stake, but only if their stake was not manually increased since the last emission drain.
    /// 7. Finally, the hotkey's own take and any undistributed emissions are added to the hotkey's total stake.
    ///
    /// This function ensures that emissions are fairly distributed according to stake proportions and delegation agreements, and it updates the necessary records to reflect these changes.    
    pub fn source_nominator_emission(
        hotkey: &T::AccountId, 
        netuid: u16,
        emission: u64, 
        _block_number: u64,
        emission_tuples: &mut Vec<(T::AccountId, T::AccountId, u16, u64)>,
    ) {
        // Append the hotkey take here.
        let emission: I96F32 = I96F32::from_num(emission);
        let take_proportion: I96F32 = I96F32::from_num(Delegates::<T>::get(hotkey)).saturating_div(I96F32::from_num(u16::MAX));
        let hotkey_take: I96F32 = take_proportion * emission;
        
        // Distribute the remainder to nominators.
        let dynamic_weight: I96F32 = Self::get_dynamic_weight();
        let alpha_weight: I96F32 = I96F32::from_num( 1.0 ) - dynamic_weight;

        let mut remainder: I96F32 = emission - hotkey_take;
        let nominator_emission: I96F32 = emission - hotkey_take;
        let hotkey_dynamic: I96F32 = I96F32::from_num( Self::get_global_for_hotkey( hotkey ) );
        let hotkey_alpha: I96F32 = I96F32::from_num( Self::get_stake_for_hotkey_on_subnet( hotkey, netuid ) );

        // Iterate over all nominators to this hotkey.
        for (nominator, _) in Stake::<T>::iter_prefix(hotkey) {
            // Get the nominator alpha and dynamic.
            let nominator_alpha: I96F32 = I96F32::from_num( Alpha::<T>::get( (&hotkey, nominator.clone(), netuid) ));
            let nominator_dynamic: I96F32 = I96F32::from_num( Self::get_global_for_hotkey_and_coldkey( hotkey, &nominator ) );  

            // Compute contributions to nominators and alpha holders.
            let nominator_emission_from_alpha: I96F32 = alpha_weight * nominator_emission * ( nominator_alpha.checked_div(hotkey_alpha).unwrap_or(I96F32::from_num(0)) );
            let nominator_emission_from_dynamic: I96F32 = dynamic_weight * nominator_emission * ( nominator_dynamic.checked_div(hotkey_dynamic).unwrap_or(I96F32::from_num(0)) ) ;

            // Append the emission tuple.
            let nominator_emission_total: I96F32 = nominator_emission_from_alpha + nominator_emission_from_dynamic;
            emission_tuples.push( (hotkey.clone(), nominator.clone(), netuid, nominator_emission_total.to_num::<u64>()) );

            // Decrement remainder.
            remainder -= nominator_emission_total;
        }

        // Distribute the remainder and the hotkey take to the hotkey.
        let hotkey_owner: T::AccountId = Owner::<T>::get(hotkey);
        emission_tuples.push( (hotkey.clone(), hotkey_owner.clone(), netuid, (hotkey_take + remainder).to_num::<u64>()) );
    }
    /// Accumulates emissions for hotkeys across different subnets.
    ///
    /// This function takes a vector of tuples, each containing a hotkey account ID,
    /// a subnet ID (netuid), and an emission value. It updates the pending emission
    /// for each hotkey on the specified subnet by adding the given emission value.
    ///
    /// # Arguments
    ///
    /// * `hotkey_tuples` - A mutable reference to a vector of tuples, each containing:
    ///   - `T::AccountId`: The account ID of the hotkey
    ///   - `u16`: The subnet ID (netuid)
    ///   - `u64`: The emission value to be added
    pub fn accumulate_hotkey_emission( hotkey_tuples: &mut Vec<(T::AccountId, u16, u64)> ) {
        for (hotkey, netuid, emission) in hotkey_tuples {
            PendingdHotkeyEmissionOnNetuid::<T>::mutate(hotkey, *netuid, |pending_emission| {
                *pending_emission = pending_emission.saturating_add(*emission);
            });
        }
    }

    /// Accumulates emissions for nominators and updates total hotkey alpha.
    ///
    /// This function processes a vector of tuples containing nominator emission data.
    /// It updates two storage items:
    /// 1. The total alpha for each hotkey on a specific subnet.
    /// 2. The individual alpha for each nominator (coldkey) associated with a hotkey on a subnet.
    ///
    /// # Arguments
    ///
    /// * `nominator_tuples` - A mutable reference to a vector of tuples, each containing:
    ///   - `T::AccountId`: The account ID of the hotkey
    ///   - `T::AccountId`: The account ID of the coldkey (nominator)
    ///   - `u16`: The subnet ID (netuid)
    ///   - `u64`: The emission value to be added
    pub fn accumulate_nominator_emission( nominator_tuples: &mut Vec<(T::AccountId, T::AccountId, u16, u64)> ) {
        for (hotkey, coldkey, netuid, emission) in nominator_tuples {
            Self::emit_into_subnet( hotkey, coldkey, *netuid, *emission );
        }
    }

    ///////////////
    /// Helpers ///
    ///////////////
    /// Determines whether the hotkey emission should be drained based on the current block and index.
    ///
    /// # Arguments
    /// * `hotkey_i` - The hotkey identifier.
    /// * `index` - The index of the hotkey in the iterable storage.
    /// * `block` - The current block number.
    ///
    /// # Returns
    /// * `bool` - True if the hotkey emission should be drained, false otherwise.
    pub fn should_drain_hotkey(hotkey: &T::AccountId, block: u64, emit_tempo: u64) -> bool {
        let hotkey_idx: u64 = Self::hash_hotkey_to_u64(hotkey);
        block.rem_euclid(emit_tempo.saturating_add(1))
            == hotkey_idx.rem_euclid(emit_tempo.saturating_add(1))
    }

    /// Checks if the epoch should run for a given subnet based on the current block.
    ///
    /// # Arguments
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `bool` - True if the epoch should run, false otherwise.
    pub fn should_run_epoch(netuid: u16, current_block: u64) -> bool {
        Self::blocks_until_next_epoch(netuid, Self::get_tempo(netuid), current_block) == 0
    }

    /// Helper function which returns the number of blocks remaining before we will run the epoch on this
    /// network. Networks run their epoch when (block_number + netuid + 1 ) % (tempo + 1) = 0
    /// tempo | netuid | # first epoch block
    ///   1        0               0
    ///   1        1               1
    ///   2        0               1
    ///   2        1               0
    ///   100      0              99
    ///   100      1              98
    /// Special case: tempo = 0, the network never runs.
    ///
    pub fn blocks_until_next_epoch(netuid: u16, tempo: u16, block_number: u64) -> u64 {
        if tempo == 0 {
            return u64::MAX;
        }
        let netuid_plus_one = (netuid as u64).saturating_add(1);
        let block_plus_netuid = block_number.saturating_add(netuid_plus_one);
        let tempo_plus_one = (tempo as u64).saturating_add(1);
        let remainder = block_plus_netuid.rem_euclid(tempo_plus_one);
        (tempo as u64).saturating_sub(remainder)
    }

    /// Returns the emission value for the given subnet.
    ///
    /// This function retrieves the emission value for the given subnet.
    ///
    /// # Returns:
    /// * 'u64': The emission value for the given subnet.
    ///
    pub fn get_subnet_emission_value(netuid: u16) -> u64 {
        EmissionValues::<T>::get(netuid)
    }

}
