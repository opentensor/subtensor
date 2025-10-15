use super::*;
use safe_math::*;
use substrate_fixed::types::{U96F32, U110F18};
use subtensor_runtime_common::{NetUid, TaoCurrency};

impl<T: Config + pallet_drand::Config> Pallet<T> {
    /// Executes the necessary operations for each block.
    pub fn block_step() -> Result<(), &'static str> {
        let block_number: u64 = Self::get_current_block_as_u64();
        log::debug!("block_step for block: {block_number:?} ");
        // --- 1. Get the current coinbase emission.
        let block_emission: U96F32 = U96F32::saturating_from_num(
            Self::get_block_emission()
                .unwrap_or(TaoCurrency::ZERO)
                .to_u64(),
        );
        log::debug!("Block emission: {block_emission:?}");
        // --- 2. Run emission through network.
        Self::run_coinbase(block_emission);
        // --- 3. Set pending children on the epoch; but only after the coinbase has been run.
        Self::try_set_pending_children(block_number);
        // Return ok.
        Ok(())
    }

    fn try_set_pending_children(block_number: u64) {
        for netuid in Self::get_all_subnet_netuids() {
            if Self::should_run_epoch(netuid, block_number) {
                // Set pending children on the epoch.
                Self::do_set_pending_children(netuid);
            }
        }
    }
}
