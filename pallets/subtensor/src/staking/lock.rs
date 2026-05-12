use super::*;
use safe_math::FixedExt;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::ops::Neg;
use substrate_fixed::transcendental::exp;
use substrate_fixed::types::{I64F64, U64F64};
use subtensor_runtime_common::NetUid;

impl<T: Config> Pallet<T> {
    pub fn insert_lock_state(
        coldkey: &T::AccountId,
        netuid: NetUid,
        hotkey: &T::AccountId,
        lock_state: LockState,
    ) {
        if !lock_state.locked_mass.is_zero() || lock_state.conviction > U64F64::saturating_from_num(0) {
            Lock::<T>::insert((coldkey, netuid, hotkey), lock_state);
        } else {
            // If there is no record previously, this is a no-op
            Lock::<T>::remove((coldkey, netuid, hotkey));
        }
    }

    pub fn insert_hotkey_lock_state(netuid: NetUid, hotkey: &T::AccountId, lock_state: LockState) {
        if !lock_state.locked_mass.is_zero() || lock_state.conviction > U64F64::saturating_from_num(0) {
            HotkeyLock::<T>::insert(netuid, hotkey, lock_state);
        } else {
            HotkeyLock::<T>::remove(netuid, hotkey);
        }
    }

    /// Computes exp(-dt / tau) as a U64F64 decay factor.
    pub fn exp_decay(dt: u64, tau: u64) -> U64F64 {
        if tau == 0 || dt == 0 {
            if dt == 0 {
                return U64F64::saturating_from_num(1);
            }
            return U64F64::saturating_from_num(0);
        }
        let min_ratio = I64F64::saturating_from_num(-40);
        let neg_ratio = I64F64::saturating_from_num((dt as i128).neg())
            .checked_div(I64F64::saturating_from_num(tau))
            .unwrap_or(min_ratio);
        let clamped = neg_ratio.max(min_ratio);
        let decay: I64F64 = exp(clamped).unwrap_or(I64F64::saturating_from_num(0));
        if decay < I64F64::saturating_from_num(0) {
            U64F64::saturating_from_num(0)
        } else {
            U64F64::saturating_from_num(decay)
        }
    }

    fn calculate_decayed_mass_and_conviction(
        locked_mass: AlphaBalance,
        conviction: U64F64,
        dt: u64,
    ) -> (AlphaBalance, U64F64) {
        let tau = TauBlocks::<T>::get();

        let decay = Self::exp_decay(dt, tau);
        let dt_fixed = U64F64::saturating_from_num(dt);
        let mass_fixed = U64F64::saturating_from_num(locked_mass);
        let tau_fixed = U64F64::saturating_from_num(tau);
        let new_locked_mass = decay
            .saturating_mul(mass_fixed)
            .saturating_to_num::<u64>()
            .into();
        let new_conviction = decay.saturating_mul(
            conviction.saturating_add(dt_fixed.safe_div(tau_fixed).saturating_mul(mass_fixed)),
        );
        (new_locked_mass, new_conviction)
    }

    /// Rolls a LockState forward to `now` using exponential decay.
    ///
    /// X_new = decay * X_old
    /// Y_new = decay * (Y_old + dt * X_old)
    pub fn roll_forward_lock(lock: LockState, now: u64) -> LockState {
        if now <= lock.last_update {
            return lock;
        }
        let dt = now.saturating_sub(lock.last_update);
        let (new_locked_mass, new_conviction) =
            Self::calculate_decayed_mass_and_conviction(lock.locked_mass, lock.conviction, dt);

        LockState {
            locked_mass: new_locked_mass,
            conviction: new_conviction,
            last_update: now,
        }
    }

    /// Returns the sum of raw alpha shares for a coldkey across all hotkeys on a given subnet.
    pub fn total_coldkey_alpha_on_subnet(coldkey: &T::AccountId, netuid: NetUid) -> AlphaBalance {
        StakingHotkeys::<T>::get(coldkey)
            .into_iter()
            .map(|hotkey| {
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(&hotkey, coldkey, netuid)
            })
            .fold(AlphaBalance::ZERO, |acc, stake| acc.saturating_add(stake))
    }

    /// Returns the current locked amount for a coldkey on a subnet.
    /// No rolling forward is needed because locked mass does not decay over time.
    pub fn get_current_locked(coldkey: &T::AccountId, netuid: NetUid) -> AlphaBalance {
        Lock::<T>::iter_prefix((coldkey, netuid))
            .next()
            .map(|(_hotkey, lock)| lock.locked_mass)
            .unwrap_or(AlphaBalance::ZERO)
    }

