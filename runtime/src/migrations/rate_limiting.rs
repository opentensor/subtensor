use core::{convert::TryFrom, marker::PhantomData};

use frame_support::{BoundedBTreeSet, BoundedVec, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use log::{info, warn};
use pallet_rate_limiting::{
    GroupSharing, RateLimit, RateLimitGroup, RateLimitKind, RateLimitTarget, TransactionIdentifier,
};
use pallet_subtensor::{
    self, AssociatedEvmAddress, Axons, Config as SubtensorConfig, HasMigrationRun, LastUpdate,
    Pallet, Prometheus,
};
use sp_runtime::traits::SaturatedConversion;
use sp_std::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    vec,
    vec::Vec,
};
use subtensor_runtime_common::{
    NetUid,
    rate_limiting::{
        GROUP_DELEGATE_TAKE, GROUP_OWNER_HPARAMS, GROUP_REGISTER_NETWORK, GROUP_SERVE,
        GROUP_STAKING_OPS, GROUP_SWAP_KEYS, GROUP_WEIGHTS_SUBNET, GroupId, RateLimitUsageKey,
        ServingEndpoint,
    },
};

use crate::{
    AccountId, Runtime,
    rate_limiting::{
        LimitSettingRule,
        legacy::{
            Hyperparameter, RateLimitKey, TransactionType, defaults as legacy_defaults,
            storage as legacy_storage,
        },
    },
};

type GroupNameOf<T> = BoundedVec<u8, <T as pallet_rate_limiting::Config>::MaxGroupNameLength>;
type GroupMembersOf<T> =
    BoundedBTreeSet<TransactionIdentifier, <T as pallet_rate_limiting::Config>::MaxGroupMembers>;

// Pallet index assigned to `pallet_subtensor` in `construct_runtime!`.
const SUBTENSOR_PALLET_INDEX: u8 = 7;
// Pallet index assigned to `pallet_admin_utils` in `construct_runtime!`.
const ADMIN_UTILS_PALLET_INDEX: u8 = 19;

/// Marker stored in `pallet_subtensor::HasMigrationRun` once the migration finishes.
pub const MIGRATION_NAME: &[u8] = b"migrate_rate_limiting";

// `set_children` is rate-limited to once every 150 blocks, it's hard-coded in the legacy code.
const SET_CHILDREN_RATE_LIMIT: u64 = 150;

// Hyperparameter extrinsics routed through owner-or-root rate limiting.
const HYPERPARAMETERS: &[Hyperparameter] = &[
    Hyperparameter::ServingRateLimit,
    Hyperparameter::MaxDifficulty,
    Hyperparameter::AdjustmentAlpha,
    Hyperparameter::ImmunityPeriod,
    Hyperparameter::MinAllowedWeights,
    Hyperparameter::MaxAllowedUids,
    Hyperparameter::Kappa,
    Hyperparameter::Rho,
    Hyperparameter::ActivityCutoff,
    Hyperparameter::PowRegistrationAllowed,
    Hyperparameter::MinBurn,
    Hyperparameter::MaxBurn,
    Hyperparameter::BondsMovingAverage,
    Hyperparameter::BondsPenalty,
    Hyperparameter::CommitRevealEnabled,
    Hyperparameter::LiquidAlphaEnabled,
    Hyperparameter::AlphaValues,
    Hyperparameter::WeightCommitInterval,
    Hyperparameter::TransferEnabled,
    Hyperparameter::AlphaSigmoidSteepness,
    Hyperparameter::Yuma3Enabled,
    Hyperparameter::BondsResetEnabled,
    Hyperparameter::ImmuneNeuronLimit,
    Hyperparameter::RecycleOrBurn,
];

/// Runtime hook that executes the rate-limiting migration.
pub struct Migration<T: SubtensorConfig>(PhantomData<T>);

impl<T> frame_support::traits::OnRuntimeUpgrade for Migration<T>
where
    T: SubtensorConfig + pallet_rate_limiting::Config<LimitScope = NetUid, GroupId = GroupId>,
    RateLimitUsageKey<T::AccountId>: Into<<T as pallet_rate_limiting::Config>::UsageKey>,
{
    fn on_runtime_upgrade() -> Weight {
        migrate_rate_limiting()
    }
}

pub fn migrate_rate_limiting() -> Weight {
    let mut weight = <Runtime as frame_system::Config>::DbWeight::get().reads(1);
    if HasMigrationRun::<Runtime>::get(MIGRATION_NAME) {
        info!("Rate-limiting migration already executed. Skipping.");
        return weight;
    }

    let (groups, commits, reads) = commits();
    weight = weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().reads(reads));

    let (limit_commits, last_seen_commits) = commits.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut limits, mut seen), commit| {
            match commit.kind {
                CommitKind::Limit(limit) => limits.push((commit.target, limit)),
                CommitKind::LastSeen(ls) => seen.push((commit.target, ls)),
            }
            (limits, seen)
        },
    );

    let (group_writes, group_count) = migrate_grouping(&groups);
    let (limit_writes, limits_len) = migrate_limits(limit_commits);
    let (last_seen_writes, last_seen_len) = migrate_last_seen(last_seen_commits);

    let mut writes = group_writes
        .saturating_add(limit_writes)
        .saturating_add(last_seen_writes);

    // Legacy parity: serving-rate-limit configuration is allowed for root OR subnet owner.
    // Everything else remains default (`AdminOrigin` / root in this runtime).
    pallet_rate_limiting::LimitSettingRules::<Runtime>::insert(
        RateLimitTarget::Group(GROUP_SERVE),
        LimitSettingRule::RootOrSubnetOwnerAdminWindow,
    );
    writes += 1;

    HasMigrationRun::<Runtime>::insert(MIGRATION_NAME, true);
    writes += 1;

    weight =
        weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().writes(writes));

    info!(
        "New migration wrote {} limits, {} last-seen entries, and {} groups into pallet-rate-limiting",
        limits_len, last_seen_len, group_count
    );

    weight
}

