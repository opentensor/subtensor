impl<T: Config> Pallet<T>
{
    // ---- The implementation for the extrinsic serve_axon which sets the ip endpoint information for a uid on a network.
    //
    // # Args:
    // 	* 'origin': (<T as frame_system::Config>RuntimeOrigin):
    // 		- The signature of the caller.
    //
    // 	* 'netuid' (u16):
    // 		- The u16 network identifier.
    //
    // 	* 'version' (u64):
    // 		- The bittensor version identifier.
    //
    // 	* 'ip' (u64):
    // 		- The endpoint ip information as a u128 encoded integer.
    //
    // 	* 'port' (u16):
    // 		- The endpoint port information as a u16 encoded integer.
    // 
    // 	* 'ip_type' (u8):
    // 		- The endpoint ip version as a u8, 4 or 6.
    //
    // 	* 'protocol' (u8):
    // 		- UDP:1 or TCP:0 
    //
    // 	* 'placeholder1' (u8):
    // 		- Placeholder for further extra params.
    //
    // 	* 'placeholder2' (u8):
    // 		- Placeholder for further extra params.
    //
    // # Event:
    // 	* AxonServed;
    // 		- On successfully serving the axon info.
    //
    // # Raises:
    // 	* 'NetworkDoesNotExist':
    // 		- Attempting to set weights on a non-existent network.
    //
    // 	* 'NotRegistered':
    // 		- Attempting to set weights from a non registered account.
    //
    // 	* 'InvalidIpType':
    // 		- The ip type is not 4 or 6.
    //
    // 	* 'InvalidIpAddress':
    // 		- The numerically encoded ip address does not resolve to a proper ip.
    //
    // 	* 'ServingRateLimitExceeded':
    // 		- Attempting to set prometheus information withing the rate limit min.
    //
    pub fn do_serve_axon(origin: T::RuntimeOrigin,  netuid: u16, version: u32, ip: u128, port: u16, ip_type: u8, protocol: u8, placeholder1: u8, placeholder2: u8) 
        -> dispatch::DispatchResult 
    {
        // --- 1. We check the callers (hotkey) signature.
        let hotkey_id: T::AccountId;
        {
            hotkey_id = ensure_signed(origin)?;
        }

        // --- 2. Ensure the hotkey is registered somewhere.
        {
            ensure!(
                Self::is_hotkey_registered_on_any_network(&hotkey_id), 
                Error::<T>::NotRegistered
            );  
        }
        
        // --- 3. Check the ip signature validity.
        {
            ensure!(
                Self::is_valid_ip_type(ip_type), 
                Error::<T>::InvalidIpType
            );

            ensure!(
                Self::is_valid_ip_address(ip_type, ip), 
                Error::<T>::InvalidIpAddress
            );
        }
  
        // --- 4. Get the previous axon information.
        let mut prev_axon: AxonInfo;
        {
            prev_axon                   = Self::get_axon_info( netuid, &hotkey_id );

            ensure!(
                Self::axon_passes_rate_limit(netuid, &prev_axon, Self::get_current_block_as_u64()),
                Error::<T>::ServingRateLimitExceeded
            ); 
        }

        // --- 6. We insert the axon meta.
        {
            prev_axon.block         = Self::get_current_block_as_u64();
            prev_axon.version       = version;
            prev_axon.ip            = ip;
            prev_axon.port          = port;
            prev_axon.ip_type       = ip_type;
            prev_axon.protocol      = protocol;
            prev_axon.placeholder1  = placeholder1;
            prev_axon.placeholder2  = placeholder2;
        }

		// --- 7. Validate axon data with delegate func
        {
            let axon_validated = Self::validate_axon_data(&prev_axon);

            ensure!(
                axon_validated.is_ok(), 
                axon_validated.err().unwrap_or(Error::<T>::InvalidPort)
            );

            Axons::<T>::insert( netuid, hotkey_id.clone(), prev_axon );
        }

        // --- 8. We deposit axon served event.
        {
            log::info!("AxonServed( hotkey:{:?} ) ", hotkey_id.clone() );
        
            Self::deposit_event(Event::AxonServed( netuid, hotkey_id ));
        }

        // --- 9. Return is successful dispatch. 
        return Ok(());
    }

    pub fn has_axon_info( netuid: u16, hotkey: &T::AccountId ) -> bool 
    {
        return Axons::<T>::contains_key( netuid, hotkey );
    }

    pub fn get_axon_info( netuid: u16, hotkey: &T::AccountId ) -> AxonInfoOf 
    {
        if Self::has_axon_info( netuid, hotkey ) 
        {
            return Axons::<T>::get( netuid, hotkey ).unwrap();
        } 
        else
        {
            return AxonInfo 
            { 
                block:          0,
                version:        0,
                ip:             0,
                port:           0,
                ip_type:        0,
                protocol:       0,
                placeholder1:   0,
                placeholder2:   0
            }

        }
    }

    pub fn axon_passes_rate_limit( netuid: u16, prev_axon_info: &AxonInfoOf, current_block: u64 ) -> bool 
    {
        let rate_limit: u64 = Self::get_serving_rate_limit(netuid);
        let last_serve: u64 = prev_axon_info.block;

        return rate_limit == 0 || last_serve == 0 || current_block - last_serve >= rate_limit;
    }

    pub fn validate_axon_data(axon_info: &AxonInfoOf) -> Result<bool, pallet::Error<T>> 
    {
		if axon_info.port.clamp(0, u16::MAX) <= 0 
        {
			return Err(Error::<T>::InvalidPort);
		}

		return Ok(true);
	}
}