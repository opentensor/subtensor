use super::*;
use alloc::string::String;
use frame_support::{traits::Get, weights::Weight};
use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;

fn get_account_id_from_ss58<T: Config>(ss58_str: &str) -> Result<T::AccountId, codec::Error> {
    let account =
        AccountId32::from_ss58check(ss58_str).map_err(|_| codec::Error::from("Invalid SS58"))?;
    let onchain_account = T::AccountId::decode(&mut account.as_ref())?;

    Ok(onchain_account)
}

/**
 * Migrates the pending emissions from the old hotkey to the new hotkey.
 * Also migrates the stake entry of (old_hotkey, 0x000) to the pending emissions of the new hotkey.
 */
fn migrate_pending_emissions_including_null_stake<T: Config>(
    old_hotkey: &T::AccountId,
    new_hotkey: &T::AccountId,
    migration_account: &T::AccountId,
) -> Weight {
    let mut weight = T::DbWeight::get().reads(0);
    let null_account = &DefaultAccount::<T>::get();
    weight.saturating_accrue(T::DbWeight::get().reads(1));

    // Get the pending emissions for the OLD hotkey
    let pending_emissions_old: u64 = PendingdHotkeyEmission::<T>::get(old_hotkey);
    PendingdHotkeyEmission::<T>::remove(old_hotkey);
    weight.saturating_accrue(T::DbWeight::get().reads(1));

    // Get the stake for the 0x000 key
    let null_stake = Stake::<T>::get(old_hotkey, null_account);
    weight.saturating_accrue(T::DbWeight::get().reads(1));
    // Remove
    Stake::<T>::remove(old_hotkey, null_account);
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    let new_total_coldkey_stake =
        TotalColdkeyStake::<T>::get(null_account).saturating_sub(null_stake);
    if new_total_coldkey_stake == 0 {
        TotalColdkeyStake::<T>::remove(null_account);
    } else {
        TotalColdkeyStake::<T>::insert(null_account, new_total_coldkey_stake);
    }
    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

    let new_staking_hotkeys = StakingHotkeys::<T>::get(null_account);
    let new_staking_hotkeys = new_staking_hotkeys
        .into_iter()
        .filter(|hk| hk != old_hotkey)
        .collect::<Vec<_>>();
    StakingHotkeys::<T>::insert(null_account, new_staking_hotkeys);
    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

    // Insert the stake from the null account to the MIGRATION account under the OLD hotkey
    Stake::<T>::insert(old_hotkey, migration_account, null_stake);
    TotalColdkeyStake::<T>::insert(
        migration_account,
        TotalColdkeyStake::<T>::get(migration_account).saturating_add(null_stake),
    );
    let mut new_staking_hotkeys = StakingHotkeys::<T>::get(migration_account);
    if !new_staking_hotkeys.contains(old_hotkey) {
        new_staking_hotkeys.push(old_hotkey.clone());
    }
    StakingHotkeys::<T>::insert(migration_account, new_staking_hotkeys);
    weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 3));

    // Get the pending emissions for the NEW hotkey
    let pending_emissions_new: u64 = PendingdHotkeyEmission::<T>::get(new_hotkey);
    weight.saturating_accrue(T::DbWeight::get().reads(1));

    // Add the pending emissions for the new hotkey and the old hotkey
    PendingdHotkeyEmission::<T>::insert(
        new_hotkey,
        pending_emissions_new.saturating_add(pending_emissions_old),
    );
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    weight
}

