use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::Parameter;
use frame_system::RawOrigin;
use pallet_admin_utils::Call as AdminUtilsCall;
use pallet_rate_limiting::{RateLimitScopeResolver, RateLimitUsageResolver};
use pallet_subtensor::{Call as SubtensorCall, Tempo};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use subtensor_runtime_common::{BlockNumber, MechId, NetUid};

use crate::{AccountId, Runtime, RuntimeCall, RuntimeOrigin};

pub mod migration;

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

#[derive(Default)]
pub struct ScopeResolver;

impl RateLimitScopeResolver<RuntimeOrigin, RuntimeCall, NetUid, BlockNumber> for ScopeResolver {
    fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<NetUid> {
        match call {
            RuntimeCall::SubtensorModule(inner) => match inner {
                SubtensorCall::serve_axon { netuid, .. }
                | SubtensorCall::serve_axon_tls { netuid, .. }
                | SubtensorCall::serve_prometheus { netuid, .. }
                | SubtensorCall::set_weights { netuid, .. }
                | SubtensorCall::commit_weights { netuid, .. }
                | SubtensorCall::reveal_weights { netuid, .. }
                | SubtensorCall::batch_reveal_weights { netuid, .. }
                | SubtensorCall::commit_timelocked_weights { netuid, .. }
                | SubtensorCall::set_mechanism_weights { netuid, .. }
                | SubtensorCall::commit_mechanism_weights { netuid, .. }
                | SubtensorCall::reveal_mechanism_weights { netuid, .. }
                | SubtensorCall::commit_crv3_mechanism_weights { netuid, .. }
                | SubtensorCall::commit_timelocked_mechanism_weights { netuid, .. } => {
                    Some(*netuid)
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn should_bypass(origin: &RuntimeOrigin, _call: &RuntimeCall) -> bool {
        matches!(origin.clone().into(), Ok(RawOrigin::Root))
    }

    fn adjust_span(_origin: &RuntimeOrigin, call: &RuntimeCall, span: BlockNumber) -> BlockNumber {
        match call {
            RuntimeCall::AdminUtils(inner) => {
                if let Some(netuid) = owner_hparam_netuid(inner) {
                    if span == 0 {
                        return span;
                    }
                    let tempo = BlockNumber::from(Tempo::<Runtime>::get(netuid) as u32);
                    span.saturating_mul(tempo)
                } else {
                    span
                }
            }
            _ => span,
        }
    }
}

#[derive(Default)]
pub struct UsageResolver;

impl RateLimitUsageResolver<RuntimeOrigin, RuntimeCall, RateLimitUsageKey<AccountId>>
    for UsageResolver
{
    fn context(origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<RateLimitUsageKey<AccountId>> {
        match call {
            RuntimeCall::SubtensorModule(inner) => match inner {
                SubtensorCall::swap_hotkey { .. } => {
                    signed_origin(origin).map(RateLimitUsageKey::<AccountId>::Account)
                }
                SubtensorCall::increase_take { hotkey, .. } => {
                    Some(RateLimitUsageKey::<AccountId>::Account(hotkey.clone()))
                }
                SubtensorCall::set_childkey_take { hotkey, netuid, .. }
                | SubtensorCall::set_children { hotkey, netuid, .. } => {
                    Some(RateLimitUsageKey::<AccountId>::AccountSubnet {
                        account: hotkey.clone(),
                        netuid: *netuid,
                    })
                }
                SubtensorCall::set_weights { netuid, .. }
                | SubtensorCall::commit_weights { netuid, .. }
                | SubtensorCall::reveal_weights { netuid, .. }
                | SubtensorCall::batch_reveal_weights { netuid, .. }
                | SubtensorCall::commit_timelocked_weights { netuid, .. } => {
                    let (_, uid) = neuron_identity(origin, *netuid)?;
                    Some(RateLimitUsageKey::<AccountId>::SubnetNeuron {
                        netuid: *netuid,
                        uid,
                    })
                }
                // legacy implementation still used netuid only, but it was recalculating it using
                // mecid, so switching to netuid AND mecid is logical here
                SubtensorCall::set_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::reveal_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_crv3_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_timelocked_mechanism_weights { netuid, mecid, .. } => {
                    let (_, uid) = neuron_identity(origin, *netuid)?;
                    Some(RateLimitUsageKey::<AccountId>::SubnetMechanismNeuron {
                        netuid: *netuid,
                        mecid: *mecid,
                        uid,
                    })
                }
                SubtensorCall::serve_axon { netuid, .. }
                | SubtensorCall::serve_axon_tls { netuid, .. }
                | SubtensorCall::serve_prometheus { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    Some(RateLimitUsageKey::<AccountId>::AccountSubnet {
                        account: hotkey,
                        netuid: *netuid,
                    })
                }
                SubtensorCall::associate_evm_key { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    let uid = pallet_subtensor::Pallet::<Runtime>::get_uid_for_net_and_hotkey(
                        *netuid, &hotkey,
                    )
                    .ok()?;
                    Some(RateLimitUsageKey::<AccountId>::SubnetNeuron {
                        netuid: *netuid,
                        uid,
                    })
                }
                SubtensorCall::add_stake { hotkey, netuid, .. }
                | SubtensorCall::add_stake_limit { hotkey, netuid, .. }
                | SubtensorCall::remove_stake { hotkey, netuid, .. }
                | SubtensorCall::remove_stake_limit { hotkey, netuid, .. }
                | SubtensorCall::remove_stake_full_limit { hotkey, netuid, .. }
                | SubtensorCall::transfer_stake {
                    hotkey,
                    origin_netuid: netuid,
                    ..
                }
                | SubtensorCall::swap_stake {
                    hotkey,
                    origin_netuid: netuid,
                    ..
                }
                | SubtensorCall::swap_stake_limit {
                    hotkey,
                    origin_netuid: netuid,
                    ..
                }
                | SubtensorCall::move_stake {
                    origin_hotkey: hotkey,
                    origin_netuid: netuid,
                    ..
                }
                | SubtensorCall::recycle_alpha { hotkey, netuid, .. }
                | SubtensorCall::burn_alpha { hotkey, netuid, .. } => {
                    let coldkey = signed_origin(origin)?;
                    Some(RateLimitUsageKey::<AccountId>::ColdkeyHotkeySubnet {
                        coldkey,
                        hotkey: hotkey.clone(),
                        netuid: *netuid,
                    })
                }
                _ => None,
            },
            RuntimeCall::AdminUtils(inner) => {
                if let Some(netuid) = owner_hparam_netuid(inner) {
                    // Hyperparameter setters share a global span but are tracked per subnet.
                    Some(RateLimitUsageKey::<AccountId>::Subnet(netuid))
                } else {
                    match inner {
                        AdminUtilsCall::sudo_set_sn_owner_hotkey { netuid, .. } => {
                            Some(RateLimitUsageKey::<AccountId>::Subnet(*netuid))
                        }
                        AdminUtilsCall::sudo_set_mechanism_count { netuid, .. }
                        | AdminUtilsCall::sudo_set_mechanism_emission_split { netuid, .. }
                        | AdminUtilsCall::sudo_trim_to_max_allowed_uids { netuid, .. } => {
                            let who = signed_origin(origin)?;
                            Some(RateLimitUsageKey::<AccountId>::AccountSubnet {
                                account: who,
                                netuid: *netuid,
                            })
                        }
                        _ => None,
                    }
                }
            }
            _ => None,
        }
    }
}

fn neuron_identity(origin: &RuntimeOrigin, netuid: NetUid) -> Option<(AccountId, u16)> {
    let hotkey = signed_origin(origin)?;
    let uid =
        pallet_subtensor::Pallet::<Runtime>::get_uid_for_net_and_hotkey(netuid, &hotkey).ok()?;
    Some((hotkey, uid))
}

fn signed_origin(origin: &RuntimeOrigin) -> Option<AccountId> {
    match origin.clone().into() {
        Ok(RawOrigin::Signed(who)) => Some(who),
        _ => None,
    }
}

fn owner_hparam_netuid(call: &AdminUtilsCall<Runtime>) -> Option<NetUid> {
    match call {
        AdminUtilsCall::sudo_set_activity_cutoff { netuid, .. }
        | AdminUtilsCall::sudo_set_adjustment_alpha { netuid, .. }
        | AdminUtilsCall::sudo_set_alpha_sigmoid_steepness { netuid, .. }
        | AdminUtilsCall::sudo_set_alpha_values { netuid, .. }
        | AdminUtilsCall::sudo_set_bonds_moving_average { netuid, .. }
        | AdminUtilsCall::sudo_set_bonds_penalty { netuid, .. }
        | AdminUtilsCall::sudo_set_bonds_reset_enabled { netuid, .. }
        | AdminUtilsCall::sudo_set_commit_reveal_weights_enabled { netuid, .. }
        | AdminUtilsCall::sudo_set_commit_reveal_weights_interval { netuid, .. }
        | AdminUtilsCall::sudo_set_immunity_period { netuid, .. }
        | AdminUtilsCall::sudo_set_liquid_alpha_enabled { netuid, .. }
        | AdminUtilsCall::sudo_set_max_allowed_uids { netuid, .. }
        | AdminUtilsCall::sudo_set_max_burn { netuid, .. }
        | AdminUtilsCall::sudo_set_max_difficulty { netuid, .. }
        | AdminUtilsCall::sudo_set_min_allowed_weights { netuid, .. }
        | AdminUtilsCall::sudo_set_min_burn { netuid, .. }
        | AdminUtilsCall::sudo_set_network_pow_registration_allowed { netuid, .. }
        | AdminUtilsCall::sudo_set_owner_immune_neuron_limit { netuid, .. }
        | AdminUtilsCall::sudo_set_recycle_or_burn { netuid, .. }
        | AdminUtilsCall::sudo_set_rho { netuid, .. }
        | AdminUtilsCall::sudo_set_serving_rate_limit { netuid, .. }
        | AdminUtilsCall::sudo_set_toggle_transfer { netuid, .. }
        | AdminUtilsCall::sudo_set_weights_version_key { netuid, .. }
        | AdminUtilsCall::sudo_set_yuma3_enabled { netuid, .. } => Some(*netuid),
        _ => None,
    }
}
