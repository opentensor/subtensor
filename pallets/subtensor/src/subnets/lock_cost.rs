


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
use sp_std::vec;

impl<T: Config> Pallet<T> {

    /// Calculates the lock cost for a network based on various parameters.
    ///
    /// This function determines the cost to lock tokens for creating a new subnet. The cost
    /// increases each time a registration occurs and decreases over time.
    ///
    /// # Calculation
    /// The lock cost is calculated using the formula:
    /// lock_cost = (last_lock * mult) - (last_lock / lock_reduction_interval) * (current_block - last_lock_block)
    ///
    /// # Parameters used
    /// - last_lock: The previous lock amount for the network
    /// - mult: Multiplier that increases lock cost each time a registration occurs
    /// - last_lock_block: The block number at which the last lock occurred
    /// - lock_reduction_interval: Number of blocks before the lock returns to the previous value
    /// - current_block: The current block number
    /// - min_lock: The minimum lock cost for the network
    ///
    /// # Returns
    /// * `u64` - The calculated lock cost for the network, never less than the minimum lock cost.
    pub fn get_network_lock_cost() -> u64 {
        // Retrieve necessary parameters
        let last_lock = Self::get_network_last_lock();
        let min_lock = Self::get_network_min_lock();
        let last_lock_block = Self::get_network_last_lock_block();
        let current_block = Self::get_current_block_as_u64();
        let lock_reduction_interval = Self::get_lock_reduction_interval();

        // Determine multiplier: 2 if not the first lock, otherwise 1
        let mult = if last_lock_block == 0 { 1 } else { 2 };

        // Calculate the lock cost
        let mut lock_cost = last_lock
            .saturating_mul(mult)
            .saturating_sub(
                last_lock
                    .saturating_div(lock_reduction_interval)
                    .saturating_mul(current_block.saturating_sub(last_lock_block))
            );

        // Ensure lock cost is not less than the minimum
        if lock_cost < min_lock {
            lock_cost = min_lock;
        }

        // Log debug information
        log::debug!(
            "Lock cost calculation: last_lock: {:?}, min_lock: {:?}, last_lock_block: {:?}, \
            lock_reduction_interval: {:?}, current_block: {:?}, mult: {:?}, lock_cost: {:?}",
            last_lock, min_lock, last_lock_block, lock_reduction_interval, current_block, mult, lock_cost
        );

        lock_cost
    }

    /// Determines which subnet to prune when the total number of networks has reached the limit.
    ///
    /// This function iterates over all networks and finds the oldest subnet with the minimum
    /// emission value that is not in the immunity period.
    ///
    /// # Returns
    /// * `u16` - The uid of the network to be pruned.
    pub fn get_subnet_to_prune() -> u16 {
        let mut netuids: Vec<u16> = vec![];
        let current_block = Self::get_current_block_as_u64();

        // Collect eligible networks (not in immunity period)
        for netuid in NetworksAdded::<T>::iter_keys_from(NetworksAdded::<T>::hashed_key_for(0)) {
            let network_age = current_block.saturating_sub(Self::get_network_registered_block(netuid));
            if network_age >= Self::get_network_immunity_period() {
                netuids.push(netuid);
            }
        }

        // Sort networks by emission value (descending) and then by creation time
        netuids.sort_by(|a, b| {
            use sp_std::cmp::Ordering;

            match Self::get_emission_value(*b).cmp(&Self::get_emission_value(*a)) {
                Ordering::Equal => Self::get_network_registered_block(*b).cmp(&Self::get_network_registered_block(*a)),
                other => other,
            }
        });

        log::info!("Sorted network UIDs for pruning consideration: {:?}", netuids);

        // Return the last (oldest, lowest emission) network, or 0 if no networks are eligible
        *netuids.last().unwrap_or(&0)
    }

    /// Retrieves the block number at which a network was registered.
    ///
    /// # Arguments
    /// * `netuid` - The unique identifier of the network.
    ///
    /// # Returns
    /// * `u64` - The block number when the network was registered.
    pub fn get_network_registered_block(netuid: u16) -> u64 {
        NetworkRegisteredAt::<T>::get(netuid)
    }

    /// Retrieves the current network immunity period.
    ///
    /// # Returns
    /// * `u64` - The current immunity period for networks.
    pub fn get_network_immunity_period() -> u64 {
        NetworkImmunityPeriod::<T>::get()
    }

    /// Sets a new network immunity period.
    ///
    /// # Arguments
    /// * `net_immunity_period` - The new immunity period to set.
    pub fn set_network_immunity_period(net_immunity_period: u64) {
        NetworkImmunityPeriod::<T>::set(net_immunity_period);
        Self::deposit_event(Event::NetworkImmunityPeriodSet(net_immunity_period));
    }

    /// Sets a new minimum lock cost for networks.
    ///
    /// # Arguments
    /// * `net_min_lock` - The new minimum lock cost to set.
    pub fn set_network_min_lock(net_min_lock: u64) {
        NetworkMinLockCost::<T>::set(net_min_lock);
        Self::deposit_event(Event::NetworkMinLockCostSet(net_min_lock));
    }

    /// Retrieves the current minimum lock cost for networks.
    ///
    /// # Returns
    /// * `u64` - The current minimum lock cost.
    pub fn get_network_min_lock() -> u64 {
        NetworkMinLockCost::<T>::get()
    }

    /// Sets the last lock cost for networks.
    ///
    /// # Arguments
    /// * `net_last_lock` - The last lock cost to set.
    pub fn set_network_last_lock(net_last_lock: u64) {
        NetworkLastLockCost::<T>::set(net_last_lock);
    }

    /// Retrieves the last lock cost for networks.
    ///
    /// # Returns
    /// * `u64` - The last lock cost.
    pub fn get_network_last_lock() -> u64 {
        NetworkLastLockCost::<T>::get()
    }

    /// Retrieves the block number of the last network registration.
    ///
    /// # Returns
    /// * `u64` - The block number of the last network registration.
    pub fn get_network_last_lock_block() -> u64 {
        NetworkLastRegistered::<T>::get()
    }

    /// Sets the interval for lock cost reduction.
    ///
    /// # Arguments
    /// * `interval` - The new interval for lock cost reduction.
    pub fn set_lock_reduction_interval(interval: u64) {
        NetworkLockReductionInterval::<T>::set(interval);
        Self::deposit_event(Event::NetworkLockCostReductionIntervalSet(interval));
    }

    /// Retrieves the current interval for lock cost reduction.
    ///
    /// # Returns
    /// * `u64` - The current interval for lock cost reduction.
    pub fn get_lock_reduction_interval() -> u64 {
        NetworkLockReductionInterval::<T>::get()
    }
    
}