use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::DispatchError, traits::GetCallMetadata};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::collections::btree_map::BTreeMap;

/// Resolves the optional identifier within which a rate limit applies and can optionally adjust
/// enforcement behaviour.
pub trait RateLimitScopeResolver<Origin, Call, Scope, Span> {
    /// Returns `Some(scope)` when the limit should be applied per-scope, or `None` for global
    /// limits.
    fn context(origin: &Origin, call: &Call) -> Option<Scope>;

    /// Returns `true` when the rate limit should be bypassed for the provided origin/call pair.
    /// Defaults to `false`.
    fn should_bypass(_origin: &Origin, _call: &Call) -> bool {
        false
    }

    /// Optionally adjusts the effective span used during enforcement. Defaults to the original
    /// `span`.
    fn adjust_span(_origin: &Origin, _call: &Call, span: Span) -> Span {
        span
    }
}

/// Resolves the optional usage tracking key applied when enforcing limits.
pub trait RateLimitUsageResolver<Origin, Call, Usage> {
    /// Returns `Some(usage)` when usage should be tracked per-key, or `None` for global usage
    /// tracking.
    fn context(origin: &Origin, call: &Call) -> Option<Usage>;
}

/// Identifies a runtime call by pallet and extrinsic indices.
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
pub struct TransactionIdentifier {
    /// Pallet variant index.
    pub pallet_index: u8,
    /// Call variant index within the pallet.
    pub extrinsic_index: u8,
}

impl TransactionIdentifier {
    /// Builds a new identifier from pallet/extrinsic indices.
    pub const fn new(pallet_index: u8, extrinsic_index: u8) -> Self {
        Self {
            pallet_index,
            extrinsic_index,
        }
    }

    /// Returns the pallet and extrinsic names associated with this identifier.
    pub fn names<T, I>(&self) -> Result<(&'static str, &'static str), DispatchError>
    where
        T: crate::pallet::Config<I>,
        I: 'static,
        <T as crate::pallet::Config<I>>::RuntimeCall: GetCallMetadata,
    {
        let modules = <T as crate::pallet::Config<I>>::RuntimeCall::get_module_names();
        let pallet_name = modules
            .get(self.pallet_index as usize)
            .copied()
            .ok_or(crate::pallet::Error::<T, I>::InvalidRuntimeCall)?;
        let call_names = <T as crate::pallet::Config<I>>::RuntimeCall::get_call_names(pallet_name);
        let extrinsic_name = call_names
            .get(self.extrinsic_index as usize)
            .copied()
            .ok_or(crate::pallet::Error::<T, I>::InvalidRuntimeCall)?;
        Ok((pallet_name, extrinsic_name))
    }

    /// Builds an identifier from a runtime call by extracting pallet/extrinsic indices.
    pub fn from_call<T, I>(
        call: &<T as crate::pallet::Config<I>>::RuntimeCall,
    ) -> Result<Self, DispatchError>
    where
        T: crate::pallet::Config<I>,
        I: 'static,
    {
        call.using_encoded(|encoded| {
            let pallet_index = *encoded
                .get(0)
                .ok_or(crate::pallet::Error::<T, I>::InvalidRuntimeCall)?;
            let extrinsic_index = *encoded
                .get(1)
                .ok_or(crate::pallet::Error::<T, I>::InvalidRuntimeCall)?;
            Ok(TransactionIdentifier::new(pallet_index, extrinsic_index))
        })
    }
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

#[cfg(test)]
mod tests {
    use sp_runtime::DispatchError;

    use super::*;
    use crate::{mock::*, pallet::Error};

    #[test]
    fn transaction_identifier_from_call_matches_expected_indices() {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });

        let identifier = TransactionIdentifier::from_call::<Test, ()>(&call).expect("identifier");

        // System is the first pallet in the mock runtime, RateLimiting is second.
        assert_eq!(identifier.pallet_index, 1);
        // set_default_rate_limit has call_index 2.
        assert_eq!(identifier.extrinsic_index, 3);
    }

    #[test]
    fn transaction_identifier_names_matches_call_metadata() {
        let call =
            RuntimeCall::RateLimiting(RateLimitingCall::set_default_rate_limit { block_span: 0 });
        let identifier = TransactionIdentifier::from_call::<Test, ()>(&call).expect("identifier");

        let (pallet, extrinsic) = identifier.names::<Test, ()>().expect("call metadata");
        assert_eq!(pallet, "RateLimiting");
        assert_eq!(extrinsic, "set_default_rate_limit");
    }

    #[test]
    fn transaction_identifier_names_error_for_unknown_indices() {
        let identifier = TransactionIdentifier::new(99, 0);

        let err = identifier.names::<Test, ()>().expect_err("should fail");
        let expected: DispatchError = Error::<Test, ()>::InvalidRuntimeCall.into();
        assert_eq!(err, expected);
    }
}