// Main entrypoint: build all groups and commits, along with storage reads.
fn commits() -> (Vec<GroupConfig>, Vec<Commit>, u64) {
    let mut groups = Vec::new();
    let mut commits = Vec::new();

    // grouped
    let mut reads = build_serving(&mut groups, &mut commits);
    reads = reads.saturating_add(build_delegate_take(&mut groups, &mut commits));
    reads = reads.saturating_add(build_weights(&mut groups, &mut commits));
    reads = reads.saturating_add(build_register_network(&mut groups, &mut commits));
    reads = reads.saturating_add(build_owner_hparams(&mut groups, &mut commits));
    reads = reads.saturating_add(build_staking_ops(&mut groups, &mut commits));
    reads = reads.saturating_add(build_swap_keys(&mut groups, &mut commits));

    // standalone
    reads = reads.saturating_add(build_childkey_take(&mut commits));
    reads = reads.saturating_add(build_set_children(&mut commits));
    reads = reads.saturating_add(build_weights_version_key(&mut commits));
    reads = reads.saturating_add(build_sn_owner_hotkey(&mut commits));
    reads = reads.saturating_add(build_associate_evm(&mut commits));
    reads = reads.saturating_add(build_mechanism_count(&mut commits));
    reads = reads.saturating_add(build_mechanism_emission(&mut commits));
    reads = reads.saturating_add(build_trim_max_uids(&mut commits));

    (groups, commits, reads)
}

fn migrate_grouping(groups: &[GroupConfig]) -> (u64, usize) {
    let mut writes: u64 = 0;
    let mut max_group_id: Option<GroupId> = None;

    for group in groups {
        let Ok(name) = GroupNameOf::<Runtime>::try_from(group.name.clone()) else {
            warn!(
                "rate-limiting migration: group name exceeds bounds, skipping id {}",
                group.id
            );
            continue;
        };

        pallet_rate_limiting::Groups::<Runtime>::insert(
            group.id,
            RateLimitGroup {
                id: group.id,
                name: name.clone(),
                sharing: group.sharing,
            },
        );
        pallet_rate_limiting::GroupNameIndex::<Runtime>::insert(name, group.id);
        writes += 2;

        let mut member_set = BTreeSet::new();
        for call in &group.members {
            member_set.insert(call.identifier());
            pallet_rate_limiting::CallGroups::<Runtime>::insert(call.identifier(), group.id);
            writes += 1;
            if call.read_only {
                pallet_rate_limiting::CallReadOnly::<Runtime>::insert(call.identifier(), true);
                writes += 1;
            }
        }
        let Ok(bounded) = GroupMembersOf::<Runtime>::try_from(member_set) else {
            warn!(
                "rate-limiting migration: group {} has too many members, skipping assignment",
                group.id
            );
            continue;
        };
        pallet_rate_limiting::GroupMembers::<Runtime>::insert(group.id, bounded);
        writes += 1;

        max_group_id = Some(max_group_id.map_or(group.id, |current| current.max(group.id)));
    }

    let next_group_id = max_group_id.map_or(0, |id| id.saturating_add(1));
    pallet_rate_limiting::NextGroupId::<Runtime>::put(next_group_id);
    writes += 1;

    (writes, groups.len())
}

fn migrate_limits(limit_commits: Vec<(RateLimitTarget<GroupId>, MigratedLimit)>) -> (u64, usize) {
    let mut writes: u64 = 0;
    let mut limits: BTreeMap<RateLimitTarget<GroupId>, RateLimit<NetUid, BlockNumberFor<Runtime>>> =
        BTreeMap::new();

    for (target, MigratedLimit { span, scope }) in limit_commits {
        let entry = limits.entry(target).or_insert_with(|| match scope {
            Some(s) => RateLimit::scoped_single(s, RateLimitKind::Exact(span)),
            None => RateLimit::global(RateLimitKind::Exact(span)),
        });

        if let Some(netuid) = scope {
            match entry {
                RateLimit::Global(_) => {
                    *entry = RateLimit::scoped_single(netuid, RateLimitKind::Exact(span));
                }
                RateLimit::Scoped(map) => {
                    map.insert(netuid, RateLimitKind::Exact(span));
                }
            }
        } else {
            *entry = RateLimit::global(RateLimitKind::Exact(span));
        }
    }

    let len = limits.len();
    for (target, limit) in limits {
        pallet_rate_limiting::Limits::<Runtime>::insert(target, limit);
        writes += 1;
    }

    (writes, len)
}

fn migrate_last_seen(
    last_seen_commits: Vec<(RateLimitTarget<GroupId>, MigratedLastSeen)>,
) -> (u64, usize) {
    let mut writes: u64 = 0;
    let mut last_seen: BTreeMap<
        (
            RateLimitTarget<GroupId>,
            Option<RateLimitUsageKey<AccountId>>,
        ),
        BlockNumberFor<Runtime>,
    > = BTreeMap::new();

    for (target, MigratedLastSeen { block, usage }) in last_seen_commits {
        let key = (target, usage);
        last_seen
            .entry(key)
            .and_modify(|existing| {
                if block > *existing {
                    *existing = block;
                }
            })
            .or_insert(block);
    }

    let len = last_seen.len();
    for ((target, usage), block) in last_seen {
        pallet_rate_limiting::LastSeen::<Runtime>::insert(target, usage, block);
        writes += 1;
    }

    (writes, len)
}

