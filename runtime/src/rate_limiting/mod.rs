//! Runtime-level rate limiting wiring and resolvers.
//!
//! `pallet-rate-limiting` supports multiple independent instances, and is intended to be deployed
//! as “one instance per pallet” with pallet-specific scope/usage-key types and resolvers.
//!
//! This runtime module is centralized today because `pallet-subtensor` is currently centralized and
//! coupled with `pallet-admin-utils`; both share a single `pallet-rate-limiting` instance and a
//! single resolver implementation.
//!
//! For new pallets, do not reuse or extend the centralized scope/usage-key types or resolvers.
//! Prefer defining pallet-local types/resolvers and using a dedicated `pallet-rate-limiting`
//! instance.
//!
//! Long-term, we should refactor `pallet-subtensor` into smaller pallets and move to dedicated
//! `pallet-rate-limiting` instances per pallet.

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_admin_utils::Call as AdminUtilsCall;
use pallet_rate_limiting::{
    BypassDecision, EnsureLimitSettingRule, RateLimitScopeResolver, RateLimitUsageResolver,
};
use pallet_subtensor::{Call as SubtensorCall, Tempo};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;
use sp_std::{collections::btree_set::BTreeSet, vec, vec::Vec};
use subtensor_runtime_common::{
    BlockNumber, NetUid,
    rate_limiting::{RateLimitUsageKey, ServingEndpoint},
};

use crate::{AccountId, Runtime, RuntimeCall, RuntimeOrigin};

pub mod legacy;

/// Authorization rules for configuring rate limits via `pallet-rate-limiting::set_rate_limit`.
///
/// Legacy note: historically, all rate-limit setters were `Root`-only except
/// `admin-utils::sudo_set_serving_rate_limit` (subnet-owner-or-root). We preserve that behavior by
/// requiring a `scope` value when using the [`LimitSettingRule::RootOrSubnetOwnerAdminWindow`] rule
/// and validating subnet ownership against that `scope` (`netuid`).
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
pub enum LimitSettingRule {
    /// Require `Root`.
    Root,
    /// Allow `Root` or the subnet owner for the provided `netuid` scope.
    ///
    /// This rule requires `scope == Some(netuid)`.
    RootOrSubnetOwnerAdminWindow,
}

pub struct DefaultLimitSettingRule;

impl Get<LimitSettingRule> for DefaultLimitSettingRule {
    fn get() -> LimitSettingRule {
        LimitSettingRule::Root
    }
}

pub struct LimitSettingOrigin;

impl EnsureLimitSettingRule<RuntimeOrigin, LimitSettingRule, NetUid> for LimitSettingOrigin {
    fn ensure_origin(
        origin: RuntimeOrigin,
        rule: &LimitSettingRule,
        scope: &Option<NetUid>,
    ) -> frame_support::dispatch::DispatchResult {
        match rule {
            LimitSettingRule::Root => frame_system::ensure_root(origin).map_err(Into::into),
            LimitSettingRule::RootOrSubnetOwnerAdminWindow => {
                let netuid = scope.ok_or(DispatchError::BadOrigin)?;
                pallet_subtensor::Pallet::<Runtime>::ensure_admin_window_open(netuid)?;
                pallet_subtensor::Pallet::<Runtime>::ensure_subnet_owner_or_root(origin, netuid)
                    .map(|_| ())
                    .map_err(Into::into)
            }
        }
    }
}

#[derive(Default)]
pub struct ScopeResolver;

