use super::*;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::ops::Neg;
use substrate_fixed::transcendental::exp;
use substrate_fixed::types::{I64F64, U64F64};
use subtensor_runtime_common::NetUid;

impl<T: Config> Pallet<T> {
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
        let result: I64F64 = exp(clamped).unwrap_or(I64F64::saturating_from_num(0));
        if result < I64F64::saturating_from_num(0) {
            U64F64::saturating_from_num(0)
        } else {
            U64F64::saturating_from_num(result)
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
        let new_locked_mass = decay
            .saturating_mul(mass_fixed)
            .saturating_to_num::<u64>()
            .into();
        let new_conviction =
            decay.saturating_mul(conviction.saturating_add(dt_fixed.saturating_mul(mass_fixed)));
        (new_locked_mass, new_conviction)
    }

    /// Rolls a LockState forward to `now` using exponential decay.
    ///
    /// X_new = decay * X_old
    /// Y_new = decay * (Y_old + dt * X_old)
    pub fn roll_forward_lock(lock: LockState<T::AccountId>, now: u64) -> LockState<T::AccountId> {
        if now <= lock.last_update {
            return lock;
        }
        let dt = now.saturating_sub(lock.last_update);
        let (new_locked_mass, new_conviction) =
            Self::calculate_decayed_mass_and_conviction(lock.locked_mass, lock.conviction, dt);

        LockState {
            hotkey: lock.hotkey,
            locked_mass: new_locked_mass,
            conviction: new_conviction,
            last_update: now,
        }
    }

