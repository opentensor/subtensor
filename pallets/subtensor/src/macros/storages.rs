#![allow(clippy::crate_in_macro_def)]

use frame_support::pallet_macros::pallet_section;

/// A [`pallet_section`] that defines the errors for a pallet.
/// This can later be imported into the pallet using [`import_section`].
#[pallet_section]
mod storages {
    #[pallet::storage] // --- ITEM ( total_issuance )
    pub type TotalIssuance2<T> = StorageValue<_, u64, ValueQuery, DefaultTotalIssuance<T>>;
}
