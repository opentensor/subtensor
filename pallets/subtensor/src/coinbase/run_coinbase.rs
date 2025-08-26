use super::*;
use alloc::collections::BTreeMap;
use safe_math::*;
use substrate_fixed::types::U96F32;
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::SwapHandler;

// Distribute dividends to each hotkey
macro_rules! asfloat {
    ($val:expr) => {
        U96F32::saturating_from_num($val)
    };
}

macro_rules! tou64 {
    ($val:expr) => {
        $val.saturating_to_num::<u64>()
    };
}

impl<T: Config> Pallet<T> {
    pub fn run_coinbase(block_emission: U96F32) {
        // --- 0. Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();
        log::debug!("Current block: {current_block:?}");

        // --- 1. Get all netuids (filter out root)
        let subnets: Vec<NetUid> = Self::get_all_subnet_netuids()
            .into_iter()
            .filter(|netuid| *netuid != NetUid::ROOT)
            .collect();
        log::debug!("All subnet netuids: {subnets:?}");
        // Filter out subnets with no first emission block number.
        let subnets_to_emit_to: Vec<NetUid> = subnets
            .clone()
            .into_iter()
            .filter(|netuid| FirstEmissionBlockNumber::<T>::get(*netuid).is_some())
            .collect();
        log::debug!("Subnets to emit to: {subnets_to_emit_to:?}");

        // --- 2. Get sum of tao reserves ( in a later version we will switch to prices. )
        let mut total_moving_prices = U96F32::saturating_from_num(0.0);
        // Only get price EMA for subnets that we emit to.
        for netuid_i in subnets_to_emit_to.iter() {
            // Get and update the moving price of each subnet adding the total together.
            total_moving_prices =
                total_moving_prices.saturating_add(Self::get_moving_alpha_price(*netuid_i));
        }
        log::debug!("total_moving_prices: {total_moving_prices:?}");

        // --- 3. Get subnet terms (tao_in, alpha_in, and alpha_out)
        // Computation is described in detail in the dtao whitepaper.
        let mut tao_in: BTreeMap<NetUid, U96F32> = BTreeMap::new();
        let mut alpha_in: BTreeMap<NetUid, U96F32> = BTreeMap::new();
        let mut alpha_out: BTreeMap<NetUid, U96F32> = BTreeMap::new();
        let mut is_subsidized: BTreeMap<NetUid, bool> = BTreeMap::new();
        // Only calculate for subnets that we are emitting to.
        for netuid_i in subnets_to_emit_to.iter() {
            // Get subnet price.
            let price_i = T::SwapInterface::current_alpha_price((*netuid_i).into());
            log::debug!("price_i: {price_i:?}");
            // Get subnet TAO.
            let moving_price_i: U96F32 = Self::get_moving_alpha_price(*netuid_i);
            log::debug!("moving_price_i: {moving_price_i:?}");
            // Emission is price over total.
            let default_tao_in_i: U96F32 = block_emission
                .saturating_mul(moving_price_i)
                .checked_div(total_moving_prices)
                .unwrap_or(asfloat!(0.0));
            log::debug!("default_tao_in_i: {default_tao_in_i:?}");
            // Get alpha_emission total
            let alpha_emission_i: U96F32 = asfloat!(
                Self::get_block_emission_for_issuance(Self::get_alpha_issuance(*netuid_i).into())
                    .unwrap_or(0)
            );
            log::debug!("alpha_emission_i: {alpha_emission_i:?}");

            // Get initial alpha_in
            let mut alpha_in_i: U96F32;
            let mut tao_in_i: U96F32;
            let tao_in_ratio: U96F32 = default_tao_in_i.safe_div_or(
                U96F32::saturating_from_num(block_emission),
                U96F32::saturating_from_num(0.0),
            );
            if price_i < tao_in_ratio {
                tao_in_i = price_i.saturating_mul(U96F32::saturating_from_num(block_emission));
                alpha_in_i = alpha_emission_i;
                let difference_tao: U96F32 = default_tao_in_i.saturating_sub(tao_in_i);
                // Difference becomes buy.
                let buy_swap_result = Self::swap_tao_for_alpha(
                    *netuid_i,
                    tou64!(difference_tao).into(),
                    T::SwapInterface::max_price().into(),
                    true,
                );
                if let Ok(buy_swap_result_ok) = buy_swap_result {
                    let bought_alpha = AlphaCurrency::from(buy_swap_result_ok.amount_paid_out);
                    SubnetAlphaOut::<T>::mutate(*netuid_i, |total| {
                        *total = total.saturating_sub(bought_alpha);
                    });
                }
                is_subsidized.insert(*netuid_i, true);
            } else {
                tao_in_i = default_tao_in_i;
                alpha_in_i = tao_in_i.safe_div_or(price_i, alpha_emission_i);
                is_subsidized.insert(*netuid_i, false);
            }
            log::debug!("alpha_in_i: {alpha_in_i:?}");

            // Get alpha_out.
            let mut alpha_out_i = alpha_emission_i;
            // Only emit TAO if the subnetwork allows registration.
            if !Self::get_network_registration_allowed(*netuid_i)
                && !Self::get_network_pow_registration_allowed(*netuid_i)
            {
                tao_in_i = asfloat!(0.0);
                alpha_in_i = asfloat!(0.0);
                alpha_out_i = asfloat!(0.0);
            }
            // Insert values into maps
            tao_in.insert(*netuid_i, tao_in_i);
            alpha_in.insert(*netuid_i, alpha_in_i);
            alpha_out.insert(*netuid_i, alpha_out_i);
        }
        log::debug!("tao_in: {tao_in:?}");
        log::debug!("alpha_in: {alpha_in:?}");
        log::debug!("alpha_out: {alpha_out:?}");

        // --- 4. Injection.
        // Actually perform the injection of alpha_in, alpha_out and tao_in into the subnet pool.
        // This operation changes the pool liquidity each block.
        for netuid_i in subnets_to_emit_to.iter() {
            // Inject Alpha in.
            let alpha_in_i =
                AlphaCurrency::from(tou64!(*alpha_in.get(netuid_i).unwrap_or(&asfloat!(0))));
            SubnetAlphaInEmission::<T>::insert(*netuid_i, alpha_in_i);
            SubnetAlphaIn::<T>::mutate(*netuid_i, |total| {
                *total = total.saturating_add(alpha_in_i);
            });
            // Injection Alpha out.
            let alpha_out_i =
                AlphaCurrency::from(tou64!(*alpha_out.get(netuid_i).unwrap_or(&asfloat!(0))));
            SubnetAlphaOutEmission::<T>::insert(*netuid_i, alpha_out_i);
            SubnetAlphaOut::<T>::mutate(*netuid_i, |total| {
                *total = total.saturating_add(alpha_out_i);
            });
            // Inject TAO in.
            let tao_in_i: TaoCurrency =
                tou64!(*tao_in.get(netuid_i).unwrap_or(&asfloat!(0))).into();
            SubnetTaoInEmission::<T>::insert(*netuid_i, TaoCurrency::from(tao_in_i));
            SubnetTAO::<T>::mutate(*netuid_i, |total| {
                *total = total.saturating_add(tao_in_i.into());
            });
            TotalStake::<T>::mutate(|total| {
                *total = total.saturating_add(tao_in_i.into());
            });
            TotalIssuance::<T>::mutate(|total| {
                *total = total.saturating_add(tao_in_i.into());
            });
            // Adjust protocol liquidity based on new reserves
            T::SwapInterface::adjust_protocol_liquidity(*netuid_i, tao_in_i, alpha_in_i);
        }

        // --- 5. Compute owner cuts and remove them from alpha_out remaining.
        // Remove owner cuts here so that we can properly seperate root dividends in the next step.
        // Owner cuts are accumulated and then fed to the drain at the end of this func.
        let cut_percent: U96F32 = Self::get_float_subnet_owner_cut();
        let mut owner_cuts: BTreeMap<NetUid, U96F32> = BTreeMap::new();
        for netuid_i in subnets_to_emit_to.iter() {
            // Get alpha out.
            let alpha_out_i: U96F32 = *alpha_out.get(netuid_i).unwrap_or(&asfloat!(0));
            log::debug!("alpha_out_i: {alpha_out_i:?}");
            // Calculate the owner cut.
            let owner_cut_i: U96F32 = alpha_out_i.saturating_mul(cut_percent);
            log::debug!("owner_cut_i: {owner_cut_i:?}");
            // Save owner cut.
            *owner_cuts.entry(*netuid_i).or_insert(asfloat!(0)) = owner_cut_i;
            // Save new alpha_out.
            alpha_out.insert(*netuid_i, alpha_out_i.saturating_sub(owner_cut_i));
            // Accumulate the owner cut in pending.
            PendingOwnerCut::<T>::mutate(*netuid_i, |total| {
                *total = total.saturating_add(tou64!(owner_cut_i).into());
            });
        }

        // Get total TAO on root.
        let root_tao: U96F32 = asfloat!(SubnetTAO::<T>::get(NetUid::ROOT));
        log::debug!("root_tao: {root_tao:?}");
        // Get tao_weight
        let tao_weight: U96F32 = root_tao.saturating_mul(Self::get_tao_weight());
        log::debug!("tao_weight: {tao_weight:?}");

        // --- 6. Seperate out root dividends in alpha and sell them into tao.
        // Then accumulate those dividends for later.
        for netuid_i in subnets_to_emit_to.iter() {
            // Get remaining alpha out.
            let alpha_out_i: U96F32 = *alpha_out.get(netuid_i).unwrap_or(&asfloat!(0.0));
            log::debug!("alpha_out_i: {alpha_out_i:?}");
            // Get total ALPHA on subnet.
            let alpha_issuance: U96F32 = asfloat!(Self::get_alpha_issuance(*netuid_i));
            log::debug!("alpha_issuance: {alpha_issuance:?}");
            // Get root proportional dividends.
            let root_proportion: U96F32 = tao_weight
                .checked_div(tao_weight.saturating_add(alpha_issuance))
                .unwrap_or(asfloat!(0.0));
            log::debug!("root_proportion: {root_proportion:?}");
            // Get root proportion of alpha_out dividends.
            let root_alpha: U96F32 = root_proportion
                .saturating_mul(alpha_out_i) // Total alpha emission per block remaining.
                .saturating_mul(asfloat!(0.5)); // 50% to validators.
            // Remove root alpha from alpha_out.
            log::debug!("root_alpha: {root_alpha:?}");
            // Get pending alpha as original alpha_out - root_alpha.
            let pending_alpha: U96F32 = alpha_out_i.saturating_sub(root_alpha);
            log::debug!("pending_alpha: {pending_alpha:?}");
            // Sell root emission through the pool (do not pay fees)
            let subsidized: bool = *is_subsidized.get(netuid_i).unwrap_or(&false);
            if !subsidized {
                let swap_result = Self::swap_alpha_for_tao(
                    *netuid_i,
                    tou64!(root_alpha).into(),
                    T::SwapInterface::min_price().into(),
                    true,
                );
                if let Ok(ok_result) = swap_result {
                    let root_tao: u64 = ok_result.amount_paid_out;
                    // Accumulate root divs for subnet.
                    PendingRootDivs::<T>::mutate(*netuid_i, |total| {
                        *total = total.saturating_add(root_tao.into());
                    });
                }
            }
            // Accumulate alpha emission in pending.
            PendingAlphaSwapped::<T>::mutate(*netuid_i, |total| {
                *total = total.saturating_add(tou64!(root_alpha).into());
            });
            // Accumulate alpha emission in pending.
            PendingEmission::<T>::mutate(*netuid_i, |total| {
                *total = total.saturating_add(tou64!(pending_alpha).into());
            });
        }

        // --- 7. Update moving prices after using them in the emission calculation.
        // Only update price EMA for subnets that we emit to.
        for netuid_i in subnets_to_emit_to.iter() {
            // Update moving prices after using them above.
            Self::update_moving_price(*netuid_i);
        }

        // --- 8. Drain pending emission through the subnet based on tempo.
        // Run the epoch for *all* subnets, even if we don't emit anything.
        for &netuid in subnets.iter() {
            // Reveal matured weights.
            if let Err(e) = Self::reveal_crv3_commits(netuid) {
                log::warn!("Failed to reveal commits for subnet {netuid} due to error: {e:?}");
            };
            // Pass on subnets that have not reached their tempo.
            if Self::should_run_epoch(netuid, current_block) {
                // Restart counters.
                BlocksSinceLastStep::<T>::insert(netuid, 0);
                LastMechansimStepBlock::<T>::insert(netuid, current_block);

                // Get and drain the subnet pending emission.
                let pending_alpha = PendingEmission::<T>::get(netuid);
                PendingEmission::<T>::insert(netuid, AlphaCurrency::ZERO);

                // Get and drain the subnet pending root divs.
                let pending_tao = PendingRootDivs::<T>::get(netuid);
                PendingRootDivs::<T>::insert(netuid, TaoCurrency::ZERO);

                // Get this amount as alpha that was swapped for pending root divs.
                let pending_swapped = PendingAlphaSwapped::<T>::get(netuid);
                PendingAlphaSwapped::<T>::insert(netuid, AlphaCurrency::ZERO);

                // Get owner cut and drain.
                let owner_cut = PendingOwnerCut::<T>::get(netuid);
                PendingOwnerCut::<T>::insert(netuid, AlphaCurrency::ZERO);

                // Drain pending root divs, alpha emission, and owner cut.
                Self::drain_pending_emission(
                    netuid,
                    pending_alpha,
                    pending_tao,
                    pending_swapped,
                    owner_cut,
                );
            } else {
                // Increment
                BlocksSinceLastStep::<T>::mutate(netuid, |total| *total = total.saturating_add(1));
            }
        }
    }

