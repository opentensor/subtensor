use super::*;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::vec;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic do_set_child_singular: Sets a single child.
    ///
    /// This function allows a coldkey to set a single child for a given hotkey on a specified network.
    /// The proportion of the hotkey's stake to be allocated to the child is also specified.
    ///
    /// # Arguments:
    /// * `origin` (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the calling coldkey. Setting a hotkey child can only be done by the coldkey.
    ///
    /// * `hotkey` (T::AccountId):
    ///     - The hotkey which will be assigned the child.
    ///
    /// * `child` (T::AccountId):
    ///     - The child which will be assigned to the hotkey.
    ///
    /// * `netuid` (u16):
    ///     - The u16 network identifier where the childkey will exist.
    ///
    /// * `proportion` (u64):
    ///     - Proportion of the hotkey's stake to be given to the child, the value must be u64 normalized.
    ///
    /// # Events:
    /// * `ChildAddedSingular`:
    ///     - On successfully registering a child to a hotkey.
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
    pub fn do_set_child_singular(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        child: T::AccountId,
        netuid: u16,
        proportion: u64,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::trace!( "do_set_children_singular( coldkey:{:?} netuid:{:?} hotkey:{:?} child:{:?} proportion:{:?} )", coldkey, netuid, hotkey, child, proportion );

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

        // --- 5. Ensure that the child is not the hotkey.
        ensure!(child != hotkey, Error::<T>::InvalidChild);

        // --- 6. Erase myself from old children's parents.
        let old_children: Vec<(u64, T::AccountId)> = ChildKeys::<T>::get(hotkey.clone(), netuid);

        // --- 6.0. Iterate over all my old children and remove myself from their parent's map.
        for (_, old_child) in old_children.clone().iter() {
            // --- 6.1. Get the old child's parents on this network.
            let my_old_child_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(old_child.clone(), netuid);

            // --- 6.2. Filter my hotkey from my old children's parents list.
            let filtered_parents: Vec<(u64, T::AccountId)> = my_old_child_parents
                .into_iter()
                .filter(|(_, parent)| *parent != hotkey)
                .collect();

            // --- 6.3. Update the parent list in storage
            ParentKeys::<T>::insert(old_child, netuid, filtered_parents);
        }

        // --- 7. Create my new children + proportion list.
        let new_children: Vec<(u64, T::AccountId)> = vec![(proportion, child.clone())];

        // --- 7.1. Insert my new children + proportion list into the map.
        ChildKeys::<T>::insert(hotkey.clone(), netuid, new_children.clone());

        // --- 7.2. Update the parents list for my new children.
        for (proportion, new_child) in new_children.clone().iter() {
            // --- 8.2.1. Get the child's parents on this network.
            let mut new_child_previous_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(new_child.clone(), netuid);

            // --- 7.2.2. Append my hotkey and proportion to my new child's parents list.
            // NOTE: There are no duplicates possible because I previously removed my self from my old children.
            new_child_previous_parents.push((*proportion, hotkey.clone()));

            // --- 7.2.3. Update the parents list in storage.
            ParentKeys::<T>::insert(new_child.clone(), netuid, new_child_previous_parents);
        }

        // --- 8. Log and return.
        log::trace!(
            "SetChildSingular( hotkey:{:?}, child:{:?}, netuid:{:?}, proportion:{:?} )",
            hotkey,
            child,
            netuid,
            proportion
        );
        Self::deposit_event(Event::SetChildSingular(
            hotkey.clone(),
            child,
            netuid,
            proportion,
        ));

        // Ok and return.
        Ok(())
    }

    /// Sets multiple children for a given hotkey on a specified network.
    ///
    /// This function allows a coldkey to set multiple children for a given hotkey on a specified network.
    /// Each child is assigned a proportion of the hotkey's stake. This is an extension of the
    /// `do_set_child_singular` function, allowing for more efficient batch operations.
    ///
    /// # Arguments
    /// * `origin` (<T as frame_system::Config>::RuntimeOrigin):
    ///     The signature of the calling coldkey. Setting hotkey children can only be done by the associated coldkey.
    ///
    /// * `hotkey` (T::AccountId):
    ///     The hotkey which will be assigned the children.
    ///
    /// * `children_with_proportions` (Vec<(T::AccountId, u64)>):
    ///     A vector of tuples, each containing a child AccountId and its corresponding proportion.
    ///     The proportion must be a u64 normalized value (0 to u64::MAX).
    ///
    /// * `netuid` (u16):
    ///     The u16 network identifier where the childkeys will exist.
    ///
    /// # Events
    /// * `SetChildrenMultiple`:
    ///     Emitted when children are successfully registered to a hotkey.
    ///
    /// # Errors
    /// * `SubNetworkDoesNotExist`:
    ///     Thrown when attempting to register to a non-existent network.
    /// * `RegistrationNotPermittedOnRootSubnet`:
    ///     Thrown when attempting to register children on the root network.
    /// * `NonAssociatedColdKey`:
    ///     Thrown when the coldkey does not own the hotkey.
    /// * `InvalidChild`:
    ///     Thrown when any of the children is the same as the hotkey.
    ///
    /// # Detailed Workflow
    /// 1. Verify the transaction signature and ownership.
    /// 2. Perform various checks (network existence, root network, ownership, valid children).
    /// 3. Remove the hotkey from its old children's parent lists.
    /// 4. Create a new list of children with their proportions.
    /// 5. Update the ChildKeys storage with the new children.
    /// 6. Update the ParentKeys storage for each new child.
    /// 7. Emit an event to log the operation.
    ///
    /// # Example
    /// ```ignore
    /// let children_with_proportions = vec![(child1, 1000), (child2, 2000), (child3, 3000)];
    /// SubtensorModule::do_set_children_multiple(origin, hotkey, children_with_proportions, netuid);
    /// ```
    pub fn do_set_children_multiple(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        children_with_proportions: Vec<(T::AccountId, u64)>,
        netuid: u16,
    ) -> DispatchResult {
        // --- 1. Verify the transaction signature (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::trace!(
        "do_set_children_multiple( coldkey:{:?} netuid:{:?} hotkey:{:?} children_with_proportions:{:?} )",
        coldkey, netuid, hotkey, children_with_proportions
    );

        // --- 2. Ensure this operation is not on the root network (child hotkeys are not valid on root)
        ensure!(
            netuid != Self::get_root_netuid(),
            Error::<T>::RegistrationNotPermittedOnRootSubnet
        );

        // --- 3. Verify the specified network exists
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 4. Verify the coldkey owns the hotkey
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 5. Ensure none of the children are the same as the hotkey
        for (child, _) in &children_with_proportions {
            ensure!(*child != hotkey, Error::<T>::InvalidChild);
        }

        // --- 6. Remove the hotkey from its old children's parent lists
        let old_children: Vec<(u64, T::AccountId)> = ChildKeys::<T>::get(hotkey.clone(), netuid);

        // Iterate over all old children and remove the hotkey from their parent's map
        for (_, old_child) in old_children.iter() {
            let mut my_old_child_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(old_child.clone(), netuid);
            my_old_child_parents.retain(|(_, parent)| *parent != hotkey);
            ParentKeys::<T>::insert(old_child, netuid, my_old_child_parents);
        }

        // --- 7. Create the new children + proportion list
        let new_children: Vec<(u64, T::AccountId)> = children_with_proportions
            .into_iter()
            .map(|(child, proportion)| (proportion, child))
            .collect();

        // --- 8. Update the ChildKeys storage with the new children list
        ChildKeys::<T>::insert(hotkey.clone(), netuid, new_children.clone());

        // --- 9. Update the ParentKeys storage for each new child
        for (proportion, new_child) in new_children.iter() {
            let mut new_child_previous_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(new_child.clone(), netuid);
            new_child_previous_parents.push((*proportion, hotkey.clone()));
            ParentKeys::<T>::insert(new_child.clone(), netuid, new_child_previous_parents);
        }

        // --- 10. Log the operation and emit an event
        log::trace!(
            "SetChildrenMultiple( hotkey:{:?}, children:{:?}, netuid:{:?} )",
            hotkey,
            new_children,
            netuid
        );
        Self::deposit_event(Event::SetChildrenMultiple(
            hotkey.clone(),
            new_children,
            netuid,
        ));

        // --- 11. Return success
        Ok(())
    }

    /// ---- The implementation for the extrinsic do_revoke_child_singular: Revokes a single child.
    ///
    /// This function allows a coldkey to revoke a single child for a given hotkey on a specified network.
    ///
    /// # Arguments:
    /// * `origin` (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the calling coldkey. Revoking a hotkey child can only be done by the coldkey.
    ///
    /// * `hotkey` (T::AccountId):
    ///     - The hotkey from which the child will be revoked.
    ///
    /// * `child` (T::AccountId):
    ///     - The child which will be revoked from the hotkey.
    ///
    /// * `netuid` (u16):
    ///     - The u16 network identifier where the childkey exists.
    ///
    /// # Events:
    /// * `ChildRevokedSingular`:
    ///     - On successfully revoking a child from a hotkey.
    ///
    /// # Errors:
    /// * `SubNetworkDoesNotExist`:
    ///     - Attempting to revoke from a non-existent network.
    /// * `NonAssociatedColdKey`:
    ///     - The coldkey does not own the hotkey or the child is not associated with the hotkey.
    /// * `HotKeyAccountNotExists`:
    ///     - The hotkey account does not exist.
    ///
    pub fn do_revoke_child_singular(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        child: T::AccountId,
        netuid: u16,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_revoke_child_singular( coldkey:{:?} netuid:{:?} hotkey:{:?} child:{:?} )",
            coldkey,
            netuid,
            hotkey,
            child
        );

        // --- 2. Check that the network we are trying to revoke the child from exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Check that the coldkey owns the hotkey.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 4. Get the current children of the hotkey.
        let mut children: Vec<(u64, T::AccountId)> = ChildKeys::<T>::get(hotkey.clone(), netuid);

        // --- 5. Ensure that the child is actually a child of the hotkey.
        ensure!(
            children.iter().any(|(_, c)| c == &child),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 6. Remove the child from the hotkey's children list.
        children.retain(|(_, c)| c != &child);

        // --- 7. Update the children list in storage.
        ChildKeys::<T>::insert(hotkey.clone(), netuid, children.clone());

        // --- 8. Remove the hotkey from the child's parent list.
        let mut parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get(child.clone(), netuid);
        parents.retain(|(_, p)| p != &hotkey);
        ParentKeys::<T>::insert(child.clone(), netuid, parents);

        // --- 9. Log and return.
        log::info!(
            "RevokeChildSingular( hotkey:{:?}, child:{:?}, netuid:{:?} )",
            hotkey,
            child,
            netuid
        );
        Self::deposit_event(Event::RevokeChildSingular(hotkey.clone(), child, netuid));

        // Ok and return.
        Ok(())
    }

    /// Revokes multiple children for a given hotkey on a specified network.
    ///
    /// This function allows a coldkey to revoke multiple children for a given hotkey on a specified network.
    ///
    /// # Arguments:
    /// * `origin` (<T as frame_system::Config>::RuntimeOrigin):
    ///     - The signature of the calling coldkey. Revoking hotkey children can only be done by the coldkey.
    ///
    /// * `hotkey` (T::AccountId):
    ///     - The hotkey from which the children will be revoked.
    ///
    /// * `children` (Vec<T::AccountId>):
    ///     - A vector of AccountIds representing the children to be revoked.
    ///
    /// * `netuid` (u16):
    ///     - The u16 network identifier where the childkeys exist.
    ///
    /// # Events:
    /// * `ChildrenRevokedMultiple`:
    ///     - On successfully revoking multiple children from a hotkey.
    ///
    /// # Errors:
    /// * `SubNetworkDoesNotExist`:
    ///     - Attempting to revoke from a non-existent network.
    /// * `NonAssociatedColdKey`:
    ///     - The coldkey does not own the hotkey.
    /// * `HotKeyAccountNotExists`:
    ///     - The hotkey account does not exist.
    pub fn do_revoke_children_multiple(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        children: Vec<T::AccountId>,
        netuid: u16,
    ) -> DispatchResult {
        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::info!(
            "do_revoke_children_multiple( coldkey:{:?} netuid:{:?} hotkey:{:?} children:{:?} )",
            coldkey,
            netuid,
            hotkey,
            children
        );

        // --- 2. Check that the network we are trying to revoke the children from exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Check that the coldkey owns the hotkey.
        ensure!(
            Self::coldkey_owns_hotkey(&coldkey, &hotkey),
            Error::<T>::NonAssociatedColdKey
        );

        // --- 4. Get the current children of the hotkey.
        let mut current_children: Vec<(u64, T::AccountId)> =
            ChildKeys::<T>::get(hotkey.clone(), netuid);

        // --- 5. Remove the specified children from the hotkey's children list.
        current_children.retain(|(_, child)| !children.contains(child));

        // --- 6. Update the children list in storage.
        ChildKeys::<T>::insert(hotkey.clone(), netuid, current_children);

        // --- 7. Remove the hotkey from each child's parent list.
        for child in children.iter() {
            let mut parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get(child.clone(), netuid);
            parents.retain(|(_, p)| p != &hotkey);
            ParentKeys::<T>::insert(child.clone(), netuid, parents);
        }

        // --- 8. Log and return.
        log::info!(
            "RevokeChildrenMultiple( hotkey:{:?}, children:{:?}, netuid:{:?} )",
            hotkey,
            children,
            netuid
        );
        Self::deposit_event(Event::RevokeChildrenMultiple(
            hotkey.clone(),
            children,
            netuid,
        ));

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
    /// # Example
    /// ```ignore
    /// let total_stake = Self::get_stake_with_children_and_parents(&hotkey, netuid);
    /// ```
    /// TODO: check for self loops.
    /// TODO: (@distributedstatemachine): check if we should return error , otherwise self loop 
    /// detection is impossible to test. 

    pub fn get_stake_with_children_and_parents(hotkey: &T::AccountId, netuid: u16) -> u64 {
        let mut visited = BTreeSet::new();
        Self::dfs_check_self_loops(hotkey, netuid, &mut visited);

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
            let stake_proportion_to_child: I96F32 = I96F32::from_num(initial_stake)
                .saturating_mul(I96F32::from_num(proportion))
                .saturating_div(I96F32::from_num(u64::MAX));
            // Accumulate the total stake given to children.
            stake_to_children =
                stake_to_children.saturating_add(stake_proportion_to_child.to_num::<u64>());
        }

        // Iterate over parents to calculate the total stake received from them.
        for (proportion, parent) in parents {
            // Retrieve the parent's total stake.
            let parent_stake: u64 = Self::get_total_stake_for_hotkey(&parent);
            // Calculate the stake proportion received from the parent.
            let stake_proportion_from_parent: I96F32 = I96F32::from_num(parent_stake)
                .saturating_mul(I96F32::from_num(proportion))
                .saturating_div(I96F32::from_num(u64::MAX));

            // Accumulate the total stake received from parents.
            stake_from_parents =
                stake_from_parents.saturating_add(stake_proportion_from_parent.to_num::<u64>());
        }

        // Calculate the final stake for the hotkey by adjusting the initial stake with the stakes
        // to/from children and parents.
        let finalized_stake: u64 = initial_stake
            .saturating_sub(stake_to_children)
            .saturating_add(stake_from_parents);

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

    /// Performs a depth-first search to detect self-loops in the delegation graph.
    ///
    /// This function traverses the delegation graph starting from a given account,
    /// checking for circular dependencies (self-loops) in the process.
    ///
    /// # Arguments
    ///
    /// * `current` - A reference to the `AccountId` of the current node being examined.
    /// * `netuid` - The network ID in which to perform the check.
    /// * `visited` - A mutable reference to a `BTreeSet` keeping track of visited accounts.
    ///
    /// # Behavior
    ///
    /// - If a node is encountered that has already been visited, a warning is logged
    ///   indicating a self-loop has been detected.
    /// - The function recursively checks all children of the current node.
    /// - After checking all children, the current node is removed from the visited set
    ///   to allow for correct backtracking in the DFS algorithm.
    ///
    /// # Note
    ///
    /// This function does not return a Result or stop execution when a loop is detected.
    /// It only logs a warning. Depending on your requirements, you might want to modify
    /// this behavior to return an error or take other actions when a loop is found.
    fn dfs_check_self_loops(
        current: &T::AccountId,
        netuid: u16,
        visited: &mut BTreeSet<T::AccountId>,
    ) {
        if !visited.insert(current.clone()) {
            // We've encountered this node before, which means there's a loop
            log::warn!("Self-loop detected for account: {:?}", current);
        }

        let children = Self::get_children(current, netuid);
        for (_, child) in children {
            Self::dfs_check_self_loops(&child, netuid, visited);
        }

        visited.remove(current);
    }
}
