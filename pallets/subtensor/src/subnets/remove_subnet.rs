
// The MIT License (MIT)
// Copyright © 2023 Yuma Rao

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
// documentation files (the “Software”), to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software,
// and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or substantial portions of
// the Software.

// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
// THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use super::*;
use frame_support::storage::{IterableStorageDoubleMap};

impl<T: Config> Pallet<T> {

  /// Facilitates the removal of a user's subnetwork.
    ///
    /// # Args:
    /// * 'origin': ('T::RuntimeOrigin'): The calling origin. Must be signed.
    /// * 'netuid': ('u16'): The unique identifier of the network to be removed.
    ///
    /// # Event:
    /// * 'NetworkRemoved': Emitted when a network is successfully removed.
    ///
    /// # Raises:
    /// * 'SubNetworkDoesNotExist': If the specified network does not exist.
    /// * 'NotSubnetOwner': If the caller does not own the specified subnet.
    ///
    pub fn user_remove_network(origin: T::RuntimeOrigin, netuid: u16) -> dispatch::DispatchResult {
        // --- 1. Ensure the function caller is a signed user.
        let coldkey = ensure_signed(origin)?;

        // --- 2. Ensure this subnet exists.
        ensure!(
            Self::if_subnet_exist(netuid),
            Error::<T>::SubNetworkDoesNotExist
        );

        // --- 3. Ensure the caller owns this subnet.
        ensure!(
            SubnetOwner::<T>::get(netuid) == coldkey,
            Error::<T>::NotSubnetOwner
        );

        // --- 4. Explicitly erase the network and all its parameters.
        Self::remove_network(netuid);

        // --- 5. Emit the NetworkRemoved event.
        log::info!("NetworkRemoved( netuid:{:?} )", netuid);
        Self::deposit_event(Event::NetworkRemoved(netuid));

        // --- 6. Return success.
        Ok(())
    }

    /// Removes a network (identified by netuid) and all associated parameters.
    ///
    /// This function is responsible for cleaning up all the data associated with a network.
    /// It ensures that all the storage values related to the network are removed, and any
    /// reserved balance is returned to the network owner.
    ///
    /// # Args:
    ///  * 'netuid': ('u16'): The unique identifier of the network to be removed.
    ///
    /// # Note:
    /// This function does not emit any events, nor does it raise any errors. It silently
    /// returns if any internal checks fail.
    ///
    pub fn remove_network(netuid: u16) {
        // --- 1. Return balance to subnet owner.
        let owner_coldkey = SubnetOwner::<T>::get(netuid);
        let reserved_amount = Self::get_subnet_locked_balance(netuid);

        // --- 2. Remove network count.
        SubnetworkN::<T>::remove(netuid);

        // --- 3. Remove network modality storage.
        NetworkModality::<T>::remove(netuid);

        // --- 4. Remove netuid from added networks.
        NetworksAdded::<T>::remove(netuid);

        // --- 6. Decrement the network counter.
        TotalNetworks::<T>::mutate(|n| *n -= 1);

        // --- 7. Remove various network-related storages.
        NetworkRegisteredAt::<T>::remove(netuid);

        // --- 8. Remove incentive mechanism memory.
        let _ = Uids::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Keys::<T>::clear_prefix(netuid, u32::MAX, None);
        let _ = Bonds::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 8. Removes the weights for this subnet (do not remove).
        let _ = Weights::<T>::clear_prefix(netuid, u32::MAX, None);

        // --- 9. Iterate over stored weights and fill the matrix.
        for (uid_i, weights_i) in
            <Weights<T> as IterableStorageDoubleMap<u16, u16, Vec<(u16, u16)>>>::iter_prefix(
                Self::get_root_netuid(),
            )
        {
            // Create a new vector to hold modified weights.
            let mut modified_weights = weights_i.clone();
            // Iterate over each weight entry to potentially update it.
            for (subnet_id, weight) in modified_weights.iter_mut() {
                if subnet_id == &netuid {
                    // If the condition matches, modify the weight
                    *weight = 0; // Set weight to 0 for the matching subnet_id.
                }
            }
            Weights::<T>::insert(Self::get_root_netuid(), uid_i, modified_weights);
        }

        // --- 10. Remove various network-related parameters.
        Rank::<T>::remove(netuid);
        Trust::<T>::remove(netuid);
        Active::<T>::remove(netuid);
        Emission::<T>::remove(netuid);
        Incentive::<T>::remove(netuid);
        Consensus::<T>::remove(netuid);
        Dividends::<T>::remove(netuid);
        PruningScores::<T>::remove(netuid);
        LastUpdate::<T>::remove(netuid);
        ValidatorPermit::<T>::remove(netuid);
        ValidatorTrust::<T>::remove(netuid);

        // --- 11. Erase network parameters.
        Tempo::<T>::remove(netuid);
        Kappa::<T>::remove(netuid);
        Difficulty::<T>::remove(netuid);
        MaxAllowedUids::<T>::remove(netuid);
        ImmunityPeriod::<T>::remove(netuid);
        ActivityCutoff::<T>::remove(netuid);
        EmissionValues::<T>::remove(netuid);
        MaxWeightsLimit::<T>::remove(netuid);
        MinAllowedWeights::<T>::remove(netuid);
        RegistrationsThisInterval::<T>::remove(netuid);
        POWRegistrationsThisInterval::<T>::remove(netuid);
        BurnRegistrationsThisInterval::<T>::remove(netuid);

        // --- 12. Add the balance back to the owner.
        Self::add_balance_to_coldkey_account(&owner_coldkey, reserved_amount);
        Self::set_subnet_locked_balance(netuid, 0);
        SubnetOwner::<T>::remove(netuid);
    }

}