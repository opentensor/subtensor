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
        let mut total_locked = TaoBalance::ZERO;
        let initial_pool_tao = NetworkMinLockCost::<T>::get();
        SubnetLocked::<T>::iter().for_each(|(netuid, tao)| {
            if Pallet::<T>::get_subnet_account_id(netuid).is_some() {
                let tao_lock = tao.saturating_sub(initial_pool_tao);
                total_locked = total_locked.saturating_add(tao_lock);
            }
        });
        log::info!("  total_locked: {}", total_locked);

        // Calculate the expected total issuance
        let mut total_stake = TaoBalance::ZERO;
        SubnetTAO::<T>::iter().for_each(|(netuid, tao)| {
            if Pallet::<T>::get_subnet_account_id(netuid).is_some() {
                total_stake = total_stake.saturating_add(tao);
            }
        });
        log::info!("  total stake: {}", total_stake);
        let expected_total_issuance = currency_issuance
            .saturating_add(total_stake)
            .saturating_add(total_locked);
        log::info!("  expected_total_issuance: {}", expected_total_issuance);

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

        if diff > delta {
            log::error!(
                "expected_total_issuance: {} != total_issuance: {}",
                expected_total_issuance,
                total_issuance
            );
        }

        ensure!(
            diff <= delta,
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
