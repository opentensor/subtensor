use super::*;
use crate::system::ensure_root;
use frame_support::IterableStorageMap;

impl<T: Config> Pallet<T> {

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

    pub fn create_mechanism( mechid: u16 ) {
        // Add the mechanism.
        MechanismsAdded::<T>::insert( mechid, true );

        // Set mechanism n  
        MechanismN::<T>::insert( mechid, Self::get_mechids().len() as u16 );
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
    pub fn do_sudo_create_mechanism( origin: T::RuntimeOrigin ) -> DispatchResult {
        // Check that this is sudo.
        let _ = ensure_root( origin );

        // Increment the mechanism.
        let mechid = Self::get_next_mechid();

        // Create the mechanism.
        Self::create_mechanism( mechid );

        // Emit event for mechanism creation
        log::info!(
            "MechanismAdded( mechid:{:?} )",
            mechid
        );
        Self::deposit_event(Event::MechanismAdded(mechid));

        // Ok.
        Ok(())
    }

}