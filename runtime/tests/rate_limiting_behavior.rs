#![allow(clippy::unwrap_used)]

use frame_support::traits::OnRuntimeUpgrade;
use frame_system::pallet_prelude::BlockNumberFor;
use node_subtensor_runtime::{
    BuildStorage, Runtime, RuntimeCall, RuntimeGenesisConfig, RuntimeOrigin, RuntimeScopeResolver,
    RuntimeUsageResolver, SubtensorModule, System, rate_limiting::migration::Migration,
};
use pallet_rate_limiting::{RateLimitScopeResolver, RateLimitUsageResolver};
use pallet_rate_limiting::{RateLimitTarget, TransactionIdentifier};
use pallet_subtensor::Call as SubtensorCall;
use pallet_subtensor::{
    AxonInfo, HasMigrationRun, LastRateLimitedBlock, LastUpdate, NetworksAdded, PrometheusInfo,
    RateLimitKey, ServingRateLimit, TransactionKeyLastBlock, WeightsSetRateLimit,
    WeightsVersionKeyRateLimit, utils::rate_limiting::TransactionType,
};
use sp_core::{H160, ecdsa};
use sp_runtime::traits::SaturatedConversion;
use subtensor_runtime_common::{
    NetUid, NetUidStorageIndex,
    rate_limiting::{GroupId, RateLimitUsageKey},
};

type AccountId = <Runtime as frame_system::Config>::AccountId;
type UsageKey = RateLimitUsageKey<AccountId>;

const MIGRATION_NAME: &[u8] = b"migrate_rate_limiting";

fn new_ext() -> sp_io::TestExternalities {
    sp_tracing::try_init_simple();
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig::default()
        .build_storage()
        .unwrap()
        .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn account(n: u8) -> AccountId {
    AccountId::from([n; 32])
}

fn resolve_target(identifier: TransactionIdentifier) -> RateLimitTarget<GroupId> {
    if let Some(group) = pallet_rate_limiting::CallGroups::<Runtime>::get(identifier) {
        RateLimitTarget::Group(group)
    } else {
        RateLimitTarget::Transaction(identifier)
    }
}

fn exact_span(span: u64) -> BlockNumberFor<Runtime> {
    span.saturated_into::<BlockNumberFor<Runtime>>()
}

fn clear_rate_limiting_storage() {
    let limit = u32::MAX;
    let _ = pallet_rate_limiting::Limits::<Runtime>::clear(limit, None);
    let _ = pallet_rate_limiting::LastSeen::<Runtime>::clear(limit, None);
    let _ = pallet_rate_limiting::Groups::<Runtime>::clear(limit, None);
    let _ = pallet_rate_limiting::GroupMembers::<Runtime>::clear(limit, None);
    let _ = pallet_rate_limiting::GroupNameIndex::<Runtime>::clear(limit, None);
    let _ = pallet_rate_limiting::CallGroups::<Runtime>::clear(limit, None);
    pallet_rate_limiting::NextGroupId::<Runtime>::kill();
}

fn parity_check<F>(
    now: u64,
    call: RuntimeCall,
    origin: RuntimeOrigin,
    usage_override: Option<Vec<UsageKey>>,
    scope_override: Option<NetUid>,
    legacy_check: F,
) where
    F: Fn() -> bool,
{
    System::set_block_number(now.saturated_into());
    HasMigrationRun::<Runtime>::remove(MIGRATION_NAME);
    clear_rate_limiting_storage();

    // Run migration to hydrate pallet-rate-limiting state.
    Migration::<Runtime>::on_runtime_upgrade();

    let identifier = TransactionIdentifier::from_call(&call).expect("identifier for call");
    let scope = scope_override.or_else(|| RuntimeScopeResolver::context(&origin, &call));
    let usage: Option<Vec<<Runtime as pallet_rate_limiting::Config>::UsageKey>> =
        usage_override.or_else(|| RuntimeUsageResolver::context(&origin, &call));
    let target = resolve_target(identifier);

    // Use the runtime-adjusted span (handles tempo scaling for admin-utils).
    let span = pallet_rate_limiting::Pallet::<Runtime>::effective_span(
        &origin.clone().into(),
        &call,
        &target,
        &scope,
    )
    .unwrap_or_default();
    let span_u64: u64 = span.saturated_into();

    let usage_keys: Vec<Option<<Runtime as pallet_rate_limiting::Config>::UsageKey>> = match usage {
        None => vec![None],
        Some(keys) => keys.into_iter().map(Some).collect(),
    };

    let within = usage_keys.iter().all(|key| {
        pallet_rate_limiting::Pallet::<Runtime>::is_within_limit(
            &origin.clone().into(),
            &call,
            &identifier,
            &scope,
            key,
        )
        .expect("pallet rate limit result")
    });
    assert_eq!(within, legacy_check(), "parity at now for {:?}", identifier);

    // Advance beyond the span and re-check (span==0 treated as allow).
    let advance: BlockNumberFor<Runtime> = span.saturating_add(exact_span(1));
    System::set_block_number(System::block_number().saturating_add(advance));

    let within_after = usage_keys.iter().all(|key| {
        pallet_rate_limiting::Pallet::<Runtime>::is_within_limit(
            &origin.clone().into(),
            &call,
            &identifier,
            &scope,
            key,
        )
        .expect("pallet rate limit result (after)")
    });
    assert!(
        within_after || span_u64 == 0,
        "parity after window for {:?}",
        identifier
    );
}

#[test]
fn register_network_parity() {
    new_ext().execute_with(|| {
        let now = 100u64;
        let cold = account(1);
        let hot = account(2);
        let span = 5u64;
        LastRateLimitedBlock::<Runtime>::insert(RateLimitKey::NetworkLastRegistered, now - 1);
        pallet_subtensor::NetworkRateLimit::<Runtime>::put(span);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::register_network { hotkey: hot });
        let origin = RuntimeOrigin::signed(cold.clone());
        let legacy = || TransactionType::RegisterNetwork.passes_rate_limit::<Runtime>(&cold);
        parity_check(now, call, origin, None, None, legacy);
    });
}

