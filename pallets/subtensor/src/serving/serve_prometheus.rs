use super::*;

impl<T: Config> Pallet<T> {
  
    /// ---- The implementation for the extrinsic serve_prometheus.
    ///
    /// # Args:
    /// * 'origin': (<T as frame_system::Config>RuntimeOrigin):
    ///     - The signature of the caller.
    ///
    /// * 'netuid' (u16):
    ///     - The u16 network identifier.
    ///
    /// * 'version' (u64):
    ///     - The bittensor version identifier.
    ///
    /// * 'ip' (u64):
    ///     - The prometheus ip information as a u128 encoded integer.
    ///
    /// * 'port' (u16):
    ///     - The prometheus port information as a u16 encoded integer.
    ///
    /// * 'ip_type' (u8):
    ///     - The prometheus ip version as a u8, 4 or 6.
    ///
    /// # Event:
    /// * PrometheusServed;
    ///     - On successfully serving the axon info.
    ///
    /// # Raises:
    /// * 'SubNetworkDoesNotExist':
    ///     - Attempting to set weights on a non-existent network.
    ///
    /// * 'NotRegistered':
    ///     - Attempting to set weights from a non registered account.
    ///
    /// * 'InvalidIpType':
    ///     - The ip type is not 4 or 6.
    ///
    /// * 'InvalidIpAddress':
    ///     - The numerically encoded ip address does not resolve to a proper ip.
    ///
    /// * 'ServingRateLimitExceeded':
    ///     - Attempting to set prometheus information withing the rate limit min.
    ///
    pub fn do_serve_prometheus(
        origin: T::RuntimeOrigin,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
    ) -> dispatch::DispatchResult {
        // We check the callers (hotkey) signature.
        let hotkey_id = ensure_signed(origin)?;

        // Ensure the hotkey is registered somewhere.
        ensure!(
            Self::is_hotkey_registered_on_any_network(&hotkey_id),
            Error::<T>::HotKeyNotRegisteredInNetwork
        );

        // Check the ip signature validity.
        ensure!(Self::is_valid_ip_type(ip_type), Error::<T>::InvalidIpType);
        ensure!(
            Self::is_valid_ip_address(ip_type, ip),
            Error::<T>::InvalidIpAddress
        );

        // We get the previous axon info assoicated with this ( netuid, uid )
        let mut prev_prometheus = Self::get_prometheus_info(netuid, &hotkey_id);
        let current_block: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::prometheus_passes_rate_limit(netuid, &prev_prometheus, current_block),
            Error::<T>::ServingRateLimitExceeded
        );

        // We insert the prometheus meta.
        prev_prometheus.block = Self::get_current_block_as_u64();
        prev_prometheus.version = version;
        prev_prometheus.ip = ip;
        prev_prometheus.port = port;
        prev_prometheus.ip_type = ip_type;

        // Validate prometheus data with delegate func
        let prom_validated = Self::validate_prometheus_data(&prev_prometheus);
        ensure!(
            prom_validated.is_ok(),
            prom_validated.err().unwrap_or(Error::<T>::InvalidPort)
        );

        // Insert new prometheus data
        Prometheus::<T>::insert(netuid, hotkey_id.clone(), prev_prometheus);

        // We deposit prometheus served event.
        log::info!("PrometheusServed( hotkey:{:?} ) ", hotkey_id.clone());
        Self::deposit_event(Event::PrometheusServed(netuid, hotkey_id));

        // Return is successful dispatch.
        Ok(())
    }

    /// Retrieves the Prometheus information for a given network and hotkey.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `hotkey` - The account ID of the hotkey.
    ///
    /// # Returns
    ///
    /// Returns the `PrometheusInfo` for the given network and hotkey. If no information is found,
    /// returns a default `PrometheusInfo` with all fields set to 0.
    pub fn get_prometheus_info(netuid: u16, hotkey: &T::AccountId) -> PrometheusInfoOf {
        Prometheus::<T>::get(netuid, hotkey).unwrap_or_else(|| PrometheusInfo {
            block: 0,
            version: 0,
            ip: 0,
            port: 0,
            ip_type: 0,
        })
    }

    /// Checks if the Prometheus instance passes the rate limit for serving.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `prev_prometheus_info` - The previous Prometheus information.
    /// * `current_block` - The current block number.
    ///
    /// # Returns
    ///
    /// Returns `true` if the Prometheus instance passes the rate limit, `false` otherwise.
    pub fn prometheus_passes_rate_limit(
        netuid: u16,
        prev_prometheus_info: &PrometheusInfoOf,
        current_block: u64,
    ) -> bool {
        let rate_limit: u64 = Self::get_serving_rate_limit(netuid);
        let last_serve = prev_prometheus_info.block;

        // Pass if rate limit is 0, or if this is the first serve, or if enough blocks have passed
        rate_limit == 0 || last_serve == 0 || current_block - last_serve >= rate_limit
    }

    /// Checks if Prometheus information exists for a given network and hotkey.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `hotkey` - The account ID of the hotkey.
    ///
    /// # Returns
    ///
    /// Returns `true` if Prometheus information exists, `false` otherwise.
    pub fn has_prometheus_info(netuid: u16, hotkey: &T::AccountId) -> bool {
        Prometheus::<T>::contains_key(netuid, hotkey)
    }

    /// Validates the Prometheus data.
    ///
    /// # Arguments
    ///
    /// * `prom_info` - The Prometheus information to validate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the Prometheus data is valid, `Err` with the specific error otherwise.
    pub fn validate_prometheus_data(
        prom_info: &PrometheusInfoOf,
    ) -> Result<bool, pallet::Error<T>> {
        // Check if the port is valid (not 0)
        if prom_info.port == 0 {
            return Err(Error::<T>::InvalidPort);
        }

        // Add more validation checks here if needed

        Ok(true)
    }

}