    /// Returns the current conviction for a coldkey on a subnet (rolled forward to now).
    pub fn get_conviction(coldkey: &T::AccountId, netuid: NetUid) -> U64F64 {
        let now = Self::get_current_block_as_u64();
        Lock::<T>::iter_prefix((coldkey, netuid))
            .next()
            .map(|(_hotkey, lock)| Self::roll_forward_lock(lock, now).conviction)
            .unwrap_or_else(|| U64F64::saturating_from_num(0))
    }

    /// Returns the alpha amount available to unstake for a coldkey on a subnet.
    pub fn available_to_unstake(coldkey: &T::AccountId, netuid: NetUid) -> AlphaBalance {
        let total = Self::total_coldkey_alpha_on_subnet(coldkey, netuid);
        let locked = Self::get_current_locked(coldkey, netuid);
        if total > locked {
            total.saturating_sub(locked)
        } else {
            AlphaBalance::ZERO
        }
    }

    /// Ensures that the amount can be unstaked
    pub fn ensure_available_to_unstake(
        coldkey: &T::AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
    ) -> Result<(), Error<T>> {
        let alpha_available = Self::available_to_unstake(coldkey, netuid);
        ensure!(alpha_available >= amount, Error::<T>::StakeUnavailable);
        Ok(())
    }

    /// Locks stake for a coldkey on a subnet to a specific hotkey.
    /// If no lock exists, creates one. If one exists, the hotkey must match.
    /// Top-up adds to locked_mass after rolling forward.
    pub fn do_lock_stake(
        coldkey: &T::AccountId,
        netuid: NetUid,
        hotkey: &T::AccountId,
        amount: AlphaBalance,
    ) -> dispatch::DispatchResult {
        ensure!(!amount.is_zero(), Error::<T>::AmountTooLow);

        let total = Self::total_coldkey_alpha_on_subnet(coldkey, netuid);
        let now = Self::get_current_block_as_u64();

        let existing = Lock::<T>::iter_prefix((coldkey, netuid)).next();

        match existing {
            None => {
                ensure!(total >= amount, Error::<T>::InsufficientStakeForLock);
                Self::insert_lock_state(
                    coldkey,
                    netuid,
                    hotkey,
                    LockState {
                        locked_mass: amount,
                        conviction: U64F64::saturating_from_num(0),
                        last_update: now,
                    },
                );
            }
            Some((existing_hotkey, existing)) => {
                ensure!(*hotkey == existing_hotkey, Error::<T>::LockHotkeyMismatch);

                let lock = Self::roll_forward_lock(existing, now);
                let new_locked = lock.locked_mass.saturating_add(amount);
                ensure!(total >= new_locked, Error::<T>::InsufficientStakeForLock);
                Self::insert_lock_state(
                    coldkey,
                    netuid,
                    hotkey,
                    LockState {
                        locked_mass: new_locked,
                        conviction: lock.conviction,
                        last_update: now,
                    },
                );
            }
        }

        // Update the total hotkey lock
        Self::upsert_hotkey_lock(hotkey, netuid, amount);

        Self::deposit_event(Event::StakeLocked {
            coldkey: coldkey.clone(),
            hotkey: hotkey.clone(),
            netuid,
            amount,
        });

        Ok(())
    }

    /// Reduces the coldkey lock by a specified alpha amount and the coldkey conviction
    /// proportionally.
    pub fn force_reduce_lock(coldkey: &T::AccountId, netuid: NetUid, amount: AlphaBalance) {
        if let Some((existing_hotkey, lock)) = Lock::<T>::iter_prefix((coldkey, netuid)).next() {
            let now = Self::get_current_block_as_u64();
            let rolled = Self::roll_forward_lock(lock, now);
            let new_locked_mass = rolled.locked_mass.saturating_sub(amount);

            // Remove or update lock
            let conviction_diff = if new_locked_mass.is_zero() {
                Lock::<T>::remove((coldkey.clone(), netuid, existing_hotkey.clone()));
                rolled.conviction
            } else {
                let removed_proportion = U64F64::saturating_from_num(u64::from(amount))
                    .safe_div(U64F64::saturating_from_num(u64::from(rolled.locked_mass)));
                let new_conviction = rolled.conviction.saturating_mul(
                    U64F64::saturating_from_num(1).saturating_sub(removed_proportion),
                );
                Lock::<T>::insert(
                    (coldkey.clone(), netuid, existing_hotkey.clone()),
                    LockState {
                        locked_mass: new_locked_mass,
                        conviction: new_conviction,
                        last_update: now,
                    },
                );
                rolled.conviction.saturating_sub(new_conviction)
            };

            // Reduce the total hotkey lock by the rolled locked mass and conviction
            Self::reduce_hotkey_lock(&existing_hotkey, netuid, amount, conviction_diff);
        }
    }

