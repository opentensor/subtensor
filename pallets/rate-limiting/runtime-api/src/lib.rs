#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use pallet_rate_limiting::RateLimitKind;
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use subtensor_runtime_common::{BlockNumber, rate_limiting::GroupId};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
/// Rate-limit configuration for a target.
pub enum RateLimitConfigRpcResponse {
    /// Global (scope-independent) limit configuration.
    Global(RateLimitKind<BlockNumber>),
    /// Per-scope limit configuration.
    ///
    /// Keys are SCALE-encoded scope bytes as stored by the pallet.
    Scoped(Vec<(Vec<u8>, RateLimitKind<BlockNumber>)>),
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
/// RPC response for `get_rate_limit`.
///
/// Returns the configured limit for the call and includes group id when the call is assigned to a
/// group.
pub enum RateLimitRpcResponse {
    /// Call has no group assignment.
    Standalone { limit: RateLimitConfigRpcResponse },
    /// Call is assigned to a group.
    Grouped {
        group_id: GroupId,
        limit: RateLimitConfigRpcResponse,
    },
}

sp_api::decl_runtime_apis! {
    pub trait RateLimitingRuntimeApi {
        fn get_rate_limit(pallet: Vec<u8>, extrinsic: Vec<u8>) -> Option<RateLimitRpcResponse>;
    }
}
