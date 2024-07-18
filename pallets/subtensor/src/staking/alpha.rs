use super::*;
use crate::epoch::math::*;
use frame_support::IterableStorageDoubleMap;
use substrate_fixed::types::{I96F32, I64F64, I32F32};


impl<T: Config> Pallet<T> {


    /// Calculates the weighted combination between alpha and dynamic tao for hotkeys on a subnet.
    ///
    /// # Arguments
    /// * `netuid` - Network unique identifier specifying the network context over which we will determine the stake weight.
    ///
    /// # Returns
    /// * `Vec<I32F32>` - The stake weights for each hotkey on the network.
    ///
    pub fn distribute_to_nominators( hotkey: &T::AccountId, netuid: u16, emission: u64 ) -> u64 {

        // Iterate over all nominators.
        // TODO add the last stake increase term here.
        let mut total_distributed_to_nominators: I96F32 = I96F32::from_num(0.0);
        let hotkey_dynamic: I96F32 =I96F32::from_num( Self::get_dynamic_for_hotkey( hotkey ) );
        let hotkey_alpha: I96F32 = I96F32::from_num( Self::get_alpha_for_hotkey_on_subnet( hotkey, netuid ) );
        let alpha_emission: I96F32 = I96F32::from_num( emission ).saturating_mul( I96F32::from_num(0.5) );
        let dynamic_emission: I96F32 = I96F32::from_num( emission ).saturating_mul( I96F32::from_num(0.5) );

        // Iterate over all nominators to this hotkey.
        for (nominator, _) in Stake::<T>::iter_prefix(hotkey) {

            // Get the nominator alpha
            let nominator_alpha: I96F32 = I96F32::from_num(Alpha::<T>::get( (&hotkey, nominator.clone(), netuid) ));
            let nominator_dynamic: I96F32 = I96F32::from_num(Self::get_dynamic_for_hotkey_and_coldkey( hotkey, &nominator ));  

            // Compute contributions to nominators and alpha holders.
            let nominator_emission_from_alpha: I96F32 = alpha_emission.saturating_mul(nominator_alpha).saturating_div(hotkey_alpha);
            let nominator_emission_from_dynamic: I96F32 = dynamic_emission.saturating_mul(nominator_dynamic).saturating_div(hotkey_dynamic);
            let nominator_emission_total: I96F32 = nominator_emission_from_alpha.saturating_add(nominator_emission_from_dynamic);

            // Increment the nominator's account.
            Alpha::<T>::insert(
                (&hotkey, nominator.clone(), netuid),
                Alpha::<T>::get(( &hotkey, nominator.clone(), netuid) ).saturating_add( nominator_emission_total.to_num::<u64>() ),
            );

            // Add the nominator's emission to the total.
            total_distributed_to_nominators = total_distributed_to_nominators.saturating_add( nominator_emission_total );
        }

        // Return the total emission distributed.
        total_distributed_to_nominators.to_num::<u64>()
    }


    /// Calculates the weighted combination between alpha and dynamic tao for hotkeys on a subnet.
    ///
    /// # Arguments
    /// * `netuid` - Network unique identifier specifying the network context over which we will determine the stake weight.
    ///
    /// # Returns
    /// * `Vec<I32F32>` - The stake weights for each hotkey on the network.
    ///
    pub fn distribute_to_parents( hotkey: &T::AccountId, netuid: u16, emission: u64 ) -> u64 {

        // Get alpha emission due to each parent from dynamic contributions.
        let mut total_distributed_to_parents: I96F32 = I96F32::from_num(0.0);
        let alpha_emission: I96F32 = I96F32::from_num( emission ).saturating_mul( I96F32::from_num(0.5) );
        let dynamic_emission: I96F32 = I96F32::from_num( emission ).saturating_mul( I96F32::from_num(0.5) );
        let hotkey_alpha: I96F32 = I96F32::from_num( Self::get_alpha_for_hotkey_on_subnet( hotkey, netuid ) );
        let hotkey_dynamic: I96F32 = I96F32::from_num( Self::get_dynamic_for_hotkey( hotkey ) );
        // TODO add the last stake increase term here.
        for (proportion, parent) in Self::get_parents(hotkey, netuid) {
            // Proportion from parent.
            let parent_proportion: I96F32 = I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));

            // Compute dynamic proportion due.
            let parent_dynamic_tao: I96F32 = I96F32::from_num( Self::get_dynamic_for_hotkey( &parent ) );
            let parent_dynamic_contribution: I96F32 = parent_dynamic_tao.saturating_mul(parent_proportion);
            let parent_dynamic_proportion: I96F32 = parent_dynamic_contribution.saturating_div(hotkey_dynamic);
            let parent_emission_from_dynamic: I96F32 = parent_dynamic_proportion.saturating_mul(I96F32::from_num(dynamic_emission));

            // Compute alpha proportion due.
            let parent_alpha: I96F32 = I96F32::from_num( Self::get_alpha_for_hotkey_on_subnet( &parent, netuid ) );
            let parent_alpha_contribution: I96F32 = parent_alpha.saturating_mul(parent_proportion);
            let parent_alpha_proportion: I96F32 = parent_alpha_contribution.saturating_div( hotkey_alpha );
            let parent_emission_from_alpha: I96F32 = parent_alpha_proportion.saturating_mul(I96F32::from_num(alpha_emission));

