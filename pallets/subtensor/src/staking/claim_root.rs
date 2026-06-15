use super::*;
use frame_support::dispatch::DispatchResult;
use frame_support::storage::{TransactionOutcome, with_transaction};
use frame_support::weights::Weight;
use sp_core::Get;
use sp_runtime::DispatchError;
use sp_runtime::traits::AccountIdConversion;
use sp_std::collections::btree_set::BTreeSet;
use substrate_fixed::types::{I96F32, U96F32};
use subtensor_runtime_common::NetUidStorageIndex;
use subtensor_swap_interface::SwapHandler;

impl<T: Config> Pallet<T> {
    pub fn block_hash_to_indices(block_hash: T::Hash, k: u64, n: u64) -> Vec<u64> {
        let block_hash_bytes = block_hash.as_ref();
        let mut indices: BTreeSet<u64> = BTreeSet::new();
        // k < n
        let start_index: u64 = u64::from_be_bytes(
            block_hash_bytes
                .get(0..8)
                .unwrap_or(&[0; 8])
                .try_into()
                .unwrap_or([0; 8]),
        );
        let mut last_idx = start_index;
        for i in 0..k {
            let bh_idx: usize = ((i.saturating_mul(8)) % 32) as usize;
            let idx_step = u64::from_be_bytes(
                block_hash_bytes
                    .get(bh_idx..(bh_idx.saturating_add(8)))
                    .unwrap_or(&[0; 8])
                    .try_into()
                    .unwrap_or([0; 8]),
            );
            let idx = last_idx
                .saturating_add(idx_step)
                .checked_rem(n)
                .unwrap_or(0);
            indices.insert(idx);
            last_idx = idx;
        }
        indices.into_iter().collect()
    }

