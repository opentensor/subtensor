use
{
    super::
    {
        *
    },
    crate::
    {
        math::
        {
            *
        }
    },
    frame_support::
    {
        StorageDoubleMap,
        StorageNMap,
        storage::
        {
            IterableStorageDoubleMap
        }
    },
    substrate_fixed::
    {
        types::
        {
            I32F32,
            I64F64
        } 
    },
    sp_std::
    {
        vec,
        vec::
        {
            Vec
        }
    }
};

include!("balance.rs");

impl<T: Config> Pallet<T> 
{
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
    // 		- The stake proportion that this hotkey takes from delegations.
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
    pub fn do_become_delegate(origin: T::RuntimeOrigin, hotkey: T::AccountId, take: u16) -> dispatch::DispatchResult 
    {
        // --- 1. We check the coldkey signuture.
        let coldkey: T::AccountId;
        {
            coldkey = ensure_signed(origin)?;

            log::info!(
                "do_become_delegate( origin:{:?} hotkey:{:?}, take:{:?} )",
                coldkey,
                hotkey,
                take
            );
        }

        // --- 2. Ensure we are delegating an known key.
        {
            ensure!(
                Self::hotkey_account_exists(&hotkey),
                Error::<T>::NotRegistered
            );
        }

        // --- 3. Ensure that the coldkey is the owner.
        {
            ensure!(
                Self::coldkey_owns_hotkey(&coldkey, &hotkey),
                Error::<T>::NonAssociatedColdKey
            );
        }

        // --- 4. Ensure we are not already a delegate (dont allow changing delegate take.)
        {
            ensure!(
                !Self::hotkey_is_delegate(&hotkey),
                Error::<T>::AlreadyDelegate
            );
        }

        // --- 5. Ensure we don't exceed tx rate limit
        let block: u64;
        {
            block = Self::get_current_block_as_u64();

            ensure!(
                !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
                Error::<T>::TxRateLimitExceeded
            );
        }   

        // --- 6. Delegate the key.
        {
            Self::delegate_hotkey(&hotkey, take);

            // Set last block for rate limiting
            Self::set_last_tx_block(&coldkey, block);
        }

        // --- 7. Emit the staking event.
        {
            log::info!(
                "DelegateAdded( coldkey:{:?}, hotkey:{:?}, take:{:?} )",
                coldkey,
                hotkey,
                take
            );
            
            Self::deposit_event(Event::DelegateAdded(coldkey, hotkey, take));
        }

        // --- 8. Ok and return.
        return Ok(());
    }

    pub fn do_add_stake(origin: T::RuntimeOrigin, hotkey: T::AccountId, stake_to_be_added: u64) -> dispatch::DispatchResult 
    {
        return Ok(());
    }

    pub fn do_remove_stake(origin: T::RuntimeOrigin, hotkey: T::AccountId, stake_to_be_removed: u64) -> dispatch::DispatchResult 
    {
        return Ok(());
    }

    // Creates a cold - hot pairing account if the hotkey is not already an active account.
    //
    pub fn create_account_if_non_existent(coldkey: &T::AccountId, hotkey: &T::AccountId) 
    {
        if !Self::hotkey_account_exists(hotkey) 
        {
            //Stake::<T>::insert(hotkey, coldkey, 0);
            Owner::<T>::insert(hotkey, coldkey);
        }
    }

    pub fn has_enough_stake(netuid: u16, coldkey: &T::AccountId, hotkey: &T::AccountId, decrement: u64) -> bool 
    {
        return Self::does_coldkey_hotkey_have_enough_subnet_stake(netuid, coldkey, hotkey, decrement);
    }

    pub fn unstake_all_coldkeys_from_hotkey_account(netuid: u16, hotkey: &T::AccountId) 
    {
        Self::remove_all_subnet_stake_for_hotkey(netuid, hotkey);
    }

    pub fn get_subnet_total_stake(netuid: u16) -> u64
    {
        return TotalSubnetStake::<T>::try_get(netuid).unwrap_or(0);
    }

