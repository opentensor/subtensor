use super::*;
use crate::math::*;
use frame_support::IterableStorageDoubleMap;
use sp_std::vec;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

impl<T: Config> Pallet<T> {

    /// The `coinbase` function performs a four-part emission distribution process involving
    /// subnets, epochs, hotkeys, and nominators.
    // It is divided into several steps, each handling a specific part of the distribution:

    // Step 1: Compute the block-wise emission for each subnet.
    // This involves calculating how much (TAO) should be emitted into each subnet using the 
    // root epoch function. 
    
    // Step 2: Accumulate the subnet block emission.
    // After calculating the block-wise emission, these values are accumulated to keep track
    // of how much each subnet should emit before the next distribution phase. This accumulation
    // is a running total that gets updated each block.
    
    // Step 3: Distribute the accumulated emissions through epochs.
    // Subnets periodically distribute their accumulated emissions to hotkeys (active validators/miners)
    // in the network on a `tempo` --- the time between an epoch. This step runs Yuma consensus to 
    // determine how emissions are split among hotkeys based on their contributions and roles.
    // The accumulation of hotkey emissions is done through the `accumulate_hotkey_emission` function.
    // The accumulate splits the rewards for a hotkey amongst itself and its `parents`. The parents are 
    // the hotkeys that are delegating their stake to the hotkey. 
    
    // Step 4: Further distribute emissions from hotkeys to nominators.
    // Finally, the emissions received by hotkeys are further distributed to their nominators,
    // who are stakeholders that support the hotkeys.   
    pub fn coinbase() {

        // --- 0. Get current block.
        let current_block: u64 = Self::get_current_block_as_u64();

        // --- 1. Get all netuids.
        let subnets: Vec<u16> = Self::get_all_netuids();

        // --- 2. Run the root epoch function which computes the block emission for each subnet.
        // coinbase --> root() --> subnet_block_emission
        match Self::root_epoch(block_number) { Ok(_) => (), Err(e) => {log::trace!("Error while running root epoch: {:?}", e);}}

        // --- 3. Drains the subnet block emission and accumulates it as subnet emission, which increases until the tempo is reached in #4.
        // subnet_blockwise_emission -> subnet_pending_emission
        for netuid in subnets {

            // --- 3.1 Get the network's block-wise emission amount.
            // This value is newly minted TAO which has not reached staking accounts yet.
            let subnet_blockwise_emission: u64 = EmissionValues::<T>::get( netuid );

            // --- 3.2 Accumulate the subnet emission on the subnet.
            PendingEmission::<T>::mutate( netuid, |subnet_emission| *subnet_emission += subnet_blockwise_emission);
        }

        // --- 4. Drains the accumulated subnet emissions, passes them through the epoch(). 
        // Before accumulating on the hotkeys the function re-distributes the emission towards hotkey parents.
        // subnet_emission --> epoch() --> hotkey_emission --> (hotkey + parent hotkeys)
        for netuid in subnets {

            // 4.1 Check to see if the subnet should run its epoch.
            if Self::should_run_epoch( netuid, current_block ) {

                // 4.2 Drain the subnet emission.
                let subnet_emission: u64 = PendingEmission::<T>::get( netuid );
                PendingEmission::<T>::insert( netuid, 0 );

                // 4.3 Pass emission through epoch() --> hotkey emission.
                let hotkey_emission: Vec<(T::AccountId, u64, u64)> = Self::epoch( netuid, subnet_emission );

                // 4.3 Drain the subnet emission through the epoch()
                for (hotkey, mining_emission, validator_emission) in hotkey_emission {

                    // 4.4 Accumulate the emission on the hotkey and parent hotkeys.
                    Self::accumulate_hotkey_emission( hotkey, netuid, mining_emission, validator_emission );
                }
            }
        }

        // --- 5. Drains the accumulated hotkey emissions through to the nominators. 
        /// The hotkey takes a proportion of the emission, the remainder is drained through to the nominators.
        // We keeping track of the last stake increase event for accounting purposes.
        // hotkeys --> nominators.
        for (index, ( hotkey, hotkey_emission )) in PendingdHotkeyEmission::<T>::iter().enumerate() {

            // --- 5.1 Check if we should drain the hotkey emission on this block.
            if Self::should_drain_hotkey( index, current_block ) {

                // --- 5.2 Drain the hotkey emission and distribute it to nominators.
                Self::drain_hotkey_emission( hotkey, hotkey_emission );

                // --- 5.3 Increase total issuance
                TotalIssuance::<T>::put( TotalIssuance::<T>::get().saturating_add( hotkey_emission ) );
            }
        }

    }

