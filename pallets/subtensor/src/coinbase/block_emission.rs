use super::*;
use frame_support::traits::Get;
use substrate_fixed::transcendental::log2;
use substrate_fixed::types::I96F32;

impl<T: Config> Pallet<T> {
    /// Calculates the block emission based on the total issuance and updates the chain if applicable.
    ///
    /// This function computes the block emission by applying a logarithmic function
    /// to the total issuance of the network. The formula used takes into account
    /// the current total issuance and adjusts the emission rate accordingly to ensure
    /// a smooth issuance curve. The emission rate decreases as the total issuance increases,
    /// following a logarithmic decay.
    ///
    /// # Returns
    /// * 'Result<u64, &'static str>': The calculated block emission rate or error.
    ///
    pub fn block_emission_step() -> Result<u64, &'static str> {
        let issuance = Self::get_total_issuance();
        Self::block_emission_step_with_issuance(issuance)
    }

    pub fn block_emission_step_with_issuance(issuance: u64) -> Result<u64, &'static str> {
        let block_emission: u64 = Self::get_block_emission_for_issuance(issuance)?;

        // Update the BlockEmission storage if the calculated emission is different from the current value
        if BlockEmission::<T>::get() != block_emission {
            // Call the on_halving hook.
            Self::on_halving(block_emission);
        }

        Ok(block_emission)
    }

    /// Returns the block emission for an issuance value.
    pub fn get_block_emission_for_issuance(issuance: u64) -> Result<u64, &'static str> {
        // Convert issuance to a fixed-point number for precise calculations
        let total_issuance: I96F32 = I96F32::from_num(issuance);

        // Check if the total issuance has reached or exceeded the total supply
        // If so, return 0 as no more tokens should be emitted
        if total_issuance >= I96F32::from_num(TotalSupply::<T>::get()) {
            return Ok(0);
        }

        // Calculate half of the total supply (10.5 million * 10^9 RAO) == 1/2 * 21 million TAO
        let half_total_supply =
            I96F32::from_num(2.0).saturating_mul(I96F32::from_num(10_500_000_000_000_000.0));

        // Calculate the ratio of total issuance to half of the total supply
        let division_result = total_issuance
            .checked_div(half_total_supply)
            .ok_or("Division failed")?;

        // Calculate 1 minus the division result
        let subtraction_result = I96F32::from_num(1.0)
            .checked_sub(division_result)
            .ok_or("Subtraction failed")?;

        // Calculate the reciprocal of the subtraction result
        let division_result_2 = I96F32::from_num(1.0)
            .checked_div(subtraction_result)
            .ok_or("Division failed")?;

        // Calculate the logarithm base 2 of the reciprocal
        let residual: I96F32 =
            log2(division_result_2).map_err(|_| "Logarithm calculation failed")?;

        // Floor the residual to smooth out the emission rate
        let floored_residual: I96F32 = residual.floor();

        // Convert floored_residual to an integer for use in the power calculation
        let floored_residual_int: u64 = floored_residual.to_num::<u64>();

        // Calculate 2^floored_residual
        let mut multiplier: I96F32 = I96F32::from_num(1.0);
        for _ in 0..floored_residual_int {
            multiplier = multiplier.saturating_mul(I96F32::from_num(2.0));
        }

        // Calculate the block emission percentage as 1 / 2^floored_residual
        let block_emission_percentage: I96F32 = I96F32::from_num(1.0).saturating_div(multiplier);

        // Calculate the actual emission by multiplying the percentage with the default block emission
        let block_emission: I96F32 = block_emission_percentage
            .saturating_mul(I96F32::from_num(DefaultBlockEmission::<T>::get()));

        // Convert the calculated emission to u64
        let block_emission_u64: u64 = block_emission.to_num::<u64>();

        // Return the calculated block emission
        Ok(block_emission_u64)
    }
}
