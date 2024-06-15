use super::*;
use crate::math::*;
use frame_support::IterableStorageDoubleMap;
use sp_std::vec;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

impl<T: Config> Pallet<T> {


    pub fn coinbase() {

        // Appends the emission for a block to each network's pending emission.
        Self::add_pending_subnet_emission();

        // Distribute pending emission into accumulated hotkey emission.
        Self::accumulate_hotkey_emission();

        // Drain the accumulated hotkey emissions through to nominators.
        Self::drain_accumulated_hotkey_emissions();
    }

    // Appends the emission for a block to each network's pending emission.
    // coinbase --> emission --> pending_emission
    pub fn add_pending_subnet_emission() {

        // --- 1. For each network append emission from coinbase.
        for (netuid_i, _) in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {

            // --- 1.1 Skip the root network or subnets with registrations turned off.
            // These networks burn their emission here.
            if netuid_i == Self::get_root_netuid() || !Self::is_registration_allowed( netuid_i ) { continue; }

            // --- 1.2 Get the network's emission value.
            let block_emission: u64 = Self::get_subnet_emission_value( netuid_i );

            // --- 1.3 Get current pending emission.
            let current_pending_emission: u64 = PendingEmission::<T>::get( netuid_i );

            // --- 1.4 Increase pending emission.
            let new_pending_emission: u64 = current_pending_emission + block_emission;

            // --- 1.5 Insert new pending emission.
            PendingEmission::<T>::insert( netuid_i, new_pending_emission );

            // --- 1.6. Here we actually increase the issuance of the token.
            TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add( emission_sum ));
        }
    }

     /// Helper function which returns the number of blocks remaining before we will run the epoch on this
    /// network. Networks run their epoch when (block_number + netuid + 1 ) % (tempo + 1) = 0
    ///
    pub fn blocks_until_next_epoch(netuid: u16, tempo: u16, block_number: u64) -> u64 {
        // tempo | netuid | # first epoch block
        //   1        0               0
        //   1        1               1
        //   2        0               1
        //   2        1               0
        //   100      0              99
        //   100      1              98
        // Special case: tempo = 0, the network never runs.
        if tempo == 0 {
            return 1000;
        }
        tempo as u64 - (block_number + netuid as u64 + 1) % (tempo as u64 + 1)
    }

    // Distribute pending emission onto accumualted hotkey emission.
    // coinbase --> emission --> pending_emission --> accumulated hotkey emission
    pub fn accumulate_hotkey_emission() {

        // --- 1. For each network append emission from coinbase.
        for (netuid_i, tempo_i) in <Tempo<T> as IterableStorageMap<u16, u16>>::iter() {

            // --- 1.1 Skip networks that do not have an epoch.
            if Self::blocks_until_next_epoch( netuid_i, tempo_i, Self::get_current_block_as_u64() ) == 0 {

                // --- 1. Get all pending emission associated with this network.
                let pending_subnet_emissions: u64 = PendingEmission::<T>::get( netuid_i );

                // --- 2. Remove pending emission associated with this network.
                PendingEmission::<T>::insert( netuid_i, 0 );

                // --- 3. Run the epoch for this network.
                let emission_per_hotkey: Vec<(T::AccountId, u64, u64)> = Self::epoch( netuid_i, pending_subnet_emissions );

                // --- 4. Check that the emission does not exceed the input pending_subnet_emission.
                let emission_sum: u128 = emission_per_hotkey.iter().map(|(_, se, ve)| *ve as u128 + *se as u128).sum();

                // --- 4.1 If the emission_sum exceeds the emission, we skip this network.
                if emission_sum > emission_to_drain as u128 { continue; } 

                // --- 5. Accumulate the hotkey emission for each hotkey.
                for ( hotkey_j, new_mining_emission_j, new_validator_emission_j ) in emission_per_hotkey.iter() {

                    // --- 5.1 Accumulate the hotkey mining and validator emission but first distribute the validator emission amongst parents.
                    Self::accumulate_hotkey_emission_for_netuid_and_tuples( hotkey_j, netuid_i, new_mining_emission_j, new_validator_emission_j );
                }

            }
        }
    }

    // Accumulate the hotkey mining and validator emission but first distribute the validator emission amongst parents.
    pub fn accumulate_hotkey_emission_for_netuid_and_tuples( hotkey: T::AccountId, netuid: u16, mining_emission: u64, validator_emission: u64 ) {

        // --- 1 First get the total amount of stake for this hotkey after child and parent removal and additions.
        let total_hotkey_stake: u64 = Self::get_stake_with_children_and_parents( hotkey, netuid );

        // --- 2. Get this hotkey's parents.
        let parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get( hotkey, netuid );

        // --- 3.0 Record remaining validator emission for this hotkey, decrements as we pay out parents.
        let mut remaining_validator_emission: u64 = validator_emission;

        // --- 3. For each parent, determine the amount of stake added to this key.
        for (proportion, parent) in parents {

            // --- 3.1 Retrieve the parent's stake.
            let parent_stake: u64 = Self::get_total_stake_for_hotkey( parent );

            // --- 3.2 Calculate the stake proportion received from the parent.
            let stake_from_parent: I96F32 = I96F32::from_num( parent_stake ) * I96F32::from_num( proportion ) / I96F32::from_num( u64::MAX );

            // --- 3.3 Compute parent proportion to hotkey stake. The amount due to the parent via being a parent of the hotkey.
            let proportion_from_parent: I96F32 = stake_from_parent / I96F32::from_num( total_hotkey_stake );

            // --- 3.4 Compute parent emission proportion. 
            let parent_validator_emission: u64 = ( proportion_from_parent * I96F32::from_num( validator_emission ) ).to_num::<u64>();  

            // --- 3.5. Accumulate validator emission for the parent.
            AccumulatedHotkeyEmission::<T>::mutate( parent, |parent_accumulated| *parent_accumulated += parent_validator_emission );

            // --- 3.6. Decrement remaining validator emission for this hotkey.
            remaining_validator_emission -= parent_validator_emission;
        }

        // --- 4 Add remaining validator emission + mining emission to hotkey
        AccumulatedHotkeyEmission::<T>::mutate( hotkey, |hotkey_accumulated| *hotkey_accumulated += remaining_validator_emission  );

        // --- 5. Directly increase the stake amount on the hotkey from the mining emission.
        Self::increase_stake_on_hotkey_account( hotkey, mining_emission );
    }

    // Drain the accumulated hotkey emissions through delegations.
    pub fn drain_accumulated_hotkey_emissions() {

        // --- 0. Get the current block number 
        let current_block_number: u64 = Self::get_current_block_as_u64();

        // --- 1. Iterate each hotkey and drain its accumulated hotkey emissions.
        for (index, (hotkey_i, emission_i)) in AccumulatedHotkeyEmission::<T>::iter().enumerate() {

            // --- 1.0 Get the last time we drained this hotkey's emissions.
            let last_hotkey_emission_drain: u64 = LastHotkeyEmissionDrain::<T>::get( hotkey_i );

            // --- 1.1 Check if it is time for us to drain this hotkey's emissions.
            // TODO: make 7200 a parameter.
            if index % 7200 != 0 { continue; } // Only drain an account once per day.

            // --- 1.2 We are draining the hotkey emission now, so we set the new block value here.
            LastHotkeyEmissionDrain::<T>::insert( hotkey_i, current_block_number );

            // --- 1.3 Get hotkey total stake from all nominations.
            let total_hotkey_stake_i: u64 = Self::get_total_stake_for_hotkey( hotkey_i );

            // --- 1.4 Calculate emission take for hotkey.
            let hotkey_emission_take_i: u64 = Self::calculate_delegate_proportional_take( hotkey_i, emission_i );

            // --- 1.5 Compute remaining emission after hotkey take.
            let emission_minus_take_i: u64 = emission_i - hotkey_emission_take_i;

            // --- 1.6 Remove emission take from remaining emission
            let mut emission_remainder_i: u64 = emission_i;

            // --- 1.7 Iterate each nominator.
            for ( coldkey_j, stake_j ) in <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix( hotkey ) {

                // --- 1.7.0 Check if the hot cold was manually increased by the user since the last time the hotkey drained emissions.
                // In this case we will skip over the hot cold pair and they will not attain their emission proportion.
                if LastAddStakeIncrease::<T>::get( hotkey_i, coldkey_j ) > last_hotkey_emission_drain { continue; }

                // --- 1.8.1 Calculate the nominator emission proportion from emission_minus_take_i
                let nominator_emission_j: u64 = Self::calculate_stake_proportional_emission( stake_i, total_hotkey_stake_i, emission_minus_take_i );

                // --- 1.9.2 Increase the stake for the nominator.
                Self::increase_stake_on_coldkey_hotkey_account( &coldkey_j, hotkey_i, nominator_emission_j );
            }

            // --- 1.8. Finally add the stake to the hotkey itself including its take and the emission remainder.
            Self::increase_stake_on_hotkey_account( hotkey_i, hotkey_emission_take_i + emission_remainder_i );
        }
    }

    /// Returns emission awarded to a hotkey as a function of its proportion of the total stake.
    ///
    pub fn calculate_stake_proportional_emission(
        stake: u64,
        total_stake: u64,
        emission: u64,
    ) -> u64 {
        if total_stake == 0 {
            return 0;
        };
        let stake_proportion: I64F64 = I64F64::from_num(stake) / I64F64::from_num(total_stake);
        let proportional_emission: I64F64 = I64F64::from_num(emission) * stake_proportion;
        proportional_emission.to_num::<u64>()
    }

    /// Returns the delegated stake 'take' assigned to this key. (If exists, otherwise 0)
    ///
    pub fn calculate_delegate_proportional_take(hotkey: &T::AccountId, emission: u64) -> u64 {
        if Self::hotkey_is_delegate(hotkey) {
            let take_proportion: I64F64 =
                I64F64::from_num(Delegates::<T>::get(hotkey)) / I64F64::from_num(u16::MAX);
            let take_emission: I64F64 = take_proportion * I64F64::from_num(emission);
            take_emission.to_num::<u64>()
        } else {
            0
        }
    }

}