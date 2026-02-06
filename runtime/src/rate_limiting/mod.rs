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
use frame_support::{
    dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
    pallet_prelude::Weight,
    traits::Get,
};
use frame_system::RawOrigin;
use pallet_admin_utils::Call as AdminUtilsCall;
use pallet_rate_limiting::{
    BypassDecision, EnsureLimitSettingRule, RateLimitScopeResolver, RateLimitUsageResolver,
};
use pallet_subtensor::{Call as SubtensorCall, Tempo};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::{
    DispatchError,
    traits::{
        DispatchInfoOf, DispatchOriginOf, Dispatchable, Implication, TransactionExtension,
        ValidateResult,
    },
    transaction_validity::{TransactionSource, TransactionValidityError},
};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
use subtensor_macros::freeze_struct;
use subtensor_runtime_common::{
    BlockNumber, MechId, NetUid,
    rate_limiting::{RateLimitUsageKey, ServingEndpoint},
};

use crate::{AccountId, Runtime, RuntimeCall, RuntimeOrigin, pallet_proxy, pallet_utility};
use pallet_multisig;
use pallet_sudo;

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
    fn context(_origin: &RuntimeOrigin, call: &RuntimeCall) -> Option<BTreeSet<NetUid>> {
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
                    let mut scopes = BTreeSet::new();
                    scopes.insert(*netuid);
                    Some(scopes)
                }
                SubtensorCall::batch_set_weights { netuids, .. }
                | SubtensorCall::batch_commit_weights { netuids, .. } => {
                    let scopes: BTreeSet<NetUid> =
                        netuids.iter().map(|netuid| (*netuid).into()).collect();
                    if scopes.is_empty() {
                        None
                    } else {
                        Some(scopes)
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
    ) -> Option<BTreeSet<RateLimitUsageKey<AccountId>>> {
        match call {
            RuntimeCall::SubtensorModule(inner) => match inner {
                SubtensorCall::swap_coldkey { new_coldkey, .. } => {
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::Account(new_coldkey.clone()));
                    Some(usage)
                }
                SubtensorCall::swap_hotkey { .. } => {
                    // Enforce only by coldkey; new_hotkey last-seen is recorded in pallet-subtensor
                    // to avoid double enforcement while preserving legacy tracking.
                    let coldkey = signed_origin(origin)?;
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::Account(coldkey));
                    Some(usage)
                }
                SubtensorCall::increase_take { hotkey, .. }
                | SubtensorCall::decrease_take { hotkey, .. } => {
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::Account(hotkey.clone()));
                    Some(usage)
                }
                SubtensorCall::set_childkey_take { hotkey, netuid, .. }
                | SubtensorCall::set_children { hotkey, netuid, .. } => {
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::AccountSubnet {
                        account: hotkey.clone(),
                        netuid: *netuid,
                    });
                    Some(usage)
                }
                SubtensorCall::batch_set_weights { netuids, .. }
                | SubtensorCall::batch_commit_weights { netuids, .. } => {
                    let mut usage = BTreeSet::new();
                    for netuid in netuids {
                        let netuid: NetUid = (*netuid).into();
                        let uid = neuron_identity(origin, netuid)?;
                        usage.insert(RateLimitUsageKey::<AccountId>::SubnetMechanismNeuron {
                            netuid,
                            mecid: MechId::MAIN,
                            uid,
                        });
                    }
                    if usage.is_empty() { None } else { Some(usage) }
                }
                SubtensorCall::set_weights { netuid, .. }
                | SubtensorCall::commit_weights { netuid, .. }
                | SubtensorCall::reveal_weights { netuid, .. }
                | SubtensorCall::batch_reveal_weights { netuid, .. }
                | SubtensorCall::commit_timelocked_weights { netuid, .. } => {
                    let uid = neuron_identity(origin, *netuid)?;
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::SubnetMechanismNeuron {
                        netuid: *netuid,
                        mecid: MechId::MAIN,
                        uid,
                    });
                    Some(usage)
                }
                SubtensorCall::set_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::reveal_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_crv3_mechanism_weights { netuid, mecid, .. }
                | SubtensorCall::commit_timelocked_mechanism_weights { netuid, mecid, .. } => {
                    let uid = neuron_identity(origin, *netuid)?;
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::SubnetMechanismNeuron {
                        netuid: *netuid,
                        mecid: *mecid,
                        uid,
                    });
                    Some(usage)
                }
                SubtensorCall::serve_axon { netuid, .. }
                | SubtensorCall::serve_axon_tls { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::AccountSubnetServing {
                        account: hotkey,
                        netuid: *netuid,
                        endpoint: ServingEndpoint::Axon,
                    });
                    Some(usage)
                }
                SubtensorCall::serve_prometheus { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::AccountSubnetServing {
                        account: hotkey,
                        netuid: *netuid,
                        endpoint: ServingEndpoint::Prometheus,
                    });
                    Some(usage)
                }
                SubtensorCall::associate_evm_key { netuid, .. } => {
                    let hotkey = signed_origin(origin)?;
                    let uid = pallet_subtensor::Pallet::<Runtime>::get_uid_for_net_and_hotkey(
                        *netuid, &hotkey,
                    )
                    .ok()?;
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::SubnetNeuron {
                        netuid: *netuid,
                        uid,
                    });
                    Some(usage)
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
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::ColdkeyHotkeySubnet {
                        coldkey,
                        hotkey: hotkey.clone(),
                        netuid: *netuid,
                    });
                    Some(usage)
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
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::ColdkeyHotkeySubnet {
                        coldkey,
                        hotkey: hotkey.clone(),
                        netuid: *netuid,
                    });
                    Some(usage)
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
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::ColdkeyHotkeySubnet {
                        coldkey,
                        hotkey: hotkey.clone(),
                        netuid: *netuid,
                    });
                    Some(usage)
                }
                _ => None,
            },
            RuntimeCall::AdminUtils(inner) => {
                if let Some(netuid) = owner_hparam_netuid(inner) {
                    let mut usage = BTreeSet::new();
                    usage.insert(RateLimitUsageKey::<AccountId>::Subnet(netuid));
                    Some(usage)
                } else {
                    match inner {
                        AdminUtilsCall::sudo_set_sn_owner_hotkey { netuid, .. } => {
                            let mut usage = BTreeSet::new();
                            usage.insert(RateLimitUsageKey::<AccountId>::Subnet(*netuid));
                            Some(usage)
                        }
                        AdminUtilsCall::sudo_set_weights_version_key { netuid, .. }
                        | AdminUtilsCall::sudo_set_mechanism_count { netuid, .. }
                        | AdminUtilsCall::sudo_set_mechanism_emission_split { netuid, .. }
                        | AdminUtilsCall::sudo_trim_to_max_allowed_uids { netuid, .. } => {
                            let who = signed_origin(origin)?;
                            let mut usage = BTreeSet::new();
                            usage.insert(RateLimitUsageKey::<AccountId>::AccountSubnet {
                                account: who,
                                netuid: *netuid,
                            });
                            Some(usage)
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

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
#[freeze_struct("6f3f3ed087b897ba")]
pub struct UnwrappedRateLimitTransactionExtension(
    pallet_rate_limiting::RateLimitTransactionExtension<Runtime>,
);

impl Default for UnwrappedRateLimitTransactionExtension {
    fn default() -> Self {
        Self(pallet_rate_limiting::RateLimitTransactionExtension::<Runtime>::new())
    }
}

impl UnwrappedRateLimitTransactionExtension {
    pub fn new() -> Self {
        Self::default()
    }

    fn unwrap_nested_calls(call: &RuntimeCall) -> Vec<&RuntimeCall> {
        let mut calls = Vec::new();
        let mut stack = Vec::new();
        stack.push(call);

        while let Some(current) = stack.pop() {
            match current {
                RuntimeCall::Sudo(
                    pallet_sudo::Call::sudo { call }
                    | pallet_sudo::Call::sudo_unchecked_weight { call, .. }
                    | pallet_sudo::Call::sudo_as { call, .. },
                ) => stack.push(call),
                RuntimeCall::Proxy(
                    pallet_proxy::Call::proxy { call, .. }
                    | pallet_proxy::Call::proxy_announced { call, .. },
                ) => stack.push(call),
                RuntimeCall::Utility(inner) => match inner {
                    pallet_utility::Call::batch { calls: inner_calls }
                    | pallet_utility::Call::batch_all { calls: inner_calls }
                    | pallet_utility::Call::force_batch { calls: inner_calls } => {
                        for call in inner_calls.iter().rev() {
                            stack.push(call);
                        }
                    }
                    pallet_utility::Call::dispatch_as { call, .. }
                    | pallet_utility::Call::as_derivative { call, .. } => stack.push(call),
                    _ => calls.push(current),
                },
                RuntimeCall::Multisig(
                    pallet_multisig::Call::as_multi { call, .. }
                    | pallet_multisig::Call::as_multi_threshold_1 { call, .. },
                ) => stack.push(call),
                _ => calls.push(current),
            }
        }

        calls
    }
}

impl TransactionExtension<RuntimeCall> for UnwrappedRateLimitTransactionExtension
where
    RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    DispatchOriginOf<RuntimeCall>: Clone,
{
    const IDENTIFIER: &'static str = "RateLimitTransactionExtension";

    type Implicit = ();
    type Val = Vec<
        <pallet_rate_limiting::RateLimitTransactionExtension<Runtime> as TransactionExtension<
            RuntimeCall,
        >>::Val,
    >;
    type Pre = Vec<
        <pallet_rate_limiting::RateLimitTransactionExtension<Runtime> as TransactionExtension<
            RuntimeCall,
        >>::Pre,
    >;

    fn weight(&self, _call: &RuntimeCall) -> Weight {
        Weight::zero()
    }

    fn validate(
        &self,
        origin: DispatchOriginOf<RuntimeCall>,
        call: &RuntimeCall,
        _info: &DispatchInfoOf<RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Implication,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, RuntimeCall> {
        let inner_calls = Self::unwrap_nested_calls(call);
        let (valid, vals, origin) = self.0.validate_calls_same_block(origin, inner_calls)?;
        Ok((valid, vals, origin))
    }

    fn prepare(
        self,
        val: Self::Val,
        _origin: &DispatchOriginOf<RuntimeCall>,
        _call: &RuntimeCall,
        _info: &DispatchInfoOf<RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(val)
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<RuntimeCall>,
        post_info: &mut PostDispatchInfo,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        for entry in pre {
            pallet_rate_limiting::RateLimitTransactionExtension::<Runtime>::post_dispatch(
                entry, info, post_info, len, result,
            )?;
        }
        Ok(())
    }
}
