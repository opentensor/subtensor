#![cfg_attr(not(feature = "std"), no_std)]

//! Interface for querying rate limits and last-seen usage, with optional write access.

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::traits::GetCallMetadata;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;
use subtensor_macros::freeze_struct;

/// Interface for rate-limiting configuration and usage tracking.
pub trait RateLimitingInterface {
    /// Group id type used by rate-limiting targets.
    type GroupId;
    /// Call type used for name/index resolution.
    type CallMetadata: GetCallMetadata;
    /// Numeric type used for returned values (commonly a block number / block span type).
    type Limit;
    /// Optional configuration scope (for example per-network `netuid`).
    type Scope;
    /// Optional usage key used to refine "last seen" tracking.
    type UsageKey;

    /// Returns the configured limit for `target` and optional `scope`.
    fn rate_limit<TargetArg>(target: TargetArg, scope: Option<Self::Scope>) -> Option<Self::Limit>
    where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>;

    /// Returns when `target` was last observed for the optional `usage_key`.
    fn last_seen<TargetArg>(
        target: TargetArg,
        usage_key: Option<Self::UsageKey>,
    ) -> Option<Self::Limit>
    where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>;

    /// Sets the last-seen block for `target` and optional `usage_key`.
    ///
    /// Passing `None` clears the value.
    fn set_last_seen<TargetArg>(
        target: TargetArg,
        usage_key: Option<Self::UsageKey>,
        block: Option<Self::Limit>,
    ) where
        TargetArg: TryIntoRateLimitTarget<Self::GroupId>;
}

/// Target identifier for rate limit and usage configuration.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
pub enum RateLimitTarget<GroupId> {
    /// Per-transaction configuration keyed by pallet/extrinsic indices.
    Transaction(TransactionIdentifier),
    /// Shared configuration for a named group.
    Group(GroupId),
}

impl<GroupId> RateLimitTarget<GroupId> {
    /// Returns the transaction identifier when the target represents a single extrinsic.
    pub fn as_transaction(&self) -> Option<&TransactionIdentifier> {
        match self {
            RateLimitTarget::Transaction(identifier) => Some(identifier),
            RateLimitTarget::Group(_) => None,
        }
    }

    /// Returns the group identifier when the target represents a group configuration.
    pub fn as_group(&self) -> Option<&GroupId> {
        match self {
            RateLimitTarget::Transaction(_) => None,
            RateLimitTarget::Group(id) => Some(id),
        }
    }
}

impl<GroupId> From<TransactionIdentifier> for RateLimitTarget<GroupId> {
    fn from(identifier: TransactionIdentifier) -> Self {
        Self::Transaction(identifier)
    }
}

/// Identifies a runtime call by pallet and extrinsic indices.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
#[freeze_struct("c865c7a9be1442a")]
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

    /// Attempts to build an identifier from a SCALE-encoded call by reading the first two bytes.
    pub fn from_call<Call: codec::Encode>(call: &Call) -> Option<Self> {
        call.using_encoded(|encoded| {
            let pallet_index = *encoded.first()?;
            let extrinsic_index = *encoded.get(1)?;
            Some(Self::new(pallet_index, extrinsic_index))
        })
    }

    /// Resolves pallet/extrinsic names for this identifier using call metadata.
    pub fn names<Call: GetCallMetadata>(&self) -> Option<(&'static str, &'static str)> {
        let modules = Call::get_module_names();
        let pallet_name = *modules.get(self.pallet_index as usize)?;
        let call_names = Call::get_call_names(pallet_name);
        let extrinsic_name = *call_names.get(self.extrinsic_index as usize)?;
        Some((pallet_name, extrinsic_name))
    }

    /// Resolves a pallet/extrinsic name pair into a transaction identifier.
    pub fn for_call_names<Call: GetCallMetadata>(
        pallet_name: &str,
        extrinsic_name: &str,
    ) -> Option<Self> {
        let modules = Call::get_module_names();
        let pallet_pos = modules.iter().position(|name| *name == pallet_name)?;
        let call_names = Call::get_call_names(pallet_name);
        let extrinsic_pos = call_names.iter().position(|name| *name == extrinsic_name)?;
        let pallet_index = u8::try_from(pallet_pos).ok()?;
        let extrinsic_index = u8::try_from(extrinsic_pos).ok()?;
        Some(Self::new(pallet_index, extrinsic_index))
    }
}

