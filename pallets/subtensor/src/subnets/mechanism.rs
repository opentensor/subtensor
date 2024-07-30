use super::*;
use crate::system::ensure_root;
use frame_support::IterableStorageMap;

impl<T: Config> Pallet<T> {

    /// Fetches the total count of subnets.
    ///
    /// This function retrieves the total number of subnets present on the chain.
    ///
    /// # Returns:
    /// * 'u16': The total number of subnets.
    ///
    pub fn get_num_subnets() -> u16 {
        TotalNetworks::<T>::get()
    }

    /// Fetches the max number of subnet
    ///
    /// This function retrieves the max number of subnet.
    ///
    /// # Returns:
    /// * 'u16': The max number of subnet
    ///
    pub fn get_max_subnets() -> u16 {
        SubnetLimit::<T>::get()
    }

    /// Sets the max number of subnet
    ///
    /// This function sets the max number of subnet.
    ///
    pub fn set_max_subnets(limit: u16) {
        SubnetLimit::<T>::put(limit);
        Self::deposit_event(Event::SubnetLimitSet(limit));
    }


    /// Returns true if the subnetwork exists.
    ///
    /// This function checks if a subnetwork with the given UID exists.
    ///
    /// # Returns:
    /// * 'bool': Whether the subnet exists.
    ///
    pub fn if_subnet_exist(netuid: u16) -> bool {
        NetworksAdded::<T>::get(netuid)
    }

    /// Returns a list of subnet netuid equal to total networks.
    ///
    ///
    /// This iterates through all the networks and returns a list of netuids.
    ///
    /// # Returns:
    /// * 'Vec<u16>': Netuids of all subnets.
    ///
    pub fn get_all_subnet_netuids() -> Vec<u16> {
        <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter()
            .map(|(netuid, _)| netuid)
            .collect()
    }

    /// Returns a vector of subnet network IDs (netuids) that use a specific mechanism.
    ///
    /// This function iterates through all subnets and collects the network IDs
    /// of those that use the specified mechanism.
    ///
    /// # Arguments
    ///
    /// * `mechid` - The mechanism ID to filter subnets by.
    ///
    /// # Returns
    ///
    /// * `Vec<u16>` - A vector of network IDs (netuids) that use the specified mechanism.
    pub fn get_mechanism_netuids( mechid: u16 ) -> Vec<u16> {
        // Get all subnets with mechanism equal to mechid
        <NetworksAdded<T> as IterableStorageMap<u16, bool>>::iter()
            .filter_map(|(netuid, exists)| {
                if exists && SubnetMechanism::<T>::get(netuid) == mechid {
                    Some(netuid)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns a vector of all existing mechanism IDs.
    ///
    /// This function iterates through the `MechanismsAdded` storage map and collects
    /// all mechanism IDs that are marked as added (true).
    ///
    /// # Returns
    /// * `Vec<u16>` - A vector containing all existing mechanism IDs.
    pub fn get_mechids() -> Vec<u16> {
        <MechanismsAdded<T> as IterableStorageMap<u16, bool>>::iter()
            .filter_map(|(mechid, is_added)| if is_added { Some(mechid) } else { None })
            .collect()
    }

    /// Returns the total number of existing mechanisms.
    ///
    /// This function calls `get_mechids()` and returns the length of the resulting vector.
    ///
    /// # Returns
    /// * `u16` - The total number of existing mechanisms.
    pub fn get_num_mechanisms() -> u16 {
        Self::get_mechids().len() as u16
    }

    /// Checks if a mechanism with the given ID exists.
    ///
    /// # Arguments
    /// * `mechid` - The mechanism ID to check.
    ///
    /// # Returns
    /// * `bool` - True if the mechanism exists, false otherwise.
    pub fn mechanism_exists( mechid: u16 ) -> bool {
        MechanismsAdded::<T>::get( mechid )
    }

    /// Finds the next available mechanism ID.
    ///
    /// This function iterates through possible mechanism IDs starting from 0
    /// until it finds an ID that is not currently in use.
    ///
    /// # Returns
    /// * `u16` - The next available mechanism ID.
    pub fn get_next_mechid() -> u16 {
        let mut next_mechid = 0;
        let mechids: Vec<u16> = Self::get_mechids();
        loop {
            if !mechids.contains(&next_mechid) {
                break next_mechid;
            }
            next_mechid += 1;
        }
    }

    /// Finds the next available mechanism ID.
    ///
    /// This function iterates through possible mechanism IDs starting from 0
    /// until it finds an ID that is not currently in use.
    ///
    /// # Returns
    /// * `u16` - The next available mechanism ID.
    pub fn get_next_netuid() -> u16 {
        let mut next_netuid = 0;
        let netuids: Vec<u16> = Self::get_all_subnet_netuids();
        loop {
            if !netuids.contains(&next_netuid) {
                break next_netuid;
            }
            next_netuid += 1;
        }
    }

    /// Creates a new mechanism (sudo only).
    ///
    /// This function can only be called by the root account. It creates a new mechanism
    /// with the next available ID, updates the relevant storage items, and emits an event.
    ///
    /// # Arguments
    /// * `origin` - The origin of the call, must be root.
    ///
    /// # Errors
    /// This function will return an error if the origin is not root.
    pub fn do_sudo_create_mechanism( origin: T::RuntimeOrigin, n: u16 ) -> DispatchResult {

        // Check that this is sudo.
        let coldkey = ensure_root( origin );

        // Increment the mechanism.
        let mechid = Self::get_next_mechid();

        // Actually create the mechanism.
        Self::create_mechanism( mechid, n );

        // Emit event for mechanism creation
        log::info!(
            "MechanismAdded( mechid:{:?}, n:{:?} )",
            mechid, n
        );
        Self::deposit_event(Event::MechanismAdded(mechid));

        // Ok.
        Ok(())
    }

    /// Creates a new mechanism with the specified ID and number of subnets.
    ///
    /// This function performs the following steps:
    /// 1. Marks the mechanism as added.
    /// 2. Creates the specified number of subnets and associates them with the mechanism.
    /// 3. Sets the number of subnets for the mechanism.
    ///
    /// # Arguments
    /// * `mechid` - The ID of the mechanism to create.
    /// * `n` - The number of subnets to create for this mechanism.
    pub fn create_mechanism(mechid: u16, n: u16) {
        // Get the current block number for subnet registration timestamp
        let current_block_number: u64 = Self::get_current_block_as_u64();

        // Mark the mechanism as added
        MechanismsAdded::<T>::insert(mechid, true);

        // Create the specified number of subnets
        for _ in 0..n {
            let next_netuid: u16 = Self::get_next_netuid();

            // Initialize the new subnet
            Self::init_new_network(next_netuid, 360);

            // Record the block number at which the subnet was registered
            NetworkRegisteredAt::<T>::insert(next_netuid, current_block_number);

            // Associate the subnet with this mechanism
            SubnetMechanism::<T>::insert(next_netuid, mechid);
        }

        // Set the number of subnets for this mechanism
        // Note: This might not be accurate if subnets were added to this mechanism previously
        MechanismN::<T>::insert(mechid, Self::get_mechids().len() as u16);
    }



}