use super::*;
use frame_support::{traits::Get, weights::Weight};
use log;
use scale_info::prelude::string::String;
use sp_std::vec::Vec;

/// `HasMigrationRun` key for this migration.
pub const MIGRATION_NAME: &[u8] = b"enable_claim_based_miner_emissions_v1";

/// Turns claim-based miner emissions on for every live subnet.
///
/// Storage default for [`ClaimBasedMinerEmissions`] stays `false` so that:
/// * an already-run migration that sudo later flips a subnet off is not undone by
///   a later upgrade that only re-reads the default, and
/// * try-runtime sees a deterministic first-pass write / later-pass no-op.
///
/// New subnets are enabled in [`Pallet::init_new_network`].
pub fn migrate_enable_claim_based_miner_emissions<T: Config>() -> Weight {
    let mig_name: Vec<u8> = MIGRATION_NAME.to_vec();
    let mig_name_str = String::from_utf8_lossy(&mig_name);

    let mut total_weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&mig_name) {
        log::info!("Migration '{mig_name_str}' already executed - skipping");
        return total_weight;
    }

    log::info!("Running migration '{mig_name_str}'");

    let mut enabled = 0u64;
    for (netuid, exists) in NetworksAdded::<T>::iter() {
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
        if !exists || netuid.is_root() {
            continue;
        }
        ClaimBasedMinerEmissions::<T>::insert(netuid, true);
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        enabled = enabled.saturating_add(1);
    }

    HasMigrationRun::<T>::insert(&mig_name, true);
    total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{mig_name_str}' completed; enabled claim-based miner emissions on {enabled} subnets"
    );

    total_weight
}

/// [`OnRuntimeUpgrade`] wrapper registered in the runtime `Migrations` tuple.
pub mod enable_claim_based_miner_emissions {
    use super::*;
    use frame_support::traits::OnRuntimeUpgrade;
    use sp_std::marker::PhantomData;

    #[cfg(feature = "try-runtime")]
    use codec::{Decode, Encode};
    #[cfg(feature = "try-runtime")]
    use frame_support::ensure;
    #[cfg(feature = "try-runtime")]
    use sp_runtime::TryRuntimeError;

    #[cfg(feature = "try-runtime")]
    type PreUpgradeState = (bool, u32);

    pub struct Migration<T: Config>(PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for Migration<T> {
        fn on_runtime_upgrade() -> Weight {
            migrate_enable_claim_based_miner_emissions::<T>()
        }

        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
            let already_run = HasMigrationRun::<T>::get(MIGRATION_NAME.to_vec());
            let enabled = NetworksAdded::<T>::iter()
                .filter(|(netuid, exists)| {
                    *exists && !netuid.is_root() && ClaimBasedMinerEmissions::<T>::get(*netuid)
                })
                .count() as u32;
            Ok((already_run, enabled).encode())
        }

        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
            let (already_run, enabled_before): PreUpgradeState =
                Decode::decode(&mut &state[..]).map_err(|_| "pre_upgrade state must decode")?;
            ensure!(
                HasMigrationRun::<T>::get(MIGRATION_NAME.to_vec()),
                "the migration must be stamped"
            );
            if already_run {
                let enabled_after = NetworksAdded::<T>::iter()
                    .filter(|(netuid, exists)| {
                        *exists && !netuid.is_root() && ClaimBasedMinerEmissions::<T>::get(*netuid)
                    })
                    .count() as u32;
                ensure!(
                    enabled_after == enabled_before,
                    "an already-run migration must not change claim-based toggles"
                );
            } else {
                for (netuid, exists) in NetworksAdded::<T>::iter() {
                    if exists && !netuid.is_root() {
                        ensure!(
                            ClaimBasedMinerEmissions::<T>::get(netuid),
                            "every live non-root subnet must have claim-based miner emissions on"
                        );
                    }
                }
            }
            Ok(())
        }
    }
}
