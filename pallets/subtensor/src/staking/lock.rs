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
        Lock::<T>::remove(coldkey, netuid);
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

    /// Transfers the lock from one coldkey to another for all subnets. This is used when a
    /// user swaps their coldkey and we want to preserve their locks.
    /// The hotkey and netuid remain the same, only the coldkey changes.
    ///
    /// If the new coldkey already has a lock for the same subnet, the locks are merged by summing
    /// the locked_mass and conviction after rolling forward both locks to now.
    pub fn transfer_lock_coldkey(_old_coldkey: &T::AccountId, _new_coldkey: &T::AccountId) {
        // let now = Self::get_current_block_as_u64();
        // let mut locks_to_transfer: Vec<(NetUid, LockState<T::AccountId>)> = Vec::new();

        // // Gather locks from old coldkey
        // for (coldkey, netuid, lock) in Lock::<T>::iter() {
        //     if coldkey == *old_coldkey {
        //         locks_to_transfer.push((netuid, lock));
        //     }
        // }

        // // Transfer each lock to new coldkey
        // for (netuid, old_lock) in locks_to_transfer {
        //     let rolled_old_lock = Self::roll_forward_lock(old_lock, now);
        //     match Lock::<T>::get(new_coldkey, netuid) {
        //         None => {
        //             // No existing lock for new coldkey, simply transfer
        //             Lock::<T>::insert(
        //                 new_coldkey,
        //                 netuid,
        //                 LockState {
        //                     hotkey: rolled_old_lock.hotkey.clone(),
        //                     locked_mass: rolled_old_lock.locked_mass,
        //                     conviction: rolled_old_lock.conviction,
        //                     last_update: now,
        //                 },
        //             );
        //         }
        //         Some(existing) => {
        //             // Existing lock for new coldkey, merge them
        //             let rolled_existing = Self::roll_forward_lock(existing, now);
        //             ensure!(
        //                 rolled_old_lock.hotkey == rolled_existing.hotkey,
        //                 Error::<T>::LockHotkeyMismatch
        //             );
        //             let new_locked_mass =
        //                 rolled_old_lock.locked_mass.saturating_add(rolled_existing.locked_mass);
        //             let new_conviction =
        //                 rolled_old_lock.conviction.saturating_add(rolled_existing.conviction);
        //             Lock::<T>::insert(
        //                 new_coldkey,
        //                 netuid,
        //                 LockState {
        //                     hotkey: rolled_old_lock.hotkey.clone(),
        //                     locked_mass: new_locked_mass,
        //                     conviction: new_conviction,
        //                     last_update: now,
        //                 },
        //             );

        //             // Remove the old lock since it's now merged
        //             Lock::<T>::remove(old_coldkey, netuid);
        //         }
        //     }
        // }
    }
}