// Serving group (config+usage shared).
// scope: netuid
// usage: account+netuid, but different keys (endpoint value) for axon/prometheus
// legacy sources: ServingRateLimit (per netuid), Axons/Prometheus
fn build_serving(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    // Create the group with all its members.
    groups.push(GroupConfig {
        id: GROUP_SERVE,
        name: b"serving".to_vec(),
        sharing: GroupSharing::ConfigAndUsage,
        members: vec![
            MigratedCall::subtensor(4, false),  // serve_axon
            MigratedCall::subtensor(40, false), // serve_axon_tls
            MigratedCall::subtensor(5, false),  // serve_prometheus
        ],
    });

    let (serving_limits, serving_reads) = legacy_storage::serving_rate_limits();
    reads = reads.saturating_add(serving_reads);
    // Limits per netuid (written to the group target).
    // Merge live subnets (which may rely on default rate-limit values) with any legacy entries that
    // exist only in storage, so we migrate both current and previously stored netuids without
    // duplicates.
    let mut netuids = Pallet::<Runtime>::get_all_subnet_netuids();
    for (&netuid, _) in &serving_limits {
        if !netuids.contains(&netuid) {
            netuids.push(netuid);
        }
    }
    let default_limit = legacy_defaults::serving_rate_limit();
    for netuid in netuids {
        reads = reads.saturating_add(1);
        push_limit_commit_if_non_zero(
            commits,
            RateLimitTarget::Group(GROUP_SERVE),
            serving_limits
                .get(&netuid)
                .copied()
                .unwrap_or(default_limit),
            Some(netuid),
        );
    }

    // Axon last-seen (group-shared usage).
    for (netuid, hotkey, axon) in Axons::<Runtime>::iter() {
        reads = reads.saturating_add(1);
        if let Some(block) = block_number::<Runtime>(axon.block) {
            commits.push(Commit {
                target: RateLimitTarget::Group(GROUP_SERVE),
                kind: CommitKind::LastSeen(MigratedLastSeen {
                    block,
                    usage: Some(RateLimitUsageKey::AccountSubnetServing {
                        account: hotkey.clone(),
                        netuid,
                        endpoint: ServingEndpoint::Axon,
                    }),
                }),
            });
        }
    }

    // Prometheus last-seen (group-shared usage).
    for (netuid, hotkey, prom) in Prometheus::<Runtime>::iter() {
        reads = reads.saturating_add(1);
        if let Some(block) = block_number::<Runtime>(prom.block) {
            commits.push(Commit {
                target: RateLimitTarget::Group(GROUP_SERVE),
                kind: CommitKind::LastSeen(MigratedLastSeen {
                    block,
                    usage: Some(RateLimitUsageKey::AccountSubnetServing {
                        account: hotkey,
                        netuid,
                        endpoint: ServingEndpoint::Prometheus,
                    }),
                }),
            });
        }
    }

    reads
}

// Delegate take group (config + usage shared).
// usage: account
// legacy sources: TxDelegateTakeRateLimit, LastTxBlockDelegateTake
fn build_delegate_take(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    groups.push(GroupConfig {
        id: GROUP_DELEGATE_TAKE,
        name: b"delegate-take".to_vec(),
        sharing: GroupSharing::ConfigAndUsage,
        members: vec![
            MigratedCall::subtensor(66, false), // increase_take
            MigratedCall::subtensor(65, false), // decrease_take
        ],
    });

    let target = RateLimitTarget::Group(GROUP_DELEGATE_TAKE);
    let (delegate_take_limit, delegate_reads) = legacy_storage::tx_delegate_take_rate_limit();
    reads = reads.saturating_add(delegate_reads);
    push_limit_commit_if_non_zero(commits, target, delegate_take_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::LastTxBlockDelegateTake(account) => {
                    Some((target, Some(RateLimitUsageKey::Account(account))))
                }
                _ => None,
            },
        ),
    );

    reads
}

// Weights group (config + usage shared).
// scope: netuid
// usage: netuid+neuron/netuid+mechanism+neuron
// legacy source: WeightsSetRateLimit, LastUpdate (subnet/mechanism)
fn build_weights(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    groups.push(GroupConfig {
        id: GROUP_WEIGHTS_SUBNET,
        name: b"weights".to_vec(),
        sharing: GroupSharing::ConfigAndUsage,
        members: vec![
            MigratedCall::subtensor(0, false),   // set_weights
            MigratedCall::subtensor(80, false),  // batch_set_weights
            MigratedCall::subtensor(96, false),  // commit_weights
            MigratedCall::subtensor(100, false), // batch_commit_weights
            MigratedCall::subtensor(113, false), // commit_timelocked_weights
            MigratedCall::subtensor(97, false),  // reveal_weights
            MigratedCall::subtensor(98, false),  // batch_reveal_weights
            MigratedCall::subtensor(119, false), // set_mechanism_weights
            MigratedCall::subtensor(115, false), // commit_mechanism_weights
            MigratedCall::subtensor(117, false), // commit_crv3_mechanism_weights
            MigratedCall::subtensor(118, false), // commit_timelocked_mechanism_weights
            MigratedCall::subtensor(116, false), // reveal_mechanism_weights
        ],
    });

    let (weights_limits, weights_reads) = legacy_storage::weights_set_rate_limits();
    reads = reads.saturating_add(weights_reads);
    let default_limit = legacy_defaults::weights_set_rate_limit();
    for netuid in Pallet::<Runtime>::get_all_subnet_netuids() {
        reads = reads.saturating_add(1);
        push_limit_commit_if_non_zero(
            commits,
            RateLimitTarget::Group(GROUP_WEIGHTS_SUBNET),
            weights_limits
                .get(&netuid)
                .copied()
                .unwrap_or(default_limit),
            Some(netuid),
        );
    }

    for (index, blocks) in LastUpdate::<Runtime>::iter() {
        reads = reads.saturating_add(1);
        let (netuid, mecid) =
            Pallet::<Runtime>::get_netuid_and_subid(index).unwrap_or((NetUid::ROOT, 0.into()));
        for (uid, last_block) in blocks.into_iter().enumerate() {
            let Some(block) = block_number::<Runtime>(last_block) else {
                continue;
            };
            let Ok(uid_u16) = u16::try_from(uid) else {
                continue;
            };
            let usage = if mecid == 0.into() {
                RateLimitUsageKey::SubnetNeuron {
                    netuid,
                    uid: uid_u16,
                }
            } else {
                RateLimitUsageKey::SubnetMechanismNeuron {
                    netuid,
                    mecid,
                    uid: uid_u16,
                }
            };
            commits.push(Commit {
                target: RateLimitTarget::Group(GROUP_WEIGHTS_SUBNET),
                kind: CommitKind::LastSeen(MigratedLastSeen {
                    block,
                    usage: Some(usage),
                }),
            });
        }
    }

    reads
}

