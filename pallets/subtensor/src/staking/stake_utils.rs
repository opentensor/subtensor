use super::*;
use safe_math::*;
use share_pool::{SharePool, SharePoolDataOperations};
use sp_std::ops::Neg;
use substrate_fixed::types::{I64F64, I96F32, U64F64, U96F32};
use subtensor_runtime_common::{AlphaCurrency, Currency, NetUid, TaoCurrency};
use subtensor_swap_interface::{Order, SwapHandler, SwapResult};

impl<T: Config> Pallet<T> {
    /// Retrieves the total alpha issuance for a given subnet.
    ///
    /// This function calculates the total alpha issuance by summing the alpha
    /// values from `SubnetAlphaIn` and `SubnetAlphaOut` for the specified subnet.
    ///
    /// # Arguments
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The total alpha issuance for the specified subnet.
    pub fn get_alpha_issuance(netuid: NetUid) -> AlphaCurrency {
        SubnetAlphaIn::<T>::get(netuid)
            .saturating_add(SubnetAlphaInProvided::<T>::get(netuid))
            .saturating_add(SubnetAlphaOut::<T>::get(netuid))
    }

    pub fn get_protocol_tao(netuid: NetUid) -> TaoCurrency {
        T::SwapInterface::get_protocol_tao(netuid)
    }

    pub fn get_moving_alpha_price(netuid: NetUid) -> U96F32 {
        let one = U96F32::saturating_from_num(1.0);
        if netuid.is_root() {
            // Root.
            one
        } else if SubnetMechanism::<T>::get(netuid) == 0 {
            // Stable
            one
        } else {
            U96F32::saturating_from_num(SubnetMovingPrice::<T>::get(netuid))
        }
    }

