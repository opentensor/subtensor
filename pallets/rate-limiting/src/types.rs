use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::DispatchError, traits::GetCallMetadata};
use scale_info::TypeInfo;
use sp_std::collections::btree_map::BTreeMap;

/// Resolves the optional context within which a rate limit applies.
pub trait RateLimitContextResolver<Call, Context> {
    /// Returns `Some(context)` when the limit should be applied per-context, or `None` for global
    /// limits.
    fn context(call: &Call) -> Option<Context>;
}

/// Identifies a runtime call by pallet and extrinsic indices.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(
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
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(
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
/// of per-context spans.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "std",
    serde(
        bound = "Context: Ord + serde::Serialize + serde::de::DeserializeOwned, BlockNumber: serde::Serialize + serde::de::DeserializeOwned"
    )
)]
#[derive(Clone, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, Debug)]
pub enum RateLimit<Context, BlockNumber> {
    /// Global span applied to every invocation.
    Global(RateLimitKind<BlockNumber>),
    /// Per-context spans keyed by `Context`.
    Contextual(BTreeMap<Context, RateLimitKind<BlockNumber>>),
}

impl<Context, BlockNumber> RateLimit<Context, BlockNumber>
where
    Context: Ord,
{
    /// Convenience helper to build a global configuration.
    pub fn global(kind: RateLimitKind<BlockNumber>) -> Self {
        Self::Global(kind)
    }

    /// Convenience helper to build a contextual configuration containing a single entry.
    pub fn contextual_single(context: Context, kind: RateLimitKind<BlockNumber>) -> Self {
        let mut map = BTreeMap::new();
        map.insert(context, kind);
        Self::Contextual(map)
    }

    /// Returns the span configured for the provided context, if any.
    pub fn kind_for(&self, context: Option<&Context>) -> Option<&RateLimitKind<BlockNumber>> {
        match self {
            RateLimit::Global(kind) => Some(kind),
            RateLimit::Contextual(map) => context.and_then(|ctx| map.get(ctx)),
        }
    }

    /// Inserts or updates a contextual entry, converting from a global configuration if needed.
    pub fn upsert_context(&mut self, context: Context, kind: RateLimitKind<BlockNumber>) {
        match self {
            RateLimit::Global(_) => {
                let mut map = BTreeMap::new();
                map.insert(context, kind);
                *self = RateLimit::Contextual(map);
            }
            RateLimit::Contextual(map) => {
                map.insert(context, kind);
            }
        }
    }

    /// Removes a contextual entry, returning whether one existed.
    pub fn remove_context(&mut self, context: &Context) -> bool {
        match self {
            RateLimit::Global(_) => false,
            RateLimit::Contextual(map) => map.remove(context).is_some(),
        }
    }

    /// Returns true when the contextual configuration contains no entries.
    pub fn is_contextual_empty(&self) -> bool {
        matches!(self, RateLimit::Contextual(map) if map.is_empty())
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
        assert_eq!(identifier.extrinsic_index, 2);
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
