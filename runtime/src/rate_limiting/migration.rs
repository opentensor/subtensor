use core::{convert::TryFrom, marker::PhantomData};

use frame_support::{BoundedBTreeSet, BoundedVec, traits::Get, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use log::{info, warn};
use pallet_rate_limiting::{
    GroupSharing, RateLimit, RateLimitGroup, RateLimitKind, RateLimitTarget, TransactionIdentifier,
};
use pallet_subtensor::{
    self, AssociatedEvmAddress, Axons, Config as SubtensorConfig, HasMigrationRun,
    LastRateLimitedBlock, LastUpdate, MaxUidsTrimmingRateLimit, MechanismCountSetRateLimit,
    MechanismEmissionRateLimit, NetworkRateLimit, OwnerHyperparamRateLimit, Pallet, Prometheus,
    RateLimitKey, ServingRateLimit, TransactionKeyLastBlock, TxChildkeyTakeRateLimit,
    TxDelegateTakeRateLimit, TxRateLimit, WeightsVersionKeyRateLimit,
    utils::rate_limiting::{Hyperparameter, TransactionType},
};
use sp_runtime::traits::SaturatedConversion;
use sp_std::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    vec,
    vec::Vec,
};
use subtensor_runtime_common::NetUid;

use super::{AccountId, RateLimitUsageKey, Runtime};

type GroupId = <Runtime as pallet_rate_limiting::Config>::GroupId;
type GroupNameOf<T> = BoundedVec<u8, <T as pallet_rate_limiting::Config>::MaxGroupNameLength>;
type GroupMembersOf<T> =
    BoundedBTreeSet<TransactionIdentifier, <T as pallet_rate_limiting::Config>::MaxGroupMembers>;

// Pallet index assigned to `pallet_subtensor` in `construct_runtime!`.
const SUBTENSOR_PALLET_INDEX: u8 = 7;
// Pallet index assigned to `pallet_admin_utils` in `construct_runtime!`.
const ADMIN_UTILS_PALLET_INDEX: u8 = 19;

/// Marker stored in `pallet_subtensor::HasMigrationRun` once the migration finishes.
pub const MIGRATION_NAME: &[u8] = b"migrate_rate_limiting";