    pub fn calculate_dividends_and_incentives(
        netuid: NetUid,
        hotkey_emission: Vec<(T::AccountId, AlphaCurrency, AlphaCurrency)>,
    ) -> (
        BTreeMap<T::AccountId, AlphaCurrency>,
        BTreeMap<T::AccountId, U96F32>,
    ) {
        // Accumulate emission of dividends and incentive per hotkey.
        let mut incentives: BTreeMap<T::AccountId, AlphaCurrency> = BTreeMap::new();
        let mut dividends: BTreeMap<T::AccountId, U96F32> = BTreeMap::new();
        for (hotkey, incentive, dividend) in hotkey_emission {
            // Accumulate incentives to miners.
            incentives
                .entry(hotkey.clone())
                .and_modify(|e| *e = e.saturating_add(incentive))
                .or_insert(incentive);
            // Accumulate dividends to parents.
            let div_tuples: Vec<(T::AccountId, AlphaCurrency)> =
                Self::get_parent_child_dividends_distribution(&hotkey, netuid, dividend);
            // Accumulate dividends per hotkey.
            for (parent, parent_div) in div_tuples {
                dividends
                    .entry(parent)
                    .and_modify(|e| *e = e.saturating_add(asfloat!(parent_div)))
                    .or_insert(asfloat!(parent_div));
            }
        }
        log::debug!("incentives: {incentives:?}");
        log::debug!("dividends: {dividends:?}");

        (incentives, dividends)
    }

