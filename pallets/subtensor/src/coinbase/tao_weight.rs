use substrate_fixed::types::I64F64;
use subtensor_runtime_common::NetUid;


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
        let weight_u64 = weight.saturating_mul(U96F32::saturating_from_num(u64::MAX)).saturating_to_num::<u64>();
        
        // Update the TaoWeight storage with the new weight value
        TaoWeight::<T>::set(weight_u64);
    }

    /// Simple one‐step proportional adjuster for tao_weight.
    /// – No smoothing, no integral memory, just immediate proportional reaction.
    /// – Allows bigger jumps, but in aggregate tracks block_emission over time.
    pub fn update_tao_weight_simple(block_emission: U96F32) {
        // 1) Sum raw TAO across all subnets (excluding ROOT).
        let current_total: U96F32 = Self::get_all_subnet_netuids()
            .into_iter()
            .filter(|&uid| uid != NetUid::ROOT)
            .map(|uid| U96F32::saturating_from_num(SubnetTAO::<T>::get(&uid)))
            .sum();
        // TODO: Check if the sum operation can overflow or if any subnet TAO retrieval can fail.

        // 2) Read last‐block total and compute “expected” next total.
        let prev_total: U96F32 = StoredTotal::<T>::get();
        let expected_total = prev_total.saturating_add(block_emission);
        // TODO: Ensure that the addition does not overflow and block_emission is valid.

        // 3) Compute difference and normalize it.
        //    A positive diff → we’re under‐filled (need to sell less / weight ↓).
        //    A negative diff → we’re over‐filled (need to sell more / weight ↑).
        let diff: U96F32 = expected_total.saturating_sub(current_total);
        // TODO: Verify that the subtraction does not result in an unexpected negative value.

        // 4) Single‐factor proportional gain: same tick = 1% per day per block.
        let tick = U96F32::saturating_from_num(0.01 / 7200.0);
        // TODO: Confirm that the tick calculation is accurate and does not lose precision.

        // 5) Apply update in one line: w_next = w * (1 + tick * (diff / current_total))
        //    We divide by current_total so that the same absolute RAO error
        //    yields a proportionally smaller change when pools are large.
        let w_prev: U96F32 = Self::get_tao_weight();
        let adjustment = tick
            .saturating_mul(diff)
            .saturating_div(current_total.max(U96F32::saturating_from_num(1)));
        // TODO: Check for division by zero and ensure adjustment is within expected bounds.
        
        let mut w_next = w_prev.saturating_mul(U96F32::one().saturating_add(adjustment));
        // TODO: Validate that w_next does not overflow or underflow.

        // 6) Clamp to your safety bounds [0.018, 0.18]
        let w_min = U96F32::saturating_from_num(0.018);
        let w_max = U96F32::saturating_from_num(0.18);
        if w_next < w_min {
            w_next = w_min;
        } else if w_next > w_max {
            w_next = w_max;
        }
        // TODO: Ensure clamping logic is correct and does not introduce errors.

        // 7) Write back new weight & persist current_total for next block.
        Self::set_tao_weight(w_next);
        StoredTotal::<T>::set(current_total);
        // TODO: Verify that storage updates are successful and atomic.
    }
}