    pub fn inc_subnet_total_stake(netuid: u16, amount: u64)
    {
        TotalSubnetStake::<T>::insert(
            netuid, 
            Self::get_subnet_total_stake(netuid) + amount
        );
    }

    pub fn dec_subnet_total_stake(netuid: u16, amount: u64)
    {
        TotalSubnetStake::<T>::insert(
            netuid, 
            Self::get_subnet_total_stake(netuid) - amount
        );
    }

    pub fn get_subnet_total_stake_for_coldkey(netuid: u16, coldkey: &T::AccountId) -> u64
    {
        return TotalSubnetColdkeyStake::<T>::try_get(netuid, coldkey).unwrap_or(0);
    }

    pub fn inc_subnet_total_stake_for_coldkey(netuid: u16, coldkey: &T::AccountId, amount: u64)
    {
        TotalSubnetColdkeyStake::<T>::insert(
            netuid, 
            coldkey,
            Self::get_subnet_total_stake_for_coldkey(netuid, coldkey) + amount
        )
    }

    pub fn dec_subnet_total_stake_for_coldkey(netuid: u16, coldkey: &T::AccountId, amount: u64)
    {
        let new_stake: u64 = Self::get_subnet_total_stake_for_coldkey(netuid, coldkey) - amount;
        if new_stake == 0
        {
            TotalSubnetColdkeyStake::<T>::remove(netuid, coldkey);
        }
        else
        {
            TotalSubnetColdkeyStake::<T>::insert(
                netuid, 
                coldkey,
                new_stake
            )
        }
    }

    pub fn get_subnet_total_stake_for_hotkey(netuid: u16, hotkey: &T::AccountId) -> u64
    {
        return TotalSubnetHotkeyStake::<T>::try_get(netuid, hotkey).unwrap_or(0);
    }

    pub fn inc_subnet_total_stake_for_hotkey(netuid: u16, hotkey: &T::AccountId, amount: u64)
    {
        TotalSubnetHotkeyStake::<T>::insert(
            netuid, 
            hotkey,
            Self::get_subnet_total_stake_for_hotkey(netuid, hotkey) + amount
        )
    }

    pub fn dec_subnet_total_stake_for_hotkey(netuid: u16, hotkey: &T::AccountId, amount: u64)
    {
        let new_stake: u64 = Self::get_subnet_total_stake_for_hotkey(netuid, hotkey) - amount;
        if new_stake == 0
        {
            TotalSubnetHotkeyStake::<T>::remove(netuid, hotkey);
        }
        else
        {
            TotalSubnetHotkeyStake::<T>::insert(
                netuid, 
                hotkey,
                new_stake
            );
        }
    }

    pub fn get_subnet_stake_for_coldkey_hotkey(netuid: u16, coldkey: &T::AccountId, hotkey: &T::AccountId) -> u64
    {
        return SubnetStake::<T>::try_get((netuid, coldkey, hotkey)).unwrap_or(0);
    }
    
    pub fn inc_subnet_stake_for_coldkey_hotkey(netuid: u16, coldkey: &T::AccountId, hotkey: &T::AccountId, amount: u64)
    {
        Self::create_account_if_non_existent(coldkey, hotkey);

        Self::inc_subnet_total_stake(netuid, amount);
        Self::inc_subnet_total_stake_for_coldkey(netuid, coldkey, amount);
        Self::inc_subnet_total_stake_for_hotkey(netuid, hotkey, amount);

        SubnetStake::<T>::insert(
            (netuid, coldkey, hotkey),
            Self::get_subnet_stake_for_coldkey_hotkey(netuid, coldkey, hotkey) + amount
        );
    }

    pub fn dec_subnet_stake_for_coldkey_hotkey(netuid: u16, coldkey: &T::AccountId, hotkey: &T::AccountId, amount: u64)
    {
        Self::dec_subnet_total_stake(netuid, amount);
        Self::dec_subnet_total_stake_for_coldkey(netuid, coldkey, amount);
        Self::dec_subnet_total_stake_for_hotkey(netuid, hotkey, amount);

        let new_stake: u64 = Self::get_subnet_stake_for_coldkey_hotkey(netuid, coldkey, hotkey) - amount;
        if new_stake == 0
        {
            SubnetStake::<T>::remove((netuid, coldkey, hotkey));
        }
        else
        {
            SubnetStake::<T>::insert(
                (netuid, coldkey, hotkey),
                new_stake  
            );
        }
    }