// Register network group (config + usage shared).
// legacy sources: NetworkRateLimit, NetworkLastRegistered
fn build_register_network(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    groups.push(GroupConfig {
        id: GROUP_REGISTER_NETWORK,
        name: b"register-network".to_vec(),
        sharing: GroupSharing::ConfigAndUsage,
        members: vec![
            MigratedCall::subtensor(59, false), // register_network
            MigratedCall::subtensor(79, false), // register_network_with_identity
        ],
    });

    let target = RateLimitTarget::Group(GROUP_REGISTER_NETWORK);
    let (network_rate_limit, network_reads) = legacy_storage::network_rate_limit();
    reads = reads.saturating_add(network_reads);
    push_limit_commit_if_non_zero(commits, target, network_rate_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::NetworkLastRegistered => Some((target, None)),
                _ => None,
            },
        ),
    );

    reads
}

// Owner hyperparameter group (config shared, usage per call).
// usage: netuid
// legacy sources: OwnerHyperparamRateLimit * tempo, LastRateLimitedBlock per OwnerHyperparamUpdate
fn build_owner_hparams(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    groups.push(GroupConfig {
        id: GROUP_OWNER_HPARAMS,
        name: b"owner-hparams".to_vec(),
        sharing: GroupSharing::ConfigOnly,
        members: HYPERPARAMETERS
            .iter()
            .filter_map(|h| identifier_for_hyperparameter(*h))
            .collect(),
    });

    let group_target = RateLimitTarget::Group(GROUP_OWNER_HPARAMS);
    let (owner_limit, owner_reads) = legacy_storage::owner_hyperparam_rate_limit();
    reads = reads.saturating_add(owner_reads);
    push_limit_commit_if_non_zero(commits, group_target, owner_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::OwnerHyperparamUpdate(netuid, hyper) => {
                    let Some(identifier) = identifier_for_hyperparameter(hyper) else {
                        return None;
                    };
                    Some((
                        RateLimitTarget::Transaction(identifier.identifier()),
                        Some(RateLimitUsageKey::Subnet(netuid)),
                    ))
                }
                _ => None,
            },
        ),
    );

    reads
}

// Staking ops group (config + usage shared, all ops 1 block).
// usage: coldkey+hotkey+netuid
// legacy sources: TxRateLimit (reset every block for staking ops), StakingOperationRateLimiter
fn build_staking_ops(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    groups.push(GroupConfig {
        id: GROUP_STAKING_OPS,
        name: b"staking-ops".to_vec(),
        sharing: GroupSharing::ConfigAndUsage,
        members: vec![
            MigratedCall::subtensor(2, false),  // add_stake
            MigratedCall::subtensor(88, false), // add_stake_limit
            MigratedCall::subtensor(3, true),   // remove_stake
            MigratedCall::subtensor(89, true),  // remove_stake_limit
            MigratedCall::subtensor(103, true), // remove_stake_full_limit
            MigratedCall::subtensor(85, false), // move_stake
            MigratedCall::subtensor(86, true),  // transfer_stake
            MigratedCall::subtensor(87, false), // swap_stake
            MigratedCall::subtensor(90, false), // swap_stake_limit
        ],
    });

    push_limit_commit_if_non_zero(commits, RateLimitTarget::Group(GROUP_STAKING_OPS), 1, None);

    // we don't need to migrate last-seen since the limiter is reset every block.

    0
}

// Swap hotkey/coldkey share the lock and usage; swap_coldkey bypasses enforcement but records
// usage.
// usage: account (coldkey)
// legacy sources: TxRateLimit, LastRateLimitedBlock per LastTxBlock
fn build_swap_keys(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    groups.push(GroupConfig {
        id: GROUP_SWAP_KEYS,
        name: b"swap-keys".to_vec(),
        sharing: GroupSharing::ConfigAndUsage,
        members: vec![
            MigratedCall::subtensor(70, false), // swap_hotkey
            MigratedCall::subtensor(71, false), // swap_coldkey
        ],
    });

    let target = RateLimitTarget::Group(GROUP_SWAP_KEYS);
    let (tx_rate_limit, tx_reads) = legacy_storage::tx_rate_limit();
    reads = reads.saturating_add(tx_reads);
    push_limit_commit_if_non_zero(commits, target, tx_rate_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::LastTxBlock(account) => {
                    Some((target, Some(RateLimitUsageKey::Account(account))))
                }
                _ => None,
            },
        ),
    );

    reads
}

