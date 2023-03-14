use super::*;
use frame_support::inherent::Vec;
use frame_support::sp_std::vec;


impl<T: Config> Pallet<T> {

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
    pub fn do_serve_axon( 
        origin: T::RuntimeOrigin, 
        version: u32, 
        ip: u128, 
        port: u16, 
        ip_type: u8,
        protocol: u8, 
		placeholder1: u8, 
		placeholder2: u8,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the callers (hotkey) signature.
        let hotkey_id = ensure_signed(origin)?;

        // --- 2. Ensure the hotkey is registered somewhere.
        // ensure!( Self::is_hotkey_registered_on_any_network( &hotkey_id ), Error::<T>::NotRegistered );  
        
        // --- 3. Check the ip signature validity.
        ensure!( Self::is_valid_ip_type(ip_type), Error::<T>::InvalidIpType );
        ensure!( Self::is_valid_ip_address(ip_type, ip), Error::<T>::InvalidIpAddress );
  
        // --- 4. Get the previous axon information.
        let mut prev_axon = Self::get_axon_info( &hotkey_id );
        let current_block:u64 = Self::get_current_block_as_u64();
        ensure!( Self::axon_passes_rate_limit( &prev_axon, current_block ), Error::<T>::ServingRateLimitExceeded );  

        // --- 6. We insert the axon meta.
        prev_axon.block = Self::get_current_block_as_u64();
        prev_axon.version = version;
        prev_axon.ip = ip;
        prev_axon.port = port;
        prev_axon.ip_type = ip_type;
        prev_axon.protocol = protocol;
        prev_axon.placeholder1 = placeholder1;
        prev_axon.placeholder2 = placeholder2;
        Axons::<T>::insert( hotkey_id.clone(), prev_axon );

        // --- 7. We deposit axon served event.
        log::info!("AxonServed( hotkey:{:?} ) ", hotkey_id.clone() );
        Self::deposit_event(Event::AxonServed( hotkey_id ));

        // --- 8. Return is successful dispatch. 
        Ok(())
    }

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
    pub fn do_serve_prometheus( 
        origin: T::RuntimeOrigin, 
        version: u32, 
        ip: u128, 
        port: u16, 
        ip_type: u8,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the callers (hotkey) signature.
        let hotkey_id = ensure_signed(origin)?;

        // --- 2. Ensure the hotkey is registered somewhere.
        // ensure!( Self::is_hotkey_registered_on_any_network( &hotkey_id ), Error::<T>::NotRegistered );  

        // --- 3. Check the ip signature validity.
        ensure!( Self::is_valid_ip_type(ip_type), Error::<T>::InvalidIpType );
        ensure!( Self::is_valid_ip_address(ip_type, ip), Error::<T>::InvalidIpAddress );
  
        // --- 5. We get the previous axon info assoicated with this ( netuid, uid )
        let mut prev_prometheus = Self::get_prometheus_info( &hotkey_id );
        let current_block:u64 = Self::get_current_block_as_u64();
        ensure!( Self::prometheus_passes_rate_limit( &prev_prometheus, current_block ), Error::<T>::ServingRateLimitExceeded );  

        // --- 6. We insert the prometheus meta.
        prev_prometheus.block = Self::get_current_block_as_u64();
        prev_prometheus.version = version;
        prev_prometheus.ip = ip;
        prev_prometheus.port = port;
        prev_prometheus.ip_type = ip_type;
        Prometheus::<T>::insert( hotkey_id.clone(), prev_prometheus );

        // --- 7. We deposit prometheus served event.
        log::info!("PrometheusServed( hotkey:{:?} ) ", hotkey_id.clone() );
        Self::deposit_event(Event::PrometheusServed( hotkey_id ));

        // --- 8. Return is successful dispatch. 
        Ok(())
    }

    /********************************
     --==[[  Helper functions   ]]==--
    *********************************/

    pub fn axon_passes_rate_limit( prev_axon_info: &AxonInfoOf, current_block: u64 ) -> bool {
        let rate_limit: u64 = Self::get_serving_rate_limit();
        let last_serve = prev_axon_info.block;
        return rate_limit == 0 || last_serve == 0 || current_block - last_serve >= rate_limit;
    }

    pub fn prometheus_passes_rate_limit( prev_prometheus_info: &PrometheusInfoOf, current_block: u64 ) -> bool {
        let rate_limit: u64 = Self::get_serving_rate_limit();
        let last_serve = prev_prometheus_info.block;
        return rate_limit == 0 || last_serve == 0 || current_block - last_serve >= rate_limit;
    }

    pub fn has_axon_info( hotkey: &T::AccountId ) -> bool {
        return Axons::<T>::contains_key( hotkey );
    }

    pub fn has_prometheus_info( hotkey: &T::AccountId ) -> bool {
        return Prometheus::<T>::contains_key( hotkey );
    }

    pub fn get_axon_info( hotkey: &T::AccountId ) -> AxonInfoOf {
        if Self::has_axon_info( hotkey ) {
            return Axons::<T>::get( hotkey ).unwrap();
        } else{
            return AxonInfo { 
                block: 0,
                version: 0,
                ip: 0,
                port: 0,
                ip_type: 0,
                protocol: 0,
                placeholder1: 0,
                placeholder2: 0
            }

        }
    }

    pub fn get_prometheus_info( hotkey: &T::AccountId ) -> PrometheusInfoOf {
        if Self::has_prometheus_info( hotkey ) {
            return Prometheus::<T>::get( hotkey ).unwrap();
        } else {
            return PrometheusInfo { 
                block: 0,
                version: 0,
                ip: 0,
                port: 0,
                ip_type: 0,
            }

        }
    }

    pub fn is_valid_ip_type(ip_type: u8) -> bool {
        let allowed_values: Vec<u8> = vec![4, 6];
        return allowed_values.contains(&ip_type);
    }

    // @todo (Parallax 2-1-2021) : Implement exclusion of private IP ranges
    pub fn is_valid_ip_address(ip_type: u8, addr: u128) -> bool {
        if !Self::is_valid_ip_type(ip_type) {
            return false;
        }
        if addr == 0 {
            return false;
        }
        if ip_type == 4 {
            if addr == 0 { return false; }
            if addr >= u32::MAX as u128 { return false; }
            if addr == 0x7f000001 { return false; } // Localhost
        }
        if ip_type == 6 {
            if addr == 0x0 { return false; }
            if addr == u128::MAX { return false; }
            if addr == 1 { return false; } // IPv6 localhost
        }
        return true;
    }


}