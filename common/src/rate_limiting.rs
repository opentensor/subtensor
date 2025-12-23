//! Shared rate-limiting types.
//!
//! Note: `pallet-rate-limiting` supports multiple independent instances, and is intended to be used
//! as “one instance per pallet” with pallet-specific scope/usage-key types and resolvers.
//!
//! The scope/usage-key types in this module are centralized today due to the current state of
//! `pallet-subtensor` (a large, centralized pallet) and its coupling with `pallet-admin-utils`,
//! which share a single `pallet-rate-limiting` instance and resolver implementation in the runtime.
//!
//! For new pallets, it is strongly recommended to:
//! - define their own `LimitScope` and `UsageKey` types (do not extend `RateLimitUsageKey` here),
//! - provide pallet-local scope/usage resolvers,
//! - and use a dedicated `pallet-rate-limiting` instance.
//!
//! Long-term, we should move away from these shared types by refactoring `pallet-subtensor` into
//! smaller pallets with dedicated `pallet-rate-limiting` instances.

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::Parameter;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use crate::{MechId, NetUid};

/// Identifier type for rate-limiting groups.
pub type GroupId = u32;

/// Group id for serving-related calls.
pub const GROUP_SERVE: GroupId = 0;
/// Group id for delegate-take related calls.
pub const GROUP_DELEGATE_TAKE: GroupId = 1;
/// Group id for subnet weight-setting calls.
pub const GROUP_WEIGHTS_SUBNET: GroupId = 2;
/// Group id for network registration calls.
pub const GROUP_REGISTER_NETWORK: GroupId = 3;
/// Group id for owner hyperparameter calls.
pub const GROUP_OWNER_HPARAMS: GroupId = 4;
/// Group id for staking operations.
pub const GROUP_STAKING_OPS: GroupId = 5;
/// Group id for key swap calls.
pub const GROUP_SWAP_KEYS: GroupId = 6;

/// Usage-key type currently shared by the centralized `pallet-subtensor` rate-limiting instance.
///
/// Do not add new variants for new pallets. Prefer defining pallet-specific types and using a
/// dedicated `pallet-rate-limiting` instance per pallet.
#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    TypeInfo,
    MaxEncodedLen,
)]
#[scale_info(skip_type_params(AccountId))]
pub enum RateLimitUsageKey<AccountId: Parameter> {
    Account(AccountId),
    Subnet(NetUid),
    AccountSubnet {
        account: AccountId,
        netuid: NetUid,
    },
    ColdkeyHotkeySubnet {
        coldkey: AccountId,
        hotkey: AccountId,
        netuid: NetUid,
    },
    SubnetNeuron {
        netuid: NetUid,
        uid: u16,
    },
    SubnetMechanismNeuron {
        netuid: NetUid,
        mecid: MechId,
        uid: u16,
    },
    AccountSubnetServing {
        account: AccountId,
        netuid: NetUid,
        endpoint: ServingEndpoint,
    },
}

#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum ServingEndpoint {
    Axon,
    Prometheus,
}
