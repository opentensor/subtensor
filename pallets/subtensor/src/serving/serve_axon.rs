use super::*;

impl<T: Config> Pallet<T> {
    /// ---- The implementation for the extrinsic serve_axon which sets the ip endpoint information for a uid on a network.
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
    ///     - The endpoint ip information as a u128 encoded integer.
    ///
    /// * 'port' (u16):
    ///     - The endpoint port information as a u16 encoded integer.
    ///
    /// * 'ip_type' (u8):
    ///     - The endpoint ip version as a u8, 4 or 6.
    ///
    /// * 'protocol' (u8):
    ///     - UDP:1 or TCP:0
    ///
    /// * 'placeholder1' (u8):
    ///     - Placeholder for further extra params.
    ///
    /// * 'placeholder2' (u8):
    ///     - Placeholder for further extra params.
    ///
    /// # Event:
    /// * AxonServed;
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
    pub fn do_serve_axon(
        origin: T::RuntimeOrigin,
        netuid: u16,
        version: u32,
        ip: u128,
        port: u16,
        ip_type: u8,
        protocol: u8,
        placeholder1: u8,
        placeholder2: u8,
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

        // Get the previous axon information.
        let mut prev_axon = Self::get_axon_info(netuid, &hotkey_id);
        let current_block: u64 = Self::get_current_block_as_u64();
        ensure!(
            Self::axon_passes_rate_limit(netuid, &prev_axon, current_block),
            Error::<T>::ServingRateLimitExceeded
        );

        // We insert the axon meta.
        prev_axon.block = Self::get_current_block_as_u64();
        prev_axon.version = version;
        prev_axon.ip = ip;
        prev_axon.port = port;
        prev_axon.ip_type = ip_type;
        prev_axon.protocol = protocol;
        prev_axon.placeholder1 = placeholder1;
        prev_axon.placeholder2 = placeholder2;

        // Validate axon data with delegate func
        let axon_validated = Self::validate_axon_data(&prev_axon);
        ensure!(
            axon_validated.is_ok(),
            axon_validated.err().unwrap_or(Error::<T>::InvalidPort)
        );

        Axons::<T>::insert(netuid, hotkey_id.clone(), prev_axon);

        // We deposit axon served event.
        log::info!("AxonServed( hotkey:{:?} ) ", hotkey_id.clone());
        Self::deposit_event(Event::AxonServed(netuid, hotkey_id));

        // Return is successful dispatch.
        Ok(())
    }

    /// Retrieves the axon information for a given network and hotkey.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `hotkey` - The account ID of the hotkey.
    ///
    /// # Returns
    ///
    /// Returns the `AxonInfo` for the given network and hotkey. If no information is found,
    /// returns a default `AxonInfo` with all fields set to 0.
    pub fn get_axon_info(netuid: u16, hotkey: &T::AccountId) -> AxonInfoOf {
        Axons::<T>::get(netuid, hotkey).unwrap_or_else(|| AxonInfo {
            block: 0,
            version: 0,
            ip: 0,
            port: 0,
            ip_type: 0,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
        })
    }

    /// Checks if the axon passes the rate limit for serving.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `prev_axon_info` - The previous axon information.
    /// * `current_block` - The current block number.
    ///
    /// # Returns
    ///
    /// Returns `true` if the axon passes the rate limit, `false` otherwise.
    pub fn axon_passes_rate_limit(
        netuid: u16,
        prev_axon_info: &AxonInfoOf,
        current_block: u64,
    ) -> bool {
        let rate_limit: u64 = Self::get_serving_rate_limit(netuid);
        let last_serve = prev_axon_info.block;

        // Pass if rate limit is 0, or if this is the first serve, or if enough blocks have passed
        rate_limit == 0 || last_serve == 0 || current_block - last_serve >= rate_limit
    }

    /// Checks if axon information exists for a given network and hotkey.
    ///
    /// # Arguments
    ///
    /// * `netuid` - The network ID.
    /// * `hotkey` - The account ID of the hotkey.
    ///
    /// # Returns
    ///
    /// Returns `true` if axon information exists, `false` otherwise.
    pub fn has_axon_info(netuid: u16, hotkey: &T::AccountId) -> bool {
        Axons::<T>::contains_key(netuid, hotkey)
    }

    /// Validates the axon data.
    ///
    /// # Arguments
    ///
    /// * `axon_info` - The axon information to validate.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the axon data is valid, `Err` with the specific error otherwise.
    pub fn validate_axon_data(axon_info: &AxonInfoOf) -> Result<bool, pallet::Error<T>> {
        // Check if the port is valid (not 0)
        if axon_info.port == 0 {
            return Err(Error::<T>::InvalidPort);
        }

        // Add more validation checks here if needed

        Ok(true)
    }
}
