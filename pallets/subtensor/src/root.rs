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
        },
    },
    frame_support::
    {
        storage::
        {
            IterableStorageDoubleMap,
        },
    },
    sp_std::
    {
        vec,
        vec::
        {
            Vec
        }
    },
    substrate_fixed::
    {
        types::
        {
            I64F64
        }
    }
};

include!("emission.rs");
include!("user.rs");

impl<T: Config> Pallet<T> 
{
    // Fetches the total count of subnets.
    //
    // This function retrieves the total number of subnets present on the chain.
    //
    // # Returns:
    // * 'u16': The total number of subnets.
    //
    pub fn get_num_subnets() -> u16 
    {
        return TotalNetworks::<T>::get();
    }

    // Checks for any UIDs in the given list that are either equal to the root netuid or exceed the total number of subnets.
    //
    // It's important to check for invalid UIDs to ensure data integrity and avoid referencing nonexistent subnets.
    //
    // # Arguments:
    // * 'uids': A reference to a vector of UIDs to check.
    //
    // # Returns:
    // * 'bool': 'true' if any of the UIDs are invalid, 'false' otherwise.
    //
    pub fn contains_invalid_root_uids(netuids: &Vec<u16>) -> bool 
    {
        for netuid in netuids 
        {
            if !Self::if_subnet_exist(*netuid) 
            {
                log::debug!(
                    "contains_invalid_root_uids: netuid {:?} does not exist",
                    netuid
                );

                return true;
            }
        }

        return false;
    }

    // This function calculates the lock cost for a network based on the last lock amount, minimum lock cost, last lock block, and current block.
    // The lock cost is calculated using the formula:
    // lock_cost = (last_lock * mult) - (last_lock / lock_reduction_interval) * (current_block - last_lock_block)
    // where:
    // - last_lock is the last lock amount for the network
    // - mult is the multiplier which increases lock cost each time a registration occurs
    // - last_lock_block is the block number at which the last lock occurred
    // - lock_reduction_interval the number of blocks before the lock returns to previous value.
    // - current_block is the current block number
    // - DAYS is the number of blocks in a day
    // - min_lock is the minimum lock cost for the network
    //
    // If the calculated lock cost is less than the minimum lock cost, the minimum lock cost is returned.
    //
    // # Returns:
    // 	* 'u64':
    // 		- The lock cost for the network.
    //
    pub fn get_network_lock_cost() -> u64 
    {
        let last_lock:                  u64 = Self::get_network_last_lock();
        let min_lock:                   u64 = Self::get_network_min_lock();
        let last_lock_block:            u64 = Self::get_network_last_lock_block();
        let current_block:              u64 = Self::get_current_block_as_u64();
        let lock_reduction_interval:    u64 = Self::get_lock_reduction_interval();
        let mult:                       u64 = if last_lock_block == 0 { 1 } else { 2 };

        let mut lock_cost: u64  = last_lock
                                .saturating_mul(mult)
                                .saturating_sub(
                                    last_lock
                                    .saturating_div(lock_reduction_interval)
                                    .saturating_mul(
                                        current_block.saturating_sub(last_lock_block)
                                    )
                                );

        if lock_cost < min_lock 
        {
            lock_cost = min_lock;
        }

        log::debug!( "last_lock: {:?}, min_lock: {:?}, last_lock_block: {:?}, lock_reduction_interval: {:?}, current_block: {:?}, mult: {:?} lock_cost: {:?}",
        last_lock, min_lock, last_lock_block, lock_reduction_interval, current_block, mult, lock_cost);

        return lock_cost;
    }

    // This function is used to determine which subnet to prune when the total number of networks has reached the limit.
    // It iterates over all the networks and finds the oldest subnet with the minimum emission value that is not in the immunity period.
    //
    // # Returns:
    // 	* 'u16':
    // 		- The uid of the network to be pruned.
    //
    pub fn get_subnet_to_prune() -> u16 
    {
        let mut netuids:    Vec<u16>    = vec![];
        let current_block:  u64         = Self::get_current_block_as_u64();

        // Even if we don't have a root subnet, this still works
        for netuid in NetworksAdded::<T>::iter_keys_from(NetworksAdded::<T>::hashed_key_for(0)) 
        {
            if current_block.saturating_sub(Self::get_network_registered_block(netuid)) < Self::get_network_immunity_period() 
            {
                continue
            }

            // This iterator seems to return them in order anyways, so no need to sort by key
            netuids.push(netuid);
        }

        // Now we sort by emission, and then by subnet creation time.
        netuids.sort_by(|a, b| {
            use sp_std::cmp::Ordering;

            match Self::get_emission_value(*b).cmp(&Self::get_emission_value(*a)) 
            {
                Ordering::Equal => 
                {
                    if Self::get_network_registered_block(*b) < Self::get_network_registered_block(*a) 
                    {
                        Ordering::Less
                    }
                    else 
                    {
                        Ordering::Equal
                    }
                },

                v => v
            }
        });

        log::info!("{:?}", netuids);

        match netuids.last() 
        {
            Some(netuid) => 
            {
                return *netuid;
            },
            None =>
            {
                return 0;
            }
        }
    }
}
