use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
pub use rate_limiting_interface::{RateLimitTarget, TransactionIdentifier};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::{collections::btree_map::BTreeMap, collections::btree_set::BTreeSet};

/// Resolves the optional identifier within which a rate limit applies and can optionally adjust
/// enforcement behaviour.
pub trait RateLimitScopeResolver<Origin, Call, Scope, Span> {
    /// Returns `Some(scopes)` when the limit should be applied per-scope, or `None` for global
    /// limits.
    fn context(origin: &Origin, call: &Call) -> Option<BTreeSet<Scope>>;

    /// Returns how the call should interact with enforcement and usage tracking.
    fn should_bypass(_origin: &Origin, _call: &Call) -> BypassDecision {
        BypassDecision::enforce_and_record()
    }

    /// Optionally adjusts the effective span used during enforcement. Defaults to the original
    /// `span`.
    fn adjust_span(_origin: &Origin, _call: &Call, span: Span) -> Span {
        span
    }
}

/// Controls whether enforcement should run and whether usage should be recorded for a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BypassDecision {
    pub bypass_enforcement: bool,
    pub record_usage: bool,
}

impl BypassDecision {
    pub const fn new(bypass_enforcement: bool, record_usage: bool) -> Self {
        Self {
            bypass_enforcement,
            record_usage,
        }
    }

    pub const fn enforce_and_record() -> Self {
        Self::new(false, true)
    }

    pub const fn bypass_and_record() -> Self {
        Self::new(true, true)
    }

    pub const fn bypass_and_skip() -> Self {
        Self::new(true, false)
    }
}

/// Resolves the optional usage tracking key applied when enforcing limits.
pub trait RateLimitUsageResolver<Origin, Call, Usage> {
    /// Returns `Some(keys)` to track usage per key, or `None` for global usage tracking.
    ///
    /// When multiple keys are returned, the rate limit is enforced against each key and all are
    /// recorded on success.
    fn context(origin: &Origin, call: &Call) -> Option<BTreeSet<Usage>>;
}

/// Origin check performed when configuring a rate limit.
///
/// `pallet-rate-limiting` supports configuring a distinct "who may set limits" rule per call/group
/// target. This trait is invoked by [`pallet::Pallet::set_rate_limit`] after loading the rule from
/// storage, allowing runtimes to implement arbitrary permissioning logic.
///
/// Note: the hook receives the provided `scope` (if any). Some policies (for example "subnet owner")
/// require a scope value (such as `netuid`) in order to validate the caller.
pub trait EnsureLimitSettingRule<Origin, Rule, Scope> {
    fn ensure_origin(origin: Origin, rule: &Rule, scope: &Option<Scope>) -> DispatchResult;
}

/// Sharing mode configured for a group.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
pub enum GroupSharing {
    /// Limits remain per transaction; usage is shared by the group.
    UsageOnly,
    /// Limits are shared by the group; usage remains per transaction.
    ConfigOnly,
    /// Both limits and usage are shared by the group.
    ConfigAndUsage,
}

impl GroupSharing {
    /// Returns `true` when configuration for this group should use the group target key.
    pub fn config_uses_group(self) -> bool {
        matches!(
            self,
            GroupSharing::ConfigOnly | GroupSharing::ConfigAndUsage
        )
    }

    /// Returns `true` when usage tracking for this group should use the group target key.
    pub fn usage_uses_group(self) -> bool {
        matches!(self, GroupSharing::UsageOnly | GroupSharing::ConfigAndUsage)
    }
}

/// Metadata describing a configured group.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
pub struct RateLimitGroup<GroupId, Name> {
    /// Stable identifier assigned to the group.
    pub id: GroupId,
    /// Human readable group name.
    pub name: Name,
    /// Sharing configuration enforced for the group.
    pub sharing: GroupSharing,
}

/// Policy describing the block span enforced by a rate limit.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
pub enum RateLimitKind<BlockNumber> {
    /// Use the pallet-level default rate limit.
    Default,
    /// Apply an exact rate limit measured in blocks.
    Exact(BlockNumber),
}

/// Stored rate limit configuration for a transaction identifier.
///
/// The configuration is mutually exclusive: either the call is globally limited or it stores a set
/// of per-scope spans.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    Debug,
)]
#[serde(
    bound = "Scope: Ord + serde::Serialize + serde::de::DeserializeOwned, BlockNumber: serde::Serialize + serde::de::DeserializeOwned"
)]
pub enum RateLimit<Scope, BlockNumber> {
    /// Global span applied to every invocation.
    Global(RateLimitKind<BlockNumber>),
    /// Per-scope spans keyed by `Scope`.
    Scoped(BTreeMap<Scope, RateLimitKind<BlockNumber>>),
}

impl<Scope, BlockNumber> RateLimit<Scope, BlockNumber>
where
    Scope: Ord,
{
    /// Convenience helper to build a global configuration.
    pub fn global(kind: RateLimitKind<BlockNumber>) -> Self {
        Self::Global(kind)
    }

    /// Convenience helper to build a scoped configuration containing a single entry.
    pub fn scoped_single(scope: Scope, kind: RateLimitKind<BlockNumber>) -> Self {
        let mut map = BTreeMap::new();
        map.insert(scope, kind);
        Self::Scoped(map)
    }

    /// Returns the span configured for the provided scope, if any.
    pub fn kind_for(&self, scope: Option<&Scope>) -> Option<&RateLimitKind<BlockNumber>> {
        match self {
            RateLimit::Global(kind) => Some(kind),
            RateLimit::Scoped(map) => scope.and_then(|key| map.get(key)),
        }
    }

    /// Inserts or updates a scoped entry, converting from a global configuration if needed.
    pub fn upsert_scope(&mut self, scope: Scope, kind: RateLimitKind<BlockNumber>) {
        match self {
            RateLimit::Global(_) => {
                let mut map = BTreeMap::new();
                map.insert(scope, kind);
                *self = RateLimit::Scoped(map);
            }
            RateLimit::Scoped(map) => {
                map.insert(scope, kind);
            }
        }
    }

    /// Removes a scoped entry, returning whether one existed.
    pub fn remove_scope(&mut self, scope: &Scope) -> bool {
        match self {
            RateLimit::Global(_) => false,
            RateLimit::Scoped(map) => map.remove(scope).is_some(),
        }
    }

    /// Returns true when the scoped configuration contains no entries.
    pub fn is_scoped_empty(&self) -> bool {
        matches!(self, RateLimit::Scoped(map) if map.is_empty())
    }
}
