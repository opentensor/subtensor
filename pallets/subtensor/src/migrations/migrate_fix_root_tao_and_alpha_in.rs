use super::migrate_init_total_issuance::migrate_init_total_issuance;
use super::*;
use alloc::string::String;

pub fn migrate_fix_root_tao_and_alpha_in<T: Config>() -> Weight {
    let migration_name = b"migrate_fix_root_tao_and_alpha_in".to_vec();
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

    // Update counters
    SubnetTAO::<T>::mutate(NetUid::ROOT, |amount| {
        *amount = amount.saturating_add(TaoCurrency::from(100_000_000_000));
    });
    SubnetAlphaIn::<T>::mutate(NetUid::ROOT, |amount| {
        *amount = amount.saturating_add(AlphaCurrency::from(100_000_000_000));
    });
    SubnetAlphaOut::<T>::mutate(NetUid::ROOT, |amount| {
        *amount = amount.saturating_add(AlphaCurrency::from(100_000_000_000));
    });
    SubnetVolume::<T>::mutate(NetUid::ROOT, |amount| {
        *amount = amount.saturating_add(100_000_000_000_u128);
    });
    TotalStake::<T>::mutate(|amount| {
        *amount = amount.saturating_add(TaoCurrency::from(100_000_000_000));
    });

    weight = weight.saturating_add(T::DbWeight::get().writes(5));

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        target: "runtime",
        "Migration '{}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    // We need to run the total issuance migration to update the total issuance
    // when the root subnet TAO has been updated.
    migrate_init_total_issuance::<T>().saturating_add(weight)
}
