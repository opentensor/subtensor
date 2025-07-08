use super::*;
use safe_math::*;
use substrate_fixed::types::U96F32;

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
    /// 1. Takes the provided weight value as a U96F32.
    /// 2. Converts the U96F32 weight to a u64 by multiplying with u64::MAX.
    /// 3. Updates the TaoWeight storage item with the new value.
    ///
    /// # Arguments
    /// * `weight` - The new global weight value to be set, as a U96F32.
    ///
    /// # Effects
    /// This function modifies the following storage item:
    /// - `TaoWeight`: Updates it with the new weight value.
    ///
    /// # Note
    /// The weight is stored as a raw u64 value. To get the normalized weight between 0 and 1,
    /// use the `get_tao_weight()` function.
    pub fn set_tao_weight_from_float(weight: U96F32) {
        // Convert the U96F32 weight to a u64 by multiplying with u64::MAX
        let weight_u64 = weight
            .saturating_mul(U96F32::saturating_from_num(u64::MAX))
            .saturating_to_num::<u64>();

        // Update the TaoWeight storage with the new weight value
        TaoWeight::<T>::set(weight_u64);
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
    pub fn update_tao_weight(block_emission: U96F32) {
        // 1) Sum raw TAO across all subnets (excluding ROOT).
        let current_total: U96F32 = Self::get_all_subnet_netuids()
            .into_iter()
            .filter(|&netuid| netuid.is_root() == false)
            .map(|netuid| U96F32::saturating_from_num(SubnetTAO::<T>::get(&netuid)))
            .sum();

        // 2) Read last‐block total and compute expected next total.
        let prev_reserves = TaoReservesAtLastBlock::<T>::get();
        let prev_reserves: U96F32 = U96F32::saturating_from_num(prev_reserves);
        let expected_total = prev_reserves.saturating_add(block_emission);

        // 3) Compute difference
        //    A positive diff → we’re over‐filled (need to sell less / weight ↓).
        //    A negative diff → we’re under‐filled (need to sell more / weight ↑).
        let diff: U96F32 = current_total.saturating_sub(expected_total);

        // 4) Single‐factor proportional gain: tick = 1% per day
        let tick = U96F32::saturating_from_num(0.01 / 7200.0);

        // 5) Calculate next_weight: n_w = w * (1 + tick * (diff / current_total))
        //    We divide by current_total so that the same absolute RAO error
        //    yields a proportionally smaller change when pools are large.
        let current_weight: U96F32 = Self::get_tao_weight();
        let adjustment = tick
            .saturating_mul(diff)
            .saturating_div(current_total.max(U96F32::saturating_from_num(1)));

        let mut next_weight = current_weight
            .saturating_mul(U96F32::saturating_from_num(1).saturating_add(adjustment));

        // 6) Clamp to safety bounds [0.09, 0.18]
        let min_weight = U96F32::saturating_from_num(0.09);
        let max_weight = U96F32::saturating_from_num(0.18);
        if next_weight < min_weight {
            next_weight = min_weight;
        } else if next_weight > max_weight {
            next_weight = max_weight;
        }

        // 7) Write back new weight & persist current_total for next cycle.
        Self::set_tao_weight_from_float(next_weight);
        TaoReservesAtLastBlock::<T>::set(current_total.saturating_to_num::<u64>());
    }
}