/// Conversion into a concrete [`RateLimitTarget`].
pub trait TryIntoRateLimitTarget<GroupId> {
    type Error;

    fn try_into_rate_limit_target<Call: GetCallMetadata>(
        self,
    ) -> Result<RateLimitTarget<GroupId>, Self::Error>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RateLimitTargetConversionError {
    InvalidUtf8,
    UnknownCall,
}

impl<GroupId> TryIntoRateLimitTarget<GroupId> for RateLimitTarget<GroupId> {
    type Error = core::convert::Infallible;

    fn try_into_rate_limit_target<Call: GetCallMetadata>(
        self,
    ) -> Result<RateLimitTarget<GroupId>, Self::Error> {
        Ok(self)
    }
}

impl<GroupId> TryIntoRateLimitTarget<GroupId> for GroupId {
    type Error = core::convert::Infallible;

    fn try_into_rate_limit_target<Call: GetCallMetadata>(
        self,
    ) -> Result<RateLimitTarget<GroupId>, Self::Error> {
        Ok(RateLimitTarget::Group(self))
    }
}

impl TryIntoRateLimitTarget<u32> for (Vec<u8>, Vec<u8>) {
    type Error = RateLimitTargetConversionError;

    fn try_into_rate_limit_target<Call: GetCallMetadata>(
        self,
    ) -> Result<RateLimitTarget<u32>, Self::Error> {
        let (pallet, extrinsic) = self;
        let pallet_name = sp_std::str::from_utf8(&pallet)
            .map_err(|_| RateLimitTargetConversionError::InvalidUtf8)?;
        let extrinsic_name = sp_std::str::from_utf8(&extrinsic)
            .map_err(|_| RateLimitTargetConversionError::InvalidUtf8)?;

        let identifier = TransactionIdentifier::for_call_names::<Call>(pallet_name, extrinsic_name)
            .ok_or(RateLimitTargetConversionError::UnknownCall)?;

        Ok(RateLimitTarget::Transaction(identifier))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use codec::Encode;
    use frame_support::traits::CallMetadata;

    #[derive(Clone, Copy, Debug, Encode)]
    #[freeze_struct("43380fb4d208f4cf")]
    struct DummyCall(u8, u8);

    impl GetCallMetadata for DummyCall {
        fn get_module_names() -> &'static [&'static str] {
            &["P0", "P1"]
        }

        fn get_call_names(module: &str) -> &'static [&'static str] {
            match module {
                "P0" => &["C0"],
                "P1" => &["C0", "C1", "C2", "C3", "C4"],
                _ => &[],
            }
        }

        fn get_call_metadata(&self) -> CallMetadata {
            CallMetadata {
                function_name: "unused",
                pallet_name: "unused",
            }
        }
    }

    #[test]
    fn transaction_identifier_from_call_reads_first_two_bytes() {
        let id = TransactionIdentifier::from_call(&DummyCall(1, 4)).expect("identifier");
        assert_eq!(id, TransactionIdentifier::new(1, 4));
    }

    #[test]
    fn transaction_identifier_names_resolves_metadata() {
        let id = TransactionIdentifier::new(1, 4);
        assert_eq!(id.names::<DummyCall>(), Some(("P1", "C4")));
    }

    #[test]
    fn transaction_identifier_for_call_names_resolves_indices() {
        let id = TransactionIdentifier::for_call_names::<DummyCall>("P1", "C4").expect("id");
        assert_eq!(id, TransactionIdentifier::new(1, 4));
    }

    #[test]
    fn rate_limit_target_accessors_work() {
        let tx = RateLimitTarget::<u32>::Transaction(TransactionIdentifier::new(1, 4));
        assert!(tx.as_group().is_none());
        assert_eq!(
            tx.as_transaction().copied(),
            Some(TransactionIdentifier::new(1, 4))
        );

        let group = RateLimitTarget::<u32>::Group(7);
        assert!(group.as_transaction().is_none());
        assert_eq!(group.as_group().copied(), Some(7));
    }
}
