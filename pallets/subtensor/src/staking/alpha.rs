use super::*;
use crate::epoch::math::*;
use frame_support::IterableStorageDoubleMap;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

impl<T: Config> Pallet<T> {
    /// Retrieves the global global weight as a normalized value between 0 and 1.
    ///
    /// This function performs the following steps:
    /// 1. Fetches the global weight from storage using the GlobalWeight storage item.
    /// 2. Converts the retrieved u64 value to a fixed-point number (I96F32).
    /// 3. Normalizes the weight by dividing it by the maximum possible u64 value.
    /// 4. Returns the normalized weight as an I96F32 fixed-point number.
    ///
    /// The normalization ensures that the returned value is always between 0 and 1,
    /// regardless of the actual stored weight value.
    ///
    /// # Returns
    /// * `I96F32` - The normalized global global weight as a fixed-point number between 0 and 1.
    ///
    /// # Note
    /// This function uses saturating division to prevent potential overflow errors.
    pub fn get_root_weight(netuid: u16) -> I96F32 {
        // Step 1: Fetch the global weight from storage
        let stored_weight = GlobalWeight::<T>::get(netuid);

        // Step 2: Convert the u64 weight to I96F32
        let weight_fixed = I96F32::from_num(stored_weight);

        // Step 3: Normalize the weight by dividing by u64::MAX
        // This ensures the result is always between 0 and 1
        weight_fixed.saturating_div(I96F32::from_num(u64::MAX))
    }

    /// Sets the global global weight in storage.
    ///
    /// This function performs the following steps:
    /// 1. Takes the provided weight value as a u64.
    /// 2. Updates the GlobalWeight storage item with the new value.
    ///
    /// # Arguments
    /// * `weight` - The new global weight value to be set, as a u64.
    ///
    /// # Effects
    /// This function modifies the following storage item:
    /// - `GlobalWeight`: Updates it with the new weight value.
    ///
    /// # Note
    /// The weight is stored as a raw u64 value. To get the normalized weight between 0 and 1,
    /// use the `get_root_weight()` function.
    pub fn set_root_weight(weight: u64, netuid: u16) {
        // Update the GlobalWeight storage with the new weight value
        GlobalWeight::<T>::insert(netuid, weight);
    }

    /// Calculates the weighted combination of alpha and global tao for hotkeys on a subnet.
    ///
    /// This function performs the following steps:
    /// 1. Retrieves the subnet size (number of neurons).
    /// 2. Fetches all hotkeys (neuron keys) on the specified subnet.
    /// 3. Calculates the alpha stake vector:
    ///    a. Initializes a vector with zeros.
    ///    b. For each hotkey, retrieves its inherited stake and stores it in the vector.
    ///    c. Normalizes the alpha stake vector.
    /// 4. Calculates the global tao stake vector:
    ///    a. Initializes a vector with zeros.
    ///    b. For each hotkey, retrieves its global stake and stores it in the vector.
    ///    c. Normalizes the global tao stake vector.
    /// 5. Combines alpha and global tao stakes:
    ///    a. Retrieves the global global weight.
    ///    b. For each neuron, calculates a weighted average of its alpha and global tao stakes.
    ///    c. Normalizes the combined stake vector.
    /// 6. Converts the combined stake vector from 64-bit to 32-bit fixed-point representation.
    /// 7. Returns the final stake weights for each neuron on the subnet.
    ///
    /// # Arguments
    /// * `netuid` - Network unique identifier specifying the subnet context for stake weight calculation.
    ///
    /// # Returns
    /// * `(Vec<I32F32>, Vec<u64>, Vec<u64>)` - A tuple containing:
    ///   - A vector of stake weights for each hotkey (neuron) on the specified subnet, represented as 32-bit fixed-point numbers.
    ///   - A vector of raw alpha stakes for each hotkey (neuron) on the specified subnet.
    ///   - A vector of raw global tao stakes for each hotkey (neuron) on the specified subnet.
    pub fn get_stake_weights_for_network(netuid: u16) -> (Vec<I32F32>, Vec<u64>, Vec<u64>) {
        // Step 1: Get the subnet size (number of neurons).
        let n: u16 = Self::get_subnetwork_n(netuid);

        // Step 2: Retrieve all hotkeys (neuron keys) on this subnet.
        let hotkeys: Vec<(u16, T::AccountId)> = <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid).collect();