#[test]
fn swap_hotkey_parity() {
    new_ext().execute_with(|| {
        let now = 200u64;
        let cold = account(10);
        let old_hot = account(11);
        let new_hot = account(12);
        let span = 10u64;
        LastRateLimitedBlock::<Runtime>::insert(RateLimitKey::LastTxBlock(cold.clone()), now - 1);
        pallet_subtensor::TxRateLimit::<Runtime>::put(span);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::swap_hotkey {
            hotkey: old_hot,
            new_hotkey: new_hot,
            netuid: None,
        });
        let origin = RuntimeOrigin::signed(cold.clone());
        let legacy = || !SubtensorModule::exceeds_tx_rate_limit(now - 1, now);
        parity_check(now, call, origin, None, None, legacy);
    });
}

#[test]
fn increase_take_parity() {
    new_ext().execute_with(|| {
        let now = 300u64;
        let hot = account(20);
        let span = 3u64;
        LastRateLimitedBlock::<Runtime>::insert(
            RateLimitKey::LastTxBlockDelegateTake(hot.clone()),
            now - 1,
        );
        pallet_subtensor::TxDelegateTakeRateLimit::<Runtime>::put(span);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::increase_take {
            hotkey: hot.clone(),
            take: 5,
        });
        let origin = RuntimeOrigin::signed(account(21));
        let legacy = || !SubtensorModule::exceeds_tx_delegate_take_rate_limit(now - 1, now);
        parity_check(now, call, origin, None, None, legacy);
    });
}

