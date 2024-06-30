
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
use frame_support::storage::{IterableStorageMap};

impl<T: Config> Pallet<T> {


    /// Fetches the total count of subnets.
    ///
    /// This function retrieves the total number of subnets present on the chain.
    ///
    /// # Returns:
    /// * 'u16': The total number of subnets.
    ///
    pub fn get_num_subnets() -> u16 {
        TotalNetworks::<T>::get()
    }

    /// Fetches the max number of subnet
    ///
    /// This function retrieves the max number of subnet.
    ///
    /// # Returns:
    /// * 'u16': The max number of subnet
    ///
    pub fn get_max_subnets() -> u16 {
        SubnetLimit::<T>::get()
    }

    /// Sets the max number of subnet
    ///
    /// This function sets the max number of subnet.
    ///
    pub fn set_max_subnets(limit: u16) {
        SubnetLimit::<T>::put(limit);
        Self::deposit_event(Event::SubnetLimitSet(limit));
    }


    /// Returns the emission value for the given subnet.
    ///
    /// This function retrieves the emission value for the given subnet.
    ///
    /// # Returns:
    /// * 'u64': The emission value for the given subnet.
    ///
    pub fn get_subnet_emission_value(netuid: u16) -> u64 {
        EmissionValues::<T>::get(netuid)
    }

    /// Returns true if the subnetwork exists.
    ///
    /// This function checks if a subnetwork with the given UID exists.
    ///
    /// # Returns:
    /// * 'bool': Whether the subnet exists.
    ///
    pub fn if_subnet_exist(netuid: u16) -> bool {
        NetworksAdded::<T>::get(netuid)
    }

    /// Returns a list of subnet netuid equal to total networks.
    ///
    ///
    /// This iterates through all the networks and returns a list of netuids.
    ///
    /// # Returns:
    /// * 'Vec<u16>': Netuids of all subnets.
    ///
    pub fn get_all_subnet_netuids() -> Vec<u16> {
        <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter()
            .map(|(netuid, _)| netuid)
            .collect()
    }

    /// Sets the network rate limit and emit the `NetworkRateLimitSet` event
    ///
    pub fn set_network_rate_limit(limit: u64) {
        NetworkRateLimit::<T>::set(limit);
        Self::deposit_event(Event::NetworkRateLimitSet(limit));
    }

    /// Checks if registrations are allowed for a given subnet.
    ///
    /// This function retrieves the subnet hyperparameters for the specified subnet and checks the `registration_allowed` flag.
    /// If the subnet doesn't exist or doesn't have hyperparameters defined, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The unique identifier of the subnet.
    ///
    /// # Returns
    ///
    /// * `bool` - `true` if registrations are allowed for the subnet, `false` otherwise.
    pub fn is_registration_allowed(netuid: u16) -> bool {
        Self::get_subnet_hyperparams(netuid)
            .map(|params| params.registration_allowed)
            .unwrap_or(false)
    }
    
}