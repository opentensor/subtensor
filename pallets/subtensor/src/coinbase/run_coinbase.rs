use super::*;
use alloc::collections::BTreeMap;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    pub fn run_coinbase() {
        // --- 0. Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!("Current block: {:?}", current_block);

        // --- 1. Get all netuids.
        let subnets: Vec<u16> = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", subnets);

        // --- 2. Get the current coinbase emission.
        let block_emission: I96F32 = I96F32::from_num(Self::get_block_emission().unwrap_or(0));
        log::debug!("Block emission: {:?}", block_emission);

        // --- 4. Sum all the SubnetTAO associated with the same mechanism
        let mut total_active_tao: I96F32 = I96F32::from_num(0);
        let mut mechanism_tao: BTreeMap<u16, I96F32> = BTreeMap::new();
        for netuid in subnets.iter() {
            if *netuid == 0 {
                continue;
            } // Skip root network
            let mechid = SubnetMechanism::<T>::get(*netuid);
            let subnet_tao = I96F32::from_num(SubnetTAO::<T>::get(*netuid));
            let new_subnet_tao = subnet_tao
                .saturating_add(*mechanism_tao.entry(mechid).or_insert(I96F32::from_num(0)));
            *mechanism_tao.entry(mechid).or_insert(I96F32::from_num(0)) = new_subnet_tao;
            total_active_tao = total_active_tao.saturating_add(subnet_tao);
        }
        log::debug!("Mechanism TAO sums: {:?}", mechanism_tao);

        // --- 5. Compute EmissionValues per subnet.
        for netuid in subnets.iter() {
            // Do not emit into root network.
            if *netuid == 0 {
                continue;
            }
            // 1. Get subnet mechanism ID
            let mechid: u16 = SubnetMechanism::<T>::get(*netuid);
            // 2. Get subnet TAO (T_s)
            let subnet_tao: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(*netuid));
            // 3. Get the denominator as the sum of all TAO associated with a specific mechanism (T_m)
            let mech_tao: I96F32 = *mechanism_tao.get(&mechid).unwrap_or(&I96F32::from_num(0));
            // 4. Compute the mechanism emission proportion: P_m = T_m / T_total
            let mech_proportion: I96F32 = mech_tao
                .checked_div(total_active_tao)
                .unwrap_or(I96F32::from_num(0));
            // 5. Compute the mechanism emission: E_m = P_m * E_b
            let mech_emission: I96F32 = mech_proportion.saturating_mul(block_emission);
            // 6. Calculate subnet's proportion of mechanism TAO: P_s = T_s / T_m
            let subnet_proportion: I96F32 = subnet_tao
                .checked_div(mech_tao)
                .unwrap_or(I96F32::from_num(0));
            // 7. Calculate subnet's TAO emission: E_s = P_s * E_m
            let subnet_emission: u64 = mech_emission
                .checked_mul(subnet_proportion)
                .unwrap_or(I96F32::from_num(0))
                .to_num::<u64>();
            // 8. Store the block emission for this subnet
            EmissionValues::<T>::insert(*netuid, subnet_emission);
            // 12. Switch on dynamic or Stable.
            if mechid == 1 {
                // 12.a.1 Stop emission on number vs pool size.
                if I96F32::from_num(total_active_tao).saturating_div(I96F32::from_num(1_000_000_000)) < I96F32::from_num(Self::get_current_block_as_u64()) {
                    // Step 12.a.1: Increase Tao in the subnet reserves.
                    SubnetTAO::<T>::mutate(*netuid, |total| {
                        *total = total.saturating_add(subnet_emission)
                    });
                    // Step 12.a.2: Increase total Tao in all pools.
                    TotalStake::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
                    // Step 12.a.3:. Increase total Tao issuance.
                    TotalIssuance::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
                }
                // 12.a.4: Increase pool reserves (Alpha in)
                SubnetAlphaIn::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(block_emission.to_num::<u64>())
                });
                // 12.a.5: Increase this subnet emission in alpha (block emission). (Alpha out).
                PendingEmission::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(block_emission.to_num::<u64>())
                });
            } else {
                // Step 12.b.1: 1Normal emission flow, increase tao on this subnet.
                SubnetTAO::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(subnet_emission)
                });
                // Step 12.b.2: Increase total stake across all subnets.
                TotalStake::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
                // Step 12.b.2: Increase total issuance of Tao.
                TotalIssuance::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
                // Step 12.b.2: Increase this subnet pending emission.
                PendingEmission::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(subnet_emission)
                });
            }
        }
        log::debug!(
            "Emission per subnet: {:?}",
            EmissionValues::<T>::iter().collect::<Vec<_>>()
        );
        log::debug!(
            "Pending Emission per subnet: {:?}",
            PendingEmission::<T>::iter().collect::<Vec<_>>()
        );
        // By this point we have PendingEmission and EmissionValues calculated for each subnet

        // --- 6. Drain the accumulated subnet emissions, pass them through the epoch().
        // Before accumulating on the hotkeys the function redistributes the emission towards hotkey parents.
        // subnet_emission --> epoch() --> hotkey_emission --> (hotkey + parent hotkeys)
        for &netuid in subnets.iter() {
            // --- 6.1 Check to see if the subnet should run its epoch.
            if Self::should_run_epoch(netuid, current_block) {
                // --- 6.2 Drain the subnet emission.
                let subnet_emission: u64 = PendingEmission::<T>::get(netuid);
                PendingEmission::<T>::insert(netuid, 0);

                // --- 6.3 Set last step counter.
                Self::set_blocks_since_last_step(netuid, 0);
                Self::set_last_mechanism_step_block(netuid, current_block);

                // --- 6.4 Distribute the owner cut.
                let owner_cut: u64 = I96F32::from_num(subnet_emission)
                    .saturating_mul(Self::get_float_subnet_owner_cut())
                    .to_num::<u64>();
                Self::distribute_owner_cut(netuid, owner_cut);
                let remaining_emission: u64 = subnet_emission.saturating_sub(owner_cut);

                // --- 6.5 Pass emission through epoch() --> hotkey emission.
                let hotkey_emission: Vec<(T::AccountId, u64, u64)> = Self::epoch_mock(netuid, remaining_emission);

                // --- 6.6 Accumulate the tuples on hotkeys:
                for (hotkey, mining_emission, validator_emission) in hotkey_emission {
                    // Accumulate the emission on the hotkey and parent hotkeys.
                    Self::distribute_hotkey_emission(
                        &hotkey,
                        netuid,
                        validator_emission, // Amount received from validating
                        mining_emission,    // Amount recieved from mining.
                    );
                    log::debug!("Accumulated emissions on hotkey {:?} for netuid {:?}: mining {:?}, validator {:?}", hotkey, netuid, mining_emission, validator_emission);
                }
            } else {
                // No epoch, increase blocks since last step and continue
                Self::set_blocks_since_last_step(
                    netuid,
                    Self::get_blocks_since_last_step(netuid).saturating_add(1),
                );
                log::debug!("Tempo not reached for subnet: {:?}", netuid);
            }
        }
    }

    /// Distributes the owner payment 18% of the block reward.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID of the subnet.
    /// * `owner_cut` - The total amount of payment to distribute.
    ///
    /// * Emits an `OwnerPaymentDistributed` event for each distribution.
    ///
    pub fn distribute_owner_cut(netuid: u16, owner_cut: u64) {
        // Check if the subnet has an owner and the owner has the hotkey
        if let Ok(owner_coldkey) = SubnetOwner::<T>::try_get(netuid) {
            let owner_hotkey = if let Ok(hotkey) = SubnetOwnerHotkey::<T>::try_get(netuid) {
                hotkey
            } else {
                owner_coldkey.clone()
            }; 
            // Add subnet owner cut to owner's stake directly under the coldkey.
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet( &owner_hotkey, &owner_coldkey, netuid, owner_cut );
            // Increase the amount of outstanding alpha stake on this subnet..
            SubnetAlphaOut::<T>::mutate(netuid, |total| {
                *total = total.saturating_sub( owner_cut );
            });
            // Emit event
            Self::deposit_event(Event::OwnerPaymentDistributed(
                netuid,
                owner_hotkey.clone(),
                owner_cut,
            ));
        }
    }

    
    pub fn distribute_hotkey_emission(
        hotkey: &T::AccountId,
        netuid: u16,
        validating_emission: u64,
        mining_emission: u64,
    ) {
        // Step 1: Init a vector to hold emission tuples for parents (and self)
        let mut self_and_parent_emission_tuples: Vec<(T::AccountId, u16, u64)> = vec![];

        // Step 2: Fill self and parent emission tuples, return emission for key itself
        let this_hotkeys_emission: u64 = Self::distribute_to_parents(
            hotkey,
            netuid,
            validating_emission, // Amount received from validating
            mining_emission,     // Amount recieved from mining.
            &mut self_and_parent_emission_tuples,
        );

        // Step 3: Distribute emission to myself immediately.
        let hotkey_owner = Owner::<T>::get(hotkey);
        Self::increase_stake_for_hotkey_and_coldkey_on_subnet( hotkey, &hotkey_owner, netuid, this_hotkeys_emission );
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_add( this_hotkeys_emission );
        });

        // Step 4: For all parents and self, distribute to nominators based on local and root emission ratio.
        for (parent_j, _, emission_j) in self_and_parent_emission_tuples {

            // Step 5. Get the current root weight.
            let root_weight: I96F32 = Self::get_root_weight( netuid );

            // Step 6. Determine proportion due to root weight and local weight.
            let local_emission_in_alpha: u64 = I96F32::from_num( emission_j ).saturating_mul( I96F32::from_num(1.0) - root_weight ).to_num::<u64>();
            let root_emission_in_alpha: u64 = I96F32::from_num( emission_j ).saturating_mul( root_weight ).to_num::<u64>();

            // Step 7. Add the local alpha emission onto the hotkey.
            Self::increase_stake_for_hotkey_on_subnet( &parent_j, netuid, local_emission_in_alpha );
            SubnetAlphaOut::<T>::mutate( netuid, |total| {
                *total = total.saturating_add( local_emission_in_alpha );
            });

            // Step 8. Swap the alpha emission into tao through the pool to attain tao emission for root.
            let root_emission: u64 = Self::swap_tao_for_alpha( Self::get_root_netuid(), Self::swap_alpha_for_tao( netuid, root_emission_in_alpha ) );

            // Step 9. Add the tao emission onto root.
            Self::increase_stake_for_hotkey_on_subnet( &parent_j, Self::get_root_netuid(), root_emission );
        }
    }

    pub fn distribute_to_parents(
        hotkey: &T::AccountId,
        netuid: u16,
        validating_emission: u64,
        mining_emission: u64,
        hotkey_emission_tuples: &mut Vec<(T::AccountId, u16, u64)>,
    ) -> u64 {
        // Calculate the hotkey's share of the validator emission based on its childkey take
        let validating_emission: I96F32 = I96F32::from_num(validating_emission);
        let childkey_take_proportion: I96F32 =
            I96F32::from_num(Self::get_childkey_take(hotkey, netuid))
                .saturating_div(I96F32::from_num(u16::MAX));
        let mut total_childkey_take: u64 = 0;
        // NOTE: Only the validation emission should be split amongst parents.

        // Initialize variables to track emission distribution
        let mut to_parents: u64 = 0;

        // Initialize variables to calculate total stakes from parents
        let mut total_root: I96F32 = I96F32::from_num(0);
        let mut total_alpha: I96F32 = I96F32::from_num(0);
        let mut contributions: Vec<(T::AccountId, I96F32, I96F32)> = Vec::new();

        // Calculate total root and alpha (subnet-specific) stakes from all parents
        for (proportion, parent) in Self::get_parents(hotkey, netuid) {
          
            // Convert the parent's stake proportion to a fractional value
            let parent_proportion: I96F32 = I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));

            // Get the parent's root and subnet-specific (alpha) stakes
            let parent_root: I96F32 = I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(&parent, Self::get_root_netuid()));
            let parent_alpha: I96F32 = I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(&parent, netuid));

            // Calculate the parent's contribution to the hotkey's stakes
            let parent_alpha_contribution: I96F32 = parent_alpha.saturating_mul(parent_proportion);
            let parent_root_contribution: I96F32 = parent_root.saturating_mul(parent_proportion);

            // Add to the total stakes
            total_root = total_root.saturating_add(parent_root_contribution);
            total_alpha = total_alpha.saturating_add(parent_alpha_contribution);

            // Store the parent's contributions for later use
            contributions.push((
                parent.clone(),
                parent_alpha_contribution,
                parent_root_contribution,
            ));
        }

        // Get the weights for root and alpha stakes in emission distribution
        let root_weight: I96F32 = Self::get_root_weight(netuid);
        let alpha_weight: I96F32 = I96F32::from_num(1.0).saturating_sub(root_weight);

        // Distribute emission to parents based on their contributions
        for (parent, alpha_contribution, root_contribution) in contributions {
            // Calculate emission based on alpha (subnet-specific) stake
            let alpha_emission: I96F32 = alpha_weight
                .saturating_mul(validating_emission)
                .saturating_mul(alpha_contribution)
                .checked_div(total_alpha)
                .unwrap_or(I96F32::from_num(0.0));

            // Calculate emission based on root stake
            let root_emission: I96F32 = root_weight
                .saturating_mul(validating_emission)
                .saturating_mul(root_contribution)
                .checked_div(total_root)
                .unwrap_or(I96F32::from_num(0.0));

            // Sum up the total emission for this parent
            let total_emission: u64 = alpha_emission
                .saturating_add(root_emission)
                .to_num::<u64>();

            // Reserve childkey take
            let child_emission_take: u64 = childkey_take_proportion
                .saturating_mul(I96F32::from_num(total_emission))
                .to_num::<u64>();
            total_childkey_take = total_childkey_take.saturating_add(child_emission_take);
            let parent_total_emission = total_emission.saturating_sub(child_emission_take);

            // Add the parent's emission to the distribution list
            hotkey_emission_tuples.push((parent, netuid, parent_total_emission));

            // Keep track of total emission distributed to parents
            to_parents = to_parents.saturating_add(parent_total_emission);
        }

        // Calculate the final emission for the hotkey itself
        let final_hotkey_emission = validating_emission
            .to_num::<u64>()
            .saturating_sub(to_parents)
            .saturating_sub(total_childkey_take);

        // Add the hotkey's own emission to the distribution list
        hotkey_emission_tuples.push((hotkey.clone(), netuid, final_hotkey_emission));

        // Return the emission that needs to be added to the hotkey stake right away
        total_childkey_take.saturating_add(mining_emission)
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
        let tempo_plus_one = (tempo as u64).saturating_add(1);
        let adjusted_block = block_number.wrapping_add(netuid_plus_one);
        let remainder = adjusted_block % tempo_plus_one;
        (tempo as u64).saturating_sub(remainder)
    }
}
