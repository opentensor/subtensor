use super::*;
use alloc::collections::BTreeMap;
use safe_math::*;
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
    pub fn run_coinbase(block_emission: I96F32) {
        // --- 0. Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!("Current block: {:?}", current_block);

        // --- 1. Get all netuids.
        let subnets: Vec<u16> = Self::get_all_subnet_netuids();
        log::debug!("All subnet netuids: {:?}", subnets);

        // --- 2. Sum all the SubnetTAO associated with the same mechanism.
        // Mechanisms get emission based on the proportion of TAO across all their subnets
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

        // --- 3. Compute subnet emission values (amount of tao inflation this block).
        let mut tao_in_map: BTreeMap<u16, u64> = BTreeMap::new();
        for netuid in subnets.iter() {
            // Do not emit into root network.
            if *netuid == 0 {
                continue;
            }
            // 3.1: Get subnet mechanism ID
            let mechid: u16 = SubnetMechanism::<T>::get(*netuid);
            log::debug!("Netuid: {:?}, Mechanism ID: {:?}", netuid, mechid);
            // 3.2: Get subnet TAO (T_s)
            let subnet_tao: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(*netuid));
            log::debug!("Subnet TAO (T_s) for netuid {:?}: {:?}", netuid, subnet_tao);
            // 3.3: Get the denominator as the sum of all TAO associated with a specific mechanism (T_m)
            let mech_tao: I96F32 = *mechanism_tao.get(&mechid).unwrap_or(&I96F32::from_num(0));
            log::debug!(
                "Mechanism TAO (T_m) for mechanism ID {:?}: {:?}",
                mechid,
                mech_tao
            );
            // 3.4: Compute the mechanism emission proportion: P_m = T_m / T_total
            let mech_proportion: I96F32 = mech_tao
                .checked_div(total_active_tao)
                .unwrap_or(I96F32::from_num(0));
            log::debug!(
                "Mechanism proportion (P_m) for mechanism ID {:?}: {:?}",
                mechid,
                mech_proportion
            );
            // 3.5: Compute the mechanism emission: E_m = P_m * E_b
            let mech_emission: I96F32 = mech_proportion.saturating_mul(block_emission);
            log::debug!(
                "Mechanism emission (E_m) for mechanism ID {:?}: {:?}",
                mechid,
                mech_emission
            );
            // 3.6: Calculate subnet's proportion of mechanism TAO: P_s = T_s / T_m
            let subnet_proportion: I96F32 = subnet_tao
                .checked_div(mech_tao)
                .unwrap_or(I96F32::from_num(0));
            log::debug!(
                "Subnet proportion (P_s) for netuid {:?}: {:?}",
                netuid,
                subnet_proportion
            );
            // 3.7: Calculate subnet's TAO emission: E_s = P_s * E_m
            let tao_in: u64 = mech_emission
                .checked_mul(subnet_proportion)
                .unwrap_or(I96F32::from_num(0))
                .to_num::<u64>();
            log::debug!(
                "Subnet TAO emission (E_s) for netuid {:?}: {:?}",
                netuid,
                tao_in
            );
            // 3.8: Store the subnet TAO emission.
            *tao_in_map.entry(*netuid).or_insert(0) = tao_in;
            // 3.9: Store the block emission for this subnet for chain storage.
            EmissionValues::<T>::insert(*netuid, tao_in);
        }

        // --- 4. Distribute subnet emission into subnets based on mechanism type.
        for netuid in subnets.iter() {
            // Do not emit into root network.
            if *netuid == 0 {
                continue;
            }
            // 4.1. Get subnet mechanism ID
            let mechid: u16 = SubnetMechanism::<T>::get(*netuid);
            log::debug!("{:?} - mechid: {:?}", netuid, mechid);
            // 4.2: Get the subnet emission TAO.
            let subnet_emission: u64 = *tao_in_map.get(netuid).unwrap_or(&0);
            log::debug!("{:?} subnet_emission: {:?}", netuid, subnet_emission);
            if mechid == 0 {
                // The mechanism is Stable (FOR TESTING PURPOSES ONLY)
                // 4.2.1 Increase Tao in the subnet "reserves" unconditionally.
                SubnetTAO::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(subnet_emission)
                });
                // 4.2.2 Increase total stake across all subnets.
                TotalStake::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
                // 4.2.3 Increase total issuance of Tao.
                TotalIssuance::<T>::mutate(|total| *total = total.saturating_add(subnet_emission));
                // 4.2.4 Increase this subnet pending emission.
                PendingEmission::<T>::mutate(*netuid, |total| {
                    *total = total.saturating_add(subnet_emission)
                });
                // 4.2.5 Go to next subnet.
                continue;
            }
            // Get the total_alpha_emission for the block
            let alpha_block_emission: u64 =
                Self::get_block_emission_for_issuance(Self::get_alpha_issuance(*netuid))
                    .unwrap_or(0);

            // Compute emission into pool.
            let (tao_in_emission, alpha_in_emission, alpha_out_emission): (u64, u64, u64) =
                Self::get_dynamic_tao_emission(*netuid, subnet_emission, alpha_block_emission);

            // Set state vars.
            SubnetTaoInEmission::<T>::insert(*netuid, tao_in_emission);
            SubnetAlphaInEmission::<T>::insert(*netuid, alpha_in_emission);
            SubnetAlphaOutEmission::<T>::insert(*netuid, alpha_out_emission);

            // Increase counters.
            SubnetAlphaIn::<T>::mutate(*netuid, |total| {
                *total = total.saturating_add(alpha_in_emission);
                log::debug!("Injected alpha_in into SubnetAlphaIn: {:?}", *total);
            });
            SubnetAlphaOut::<T>::mutate(*netuid, |total| {
                *total = total.saturating_add(alpha_out_emission);
                log::debug!("Injected alpha_in into SubnetAlphaIn: {:?}", *total);
            });
            SubnetTAO::<T>::mutate(*netuid, |total| {
                *total = total.saturating_add(tao_in_emission);
                log::debug!("Increased Tao in SubnetTAO: {:?}", *total);
            });
            TotalStake::<T>::mutate(|total| {
                *total = total.saturating_add(tao_in_emission);
                log::debug!("Increased TotalStake: {:?}", *total);
            });
            TotalIssuance::<T>::mutate(|total| {
                *total = total.saturating_add(tao_in_emission);
                log::debug!("Increased TotalIssuance: {:?}", *total);
            });
            // Accumulate alpha emission in pending.
            PendingEmission::<T>::mutate(*netuid, |total| {
                *total = total.saturating_add(alpha_out_emission);
            });
        }

        // --- 5. Drain pending emission through the subnet based on tempo.
        for &netuid in subnets.iter() {
            // 5.1: Pass on subnets that have not reached their tempo.
            if Self::should_run_epoch(netuid, current_block) {
                if let Err(e) = Self::reveal_crv3_commits(netuid) {
                    log::warn!(
                        "Failed to reveal commits for subnet {} due to error: {:?}",
                        netuid,
                        e
                    );
                };

                // Restart counters.
                BlocksSinceLastStep::<T>::insert(netuid, 0);
                LastMechansimStepBlock::<T>::insert(netuid, current_block);

                // 5.2.1 Get and drain the subnet pending emission.
                let pending_emission: u64 = PendingEmission::<T>::get(netuid);
                PendingEmission::<T>::insert(netuid, 0);

                // 5.2.4 Drain pending root divs, alpha emission, and owner cut.
                Self::drain_pending_emission(netuid, pending_emission);
            } else {
                // Increment
                BlocksSinceLastStep::<T>::mutate(netuid, |total| *total = total.saturating_add(1));
            }
        }
    }

    pub fn drain_pending_emission(netuid: u16, mut emission: u64) {
        log::debug!(
            "Draining pending alpha emission for netuid {:?} emission {:?}",
            netuid,
            emission,
        );
        let zero: I96F32 = I96F32::from_num(0.0);

        // Check for existence of owner cold/hot pair and distribute emission directly to them.
        if let Ok(owner_coldkey) = SubnetOwner::<T>::try_get(netuid) {
            if let Ok(owner_hotkey) = SubnetOwnerHotkey::<T>::try_get(netuid) {
                // Calculate the owner cut.
                let owner_cut: u64 = I96F32::from_num(emission)
                    .saturating_mul(Self::get_float_subnet_owner_cut())
                    .to_num::<u64>();
                log::debug!("Owner cut for netuid {:?}: {:?}", netuid, owner_cut);

                // Subtract the owner cut from the emission
                emission = emission.saturating_sub(owner_cut);

                // Compute and remove the owner cut.
                // Increase stake for both coldkey and hotkey on the subnet
                Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                    &owner_hotkey,
                    &owner_coldkey,
                    netuid,
                    owner_cut,
                );
                log::debug!("Distributed owner cut for netuid {:?} to owner_hotkey {:?} and owner_coldkey {:?}", netuid, owner_hotkey, owner_coldkey);
            }
        }

        // Run the epoch() --> hotkey emission.
        // Needs to run on the full emission to the subnet.
        let hotkey_emission: Vec<(T::AccountId, u64, u64)> = Self::epoch(netuid, emission);
        log::debug!(
            "Hotkey emission for netuid {:?}: {:?}",
            netuid,
            hotkey_emission
        );

        // Precompute all parent dividends.
        let mut all_parent_dividends: Vec<(T::AccountId, u64)> = vec![];
        for (hotkey, _, dividends) in &hotkey_emission {
            all_parent_dividends
                .extend(Self::get_dividends_distribution(hotkey, netuid, *dividends));
        }

        // Pay out all mining incentives.
        for (hotkey, incentive, _) in &hotkey_emission {
            // First automatically distribute mining emission.
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                &Owner::<T>::get(hotkey),
                netuid,
                *incentive,
            );
        }

        // Actually perform the distribution to parents.
        let _ = AlphaDividendsPerSubnet::<T>::clear_prefix(netuid, u32::MAX, None);
        for (parent, divs) in all_parent_dividends.iter() {
            // Set of values.
            let mut rem_divs: I96F32 = I96F32::from_num(*divs);
            let owner: T::AccountId = Owner::<T>::get(parent.clone());

            // Remove the hotkey take straight off the top.
            let take: I96F32 = Self::get_hotkey_take_float(parent).saturating_mul(rem_divs);
            rem_divs = rem_divs.saturating_sub(take);
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                parent,
                &owner,
                netuid,
                take.to_num::<u64>(),
            );

            // Get the hotkey root TAO.
            let root_tao: I96F32 = I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(
                parent,
                Self::get_root_netuid(),
            ));
            let root_alpha: I96F32 = root_tao.saturating_mul(Self::get_tao_weight());
            let local_alpha: I96F32 =
                I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(parent, netuid));
            let total_alpha: I96F32 = root_alpha.saturating_add(local_alpha);

            // Compute alpha and root proportions.
            let local_prop: I96F32 = local_alpha.checked_div(total_alpha).unwrap_or(zero);
            let root_prop: I96F32 = root_alpha.checked_div(total_alpha).unwrap_or(zero);

            // Compute alpha divs
            let local_divs: I96F32 = rem_divs.saturating_mul(local_prop);
            let root_divs: I96F32 = rem_divs.saturating_mul(root_prop);

            // Pay out alpha divs.
            Self::increase_stake_for_hotkey_on_subnet(parent, netuid, local_divs.to_num::<u64>());
            // Add claimable
            Self::increase_root_claimable_for_hotkey_and_subnet(
                parent,
                netuid,
                root_divs.to_num::<u64>(),
            );

            AlphaDividendsPerSubnet::<T>::mutate(netuid, parent.clone(), |total| {
                *total = total.saturating_add(rem_divs.to_num::<u64>());
            });
        }
    }

    /// Returns the self contribution of a hotkey on a subnet.
    /// This is the portion of the hotkey's stake that is provided by itself, and not delegated to other hotkeys.
    pub fn get_self_contribution(hotkey: &T::AccountId, netuid: u16) -> u64 {
        // Get all childkeys for this hotkey.
        let childkeys = Self::get_children(hotkey, netuid);
        let mut remaining_proportion: I96F32 = I96F32::saturating_from_num(1.0);
        for (proportion, _) in childkeys {
            remaining_proportion = remaining_proportion.saturating_sub(
                I96F32::saturating_from_num(proportion) // Normalize
                    .safe_div(I96F32::saturating_from_num(u64::MAX)),
            );
        }

        // Get TAO weight
        let tao_weight: I96F32 = Self::get_tao_weight();

        // Get the hotkey's stake including weight
        let root_stake: I96F32 = I96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(
            hotkey,
            Self::get_root_netuid(),
        ));
        let alpha_stake: I96F32 =
            I96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, netuid));

        // Calculate the
        let alpha_contribution: I96F32 = alpha_stake.saturating_mul(remaining_proportion);
        let root_contribution: I96F32 = root_stake
            .saturating_mul(remaining_proportion)
            .saturating_mul(tao_weight);
        let combined_contribution: I96F32 = alpha_contribution.saturating_add(root_contribution);

        // Return the combined contribution as a u64
        combined_contribution.saturating_to_num::<u64>()
    }

    /// Returns a list of tuples for each parent associated with this hotkey including self
    /// Each tuples contains the dividends owed to that hotkey given their parent proportion
    /// The hotkey child take proportion is removed from this and added to the tuples for self.
    /// The hotkey also gets a portion based on its own stake contribution, this is added to the childkey take.
    ///
    /// # Arguments
    /// * `hotkye` - The hotkey to distribute out from.
    /// * `netuid` - The netuid we are computing on.
    /// * `dividends` - the dividends to distribute.
    ///
    /// # Returns
    /// * dividend_tuples: `Vec<(T::AccountId, u64)>` - Vector of (hotkey, divs) for each parent including self.
    ///
    pub fn get_dividends_distribution(
        hotkey: &T::AccountId,
        netuid: u16,
        dividends: u64,
    ) -> Vec<(T::AccountId, u64)> {
        // hotkey dividends.
        let mut dividend_tuples: Vec<(T::AccountId, u64)> = vec![];

        // Calculate the hotkey's share of the validator emission based on its childkey take
        let validating_emission: I96F32 = I96F32::saturating_from_num(dividends);
        let childkey_take_proportion: I96F32 =
            I96F32::saturating_from_num(Self::get_childkey_take(hotkey, netuid))
                .safe_div(I96F32::saturating_from_num(u16::MAX));
        log::debug!(
            "Childkey take proportion: {:?} for hotkey {:?}",
            childkey_take_proportion,
            hotkey
        );
        // NOTE: Only the validation emission should be split amongst parents.

        // Reserve childkey take
        let child_emission_take: I96F32 = childkey_take_proportion
            .saturating_mul(I96F32::saturating_from_num(validating_emission));
        let remaining_emission: I96F32 = validating_emission.saturating_sub(child_emission_take);
        log::debug!(
            "Child emission take: {:?} for hotkey {:?}",
            child_emission_take,
            hotkey
        );
        log::debug!(
            "Remaining emission: {:?} for hotkey {:?}",
            remaining_emission,
            hotkey
        );

        // Initialize variables to track emission distribution
        let mut to_parents: u64 = 0;

        // Initialize variables to calculate total stakes from parents
        let mut total_contribution: I96F32 = I96F32::saturating_from_num(0);
        let mut parent_contributions: Vec<(T::AccountId, I96F32)> = Vec::new();

        // Get the weights for root and alpha stakes in emission distribution
        let tao_weight: I96F32 = Self::get_tao_weight();

        // Get self contribution, removing any childkey proportions.
        let self_contribution = Self::get_self_contribution(hotkey, netuid);
        log::debug!(
            "Self contribution for hotkey {:?} on netuid {:?}: {:?}",
            hotkey,
            netuid,
            self_contribution
        );
        // Add self contribution to total contribution but not to the parent contributions.
        total_contribution =
            total_contribution.saturating_add(I96F32::saturating_from_num(self_contribution));

        // Calculate total root and alpha (subnet-specific) stakes from all parents
        for (proportion, parent) in Self::get_parents(hotkey, netuid) {
            // Convert the parent's stake proportion to a fractional value
            let parent_proportion: I96F32 = I96F32::saturating_from_num(proportion)
                .safe_div(I96F32::saturating_from_num(u64::MAX));

            // Get the parent's root and subnet-specific (alpha) stakes
            let parent_root: I96F32 = I96F32::saturating_from_num(
                Self::get_stake_for_hotkey_on_subnet(&parent, Self::get_root_netuid()),
            );
            let parent_alpha: I96F32 =
                I96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(&parent, netuid));

            // Calculate the parent's contribution to the hotkey's stakes
            let parent_alpha_contribution: I96F32 = parent_alpha.saturating_mul(parent_proportion);
            let parent_root_contribution: I96F32 = parent_root
                .saturating_mul(parent_proportion)
                .saturating_mul(tao_weight);
            let combined_contribution: I96F32 =
                parent_alpha_contribution.saturating_add(parent_root_contribution);

            // Add to the total stakes
            total_contribution = total_contribution.saturating_add(combined_contribution);
            // Store the parent's contributions for later use
            parent_contributions.push((parent.clone(), combined_contribution));
            log::debug!(
                "Parent contribution for hotkey {:?} from parent {:?}: {:?}",
                hotkey,
                parent,
                combined_contribution
            );
        }

        // Distribute emission to parents based on their contributions.
        // Deduct childkey take from parent contribution.
        for (parent, contribution) in parent_contributions {
            // Sum up the total emission for this parent
            let emission_factor: I96F32 = contribution
                .checked_div(total_contribution)
                .unwrap_or(I96F32::saturating_from_num(0));
            let parent_emission: u64 =
                (remaining_emission.saturating_mul(emission_factor)).saturating_to_num::<u64>();

            // Add the parent's emission to the distribution list
            dividend_tuples.push((parent, parent_emission));

            // Keep track of total emission distributed to parents
            to_parents = to_parents.saturating_add(parent_emission);
        }
        // Calculate the final emission for the hotkey itself.
        // This includes the take left from the parents and the self contribution.
        let child_emission = remaining_emission
            .saturating_add(child_emission_take)
            .saturating_to_num::<u64>()
            .saturating_sub(to_parents);

        // Add the hotkey's own emission to the distribution list
        dividend_tuples.push((hotkey.clone(), child_emission));

        dividend_tuples
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
        let remainder = adjusted_block.checked_rem(tempo_plus_one).unwrap_or(0);
        (tempo as u64).saturating_sub(remainder)
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
}
