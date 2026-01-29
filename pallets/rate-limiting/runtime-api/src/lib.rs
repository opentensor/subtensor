#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use pallet_rate_limiting::RateLimitKind;
use scale_info::TypeInfo;
use sp_std::vec::Vec;
use subtensor_runtime_common::BlockNumber;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct RateLimitRpcResponse {
    pub global: Option<RateLimitKind<BlockNumber>>,
    pub contextual: Vec<(Vec<u8>, RateLimitKind<BlockNumber>)>,
    pub default_limit: BlockNumber,
    pub resolved: Option<BlockNumber>,
}

sp_api::decl_runtime_apis! {
    pub trait RateLimitingRuntimeApi {
        fn get_rate_limit(pallet: Vec<u8>, extrinsic: Vec<u8>) -> Option<RateLimitRpcResponse>;
    }
}
