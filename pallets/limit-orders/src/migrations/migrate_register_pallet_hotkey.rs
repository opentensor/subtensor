use alloc::string::String;
use frame_support::{BoundedVec, traits::Get, weights::Weight};

use crate::*;

fn migration_key() -> BoundedVec<u8, MigrationKeyMaxLen> {
    BoundedVec::truncate_from(b"migrate_register_pallet_hotkey".to_vec())
}

/// One-shot migration that disables the limit-orders pallet on first upgrade and
/// registers the pallet intermediary hotkey if it has not been registered yet.
///
/// Guarded by `HasMigrationRun` so it is safe to include in every runtime upgrade:
/// subsequent calls return immediately after a single storage read.
pub fn migrate_register_pallet_hotkey<T: Config>() -> Weight {
    let migration_name = migration_key();
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

    // Register the pallet intermediary hotkey if it has not been registered yet.
    let pallet_acct = Pallet::<T>::pallet_account();
    let pallet_hotkey = T::PalletHotkey::get();
    weight = weight.saturating_add(T::DbWeight::get().reads(1));

    if !T::SwapInterface::pallet_hotkey_registered(&pallet_acct, &pallet_hotkey) {
        let _ = T::SwapInterface::register_pallet_hotkey(&pallet_acct, &pallet_hotkey);
        // register_pallet_hotkey writes Owner, OwnedHotkeys, StakingHotkeys
        weight = weight.saturating_add(T::DbWeight::get().writes(3));
    }

    HasMigrationRun::<T>::insert(&migration_name, true);
    weight = weight.saturating_add(T::DbWeight::get().writes(1));

    log::info!(
        "Migration '{:?}' completed successfully.",
        String::from_utf8_lossy(&migration_name)
    );

    weight
}

#[cfg(test)]
mod tests {
    use frame_support::traits::{Get, Hooks};
    use sp_runtime::traits::AccountIdConversion;

    use super::*;
    use crate::tests::mock::{
        LimitOrdersPalletId, MockSwap, PalletHotkeyAccount, System, Test,
    };

    /// Minimal externalities: system genesis only, no pallet hotkey pre-registered,
    /// `LimitOrdersEnabled` at its storage default (`false`).
    fn migration_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    #[test]
    fn migration_registers_hotkey_and_marks_run_on_first_call() {
        migration_ext().execute_with(|| {
            let pallet_acct: crate::tests::mock::AccountId =
                LimitOrdersPalletId::get().into_account_truncating();
            let pallet_hotkey = PalletHotkeyAccount::get();

            assert!(!MockSwap::pallet_hotkey_registered(&pallet_acct, &pallet_hotkey));
            assert!(!HasMigrationRun::<Test>::get(migration_key()));

            migrate_register_pallet_hotkey::<Test>();

            assert!(
                MockSwap::pallet_hotkey_registered(&pallet_acct, &pallet_hotkey),
                "hotkey must be registered after migration"
            );
            assert!(
                HasMigrationRun::<Test>::get(migration_key()),
                "migration must be marked as run"
            );
            // Migration no longer touches LimitOrdersEnabled — value is unchanged.
            assert!(!LimitOrdersEnabled::<Test>::get());
        });
    }

    #[test]
    fn migration_does_not_touch_limit_orders_enabled() {
        migration_ext().execute_with(|| {
            // Enable the pallet before running the migration (simulates a chain
            // that already had it enabled via genesis or admin action).
            LimitOrdersEnabled::<Test>::set(true);

            migrate_register_pallet_hotkey::<Test>();

            assert!(
                LimitOrdersEnabled::<Test>::get(),
                "migration must not change LimitOrdersEnabled"
            );
        });
    }

    #[test]
    fn migration_skips_hotkey_registration_when_already_registered() {
        migration_ext().execute_with(|| {
            let pallet_acct: crate::tests::mock::AccountId =
                LimitOrdersPalletId::get().into_account_truncating();
            let pallet_hotkey = PalletHotkeyAccount::get();
            let _ = MockSwap::register_pallet_hotkey(&pallet_acct, &pallet_hotkey);

            // Must not panic on duplicate registration.
            migrate_register_pallet_hotkey::<Test>();

            assert!(HasMigrationRun::<Test>::get(migration_key()));
        });
    }

    #[test]
    fn migration_is_idempotent() {
        migration_ext().execute_with(|| {
            let pallet_acct: crate::tests::mock::AccountId =
                LimitOrdersPalletId::get().into_account_truncating();
            let pallet_hotkey = PalletHotkeyAccount::get();

            migrate_register_pallet_hotkey::<Test>();
            assert!(MockSwap::pallet_hotkey_registered(&pallet_acct, &pallet_hotkey));

            // Second run must be a no-op — hotkey stays registered, flag stays set.
            migrate_register_pallet_hotkey::<Test>();
            assert!(MockSwap::pallet_hotkey_registered(&pallet_acct, &pallet_hotkey));
            assert!(HasMigrationRun::<Test>::get(migration_key()));
        });
    }

    #[test]
    fn on_runtime_upgrade_delegates_to_migration() {
        migration_ext().execute_with(|| {
            assert!(!HasMigrationRun::<Test>::get(migration_key()));

            <pallet_limit_orders::Pallet<Test> as Hooks<u64>>::on_runtime_upgrade();

            assert!(HasMigrationRun::<Test>::get(migration_key()));
        });
    }
}
