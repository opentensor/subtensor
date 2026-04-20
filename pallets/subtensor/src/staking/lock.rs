use super::*;
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

    /// Rolls a LockState forward to `now` using exponential decay.
    ///
    /// X_new = decay * X_old
    /// Y_new = decay * (Y_old + dt * X_old)
    pub fn roll_forward_lock(lock: LockState<T::AccountId>, now: u64) -> LockState<T::AccountId> {
        if now <= lock.last_update {
            return lock;
        }
        let dt = now.saturating_sub(lock.last_update);
        let tau = TauBlocks::<T>::get();
        let decay = Self::exp_decay(dt, tau);

        let dt_fixed = U64F64::saturating_from_num(dt);
        let mass_fixed = U64F64::saturating_from_num(lock.locked_mass);
        let new_locked_mass = decay
            .saturating_mul(mass_fixed)
            .saturating_to_num::<u64>()
            .into();
        let new_conviction = decay.saturating_mul(
            lock.conviction
                .saturating_add(dt_fixed.saturating_mul(mass_fixed)),
        );

        LockState {
            hotkey: lock.hotkey,
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

    /// Returns the total conviction for a hotkey on a subnet,
    /// summed over all coldkeys that have locked to this hotkey.
    pub fn hotkey_conviction(hotkey: &T::AccountId, netuid: NetUid) -> U64F64 {
        let now = Self::get_current_block_as_u64();
        let mut total = U64F64::saturating_from_num(0);
        for (_coldkey, _subnet_id, lock) in Lock::<T>::iter() {
            if _subnet_id != netuid {
                continue;
            }
            if *hotkey == lock.hotkey {
                let rolled = Self::roll_forward_lock(lock, now);
                total = total.saturating_add(rolled.conviction);
            }
        }
        total
    }

    /// Finds the hotkey with the highest conviction on a given subnet.
    pub fn subnet_king(netuid: NetUid) -> Option<T::AccountId> {
        let now = Self::get_current_block_as_u64();
        let mut scores: sp_std::collections::btree_map::BTreeMap<Vec<u8>, (T::AccountId, U64F64)> =
            sp_std::collections::btree_map::BTreeMap::new();

        for (_coldkey, subnet_id, lock) in Lock::<T>::iter() {
            if subnet_id != netuid {
                continue;
            }
            let rolled = Self::roll_forward_lock(lock, now);
            let key = rolled.hotkey.encode();
            let entry = scores
                .entry(key)
                .or_insert_with(|| (rolled.hotkey.clone(), U64F64::saturating_from_num(0)));
            entry.1 = entry.1.saturating_add(rolled.conviction);
        }

        scores
            .into_values()
            .max_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(sp_std::cmp::Ordering::Equal)
            })
            .map(|(hotkey, _)| hotkey)
    }
}