const GROUP_SERVE: GroupId = 0;
const GROUP_DELEGATE_TAKE: GroupId = 1;
const GROUP_WEIGHTS_SUBNET: GroupId = 2;
pub const GROUP_REGISTER_NETWORK: GroupId = 3;
const GROUP_OWNER_HPARAMS: GroupId = 4;
const GROUP_STAKING_OPS: GroupId = 5;

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
    let mut reads: u64 = 0;

    // grouped
    reads += build_serving(&mut groups, &mut commits);
    reads += build_delegate_take(&mut groups, &mut commits);
    reads += build_weights(&mut groups, &mut commits);
    reads += build_register_network(&mut groups, &mut commits);
    reads += build_owner_hparams(&mut groups, &mut commits);
    reads += build_staking_ops(&mut groups, &mut commits);

    // standalone
    reads += build_swap_hotkey(&mut commits);
    reads += build_childkey_take(&mut commits);
    reads += build_set_children(&mut commits);
    reads += build_weights_version_key(&mut commits);
    reads += build_sn_owner_hotkey(&mut commits);
    reads += build_associate_evm(&mut commits);
    reads += build_mechanism_count(&mut commits);
    reads += build_mechanism_emission(&mut commits);
    reads += build_trim_max_uids(&mut commits);

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

    // Limits per netuid (written to the group target).
    reads += 1;
    // Merge live subnets (which may rely on default rate-limit values) with any legacy entries that
    // exist only in storage, so we migrate both current and previously stored netuids without
    // duplicates.
    let mut netuids = Pallet::<Runtime>::get_all_subnet_netuids();
    for (netuid, _) in ServingRateLimit::<Runtime>::iter() {
        if !netuids.contains(&netuid) {
            netuids.push(netuid);
        }
    }
    for netuid in netuids {
        reads += 1;
        push_limit_commit_if_non_zero(
            commits,
            RateLimitTarget::Group(GROUP_SERVE),
            Pallet::<Runtime>::get_serving_rate_limit(netuid),
            Some(netuid),
        );
    }

    // Axon last-seen (group-shared usage).
    for (netuid, hotkey, axon) in Axons::<Runtime>::iter() {
        reads += 1;
        if let Some(block) = block_number::<Runtime>(axon.block) {
            commits.push(Commit {
                target: RateLimitTarget::Group(GROUP_SERVE),
                kind: CommitKind::LastSeen(MigratedLastSeen {
                    block,
                    usage: Some(RateLimitUsageKey::AccountSubnetServing {
                        account: hotkey.clone(),
                        netuid,
                        endpoint: crate::rate_limiting::ServingEndpoint::Axon,
                    }),
                }),
            });
        }
    }

    // Prometheus last-seen (group-shared usage).
    for (netuid, hotkey, prom) in Prometheus::<Runtime>::iter() {
        reads += 1;
        if let Some(block) = block_number::<Runtime>(prom.block) {
            commits.push(Commit {
                target: RateLimitTarget::Group(GROUP_SERVE),
                kind: CommitKind::LastSeen(MigratedLastSeen {
                    block,
                    usage: Some(RateLimitUsageKey::AccountSubnetServing {
                        account: hotkey,
                        netuid,
                        endpoint: crate::rate_limiting::ServingEndpoint::Prometheus,
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
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        TxDelegateTakeRateLimit::<Runtime>::get(),
        None,
    );

    reads +=
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::LastTxBlockDelegateTake(account) => {
                    Some((target, Some(RateLimitUsageKey::Account(account))))
                }
                _ => None,
            },
        );

    reads
}

// Weights group (config + usage shared).
// scope: netuid
// usage: netuid+neuron/netuid+mechanism+neuron
// legacy source: SubnetWeightsSetRateLimit, LastUpdate (subnet/mechanism)
fn build_weights(groups: &mut Vec<GroupConfig>, commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    groups.push(GroupConfig {
        id: GROUP_WEIGHTS_SUBNET,
        name: b"weights".to_vec(),
        sharing: GroupSharing::ConfigAndUsage,
        members: vec![
            MigratedCall::subtensor(0, false),   // set_weights
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

    reads += 1;
    for netuid in Pallet::<Runtime>::get_all_subnet_netuids() {
        reads += 1;
        push_limit_commit_if_non_zero(
            commits,
            RateLimitTarget::Group(GROUP_WEIGHTS_SUBNET),
            Pallet::<Runtime>::get_weights_set_rate_limit(netuid),
            Some(netuid),
        );
    }

    for (index, blocks) in LastUpdate::<Runtime>::iter() {
        reads += 1;
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
    reads += 1;
    push_limit_commit_if_non_zero(commits, target, NetworkRateLimit::<Runtime>::get(), None);

    reads +=
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::NetworkLastRegistered => Some((target, None)),
                _ => None,
            },
        );

    reads
}

// Owner hyperparameter group (config shared, usage per call).
// usage: netuid
// legacy sources: OwnerHyperparamRateLimit * tempo,
// 				   LastRateLimitedBlock per OwnerHyperparamUpdate
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
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        group_target,
        u64::from(OwnerHyperparamRateLimit::<Runtime>::get()),
        None,
    );

    reads +=
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

// Standalone swap_hotkey.
// usage: account
// legacy sources: TxRateLimit, LastRateLimitedBlock per LastTxBlock
fn build_swap_hotkey(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(SUBTENSOR_PALLET_INDEX, 70));

    reads += 1;
    push_limit_commit_if_non_zero(commits, target, TxRateLimit::<Runtime>::get(), None);

    reads +=
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::LastTxBlock(account) => {
                    Some((target, Some(RateLimitUsageKey::Account(account))))
                }
                _ => None,
            },
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
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        TxChildkeyTakeRateLimit::<Runtime>::get(),
        None,
    );

    reads += last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
        commits,
        target,
        TransactionType::SetChildkeyTake,
    );

    reads
}

// Standalone set_children.
// usage: account+netuid
// legacy sources: SET_CHILDREN_RATE_LIMIT (constant 150),
//                 TransactionKeyLastBlock per SetChildren
fn build_set_children(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(SUBTENSOR_PALLET_INDEX, 67));
    push_limit_commit_if_non_zero(commits, target, SET_CHILDREN_RATE_LIMIT, None);

    reads += last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
        commits,
        target,
        TransactionType::SetChildren,
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
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        WeightsVersionKeyRateLimit::<Runtime>::get(),
        None,
    );

    reads += last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
        commits,
        target,
        TransactionType::SetWeightsVersionKey,
    );

    reads
}

