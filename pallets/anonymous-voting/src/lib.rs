#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode};
use core::marker::PhantomData;
use frame_support::{
    dispatch::{ClassifyDispatch, DispatchClass, DispatchResult, Pays, PaysFee, WeighData},
    pallet_prelude::TransactionSource,
    pallet_prelude::*,
    traits::IsSubType,
    weights::Weight,
};
use frame_system::pallet_prelude::*;
use log::info;
use scale_info::TypeInfo;
use sp_runtime::{
    impl_tx_ext_default,
    traits::{
        Bounded, DispatchInfoOf, DispatchOriginOf, SaturatedConversion, Saturating,
        TransactionExtension, ValidateResult,
    },
    transaction_validity::{InvalidTransaction, ValidTransaction},
};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent;
    }

    #[pallet::storage]
    pub(super) type Members<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::CollectiveId,
        BoundedVec<T::AccountId, T::MaxMembers>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn anonymous_vote(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn remove_anonymous_vote(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
    }
}
