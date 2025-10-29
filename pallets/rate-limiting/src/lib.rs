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
//!   `RateLimit::Exact(blocks)` or `RateLimit::Default`. The optional context parameter lets you
//!   scope the configuration to a particular subnet/key/account while keeping a global fallback.
//! - [`clear_rate_limit`](pallet::Pallet::clear_rate_limit): remove a stored limit for the provided
//!   context (or for the global entry when `None` is supplied).
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
//! Each storage map is namespaced by pallet instance; runtimes can deploy multiple independent
//! instances to manage distinct rate-limiting scopes.
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
//! ```ignore
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
//! `Some(context)` if the rate should be scoped (e.g. by `netuid`), or `None` to use the global
//! entry. The resolver is only used when *tracking* executions; you still configure limits via the
//! explicit `context` argument on `set_rate_limit`/`clear_rate_limit`.
//!
//! ```ignore
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

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::BenchmarkHelper;
pub use pallet::*;
pub use tx_extension::RateLimitTransactionExtension;
pub use types::{RateLimit, RateLimitContextResolver, TransactionIdentifier};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
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
    use sp_std::{convert::TryFrom, marker::PhantomData, vec::Vec};

    #[cfg(feature = "runtime-benchmarks")]
    use crate::benchmarking::BenchmarkHelper as BenchmarkHelperTrait;
    use crate::types::{RateLimit, RateLimitContextResolver, TransactionIdentifier};

    /// Configuration trait for the rate limiting pallet.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The overarching runtime call type.
        type RuntimeCall: Parameter
            + Codec
            + GetCallMetadata
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// Context type used for contextual (per-group) rate limits.
        type LimitContext: Parameter + Clone + PartialEq + Eq + MaybeSerializeDeserialize;

        /// Resolves the context for a given runtime call.
        type ContextResolver: RateLimitContextResolver<<Self as Config<I>>::RuntimeCall, Self::LimitContext>;

        /// Helper used to construct runtime calls for benchmarking.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelperTrait<<Self as Config<I>>::RuntimeCall>;
    }

    /// Storage mapping from transaction identifier and optional context to its configured rate limit.
    #[pallet::storage]
    #[pallet::getter(fn limits)]
    pub type Limits<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        TransactionIdentifier,
        Blake2_128Concat,
        Option<<T as Config<I>>::LimitContext>,
        RateLimit<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Tracks when a transaction was last observed.
    ///
    /// The second key is `None` for global limits and `Some(context)` for contextual limits.
    #[pallet::storage]
    pub type LastSeen<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        TransactionIdentifier,
        Blake2_128Concat,
        Option<<T as Config<I>>::LimitContext>,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// Default block span applied when an extrinsic uses the default rate limit.
    #[pallet::storage]
    #[pallet::getter(fn default_limit)]
    pub type DefaultLimit<T: Config<I>, I: 'static = ()> =
        StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Events emitted by the rate limiting pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// A rate limit was set or updated.
        RateLimitSet {
            /// Identifier of the affected transaction.
            transaction: TransactionIdentifier,
            /// Context to which the limit applies, if any.
            context: Option<<T as Config<I>>::LimitContext>,
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
            /// Context from which the limit was cleared, if any.
            context: Option<<T as Config<I>>::LimitContext>,
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
    pub enum Error<T, I = ()> {
        /// Failed to extract the pallet and extrinsic indices from the call.
        InvalidRuntimeCall,
        /// Attempted to remove a limit that is not present.
        MissingRateLimit,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub default_limit: BlockNumberFor<T>,
        pub limits: Vec<(
            TransactionIdentifier,
            Option<<T as Config<I>>::LimitContext>,
            RateLimit<BlockNumberFor<T>>,
        )>,
    }

    #[cfg(feature = "std")]
    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                default_limit: Zero::zero(),
                limits: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
        fn build(&self) {
            DefaultLimit::<T, I>::put(self.default_limit);

            for (identifier, context, limit) in &self.limits {
                Limits::<T, I>::insert(identifier, context.clone(), limit.clone());
            }
        }
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Returns `true` when the given transaction identifier passes its configured rate limit
        /// within the provided context.
        pub fn is_within_limit(
            identifier: &TransactionIdentifier,
            context: &Option<<T as Config<I>>::LimitContext>,
        ) -> Result<bool, DispatchError> {
            let Some(block_span) = Self::resolved_limit(identifier, context) else {
                return Ok(true);
            };

            let current = frame_system::Pallet::<T>::block_number();

            if let Some(last) = LastSeen::<T, I>::get(identifier, context) {
                let delta = current.saturating_sub(last);
                if delta < block_span {
                    return Ok(false);
                }
            }

            Ok(true)
        }

        pub(crate) fn resolved_limit(
            identifier: &TransactionIdentifier,
            context: &Option<<T as Config<I>>::LimitContext>,
        ) -> Option<BlockNumberFor<T>> {
            let lookup = Limits::<T, I>::get(identifier, context).or_else(|| {
                Limits::<T, I>::get(identifier, None::<<T as Config<I>>::LimitContext>)
            });
            let limit = lookup?;
            Some(match limit {
                RateLimit::Default => DefaultLimit::<T, I>::get(),
                RateLimit::Exact(block_span) => block_span,
            })
        }

        /// Returns the configured limit for the specified pallet/extrinsic names, if any.
        pub fn limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
            context: Option<<T as Config<I>>::LimitContext>,
        ) -> Option<RateLimit<BlockNumberFor<T>>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            Limits::<T, I>::get(&identifier, context.clone()).or_else(|| {
                Limits::<T, I>::get(&identifier, None::<<T as Config<I>>::LimitContext>)
            })
        }

        /// Returns the resolved block span for the specified pallet/extrinsic names, if any.
        pub fn resolved_limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
            context: Option<<T as Config<I>>::LimitContext>,
        ) -> Option<BlockNumberFor<T>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            Self::resolved_limit(&identifier, &context)
        }

        fn identifier_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
        ) -> Option<TransactionIdentifier> {
            let modules = <T as Config<I>>::RuntimeCall::get_module_names();
            let pallet_pos = modules.iter().position(|name| *name == pallet_name)?;
            let call_names = <T as Config<I>>::RuntimeCall::get_call_names(pallet_name);
            let extrinsic_pos = call_names.iter().position(|name| *name == extrinsic_name)?;
            let pallet_index = u8::try_from(pallet_pos).ok()?;
            let extrinsic_index = u8::try_from(extrinsic_pos).ok()?;
            Some(TransactionIdentifier::new(pallet_index, extrinsic_index))
        }
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Sets the rate limit configuration for the given call and optional context.
        ///
        /// The supplied `call` is only used to derive the pallet and extrinsic indices; **any
        /// arguments embedded in the call are ignored**. The `context` parameter determines which
        /// scoped entry is updated (for example a subnet identifier). Passing `None` updates the
        /// global entry, which acts as a fallback when no context-specific limit exists.
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_rate_limit(
            origin: OriginFor<T>,
            call: Box<<T as Config<I>>::RuntimeCall>,
            limit: RateLimit<BlockNumberFor<T>>,
            context: Option<<T as Config<I>>::LimitContext>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let identifier = TransactionIdentifier::from_call::<T, I>(call.as_ref())?;
            Limits::<T, I>::insert(&identifier, context.clone(), limit.clone());

            let (pallet_name, extrinsic_name) = identifier.names::<T, I>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            Self::deposit_event(Event::RateLimitSet {
                transaction: identifier,
                context,
                limit,
                pallet,
                extrinsic,
            });

            Ok(())
        }

        /// Clears the rate limit for the given call, if present.
        ///
        /// The supplied `call` is only used to derive the pallet and extrinsic indices; **any
        /// arguments embedded in the call are ignored**. The `context` parameter must match the
        /// entry that should be removed (use `None` to remove the global configuration).
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn clear_rate_limit(
            origin: OriginFor<T>,
            call: Box<<T as Config<I>>::RuntimeCall>,
            context: Option<<T as Config<I>>::LimitContext>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let identifier = TransactionIdentifier::from_call::<T, I>(call.as_ref())?;

            let (pallet_name, extrinsic_name) = identifier.names::<T, I>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            ensure!(
                Limits::<T, I>::take(&identifier, context.clone()).is_some(),
                Error::<T, I>::MissingRateLimit
            );

            Self::deposit_event(Event::RateLimitCleared {
                transaction: identifier,
                context,
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

            DefaultLimit::<T, I>::put(block_span);

            Self::deposit_event(Event::DefaultRateLimitSet { block_span });

            Ok(())
        }
    }
}