// This executes the migration to fix the pending emissions
// This also migrates the stake entry of (old_hotkey, 0x000) to the Migration Account for
// both the old hotkeys.
pub fn do_migrate_fix_pending_emission<T: Config>() -> Weight {
    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    let taostats_old_hotkey = "5Hddm3iBFD2GLT5ik7LZnT3XJUnRnN8PoeCFgGQgawUVKNm8";
    let taostats_new_hotkey = "5GKH9FPPnWSUoeeTJp19wVtd84XqFW4pyK2ijV2GsFbhTrP1";
    let migration_coldkey = "5GeRjQYsobRWFnrbBmGe5ugme3rfnDVF69N45YtdBpUFsJG8";

    let taostats_old_hk_account = get_account_id_from_ss58::<T>(taostats_old_hotkey);
    let taostats_new_hk_account = get_account_id_from_ss58::<T>(taostats_new_hotkey);
    let migration_ck_account = get_account_id_from_ss58::<T>(migration_coldkey);

    match (
        taostats_old_hk_account,
        taostats_new_hk_account,
        migration_ck_account.clone(),
    ) {
        (Ok(taostats_old_hk_acct), Ok(taostats_new_hk_acct), Ok(migration_ck_account)) => {
            weight.saturating_accrue(migrate_pending_emissions_including_null_stake::<T>(
                &taostats_old_hk_acct,
                &taostats_new_hk_acct,
                &migration_ck_account,
            ));
            log::info!("Migrated pending emissions from taostats old hotkey to new hotkey");
        }
        _ => {
            log::warn!("Failed to get account id from ss58 for taostats hotkeys");
            return weight;
        }
    }

    let datura_old_hotkey = "5FKstHjZkh4v3qAMSBa1oJcHCLjxYZ8SNTSz1opTv4hR7gVB";
    let datura_new_hotkey = "5GP7c3fFazW9GXK8Up3qgu2DJBk8inu4aK9TZy3RuoSWVCMi";

    let datura_old_hk_account = get_account_id_from_ss58::<T>(datura_old_hotkey);
    let datura_new_hk_account = get_account_id_from_ss58::<T>(datura_new_hotkey);

    match (
        datura_old_hk_account,
        datura_new_hk_account,
        migration_ck_account,
    ) {
        (Ok(datura_old_hk_acct), Ok(datura_new_hk_acct), Ok(migration_ck_account)) => {
            weight.saturating_accrue(migrate_pending_emissions_including_null_stake::<T>(
                &datura_old_hk_acct,
                &datura_new_hk_acct,
                &migration_ck_account,
            ));
            log::info!("Migrated pending emissions from datura old hotkey to new hotkey");
        }
        _ => {
            log::warn!("Failed to get account id from ss58 for datura hotkeys");
            return weight;
        }
    }

    weight
}

/// Collection of storage item formats from the previous storage version.
///
/// Required so we can read values in the v0 storage format during the migration.
#[cfg(feature = "try-runtime")]
mod v0 {
    use subtensor_macros::freeze_struct;

    #[freeze_struct("2228babfc0580c62")]
    #[derive(codec::Encode, codec::Decode, Clone, PartialEq, Debug)]
    pub struct OldStorage {
        pub total_issuance_before: u64,
        pub total_stake_before: u64,
        pub expected_taostats_new_hk_pending_emission: u64,
        pub expected_datura_new_hk_pending_emission: u64,
        pub old_migration_stake_taostats: u64,
        pub old_null_stake_taostats: u64,
        pub old_migration_stake_datura: u64,
        pub old_null_stake_datura: u64,
    }
}

