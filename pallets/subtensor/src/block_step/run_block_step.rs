use super::*;

impl<T: Config> Pallet<T> {
    /// Executes the necessary operations for each block.
    /// 
    /// This function performs the following tasks:
    /// 1. Retrieves the current block number.
    /// 2. Logs the block number for debugging purposes.
    /// 3. Adjusts registration terms for all networks.
    /// 4. Runs the coinbase operation.
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if all operations are successful, otherwise an error message.
    pub fn block_step() -> Result<(), &'static str> {
        // Get the current block number
        let block_number: u64 = Self::get_current_block_as_u64();

        // Log the current block number for debugging
        log::debug!("block_step for block: {:?}", block_number);

        // Adjust registration terms for all networks
        Self::adjust_registration_terms_for_networks();

        // Run the coinbase operation
        Self::run_coinbase();

        // Return success
        Ok(())
    }

}
