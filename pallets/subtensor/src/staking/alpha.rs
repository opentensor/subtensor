use super::*;
use crate::epoch::math::*;
use frame_support::IterableStorageDoubleMap;
use substrate_fixed::types::{I96F32, I64F64, I32F32};


impl<T: Config> Pallet<T> {

    pub fn get_dynamic_weight() -> I96F32 {
        I96F32::from_num(DynamicWeight::<T>::get())/I96F32::from_num(u64::MAX)
    }
    pub fn set_dynamic_weight( weight: u64 ) {
        DynamicWeight::<T>::put( weight );
    }

    /// Calculates the weighted combination between alpha and dynamic tao for hotkeys on a subnet.
    ///
    /// # Arguments
    /// * `netuid` - Network unique identifier specifying the network context over which we will determine the stake weight.
    ///
    /// # Returns
    /// * `Vec<I32F32>` - The stake weights for each hotkey on the network.
    ///
    pub fn get_stake_weights_for_network(netuid: u16) -> Vec<I32F32> {

        // Get the subnet size.
        let n: u16 = Self::get_subnetwork_n(netuid);

        // First retreive all the hotkeys on this network
        let hotkeys: Vec<(u16, T::AccountId)> =
            <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(netuid)
                .collect();

        // Attain the alpha stake vector.
        let mut alpha_stake: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        for (uid_i, hotkey) in &hotkeys {
            alpha_stake[*uid_i as usize] = I64F64::from_num(Self::get_inherited_stake_for_hotkey_on_subnet(hotkey, netuid));
        }
        inplace_normalize_64(&mut alpha_stake);
        log::trace!("Alpha Stake:\n{:?}\n", &alpha_stake);
 
        // Attain the dynamic tao stake vector.
        let mut dynamic_tao_stake: Vec<I64F64> = vec![I64F64::from_num(0.0); n as usize];
        for (uid_i, hotkey) in &hotkeys {
            dynamic_tao_stake[*uid_i as usize] = I64F64::from_num(Self::get_dynamic_stake_for_hotkey_on_subnet(hotkey, netuid));
        }
        inplace_normalize_64(&mut dynamic_tao_stake);
        log::trace!("Dynamic TAO Stake:\n{:?}\n", &dynamic_tao_stake);

        // Average local and global weights.
        let dynamic_weight: I64F64 = I64F64::from_num( Self::get_dynamic_weight() );
        let mut stake_weights: Vec<I64F64> = alpha_stake
            .iter()
            .zip(dynamic_tao_stake.iter())
            .map(|(alpha, dynamic)| (I64F64::from_num(1.0) - dynamic_weight) * (*alpha) + dynamic_weight * (*dynamic))
            .collect();
        inplace_normalize_64(&mut stake_weights);

        // Convert the averaged stake values from 64-bit fixed-point to 32-bit fixed-point representation.
        let converted_stake_weights: Vec<I32F32> = vec_fixed64_to_fixed32(stake_weights);

        // Normalized the two and combine.
        log::trace!("Stake Weights:\n{:?}\n", &converted_stake_weights);
        converted_stake_weights
    }

