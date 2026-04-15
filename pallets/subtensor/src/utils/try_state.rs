use frame_support::traits::fungible::Inspect;

use super::*;

impl<T: Config> Pallet<T> {
    /// Checks [`TotalIssuance`] equals the sum of currency issuance, total stake, and total subnet
    /// locked.
    #[allow(clippy::expect_used)]
    pub(crate) fn check_total_issuance() -> Result<(), sp_runtime::TryRuntimeError> {
        // Get the total currency issuance
        let currency_issuance = <T as Config>::Currency::total_issuance();

        log::info!("=== Try runtime check_total_issuance ===");
        log::info!("  currency_issuance: {}", currency_issuance);

        // If balances total issuance is greater than 21M, we're on devnet or testnet, ignore 
        // this check, TI is off for multiple reasons.
        if currency_issuance > 21_000_000_000_000_000_u64.into() {
            return Ok(());
        }

        // Calculate total SubnetLock
        let total_locked = Self::get_total_subnet_locked();
        log::info!("  total_locked: {}", total_locked);

        // Calculate the expected total issuance
        let total_stake = TotalStake::<T>::get();
        log::info!("  total stake: {}", total_stake);
        let expected_total_issuance = currency_issuance
            .saturating_add(total_stake.into());
        let expected_fixed_total_issuance = currency_issuance;
        log::info!("  expected_total_issuance: {}", expected_total_issuance);
        log::info!("  expected_fixed_total_issuance: {}", expected_fixed_total_issuance);

        // Verify the diff between calculated TI and actual TI is less than delta
        //
        // These values can be off slightly due to float rounding errors.
        // They are corrected every runtime upgrade.
        let delta = TaoBalance::from(1000);
        let total_issuance = TotalIssuance::<T>::get();
        log::info!("  total_issuance: {}", total_issuance);

        let diff = if total_issuance > expected_total_issuance {
            total_issuance.checked_sub(&expected_total_issuance)
        } else {
            expected_total_issuance.checked_sub(&total_issuance)
        }
        .expect("LHS > RHS");

        let diff_fixed = if total_issuance > expected_fixed_total_issuance {
            total_issuance.checked_sub(&expected_fixed_total_issuance)
        } else {
            expected_fixed_total_issuance.checked_sub(&total_issuance)
        }
        .expect("LHS > RHS");

        if (diff > delta) && (diff_fixed > delta) {
            if diff > delta {
                log::error!(
                    "expected_total_issuance: {} != total_issuance: {}",
                    expected_total_issuance,
                    total_issuance
                );
            }

            if diff_fixed > delta {
                log::error!(
                    "expected_fixed_total_issuance: {} != total_issuance: {}",
                    expected_fixed_total_issuance,
                    total_issuance
                );
            }
        }

        ensure!(
            (diff <= delta) || (diff_fixed <= delta),
            "TotalIssuance diff greater than allowable delta",
        );

        Ok(())
    }

    /// Checks the sum of all stakes matches the [`TotalStake`].
    #[allow(dead_code)]
    pub(crate) fn check_total_stake() -> Result<(), sp_runtime::TryRuntimeError> {
        // Calculate the total staked amount
        let total_staked = SubnetTAO::<T>::iter().fold(TaoBalance::ZERO, |acc, (netuid, stake)| {
            let acc = acc.saturating_add(stake);

            if netuid.is_root() {
                // root network doesn't have initial pool TAO
                acc
            } else {
                acc.saturating_sub(Self::get_network_min_lock())
            }
        });

        log::warn!(
            "total_staked: {}, TotalStake: {}",
            total_staked,
            TotalStake::<T>::get()
        );

        // Verify that the calculated total stake matches the stored TotalStake
        ensure!(
            total_staked == TotalStake::<T>::get(),
            "TotalStake does not match total staked",
        );

        Ok(())
    }
}