    pub fn does_coldkey_hotkey_have_enough_subnet_stake(netuid: u16, coldkey: &T::AccountId, hotkey: &T::AccountId, stake: u64) -> bool
    {
        return Self::get_subnet_stake_for_coldkey_hotkey(netuid, coldkey, hotkey) >= stake;
    }

    pub fn get_staking_map_for_coldkey(coldkey: &T::AccountId) -> Vec<(u16, u64)>
    {
        let mut stake: Vec<(u16, u64)> = vec![(0, 0); Self::get_max_subnets() as usize];
        for netuid in 0..Self::get_max_subnets() as u16
        {
            let subnet_stake: u64 = Self::get_subnet_total_stake_for_coldkey(netuid + 1, coldkey);
            if subnet_stake > 0
            {
                stake[netuid as usize] = (netuid + 1, subnet_stake);
            }
        }

        return stake;
    }

    pub fn get_combined_subnet_stake() -> u64
    {
        let mut stake: u64 = 0;
        for netuid in 0..Self::get_max_subnets() as u16
        {
            stake = stake + Self::get_subnet_total_stake(netuid + 1);
        }

        return stake;
    }

    pub fn get_combined_subnet_stake_for_coldkey(coldkey: &T::AccountId) -> u64
    {
        let mut stake: u64 = 0;
        for netuid in 0..Self::get_max_subnets() as u16
        {
            stake = stake + Self::get_subnet_total_stake_for_coldkey(netuid + 1, coldkey);
        }

        return stake;
    }

    pub fn get_combined_subnet_stake_for_hotkey(hotkey: &T::AccountId) -> u64
    {
        let mut stake: u64 = 0;
        for netuid in 0..Self::get_max_subnets() as u16
        {
            stake = stake + Self::get_subnet_total_stake_for_hotkey(netuid + 1, hotkey);
        }

        return stake;
    }

    pub fn get_stake_map_for_subnet(netuid: u16) -> Vec<(T::AccountId, T::AccountId, u64)>
    {
        let mut stake: Vec<(T::AccountId, T::AccountId, u64)> = vec![];
        for (subnetid, delegate_coldkey, hotkey) in SubnetStake::<T>::iter_keys()
        {
            if subnetid == netuid
            {
                stake.push((
                    delegate_coldkey.clone(),
                    hotkey.clone(),
                    Self::get_subnet_stake_for_coldkey_hotkey(netuid, &delegate_coldkey, &hotkey)
                ));
            }
        }

        return stake;
    }

    pub fn get_stake_map() -> Vec<u64>
    {
        let mut stake_map: Vec<u64> = vec![0; Self::get_max_subnets() as usize];
        for netuid in 0..Self::get_max_subnets() as u16
        {
            stake_map[netuid as usize] = Self::get_subnet_total_stake(netuid + 1);
        }

        return stake_map;
    }

    pub fn get_normalized_stake_map() -> Vec<I32F32>
    {
        let mut stake_map: Vec<I32F32> = vec![I32F32::from_num(0.0); Self::get_max_subnets() as usize];
        for netuid in 0..Self::get_max_subnets() as u16
        {
            stake_map[netuid as usize] = I32F32::from_num(
                Self::get_subnet_total_stake(netuid + 1)
            );
        }

        inplace_normalize(&mut stake_map);

        return stake_map;
    }