    /// Calculates the total dynamic stake held by a hotkey on a network, considering child/parent relationships.
    ///
    /// This function performs the following steps:
    /// 1. Retrieves the initial dynamic for the hotkey on the network.
    /// 2. Calculates the dynamic allocated to children.
    /// 3. Calculates the dynamic received from parents.
    /// 4. Computes the final dynamic by adjusting the initial dynamic with child and parent contributions.
    ///
    /// # Arguments
    /// * `hotkey` - AccountId of the hotkey whose total network stake is to be calculated.
    /// * `netuid` - Network unique identifier specifying the network context.
    ///
    /// # Returns
    /// * `u64` - The total dynamic for the hotkey on the network after considering the stakes
    ///           from children and parents.
    ///
    /// # Panics
    /// This function does not explicitly panic, but underlying arithmetic operations
    /// use saturating arithmetic to prevent overflows.
    pub fn get_dynamic_stake_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {

        // Get the initial dynamic tao for the hotkey.
        let initial_dynamic_tao: u64 = Self::get_global_for_hotkey( hotkey );
        let mut dynamic_tao_to_children: u64 = 0;
        let mut dynamic_tao_from_parents: u64 = 0;
        let parents: Vec<(u64, T::AccountId)> = Self::get_parents(hotkey, netuid);
        let children: Vec<(u64, T::AccountId)> = Self::get_children(hotkey, netuid);
        for (proportion, _) in children {
            // Calculate the stake proportion allocated to the child based on the initial stake.
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));
            let dynamic_tao_proportion_to_child: I96F32 =
                I96F32::from_num(initial_dynamic_tao).saturating_mul(normalized_proportion);
            dynamic_tao_to_children = dynamic_tao_to_children.saturating_add(dynamic_tao_proportion_to_child.to_num::<u64>());
        }
        // Iterate over parents to calculate the total stake received from them.
        for (proportion, parent) in parents {
            let parent_dynamic_tao: u64 = Self::get_global_for_hotkey( &parent );
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));
            let dynamic_tao_proportion_from_parent: I96F32 =
                I96F32::from_num(parent_dynamic_tao).saturating_mul(normalized_proportion);
            dynamic_tao_from_parents = dynamic_tao_from_parents.saturating_add(dynamic_tao_proportion_from_parent.to_num::<u64>());
        }
        let finalized_dynamic: u64 = initial_dynamic_tao
            .saturating_sub(dynamic_tao_to_children)
            .saturating_add(dynamic_tao_from_parents);

        finalized_dynamic
    }


    /// Calculates the total alpha held by a hotkey on a network, considering child/parent relationships.
    ///
    /// This function performs the following steps:
    /// 1. Retrieves the initial alpha for the hotkey on the network.
    /// 2. Calculates the alpha allocated to children.
    /// 3. Calculates the alpha received from parents.
    /// 4. Computes the final alpha by adjusting the initial alpha with child and parent contributions.
    ///
    /// # Arguments
    /// * `hotkey` - AccountId of the hotkey whose total network stake is to be calculated.
    /// * `netuid` - Network unique identifier specifying the network context.
    ///
    /// # Returns
    /// * `u64` - The total alpha for the hotkey on the network after considering the stakes
    ///           from children and parents.
    ///
    /// # Panics
    /// This function does not explicitly panic, but underlying arithmetic operations
    /// use saturating arithmetic to prevent overflows.
    pub fn get_inherited_stake_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {

        // Retrieve the initial total stake for the hotkey without any child/parent adjustments.
        let initial_alpha: u64 = Self::get_stake_for_hotkey_on_subnet(hotkey, netuid);
        let mut alpha_to_children: u64 = 0;
        let mut alpha_from_parents: u64 = 0;
        let parents: Vec<(u64, T::AccountId)> = Self::get_parents(hotkey, netuid);
        let children: Vec<(u64, T::AccountId)> = Self::get_children(hotkey, netuid);
        // Iterate over children to calculate the total alpha allocated to them.
        // We only allocated alpha to children, not the entire stake.
        for (proportion, _) in children {
            // Calculate the stake proportion allocated to the child based on the initial stake.
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));
            let alpha_proportion_to_child: I96F32 =
                I96F32::from_num(initial_alpha).saturating_mul(normalized_proportion);
            // Accumulate the total stake given to children.
            alpha_to_children = alpha_to_children.saturating_add(alpha_proportion_to_child.to_num::<u64>());
        }
        // Iterate over parents to calculate the total stake received from them.
        for (proportion, parent) in parents {
            // Retrieve the parent's total stake.
            let parent_alpha: u64 = Self::get_stake_for_hotkey_on_subnet( &parent, netuid );
            // Calculate the stake proportion received from the parent.
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));
            let alpha_proportion_from_parent: I96F32 =
                I96F32::from_num(parent_alpha).saturating_mul(normalized_proportion);
            // Accumulate the total stake received from parents.
            alpha_from_parents = alpha_from_parents.saturating_add(alpha_proportion_from_parent.to_num::<u64>());
        }

        // Calculate the final stake for the hotkey by adjusting the initial stake with the stakes
        // to/from children and parents.
        let finalized_alpha: u64 = initial_alpha
            .saturating_sub(alpha_to_children)
            .saturating_add(alpha_from_parents);

        // Return the finalized stake value for the hotkey.
        finalized_alpha
    }

    /// Retrieves the dynamic value for a given hotkey and coldkey across all subnets.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `coldkey` - The account ID of the coldkey.
    ///
    /// # Returns
    /// * `u64` - The total dynamic value for the hotkey and coldkey across all subnets.
    pub fn get_global_for_hotkey_and_coldkey( hotkey: &T::AccountId, coldkey: &T::AccountId ) -> u64 {
        log::trace!(target: "subtensor", "Entering get_global_for_hotkey_and_coldkey. hotkey: {:?}, coldkey: {:?}", hotkey, coldkey);

        // Initialize the total tao equivalent to zero.
        let mut total_tao_equivalent: I96F32 = I96F32::from_num( 0 );
        log::trace!(target: "subtensor", "Initial total_tao_equivalent: {:?}", total_tao_equivalent);

        // Iterate over all subnet netuids.
        for netuid in Self::get_all_subnet_netuids() {
            log::trace!(target: "subtensor", "Processing netuid: {}", netuid);

            // Accumulate the dynamic value for the hotkey and coldkey on each subnet.
            let subnet_dynamic = Self::get_global_for_hotkey_and_coldey_on_subnet( hotkey, coldkey, netuid );
            log::trace!(target: "subtensor", "Subnet {} dynamic value: {}", netuid, subnet_dynamic);

            total_tao_equivalent = total_tao_equivalent.saturating_add(I96F32::from_num(subnet_dynamic));
            log::trace!(target: "subtensor", "Updated total_tao_equivalent: {:?}", total_tao_equivalent);
        }

        // Return the total tao equivalent as a u64 value.
        let result = total_tao_equivalent.to_num::<u64>();
        log::trace!(target: "subtensor", "Exiting get_global_for_hotkey_and_coldkey. Result: {}", result);
        result
    }

    /// Retrieves the dynamic value for a given hotkey across all subnets.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    ///
    /// # Returns
    /// * `u64` - The total dynamic value for the hotkey across all subnets.
    pub fn get_global_for_hotkey( hotkey: &T::AccountId ) -> u64 {
        log::trace!(target: "subtensor", "Entering get_global_for_hotkey. hotkey: {:?}", hotkey);

        // Initialize the total tao equivalent to zero.
        let mut total_tao_equivalent: I96F32 = I96F32::from_num( 0 );
        log::trace!(target: "subtensor", "Initial total_tao_equivalent: {:?}", total_tao_equivalent);

        // Iterate over all subnet netuids.
        for netuid in Self::get_all_subnet_netuids() {
            log::trace!(target: "subtensor", "Processing netuid: {}", netuid);

            // Accumulate the dynamic value for the hotkey on each subnet.
            let subnet_dynamic = Self::get_global_for_hotkey_on_subnet( hotkey, netuid );
            log::trace!(target: "subtensor", "Subnet {} dynamic value: {}", netuid, subnet_dynamic);

            total_tao_equivalent = total_tao_equivalent.saturating_add(I96F32::from_num(subnet_dynamic));
            log::trace!(target: "subtensor", "Updated total_tao_equivalent: {:?}", total_tao_equivalent);
        }

        // Return the total tao equivalent as a u64 value.
        let result = total_tao_equivalent.to_num::<u64>();
        log::trace!(target: "subtensor", "Exiting get_global_for_hotkey. Result: {}", result);
        result
    }

    /// Retrieves the dynamic value for a given hotkey on a specific subnet.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The dynamic value for the hotkey on the specified subnet.
    pub fn get_global_for_hotkey_on_subnet( hotkey: &T::AccountId, netuid: u16 ) -> u64 {
        log::trace!(target: "subtensor", "Entering get_global_for_hotkey_on_subnet. hotkey: {:?}, netuid: {}", hotkey, netuid);

        // Get the hotkey's alpha value on this subnet.
        let alpha: u64 = Self::get_stake_for_hotkey_on_subnet( hotkey, netuid );
        log::trace!(target: "subtensor", "Alpha value for hotkey on subnet: {}", alpha);

        // Convert the alpha value to dynamic value and return it.
        let dynamic_value = Self::alpha_to_dynamic( alpha, netuid );
        log::trace!(target: "subtensor", "Converted dynamic value: {}", dynamic_value);

        log::trace!(target: "subtensor", "Exiting get_global_for_hotkey_on_subnet");
        return dynamic_value;
    }

    /// Retrieves the dynamic value for a given hotkey and coldkey on a specific subnet.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `coldkey` - The account ID of the coldkey.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The dynamic value for the hotkey and coldkey on the specified subnet.
    pub fn get_global_for_hotkey_and_coldey_on_subnet( hotkey: &T::AccountId, coldkey: &T::AccountId, netuid: u16 ) -> u64 {
        // Get the hotkey's alpha value on this subnet.
        let alpha: u64 = Alpha::<T>::get( ( hotkey, coldkey, netuid) );
        // Convert the alpha value to dynamic value and return it.
        return Self::alpha_to_dynamic( alpha, netuid );
    }

    /// Converts an alpha value to a dynamic value on a specific subnet.
    ///
    /// # Arguments
    /// * `alpha` - The alpha value to be converted.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The dynamic value equivalent to the given alpha value on the specified subnet.
    pub fn alpha_to_dynamic( alpha: u64, netuid: u16 ) -> u64 {
        // Retrieve the total alpha and total tao values for the subnet.
        let total_alpha: I96F32 = I96F32::from_num( SubnetAlpha::<T>::get( netuid ) );
        let total_tao: I96F32 = I96F32::from_num( SubnetTAO::<T>::get( netuid ) );
        // Calculate the tao equivalent for the given alpha value.
        let tao_equivalent: I96F32 = (I96F32::from_num(alpha).checked_div(total_alpha).unwrap_or(I96F32::from_num(0.0))) * total_tao;
        // Return the tao equivalent as a u64 value.
        tao_equivalent.to_num::<u64>()
    }

    /// Converts an alpha value to a dynamic value on a specific subnet.
    ///
    /// # Arguments
    /// * `alpha` - The alpha value to be converted.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The dynamic value equivalent to the given alpha value on the specified subnet.
    pub fn alpha_to_tao( alpha: u64, netuid: u16 ) -> u64 {
        let mechid: u16 = SubnetMechanism::<T>::get( netuid );
        if mechid == 2 { 
            // Dynamic
            let alpha_in: I96F32 = I96F32::from_num( alpha );

            let total_mechanism_tao: I96F32 = I96F32::from_num( Self::get_total_mechanism_tao( mechid ) );
            let total_alpha: I96F32 = I96F32::from_num( SubnetAlpha::<T>::get( netuid ) );
            let total_tao: I96F32 = I96F32::from_num( SubnetTAO::<T>::get( netuid ) );

            // Compute alpha proportion in tao.
            let alpha_proportion: I96F32 = alpha_in.checked_div(total_alpha).unwrap_or(I96F32::from_num(0.0));
            let alpha_proportion_in_tao: I96F32 = alpha_proportion * total_tao;

            // Compute price per alpha with slipapge.
            let numerator: I96F32 = total_tao.saturating_sub(alpha_proportion_in_tao);
            let denominator: I96F32 = total_mechanism_tao.saturating_sub(alpha_proportion_in_tao);
            let price_per_alpha: I96F32 = numerator.checked_div(denominator).unwrap_or(I96F32::from_num(1.0));

            // Compute unstake amount in TAO.
            (price_per_alpha * alpha_in).to_num::<u64>()
        } else {
            // Stable.
            return alpha;
        }
    }

     /// Converts an alpha value to a dynamic value on a specific subnet.
    ///
    /// # Arguments
    /// * `alpha` - The alpha value to be converted.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The dynamic value equivalent to the given alpha value on the specified subnet.
    pub fn tao_to_alpha( tao: u64, netuid: u16 ) -> u64 {
        let mechid: u16 = SubnetMechanism::<T>::get( netuid );
        if mechid == 2 { 
            // Dynamic
            let total_subnet_tao: u64 = SubnetTAO::<T>::get( netuid );
            let total_mechanism_tao: u64 = Self::get_total_mechanism_tao( mechid );
            return tao * ((total_mechanism_tao + tao) / (total_subnet_tao + tao));
        } else {
            // Stable.
            return tao;
        }
    }


    /// Retrieves the alpha value for a given hotkey on a specific subnet.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The alpha value for the hotkey on the specified subnet.
    pub fn get_stake_for_hotkey_on_subnet( hotkey: &T::AccountId, netuid: u16 ) -> u64 {
        // Return the alpha this hotkey owns on this subnet.
        TotalHotkeyAlpha::<T>::get( hotkey, netuid )
    }

    /// Retrieves the total stake (alpha) for a given coldkey on a specific subnet.
    ///
    /// # Arguments
    ///
    /// * `coldkey` - The account ID of the coldkey.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    ///
    /// * `u64` - The total stake (alpha) for the coldkey on the specified subnet.
    pub fn get_stake_for_coldkey_on_subnet( coldkey: &T::AccountId, netuid: u16 ) -> u64 {
        // Return the alpha this coldkey owns on this subnet.
        TotalColdkeyAlpha::<T>::get( coldkey, netuid )
    }

    /// Returns true if the cold-hot staking account has enough balance to fulfill the decrement.
    ///
    /// # Arguments
    /// * `coldkey` - The coldkey account ID.
    /// * `hotkey` - The hotkey account ID.
    /// * `decrement` - The amount to be decremented.
    ///
    /// # Returns
    /// True if the account has enough balance, false otherwise.
    pub fn has_enough_stake_on_subnet(hotkey: &T::AccountId, coldkey: &T::AccountId, netuid: u16, decrement: u64) -> bool {
        Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, coldkey, netuid) >= decrement
    }

    /// Retrieves the alpha value for a given hotkey on a specific subnet.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The alpha value for the hotkey on the specified subnet.
    pub fn get_stake_for_hotkey_and_coldkey_on_subnet( hotkey: &T::AccountId, coldkey: &T::AccountId, netuid: u16 ) -> u64 {
        // Return the alpha this hotkey owns on this subnet.
        Alpha::<T>::get( (hotkey, coldkey, netuid ))
    }

    /// Stakes TAO into a subnet for a given hotkey and coldkey pair.
    ///
    /// This function performs the following operations:
    /// 1. Computes the stake operation based on the subnet's mechanism.
    /// 2. Increments the subnet's total TAO.
    /// 3. Increments the total TAO staked for the hotkey-coldkey pair.
    /// 4. Converts TAO to alpha based on the subnet's mechanism.
    /// 5. Increases the subnet alpha for the hotkey-coldkey pair.
    /// 6. Updates the staking hotkeys map for the coldkey.
    ///
    /// # Arguments
    ///
    /// * `hotkey` - The account ID of the hotkey.
    /// * `coldkey` - The account ID of the coldkey.
    /// * `netuid` - The unique identifier of the subnet.
    /// * `tao` - The amount of TAO to stake.
    ///
    /// # Returns
    ///
    /// * `u64` - The amount of alpha staked.
    ///
    /// # Effects
    ///
    /// This function mutates the following storage items:
    /// - `SubnetMechanism`
    /// - `SubnetTAO`
    /// - `Stake`
    /// - `StakingHotkeys`
    pub fn stake_into_subnet( hotkey: &T::AccountId, coldkey: &T::AccountId, netuid: u16, tao_staked: u64 ) -> u64{
        // Increment total stake.
        TotalStake::<T>::mutate(|total| { *total = total.saturating_add( tao_staked );});
        // Increment the subnet tao.
        SubnetTAO::<T>::mutate(netuid, |total| { *total = total.saturating_add( tao_staked );});
        // Increment the total tao staked
        Stake::<T>::mutate(&hotkey, &coldkey, |stake| {*stake = stake.saturating_add( tao_staked );});
        // Convert tao to alpha.
        let alpha_staked: u64 = Self::tao_to_alpha( tao_staked, netuid );
        // Increment the alpha on the account.
        SubnetAlpha::<T>::mutate( netuid, |total| { *total = total.saturating_add(alpha_staked); });
        // Increment the coldkey total.
        TotalColdkeyAlpha::<T>::mutate(coldkey, netuid, |total| { *total = total.saturating_add(alpha_staked); });
        // Increment the hotkey total.
        TotalHotkeyAlpha::<T>::mutate(hotkey, netuid, |total| { *total = total.saturating_add(alpha_staked); });
        // Increment the hotkey alpha.
        Alpha::<T>::mutate((hotkey, coldkey, netuid), |alpha| { *alpha = alpha.saturating_add(alpha_staked);});
        // Update Staking hotkeys map.
        let mut staking_hotkeys = StakingHotkeys::<T>::get(&coldkey);
        if !staking_hotkeys.contains(&hotkey) {
            staking_hotkeys.push(hotkey.clone());
            StakingHotkeys::<T>::insert(&coldkey, staking_hotkeys);
        }
        // Return the converted alpha.
        alpha_staked
    }
    /// Unstakes alpha from a subnet for a given hotkey and coldkey pair.
    ///
    /// This function performs the following operations:
    /// 1. Decreases the subnet's total alpha.
    /// 2. Decreases the total alpha for the hotkey on the subnet.
    /// 3. Decreases the alpha for the hotkey-coldkey pair on the subnet.
    /// 4. Converts alpha to TAO based on the subnet's mechanism.
    /// 5. Decrements the total stake counter.
    /// 6. Decrements the subnet's total TAO.
    ///
    /// # Arguments
    ///
    /// * `hotkey` - The account ID of the hotkey.
    /// * `coldkey` - The account ID of the coldkey.
    /// * `netuid` - The unique identifier of the subnet.
    /// * `alpha` - The amount of alpha to unstake.
    ///
    /// # Returns
    ///
    /// * `u64` - The amount of TAO unstaked.
    ///
    /// # Effects
    ///
    /// This function mutates the following storage items:
    /// - `SubnetMechanism`
    /// - `SubnetAlpha`
    /// - `TotalHotkeyAlpha`
    /// - `Alpha`
    /// - `TotalStake`
    /// - `SubnetTAO`
    pub fn unstake_from_subnet( hotkey: &T::AccountId, coldkey: &T::AccountId, netuid: u16, alpha_unstaked: u64 ) -> u64 {
        // Decrease the account value and remove if zero
        Alpha::<T>::mutate_exists((hotkey, coldkey, netuid), |maybe_total| {
            if let Some(total) = maybe_total {
                let new_total = total.saturating_sub(alpha_unstaked);
                if new_total == 0 {
                    *maybe_total = None;
                } else {
                    *total = new_total;
                }
            }
        });
        // Mutate remove the alpha
        SubnetAlpha::<T>::mutate( netuid, |total| { *total = total.saturating_sub( alpha_unstaked ); });
        // Increment the coldkey total.
        TotalColdkeyAlpha::<T>::mutate(coldkey, netuid, |total| { *total = total.saturating_sub(alpha_unstaked); });
        // Decrease the totals.
        TotalHotkeyAlpha::<T>::mutate(hotkey, netuid, |total| { *total = total.saturating_sub( alpha_unstaked ); });
        // Convert the alpha to tao.
        let tao_unstaked = Self::alpha_to_tao( alpha_unstaked, netuid );
        // Decrement the total stake counter.
        TotalStake::<T>::put( TotalStake::<T>::get().saturating_sub( tao_unstaked ) );
        // Decrement the subnet tao.
        SubnetTAO::<T>::mutate(netuid, |total| { *total = total.saturating_sub(tao_unstaked); });
        // Decrement the coldkey stake.
        // TODO remove this map.
        // Stake::<T>::mutate( &coldkey, &hotkey, |stake| { *stake = stake.saturating_sub(tao_unstaked); });
        tao_unstaked
    }



}