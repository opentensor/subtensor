use super::*;
use crate::math::*;
use frame_support::IterableStorageDoubleMap;
use sp_std::vec;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

impl<T: Config> Pallet<T> {

    // Coinbase
    // Performs a four part emission distribution process:
    //  1. coinbase --> root() --> subnet_block_emission: Compute the block-wise emission per subnet.
    //  2. subnet_block_emission --> subnet_pending_emission: accumulate the subnet block emission as subnet pending emission.
    //  3. subnet_pending_emission --> epoch() --> hotkey_pending_emission: distributes subnet emission through the epoch onto hotkey accounts.
    //  4. hotkey_pending_emission --> nominators: distributes hotkey emission onto nominators.
    pub fn coinbase() {

        // --- 0. Get current block.
        let block: u64 = Self::get_current_block_as_u64();

        // --- 1. Get all netuids.
        let subnets: Vec<u16> = Self::get_all_netuids();

        // --- 2. Run the epoch function which computes the block emission for each subnet.
        // coinbase --> root() --> subnet_block_emission
        match Self::root_epoch(block_number) { Ok(_) => (), Err(e) => {log::trace!("Error while running root epoch: {:?}", e);}}

        // --- 3. Drains the subnet block emission and accumulates it as subnet emission, which increases until the tempo is reached in #4.
        // subnet_blockwise_emission -> subnet_pending_emission
        for netuid in subnets {

            // --- 3.1 Get the network's block-wise emission amount.
            // This value is newly minted TAO which has not reached staking accounts yet.
            let subnet_blockwise_emission: u64 = Self::get_subnet_block_emission( netuid );

            // --- 3.2 Accumulate the subnet emission on the subnet.
            Self::accumulate_subnet_emission( netuid, subnet_blockwise_emission );
        }

        // --- 4. Drains the accumulated subnet emissions and passes it through the the epoch(). 
        // The emission from the epoch() 
        // Then accumulates on the  function and then accumulates the emission on the hotkeys.
        // Before accumulating on the hotkeys the function re-distributes the emission towards hotkey parents.
        // subnet_emission --> epoch() --> hotkey_emission --> (hotkey + parent hotkeys)
        for netuid in subnets {

            // 4.1 Check to see if the subnet should run its epoch.
            if Self::should_run_epoch( netuid, block ) {

                // 4.2 Drain the subnet emission.
                let subnet_emission: u64 = Self::drain_subnet_emission( netuid );

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
        for (index, (hotkey_i, emission_i)) in PendingdHotkeyEmission::<T>::iter().enumerate() {

            // --- 5.1 Check if we should drain the hotkey emission on this block.
            if Self::should_drain_hotkey( hotkey_i, index, block ) {

                // --- 5.2 Drain the hotkey emission and distribute it to nominators.
                Self::drain_hotkey_emission( hotkey_i, emission_i );

                // --- 5.3 Increase total issuance
                Self::increase_issuance( emission_i );
            }
        }

    }

    pub fn increase_issuance( emission: u64 ) {
        TotalIssuance::<T>::put( TotalIssuance::<T>::get().saturating_add( emission ) );
    }

    pub fn should_drain_hotkey( hotkey_i, index, block ){
        return index % block != 7200
    }

    pub fn should_run_epoch( netuid: u16 ) {
        return Self::blocks_until_next_epoch( netuid, Self::get_tempo( netuid ), Self::get_current_block_as_u64() ) == 0
    }

    pub fn get_subnet_block_emission(netuid: u16) -> u64 {
        EmissionValues::<T>::get(netuid)
    }

    pub fn accumulate_subnet_emission(netuid: u16, blockwise_emission: u64 ) {
        // --- 1 Increase the accumulated pending emission for this network with the blockwise value.
        PendingEmission::<T>::mutate( netuid, |emission| *emission += blockwise_emission);

        // --- 2. Here we actually increase the issuance of the token since it exists in a counter.
        TotalIssuance::<T>::put(TotalIssuance::<T>::get().saturating_add( blockwise_emission ));
    }

    pub fn drain_subnet_emission( netuid: u16 ) -> u64 {
        let emission:u64 = PendingEmission::<T>::get( netuid );
        PendingEmission::<T>::insert( netuid, 0 );
        emssion
    }

    /// Accumulates the mining and validator emissions on a hotkey and distributes the validator emission among its parents.
    ///
    /// This function is responsible for distributing the mining and validator emissions associated with a hotkey.
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
    /// # Panics
    /// This function may panic if the total stake calculation results in a zero, which would lead to a division by zero error during emission distribution.
    pub fn accumulate_hotkey_emission( hotkey: T::AccountId, netuid: u16, mining_emission: u64, validator_emission: u64 ) {

        // --- 1 Get the the hotkey total stake with parent additions and child reductions. 
        // Parents contribute stake and children remove a proportion of the hotkey stake.
        let total_hotkey_stake: u64 = Self::get_stake_with_children_and_parents( hotkey, netuid );

        // --- 2 Get this hotkey's parents.
        let parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get( hotkey, netuid );

        // --- 3 Remainder counter, decrements emissions as we pay out parents.
        let mut remaining_validator_emission: u64 = validator_emission;

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

        // --- 5 Add remaining validator emission + mining emission to hotkey
        PendingdHotkeyEmission::<T>::mutate( hotkey, |hotkey_accumulated| *hotkey_accumulated += remaining_validator_emission + mining_emission );
    }

    //. --- 4. Drains the accumulated hotkey emissions through to the nominators. The hotkey takes a proportion of the emission.
    /// The remainder is drained through to the nominators keeping track of the last stake increase event to ensure that the hotkey does not 
    /// gain more emission than it's stake since the last drain.
    /// hotkeys --> nominators.
    ///
    /// This function iterates over all hotkeys that have accumulated emissions and performs several operations:
    /// 1. It resets the accumulated emissions for each hotkey to zero.
    /// 2. It checks if the hotkey's emissions should be drained based on a modulo operation with a fixed interval (currently hardcoded as 7200).
    /// 3. If it's time to drain, it updates the last emission drain time to the current block number.
    /// 4. It calculates the total stake for the hotkey and determines the hotkey's own take from the emissions based on its delegation status.
    /// 5. It then calculates the remaining emissions after the hotkey's take and distributes this remaining amount proportionally among the hotkey's nominators.
    /// 6. Each nominator's share of the emissions is added to their stake, but only if their stake was not manually increased since the last emission drain.
    /// 7. Finally, the hotkey's own take and any undistributed emissions are added to the hotkey's total stake.
    ///
    /// This function ensures that emissions are fairly distributed according to stake proportions and delegation agreements, and it updates the necessary records to reflect these changes.
    pub fn drain_hotkey_emission( hotkey: T::AccountId, emission: u64, block_number: u64 ) {

        // --- 1.1 Drain the hotkey emission.
        PendingdHotkeyEmission::<T>::insert( hotkey, 0 );

        // --- 1.0 Get the last time we drained this hotkey's emissions.
        let last_hotkey_emission_drain: u64 = LastHotkeyEmissionDrain::<T>::get( hotkey );

        // --- 1.2 We are draining the hotkey emission now, so we set the new block value here.
        LastHotkeyEmissionDrain::<T>::insert( hotkey, block_number );

        // --- 1.3 Get hotkey total stake from all nominations.
        let total_hotkey_stake: u64 = Self::get_total_stake_for_hotkey( hotkey );

        // --- 1.4 Calculate emission take for hotkey.
        let hotkey_emission_take: u64 = Self::calculate_delegate_proportional_take( hotkey, emission );

        // --- 1.5 Compute remaining emission after hotkey take.
        let emission_minus_take: u64 = emission_i - hotkey_emission_take;

        // --- 1.6 Remove emission take from remaining emission
        let mut emission_remainder: u64 = emission;

        // --- 1.7 Iterate each nominator.
        for ( coldkey, stake ) in <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix( hotkey ) {

            // --- 1.7.0 Check if the hot cold was manually increased by the user since the last time the hotkey drained emissions.
            // In this case we will skip over the hot cold pair and they will not attain their emission proportion.
            if LastAddStakeIncrease::<T>::get( hotkey, coldkey ) > last_hotkey_emission_drain { continue; }

            // --- 1.8.1 Calculate the nominator emission proportion from emission_minus_take_i
            let nominator_emission: u64 = Self::calculate_stake_proportional_emission( stake, total_hotkey_stake, emission_minus_take );

            // --- 1.9.2 Increase the stake for the nominator.
            Self::increase_stake_on_coldkey_hotkey_account( &coldkey, hotkey, nominator_emission );
        }

        // --- 1.8. Finally add the stake to the hotkey itself including its take and the emission remainder.
        Self::increase_stake_on_hotkey_account( hotkey, hotkey_emission_take + emission_remainder );
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