    pub fn calculate_dividend_distribution(
        pending_alpha: AlphaCurrency,
        pending_tao: TaoCurrency,
        tao_weight: U96F32,
        stake_map: BTreeMap<T::AccountId, (AlphaCurrency, AlphaCurrency)>,
        dividends: BTreeMap<T::AccountId, U96F32>,
    ) -> (
        BTreeMap<T::AccountId, U96F32>,
        BTreeMap<T::AccountId, U96F32>,
    ) {
        log::debug!("dividends: {dividends:?}");
        log::debug!("stake_map: {stake_map:?}");
        log::debug!("pending_alpha: {pending_alpha:?}");
        log::debug!("pending_tao: {pending_tao:?}");
        log::debug!("tao_weight: {tao_weight:?}");

        // Setup.
        let zero: U96F32 = asfloat!(0.0);

        // Accumulate root divs and alpha_divs. For each hotkey we compute their
        // local and root dividend proportion based on their alpha_stake/root_stake
        let mut total_root_divs: U96F32 = asfloat!(0);
        let mut total_alpha_divs: U96F32 = asfloat!(0);
        let mut root_dividends: BTreeMap<T::AccountId, U96F32> = BTreeMap::new();
        let mut alpha_dividends: BTreeMap<T::AccountId, U96F32> = BTreeMap::new();
        for (hotkey, dividend) in dividends {
            if let Some((alpha_stake, root_stake)) = stake_map.get(&hotkey) {
                let alpha_stake = alpha_stake.to_u64();
                let root_stake = root_stake.to_u64();
                // Get hotkey ALPHA on subnet.
                let alpha_stake = asfloat!(alpha_stake);
                // Get hotkey TAO on root.
                let root_stake = asfloat!(root_stake);

                // Convert TAO to alpha with weight.
                let root_alpha = root_stake.saturating_mul(tao_weight);
                // Get total from root and local
                let total_alpha = alpha_stake.saturating_add(root_alpha);
                // Compute root prop.
                let root_prop = root_alpha.checked_div(total_alpha).unwrap_or(zero);
                // Compute root dividends
                let root_divs = dividend.saturating_mul(root_prop);
                // Compute alpha dividends
                let alpha_divs = dividend.saturating_sub(root_divs);
                // Record the alpha dividends.
                alpha_dividends
                    .entry(hotkey.clone())
                    .and_modify(|e| *e = e.saturating_add(alpha_divs))
                    .or_insert(alpha_divs);
                // Accumulate total alpha divs.
                total_alpha_divs = total_alpha_divs.saturating_add(alpha_divs);
                // Record the root dividends.
                root_dividends
                    .entry(hotkey.clone())
                    .and_modify(|e| *e = e.saturating_add(root_divs))
                    .or_insert(root_divs);
                // Accumulate total root divs.
                total_root_divs = total_root_divs.saturating_add(root_divs);
            }
        }
        log::debug!("alpha_dividends: {alpha_dividends:?}");
        log::debug!("root_dividends: {root_dividends:?}");
        log::debug!("total_root_divs: {total_root_divs:?}");
        log::debug!("total_alpha_divs: {total_alpha_divs:?}");

        // Compute root divs as TAO. Here we take
        let mut tao_dividends: BTreeMap<T::AccountId, U96F32> = BTreeMap::new();
        for (hotkey, root_divs) in root_dividends {
            // Root proportion.
            let root_share: U96F32 = root_divs.checked_div(total_root_divs).unwrap_or(zero);
            log::debug!("hotkey: {hotkey:?}, root_share: {root_share:?}");
            // Root proportion in TAO
            let root_tao: U96F32 = asfloat!(pending_tao).saturating_mul(root_share);
            log::debug!("hotkey: {hotkey:?}, root_tao: {root_tao:?}");
            // Record root dividends as TAO.
            tao_dividends
                .entry(hotkey)
                .and_modify(|e| *e = root_tao)
                .or_insert(root_tao);
        }
        log::debug!("tao_dividends: {tao_dividends:?}");

        // Compute proportional alpha divs using the pending alpha and total alpha divs from the epoch.
        let mut prop_alpha_dividends: BTreeMap<T::AccountId, U96F32> = BTreeMap::new();
        for (hotkey, alpha_divs) in alpha_dividends {
            // Alpha proportion.
            let alpha_share: U96F32 = alpha_divs.checked_div(total_alpha_divs).unwrap_or(zero);
            log::debug!("hotkey: {hotkey:?}, alpha_share: {alpha_share:?}");

            // Compute the proportional pending_alpha to this hotkey.
            let prop_alpha = asfloat!(pending_alpha).saturating_mul(alpha_share);
            log::debug!("hotkey: {hotkey:?}, prop_alpha: {prop_alpha:?}");
            // Record the proportional alpha dividends.
            prop_alpha_dividends
                .entry(hotkey.clone())
                .and_modify(|e| *e = prop_alpha)
                .or_insert(prop_alpha);
        }
        log::debug!("prop_alpha_dividends: {prop_alpha_dividends:?}");

        (prop_alpha_dividends, tao_dividends)
    }