    pub fn increase_root_claimable_for_hotkey_and_subnet(
        hotkey: &T::AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
    ) {
        // Get total stake on this hotkey on root.
        let total: I96F32 =
            I96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, NetUid::ROOT));

        // Get increment
        let increment: I96F32 = I96F32::saturating_from_num(amount)
            .checked_div(total)
            .unwrap_or(I96F32::saturating_from_num(0.0));

        // Unlikely to happen. This is mostly for test environment sanity checks.
        if u64::from(amount) > total.saturating_to_num::<u64>() {
            log::warn!("Not enough root stake. NetUID = {netuid}");

            let owner = Owner::<T>::get(hotkey);
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &owner, netuid, amount);
            return;
        }

        // Increment claimable for this subnet.
        Self::bump_root_claimable_rate(hotkey, netuid, increment);
    }

    /// Adds `increment` (alpha-principal per unit of root stake) to a hotkey's claimable
    /// rate on `netuid`. This is the unit-agnostic core shared by the legacy single-subnet
    /// crediting and the beta basket distribution.
    pub fn bump_root_claimable_rate(hotkey: &T::AccountId, netuid: NetUid, increment: I96F32) {
        if increment == I96F32::saturating_from_num(0) {
            return;
        }
        RootClaimable::<T>::mutate(hotkey, |claimable| {
            claimable
                .entry(netuid)
                .and_modify(|claim_total| *claim_total = claim_total.saturating_add(increment))
                .or_insert(increment);
        });
    }

    /// The single global escrow coldkey that custodies every validator's beta basket.
    ///
    /// Baskets are held as positions `(validator_hotkey, this_account, netuid)` in the normal
    /// alpha share pool, so they count toward each validator's stake and compound with that
    /// validator's dividends, while the account itself stays inert (no user controls it). A
    /// single global coldkey is used deliberately: positions stay distinct per validator via
    /// the hotkey key, and hotkey swaps migrate them by value automatically.
    pub fn get_beta_escrow_account_id() -> T::AccountId {
        T::SubtensorPalletId::get().into_sub_account_truncating(b"beta/esc")
    }

    /// Distributes a validator's root dividend (origin-subnet alpha, net of take) into its beta
    /// basket according to the validator's root weight vector `w` (set on subnet 0).
    ///
    /// Flow: sell the origin alpha for TAO, then split that TAO across subnets per `w`, buying
    /// each subnet's alpha and staking it to the validator under the global escrow coldkey. Each
    /// slot records the bought alpha as basket principal and bumps the per-staker claimable rate.
    /// The whole operation is transactional: if any swap fails, it is rolled back and the original
    /// alpha is recycled. If the validator has no usable weights (or no root stake), the dividend
    /// is recycled.
    pub fn distribute_root_alpha_to_basket(
        hotkey: &T::AccountId,
        origin_netuid: NetUid,
        root_alpha: AlphaBalance,
    ) {
        if root_alpha.is_zero() {
            return;
        }

        // Resolve the validator's basket weight vector w = Weights[ROOT][uid]. The vector follows
        // the validator's root uid (so it survives hotkey swaps automatically) and reuses the
        // existing root weights plumbing.
        let maybe_uid = Uids::<T>::try_get(NetUid::ROOT, hotkey).ok();
        let weights = maybe_uid
            .map(|uid| Weights::<T>::get(NetUidStorageIndex::ROOT, uid))
            .unwrap_or_default();

        // Keep only weights that point at existing, non-root subnets.
        let valid: Vec<(NetUid, u64)> = weights
            .into_iter()
            .filter_map(|(dest, weight)| {
                let dest_netuid = NetUid::from(dest);
                if weight > 0 && !dest_netuid.is_root() && Self::if_subnet_exist(dest_netuid) {
                    Some((dest_netuid, weight as u64))
                } else {
                    None
                }
            })
            .collect();

        let weight_sum: u64 = valid.iter().map(|(_, w)| *w).sum();
        let total_root = Self::get_stake_for_hotkey_on_subnet(hotkey, NetUid::ROOT);

        // No usable weights or no root stake to apportion against: recycle.
        if valid.is_empty() || weight_sum == 0 || total_root.is_zero() {
            Self::recycle_subnet_alpha(origin_netuid, root_alpha);
            return;
        }

        let total_root_float = I96F32::saturating_from_num(total_root);
        let escrow = Self::get_beta_escrow_account_id();

        let outcome = with_transaction(|| {
            // 1. Sell the origin-subnet alpha for TAO.
            let tao_total: TaoBalance = match Self::swap_alpha_for_tao(
                origin_netuid,
                root_alpha,
                T::SwapInterface::min_price::<TaoBalance>(),
                true,
            ) {
                Ok(res) => res.amount_paid_out,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            // 2. Split the TAO across subnets per w and buy each subnet's alpha.
            let tao_total_u64: u64 = tao_total.to_u64();
            let mut spent: u64 = 0;
            let last_idx = valid.len().saturating_sub(1);
            for (i, (dest_netuid, weight)) in valid.iter().enumerate() {
                // Last slot absorbs the rounding remainder so Σ tao_s == tao_total exactly.
                let tao_s: u64 = if i == last_idx {
                    tao_total_u64.saturating_sub(spent)
                } else {
                    U96F32::saturating_from_num(tao_total_u64)
                        .saturating_mul(U96F32::saturating_from_num(*weight))
                        .checked_div(U96F32::saturating_from_num(weight_sum))
                        .unwrap_or(U96F32::saturating_from_num(0))
                        .saturating_to_num::<u64>()
                };
                spent = spent.saturating_add(tao_s);
                if tao_s == 0 {
                    continue;
                }

                let bought: AlphaBalance = match Self::swap_tao_for_alpha(
                    *dest_netuid,
                    tao_s.into(),
                    T::SwapInterface::max_price(),
                    true,
                ) {
                    Ok(res) => res.amount_paid_out,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };
                if bought.is_zero() {
                    continue;
                }

                // Per-staker claimable rate increment: bought alpha per unit of root stake.
                let increment: I96F32 = I96F32::saturating_from_num(bought)
                    .checked_div(total_root_float)
                    .unwrap_or(I96F32::saturating_from_num(0));

                // If the increment underflows to zero (bought is tiny relative to the root pool),
                // crediting would grow principal/escrow with no claimable rate, stranding the
                // value. Recycle this slot's alpha instead, keeping `Σ owed == BasketPrincipal`
                // exact. (TAO stays neutral: the buy's `tao_s` already balances the origin sell.)
                if increment == I96F32::saturating_from_num(0) {
                    Self::recycle_subnet_alpha(*dest_netuid, bought);
                    continue;
                }

                // Stake the bought alpha to the validator under the escrow coldkey.
                Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                    hotkey,
                    &escrow,
                    *dest_netuid,
                    bought,
                );

                // Record basket principal (alpha) for the E/P compounding multiplier.
                BasketPrincipal::<T>::mutate(hotkey, *dest_netuid, |p| {
                    *p = p.saturating_add(bought);
                });

                Self::bump_root_claimable_rate(hotkey, *dest_netuid, increment);
            }

            TransactionOutcome::Commit(Ok(()))
        });

        // On any failure the swaps were rolled back; recycle the original alpha.
        if outcome.is_err() {
            Self::recycle_subnet_alpha(origin_netuid, root_alpha);
        }
    }

    pub fn get_root_claimable_for_hotkey_coldkey(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> I96F32 {
        // Get this keys stake balance on root.
        let root_stake: I96F32 = I96F32::saturating_from_num(
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, NetUid::ROOT),
        );

        // Get the total claimable_rate for this hotkey and this network
        let claimable_rate: I96F32 = *RootClaimable::<T>::get(hotkey)
            .get(&netuid)
            .unwrap_or(&I96F32::from(0));

        // Compute the proportion owed to this coldkey via balance.
        let claimable: I96F32 = claimable_rate.saturating_mul(root_stake);

        claimable
    }

    pub fn get_root_owed_for_hotkey_coldkey_float(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> I96F32 {
        let claimable = Self::get_root_claimable_for_hotkey_coldkey(hotkey, coldkey, netuid);

        // Attain the root claimed to avoid overclaiming.
        let root_claimed: I96F32 =
            I96F32::saturating_from_num(RootClaimed::<T>::get((netuid, hotkey, coldkey)));

        // Subtract the already claimed alpha.
        let owed: I96F32 = claimable.saturating_sub(root_claimed);

        owed
    }

    pub fn get_root_owed_for_hotkey_coldkey(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> u64 {
        let owed = Self::get_root_owed_for_hotkey_coldkey_float(hotkey, coldkey, netuid);

        // Convert owed to u64, mapping negative values to 0
        let owed_u64: u64 = if owed.is_negative() {
            0
        } else {
            owed.saturating_to_num::<u64>()
        };

        owed_u64
    }

    /// Claims (redeems) a staker's share of a validator's beta basket on `netuid`.
    ///
    /// Redemption is always a full swap to TAO: the staker's owed *principal* is scaled by the
    /// basket's live growth multiplier `E / P` (escrow value over outstanding principal) to get
    /// the current payout, that payout alpha is removed from the escrow position, swapped to TAO,
    /// and staked on root for the staker. `root_claim_type` is retained for signature
    /// compatibility but no longer branches behavior (Keep was removed).
    pub fn root_claim_on_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
        _root_claim_type: RootClaimTypeEnum,
        ignore_minimum_condition: bool,
    ) -> DispatchResult {
        // Owed *principal* (alpha) = rate * root_stake - already-claimed.
        let owed: I96F32 = Self::get_root_owed_for_hotkey_coldkey_float(hotkey, coldkey, netuid);
        let owed_principal: u64 = if owed.is_negative() {
            0
        } else {
            owed.saturating_to_num::<u64>()
        };
        if owed_principal == 0 {
            return Ok(()); // no-op
        }

        // Live basket value via the escrow position, and outstanding principal.
        let escrow = Self::get_beta_escrow_account_id();
        let escrow_value: u64 =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &escrow, netuid).to_u64();
        let principal_total: u64 = BasketPrincipal::<T>::get(hotkey, netuid).to_u64();

        // Payout = owed_principal * (E / P), capped at the live escrow value.
        let payout: u64 = Self::basket_payout_from(owed_principal, escrow_value, principal_total);

        // Skip dust unless forced.
        if !ignore_minimum_condition
            && I96F32::saturating_from_num(payout)
                < I96F32::saturating_from_num(RootClaimableThreshold::<T>::get(&netuid))
        {
            log::debug!(
                "root claim on subnet {netuid} skipped (below threshold): payout={payout:?} h={hotkey:?} c={coldkey:?}"
            );
            return Ok(()); // no-op
        }

        // Nothing realizable yet (basket drained / zero value); leave the watermark untouched
        // so it can be claimed once the basket has value.
        if payout == 0 {
            return Ok(());
        }

        with_transaction(|| {
            // Remove the payout alpha from the validator's basket (escrow position).
            Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                &escrow,
                netuid,
                payout.into(),
            );

            // Swap the basket alpha to TAO.
            let owed_tao = match Self::swap_alpha_for_tao(
                netuid,
                payout.into(),
                T::SwapInterface::min_price::<TaoBalance>(),
                true,
            ) {
                Ok(owed_tao) => owed_tao,
                Err(err) => {
                    log::error!("Error swapping basket alpha for TAO: {err:?}");
                    return TransactionOutcome::Rollback(Err(err));
                }
            };

            let root_subnet_account_id = match Self::get_subnet_account_id(NetUid::ROOT) {
                Some(account_id) => account_id,
                None => {
                    return TransactionOutcome::Rollback(Err(
                        Error::<T>::RootNetworkDoesNotExist.into()
                    ));
                }
            };

            if let Err(err) = Self::transfer_tao_from_subnet(
                netuid,
                &root_subnet_account_id,
                owed_tao.amount_paid_out.into(),
            ) {
                log::error!("Error transferring root claim TAO from subnet: {err:?}");
                return TransactionOutcome::Rollback(Err(err));
            }

            // Record root sell as protocol outflow (reduces protocol cost).
            let root_sell_tao: TaoBalance = owed_tao.amount_paid_out;
            SubnetRootSellTao::<T>::mutate(netuid, |total| {
                *total = total.saturating_add(root_sell_tao);
            });
            Self::record_protocol_outflow(netuid, root_sell_tao);

            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                coldkey,
                NetUid::ROOT,
                owed_tao.amount_paid_out.to_u64().into(),
            );

            // Increase root subnet SubnetTAO
            SubnetTAO::<T>::mutate(NetUid::ROOT, |total| {
                *total = total.saturating_add(owed_tao.amount_paid_out.into());
            });

            // Increase root SubnetAlphaOut
            SubnetAlphaOut::<T>::mutate(NetUid::ROOT, |total| {
                *total = total.saturating_add(u64::from(owed_tao.amount_paid_out).into());
            });

            // Increase Total Stake
            TotalStake::<T>::mutate(|total| {
                *total = total.saturating_add(owed_tao.amount_paid_out.into());
            });

            Self::add_stake_adjust_root_claimed_for_hotkey_and_coldkey(
                hotkey,
                coldkey,
                owed_tao.amount_paid_out.into(),
            );

            TransactionOutcome::Commit(Ok(()))
        })?;

        // Consume the claimed principal from the basket and advance the watermark.
        BasketPrincipal::<T>::mutate(hotkey, netuid, |p| {
            *p = p.saturating_sub(owed_principal.into());
        });
        RootClaimed::<T>::mutate((netuid, hotkey, coldkey), |root_claimed| {
            *root_claimed = root_claimed.saturating_add(owed_principal.into());
        });

        Ok(())
    }

    fn root_claim_on_subnet_weight(_root_claim_type: RootClaimTypeEnum) -> Weight {
        Weight::from_parts(60_000_000, 6987)
            .saturating_add(T::DbWeight::get().reads(7_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    pub fn root_claim_all(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        subnets: Option<BTreeSet<NetUid>>,
    ) -> Result<Weight, DispatchError> {
        let mut weight = Weight::default();

        let root_claim_type = RootClaimType::<T>::get(coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        // Iterate over all the subnets this hotkey has claimable for root.
        let root_claimable = RootClaimable::<T>::get(hotkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        for (netuid, _) in root_claimable.iter() {
            let skip = subnets
                .as_ref()
                .map(|subnets| !subnets.contains(netuid))
                .unwrap_or(false);

            if skip {
                continue;
            }

            Self::root_claim_on_subnet(hotkey, coldkey, *netuid, root_claim_type.clone(), false)?;
            weight.saturating_accrue(Self::root_claim_on_subnet_weight(root_claim_type.clone()));
        }

        Ok(weight)
    }

    pub fn add_stake_adjust_root_claimed_for_hotkey_and_coldkey(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        amount: u64,
    ) {
        // Iterate over all the subnets this hotkey is staked on for root.
        let root_claimable = RootClaimable::<T>::get(hotkey);
        for (netuid, claimable_rate) in root_claimable.iter() {
            // Get current staker root claimed value.
            let root_claimed: u128 = RootClaimed::<T>::get((netuid, hotkey, coldkey));

            // Increase root claimed based on the claimable rate.
            let new_root_claimed = root_claimed.saturating_add(
                claimable_rate
                    .saturating_mul(I96F32::from(u64::from(amount)))
                    .saturating_to_num(),
            );

            // Set the new root claimed value.
            RootClaimed::<T>::insert((netuid, hotkey, coldkey), new_root_claimed);
        }
    }

    pub fn remove_stake_adjust_root_claimed_for_hotkey_and_coldkey(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        amount: AlphaBalance,
    ) {
        // Iterate over all the subnets this hotkey is staked on for root.
        let root_claimable = RootClaimable::<T>::get(hotkey);
        for (netuid, claimable_rate) in root_claimable.iter() {
            if *netuid == NetUid::ROOT.into() {
                continue; // Skip the root netuid.
            }

            // Get current staker root claimed value.
            let root_claimed: u128 = RootClaimed::<T>::get((netuid, hotkey, coldkey));

            // Decrease root claimed based on the claimable rate.
            let new_root_claimed = root_claimed.saturating_sub(
                claimable_rate
                    .saturating_mul(I96F32::from(u64::from(amount)))
                    .saturating_to_num(),
            );

            // Set the new root_claimed value.
            RootClaimed::<T>::insert((netuid, hotkey, coldkey), new_root_claimed);
        }
    }

    pub fn do_root_claim(
        coldkey: T::AccountId,
        subnets: Option<BTreeSet<NetUid>>,
    ) -> Result<Weight, DispatchError> {
        with_transaction(|| match Self::try_do_root_claim(coldkey, subnets) {
            Ok(weight) => TransactionOutcome::Commit(Ok(weight)),
            Err(err) => TransactionOutcome::Rollback(Err(err)),
        })
    }

    fn try_do_root_claim(
        coldkey: T::AccountId,
        subnets: Option<BTreeSet<NetUid>>,
    ) -> Result<Weight, DispatchError> {
        let mut weight = Weight::default();

        let hotkeys = StakingHotkeys::<T>::get(&coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        for hotkey in hotkeys.iter() {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            weight.saturating_accrue(Self::root_claim_all(hotkey, &coldkey, subnets.clone())?);
        }

        Self::deposit_event(Event::RootClaimed { coldkey });

        Ok(weight)
    }

    fn block_hash_to_indices_weight(k: u64, _n: u64) -> Weight {
        Weight::from_parts(3_000_000, 1517)
            .saturating_add(Weight::from_parts(100_412, 0).saturating_mul(k.into()))
    }

    pub fn maybe_add_coldkey_index(coldkey: &T::AccountId) {
        if !StakingColdkeys::<T>::contains_key(coldkey) {
            let n = NumStakingColdkeys::<T>::get();
            StakingColdkeysByIndex::<T>::insert(n, coldkey.clone());
            StakingColdkeys::<T>::insert(coldkey.clone(), n);
            NumStakingColdkeys::<T>::mutate(|n| *n = n.saturating_add(1));
        }
    }

    pub fn run_auto_claim_root_divs(last_block_hash: T::Hash) -> Weight {
        let mut weight: Weight = Weight::default();

        let n = NumStakingColdkeys::<T>::get();
        let k = NumRootClaim::<T>::get();
        weight.saturating_accrue(T::DbWeight::get().reads(2));

        let coldkeys_to_claim: Vec<u64> = Self::block_hash_to_indices(last_block_hash, k, n);
        weight.saturating_accrue(Self::block_hash_to_indices_weight(k, n));

        for i in coldkeys_to_claim.iter() {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            if let Ok(coldkey) = StakingColdkeysByIndex::<T>::try_get(i) {
                match Self::do_root_claim(coldkey.clone(), None) {
                    Ok(claim_weight) => weight.saturating_accrue(claim_weight),
                    Err(err) => log::error!("Error auto-claiming root dividends: {err:?}"),
                }
            }
        }

        weight
    }

    pub fn change_root_claim_type(coldkey: &T::AccountId, new_type: RootClaimTypeEnum) {
        RootClaimType::<T>::insert(coldkey.clone(), new_type.clone());

        Self::deposit_event(Event::RootClaimTypeSet {
            coldkey: coldkey.clone(),
            root_claim_type: new_type,
        });
    }

    pub fn transfer_root_claimed_for_new_keys(
        netuid: NetUid,
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
        old_coldkey: &T::AccountId,
        new_coldkey: &T::AccountId,
    ) {
        let old_root_claimed = RootClaimed::<T>::get((netuid, old_hotkey, old_coldkey));
        RootClaimed::<T>::remove((netuid, old_hotkey, old_coldkey));

        RootClaimed::<T>::mutate((netuid, new_hotkey, new_coldkey), |new_root_claimed| {
            *new_root_claimed = old_root_claimed.saturating_add(*new_root_claimed);
        });
    }
    pub fn transfer_root_claimable_for_new_hotkey(
        old_hotkey: &T::AccountId,
        new_hotkey: &T::AccountId,
    ) {
        let src_root_claimable = RootClaimable::<T>::get(old_hotkey);
        let mut dst_root_claimable = RootClaimable::<T>::get(new_hotkey);
        RootClaimable::<T>::remove(old_hotkey);

        for (netuid, claimable_rate) in src_root_claimable.into_iter() {
            dst_root_claimable
                .entry(netuid)
                .and_modify(|total| *total = total.saturating_add(claimable_rate))
                .or_insert(claimable_rate);
        }

        RootClaimable::<T>::insert(new_hotkey, dst_root_claimable);
    }

    /// Liquidates a validator's beta basket on `netuid` back to its root stakers.
    ///
    /// Used when a subnet is dissolved: the escrow position `(hotkey, H, netuid)` is removed,
    /// swapped to TAO, and credited to the validator's root nominators (proportional to their
    /// root stake) via the root share pool — so basket value reaches the actual stakers instead
    /// of being orphaned in the escrow account by subnet teardown. Best-effort: swap failures are
    /// logged and the slot is left for subnet teardown to handle.
    pub fn liquidate_basket_to_root_stakers(
        hotkey: &T::AccountId,
        escrow: &T::AccountId,
        netuid: NetUid,
    ) {
        let basket_alpha = Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, escrow, netuid);
        if basket_alpha.is_zero() {
            return;
        }

        let _ = with_transaction(|| {
            // Remove the basket alpha from the escrow position.
            Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                escrow,
                netuid,
                basket_alpha,
            );

            // Swap the basket alpha to TAO.
            let owed_tao = match Self::swap_alpha_for_tao(
                netuid,
                basket_alpha,
                T::SwapInterface::min_price::<TaoBalance>(),
                true,
            ) {
                Ok(owed_tao) => owed_tao,
                Err(err) => {
                    log::error!("Error liquidating basket alpha for TAO: {err:?}");
                    return TransactionOutcome::Rollback(Err(err));
                }
            };

            let root_subnet_account_id = match Self::get_subnet_account_id(NetUid::ROOT) {
                Some(account_id) => account_id,
                None => {
                    return TransactionOutcome::Rollback(Err(
                        Error::<T>::RootNetworkDoesNotExist.into()
                    ));
                }
            };

            if let Err(err) = Self::transfer_tao_from_subnet(
                netuid,
                &root_subnet_account_id,
                owed_tao.amount_paid_out.into(),
            ) {
                log::error!("Error transferring liquidated basket TAO from subnet: {err:?}");
                return TransactionOutcome::Rollback(Err(err));
            }

            Self::record_protocol_outflow(netuid, owed_tao.amount_paid_out);

            // Credit the validator's root nominators proportionally to their root stake.
            Self::increase_stake_for_hotkey_on_subnet(
                hotkey,
                NetUid::ROOT,
                owed_tao.amount_paid_out.to_u64().into(),
            );
            SubnetTAO::<T>::mutate(NetUid::ROOT, |total| {
                *total = total.saturating_add(owed_tao.amount_paid_out.into());
            });
            SubnetAlphaOut::<T>::mutate(NetUid::ROOT, |total| {
                *total = total.saturating_add(u64::from(owed_tao.amount_paid_out).into());
            });
            TotalStake::<T>::mutate(|total| {
                *total = total.saturating_add(owed_tao.amount_paid_out.into());
            });

            TransactionOutcome::Commit(Ok::<(), DispatchError>(()))
        });
    }

    /// Claim all root dividends for subnet and remove all associated data.
    pub fn finalize_all_subnet_root_dividends(netuid: NetUid) {
        let hotkeys = RootClaimable::<T>::iter_keys().collect::<Vec<_>>();
        let escrow = Self::get_beta_escrow_account_id();

        for hotkey in hotkeys.iter() {
            // Liquidate the validator's beta basket on this subnet back to root stakers before
            // clearing rates, so subnet teardown does not orphan basket value in the escrow.
            Self::liquidate_basket_to_root_stakers(hotkey, &escrow, netuid);
            BasketPrincipal::<T>::remove(hotkey, netuid);

            RootClaimable::<T>::mutate(hotkey, |claimable| {
                claimable.remove(&netuid);
            });
        }

        let _ = RootClaimed::<T>::clear_prefix((netuid,), u32::MAX, None);
    }

    // =========================================================================
    // Beta basket: read-only views (for RPC / dashboards)
    // =========================================================================

    /// Mark-to-market TAO value of `alpha` on `netuid` at the current pool price.
    /// This is a *marked* value (price x amount); actual redemption realizes slightly less
    /// due to AMM slippage.
    pub fn alpha_to_tao_value(netuid: NetUid, alpha: u64) -> u64 {
        if alpha == 0 {
            return 0;
        }
        let price =
            U96F32::saturating_from_num(T::SwapInterface::current_alpha_price(netuid.into()));
        U96F32::saturating_from_num(alpha)
            .saturating_mul(price)
            .saturating_to_num::<u64>()
    }

    /// Single source of truth for the basket growth multiplier: scales an owed principal by
    /// `E/P` (escrow value over outstanding principal), capped at the live escrow value so a
    /// claim can never draw more than the escrow holds.
    pub fn basket_payout_from(owed_principal: u64, escrow_value: u64, principal_total: u64) -> u64 {
        if owed_principal == 0 || principal_total == 0 || escrow_value == 0 {
            return 0;
        }
        U96F32::saturating_from_num(owed_principal)
            .saturating_mul(U96F32::saturating_from_num(escrow_value))
            .checked_div(U96F32::saturating_from_num(principal_total))
            .unwrap_or(U96F32::saturating_from_num(0))
            .saturating_to_num::<u64>()
            .min(escrow_value)
    }

    /// Current basket payout (in alpha) a staker would receive on `netuid` for a validator:
    /// owed principal scaled by the live `E/P` growth multiplier. Capped at the escrow value.
    pub fn get_basket_payout_alpha(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> u64 {
        let owed_principal = Self::get_root_owed_for_hotkey_coldkey(hotkey, coldkey, netuid);
        let escrow = Self::get_beta_escrow_account_id();
        let escrow_value =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &escrow, netuid).to_u64();
        let principal_total = BasketPrincipal::<T>::get(hotkey, netuid).to_u64();
        Self::basket_payout_from(owed_principal, escrow_value, principal_total)
    }

    /// Total TAO a coldkey would realize by redeeming every beta basket it holds across all of
    /// its validators (mark-to-market). This is the "pending TAO owed" figure for a staker.
    pub fn get_root_basket_owed_tao(coldkey: &T::AccountId) -> TaoBalance {
        let mut total: u64 = 0;
        for hotkey in StakingHotkeys::<T>::get(coldkey) {
            for (netuid, _principal) in BasketPrincipal::<T>::iter_prefix(&hotkey) {
                let payout = Self::get_basket_payout_alpha(&hotkey, coldkey, netuid);
                total = total.saturating_add(Self::alpha_to_tao_value(netuid, payout));
            }
        }
        total.into()
    }

    /// A validator's beta basket net asset value, in TAO (mark-to-market). This is the total
    /// "assets under management" backing all of the validator's stakers' baskets.
    pub fn get_validator_basket_nav_tao(hotkey: &T::AccountId) -> TaoBalance {
        let escrow = Self::get_beta_escrow_account_id();
        let mut nav: u64 = 0;
        for (netuid, _principal) in BasketPrincipal::<T>::iter_prefix(hotkey) {
            let escrow_value =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &escrow, netuid).to_u64();
            nav = nav.saturating_add(Self::alpha_to_tao_value(netuid, escrow_value));
        }
        nav.into()
    }

    /// A validator's full basket breakdown: per subnet, the alpha held and its TAO value.
    pub fn get_validator_basket(hotkey: &T::AccountId) -> Vec<(NetUid, AlphaBalance, TaoBalance)> {
        let escrow = Self::get_beta_escrow_account_id();
        let mut out: Vec<(NetUid, AlphaBalance, TaoBalance)> = Vec::new();
        for (netuid, _principal) in BasketPrincipal::<T>::iter_prefix(hotkey) {
            let escrow_value =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &escrow, netuid);
            if escrow_value.is_zero() {
                continue;
            }
            let tao = Self::alpha_to_tao_value(netuid, escrow_value.to_u64());
            out.push((netuid, escrow_value, tao.into()));
        }
        out
    }

    /// Network-wide total beta basket NAV across all validators, in TAO (mark-to-market).
    /// Sampling this over time yields the TAO/day flowing to root stakers.
    pub fn get_root_basket_total_nav_tao() -> TaoBalance {
        let escrow = Self::get_beta_escrow_account_id();
        let mut nav: u64 = 0;
        for (hotkey, netuid, _principal) in BasketPrincipal::<T>::iter() {
            let escrow_value =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, &escrow, netuid).to_u64();
            nav = nav.saturating_add(Self::alpha_to_tao_value(netuid, escrow_value));
        }
        nav.into()
    }
}