        // Step 3: Calculate the alpha stake vector.
        // Initialize a vector to store alpha stakes for each neuron.
        let mut alpha_stake: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        let mut raw_alpha_stake: Vec<u64> = vec![0; n as usize];
        for (uid_i, hotkey) in &hotkeys {
            let alpha: u64 = Self::get_inherited_for_hotkey_on_subnet(hotkey, netuid);
            alpha_stake[*uid_i as usize] = I64F64::from_num(alpha);
            raw_alpha_stake[*uid_i as usize] = alpha;
        }
        // Normalize the alpha stake vector.
        inplace_normalize_64(&mut alpha_stake);

        // Step 4: Calculate the global tao stake vector.
        // Initialize a vector to store global tao stakes for each neuron.
        let mut root_stake: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        let mut raw_root_stake: Vec<u64> = vec![0; n as usize];
        for (uid_i, hotkey) in &hotkeys {
            let global: u64 = Self::get_inherited_for_hotkey_on_subnet(hotkey, 0);
            root_stake[*uid_i as usize] = I64F64::from_num(global);
            raw_root_stake[*uid_i as usize] = global;
        }
        // Normalize the global tao stake vector.
        inplace_normalize_64(&mut root_stake);

        // Step 5: Combine alpha and root tao stakes.
        // Retrieve the global global weight.
        let root_weight: I64F64 = I64F64::from_num(Self::get_root_weight(netuid));
        // Calculate the weighted average of alpha and global tao stakes for each neuron.
        let mut stake_weights: Vec<I64F64> = alpha_stake
            .iter()
            .zip(root_stake.iter())
            .map(|(alpha, global)| {
                // Weighted average: (1 - root_weight) * alpha + root_weight * global
                (I64F64::from_num(1.0).saturating_sub(root_weight))
                    .saturating_mul(*alpha)
                    .saturating_add(root_weight.saturating_mul(*global))
            })
            .collect();
        // Normalize the combined stake weights.
        inplace_normalize_64(&mut stake_weights); // no need to normalize

