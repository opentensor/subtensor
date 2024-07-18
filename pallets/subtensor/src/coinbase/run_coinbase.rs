use super::*;
use substrate_fixed::types::I64F64;
use substrate_fixed::types::I96F32;
use alloc::collections::BTreeMap;

impl<T: Config> Pallet<T> {

    pub fn get_total_mechanism_tao(mechid: u16) -> u64 {
        let mut total_mechanism_tao: u64 = 0;
        for netuid in Self::get_all_subnet_netuids().iter() {
            let other_mechid: u16 = SubnetMechanism::<T>::get( *netuid );
            if mechid == other_mechid {
                let subnet_tao: u64 = SubnetTAO::<T>::get( *netuid );
                total_mechanism_tao += subnet_tao;
            }
        }
        total_mechanism_tao
    }
        
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

        // -- 2. Count tao per mechanism
        let mut tao_per_mechanism: BTreeMap<u16, u64> = BTreeMap::new();
        for netuid in subnets.clone().iter() {
            let mechid: u16 = SubnetMechanism::<T>::get( *netuid );
            let subnet_tao: u64 = SubnetTAO::<T>::get( *netuid );
            *tao_per_mechanism.entry(mechid).or_insert(0) += subnet_tao;
        }
        log::debug!("TAO per mechanism: {:?}", tao_per_mechanism);

        // --- 3. Compute emission per mechanism.
        let total_tao_on_mechanisms: u64 = tao_per_mechanism.values().sum();
        let mut emission_per_mechanism: BTreeMap<u16, u64> = BTreeMap::new();
        for (mechid, total_mechanism_tao) in tao_per_mechanism.iter() {
            let mechanism_emission: I96F32 = I96F32::from_num( *total_mechanism_tao ).checked_div( I96F32::from_num( total_tao_on_mechanisms ) ).unwrap_or(I96F32::from_num(0));
            emission_per_mechanism.insert(*mechid, mechanism_emission.to_num::<u64>());
        }
        log::debug!("Emission per mechanism: {:?}", emission_per_mechanism);