// Standalone set_childkey_take.
// usage: account+netuid
// legacy sources: TxChildkeyTakeRateLimit, TransactionKeyLastBlock per SetChildkeyTake
fn build_childkey_take(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(SUBTENSOR_PALLET_INDEX, 75));
    let (childkey_limit, childkey_reads) = legacy_storage::tx_childkey_take_rate_limit();
    reads = reads.saturating_add(childkey_reads);
    push_limit_commit_if_non_zero(commits, target, childkey_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
            commits,
            target,
            TransactionType::SetChildkeyTake,
        ),
    );

    reads
}

// Standalone set_children.
// usage: account+netuid
// legacy sources: SET_CHILDREN_RATE_LIMIT (constant 150), TransactionKeyLastBlock per SetChildren
fn build_set_children(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(SUBTENSOR_PALLET_INDEX, 67));
    push_limit_commit_if_non_zero(commits, target, SET_CHILDREN_RATE_LIMIT, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
            commits,
            target,
            TransactionType::SetChildren,
        ),
    );

    reads
}

// Standalone set_weights_version_key.
// scope: netuid
// usage: account+netuid
// legacy sources: WeightsVersionKeyRateLimit * tempo,
// 			       TransactionKeyLastBlock per SetWeightsVersionKey
fn build_weights_version_key(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 6));
    let (weights_version_limit, weights_version_reads) =
        legacy_storage::weights_version_key_rate_limit();
    reads = reads.saturating_add(weights_version_reads);
    push_limit_commit_if_non_zero(commits, target, weights_version_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
            commits,
            target,
            TransactionType::SetWeightsVersionKey,
        ),
    );

    reads
}

// Standalone set_sn_owner_hotkey.
// usage: netuid
// legacy sources: DefaultSetSNOwnerHotkeyRateLimit, LastRateLimitedBlock per SetSNOwnerHotkey
fn build_sn_owner_hotkey(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 67));
    let sn_owner_limit = legacy_defaults::sn_owner_hotkey_rate_limit();
    reads += 1;
    push_limit_commit_if_non_zero(commits, target, sn_owner_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::SetSNOwnerHotkey(netuid) => {
                    Some((target, Some(RateLimitUsageKey::Subnet(netuid))))
                }
                _ => None,
            },
        ),
    );

    reads
}

// Standalone associate_evm_key.
// usage: netuid+neuron
// legacy sources: EvmKeyAssociateRateLimit, AssociatedEvmAddress
fn build_associate_evm(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(SUBTENSOR_PALLET_INDEX, 93));
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        <Runtime as SubtensorConfig>::EvmKeyAssociateRateLimit::get(),
        None,
    );

    for (netuid, uid, (_, block)) in AssociatedEvmAddress::<Runtime>::iter() {
        reads = reads.saturating_add(1);
        let Some(block) = block_number::<Runtime>(block) else {
            continue;
        };
        commits.push(Commit {
            target,
            kind: CommitKind::LastSeen(MigratedLastSeen {
                block,
                usage: Some(RateLimitUsageKey::SubnetNeuron { netuid, uid }),
            }),
        });
    }

    reads
}

// Standalone mechanism count.
// usage: account+netuid
// legacy sources: MechanismCountSetRateLimit, TransactionKeyLastBlock per MechanismCountUpdate
// sudo_set_mechanism_count
fn build_mechanism_count(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 76));
    let mechanism_limit = legacy_defaults::mechanism_count_rate_limit();
    push_limit_commit_if_non_zero(commits, target, mechanism_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
            commits,
            target,
            TransactionType::MechanismCountUpdate,
        ),
    );

    reads
}

// Standalone mechanism emission.
// usage: account+netuid
// legacy sources: MechanismEmissionRateLimit, TransactionKeyLastBlock per MechanismEmission
// sudo_set_mechanism_emission_split
fn build_mechanism_emission(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 77));
    let emission_limit = legacy_defaults::mechanism_emission_rate_limit();
    push_limit_commit_if_non_zero(commits, target, emission_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
            commits,
            target,
            TransactionType::MechanismEmission,
        ),
    );

    reads
}

// Standalone trim_to_max_allowed_uids.
// usage: account+netuid
// legacy sources: MaxUidsTrimmingRateLimit, TransactionKeyLastBlock per MaxUidsTrimming
// sudo_trim_to_max_allowed_uids
fn build_trim_max_uids(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 78));
    let trim_limit = legacy_defaults::max_uids_trimming_rate_limit();
    push_limit_commit_if_non_zero(commits, target, trim_limit, None);

    reads = reads.saturating_add(
        last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
            commits,
            target,
            TransactionType::MaxUidsTrimming,
        ),
    );

    reads
}

struct Commit {
    target: RateLimitTarget<GroupId>,
    kind: CommitKind,
}

enum CommitKind {
    Limit(MigratedLimit),
    LastSeen(MigratedLastSeen),
}

struct MigratedLimit {
    span: BlockNumberFor<Runtime>,
    scope: Option<NetUid>,
}

struct MigratedLastSeen {
    block: BlockNumberFor<Runtime>,
    usage: Option<RateLimitUsageKey<AccountId>>,
}

struct GroupConfig {
    id: GroupId,
    name: Vec<u8>,
    sharing: GroupSharing,
    members: Vec<MigratedCall>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MigratedCall {
    identifier: TransactionIdentifier,
    read_only: bool,
}

impl MigratedCall {
    const fn new(pallet_index: u8, call_index: u8, read_only: bool) -> Self {
        Self {
            identifier: TransactionIdentifier::new(pallet_index, call_index),
            read_only,
        }
    }

    const fn subtensor(call_index: u8, read_only: bool) -> Self {
        Self::new(SUBTENSOR_PALLET_INDEX, call_index, read_only)
    }