    /// Accumulates the mining and validator emissions on a hotkey and distributes the validator emission among its parents.
    ///
    /// This function is responsible for accumulating the mining and validator emissions associated with a hotkey onto a hotkey.
    /// It first calculates the total stake of the hotkey, considering the stakes contributed by its parents and reduced by its children.
    /// It then retrieves the list of parents of the hotkey and distributes the validator emission proportionally based on the stake contributed by each parent.
    /// The remaining validator emission, after distribution to the parents, along with the mining emission, is then added to the hotkey's own accumulated emission.
    ///
    /// # Arguments
    /// * `hotkey` - The account ID of the hotkey for which emissions are being calculated.
    /// * `netuid` - The unique identifier of the network to which the hotkey belongs.
    /// * `mining_emission` - The amount of mining emission allocated to the hotkey.
    /// * `validator_emission` - The amount of validator emission allocated to the hotkey.
    ///
    pub fn accumulate_hotkey_emission( hotkey: T::AccountId, netuid: u16, mining_emission: u64, validator_emission: u64 ) {

        // --- 1 Get the the hotkey total stake with parent additions and child reductions. 
        // Parents contribute stake and children remove a proportion of the hotkey stake.
        let total_hotkey_stake: u64 = Self::get_stake_with_children_and_parents( hotkey, netuid );

        // --- 2 Get this hotkey's parents.
        let parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get( hotkey, netuid );

        // --- 3 Remainder counter, decrements emissions as we pay out parents.
        let mut remaining_validator_emission: u64 = validator_emission;

        // Ensure the denominator is not zero. Removing this line can cause a panic division by zero.
        if total_hotkey_stake != 0 { 

            // --- 4 For each parent, determine the amount of stake added to this key.
            for (proportion, parent) in parents {

                // --- 4.1 Retrieve the parent's stake. This is the hotkey's raw stake value.
                let parent_stake: u64 = Self::get_total_stake_for_hotkey( parent );

                // --- 4.2 Calculate the stake proportion received from the parent.
                let stake_from_parent: I96F32 = I96F32::from_num( parent_stake ) * ( I96F32::from_num( proportion ) / I96F32::from_num( u64::MAX ) );

                // --- 4.3 Compute parent proportion to hotkey stake. The amount due to the parent via being a parent of the hotkey.
                let proportion_from_parent: I96F32 = stake_from_parent / I96F32::from_num( total_hotkey_stake );

                // --- 4.4 Compute parent emission proportion. 
                let parent_validator_emission: u64 = ( proportion_from_parent * I96F32::from_num( validator_emission ) ).to_num::<u64>();  

                // --- 4.5. Accumulate hotkey emission for the parent.
                PendingdHotkeyEmission::<T>::mutate( parent, |parent_accumulated| *parent_accumulated += parent_validator_emission );

                // --- 4.6. Decrement remaining validator emission for this hotkey.
                remaining_validator_emission -= parent_validator_emission;
            }
        }

        // --- 5 Add remaining validator emission + mining emission to hotkey
        PendingdHotkeyEmission::<T>::mutate( hotkey, |hotkey_accumulated| *hotkey_accumulated += remaining_validator_emission + mining_emission );
    }