    pub fn remove_all_subnet_stake(netuid: u16)
    {
        {
            let mut stake_to_remove: Vec<(T::AccountId, T::AccountId, u64)> = vec![];

            for (subnetid, delegate_coldkey, hotkey) in SubnetStake::<T>::iter_keys()
            {
                if subnetid == netuid
                {
                    stake_to_remove.push((
                        delegate_coldkey.clone(), 
                        hotkey.clone(), 
                        Self::get_subnet_stake_for_coldkey_hotkey(netuid, &delegate_coldkey, &hotkey)
                    ));
                }
            }

            for (coldkey, hotkey, stake) in stake_to_remove
            {
                Self::dec_subnet_stake_for_coldkey_hotkey(netuid, &coldkey, &hotkey, stake);
                Self::add_balance_to_coldkey_account(&coldkey, Self::u64_to_balance(stake).unwrap());

                Self::deposit_event(Event::SubnetStakeRemoved(netuid, hotkey, stake));
            }
        }

        {
            let mut stake_to_remove: Vec<(T::AccountId, u64)> = vec![];

            for (subnetid, hotkey) in TotalSubnetHotkeyStake::<T>::iter_keys()
            {
                if subnetid == netuid
                {
                    stake_to_remove.push((
                        hotkey.clone(),
                        Self::get_subnet_total_stake_for_hotkey(netuid, &hotkey)
                    ));
                }
            }

            for (hotkey, stake) in stake_to_remove
            {
                Self::dec_subnet_total_stake_for_hotkey(netuid, &hotkey, stake);
                Self::add_balance_to_coldkey_account(&Self::get_owning_coldkey_for_hotkey(&hotkey), Self::u64_to_balance(stake).unwrap());

                Self::deposit_event(Event::SubnetStakeRemoved(netuid, hotkey, stake));
            }
        }

        /*
        {
            let mut stake_to_remove: Vec<(T::AccountId, u64)> = vec![];

            for (subnetid, coldkey) in TotalSubnetColdkeyStake::<T>::iter_keys()
            {
                if subnetid == netuid
                {
                    stake_to_remove.push((
                        coldkey.clone(),
                        Self::get_subnet_total_stake_for_coldkey(netuid, &coldkey)
                    ));
                }
            }

            for (coldkey, stake) in stake_to_remove
            {
                Self::dec_subnet_total_stake_for_coldkey(netuid, &coldkey, stake);
                Self::add_balance_to_coldkey_account(&coldkey, Self::u64_to_balance(stake).unwrap());

                Self::deposit_event(Event::SubnetStakeRemoved(netuid, coldkey, stake));
            }
        }
        */

        TotalSubnetStake::<T>::remove(netuid);

        while TotalSubnetHotkeyStake::<T>::contains_prefix(netuid)
        {
            let _ = TotalSubnetHotkeyStake::<T>::clear_prefix(netuid, u32::MAX, None);
        }

        while TotalSubnetColdkeyStake::<T>::contains_prefix(netuid)
        {
            let _ = TotalSubnetColdkeyStake::<T>::clear_prefix(netuid, u32::MAX, None);
        }
    }

    pub fn remove_all_subnet_stake_for_hotkey(netuid: u16, hotkey: &T::AccountId)
    {
        {
            let mut stake_to_remove: Vec<(T::AccountId, T::AccountId, u64)> = vec![];

            for (subnetid, delegate_coldkey, s_hotkey) in SubnetStake::<T>::iter_keys()
            {
                if subnetid == netuid && s_hotkey == *hotkey
                {
                    stake_to_remove.push((
                        delegate_coldkey.clone(), 
                        hotkey.clone(), 
                        Self::get_subnet_stake_for_coldkey_hotkey(subnetid, &delegate_coldkey, &hotkey)
                    ));
                }
            }

            for (coldkey, hotkey, stake) in stake_to_remove
            {
                Self::dec_subnet_stake_for_coldkey_hotkey(netuid, &coldkey, &hotkey, stake);
                Self::add_balance_to_coldkey_account(&coldkey, Self::u64_to_balance(stake).unwrap());

                Self::deposit_event(Event::SubnetStakeRemoved(netuid, coldkey, stake));
            }
        }
        
        {            
            let stake: u64 = Self::get_subnet_total_stake_for_hotkey(netuid, hotkey);
            if stake > 0
            {
                Self::dec_subnet_total_stake_for_hotkey(netuid, hotkey, stake);

                let coldkey = Self::get_owning_coldkey_for_hotkey(hotkey);
                Self::add_balance_to_coldkey_account(&coldkey, Self::u64_to_balance(stake).unwrap());

                Self::deposit_event(Event::SubnetStakeRemoved(netuid, coldkey, stake));
            }   
        }
    }

