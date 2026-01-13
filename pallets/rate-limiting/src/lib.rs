#![cfg_attr(not(feature = "std"), no_std)]

//! Rate limiting for runtime calls with optional contextual restrictions.
//!
//! # Overview
//!
//! `pallet-rate-limiting` lets a runtime restrict how frequently particular calls can execute.
//! Limits are stored on-chain, keyed by explicit [`RateLimitTarget`] values. A target is either a
//! single [`TransactionIdentifier`] (the pallet/extrinsic indices) or a named *group* managed by
//! the admin APIs. Groups provide a way to give multiple calls the same configuration and/or usage
//! tracking without duplicating storage. Each target entry stores either a global span or a set of
//! scoped spans resolved at runtime. The pallet exposes a handful of extrinsics, restricted by
//! [`Config::AdminOrigin`], to manage this data:
//!
//! - [`register_call`](pallet::Pallet::register_call): register a call for rate limiting, seed its
//!   initial configuration using [`Config::LimitScopeResolver`], and optionally place it into a
//!   group.
//! - [`set_rate_limit`](pallet::Pallet::set_rate_limit): assign or override the limit at a specific
//!   target/scope by supplying a [`RateLimitKind`] span.
//! - [`assign_call_to_group`](pallet::Pallet::assign_call_to_group) and
//!   [`remove_call_from_group`](pallet::Pallet::remove_call_from_group): manage group membership
//!   for registered calls.
//! - [`set_call_read_only`](pallet::Pallet::set_call_read_only): for grouped calls, choose whether
//!   successful dispatches should update the shared usage row (`false` by default).
//! - [`deregister_call`](pallet::Pallet::deregister_call): remove scoped configuration or wipe the
//!   registration entirely.
//! - [`set_default_rate_limit`](pallet::Pallet::set_default_rate_limit): set the global default
//!   block span used by `RateLimitKind::Default` entries.
//!
//! The pallet also tracks the last block in which a target was observed, per optional *usage key*.
//! A usage key may refine tracking beyond the limit scope (for example combining a `netuid` with a
//! hyperparameter), so the two concepts are explicitly separated in the configuration. When the
//! admin puts several calls into a group and marks usage as shared, each dispatch still runs the
//! resolver: the group only chooses the storage target, while the resolver output (or `None`) picks
//! the row under that target. Calls that resolve to the same usage key update the same timestamp;
//! calls that resolve to different keys keep isolated timers even when they share a group. The same
//! rule applies to limit scopes—grouping funnels configuration into the same target, but the scope
//! resolver decides whether that entry is global or per-context.
//!
//! Each storage map is namespaced by pallet instance; runtimes can deploy multiple independent
//! instances to manage distinct rate-limiting scopes (in the global sense).
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
//! Each resolver receives the origin and call and may return `Some(identifier)` when scoping is
//! required, or `None` to use the global entry. Extrinsics such as
//! [`set_rate_limit`](pallet::Pallet::set_rate_limit) automatically consult these resolvers. When a
//! call belongs to a group the pallet still runs the resolver—instead of indexing storage at the
//! transaction-level target, it indexes at the group target. Resolving to different contexts keeps
//! independent limit/usage rows even though the calls share a group; resolving to the same context
//! causes them to share enforcement state.
//!
//! ```ignore
//! pub struct WeightsContextResolver;
//!
//! // Limits are scoped per netuid.
//! pub struct ScopeResolver;
//! impl pallet_rate_limiting::RateLimitScopeResolver<
//!     RuntimeOrigin,
//!     RuntimeCall,
//!     NetUid,
//!     BlockNumber,
//! > for ScopeResolver {
//!     fn context(origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<Vec<NetUid>> {
//!         match call {
//!             RuntimeCall::Subtensor(pallet_subtensor::Call::set_weights { netuid, .. }) => {
//!                 Some(vec![*netuid])
//!             }
//!             _ => None,
//!         }
//!     }
//!
//!     fn should_bypass(origin: &RuntimeOrigin, _call: &RuntimeCall) -> BypassDecision {
//!         if matches!(origin, RuntimeOrigin::Root) {
//!             BypassDecision::bypass_and_skip()
//!         } else {
//!             BypassDecision::enforce_and_record()
//!         }
//!     }
//!
//!     fn adjust_span(_origin: &RuntimeOrigin, _call: &RuntimeCall, span: BlockNumber) -> BlockNumber {
//!         span
//!     }
//! }
//!
//! // Usage tracking distinguishes hyperparameter + netuid.
//! pub struct UsageResolver;
//! impl pallet_rate_limiting::RateLimitUsageResolver<
//!     RuntimeOrigin,
//!     RuntimeCall,
//!     (NetUid, HyperParam),
//! > for UsageResolver {
//!     fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<(NetUid, HyperParam)> {
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
pub use rate_limiting_interface::{RateLimitTarget, TransactionIdentifier};
pub use rate_limiting_interface::{RateLimitingInterface, TryIntoRateLimitTarget};
pub use tx_extension::RateLimitTransactionExtension;
pub use types::{
    BypassDecision, EnsureLimitSettingRule, GroupSharing, RateLimit, RateLimitGroup, RateLimitKind,
    RateLimitScopeResolver, RateLimitUsageResolver,
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
        BoundedBTreeSet, BoundedVec,
        pallet_prelude::*,
        traits::{BuildGenesisConfig, EnsureOrigin, GetCallMetadata},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{
        AtLeast32BitUnsigned, DispatchOriginOf, Dispatchable, Member, One, Saturating, Zero,
    };
    use sp_std::{
        boxed::Box, collections::btree_map::BTreeMap, convert::TryFrom, marker::PhantomData, vec,
        vec::Vec,
    };

    #[cfg(feature = "runtime-benchmarks")]
    use crate::benchmarking::BenchmarkHelper as BenchmarkHelperTrait;
    use crate::types::{
        EnsureLimitSettingRule, GroupSharing, RateLimit, RateLimitGroup, RateLimitKind,
        RateLimitScopeResolver, RateLimitTarget, RateLimitUsageResolver, TransactionIdentifier,
    };

    type GroupNameOf<T, I> = BoundedVec<u8, <T as Config<I>>::MaxGroupNameLength>;
    type GroupMembersOf<T, I> =
        BoundedBTreeSet<TransactionIdentifier, <T as Config<I>>::MaxGroupMembers>;
    type GroupDetailsOf<T, I> = RateLimitGroup<<T as Config<I>>::GroupId, GroupNameOf<T, I>>;

    /// Configuration trait for the rate limiting pallet.
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config
    where
        BlockNumberFor<Self>: MaybeSerializeDeserialize,
        <<Self as Config<I>>::RuntimeCall as Dispatchable>::RuntimeOrigin:
            From<<Self as frame_system::Config>::RuntimeOrigin>,
    {
        /// The overarching runtime call type.
        type RuntimeCall: Parameter
            + Codec
            + GetCallMetadata
            + Dispatchable
            + IsType<<Self as frame_system::Config>::RuntimeCall>;

        /// Origin permitted to configure rate limits.
        type AdminOrigin: EnsureOrigin<OriginFor<Self>>;

        /// Rule type that decides which origins may call [`Pallet::set_rate_limit`].
        type LimitSettingRule: Parameter + Member + MaxEncodedLen + MaybeSerializeDeserialize;

        /// Default rule applied when a target does not have an explicit entry in
        /// [`LimitSettingRules`].
        type DefaultLimitSettingRule: Get<Self::LimitSettingRule>;

        /// Origin checker invoked when setting a rate limit, parameterized by the stored rule.
        type LimitSettingOrigin: EnsureLimitSettingRule<OriginFor<Self>, Self::LimitSettingRule, Self::LimitScope>;

        /// Scope identifier used to namespace stored rate limits.
        type LimitScope: Parameter + Clone + PartialEq + Eq + Ord + MaybeSerializeDeserialize;

        /// Resolves the scope for the given runtime call when configuring limits.
        type LimitScopeResolver: RateLimitScopeResolver<
                DispatchOriginOf<<Self as Config<I>>::RuntimeCall>,
                <Self as Config<I>>::RuntimeCall,
                Self::LimitScope,
                BlockNumberFor<Self>,
            >;

        /// Usage key tracked in [`LastSeen`] for rate-limited calls.
        type UsageKey: Parameter + Clone + PartialEq + Eq + Ord + MaybeSerializeDeserialize;

        /// Resolves the usage key for the given runtime call when enforcing limits.
        type UsageResolver: RateLimitUsageResolver<
                DispatchOriginOf<<Self as Config<I>>::RuntimeCall>,
                <Self as Config<I>>::RuntimeCall,
                Self::UsageKey,
            >;

        /// Identifier assigned to managed groups.
        type GroupId: Parameter
            + Member
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + AtLeast32BitUnsigned
            + Default;

        /// Maximum number of extrinsics that may belong to a single group.
        #[pallet::constant]
        type MaxGroupMembers: Get<u32>;

        /// Maximum length (in bytes) of a group name.
        #[pallet::constant]
        type MaxGroupNameLength: Get<u32>;

        /// Helper used to construct runtime calls for benchmarking.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelperTrait<<Self as Config<I>>::RuntimeCall>;
    }

    /// Storage mapping from rate limit target to its configured rate limit.
    #[pallet::storage]
    #[pallet::getter(fn limits)]
    pub type Limits<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        RateLimitTarget<<T as Config<I>>::GroupId>,
        RateLimit<<T as Config<I>>::LimitScope, BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::type_value]
    pub fn DefaultLimitSettingRuleFor<T: Config<I>, I: 'static>() -> T::LimitSettingRule {
        T::DefaultLimitSettingRule::get()
    }

    /// Stores the rule used to authorize [`Pallet::set_rate_limit`] per call/group target.
    #[pallet::storage]
    #[pallet::getter(fn limit_setting_rule)]
    pub type LimitSettingRules<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        RateLimitTarget<<T as Config<I>>::GroupId>,
        <T as Config<I>>::LimitSettingRule,
        ValueQuery,
        DefaultLimitSettingRuleFor<T, I>,
    >;

    /// Tracks when a rate-limited target was last observed per usage key.
    #[pallet::storage]
    pub type LastSeen<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        RateLimitTarget<<T as Config<I>>::GroupId>,
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

    /// Maps a transaction identifier to its assigned group.
    #[pallet::storage]
    #[pallet::getter(fn call_group)]
    pub type CallGroups<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        TransactionIdentifier,
        <T as Config<I>>::GroupId,
        OptionQuery,
    >;

    /// Tracks whether a grouped call should skip writing usage metadata on success.
    #[pallet::storage]
    #[pallet::getter(fn call_read_only)]
    pub type CallReadOnly<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, TransactionIdentifier, bool, OptionQuery>;

    /// Metadata for each configured group.
    #[pallet::storage]
    #[pallet::getter(fn groups)]
    pub type Groups<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        <T as Config<I>>::GroupId,
        GroupDetailsOf<T, I>,
        OptionQuery,
    >;

    /// Tracks membership for each group.
    #[pallet::storage]
    #[pallet::getter(fn group_members)]
    pub type GroupMembers<T: Config<I>, I: 'static = ()> = StorageMap<
        _,
        Blake2_128Concat,
        <T as Config<I>>::GroupId,
        GroupMembersOf<T, I>,
        ValueQuery,
    >;

    /// Enforces unique group names.
    #[pallet::storage]
    #[pallet::getter(fn group_id_by_name)]
    pub type GroupNameIndex<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Blake2_128Concat, GroupNameOf<T, I>, <T as Config<I>>::GroupId, OptionQuery>;

    /// Identifier used for the next group creation.
    #[pallet::storage]
    #[pallet::getter(fn next_group_id)]
    pub type NextGroupId<T: Config<I>, I: 'static = ()> =
        StorageValue<_, <T as Config<I>>::GroupId, ValueQuery>;

    /// Events emitted by the rate limiting pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// A call was registered for rate limiting.
        CallRegistered {
            /// Identifier of the registered transaction.
            transaction: TransactionIdentifier,
            /// Scope seeded during registration (if any).
            scope: Option<Vec<<T as Config<I>>::LimitScope>>,
            /// Optional group assignment applied at registration time.
            group: Option<<T as Config<I>>::GroupId>,
            /// Pallet name associated with the transaction.
            pallet: Vec<u8>,
            /// Extrinsic name associated with the transaction.
            extrinsic: Vec<u8>,
        },
        /// A rate limit was set or updated for the specified target.
        RateLimitSet {
            /// Target whose configuration changed.
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            /// Identifier of the transaction when the target represents a call.
            transaction: Option<TransactionIdentifier>,
            /// Limit scope to which the configuration applies, if any.
            scope: Option<<T as Config<I>>::LimitScope>,
            /// The rate limit policy applied to the target.
            limit: RateLimitKind<BlockNumberFor<T>>,
            /// Pallet name associated with the transaction, when available.
            pallet: Option<Vec<u8>>,
            /// Extrinsic name associated with the transaction, when available.
            extrinsic: Option<Vec<u8>>,
        },
        /// The rule that authorizes [`Pallet::set_rate_limit`] was updated for a target.
        LimitSettingRuleUpdated {
            /// Target whose limit-setting rule changed.
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            /// Updated rule.
            rule: <T as Config<I>>::LimitSettingRule,
        },
        /// A rate-limited call was deregistered or had a scoped entry cleared.
        CallDeregistered {
            /// Target whose configuration changed.
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            /// Identifier of the transaction when the target represents a call.
            transaction: Option<TransactionIdentifier>,
            /// Limit scope from which the configuration was cleared, if any.
            scope: Option<<T as Config<I>>::LimitScope>,
            /// Pallet name associated with the transaction, when available.
            pallet: Option<Vec<u8>>,
            /// Extrinsic name associated with the transaction, when available.
            extrinsic: Option<Vec<u8>>,
        },
        /// The default rate limit was set or updated.
        DefaultRateLimitSet {
            /// The new default limit expressed in blocks.
            block_span: BlockNumberFor<T>,
        },
        /// A group was created.
        GroupCreated {
            /// Identifier of the new group.
            group: <T as Config<I>>::GroupId,
            /// Human readable group name.
            name: Vec<u8>,
            /// Sharing policy configured for the group.
            sharing: GroupSharing,
        },
        /// A group's metadata or policy changed.
        GroupUpdated {
            /// Identifier of the group.
            group: <T as Config<I>>::GroupId,
            /// Human readable name.
            name: Vec<u8>,
            /// Updated sharing configuration.
            sharing: GroupSharing,
        },
        /// A group was deleted.
        GroupDeleted {
            /// Identifier of the removed group.
            group: <T as Config<I>>::GroupId,
        },
        /// A transaction was assigned to or removed from a group.
        CallGroupUpdated {
            /// Identifier of the transaction.
            transaction: TransactionIdentifier,
            /// Updated group assignment (None when cleared).
            group: Option<<T as Config<I>>::GroupId>,
        },
        /// A grouped call toggled whether it writes usage after enforcement.
        CallReadOnlyUpdated {
            /// Identifier of the transaction.
            transaction: TransactionIdentifier,
            /// Group to which the call belongs.
            group: <T as Config<I>>::GroupId,
            /// Current read-only flag.
            read_only: bool,
        },
    }

    /// Errors that can occur while configuring rate limits.
    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Failed to extract the pallet and extrinsic indices from the call.
        InvalidRuntimeCall,
        /// Attempted to remove a limit that is not present.
        MissingRateLimit,
        /// Group metadata was not found.
        UnknownGroup,
        /// Attempted to create or rename a group to an existing name.
        DuplicateGroupName,
        /// Group name exceeds the configured maximum length.
        GroupNameTooLong,
        /// Operation requires the group to have no members.
        GroupHasMembers,
        /// Adding a member would exceed the configured limit.
        GroupMemberLimitExceeded,
        /// Call already belongs to the requested group.
        CallAlreadyInGroup,
        /// Call is not assigned to a group.
        CallNotInGroup,
        /// Operation requires the call to be registered first.
        CallNotRegistered,
        /// Attempted to register a call that already exists.
        CallAlreadyRegistered,
        /// Rate limit for this call must be configured via its group target.
        MustTargetGroup,
        /// Resolver failed to supply a required context value.
        MissingScope,
        /// Group cannot be removed because configuration or usage entries remain.
        GroupInUse,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub default_limit: BlockNumberFor<T>,
        pub limits: Vec<(
            RateLimitTarget<<T as Config<I>>::GroupId>,
            Option<<T as Config<I>>::LimitScope>,
            RateLimitKind<BlockNumberFor<T>>,
        )>,
        pub groups: Vec<(<T as Config<I>>::GroupId, Vec<u8>, GroupSharing)>,
        pub limit_setting_rules: Vec<(
            RateLimitTarget<<T as Config<I>>::GroupId>,
            <T as Config<I>>::LimitSettingRule,
        )>,
    }

    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                default_limit: Zero::zero(),
                limits: Vec::new(),
                groups: Vec::new(),
                limit_setting_rules: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
        fn build(&self) {
            DefaultLimit::<T, I>::put(self.default_limit);

            // Seed groups first so limit targets can reference them.
            let mut max_group: <T as Config<I>>::GroupId = Zero::zero();
            for (group_id, name, sharing) in &self.groups {
                let bounded = GroupNameOf::<T, I>::try_from(name.clone())
                    .expect("Genesis group name exceeds MaxGroupNameLength");

                assert!(
                    !Groups::<T, I>::contains_key(group_id),
                    "Duplicate group id in genesis config"
                );
                assert!(
                    !GroupNameIndex::<T, I>::contains_key(&bounded),
                    "Duplicate group name in genesis config"
                );

                Groups::<T, I>::insert(
                    group_id,
                    RateLimitGroup {
                        id: *group_id,
                        name: bounded.clone(),
                        sharing: *sharing,
                    },
                );
                GroupNameIndex::<T, I>::insert(&bounded, *group_id);
                GroupMembers::<T, I>::insert(*group_id, GroupMembersOf::<T, I>::new());
                if *group_id > max_group {
                    max_group = *group_id;
                }
            }
            let next = max_group.saturating_add(One::one());
            NextGroupId::<T, I>::put(next);

            for (identifier, scope, kind) in &self.limits {
                if let RateLimitTarget::Group(group) = identifier {
                    assert!(
                        Groups::<T, I>::contains_key(group),
                        "Genesis limit references unknown group"
                    );
                }
                let target = *identifier;
                Limits::<T, I>::mutate(target, |entry| match scope {
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

            for (target, rule) in &self.limit_setting_rules {
                LimitSettingRules::<T, I>::insert(target, rule.clone());
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
            origin: &DispatchOriginOf<<T as Config<I>>::RuntimeCall>,
            call: &<T as Config<I>>::RuntimeCall,
            identifier: &TransactionIdentifier,
            scopes: &Option<Vec<<T as Config<I>>::LimitScope>>,
            usage_key: &Option<<T as Config<I>>::UsageKey>,
        ) -> Result<bool, DispatchError> {
            let bypass = <T as Config<I>>::LimitScopeResolver::should_bypass(origin, call);
            if bypass.bypass_enforcement {
                return Ok(true);
            }

            let target = Self::config_target(identifier)?;
            Self::ensure_scope_available(&target, scopes)?;

            let usage_target = Self::usage_target(identifier)?;
            let scope_list: Vec<Option<<T as Config<I>>::LimitScope>> = match scopes {
                None => vec![None],
                Some(resolved) if resolved.is_empty() => vec![None],
                Some(resolved) => resolved.iter().cloned().map(Some).collect(),
            };

            for scope in scope_list {
                let Some(block_span) = Self::effective_span(origin, call, &target, &scope) else {
                    continue;
                };
                if !Self::within_span(&usage_target, usage_key, block_span) {
                    return Ok(false);
                }
            }

            Ok(true)
        }

        /// Resolves the configured span for the provided target/scope, applying the pallet default
        /// when the stored value uses [`RateLimitKind::Default`].
        pub fn resolved_limit(
            target: &RateLimitTarget<<T as Config<I>>::GroupId>,
            scope: &Option<<T as Config<I>>::LimitScope>,
        ) -> Option<BlockNumberFor<T>> {
            let config = Limits::<T, I>::get(target)?;
            let kind = config.kind_for(scope.as_ref())?;
            Some(match *kind {
                RateLimitKind::Default => DefaultLimit::<T, I>::get(),
                RateLimitKind::Exact(block_span) => block_span,
            })
        }

        /// Resolves the span for a target/scope and applies the configured span adjustment (e.g.,
        /// tempo scaling) using the pallet's scope resolver.
        pub fn effective_span(
            origin: &DispatchOriginOf<<T as Config<I>>::RuntimeCall>,
            call: &<T as Config<I>>::RuntimeCall,
            target: &RateLimitTarget<<T as Config<I>>::GroupId>,
            scope: &Option<<T as Config<I>>::LimitScope>,
        ) -> Option<BlockNumberFor<T>> {
            let span = Self::resolved_limit(target, scope)?;
            Some(<T as Config<I>>::LimitScopeResolver::adjust_span(
                origin, call, span,
            ))
        }

        pub(crate) fn within_span(
            target: &RateLimitTarget<<T as Config<I>>::GroupId>,
            usage_key: &Option<<T as Config<I>>::UsageKey>,
            block_span: BlockNumberFor<T>,
        ) -> bool {
            if block_span.is_zero() {
                return true;
            }

            if let Some(last) = LastSeen::<T, I>::get(target, usage_key) {
                let current = frame_system::Pallet::<T>::block_number();
                let delta = current.saturating_sub(last);
                if delta < block_span {
                    return false;
                }
            }

            true
        }

        pub(crate) fn should_record_usage(
            identifier: &TransactionIdentifier,
            usage_target: &RateLimitTarget<<T as Config<I>>::GroupId>,
        ) -> bool {
            match usage_target {
                RateLimitTarget::Group(_) => {
                    !CallReadOnly::<T, I>::get(identifier).unwrap_or(false)
                }
                RateLimitTarget::Transaction(_) => true,
            }
        }

        /// Inserts or updates the cached usage timestamp for a rate-limited call.
        ///
        /// This is primarily intended for migrations that need to hydrate the new tracking storage
        /// from legacy pallets.
        pub fn record_last_seen(
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            usage_key: Option<<T as Config<I>>::UsageKey>,
            block_number: BlockNumberFor<T>,
        ) {
            LastSeen::<T, I>::insert(target, usage_key, block_number);
        }

        /// Migrates a stored rate limit configuration from one scope to another.
        ///
        /// Returns `true` when an entry was moved. Passing identical `from`/`to` scopes simply
        /// checks that a configuration exists.
        pub fn migrate_limit_scope(
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            from: Option<<T as Config<I>>::LimitScope>,
            to: Option<<T as Config<I>>::LimitScope>,
        ) -> bool {
            if from == to {
                return Limits::<T, I>::contains_key(target);
            }

            let mut migrated = false;
            Limits::<T, I>::mutate(target, |maybe_config| {
                if let Some(config) = maybe_config {
                    match (from.as_ref(), to.as_ref()) {
                        (None, Some(target)) => {
                            if let RateLimit::Global(kind) = config {
                                *config = RateLimit::scoped_single(target.clone(), *kind);
                                migrated = true;
                            }
                        }
                        (Some(source), Some(target)) => {
                            if let RateLimit::Scoped(map) = config {
                                if let Some(kind) = map.remove(source) {
                                    map.insert(target.clone(), kind);
                                    migrated = true;
                                }
                            }
                        }
                        (Some(source), None) => {
                            if let RateLimit::Scoped(map) = config {
                                if map.len() == 1 && map.contains_key(source) {
                                    if let Some(kind) = map.remove(source) {
                                        *config = RateLimit::global(kind);
                                        migrated = true;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });

            migrated
        }

        /// Migrates the cached usage information for a rate-limited call to a new key.
        ///
        /// Returns `true` when an entry was moved. Passing identical keys simply checks that an
        /// entry exists.
        pub fn migrate_usage_key(
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            from: Option<<T as Config<I>>::UsageKey>,
            to: Option<<T as Config<I>>::UsageKey>,
        ) -> bool {
            if from == to {
                return LastSeen::<T, I>::contains_key(target, to);
            }

            let Some(block) = LastSeen::<T, I>::take(target, from) else {
                return false;
            };

            LastSeen::<T, I>::insert(target, to, block);
            true
        }

        /// Returns the configured limit for the specified pallet/extrinsic names, if any.
        pub fn limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
            scope: Option<<T as Config<I>>::LimitScope>,
        ) -> Option<RateLimitKind<BlockNumberFor<T>>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            let target = Self::config_target(&identifier).ok()?;
            Limits::<T, I>::get(target).and_then(|config| config.kind_for(scope.as_ref()).copied())
        }

        /// Returns the resolved block span for the specified pallet/extrinsic names, if any.
        pub fn resolved_limit_for_call_names(
            pallet_name: &str,
            extrinsic_name: &str,
            scope: Option<<T as Config<I>>::LimitScope>,
        ) -> Option<BlockNumberFor<T>> {
            let identifier = Self::identifier_for_call_names(pallet_name, extrinsic_name)?;
            let target = Self::config_target(&identifier).ok()?;
            Self::resolved_limit(&target, &scope)
        }

        /// Looks up the transaction identifier for a pallet/extrinsic name pair.
        pub fn identifier_for_call_names(
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

        fn ensure_call_registered(identifier: &TransactionIdentifier) -> DispatchResult {
            let target = RateLimitTarget::Transaction(*identifier);
            ensure!(
                Limits::<T, I>::contains_key(target),
                Error::<T, I>::CallNotRegistered
            );
            Ok(())
        }

        fn ensure_call_unregistered(identifier: &TransactionIdentifier) -> DispatchResult {
            let target = RateLimitTarget::Transaction(*identifier);
            ensure!(
                !Limits::<T, I>::contains_key(target),
                Error::<T, I>::CallAlreadyRegistered
            );
            Ok(())
        }

        /// Returns true when the call has been registered (either directly or via a group).
        pub fn is_registered(identifier: &TransactionIdentifier) -> bool {
            let tx_target = RateLimitTarget::Transaction(*identifier);
            Limits::<T, I>::contains_key(tx_target) || CallGroups::<T, I>::contains_key(identifier)
        }

        fn call_metadata(
            identifier: &TransactionIdentifier,
        ) -> Result<(Vec<u8>, Vec<u8>), DispatchError> {
            let (pallet_name, extrinsic_name) = identifier
                .names::<<T as Config<I>>::RuntimeCall>()
                .ok_or(Error::<T, I>::InvalidRuntimeCall)?;
            Ok((
                Vec::from(pallet_name.as_bytes()),
                Vec::from(extrinsic_name.as_bytes()),
            ))
        }

        /// Returns the storage target used to store configuration for the provided identifier,
        /// respecting any configured group assignment.
        pub fn config_target(
            identifier: &TransactionIdentifier,
        ) -> Result<RateLimitTarget<<T as Config<I>>::GroupId>, DispatchError> {
            Self::target_for(identifier, GroupSharing::config_uses_group)
        }

        pub(crate) fn usage_target(
            identifier: &TransactionIdentifier,
        ) -> Result<RateLimitTarget<<T as Config<I>>::GroupId>, DispatchError> {
            Self::target_for(identifier, GroupSharing::usage_uses_group)
        }

        fn target_for(
            identifier: &TransactionIdentifier,
            predicate: impl Fn(GroupSharing) -> bool,
        ) -> Result<RateLimitTarget<<T as Config<I>>::GroupId>, DispatchError> {
            let group = Self::group_assignment(identifier)?;
            Ok(Self::target_from_details(
                identifier,
                group.as_ref(),
                predicate,
            ))
        }

        fn group_assignment(
            identifier: &TransactionIdentifier,
        ) -> Result<Option<GroupDetailsOf<T, I>>, DispatchError> {
            let Some(group) = CallGroups::<T, I>::get(identifier) else {
                return Ok(None);
            };
            let details = Self::ensure_group_details(group)?;
            Ok(Some(details))
        }

        fn target_from_details(
            identifier: &TransactionIdentifier,
            details: Option<&GroupDetailsOf<T, I>>,
            predicate: impl Fn(GroupSharing) -> bool,
        ) -> RateLimitTarget<<T as Config<I>>::GroupId> {
            if let Some(details) = details {
                if predicate(details.sharing) {
                    return RateLimitTarget::Group(details.id);
                }
            }
            RateLimitTarget::Transaction(*identifier)
        }

        fn ensure_group_details(
            group: <T as Config<I>>::GroupId,
        ) -> Result<GroupDetailsOf<T, I>, DispatchError> {
            Groups::<T, I>::get(group).ok_or(Error::<T, I>::UnknownGroup.into())
        }

        fn ensure_scope_available(
            target: &RateLimitTarget<<T as Config<I>>::GroupId>,
            scopes: &Option<Vec<<T as Config<I>>::LimitScope>>,
        ) -> Result<(), DispatchError> {
            let has_scope = scopes.as_ref().map_or(false, |scopes| !scopes.is_empty());
            if has_scope {
                return Ok(());
            }

            if let Some(RateLimit::Scoped(map)) = Limits::<T, I>::get(target) {
                if !map.is_empty() {
                    return Err(Error::<T, I>::MissingScope.into());
                }
            }

            Ok(())
        }

        fn bounded_group_name(name: Vec<u8>) -> Result<GroupNameOf<T, I>, DispatchError> {
            GroupNameOf::<T, I>::try_from(name).map_err(|_| Error::<T, I>::GroupNameTooLong.into())
        }

        fn ensure_group_name_available(
            name: &GroupNameOf<T, I>,
            current: Option<<T as Config<I>>::GroupId>,
        ) -> DispatchResult {
            if let Some(existing) = GroupNameIndex::<T, I>::get(name) {
                ensure!(Some(existing) == current, Error::<T, I>::DuplicateGroupName);
            }
            Ok(())
        }

        fn ensure_group_deletable(group: <T as Config<I>>::GroupId) -> DispatchResult {
            ensure!(
                GroupMembers::<T, I>::get(group).is_empty(),
                Error::<T, I>::GroupHasMembers
            );
            let target = RateLimitTarget::Group(group);
            ensure!(
                !Limits::<T, I>::contains_key(target),
                Error::<T, I>::GroupInUse
            );
            ensure!(
                LastSeen::<T, I>::iter_prefix(target).next().is_none(),
                Error::<T, I>::GroupInUse
            );
            Ok(())
        }

        fn insert_call_into_group(
            identifier: &TransactionIdentifier,
            group: <T as Config<I>>::GroupId,
        ) -> DispatchResult {
            GroupMembers::<T, I>::try_mutate(group, |members| -> DispatchResult {
                match members.try_insert(*identifier) {
                    Ok(true) => Ok(()),
                    Ok(false) => Err(Error::<T, I>::CallAlreadyInGroup.into()),
                    Err(_) => Err(Error::<T, I>::GroupMemberLimitExceeded.into()),
                }
            })?;
            Ok(())
        }

        fn detach_call_from_group(
            identifier: &TransactionIdentifier,
            group: <T as Config<I>>::GroupId,
        ) -> bool {
            GroupMembers::<T, I>::mutate(group, |members| members.remove(identifier))
        }
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Registers a call for rate limiting and seeds its initial configuration.
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3))]
        pub fn register_call(
            origin: OriginFor<T>,
            call: Box<<T as Config<I>>::RuntimeCall>,
            group: Option<<T as Config<I>>::GroupId>,
        ) -> DispatchResult {
            let resolver_origin: DispatchOriginOf<<T as Config<I>>::RuntimeCall> =
                Into::<DispatchOriginOf<<T as Config<I>>::RuntimeCall>>::into(origin.clone());
            let scopes =
                <T as Config<I>>::LimitScopeResolver::context(&resolver_origin, call.as_ref());

            T::AdminOrigin::ensure_origin(origin)?;

            let identifier = TransactionIdentifier::from_call(call.as_ref())
                .ok_or(Error::<T, I>::InvalidRuntimeCall)?;
            Self::ensure_call_unregistered(&identifier)?;

            let target = RateLimitTarget::Transaction(identifier);

            let scopes = scopes.and_then(|scopes| {
                if scopes.is_empty() {
                    None
                } else {
                    Some(scopes)
                }
            });
            if let Some(ref resolved) = scopes {
                let mut map = BTreeMap::new();
                for scope in resolved {
                    map.insert(scope.clone(), RateLimitKind::Default);
                }
                Limits::<T, I>::insert(target, RateLimit::Scoped(map));
            } else {
                Limits::<T, I>::insert(target, RateLimit::global(RateLimitKind::Default));
            }

            let mut assigned_group = None;
            if let Some(group_id) = group {
                Self::ensure_group_details(group_id)?;
                Self::insert_call_into_group(&identifier, group_id)?;
                CallGroups::<T, I>::insert(&identifier, group_id);
                CallReadOnly::<T, I>::insert(&identifier, false);
                assigned_group = Some(group_id);
            }

            let (pallet, extrinsic) = Self::call_metadata(&identifier)?;
            Self::deposit_event(Event::CallRegistered {
                transaction: identifier,
                scope: scopes,
                group: assigned_group,
                pallet: pallet,
                extrinsic: extrinsic,
            });

            if let Some(group_id) = assigned_group {
                Self::deposit_event(Event::CallGroupUpdated {
                    transaction: identifier,
                    group: Some(group_id),
                });
                Self::deposit_event(Event::CallReadOnlyUpdated {
                    transaction: identifier,
                    group: group_id,
                    read_only: false,
                });
            }

            Ok(())
        }

        /// Configures a rate limit for either a transaction or group target.
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn set_rate_limit(
            origin: OriginFor<T>,
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            scope: Option<<T as Config<I>>::LimitScope>,
            limit: RateLimitKind<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let rule = LimitSettingRules::<T, I>::get(&target);
            T::LimitSettingOrigin::ensure_origin(origin, &rule, &scope)?;

            let (transaction, pallet, extrinsic) = match target {
                RateLimitTarget::Transaction(identifier) => {
                    Self::ensure_call_registered(&identifier)?;
                    if let Some(group) = CallGroups::<T, I>::get(&identifier) {
                        let details = Self::ensure_group_details(group)?;
                        ensure!(
                            !details.sharing.config_uses_group(),
                            Error::<T, I>::MustTargetGroup
                        );
                    }
                    let (pallet, extrinsic) = Self::call_metadata(&identifier)?;
                    (Some(identifier), Some(pallet), Some(extrinsic))
                }
                RateLimitTarget::Group(group) => {
                    Self::ensure_group_details(group)?;
                    (None, None, None)
                }
            };

            if let Some(ref scoped) = scope {
                Limits::<T, I>::mutate(target, |slot| match slot {
                    Some(config) => config.upsert_scope(scoped.clone(), limit),
                    None => *slot = Some(RateLimit::scoped_single(scoped.clone(), limit)),
                });
            } else {
                Limits::<T, I>::insert(target, RateLimit::global(limit));
            }

            Self::deposit_event(Event::RateLimitSet {
                target,
                transaction,
                scope,
                limit,
                pallet,
                extrinsic,
            });
            Ok(())
        }

        /// Sets the rule used to authorize [`Pallet::set_rate_limit`] for the provided target.
        #[pallet::call_index(10)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
        pub fn set_limit_setting_rule(
            origin: OriginFor<T>,
            target: RateLimitTarget<<T as Config<I>>::GroupId>,
            rule: <T as Config<I>>::LimitSettingRule,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            match target {
                RateLimitTarget::Transaction(identifier) => {
                    Self::ensure_call_registered(&identifier)?;
                }
                RateLimitTarget::Group(group) => {
                    Self::ensure_group_details(group)?;
                }
            }

            LimitSettingRules::<T, I>::insert(target, rule.clone());
            Self::deposit_event(Event::LimitSettingRuleUpdated { target, rule });

            Ok(())
        }

        /// Assigns a registered call to the specified group and optionally marks it as read-only
        /// for usage tracking.
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3))]
        pub fn assign_call_to_group(
            origin: OriginFor<T>,
            transaction: TransactionIdentifier,
            group: <T as Config<I>>::GroupId,
            read_only: bool,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            Self::ensure_call_registered(&transaction)?;
            Self::ensure_group_details(group)?;

            let current = CallGroups::<T, I>::get(&transaction);
            ensure!(current.is_none(), Error::<T, I>::CallAlreadyInGroup);
            Self::insert_call_into_group(&transaction, group)?;
            CallGroups::<T, I>::insert(&transaction, group);
            CallReadOnly::<T, I>::insert(&transaction, read_only);

            Self::deposit_event(Event::CallGroupUpdated {
                transaction,
                group: Some(group),
            });
            Self::deposit_event(Event::CallReadOnlyUpdated {
                transaction,
                group,
                read_only,
            });

            Ok(())
        }

        /// Removes a registered call from its current group assignment.
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn remove_call_from_group(
            origin: OriginFor<T>,
            transaction: TransactionIdentifier,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            Self::ensure_call_registered(&transaction)?;
            let Some(group) = CallGroups::<T, I>::take(&transaction) else {
                return Err(Error::<T, I>::CallNotInGroup.into());
            };
            CallReadOnly::<T, I>::remove(&transaction);
            Self::detach_call_from_group(&transaction, group);

            Self::deposit_event(Event::CallGroupUpdated {
                transaction,
                group: None,
            });

            Ok(())
        }

        /// Sets the default rate limit that applies when an extrinsic uses [`RateLimitKind::Default`].
        #[pallet::call_index(4)]
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

        /// Creates a new rate-limiting group with the provided name and sharing configuration.
        #[pallet::call_index(5)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 3))]
        pub fn create_group(
            origin: OriginFor<T>,
            name: Vec<u8>,
            sharing: GroupSharing,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            let bounded = Self::bounded_group_name(name)?;
            Self::ensure_group_name_available(&bounded, None)?;

            let group = NextGroupId::<T, I>::mutate(|current| {
                let next = current.saturating_add(One::one());
                sp_std::mem::replace(current, next)
            });

            Groups::<T, I>::insert(
                group,
                RateLimitGroup {
                    id: group,
                    name: bounded.clone(),
                    sharing,
                },
            );
            GroupNameIndex::<T, I>::insert(&bounded, group);
            GroupMembers::<T, I>::insert(group, GroupMembersOf::<T, I>::new());

            let name_bytes: Vec<u8> = bounded.into();
            Self::deposit_event(Event::GroupCreated {
                group,
                name: name_bytes,
                sharing,
            });
            Ok(())
        }

        /// Updates the metadata or sharing configuration of an existing group.
        #[pallet::call_index(6)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3))]
        pub fn update_group(
            origin: OriginFor<T>,
            group: <T as Config<I>>::GroupId,
            name: Option<Vec<u8>>,
            sharing: Option<GroupSharing>,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            Groups::<T, I>::try_mutate(group, |maybe_details| -> DispatchResult {
                let details = maybe_details.as_mut().ok_or(Error::<T, I>::UnknownGroup)?;

                if let Some(new_name) = name {
                    let bounded = Self::bounded_group_name(new_name)?;
                    Self::ensure_group_name_available(&bounded, Some(group))?;
                    GroupNameIndex::<T, I>::remove(&details.name);
                    GroupNameIndex::<T, I>::insert(&bounded, group);
                    details.name = bounded;
                }

                if let Some(new_sharing) = sharing {
                    details.sharing = new_sharing;
                }

                Ok(())
            })?;

            let updated = Self::ensure_group_details(group)?;
            let name_bytes: Vec<u8> = updated.name.clone().into();
            Self::deposit_event(Event::GroupUpdated {
                group,
                name: name_bytes,
                sharing: updated.sharing,
            });

            Ok(())
        }

        /// Deletes an existing group. The group must be empty and unused.
        #[pallet::call_index(7)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3))]
        pub fn delete_group(
            origin: OriginFor<T>,
            group: <T as Config<I>>::GroupId,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            Self::ensure_group_deletable(group)?;

            let details = Groups::<T, I>::take(group).ok_or(Error::<T, I>::UnknownGroup)?;
            GroupNameIndex::<T, I>::remove(&details.name);
            GroupMembers::<T, I>::remove(group);

            Self::deposit_event(Event::GroupDeleted { group });

            Ok(())
        }

        /// Deregisters a call or removes a scoped entry from its configuration.
        #[pallet::call_index(8)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn deregister_call(
            origin: OriginFor<T>,
            transaction: TransactionIdentifier,
            scope: Option<<T as Config<I>>::LimitScope>,
            clear_usage: bool,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            Self::ensure_call_registered(&transaction)?;
            let target = Self::config_target(&transaction)?;
            let tx_target = RateLimitTarget::Transaction(transaction);
            let usage_target = Self::usage_target(&transaction)?;

            match &scope {
                Some(sc) => {
                    let mut removed = false;
                    Limits::<T, I>::mutate_exists(target, |maybe_config| {
                        if let Some(RateLimit::Scoped(map)) = maybe_config {
                            if map.remove(sc).is_some() {
                                removed = true;
                                if map.is_empty() {
                                    *maybe_config = None;
                                }
                            }
                        }
                    });
                    ensure!(removed, Error::<T, I>::MissingRateLimit);

                    if let Some(group) = CallGroups::<T, I>::take(&transaction) {
                        CallReadOnly::<T, I>::remove(&transaction);
                        Self::detach_call_from_group(&transaction, group);
                        Self::deposit_event(Event::CallGroupUpdated {
                            transaction,
                            group: None,
                        });
                    }
                }
                None => {
                    Limits::<T, I>::remove(target);
                    if target != tx_target {
                        Limits::<T, I>::remove(tx_target);
                    }

                    if let Some(group) = CallGroups::<T, I>::take(&transaction) {
                        CallReadOnly::<T, I>::remove(&transaction);
                        Self::detach_call_from_group(&transaction, group);
                        Self::deposit_event(Event::CallGroupUpdated {
                            transaction,
                            group: None,
                        });
                    }
                }
            }

            if clear_usage {
                let _ = LastSeen::<T, I>::clear_prefix(&usage_target, u32::MAX, None);
            }

            let (pallet, extrinsic) = Self::call_metadata(&transaction)?;
            Self::deposit_event(Event::CallDeregistered {
                target,
                transaction: Some(transaction),
                scope,
                pallet: Some(pallet),
                extrinsic: Some(extrinsic),
            });

            Ok(())
        }

        /// Updates whether a grouped call should skip writing usage metadata after enforcement.
        ///
        /// The call must already be assigned to a group.
        #[pallet::call_index(9)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
        pub fn set_call_read_only(
            origin: OriginFor<T>,
            transaction: TransactionIdentifier,
            read_only: bool,
        ) -> DispatchResult {
            T::AdminOrigin::ensure_origin(origin)?;

            Self::ensure_call_registered(&transaction)?;
            let group =
                CallGroups::<T, I>::get(&transaction).ok_or(Error::<T, I>::CallNotInGroup)?;
            CallReadOnly::<T, I>::insert(&transaction, read_only);

            Self::deposit_event(Event::CallReadOnlyUpdated {
                transaction,
                group,
                read_only,
            });

            Ok(())
        }
    }
}

