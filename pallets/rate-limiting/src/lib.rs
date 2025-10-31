#![cfg_attr(not(feature = "std"), no_std)]

//! Rate limiting for runtime calls with optional contextual restrictions.
//!
//! # Overview
//!
//! `pallet-rate-limiting` lets a runtime restrict how frequently particular calls can execute.
//! Limits are stored on-chain, keyed by the call's pallet/variant pair. Each entry can specify an
//! exact block span or defer to a configured default. The pallet exposes three extrinsics,
//! restricted by [`Config::AdminOrigin`], to manage this data:
//!
//! - [`set_rate_limit`](pallet::Pallet::set_rate_limit): assign a limit to an extrinsic by
//!   supplying a [`RateLimitKind`] span. The pallet infers the *limit scope* (for example a
//!   `netuid`) using [`Config::LimitScopeResolver`] and stores the configuration for that scope, or
//!   globally when no scope is resolved.
//! - [`clear_rate_limit`](pallet::Pallet::clear_rate_limit): remove a stored limit for the scope
//!   derived from the provided call (or the global entry when no scope resolves).
//! - [`set_default_rate_limit`](pallet::Pallet::set_default_rate_limit): set the global default
//!   block span used by `RateLimitKind::Default` entries.
//!
//! The pallet also tracks the last block in which a rate-limited call was executed, per optional
//! *usage key*. A usage key may refine tracking beyond the limit scope (for example combining a
//! `netuid` with a hyperparameter name), so the two concepts are explicitly separated in the
//! configuration.
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
//! # Context resolvers
//!
//! The pallet relies on two resolvers:
//!
//! - [`Config::LimitScopeResolver`], which determines how limits are stored (for example by
//!   returning a `netuid`). The resolver can also signal that a call should bypass rate limiting or
//!   adjust the effective span at validation time. When it returns `None`, the configuration is
//!   stored as a global fallback.
//! - [`Config::UsageResolver`], which decides how executions are tracked in
//!   [`LastSeen`](pallet::LastSeen). This can refine the limit scope (for example by returning a
//!   tuple of `(netuid, hyperparameter)`).
//!
//! Each resolver receives the call and may return `Some(identifier)` when scoping is required, or
//! `None` to use the global entry. Extrinsics such as
//! [`set_rate_limit`](pallet::Pallet::set_rate_limit) automatically consult these resolvers.
//!
//! ```ignore
//! pub struct WeightsContextResolver;
//!
//! // Limits are scoped per netuid.
//! pub struct ScopeResolver;
//! impl pallet_rate_limiting::RateLimitScopeResolver<RuntimeCall, NetUid, BlockNumber> for ScopeResolver {
//!     fn context(call: &RuntimeCall) -> Option<NetUid> {
//!         match call {
//!             RuntimeCall::Subtensor(pallet_subtensor::Call::set_weights { netuid, .. }) => {
//!                 Some(*netuid)
//!             }
//!             _ => None,
//!         }
//!     }
//!
//!     fn adjust_span(_call: &RuntimeCall, span: BlockNumber) -> BlockNumber {
//!         span
//!     }
//! }
//!
//! // Usage tracking distinguishes hyperparameter + netuid.
//! pub struct UsageResolver;
//! impl pallet_rate_limiting::RateLimitUsageResolver<RuntimeCall, (NetUid, HyperParam)> for UsageResolver {
//!     fn context(call: &RuntimeCall) -> Option<(NetUid, HyperParam)> {
//!         match call {
//!             RuntimeCall::Subtensor(pallet_subtensor::Call::set_hyperparam {
//!                 netuid,
//!                 hyper,
//!                 ..
//!             }) => Some((*netuid, *hyper)),
//!             _ => None,
//!         }
//!     }
//! }
//!
//! impl pallet_rate_limiting::Config for Runtime {
//!     type RuntimeCall = RuntimeCall;
//!     type LimitScope = NetUid;
//!     type LimitScopeResolver = ScopeResolver;
//!     type UsageKey = (NetUid, HyperParam);
//!     type UsageResolver = UsageResolver;
//!     type AdminOrigin = frame_system::EnsureRoot<Self::AccountId>;
//! }
//! ```

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::BenchmarkHelper;
pub use pallet::*;
pub use tx_extension::RateLimitTransactionExtension;
pub use types::{
    RateLimit, RateLimitKind, RateLimitScopeResolver, RateLimitUsageResolver, TransactionIdentifier,
};

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
        traits::{BuildGenesisConfig, EnsureOrigin, GetCallMetadata},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::{convert::TryFrom, marker::PhantomData, vec::Vec};

    #[cfg(feature = "runtime-benchmarks")]
    use crate::benchmarking::BenchmarkHelper as BenchmarkHelperTrait;
    use crate::types::{
        RateLimit, RateLimitKind, RateLimitScopeResolver, RateLimitUsageResolver,
        TransactionIdentifier,
    };

    /// Configuration trait for the rate limiting pallet.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config
    where
        BlockNumberFor<Self>: MaybeSerializeDeserialize,
    {
        /// The overarching runtime call type.
        type RuntimeCall: Parameter
            + Codec
            + GetCallMetadata
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// Origin permitted to configure rate limits.
        type AdminOrigin: EnsureOrigin<OriginFor<Self>>;

        /// Scope identifier used to namespace stored rate limits.
        type LimitScope: Parameter + Clone + PartialEq + Eq + Ord + MaybeSerializeDeserialize;

        /// Resolves the scope for the given runtime call when configuring limits.
        type LimitScopeResolver: RateLimitScopeResolver<
                <Self as Config<I>>::RuntimeCall,
                Self::LimitScope,
                BlockNumberFor<Self>,
            >;

        /// Usage key tracked in [`LastSeen`] for rate-limited calls.
        type UsageKey: Parameter + Clone + PartialEq + Eq + Ord + MaybeSerializeDeserialize;

        /// Resolves the usage key for the given runtime call when enforcing limits.
        type UsageResolver: RateLimitUsageResolver<<Self as Config<I>>::RuntimeCall, Self::UsageKey>;

        /// Helper used to construct runtime calls for benchmarking.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelperTrait<<Self as Config<I>>::RuntimeCall>;
    }

    /// Storage mapping from transaction identifier to its configured rate limit.
    #[pallet::storage]
    #[pallet::getter(fn limits)]
    pub type Limits<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        TransactionIdentifier,
        RateLimit<<T as Config<I>>::LimitScope, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Tracks when a transaction was last observed.
    ///
    /// The second key is `None` for global tracking and `Some(key)` for scoped usage tracking.
    #[pallet::storage]
    pub type LastSeen<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        TransactionIdentifier,
        Blake2_128Concat,
        Option<<T as Config<I>>::UsageKey>,
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
            /// Limit scope to which the configuration applies, if any.
            scope: Option<<T as Config<I>>::LimitScope>,
            /// The rate limit policy applied to the transaction.
            limit: RateLimitKind<BlockNumberFor<T>>,
            /// Pallet name associated with the transaction.
            pallet: Vec<u8>,
            /// Extrinsic name associated with the transaction.
            extrinsic: Vec<u8>,
        },
        /// A rate limit was cleared.
        RateLimitCleared {
            /// Identifier of the affected transaction.
            transaction: TransactionIdentifier,
            /// Limit scope from which the configuration was cleared, if any.
            scope: Option<<T as Config<I>>::LimitScope>,
            /// Pallet name associated with the transaction.
            pallet: Vec<u8>,
            /// Extrinsic name associated with the transaction.
            extrinsic: Vec<u8>,
        },
        /// All scoped and global rate limits for a call were cleared.
        AllRateLimitsCleared {
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
            Option<<T as Config<I>>::LimitScope>,
            RateLimitKind<BlockNumberFor<T>>,
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

            for (identifier, scope, kind) in &self.limits {
                Limits::<T, I>::mutate(identifier, |entry| match scope {
                    None => {
                        *entry = Some(RateLimit::global(*kind));
                    }
                    Some(sc) => {
                        if let Some(config) = entry {
                            config.upsert_scope(sc.clone(), *kind);
                        } else {
                            *entry = Some(RateLimit::scoped_single(sc.clone(), *kind));
                        }
                    }
                });
            }
        }
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Returns `true` when the given transaction identifier passes its configured rate limit
        /// within the provided usage scope.
        pub fn is_within_limit(
            identifier: &TransactionIdentifier,
            scope: &Option<<T as Config<I>>::LimitScope>,
            usage_key: &Option<<T as Config<I>>::UsageKey>,
            call: &<T as Config<I>>::RuntimeCall,
        ) -> Result<bool, DispatchError> {
            if <T as Config<I>>::LimitScopeResolver::should_bypass(call) {
                return Ok(true);
            }

            let Some(block_span) = Self::effective_span(call, identifier, scope) else {
                return Ok(true);
            };

            Ok(Self::within_span(identifier, usage_key, block_span))
        }

        pub(crate) fn resolved_limit(
            identifier: &TransactionIdentifier,
            scope: &Option<<T as Config<I>>::LimitScope>,
        ) -> Option<BlockNumberFor<T>> {
            let config = Limits::<T, I>::get(identifier)?;
            let kind = config.kind_for(scope.as_ref())?;
            Some(match *kind {
                RateLimitKind::Default => DefaultLimit::<T, I>::get(),
                RateLimitKind::Exact(block_span) => block_span,
            })
        }

        pub(crate) fn effective_span(
            call: &<T as Config<I>>::RuntimeCall,
            identifier: &TransactionIdentifier,
            scope: &Option<<T as Config<I>>::LimitScope>,
        ) -> Option<BlockNumberFor<T>> {
            let span = Self::resolved_limit(identifier, scope)?;
            Some(<T as Config<I>>::LimitScopeResolver::adjust_span(
                call, span,
            ))
        }

        pub(crate) fn within_span(
            identifier: &TransactionIdentifier,
            usage_key: &Option<<T as Config<I>>::UsageKey>,
            block_span: BlockNumberFor<T>,
        ) -> bool {
            if block_span.is_zero() {
                return true;
            }

            if let Some(last) = LastSeen::<T, I>::get(identifier, usage_key) {
                let current = frame_system::Pallet::<T>::block_number();
                let delta = current.saturating_sub(last);
                if delta < block_span {
                    return false;
                }
            }

            true
        }

        /// Returns the configured limit for the specified pallet/extrinsic names, if any.
        pub fn limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
            scope: Option<<T as Config<I>>::LimitScope>,
        ) -> Option<RateLimitKind<BlockNumberFor<T>>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            Limits::<T, I>::get(&identifier)
                .and_then(|config| config.kind_for(scope.as_ref()).copied())
        }

        /// Returns the resolved block span for the specified pallet/extrinsic names, if any.
        pub fn resolved_limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
            scope: Option<<T as Config<I>>::LimitScope>,
        ) -> Option<BlockNumberFor<T>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            Self::resolved_limit(&identifier, &scope)
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
        /// Sets the rate limit configuration for the given call.
        ///
        /// The supplied `call` is inspected to derive the pallet/extrinsic indices and passed to
        /// [`Config::LimitScopeResolver`] to determine the applicable scope. The pallet never
        /// persists the call arguments directly, but a resolver may read them in order to resolve
        /// its context. When a scope resolves, the configuration is stored against that scope;
        /// otherwise the global entry is updated.
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_rate_limit(
            origin: OriginFor<T>,
            call: Box<<T as Config<I>>::RuntimeCall>,
            limit: RateLimitKind<BlockNumberFor<T>>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            let identifier = TransactionIdentifier::from_call::<T, I>(call.as_ref())?;
            let scope = <T as Config<I>>::LimitScopeResolver::context(call.as_ref());
            let scope_for_event = scope.clone();

            if let Some(ref sc) = scope {
                Limits::<T, I>::mutate(&identifier, |slot| match slot {
                    Some(config) => config.upsert_scope(sc.clone(), limit),
                    None => *slot = Some(RateLimit::scoped_single(sc.clone(), limit)),
                });
            } else {
                Limits::<T, I>::insert(&identifier, RateLimit::global(limit));
            }

            let (pallet_name, extrinsic_name) = identifier.names::<T, I>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            Self::deposit_event(Event::RateLimitSet {
                transaction: identifier,
                scope: scope_for_event,
                limit,
                pallet,
                extrinsic,
            });
            Ok(())
        }

        /// Clears the rate limit for the given call, if present.
        ///
        /// The supplied `call` is inspected to derive the pallet/extrinsic indices and passed to
        /// [`Config::LimitScopeResolver`] when determining which scoped configuration to clear.
        /// The pallet does not persist the call arguments, but resolvers may read them while
        /// computing the scope. When no scope resolves, the global entry is cleared.
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn clear_rate_limit(
            origin: OriginFor<T>,
            call: Box<<T as Config<I>>::RuntimeCall>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            let identifier = TransactionIdentifier::from_call::<T, I>(call.as_ref())?;
            let scope = <T as Config<I>>::LimitScopeResolver::context(call.as_ref());
            let usage = <T as Config<I>>::UsageResolver::context(call.as_ref());

            let (pallet_name, extrinsic_name) = identifier.names::<T, I>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            let mut removed = false;
            Limits::<T, I>::mutate_exists(&identifier, |maybe_config| {
                if let Some(config) = maybe_config {
                    match (&scope, config) {
                        (None, _) => {
                            removed = true;
                            *maybe_config = None;
                        }
                        (Some(sc), RateLimit::Scoped(map)) => {
                            if map.remove(sc).is_some() {
                                removed = true;
                                if map.is_empty() {
                                    *maybe_config = None;
                                }
                            }
                        }
                        (Some(_), RateLimit::Global(_)) => {}
                    }
                }
            });

            ensure!(removed, Error::<T, I>::MissingRateLimit);

            if removed {
                match (scope.as_ref(), usage) {
                    (None, _) => {
                        let _ = LastSeen::<T, I>::clear_prefix(&identifier, u32::MAX, None);
                    }
                    (_, Some(key)) => {
                        LastSeen::<T, I>::remove(&identifier, Some(key));
                    }
                    (_, None) => {
                        LastSeen::<T, I>::remove(&identifier, None::<<T as Config<I>>::UsageKey>);
                    }
                }
            }

            Self::deposit_event(Event::RateLimitCleared {
                transaction: identifier,
                scope,
                pallet,
                extrinsic,
            });

            Ok(())
        }

        /// Clears every stored rate limit configuration for the given call, including scoped
        /// entries.
        ///
        /// The supplied `call` is inspected to derive the pallet and extrinsic indices. All stored
        /// scopes for that call, along with any associated usage tracking entries, are removed when
        /// this extrinsic succeeds.
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn clear_all_rate_limits(
            origin: OriginFor<T>,
            call: Box<<T as Config<I>>::RuntimeCall>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            let identifier = TransactionIdentifier::from_call::<T, I>(call.as_ref())?;
            let (pallet_name, extrinsic_name) = identifier.names::<T, I>()?;
            let pallet = Vec::from(pallet_name.as_bytes());
            let extrinsic = Vec::from(extrinsic_name.as_bytes());

            let removed = Limits::<T, I>::take(&identifier).is_some();
            ensure!(removed, Error::<T, I>::MissingRateLimit);

            let _ = LastSeen::<T, I>::clear_prefix(&identifier, u32::MAX, None);

            Self::deposit_event(Event::AllRateLimitsCleared {
                transaction: identifier,
                pallet,
                extrinsic,
            });

            Ok(())
        }

        /// Sets the default rate limit in blocks applied to calls configured to use it.
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().writes(1))]
        pub fn set_default_rate_limit(
            origin: OriginFor<T>,
            block_span: BlockNumberFor<T>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            DefaultLimit::<T, I>::put(block_span);

            Self::deposit_event(Event::DefaultRateLimitSet { block_span });

            Ok(())
        }
    }
}