    const fn admin(call_index: u8, read_only: bool) -> Self {
        Self::new(ADMIN_UTILS_PALLET_INDEX, call_index, read_only)
    }

    pub fn identifier(&self) -> TransactionIdentifier {
        self.identifier
    }
}

fn push_limit_commit_if_non_zero(
    commits: &mut Vec<Commit>,
    target: RateLimitTarget<GroupId>,
    span: u64,
    scope: Option<NetUid>,
) {
    if let Some(span) = block_number::<Runtime>(span) {
        commits.push(Commit {
            target,
            kind: CommitKind::Limit(MigratedLimit { span, scope }),
        });
    }
}

mod last_seen_helpers {
    use core::mem::discriminant;

    use super::*;

    pub(super) fn collect_last_seen_from_last_rate_limited_block(
        commits: &mut Vec<Commit>,
        map: impl Fn(
            RateLimitKey<AccountId>,
        ) -> Option<(
            RateLimitTarget<GroupId>,
            Option<RateLimitUsageKey<AccountId>>,
        )>,
    ) -> u64 {
        let mut reads: u64 = 0;

        let (entries, iter_reads) = legacy_storage::last_rate_limited_blocks();
        reads = reads.saturating_add(iter_reads);
        for (key, block) in entries {
            let Some((target, usage)) = map(key) else {
                continue;
            };
            let Some(block) = block_number::<Runtime>(block) else {
                continue;
            };
            commits.push(Commit {
                target,
                kind: CommitKind::LastSeen(MigratedLastSeen { block, usage }),
            });
        }

        reads
    }