    fn get_immune_owner_hotkeys(netuid: NetUid, coldkey: &T::AccountId) -> Vec<T::AccountId> {
        // Gather (block, uid, hotkey) only for hotkeys that have a UID and a registration block.
        let mut triples: Vec<(u64, u16, T::AccountId)> = OwnedHotkeys::<T>::get(coldkey)
            .into_iter()
            .filter_map(|hotkey| {
                // Uids must exist, filter_map ignores hotkeys without UID
                Uids::<T>::get(netuid, &hotkey).map(|uid| {
                    let block = BlockAtRegistration::<T>::get(netuid, uid);
                    (block, uid, hotkey)
                })
            })
            .collect();

        // Sort by BlockAtRegistration (descending), then by uid (ascending)
        // Recent registration is priority so that we can let older keys expire (get non-immune)
        triples.sort_by(|(b1, u1, _), (b2, u2, _)| b2.cmp(b1).then(u1.cmp(u2)));

        // Keep first ImmuneOwnerUidsLimit
        let limit = ImmuneOwnerUidsLimit::<T>::get(netuid).into();
        if triples.len() > limit {
            triples.truncate(limit);
        }

        // Project to just hotkeys
        let mut immune_hotkeys: Vec<T::AccountId> =
            triples.into_iter().map(|(_, _, hk)| hk).collect();

        // Insert subnet owner hotkey in the beginning of the list if valid and not
        // already present
        if let Ok(owner_hk) = SubnetOwnerHotkey::<T>::try_get(netuid) {
            if Uids::<T>::get(netuid, &owner_hk).is_some() && !immune_hotkeys.contains(&owner_hk) {
                immune_hotkeys.insert(0, owner_hk);
                if immune_hotkeys.len() > limit {
                    immune_hotkeys.truncate(limit);
                }
            }
        }

        immune_hotkeys
    }

