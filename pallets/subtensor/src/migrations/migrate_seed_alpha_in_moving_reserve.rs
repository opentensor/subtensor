use alloc::string::String;

use frame_support::{traits::Get, weights::Weight};
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::Token;

use super::*;

/// Seed the Alpha-reserve EMA (`SubnetAlphaInMovingReserve`) from the live
/// `SubnetAlphaIn` for every subnet.
///
/// The derivative references (`short_t_ref` / `long_a_ref`) read this lagged
/// reserve instead of the spot reserve so an in-block swap cannot inflate
/// capacity. A cold (`0`) EMA makes them fall back to the live reserve until it
/// warms — a brief window where the spot value is read again. Seeding it to the
/// current reserve at upgrade closes that window immediately; the per-block tick
/// in `update_moving_price` carries it forward from there.
pub fn migrate_seed_alpha_in_moving_reserve<T: Config>() -> Weight {
    let migration_name = b"migrate_seed_alpha_in_moving_reserve".to_vec();
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

    let mut seeded: u32 = 0;
    for (netuid, alpha_in) in SubnetAlphaIn::<T>::iter() {
        // Only seed a cold entry; never clobber a value that a block tick may
        // already have started warming.
        if SubnetAlphaInMovingReserve::<T>::get(netuid) == U64F64::saturating_from_num(0) {
            SubnetAlphaInMovingReserve::<T>::insert(
                netuid,
                U64F64::saturating_from_num(alpha_in.to_u64()),
            );
            seeded = seeded.saturating_add(1);
        }
        weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
    }

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed - seeded {} A_EMA entries.",
        String::from_utf8_lossy(&migration_name),
        seeded
    );

    weight
}