    pub(super) fn collect_last_seen_from_transaction_key_last_block(
        commits: &mut Vec<Commit>,
        target: RateLimitTarget<GroupId>,
        tx_filter: TransactionType,
    ) -> u64 {
        let mut reads: u64 = 0;

        let (entries, iter_reads) = legacy_storage::transaction_key_last_block();
        reads = reads.saturating_add(iter_reads);
        for ((account, netuid, tx_kind), block) in entries {
            let tx = TransactionType::from(tx_kind);
            if discriminant(&tx) != discriminant(&tx_filter) {
                continue;
            }
            let Some(usage) = usage_key_from_transaction_type(tx, &account, netuid) else {
                continue;
            };
            let Some(block) = block_number::<Runtime>(block) else {
                continue;
            };
            commits.push(Commit {
                target,
                kind: CommitKind::LastSeen(MigratedLastSeen {
                    block,
                    usage: Some(usage),
                }),
            });
        }

        reads
    }
}

// Produces the usage key for a `TransactionType` that was stored in `TransactionKeyLastBlock`.
fn usage_key_from_transaction_type(
    tx: TransactionType,
    account: &AccountId,
    netuid: NetUid,
) -> Option<RateLimitUsageKey<AccountId>> {
    match tx {
        TransactionType::MechanismCountUpdate
        | TransactionType::MaxUidsTrimming
        | TransactionType::MechanismEmission
        | TransactionType::SetChildkeyTake
        | TransactionType::SetChildren
        | TransactionType::SetWeightsVersionKey => Some(RateLimitUsageKey::AccountSubnet {
            account: account.clone(),
            netuid,
        }),
        TransactionType::SetSNOwnerHotkey | TransactionType::OwnerHyperparamUpdate(_) => {
            Some(RateLimitUsageKey::Subnet(netuid))
        }
        _ => None,
    }
}

// Returns the migrated call wrapper for the admin-utils extrinsic that controls `hparam`.
//
// Only hyperparameters that are currently rate-limited (i.e. routed through
// `ensure_sn_owner_or_root_with_limits`) are mapped; others return `None`.
fn identifier_for_hyperparameter(hparam: Hyperparameter) -> Option<MigratedCall> {
    use Hyperparameter::*;

    let identifier = match hparam {
        ServingRateLimit => MigratedCall::admin(3, false),
        MaxDifficulty => MigratedCall::admin(5, false),
        AdjustmentAlpha => MigratedCall::admin(9, false),
        ImmunityPeriod => MigratedCall::admin(13, false),
        MinAllowedWeights => MigratedCall::admin(14, false),
        MaxAllowedUids => MigratedCall::admin(15, false),
        Kappa => MigratedCall::admin(16, false),
        Rho => MigratedCall::admin(17, false),
        ActivityCutoff => MigratedCall::admin(18, false),
        PowRegistrationAllowed => MigratedCall::admin(20, false),
        MinBurn => MigratedCall::admin(22, false),
        MaxBurn => MigratedCall::admin(23, false),
        BondsMovingAverage => MigratedCall::admin(26, false),
        BondsPenalty => MigratedCall::admin(60, false),
        CommitRevealEnabled => MigratedCall::admin(49, false),
        LiquidAlphaEnabled => MigratedCall::admin(50, false),
        AlphaValues => MigratedCall::admin(51, false),
        WeightCommitInterval => MigratedCall::admin(57, false),
        TransferEnabled => MigratedCall::admin(61, false),
        AlphaSigmoidSteepness => MigratedCall::admin(68, false),
        Yuma3Enabled => MigratedCall::admin(69, false),
        BondsResetEnabled => MigratedCall::admin(70, false),
        ImmuneNeuronLimit => MigratedCall::admin(72, false),
        RecycleOrBurn => MigratedCall::admin(80, false),
        _ => return None,
    };

    Some(identifier)
}

fn block_number<T: SubtensorConfig>(value: u64) -> Option<BlockNumberFor<T>> {
    if value == 0 {
        return None;
    }
    Some(value.saturated_into::<BlockNumberFor<T>>())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use frame_support::traits::OnRuntimeUpgrade;
    use frame_system::pallet_prelude::BlockNumberFor;
    use pallet_rate_limiting::{
        RateLimit, RateLimitKind, RateLimitScopeResolver, RateLimitTarget, RateLimitUsageResolver,
        TransactionIdentifier,
    };
    use pallet_subtensor::{
        AxonInfo, Call as SubtensorCall, HasMigrationRun, LastRateLimitedBlock, LastUpdate,
        NetworksAdded, PrometheusInfo, RateLimitKey, TransactionKeyLastBlock, WeightsSetRateLimit,
        WeightsVersionKeyRateLimit, utils::rate_limiting::TransactionType,
    };
    use sp_core::{H160, ecdsa};
    use sp_io::TestExternalities;
    use sp_runtime::traits::{SaturatedConversion, Zero};

    use super::*;
    use crate::{
        BuildStorage, RuntimeCall, RuntimeOrigin, RuntimeScopeResolver, RuntimeUsageResolver,
        SubtensorModule, System,
    };
    use subtensor_runtime_common::NetUidStorageIndex;

    const ACCOUNT: [u8; 32] = [7u8; 32];
    const DELEGATE_TAKE_GROUP_ID: GroupId = GROUP_DELEGATE_TAKE;
    type UsageKey = RateLimitUsageKey<AccountId>;

    fn new_test_ext() -> TestExternalities {
        sp_tracing::try_init_simple();
        let mut ext: TestExternalities = crate::RuntimeGenesisConfig::default()
            .build_storage()
            .expect("runtime storage")
            .into();
        ext.execute_with(|| crate::System::set_block_number(1));
        ext
    }

    fn new_ext() -> TestExternalities {
        new_test_ext()
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
        scope_override: Option<Vec<NetUid>>,
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
        let span = match scope.as_ref() {
            None => pallet_rate_limiting::Pallet::<Runtime>::effective_span(
                &origin.clone().into(),
                &call,
                &target,
                &None,
            )
            .unwrap_or_default(),
            Some(scopes) => scopes
                .iter()
                .filter_map(|scope| {
                    pallet_rate_limiting::Pallet::<Runtime>::effective_span(
                        &origin.clone().into(),
                        &call,
                        &target,
                        &Some(*scope),
                    )
                })
                .max()
                .unwrap_or_default(),
        };
        let span_u64: u64 = span.saturated_into();

        let usage_keys: Vec<Option<<Runtime as pallet_rate_limiting::Config>::UsageKey>> =
            match usage {
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
    fn maps_hyperparameters() {
        assert_eq!(
            identifier_for_hyperparameter(Hyperparameter::ServingRateLimit),
            Some(MigratedCall::admin(3, false))
        );
        assert!(identifier_for_hyperparameter(Hyperparameter::MaxWeightLimit).is_none());
    }

    #[test]
    fn migration_populates_limits_last_seen_and_groups() {
        new_test_ext().execute_with(|| {
            let account: AccountId = ACCOUNT.into();
            pallet_subtensor::HasMigrationRun::<Runtime>::remove(MIGRATION_NAME);

            legacy_storage::set_tx_rate_limit(10);
            legacy_storage::set_tx_delegate_take_rate_limit(3);
            legacy_storage::set_last_rate_limited_block(
                super::RateLimitKey::LastTxBlock(account.clone()),
                5,
            );

            let weight = migrate_rate_limiting();
            assert!(!weight.is_zero());
            assert!(pallet_subtensor::HasMigrationRun::<Runtime>::get(
                MIGRATION_NAME
            ));

            let tx_target = RateLimitTarget::Group(GROUP_SWAP_KEYS);
            let delegate_group = RateLimitTarget::Group(DELEGATE_TAKE_GROUP_ID);

            assert_eq!(
                pallet_rate_limiting::Limits::<Runtime>::get(tx_target),
                Some(RateLimit::Global(RateLimitKind::Exact(
                    10u64.saturated_into()
                )))
            );
            assert_eq!(
                pallet_rate_limiting::Limits::<Runtime>::get(delegate_group),
                Some(RateLimit::Global(RateLimitKind::Exact(
                    3u64.saturated_into()
                )))
            );

            let usage_key = RateLimitUsageKey::Account(account.clone());
            assert_eq!(
                pallet_rate_limiting::LastSeen::<Runtime>::get(tx_target, Some(usage_key.clone())),
                Some(5u64.saturated_into())
            );

            let group = pallet_rate_limiting::Groups::<Runtime>::get(DELEGATE_TAKE_GROUP_ID)
                .expect("group stored");
            assert_eq!(group.id, DELEGATE_TAKE_GROUP_ID);
            assert_eq!(group.name.as_slice(), b"delegate-take");
            assert_eq!(
                pallet_rate_limiting::CallGroups::<Runtime>::get(
                    MigratedCall::subtensor(66, false).identifier()
                ),
                Some(DELEGATE_TAKE_GROUP_ID)
            );
            assert_eq!(pallet_rate_limiting::NextGroupId::<Runtime>::get(), 7);

            let serve_target = RateLimitTarget::Group(GROUP_SERVE);
            assert!(pallet_rate_limiting::LimitSettingRules::<Runtime>::contains_key(serve_target));
            assert_eq!(
                pallet_rate_limiting::LimitSettingRules::<Runtime>::get(serve_target),
                crate::rate_limiting::LimitSettingRule::RootOrSubnetOwnerAdminWindow
            );
        });
    }

    #[test]
    fn migrates_global_register_network_last_seen() {
        new_test_ext().execute_with(|| {
            HasMigrationRun::<Runtime>::remove(MIGRATION_NAME);

            // Seed legacy global register rate-limit state.
            LastRateLimitedBlock::<Runtime>::insert(RateLimitKey::NetworkLastRegistered, 10u64);
            System::set_block_number(12);

            // Run migration.
            Migration::<Runtime>::on_runtime_upgrade();

            let target = RateLimitTarget::Group(GROUP_REGISTER_NETWORK);

            // LastSeen preserved globally (usage = None).
            let stored = pallet_rate_limiting::LastSeen::<Runtime>::get(target, None::<UsageKey>)
                .expect("last seen entry");
            assert_eq!(stored, 10u64.saturated_into::<BlockNumberFor<Runtime>>());
        });
    }

    #[test]
    fn sn_owner_hotkey_limit_not_tempo_scaled_and_last_seen_preserved() {
        new_test_ext().execute_with(|| {
            HasMigrationRun::<Runtime>::remove(MIGRATION_NAME);

            let netuid = NetUid::from(1);
            // Give the subnet a non-1 tempo to catch accidental scaling.
            SubtensorModule::set_tempo(netuid, 5);
            LastRateLimitedBlock::<Runtime>::insert(RateLimitKey::SetSNOwnerHotkey(netuid), 100u64);

            Migration::<Runtime>::on_runtime_upgrade();

            let target = RateLimitTarget::Transaction(TransactionIdentifier::new(19, 67));

            // Limit should remain the fixed default (50400 blocks), not tempo-scaled.
            let limit = pallet_rate_limiting::Limits::<Runtime>::get(target).expect("limit stored");
            assert!(
                matches!(limit, RateLimit::Global(kind) if kind == RateLimitKind::Exact(50_400))
            );

            // LastSeen preserved per subnet.
            let usage: Option<<Runtime as pallet_rate_limiting::Config>::UsageKey> =
                Some(UsageKey::Subnet(netuid).into());
            let stored = pallet_rate_limiting::LastSeen::<Runtime>::get(target, usage)
                .expect("last seen entry");
            assert_eq!(stored, 100u64.saturated_into::<BlockNumberFor<Runtime>>());
        });
    }

    #[test]
    fn register_network_parity() {
        new_ext().execute_with(|| {
            HasMigrationRun::<Runtime>::remove(MIGRATION_NAME);
            let now = 100u64;
            let span = 5u64;
            System::set_block_number(now.saturated_into());
            LastRateLimitedBlock::<Runtime>::insert(RateLimitKey::NetworkLastRegistered, now - 1);
            legacy_storage::set_network_rate_limit(span);

            Migration::<Runtime>::on_runtime_upgrade();

            let target = RateLimitTarget::Group(GROUP_REGISTER_NETWORK);
            let limit = pallet_rate_limiting::Limits::<Runtime>::get(target).expect("limit stored");
            assert!(
                matches!(limit, RateLimit::Global(kind) if kind == RateLimitKind::Exact(exact_span(span)))
            );

            let stored = pallet_rate_limiting::LastSeen::<Runtime>::get(target, None::<UsageKey>)
                .expect("last seen entry");
            assert_eq!(stored, (now - 1).saturated_into::<BlockNumberFor<Runtime>>());
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
            LastRateLimitedBlock::<Runtime>::insert(
                RateLimitKey::LastTxBlock(cold.clone()),
                now - 1,
            );
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
            legacy_storage::set_tx_delegate_take_rate_limit(span);

            let call = RuntimeCall::SubtensorModule(SubtensorCall::increase_take {
                hotkey: hot.clone(),
                take: 5,
            });
            let origin = RuntimeOrigin::signed(account(21));
            let legacy = || {
                let last = now - 1;
                if span == 0 || last == 0 {
                    return true;
                }
                now - last > span
            };
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
                TransactionType::SetChildkeyTake
                    .passes_rate_limit_on_subnet::<Runtime>(&hot, netuid)
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
            let legacy = || {
                TransactionType::SetChildren.passes_rate_limit_on_subnet::<Runtime>(&hot, netuid)
            };
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
            legacy_storage::set_serving_rate_limit(netuid, span);
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
                let info = AxonInfo {
                    block: now.saturating_sub(1),
                    ..Default::default()
                };
                now.saturating_sub(info.block) >= span
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
                let info = PrometheusInfo {
                    block: now.saturating_sub(1),
                    ..Default::default()
                };
                now.saturating_sub(info.block) >= span
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
            let scope = Some(vec![netuid]);
            let usage = Some(vec![UsageKey::SubnetNeuron { netuid, uid }]);

            let legacy_weights = || {
                let last = LastUpdate::<Runtime>::get(NetUidStorageIndex::from(netuid))
                    .get(uid as usize)
                    .copied()
                    .unwrap_or_default();
                let limit = WeightsSetRateLimit::<Runtime>::get(netuid);
                now.saturating_sub(last) >= limit
            };
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
            let scope = Some(vec![netuid]);
            let limit = <Runtime as pallet_subtensor::Config>::EvmKeyAssociateRateLimit::get();
            let legacy = || {
                let last = now - 1;
                let delta = now.saturating_sub(last);
                delta >= limit
            };
            parity_check(now, call, origin, usage, scope, legacy);
        });
    }

    #[test]
    fn migration_skips_when_already_run() {
        new_test_ext().execute_with(|| {
            pallet_subtensor::HasMigrationRun::<Runtime>::insert(MIGRATION_NAME, true);
            legacy_storage::set_tx_rate_limit(99);

            let base_weight = <Runtime as frame_system::Config>::DbWeight::get().reads(1);
            let weight = migrate_rate_limiting();

            assert_eq!(weight, base_weight);
            assert!(
                pallet_rate_limiting::Limits::<Runtime>::iter()
                    .next()
                    .is_none()
            );
            assert!(
                pallet_rate_limiting::LastSeen::<Runtime>::iter()
                    .next()
                    .is_none()
            );
        });
    }
}