    pub fn update_moving_price(netuid: NetUid) {
        let blocks_since_start_call = U96F32::saturating_from_num({
            // We expect FirstEmissionBlockNumber to be set earlier, and we take the block when
            // `start_call` was called (first block before FirstEmissionBlockNumber).
            let start_call_block = FirstEmissionBlockNumber::<T>::get(netuid)
                .unwrap_or_default()
                .saturating_sub(1);

            Self::get_current_block_as_u64().saturating_sub(start_call_block)
        });

        // Use halving time hyperparameter. The meaning of this parameter can be best explained under
        // the assumption of a constant price and SubnetMovingAlpha == 0.5: It is how many blocks it
        // will take in order for the distance between current EMA of price and current price to shorten
        // by half.
        let halving_time = EMAPriceHalvingBlocks::<T>::get(netuid);
        let current_ma_unsigned = U96F32::saturating_from_num(SubnetMovingAlpha::<T>::get());
        let alpha: U96F32 = current_ma_unsigned.saturating_mul(blocks_since_start_call.safe_div(
            blocks_since_start_call.saturating_add(U96F32::saturating_from_num(halving_time)),
        ));
        // Because alpha = b / (b + h), where b and h > 0, alpha < 1, so 1 - alpha > 0.
        // We can use unsigned type here: U96F32
        let one_minus_alpha: U96F32 = U96F32::saturating_from_num(1.0).saturating_sub(alpha);
        let current_price: U96F32 = alpha.saturating_mul(
            T::SwapInterface::current_alpha_price(netuid.into())
                .min(U96F32::saturating_from_num(1.0)),
        );
        let current_moving: U96F32 =
            one_minus_alpha.saturating_mul(Self::get_moving_alpha_price(netuid));
        // Convert batch to signed I96F32 to avoid migration of SubnetMovingPrice for now``
        let new_moving: I96F32 =
            I96F32::saturating_from_num(current_price.saturating_add(current_moving));
        SubnetMovingPrice::<T>::insert(netuid, new_moving);
    }

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
    pub fn get_ck_burn() -> U96F32 {
        let stored_weight = CKBurn::<T>::get();
        let weight_fixed = U96F32::saturating_from_num(stored_weight);
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
    // Set the amount burned on non owned CK
    pub fn set_ck_burn(weight: u64) {
        // Update the ck burn value.
        CKBurn::<T>::set(weight);
    }

    /// Calculates the weighted combination of alpha and global tao for a single hotkey onet a subnet.
    ///
    pub fn get_stake_weights_for_hotkey_on_subnet(
        hotkey: &T::AccountId,
        netuid: NetUid,
    ) -> (I64F64, I64F64, I64F64) {
        // Retrieve the global tao weight.
        let tao_weight = I64F64::saturating_from_num(Self::get_tao_weight());
        log::debug!("tao_weight: {tao_weight:?}");

        // Step 1: Get stake of hotkey (neuron)
        let alpha_stake =
            I64F64::saturating_from_num(Self::get_inherited_for_hotkey_on_subnet(hotkey, netuid));
        log::debug!("alpha_stake: {alpha_stake:?}");

        // Step 2: Get the global tao stake for the hotkey
        let tao_stake = I64F64::saturating_from_num(Self::get_tao_inherited_for_hotkey_on_subnet(
            hotkey, netuid,
        ));
        log::debug!("tao_stake: {tao_stake:?}");

        // Step 3: Combine alpha and tao stakes
        let total_stake = alpha_stake.saturating_add(tao_stake.saturating_mul(tao_weight));
        log::debug!("total_stake: {total_stake:?}");

        (total_stake, alpha_stake, tao_stake)
    }

    /// Calculates the weighted combination of alpha and global tao for hotkeys on a subnet.
    ///
    pub fn get_stake_weights_for_network(
        netuid: NetUid,
    ) -> (Vec<I64F64>, Vec<I64F64>, Vec<I64F64>) {
        // Retrieve the global tao weight.
        let tao_weight: I64F64 = I64F64::saturating_from_num(Self::get_tao_weight());
        log::debug!("tao_weight: {tao_weight:?}");

        // Step 1: Get subnetwork size
        let n: u16 = Self::get_subnetwork_n(netuid);

        // Step 2: Get stake of all hotkeys (neurons) ordered by uid
        let alpha_stake: Vec<I64F64> = (0..n)
            .map(|uid| {
                if Keys::<T>::contains_key(netuid, uid) {
                    let hotkey: T::AccountId = Keys::<T>::get(netuid, uid);
                    I64F64::saturating_from_num(Self::get_inherited_for_hotkey_on_subnet(
                        &hotkey, netuid,
                    ))
                } else {
                    I64F64::saturating_from_num(0)
                }
            })
            .collect();
        log::debug!("alpha_stake: {alpha_stake:?}");

        // Step 3: Calculate the global tao stake vector.
        // Initialize a vector to store global tao stakes for each neuron.
        let tao_stake: Vec<I64F64> = (0..n)
            .map(|uid| {
                if Keys::<T>::contains_key(netuid, uid) {
                    let hotkey: T::AccountId = Keys::<T>::get(netuid, uid);
                    I64F64::saturating_from_num(Self::get_tao_inherited_for_hotkey_on_subnet(
                        &hotkey, netuid,
                    ))
                } else {
                    I64F64::saturating_from_num(0)
                }
            })
            .collect();
        log::trace!("tao_stake: {tao_stake:?}");

        // Step 4: Combine alpha and root tao stakes.
        // Calculate the weighted average of alpha and global tao stakes for each neuron.
        let total_stake: Vec<I64F64> = alpha_stake
            .iter()
            .zip(tao_stake.iter())
            .map(|(alpha_i, tao_i)| alpha_i.saturating_add(tao_i.saturating_mul(tao_weight)))
            .collect();
        log::trace!("total_stake: {total_stake:?}");

        (total_stake, alpha_stake, tao_stake)
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
    /// * `u64`: The total inherited alpha for the hotkey on the subnet after considering the
    ///   stakes allocated to children and inherited from parents.
    ///
    /// # Note
    /// This function uses saturating arithmetic to prevent overflows.
    pub fn get_tao_inherited_for_hotkey_on_subnet(
        hotkey: &T::AccountId,
        netuid: NetUid,
    ) -> TaoCurrency {
        let initial_tao: U96F32 =
            U96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, NetUid::ROOT));

        // Initialize variables to track alpha allocated to children and inherited from parents.
        let mut tao_to_children: U96F32 = U96F32::saturating_from_num(0);
        let mut tao_from_parents: U96F32 = U96F32::saturating_from_num(0);

        // Step 2: Retrieve the lists of parents and children for the hotkey on the subnet.
        let parents: Vec<(u64, T::AccountId)> = Self::get_parents(hotkey, netuid);
        let children: Vec<(u64, T::AccountId)> = Self::get_children(hotkey, netuid);
        log::trace!("Parents for hotkey {hotkey:?} on subnet {netuid}: {parents:?}");
        log::trace!("Children for hotkey {hotkey:?} on subnet {netuid}: {children:?}");

        // Step 3: Calculate the total tao allocated to children.
        for (proportion, _) in children {
            // Convert the proportion to a normalized value between 0 and 1.
            let normalized_proportion: U96F32 = U96F32::saturating_from_num(proportion)
                .safe_div(U96F32::saturating_from_num(u64::MAX));
            log::trace!("Normalized proportion for child: {normalized_proportion:?}");

            // Calculate the amount of tao to be allocated to this child.
            let tao_proportion_to_child: U96F32 =
                U96F32::saturating_from_num(initial_tao).saturating_mul(normalized_proportion);
            log::trace!("Tao proportion to child: {tao_proportion_to_child:?}");

            // Add this child's allocation to the total tao allocated to children.
            tao_to_children = tao_to_children.saturating_add(tao_proportion_to_child);
        }
        log::trace!("Total tao allocated to children: {tao_to_children:?}");

        // Step 4: Calculate the total tao inherited from parents.
        for (proportion, parent) in parents {
            // Retrieve the parent's total stake on this subnet.
            let parent_tao = U96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(
                &parent,
                NetUid::ROOT,
            ));
            log::trace!("Parent tao for parent {parent:?} on subnet {netuid}: {parent_tao:?}");

            // Convert the proportion to a normalized value between 0 and 1.
            let normalized_proportion = U96F32::saturating_from_num(proportion)
                .safe_div(U96F32::saturating_from_num(u64::MAX));
            log::trace!("Normalized proportion from parent: {normalized_proportion:?}");

            // Calculate the amount of tao to be inherited from this parent.
            let tao_proportion_from_parent: U96F32 =
                U96F32::saturating_from_num(parent_tao).saturating_mul(normalized_proportion);
            log::trace!("Tao proportion from parent: {tao_proportion_from_parent:?}");

            // Add this parent's contribution to the total tao inherited from parents.
            tao_from_parents = tao_from_parents.saturating_add(tao_proportion_from_parent);
        }
        log::trace!("Total tao inherited from parents: {tao_from_parents:?}");

        // Step 5: Calculate the final inherited tao for the hotkey.
        let finalized_tao: U96F32 = initial_tao
            .saturating_sub(tao_to_children) // Subtract tao allocated to children
            .saturating_add(tao_from_parents); // Add tao inherited from parents
        log::trace!("Finalized tao for hotkey {hotkey:?} on subnet {netuid}: {finalized_tao:?}");