    /// Rolls a HotkeyLockState forward to `now` using exponential decay.
    pub fn roll_forward_hotkey_lock(lock: HotkeyLockState, now: u64) -> HotkeyLockState {
        if now <= lock.last_update {
            return lock;
        }
        let dt = now.saturating_sub(lock.last_update);
        let (new_locked_mass, new_conviction) =
            Self::calculate_decayed_mass_and_conviction(lock.locked_mass, lock.conviction, dt);

        HotkeyLockState {
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

    /// Returns the current locked amount for a coldkey on a subnet (rolled forward to now).
    pub fn get_current_locked(coldkey: &T::AccountId, netuid: NetUid) -> AlphaBalance {
        let now = Self::get_current_block_as_u64();
        match Lock::<T>::get(coldkey, netuid) {
            Some(lock) => Self::roll_forward_lock(lock, now).locked_mass,
            None => AlphaBalance::ZERO,
        }
    }

    /// Returns the current conviction for a coldkey on a subnet (rolled forward to now).
    pub fn get_conviction(coldkey: &T::AccountId, netuid: NetUid) -> U64F64 {
        let now = Self::get_current_block_as_u64();
        match Lock::<T>::get(coldkey, netuid) {
            Some(lock) => Self::roll_forward_lock(lock, now).conviction,
            None => U64F64::saturating_from_num(0),
        }
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
        ensure!(alpha_available >= amount, Error::<T>::CannotUnstakeLock);
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

        match Lock::<T>::get(coldkey, netuid) {
            None => {
                ensure!(total >= amount, Error::<T>::InsufficientStakeForLock);
                Lock::<T>::insert(
                    coldkey,
                    netuid,
                    LockState {
                        hotkey: hotkey.clone(),
                        locked_mass: amount,
                        conviction: U64F64::saturating_from_num(0),
                        last_update: now,
                    },
                );
            }
            Some(existing) => {
                ensure!(*hotkey == existing.hotkey, Error::<T>::LockHotkeyMismatch);
                let lock = Self::roll_forward_lock(existing, now);
                let new_locked = lock.locked_mass.saturating_add(amount);
                ensure!(total >= new_locked, Error::<T>::InsufficientStakeForLock);
                Lock::<T>::insert(
                    coldkey,
                    netuid,
                    LockState {
                        hotkey: lock.hotkey,
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

    /// Clears the lock. This function will be called if the alpha stake drops below minimum
    /// threshold.
    pub fn maybe_cleanup_lock(coldkey: &T::AccountId, netuid: NetUid) {
        if let Some(lock) = Lock::<T>::get(coldkey, netuid) {
            let now = Self::get_current_block_as_u64();
            let rolled = Self::roll_forward_lock(lock, now);
            Lock::<T>::remove(coldkey, netuid);

            // Reduce the total hotkey lock by the rolled locked mass and conviction
            Self::reduce_hotkey_lock(
                &rolled.hotkey,
                netuid,
                rolled.locked_mass,
                rolled.conviction,
            );
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
            Self::roll_forward_hotkey_lock(lock, now)
        } else {
            HotkeyLockState {
                locked_mass: 0.into(),
                conviction: U64F64::saturating_from_num(0),
                last_update: now,
            }
        };

        // Merge the new lock into the rolled total lock (only add mass)
        let new_locked_mass = rolled_hotkey_lock.locked_mass.saturating_add(amount);
        let new_hotkey_lock = HotkeyLockState {
            locked_mass: new_locked_mass,
            conviction: rolled_hotkey_lock.conviction,
            last_update: now,
        };
        HotkeyLock::<T>::insert(netuid, hotkey, new_hotkey_lock);
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
            let rolled_hotkey_lock = Self::roll_forward_hotkey_lock(lock, now);
            let new_locked_mass = rolled_hotkey_lock.locked_mass.saturating_sub(amount);
            let new_conviction = rolled_hotkey_lock.conviction.saturating_sub(conviction);
            if new_locked_mass.is_zero() {
                HotkeyLock::<T>::remove(netuid, hotkey);
            } else {
                let new_hotkey_lock = HotkeyLockState {
                    locked_mass: new_locked_mass,
                    conviction: new_conviction,
                    last_update: now,
                };
                HotkeyLock::<T>::insert(netuid, hotkey, new_hotkey_lock);
            }
        }
    }

    /// Returns the total conviction for a hotkey on a subnet,
    /// summed over all coldkeys that have locked to this hotkey.
    pub fn hotkey_conviction(hotkey: &T::AccountId, netuid: NetUid) -> U64F64 {
        let lock = HotkeyLock::<T>::get(netuid, hotkey);
        if let Some(lock) = lock {
            Self::roll_forward_hotkey_lock(lock, Self::get_current_block_as_u64()).conviction
        } else {
            U64F64::saturating_from_num(0)
        }
    }

    /// Finds the hotkey with the highest conviction on a given subnet.
    pub fn subnet_king(netuid: NetUid) -> Option<T::AccountId> {
        let now = Self::get_current_block_as_u64();
        let mut scores: BTreeMap<T::AccountId, U64F64> = BTreeMap::new();

        HotkeyLock::<T>::iter_prefix(netuid).for_each(|(hotkey, lock)| {
            let rolled = Self::roll_forward_hotkey_lock(lock, now);
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
        let netuids = Self::get_all_subnet_netuids();
        for netuid in netuids {
            if let Some(lock) = Lock::<T>::get(coldkey, netuid) {
                let rolled = Self::roll_forward_lock(lock, now);
                if rolled.locked_mass > AlphaBalance::ZERO {
                    return Err(Error::<T>::ActiveLockExists);
                }
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
        let mut locks_to_transfer: Vec<(NetUid, LockState<T::AccountId>)> = Vec::new();

        // Gather locks for old coldkey
        Lock::<T>::iter()
            .filter(|(coldkey, _, _)| coldkey == old_coldkey)
            .for_each(|(_, netuid, lock)| {
                locks_to_transfer.push((netuid, lock));
            });

        // Remove locks for old coldkey and insert for new
        for (netuid, lock) in locks_to_transfer {
            Lock::<T>::remove(old_coldkey, netuid);
            Lock::<T>::insert(new_coldkey, netuid, lock);
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
    /// If the hotkeys are owned by different coldkeys, the conviction is reset on this
    /// swap.
    pub fn swap_hotkey_locks(old_hotkey: &T::AccountId, new_hotkey: &T::AccountId) -> (u64, u64) {
        let mut locks_to_transfer: Vec<(T::AccountId, NetUid, LockState<T::AccountId>)> =
            Vec::new();
        let mut hotkey_locks_to_transfer: Vec<(NetUid, HotkeyLockState)> = Vec::new();
        let mut reads: u64 = 0;
        let mut writes: u64 = 0;

        let old_hotkey_owner = Self::get_owning_coldkey_for_hotkey(old_hotkey);
        let new_hotkey_owner = Self::get_owning_coldkey_for_hotkey(new_hotkey);
        let same_owner = old_hotkey_owner != DefaultAccount::<T>::get()
            && new_hotkey_owner != DefaultAccount::<T>::get()
            && old_hotkey_owner == new_hotkey_owner;
        reads = reads.saturating_add(2);

        // Gather locks for old hotkey
        Lock::<T>::iter()
            .filter(|(_, _, lock)| lock.hotkey == *old_hotkey)
            .for_each(|(coldkey, netuid, lock)| {
                locks_to_transfer.push((coldkey, netuid, lock));
                reads = reads.saturating_add(1);
            });

        // Gather hotkey locks for old hotkey
        HotkeyLock::<T>::iter()
            .filter(|(_, hotkey, _)| hotkey == old_hotkey)
            .for_each(|(netuid, _, lock)| {
                hotkey_locks_to_transfer.push((netuid, lock));
                reads = reads.saturating_add(1);
            });

        // Remove locks for old hotkey and insert for new
        for (coldkey, netuid, mut lock) in locks_to_transfer {
            Lock::<T>::remove(&coldkey, netuid);
            lock.hotkey = new_hotkey.clone();
            if !same_owner {
                // Reset conviction if hotkey ownership changes
                lock.conviction = U64F64::saturating_from_num(0);
            }
            Lock::<T>::insert(coldkey, netuid, lock);
            writes = writes.saturating_add(2);
        }

        // Remove hotkey locks for old hotkey and insert for new
        for (netuid, mut lock) in hotkey_locks_to_transfer {
            HotkeyLock::<T>::remove(netuid, old_hotkey);
            if !same_owner {
                // Reset conviction if hotkey ownership changes
                lock.conviction = U64F64::saturating_from_num(0);
            }
            HotkeyLock::<T>::insert(netuid, new_hotkey, lock);
            writes = writes.saturating_add(2);
        }
        (reads, writes)
    }

    /// Moves lock from one hotkey to another and clears conviction
    ///
    /// The lock is rolled forward to the current block before switching the
    /// associated hotkey so that the lock stays mathematically correct and
    /// preserves current decayed locked mass. The conviction is
    /// reset to zero.
    pub fn do_move_lock(
        coldkey: &T::AccountId,
        destination_hotkey: &T::AccountId,
        netuid: NetUid,
    ) -> DispatchResult {
        let now = Self::get_current_block_as_u64();
        match Lock::<T>::get(coldkey, netuid) {
            Some(existing) => {
                let lock = Self::roll_forward_lock(existing, now);
                Lock::<T>::insert(
                    coldkey,
                    netuid,
                    LockState {
                        hotkey: destination_hotkey.clone(),
                        locked_mass: lock.locked_mass,
                        conviction: U64F64::saturating_from_num(0),
                        last_update: now,
                    },
                );

                // Update the total hotkey locks for destination hotkey
                Self::upsert_hotkey_lock(destination_hotkey, netuid, lock.locked_mass);

                // Reduce the total hotkey locks for the origin hotkey
                Self::reduce_hotkey_lock(&lock.hotkey, netuid, lock.locked_mass, lock.conviction);

                Self::deposit_event(Event::LockMoved {
                    coldkey: coldkey.clone(),
                    origin_hotkey: lock.hotkey,
                    destination_hotkey: destination_hotkey.clone(),
                    netuid,
                });
                Ok(())
            }
            None => Err(Error::<T>::NoExistingLock.into()),
        }
    }
}