    /// Rolls the lock forward to now and persists it if the locked mass is zero. This is used when we want to
    /// update the lock when a user stakes or unstakes.
    pub fn cleanup_lock_if_zero(coldkey: &T::AccountId, netuid: NetUid) {
        let now = Self::get_current_block_as_u64();

        // Cleanup locks for the specific coldkey and hotkey
        if let Some((hotkey, lock)) = Lock::<T>::iter_prefix((coldkey.clone(), netuid)).next() {
            let rolled = Self::roll_forward_lock(lock, now);
            if rolled.locked_mass.is_zero() {
                Lock::<T>::remove((coldkey.clone(), netuid, hotkey.clone()));
            }

            // Also cleanup the hotkey lock
            if let Some(lock) = HotkeyLock::<T>::get(netuid, &hotkey) {
                let rolled = Self::roll_forward_lock(lock, now);
                if rolled.locked_mass.is_zero() {
                    HotkeyLock::<T>::remove(netuid, hotkey);
                }
            }
        }
    }

    /// Update the total lock for a hotkey on a subnet or create one if
    /// it doesn't exist.
    ///
    /// Roll the existing hotkey lock forward to now, then add the
    /// latest conviction and locked mass.
    pub fn upsert_hotkey_lock(hotkey: &T::AccountId, netuid: NetUid, amount: AlphaBalance) {
        let total_lock = HotkeyLock::<T>::get(netuid, hotkey);

        // Roll forward the total lock to now
        let now = Self::get_current_block_as_u64();
        let rolled_hotkey_lock = if let Some(lock) = total_lock {
            Self::roll_forward_lock(lock, now)
        } else {
            LockState {
                locked_mass: 0.into(),
                conviction: U64F64::saturating_from_num(0),
                last_update: now,
            }
        };

        // Merge the new lock into the rolled total lock (only add mass)
        let new_locked_mass = rolled_hotkey_lock.locked_mass.saturating_add(amount);
        let new_hotkey_lock = LockState {
            locked_mass: new_locked_mass,
            conviction: rolled_hotkey_lock.conviction,
            last_update: now,
        };
        Self::insert_hotkey_lock_state(netuid, hotkey, new_hotkey_lock);
    }

    /// Reduce the total lock for a hotkey on a subnet. This is called when a lock is removed or reduced.
    pub fn reduce_hotkey_lock(
        hotkey: &T::AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
        conviction: U64F64,
    ) {
        if let Some(lock) = HotkeyLock::<T>::get(netuid, hotkey) {
            let now = Self::get_current_block_as_u64();
            let rolled_hotkey_lock = Self::roll_forward_lock(lock, now);
            let new_locked_mass = rolled_hotkey_lock.locked_mass.saturating_sub(amount);
            let new_conviction = rolled_hotkey_lock.conviction.saturating_sub(conviction);
            Self::insert_hotkey_lock_state(
                netuid,
                hotkey,
                LockState {
                    locked_mass: new_locked_mass,
                    conviction: new_conviction,
                    last_update: now,
                },
            );
        }
    }

    /// Returns the total conviction for a hotkey on a subnet,
    /// summed over all coldkeys that have locked to this hotkey.
    pub fn hotkey_conviction(hotkey: &T::AccountId, netuid: NetUid) -> U64F64 {
        let lock = HotkeyLock::<T>::get(netuid, hotkey);
        if let Some(lock) = lock {
            Self::roll_forward_lock(lock, Self::get_current_block_as_u64()).conviction
        } else {
            U64F64::saturating_from_num(0)
        }
    }