impl RateLimitScopeResolver<RuntimeOrigin, RuntimeCall, NetUid, BlockNumber> for ScopeResolver {
    fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<Vec<NetUid>> {
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
                    Some(vec![*netuid])
                }
                SubtensorCall::batch_set_weights { netuids, .. }
                | SubtensorCall::batch_commit_weights { netuids, .. } => {
                    let scopes: BTreeSet<NetUid> =
                        netuids.iter().map(|netuid| (*netuid).into()).collect();
                    if scopes.is_empty() {
                        None
                    } else {
                        Some(scopes.into_iter().collect())
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn should_bypass(origin: &RuntimeOrigin, call: &RuntimeCall) -> BypassDecision {
        if let RuntimeCall::SubtensorModule(inner) = call {
            if matches!(origin.clone().into(), Ok(RawOrigin::Root)) {
                // swap_coldkey should record last-seen but never fail; other root calls skip.
                if matches!(inner, SubtensorCall::swap_coldkey { .. }) {
                    return BypassDecision::bypass_and_record();
                }
                return BypassDecision::bypass_and_skip();
            }

            match inner {
                SubtensorCall::move_stake {
                    origin_netuid,
                    destination_netuid,
                    ..
                } if origin_netuid == destination_netuid => {
                    // Legacy: same-netuid moves enforced but did not record usage.
                    return BypassDecision::new(false, false);
                }
                SubtensorCall::set_childkey_take {
                    hotkey,
                    netuid,
                    take,
                    ..
                } => {
                    let current =
                        pallet_subtensor::Pallet::<Runtime>::get_childkey_take(hotkey, *netuid);
                    return if *take <= current {
                        BypassDecision::bypass_and_record()
                    } else {
                        BypassDecision::enforce_and_record()
                    };
                }
                SubtensorCall::add_stake { .. }
                | SubtensorCall::add_stake_limit { .. }
                | SubtensorCall::decrease_take { .. }
                | SubtensorCall::swap_coldkey { .. } => {
                    return BypassDecision::bypass_and_record();
                }
                SubtensorCall::reveal_weights { netuid, .. }
                | SubtensorCall::batch_reveal_weights { netuid, .. }
                | SubtensorCall::reveal_mechanism_weights { netuid, .. } => {
                    if pallet_subtensor::Pallet::<Runtime>::get_commit_reveal_weights_enabled(
                        *netuid,
                    ) {
                        // Legacy: reveals are not rate-limited while commit-reveal is enabled.
                        return BypassDecision::bypass_and_skip();
                    }
                }
                _ => {}
            }
        }

        BypassDecision::enforce_and_record()
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
                } else if let AdminUtilsCall::sudo_set_weights_version_key { netuid, .. } = inner {
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
    fn context(
        origin: &RuntimeOrigin,
        call: &RuntimeCall,
    ) -> Option<Vec<RateLimitUsageKey<AccountId>>> {
        match call {
            RuntimeCall::SubtensorModule(inner) => match inner {
                SubtensorCall::swap_coldkey { new_coldkey, .. } => {
                    Some(vec![RateLimitUsageKey::<AccountId>::Account(
                        new_coldkey.clone(),
                    )])
                }
                SubtensorCall::swap_hotkey { new_hotkey, .. } => {
                    // Record against the coldkey (enforcement) and the new hotkey to mirror legacy
                    // writes.
                    let coldkey = signed_origin(origin)?;
                    Some(vec![
                        RateLimitUsageKey::<AccountId>::Account(coldkey),
                        RateLimitUsageKey::<AccountId>::Account(new_hotkey),
                    ])
                }
                SubtensorCall::increase_take { hotkey, .. }
                | SubtensorCall::decrease_take { hotkey, .. } => {
                    Some(vec![RateLimitUsageKey::<AccountId>::Account(
                        hotkey.clone(),
                    )])
                }
                SubtensorCall::set_childkey_take { hotkey, netuid, .. }
                | SubtensorCall::set_children { hotkey, netuid, .. } => {
                    Some(vec![RateLimitUsageKey::<AccountId>::AccountSubnet {
                        account: hotkey.clone(),
                        netuid: *netuid,
                    }])
                }
                SubtensorCall::batch_set_weights { netuids, .. }
                | SubtensorCall::batch_commit_weights { netuids, .. } => {
                    let mut usage = BTreeSet::new();
                    for netuid in netuids {
                        let netuid: NetUid = (*netuid).into();
                        let uid = neuron_identity(origin, netuid)?;
                        usage.insert(RateLimitUsageKey::<AccountId>::SubnetNeuron { netuid, uid });
                    }
                    if usage.is_empty() {
                        None
                    } else {
                        Some(usage.into_iter().collect())
                    }
                }
                SubtensorCall::set_weights { netuid, .. }
                | SubtensorCall::commit_weights { netuid, .. }
                | SubtensorCall::reveal_weights { netuid, .. }
                | SubtensorCall::batch_reveal_weights { netuid, .. }
                | SubtensorCall::commit_timelocked_weights { netuid, .. } => {
                    let uid = neuron_identity(origin, *netuid)?;
                    Some(vec![RateLimitUsageKey::<AccountId>::SubnetNeuron {
                        netuid: *netuid,
                        uid,
                    }])
                }
                // legacy implementation still used netuid only, but it was recalculating it using
                // mecid, so switching to netuid AND mecid is logical here
                SubtensorCall::set_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::reveal_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_crv3_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_timelocked_mechanism_weights { netuid, mecid, .. } => {
                    let uid = neuron_identity(origin, *netuid)?;
                    Some(vec![
                        RateLimitUsageKey::<AccountId>::SubnetMechanismNeuron {
                            netuid: *netuid,
                            mecid: *mecid,
                            uid,
                        },
                    ])
                }
                SubtensorCall::serve_axon { netuid, .. }
                | SubtensorCall::serve_axon_tls { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    Some(vec![RateLimitUsageKey::<AccountId>::AccountSubnetServing {
                        account: hotkey,
                        netuid: *netuid,
                        endpoint: ServingEndpoint::Axon,
                    }])
                }
                SubtensorCall::serve_prometheus { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    Some(vec![RateLimitUsageKey::<AccountId>::AccountSubnetServing {
                        account: hotkey,
                        netuid: *netuid,
                        endpoint: ServingEndpoint::Prometheus,
                    }])
                }
                SubtensorCall::associate_evm_key { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    let uid = pallet_subtensor::Pallet::<Runtime>::get_uid_for_net_and_hotkey(
                        *netuid, &hotkey,
                    )
                    .ok()?;
                    Some(vec![RateLimitUsageKey::<AccountId>::SubnetNeuron {
                        netuid: *netuid,
                        uid,
                    }])
                }
                // Staking calls share a group lock; only add_* write usage, the rest are read-only.
                // Keep the usage key granular so the lock applies per (coldkey, hotkey, netuid).
                SubtensorCall::add_stake { hotkey, netuid, .. }
                | SubtensorCall::add_stake_limit { hotkey, netuid, .. }
                | SubtensorCall::remove_stake { hotkey, netuid, .. }
                | SubtensorCall::remove_stake_limit { hotkey, netuid, .. }
                | SubtensorCall::remove_stake_full_limit { hotkey, netuid, .. }
                | SubtensorCall::transfer_stake {
                    hotkey,
                    origin_netuid: netuid,
                    ..
                } => {
                    let coldkey = signed_origin(origin)?;
                    Some(vec![RateLimitUsageKey::<AccountId>::ColdkeyHotkeySubnet {
                        coldkey,
                        hotkey: hotkey.clone(),
                        netuid: *netuid,
                    }])
                }
                SubtensorCall::swap_stake {
                    hotkey,
                    destination_netuid: netuid,
                    ..
                }
                | SubtensorCall::swap_stake_limit {
                    hotkey,
                    destination_netuid: netuid,
                    ..
                } => {
                    let coldkey = signed_origin(origin)?;
                    Some(vec![RateLimitUsageKey::<AccountId>::ColdkeyHotkeySubnet {
                        coldkey,
                        hotkey: hotkey.clone(),
                        netuid: *netuid,
                    }])
                }
                SubtensorCall::move_stake {
                    origin_hotkey,
                    destination_hotkey,
                    origin_netuid,
                    destination_netuid,
                    ..
                } => {
                    let coldkey = signed_origin(origin)?;
                    let (hotkey, netuid) = if origin_netuid == destination_netuid {
                        (origin_hotkey, origin_netuid)
                    } else {
                        (destination_hotkey, destination_netuid)
                    };
                    Some(vec![RateLimitUsageKey::<AccountId>::ColdkeyHotkeySubnet {
                        coldkey,
                        hotkey: hotkey.clone(),
                        netuid: *netuid,
                    }])
                }
                _ => None,
            },
            RuntimeCall::AdminUtils(inner) => {
                if let Some(netuid) = owner_hparam_netuid(inner) {
                    Some(vec![RateLimitUsageKey::<AccountId>::Subnet(netuid)])
                } else {
                    match inner {
                        AdminUtilsCall::sudo_set_sn_owner_hotkey { netuid, .. } => {
                            Some(vec![RateLimitUsageKey::<AccountId>::Subnet(*netuid)])
                        }
                        AdminUtilsCall::sudo_set_weights_version_key { netuid, .. }
                        | AdminUtilsCall::sudo_set_mechanism_count { netuid, .. }
                        | AdminUtilsCall::sudo_set_mechanism_emission_split { netuid, .. }
                        | AdminUtilsCall::sudo_trim_to_max_allowed_uids { netuid, .. } => {
                            let who = signed_origin(origin)?;
                            Some(vec![RateLimitUsageKey::<AccountId>::AccountSubnet {
                                account: who,
                                netuid: *netuid,
                            }])
                        }
                        _ => None,
                    }
                }
            }
            _ => None,
        }
    }
}

fn neuron_identity(origin: &RuntimeOrigin, netuid: NetUid) -> Option<u16> {
    let hotkey = signed_origin(origin)?;
    let uid =
        pallet_subtensor::Pallet::<Runtime>::get_uid_for_net_and_hotkey(netuid, &hotkey).ok()?;
    Some(uid)
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
        | AdminUtilsCall::sudo_set_yuma3_enabled { netuid, .. } => Some(*netuid),
        _ => None,
    }
}
