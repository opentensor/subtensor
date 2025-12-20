use super::*;
extern crate alloc;
use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;
use sp_runtime::traits::Zero;
use crate::pallet::*;
use subtensor_runtime_common::{NetUid, AlphaCurrency, TaoCurrency};

impl<T: Config> Pallet<T> {
    /// Checks invariants and handles violations.
    /// Should be called in on_finalize.
    pub fn check_invariants() {
        // 1. Check Emission Invariant (Global vs Sum of Subnets)
        // Invariant: sum(subnet_emissions) <= global_emission
        // We check this every block as emission happens every block.
        Self::check_emission_invariant();

        // 2. Check Stake Invariant (Per Subnet)
        // Invariant: sum(neuron_stake in subnet) == stored subnet_total_stake
        // We check this only at epoch boundaries (tempo) to save weight.
        Self::check_stake_invariant();
    }

    fn check_emission_invariant() {
        let block_emission_u64 = BlockEmission::<T>::get();
        let block_emission: TaoCurrency = block_emission_u64.into();

        // Sum of all tao injected into subnets this block
        let mut total_injected = TaoCurrency::zero();
        
        // We iterate all subnets. SubnetTaoInEmission is set in run_coinbase for the current block.
        for netuid in Self::get_all_subnet_netuids() {
             let injected = SubnetTaoInEmission::<T>::get(netuid);
             total_injected = total_injected.saturating_add(injected);
        }

        // Invariant: Total injected should not exceed block emission.
        // It can be LESS than block_emission due to:
        // 1. Excess TAO being burned or added to issuance directly (not injected into pool)
        // 2. Rounding errors (dust)
        // 3. Subnets not emitting (paused or strictly no emission)
        // It should NEVER be MORE.
        if total_injected > block_emission {
             // We use NetUid::ROOT (0) as a placeholder for global violation if we can't pin it to a subnet.
             Self::handle_invariant_violation(NetUid::ROOT, "Emission invariant violation: injected > block_emission");
        }
    }

    fn check_stake_invariant() {
        for netuid in Self::get_all_subnet_netuids() {
            // Check if emission is paused to avoid repeated spam.
            if SubnetEmissionPaused::<T>::get(netuid) {
                continue;
            }

            // Only check at epoch boundary (when BlocksSinceLastStep was reset to 0 in this block)
            // This ensures we verify the state right after pending emissions are drained and stake is "settled" for the epoch.
            if BlocksSinceLastStep::<T>::get(netuid) != 0 {
                continue;
            }

            // SubnetAlphaOut represents the total outstanding Alpha shares in the subnet.
            let stored_total_alpha = SubnetAlphaOut::<T>::get(netuid);
            let mut calculated_total_alpha = AlphaCurrency::zero();

            // Iterate all hotkeys in subnet.
            // Keys::iter_prefix(netuid) gives us all (uid, hotkey) pairs in the subnet.
            // We sum the TotalHotkeyAlpha for each hotkey.
            // Requirement matches: "sum(neuron_stake in subnet) == stored subnet_total_stake"
            for (_, hotkey) in Keys::<T>::iter_prefix(netuid) {
                let alpha = TotalHotkeyAlpha::<T>::get(&hotkey, netuid);
                calculated_total_alpha = calculated_total_alpha.saturating_add(alpha);
            }

            // We expect strict equality. Alpha represents shares, which are integers.
            if stored_total_alpha != calculated_total_alpha {
                 let msg = alloc::format!("Stake (Alpha) mismatch: stored={:?}, calc={:?}", stored_total_alpha, calculated_total_alpha);
                 Self::handle_invariant_violation(netuid, &msg);
            }
        }
    }

    fn handle_invariant_violation(netuid: NetUid, details: &str) {
        log::error!("CRITICAL INVARIANT VIOLATION on subnet {}: {}", netuid, details);

        // Pause emissions for this subnet to prevent further economic corruption.
        SubnetEmissionPaused::<T>::insert(netuid, true);

        // Emit event for off-chain monitoring.
        let details_bytes = details.as_bytes().to_vec();
        Self::deposit_event(Event::CriticalInvariantViolation(netuid, details_bytes));

        // Panic only in test environments or debug builds.
        // In production, we assume the paused state is sufficient protection and we do NOT want to brick the chain.
        #[cfg(any(test, feature = "std", debug_assertions))]
        {
            // We only panic if we are in a test or strictly debugging.
            // Note: 'feature = "std"' is often enabled in dev nodes, but careful with production wasm builds (usually no-std).
            // 'debug_assertions' is the standard way to detect dev/debug profile.
            panic!("Invariant violation: subnet {}, {}", netuid, details);
        }
    }
}