        // --- 4. Compute EmissionValues per subnet.
        // Iterate over mechanisms.
        for netuid in subnets.clone().iter() {
            // 1. Get subnet mechanism ID
            let mechid: u16 = SubnetMechanism::<T>::get(*netuid);
            // 2. Get mechanism emission (E_m)
            let mechanism_emission: I96F32 = I96F32::from_num(*emission_per_mechanism.get(&mechid).unwrap());
            // 3. Get mechanism TAO (T_m)
            let mechanism_tao: I96F32 = I96F32::from_num(*tao_per_mechanism.get(&mechid).unwrap());
            // 4. Get subnet TAO (T_s)
            let subnet_tao: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(*netuid));
            // 5. Calculate subnet's proportion of mechanism TAO: P_s = T_s / T_m
            let subnet_proportion: I96F32 = subnet_tao.checked_div(mechanism_tao).unwrap_or(I96F32::from_num(0));
            // 6. Calculate subnet's TAO emission: E_s = P_s * E_m
            let tao_emission: u64 = subnet_proportion.checked_mul(mechanism_emission).unwrap_or(I96F32::from_num(0)).to_num::<u64>();
            // 7. Convert TAO emission to alpha emission
            let alpha_emission: u64 = Self::tao_to_alpha(tao_emission, *netuid);
            // 8. Update total issuance: I_new = I_old + E_s
            TotalIssuance::<T>::mutate(|issuance| { *issuance = issuance.saturating_add(tao_emission) });
            // 9. Update subnet TAO: T_s_new = T_s_old + E_s
            SubnetTAO::<T>::mutate(*netuid, |subtao| { *subtao = subtao.saturating_add(tao_emission) });
            // 10. Update subnet alpha: A_s_new = A_s_old + E_α
            SubnetAlpha::<T>::mutate(*netuid, |alpha| { *alpha = alpha.saturating_add(alpha_emission) });
            // 11. Store alpha emission for this subnet
            EmissionValues::<T>::insert(*netuid, alpha_emission);
            // 12. Accumulate pending emission: P_e_new = P_e_old + E_α
            PendingEmission::<T>::mutate(netuid, |emission| { *emission = emission.saturating_add(alpha_emission) });
        }
        log::debug!("Emission per subnet: {:?}", EmissionValues::<T>::iter().collect::<Vec<_>>());

        // --- 5. Drain the accumulated subnet emissions, pass them through the epoch().
        // Before accumulating on the hotkeys the function redistributes the emission towards hotkey parents.
        // subnet_emission --> epoch() --> hotkey_emission --> (hotkey + parent hotkeys)
        for netuid in subnets.clone().iter() {
            // --- 5.1 Check to see if the subnet should run its epoch.
            if Self::should_run_epoch(*netuid, current_block) {
                // --- 5.2 Drain the subnet emission.
                let subnet_alpha_emission: u64 = PendingEmission::<T>::get(*netuid);
                PendingEmission::<T>::insert(*netuid, 0);
                log::debug!(
                    "Drained subnet alpha emission for netuid {:?}: {:?}",
                    *netuid,
                    subnet_alpha_emission
                );

                // --- 5.3 Set last step counter.
                Self::set_blocks_since_last_step(*netuid, 0);
                Self::set_last_mechanism_step_block(*netuid, current_block);


                // 5.3 Pass emission through epoch() --> hotkey emission.
                let hotkey_alpha_emission: Vec<(T::AccountId, u64, u64)> = Self::epoch(*netuid, subnet_alpha_emission);
                log::debug!(
                    "Hotkey alpha emission results for netuid {:?}: {:?}",
                    *netuid,
                    hotkey_alpha_emission
                );

                // 5.4 Accumulate the tuples on hotkeys:
                for (hotkey, mining_emission, validator_emission) in hotkey_alpha_emission {
                    // 5.5 Accumulate the emission on the hotkey and parent hotkeys.
                    Self::accumulate_hotkey_emission(
                        &hotkey,
                        *netuid,
                        validator_emission, // Amount received from validating
                        mining_emission,    // Amount recieved from mining.
                    );
                    log::debug!("Accumulated emissions on hotkey {:?} for netuid {:?}: mining {:?}, validator {:?}", hotkey, *netuid, mining_emission, validator_emission);
                }
            } else {
                log::debug!("Tempo not reached for subnet: {:?}", *netuid);
            }
        }

        // --- 6. Drain the accumulated hotkey emissions through to the nominators.
        // The hotkey takes a proportion of the emission, the remainder is drained through to the nominators.
        // We keep track of the last stake increase event for accounting purposes.
        // hotkeys --> nominators.
        let emission_tempo: u64 = Self::get_hotkey_emission_tempo();
        for (hotkey, netuid_i, hotkey_alpha_emission) in PendingdHotkeyEmissionOnNetuid::<T>::iter() {
            if Self::should_drain_hotkey(&hotkey, current_block, emission_tempo) {
                // Remove the hotkey emission from the pending emissions.
                PendingdHotkeyEmissionOnNetuid::<T>::remove( &hotkey, netuid_i );
                // Drain the hotkey emission.
                Self::drain_hotkey_emission_on_netuid( &hotkey, netuid_i, hotkey_alpha_emission, current_block);
            }
        }
        // Update drain blocks.
        for (hotkey, _, _) in PendingdHotkeyEmissionOnNetuid::<T>::iter() {
            if Self::should_drain_hotkey(&hotkey, current_block, emission_tempo) {
                LastHotkeyEmissionDrain::<T>::insert(hotkey, current_block);
            }
        }
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
    pub fn accumulate_hotkey_emission(
        hotkey: &T::AccountId,
        netuid: u16,
        validating_emission: u64,
        mining_emission: u64,
    ) {
        // Compute the hotkey's take proportion and remove it from the validating emission off the top.
        let take_proportion: I64F64 = I64F64::from_num(Delegates::<T>::get(hotkey)).saturating_div(I64F64::from_num(u16::MAX));
        let hotkey_take: u64 = take_proportion.saturating_mul(I64F64::from_num(validating_emission)).to_num::<u64>();
        let remaining_validating_emission: u64 = validating_emission.saturating_sub(hotkey_take);
        // Then distribute the remainder proportionally to parents.
        let total_distributed_to_parents: u64 = Self::distribute_to_parents( hotkey, netuid, remaining_validating_emission );
        // Remove this off the top from the parents.
        let remainder_after_parents: u64 = remaining_validating_emission - total_distributed_to_parents;
        // Finally increment the amount of mining and validating emissions for the hotkey.
        PendingdHotkeyEmissionOnNetuid::<T>::mutate(hotkey, netuid, |hotkey_pending| {
            *hotkey_pending = hotkey_pending.saturating_add(
                remainder_after_parents
                    .saturating_add(hotkey_take)
                    .saturating_add(mining_emission),
            )
        });
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
    pub fn drain_hotkey_emission_on_netuid(hotkey: &T::AccountId, netuid: u16, alpha_emission: u64, block_number: u64) {
        log::debug!(
            "Draining hotkey alpha emission for hotkey {:?} on netuid {:?} on block {:?}: {:?}",
            hotkey,
            netuid,
            block_number,
            alpha_emission
        );
        // Compute this hotkey's take value.
        let take_proportion: I64F64 = I64F64::from_num(Delegates::<T>::get(hotkey)).saturating_div(I64F64::from_num(u16::MAX));
        let hotkey_alpha_take: u64 = (take_proportion.saturating_mul(I64F64::from_num(alpha_emission))).to_num::<u64>();
        // Remove the take from the alpha emission.
        let alpha_emission_minus_hotkey_take: u64 = alpha_emission.saturating_sub(hotkey_alpha_take);
        //  distribute the remainder proportionally to nominators.
        let total_distributed_to_nominators: u64 = Self::distribute_to_nominators( hotkey, netuid, alpha_emission_minus_hotkey_take );
        // Remove the nominators distribution from the alpha emission.
        let remainder_after_nominators: u64 = alpha_emission_minus_hotkey_take - total_distributed_to_nominators;
        // Increment the hotkey's alpha based on the remainder and take.
        let hotkey_owning_coldkey: T::AccountId = Owner::<T>::get(hotkey);
        let hotkey_alpha_increment: u64 = hotkey_alpha_take.saturating_add(remainder_after_nominators);
        TotalHotkeyAlpha::<T>::insert(
            hotkey,
            netuid,
            TotalHotkeyAlpha::<T>::get( &hotkey, netuid ).saturating_add( hotkey_alpha_increment ),
        );
        Alpha::<T>::insert(
            (hotkey, &hotkey_owning_coldkey, netuid),
            Alpha::<T>::get((hotkey, &hotkey_owning_coldkey, netuid)).saturating_add( hotkey_alpha_increment ),
        );
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
}
