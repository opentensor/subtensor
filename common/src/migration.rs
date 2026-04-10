//! Shared building blocks for runtime migrations across pallets.
//!
//! Provides:
//!  - [`MAINNET_GENESIS`]: the canonical Finney mainnet genesis block hash, used
//!    to gate mainnet-only migrations without hard-coding the literal in every
//!    migration file.
//!  - [`MigrationGuard`]: a tiny trait abstracting over a per-pallet
//!    "has-this-migration-run" storage map. Each pallet provides its own impl
//!    against its own storage, and the macro below stays storage-agnostic.
//!  - [`decl_migration!`]: a declarative macro that wraps the standard guard /
//!    log / mainnet-gating / mark-completed boilerplate so individual migration
//!    files only contain the actual business logic.

/// Genesis block hash of the Finney mainnet. Used by [`decl_migration!`] to
/// gate `mainnet_only` migrations without hard-coding the literal in each
/// migration file.
pub const MAINNET_GENESIS: [u8; 32] =
    hex_literal::hex!("2f0555cc76fc2840a25a6ea3b9637146806f1f44b090c175ffde2a7e5ab36c03");

/// Storage abstraction over a per-pallet "has-run" map.
///
/// Each pallet that wants to use [`decl_migration!`] should provide a unit type
/// implementing this trait against its own migration-tracking storage. The trait
/// deliberately operates on `&[u8]` so impls can use either `Vec<u8>` or
/// `BoundedVec<u8, _>` keys (different pallets in this workspace use both).
pub trait MigrationGuard {
    /// Returns `true` if the migration named `name` has already been recorded.
    fn has_run(name: &[u8]) -> bool;
    /// Records that the migration named `name` has run.
    fn mark_run(name: &[u8]);
}

/// Declares a runtime migration with a `HasMigrationRun`-style guard, optional
/// mainnet-only gating, logging and weight accounting.
///
/// Each invocation must specify a `guard = <Type>;` line — a type implementing
/// [`MigrationGuard`]. The body must return additional `Weight` for its own
/// operations; the macro adds the cost of the guard read and the final mark.
///
/// The migration is identified by `stringify!($name).as_bytes()`, so renaming
/// the function changes the migration key — the same convention used by
/// hand-written migrations in this workspace.
///
/// # Forms
///
/// ```ignore
/// use subtensor_runtime_common::decl_migration;
///
/// decl_migration! {
///     guard = MyPalletMigrationGuard<T>;
///     /// Always runs.
///     fn migrate_foo<T: Config>() -> Weight {
///         frame_support::pallet_prelude::Weight::zero()
///     }
/// }
///
/// decl_migration! {
///     guard = MyPalletMigrationGuard<T>;
///     /// Only runs on mainnet; on other chains the migration is still marked
///     /// completed (so it does not get retried) but its body is skipped.
///     mainnet_only fn migrate_bar<T: Config>() -> Weight {
///         frame_support::pallet_prelude::Weight::zero()
///     }
/// }
/// ```
#[macro_export]
macro_rules! decl_migration {
    (
        guard = $guard:ty;
        $(#[$meta:meta])*
        mainnet_only fn $name:ident <T: Config>() -> Weight $body:block
    ) => {
        $crate::__decl_migration_inner!(
            guard = $guard;
            mainnet_only = true;
            $(#[$meta])*
            fn $name() -> Weight $body
        );
    };

    (
        guard = $guard:ty;
        $(#[$meta:meta])*
        fn $name:ident <T: Config>() -> Weight $body:block
    ) => {
        $crate::__decl_migration_inner!(
            guard = $guard;
            mainnet_only = false;
            $(#[$meta])*
            fn $name() -> Weight $body
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __decl_migration_inner {
    (
        guard = $guard:ty;
        mainnet_only = $mainnet:literal;
        $(#[$meta:meta])*
        fn $name:ident () -> Weight $body:block
    ) => {
        $(#[$meta])*
        pub fn $name<T: Config>() -> ::frame_support::pallet_prelude::Weight {
            use ::frame_support::pallet_prelude::Weight;
            use ::frame_support::traits::Get;
            use $crate::migration::MigrationGuard as _;

            const MIGRATION_NAME: &[u8] = stringify!($name).as_bytes();

            let mut weight = T::DbWeight::get().reads(1);
            if <$guard as $crate::migration::MigrationGuard>::has_run(MIGRATION_NAME) {
                ::log::info!(
                    "Migration '{}' already run, skipping",
                    stringify!($name)
                );
                return weight;
            }
            ::log::info!("Running migration '{}'", stringify!($name));

            if $mainnet {
                weight = weight.saturating_add(T::DbWeight::get().reads(1));
                let on_mainnet = {
                    use ::sp_runtime::traits::Zero;
                    let h = ::frame_system::Pallet::<T>::block_hash(
                        <::frame_system::pallet_prelude::BlockNumberFor<T>>::zero()
                    );
                    h.as_ref() == $crate::migration::MAINNET_GENESIS
                };
                if !on_mainnet {
                    ::log::info!(
                        "Migration '{}' is mainnet-only, marking complete and skipping body",
                        stringify!($name)
                    );
                    <$guard as $crate::migration::MigrationGuard>::mark_run(MIGRATION_NAME);
                    weight = weight.saturating_add(T::DbWeight::get().writes(1));
                    return weight;
                }
            }

            // Body returns its *additional* weight.
            let body_weight: Weight = (|| -> Weight { $body })();
            weight = weight.saturating_add(body_weight);

            <$guard as $crate::migration::MigrationGuard>::mark_run(MIGRATION_NAME);
            weight = weight.saturating_add(T::DbWeight::get().writes(1));
            ::log::info!("Migration '{}' completed", stringify!($name));
            weight
        }
    };
}
