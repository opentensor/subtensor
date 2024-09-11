use super::*;
use alloc::string::String;
use frame_support::{traits::Get, weights::Weight};
use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;

fn swap_pending_emissions<T: Config>(
    old_hotkey: &T::AccountId,
    new_hotkey: &T::AccountId,
) -> Weight {
    let mut weight = T::DbWeight::get().reads(0);

    // Get the pending emissions for the old hotkey
    let pending_emissions = PendingdHotkeyEmission::<T>::get(old_hotkey);
    weight.saturating_accrue(T::DbWeight::get().reads(1));

    // Remove the pending emissions for the old hotkey
    PendingdHotkeyEmission::<T>::remove(old_hotkey);
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    // Add the pending emissions for the new hotkey
    PendingdHotkeyEmission::<T>::insert(new_hotkey, pending_emissions);
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    weight
}

fn get_account_id_from_ss58<T: Config>(ss58_str: &str) -> T::AccountId {
    let account = AccountId32::from_ss58check(ss58_str).unwrap();
    let onchain_account = T::AccountId::decode(&mut account.as_ref()).unwrap();

    onchain_account
}

fn unstake_old_hotkey_and_move_to_pending<T: Config>(
    old_hotkey: &T::AccountId,
    new_hotkey: &T::AccountId,
) -> Weight {
    let mut weight = T::DbWeight::get().reads(0);
    let null_account = DefaultAccount::<T>::get();
    weight.saturating_accrue(T::DbWeight::get().reads(1));

    // Get the pending emissions for the new hotkey
    let pending_emissions = PendingdHotkeyEmission::<T>::get(new_hotkey);
    weight.saturating_accrue(T::DbWeight::get().reads(1));

    // Get the stake for the 0x000 key
    let null_stake = Stake::<T>::get(&old_hotkey, &null_account);
    weight.saturating_accrue(T::DbWeight::get().reads(1));
    // Remove
    Stake::<T>::remove(&old_hotkey, &null_account);
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    let new_total_coldkey_stake =
        TotalColdkeyStake::<T>::get(old_hotkey).saturating_sub(null_stake);
    if new_total_coldkey_stake == 0 {
        TotalColdkeyStake::<T>::remove(old_hotkey);
    } else {
        TotalColdkeyStake::<T>::insert(old_hotkey, new_total_coldkey_stake);
    }
    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

    let new_total_hotkey_stake = TotalHotkeyStake::<T>::get(old_hotkey).saturating_sub(null_stake);
    if new_total_hotkey_stake == 0 {
        TotalHotkeyStake::<T>::remove(old_hotkey);
    } else {
        TotalHotkeyStake::<T>::insert(old_hotkey, new_total_hotkey_stake);
    }
    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

    TotalStake::<T>::put(TotalStake::<T>::get().saturating_sub(null_stake));
    weight.saturating_accrue(T::DbWeight::get().reads_writes(1, 1));

    // Add stake to the pending emissions for the new hotkey
    PendingdHotkeyEmission::<T>::insert(new_hotkey, pending_emissions.saturating_add(null_stake));
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    weight
}

pub fn do_migrate_fix_pending_emission<T: Config>() -> Weight {
    // Initialize the weight with one read operation.
    let mut weight = T::DbWeight::get().reads(1);

    let taostats_old_hotkey = "5Hddm3iBFD2GLT5ik7LZnT3XJUnRnN8PoeCFgGQgawUVKNm8";
    let taostats_new_hotkey = "5GKH9FPPnWSUoeeTJp19wVtd84XqFW4pyK2ijV2GsFbhTrP1";

    let taostats_old_hk_account: T::AccountId = get_account_id_from_ss58::<T>(taostats_old_hotkey);
    let taostats_new_hk_account: T::AccountId = get_account_id_from_ss58::<T>(taostats_new_hotkey);

    weight.saturating_accrue(unstake_old_hotkey_and_move_to_pending::<T>(
        &taostats_old_hk_account,
        &taostats_new_hk_account,
    ));

    let datura_old_hotkey = "5FKstHjZkh4v3qAMSBa1oJcHCLjxYZ8SNTSz1opTv4hR7gVB";
    let datura_new_hotkey = "5GP7c3fFazW9GXK8Up3qgu2DJBk8inu4aK9TZy3RuoSWVCMi";

    let datura_old_hk_account: T::AccountId = get_account_id_from_ss58::<T>(datura_old_hotkey);
    let datura_new_hk_account: T::AccountId = get_account_id_from_ss58::<T>(datura_new_hotkey);

    weight.saturating_accrue(swap_pending_emissions::<T>(
        &datura_old_hk_account,
        &datura_new_hk_account,
    ));

    weight
}
// Public migrate function to be called by Lib.rs on upgrade.
pub fn migrate_fix_pending_emission<T: Config>() -> Weight {
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
    weight.saturating_accrue(do_migrate_fix_pending_emission::<T>());

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