    //. --- 4. Drains the accumulated hotkey emission through to the nominators. The hotkey takes a proportion of the emission.
    /// The remainder is drained through to the nominators keeping track of the last stake increase event to ensure that the hotkey does not 
    /// gain more emission than it's stake since the last drain.
    /// hotkeys --> nominators.
    ///
    /// 1. It resets the accumulated emissions for the hotkey to zero.
    /// 4. It calculates the total stake for the hotkey and determines the hotkey's own take from the emissions based on its delegation status.
    /// 5. It then calculates the remaining emissions after the hotkey's take and distributes this remaining amount proportionally among the hotkey's nominators.
    /// 6. Each nominator's share of the emissions is added to their stake, but only if their stake was not manually increased since the last emission drain.
    /// 7. Finally, the hotkey's own take and any undistributed emissions are added to the hotkey's total stake.
    ///
    /// This function ensures that emissions are fairly distributed according to stake proportions and delegation agreements, and it updates the necessary records to reflect these changes.
    pub fn drain_hotkey_emission( hotkey: T::AccountId, emission: u64, block_number: u64 ) {

        // --- 1.0 Drain the hotkey emission.
        PendingdHotkeyEmission::<T>::insert( hotkey, 0 );

        // --- 1.1 Get the last time we drained this hotkey's emissions.
        let last_hotkey_emission_drain: u64 = LastHotkeyEmissionDrain::<T>::get( hotkey );

        // --- 1.2 Set the new block value here.
        LastHotkeyEmissionDrain::<T>::insert( hotkey, block_number );

        // --- 1.3 Get hotkey total stake from all nominations.
        let total_hotkey_stake: u64 = Self::get_total_stake_for_hotkey( hotkey );

        // --- 1.4 Calculate emission take for hotkey.
        let take_proportion: I64F64 = I64F64::from_num( Delegates::<T>::get( hotkey ) ) / I64F64::from_num( u16::MAX );
        let hotkey_take: I64F64 = ( take_proportion * I64F64::from_num( emission ) ).to_num::<u64>();

        // --- 1.5 Compute remaining emission after hotkey take.
        let emission_minus_take: u64 = emission_i - hotkey_take;

        // --- 1.6 Remove emission take from remaining emission
        let mut remainder: u64 = emission_minus_take;

        // --- 1.7 Iterate each nominator.
        for ( nominator, nominator_stake ) in <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix( hotkey ) {

            // --- 1.7.0 Check if the hot cold was manually increased by the user since the last time the hotkey drained emissions.
            // In this case we will skip over the hot cold pair and they will not attain their emission proportion.
            if LastAddStakeIncrease::<T>::get( hotkey, nominator ) > last_hotkey_emission_drain { continue; }

            // --- 1.7.2 Compute this nominator's proportion of the emission.
            let nominator_emission: I64F64 = I64F64::from_num( emission_minus_take ) * ( I64F64::from_num( nominator_stake ) / I64F64::from_num( total_hotkey_stake ) );

            // --- 1.7.2 Increase the stake for the nominator.
            Self::increase_stake_on_coldkey_hotkey_account( &nominator, hotkey, nominator_emission.to_num::<u64>() );

            // --- 1.7.4 Decrement the remainder by the nominator's emission.
            remainder -= nominator_emission.to_num::<u64>();
        }

        // --- 1.8. Finally add the stake to the hotkey itself including its take and the emission remainder.
        Self::increase_stake_on_hotkey_account( hotkey, hotkey_take + remainder );
    }

    ///////////////
    /// Helpers ///
    ///////////////

    /// Determines whether the hotkey emission should be drained based on the current block and index.
    ///
    /// # Arguments
    /// * `hotkey_i` - The hotkey identifier.
    /// * `index` - The index of the hotkey in the iterable storage.
    /// * `block` - The current block number.
    ///
    /// # Returns
    /// * `bool` - True if the hotkey emission should be drained, false otherwise.
    pub fn should_drain_hotkey( index, block ){
        return block % 7200 == index % 7200 // True once per day for each index assumer we run this every block.
    }

    /// Checks if the epoch should run for a given subnet based on the current block.
    ///
    /// # Arguments
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    /// * `bool` - True if the epoch should run, false otherwise.
    pub fn should_run_epoch( netuid: u16 ) {
        return Self::blocks_until_next_epoch( netuid, Self::get_tempo( netuid ), Self::get_current_block_as_u64() ) == 0
    }

    /// Helper function which returns the number of blocks remaining before we will run the epoch on this
    /// network. Networks run their epoch when (block_number + netuid + 1 ) % (tempo + 1) = 0
    /// tempo | netuid | # first epoch block
    ///   1        0               0
    ///   1        1               1
    ///   2        0               1
    ///   2        1               0
    ///   100      0              99
    ///   100      1              98
    /// Special case: tempo = 0, the network never runs.
    ///
    pub fn blocks_until_next_epoch(netuid: u16, tempo: u16, block_number: u64) -> u64 {
        if tempo == 0 { return u64::MAX; }
        tempo as u64 - (block_number + netuid as u64 + 1) % (tempo as u64 + 1)
    }
}