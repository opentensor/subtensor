use frame_support::traits::fungible::Inspect;

use super::*;

impl<T: Config> Pallet<T> {
    /// Checks [`TotalIssuance`] equals the sum of currency issuance, total stake, and total subnet
    /// locked.
    #[allow(clippy::arithmetic_side_effects, clippy::expect_used)]
    pub(crate) fn check_total_issuance() -> Result<(), sp_runtime::TryRuntimeError> {
        // Get the total currency issuance
        let currency_issuance = u64::from(<T as Config>::Currency::total_issuance()) as i128;
        let mut total_issuance = u64::from(TotalIssuance::<T>::get()) as i128;

        log::info!("=== Try runtime check_total_issuance ===");
        log::info!("  currency_issuance: {}", currency_issuance);
        log::info!("  total_issuance: {}", total_issuance);

        // If balances total issuance is greater than 21M, we're on devnet or testnet, ignore
        // this check, TI is off for multiple reasons.
        if currency_issuance > 21_000_000_000_000_000_i128 {
            return Ok(());
        }

        // If there's an exact match, it means we are past imbalances upgrade
        if currency_issuance == total_issuance {
            return Ok(());
        }

        // Effect from migrate_total_issuance adjustment diff
        total_issuance =
            u64::from(SubnetTAO::<T>::iter().fold(TaoBalance::ZERO, |acc, (_, v)| acc + v)) as i128;

        // Calculate total SubnetLock
        let mut total_locked = 0_i128;
        let initial_pool_tao = NetworkMinLockCost::<T>::get();
        SubnetLocked::<T>::iter().for_each(|(netuid, tao)| {
            if Pallet::<T>::get_subnet_account_id(netuid).is_some() {
                let tao_lock = tao.saturating_sub(initial_pool_tao);
                total_locked += u64::from(tao_lock) as i128;
            }
        });
        log::info!("  total_locked: {}", total_locked);

        // Calculate the expected total issuance
        let mut total_stake = 0_i128;
        SubnetTAO::<T>::iter().for_each(|(netuid, tao)| {
            if Pallet::<T>::get_subnet_account_id(netuid).is_some() {
                total_stake += u64::from(tao) as i128;
            }
        });
        log::info!("  total stake: {}", total_stake);
        let expected_total_issuance = currency_issuance + total_stake + total_locked;
        log::info!("  expected_total_issuance: {}", expected_total_issuance);

        // Verify the diff between calculated TI and actual TI is less than delta
        //
        // These values can be off slightly due to float rounding errors.
        // They are corrected every runtime upgrade.
        let delta = 1000_i128;
        let diff = (total_issuance - expected_total_issuance).abs();
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