    pub fn get_last_staking_tx_block(netuid: u16, coldkey: &T::AccountId) -> u64
    {
        return LastStakingTxBlock::<T>::try_get(netuid, coldkey).unwrap_or(0);
    }

    pub fn exceeds_staking_tx_rate_limit(netuid: u16, coldkey: &T::AccountId, block: u64) -> bool
    {
        let delta: u64 = block - Self::get_last_staking_tx_block(netuid, &coldkey);

        return delta >= Self::get_tempo(netuid) as u64;
    }

    pub fn check_and_set_staking_rate_limit(netuid: u16, coldkey: &T::AccountId) -> bool
    {
        let block: u64 = Self::get_current_block_as_u64();
        if Self::exceeds_staking_tx_rate_limit(netuid, &coldkey, block)
        {
            return false;
        }

        LastStakingTxBlock::<T>::insert(netuid, coldkey, block);

        return true;
    }

    pub fn do_add_subnet_stake(origin: T::RuntimeOrigin, hotkey: T::AccountId, netuid: u16, stake_to_be_added: u64) -> dispatch::DispatchResult
    {
        // --- 1. We check that the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey: T::AccountId;
        {
            coldkey = ensure_signed(origin)?;

            log::info!(
                "add_subnet_stake( origin:{:?}, hotkey:{:?}, netuid:{:?}, stake_to_be_added:{:?} )",
                coldkey,
                hotkey,
                netuid,
                stake_to_be_added
            );
        }

        // --- 1.5. Check if subnet exists
        {
            ensure!(
                netuid != 0 && Self::if_subnet_exist(netuid),
                Error::<T>::NetworkDoesNotExist
            );
        }

        // --- 2. We convert the stake u64 into a balance
        let stake_as_balance;
        {
            stake_as_balance = Self::u64_to_balance(stake_to_be_added);

            ensure!(
                stake_as_balance.is_some(), 
                Error::<T>::CouldNotConvertToBalance
            );
        }

        // --- 3. Ensure the callers coldkey has enough stake to perform the transaction.
        {
            ensure!(
                Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()),
                Error::<T>::NotEnoughBalanceToStake
            );
        }

        // 4. Ensure that the hotkey account exists this is only possible through registration
        {
            let uid = Self::get_uid_for_net_and_hotkey(netuid, &hotkey);
            ensure!(
                uid.is_ok(),
                Error::<T>::NotRegistered
            );
        }

