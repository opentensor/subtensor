
use super::*;
use crate::math::*;
use frame_support::IterableStorageDoubleMap;
use sp_std::vec;
use substrate_fixed::types::{I32F32, I64F64, I96F32};

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
        proportion: u64
    ) -> DispatchResult {

        // --- 1. Check that the caller has signed the transaction. (the coldkey of the pairing)
        let coldkey = ensure_signed(origin)?;
        log::info!( "do_set_children_singular( coldkey:{:?} netuid:{:?} hotkey:{:?} child:{:?} proportion:{:?} )", coldkey, netuid, hotkey, child, proportion );

        // --- 2. Check that this delegation is not on the root network. Child hotkeys are not valid on root.
        ensure!( netuid != Self::get_root_netuid(), Error::<T>::RegistrationNotPermittedOnRootSubnet );

        // --- 3. Check that the network we are trying to create the child on exists.
        ensure!( Self::if_subnet_exist(netuid), Error::<T>::SubNetworkDoesNotExist );

        // --- 4. Check that the coldkey owns the hotkey.
        ensure!( Self::coldkey_owns_hotkey(hotkey), Error::<T>::NonAssociatedColdKey );

        // --- 5. Ensure that the hotkey account exists already (this is only possible through registration).
        ensure!( Self::hotkey_account_exists(&hotkey), Error::<T>::HotKeyAccountNotExists);

        // --- 6. Ensure that the child is not the hotkey.
        ensure!( child != hotkey, Error::<T>::InvalidChild );

        // --- 7. Erase myself from old children's parents.
        let old_children: Vec<(u64, T::AccountId)> = ChildKeys::<T>::get( hotkey, netuid );

        // --- 7.0. Iterate over all my old children and remove myself from their parent's map.
        for (_, old_child) in old_children {

            // --- 7.1. Get the old child's parents on this network.
            let my_old_child_parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get( old_child, netuid );

            // --- 7.2. Filter my hotkey from my old children's parents list.
            let filtered_parents: Vec<(u64, T::AccountId)> = my_old_child_parents
                .into_iter()
                .filter(|(_, parent)| *parent != hotkey)
                .collect();

            // --- 7.3. Update the parent list in storage
            ParentKeys::<T>::insert( old_child, netuid, filtered_parents );
        }

        // --- 8. Create my new children + proportion list.
        let new_children: Vec<(u64, T::AccountId)> = vec![ (proportion, child) ];

        // --- 8.1. Insert my new children + proportion list into the map.
        ChildKeys::<T>::insert( hotkey, netuid, new_children );

        // --- 8.2. Update the parents list for my new children.
        for (proportion, new_child) in new_children {
            
            // --- 8.2.1. Get the child's parents on this network.
            let new_child_previous_parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get( new_child, netuid );

            // --- 8.2.2. Append my hotkey and proportion to my new child's parents list.
            // NOTE: There are no duplicates possible because I previously removed my self from my old children.
            new_child_previous_parents.push((proportion, hotkey.clone()));

            // --- 8.2.3. Update the parents list in storage.
            ParentKeys::<T>::insert( new_child, netuid, new_child_previous_parents );
        }

        // --- 9. Log and return.
        log::info!(
            "SetChildSingular( hotkey:{:?}, child:{:?}, netuid:{:?}, proportion:{:?} )",
            hotkey,
            child,
            netuid,
            proportion
        );
        Self::deposit_event(Event::SetChildSingular(hotkey, child, netuid, proportion));

        // Ok and return.
        Ok(())
    }


    /// Function which returns the amount of stake held by a hotkey on the network after applying
    /// child/parent relationships.
    ///
    /// # Arguments
    /// * `hotkey` - AccountId of the hotkey whose total network stake is to be calculated.
    /// * `netuid` - Network unique identifier to specify the network context.
    ///
    /// # Returns
    /// * `u64` - The total stake for the hotkey on the network after considering the stakes
    ///           from children and parents.
    pub fn get_stake_with_children_and_parents( hotkey: T::AccountId, netuid: u16 ) -> u64 {
        // Retrieve the initial total stake for the hotkey without any child/parent adjustments.
        let initial_stake: u64 = Self::get_total_stake_for_hotkey(hotkey);
        let stake_to_children: u64 = 0;
        let stake_from_parents: u64 = 0;

        // Retrieve lists of parents and children from storage, based on the hotkey and network ID.
        let parents: Vec<(u64, T::AccountId)> = ParentKeys::<T>::get(hotkey, netuid);
        let children: Vec<(u64, T::AccountId)> = ChildKeys::<T>::get(hotkey, netuid);

        // Iterate over children to calculate the total stake allocated to them.
        for (proportion, child) in children {
            // Calculate the stake proportion allocated to the child based on the initial stake.
            let stake_proportion_to_child: I96F32 = I96F32::from_num(initial_stake) * I96F32::from_num(proportion) / I96F32::from_num(u64::MAX);
            // Accumulate the total stake given to children.
            stake_to_children += stake_proportion_to_child.to_num::<u64>();
        }

        // Iterate over parents to calculate the total stake received from them.
        for (proportion, parent) in parents {
            // Retrieve the parent's total stake.
            let parent_stake: u64 = Self::get_total_stake_for_hotkey(parent);
            // Calculate the stake proportion received from the parent.
            let stake_proportion_from_parent: I96F32 = I96F32::from_num(parent_stake) * I96F32::from_num(proportion) / I96F32::from_num(u64::MAX);
            // Accumulate the total stake received from parents.
            stake_from_parents += stake_proportion_from_parent.to_num::<u64>();
        }

        // Calculate the final stake for the hotkey by adjusting the initial stake with the stakes
        // to/from children and parents.
        let finalized_stake: u64 = initial_stake - stake_to_children + stake_from_parents;

        // Return the finalized stake value for the hotkey.
        return finalized_stake;
    }
    
}