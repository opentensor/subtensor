use crate::epoch::math::clamp_u64f64;

use super::*;
use crate::epoch::math::*;
use safe_math::*;
use substrate_fixed::types::{U64F64, U96F32};

impl<T: Config> Pallet<T> {
    /// Retrieves the global global weight as a normalized value between 0 and 1.
    ///
    /// This function performs the following steps:
    /// 1. Fetches the global weight from storage using the TaoWeight storage item.
    /// 2. Converts the retrieved u64 value to a fixed-point number (U96F32).
    /// 3. Normalizes the weight by dividing it by the maximum possible u64 value.
    /// 4. Returns the normalized weight as an U96F32 fixed-point number.
    ///
    /// The normalization ensures that the returned value is always between 0 and 1,
    /// regardless of the actual stored weight value.
    ///
    /// # Returns
    /// * `U96F32` - The normalized global global weight as a fixed-point number between 0 and 1.
    ///
    /// # Note
    /// This function uses saturating division to prevent potential overflow errors.
    pub fn get_tao_weight() -> U96F32 {
        // Step 1: Fetch the global weight from storage
        let stored_weight = TaoWeight::<T>::get();

        // Step 2: Convert the u64 weight to U96F32
        let weight_fixed = U96F32::saturating_from_num(stored_weight);

        // Step 3: Normalize the weight by dividing by u64::MAX
        // This ensures the result is always between 0 and 1
        weight_fixed.safe_div(U96F32::saturating_from_num(u64::MAX))
    }

    /// As above but for internal math
    fn get_tao_weight_float() -> U64F64 {
        // Step 1: Fetch the global weight from storage
        let stored_weight = TaoWeight::<T>::get();

        // Step 2: Convert the u64 weight to U64F64
        let weight_fixed = U64F64::saturating_from_num(stored_weight);

        // Step 3: Normalize the weight by dividing by u64::MAX
        // This ensures the result is always between 0 and 1
        weight_fixed.safe_div(U64F64::saturating_from_num(u64::MAX))
    }

    /// Sets the global global weight in storage.
    ///
    /// This function performs the following steps:
    /// 1. Takes the provided weight value as a u64.
    /// 2. Updates the TaoWeight storage item with the new value.
    ///
    /// # Arguments
    /// * `weight` - The new global weight value to be set, as a u64.
    ///
    /// # Effects
    /// This function modifies the following storage item:
    /// - `TaoWeight`: Updates it with the new weight value.
    ///
    /// # Note
    /// The weight is stored as a raw u64 value. To get the normalized weight between 0 and 1,
    /// use the `get_tao_weight()` function.
    pub fn set_tao_weight(weight: u64) {
        // Update the TaoWeight storage with the new weight value
        TaoWeight::<T>::set(weight);
    }

    /// Sets the global tao weight in storage from a floating-point representation.
    ///
    /// This function performs the following steps:
    /// 1. Takes the provided weight value as a U64F64.
    /// 2. Clamps the weight to ensure it is within the defined minimum and maximum bounds
    /// 3. Converts the U64F64 weight to a u64 by multiplying with u64::MAX.
    /// 4. Updates the TaoWeight storage item with the new value.
    ///
    /// # Arguments
    /// * `weight` - The new global weight value to be set, as a U64F64.
    ///
    /// # Effects
    /// This function modifies the following storage item:
    /// - `TaoWeight`: Updates it with the new weight value.
    ///
    /// # Note
    /// The weight is stored as a raw u64 value. To get the normalized weight between 0 and 1,
    /// use the `get_tao_weight()` function.
    pub fn set_tao_weight_from_float(weight: U64F64) {
        let clamped_weight =
            clamp_u64f64(weight, MinTaoWeight::<T>::get(), MaxTaoWeight::<T>::get());

        TaoWeight::<T>::set(fixed128_to_u64(clamped_weight));
    }

    /// Compares TAO injection to TAO reserves.
    ///
    /// - When TAO injection is greater than the change in TAO reserves
    ///   since the last block, the tao_weight is decreased proportionally
    ///   to the change.
    ///
    /// - When TAO injection is less than the change in TAO reserves
    ///   since the last update, the tao_weight is increased proportionally
    ///   to the change.
    ///
    /// – 1 tick in TAO weight equates to 1% per day
    ///
    /// – Simple 1 tick adjustment per execution, clamped at 9% and 18%.
    pub fn update_tao_weight(block_emission: U64F64) {
        // 1) Sum raw TAO across all subnets (excluding ROOT).
        let current_reserves: U64F64 = SubnetTAO::<T>::iter()
            .filter(|(netuid, _)| !netuid.is_root())
            .map(|(netuid, tao)| {
                U64F64::saturating_from_num(tao).saturating_add(U64F64::saturating_from_num(
                    SubnetTaoProvided::<T>::get(&netuid),
                ))
            })
            .sum();

        // 2) Read last‐block total and compute expected next total.
        let prev_reserves = TaoReservesAtLastBlock::<T>::get();
        let prev_reserves: U64F64 = U64F64::saturating_from_num(prev_reserves);
        let expected_reserves = prev_reserves.saturating_add(block_emission);

        // Tick size: 1/3,000,000 = 0.00000033333... (0.000033333...%)
        let tick =
            U64F64::saturating_from_num(1u64).safe_div(U64F64::saturating_from_num(3_000_000u64));

        let mut next_tao_weight_float = Self::get_tao_weight_float();

        let tao_scale = U64F64::saturating_from_num(1_000_000_000u64);

        // 3)  Compare difference between current and expected reserves and adjust weight accordingly.
        // 3a) We’re under‐filled (need to sell less / weight ↓).
        if current_reserves < expected_reserves {
            let diff_raw = expected_reserves.saturating_sub(current_reserves);
            // Convert to actual TAO units by dividing by 1e9
            let diff_tao = diff_raw.safe_div(tao_scale);
            // Apply 1 tick per TAO difference
            let adjustment = diff_tao.saturating_mul(tick);
            next_tao_weight_float = next_tao_weight_float.saturating_sub(adjustment);

            log::debug!(
                "TAO reserves under-filled: current_reserves: {}, expected_reserves: {}, diff_tao: {}, adjustment: {}, next_tao_weight_float: {}",
                current_reserves,
                expected_reserves,
                diff_tao,
                adjustment,
                next_tao_weight_float,
            );

            Self::set_tao_weight_from_float(next_tao_weight_float);
        // 3b) We’re over‐filled (need to sell more / weight ↑).
        } else if expected_reserves < current_reserves {
            let diff_raw = current_reserves.saturating_sub(expected_reserves);
            // Convert to actual TAO units by dividing by 1e9
            let diff_tao = diff_raw.safe_div(tao_scale);
            // Apply 1 tick per TAO difference
            let adjustment = diff_tao.saturating_mul(tick);
            next_tao_weight_float = next_tao_weight_float.saturating_add(adjustment);

            log::debug!(
                "TAO reserves over-filled: current_reserves: {}, expected_reserves: {}, diff_tao: {}, adjustment: {}, next_tao_weight_float: {}",
                current_reserves,
                expected_reserves,
                diff_tao,
                adjustment,
                next_tao_weight_float,
            );

            Self::set_tao_weight_from_float(next_tao_weight_float);
        }

        // 4) Update the reserves at last block.
        TaoReservesAtLastBlock::<T>::set(current_reserves.saturating_to_num::<u64>());
    }
}
