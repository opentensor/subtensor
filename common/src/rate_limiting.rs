use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::Parameter;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

use crate::{MechId, NetUid};

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
pub enum RateLimitScope {
    Subnet(NetUid),
    SubnetMechanism { netuid: NetUid, mecid: MechId },
}

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
}