        // Step 6: Convert the combined stake values from 64-bit to 32-bit fixed-point representation.
        let stake_weights_32 = vec_fixed64_to_fixed32(stake_weights);
        (stake_weights_32, raw_alpha_stake, raw_root_stake)
    }

    /// Calculates the total inherited stake (alpha) held by a hotkey on a network, considering child/parent relationships.
    ///
    /// This function performs the following steps:
    /// 1. Retrieves the initial alpha (stake) for the hotkey on the specified subnet.
    /// 2. Retrieves the list of children and parents for the hotkey on the subnet.
    /// 3. Calculates the alpha allocated to children:
    ///    a. For each child, computes the proportion of alpha to be allocated.
    ///    b. Accumulates the total alpha allocated to all children.
    /// 4. Calculates the alpha received from parents:
    ///    a. For each parent, retrieves the parent's stake on the subnet.
    ///    b. Computes the proportion of the parent's stake to be inherited.
    ///    c. Accumulates the total alpha inherited from all parents.
    /// 5. Computes the final inherited alpha by adjusting the initial alpha:
    ///    a. Subtracts the alpha allocated to children.
    ///    b. Adds the alpha inherited from parents.
    /// 6. Returns the final inherited alpha value.
    ///
    /// # Arguments
    /// * `hotkey` - AccountId of the hotkey whose total inherited stake is to be calculated.
    /// * `netuid` - Network unique identifier specifying the subnet context.
    ///
    /// # Returns
    /// * `u64` - The total inherited alpha for the hotkey on the subnet after considering the stakes
    ///           allocated to children and inherited from parents.
    ///
    /// # Note
    /// This function uses saturating arithmetic to prevent overflows.
    pub fn get_inherited_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {
        // Step 1: Retrieve the initial total stake (alpha) for the hotkey on the specified subnet.
        let initial_alpha: I96F32 =
            I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, netuid));

        // Initialize variables to track alpha allocated to children and inherited from parents.
        let mut alpha_to_children: I96F32 = I96F32::from_num(0);
        let mut alpha_from_parents: I96F32 = I96F32::from_num(0);

        // Step 2: Retrieve the lists of parents and children for the hotkey on the subnet.
        let parents: Vec<(u64, T::AccountId)> = Self::get_parents(hotkey, netuid);
        let children: Vec<(u64, T::AccountId)> = Self::get_children(hotkey, netuid);

        // Step 3: Calculate the total alpha allocated to children.
        for (proportion, _) in children {
            // Convert the proportion to a normalized value between 0 and 1.
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));

            // Calculate the amount of alpha to be allocated to this child.
            let alpha_proportion_to_child: I96F32 =
                I96F32::from_num(initial_alpha).saturating_mul(normalized_proportion);

            // Add this child's allocation to the total alpha allocated to children.
            alpha_to_children = alpha_to_children.saturating_add(alpha_proportion_to_child);
        }

        // Step 4: Calculate the total alpha inherited from parents.
        for (proportion, parent) in parents {
            // Retrieve the parent's total stake on this subnet.
            let parent_alpha: I96F32 =
                I96F32::from_num(Self::get_stake_for_hotkey_on_subnet(&parent, netuid));

            // Convert the proportion to a normalized value between 0 and 1.
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));

            // Calculate the amount of alpha to be inherited from this parent.
            let alpha_proportion_from_parent: I96F32 =
                I96F32::from_num(parent_alpha).saturating_mul(normalized_proportion);

            // Add this parent's contribution to the total alpha inherited from parents.
            alpha_from_parents = alpha_from_parents.saturating_add(alpha_proportion_from_parent);
        }

        // Step 5: Calculate the final inherited alpha for the hotkey.
        let finalized_alpha: I96F32 = initial_alpha
            .saturating_sub(alpha_to_children) // Subtract alpha allocated to children
            .saturating_add(alpha_from_parents); // Add alpha inherited from parents

        // Step 6: Return the final inherited alpha value.
        finalized_alpha.to_num::<u64>()
    }


    /// Checks if a specific hotkey-coldkey pair has enough stake on a subnet to fulfill a given decrement.
    ///
    /// This function performs the following steps:
    /// 1. Retrieves the current stake for the hotkey-coldkey pair on the specified subnet.
    /// 2. Compares this stake with the requested decrement amount.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `coldkey` - The account ID of the coldkey.
    /// * `netuid` - The unique identifier of the subnet.
    /// * `decrement` - The amount of stake to be potentially decremented.
    ///
    /// # Returns
    /// * `bool` - True if the account has enough stake to fulfill the decrement, false otherwise.
    ///
    /// # Note
    /// This function only checks the stake for the specific hotkey-coldkey pair, not the total stake of the hotkey or coldkey individually.
    pub fn has_enough_stake_on_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        decrement: u64,
    ) -> bool {
        // Retrieve the current stake for this hotkey-coldkey pair on the subnet
        let current_stake = Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid);

        // Compare the current stake with the requested decrement
        // Return true if the current stake is greater than or equal to the decrement
        current_stake >= decrement
    }

    /// Retrieves the alpha (stake) value for a given hotkey and coldkey pair on a specific subnet.
    ///
    /// This function performs the following steps:
    /// 1. Takes the hotkey, coldkey, and subnet ID as input parameters.
    /// 2. Accesses the Alpha storage map to retrieve the stake value.
    /// 3. Returns the retrieved stake value as a u64.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey (neuron).
    /// * `coldkey` - The account ID of the coldkey (owner).
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The alpha (stake) value for the specified hotkey-coldkey pair on the given subnet.
    ///
    /// # Note
    /// This function retrieves the stake specific to the hotkey-coldkey pair, not the total stake of the hotkey or coldkey individually.
    pub fn get_stake_for_hotkey_and_coldkey_on_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
    ) -> u64 {
        // Step 1: Get the total number of shares that this hotkey has on this subnet.
        let total_hotkey_shares: u64 = TotalHotkeyShares::<T>::get(hotkey, netuid);

        // Step 2: Get the total alpha allocated to those shares on this subnet.
        let total_hotkey_alpha: u64 = TotalHotkeyAlpha::<T>::get(hotkey, netuid);

        // Step 3: Get the number of shares this coldkey has with this hotkey.
        let coldkey_shares: u64 = Alpha::<T>::get((hotkey, coldkey, netuid));

        // Step 4: If there are no shares or alpha on this hotkey, return 0;
        if total_hotkey_shares == 0 || total_hotkey_alpha == 0 || coldkey_shares == 0 {
            return 0;
        }

        // Step 5: Compute the alphas per share.
        let alphas_per_share: I96F32 = I96F32::from_num( total_hotkey_alpha ).checked_div( I96F32::from_num( total_hotkey_shares ) ).unwrap_or( I96F32::from_num( 0.0) );

        // Step 6: Compute implied alphas from coldkey shares
        let coldkey_alphas: I96F32 = alphas_per_share * I96F32::from_num( coldkey_shares );

        // Return 
        coldkey_alphas.to_num::<u64>()
    }

    /// Retrieves the total stake (alpha) for a given hotkey on a specific subnet.
    ///
    /// This function performs the following step:
    /// 1. Retrieves and returns the total alpha value associated with the hotkey on the specified subnet.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The total alpha value for the hotkey on the specified subnet.
    ///
    /// # Note
    /// This function returns the cumulative stake across all coldkeys associated with this hotkey on the subnet.
    pub fn get_stake_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {
        // Retrieve and return the total alpha this hotkey owns on this subnet.
        // This value represents the sum of stakes from all coldkeys associated with this hotkey.
        TotalHotkeyAlpha::<T>::get(hotkey, netuid)
    }


    /// Increase hotkey stake on a subnet.
    ///
    /// The function updates share totals given current prices.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier of the subnet.
    /// * `amount` - The amount of alpha to be added.
    ///
    pub fn increase_stake_for_hotkey_on_subnet(  
        hotkey: &T::AccountId,
        netuid: u16,
        amount: u64 
    ) {
        // Add to the total for this hotkey on this subnet.
        TotalHotkeyAlpha::<T>::mutate( hotkey, netuid , |total| {
            *total = total.saturating_add(amount);
        });
    }

    /// Decrease hotkey stake on a subnet.
    ///
    /// The function updates share totals given current prices.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier of the subnet.
    /// * `amount` - The amount of alpha to be added.
    ///
    pub fn decrease_stake_for_hotkey_on_subnet(  
        hotkey: &T::AccountId,
        netuid: u16,
        amount: u64 
    ) {
        // Add to the total for this hotkey on this subnet.
        TotalHotkeyAlpha::<T>::mutate( hotkey, netuid , |total| {
            *total = total.saturating_add(amount);
        });
    }

    /// Buys shares in the hotkey on a given subnet
    ///
    /// The function updates share totals given current prices.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `coldkey` - The account ID of the coldkey (owner).
    /// * `netuid` - The unique identifier of the subnet.
    /// * `amount` - The amount of alpha to be added.
    ///
    pub fn increase_stake_for_hotkey_and_coldkey_on_subnet(  
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        amount: u64 
    ) {

        // Step 1: Get the total number of shares associated with this hotkey on this subnet.
        let total_hotkey_shares: u64 = TotalHotkeyShares::<T>::get(hotkey, netuid);

        // Step 2: Increment the total amount of alpha on this subnet.
        TotalHotkeyAlpha::<T>::mutate( hotkey, netuid, |total| {
            *total = total.saturating_add(amount);
        });

        // Step 3: Get the new total alpha for this hotkey on this subnet.
        let new_total_hotkey_alpha: u64 = TotalHotkeyAlpha::<T>::get(hotkey, netuid);

        // Step 4: Compute shares bought.
        let alpha_shares_bought: u64 = if total_hotkey_shares == 0 {
            // Step 5a: No shares, we use the initial value as the shares to bootstrap.
            amount
        } else {
            // Step 5b: Get the price per share: total_alpha / total_shares
            let price_per_share: I96F32 = I96F32::from_num(new_total_hotkey_alpha)
                .checked_div(I96F32::from_num(total_hotkey_shares))
                .unwrap_or(I96F32::from_num(0.0));

            // Step 5c: Attain the number of shares: alpha / price_per_alpha
            (I96F32::from_num(amount)
                .checked_div(price_per_share)
                .unwrap_or(I96F32::from_num(0.0)))
            .to_num::<u64>()
        };

        // Step 6: Increment the total number of shares associated with this hotkey on this subnet.
        TotalHotkeyShares::<T>::mutate( hotkey, netuid , |total| {
            *total = total.saturating_add(alpha_shares_bought);
        });

        // Step 7: Actually set the shares here in the map.
        Alpha::<T>::mutate((hotkey, coldkey, netuid), |total| {
            *total = total.saturating_add(alpha_shares_bought);
        });
    }

    /// Sell shares in the hotkey on a given subnet
    ///
    /// The function updates share totals given current prices.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `coldkey` - The account ID of the coldkey (owner).
    /// * `netuid` - The unique identifier of the subnet.
    /// * `amount` - The amount of alpha to be added.
    ///
    pub fn decrease_stake_for_hotkey_and_coldkey_on_subnet(  
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        amount: u64 
    ) {

        // Step 1: Get the total shares on this hotkey.
        let total_hotkey_shares: u64 = TotalHotkeyShares::<T>::get(hotkey, netuid);

        // Step 2: Get the total alpha on this hotkey and subnet.
        let total_hotkey_alpha: u64 = TotalHotkeyAlpha::<T>::get(hotkey, netuid);

        // Step 3: Return if values are zero.
        if total_hotkey_shares == 0 || total_hotkey_alpha == 0 || amount == 0 {
            return;
        }

        // Step 4: Get the price per share: total_alpha / total_shares
        let price_per_share: I96F32 = I96F32::from_num( total_hotkey_alpha ).checked_div( I96F32::from_num( total_hotkey_shares ) ).unwrap_or( I96F32::from_num( 0.0) );
        
        // Step 5: Attain the number of shares sold: alpha_unstaked / price_per_alpha
        let alpha_shares_sold: u64 = (I96F32::from_num( amount ).checked_div( I96F32::from_num( price_per_share )).unwrap_or( I96F32::from_num( 0.0 ) )).to_num::<u64>();

        // Step 6: Ensure we are not selling more shares than we have.
        let current_alpha_shares: u64 = Alpha::<T>::get( (hotkey, coldkey, netuid) );
        if alpha_shares_sold > current_alpha_shares {
            return;
        }

        // Step 7: Decrement the amount of alpha associated with the hotkey.
        TotalHotkeyAlpha::<T>::mutate_exists( hotkey, netuid, |maybe_total| {
            if let Some(total) = maybe_total {
                let new_total = total.saturating_sub( amount );
                if new_total == 0 {
                    *maybe_total = None;
                } else {
                    *total = new_total;
                }
            }
        });

        // Step 9: Decrement the number of shares here.
        TotalHotkeyShares::<T>::mutate_exists( hotkey, netuid , |maybe_total| {
            if let Some(total) = maybe_total {
                let new_total = total.saturating_sub(alpha_shares_sold);
                if new_total == 0 {
                    *maybe_total = None;
                } else {
                    *total = new_total;
                }
            }
        });

        // Step 10: Actually reduce the number of shares here.
        Alpha::<T>::mutate_exists((hotkey, coldkey, netuid), |maybe_total| {
            if let Some(total) = maybe_total {
                let new_total = total.saturating_sub(alpha_shares_sold);
                if new_total == 0 {
                    *maybe_total = None;
                } else {
                    *total = new_total;
                }
            }
        });

    }
}