// Standalone set_sn_owner_hotkey.
// usage: netuid
// legacy sources: DefaultSetSNOwnerHotkeyRateLimit,
//                 LastRateLimitedBlock per SetSNOwnerHotkey
fn build_sn_owner_hotkey(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 67));
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        pallet_subtensor::pallet::DefaultSetSNOwnerHotkeyRateLimit::<Runtime>::get(),
        None,
    );

    reads +=
        last_seen_helpers::collect_last_seen_from_last_rate_limited_block(
            commits,
            |key| match key {
                RateLimitKey::SetSNOwnerHotkey(netuid) => {
                    Some((target, Some(RateLimitUsageKey::Subnet(netuid))))
                }
                _ => None,
            },
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
        reads += 1;
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
// legacy sources: MechanismCountSetRateLimit,
//                 TransactionKeyLastBlock per MechanismCountUpdate
// sudo_set_mechanism_count
fn build_mechanism_count(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 76));
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        MechanismCountSetRateLimit::<Runtime>::get(),
        None,
    );

    reads += last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
        commits,
        target,
        TransactionType::MechanismCountUpdate,
    );

    reads
}

// Standalone mechanism emission.
// usage: account+netuid
// legacy sources: MechanismEmissionRateLimit,
// 				   TransactionKeyLastBlock per MechanismEmission
// sudo_set_mechanism_emission_split
fn build_mechanism_emission(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 77));
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        MechanismEmissionRateLimit::<Runtime>::get(),
        None,
    );

    reads += last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
        commits,
        target,
        TransactionType::MechanismEmission,
    );

    reads
}

// Standalone trim_to_max_allowed_uids.
// usage: account+netuid
// legacy sources: MaxUidsTrimmingRateLimit,
// 				   TransactionKeyLastBlock per MaxUidsTrimming
// sudo_trim_to_max_allowed_uids
fn build_trim_max_uids(commits: &mut Vec<Commit>) -> u64 {
    let mut reads: u64 = 0;
    let target =
        RateLimitTarget::Transaction(TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, 78));
    reads += 1;
    push_limit_commit_if_non_zero(
        commits,
        target,
        MaxUidsTrimmingRateLimit::<Runtime>::get(),
        None,
    );

    reads += last_seen_helpers::collect_last_seen_from_transaction_key_last_block(
        commits,
        target,
        TransactionType::MaxUidsTrimming,
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

        for (key, block) in LastRateLimitedBlock::<Runtime>::iter() {
            reads += 1;
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

        for ((account, netuid, tx_kind), block) in TransactionKeyLastBlock::<Runtime>::iter() {
            reads += 1;
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
mod tests {
    use sp_io::TestExternalities;
    use sp_runtime::traits::{SaturatedConversion, Zero};

    use super::*;
    use crate::BuildStorage;

    const ACCOUNT: [u8; 32] = [7u8; 32];
    const DELEGATE_TAKE_GROUP_ID: GroupId = GROUP_DELEGATE_TAKE;

    fn new_test_ext() -> TestExternalities {
        sp_tracing::try_init_simple();
        let mut ext: TestExternalities = crate::RuntimeGenesisConfig::default()
            .build_storage()
            .expect("runtime storage")
            .into();
        ext.execute_with(|| crate::System::set_block_number(1));
        ext
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

            pallet_subtensor::TxRateLimit::<Runtime>::put(10);
            pallet_subtensor::TxDelegateTakeRateLimit::<Runtime>::put(3);
            pallet_subtensor::LastRateLimitedBlock::<Runtime>::insert(
                RateLimitKey::LastTxBlock(account.clone()),
                5,
            );

            let weight = migrate_rate_limiting();
            assert!(!weight.is_zero());
            assert!(pallet_subtensor::HasMigrationRun::<Runtime>::get(
                MIGRATION_NAME
            ));

            let tx_target =
                RateLimitTarget::Transaction(MigratedCall::subtensor(70, false).identifier());
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
            assert_eq!(pallet_rate_limiting::NextGroupId::<Runtime>::get(), 6);
        });
    }

    #[test]
    fn migration_skips_when_already_run() {
        new_test_ext().execute_with(|| {
            pallet_subtensor::HasMigrationRun::<Runtime>::insert(MIGRATION_NAME, true);
            pallet_subtensor::TxRateLimit::<Runtime>::put(99);

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
