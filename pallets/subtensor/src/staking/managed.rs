use super::*;
// use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {


    /// Sets the funds of a coldkey to managed under a hotkey, this allows the hotkey to perform a managaged
    /// swap operation on these funds, moving them potentially from one subnet and buying into another.
    ///
    /// # Arguments
    /// * `origin` - The origin coldkey signer.
    /// * `hotkey` - The holding manager
    ///
    /// # Returns
    /// * `DispatchResult` - Indicates the success or failure of the operation.
    ///
    pub fn do_set_managed(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
    ) -> dispatch::DispatchResult {

        // --- 1. Check that the origin is signed by the coldkey.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Check that the hotkey account exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 3. Get the amount of TAO this key has on root.
        let stake_amount_on_root: u64 = Self::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey,
            coldkey,
            Self::get_root_netuid()
        )

        // --- 4. Check that the coldkey has enough stake to be managed.
        ensure!(
            stake_amount_on_root > 100,
            Error::<T>::NotEnoughStake
        );

        // --- 5. Get the current set of managed keys.
        let managed_keys: Vec<T::AccountId>;
        if !ManagedKeys::<T>::contains_key(&hotkey) {
            managed_keys = vec![];
        } else {
            managed_keys = ManagedKeys::<T>::get( &hotkey );
        }
        
        // TODO(nucleus): limit size of this set, if we exceed X managed keys.
        // We should at this point begin popping them based on their total value.

        // --- 5. Append the key to the managed keys.
        managed_keys.push( coldkey );

        // --- 6. Set the new managed keys.
        ManagedKeys::<T>::insert( &hotkey.clone(), managed_keys.clone() );

        // --- 7. Log the event.
        log::info!(
            "NewManagedKey( hotkey:{:?}, coldkey:{:?})",
            hotkey.clone(),
            coldkey.clone(),
        );
        Self::deposit_event(Event::ManagedStakeSwap(
            hotkey,
            coldkey,
        ));

        // -- 8. Ok and return.
        Ok(())
    
    }

    /// Moves stake from one hotkey to the same hotkey on another subnet performing a stake swap.
    /// The stake it taken from the total of the pool so this is managed, it moves multiple parties funds.
    ///
    /// # Arguments
    /// * `origin` - The origin of the transaction, the coldkey owning the hotkey pool.
    /// * `hotkey` - The holding hotkey for the poll.
    /// * `amount` - The amount of the original token to move.
    /// * `origin_netuid` - The network ID of the origin subnet.
    /// * `destination_netuid` - The network ID of the destination subnet.
    ///
    /// # Returns
    /// * `DispatchResult` - Indicates the success or failure of the operation.
    ///
    /// # Events
    /// Emits a `ManagedStakeSwap` event upon successful completion of the managed stake swap.
    pub fn do_managed_swap_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        amount: T::AccountId,
        origin_netuid: u16,
        dest_netuid: u16,
    ) -> dispatch::DispatchResult {
        // --- 1. Check that the origin is signed by the coldkey.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Check that the subnet exists.
        ensure!(
            Self::if_subnet_exist(origin_netuid),
            Error::<T>::SubnetNotExists
        );
        ensure!(
            Self::if_subnet_exist(dest_netuid),
            Error::<T>::SubnetNotExists
        );

        // --- 3. Check that the origin_hotkey exists.
        ensure!(
            Self::hotkey_account_exists(&hotkey),
            Error::<T>::HotKeyAccountNotExists
        );

        // --- 4. Ensure that the coldkey owns the hotkey
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 5. Ensure that the coldkey owns the hotkey
        ensure!(
            ManagedKeys::<T>::contains_key(&hotkey),
            Error::<T>::HotkeyNotManaged
        );

        // --- 6. Get the balance of all managed keys on this subnet.
        let total_managed_balance: u64 = 0;
        let managed_keys: Vec<T::AccountId> = ManagedKeys::<T>::get( &hotkey );
        for managed_coldkey in managed_keys.iter() {
            total_managed_balance += Self::get_stake_for_hotkey_and_coldkey_on_subnet( &hotkey, managed_coldkey, origin_netuid );
        }

        // --- 7. Check if amount moved does not exceed total.
        ensure!(
             amount <= total_managed_balance,
             Error::<T>::NotEnoughStakeToWithdraw
         );

        // --- 8. Swap the total amount through the pool.
        let total_tao_moved: u64 = Self::swap_alpha_for_tao( origin_netuid, total_managed_balance );

        // --- 9. Swap the tao through to alpba
        let total_alpha_received: u64 = Self::swap_alpha_for_tao( dest_netuid, total_tao_moved );

        // --- 7. For each key, swap their funds as a proportion of the total moved.
        for managed_coldkey in managed_keys.iter() {

            // 7.1: Get the proportion of the total moved based on my proportion of the total. 
            let this_key_stake: I96F32 = I96F32::from_num( Self::get_stake_for_hotkey_and_coldkey_on_subnet( &hotkey, managed_coldkey, origin_netuid) );

            // 7.2: Get the proportional amount of the move.
            let proportional_stake: I96F32 = this_key_stake.checked_div( I96F32::from_num(total_managed_balance) ).unwrap_or( I96F32::from_num(0.0) );

            // 7.3. Get the amount moved in during the unstake operation.
            let amount_removed: I96F32 = I96F32::from_num( amount ).saturating_mul( proportional_stake );

            // 7.4. Get the amount recieved on the other netuid.
            let amount_received: I96F32 = total_alpha_received.saturating_mul( proportional_stake );

            // 7.5. Decrease amount on origin subnet.
            Self::decrease_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, 
                managed_coldkey,
                origin_netuid,
                amount_removed
            )

            // 7.6. Decrease amount on origin subnet.
            Self::increase_stake_for_hotkey_and_coldkey_on_subnet(
                &hotkey, 
                managed_coldkey,
                origin_netuid,
                amount_received
            )

        }

        // --- 11. Log the event.
        log::info!(
            "ManagedStakeSwap( hotkey:{:?}, origin_netuid:{:?}, dest_netuid:{:?}, amount:{:?}, recieved:{:?} )",
            hotkey.clone(),
            origin_netuid,
            dest_netuid,
            amount,
            total_alpha_received,
        );
        Self::deposit_event(Event::ManagedStakeSwap(
            hotkey,
            origin_netuid,
            dest_hotkey,
            amount,
            total_alpha_received
        ));

        // -- 12. Ok and return.
        Ok(())
    }
}
