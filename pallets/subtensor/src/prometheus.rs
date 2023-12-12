impl<T: Config> Pallet<T>
{
    // ---- The implementation for the extrinsic serve_prometheus.
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
    // 		- The prometheus ip information as a u128 encoded integer.
    //
    // 	* 'port' (u16):
    // 		- The prometheus port information as a u16 encoded integer.
    // 
    // 	* 'ip_type' (u8):
    // 		- The prometheus ip version as a u8, 4 or 6.
    //
    // # Event:
    // 	* PrometheusServed;
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
    pub fn do_serve_prometheus( origin: T::RuntimeOrigin, netuid: u16, version: u32, ip: u128, port: u16, ip_type: u8) -> dispatch::DispatchResult 
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
  
        // --- 5. We get the previous axon info assoicated with this ( netuid, uid )
        let mut prev_prometheus: PrometheusInfo;
        {
            prev_prometheus = Self::get_prometheus_info( netuid, &hotkey_id );

            ensure!( 
                Self::prometheus_passes_rate_limit(netuid, &prev_prometheus, Self::get_current_block_as_u64()),
                Error::<T>::ServingRateLimitExceeded
            ); 
        } 

        // --- 6. We insert the prometheus meta.
        {
            prev_prometheus.block   = Self::get_current_block_as_u64();
            prev_prometheus.version = version;
            prev_prometheus.ip      = ip;
            prev_prometheus.port    = port;
            prev_prometheus.ip_type = ip_type;
        }

		// --- 7. Validate prometheus data with delegate func
        {
		    let prom_validated = Self::validate_prometheus_data(&prev_prometheus);

		    ensure!(
                prom_validated.is_ok(),
                prom_validated.err().unwrap_or(Error::<T>::InvalidPort)
            );
        }

		// --- 8. Insert new prometheus data
        {
            Prometheus::<T>::insert(netuid, hotkey_id.clone(), prev_prometheus);
        }

        // --- 9. We deposit prometheus served event.
        {
            log::info!("PrometheusServed( hotkey:{:?} ) ", hotkey_id.clone());

            Self::deposit_event(Event::PrometheusServed(netuid, hotkey_id));
        }

        // --- 10. Return is successful dispatch. 
        return Ok(());
    }

    pub fn validate_prometheus_data(prom_info: &PrometheusInfoOf) -> Result<bool, pallet::Error<T>> 
    {
		if prom_info.port.clamp(0, u16::MAX) <= 0 
        {
			return Err(Error::<T>::InvalidPort);
		}

		return Ok(true);
	}

    pub fn prometheus_passes_rate_limit( netuid: u16, prev_prometheus_info: &PrometheusInfoOf, current_block: u64 ) -> bool 
    {
        let rate_limit: u64 = Self::get_serving_rate_limit(netuid);
        let last_serve: u64 = prev_prometheus_info.block;

        return rate_limit == 0 || last_serve == 0 || current_block - last_serve >= rate_limit;
    }    

    pub fn has_prometheus_info( netuid: u16, hotkey: &T::AccountId ) -> bool 
    {
        return Prometheus::<T>::contains_key( netuid, hotkey );
    }

    pub fn get_prometheus_info( netuid: u16, hotkey: &T::AccountId ) -> PrometheusInfoOf 
    {
        if Self::has_prometheus_info(netuid, hotkey) 
        {
            return Prometheus::<T>::get( netuid, hotkey ).unwrap();
        } 
        else 
        {
            return PrometheusInfo 
            { 
                block:      0,
                version:    0,
                ip:         0,
                port:       0,
                ip_type:    0,
            }

        }
    }
}