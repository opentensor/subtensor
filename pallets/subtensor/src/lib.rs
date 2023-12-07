#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]
// Edit this file to define custom logic or remove it if it is not needed.
// Learn more about FRAME and the core library of Substrate FRAME pallets:
// <https://docs.substrate.io/reference/frame-pallets/>

use frame_system::{self as system, ensure_signed};

use frame_support::{
    dispatch,
    ensure,
    traits::{tokens::WithdrawReasons, Currency, ExistenceRequirement, IsSubType},
};

use codec::{Decode, Encode};
use frame_support::sp_runtime::transaction_validity::ValidTransaction;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
    transaction_validity::{TransactionValidity, TransactionValidityError},
};
use sp_std::marker::PhantomData;

// ============================
//	==== Benchmark Imports =====
// ============================
#[cfg(feature = "runtime-benchmarks")]
//mod benchmarks;


extern crate alloc;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::GetDispatchInfo,
        pallet_prelude::{DispatchResult, StorageMap, ValueQuery, *},
        traits::{Currency, UnfilteredDispatchable},
    };

    use frame_system::pallet_prelude::*;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
    }
}