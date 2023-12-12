impl<T: Config> Pallet<T>
{
    // Returns the emission value for the given subnet.
    //
    // This function retrieves the emission value for the given subnet.
    //
    // # Returns:
    // * 'u64': The emission value for the given subnet.
    //
    pub fn get_subnet_emission_value(netuid: u16) -> u64 
    {
        return EmissionValues::<T>::get(netuid);
    }

    // Sets the emission values for each netuid
    //
    //
    pub fn set_emission_values(netuids: &Vec<u16>, emission: Vec<u64>) -> Result<(), &'static str> 
    {
        log::debug!(
            "set_emission_values: netuids: {:?} emission:{:?}",
            netuids,
            emission
        );

        // Be careful this function can fail.
        if Self::contains_invalid_root_uids(netuids) 
        {
            log::error!("set_emission_values: contains_invalid_root_uids");

            return Err("Invalid netuids");
        }

        if netuids.len() != emission.len() 
        {
            log::error!("set_emission_values: netuids.len() != emission.len()");

            return Err("netuids and emission must have the same length");
        }

        for (i, netuid_i) in netuids.iter().enumerate() 
        {
            log::debug!("set netuid:{:?} emission:{:?}", netuid_i, emission[i]);

            EmissionValues::<T>::insert(*netuid_i, emission[i]);
        }

        return Ok(());
    }
}