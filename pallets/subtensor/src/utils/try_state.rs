use alloc::collections::{BTreeMap, BTreeSet};

use frame_support::traits::fungible::Inspect;
use frame_system::pallet_prelude::BlockNumberFor;
use subtensor_runtime_common::NetUid;

use super::*;
use crate::root_registered::RootRegisteredInspector;

impl<T: Config> Pallet<T> {
    /// Checks [`TotalIssuance`] equals the sum of currency issuance, total stake, and total subnet
    /// locked.
    #[allow(clippy::arithmetic_side_effects, clippy::expect_used)]
    pub(crate) fn check_total_issuance() -> Result<(), sp_runtime::TryRuntimeError> {
        // Get the total currency issuance
        let currency_issuance = <T as Config>::Currency::total_issuance();
        let total_issuance = TotalIssuance::<T>::get();

        log::info!("=== Try runtime check_total_issuance ===");
        log::info!("  currency_issuance: {}", currency_issuance);
        log::info!("  total_issuance: {}", total_issuance);

        // If balances total issuance is greater than 21M, we're on devnet or testnet, ignore
        // this check, TI is off for multiple reasons.
        if currency_issuance > 21_000_000_000_000_000_u64.into() {
            return Ok(());
        }

        // If there's an exact match, it means we are past imbalances upgrade
        if currency_issuance == total_issuance {
            return Ok(());
        }

        // Calculate the expected total issuance
        let expected_total_issuance =
            currency_issuance.saturating_add(TotalStake::<T>::get().into());

        // Verify the diff between calculated TI and actual TI is less than delta
        // Allow greater tolerance for non-mainnet
        let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
        let genesis_bytes = genesis_hash.as_ref();
        let mainnet_genesis =
            hex_literal::hex!("2f0555cc76fc2840a25a6ea3b9637146806f1f44b090c175ffde2a7e5ab36c03");
        let delta = if genesis_bytes == mainnet_genesis {
            TaoBalance::from(1000)
        } else {
            TaoBalance::from(1_000_000_000_000_u64)
        };

        let diff = if total_issuance > expected_total_issuance {
            total_issuance.checked_sub(&expected_total_issuance)
        } else {
            expected_total_issuance.checked_sub(&total_issuance)
        }
        .expect("LHS > RHS");

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

    /// Verifies that `RootRegisteredHotkeyCount` matches, for every coldkey,
    /// the actual number of owned hotkeys that are registered on the root
    /// subnet. Both directions are checked: stored entries must agree with
    /// the computed count, and no coldkey with root-registered hotkeys may
    /// be missing from the index.
    pub(crate) fn check_root_registered_hotkey_count() -> Result<(), sp_runtime::TryRuntimeError> {
        let mut expected: BTreeMap<T::AccountId, u32> = BTreeMap::new();
        for (_uid, hotkey) in Keys::<T>::iter_prefix(NetUid::ROOT) {
            let owner = Owner::<T>::get(&hotkey);
            expected
                .entry(owner)
                .and_modify(|c| *c = c.saturating_add(1))
                .or_insert(1);
        }

        for (coldkey, stored) in RootRegisteredHotkeyCount::<T>::iter() {
            let expected_count = expected.remove(&coldkey).unwrap_or(0);
            ensure!(
                stored == expected_count,
                "RootRegisteredHotkeyCount mismatch for coldkey",
            );
        }

        ensure!(
            expected.is_empty(),
            "RootRegisteredHotkeyCount missing entries for coldkeys with root hotkeys",
        );

        Ok(())
    }

    /// Verifies that the inspector's view of the root-registered
    /// coldkey set matches `RootRegisteredHotkeyCount` exactly.
    /// Skipped when `T::RootRegisteredInspector` returns `None`
    /// (test mocks that do not wire up an external mirror).
    pub(crate) fn check_root_registered_matches_inspector()
    -> Result<(), sp_runtime::TryRuntimeError> {
        let Some(actual_members) = T::RootRegisteredInspector::members() else {
            return Ok(());
        };
        let actual: BTreeSet<T::AccountId> = actual_members.into_iter().collect();
        let expected: BTreeSet<T::AccountId> = RootRegisteredHotkeyCount::<T>::iter()
            .map(|(coldkey, _)| coldkey)
            .collect();
        ensure!(
            actual == expected,
            "RootRegisteredInspector members do not match root-registered coldkey set",
        );
        Ok(())
    }
}