impl<T: Config> Pallet<T> {
    #[cfg(feature = "try-runtime")]
    fn check_null_stake_invariants(
        old_storage: v0::OldStorage,
    ) -> Result<(), sp_runtime::TryRuntimeError> {
        let null_account = &DefaultAccount::<T>::get();

        let taostats_old_hotkey = "5Hddm3iBFD2GLT5ik7LZnT3XJUnRnN8PoeCFgGQgawUVKNm8";
        let taostats_new_hotkey = "5GKH9FPPnWSUoeeTJp19wVtd84XqFW4pyK2ijV2GsFbhTrP1";
        let migration_coldkey = "5GeRjQYsobRWFnrbBmGe5ugme3rfnDVF69N45YtdBpUFsJG8";

        let taostats_old_hk_account = &get_account_id_from_ss58::<T>(taostats_old_hotkey);
        let taostats_new_hk_account = &get_account_id_from_ss58::<T>(taostats_new_hotkey);
        let migration_ck_account = &get_account_id_from_ss58::<T>(migration_coldkey);

        let old = old_storage;
        let null_stake_total = old
            .old_null_stake_taostats
            .saturating_add(old.old_null_stake_datura)
            .saturating_add(old.old_migration_stake_taostats)
            .saturating_add(old.old_migration_stake_datura);

        match (
            taostats_old_hk_account,
            taostats_new_hk_account,
            migration_ck_account,
        ) {
            (Ok(taostats_old_hk_acct), Ok(taostats_new_hk_acct), Ok(migration_ck_acct)) => {
                // Check the pending emission is added to new the TaoStats hotkey
                assert_eq!(
                    PendingdHotkeyEmission::<T>::get(taostats_new_hk_acct),
                    old.expected_taostats_new_hk_pending_emission
                );

                assert_eq!(PendingdHotkeyEmission::<T>::get(taostats_old_hk_acct), 0);

                assert_eq!(Stake::<T>::get(taostats_old_hk_acct, null_account), 0);

                assert!(StakingHotkeys::<T>::get(migration_ck_acct).contains(taostats_old_hk_acct));

                assert_eq!(
                    Self::get_stake_for_coldkey_and_hotkey(null_account, taostats_old_hk_acct),
                    0
                );

                // Check the total hotkey stake is the same
                assert_eq!(
                    TotalHotkeyStake::<T>::get(taostats_old_hk_acct),
                    old.old_null_stake_taostats
                        .saturating_add(old.old_migration_stake_taostats)
                );

                let new_null_stake_taostats =
                    Self::get_stake_for_coldkey_and_hotkey(migration_ck_acct, taostats_old_hk_acct);

                assert_eq!(
                    new_null_stake_taostats,
                    old.old_null_stake_taostats
                        .saturating_add(old.old_migration_stake_taostats)
                );
            }
            _ => {
                log::warn!("Failed to get account id from ss58 for taostats hotkeys");
                return Err("Failed to get account id from ss58 for taostats hotkeys".into());
            }
        }

        let datura_old_hotkey = "5FKstHjZkh4v3qAMSBa1oJcHCLjxYZ8SNTSz1opTv4hR7gVB";
        let datura_new_hotkey = "5GP7c3fFazW9GXK8Up3qgu2DJBk8inu4aK9TZy3RuoSWVCMi";

        let datura_old_hk_account = &get_account_id_from_ss58::<T>(datura_old_hotkey);
        let datura_new_hk_account = &get_account_id_from_ss58::<T>(datura_new_hotkey);

        match (
            datura_old_hk_account,
            datura_new_hk_account,
            migration_ck_account,
        ) {
            (Ok(datura_old_hk_acct), Ok(datura_new_hk_acct), Ok(migration_ck_acct)) => {
                // Check the pending emission is added to new Datura hotkey
                assert_eq!(
                    crate::PendingdHotkeyEmission::<T>::get(datura_new_hk_acct),
                    old.expected_datura_new_hk_pending_emission
                );

                // Check the pending emission is removed from old ones
                assert_eq!(PendingdHotkeyEmission::<T>::get(datura_old_hk_acct), 0);

                // Check the stake entry is removed
                assert_eq!(Stake::<T>::get(datura_old_hk_acct, null_account), 0);

                assert!(StakingHotkeys::<T>::get(migration_ck_acct).contains(datura_old_hk_acct));

                assert_eq!(
                    Self::get_stake_for_coldkey_and_hotkey(null_account, datura_old_hk_acct),
                    0
                );

                // Check the total hotkey stake is the same
                assert_eq!(
                    TotalHotkeyStake::<T>::get(datura_old_hk_acct),
                    old.old_null_stake_datura
                        .saturating_add(old.old_migration_stake_datura)
                );

                let new_null_stake_datura =
                    Self::get_stake_for_coldkey_and_hotkey(migration_ck_acct, datura_old_hk_acct);

                assert_eq!(
                    new_null_stake_datura,
                    old.old_null_stake_datura
                        .saturating_add(old.old_migration_stake_datura)
                );
            }
            _ => {
                log::warn!("Failed to get account id from ss58 for datura hotkeys");
                return Err("Failed to get account id from ss58 for datura hotkeys".into());
            }
        }

        match migration_ck_account {
            Ok(migration_ck_acct) => {
                // Check the migration key has stake with both *old* hotkeys
                assert_eq!(
                    TotalColdkeyStake::<T>::get(migration_ck_acct),
                    null_stake_total
                );
            }
            _ => {
                log::warn!("Failed to get account id from ss58 for migration coldkey");
                return Err("Failed to get account id from ss58 for migration coldkey".into());
            }
        }

        // Check the total issuance is the SAME following migration (no TAO issued)
        let expected_total_issuance = old.total_issuance_before;
        let expected_total_stake = old.total_stake_before;
        assert_eq!(Self::get_total_issuance(), expected_total_issuance);

        // Check total stake is the SAME following the migration (no new TAO staked)
        assert_eq!(TotalStake::<T>::get(), expected_total_stake);
        // Check the total stake maps are updated following the migration (removal of old null_account stake entries)
        assert_eq!(TotalColdkeyStake::<T>::get(null_account), 0);

        // Check staking hotkeys is updated
        assert_eq!(StakingHotkeys::<T>::get(null_account), vec![]);

        Ok(())
    }
}

