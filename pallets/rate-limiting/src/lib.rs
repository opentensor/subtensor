#![cfg_attr(not(feature = "std"), no_std)]

//! Basic rate limiting pallet.

pub use pallet::*;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::DispatchError, traits::GetCallMetadata};
use scale_info::TypeInfo;
use sp_std::fmt;

#[frame_support::pallet]
pub mod pallet {
    use crate::TransactionIdentifier;
    use codec::Codec;
    use frame_support::{
        pallet_prelude::*, sp_runtime::traits::Saturating, traits::GetCallMetadata,
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    /// Configuration trait for the rate limiting pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime call type.
        type RuntimeCall: Parameter
            + Codec
            + GetCallMetadata
            + IsType<<Self as frame_system::Config>::RuntimeCall>;
    }

    /// Storage mapping from transaction identifier to its block-based rate limit.
    #[pallet::storage]
    #[pallet::getter(fn limits)]
    pub type Limits<T> =
        StorageMap<_, Blake2_128Concat, TransactionIdentifier, BlockNumberFor<T>, OptionQuery>;

    /// Tracks when a transaction was last observed.
    #[pallet::storage]
    pub type LastSeen<T> =
        StorageMap<_, Blake2_128Concat, TransactionIdentifier, BlockNumberFor<T>, OptionQuery>;

    /// Events emitted by the rate limiting pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A rate limit was set or updated.
        RateLimitSet {
            /// Identifier of the affected transaction.
            transaction: TransactionIdentifier,
            /// The new limit expressed in blocks.
            limit: BlockNumberFor<T>,
            /// Pallet name associated with the transaction.
            pallet: Vec<u8>,
            /// Extrinsic name associated with the transaction.
            extrinsic: Vec<u8>,
        },
        /// A rate limit was cleared.
        RateLimitCleared {
            /// Identifier of the affected transaction.
            transaction: TransactionIdentifier,
            /// Pallet name associated with the transaction.
            pallet: Vec<u8>,
            /// Extrinsic name associated with the transaction.
            extrinsic: Vec<u8>,
        },
    }

    /// Errors that can occur while configuring rate limits.
    #[pallet::error]
    pub enum Error<T> {
        /// Failed to extract the pallet and extrinsic indices from the call.
        InvalidRuntimeCall,
        /// Attempted to remove a limit that is not present.
        MissingRateLimit,
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    impl<T: Config> Pallet<T> {
        /// Returns `true` when the given transaction identifier passes its configured rate limit.
        pub fn is_within_limit(identifier: &TransactionIdentifier) -> Result<bool, DispatchError> {
            let Some(limit) = Limits::<T>::get(identifier) else {
                return Ok(true);
            };

            let current = frame_system::Pallet::<T>::block_number();
            if let Some(last) = LastSeen::<T>::get(identifier) {
                let delta = current.saturating_sub(last);
                if delta < limit {
                    return Ok(false);
                }
            }

            Ok(true)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sets the rate limit, in blocks, for the given call.
        ///
        /// The supplied `call` is only used to derive the pallet and extrinsic indices; **any
        /// arguments embedded in the call are ignored**.
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_rate_limit(
            origin: OriginFor<T>,
            call: Box<<T as Config>::RuntimeCall>,
            limit: BlockNumberFor<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let identifier = TransactionIdentifier::from_call::<T>(call.as_ref())?;
            Limits::<T>::insert(&identifier, limit);

            let (pallet_name, extrinsic_name) = identifier.names::<T>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            Self::deposit_event(Event::RateLimitSet {
                transaction: identifier,
                limit,
                pallet,
                extrinsic,
            });

            Ok(())
        }

        /// Clears the rate limit for the given call, if present.
        ///
        /// The supplied `call` is only used to derive the pallet and extrinsic indices; **any
        /// arguments embedded in the call are ignored**.
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn clear_rate_limit(
            origin: OriginFor<T>,
            call: Box<<T as Config>::RuntimeCall>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let identifier = TransactionIdentifier::from_call::<T>(call.as_ref())?;

            let (pallet_name, extrinsic_name) = identifier.names::<T>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            ensure!(
                Limits::<T>::take(&identifier).is_some(),
                Error::<T>::MissingRateLimit
            );

            Self::deposit_event(Event::RateLimitCleared {
                transaction: identifier,
                pallet,
                extrinsic,
            });

            Ok(())
        }
    }
}

/// Identifies a runtime call by pallet and extrinsic indices.
#[derive(
    Clone, Copy, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct TransactionIdentifier {
    /// Index of the pallet containing the extrinsic.
    pub pallet_index: u8,
    /// Variant index of the extrinsic within the pallet.
    pub extrinsic_index: u8,
}

impl TransactionIdentifier {
    /// Builds a new identifier from pallet/extrinsic indices.
    const fn new(pallet_index: u8, extrinsic_index: u8) -> Self {
        Self {
            pallet_index,
            extrinsic_index,
        }
    }

    /// Returns the pallet and extrinsic name associated with this identifier.
    fn names<T>(&self) -> Result<(&'static str, &'static str), DispatchError>
    where
        T: Config,
    {
        let modules = <T as Config>::RuntimeCall::get_module_names();
        let pallet_name = modules
            .get(self.pallet_index as usize)
            .copied()
            .ok_or(Error::<T>::InvalidRuntimeCall)?;
        let call_names = <T as Config>::RuntimeCall::get_call_names(pallet_name);
        let extrinsic_name = call_names
            .get(self.extrinsic_index as usize)
            .copied()
            .ok_or(Error::<T>::InvalidRuntimeCall)?;
        Ok((pallet_name, extrinsic_name))
    }

    /// Builds an identifier from a runtime call by extracting its pallet/extrinsic indices.
    fn from_call<T>(call: &<T as Config>::RuntimeCall) -> Result<Self, DispatchError>
    where
        T: Config,
    {
        call.using_encoded(|encoded| {
            let pallet_index = *encoded.get(0).ok_or(Error::<T>::InvalidRuntimeCall)?;
            let extrinsic_index = *encoded.get(1).ok_or(Error::<T>::InvalidRuntimeCall)?;
            Ok(TransactionIdentifier::new(pallet_index, extrinsic_index))
        })
    }
}

impl fmt::Debug for TransactionIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransactionIdentifier")
            .field("pallet_index", &self.pallet_index)
            .field("extrinsic_index", &self.extrinsic_index)
            .finish()
    }
}