    /// Finds the hotkey with the highest conviction on a given subnet.
    pub fn subnet_king(netuid: NetUid) -> Option<T::AccountId> {
        let now = Self::get_current_block_as_u64();
        let mut scores: BTreeMap<T::AccountId, U64F64> = BTreeMap::new();

        HotkeyLock::<T>::iter_prefix(netuid).for_each(|(hotkey, lock)| {
            let rolled = Self::roll_forward_lock(lock, now);
            let entry = scores
                .entry(hotkey)
                .or_insert_with(|| U64F64::saturating_from_num(0));
            *entry = entry.saturating_add(rolled.conviction);
        });

        scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal))
            .map(|(hotkey, _)| hotkey)
    }

    /// Ensure the coldkey does not have an active lock on any subnets.
    pub fn ensure_no_active_locks(coldkey: &T::AccountId) -> Result<(), Error<T>> {
        let now = Self::get_current_block_as_u64();

        for ((_netuid, _hotkey), lock) in Lock::<T>::iter_prefix((coldkey,)) {
            let rolled = Self::roll_forward_lock(lock, now);
            if rolled.locked_mass > AlphaBalance::ZERO {
                return Err(Error::<T>::ActiveLockExists);
            }
        }

        Ok(())
    }

    /// Transfers the lock from one coldkey to another for all subnets. This is used when a
    /// user swaps their coldkey and we want to preserve their locks.
    ///
    /// The hotkey and netuid remain the same, only the coldkey changes.
    ///
    /// The new coldkey is guaranteed to have no active locks (checked in ensure_no_active_locks),
    /// so we can simply transfer the locks "as is" without rolling them forward and the
    /// HotkeyLock map does not change (because it only contains totals, not individual coldkey locks).
    pub fn swap_coldkey_locks(old_coldkey: &T::AccountId, new_coldkey: &T::AccountId) {
        let mut locks_to_transfer: Vec<(NetUid, T::AccountId, LockState)> = Vec::new();

        // Gather locks for old coldkey
        for ((netuid, hotkey), lock) in Lock::<T>::iter_prefix((old_coldkey,)) {
            locks_to_transfer.push((netuid, hotkey, lock));
        }

        // Remove locks for old coldkey and insert for new
        for (netuid, hotkey, lock) in locks_to_transfer {
            Lock::<T>::remove((old_coldkey.clone(), netuid, hotkey.clone()));
            Self::insert_lock_state(new_coldkey, netuid, &hotkey, lock);
        }
    }

    /// Swap all locks made to the old_hotkey to new_hotkey on all netuids
    ///
    /// There is no need to roll the locks, they can be just copied "as is":
    /// The lock relation between coldkeys and hotkey is 1:1, so if old hotkey has a
    /// coldkey locking to it, then the same coldkey cannot lock to the new hotkey.
    /// And in reverse: If a coldkey is locking to the new hotkey, it will not appear
    /// in the transfer list because it does not lock to the old hotkey.
    ///
    /// Conviction is not reset because the hotkey ownership does not change, it's still
    /// the same hotkey owner who will own the new hotkey.
    pub fn swap_hotkey_locks(old_hotkey: &T::AccountId, new_hotkey: &T::AccountId) -> (u64, u64) {
        let mut locks_to_transfer: Vec<(T::AccountId, NetUid, T::AccountId, LockState)> =
            Vec::new();
        let mut hotkey_locks_to_transfer: Vec<(NetUid, LockState)> = Vec::new();
        let mut reads: u64 = 0;
        let mut writes: u64 = 0;

        let netuids = Self::get_all_subnet_netuids();

        // Gather hotkey locks for old hotkey
        for netuid in netuids {
            if let Some(lock) = HotkeyLock::<T>::get(netuid, old_hotkey) {
                hotkey_locks_to_transfer.push((netuid, lock));
            }
            reads = reads.saturating_add(1);
        }

        // Gather locks for old hotkey (only if hotkey locks exist, otherwise skip to save reads)
        if !hotkey_locks_to_transfer.is_empty() {
            for ((coldkey, netuid, hotkey), lock) in Lock::<T>::iter() {
                if hotkey == *old_hotkey {
                    locks_to_transfer.push((coldkey, netuid, hotkey, lock));
                }
                reads = reads.saturating_add(1);
            }
        }

        // Remove locks for old hotkey and insert for new
        for (coldkey, netuid, _hotkey, lock) in locks_to_transfer {
            Lock::<T>::remove((coldkey.clone(), netuid, old_hotkey.clone()));
            Self::insert_lock_state(&coldkey, netuid, new_hotkey, lock);
            writes = writes.saturating_add(2);
        }

        // Remove hotkey locks for old hotkey and insert for new
        for (netuid, lock) in hotkey_locks_to_transfer {
            HotkeyLock::<T>::remove(netuid, old_hotkey);
            Self::insert_hotkey_lock_state(netuid, new_hotkey, lock);
            writes = writes.saturating_add(2);
        }
        (reads, writes)
    }

    /// Moves lock from one hotkey to another and clears conviction
    ///
    /// The lock is rolled forward to the current block before switching the
    /// associated hotkey so that the lock stays mathematically correct and
    /// preserves current decayed locked mass.
    ///
    /// The conviction is reset to zero if the destination and source hotkeys
    /// are owned by different coldkeys, otherwise it is preserved.
    pub fn do_move_lock(
        coldkey: &T::AccountId,
        destination_hotkey: &T::AccountId,
        netuid: NetUid,
    ) -> DispatchResult {
        let now = Self::get_current_block_as_u64();

        match Lock::<T>::iter_prefix((coldkey, netuid)).next() {
            Some((origin_hotkey, existing)) => {
                let old_hotkey_owner = Self::get_owning_coldkey_for_hotkey(&origin_hotkey);
                let new_hotkey_owner = Self::get_owning_coldkey_for_hotkey(destination_hotkey);
                let same_owner = old_hotkey_owner != DefaultAccount::<T>::get()
                    && new_hotkey_owner != DefaultAccount::<T>::get()
                    && old_hotkey_owner == new_hotkey_owner;

                let mut existing_rolled = Self::roll_forward_lock(existing, now);
                let existing_conviction = existing_rolled.conviction;
                if !same_owner {
                    existing_rolled.conviction = U64F64::saturating_from_num(0);
                }

                Lock::<T>::remove((coldkey.clone(), netuid, origin_hotkey.clone()));
                Self::insert_lock_state(
                    coldkey,
                    netuid,
                    destination_hotkey,
                    LockState {
                        locked_mass: existing_rolled.locked_mass,
                        conviction: existing_rolled.conviction,
                        last_update: now,
                    },
                );

                // Update the total hotkey locks for destination hotkey
                Self::upsert_hotkey_lock(destination_hotkey, netuid, existing_rolled.locked_mass);

                // Reduce the total hotkey locks and conviction for the origin hotkey
                Self::reduce_hotkey_lock(
                    &origin_hotkey,
                    netuid,
                    existing_rolled.locked_mass,
                    existing_conviction,
                );

                // If the same coldkey owns both the origin and destination hotkeys, also
                // transfer the conviction instead of resetting it
                if same_owner {
                    HotkeyLock::<T>::mutate(netuid, destination_hotkey, |dest_lock_opt| {
                        if let Some(dest_lock) = dest_lock_opt {
                            dest_lock.conviction =
                                dest_lock.conviction.saturating_add(existing_conviction);
                        }
                    });
                }

                Self::deposit_event(Event::LockMoved {
                    coldkey: coldkey.clone(),
                    origin_hotkey,
                    destination_hotkey: destination_hotkey.clone(),
                    netuid,
                });
                Ok(())
            }
            None => Err(Error::<T>::NoExistingLock.into()),
        }
    }

    pub fn auto_lock_owner_cut(netuid: NetUid, amount: AlphaBalance) {
        let subnet_owner_coldkey = Self::get_subnet_owner(netuid);

        // Determine the lock hotkey. If no locks exist, assign subnet owner's hotkey, otherwise
        // auto-lock to existing lock hotkey
        let lock_hotkey = if let Some((existing_hotkey, _existing)) =
            Lock::<T>::iter_prefix((&subnet_owner_coldkey, netuid)).next()
        {
            existing_hotkey
        } else {
            SubnetOwnerHotkey::<T>::get(netuid)
        };

        // Ignore the result. It may only fail if amount is zero, which is OK to ignore because nothing
        // needs to happen in that case
        let _ = Self::do_lock_stake(&subnet_owner_coldkey, netuid, &lock_hotkey, amount);
    }

    /// When locked stake is transfered, the lock should follow the stake
    ///
    /// First, this function rolls the lock forward and checks if amount is over available
    /// stake and if it is, the stake that's over the available amount on the destination
    /// coldkey is locked in the same way as the original stake: If original stake is locked to 
    /// a hotkey, it remains actively locked to the same hotkey
    pub fn transfer_lock(
        origin_coldkey: &T::AccountId,
        destination_coldkey: &T::AccountId,
        netuid: NetUid,
        amount: AlphaBalance,
    ) -> DispatchResult {
        let now = Self::get_current_block_as_u64();

        // If no actual transfer happens, this is ok
        if origin_coldkey == destination_coldkey || amount.is_zero() {
            return Ok(());
        }

        // Read total alpha of the coldkey on this netuid. Do not check if total alpha is
        // lower than amount transferred, this is responsibility of a higher level, this
        // function needs to act protectively.
        let total_alpha = Self::total_coldkey_alpha_on_subnet(origin_coldkey, netuid);
        let mut remaining_to_transfer = amount;

        // Read the locks for source and destination coldkey (if exist) and roll forward
        let Some((source_hotkey, source_lock)) =
            Lock::<T>::iter_prefix((origin_coldkey, netuid)).next()
        else {
            return Ok(());
        };

        let mut source_lock = Self::roll_forward_lock(source_lock, now);
        let maybe_destination_lock = Lock::<T>::iter_prefix((destination_coldkey, netuid))
            .next()
            .map(|(hotkey, lock)| (hotkey, Self::roll_forward_lock(lock, now)));

        let mut destination_hotkey = maybe_destination_lock
            .as_ref()
            .map(|(hotkey, _)| hotkey.clone())
            .unwrap_or_else(|| source_hotkey.clone());
        let mut destination_lock = maybe_destination_lock
            .as_ref()
            .map(|(_, lock)| lock.clone())
            .unwrap_or(LockState {
                locked_mass: AlphaBalance::ZERO,
                conviction: U64F64::saturating_from_num(0),
                last_update: now,
            });

        // Calculate available stake by subtracting locked_mass from total alpha.
        let unavailable = source_lock.locked_mass;
        let available_stake = total_alpha.saturating_sub(unavailable);

        // Reduce remaining_to_transfer by min(remaining_to_transfer, available stake)
        let available_transfer = remaining_to_transfer.min(available_stake);
        remaining_to_transfer = remaining_to_transfer.saturating_sub(available_transfer);

        // If result is non-zero, check the hotkey match between source and destination coldkey locks
        // (if destination coldkey lock exists). If no match, error out with LockHotkeyMismatch, otherwise,
        // reduce remaining_to_transfer by min(remaining_to_transfer, locked_mass), reduce locked_mass on
        // the source coldkey by the same amount, increase locked_mass on the destination coldkey by the
        // same amount, reduce conviction on the source coldkey proportionally, and increase conviction
        // on the destination coldkey proportionally.
        if !remaining_to_transfer.is_zero() {
            if let Some((existing_hotkey, _)) = maybe_destination_lock.as_ref() {
                ensure!(
                    existing_hotkey == &source_hotkey,
                    Error::<T>::LockHotkeyMismatch
                );
                destination_hotkey = existing_hotkey.clone();
            }

            let locked_transfer = remaining_to_transfer.min(source_lock.locked_mass);
            let conviction_transfer =
                if locked_transfer.is_zero() || source_lock.locked_mass.is_zero() {
                    U64F64::saturating_from_num(0)
                } else {
                    // Conviction never exceeds locked_mass, so we can scale it proportionally
                    // using integer arithmetic without overflowing fixed-point multiplication.
                    let conviction_u128 = source_lock.conviction.saturating_to_num::<u128>();
                    let locked_transfer_u128 = locked_transfer.to_u64() as u128;
                    let source_locked_u128 = source_lock.locked_mass.to_u64() as u128;
                    let transferred_conviction_u128 = conviction_u128
                        .saturating_mul(locked_transfer_u128)
                        .checked_div(source_locked_u128)
                        .unwrap_or(0);
                    U64F64::saturating_from_num(transferred_conviction_u128)
                };

            source_lock.locked_mass = source_lock.locked_mass.saturating_sub(locked_transfer);
            source_lock.conviction = source_lock.conviction.saturating_sub(conviction_transfer);
            destination_lock.locked_mass =
                destination_lock.locked_mass.saturating_add(locked_transfer);
            destination_lock.conviction = destination_lock
                .conviction
                .saturating_add(conviction_transfer);
        }

        source_lock.last_update = now;
        destination_lock.last_update = now;

        // Upsert updated locks (only once per this fn) even if there were no updates because
        // of roll-forward
        Self::insert_lock_state(origin_coldkey, netuid, &source_hotkey, source_lock);
        Self::insert_lock_state(
            destination_coldkey,
            netuid,
            &destination_hotkey,
            destination_lock,
        );

        Ok(())
    }
}
