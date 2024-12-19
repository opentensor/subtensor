use super::*;
use crate::epoch::math::safe_modulo;
use alloc::collections::{BTreeMap, BinaryHeap};

use subnets::Mechanism;
use substrate_fixed::types::I96F32;
use tle::stream_ciphers::AESGCMStreamCipherProvider;
use tle::tlock::tld;

/// Contains all necesarry information to set weights.
///
/// In the context of commit-reveal v3, this is the payload which should be
/// encrypted, compressed, serialized, and submitted to the `commit_crv3_weights`
/// extrinsic.
#[derive(Encode, Decode)]
#[freeze_struct("46e75a8326ba3665")]
pub struct WeightsTlockPayload {
    pub uids: Vec<u16>,
    pub values: Vec<u16>,
    pub version_key: u64,
}

impl<T: Config> Pallet<T> {
    /// The `coinbase` function performs a four-part emission distribution process involving
    /// subnets, epochs, hotkeys, and nominators.
    ///
    /// It is divided into several steps, each handling a specific part of the distribution:
    ///
    /// Step 1: Compute the block-wise emission for each subnet.
    /// This involves calculating how much (TAO) should be emitted into each subnet using the root
    /// epoch function.
    ///
    /// Step 2: Accumulate the subnet block emission.
    /// After calculating the block-wise emission, these values are accumulated to keep track of how
    /// much each subnet should emit before the next distribution phase. This accumulation is a
    /// running total that gets updated each block.
    ///
    /// Step 3: Distribute the accumulated emissions through epochs.
    /// Subnets periodically distribute their accumulated emissions to hotkeys (active
    /// validators/miners) in the network on a `tempo` --- the time between epochs. This step runs
    /// Yuma consensus to determine how emissions are split among hotkeys based on their
    /// contributions and roles. The accumulation of hotkey emissions is done through the
    /// `accumulate_hotkey_emission` function. The function splits the rewards for a hotkey amongst
    /// itself and its `parents`. The parents are the hotkeys that are delegating their stake to the
    /// hotkey.
    ///
    /// Step 4: Further distribute emissions from hotkeys to nominators.
    /// Finally, the emissions received by hotkeys are further distributed to their nominators, who
    /// are stakeholders that support the hotkeys.
    pub fn run_coinbase() {
        // --- 0. Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!("Current block: {:?}", current_block);

        // --- 1. Get all netuids.
        let subnets: Vec<u16> = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", subnets);

        // The root_epoch function used to be here. In rao the code from 2 to 5 inclusively
        // replaces root epoch and uses a different algorithm to calculate EmissionValues and
        // PendingEmission for each subnet.
        // --- 2. Get the current coinbase emission.
        let block_emission: I96F32 = I96F32::from_num(Self::get_block_emission().unwrap_or(0));
        log::debug!("Block emission: {:?}", block_emission);

        // --- 3. Total subnet TAO.
        let total_issuance: I96F32 = I96F32::from_num(Self::get_total_issuance());
        log::debug!("Total issuance: {:?}", total_issuance);

        // --- 4. Sum all the SubnetTAO associated with the same mechanism
        let mut total_active_tao: I96F32 = I96F32::from_num(0);
        let mut mechanism_tao: BTreeMap<Mechanism, I96F32> = BTreeMap::new();
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
        // This loop calculates the emission for each subnet based on its mechanism and proportion of TAO.
        // For each subnet s in a mechanism m:
        // 1. Calculate subnet's proportion of mechanism TAO: P_s = T_s / T_m
        // 2. Calculate subnet's TAO emission: E_s = P_s * E_m
        // 3. Convert TAO emission to alpha emission: E_α = tao_to_alpha(E_s)
        // 4. Update total issuance: I_new = I_old + E_s
        // 5. Update subnet TAO: T_s_new = T_s_old + E_s
        // 6. Update subnet alpha: A_s_new = A_s_old + E_α
        // 7. Accumulate pending emission: P_e_new = P_e_old + E_α
        // Mathematical notation:
        // Let s be a subnet, m be a mechanism, T_s be subnet TAO, T_m be mechanism TAO,
        // E_b be block emission, E_m be mechanism emission, P_s be subnet proportion,
        // E_s be subnet emission, E_α be alpha emission, I be total issuance,
        // A_s be subnet alpha, and P_e be pending emission.
        for netuid in subnets.iter() {
            // Do not emit into root network.
            if *netuid == 0 || !Self::is_registration_allowed(*netuid) {
                continue;
            }
            // 1. Get subnet mechanism ID
            let mechid: Mechanism = SubnetMechanism::<T>::get(*netuid);
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
            // 9. Add the TAO into the subnet immediately: T_s_new = T_s_old + E_s
            SubnetTAO::<T>::mutate(*netuid, |total| {
                *total = total.saturating_add(subnet_emission)
            });
            // 10. Increase total stake here: T_total_new = T_total_old + E_s
            TotalStake::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
            // 11. Increase total issuance: I_new = I_old + E_s
            TotalIssuance::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
            // 12. Switch on dynamic or Stable.
            if mechid.is_dynamic() {
                // 12a Dynamic: Add the SubnetAlpha directly into the pool immediately: A_s_new = A_s_old + E_m
                SubnetAlphaIn::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(block_emission.to_num::<u64>())
                });
                // 12b Dynamic: Set the pending emission directly as alpha always block emission total: P_e_new = P_e_old + E_m
                PendingEmission::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(block_emission.to_num::<u64>())
                });
            } else {
                // 12c Stable: Set the pending emission as tao emission: P_e_new = P_e_old + E_s
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
                // --- 6.2 Reveal weights from the n-2nd epoch.
                if Self::get_commit_reveal_weights_enabled(netuid) {
                    if let Err(e) = Self::reveal_crv3_commits(netuid) {
                        log::warn!(
                            "Failed to reveal commits for subnet {} due to error: {:?}",
                            netuid,
                            e
                        );
                    };
                }

                // --- 6.3 Drain the subnet emission.
                let subnet_emission: u64 = PendingEmission::<T>::get(netuid);
                PendingEmission::<T>::insert(netuid, 0);
                log::debug!(
                    "Drained subnet emission for netuid {:?}: {:?}",
                    netuid,
                    subnet_emission
                );

                // --- 6.4 Set last step counter.
                Self::set_blocks_since_last_step(netuid, 0);
                Self::set_last_mechanism_step_block(netuid, current_block);

                if netuid == 0 || !Self::is_registration_allowed(netuid) {
                    // Skip netuid 0 payouts
                    continue;
                }

                // --- 6.5 Distribute the owner cut.
                let owner_cut: u64 = I96F32::from_num(subnet_emission)
                    .saturating_mul(Self::get_float_subnet_owner_cut())
                    .to_num::<u64>();
                Self::distribute_owner_cut(netuid, owner_cut);
                let remaining_emission: u64 = subnet_emission.saturating_sub(owner_cut);

                // --- 6.6 Pass emission through epoch() --> hotkey emission.
                let hotkey_emission: Vec<(T::AccountId, u64, u64)> =
                    Self::epoch(netuid, remaining_emission);

                // --- 6.6 Accumulate the tuples on hotkeys:
                for (hotkey, mining_emission, validator_emission) in hotkey_emission {
                    // 6.7 Accumulate the emission on the hotkey and parent hotkeys.
                    Self::accumulate_hotkey_emission(
                        &hotkey,
                        netuid,
                        validator_emission, // Amount received from validating
                        mining_emission,    // Amount received from mining.
                    );
                    log::debug!("Accumulated emissions on hotkey {:?} for netuid {:?}: mining {:?}, validator {:?}", hotkey, netuid, mining_emission, validator_emission);

                    // --- 6.8 Reset the stake delta for the hotkey.
                    let _ = StakeDeltaSinceLastEmissionDrain::<T>::clear_prefix(
                        (hotkey,),
                        u32::MAX,
                        None,
                    );
                }

                // --- 6.9 Apply pending childkeys of this subnet for the next epoch
                Self::do_set_pending_children(netuid);
            } else {
                // No epoch, increase blocks since last step and continue
                Self::set_blocks_since_last_step(
                    netuid,
                    Self::get_blocks_since_last_step(netuid).saturating_add(1),
                );
                log::debug!("Tempo not reached for subnet: {:?}", netuid);
            }
        }

        // --- 7. Drain the accumulated hotkey emissions through to the nominators.
        // The hotkey takes a proportion of the emission, the remainder is drained through to the nominators.
        // We keep track of the last stake increase event for accounting purposes.
        // hotkeys --> nominators.
        Self::drain_hotkey_emission(current_block);
    }

    /// Distributes the owner payment
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
            // Use subnet owner coldkey as hotkey
            let owner_hotkey = if let Ok(hotkey) = SubnetOwnerHotkey::<T>::try_get(netuid) {
                hotkey
            } else {
                owner_coldkey.clone()
            };
            // Add subnet owner cut to owner's stake
            Self::emit_into_subnet(&owner_hotkey, &owner_coldkey, netuid, owner_cut);
            // Emit event
            Self::deposit_event(Event::OwnerPaymentDistributed(
                netuid,
                owner_hotkey.clone(),
                owner_cut,
            ));
        }
    }

    /// Accumulates and distributes mining and validator emissions for a hotkey.
    ///
    /// This function performs the following key operations:
    /// 1. Calculates the hotkey's share of the validator emission based on its delegation status.
    /// 2. Computes the remaining validator emission to be distributed among parents.
    /// 3. Retrieves the list of parents and their stake contributions.
    /// 4. Calculates the total global and alpha (subnet-specific) stakes from parents.
    /// 5. Distributes the remaining validator emission to parents based on their contributions.
    /// 6. Allocates any undistributed validator emission, the hotkey's take, and the mining emission to the hotkey itself.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey for which emissions are being calculated.
    /// * `netuid` - The unique identifier of the subnet to which the hotkey belongs.
    /// * `validating_emission` - The amount of validator emission allocated to the hotkey.
    /// * `mining_emission` - The amount of mining emission allocated to the hotkey.
    /// * `hotkey_emission_tuples` - A mutable reference to a vector that will be populated with emission distribution data.
    ///
    /// # Returns
    /// * `u64` - The portion of emission that should be immediately added to the hotkey stake. It consists of mining_emission
    ///   and childkey take
    ///
    /// # Effects
    /// - Modifies `hotkey_emission_tuples` by adding entries for each parent receiving emission and the hotkey itself.
    /// - Does not directly modify any storage; all changes are recorded in `hotkey_emission_tuples` for later processing.
    ///
    /// # Note
    /// This function ensures fair distribution of emissions based on stake proportions and delegation agreements.
    /// It handles edge cases such as zero contributions and potential overflows using saturating arithmetic.
    pub fn source_hotkey_emission(
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
        let mut total_global: I96F32 = I96F32::from_num(0);
        let mut total_alpha: I96F32 = I96F32::from_num(0);
        let mut contributions: Vec<(T::AccountId, I96F32, I96F32)> = Vec::new();

        let current_block = Self::get_current_block_as_u64();
        let min_required_block_diff = 2u64.saturating_mul(Self::get_tempo(netuid) as u64);

        // Calculate total global and alpha (subnet-specific) stakes from all parents
        for (proportion, parent) in Self::get_parents(hotkey, netuid) {
            // TODO: deal with parent that staked recently.
            // Get the last block this parent added some stake
            let stake_add_block = Self::get_last_stake_increase_block(hotkey, &parent);

            let stake_added_block_diff = current_block.saturating_sub(stake_add_block);

            // If the last block this parent added any stake is old enough (older than two subnet tempos),
            // consider this parent's contribution
            if stake_added_block_diff >= min_required_block_diff {
                // Convert the parent's stake proportion to a fractional value
                let parent_proportion: I96F32 =
                    I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));

                // Get the parent's global and subnet-specific (alpha) stakes
                let parent_global: I96F32 = I96F32::from_num(Self::get_global_for_hotkey(&parent));
                let parent_alpha: I96F32 =
                    I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(&parent, netuid));

                // Calculate the parent's contribution to the hotkey's stakes
                let parent_alpha_contribution: I96F32 =
                    parent_alpha.saturating_mul(parent_proportion);
                let parent_global_contribution: I96F32 =
                    parent_global.saturating_mul(parent_proportion);

                // Add to the total stakes
                total_global = total_global.saturating_add(parent_global_contribution);
                total_alpha = total_alpha.saturating_add(parent_alpha_contribution);

                // Store the parent's contributions for later use
                contributions.push((
                    parent.clone(),
                    parent_alpha_contribution,
                    parent_global_contribution,
                ));
            }
        }

        // Get the weights for global and alpha stakes in emission distribution
        let global_weight: I96F32 = Self::get_global_weight(netuid);
        let alpha_weight: I96F32 = I96F32::from_num(1.0).saturating_sub(global_weight);

        // Distribute emission to parents based on their contributions
        for (parent, alpha_contribution, global_contribution) in contributions {
            // Calculate emission based on alpha (subnet-specific) stake
            let alpha_emission: I96F32 = alpha_weight
                .saturating_mul(validating_emission)
                .saturating_mul(alpha_contribution)
                .checked_div(total_alpha)
                .unwrap_or(I96F32::from_num(0.0));

            // Calculate emission based on global stake
            let global_emission: I96F32 = global_weight
                .saturating_mul(validating_emission)
                .saturating_mul(global_contribution)
                .checked_div(total_global)
                .unwrap_or(I96F32::from_num(0.0));

            // Sum up the total emission for this parent
            let total_emission: u64 = alpha_emission
                .saturating_add(global_emission)
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

    /// Distributes emission to nominators and the hotkey owner based on their contributions and delegation status.
    ///
    /// This function performs the following steps:
    /// 1. Calculates the hotkey's share of the emission based on its delegation status.
    /// 2. Computes the remaining emission to be distributed among nominators.
    /// 3. Retrieves global and alpha scores for the hotkey.
    /// 4. Iterates over all nominators, calculating their individual contributions based on alpha and global scores.
    /// 5. Distributes the emission to nominators proportionally based on their contributions.
    /// 6. Allocates any remaining emission and the hotkey's take to the hotkey owner.
    ///
    /// # Arguments
    /// * `hotkey` - The AccountId of the hotkey.
    /// * `netuid` - The subnet ID.
    /// * `emission` - The total emission to be distributed.
    /// * `_block_number` - The current block number (unused in this function).
    /// * `emission_tuples` - A mutable reference to a vector that will be populated with emission distribution data.
    ///
    /// # Effects
    /// - Modifies `emission_tuples` by adding entries for each nominator receiving emission and the hotkey owner.
    /// - Does not directly modify any storage; all changes are recorded in `emission_tuples` for later processing.
    ///
    /// # Note
    /// This function ensures fair distribution of emissions based on stake proportions and delegation agreements.
    /// It handles edge cases such as zero contributions and potential overflows using saturating arithmetic.
    ///
    /// # Runtime
    /// - Gets the nominator contributions and sum. O(n)
    /// - Calculates each nominator's contribution as a weight. O(n)
    /// - Constructs MaxHeap at the same time as the above. ~O(1)
    /// - Gets the top k weights and their weight sum. O(k log n)
    /// - Calculates the normalized weights for only the top k nominators. O(n)
    ///
    /// Total: O(3n + k log n)
    pub fn source_nominator_emission(
        hotkey: &T::AccountId,
        netuid: u16,
        emission: u64,
        _current_block: u64,
        emission_tuples: &mut BTreeMap<(T::AccountId, T::AccountId), Vec<(u16, u64)>>,
    ) {
        // Calculate the hotkey's share of the emission based on its delegation status
        let emission: I96F32 = I96F32::from_num(emission);
        let take_proportion: I96F32 = I96F32::from_num(Delegates::<T>::get(hotkey))
            .saturating_div(I96F32::from_num(u16::MAX));
        let hotkey_take: I96F32 = take_proportion.saturating_mul(emission);

        // Initialize variables to track emission distribution
        let mut to_nominators: u64 = 0;
        let nominator_emission: I96F32 = emission.saturating_sub(hotkey_take);

        // Prepare to calculate contributions from nominators
        let mut total_global: I96F32 = I96F32::from_num(0);
        let mut total_alpha: I96F32 = I96F32::from_num(0);
        let mut contributions: Vec<(T::AccountId, I96F32, I96F32)> = Vec::new();

        let _hotkey_tempo = HotkeyEmissionTempo::<T>::get();

        // Calculate total global and alpha scores for all nominators
        for (nominator, nominator_alpha) in Alpha::<T>::iter_prefix((hotkey, netuid)) {
            let nonviable_nominator_stake: (u64, u64) =
                Self::get_nonviable_stake(hotkey, &nominator, netuid);
            let (nonviable_global, nonviable_alpha) = nonviable_nominator_stake;

            let alpha_contribution: I96F32 =
                I96F32::from_num(nominator_alpha.saturating_sub(nonviable_alpha));
            let global_contribution: I96F32 = I96F32::from_num(
                Self::get_global_for_hotkey_and_coldkey(hotkey, &nominator)
                    .saturating_sub(nonviable_global),
            );
            total_global = total_global.saturating_add(global_contribution);
            total_alpha = total_alpha.saturating_add(alpha_contribution);
            contributions.push((nominator.clone(), alpha_contribution, global_contribution));
        }

        // Get the weights for global and alpha scores
        let global_weight: I96F32 = Self::get_global_weight(netuid);
        let alpha_weight: I96F32 = I96F32::from_num(1.0).saturating_sub(global_weight);

        // Distribute emission to nominators based on their contributions
        if total_alpha > I96F32::from_num(0) || total_global > I96F32::from_num(0) {
            let max_nominators: u16 = Self::get_max_nominators_per_subnet(netuid);
            let mut top_k_heap: BinaryHeap<I96F32> = BinaryHeap::new(); // Max heap

            let mut contributions_as_weight: Vec<(T::AccountId, I96F32)> = vec![];
            for (nominator, alpha_contribution, global_contribution) in contributions {
                // Calculate the nominator's contribution as a weight
                let alpha_emission_weight: I96F32 = alpha_contribution
                    .checked_div(total_alpha)
                    .unwrap_or(I96F32::from_num(0))
                    .saturating_mul(alpha_weight);

                let global_emission_weight: I96F32 = global_contribution
                    .checked_div(total_global)
                    .unwrap_or(I96F32::from_num(0))
                    .saturating_mul(global_weight);

                let nominator_weight: I96F32 =
                    alpha_emission_weight.saturating_add(global_emission_weight);

                contributions_as_weight.push((nominator, nominator_weight));
                top_k_heap.push(nominator_weight);
            }

            let mut popped: u16 = 0;
            let mut top_k_sum: I96F32 = I96F32::from_num(0);
            while let Some(top_k_max) = top_k_heap.pop() {
                // Pop the largest weights first
                top_k_sum = top_k_sum.saturating_add(top_k_max); // Sum the top k weights
                popped = popped.saturating_add(1);

                if popped >= max_nominators.saturating_sub(1) {
                    break;
                }
            }
            let top_k_min = top_k_heap.pop().unwrap_or(I96F32::from_num(0)); // This is the smallest weight in the top k
            top_k_sum = top_k_sum.saturating_add(top_k_min); // Also add the smallest weight

            for (nominator, nominator_weight) in contributions_as_weight {
                if nominator_weight < top_k_min {
                    continue; // Skip nominator if it's not in the top k
                }

                let normalized_weight: I96F32 = nominator_weight
                    .checked_div(top_k_sum) // Normalize the nominator's weight against the top k weights
                    .unwrap_or(I96F32::from_num(0));

                let total_emission: u64 = nominator_emission
                    .saturating_mul(normalized_weight)
                    .to_num::<u64>();
                if total_emission > 0 {
                    // Record the emission for this nominator
                    to_nominators = to_nominators.saturating_add(total_emission);
                    emission_tuples
                        .entry((hotkey.clone(), nominator.clone()))
                        .or_default()
                        .push((netuid, total_emission));
                }
            }
        }

        // Calculate and distribute the remaining emission to the hotkey
        let hotkey_owner: T::AccountId = Owner::<T>::get(hotkey);
        let remainder: u64 = emission
            .to_num::<u64>()
            .saturating_sub(hotkey_take.to_num::<u64>())
            .saturating_sub(to_nominators);
        let final_hotkey_emission: u64 = hotkey_take.to_num::<u64>().saturating_add(remainder);
        emission_tuples
            .entry((hotkey.clone(), hotkey_owner.clone()))
            .or_default()
            .push((netuid, final_hotkey_emission));
    }

    /// The `reveal_crv3_commits` function is run at the very beginning of epoch `n`,
    pub fn reveal_crv3_commits(netuid: u16) -> dispatch::DispatchResult {
        use ark_serialize::CanonicalDeserialize;
        use frame_support::traits::OriginTrait;
        use tle::curves::drand::TinyBLS381;
        use tle::tlock::TLECiphertext;
        use w3f_bls::EngineBLS;

        let cur_block = Self::get_current_block_as_u64();
        let cur_epoch = Self::get_epoch_index(netuid, cur_block);

        // Weights revealed must have been committed during epoch `cur_epoch - reveal_period`.
        let reveal_epoch =
            cur_epoch.saturating_sub(Self::get_reveal_period(netuid).saturating_sub(1));

        // Clean expired commits
        for (epoch, _) in CRV3WeightCommits::<T>::iter_prefix(netuid) {
            if epoch < reveal_epoch {
                CRV3WeightCommits::<T>::remove(netuid, epoch);
            }
        }

        // No commits to reveal until at least epoch 2.
        if cur_epoch < 2 {
            log::warn!("Failed to reveal commit for subnet {} Too early", netuid);
            return Ok(());
        }

        let mut entries = CRV3WeightCommits::<T>::take(netuid, reveal_epoch);

        // Keep popping item off the end of the queue until we sucessfully reveal a commit.
        while let Some((who, serialized_compresssed_commit, round_number)) = entries.pop_front() {
            let reader = &mut &serialized_compresssed_commit[..];
            let commit = match TLECiphertext::<TinyBLS381>::deserialize_compressed(reader) {
                Ok(c) => c,
                Err(e) => {
                    log::warn!(
						"Failed to reveal commit for subnet {} submitted by {:?} due to error deserializing the commit: {:?}",
						netuid,
						who,
						e
					);
                    continue;
                }
            };

            // Try to get the round number from pallet_drand.
            let pulse = match pallet_drand::Pulses::<T>::get(round_number) {
                Some(p) => p,
                None => {
                    // Round number used was not found on the chain. Skip this commit.
                    log::warn!(
                        "Failed to reveal commit for subnet {} submitted by {:?} due to missing round number {} at time of reveal.",
						netuid,
						who,
                        round_number
                    );
                    continue;
                }
            };

            let signature_bytes = pulse
                .signature
                .strip_prefix(b"0x")
                .unwrap_or(&pulse.signature);

            let sig_reader = &mut &signature_bytes[..];
            let sig = match <TinyBLS381 as EngineBLS>::SignatureGroup::deserialize_compressed(
                sig_reader,
            ) {
                Ok(s) => s,
                Err(e) => {
                    log::error!(
						"Failed to reveal commit for subnet {} submitted by {:?} due to error deserializing signature from drand pallet: {:?}",
						netuid,
						who,
						e
					);
                    continue;
                }
            };

            let decrypted_bytes: Vec<u8> = match tld::<TinyBLS381, AESGCMStreamCipherProvider>(
                commit, sig,
            ) {
                Ok(d) => d,
                Err(e) => {
                    log::warn!(
							"Failed to reveal commit for subnet {} submitted by {:?} due to error decrypting the commit: {:?}",
							netuid,
							who,
							e
												);
                    continue;
                }
            };

            // Decrypt the bytes into WeightsPayload
            let mut reader = &decrypted_bytes[..];
            let payload: WeightsTlockPayload = match Decode::decode(&mut reader) {
                Ok(w) => w,
                Err(e) => {
                    log::warn!("Failed to reveal commit for subnet {} submitted by {:?} due to error deserializing WeightsPayload: {:?}", netuid, who, e);
                    continue;
                }
            };

            if let Err(e) = Self::do_set_weights(
                T::RuntimeOrigin::signed(who.clone()),
                netuid,
                payload.uids,
                payload.values,
                payload.version_key,
            ) {
                log::warn!(
                    "Failed to `do_set_weights` for subnet {} submitted by {:?}: {:?}",
                    netuid,
                    who,
                    e
                );
                continue;
            };
        }

        Ok(())
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
        // This netuid hotkey emission tuples
        let mut hotkey_emission_tuples: Vec<(T::AccountId, u16, u64)> = vec![];

        // Distribute the emission on the hotkey and parent hotkeys appending new vectors to hotkey_emission_tuples.
        let untouchable_emission = Self::source_hotkey_emission(
            hotkey,
            netuid,
            validating_emission, // Amount received from validating
            mining_emission,     // Amount recieved from mining.
            &mut hotkey_emission_tuples,
        );

        // Add mining and childkey take emission to stake right away
        let coldkey = Owner::<T>::get(hotkey);
        Self::emit_into_subnet(hotkey, &coldkey, netuid, untouchable_emission);

        // Accounting: Add emission to the pending hotkey emission for further distribution to nominators
        let mut processed_hotkeys_on_netuid: BTreeMap<T::AccountId, ()> = BTreeMap::new();
        for (hotkey, netuid_j, emission) in hotkey_emission_tuples {
            PendingHotkeyEmissionOnNetuid::<T>::mutate(&hotkey, netuid_j, |total| {
                *total = total.saturating_add(emission)
            });
            // If the hotkey has not been processed yet, update the last emission drain block
            if !processed_hotkeys_on_netuid.contains_key(&hotkey) {
                LastHotkeyEmissionOnNetuid::<T>::insert(&hotkey, netuid_j, emission);
                processed_hotkeys_on_netuid.insert(hotkey.clone(), ());
            } else {
                LastHotkeyEmissionOnNetuid::<T>::mutate(&hotkey, netuid_j, |total| {
                    *total = total.saturating_add(emission)
                });
            }
        }
    }

    /// Accumulates emissions for nominators and updates the last emission drain block for hotkeys.
    ///
    /// This function processes a vector of tuples containing nominator emission data.
    /// It updates two storage items:
    /// 1. The emission for each nominator (coldkey) associated with a hotkey on a subnet.
    /// 2. The last emission drain block for each hotkey.
    ///
    /// # Arguments
    ///
    /// * `nominator_tuples` - A mutable reference to a vector of tuples, each containing:
    ///   - `T::AccountId`: The account ID of the hotkey.
    ///   - `T::AccountId`: The account ID of the coldkey (nominator).
    ///   - `u16`: The subnet ID (netuid).
    ///   - `u64`: The emission value to be added.
    /// * `block` - The current block number.
    pub fn accumulate_nominator_emission(
        nominator_tuples: &mut BTreeMap<(T::AccountId, T::AccountId), Vec<(u16, u64)>>,
        block: u64,
    ) {
        // Iterate over each tuple in the nominator_tuples map
        for ((hotkey, coldkey), emission_vec) in nominator_tuples {
            LastHotkeyEmissionDrain::<T>::insert(hotkey.clone(), block);

            for (netuid, emission) in emission_vec {
                // If the emission value is greater than 0, update the subnet emission
                if *emission > 0 {
                    Self::emit_into_subnet(hotkey, coldkey, *netuid, *emission);
                    // Record the last emission value for the hotkey-coldkey pair on the subnet
                    LastHotkeyColdkeyEmissionOnNetuid::<T>::insert(
                        (hotkey.clone(), coldkey.clone(), *netuid),
                        *emission,
                    );
                }
            }
        }
    }

    /// Drains the accumulated hotkey emission through to the nominators. The hotkey takes a proportion of the emission.
    ///
    /// The remainder is drained through to the nominators keeping track of the last stake increase event to ensure that the hotkey does not
    /// gain more emission than it's stake since the last drain.
    /// hotkeys --> nominators.
    ///
    /// The untouchable part of pending hotkey emission that consists of mining emission and childkey
    /// take has already been distributed in accumulate_hotkey_emission, so it is already excluded from
    /// PendingdHotkeyEmission
    ///
    /// 1. It resets the accumulated emissions for the hotkey to zero.
    /// 4. It calculates the total stake for the hotkey and determines the hotkey's own take from the emissions based on its delegation status.
    /// 5. It then calculates the remaining emissions after the hotkey's take and distributes this remaining amount proportionally among the hotkey's nominators.
    /// 6. Each nominator's share of the emissions is added to their stake, but only if their stake was not manually increased since the last emission drain.
    /// 7. Finally, the hotkey's own take and any undistributed emissions are added to the hotkey's total stake.
    ///
    /// This function ensures that emissions are fairly distributed according to stake proportions and delegation agreements, and it updates the necessary records to reflect these changes.
    ///
    pub fn drain_hotkey_emission(current_block: u64) {
        // Nominator emission will not allow duplicate hotkey-coldkey pairs. Each entry for an individual
        // hotkey-coldkey pair is a vector of (netuid, emission) tuples.
        let mut nominator_emission: BTreeMap<(T::AccountId, T::AccountId), Vec<(u16, u64)>> =
            BTreeMap::new();

        let emission_tempo: u64 = Self::get_hotkey_emission_tempo();

        for (hotkey, netuid, hotkey_emission) in PendingHotkeyEmissionOnNetuid::<T>::iter() {
            if Self::should_drain_hotkey(&hotkey, current_block, emission_tempo) {
                // Remove the hotkey emission from the pending emissions.
                PendingHotkeyEmissionOnNetuid::<T>::remove(&hotkey, netuid);

                // Drain the hotkey emission.
                Self::source_nominator_emission(
                    &hotkey,
                    netuid,
                    hotkey_emission,
                    current_block,
                    &mut nominator_emission,
                );

                log::debug!(
                    "Drained hotkey emission for hotkey {:?} for netuid {:?} on block {:?}: {:?}",
                    hotkey,
                    netuid,
                    current_block,
                    hotkey_emission,
                );
            }
        }
        Self::accumulate_nominator_emission(&mut nominator_emission, current_block);
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
        let tempo_plus_one = (tempo as u64).saturating_add(1);
        let adjusted_block = block_number.wrapping_add(netuid_plus_one);
        let remainder = safe_modulo(adjusted_block, tempo_plus_one);
        (tempo as u64).saturating_sub(remainder)
    }
}