pub mod migration {
    use frame_support::pallet_prelude::Weight;
    use frame_support::traits::OnRuntimeUpgrade;
    use sp_core::Get;

    use super::*;

    pub struct Migration<T: Config>(PhantomData<T>);

    #[cfg(feature = "try-runtime")]
    fn get_old_storage_values<T: Config>() -> Result<v0::OldStorage, sp_runtime::TryRuntimeError> {
        log::info!("Getting old storage values for migration");

        let null_account = &DefaultAccount::<T>::get();
        let migration_coldkey = "5GeRjQYsobRWFnrbBmGe5ugme3rfnDVF69N45YtdBpUFsJG8";
        let migration_account = &get_account_id_from_ss58::<T>(migration_coldkey);

        let taostats_old_hotkey = "5Hddm3iBFD2GLT5ik7LZnT3XJUnRnN8PoeCFgGQgawUVKNm8";
        let taostats_new_hotkey = "5GKH9FPPnWSUoeeTJp19wVtd84XqFW4pyK2ijV2GsFbhTrP1";

        let taostats_old_hk_account = &get_account_id_from_ss58::<T>(taostats_old_hotkey);
        let taostats_new_hk_account = &get_account_id_from_ss58::<T>(taostats_new_hotkey);

        let total_issuance_before = crate::Pallet::<T>::get_total_issuance();
        let mut expected_taostats_new_hk_pending_emission: u64 = 0;
        let mut expected_datura_new_hk_pending_emission: u64 = 0;
        let (old_null_stake_taostats, old_migration_stake_taostats) = match (
            taostats_old_hk_account,
            taostats_new_hk_account,
            migration_account,
        ) {
            (Ok(taostats_old_hk_acct), Ok(taostats_new_hk_acct), Ok(migration_acct)) => {
                expected_taostats_new_hk_pending_emission =
                    expected_taostats_new_hk_pending_emission
                        .saturating_add(PendingdHotkeyEmission::<T>::get(taostats_old_hk_acct))
                        .saturating_add(PendingdHotkeyEmission::<T>::get(taostats_new_hk_acct));

                Ok::<(u64, u64), sp_runtime::TryRuntimeError>((
                    crate::Pallet::<T>::get_stake_for_coldkey_and_hotkey(
                        null_account,
                        taostats_old_hk_acct,
                    ),
                    crate::Pallet::<T>::get_stake_for_coldkey_and_hotkey(
                        migration_acct,
                        taostats_old_hk_acct,
                    ),
                ))
            }
            _ => {
                log::warn!("Failed to get account id from ss58 for taostats hotkeys");
                Err("Failed to get account id from ss58 for taostats hotkeys".into())
            }
        }?;

        let datura_old_hotkey = "5FKstHjZkh4v3qAMSBa1oJcHCLjxYZ8SNTSz1opTv4hR7gVB";
        let datura_new_hotkey = "5GP7c3fFazW9GXK8Up3qgu2DJBk8inu4aK9TZy3RuoSWVCMi";

        let datura_old_hk_account = &get_account_id_from_ss58::<T>(datura_old_hotkey);
        let datura_new_hk_account = &get_account_id_from_ss58::<T>(datura_new_hotkey);

        let (old_null_stake_datura, old_migration_stake_datura) = match (
            datura_old_hk_account,
            datura_new_hk_account,
            migration_account,
        ) {
            (Ok(datura_old_hk_acct), Ok(datura_new_hk_acct), Ok(migration_acct)) => {
                expected_datura_new_hk_pending_emission = expected_datura_new_hk_pending_emission
                    .saturating_add(PendingdHotkeyEmission::<T>::get(datura_old_hk_acct))
                    .saturating_add(PendingdHotkeyEmission::<T>::get(datura_new_hk_acct));

                Ok::<(u64, u64), sp_runtime::TryRuntimeError>((
                    crate::Pallet::<T>::get_stake_for_coldkey_and_hotkey(
                        null_account,
                        datura_old_hk_acct,
                    ),
                    crate::Pallet::<T>::get_stake_for_coldkey_and_hotkey(
                        migration_acct,
                        datura_old_hk_acct,
                    ),
                ))
            }
            _ => {
                log::warn!("Failed to get account id from ss58 for datura hotkeys");
                Err("Failed to get account id from ss58 for datura hotkeys".into())
            }
        }?;

        let total_stake_before: u64 = crate::Pallet::<T>::get_total_stake();

        let result = v0::OldStorage {
            total_issuance_before,
            total_stake_before,
            expected_taostats_new_hk_pending_emission,
            expected_datura_new_hk_pending_emission,
            old_migration_stake_taostats,
            old_null_stake_taostats,
            old_migration_stake_datura,
            old_null_stake_datura,
        };

        log::info!("Got old storage values for migration");

        Ok(result)
    }

