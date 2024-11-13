use super::*;

impl<T: Config> Pallet<T> {
    /// Checks if the accounting invariants for [`TotalStake`], [`TotalSubnetLocked`], and [`TotalIssuance`] are correct.
    ///
    /// This function verifies that:
    /// 1. The sum of all stakes matches the [`TotalStake`].
    /// 2. The [`TotalSubnetLocked`] is correctly calculated.
    /// 3. The [`TotalIssuance`] equals the sum of currency issuance, total stake, and total subnet locked.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all invariants are correct, otherwise returns an error.
    #[cfg(feature = "try-runtime")]
    pub fn check_accounting_invariants() -> Result<(), sp_runtime::TryRuntimeError> {
        use frame_support::traits::fungible::Inspect;

        // Calculate the total staked amount
        let mut total_staked: u64 = 0;
        for (_hotkey, _coldkey, stake) in Stake::<T>::iter() {
            total_staked = total_staked.saturating_add(stake);
        }

        // Verify that the calculated total stake matches the stored TotalStake
        ensure!(
            total_staked == TotalStake::<T>::get(),
            "TotalStake does not match total staked",
        );

        // Get the total subnet locked amount
        let total_subnet_locked: u64 = Self::get_total_subnet_locked();

        // Get the total currency issuance
        let currency_issuance: u64 = T::Currency::total_issuance();

        // Calculate the expected total issuance
        let expected_total_issuance: u64 = currency_issuance
            .saturating_add(total_staked)
            .saturating_add(total_subnet_locked);

        // Verify the diff between calculated TI and actual TI is less than delta
        //
        // These values can be off slightly due to float rounding errors.
        // They are corrected every runtime upgrade.
        const DELTA: u64 = 1000;
        let diff = if TotalIssuance::<T>::get() > expected_total_issuance {
            TotalIssuance::<T>::get().checked_sub(expected_total_issuance)
        } else {
            expected_total_issuance.checked_sub(TotalIssuance::<T>::get())
        }
        .expect("LHS > RHS");
        ensure!(
            diff <= DELTA,
            "TotalIssuance diff greater than allowable delta",
        );

        Ok(())
    }
}
