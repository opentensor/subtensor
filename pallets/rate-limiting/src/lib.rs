#![cfg_attr(not(feature = "std"), no_std)]

//! Rate limiting for runtime calls with optional contextual restrictions.
//!
//! # Overview
//!
//! `pallet-rate-limiting` lets a runtime restrict how frequently particular calls can execute.
//! Limits are stored on-chain, keyed by the call's pallet/variant pair. Each entry can specify an
//! exact block span or defer to a configured default. The pallet exposes three roots-only
//! extrinsics to manage this data:
//!
//! - [`set_rate_limit`](pallet::Pallet::set_rate_limit): assign a limit to an extrinsic, either as
//!   `RateLimit::Exact(blocks)` or `RateLimit::Default`.
//! - [`clear_rate_limit`](pallet::Pallet::clear_rate_limit): remove a stored limit.
//! - [`set_default_rate_limit`](pallet::Pallet::set_default_rate_limit): set the global default
//!   block span used by `RateLimit::Default` entries.
//!
//! The pallet also tracks the last block in which a rate-limited call was executed, per optional
//! *context*. Context allows one limit definition (for example, “set weights”) to be enforced per
//! subnet, account, or other grouping chosen by the runtime. The storage layout is:
//!
//! - [`Limits`](pallet::Limits): `TransactionIdentifier → RateLimit<BlockNumber>`
//! - [`DefaultLimit`](pallet::DefaultLimit): `BlockNumber`
//! - [`LastSeen`](pallet::LastSeen): `(TransactionIdentifier, Option<Context>) → BlockNumber`
//!
//! # Transaction extension
//!
//! Enforcement happens via [`RateLimitTransactionExtension`], which implements
//! `sp_runtime::traits::TransactionExtension`. The extension consults `Limits`, fetches the current
//! block, and decides whether the call is eligible. If successful, it returns metadata that causes
//! [`LastSeen`](pallet::LastSeen) to update after dispatch. A rejected call yields
//! `InvalidTransaction::Custom(1)`.
//!
//! To enable the extension, add it to your runtime's transaction extension tuple. For example:
//!
//! ```rust
//! pub type TransactionExtensions = (
//!     // ... other extensions ...
//!     pallet_rate_limiting::RateLimitTransactionExtension<Runtime>,
//! );
//! ```
//!
//! # Context resolver
//!
//! The extension needs to know when two invocations should share a rate limit. This is controlled
//! by implementing [`RateLimitContextResolver`] for the runtime call type (or for a helper that the
//! runtime wires into [`Config::ContextResolver`]). The resolver receives the call and returns
//! `Some(context)` if the rate should be scoped (e.g. by `netuid`), or `None` for a global limit.
//!
//! ```rust
//! pub struct WeightsContextResolver;
//!
//! impl pallet_rate_limiting::RateLimitContextResolver<RuntimeCall, NetUid>
//!     for WeightsContextResolver
//! {
//!     fn context(call: &RuntimeCall) -> Option<NetUid> {
//!         match call {
//!             RuntimeCall::Subtensor(pallet_subtensor::Call::set_weights { netuid, .. }) => {
//!                 Some(*netuid)
//!             }
//!             _ => None,
//!         }
//!     }
//! }
//!
//! impl pallet_rate_limiting::Config for Runtime {
//!     type RuntimeCall = RuntimeCall;
//!     type LimitContext = NetUid;
//!     type ContextResolver = WeightsContextResolver;
//! }
//! ```

pub use pallet::*;
pub use tx_extension::RateLimitTransactionExtension;
pub use types::{RateLimit, RateLimitContextResolver, TransactionIdentifier};

