use super::*;
use alloc::collections::BTreeMap;
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


    pub fn get_root_divs_in_alpha( netuid: u16, alpha_out_emission: I96F32 ) -> I96F32 {
        // Get total TAO on root.
        let total_root_tao: I96F32 = I96F32::from_num( SubnetTAO::<T>::get( 0 ) );
        // Get total ALPHA on subnet.
        let total_alpha_issuance: I96F32 = I96F32::from_num(Self::get_alpha_issuance( netuid ));
        // Get tao_weight
        let tao_weight: I96F32 = total_root_tao.saturating_mul( Self::get_tao_weight() );
        // Get root proportional dividends.
        let root_proportion: I96F32 = tao_weight.checked_div( tao_weight.saturating_add( total_alpha_issuance ) ).unwrap_or( I96F32::from_num( 0.0 ) );
        // Get root proportion of alpha_out dividends.
        let root_divs_in_alpha: I96F32 = root_proportion.saturating_mul( alpha_out_emission ).saturating_mul( I96F32::from_num( 0.41 ) );
        // Return
        root_divs_in_alpha
    }


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
        
        // --- 4. Sum all the SubnetTAO associated with the same mechanism.
        // Mechanisms get emission based on the proportion of TAO across all their subnets
        let mut total_active_tao: I96F32 = I96F32::from_num(0);
        let mut mechanism_tao: BTreeMap<u16, I96F32> = BTreeMap::new();
        for netuid in subnets.iter() {
            if *netuid == 0 { continue; } // Skip root network
            let mechid = SubnetMechanism::<T>::get(*netuid);
            let subnet_tao = I96F32::from_num(SubnetTAO::<T>::get(*netuid));
            let new_subnet_tao = subnet_tao.saturating_add(*mechanism_tao.entry(mechid).or_insert(I96F32::from_num(0)));
            *mechanism_tao.entry(mechid).or_insert(I96F32::from_num(0)) = new_subnet_tao;
            total_active_tao = total_active_tao.saturating_add(subnet_tao);
        }
        log::debug!("Mechanism TAO sums: {:?}", mechanism_tao);

        // --- 5. Compute subnet emission values (amount of tao inflation this block).
        let mut tao_in_map: BTreeMap<u16, u64> = BTreeMap::new();
        for netuid in subnets.iter() {
            // Do not emit into root network.
            if *netuid == 0 {continue;}
            // 5.1: Get subnet mechanism ID
            let mechid: u16 = SubnetMechanism::<T>::get(*netuid);
            log::debug!("Netuid: {:?}, Mechanism ID: {:?}", netuid, mechid);
            // 5.2: Get subnet TAO (T_s)
            let subnet_tao: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(*netuid));
            log::debug!("Subnet TAO (T_s) for netuid {:?}: {:?}", netuid, subnet_tao);
            // 5.3: Get the denominator as the sum of all TAO associated with a specific mechanism (T_m)
            let mech_tao: I96F32 = *mechanism_tao.get(&mechid).unwrap_or(&I96F32::from_num(0));
            log::debug!("Mechanism TAO (T_m) for mechanism ID {:?}: {:?}", mechid, mech_tao);
            // 5.4: Compute the mechanism emission proportion: P_m = T_m / T_total
            let mech_proportion: I96F32 = mech_tao.checked_div(total_active_tao).unwrap_or(I96F32::from_num(0));
            log::debug!("Mechanism proportion (P_m) for mechanism ID {:?}: {:?}", mechid, mech_proportion);
            // 5.5: Compute the mechanism emission: E_m = P_m * E_b
            let mech_emission: I96F32 = mech_proportion.saturating_mul(block_emission);
            log::debug!("Mechanism emission (E_m) for mechanism ID {:?}: {:?}", mechid, mech_emission);
            // 5.6: Calculate subnet's proportion of mechanism TAO: P_s = T_s / T_m
            let subnet_proportion: I96F32 = subnet_tao.checked_div(mech_tao).unwrap_or(I96F32::from_num(0));
            log::debug!("Subnet proportion (P_s) for netuid {:?}: {:?}", netuid, subnet_proportion);
            // 5.7: Calculate subnet's TAO emission: E_s = P_s * E_m
            let tao_in: u64 = mech_emission.checked_mul(subnet_proportion).unwrap_or(I96F32::from_num(0)).to_num::<u64>();
            log::debug!("Subnet TAO emission (E_s) for netuid {:?}: {:?}", netuid, tao_in);
            // 5.8: Store the subnet TAO emission. 
            *tao_in_map.entry(*netuid).or_insert(0) = tao_in;
            // 5.9: Store the block emission for this subnet for chain storage.
            EmissionValues::<T>::insert(*netuid, tao_in);
        }
        
        // --- 6. Distribute subnet emission into subnets based on mechanism type.
        for netuid in subnets.iter() {
            // Do not emit into root network.
            if *netuid == 0 {continue;}
            // 6.1. Get subnet mechanism ID
            let mechid: u16 = SubnetMechanism::<T>::get(*netuid);
            log::debug!("{:?} - mechid: {:?}", netuid, mechid);
            // 6.2: Get the subnet emission TAO.
            let subnet_emission: u64 = *tao_in_map.get(netuid).unwrap_or(&0);
            log::debug!("{:?} subnet_emission: {:?}", netuid, subnet_emission );
            if mechid == 0 {
                // The mechanism is Stable (FOR TESTING PURPOSES ONLY)
                // 6.2.1 Increase Tao in the subnet "reserves" unconditionally.
                SubnetTAO::<T>::mutate(*netuid, |total| { *total = total.saturating_add( subnet_emission ) });
                // 6.2.2 Increase total stake across all subnets.
                TotalStake::<T>::mutate(|total| *total = total.saturating_add( subnet_emission ));
                // 6.2.3 Increase total issuance of Tao.
                TotalIssuance::<T>::mutate(|total| *total = total.saturating_add( subnet_emission ));
                // 6.2.4 Increase this subnet pending emission.
                PendingEmission::<T>::mutate(*netuid, |total| { *total = total.saturating_add( subnet_emission ) });
                // 6.2.5 Go to next subnet.
                continue
            }
            // Get the total_alpha_emission for the block
            let alpha_block_emission: u64 = Self::get_block_emission_for_issuance( Self::get_alpha_issuance( *netuid ) ).unwrap_or( 0 );

            // Compute emission into pool.
            let (tao_in_emission, alpha_in_emission, alpha_out_emission):( u64, u64, u64 ) = Self::get_dynamic_tao_emission(
                *netuid, 
                subnet_emission, 
                alpha_block_emission
            );

            // Set state vars.
            SubnetTaoInEmission::<T>::insert( *netuid, tao_in_emission );
            SubnetAlphaInEmission::<T>::insert( *netuid, alpha_in_emission ); 
            SubnetAlphaOutEmission::<T>::insert( *netuid, alpha_out_emission ); 

            // Increase counters.
            SubnetAlphaIn::<T>::mutate(*netuid, |total| { 
                *total = total.saturating_add( alpha_in_emission );
                log::debug!("Injected alpha_in into SubnetAlphaIn: {:?}", *total);
            });
            SubnetAlphaOut::<T>::mutate(*netuid, |total| { 
                *total = total.saturating_add( alpha_out_emission );
                log::debug!("Injected alpha_in into SubnetAlphaIn: {:?}", *total);
            });
            SubnetTAO::<T>::mutate(*netuid, |total| { 
                *total = total.saturating_add( tao_in_emission );
                log::debug!("Increased Tao in SubnetTAO: {:?}", *total);
            });
            TotalStake::<T>::mutate(|total| { 
                *total = total.saturating_add( tao_in_emission );
                log::debug!("Increased TotalStake: {:?}", *total);
            });
            TotalIssuance::<T>::mutate(|total| { 
                *total = total.saturating_add( tao_in_emission );
                log::debug!("Increased TotalIssuance: {:?}", *total);
            });

            // Get proportion of alpha out emission as root divs.
            let root_emission_in_alpha: I96F32 = Self::get_root_divs_in_alpha( *netuid, I96F32::from_num(alpha_out_emission) );
            // Subtract root divs from alpha divs.
            let pending_alpha_emission: I96F32 = I96F32::from_num( alpha_out_emission ).saturating_sub( root_emission_in_alpha );
            // Sell root emission through the pool.
            let root_emission_in_tao: u64 = Self::swap_alpha_for_tao( *netuid, root_emission_in_alpha.to_num::<u64>() );
            SubnetAlphaEmissionSell::<T>::insert( *netuid, root_emission_in_alpha.to_num::<u64>() ); 
            // Accumulate root divs for subnet.
            PendingRootDivs::<T>::mutate( *netuid, |total| { 
                *total = total.saturating_add( root_emission_in_tao );
            });
            // Accumulate alpha emission in pending.
            PendingEmission::<T>::mutate(*netuid, |total| { 
                *total = total.saturating_add( pending_alpha_emission.to_num::<u64>() );
            });
        }

        // --- 7. Drain pending emission through the subnet based on tempo.
        for &netuid in subnets.iter() {
            // 7.1: Pass on subnets that have not reached their tempo.
            if Self::should_run_epoch(netuid, current_block) {
                // Restart counters.
                BlocksSinceLastStep::<T>::insert( netuid, 0 );
                LastMechansimStepBlock::<T>::insert( netuid, current_block );  
                // Drain pending root divs and alpha emission.      
                Self::drain_pending_emission( netuid ); 
            } else {
                // Increment 
                BlocksSinceLastStep::<T>::mutate( netuid,|total| *total = total.saturating_add(1) );
            }
        }
    }

    pub fn drain_pending_emission( netuid: u16 ) {
        // 7.2 Get and drain the subnet pending emission.
        let alpha_out: u64 = PendingEmission::<T>::get(netuid);
        log::debug!("Draining pending emission for netuid {:?}: {:?}", netuid, alpha_out);
        PendingEmission::<T>::insert(netuid, 0);
    
        // 7.4 Distribute the 18% owner cut.
        let owner_cut: u64 = I96F32::from_num(alpha_out).saturating_mul(Self::get_float_subnet_owner_cut()).to_num::<u64>();
        log::debug!("Owner cut for netuid {:?}: {:?}", netuid, owner_cut);
        // 7.4.1: Check for existence of owner cold/hot pair and distribute emission directly to them.
        if let Ok(owner_coldkey) = SubnetOwner::<T>::try_get(netuid) {
            if let Ok(owner_hotkey) = SubnetOwnerHotkey::<T>::try_get(netuid) {
                // Increase stake for both coldkey and hotkey on the subnet
                Self::increase_stake_for_hotkey_and_coldkey_on_subnet(&owner_hotkey, &owner_coldkey, netuid, owner_cut);
                log::debug!("Distributed owner cut for netuid {:?} to owner_hotkey {:?} and owner_coldkey {:?}", netuid, owner_hotkey, owner_coldkey);
            }
        }
        let remaining_emission: u64 = alpha_out.saturating_sub(owner_cut);
        log::debug!("Remaining emission for netuid {:?}: {:?}", netuid, remaining_emission);
        
        // 7.5 Run the epoch() --> hotkey emission.
        let hotkey_emission: Vec<(T::AccountId, u64, u64)> = Self::epoch(netuid, remaining_emission);
        log::debug!("Hotkey emission for netuid {:?}: {:?}", netuid, hotkey_emission);

        // 7.6 Pay out the hotkey alpha dividends.
        // First clear the netuid from HotkeyDividends
        let mut total_root_alpha_divs: u64 = 0;
        let mut root_alpha_divs: BTreeMap<T::AccountId, u64> = BTreeMap::new();
        let _ = HotkeyDividendsPerSubnet::<T>::clear_prefix(netuid, u32::MAX, None);
        for (hotkey, incentive, dividends) in hotkey_emission {
            log::debug!("Processing hotkey {:?} with incentive {:?} and dividends {:?}", hotkey, incentive, dividends);

            // 7.6.1: Distribute mining incentive immediately.
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet( &hotkey.clone(), &Owner::<T>::get( hotkey.clone() ), netuid, incentive );
            log::debug!("Distributed mining incentive for hotkey {:?} on netuid {:?}: {:?}", hotkey, netuid, incentive);

            // 7.6.2: Get dividend tuples for parents and self based on childkey relationships and child-take.
            let dividend_tuples: Vec<(T::AccountId, u64)> = Self::get_parent_dividends(
                &hotkey,
                netuid,
                dividends,
            );
            log::debug!("Dividend tuples for hotkey {:?} on netuid {:?}: {:?}", hotkey, netuid, dividend_tuples);

            // 7.6.3 Pay out dividends to hotkeys based on the local vs root proportion.
            for (hotkey_j, divs_j) in dividend_tuples {
                log::debug!("Processing dividend for hotkey {:?} to hotkey {:?}: {:?}", hotkey, hotkey_j, divs_j);

                // 7.6.3.1: Remove the hotkey take straight off the top.
                let take_prop: I96F32 = I96F32::from_num(Self::get_hotkey_take( &hotkey_j )).checked_div( I96F32::from_num(u16::MAX) ).unwrap_or( I96F32::from_num( 0.0 ) );
                let validator_take: I96F32 = take_prop.saturating_mul(I96F32::from_num(divs_j));
                let rem_divs_j: I96F32 = I96F32::from_num(divs_j).saturating_sub(validator_take);
                log::debug!("Validator take for hotkey {:?}: {:?}, remaining dividends: {:?}", hotkey_j, validator_take, rem_divs_j);

                // 7.6.3.2: Distribute validator take automatically.
                Self::increase_stake_for_hotkey_and_coldkey_on_subnet( &hotkey_j, &Owner::<T>::get( hotkey_j.clone() ), netuid, validator_take.to_num::<u64>() );
                log::debug!("Distributed validator take for hotkey {:?} on netuid {:?}: {:?}", hotkey_j, netuid, validator_take.to_num::<u64>());

                // 7.6.3.3: Get the local alpha and root alpha.
                let hotkey_tao: I96F32 = I96F32::from_num( Self::get_stake_for_hotkey_on_subnet( &hotkey, Self::get_root_netuid() ) );
                let hotkey_tao_as_alpha: I96F32 = hotkey_tao.saturating_mul( Self::get_tao_weight() );
                let hotkey_alpha = I96F32::from_num(Self::get_stake_for_hotkey_on_subnet( &hotkey, netuid ));
                log::debug!("Hotkey tao for hotkey {:?} on root netuid: {:?}, hotkey tao as alpha: {:?}, hotkey alpha: {:?}", hotkey, hotkey_tao, hotkey_tao_as_alpha, hotkey_alpha);

                // 7.6.3.3 Compute alpha and root proportions.
                let alpha_prop: I96F32 = hotkey_alpha.checked_div( hotkey_alpha.saturating_add(hotkey_tao_as_alpha) ).unwrap_or( I96F32::from_num( 0.0 ) );
                let root_prop: I96F32 = hotkey_tao_as_alpha.checked_div( hotkey_alpha.saturating_add(hotkey_tao_as_alpha) ).unwrap_or( I96F32::from_num( 0.0 ) );
                log::debug!("Alpha proportion: {:?}, root proportion: {:?}", alpha_prop, root_prop);

                // 7.6.3.5: Compute alpha and root dividends
                let root_divs: I96F32 = rem_divs_j.saturating_mul( root_prop );
                log::debug!("Alpha dividends: {:?}, root dividends: {:?}", rem_divs_j, root_divs);

                // 7.6.3.6. Store the root alpha divs under hotkey_j
                root_alpha_divs.entry(hotkey_j.clone())
                    .and_modify(|e| *e = e.saturating_add(root_divs.to_num::<u64>()))
                    .or_insert(root_divs.to_num::<u64>());
                total_root_alpha_divs = total_root_alpha_divs.saturating_add(root_divs.to_num::<u64>());
                log::debug!("Stored root alpha dividends for hotkey {:?}: {:?}", hotkey_j, root_divs.to_num::<u64>());

                // 7.6.3.7: Distribute the alpha divs to the hotkey.
                Self::increase_stake_for_hotkey_on_subnet( &hotkey_j, netuid, rem_divs_j.to_num::<u64>() );
                log::debug!("Distributed alpha dividends for hotkey {:?} on netuid {:?}: {:?}", hotkey_j, netuid, rem_divs_j.to_num::<u64>() );

                // 7.6.3.9: Record dividends for this hotkey on this subnet.
                HotkeyDividendsPerSubnet::<T>::mutate( netuid, hotkey_j.clone(), |divs| {
                    *divs = divs.saturating_add(divs_j);
                });
                log::debug!("Recorded dividends for hotkey {:?} on netuid {:?}: {:?}", hotkey_j, netuid, divs_j);
            }
        }
        // For all the root alpha divs give this proportion of the swapped tao to the root participants. 
        let _ = RootDividendsPerSubnet::<T>::clear_prefix(netuid, u32::MAX, None);
        let total_root_divs_to_distribute = PendingRootDivs::<T>::get( netuid );
        PendingRootDivs::<T>::insert(netuid, 0);
        for (hotkey_j, root_divs) in root_alpha_divs.iter() {
            let proportion: I96F32 = I96F32::from_num(*root_divs).checked_div(I96F32::from_num(total_root_alpha_divs)).unwrap_or(I96F32::from_num(0));
            let root_divs_to_pay: u64 = proportion.saturating_mul(I96F32::from_num(total_root_divs_to_distribute)).to_num::<u64>();
            log::debug!("Proportion for hotkey {:?}: {:?}, root_divs_to_pay: {:?}", hotkey_j, proportion, root_divs_to_pay);

            // Pay the tao to the hotkey on netuid 0
            Self::increase_stake_for_hotkey_on_subnet(hotkey_j, Self::get_root_netuid(), root_divs_to_pay);
            log::debug!("Paid tao to hotkey {:?} on root netuid: {:?}", hotkey_j, root_divs_to_pay);

            // 7.6.3.9: Record dividends for this hotkey on this subnet.
            RootDividendsPerSubnet::<T>::mutate( netuid, hotkey_j.clone(), |divs| {
                *divs = divs.saturating_add(root_divs_to_pay);
            });
        }
    }

    /// Returns a list of tuples for each parent associated with this hotkey including self
    /// Each tuples contains the dividends owed to that hotkey given their parent proportion
    /// The hotkey child take proportion is removed from this and added to the tuples for self.
    ///
    /// # Arguments
    /// * `hotkye` - The hotkey to distribute out from.
    /// * `netuid` - The netuid we are computing on.
    /// * `dividends` - the dividends to distribute.
    ///
    /// # Returns
    /// * dividend_tuples: `Vec<(T::AccountId, u64)>` - Vector of (hotkey, divs) for each parent including self.
    ///
    pub fn get_parent_dividends(
        hotkey: &T::AccountId,
        netuid: u16,
        dividends: u64,
    ) -> Vec<(T::AccountId, u64)> {

        // hotkey dividends.
        let mut dividend_tuples: Vec<(T::AccountId, u64)> = vec![];

        // Calculate the hotkey's share of the validator emission based on its childkey take
        let validating_emission: I96F32 = I96F32::from_num(dividends);
        let childkey_take_proportion: I96F32 =
            I96F32::from_num(Self::get_childkey_take(hotkey, netuid))
                .saturating_div(I96F32::from_num(u16::MAX));
        let mut total_childkey_take: u64 = 0;
        // NOTE: Only the validation emission should be split amongst parents.

        // Initialize variables to track emission distribution
        let mut to_parents: u64 = 0;

        // Initialize variables to calculate total stakes from parents
        let mut total_contribution: I96F32 = I96F32::from_num(0);
        let mut contributions: Vec<(T::AccountId, I96F32)> = Vec::new();

        // Get the weights for root and alpha stakes in emission distribution
        let tao_weight: I96F32 = Self::get_tao_weight();

        // Calculate total root and alpha (subnet-specific) stakes from all parents
        for (proportion, parent) in Self::get_parents(hotkey, netuid) {
          
            // Convert the parent's stake proportion to a fractional value
            let parent_proportion: I96F32 = I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));

            // Get the parent's root and subnet-specific (alpha) stakes
            let parent_root: I96F32 = I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(&parent, Self::get_root_netuid()));
            let parent_alpha: I96F32 = I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(&parent, netuid));

            // Calculate the parent's contribution to the hotkey's stakes
            let parent_alpha_contribution: I96F32 = parent_alpha.saturating_mul(parent_proportion);
            let parent_root_contribution: I96F32 = parent_root.saturating_mul(parent_proportion).saturating_mul( tao_weight );
            let combined_contribution: I96F32 = parent_alpha_contribution.saturating_add(parent_root_contribution);

            // Add to the total stakes
            total_contribution = total_contribution.saturating_add(combined_contribution);
            // Store the parent's contributions for later use
            contributions.push((
                parent.clone(),
                combined_contribution,
            ));
        }

        // Distribute emission to parents based on their contributions
        for (parent, contribution) in contributions {
            // Sum up the total emission for this parent
            let emission_factor: I96F32 = contribution.checked_div(total_contribution).unwrap_or(I96F32::from_num(0));
            let total_emission: u64 = ( validating_emission.saturating_mul(emission_factor) ).to_num::<u64>();

            // Reserve childkey take
            let child_emission_take: u64 = childkey_take_proportion.saturating_mul(I96F32::from_num(total_emission)).to_num::<u64>();
            total_childkey_take = total_childkey_take.saturating_add(child_emission_take);
            let parent_total_emission = total_emission.saturating_sub(child_emission_take);

            // Add the parent's emission to the distribution list
            dividend_tuples.push((parent, parent_total_emission));

            // Keep track of total emission distributed to parents
            to_parents = to_parents.saturating_add(parent_total_emission);
        }
        // Calculate the final emission for the hotkey itself
        let final_hotkey_emission = validating_emission.to_num::<u64>().saturating_sub(to_parents);

        // Add the hotkey's own emission to the distribution list
        dividend_tuples.push((hotkey.clone(), final_hotkey_emission));

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
