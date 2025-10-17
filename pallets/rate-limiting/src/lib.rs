#![cfg_attr(not(feature = "std"), no_std)]

//! Basic rate limiting pallet.

pub use pallet::*;
pub use tx_extension::RateLimitTransactionExtension;
pub use types::{RateLimitContextResolver, TransactionIdentifier};

mod tx_extension;
mod types;

#[frame_support::pallet]
pub mod pallet {
    use codec::Codec;
    use frame_support::{
        pallet_prelude::*, sp_runtime::traits::Saturating, traits::GetCallMetadata,
    };
    use frame_system::{ensure_root, pallet_prelude::*};
    use sp_std::vec::Vec;

    use crate::types::{RateLimitContextResolver, TransactionIdentifier};

    /// Configuration trait for the rate limiting pallet.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime call type.
        type RuntimeCall: Parameter
            + Codec
            + GetCallMetadata
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// Context type used for contextual (per-group) rate limits.
        type LimitContext: Parameter + Clone + PartialEq + Eq;

        /// Resolves the context for a given runtime call.
        type ContextResolver: RateLimitContextResolver<<Self as Config>::RuntimeCall, Self::LimitContext>;
    }

    /// Storage mapping from transaction identifier to its configured rate limit.
    #[pallet::storage]
    #[pallet::getter(fn limits)]
    pub type Limits<T> =
        StorageMap<_, Blake2_128Concat, TransactionIdentifier, BlockNumberFor<T>, OptionQuery>;

    /// Tracks when a transaction was last observed.
    ///
    /// The second key is `None` for global limits and `Some(context)` for contextual limits.
    #[pallet::storage]
    pub type LastSeen<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        TransactionIdentifier,
        Blake2_128Concat,
        Option<<T as Config>::LimitContext>,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// Events emitted by the rate limiting pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A rate limit was set or updated.
        RateLimitSet {
            /// Identifier of the affected transaction.
            transaction: TransactionIdentifier,
            /// The new limit expressed in blocks.
            block_span: BlockNumberFor<T>,
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
        /// Returns `true` when the given transaction identifier passes its configured rate limit
        /// within the provided context.
        pub fn is_within_limit(
            identifier: &TransactionIdentifier,
            context: Option<<T as Config>::LimitContext>,
        ) -> Result<bool, DispatchError> {
            let Some(block_span) = Limits::<T>::get(identifier) else {
                return Ok(true);
            };

            let current = frame_system::Pallet::<T>::block_number();

            if let Some(last) = LastSeen::<T>::get(identifier, &context) {
                let delta = current.saturating_sub(last);
                if delta < block_span {
                    return Ok(false);
                }
            }

            Ok(true)
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Sets the rate limit configuration for the given call.
        ///
        /// The supplied `call` is only used to derive the pallet and extrinsic indices; **any
        /// arguments embedded in the call are ignored**.
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_rate_limit(
            origin: OriginFor<T>,
            call: Box<<T as Config>::RuntimeCall>,
            block_span: BlockNumberFor<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let identifier = TransactionIdentifier::from_call::<T>(call.as_ref())?;
            Limits::<T>::insert(&identifier, block_span);

            let (pallet_name, extrinsic_name) = identifier.names::<T>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            Self::deposit_event(Event::RateLimitSet {
                transaction: identifier,
                block_span,
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
