use alloc::vec::Vec;
use frame_support::weights::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::Zero;

use super::*;
use crate::root_registered::{EmaState, EmaStrategy};

impl<T: Config> Pallet<T> {
    /// Advances the EMA sampler by one tick. Updates one member's EMA
    /// when `block_number` is a multiple of `EmaSamplingInterval`,
    /// otherwise no-ops. Returns the actual consumed weight.
    pub fn tick_root_registered_ema(block_number: BlockNumberFor<T>) -> Weight {
        let db = T::DbWeight::get();

        let interval = T::EmaSamplingInterval::get();
        if interval.is_zero() || (block_number % interval) != BlockNumberFor::<T>::zero() {
            return Weight::zero();
        }

        // Bounded by root cap.
        let entries: Vec<(T::AccountId, EmaState)> = RootRegisteredEma::<T>::iter().collect();
        let total = entries.len() as u32;
        let mut weight = db.reads(u64::from(total));
        if total == 0 {
            return weight;
        }

        let cursor = EmaSampleCursor::<T>::get();
        weight.saturating_accrue(db.reads(1));

        let (coldkey, previous) = &entries[(cursor % total) as usize];

        let (next_ema, strategy_weight) = T::EmaStrategy::next(coldkey, *previous);
        weight.saturating_accrue(strategy_weight);

        let next = EmaState {
            ema: next_ema,
            samples: previous.samples.saturating_add(1),
        };
        RootRegisteredEma::<T>::insert(coldkey, next);
        EmaSampleCursor::<T>::put(cursor.wrapping_add(1));
        weight.saturating_add(db.writes(2))
    }

    /// Seeds a fresh EMA slot at zero. The zero value enforces a
    /// warmup window before the EMA carries meaningful weight.
    pub(crate) fn init_root_registered_ema(coldkey: &T::AccountId) {
        RootRegisteredEma::<T>::insert(coldkey, EmaState::default());
    }

    pub(crate) fn clear_root_registered_ema(coldkey: &T::AccountId) {
        RootRegisteredEma::<T>::remove(coldkey);
    }
}