    impl<T: Config> OnRuntimeUpgrade for Migration<T> {
        /// Runs the migration to fix the pending emissions.
        #[cfg(feature = "try-runtime")]
        fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
            use codec::Encode;

            // Get the old storage values
            match get_old_storage_values::<T>() {
                Ok(old_storage) => {
                    log::info!("Successfully got old storage values for migration");
                    let encoded = old_storage.encode();

                    Ok(encoded)
                }
                Err(e) => {
                    log::error!("Failed to get old storage values for migration: {:?}", e);
                    Err("Failed to get old storage values for migration".into())
                }
            }
        }

        // Runs the migrate function for the fix_pending_emission migration
        fn on_runtime_upgrade() -> Weight {
            let migration_name = b"fix_pending_emission".to_vec();

            // Initialize the weight with one read operation.
            let mut weight = T::DbWeight::get().reads(1);

            // Check if the migration has already run
            if HasMigrationRun::<T>::get(&migration_name) {
                log::info!(
                    "Migration '{:?}' has already run. Skipping.",
                    migration_name
                );
                return Weight::zero();
            }

            log::info!(
                "Running migration '{}'",
                String::from_utf8_lossy(&migration_name)
            );

            // Run the migration
            weight.saturating_accrue(
                migrations::migrate_fix_pending_emission::do_migrate_fix_pending_emission::<T>(),
            );

            // Mark the migration as completed
            HasMigrationRun::<T>::insert(&migration_name, true);
            weight.saturating_accrue(T::DbWeight::get().writes(1));

            log::info!(
                "Migration '{:?}' completed. Marked in storage.",
                String::from_utf8_lossy(&migration_name)
            );

            // Return the migration weight.
            weight
        }

        /// Performs post-upgrade checks to ensure the migration was successful.
        ///
        /// This function is only compiled when the "try-runtime" feature is enabled.
        #[cfg(feature = "try-runtime")]
        fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
            use codec::Decode;

            let old_storage: v0::OldStorage =
                v0::OldStorage::decode(&mut &state[..]).map_err(|_| {
                    sp_runtime::TryRuntimeError::Other("Failed to decode old value from storage")
                })?;

            // Verify that all null stake invariants are satisfied after the migration
            crate::Pallet::<T>::check_null_stake_invariants(old_storage)?;

            Ok(())
        }
    }
}
