use super::*;
use alloc::collections::BTreeMap;
use substrate_fixed::types::I96F32;
use crate::coinbase::run_coinbase::SubnetInfo;

impl<T: Config> Pallet<T> {

    /// Handles the emission for the RAO (Relative Alpha Optimization) mechanism.
    ///
    /// This function determines how to distribute the emission based on the ratio of TAO to Alpha
    /// across all subnets using this mechanism.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID of the subnet.
    /// * `block_emission` - The total emission for the current block.
    /// * `mech_emission` - The emission allocated to this mechanism.
    /// * `subnet_emission` - The emission allocated to this specific subnet.
    /// * `subnet_info` - A map containing information about all subnets.
    pub fn sink_rao_emission(
        netuid: u16,
        _block_emission: I96F32,
        mech_emission: I96F32,
        _subnet_emission: I96F32,
        subnet_info: &BTreeMap<u16, SubnetInfo>,
    ) {
        // Check if the sum of tao/alpha_in ratios exceeds 1 for all subnets with this mechanism
        // Initialize the sum of TAO/Alpha ratios
        let mut price_sum = I96F32::from_num(0);
        
        // Iterate through all subnets
        for (_, info) in subnet_info.iter() {
            // Check if the subnet uses the RAO mechanism (mechanism 2)
            if info.mechanism == 2 {
                // Calculate the TAO/Alpha ratio for this subnet
                // If alpha_in is zero, default to zero to avoid division by zero
                let subnet_price = info.tao.checked_div(info.alpha_in).unwrap_or(I96F32::from_num(0));
                
                // Add this subnet's ratio to the total sum
                price_sum += subnet_price;
            }
        }
        
        // Switch on prices.
        if price_sum > I96F32::from_num(1) {
            // If the sum of tao/alpha_in ratios exceeds 1, double the emission
            // This incentivizes subnets to maintain a balanced tao/alpha ratio
            
            // Add double the mechanism emission to the subnet's alpha input
            SubnetAlphaIn::<T>::mutate(netuid, |total| { 
                *total = total.saturating_add(mech_emission.to_num::<u64>() * 2) 
            });
            
            // Set the pending emission to double the mechanism emission
            // This ensures the subnet receives increased rewards for maintaining balance
            PendingEmission::<T>::mutate(netuid, |total| { 
                *total = total.saturating_add(mech_emission.to_num::<u64>() * 2) 
            });
        } else {
            // If the sum of tao/alpha_in ratios is 1 or less, halve the emission
            // This encourages subnets to increase their tao/alpha ratio
            
            // Add half the mechanism emission to the subnet's alpha input
            SubnetAlphaIn::<T>::mutate(netuid, |total| { 
                *total = total.saturating_add(mech_emission.to_num::<u64>() / 2 ) 
            });
            
            // Set the pending emission to half the mechanism emission
            // This reduces rewards for subnets with low tao/alpha ratios
            PendingEmission::<T>::mutate(netuid, |total| { 
                *total = total.saturating_add(mech_emission.to_num::<u64>() / 2 ) 
            });
        }
    }

    /// Handles the emission for the DTAO (Dynamic TAO) mechanism.
    ///
    /// This function adds the total mechanism emission to the subnet's alpha and sets the pending
    /// emission.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID of the subnet.
    /// * `block_emission` - The total emission for the current block.
    /// * `mech_emission` - The emission allocated to this mechanism.
    /// * `subnet_emission` - The emission allocated to this specific subnet.
    /// * `subnet_info` - A map containing information about all subnets.
    pub fn sink_dtao_emission(
        netuid: u16,
        _block_emission: I96F32,
        mech_emission: I96F32,
        _subnet_emission: I96F32,
        _subnet_info: &BTreeMap<u16, SubnetInfo>,
    ) {
        // Dynamic TAO (DTAO) emission handling

        // Update the subnet's alpha input
        SubnetAlphaIn::<T>::mutate(netuid, |total| { 
            // Add the full mechanism emission to the subnet's alpha input
            *total = total.saturating_add(mech_emission.to_num::<u64>()) 
        });

        // Update the pending emission for the subnet
        PendingEmission::<T>::mutate(netuid, |total| { 
            // Set the pending emission to the full mechanism emission.
            *total = total.saturating_add(mech_emission.to_num::<u64>()) 
        });
    }

    /// Handles the emission for the STAO (Stable TAO) mechanism.
    ///
    /// This function ensures that the alpha input equals the alpha output and sets the pending
    /// emission based on the subnet's emission.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID of the subnet.
    /// * `block_emission` - The total emission for the current block.
    /// * `mech_emission` - The emission allocated to this mechanism.
    /// * `subnet_emission` - The emission allocated to this specific subnet.
    /// * `subnet_info` - A map containing information about all subnets.
    pub fn sink_stao_emission(
        netuid: u16,
        _block_emission: I96F32,
        _mech_emission: I96F32,
        subnet_emission: I96F32,
        _subnet_info: &BTreeMap<u16, SubnetInfo>,
    ) {
        // Stable: ALPHA in == ALPHA out.
        SubnetAlphaIn::<T>::mutate(netuid, |total| { *total = total.saturating_add(subnet_emission.to_num::<u64>()) });
        // Stable: Set the pending emission as tao emission.
        PendingEmission::<T>::mutate(netuid, |total| { *total = total.saturating_add(subnet_emission.to_num::<u64>()) });
    }
    
}