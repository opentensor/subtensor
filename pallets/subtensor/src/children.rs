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
    /// * `proportion` (u16):
    ///     - Proportion of the hotkey's stake to be given to the child, the value must be u16 normalized.
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
        proportion: u16,
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
    /// * `children_with_proportions` (Vec<(T::AccountId, u16)>):
    ///     A vector of tuples, each containing a child AccountId and its corresponding proportion.
    ///     The proportion must be a u16 normalized value (0 to u16::MAX).
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
    /// 3. Ensure the sum of proportions equals u16::MAX.
    /// 4. Remove the hotkey from its old children's parent lists.
    /// 5. Create a new list of children with their proportions.
    /// 6. Update the ChildKeys storage with the new children.
    /// 7. Update the ParentKeys storage for each new child.
    /// 8. Emit an event to log the operation.
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

        let mut child_set: BTreeSet<T::AccountId> = BTreeSet::new();
        // --- 5. Ensure none of the children are the same as the hotkey
        for (child, _) in &children_with_proportions {
            ensure!(*child != hotkey, Error::<T>::InvalidChild);
            ensure!(child_set.insert(child.clone()), Error::<T>::DuplicateChild);
        }

        // --- 6. Ensure the sum of proportions equals u64::MAX (representing 100%)
        let (overflowed, total_proportion): (bool, u16) = {
            let mut sum: u16 = 0;
            let mut overflowed = false;
            for (_, proportion) in children_with_proportions.iter() {
                let result = sum.checked_add(*proportion);
                if let Some(data) = result {
                    sum = data;
                } else {
                    overflowed = true;
                    break;
                }
            }
            (overflowed, sum)
        };

        ensure!(
            !overflowed && total_proportion == u16::MAX,
            Error::<T>::ProportionSumIncorrect
        );

        // --- 7. Remove the hotkey from its old children's parent lists
        let old_children: Vec<(u64, T::AccountId)> = ChildKeys::<T>::get(hotkey.clone(), netuid);

        // Iterate over all old children and remove the hotkey from their parent's map
        for (_, old_child) in old_children.iter() {
            let mut my_old_child_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(old_child.clone(), netuid);
            my_old_child_parents.retain(|(_, parent)| *parent != hotkey);
            ParentKeys::<T>::insert(old_child, netuid, my_old_child_parents);
        }

        // --- 8. Create the new children + proportion list
        let new_children: Vec<(u64, T::AccountId)> = children_with_proportions
            .into_iter()
            .map(|(child, proportion)| (proportion, child))
            .collect();

        // --- 9. Update the ChildKeys storage with the new children list
        ChildKeys::<T>::insert(hotkey.clone(), netuid, new_children.clone());

        // --- 10. Update the ParentKeys storage for each new child
        for (proportion, new_child) in new_children.iter() {
            let mut new_child_previous_parents: Vec<(u64, T::AccountId)> =
                ParentKeys::<T>::get(new_child.clone(), netuid);
            new_child_previous_parents.push((*proportion, hotkey.clone()));
            ParentKeys::<T>::insert(new_child.clone(), netuid, new_child_previous_parents);
        }

        // --- 11. Log the operation and emit an event
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

        // Ok and return.
        Ok(())
    }

}
