use super::*;
use frame_support::pallet_prelude::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::prelude::string::String;
use sp_core::crypto::Ss58Codec;
use sp_runtime::AccountId32;
use substrate_fixed::types::U64F64;

pub fn decode_account_id32<T: Config>(ss58_string: &str) -> Option<T::AccountId> {
    let account_id32: AccountId32 = AccountId32::from_ss58check(ss58_string).ok()?;
    let mut account_id32_slice: &[u8] = account_id32.as_ref();
    T::AccountId::decode(&mut account_id32_slice).ok()
}

/// Fixes the consequences of a bug in `perform_hotkey_swap_on_one_subnet` where
/// `transfer_root_claimable_for_new_hotkey` unconditionally transferred the **entire**
/// `RootClaimable` BTreeMap (all subnets) from the old hotkey to the new hotkey, even
/// during a single-subnet swap.
///
/// This left the old hotkey with:
///   - `RootClaimable[old_hotkey]` = empty (wiped for ALL subnets)
///   - `RootClaimed[(subnet, old_hotkey, coldkey)]` = old watermarks (for non-swapped subnets)
///
/// Resulting in `owed = claimable_rate * root_stake - root_claimed = 0 - positive = negative → 0`,
/// effectively freezing root dividends for the old hotkey.
///
/// Remediation: restore the pre-swap `RootClaimable` and `RootClaimed` storage maps
pub fn migrate_fix_root_claimed_overclaim<T: Config>() -> Weight {
    let migration_name = b"migrate_fix_root_claimed_overclaim".to_vec();
    let mut weight = T::DbWeight::get().reads(1);

    if HasMigrationRun::<T>::get(&migration_name) {
        log::info!(
            "Migration '{:?}' has already run. Skipping.",
            String::from_utf8_lossy(&migration_name)
        );
        return weight;
    }

    log::info!(
        "Running migration '{}'",
        String::from_utf8_lossy(&migration_name)
    );

    // Only run on mainnet.
    // Mainnet genesis: 0x2f0555cc76fc2840a25a6ea3b9637146806f1f44b090c175ffde2a7e5ab36c03
    let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
    let genesis_bytes = genesis_hash.as_ref();
    let mainnet_genesis =
        hex_literal::hex!("2f0555cc76fc2840a25a6ea3b9637146806f1f44b090c175ffde2a7e5ab36c03");
    if genesis_bytes == mainnet_genesis {
        let old_hotkey_ss58 = "5GmvyePN9aYErXBBhBnxZKGoGk4LKZApE4NkaSzW62CYCYNA";
        let new_hotkey_ss58 = "5H6BqkzjYvViiqp7rQLXjpnaEmW7U9CoKxXhQ4efMqtX1mQw";
        let netuid = NetUid::from(27);

        if let (Some(old_hotkey), Some(new_hotkey)) = (
            decode_account_id32::<T>(old_hotkey_ss58),
            decode_account_id32::<T>(new_hotkey_ss58),
        ) {
            // Reverting the Root Claimable because it only should happen for root subnet
            Pallet::<T>::transfer_root_claimable_for_new_hotkey(&new_hotkey, &old_hotkey);
            weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));

            let alpha_values: Vec<((T::AccountId, NetUid), U64F64)> =
                Alpha::<T>::iter_prefix((&new_hotkey,)).collect();
            weight.saturating_accrue(T::DbWeight::get().reads(alpha_values.len() as u64));

            // Reverting back root claimed
            for ((coldkey, _), _) in alpha_values
                .into_iter()
                .filter(|((_, netuid_alpha), alpha)| *netuid_alpha == netuid && *alpha != 0)
            {
                Pallet::<T>::transfer_root_claimed_for_new_keys(
                    netuid,
                    &new_hotkey,
                    &old_hotkey,
                    &coldkey,
                    &coldkey,
                );
                weight.saturating_accrue(T::DbWeight::get().reads_writes(2, 2));
            }
        } else {
            log::error!("Failed to decode hotkeys, skipping");
        }
    }

    // Mark migration as completed
    HasMigrationRun::<T>::insert(&migration_name, true);
    weight.saturating_accrue(T::DbWeight::get().writes(1));

    log::info!("Migration 'migrate_fix_root_claimed_overclaim' completed.");

    weight
}
