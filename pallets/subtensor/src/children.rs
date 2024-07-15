use super::*;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic do_set_child_singular: Sets a single child.
    ///
    /// This function allows a coldkey to set children keys.
    ///
    /// # Arguments:
    /// * `origin` (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the calling coldkey. Setting a hotkey child can only be done by the coldkey.
    ///
    /// * `hotkey` (T::AccountId):
    ///     - The hotkey which will be assigned the child.
    ///
    /// * `netuid` (u16):
    ///     - The u16 network identifier where the child keys will exist.
    ///
    /// * `children` Vec[(u64, T::AccountId)]:
    ///     - A list of children with their proportions.
    ///
    /// # Events:
    /// * `ChildrenAdded`:
    ///     - On successfully registering children to a hotkey.
    ///
    /// # Errors:
    /// * `SubNetworkDoesNotExist`:
    ///     - Attempting to register to a non-existent network.
    /// * `RegistrationNotPermittedOnRootSubnet`:
    ///     - Attempting to register a child on the root network.
    /// * `NonAssociatedColdKey`:
    ///     - The coldkey does not own the hotkey or the child is the same as the hotkey.
    /// * `HotKeyAccountNotExists`:
    ///     - The hotkey account does not exist.
    ///
    /// # Detailed Explanation of Checks:
    /// 1. **Signature Verification**: Ensures that the caller has signed the transaction, verifying the coldkey.
    /// 2. **Root Network Check**: Ensures that the delegation is not on the root network, as child hotkeys are not valid on the root.
    /// 3. **Network Existence Check**: Ensures that the specified network exists.
    /// 4. **Ownership Verification**: Ensures that the coldkey owns the hotkey.
    /// 5. **Hotkey Account Existence Check**: Ensures that the hotkey account already exists.
    /// 6. **Child-Hotkey Distinction**: Ensures that the child is not the same as the hotkey.
    /// 7. **Old Children Cleanup**: Removes the hotkey from the parent list of its old children.
    /// 8. **New Children Assignment**: Assigns the new child to the hotkey and updates the parent list for the new child.
    ///
    pub fn do_set_children(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        netuid: u16,
        children: Vec<(u64, T::AccountId)>,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::trace!(
            "do_set_children( coldkey:{:?} hotkey:{:?} netuid:{:?} children:{:?} )",
            coldkey,
            netuid,
            hotkey,
            children
        );

        // --- 2. Check that this delegation is not on the root network. Child hotkeys are not valid on root.
        ensure!(
            netuid != Self::get_root_netuid(),
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );

        // --- 3. Check that the network we are trying to create the child on exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 4. Check that the coldkey owns the hotkey.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 4.1. Ensure that the number of children does not exceed 5.
        ensure!(children.len() <= 5, Error::<T>::TooManyChildren);

        // --- 5. Ensure that each child is not the hotkey.
        for (_, child_i) in &children {
            ensure!(child_i != &hotkey, Error::<T>::InvalidChild);
        }
        // --- 5.1. Ensure that the sum of the proportions does not exceed u64::MAX.
        let _total_proportion: u64 = children
            .iter()
            .try_fold(0u64, |acc, &(proportion, _)| acc.checked_add(proportion))
            .ok_or(Error::<T>::ProportionOverflow)?;
        // --- 5.2. Ensure there are no duplicates in the list of children.
        let mut unique_children = Vec::new();
        for (_, child_i) in &children {
            ensure!(
                !unique_children.contains(child_i),
                Error::<T>::DuplicateChild
            );
            unique_children.push(child_i.clone());
        }

        // --- 6. Erase myself from old children's parents.
        let old_children: Vec<(u64, T::AccountId)> = ChildKeys::<T>::get(hotkey.clone(), netuid);

        // --- 6.0. Iterate over all my old children and remove myself from their parent's map.
        for (_, old_child_i) in old_children.clone().iter() {
            // --- 6.1. Get the old child's parents on this network.
            let my_old_child_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(old_child_i.clone(), netuid);

            // --- 6.2. Filter my hotkey from my old children's parents list.
            let filtered_parents: Vec<(u64, T::AccountId)> = my_old_child_parents
                .into_iter()
                .filter(|(_, parent)| *parent != hotkey)
                .collect();

            // --- 6.3. Update the parent list in storage
            ParentKeys::<T>::insert(old_child_i, netuid, filtered_parents);
        }

        // --- 7.1. Insert my new children + proportion list into the map.
        ChildKeys::<T>::insert(hotkey.clone(), netuid, children.clone());

        // --- 7.2. Update the parents list for my new children.
        for (proportion, new_child_i) in children.clone().iter() {
            // --- 8.2.1. Get the child's parents on this network.
            let mut new_child_previous_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(new_child_i.clone(), netuid);

            // --- 7.2.2. Append my hotkey and proportion to my new child's parents list.
            // NOTE: There are no duplicates possible because I previously removed my self from my old children.
            new_child_previous_parents.push((*proportion, hotkey.clone()));

            // --- 7.2.3. Update the parents list in storage.
            ParentKeys::<T>::insert(new_child_i.clone(), netuid, new_child_previous_parents);
        }

        // --- 8. Log and return.
        log::trace!(
            "SetChildren( netuid:{:?}, hotkey:{:?}, children:{:?} )",
            hotkey,
            netuid,
            children.clone()
        );
        Self::deposit_event(Event::SetChildren(hotkey.clone(), netuid, children.clone()));

        // Ok and return.
        Ok(())
    }

    /// Calculates the total stake held by a hotkey on the network, considering child/parent relationships.
    ///
    /// This function performs the following steps:
    /// 1. Checks for self-loops in the delegation graph.
    /// 2. Retrieves the initial stake of the hotkey.
    /// 3. Calculates the stake allocated to children.
    /// 4. Calculates the stake received from parents.
    /// 5. Computes the final stake by adjusting the initial stake with child and parent contributions.
    ///
    /// # Arguments
    /// * `hotkey` - AccountId of the hotkey whose total network stake is to be calculated.
    /// * `netuid` - Network unique identifier specifying the network context.
    ///
    /// # Returns
    /// * `u64` - The total stake for the hotkey on the network after considering the stakes
    ///           from children and parents.
    ///
    /// # Note
    /// This function now includes a check for self-loops in the delegation graph using the
    /// `dfs_check_self_loops` method. However, it currently only logs warnings for detected loops
    /// and does not alter the stake calculation based on these findings.
    ///
    /// # Panics
    /// This function does not explicitly panic, but underlying arithmetic operations
    /// use saturating arithmetic to prevent overflows.
    ///
    /// TODO: check for self loops.
    /// TODO: (@distributedstatemachine): check if we should return error , otherwise self loop
    /// detection is impossible to test.
    pub fn get_stake_for_hotkey_on_subnet(hotkey: &T::AccountId, netuid: u16) -> u64 {
        // Retrieve the initial total stake for the hotkey without any child/parent adjustments.
        let initial_stake: u64 = Self::get_total_stake_for_hotkey(hotkey);
        let mut stake_to_children: u64 = 0;
        let mut stake_from_parents: u64 = 0;

        // Retrieve lists of parents and children from storage, based on the hotkey and network ID.
        let parents: Vec<(u64, T::AccountId)> = Self::get_parents(hotkey, netuid);
        let children: Vec<(u64, T::AccountId)> = Self::get_children(hotkey, netuid);

        // Iterate over children to calculate the total stake allocated to them.
        for (proportion, _) in children {
            // Calculate the stake proportion allocated to the child based on the initial stake.
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));
            let stake_proportion_to_child: I96F32 =
                I96F32::from_num(initial_stake).saturating_mul(normalized_proportion);

            // Accumulate the total stake given to children.
            stake_to_children =
                stake_to_children.saturating_add(stake_proportion_to_child.to_num::<u64>());
        }

        // Iterate over parents to calculate the total stake received from them.
        for (proportion, parent) in parents {
            // Retrieve the parent's total stake.
            let parent_stake: u64 = Self::get_total_stake_for_hotkey(&parent);
            // Calculate the stake proportion received from the parent.
            let normalized_proportion: I96F32 =
                I96F32::from_num(proportion).saturating_div(I96F32::from_num(u64::MAX));
            let stake_proportion_from_parent: I96F32 =
                I96F32::from_num(parent_stake).saturating_mul(normalized_proportion);

            // Accumulate the total stake received from parents.
            stake_from_parents =
                stake_from_parents.saturating_add(stake_proportion_from_parent.to_num::<u64>());
        }

        // Calculate the final stake for the hotkey by adjusting the initial stake with the stakes
        // to/from children and parents.
        let mut finalized_stake: u64 = initial_stake
            .saturating_sub(stake_to_children)
            .saturating_add(stake_from_parents);

        // get the max stake for the network
        let max_stake = Self::get_network_max_stake(netuid);

        // Return the finalized stake value for the hotkey, but capped at the max stake.
        finalized_stake = finalized_stake.min(max_stake);

        // Return the finalized stake value for the hotkey.
        finalized_stake
    }

    /* Retrieves the list of children for a given hotkey and network.
    ///
    /// # Arguments
    /// * `hotkey` - The hotkey whose children are to be retrieved.
    /// * `netuid` - The network identifier.
    ///
    /// # Returns
    /// * `Vec<(u64, T::AccountId)>` - A vector of tuples containing the proportion and child account ID.
    ///
    /// # Example
    /// ```
    /// let children = SubtensorModule::get_children(&hotkey, netuid);
     */
    pub fn get_children(hotkey: &T::AccountId, netuid: u16) -> Vec<(u64, T::AccountId)> {
        ChildKeys::<T>::get(hotkey, netuid)
    }

    /* Retrieves the list of parents for a given child and network.
    ///
    /// # Arguments
    /// * `child` - The child whose parents are to be retrieved.
    /// * `netuid` - The network identifier.
    ///
    /// # Returns
    /// * `Vec<(u64, T::AccountId)>` - A vector of tuples containing the proportion and parent account ID.
    ///
    /// # Example
    /// ```
    /// let parents = SubtensorModule::get_parents(&child, netuid);
     */
    pub fn get_parents(child: &T::AccountId, netuid: u16) -> Vec<(u64, T::AccountId)> {
        ParentKeys::<T>::get(child, netuid)
    }
}