        // Step 6: Return the final inherited tao value.
        finalized_tao.saturating_to_num::<u64>().into()
    }

    pub fn get_inherited_for_hotkey_on_subnet(
        hotkey: &T::AccountId,
        netuid: NetUid,
    ) -> AlphaCurrency {
        // Step 1: Retrieve the initial total stake (alpha) for the hotkey on the specified subnet.
        let initial_alpha: U96F32 =
            U96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(hotkey, netuid));
        log::debug!("Initial alpha for hotkey {hotkey:?} on subnet {netuid}: {initial_alpha:?}");
        if netuid.is_root() {
            return initial_alpha.saturating_to_num::<u64>().into();
        }

        // Initialize variables to track alpha allocated to children and inherited from parents.
        let mut alpha_to_children: U96F32 = U96F32::saturating_from_num(0);
        let mut alpha_from_parents: U96F32 = U96F32::saturating_from_num(0);

        // Step 2: Retrieve the lists of parents and children for the hotkey on the subnet.
        let parents: Vec<(u64, T::AccountId)> = Self::get_parents(hotkey, netuid);
        let children: Vec<(u64, T::AccountId)> = Self::get_children(hotkey, netuid);
        log::debug!("Parents for hotkey {hotkey:?} on subnet {netuid}: {parents:?}");
        log::debug!("Children for hotkey {hotkey:?} on subnet {netuid}: {children:?}");

        // Step 3: Calculate the total alpha allocated to children.
        for (proportion, _) in children {
            // Convert the proportion to a normalized value between 0 and 1.
            let normalized_proportion: U96F32 = U96F32::saturating_from_num(proportion)
                .safe_div(U96F32::saturating_from_num(u64::MAX));
            log::trace!("Normalized proportion for child: {normalized_proportion:?}");

            // Calculate the amount of alpha to be allocated to this child.
            let alpha_proportion_to_child: U96F32 =
                U96F32::saturating_from_num(initial_alpha).saturating_mul(normalized_proportion);
            log::trace!("Alpha proportion to child: {alpha_proportion_to_child:?}");

            // Add this child's allocation to the total alpha allocated to children.
            alpha_to_children = alpha_to_children.saturating_add(alpha_proportion_to_child);
        }
        log::debug!("Total alpha allocated to children: {alpha_to_children:?}");

        // Step 4: Calculate the total alpha inherited from parents.
        for (proportion, parent) in parents {
            // Retrieve the parent's total stake on this subnet.
            let parent_alpha: U96F32 =
                U96F32::saturating_from_num(Self::get_stake_for_hotkey_on_subnet(&parent, netuid));
            log::trace!("Parent alpha for parent {parent:?} on subnet {netuid}: {parent_alpha:?}");

            // Convert the proportion to a normalized value between 0 and 1.
            let normalized_proportion: U96F32 = U96F32::saturating_from_num(proportion)
                .safe_div(U96F32::saturating_from_num(u64::MAX));
            log::trace!("Normalized proportion from parent: {normalized_proportion:?}");

            // Calculate the amount of alpha to be inherited from this parent.
            let alpha_proportion_from_parent: U96F32 =
                U96F32::saturating_from_num(parent_alpha).saturating_mul(normalized_proportion);
            log::trace!("Alpha proportion from parent: {alpha_proportion_from_parent:?}");

            // Add this parent's contribution to the total alpha inherited from parents.
            alpha_from_parents = alpha_from_parents.saturating_add(alpha_proportion_from_parent);
        }
        log::debug!("Total alpha inherited from parents: {alpha_from_parents:?}");

        // Step 5: Calculate the final inherited alpha for the hotkey.
        let finalized_alpha: U96F32 = initial_alpha
            .saturating_sub(alpha_to_children) // Subtract alpha allocated to children
            .saturating_add(alpha_from_parents); // Add alpha inherited from parents
        log::trace!(
            "Finalized alpha for hotkey {hotkey:?} on subnet {netuid}: {finalized_alpha:?}"
        );

        // Step 6: Return the final inherited alpha value.
        finalized_alpha.saturating_to_num::<u64>().into()
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
    pub fn calculate_reduced_stake_on_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
        decrement: AlphaCurrency,
    ) -> Result<AlphaCurrency, Error<T>> {
        // Retrieve the current stake for this hotkey-coldkey pair on the subnet
        let current_stake =
            Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid);

        // Compare the current stake with the requested decrement
        // Return true if the current stake is greater than or equal to the decrement
        if current_stake >= decrement {
            Ok(current_stake.saturating_sub(decrement))
        } else {
            Err(Error::<T>::NotEnoughStakeToWithdraw)
        }
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
        netuid: NetUid,
    ) -> AlphaCurrency {
        let alpha_share_pool = Self::get_alpha_share_pool(hotkey.clone(), netuid);
        alpha_share_pool.try_get_value(coldkey).unwrap_or(0).into()
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
    pub fn get_stake_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: NetUid) -> AlphaCurrency {
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
        netuid: NetUid,
        amount: AlphaCurrency,
    ) {
        let mut alpha_share_pool = Self::get_alpha_share_pool(hotkey.clone(), netuid);
        alpha_share_pool.update_value_for_all(amount.to_u64() as i64);
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
    pub fn decrease_stake_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: NetUid, amount: u64) {
        let mut alpha_share_pool = Self::get_alpha_share_pool(hotkey.clone(), netuid);
        alpha_share_pool.update_value_for_all((amount as i64).neg());
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
        netuid: NetUid,
        amount: AlphaCurrency,
    ) -> AlphaCurrency {
        if !amount.is_zero() {
            let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
            if !staking_hotkeys.contains(hotkey) {
                staking_hotkeys.push(hotkey.clone());
                StakingHotkeys::<T>::insert(coldkey, staking_hotkeys.clone());
            }
        }

        let mut alpha_share_pool = Self::get_alpha_share_pool(hotkey.clone(), netuid);
        // We expect to add a positive amount here.
        let amount = amount.to_u64() as i64;
        let actual_alpha = alpha_share_pool.update_value_for_one(coldkey, amount);

        // We should return a positive amount, or 0 if the operation failed.
        // e.g. the stake was removed due to precision issues.
        actual_alpha.max(0).unsigned_abs().into()
    }

    pub fn try_increase_stake_for_hotkey_and_coldkey_on_subnet(
        hotkey: &T::AccountId,
        netuid: NetUid,
        amount: AlphaCurrency,
    ) -> bool {
        let mut alpha_share_pool = Self::get_alpha_share_pool(hotkey.clone(), netuid);
        let amount = amount.to_u64() as i64;
        alpha_share_pool.sim_update_value_for_one(amount)
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
        netuid: NetUid,
        amount: AlphaCurrency,
    ) -> AlphaCurrency {
        let mut alpha_share_pool = Self::get_alpha_share_pool(hotkey.clone(), netuid);
        let amount = amount.to_u64();

        // We expect a negative value here
        let mut actual_alpha = 0;
        if let Ok(value) = alpha_share_pool.try_get_value(coldkey)
            && value >= amount
        {
            actual_alpha = alpha_share_pool.update_value_for_one(coldkey, (amount as i64).neg());
        }

        // Get the negation of the removed alpha, and clamp at 0.
        // This ensures we return a positive value, but only if
        // `actual_alpha` was negative (i.e. a decrease in stake).
        actual_alpha.neg().max(0).unsigned_abs().into()
    }

    /// Swaps TAO for the alpha token on the subnet.
    ///
    /// Updates TaoIn, AlphaIn, and AlphaOut
    pub fn swap_tao_for_alpha(
        netuid: NetUid,
        tao: TaoCurrency,
        price_limit: TaoCurrency,
        drop_fees: bool,
    ) -> Result<SwapResult<TaoCurrency, AlphaCurrency>, DispatchError> {
        // Step 1: Get the mechanism type for the subnet (0 for Stable, 1 for Dynamic)
        let mechanism_id: u16 = SubnetMechanism::<T>::get(netuid);
        let swap_result = if mechanism_id == 1 {
            let order = GetAlphaForTao::<T>::with_amount(tao);
            T::SwapInterface::swap(netuid.into(), order, price_limit.into(), drop_fees, false)?
        } else {
            // Step 3.b.1: Stable mechanism, just return the value 1:1
            SwapResult {
                amount_paid_in: tao,
                amount_paid_out: tao.to_u64().into(),
                fee_paid: TaoCurrency::ZERO,
            }
        };

        let alpha_decrease = swap_result.paid_out_reserve_delta_i64().unsigned_abs();

        // Decrease Alpha reserves.
        Self::decrease_provided_alpha_reserve(netuid.into(), alpha_decrease.into());

        // Increase Alpha outstanding.
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(swap_result.amount_paid_out.into());
        });

        // Increase only the protocol TAO reserve. We only use the sum of
        // (SubnetTAO + SubnetTaoProvided) in tao_reserve(), so it is irrelevant
        // which one to increase.
        SubnetTAO::<T>::mutate(netuid, |total| {
            let delta = swap_result.paid_in_reserve_delta_i64().unsigned_abs();
            *total = total.saturating_add(delta.into());
        });

        // Increase Total Tao reserves.
        TotalStake::<T>::mutate(|total| *total = total.saturating_add(tao));

        // Increase total subnet TAO volume.
        SubnetVolume::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(tao.to_u64() as u128);
        });

        Ok(swap_result)
    }

    /// Swaps a subnet's Alpha token for TAO.
    ///
    /// Updates TaoIn, AlphaIn, and AlphaOut
    pub fn swap_alpha_for_tao(
        netuid: NetUid,
        alpha: AlphaCurrency,
        price_limit: TaoCurrency,
        drop_fees: bool,
    ) -> Result<SwapResult<AlphaCurrency, TaoCurrency>, DispatchError> {
        // Step 1: Get the mechanism type for the subnet (0 for Stable, 1 for Dynamic)
        let mechanism_id: u16 = SubnetMechanism::<T>::get(netuid);
        // Step 2: Swap alpha and attain tao
        let swap_result = if mechanism_id == 1 {
            let order = GetTaoForAlpha::<T>::with_amount(alpha);
            T::SwapInterface::swap(netuid.into(), order, price_limit.into(), drop_fees, false)?
        } else {
            // Step 3.b.1: Stable mechanism, just return the value 1:1
            SwapResult {
                amount_paid_in: alpha,
                amount_paid_out: alpha.to_u64().into(),
                fee_paid: AlphaCurrency::ZERO,
            }
        };

        // Increase only the protocol Alpha reserve. We only use the sum of
        // (SubnetAlphaIn + SubnetAlphaInProvided) in alpha_reserve(), so it is irrelevant
        // which one to increase.
        let alpha_delta = swap_result.paid_in_reserve_delta_i64().unsigned_abs();
        SubnetAlphaIn::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(alpha_delta.into());
        });

        // Decrease Alpha outstanding.
        // TODO: Deprecate, not accurate in v3 anymore
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub(alpha_delta.into());
        });

        // Decrease tao reserves.
        let tao_delta = swap_result.paid_out_reserve_delta_i64().unsigned_abs();
        Self::decrease_provided_tao_reserve(netuid.into(), tao_delta.into());

        // Reduce total TAO reserves.
        TotalStake::<T>::mutate(|total| *total = total.saturating_sub(swap_result.amount_paid_out));

        // Increase total subnet TAO volume.
        SubnetVolume::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(swap_result.amount_paid_out.to_u64() as u128)
        });

        // Return the tao received.
        Ok(swap_result)
    }

    /// Unstakes alpha from a subnet for a given hotkey and coldkey pair.
    ///
    /// We update the pools associated with a subnet as well as update hotkey alpha shares.
    pub fn unstake_from_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
        price_limit: TaoCurrency,
        drop_fees: bool,
    ) -> Result<TaoCurrency, DispatchError> {
        //  Decrease alpha on subnet
        let actual_alpha_decrease =
            Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid, alpha);

        // Swap the alpha for TAO.
        let swap_result =
            Self::swap_alpha_for_tao(netuid, actual_alpha_decrease, price_limit, drop_fees)?;

        // Refund the unused alpha (in case if limit price is hit)
        let refund = actual_alpha_decrease.saturating_sub(
            swap_result
                .amount_paid_in
                .saturating_add(swap_result.fee_paid)
                .into(),
        );
        if !refund.is_zero() {
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid, refund);
        }

        // If this is a root-stake
        if netuid == NetUid::ROOT {
            // Adjust root claimed value for this hotkey and coldkey.
            Self::remove_stake_adjust_root_claimed_for_hotkey_and_coldkey(hotkey, coldkey, alpha);
        }

        // Step 3: Update StakingHotkeys if the hotkey's total alpha, across all subnets, is zero
        // TODO const: fix.
        // if Self::get_stake(hotkey, coldkey) == 0 {
        //     StakingHotkeys::<T>::mutate(coldkey, |hotkeys| {
        //         hotkeys.retain(|k| k != hotkey);
        //     });
        // }

        // Record TAO outflow
        Self::record_tao_outflow(netuid, swap_result.amount_paid_out.into());

        LastColdkeyHotkeyStakeBlock::<T>::insert(coldkey, hotkey, Self::get_current_block_as_u64());

        // Deposit and log the unstaking event.
        Self::deposit_event(Event::StakeRemoved(
            coldkey.clone(),
            hotkey.clone(),
            swap_result.amount_paid_out.into(),
            actual_alpha_decrease,
            netuid,
            swap_result.fee_paid.to_u64(),
        ));

        log::debug!(
            "StakeRemoved( coldkey: {:?}, hotkey:{:?}, tao: {:?}, alpha:{:?}, netuid: {:?}, fee {} )",
            coldkey.clone(),
            hotkey.clone(),
            swap_result.amount_paid_out,
            actual_alpha_decrease,
            netuid,
            swap_result.fee_paid
        );

        Ok(swap_result.amount_paid_out.into())
    }

    /// Stakes TAO into a subnet for a given hotkey and coldkey pair.
    ///
    /// We update the pools associated with a subnet as well as update hotkey alpha shares.
    pub(crate) fn stake_into_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
        tao: TaoCurrency,
        price_limit: TaoCurrency,
        set_limit: bool,
        drop_fees: bool,
    ) -> Result<AlphaCurrency, DispatchError> {
        // Swap the tao to alpha.
        let swap_result = Self::swap_tao_for_alpha(netuid, tao, price_limit, drop_fees)?;

        ensure!(
            !swap_result.amount_paid_out.is_zero(),
            Error::<T>::AmountTooLow
        );

        ensure!(
            Self::try_increase_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                netuid,
                swap_result.amount_paid_out.into(),
            ),
            Error::<T>::InsufficientLiquidity
        );

        // Increase the alpha on the hotkey account.
        if Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
            hotkey,
            coldkey,
            netuid,
            swap_result.amount_paid_out.into(),
        )
        .is_zero()
            || swap_result.amount_paid_out.is_zero()
        {
            return Ok(AlphaCurrency::ZERO);
        }

        // Step 4: Update the list of hotkeys staking for this coldkey
        let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
        if !staking_hotkeys.contains(hotkey) {
            staking_hotkeys.push(hotkey.clone());
            StakingHotkeys::<T>::insert(coldkey, staking_hotkeys.clone());
        }

        // Record TAO inflow
        Self::record_tao_inflow(netuid, swap_result.amount_paid_in.into());

        LastColdkeyHotkeyStakeBlock::<T>::insert(coldkey, hotkey, Self::get_current_block_as_u64());

        if set_limit {
            Self::set_stake_operation_limit(hotkey, coldkey, netuid.into());
        }

        // If this is a root-stake
        if netuid == NetUid::ROOT {
            // Adjust root claimed for this hotkey and coldkey.
            let alpha = swap_result.amount_paid_out.into();
            Self::add_stake_adjust_root_claimed_for_hotkey_and_coldkey(hotkey, coldkey, alpha);
            Self::maybe_add_coldkey_index(coldkey);
        }

        // Deposit and log the staking event.
        Self::deposit_event(Event::StakeAdded(
            coldkey.clone(),
            hotkey.clone(),
            tao,
            swap_result.amount_paid_out.into(),
            netuid,
            swap_result.fee_paid.to_u64(),
        ));

        log::debug!(
            "StakeAdded( coldkey: {:?}, hotkey:{:?}, tao: {:?}, alpha:{:?}, netuid: {:?}, fee {} )",
            coldkey.clone(),
            hotkey.clone(),
            tao,
            swap_result.amount_paid_out,
            netuid,
            swap_result.fee_paid,
        );

        Ok(swap_result.amount_paid_out.into())
    }

    /// Transfers stake between coldkeys and/or hotkey within one subnet without running it
    /// through swap.
    ///
    /// Does not incur any swapping nor fees
    pub fn transfer_stake_within_subnet(
        origin_coldkey: &T::AccountId,
        origin_hotkey: &T::AccountId,
        destination_coldkey: &T::AccountId,
        destination_hotkey: &T::AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
    ) -> Result<TaoCurrency, DispatchError> {
        // Decrease alpha on origin keys
        let actual_alpha_decrease = Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
            origin_hotkey,
            origin_coldkey,
            netuid,
            alpha,
        );
        if netuid == NetUid::ROOT {
            Self::remove_stake_adjust_root_claimed_for_hotkey_and_coldkey(
                origin_hotkey,
                origin_coldkey,
                alpha,
            );
        }

        // Increase alpha on destination keys
        let actual_alpha_moved = Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
            destination_hotkey,
            destination_coldkey,
            netuid,
            actual_alpha_decrease,
        );
        if netuid == NetUid::ROOT {
            Self::add_stake_adjust_root_claimed_for_hotkey_and_coldkey(
                destination_hotkey,
                destination_coldkey,
                actual_alpha_decrease.into(),
            );
        }

        // Calculate TAO equivalent based on current price (it is accurate because
        // there's no slippage in this move)
        let current_price =
            <T as pallet::Config>::SwapInterface::current_alpha_price(netuid.into());
        let tao_equivalent: TaoCurrency = current_price
            .saturating_mul(U96F32::saturating_from_num(actual_alpha_moved))
            .saturating_to_num::<u64>()
            .into();

        // Ensure tao_equivalent is above DefaultMinStake
        ensure!(
            tao_equivalent >= DefaultMinStake::<T>::get(),
            Error::<T>::AmountTooLow
        );

        // Step 3: Update StakingHotkeys if the hotkey's total alpha, across all subnets, is zero
        // TODO: fix.
        // if Self::get_stake(hotkey, coldkey) == 0 {
        //     StakingHotkeys::<T>::mutate(coldkey, |hotkeys| {
        //         hotkeys.retain(|k| k != hotkey);
        //     });
        // }

        LastColdkeyHotkeyStakeBlock::<T>::insert(
            destination_coldkey,
            destination_hotkey,
            Self::get_current_block_as_u64(),
        );

        // Deposit and log the unstaking event.
        Self::deposit_event(Event::StakeRemoved(
            origin_coldkey.clone(),
            origin_hotkey.clone(),
            tao_equivalent,
            actual_alpha_decrease,
            netuid,
            0_u64, // 0 fee
        ));
        Self::deposit_event(Event::StakeAdded(
            destination_coldkey.clone(),
            destination_hotkey.clone(),
            tao_equivalent,
            actual_alpha_moved,
            netuid,
            0_u64, // 0 fee
        ));

        Ok(tao_equivalent)
    }

    pub fn get_alpha_share_pool(
        hotkey: <T as frame_system::Config>::AccountId,
        netuid: NetUid,
    ) -> SharePool<AlphaShareKey<T>, HotkeyAlphaSharePoolDataOperations<T>> {
        let ops = HotkeyAlphaSharePoolDataOperations::new(hotkey, netuid);
        SharePool::<AlphaShareKey<T>, HotkeyAlphaSharePoolDataOperations<T>>::new(ops)
    }

    /// Validate add_stake user input
    pub fn validate_add_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        mut stake_to_be_added: TaoCurrency,
        max_amount: TaoCurrency,
        allow_partial: bool,
    ) -> Result<(), Error<T>> {
        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        // Ensure that the subnet is enabled.
        Self::ensure_subtoken_enabled(netuid)?;

        // Get the minimum balance (and amount) that satisfies the transaction
        let min_stake = DefaultMinStake::<T>::get();
        let min_amount = {
            let order = GetAlphaForTao::<T>::with_amount(min_stake);
            let fee = T::SwapInterface::sim_swap(netuid.into(), order)
                .map(|res| res.fee_paid)
                .unwrap_or(T::SwapInterface::approx_fee_amount(
                    netuid.into(),
                    min_stake.into(),
                ));
            min_stake.saturating_add(fee.into())
        };

        // Ensure that the stake_to_be_added is at least the min_amount
        ensure!(stake_to_be_added >= min_amount, Error::<T>::AmountTooLow);

        // Ensure that if partial execution is not allowed, the amount will not cause
        // slippage over desired
        if !allow_partial {
            ensure!(stake_to_be_added <= max_amount, Error::<T>::SlippageTooHigh);
        } else {
            stake_to_be_added = max_amount.min(stake_to_be_added);
        }

        // Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!(
            Self::can_remove_balance_from_coldkey_account(coldkey, stake_to_be_added.into()),
            Error::<T>::NotEnoughBalanceToPayStake
        );

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        let order = GetAlphaForTao::<T>::with_amount(stake_to_be_added);
        let swap_result = T::SwapInterface::sim_swap(netuid.into(), order)
            .map_err(|_| Error::<T>::InsufficientLiquidity)?;

        // Check that actual withdrawn TAO amount is not lower than the minimum stake
        ensure!(
            swap_result.amount_paid_in >= min_stake,
            Error::<T>::AmountTooLow
        );

        ensure!(
            !swap_result.amount_paid_out.is_zero(),
            Error::<T>::InsufficientLiquidity
        );

        // Ensure hotkey pool is precise enough
        let try_stake_result = Self::try_increase_stake_for_hotkey_and_coldkey_on_subnet(
            hotkey,
            netuid,
            swap_result.amount_paid_out.into(),
        );
        ensure!(try_stake_result, Error::<T>::InsufficientLiquidity);

        Ok(())
    }

    /// Validate remove_stake user input
    ///
    pub fn validate_remove_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: NetUid,
        alpha_unstaked: AlphaCurrency,
        max_amount: AlphaCurrency,
        allow_partial: bool,
    ) -> Result<(), Error<T>> {
        // Ensure that the subnet exists.
        ensure!(Self::if_subnet_exist(netuid), Error::<T>::SubnetNotExists);

        Self::ensure_stake_operation_limit_not_exceeded(hotkey, coldkey, netuid.into())?;

        // Ensure that the subnet is enabled.
        // Self::ensure_subtoken_enabled(netuid)?;

        // Do not allow zero unstake amount
        ensure!(!alpha_unstaked.is_zero(), Error::<T>::AmountTooLow);

        // Ensure that the stake amount to be removed is above the minimum in tao equivalent.
        // Bypass this check if the user unstakes full amount
        let remaining_alpha_stake =
            Self::calculate_reduced_stake_on_subnet(hotkey, coldkey, netuid, alpha_unstaked)?;
        let order = GetTaoForAlpha::<T>::with_amount(alpha_unstaked);
        match T::SwapInterface::sim_swap(netuid.into(), order) {
            Ok(res) => {
                if !remaining_alpha_stake.is_zero() {
                    ensure!(
                        res.amount_paid_out >= DefaultMinStake::<T>::get(),
                        Error::<T>::AmountTooLow
                    );
                }
            }
            Err(_) => return Err(Error::<T>::InsufficientLiquidity),
        }

        // Ensure that if partial execution is not allowed, the amount will not cause
        // slippage over desired
        if !allow_partial {
            ensure!(alpha_unstaked <= max_amount, Error::<T>::SlippageTooHigh);
        }

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        Ok(())
    }

    /// Validate if unstake_all can be executed
    ///
    pub fn validate_unstake_all(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        only_alpha: bool,
    ) -> Result<(), Error<T>> {
        // Get all netuids (filter out root)
        let subnets = Self::get_all_subnet_netuids();

        // Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        let mut unstaking_any = false;
        for netuid in subnets.iter() {
            if only_alpha && netuid.is_root() {
                continue;
            }

            // Get user's stake in this subnet
            let alpha = Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, *netuid);

            if Self::validate_remove_stake(coldkey, hotkey, *netuid, alpha, alpha, false).is_ok() {
                unstaking_any = true;
            }
        }

        // If no unstaking happens, return error
        ensure!(unstaking_any, Error::<T>::AmountTooLow);

        Ok(())
    }

    /// Validate stake transition user input
    /// That works for move_stake, transfer_stake, and swap_stake
    ///
    pub fn validate_stake_transition(
        origin_coldkey: &T::AccountId,
        destination_coldkey: &T::AccountId,
        origin_hotkey: &T::AccountId,
        destination_hotkey: &T::AccountId,
        origin_netuid: NetUid,
        destination_netuid: NetUid,
        alpha_amount: AlphaCurrency,
        max_amount: AlphaCurrency,
        maybe_allow_partial: Option<bool>,
        check_transfer_toggle: bool,
    ) -> Result<(), Error<T>> {
        // Ensure stake transition is actually happening
        if origin_coldkey == destination_coldkey && origin_hotkey == destination_hotkey {
            ensure!(origin_netuid != destination_netuid, Error::<T>::SameNetuid);
        }

        Self::ensure_stake_operation_limit_not_exceeded(
            origin_hotkey,
            origin_coldkey,
            origin_netuid.into(),
        )?;

        // Ensure that both subnets exist.
        ensure!(
            Self::if_subnet_exist(origin_netuid),
            Error::<T>::SubnetNotExists
        );
        if origin_netuid != destination_netuid {
            ensure!(
                Self::if_subnet_exist(destination_netuid),
                Error::<T>::SubnetNotExists
            );
        }

        ensure!(
            SubtokenEnabled::<T>::get(origin_netuid),
            Error::<T>::SubtokenDisabled
        );

        ensure!(
            SubtokenEnabled::<T>::get(destination_netuid),
            Error::<T>::SubtokenDisabled
        );

        // Ensure that the origin hotkey account exists
        ensure!(
            Self::hotkey_account_exists(origin_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure that the destination hotkey account exists
        ensure!(
            Self::hotkey_account_exists(destination_hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // Ensure there is enough stake in the origin subnet.
        let origin_alpha = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
            origin_hotkey,
            origin_coldkey,
            origin_netuid,
        );
        ensure!(
            alpha_amount <= origin_alpha,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // If origin and destination netuid are different, do the swap-related checks
        if origin_netuid != destination_netuid {
            // Ensure that the stake amount to be removed is above the minimum in tao equivalent.
            let order = GetTaoForAlpha::<T>::with_amount(alpha_amount);
            let tao_equivalent = T::SwapInterface::sim_swap(origin_netuid.into(), order)
                .map(|res| res.amount_paid_out)
                .map_err(|_| Error::<T>::InsufficientLiquidity)?;
            ensure!(
                tao_equivalent > DefaultMinStake::<T>::get(),
                Error::<T>::AmountTooLow
            );

            // Ensure that if partial execution is not allowed, the amount will not cause
            // slippage over desired
            if let Some(allow_partial) = maybe_allow_partial
                && !allow_partial
            {
                ensure!(alpha_amount <= max_amount, Error::<T>::SlippageTooHigh);
            }
        }

        if check_transfer_toggle {
            // Ensure transfer is toggled.
            ensure!(
                TransferToggle::<T>::get(origin_netuid),
                Error::<T>::TransferDisallowed
            );
            if origin_netuid != destination_netuid {
                ensure!(
                    TransferToggle::<T>::get(destination_netuid),
                    Error::<T>::TransferDisallowed
                );
            }
        }

        Ok(())
    }

    pub fn increase_provided_tao_reserve(netuid: NetUid, tao: TaoCurrency) {
        SubnetTaoProvided::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(tao);
        });
    }

    pub fn decrease_provided_tao_reserve(netuid: NetUid, tao: TaoCurrency) {
        // First, decrease SubnetTaoProvided, then deduct the rest from SubnetTAO
        let subnet_tao = SubnetTAO::<T>::get(netuid);
        let subnet_tao_provided = SubnetTaoProvided::<T>::get(netuid);
        let remainder = subnet_tao_provided.saturating_sub(tao);
        let carry_over = tao.saturating_sub(subnet_tao_provided);
        if carry_over.is_zero() {
            SubnetTaoProvided::<T>::set(netuid, remainder);
        } else {
            SubnetTaoProvided::<T>::set(netuid, TaoCurrency::ZERO);
            SubnetTAO::<T>::set(netuid, subnet_tao.saturating_sub(carry_over));
        }
    }

    pub fn increase_provided_alpha_reserve(netuid: NetUid, alpha: AlphaCurrency) {
        SubnetAlphaInProvided::<T>::mutate(netuid, |total| {
            *total = total.saturating_add(alpha);
        });
    }

    pub fn decrease_provided_alpha_reserve(netuid: NetUid, alpha: AlphaCurrency) {
        // First, decrease SubnetAlphaInProvided, then deduct the rest from SubnetAlphaIn
        let subnet_alpha = SubnetAlphaIn::<T>::get(netuid);
        let subnet_alpha_provided = SubnetAlphaInProvided::<T>::get(netuid);
        let remainder = subnet_alpha_provided.saturating_sub(alpha);
        let carry_over = alpha.saturating_sub(subnet_alpha_provided);
        if carry_over.is_zero() {
            SubnetAlphaInProvided::<T>::set(netuid, remainder);
        } else {
            SubnetAlphaInProvided::<T>::set(netuid, AlphaCurrency::ZERO);
            SubnetAlphaIn::<T>::set(netuid, subnet_alpha.saturating_sub(carry_over));
        }
    }

    pub fn set_stake_operation_limit(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) {
        StakingOperationRateLimiter::<T>::insert((hotkey, coldkey, netuid), true);
    }

    pub fn ensure_stake_operation_limit_not_exceeded(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: NetUid,
    ) -> Result<(), Error<T>> {
        ensure!(
            !StakingOperationRateLimiter::<T>::contains_key((hotkey, coldkey, netuid)),
            Error::<T>::StakingOperationRateLimitExceeded
        );

        Ok(())
    }
}

///////////////////////////////////////////
// Alpha share pool chain data layer

#[derive(Debug)]
pub struct HotkeyAlphaSharePoolDataOperations<T: frame_system::Config> {
    netuid: NetUid,
    hotkey: <T as frame_system::Config>::AccountId,
    _marker: sp_std::marker::PhantomData<T>,
}

impl<T: Config> HotkeyAlphaSharePoolDataOperations<T> {
    fn new(hotkey: <T as frame_system::Config>::AccountId, netuid: NetUid) -> Self {
        HotkeyAlphaSharePoolDataOperations {
            netuid,
            hotkey,
            _marker: sp_std::marker::PhantomData,
        }
    }
}

// Alpha share key is coldkey because the HotkeyAlphaSharePoolDataOperations struct already has hotkey and netuid
type AlphaShareKey<T> = <T as frame_system::Config>::AccountId;

impl<T: Config> SharePoolDataOperations<AlphaShareKey<T>>
    for HotkeyAlphaSharePoolDataOperations<T>
{
    fn get_shared_value(&self) -> U64F64 {
        U64F64::saturating_from_num(crate::TotalHotkeyAlpha::<T>::get(&self.hotkey, self.netuid))
    }

    fn get_share(&self, key: &AlphaShareKey<T>) -> U64F64 {
        crate::Alpha::<T>::get((&(self.hotkey), key, self.netuid))
    }

    fn try_get_share(&self, key: &AlphaShareKey<T>) -> Result<U64F64, ()> {
        crate::Alpha::<T>::try_get((&(self.hotkey), key, self.netuid))
    }

    fn get_denominator(&self) -> U64F64 {
        crate::TotalHotkeyShares::<T>::get(&(self.hotkey), self.netuid)
    }

    fn set_shared_value(&mut self, value: U64F64) {
        if value != 0 {
            crate::TotalHotkeyAlpha::<T>::insert(
                &(self.hotkey),
                self.netuid,
                AlphaCurrency::from(value.saturating_to_num::<u64>()),
            );
        } else {
            crate::TotalHotkeyAlpha::<T>::remove(&(self.hotkey), self.netuid);
        }
    }

    fn set_share(&mut self, key: &AlphaShareKey<T>, share: U64F64) {
        if share != 0 {
            crate::Alpha::<T>::insert((&self.hotkey, key, self.netuid), share);
        } else {
            crate::Alpha::<T>::remove((&self.hotkey, key, self.netuid));
        }
    }

    fn set_denominator(&mut self, update: U64F64) {
        if update != 0 {
            crate::TotalHotkeyShares::<T>::insert(&self.hotkey, self.netuid, update);
        } else {
            crate::TotalHotkeyShares::<T>::remove(&self.hotkey, self.netuid);
        }
    }
}