            // Compute total due to parent.
            let parent_emission_total: I96F32 = parent_emission_from_dynamic.saturating_add(parent_emission_from_alpha);
            PendingdHotkeyEmissionOnNetuid::<T>::mutate( parent, netuid, |parent_accumulated| {
                *parent_accumulated = parent_accumulated.saturating_add( parent_emission_total.to_num::<u64>() )
            });
            // Account for total distributed.
            total_distributed_to_parents = total_distributed_to_parents.saturating_add( parent_emission_total );
        }

        // Return total distributed.
        total_distributed_to_parents.to_num::<u64>()
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
            alpha_stake[*uid_i as usize] = I64F64::from_num(Self::get_alpha_stake_for_hotkey_on_subnet(hotkey, netuid));
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
        let ratio: I64F64 = I64F64::from_num(0.5);
        let stake_weights: Vec<I64F64> = alpha_stake
            .iter()
            .zip(dynamic_tao_stake.iter())
            .map(|(alpha, dynamic)| (I64F64::from_num(1.0) - ratio) * (*alpha) + ratio * (*dynamic))
            .collect();

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
        let initial_dynamic_tao: u64 = Self::get_dynamic_for_hotkey( hotkey );
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
            let parent_dynamic_tao: u64 = Self::get_dynamic_for_hotkey( &parent );
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
    pub fn get_alpha_stake_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {

        // Retrieve the initial total stake for the hotkey without any child/parent adjustments.
        let initial_alpha: u64 = Self::get_alpha_for_hotkey_on_subnet(hotkey, netuid);
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
            let parent_alpha: u64 = Self::get_alpha_for_hotkey_on_subnet( &parent, netuid );
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
    pub fn get_dynamic_for_hotkey_and_coldkey( hotkey: &T::AccountId, coldkey: &T::AccountId ) -> u64 {
        // Initialize the total tao equivalent to zero.
        let total_tao_equivalent: I96F32 = I96F32::from_num( 0 );
        // Iterate over all subnet netuids.
        for netuid in Self::get_all_subnet_netuids() {
            // Accumulate the dynamic value for the hotkey and coldkey on each subnet.
            total_tao_equivalent.saturating_add(I96F32::from_num(Self::get_dynamic_for_hotkey_and_coldey_on_subnet( hotkey, coldkey, netuid )));
        }
        // Return the total tao equivalent as a u64 value.
        total_tao_equivalent.to_num::<u64>()
    }

    /// Retrieves the dynamic value for a given hotkey across all subnets.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    ///
    /// # Returns
    /// * `u64` - The total dynamic value for the hotkey across all subnets.
    pub fn get_dynamic_for_hotkey( hotkey: &T::AccountId ) -> u64 {
        // Initialize the total tao equivalent to zero.
        let total_tao_equivalent: I96F32 = I96F32::from_num( 0 );
        // Iterate over all subnet netuids.
        for netuid in Self::get_all_subnet_netuids() {
            // Accumulate the dynamic value for the hotkey on each subnet.
            total_tao_equivalent.saturating_add(I96F32::from_num(Self::get_dynamic_for_hotkey_on_subnet( hotkey, netuid )));
        }
        // Return the total tao equivalent as a u64 value.
        total_tao_equivalent.to_num::<u64>()
    }

    /// Retrieves the dynamic value for a given hotkey on a specific subnet.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `u64` - The dynamic value for the hotkey on the specified subnet.
    pub fn get_dynamic_for_hotkey_on_subnet( hotkey: &T::AccountId, netuid: u16 ) -> u64 {
        // Get the hotkey's alpha value on this subnet.
        let alpha: u64 = Self::get_alpha_for_hotkey_on_subnet( hotkey, netuid );
        // Convert the alpha value to dynamic value and return it.
        return Self::alpha_to_dynamic( alpha, netuid );
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
    pub fn get_dynamic_for_hotkey_and_coldey_on_subnet( hotkey: &T::AccountId, coldkey: &T::AccountId, netuid: u16 ) -> u64 {
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
        if mechid == 2 { 
            // Dynamic
            let alpha_in: I96F32 = I96F32::from_num( alpha );

            let mechid: u16 = SubnetMechanism::<T>::get( netuid );
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
        if mechid == 2 { 
            // Dynamic
            let total_subnet_tao: u64 = SubnetTAO::<T>::get( netuid );
            let total_mechanism_tao: u64 = Self::get_total_mechanism_tao( SubnetMechanism::<T>::get( netuid ) );
            alpha = amount_tao_staked * ((total_mechanism_tao + amount_tao_staked) / (total_subnet_tao + amount_tao_staked));
            return alpha;
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
    pub fn get_alpha_for_hotkey_on_subnet( hotkey: &T::AccountId, netuid: u16 ) -> u64 {
        // Return the alpha this hotkey owns on this subnet.
        TotalHotkeyAlpha::<T>::get( hotkey, netuid )
    }



}