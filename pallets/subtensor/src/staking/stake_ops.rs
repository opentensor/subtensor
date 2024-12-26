use super::*;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {


    /// Swaps TAO for the alpha token on the subnet.
    ///
    /// Updates TaoIn, AlphaIn, and AlphaOut
    pub fn swap_tao_for_alpha( netuid: u16, tao: u64 ) -> u64 {
        // Step 1: Get the mechanism type for the subnet (0 for Stable, 1 for Dynamic)
        let mechanism_id: u16 = SubnetMechanism::<T>::get(netuid);
        // Step 2: Initialized vars.
        let alpha: I96F32 = if mechanism_id == 1 {
            // Step 3.a.1: Dynamic mechanism calculations
            let tao_reserves: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(netuid));
            let alpha_reserves: I96F32 = I96F32::from_num(SubnetAlphaIn::<T>::get(netuid));
            // Step 3.a.2: Compute constant product k = alpha * tao
            let k: I96F32 = alpha_reserves.saturating_mul(tao_reserves);
            // Step 3.a.3: Calculate alpha staked using the constant product formula
            // alpha_stake_recieved = current_alpha - (k / (current_tao + new_tao))
            alpha_reserves.saturating_sub(
                k.checked_div(tao_reserves.saturating_add(I96F32::from_num(tao)))
                    .unwrap_or(I96F32::from_num(0)),
            )
        } else {
            // Step 3.b.1: Stable mechanism, just return the value 1:1
            I96F32::from_num( tao ) 
        };
        // Step 4. Decrease Alpha reserves.
        SubnetAlphaIn::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub( alpha.to_num::<u64>() );
        });
        // Step 5: Increase Alpha outstanding.
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_add( alpha.to_num::<u64>() );
        });
        // Step 6: Increase Tao reserves.
        SubnetTAO::<T>::mutate(netuid, |total| {
            *total = total.saturating_add( tao );
        });
        // Step 7: Increase Total Tao reserves.
        TotalStake::<T>::mutate(|total| {
            *total = total.saturating_add( tao );
        });
        // Step 8. Return the alpha received.
        alpha.to_num::<u64>()
    }

    /// Swaps a subnet's Alpba token for TAO.
    ///
    /// Updates TaoIn, AlphaIn, and AlphaOut
    pub fn swap_alpha_for_tao( netuid: u16, alpha: u64 ) -> u64 {
        // Step 1: Get the mechanism type for the subnet (0 for Stable, 1 for Dynamic)
        let mechanism_id: u16 = SubnetMechanism::<T>::get(netuid);
        // Step 2: Swap alpha and attain tao
        let tao: I96F32 = if mechanism_id == 1 {
            // Step 3.a.1: Dynamic mechanism calculations
            let tao_reserves: I96F32 = I96F32::from_num(SubnetTAO::<T>::get(netuid));
            let alpha_reserves: I96F32 = I96F32::from_num(SubnetAlphaIn::<T>::get(netuid));
            // Step 3.a.2: Compute constant product k = alpha * tao
            let k: I96F32 = alpha_reserves.saturating_mul(tao_reserves);
            // Step 3.a.3: Calculate alpha staked using the constant product formula
            // tao_recieved = tao_reserves - (k / (alpha_reserves + new_tao))
            tao_reserves.saturating_sub(
                k.checked_div(alpha_reserves.saturating_add(I96F32::from_num( alpha )))
                    .unwrap_or(I96F32::from_num(0)),
            )
        } else {
            // Step 3.b.1: Stable mechanism, just return the value 1:1
            I96F32::from_num( alpha )
        };        
        // Step 4: Increase Alpha reserves.
        SubnetAlphaIn::<T>::mutate(netuid, |total| {
            *total = total.saturating_add( alpha );
        });
        // Step 5: Decrease Alpha outstanding.
        SubnetAlphaOut::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub( alpha ) ;
        });
        // Step 6: Decrease tao reserves.
        SubnetTAO::<T>::mutate(netuid, |total| {
            *total = total.saturating_sub( tao.to_num::<u64>() );
        });
        // Step 7: Reduce total TAO reserves.
        TotalStake::<T>::mutate(|total| {
            *total = total.saturating_sub( tao.to_num::<u64>() );
        });        
        // Step 8. Return the tao received.
        tao.to_num::<u64>()
    }

    /// Unstakes alpha from a subnet for a given hotkey and coldkey pair.
    ///
    /// We update the pools associated with a subnet as well as update hotkey alpha shares.
    pub fn unstake_from_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        alpha: u64,
    ) -> u64 {
       
        // Step 1: Swap the alpha for TAO.
        let tao: u64 = Self::swap_alpha_for_tao( netuid, alpha );

        // Step 2: Decrease alpha on subneet
        Self::decrease_stake_for_hotkey_and_coldkey_on_subnet( hotkey, coldkey, netuid, alpha );

        // Step 3: Update StakingHotkeys if the hotkey's total alpha, across all subnets, is zero
        // TODO const: fix.
        // if Self::get_stake(hotkey, coldkey) == 0 {
        //     StakingHotkeys::<T>::mutate(coldkey, |hotkeys| {
        //         hotkeys.retain(|k| k != hotkey);
        //     });
        // }

        // Step 4. Deposit the unstaking event.
        Self::deposit_event(Event::StakeRemoved(
            coldkey.clone(),
            hotkey.clone(),
            tao,
            alpha,
            netuid,
        ));

        // Step 5: Return the amount of TAO unstaked.
        tao
    }

    /// Stakes TAO into a subnet for a given hotkey and coldkey pair.
    ///
    /// We update the pools associated with a subnet as well as update hotkey alpha shares.
    pub fn stake_into_subnet(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
        netuid: u16,
        tao: u64,
    ) -> u64 {
        
        // Step 1. Swap the tao to alpha.
        let alpha: u64 = Self::swap_tao_for_alpha( netuid, tao );

        // Step 2: Increase the alpha on the hotkey account.
        Self::increase_stake_for_hotkey_and_coldkey_on_subnet( hotkey, coldkey, netuid, alpha );

        // Step 4: Update the list of hotkeys staking for this coldkey
        let mut staking_hotkeys = StakingHotkeys::<T>::get(coldkey);
        if !staking_hotkeys.contains(hotkey) {
            staking_hotkeys.push(hotkey.clone());
            StakingHotkeys::<T>::insert(coldkey, staking_hotkeys.clone());
        }

        // Step 5: Deposit the staking event.
        Self::deposit_event(Event::StakeAdded(
            coldkey.clone(),
            hotkey.clone(),
            tao,
            alpha,
            netuid,
        ));

        // Step 6: Return the amount of alpha staked
        alpha
    }

    
}
