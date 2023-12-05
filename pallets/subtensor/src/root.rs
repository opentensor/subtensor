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
        pallet::
        {
            *
        }
    },
    frame_support::
    {
        dispatch::
        {
            DispatchResultWithPostInfo,
            Pays
        },
        storage::
        {
            IterableStorageDoubleMap,
            IterableStorageMap
        },
        traits::
        {
            Get
        },
        weights::
        {
            Weight
        }
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
            I32F32,
            I64F64
        }
    }
};

include!("emission.rs");
include!("user.rs");

impl<T: Config> Pallet<T> 
{
    // Retrieves the unique identifier (UID) for the root network.
    //
    // The root network is a special case and has a fixed UID of 0.
    //
    // # Returns:
    // * 'u16': The UID for the root network.
    //
    pub fn get_root_netuid() -> u16 
    {
        return 0;
    }

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

    // Fetches the total count of subnet validators (those that set weights.)
    //
    // This function retrieves the total number of subnet validators.
    //
    // # Returns:
    // * 'u16': The total number of validators
    //
    pub fn get_num_root_validators() -> u16 
    {
        return Self::get_subnetwork_n(Self::get_root_netuid());
    }

    // Fetches the total allowed number of root validators.
    //
    // This function retrieves the max allowed number of validators
    // it is equal to SenateMaxMembers
    //
    // # Returns:
    // * 'u16': The max allowed root validators.
    //
    pub fn get_max_root_validators() -> u16 
    {
        return Self::get_max_allowed_uids(Self::get_root_netuid());
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

    // Computes and sets emission values for the root network which determine the emission for all subnets.
    //
    // This function is responsible for calculating emission based on network weights, stake values,
    // and registered hotkeys.
    //
    pub fn root_epoch(block_number: u64) -> Result<(), &'static str> 
    {
        // --- 0. The unique ID associated with the root network.
        let root_netuid: u16 = Self::get_root_netuid();

        // --- 1. Check if we should update the emission values based on blocks since emission was last set.
        {
            let blocks_until_next_epoch: u64 = Self::blocks_until_next_epoch(root_netuid, Self::get_tempo(root_netuid), block_number);
            if blocks_until_next_epoch != 0 
            {
                // Not the block to update emission values.
                log::debug!("blocks_until_next_epoch: {:?}", blocks_until_next_epoch);
                return Err("Not the block to update emission values.");
            }
        }

        // --- 2. Retrieves the number of root validators on subnets.
        let n: u16;
        {
            n = Self::get_num_root_validators();
            log::debug!("n:\n{:?}\n", n);

            if n == 0 // No validators.
            {
                return Err("No validators to validate emission values.");
            }
        }

        // --- 3. Obtains the number of registered subnets.
        let k: u16;
        {
            k = Self::get_all_subnet_netuids().len() as u16;
            
            log::debug!("k:\n{:?}\n", k);

            if k == 0 // No networks to validate. 
            {
                return Err("No networks to validate emission values.");
            }
        }

        // --- 4. Determines the total block emission across all the subnetworks. This is the
        // value which will be distributed based on the computation below.
        let block_emission: I64F64;
        {
            block_emission = I64F64::from_num(Self::get_block_emission());

            log::debug!("block_emission:\n{:?}\n", block_emission);
        }

        // --- 5. A collection of all registered hotkeys on the root network. Hotkeys
        // pairs with network UIDs and stake values.
        let mut hotkeys: Vec<(u16, T::AccountId)>;
        {
            hotkeys = vec![];

            for (uid_i, hotkey) in <Keys<T> as IterableStorageDoubleMap<u16, u16, T::AccountId>>::iter_prefix(root_netuid)
            {
                hotkeys.push((uid_i, hotkey));
            }

            log::debug!("hotkeys:\n{:?}\n", hotkeys);
        }

        // --- 6. Retrieves and stores the stake value associated with each hotkey on the root network.
        // Stakes are stored in a 64-bit fixed point representation for precise calculations.
        let mut stake_i64: Vec<I64F64>;
        {
            stake_i64 = vec![I64F64::from_num(0.0); n as usize];
            for (uid_i, hotkey) in hotkeys.iter() 
            {
                stake_i64[*uid_i as usize] = I64F64::from_num(Self::get_total_stake_for_hotkey(hotkey));
            }

            inplace_normalize_64(&mut stake_i64);

            log::debug!("S:\n{:?}\n", &stake_i64);
        }

        // --- 8. Retrieves the network weights in a 2D Vector format. Weights have shape
        // n x k where is n is the number of registered peers and k is the number of subnets.
        let weights: Vec<Vec<I64F64>>;
        {
            weights = Self::get_root_weights();
            
            log::debug!("W:\n{:?}\n", &weights);
        }

        // --- 9. Calculates the rank of networks. Rank is a product of weights and stakes.
        // Ranks will have shape k, a score for each subnet.
        let ranks: Vec<I64F64>;
        {
            ranks = matmul_64(&weights, &stake_i64);
            
            log::debug!("R:\n{:?}\n", &ranks);
        }

        // --- 10. Calculates the trust of networks. Trust is a sum of all stake with weights > 0.
        // Trust will have shape k, a score for each subnet.
        let mut trust;
        let mut total_stake: I64F64;
        let total_networks;
        {
            total_networks  = Self::get_num_subnets();
            trust           = vec![I64F64::from_num(0); total_networks as usize];
            total_stake     = I64F64::from_num(0);

            for (idx, weights) in weights.iter().enumerate() 
            {
                let hotkey_stake = stake_i64[idx];
                total_stake += hotkey_stake;

                for (weight_idx, weight) in weights.iter().enumerate() 
                {
                    if *weight > 0 
                    {
                        trust[weight_idx] += hotkey_stake;
                    }
                }
            }

            log::debug!("T_before normalization:\n{:?}\n", &trust);
            log::debug!("Total_stake:\n{:?}\n", &total_stake);
    
            if total_stake == 0 
            {
                return Err("No stake on network")
            }
    
            for trust_score in trust.iter_mut() 
            {
                match trust_score.checked_div(total_stake) 
                {
                    Some(quotient) => 
                    {
                        *trust_score = quotient;
                    }

                    None => {}
                }
            }
        }

        // --- 11. Calculates the consensus of networks. Consensus is a sigmoid normalization of the trust scores.
        // Consensus will have shape k, a score for each subnet.
        let mut weighted_emission;
        {
            log::debug!("T:\n{:?}\n", &trust);

            let one = I64F64::from_num(1);
            let mut consensus = vec![I64F64::from_num(0); total_networks as usize];
            for (idx, trust_score) in trust.iter_mut().enumerate() 
            {
                let shifted_trust               = *trust_score 
                                                - I64F64::from_num(Self::get_float_kappa(0)); // Range( -kappa, 1 - kappa )

                let temperatured_trust          = shifted_trust 
                                                * I64F64::from_num(Self::get_rho(0)); // Range( -rho * kappa, rho ( 1 - kappa ) )

                let exponentiated_trust: I64F64 = substrate_fixed::transcendental::exp(-temperatured_trust)
                                                .expect("temperatured_trust is on range( -rho * kappa, rho ( 1 - kappa ) )");

                consensus[idx]                  = one / (one + exponentiated_trust);
            }

            log::debug!("C:\n{:?}\n", &consensus);

            weighted_emission = vec![I64F64::from_num(0); total_networks as usize];
            for (idx, emission) in weighted_emission.iter_mut().enumerate() 
            {
                *emission = consensus[idx] * ranks[idx];
            }
            inplace_normalize_64(&mut weighted_emission);

            log::debug!("Ei64:\n{:?}\n", &weighted_emission);
        }

        // -- 11. Converts the normalized 64-bit fixed point rank values to u64 for the final emission calculation.
        let emission_as_tao: Vec<I64F64>;
        {
            emission_as_tao = weighted_emission
                .iter()
                .map(|v: &I64F64| *v * block_emission)
                .collect();
        }

        // --- 12. Converts the normalized 64-bit fixed point rank values to u64 for the final emission calculation.
        let emission_u64: Vec<u64>;
        {
            emission_u64 = vec_fixed64_to_u64(emission_as_tao);

            log::debug!("Eu64:\n{:?}\n", &emission_u64);
        }

        // --- 13. Set the emission values for each subnet directly.
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        log::debug!("netuids: {:?} values: {:?}", netuids, emission_u64);

        return Self::set_emission_values(&netuids, emission_u64);
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