#[test]
fn set_childkey_take_parity() {
    new_ext().execute_with(|| {
        let now = 400u64;
        let hot = account(30);
        let netuid = NetUid::from(1u16);
        let span = 7u64;
        let tx_kind: u16 = TransactionType::SetChildkeyTake.into();
        TransactionKeyLastBlock::<Runtime>::insert((hot.clone(), netuid, tx_kind), now - 1);
        pallet_subtensor::TxChildkeyTakeRateLimit::<Runtime>::put(span);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_childkey_take {
            hotkey: hot.clone(),
            netuid,
            take: 1,
        });
        let origin = RuntimeOrigin::signed(account(31));
        let legacy = || {
            TransactionType::SetChildkeyTake.passes_rate_limit_on_subnet::<Runtime>(&hot, netuid)
        };
        parity_check(now, call, origin, None, None, legacy);
    });
}

#[test]
fn set_children_parity() {
    new_ext().execute_with(|| {
        let now = 500u64;
        let hot = account(40);
        let netuid = NetUid::from(2u16);
        let tx_kind: u16 = TransactionType::SetChildren.into();
        TransactionKeyLastBlock::<Runtime>::insert((hot.clone(), netuid, tx_kind), now - 1);

        let call = RuntimeCall::SubtensorModule(SubtensorCall::set_children {
            hotkey: hot.clone(),
            netuid,
            children: Vec::new(),
        });
        let origin = RuntimeOrigin::signed(account(41));
        let legacy =
            || TransactionType::SetChildren.passes_rate_limit_on_subnet::<Runtime>(&hot, netuid);
        parity_check(now, call, origin, None, None, legacy);
    });
}

#[test]
fn serving_parity() {
    new_ext().execute_with(|| {
        let now = 600u64;
        let hot = account(50);
        let netuid = NetUid::from(3u16);
        let span = 5u64;
        ServingRateLimit::<Runtime>::insert(netuid, span);
        pallet_subtensor::Axons::<Runtime>::insert(
            netuid,
            hot.clone(),
            AxonInfo {
                block: now - 1,
                ..Default::default()
            },
        );
        pallet_subtensor::Prometheus::<Runtime>::insert(
            netuid,
            hot.clone(),
            PrometheusInfo {
                block: now - 1,
                ..Default::default()
            },
        );

        // Axon
        let axon_call = RuntimeCall::SubtensorModule(SubtensorCall::serve_axon {
            netuid,
            version: 1,
            ip: 0,
            port: 0,
            ip_type: 4,
            protocol: 0,
            placeholder1: 0,
            placeholder2: 0,
        });
        let origin = RuntimeOrigin::signed(hot.clone());
        let legacy_axon = || {
            SubtensorModule::axon_passes_rate_limit(
                netuid,
                &AxonInfo {
                    block: now - 1,
                    ..Default::default()
                },
                now,
            )
        };
        parity_check(now, axon_call, origin.clone(), None, None, legacy_axon);

        // Prometheus
        let prom_call = RuntimeCall::SubtensorModule(SubtensorCall::serve_prometheus {
            netuid,
            version: 1,
            ip: 0,
            port: 0,
            ip_type: 4,
        });
        let legacy_prom = || {
            SubtensorModule::prometheus_passes_rate_limit(
                netuid,
                &PrometheusInfo {
                    block: now - 1,
                    ..Default::default()
                },
                now,
            )
        };
        parity_check(now, prom_call, origin, None, None, legacy_prom);
    });
}