        // 5. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        {
            ensure!(
                Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
                Error::<T>::NonAssociatedColdKey
            );
        }

        // --- 6. Ensure we don't exceed tx rate limit
        let block: u64;
        {
            block = Self::get_current_block_as_u64();

            ensure!(
                !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
                Error::<T>::TxRateLimitExceeded
            );

            ensure!(
                Self::check_and_set_staking_rate_limit(netuid, &coldkey),
                Error::<T>::StakingTxRateLimitExceeded
            );
        }

        // --- 7. Ensure the remove operation from the coldkey is a success.
        {
            ensure!(
                Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true,
                Error::<T>::BalanceWithdrawalError
            );
        }

        // --- 8. If we reach here, add the balance to the hotkey.
        {
            log::info!("staking map before add: {:?}", Self::get_staking_map_for_coldkey(&coldkey));
            Self::inc_subnet_stake_for_coldkey_hotkey(netuid, &coldkey, &hotkey, stake_to_be_added);
            log::info!("staking map after add: {:?}", Self::get_staking_map_for_coldkey(&coldkey));

            // Set last block for rate limiting
            Self::set_last_tx_block(&coldkey, block);
        }

        // --- 9. Emit the staking event.
        {
            log::info!(
                "SubnetStakeAdded( netuid:{:?}, hotkey:{:?}, stake_to_be_added:{:?} )",
                netuid,
                hotkey,
                stake_to_be_added
            );

            Self::deposit_event(Event::SubnetStakeAdded(netuid, hotkey, stake_to_be_added));
        }
        
        // --- 10. Recalculate emission values
        {
            ensure!(
                Self::calc_subnet_emissions().is_ok(),
                Error::<T>::ErrorCalculatingEmissions
            );
        }

        // --- 11. Ok and return.

        return Ok(());
    }


    pub fn do_remove_subnet_stake(origin: T::RuntimeOrigin, hotkey: T::AccountId, netuid: u16, stake_to_be_removed: u64) -> dispatch::DispatchResult 
    {
        // --- 1. We check the transaction is signed by the caller and retrieve the T::AccountId coldkey information.
        let coldkey: T::AccountId;
        {
            coldkey = ensure_signed(origin)?;

            log::info!(
                "remove_subnet_stake( origin:{:?} hotkey:{:?}, netuid:{:?}, stake_to_be_removed:{:?} )",
                coldkey,
                hotkey,
                netuid,
                stake_to_be_removed
            );
        }

        // --- 1.5. Check if subnet exists
        {
            ensure!(
                netuid != 0 && Self::if_subnet_exist(netuid),
                Error::<T>::NetworkDoesNotExist
            );

            // --- Ensure that the stake amount to be removed is above zero.
            ensure!(
                stake_to_be_removed > 0,
                Error::<T>::NotEnoughStaketoWithdraw
            );
        }

        // --- 2. Ensure that the hotkey account exists this is only possible through registration.
        {
            ensure!(
                Self::hotkey_account_exists(&hotkey),
                Error::<T>::NotRegistered
            );
        }

        // --- 3. Ensure that the hotkey allows delegation or that the hotkey is owned by the calling coldkey.
        {
            ensure!(
                Self::hotkey_is_delegate(&hotkey) || Self::coldkey_owns_hotkey(&coldkey, &hotkey),
                Error::<T>::NonAssociatedColdKey
            );
        }

        // --- 4. Ensure that the hotkey has enough stake to withdraw.
        {
            ensure!(
                Self::does_coldkey_hotkey_have_enough_subnet_stake(netuid, &coldkey, &hotkey, stake_to_be_removed),
                Error::<T>::NotEnoughStaketoWithdraw
            );
        }

        // --- 5. Ensure that we can conver this u64 to a balance.
        let stake_to_be_added_as_currency;
        { 
            stake_to_be_added_as_currency = Self::u64_to_balance(stake_to_be_removed);

            ensure!(
                stake_to_be_added_as_currency.is_some(),
                Error::<T>::CouldNotConvertToBalance
            );
        }

        // --- 6. Ensure we don't exceed tx rate limit
        let block: u64;
        {
            block = Self::get_current_block_as_u64();

            ensure!(
                !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
                Error::<T>::TxRateLimitExceeded
            );

            ensure!(
                Self::check_and_set_staking_rate_limit(netuid, &coldkey),
                Error::<T>::StakingTxRateLimitExceeded
            );
        }

        // --- 7. We remove the balance from the hotkey.
        {
            log::info!("staking map before dec: {:?}", Self::get_staking_map_for_coldkey(&coldkey));
            Self::dec_subnet_stake_for_coldkey_hotkey(netuid, &coldkey, &hotkey, stake_to_be_removed);
            log::info!("staking map after dec: {:?}", Self::get_staking_map_for_coldkey(&coldkey));
        }

        // --- 8. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
        {
            Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_added_as_currency.unwrap());

            // Set last block for rate limiting
            Self::set_last_tx_block(&coldkey, block);
        }

        // --- 9. Emit the unstaking event.
        {
            log::info!(
                "SubnetStakeRemoved( netuid:{:?}, hotkey:{:?}, stake_to_be_removed:{:?} )",
                netuid,
                hotkey,
                stake_to_be_removed
            );

            Self::deposit_event(Event::SubnetStakeRemoved(netuid, hotkey, stake_to_be_removed));
        }

        // --- 10. Recalculate emission values
        {
            ensure!(
                Self::calc_subnet_emissions().is_ok(),
                Error::<T>::ErrorCalculatingEmissions
            );
        }

        // --- 11. Done and ok.
        return Ok(());
    }

}