impl<T: pallet::Config<I>, I: 'static> RateLimitingInterface for pallet::Pallet<T, I> {
    type GroupId = <T as pallet::Config<I>>::GroupId;
    type CallMetadata = <T as pallet::Config<I>>::RuntimeCall;
    type Limit = frame_system::pallet_prelude::BlockNumberFor<T>;
    type Scope = <T as pallet::Config<I>>::LimitScope;
    type UsageKey = <T as pallet::Config<I>>::UsageKey;

    fn rate_limit<TargetArg>(target: TargetArg, scope: Option<Self::Scope>) -> Option<Self::Limit>
    where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>,
    {
        let raw_target = target
            .try_into_rate_limit_target::<Self::CallMetadata>()
            .ok()?;
        let config_target = match raw_target {
            // A transaction identifier may be assigned to a group; resolve the effective storage
            // target.
            RateLimitTarget::Transaction(identifier) => Self::config_target(&identifier).ok()?,
            _ => raw_target,
        };
        Self::resolved_limit(&config_target, &scope)
    }

    fn last_seen<TargetArg>(
        target: TargetArg,
        usage_key: Option<Self::UsageKey>,
    ) -> Option<Self::Limit>
    where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>,
    {
        let raw_target = target
            .try_into_rate_limit_target::<Self::CallMetadata>()
            .ok()?;
        let usage_target = match raw_target {
            // A transaction identifier may be assigned to a group; resolve the effective storage
            // target.
            RateLimitTarget::Transaction(identifier) => Self::usage_target(&identifier).ok()?,
            _ => raw_target,
        };
        pallet::LastSeen::<T, I>::get(usage_target, usage_key)
    }

    fn set_last_seen<TargetArg>(
        target: TargetArg,
        usage_key: Option<Self::UsageKey>,
        block: Option<Self::Limit>,
    ) where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>,
    {
        let Some(raw_target) = target
            .try_into_rate_limit_target::<Self::CallMetadata>()
            .ok()
        else {
            return;
        };

        let usage_target = match raw_target {
            RateLimitTarget::Transaction(identifier) => {
                if let Ok(resolved) = Self::usage_target(&identifier) {
                    resolved
                } else {
                    return;
                }
            }
            _ => raw_target,
        };

        match block {
            Some(block) => pallet::LastSeen::<T, I>::insert(usage_target, usage_key, block),
            None => pallet::LastSeen::<T, I>::remove(usage_target, usage_key),
        }
    }
}
