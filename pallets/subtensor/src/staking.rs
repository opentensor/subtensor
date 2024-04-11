use super::*;
use frame_support::{
    storage::IterableStorageDoubleMap,
    traits::{
        tokens::{
            fungible::{Balanced as _, Inspect as _, Mutate as _},
            Fortitude, Precision, Preservation,
        },
        Imbalance,
    },
};
use sp_core::Get;
use substrate_fixed::types::{I64F64};

impl<T: Config> Pallet<T> {
    // ---- The implementation for the extrinsic become_delegate: signals that this hotkey allows delegated stake.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The hotkey we are delegating (must be owned by the coldkey.)
    //
    // 	* 'take' (u16):
    // 		- The stake proportion that this hotkey takes from delegations for subnet ID.
    //
    // # Event:
    // 	* DelegateAdded;
    // 		- On successfully setting a hotkey as a delegate.
    //
    // # Raises:
    // 	* 'NotRegistered':
    // 		- The hotkey we are delegating is not registered on the network.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- The hotkey we are delegating is not owned by the calling coldket.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    pub fn do_become_delegate(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_become_delegate( origin:{:?} hotkey:{:?} )",
            coldkey,
            hotkey
        );

        // --- 2. Ensure we are delegating a known key.
        // --- 2. Ensure we are delegating a known key.
        // --- 3. Ensure that the coldkey is the owner.
        Self::do_account_checks(&coldkey, &hotkey)?;

        // --- 5. Ensure we are not already a delegate
        ensure!(
            !Self::hotkey_is_delegate(&hotkey),
            Error::<T>::AlreadyDelegate
        );

        // --- 6. Ensure we don't exceed tx rate limit
        // --- 6. Ensure we don't exceed tx rate limit
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::TxRateLimitExceeded
        );

        // --- 7. Delegate the key.
        // With introduction of DelegatesTake Delegates became just a flag.
        // Probably there is a migration needed to convert it to bool or something down the road
        Self::delegate_hotkey(&hotkey, Self::get_default_take());

        // Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);

        // Also, set last block for take increase rate limiting, since default take is max
        Self::set_last_tx_block_delegate_take(&coldkey, block);

        // --- 8. Emit the staking event.
        log::info!(
            "DelegateAdded( coldkey:{:?}, hotkey:{:?} )",
            coldkey,
            hotkey
        );
        Self::deposit_event(Event::DelegateAdded(coldkey, hotkey, Self::get_default_take()));

        // --- 9. Ok and return.
        // --- 9. Ok and return.
        Ok(())
    }

    // ---- The implementation for the extrinsic decrease_take
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>::RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The hotkey we are delegating (must be owned by the coldkey.)
    //
    // 	* 'netuid' (u16):
    // 		- Subnet ID to decrease take for
    //
    // 	* 'take' (u16):
    // 		- The stake proportion that this hotkey takes from delegations for subnet ID.
    //
    // # Event:
    // 	* TakeDecreased;
    // 		- On successfully setting a decreased take for this hotkey.
    //
    // # Raises:
    // 	* 'NotRegistered':
    // 		- The hotkey we are delegating is not registered on the network.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- The hotkey we are delegating is not owned by the calling coldket.
    //
    pub fn do_decrease_take(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_decrease_take( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_account_checks(&coldkey, &hotkey)?;

        // --- 3. Ensure we are always strictly decreasing, never increasing take
        if let Ok(current_take) = DelegatesTake::<T>::try_get(&hotkey, netuid) {
            ensure!(
                take < current_take,
                Error::<T>::InvalidTake
            );
        }

        // --- 4. Set the new take value.
        DelegatesTake::<T>::insert(hotkey.clone(), netuid, take);

        // --- 5. Emit the take value.
        log::info!(
            "TakeDecreased( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        Self::deposit_event(Event::TakeDecreased(coldkey, hotkey, take));

        // --- 6. Ok and return.
        Ok(())
    }

    // ---- The implementation for the extrinsic increase_take
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>::RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The hotkey we are delegating (must be owned by the coldkey.)
    //
    // 	* 'netuid' (u16):
    // 		- Subnet ID to increase take for
    //
    // 	* 'take' (u16):
    // 		- The stake proportion that this hotkey takes from delegations for subnet ID.
    //
    // # Event:
    // 	* TakeDecreased;
    // 		- On successfully setting a decreased take for this hotkey.
    //
    // # Raises:
    // 	* 'NotRegistered':
    // 		- The hotkey we are delegating is not registered on the network.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- The hotkey we are delegating is not owned by the calling coldket.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    pub fn do_increase_take(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        take: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the coldkey signature.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_increase_take( origin:{:?} hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );

        // --- 2. Ensure we are delegating a known key.
        //        Ensure that the coldkey is the owner.
        Self::do_account_checks(&coldkey, &hotkey)?;

        // --- 3. Ensure we are strinctly increasing take
        if let Ok(current_take) = DelegatesTake::<T>::try_get(&hotkey, netuid) {
            ensure!(
                take > current_take,
                Error::<T>::InvalidTake
            );
        }

        // --- 4. Ensure take is within the 0 ..= InitialDefaultTake (18%) range
        let max_take = T::InitialDefaultTake::get();
        ensure!(
            take <= max_take,
            Error::<T>::InvalidTake
        );

        // --- 5. Enforce the rate limit (independently on do_add_stake rate limits)
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_delegate_take_rate_limit(Self::get_last_tx_block_delegate_take(&coldkey), block),
            Error::<T>::TxRateLimitExceeded
        );

        // Set last block for rate limiting
        Self::set_last_tx_block_delegate_take(&coldkey, block);

        // --- 6. Set the new take value.
        DelegatesTake::<T>::insert(hotkey.clone(), netuid, take);

        // --- 5. Emit the take value.
        log::info!(
            "TakeIncreased( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
            coldkey,
            hotkey,
            take
        );
        Self::deposit_event(Event::TakeIncreased(coldkey, hotkey, take));

        // --- 8. Ok and return.
        Ok(())
    }


    /// Adds or redistributes weighted stake across specified subnets for a given hotkey.
    ///
    /// This function allows a coldkey to allocate or reallocate stake across different subnets
    /// based on provided weights. It first unstakes from all specified subnets, then redistributes
    /// the stake according to the new weights. If there's any remainder from rounding errors or
    /// unallocated stake, it is staked into the root network.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The associated hotkey account.
    //
    // 	* 'netuids' ( Vec<u16> ):
    // 		- The netuids of the weights to be set on the chain.
    //
    // 	* 'values' ( Vec<u16> ):
    // 		- The values of the weights to set on the chain. u16 normalized.
    //
    // 	* 'stake_to_be_added' (u64):
    // 		- The amount of stake to be added to the hotkey staking account.
    //
    // # Event:
    // 	* StakeAdded;
    // 		- On the successfully adding stake to a global account.
    //
    // # Raises:
    // 	* 'CouldNotConvertToBalance':
    // 		- Unable to convert the passed stake value to a balance.
    //
    // 	* 'NotEnoughBalanceToStake':
    // 		- Not enough balance on the coldkey to add onto the global account.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- The calling coldkey is not associated with this hotkey.
    //
    // 	* 'BalanceWithdrawalError':
    // 		- Errors stemming from transaction pallet.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    // TODO(greg) test this.
    pub fn do_add_weighted_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuids: Vec<u16>,
        values: Vec<u16>,
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_add_weighted_stake( origin:{:?} hotkey:{:?}, netuids:{:?}, values:{:?} )",
            coldkey,
            hotkey,
            netuids,
            values
        );

        // --- 2. Ensure that the hotkey account exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::NotRegistered
        );

        // --- 3. We are either moving nominated stake or we own the hotkey.
        ensure!(
            Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 4. Check weights rate limit.
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::TxRateLimitExceeded
        );

        // --- 5. Check that the length of netuid list and value list are equal for this network.
        ensure!(
            Self::uids_match_values(&netuids, &values),
            Error::<T>::WeightVecNotEqualSize
        );

        // --- 6. Ensure the passed netuids contain no duplicates.
        ensure!(!Self::has_duplicate_uids(&netuids), Error::<T>::DuplicateUids);

        // --- 7. Ensure that the netuids are valid.
        for netuid in netuids.iter() {
            ensure!(
                Self::if_subnet_exist(*netuid),
                Error::<T>::NetworkDoesNotExist
            );
        }

        // --- 8. Unstake from all subnets here.
        let all_netuids: Vec<u16> = Self::get_all_subnet_netuids();
        let mut total_tao_unstaked: u64 = 0;
        for netuid_i in all_netuids.iter() {

            // --- 8.a Get the stake on all of the subnets.
            let netuid_stake_for_coldkey_i: u64 = Self::get_subnet_stake_for_coldkey_and_hotkey( &coldkey, &hotkey, *netuid_i );

            // --- 8.b Compute the dynamic unstake amount.
            let dynamic_unstake_amount_tao:u64 = Self::compute_dynamic_unstake( *netuid_i, netuid_stake_for_coldkey_i );

            // --- 8.c Remove this stake from this network.
            Self::decrease_stake_on_coldkey_hotkey_account(
                &coldkey,
                &hotkey,
                *netuid_i,
                netuid_stake_for_coldkey_i,
            );

            // --- 8.d Increment tao unstaked
            total_tao_unstaked += dynamic_unstake_amount_tao;
        }

        // --- 9. Get sum of stake weights being set.
        let value_sum: u64 = values.iter().map(|&val| val as u64).sum();
        let weights_sum: I64F64 = I64F64::from_num(value_sum);

        // -- 10. Iterate over netuid value and stake to individual subnets proportional to weights.
        let mut amounts_staked: Vec<u64> = vec![];
        for (netuid_i, weight_i) in netuids.iter().zip(values.iter()) {

            // 10.a -- Normalize the weight.
            let normalized_weight:I64F64 = I64F64::from_num( *weight_i ) / weights_sum;
            // 10.b -- Calculate effective stake based on the total removed in the previous step.
            let stake_to_be_added_netuid: u64 = (normalized_weight * I64F64::from_num( total_tao_unstaked )).to_num::<u64>();
            // 10.c Compute the dynamic stake amount.
            let dynamic_stake_amount_added:u64 = Self::compute_dynamic_stake( *netuid_i, stake_to_be_added_netuid );
            // 10.c -- Set stake on subnet the effective stake.
            Self::increase_stake_on_coldkey_hotkey_account(
                &coldkey,
                &hotkey,
                *netuid_i,
                dynamic_stake_amount_added,
            );
            // 10.d -- Sum amounts for accounting remainder
            amounts_staked.push( dynamic_stake_amount_added );
        }

        // -- 11. Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);

        // --- 12. Emit the staking event.
        log::info!(
            "StakeWeightAdded( hotkey:{:?}, netuids:{:?}, values:{:?}, stakes:{:?} )",
            hotkey,
            netuids,
            values,
            amounts_staked
        );
        Self::deposit_event(Event::StakeAdded(hotkey, 0, total_tao_unstaked)); // Restaking the total_removed amount.

        // --- 13. Ok and return.
        Ok(())
    }

    // ---- The implementation for the extrinsic add_stake: Adds stake to a hotkey account.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The associated hotkey account.
    //
    // 	* 'netuid' (u16):
    // 		- The netuid to stake into.
    //
    // 	* 'stake_to_be_added' (u64):
    // 		- The amount of stake to be added to the hotkey staking account.
    //
    // # Event:
    // 	* StakeAdded;
    // 		- On the successfully adding stake to a global account.
    //
    // # Raises:
    // 	* 'CouldNotConvertToBalance':
    // 		- Unable to convert the passed stake value to a balance.
    //
    // 	* 'NotEnoughBalanceToStake':
    // 		- Not enough balance on the coldkey to add onto the global account.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- The calling coldkey is not associated with this hotkey.
    //
    // 	* 'BalanceWithdrawalError':
    // 		- Errors stemming from transaction pallet.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    pub fn do_add_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        stake_to_be_added: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_add_stake( origin:{:?} hotkey:{:?}, netuid:{:?}, stake_to_be_added:{:?} )",
            coldkey,
            hotkey,
            netuid,
            stake_to_be_added
        );

        // --- 2. Ensure that the netuid exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::NetworkDoesNotExist
        );

        // --- 3. We convert the stake u64 into a balance.
        let stake_as_balance = Self::u64_to_balance(stake_to_be_added);
        ensure!(
            stake_as_balance.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );


        // --- 4. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::NotRegistered
        );

        // --- 5. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!(
            Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 6. Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()),
            Error::<T>::NotEnoughBalanceToStake
        );

        // --- 7. Remove balance.
        Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()).map_err(|_| Error::<T>::BalanceWithdrawalError)?;

        // --- 8. Ensure we don't exceed tx rate limit
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::TxRateLimitExceeded
        );

        // --- 9. Compute Dynamic Stake.
        let dynamic_stake = Self::compute_dynamic_stake( netuid, stake_to_be_added );

        // --- 10. If we reach here, add the balance to the hotkey.
        Self::increase_stake_on_coldkey_hotkey_account(
            &coldkey,
            &hotkey,
            netuid,
            dynamic_stake,
        );

        // -- 12. Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);

        // --- 13. Emit the staking event.
        log::info!(
            "StakeAdded( hotkey:{:?}, netuid:{:?}, stake_to_be_added:{:?} )",
            hotkey,
            netuid,
            stake_to_be_added
        );
        Self::deposit_event(Event::StakeAdded(hotkey, netuid, stake_to_be_added));

        // --- 14. Ok and return.
        Ok(())
    }

    // ---- The implementation for the extrinsic remove_stake: Removes stake from a hotkey account and adds it onto a coldkey.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the caller's coldkey.
    //
    // 	* 'hotkey' (T::AccountId):
    // 		- The associated hotkey account.
    //
    // 	* 'netuid' (u16):
    // 		- The netuid to remove stake from.
    //
    // 	* 'stake_to_be_added' (u64):
    // 		- The amount of stake to be added to the hotkey staking account.
    //
    // # Event:
    // 	* StakeRemoved;
    // 		- On the successfully removing stake from the hotkey account.
    //
    // # Raises:
    //
    //  * 'NetworkDoesNotExist':
    //      - Thrown if the subnet we are attempting to stake into does not exist.
    //
    // 	* 'NotRegistered':
    // 		- Thrown if the account we are attempting to unstake from is non existent.
    //
    // 	* 'NonAssociatedColdKey':
    // 		- Thrown if the coldkey does not own the hotkey we are unstaking from.
    //
    // 	* 'NotEnoughStaketoWithdraw':
    // 		- Thrown if there is not enough stake on the hotkey to withdwraw this amount.
    //
    // 	* 'CouldNotConvertToBalance':
    // 		- Thrown if we could not convert this amount to a balance.
    //
    // 	* 'TxRateLimitExceeded':
    // 		- Thrown if key has hit transaction rate limit
    //
    //
    pub fn do_remove_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        stake_to_be_removed: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_remove_stake( origin:{:?} netuid:{:?}, hotkey:{:?}, stake_to_be_removed:{:?} )",
            coldkey,
            hotkey,
            netuid,
            stake_to_be_removed
        );

        // --- 2. Ensure that the netuid exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::NetworkDoesNotExist
        );

        // --- 3. Ensure that the hotkey account exists this is only possible through registration.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::NotRegistered
        );

        // --- 4. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        ensure!(
            Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 5. Ensure that the stake amount to be removed is above zero.
        ensure!(
            stake_to_be_removed > 0,
            Error::<T>::NotEnoughStaketoWithdraw
        );

        // --- 6. Ensure that the hotkey has enough stake to withdraw.
        ensure!(
            Self::has_enough_stake(&coldkey, &hotkey, netuid, stake_to_be_removed),
            Error::<T>::NotEnoughStaketoWithdraw
        );

        // --- 7. Ensure that we can conver this u64 to a balance.
        let stake_to_be_added_as_currency = Self::u64_to_balance(stake_to_be_removed);
        ensure!(
            stake_to_be_added_as_currency.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );

        // --- 8. Ensure we don't exceed tx rate limit
        let block: u64 = Self::get_current_block_as_u64();
        ensure!(
            !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
            Error::<T>::TxRateLimitExceeded
        );


        // --- 8. We remove the balance from the hotkey.
        let SIX_MONTHS_IN_BLOCKS: u64 = 7200 * 30 * 3;
        if Self::get_subnet_creator_hotkey( netuid ) == hotkey {
            ensure!(
                block - Self::get_network_registered_block( netuid ) > SIX_MONTHS_IN_BLOCKS,
                Error::<T>::SubnetCreatorLock
            )
        }

        // --- 9. We remove the balance from the hotkey.
        Self::decrease_stake_on_coldkey_hotkey_account(
            &coldkey,
            &hotkey,
            netuid,
            stake_to_be_removed,
        );

        // --- 10. Compute Dynamic un stake.
        let dynamic_unstake:u64 = Self::compute_dynamic_unstake( netuid, stake_to_be_removed );

        // --- 10. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
        Self::add_balance_to_coldkey_account(&coldkey, Self::u64_to_balance( dynamic_unstake ).unwrap() );

        // Set last block for rate limiting
        Self::set_last_tx_block(&coldkey, block);

        log::info!(
            "StakeRemoved( hotkey:{:?}, stake_to_be_removed:{:?} )",
            hotkey,
            stake_to_be_removed
        );
        Self::deposit_event(Event::StakeRemoved(hotkey, netuid, stake_to_be_removed));

        // --- 11. Done and ok.
        Ok(())
    }

    /// Computes the dynamic unstake amount based on the current reserves and the stake to be removed.
    ///
    /// # Arguments
    /// * `coldkey` - The account ID of the coldkey.
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier for the network.
    /// * `stake_to_be_removed` - The amount of stake to be removed.
    ///
    /// # Returns
    /// * The amount of tao to be pulled out as a result of the unstake operation.
    pub fn compute_dynamic_unstake(
        netuid: u16,
        stake_to_be_removed: u64,
    ) -> u64 {
        // Root network does not have dynamic stake.
        if !Self::is_subnet_dynamic( netuid ) {
            return stake_to_be_removed;
        }

        let tao_reserve = DynamicTAOReserve::<T>::get(netuid);
        let dynamic_reserve = DynamicAlphaReserve::<T>::get(netuid);
        let k = DynamicK::<T>::get(netuid);

        // Calculate the new dynamic reserve after adding the stake to be removed
        let new_dynamic_reserve = dynamic_reserve.saturating_add(stake_to_be_removed);
        // Calculate the new tao reserve based on the new dynamic reserve
        let new_tao_reserve:u64 = ( k / ( new_dynamic_reserve as u128)) as u64;
        // Calculate the amount of tao to be pulled out based on the difference in tao reserves
        let tao = tao_reserve.saturating_sub(new_tao_reserve);

        // Update the reserves with the new values
        DynamicTAOReserve::<T>::insert(netuid, new_tao_reserve);
        DynamicAlphaReserve::<T>::insert(netuid, new_dynamic_reserve);
        DynamicAlphaOutstanding::<T>::mutate( netuid, |outstanding| *outstanding -= stake_to_be_removed ); // Decrement outstanding alpha.

        tao
    }

    /// Computes the dynamic stake amount based on the current reserves and the stake to be added.
    ///
    /// # Arguments
    /// * `coldkey` - The account ID of the coldkey.
    /// * `hotkey` - The account ID of the hotkey.
    /// * `netuid` - The unique identifier for the network.
    /// * `stake_to_be_added` - The amount of stake to be added.
    ///
    /// # Returns
    /// * The amount of dynamic token to be pulled out as a result of the stake operation.
    pub fn compute_dynamic_stake(
        netuid: u16,
        stake_to_be_added: u64,
    ) -> u64 {
        // Root network does not have dynamic stake.
        if !Self::is_subnet_dynamic( netuid ) {
            return stake_to_be_added;
        }


        let tao_reserve = DynamicTAOReserve::<T>::get(netuid);
        let dynamic_reserve = DynamicAlphaReserve::<T>::get(netuid);
        let k = DynamicK::<T>::get(netuid);

        // Calculate the new tao reserve after adding the stake
        let new_tao_reserve = tao_reserve.saturating_add(stake_to_be_added);
        // Calculate the new dynamic reserve based on the new tao reserve
        let new_dynamic_reserve:u64 = (k / ( new_tao_reserve as u128)) as u64;
        // Calculate the amount of dynamic token to be pulled out based on the difference in dynamic reserves
        let dynamic_token = dynamic_reserve.saturating_sub(new_dynamic_reserve);

        // Update the reserves with the new values
        DynamicTAOReserve::<T>::insert(netuid, new_tao_reserve);
        DynamicAlphaReserve::<T>::insert(netuid, new_dynamic_reserve);
        DynamicAlphaOutstanding::<T>::mutate( netuid, |outstanding| *outstanding += dynamic_token ); // Increment outstanding alpha.

        dynamic_token
    }

    // Returns true if the passed hotkey allow delegative staking.
    //
    pub fn hotkey_is_delegate(hotkey: &T::AccountId) -> bool {
        Delegates::<T>::contains_key(hotkey)
    }

    // Sets the hotkey as a delegate with take.
    //
    pub fn delegate_hotkey(hotkey: &T::AccountId, take: u16) {
        Delegates::<T>::insert(hotkey, take);
    }

    // Returns the total amount of stake in the staking table.
    //
    pub fn get_total_stake() -> u64 {
        return TotalStake::<T>::get();
    }

    // Getters for Dynamic terms
    //
    pub fn get_tao_reserve( netuid: u16 ) -> u64 {
        DynamicTAOReserve::<T>::get( netuid )
    }
    pub fn get_alpha_reserve( netuid: u16 ) -> u64 {
        DynamicAlphaReserve::<T>::get( netuid )
    }
    pub fn get_pool_k( netuid: u16 ) -> u128 {
        DynamicK::<T>::get( netuid )
    }
    pub fn is_subnet_dynamic( netuid: u16 ) -> bool {
        IsDynamic::<T>::get( netuid )
    }

    // Returns the total amount of stake under a subnet (delegative or otherwise)
    pub fn get_total_stake_for_subnet(target_subnet: u16) -> u64 {
        SubStake::<T>::iter()
            .filter(|((_, _, subnet), _)| *subnet == target_subnet)
            .fold(0, |acc, (_, stake)| acc.saturating_add(stake))
    }

    // Increases the total amount of stake by the passed amount.
    //
    pub fn increase_total_stake(increment: u64) {
        TotalStake::<T>::put(Self::get_total_stake().saturating_add(increment));
    }

    // Decreases the total amount of stake by the passed amount.
    //
    pub fn decrease_total_stake(decrement: u64) {
        TotalStake::<T>::put(Self::get_total_stake().saturating_sub(decrement));
    }

    // Returns the total amount of stake under a hotkey (delegative or otherwise)
    //
    pub fn get_total_stake_for_hotkey(hotkey: &T::AccountId) -> u64 {
        return TotalHotkeyStake::<T>::get(hotkey);
    }

    // Returns the total amount of stake held by the coldkey (delegative or otherwise)
    //
    pub fn get_total_stake_for_coldkey(coldkey: &T::AccountId) -> u64 {
        return TotalColdkeyStake::<T>::get(coldkey);
    }

    // Returns the total amount of stake under a hotkey for a subnet (delegative or otherwise)
    //
    pub fn get_total_stake_for_hotkey_and_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {
        return TotalHotkeySubStake::<T>::get(hotkey, netuid);
    }

    pub fn get_target_stakes_per_interval() -> u64 {
        return TargetStakesPerInterval::<T>::get();
    }

    pub fn set_target_stakes_per_interval(stakes_per_interval: u64) {
        TargetStakesPerInterval::<T>::put(stakes_per_interval);
    }

    // Creates a cold - hot pairing account if the hotkey is not already an active account.
    //
    pub fn create_account_if_non_existent(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
    ) {
        if !Self::hotkey_account_exists(hotkey) {
            Stake::<T>::insert(hotkey, coldkey, 0);
            SubStake::<T>::insert((hotkey, coldkey, netuid), 0);
            Owner::<T>::insert(hotkey, coldkey);
        }
    }

    // Returns the coldkey owning this hotkey. This function should only be called for active accounts.
    //
    pub fn get_owning_coldkey_for_hotkey(hotkey: &T::AccountId) -> T::AccountId {
        return Owner::<T>::get(hotkey);
    }

    // Returns the hotkey take
    //
    pub fn get_hotkey_take(hotkey: &T::AccountId) -> u16 {
        Delegates::<T>::get(hotkey)
    }

    // Returns the hotkey take
    //
    pub fn get_delegate_take(hotkey: &T::AccountId, netuid: u16) -> u16 {
        DelegatesTake::<T>::get(hotkey, netuid)
    }

    // Returns true if the hotkey account has been created.
    //
    pub fn hotkey_account_exists(hotkey: &T::AccountId) -> bool {
        return Owner::<T>::contains_key(hotkey);
    }

    // Return true if the passed coldkey owns the hotkey.
    //
    pub fn coldkey_owns_hotkey(coldkey: &T::AccountId, hotkey: &T::AccountId) -> bool {
        if Self::hotkey_account_exists(hotkey) {
            return Owner::<T>::get(hotkey) == *coldkey;
        } else {
            return false;
        }
    }

    // Returns true if the cold-hot staking account has enough balance to fufil the decrement.
    //
    pub fn has_enough_stake(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
        decrement: u64,
    ) -> bool {
        return Self::get_subnet_stake_for_coldkey_and_hotkey(coldkey, hotkey, netuid) >= decrement;
    }

    // Increases the stake on the hotkey account under its owning coldkey.
    //
    pub fn increase_stake_on_hotkey_account(hotkey: &T::AccountId, netuid: u16, increment: u64) {
        Self::increase_stake_on_coldkey_hotkey_account(
            &Self::get_owning_coldkey_for_hotkey(hotkey),
            hotkey,
            netuid,
            increment,
        );
    }

    // Decreases the stake on the hotkey account under its owning coldkey.
    //
    pub fn decrease_stake_on_hotkey_account(hotkey: &T::AccountId, netuid: u16, decrement: u64) {
        Self::decrease_stake_on_coldkey_hotkey_account(
            &Self::get_owning_coldkey_for_hotkey(hotkey),
            hotkey,
            netuid,
            decrement,
        );
    }

    // Returns the subent stake under the cold - hot pairing in the staking table.
    //
    pub fn get_subnet_stake_for_coldkey_and_hotkey(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
    ) -> u64 {
        SubStake::<T>::try_get((hotkey, coldkey, netuid)).unwrap_or(0)
    }

    // Returns the stake under the cold - hot pairing in the staking table.
    //
    pub fn get_total_stake_for_hotkey_and_coldkey(
        hotkey: &T::AccountId,
        coldkey: &T::AccountId,
    ) -> u64 {
        Stake::<T>::try_get(hotkey, coldkey).unwrap_or(0)
    }

    pub fn get_tao_per_alpha_price( netuid: u16 ) -> I64F64 {
        let tao_reserve: u64 = DynamicTAOReserve::<T>::get( netuid );
        let alpha_reserve: u64 = DynamicAlphaReserve::<T>::get( netuid );
        if alpha_reserve == 0 {
            return I64F64::from_num( 1.0 );
        } else {
            return I64F64::from_num( tao_reserve ) / I64F64::from_num( alpha_reserve );
        }
    }

    // Returns the stake under the cold - hot pairing in the staking table.
    //
    pub fn get_global_dynamic_tao(
        hotkey: &T::AccountId,
    ) -> u64 {
        let mut global_dynamic_tao: I64F64 = I64F64::from_num( 0.0 );
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        for netuid in netuids.iter() {
            let alpha_stake: I64F64 = I64F64::from_num( Self::get_total_stake_for_hotkey_and_subnet( hotkey, *netuid ) );
            let tao_per_alpha_price: I64F64 = Self::get_tao_per_alpha_price( *netuid );
            global_dynamic_tao += alpha_stake * tao_per_alpha_price;
        }
        return global_dynamic_tao.to_num::<u64>();
    }

    // Returns the stake under the cold - hot pairing in the staking table.
    //
    pub fn get_coldkey_hotkey_global_dynamic_tao(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
    ) -> u64 {
        let mut global_dynamic_tao: I64F64 = I64F64::from_num( 0.0 );
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        for netuid in netuids.iter() {
            let alpha_stake: I64F64 = I64F64::from_num( Self::get_subnet_stake_for_coldkey_and_hotkey( coldkey, hotkey, *netuid ) );
            let tao_per_alpha_price: I64F64 = Self::get_tao_per_alpha_price( *netuid );
            global_dynamic_tao += alpha_stake * tao_per_alpha_price;
        }
        return global_dynamic_tao.to_num::<u64>();
    }

    // Increases the stake on the cold - hot pairing by increment while also incrementing other counters.
    // This function should be called rather than set_stake under account.
    //
    pub fn increase_stake_on_coldkey_hotkey_account(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
        increment: u64,
    ) {
        if increment == 0 {
            return;
        }
        TotalColdkeyStake::<T>::mutate(coldkey,|stake| {
            *stake = stake.saturating_add(increment);
        });
        TotalHotkeyStake::<T>::mutate(hotkey, |stake| {
            *stake = stake.saturating_add(increment);
        });
        TotalHotkeySubStake::<T>::mutate(hotkey, netuid, |stake| {
            *stake = stake.saturating_add(increment);
        });
        Stake::<T>::mutate(hotkey, coldkey, |stake| {
            *stake = stake.saturating_add(increment);
        });
        SubStake::<T>::insert(
            (hotkey, coldkey, netuid),
            SubStake::<T>::try_get((hotkey, coldkey, netuid))
                .unwrap_or(0)
                .saturating_add(increment),
        );
        TotalStake::<T>::mutate(|stake| *stake = stake.saturating_add(increment));
    }

    // Decreases the stake on the cold - hot pairing by the decrement while decreasing other counters.
    //
    pub fn decrease_stake_on_coldkey_hotkey_account(
        coldkey: &T::AccountId,
        hotkey: &T::AccountId,
        netuid: u16,
        decrement: u64,
    ) {
        if decrement == 0 {
            return;
        }
        TotalColdkeyStake::<T>::mutate(coldkey, |stake| {
            *stake = stake.saturating_sub(decrement);
        });
        TotalHotkeyStake::<T>::mutate(hotkey, |stake| {
            *stake = stake.saturating_sub(decrement);
        });
        TotalHotkeySubStake::<T>::mutate(hotkey, netuid, |stake| {
            *stake = stake.saturating_sub(decrement);
        });
        Stake::<T>::mutate(hotkey, coldkey, |stake| {
            *stake = stake.saturating_sub(decrement);
        });
        SubStake::<T>::insert(
            (hotkey, coldkey, netuid),
            SubStake::<T>::try_get((hotkey, coldkey, netuid))
                .unwrap_or(0)
                .saturating_sub(decrement),
        );
        TotalStake::<T>::mutate(|stake| *stake = stake.saturating_sub(decrement));
    }

    pub fn u64_to_balance(
        input: u64,
    ) -> Option<
        <<T as Config>::Currency as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance,
    >{
        input.try_into().ok()
    }

    pub fn add_balance_to_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) {
        // infallible
        let _ = T::Currency::deposit(&coldkey, amount, Precision::BestEffort);
    }

    pub fn set_balance_on_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) {
        T::Currency::set_balance(&coldkey, amount);
    }

    pub fn can_remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> bool {
        let current_balance = Self::get_coldkey_balance(coldkey);
        if amount > current_balance {
            return false;
        }

        // This bit is currently untested. @todo
        let can_withdraw = T::Currency::can_withdraw(&coldkey, amount)
            .into_result(false)
            .is_ok();
        can_withdraw
    }

    pub fn get_coldkey_balance(
        coldkey: &T::AccountId,
    ) -> <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance
    {
        return T::Currency::reducible_balance(
            &coldkey,
            Preservation::Expendable,
            Fortitude::Polite,
        );
    }

    #[must_use = "Balance must be used to preserve total issuance of token"]
    pub fn remove_balance_from_coldkey_account(
        coldkey: &T::AccountId,
        amount: <<T as Config>::Currency as fungible::Inspect<<T as system::Config>::AccountId>>::Balance,
    ) -> Result<u64, DispatchError> {
        let amount_u64: u64 = amount
            .try_into()
            .map_err(|_| Error::<T>::CouldNotConvertToU64)?;

        if amount_u64 == 0 {
            return Ok(0);
        }

        let credit = T::Currency::withdraw(
            &coldkey,
            amount,
            Precision::BestEffort,
            Preservation::Preserve,
            Fortitude::Polite,
        )
        .map_err(|_| Error::<T>::BalanceWithdrawalError)?
        .peek();

        let credit_u64: u64 = credit
            .try_into()
            .map_err(|_| Error::<T>::CouldNotConvertToU64)?;

        if credit_u64 == 0 {
            return Err(Error::<T>::BalanceWithdrawalError.into());
        }

        Ok(credit_u64)
    }

    pub fn unstake_all_coldkeys_from_hotkey_account(hotkey: &T::AccountId) {
        // Iterate through all coldkeys that have a stake on this hotkey account.
        let all_netuids: Vec<u16> = Self::get_all_subnet_netuids();
        for (coldkey_i, _) in
            <Stake<T> as IterableStorageDoubleMap<T::AccountId, T::AccountId, u64>>::iter_prefix(
                hotkey,
            )
        {
            for netuid_i in all_netuids.iter() {
                // Get the stake on this uid.
                let stake_i =
                    Self::get_subnet_stake_for_coldkey_and_hotkey(&coldkey_i, hotkey, *netuid_i);

                // Convert to balance and add to the coldkey account.
                let stake_i_as_balance = Self::u64_to_balance(stake_i);
                if stake_i_as_balance.is_none() {
                    continue; // Don't unstake if we can't convert to balance.
                } else {
                    // Stake is successfully converted to balance.

                    // Remove the stake from the coldkey - hotkey pairing.
                    Self::decrease_stake_on_coldkey_hotkey_account(
                        &coldkey_i, hotkey, *netuid_i, stake_i,
                    );

                    // Add the balance to the coldkey account.
                    Self::add_balance_to_coldkey_account(&coldkey_i, stake_i_as_balance.unwrap());
                }
            }
        }
    }
}