    pub fn distribute_dividends_and_incentives(
        netuid: NetUid,
        owner_cut: AlphaCurrency,
        incentives: BTreeMap<T::AccountId, AlphaCurrency>,
        alpha_dividends: BTreeMap<T::AccountId, U96F32>,
        tao_dividends: BTreeMap<T::AccountId, U96F32>,
    ) {
        // Distribute the owner cut.
        if let Ok(owner_coldkey) = SubnetOwner::<T>::try_get(netuid) {
            if let Ok(owner_hotkey) = SubnetOwnerHotkey::<T>::try_get(netuid) {
                // Increase stake for owner hotkey and coldkey.
                log::debug!(
                    "owner_hotkey: {owner_hotkey:?} owner_coldkey: {owner_coldkey:?}, owner_cut: {owner_cut:?}"
                );
                let real_owner_cut = Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                    &owner_hotkey,
                    &owner_coldkey,
                    netuid,
                    owner_cut,
                );
                // If the subnet is leased, notify the lease logic that owner cut has been distributed.
                if let Some(lease_id) = SubnetUidToLeaseId::<T>::get(netuid) {
                    Self::distribute_leased_network_dividends(lease_id, real_owner_cut);
                }
            }
        }

        // Distribute mining incentives.
        let subnet_owner_coldkey = SubnetOwner::<T>::get(netuid);
        let owner_hotkeys = Self::get_immune_owner_hotkeys(netuid, &subnet_owner_coldkey);
        log::debug!("incentives: owner hotkeys: {owner_hotkeys:?}");
        for (hotkey, incentive) in incentives {
            log::debug!("incentives: hotkey: {incentive:?}");

            // Skip/burn miner-emission for immune keys
            if owner_hotkeys.contains(&hotkey) {
                log::debug!(
                    "incentives: hotkey: {hotkey:?} is SN owner hotkey or associated hotkey, skipping {incentive:?}"
                );
                continue;
            }

            // Increase stake for miner.
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey.clone(),
                &Owner::<T>::get(hotkey.clone()),
                netuid,
                incentive,
            );
        }

        // Distribute alpha divs.
        let _ = AlphaDividendsPerSubnet::<T>::clear_prefix(netuid, u32::MAX, None);
        for (hotkey, mut alpha_divs) in alpha_dividends {
            // Get take prop
            let alpha_take: U96F32 =
                Self::get_hotkey_take_float(&hotkey).saturating_mul(alpha_divs);
            // Remove take prop from alpha_divs
            alpha_divs = alpha_divs.saturating_sub(alpha_take);
            // Give the validator their take.
            log::debug!("hotkey: {hotkey:?} alpha_take: {alpha_take:?}");
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &Owner::<T>::get(&hotkey),
                netuid,
                tou64!(alpha_take).into(),
            );
            // Give all other nominators.
            log::debug!("hotkey: {hotkey:?} alpha_divs: {alpha_divs:?}");
            Self::increase_stake_for_hotkey_on_subnet(&hotkey, netuid, tou64!(alpha_divs).into());
            // Record dividends for this hotkey.
            AlphaDividendsPerSubnet::<T>::mutate(netuid, &hotkey, |divs| {
                *divs = divs.saturating_add(tou64!(alpha_divs).into());
            });
            // Record total hotkey alpha based on which this value of AlphaDividendsPerSubnet
            // was calculated
            let total_hotkey_alpha = TotalHotkeyAlpha::<T>::get(&hotkey, netuid);
            TotalHotkeyAlphaLastEpoch::<T>::insert(hotkey, netuid, total_hotkey_alpha);
        }

        // Distribute root tao divs.
        let _ = TaoDividendsPerSubnet::<T>::clear_prefix(netuid, u32::MAX, None);
        for (hotkey, mut root_tao) in tao_dividends {
            // Get take prop
            let tao_take: U96F32 = Self::get_hotkey_take_float(&hotkey).saturating_mul(root_tao);
            // Remove take prop from root_tao
            root_tao = root_tao.saturating_sub(tao_take);
            // Give the validator their take.
            log::debug!("hotkey: {hotkey:?} tao_take: {tao_take:?}");
            let validator_stake = Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey,
                &Owner::<T>::get(hotkey.clone()),
                NetUid::ROOT,
                tou64!(tao_take).into(),
            );
            // Give rest to nominators.
            log::debug!("hotkey: {hotkey:?} root_tao: {root_tao:?}");
            Self::increase_stake_for_hotkey_on_subnet(
                &hotkey,
                NetUid::ROOT,
                tou64!(root_tao).into(),
            );
            // Record root dividends for this validator on this subnet.
            TaoDividendsPerSubnet::<T>::mutate(netuid, hotkey.clone(), |divs| {
                *divs = divs.saturating_add(tou64!(root_tao).into());
            });
            // Update the total TAO on the subnet with root tao dividends.
            SubnetTAO::<T>::mutate(NetUid::ROOT, |total| {
                *total = total
                    .saturating_add(validator_stake.to_u64().into())
                    .saturating_add(tou64!(root_tao).into());
            });
        }
    }

    pub fn get_stake_map(
        netuid: NetUid,
        hotkeys: Vec<&T::AccountId>,
    ) -> BTreeMap<T::AccountId, (AlphaCurrency, AlphaCurrency)> {
        let mut stake_map: BTreeMap<T::AccountId, (AlphaCurrency, AlphaCurrency)> = BTreeMap::new();
        for hotkey in hotkeys {
            // Get hotkey ALPHA on subnet.
            let alpha_stake = Self::get_stake_for_hotkey_on_subnet(hotkey, netuid);
            // Get hotkey TAO on root.
            let root_stake = Self::get_stake_for_hotkey_on_subnet(hotkey, NetUid::ROOT);
            stake_map.insert(hotkey.clone(), (alpha_stake, root_stake));
        }
        stake_map
    }

    pub fn calculate_dividend_and_incentive_distribution(
        netuid: NetUid,
        pending_tao: TaoCurrency,
        pending_validator_alpha: AlphaCurrency,
        hotkey_emission: Vec<(T::AccountId, AlphaCurrency, AlphaCurrency)>,
        tao_weight: U96F32,
    ) -> (
        BTreeMap<T::AccountId, AlphaCurrency>,
        (
            BTreeMap<T::AccountId, U96F32>,
            BTreeMap<T::AccountId, U96F32>,
        ),
    ) {
        let (incentives, dividends) =
            Self::calculate_dividends_and_incentives(netuid, hotkey_emission);

        let stake_map = Self::get_stake_map(netuid, dividends.keys().collect::<Vec<_>>());

        let (alpha_dividends, tao_dividends) = Self::calculate_dividend_distribution(
            pending_validator_alpha,
            pending_tao,
            tao_weight,
            stake_map,
            dividends,
        );

        (incentives, (alpha_dividends, tao_dividends))
    }

    pub fn drain_pending_emission(
        netuid: NetUid,
        pending_alpha: AlphaCurrency,
        pending_tao: TaoCurrency,
        pending_swapped: AlphaCurrency,
        owner_cut: AlphaCurrency,
    ) {
        log::debug!(
            "Draining pending alpha emission for netuid {netuid:?}, pending_alpha: {pending_alpha:?}, pending_tao: {pending_tao:?}, pending_swapped: {pending_swapped:?}, owner_cut: {owner_cut:?}"
        );

        let tao_weight = Self::get_tao_weight();

        // Run the epoch.
        let hotkey_emission: Vec<(T::AccountId, AlphaCurrency, AlphaCurrency)> =
            Self::epoch(netuid, pending_alpha.saturating_add(pending_swapped));
        log::debug!("hotkey_emission: {hotkey_emission:?}");

        // Compute the pending validator alpha.
        // This is the total alpha being injected,
        // minus the the alpha for the miners, (50%)
        // and minus the alpha swapped for TAO (pending_swapped).
        // Important! If the incentives are 0, then Validators get 100% of the alpha.
        let incentive_sum = hotkey_emission
            .iter()
            .fold(AlphaCurrency::default(), |acc, (_, incentive, _)| {
                acc.saturating_add(*incentive)
            });
        log::debug!("incentive_sum: {incentive_sum:?}");

        let pending_validator_alpha = if !incentive_sum.is_zero() {
            pending_alpha
                .saturating_add(pending_swapped)
                .saturating_div(2.into())
                .saturating_sub(pending_swapped)
        } else {
            // If the incentive is 0, then Validators get 100% of the alpha.
            pending_alpha
        };

        let (incentives, (alpha_dividends, tao_dividends)) =
            Self::calculate_dividend_and_incentive_distribution(
                netuid,
                pending_tao,
                pending_validator_alpha,
                hotkey_emission,
                tao_weight,
            );

        Self::distribute_dividends_and_incentives(
            netuid,
            owner_cut,
            incentives,
            alpha_dividends,
            tao_dividends,
        );
    }

    /// Returns the self contribution of a hotkey on a subnet.
    /// This is the portion of the hotkey's stake that is provided by itself, and not delegated to other hotkeys.
    pub fn get_self_contribution(hotkey: &T::AccountId, netuid: NetUid) -> u64 {
        // Get all childkeys for this hotkey.
        let childkeys = Self::get_children(hotkey, netuid);
        let mut remaining_proportion: U96F32 = U96F32::saturating_from_num(1.0);
        for (proportion, _) in childkeys {
            remaining_proportion = remaining_proportion.saturating_sub(
                U96F32::saturating_from_num(proportion) // Normalize
                    .safe_div(U96F32::saturating_from_num(u64::MAX)),
            );
        }

        // Get TAO weight
        let tao_weight: U96F32 = Self::get_tao_weight();

        // Get the hotkey's stake including weight
        let root_stake: U96F32 =
            U96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, NetUid::ROOT));
        let alpha_stake: U96F32 =
            U96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, netuid));

        // Calculate the
        let alpha_contribution: U96F32 = alpha_stake.saturating_mul(remaining_proportion);
        let root_contribution: U96F32 = root_stake
            .saturating_mul(remaining_proportion)
            .saturating_mul(tao_weight);
        let combined_contribution: U96F32 = alpha_contribution.saturating_add(root_contribution);

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
    pub fn get_parent_child_dividends_distribution(
        hotkey: &T::AccountId,
        netuid: NetUid,
        dividends: AlphaCurrency,
    ) -> Vec<(T::AccountId, AlphaCurrency)> {
        // hotkey dividends.
        let mut dividend_tuples: Vec<(T::AccountId, AlphaCurrency)> = vec![];

        // Calculate the hotkey's share of the validator emission based on its childkey take
        let validating_emission: U96F32 = U96F32::saturating_from_num(dividends);
        let mut remaining_emission: U96F32 = validating_emission;
        let childkey_take_proportion: U96F32 =
            U96F32::saturating_from_num(Self::get_childkey_take(hotkey, netuid))
                .safe_div(U96F32::saturating_from_num(u16::MAX));
        log::debug!("Childkey take proportion: {childkey_take_proportion:?} for hotkey {hotkey:?}");
        // NOTE: Only the validation emission should be split amongst parents.

        // Grab the owner of the childkey.
        let childkey_owner = Self::get_owning_coldkey_for_hotkey(hotkey);

        // Initialize variables to track emission distribution
        let mut to_parents: u64 = 0;
        let mut total_child_emission_take: U96F32 = U96F32::saturating_from_num(0);

        // Initialize variables to calculate total stakes from parents
        let mut total_contribution: U96F32 = U96F32::saturating_from_num(0);
        let mut parent_contributions: Vec<(T::AccountId, U96F32)> = Vec::new();

        // Get the weights for root and alpha stakes in emission distribution
        let tao_weight: U96F32 = Self::get_tao_weight();

        // Get self contribution, removing any childkey proportions.
        let self_contribution = Self::get_self_contribution(hotkey, netuid);
        log::debug!(
            "Self contribution for hotkey {hotkey:?} on netuid {netuid:?}: {self_contribution:?}"
        );
        // Add self contribution to total contribution but not to the parent contributions.
        total_contribution =
            total_contribution.saturating_add(U96F32::saturating_from_num(self_contribution));

        // Calculate total root and alpha (subnet-specific) stakes from all parents
        for (proportion, parent) in Self::get_parents(hotkey, netuid) {
            // Convert the parent's stake proportion to a fractional value
            let parent_proportion: U96F32 = U96F32::saturating_from_num(proportion)
                .safe_div(U96F32::saturating_from_num(u64::MAX));

            // Get the parent's root and subnet-specific (alpha) stakes
            let parent_root: U96F32 = U96F32::saturating_from_num(
                Self::get_stake_for_hotkey_on_subnet(&parent, NetUid::ROOT),
            );
            let parent_alpha: U96F32 =
                U96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(&parent, netuid));

            // Calculate the parent's contribution to the hotkey's stakes
            let parent_alpha_contribution: U96F32 = parent_alpha.saturating_mul(parent_proportion);
            let parent_root_contribution: U96F32 = parent_root
                .saturating_mul(parent_proportion)
                .saturating_mul(tao_weight);
            let combined_contribution: U96F32 =
                parent_alpha_contribution.saturating_add(parent_root_contribution);

            // Add to the total stakes
            total_contribution = total_contribution.saturating_add(combined_contribution);
            // Store the parent's contributions for later use
            parent_contributions.push((parent.clone(), combined_contribution));
            log::debug!(
                "Parent contribution for hotkey {hotkey:?} from parent {parent:?}: {combined_contribution:?}"
            );
        }

        // Distribute emission to parents based on their contributions.
        // Deduct childkey take from parent contribution.
        for (parent, contribution) in parent_contributions {
            let parent_owner = Self::get_owning_coldkey_for_hotkey(&parent);

            // Get the stake contribution of this parent key of the total stake.
            let emission_factor: U96F32 = contribution
                .checked_div(total_contribution)
                .unwrap_or(U96F32::saturating_from_num(0));

            // Get the parent's portion of the validating emission based on their contribution.
            let mut parent_emission: U96F32 = validating_emission.saturating_mul(emission_factor);
            // Remove this emission from the remaining emission.
            remaining_emission = remaining_emission.saturating_sub(parent_emission);

            // Get the childkey take for this parent.
            let child_emission_take: U96F32 = if parent_owner == childkey_owner {
                // The parent is from the same coldkey, so we don't remove any childkey take.
                U96F32::saturating_from_num(0)
            } else {
                childkey_take_proportion
                    .saturating_mul(U96F32::saturating_from_num(parent_emission))
            };

            // Remove the childkey take from the parent's emission.
            parent_emission = parent_emission.saturating_sub(child_emission_take);

            // Add the childkey take to the total childkey take tracker.
            total_child_emission_take =
                total_child_emission_take.saturating_add(child_emission_take);

            log::debug!("Child emission take: {child_emission_take:?} for hotkey {hotkey:?}");
            log::debug!("Parent emission: {parent_emission:?} for hotkey {hotkey:?}");
            log::debug!("remaining emission: {remaining_emission:?}");

            // Add the parent's emission to the distribution list
            dividend_tuples.push((
                parent.clone(),
                parent_emission.saturating_to_num::<u64>().into(),
            ));

            // Keep track of total emission distributed to parents
            to_parents = to_parents.saturating_add(parent_emission.saturating_to_num::<u64>());
            log::debug!(
                "Parent contribution for parent {parent:?} with contribution: {contribution:?}, of total: {total_contribution:?} ({emission_factor:?}), of emission: {validating_emission:?} gets: {parent_emission:?}",
            );
        }
        // Calculate the final emission for the hotkey itself.
        // This includes the take left from the parents and the self contribution.
        let child_emission = remaining_emission
            .saturating_add(total_child_emission_take)
            .saturating_to_num::<u64>()
            .into();

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
    pub fn should_run_epoch(netuid: NetUid, current_block: u64) -> bool {
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
    pub fn blocks_until_next_epoch(netuid: NetUid, tempo: u16, block_number: u64) -> u64 {
        if tempo == 0 {
            return u64::MAX;
        }
        let netuid_plus_one = (u16::from(netuid) as u64).saturating_add(1);
        let tempo_plus_one = (tempo as u64).saturating_add(1);
        let adjusted_block = block_number.wrapping_add(netuid_plus_one);
        let remainder = adjusted_block.checked_rem(tempo_plus_one).unwrap_or(0);
        (tempo as u64).saturating_sub(remainder)
    }
}