#[test]
fn weights_and_hparam_parity() {
    new_ext().execute_with(|| {
        let now = 700u64;
        let hot = account(60);
        let netuid = NetUid::from(4u16);
        let uid: u16 = 0;
        let weights_span = 4u64;
        let tempo = 3u16;
        // Ensure subnet exists so LastUpdate is imported.
        NetworksAdded::<Runtime>::insert(netuid, true);
        SubtensorModule::set_tempo(netuid, tempo);
        WeightsSetRateLimit::<Runtime>::insert(netuid, weights_span);
        LastUpdate::<Runtime>::insert(NetUidStorageIndex::from(netuid), vec![now - 1]);

        let weights_call = RuntimeCall::SubtensorModule(SubtensorCall::set_weights {
            netuid,
            dests: Vec::new(),
            weights: Vec::new(),
            version_key: 0,
        });
        let origin = RuntimeOrigin::signed(hot.clone());
        let scope = Some(netuid);
        let usage = Some(vec![UsageKey::SubnetNeuron { netuid, uid }]);

        let legacy_weights = || SubtensorModule::check_rate_limit(netuid.into(), uid, now);
        parity_check(
            now,
            weights_call,
            origin.clone(),
            usage,
            scope,
            legacy_weights,
        );

        // Hyperparam (activity_cutoff) with tempo scaling.
        let hparam_span_epochs = 2u16;
        pallet_subtensor::OwnerHyperparamRateLimit::<Runtime>::put(hparam_span_epochs);
        LastRateLimitedBlock::<Runtime>::insert(
            RateLimitKey::OwnerHyperparamUpdate(
                netuid,
                pallet_subtensor::utils::rate_limiting::Hyperparameter::ActivityCutoff,
            ),
            now - 1,
        );
        let hparam_call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_activity_cutoff {
                netuid,
                activity_cutoff: 1,
            });
        let hparam_origin = RuntimeOrigin::signed(hot);
        let legacy_hparam = || {
            let span = (tempo as u64) * (hparam_span_epochs as u64);
            let last = now - 1;
            // same logic as TransactionType::OwnerHyperparamUpdate in legacy: passes if delta >= span.
            let delta = now.saturating_sub(last);
            delta >= span
        };
        parity_check(now, hparam_call, hparam_origin, None, None, legacy_hparam);
    });
}

#[test]
fn weights_version_parity() {
    new_ext().execute_with(|| {
        let now = 800u64;
        let hot = account(70);
        let netuid = NetUid::from(5u16);
        NetworksAdded::<Runtime>::insert(netuid, true);
        SubtensorModule::set_tempo(netuid, 4);
        WeightsVersionKeyRateLimit::<Runtime>::put(2u64);
        let tx_kind_wvk: u16 = TransactionType::SetWeightsVersionKey.into();
        TransactionKeyLastBlock::<Runtime>::insert((hot.clone(), netuid, tx_kind_wvk), now - 1);

        let wvk_call =
            RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_weights_version_key {
                netuid,
                weights_version_key: 0,
            });
        let origin = RuntimeOrigin::signed(hot.clone());
        let legacy_wvk = || {
            let limit = SubtensorModule::get_tempo(netuid) as u64
                * WeightsVersionKeyRateLimit::<Runtime>::get();
            let delta = now.saturating_sub(now - 1);
            delta >= limit
        };
        parity_check(now, wvk_call, origin, None, None, legacy_wvk);
    });
}

#[test]
fn associate_evm_key_parity() {
    new_ext().execute_with(|| {
        let now = 900u64;
        let hot = account(80);
        let netuid = NetUid::from(6u16);
        let uid: u16 = 0;
        NetworksAdded::<Runtime>::insert(netuid, true);
        pallet_subtensor::AssociatedEvmAddress::<Runtime>::insert(
            netuid,
            uid,
            (H160::zero(), now - 1),
        );

        let call = RuntimeCall::SubtensorModule(SubtensorCall::associate_evm_key {
            netuid,
            evm_key: H160::zero(),
            block_number: now,
            signature: ecdsa::Signature::from_raw([0u8; 65]),
        });
        let origin = RuntimeOrigin::signed(hot.clone());
        let usage = Some(vec![UsageKey::SubnetNeuron { netuid, uid }]);
        let scope = Some(netuid);
        let limit = <Runtime as pallet_subtensor::Config>::EvmKeyAssociateRateLimit::get();
        let legacy = || {
            let last = now - 1;
            let delta = now.saturating_sub(last);
            delta >= limit
        };
        parity_check(now, call, origin, usage, scope, legacy);
    });
}