mod tx_extension;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use codec::Codec;
    use frame_support::{
        pallet_prelude::*,
        sp_runtime::traits::{Saturating, Zero},
        traits::{BuildGenesisConfig, GetCallMetadata},
    };
    use frame_system::{ensure_root, pallet_prelude::*};
    use sp_std::{convert::TryFrom, vec::Vec};

    use crate::types::{RateLimit, RateLimitContextResolver, TransactionIdentifier};

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
    pub type Limits<T> = StorageMap<
        _,
        Blake2_128Concat,
        TransactionIdentifier,
        RateLimit<BlockNumberFor<T>>,
        OptionQuery,
    >;

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

    /// Default block span applied when an extrinsic uses the default rate limit.
    #[pallet::storage]
    #[pallet::getter(fn default_limit)]
    pub type DefaultLimit<T> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Events emitted by the rate limiting pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A rate limit was set or updated.
        RateLimitSet {
            /// Identifier of the affected transaction.
            transaction: TransactionIdentifier,
            /// The new limit configuration applied to the transaction.
            limit: RateLimit<BlockNumberFor<T>>,
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
        /// The default rate limit was set or updated.
        DefaultRateLimitSet {
            /// The new default limit expressed in blocks.
            block_span: BlockNumberFor<T>,
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

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub default_limit: BlockNumberFor<T>,
        pub limits: Vec<(TransactionIdentifier, RateLimit<BlockNumberFor<T>>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                default_limit: Zero::zero(),
                limits: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            DefaultLimit::<T>::put(self.default_limit);

            for (identifier, limit) in &self.limits {
                Limits::<T>::insert(identifier, limit.clone());
            }
        }
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    impl<T: Config> Pallet<T> {
        /// Returns `true` when the given transaction identifier passes its configured rate limit
        /// within the provided context.
        pub fn is_within_limit(
            identifier: &TransactionIdentifier,
            context: &Option<<T as Config>::LimitContext>,
        ) -> Result<bool, DispatchError> {
            let Some(block_span) = Self::resolved_limit(identifier) else {
                return Ok(true);
            };

            let current = frame_system::Pallet::<T>::block_number();

            if let Some(last) = LastSeen::<T>::get(identifier, context) {
                let delta = current.saturating_sub(last);
                if delta < block_span {
                    return Ok(false);
                }
            }

            Ok(true)
        }

        fn resolved_limit(identifier: &TransactionIdentifier) -> Option<BlockNumberFor<T>> {
            let limit = Limits::<T>::get(identifier)?;
            Some(match limit {
                RateLimit::Default => DefaultLimit::<T>::get(),
                RateLimit::Exact(block_span) => block_span,
            })
        }

        /// Returns the configured limit for the specified pallet/extrinsic names, if any.
        pub fn limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
        ) -> Option<RateLimit<BlockNumberFor<T>>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            Limits::<T>::get(&identifier)
        }

        /// Returns the resolved block span for the specified pallet/extrinsic names, if any.
        pub fn resolved_limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
        ) -> Option<BlockNumberFor<T>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            Self::resolved_limit(&identifier)
        }

        fn identifier_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
        ) -> Option<TransactionIdentifier> {
            let modules = <T as Config>::RuntimeCall::get_module_names();
            let pallet_pos = modules.iter().position(|name| *name == pallet_name)?;
            let call_names = <T as Config>::RuntimeCall::get_call_names(pallet_name);
            let extrinsic_pos = call_names.iter().position(|name| *name == extrinsic_name)?;
            let pallet_index = u8::try_from(pallet_pos).ok()?;
            let extrinsic_index = u8::try_from(extrinsic_pos).ok()?;
            Some(TransactionIdentifier::new(pallet_index, extrinsic_index))
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
            limit: RateLimit<BlockNumberFor<T>>,
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

        /// Sets the default rate limit in blocks applied to calls configured to use it.
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn set_default_rate_limit(
            origin: OriginFor<T>,
            block_span: BlockNumberFor<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            DefaultLimit::<T>::put(block_span);

            Self::deposit_event(Event::DefaultRateLimitSet { block_span });

            Ok(())
        }
    }
}
