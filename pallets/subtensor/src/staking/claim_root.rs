use super::*;
use frame_support::weights::Weight;
use sp_core::Get;
use sp_runtime::SaturatedConversion;
use sp_std::collections::btree_set::BTreeSet;
use substrate_fixed::types::I96F32;
use subtensor_swap_interface::SwapHandler;

pub(crate) const ROOT_CLAIM_CLEANUP_BATCH_SIZE: usize = 100;

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
        amount: AlphaCurrency,
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
        RootClaimable::<T>::mutate(hotkey, |claimable| {
            claimable
                .entry(netuid)
                .and_modify(|claim_total| *claim_total = claim_total.saturating_add(increment))
                .or_insert(increment);
        });
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

    pub fn root_claim_on_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
        root_claim_type: RootClaimTypeEnum,
        ignore_minimum_condition: bool,
    ) {
        // Subtract the root claimed.
        let owed: I96F32 = Self::get_root_owed_for_hotkey_coldkey_float(hotkey, coldkey, netuid);

        if !ignore_minimum_condition
            && owed < I96F32::saturating_from_num(RootClaimableThreshold::<T>::get(&netuid))
        {
            log::debug!(
                "root claim on subnet {netuid} is skipped: {owed:?} for h={hotkey:?},c={coldkey:?} "
            );
            return; // no-op
        }

        // Convert owed to u64, mapping negative values to 0
        let owed_u64: u64 = if owed.is_negative() {
            0
        } else {
            owed.saturating_to_num::<u64>()
        };

        if owed_u64 == 0 {
            log::debug!(
                "root claim on subnet {netuid} is skipped: {owed:?} for h={hotkey:?},c={coldkey:?}"
            );
            return; // no-op
        }

        match root_claim_type {
            //  Increase stake on root
            RootClaimTypeEnum::Swap => {
                // Swap the alpha owed to TAO
                let owed_tao = match Self::swap_alpha_for_tao(
                    netuid,
                    owed_u64.into(),
                    T::SwapInterface::min_price::<TaoCurrency>(),
                    true,
                ) {
                    Ok(owed_tao) => owed_tao,
                    Err(err) => {
                        log::error!("Error swapping alpha for TAO: {err:?}");

                        return;
                    }
                };

                Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                    hotkey,
                    coldkey,
                    NetUid::ROOT,
                    owed_tao.amount_paid_out.to_u64().into(),
                );

                Self::add_stake_adjust_root_claimed_for_hotkey_and_coldkey(
                    hotkey,
                    coldkey,
                    owed_tao.amount_paid_out.into(),
                );
            }
            RootClaimTypeEnum::Keep => {
                // Increase the stake with the alpha owned
                Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                    hotkey,
                    coldkey,
                    netuid,
                    owed_u64.into(),
                );
            }
        };

        // Increase root claimed by owed amount.
        RootClaimed::<T>::mutate((netuid, hotkey, coldkey), |root_claimed| {
            *root_claimed = root_claimed.saturating_add(owed_u64.into());
        });
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
    ) -> Weight {
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

            Self::root_claim_on_subnet(hotkey, coldkey, *netuid, root_claim_type.clone(), false);
            weight.saturating_accrue(Self::root_claim_on_subnet_weight(root_claim_type.clone()));
        }

        weight
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
        amount: AlphaCurrency,
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

    pub fn do_root_claim(coldkey: T::AccountId, subnets: Option<BTreeSet<NetUid>>) -> Weight {
        let mut weight = Weight::default();

        let hotkeys = StakingHotkeys::<T>::get(&coldkey);
        weight.saturating_accrue(T::DbWeight::get().reads(1));

        hotkeys.iter().for_each(|hotkey| {
            weight.saturating_accrue(T::DbWeight::get().reads(1));
            weight.saturating_accrue(Self::root_claim_all(hotkey, &coldkey, subnets.clone()));
        });

        Self::deposit_event(Event::RootClaimed { coldkey });

        weight
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
                weight.saturating_accrue(Self::do_root_claim(coldkey.clone(), None));
            }

            continue;
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

    /// Start removing root claimable for the given subnet.
    pub fn start_removing_root_claim_data_for_subnet(netuid: NetUid) {
        // TODO: check for empty RootClaimSubnetCleanup and return DispatchResult
        // TODO: prevent double network dissolution
        // TODO: prevent network creation during removal

        let root_cleanup_data = RootClaimSubnetCleanup {
            netuid,
            last_root_claimable_hotkey: None,
            root_claim_cleanup_started: false,
        };

        Self::deposit_event(Event::RootClaimCleanupStarted { netuid });

        LastRootClaimCleanupData::<T>::put(root_cleanup_data);
    }

    pub fn iterate_and_clean_root_claim_data_for_subnet() {
        let Some(mut root_cleanup_data) = LastRootClaimCleanupData::<T>::get() else {
            return; // nothing to clean yet
        };

        let netuid = root_cleanup_data.netuid;

        // Initialize RootClaimable cleanup
        let mut starting_root_claimable_key = None;
        if root_cleanup_data.last_root_claimable_hotkey.is_none()
            && !root_cleanup_data.root_claim_cleanup_started
        {
            starting_root_claimable_key = RootClaimable::<T>::iter_keys().next();

            root_cleanup_data.last_root_claimable_hotkey = starting_root_claimable_key
                .as_ref()
                .map(RootClaimable::<T>::hashed_key_for);
        }

        // Clean RootClaimable
        if let Some(starting_raw_key) = root_cleanup_data.last_root_claimable_hotkey {
            // Get the key batch
            let mut hotkeys = RootClaimable::<T>::iter_keys_from(starting_raw_key)
                .take(ROOT_CLAIM_CLEANUP_BATCH_SIZE)
                .collect::<Vec<_>>();

            // New iteration: insert the starting key in the batch if it's a new iteration
            // iter_keys_from() skips the starting key
            if let Some(starting_key) = starting_root_claimable_key {
                if hotkeys.len() == ROOT_CLAIM_CLEANUP_BATCH_SIZE {
                    hotkeys.remove(hotkeys.len().saturating_sub(1));
                }
                hotkeys.insert(0, starting_key);
            }

            let mut new_starting_key = None;
            let new_iteration = hotkeys.len() < ROOT_CLAIM_CLEANUP_BATCH_SIZE;

            // Remove RootClaimable for subnet
            for hotkey in hotkeys {
                RootClaimable::<T>::mutate(&hotkey, |claimable| {
                    claimable.remove(&netuid);
                });

                new_starting_key = Some(RootClaimable::<T>::hashed_key_for(&hotkey));
            }

            Self::deposit_event(Event::RootClaimCleanupProgress {
                netuid,
                root_claimable_iteration: true,
                root_claimed_iteration: false,
            });

            // Continue with RootClaimed if it's the last batch
            if new_iteration {
                new_starting_key = None;

                // Start RootClaimed cleanup next block
                root_cleanup_data.root_claim_cleanup_started = true;
            }

            root_cleanup_data.last_root_claimable_hotkey = new_starting_key;

            LastRootClaimCleanupData::<T>::put(root_cleanup_data);

            return; // single iteration finished
        }

        // Clean RootClaimed
        if root_cleanup_data.root_claim_cleanup_started {
            let root_claimed_result = RootClaimed::<T>::clear_prefix(
                (netuid,),
                ROOT_CLAIM_CLEANUP_BATCH_SIZE.saturated_into::<u32>(),
                None,
            );

            Self::deposit_event(Event::RootClaimCleanupProgress {
                netuid,
                root_claimable_iteration: false,
                root_claimed_iteration: true,
            });

            if root_claimed_result.maybe_cursor.is_some() {
                return; // continue next block
            }
            // Finished cleaning RootClaimed
        }

        // Finish cleaning RootClaimable for subnet
        LastRootClaimCleanupData::<T>::take();

        Self::deposit_event(Event::RootClaimCleanupFinished { netuid });
    }
}
