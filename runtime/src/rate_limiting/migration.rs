use core::{convert::TryFrom, marker::PhantomData};

use frame_support::{
    BoundedBTreeSet, BoundedVec, pallet_prelude::Parameter, traits::Get, weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
use log::{info, warn};
use pallet_rate_limiting::{
    GroupSharing, RateLimit, RateLimitGroup, RateLimitKind, RateLimitTarget, TransactionIdentifier,
};
use pallet_subtensor::{
    self, AssociatedEvmAddress, Axons, Config as SubtensorConfig, HasMigrationRun,
    LastRateLimitedBlock, LastUpdate, MaxUidsTrimmingRateLimit, MechanismCountCurrent,
    MechanismCountSetRateLimit, MechanismEmissionRateLimit, NetworkRateLimit,
    OwnerHyperparamRateLimit, Pallet, Prometheus, RateLimitKey, TransactionKeyLastBlock,
    TxChildkeyTakeRateLimit, TxDelegateTakeRateLimit, TxRateLimit, WeightsVersionKeyRateLimit,
    utils::rate_limiting::{Hyperparameter, TransactionType},
};
use sp_runtime::traits::SaturatedConversion;
use sp_std::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    vec,
    vec::Vec,
};
use subtensor_runtime_common::{MechId, NetUid, RateLimitScope, RateLimitUsageKey};

use crate::RateLimitingInstance;

type GroupIdOf<T> = <T as pallet_rate_limiting::Config<RateLimitingInstance>>::GroupId;
type LimitEntries<T> = Vec<(
    RateLimitTarget<GroupId>,
    RateLimit<RateLimitScope, BlockNumberFor<T>>,
)>;
type LastSeenEntries<T> = Vec<(
    (
        RateLimitTarget<GroupId>,
        Option<RateLimitUsageKey<<T as frame_system::Config>::AccountId>>,
    ),
    BlockNumberFor<T>,
)>;
type GroupNameOf<T> =
    BoundedVec<u8, <T as pallet_rate_limiting::Config<RateLimitingInstance>>::MaxGroupNameLength>;
type GroupMembersOf<T> = BoundedBTreeSet<
    TransactionIdentifier,
    <T as pallet_rate_limiting::Config<RateLimitingInstance>>::MaxGroupMembers,
>;

/// Pallet index assigned to `pallet_subtensor` in `construct_runtime!`.
const SUBTENSOR_PALLET_INDEX: u8 = 7;
/// Pallet index assigned to `pallet_admin_utils` in `construct_runtime!`.
const ADMIN_UTILS_PALLET_INDEX: u8 = 19;

/// Marker stored in `HasMigrationRun` once the migration finishes.
const MIGRATION_NAME: &[u8] = b"migrate_rate_limiting";

/// `set_children` is rate-limited to once every 150 blocks.
const SET_CHILDREN_RATE_LIMIT: u64 = 150;
/// `set_sn_owner_hotkey` default interval (blocks).
const DEFAULT_SET_SN_OWNER_HOTKEY_LIMIT: u64 = 50_400;

type GroupId = u32;

struct GroupDefinition {
    id: GroupId,
    name: &'static [u8],
    sharing: GroupSharing,
    members: Vec<TransactionIdentifier>,
}

const GROUP_SERVE_AXON: GroupId = 0;
const GROUP_DELEGATE_TAKE: GroupId = 1;
const GROUP_WEIGHTS_SUBNET: GroupId = 2;
const GROUP_WEIGHTS_MECHANISM: GroupId = 3;
const GROUP_REGISTER_NETWORK: GroupId = 4;
const GROUP_OWNER_HPARAMS: GroupId = 5;

fn hyperparameter_identifiers() -> Vec<TransactionIdentifier> {
    HYPERPARAMETERS
        .iter()
        .filter_map(|h| identifier_for_hyperparameter(*h))
        .collect()
}

fn group_definitions() -> Vec<GroupDefinition> {
    vec![
        GroupDefinition {
            id: GROUP_SERVE_AXON,
            name: b"serve-axon",
            sharing: GroupSharing::ConfigAndUsage,
            members: vec![subtensor_identifier(4), subtensor_identifier(40)],
        },
        GroupDefinition {
            id: GROUP_DELEGATE_TAKE,
            name: b"delegate-take",
            sharing: GroupSharing::ConfigAndUsage,
            members: vec![subtensor_identifier(66), subtensor_identifier(65)],
        },
        GroupDefinition {
            id: GROUP_WEIGHTS_SUBNET,
            name: b"weights-subnet",
            sharing: GroupSharing::ConfigAndUsage,
            members: vec![
                subtensor_identifier(0),
                subtensor_identifier(96),
                subtensor_identifier(100),
                subtensor_identifier(113),
            ],
        },
        GroupDefinition {
            id: GROUP_WEIGHTS_MECHANISM,
            name: b"weights-mechanism",
            sharing: GroupSharing::ConfigAndUsage,
            members: vec![
                subtensor_identifier(119),
                subtensor_identifier(115),
                subtensor_identifier(117),
                subtensor_identifier(118),
            ],
        },
        GroupDefinition {
            id: GROUP_REGISTER_NETWORK,
            name: b"register-network",
            sharing: GroupSharing::ConfigAndUsage,
            members: vec![subtensor_identifier(59), subtensor_identifier(79)],
        },
        GroupDefinition {
            id: GROUP_OWNER_HPARAMS,
            name: b"owner-hparams",
            sharing: GroupSharing::ConfigOnly,
            members: hyperparameter_identifiers(),
        },
    ]
}

/// Hyperparameter extrinsics routed through owner-or-root rate limiting.
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

#[derive(Clone, Copy)]
struct GroupInfo {
    id: GroupId,
    sharing: GroupSharing,
}

#[derive(Default)]
struct Grouping {
    assignments: BTreeMap<TransactionIdentifier, GroupInfo>,
    members: BTreeMap<GroupId, BTreeSet<TransactionIdentifier>>,
    details: Vec<RateLimitGroup<GroupId, Vec<u8>>>,
    next_group_id: GroupId,
    max_group_id: Option<GroupId>,
}

impl Grouping {
    fn members(&self, id: GroupId) -> Option<&BTreeSet<TransactionIdentifier>> {
        self.members.get(&id)
    }

    fn insert_group(
        &mut self,
        id: GroupId,
        name: &[u8],
        sharing: GroupSharing,
        members: &[TransactionIdentifier],
    ) {
        let entry = self.members.entry(id).or_insert_with(BTreeSet::new);
        for member in members {
            self.assignments.insert(*member, GroupInfo { id, sharing });
            entry.insert(*member);
        }

        self.details.push(RateLimitGroup {
            id,
            name: name.to_vec(),
            sharing,
        });

        self.max_group_id = Some(self.max_group_id.map_or(id, |current| current.max(id)));
    }

    fn finalize_next_id(&mut self) {
        self.next_group_id = self.max_group_id.map_or(0, |id| id.saturating_add(1));
    }

    fn config_target(&self, identifier: TransactionIdentifier) -> RateLimitTarget<GroupId> {
        if let Some(info) = self.assignments.get(&identifier) {
            if info.sharing.config_uses_group() {
                return RateLimitTarget::Group(info.id);
            }
        }
        RateLimitTarget::Transaction(identifier)
    }

    fn usage_target(&self, identifier: TransactionIdentifier) -> RateLimitTarget<GroupId> {
        if let Some(info) = self.assignments.get(&identifier) {
            if info.sharing.usage_uses_group() {
                return RateLimitTarget::Group(info.id);
            }
        }
        RateLimitTarget::Transaction(identifier)
    }
}

const SERVE_PROM_IDENTIFIER: TransactionIdentifier = subtensor_identifier(5);

fn serve_calls(grouping: &Grouping) -> Vec<TransactionIdentifier> {
    let mut calls = Vec::new();
    if let Some(members) = grouping.members(GROUP_SERVE_AXON) {
        calls.extend(members.iter().copied());
    }
    calls.push(SERVE_PROM_IDENTIFIER);
    calls
}

fn weight_calls_subnet(grouping: &Grouping) -> Vec<TransactionIdentifier> {
    grouping
        .members(GROUP_WEIGHTS_SUBNET)
        .map(|m| m.iter().copied().collect())
        .unwrap_or_default()
}

fn weight_calls_mechanism(grouping: &Grouping) -> Vec<TransactionIdentifier> {
    grouping
        .members(GROUP_WEIGHTS_MECHANISM)
        .map(|m| m.iter().copied().collect())
        .unwrap_or_default()
}

fn build_grouping() -> Grouping {
    let mut grouping = Grouping::default();

    for definition in group_definitions() {
        grouping.insert_group(
            definition.id,
            definition.name,
            definition.sharing,
            &definition.members,
        );
    }

    grouping.finalize_next_id();
    grouping
}

pub fn migrate_rate_limiting<T>() -> Weight
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            RateLimitingInstance,
            LimitScope = RateLimitScope,
            GroupId = GroupId,
        >,
    RateLimitUsageKey<T::AccountId>:
        Into<<T as pallet_rate_limiting::Config<RateLimitingInstance>>::UsageKey>,
{
    let mut weight = T::DbWeight::get().reads(1);
    if HasMigrationRun::<T>::get(MIGRATION_NAME) {
        info!("Rate-limiting migration already executed. Skipping.");
        return weight;
    }

    let grouping = build_grouping();
    let (limits, limit_reads) = build_limits::<T>(&grouping);
    let (last_seen, seen_reads) = build_last_seen::<T>(&grouping);

    let limit_writes = write_limits::<T>(&limits);
    let seen_writes = write_last_seen::<T>(&last_seen);
    let group_writes = write_groups::<T>(&grouping);

    HasMigrationRun::<T>::insert(MIGRATION_NAME, true);

    weight = weight
        .saturating_add(T::DbWeight::get().reads(limit_reads.saturating_add(seen_reads)))
        .saturating_add(
            T::DbWeight::get().writes(
                limit_writes
                    .saturating_add(seen_writes)
                    .saturating_add(group_writes)
                    .saturating_add(1),
            ),
        );

    info!(
        "Migrated {} rate-limit configs, {} last-seen entries, and {} groups into pallet-rate-limiting",
        limits.len(),
        last_seen.len(),
        grouping.details.len()
    );

    weight
}

fn build_limits<T: SubtensorConfig>(grouping: &Grouping) -> (LimitEntries<T>, u64) {
    let mut limits = LimitEntries::<T>::new();
    let mut reads: u64 = 0;

    reads += gather_simple_limits::<T>(&mut limits, grouping);
    reads += gather_owner_hparam_limits::<T>(&mut limits, grouping);
    reads += gather_serving_limits::<T>(&mut limits, grouping);
    reads += gather_weight_limits::<T>(&mut limits, grouping);

    (limits, reads)
}

fn gather_simple_limits<T: SubtensorConfig>(
    limits: &mut LimitEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;

    reads += 1;
    if let Some(span) = block_number::<T>(TxRateLimit::<T>::get()) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(subtensor_identifier(70)),
            span,
        );
    }

    reads += 1;
    if let Some(span) = block_number::<T>(TxDelegateTakeRateLimit::<T>::get()) {
        if let Some(members) = grouping.members(GROUP_DELEGATE_TAKE) {
            for call in members {
                set_global_limit::<T>(limits, grouping.config_target(*call), span);
            }
        }
    }

    reads += 1;
    if let Some(span) = block_number::<T>(TxChildkeyTakeRateLimit::<T>::get()) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(subtensor_identifier(75)),
            span,
        );
    }

    reads += 1;
    if let Some(span) = block_number::<T>(NetworkRateLimit::<T>::get()) {
        if let Some(members) = grouping.members(GROUP_REGISTER_NETWORK) {
            for call in members {
                set_global_limit::<T>(limits, grouping.config_target(*call), span);
            }
        }
    }

    reads += 1;
    if let Some(span) = block_number::<T>(WeightsVersionKeyRateLimit::<T>::get()) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(admin_utils_identifier(6)),
            span,
        );
    }

    if let Some(span) = block_number::<T>(DEFAULT_SET_SN_OWNER_HOTKEY_LIMIT) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(admin_utils_identifier(67)),
            span,
        );
    }

    if let Some(span) = block_number::<T>(<T as SubtensorConfig>::EvmKeyAssociateRateLimit::get()) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(subtensor_identifier(93)),
            span,
        );
    }

    if let Some(span) = block_number::<T>(MechanismCountSetRateLimit::<T>::get()) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(admin_utils_identifier(76)),
            span,
        );
    }

    if let Some(span) = block_number::<T>(MechanismEmissionRateLimit::<T>::get()) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(admin_utils_identifier(77)),
            span,
        );
    }

    if let Some(span) = block_number::<T>(MaxUidsTrimmingRateLimit::<T>::get()) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(admin_utils_identifier(78)),
            span,
        );
    }

    if let Some(span) = block_number::<T>(SET_CHILDREN_RATE_LIMIT) {
        set_global_limit::<T>(
            limits,
            grouping.config_target(subtensor_identifier(67)),
            span,
        );
    }

    reads
}

fn gather_owner_hparam_limits<T: SubtensorConfig>(
    limits: &mut LimitEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;

    reads += 1;
    if let Some(span) = block_number::<T>(u64::from(OwnerHyperparamRateLimit::<T>::get())) {
        for hparam in HYPERPARAMETERS {
            if let Some(identifier) = identifier_for_hyperparameter(*hparam) {
                set_global_limit::<T>(limits, grouping.config_target(identifier), span);
            }
        }
    }

    reads
}

fn gather_serving_limits<T: SubtensorConfig>(
    limits: &mut LimitEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;
    let netuids = Pallet::<T>::get_all_subnet_netuids();

    for netuid in netuids {
        reads += 1;
        if let Some(span) = block_number::<T>(Pallet::<T>::get_serving_rate_limit(netuid)) {
            for call in serve_calls(grouping) {
                set_scoped_limit::<T>(
                    limits,
                    grouping.config_target(call),
                    RateLimitScope::Subnet(netuid),
                    span,
                );
            }
        }
    }

    reads
}

fn gather_weight_limits<T: SubtensorConfig>(
    limits: &mut LimitEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;
    let netuids = Pallet::<T>::get_all_subnet_netuids();

    let mut subnet_limits = BTreeMap::<NetUid, BlockNumberFor<T>>::new();
    let subnet_calls = weight_calls_subnet(grouping);
    let mechanism_calls = weight_calls_mechanism(grouping);
    for netuid in &netuids {
        reads += 1;
        if let Some(span) = block_number::<T>(Pallet::<T>::get_weights_set_rate_limit(*netuid)) {
            subnet_limits.insert(*netuid, span);
            for call in &subnet_calls {
                set_scoped_limit::<T>(
                    limits,
                    grouping.config_target(*call),
                    RateLimitScope::Subnet(*netuid),
                    span,
                );
            }
        }
    }

    for netuid in &netuids {
        reads += 1;
        let mech_count: u8 = MechanismCountCurrent::<T>::get(*netuid).into();
        if mech_count <= 1 {
            continue;
        }
        let Some(span) = subnet_limits.get(netuid).copied() else {
            continue;
        };
        for mecid in 1..mech_count {
            let scope = RateLimitScope::SubnetMechanism {
                netuid: *netuid,
                mecid: MechId::from(mecid),
            };
            for call in &mechanism_calls {
                set_scoped_limit::<T>(limits, grouping.config_target(*call), scope.clone(), span);
            }
        }
    }

    reads
}

fn build_last_seen<T: SubtensorConfig>(grouping: &Grouping) -> (LastSeenEntries<T>, u64) {
    let mut last_seen = LastSeenEntries::<T>::new();
    let mut reads: u64 = 0;

    reads += import_last_rate_limited_blocks::<T>(&mut last_seen, grouping);
    reads += import_transaction_key_last_blocks::<T>(&mut last_seen, grouping);
    reads += import_last_update_entries::<T>(&mut last_seen, grouping);
    reads += import_serving_entries::<T>(&mut last_seen, grouping);
    reads += import_evm_entries::<T>(&mut last_seen, grouping);

    (last_seen, reads)
}

fn import_last_rate_limited_blocks<T: SubtensorConfig>(
    entries: &mut LastSeenEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;
    for (key, block) in LastRateLimitedBlock::<T>::iter() {
        reads += 1;
        if block == 0 {
            continue;
        }
        match key {
            RateLimitKey::SetSNOwnerHotkey(netuid) => {
                if let Some(identifier) =
                    identifier_for_transaction_type(TransactionType::SetSNOwnerHotkey)
                {
                    record_last_seen_entry::<T>(
                        entries,
                        grouping.usage_target(identifier),
                        Some(RateLimitUsageKey::Subnet(netuid)),
                        block,
                    );
                }
            }
            RateLimitKey::OwnerHyperparamUpdate(netuid, hyper) => {
                if let Some(identifier) = identifier_for_hyperparameter(hyper) {
                    record_last_seen_entry::<T>(
                        entries,
                        grouping.usage_target(identifier),
                        Some(RateLimitUsageKey::Subnet(netuid)),
                        block,
                    );
                }
            }
            RateLimitKey::LastTxBlock(account) => {
                record_last_seen_entry::<T>(
                    entries,
                    grouping.usage_target(subtensor_identifier(70)),
                    Some(RateLimitUsageKey::Account(account.clone())),
                    block,
                );
            }
            RateLimitKey::LastTxBlockDelegateTake(account) => {
                record_last_seen_entry::<T>(
                    entries,
                    grouping.usage_target(subtensor_identifier(66)),
                    Some(RateLimitUsageKey::Account(account.clone())),
                    block,
                );
            }
            RateLimitKey::NetworkLastRegistered => {
                record_last_seen_entry::<T>(
                    entries,
                    grouping.usage_target(subtensor_identifier(59)),
                    None,
                    block,
                );
            }
            RateLimitKey::LastTxBlockChildKeyTake(_) => {
                // Deprecated storage; ignored.
            }
        }
    }
    reads
}

fn import_transaction_key_last_blocks<T: SubtensorConfig>(
    entries: &mut LastSeenEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;
    for ((account, netuid, tx_kind), block) in TransactionKeyLastBlock::<T>::iter() {
        reads += 1;
        if block == 0 {
            continue;
        }
        let tx_type = TransactionType::from(tx_kind);
        let Some(identifier) = identifier_for_transaction_type(tx_type) else {
            continue;
        };
        let Some(usage) = usage_key_from_transaction_type(tx_type, &account, netuid) else {
            continue;
        };
        record_last_seen_entry::<T>(
            entries,
            grouping.usage_target(identifier),
            Some(usage),
            block,
        );
    }
    reads
}

fn import_last_update_entries<T: SubtensorConfig>(
    entries: &mut LastSeenEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;
    let subnet_calls = weight_calls_subnet(grouping);
    let mechanism_calls = weight_calls_mechanism(grouping);
    for (index, blocks) in LastUpdate::<T>::iter() {
        reads += 1;
        let netuid = Pallet::<T>::get_netuid(index);
        let sub_id = u16::from(index)
            .checked_div(pallet_subtensor::subnets::mechanism::GLOBAL_MAX_SUBNET_COUNT)
            .unwrap_or_default();
        let is_mechanism = sub_id != 0;
        let Ok(sub_id) = u8::try_from(sub_id) else {
            continue;
        };
        let mecid = MechId::from(sub_id);

        for (uid, last_block) in blocks.into_iter().enumerate() {
            if last_block == 0 {
                continue;
            }
            let Ok(uid_u16) = u16::try_from(uid) else {
                continue;
            };
            let usage = if is_mechanism {
                RateLimitUsageKey::SubnetMechanismNeuron {
                    netuid,
                    mecid,
                    uid: uid_u16,
                }
            } else {
                RateLimitUsageKey::SubnetNeuron {
                    netuid,
                    uid: uid_u16,
                }
            };

            let call_set: &[TransactionIdentifier] = if is_mechanism {
                mechanism_calls.as_slice()
            } else {
                subnet_calls.as_slice()
            };

            for call in call_set {
                record_last_seen_entry::<T>(
                    entries,
                    grouping.usage_target(*call),
                    Some(usage.clone()),
                    last_block,
                );
            }
        }
    }
    reads
}

fn import_serving_entries<T: SubtensorConfig>(
    entries: &mut LastSeenEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;
    for (netuid, hotkey, axon) in Axons::<T>::iter() {
        reads += 1;
        if axon.block == 0 {
            continue;
        }
        let usage = RateLimitUsageKey::AccountSubnet {
            account: hotkey.clone(),
            netuid,
        };
        let axon_calls: Vec<_> = grouping
            .members(GROUP_SERVE_AXON)
            .map(|m| m.iter().copied().collect())
            .unwrap_or_else(|| vec![subtensor_identifier(4), subtensor_identifier(40)]);
        for call in axon_calls {
            record_last_seen_entry::<T>(
                entries,
                grouping.usage_target(call),
                Some(usage.clone()),
                axon.block,
            );
        }
    }

    for (netuid, hotkey, prom) in Prometheus::<T>::iter() {
        reads += 1;
        if prom.block == 0 {
            continue;
        }
        let usage = RateLimitUsageKey::AccountSubnet {
            account: hotkey,
            netuid,
        };
        record_last_seen_entry::<T>(
            entries,
            grouping.usage_target(SERVE_PROM_IDENTIFIER),
            Some(usage),
            prom.block,
        );
    }

    reads
}

fn import_evm_entries<T: SubtensorConfig>(
    entries: &mut LastSeenEntries<T>,
    grouping: &Grouping,
) -> u64 {
    let mut reads: u64 = 0;
    for (netuid, uid, (_, block)) in AssociatedEvmAddress::<T>::iter() {
        reads += 1;
        if block == 0 {
            continue;
        }
        record_last_seen_entry::<T>(
            entries,
            grouping.usage_target(subtensor_identifier(93)),
            Some(RateLimitUsageKey::SubnetNeuron { netuid, uid }),
            block,
        );
    }
    reads
}

fn convert_target<T>(target: &RateLimitTarget<GroupId>) -> RateLimitTarget<GroupIdOf<T>>
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            RateLimitingInstance,
            LimitScope = RateLimitScope,
            GroupId = GroupId,
        >,
    RateLimitUsageKey<T::AccountId>:
        Into<<T as pallet_rate_limiting::Config<RateLimitingInstance>>::UsageKey>,
{
    match target {
        RateLimitTarget::Transaction(identifier) => RateLimitTarget::Transaction(*identifier),
        RateLimitTarget::Group(id) => RateLimitTarget::Group((*id).saturated_into()),
    }
}

fn write_limits<T>(limits: &LimitEntries<T>) -> u64
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            RateLimitingInstance,
            LimitScope = RateLimitScope,
            GroupId = GroupId,
        >,
    RateLimitUsageKey<T::AccountId>:
        Into<<T as pallet_rate_limiting::Config<RateLimitingInstance>>::UsageKey>,
{
    let mut writes: u64 = 0;
    for (identifier, limit) in limits.iter() {
        let target = convert_target::<T>(identifier);
        pallet_rate_limiting::Limits::<T, RateLimitingInstance>::insert(target, limit.clone());
        writes += 1;
    }
    writes
}

fn write_last_seen<T>(entries: &LastSeenEntries<T>) -> u64
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            RateLimitingInstance,
            LimitScope = RateLimitScope,
            GroupId = GroupId,
        >,
    RateLimitUsageKey<T::AccountId>:
        Into<<T as pallet_rate_limiting::Config<RateLimitingInstance>>::UsageKey>,
{
    let mut writes: u64 = 0;
    for ((identifier, usage), block) in entries.iter() {
        let target = convert_target::<T>(identifier);
        let usage_key = usage.clone().map(Into::into);
        pallet_rate_limiting::LastSeen::<T, RateLimitingInstance>::insert(
            target, usage_key, *block,
        );
        writes += 1;
    }
    writes
}

fn write_groups<T>(grouping: &Grouping) -> u64
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            RateLimitingInstance,
            LimitScope = RateLimitScope,
            GroupId = GroupId,
        >,
    RateLimitUsageKey<T::AccountId>:
        Into<<T as pallet_rate_limiting::Config<RateLimitingInstance>>::UsageKey>,
{
    let mut writes: u64 = 0;

    for detail in &grouping.details {
        let Ok(name) = GroupNameOf::<T>::try_from(detail.name.clone()) else {
            warn!(
                "rate-limiting migration: group name exceeds bounds, skipping id {}",
                detail.id
            );
            continue;
        };
        let group_id = detail.id.saturated_into::<GroupIdOf<T>>();
        let stored = RateLimitGroup {
            id: group_id,
            name: name.clone(),
            sharing: detail.sharing,
        };

        pallet_rate_limiting::Groups::<T, RateLimitingInstance>::insert(group_id, stored);
        pallet_rate_limiting::GroupNameIndex::<T, RateLimitingInstance>::insert(name, group_id);
        writes += 2;
    }

    for (group, members) in &grouping.members {
        let group_id = (*group).saturated_into::<GroupIdOf<T>>();
        let Ok(bounded) = GroupMembersOf::<T>::try_from(members.clone()) else {
            warn!(
                "rate-limiting migration: group {} has too many members, skipping assignment",
                group
            );
            continue;
        };
        pallet_rate_limiting::GroupMembers::<T, RateLimitingInstance>::insert(group_id, bounded);
        writes += 1;
    }

    for (identifier, info) in &grouping.assignments {
        let group_id = info.id.saturated_into::<GroupIdOf<T>>();
        pallet_rate_limiting::CallGroups::<T, RateLimitingInstance>::insert(*identifier, group_id);
        writes += 1;
    }

    let next_group_id = grouping.next_group_id.saturated_into::<GroupIdOf<T>>();
    pallet_rate_limiting::NextGroupId::<T, RateLimitingInstance>::put(next_group_id);
    writes += 1;

    writes
}

fn block_number<T: SubtensorConfig>(value: u64) -> Option<BlockNumberFor<T>> {
    if value == 0 {
        return None;
    }
    Some(value.saturated_into::<BlockNumberFor<T>>())
}

fn set_global_limit<T: SubtensorConfig>(
    limits: &mut LimitEntries<T>,
    target: RateLimitTarget<GroupId>,
    span: BlockNumberFor<T>,
) {
    if let Some((_, config)) = limits.iter_mut().find(|(id, _)| *id == target) {
        *config = RateLimit::global(RateLimitKind::Exact(span));
    } else {
        limits.push((target, RateLimit::global(RateLimitKind::Exact(span))));
    }
}

fn set_scoped_limit<T: SubtensorConfig>(
    limits: &mut LimitEntries<T>,
    target: RateLimitTarget<GroupId>,
    scope: RateLimitScope,
    span: BlockNumberFor<T>,
) {
    if let Some((_, config)) = limits.iter_mut().find(|(id, _)| *id == target) {
        match config {
            RateLimit::Global(_) => {
                *config = RateLimit::scoped_single(scope, RateLimitKind::Exact(span));
            }
            RateLimit::Scoped(map) => {
                map.insert(scope, RateLimitKind::Exact(span));
            }
        }
    } else {
        limits.push((
            target,
            RateLimit::scoped_single(scope, RateLimitKind::Exact(span)),
        ));
    }
}

fn record_last_seen_entry<T: SubtensorConfig>(
    entries: &mut LastSeenEntries<T>,
    target: RateLimitTarget<GroupId>,
    usage: Option<RateLimitUsageKey<T::AccountId>>,
    block: u64,
) {
    let Some(block_number) = block_number::<T>(block) else {
        return;
    };

    let key = (target, usage);
    if let Some((_, existing)) = entries.iter_mut().find(|(entry_key, _)| *entry_key == key) {
        if block_number > *existing {
            *existing = block_number;
        }
    } else {
        entries.push((key, block_number));
    }
}

/// Runtime hook that executes the rate-limiting migration.
pub struct Migration<T: SubtensorConfig>(PhantomData<T>);

impl<T> frame_support::traits::OnRuntimeUpgrade for Migration<T>
where
    T: SubtensorConfig
        + pallet_rate_limiting::Config<
            RateLimitingInstance,
            LimitScope = RateLimitScope,
            GroupId = GroupId,
        >,
    RateLimitUsageKey<T::AccountId>:
        Into<<T as pallet_rate_limiting::Config<RateLimitingInstance>>::UsageKey>,
{
    fn on_runtime_upgrade() -> Weight {
        migrate_rate_limiting::<T>()
    }
}

const fn admin_utils_identifier(call_index: u8) -> TransactionIdentifier {
    TransactionIdentifier::new(ADMIN_UTILS_PALLET_INDEX, call_index)
}

const fn subtensor_identifier(call_index: u8) -> TransactionIdentifier {
    TransactionIdentifier::new(SUBTENSOR_PALLET_INDEX, call_index)
}

/// Returns the `TransactionIdentifier` for the admin-utils extrinsic that controls `hparam`.
///
/// Only hyperparameters that are currently rate-limited (i.e. routed through
/// `ensure_sn_owner_or_root_with_limits`) are mapped; others return `None`.
pub fn identifier_for_hyperparameter(hparam: Hyperparameter) -> Option<TransactionIdentifier> {
    use Hyperparameter::*;

    let identifier = match hparam {
        Unknown | MaxWeightLimit => return None,
        ServingRateLimit => admin_utils_identifier(3),
        MaxDifficulty => admin_utils_identifier(5),
        AdjustmentAlpha => admin_utils_identifier(9),
        ImmunityPeriod => admin_utils_identifier(13),
        MinAllowedWeights => admin_utils_identifier(14),
        MaxAllowedUids => admin_utils_identifier(15),
        Kappa => admin_utils_identifier(16),
        Rho => admin_utils_identifier(17),
        ActivityCutoff => admin_utils_identifier(18),
        PowRegistrationAllowed => admin_utils_identifier(20),
        MinBurn => admin_utils_identifier(22),
        MaxBurn => admin_utils_identifier(23),
        BondsMovingAverage => admin_utils_identifier(26),
        BondsPenalty => admin_utils_identifier(60),
        CommitRevealEnabled => admin_utils_identifier(49),
        LiquidAlphaEnabled => admin_utils_identifier(50),
        AlphaValues => admin_utils_identifier(51),
        WeightCommitInterval => admin_utils_identifier(57),
        TransferEnabled => admin_utils_identifier(61),
        AlphaSigmoidSteepness => admin_utils_identifier(68),
        Yuma3Enabled => admin_utils_identifier(69),
        BondsResetEnabled => admin_utils_identifier(70),
        ImmuneNeuronLimit => admin_utils_identifier(72),
        RecycleOrBurn => admin_utils_identifier(80),
        _ => return None,
    };

    Some(identifier)
}

/// Returns the `TransactionIdentifier` for the extrinsic associated with the given transaction
/// type, mirroring current rate-limit enforcement.
pub fn identifier_for_transaction_type(tx: TransactionType) -> Option<TransactionIdentifier> {
    use TransactionType::*;

    let identifier = match tx {
        SetChildren => subtensor_identifier(67),
        SetChildkeyTake => subtensor_identifier(75),
        RegisterNetwork => subtensor_identifier(59),
        SetWeightsVersionKey => admin_utils_identifier(6),
        SetSNOwnerHotkey => admin_utils_identifier(67),
        OwnerHyperparamUpdate(hparam) => return identifier_for_hyperparameter(hparam),
        MechanismCountUpdate => admin_utils_identifier(76),
        MechanismEmission => admin_utils_identifier(77),
        MaxUidsTrimming => admin_utils_identifier(78),
        Unknown => return None,
        _ => return None,
    };

    Some(identifier)
}

/// Produces the usage key for a `TransactionType` that was stored in `TransactionKeyLastBlock`.
pub fn usage_key_from_transaction_type<AccountId>(
    tx: TransactionType,
    account: &AccountId,
    netuid: NetUid,
) -> Option<RateLimitUsageKey<AccountId>>
where
    AccountId: Parameter + Clone,
{
    match tx {
        TransactionType::SetChildren | TransactionType::SetChildkeyTake => {
            Some(RateLimitUsageKey::AccountSubnet {
                account: account.clone(),
                netuid,
            })
        }
        TransactionType::SetWeightsVersionKey => Some(RateLimitUsageKey::Subnet(netuid)),
        TransactionType::MechanismCountUpdate
        | TransactionType::MechanismEmission
        | TransactionType::MaxUidsTrimming => Some(RateLimitUsageKey::AccountSubnet {
            account: account.clone(),
            netuid,
        }),
        TransactionType::OwnerHyperparamUpdate(_) => Some(RateLimitUsageKey::Subnet(netuid)),
        TransactionType::RegisterNetwork => None,
        TransactionType::SetSNOwnerHotkey => Some(RateLimitUsageKey::Subnet(netuid)),
        TransactionType::Unknown => None,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountId, BuildStorage, RateLimitingInstance, Runtime};
    use sp_io::TestExternalities;
    use sp_runtime::traits::{SaturatedConversion, Zero};
    use subtensor_runtime_common::RateLimitUsageKey;

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
            Some(admin_utils_identifier(3))
        );
        assert!(identifier_for_hyperparameter(Hyperparameter::MaxWeightLimit).is_none());
    }

    #[test]
    fn maps_transaction_types() {
        assert_eq!(
            identifier_for_transaction_type(TransactionType::SetChildren),
            Some(subtensor_identifier(67))
        );
        assert!(identifier_for_transaction_type(TransactionType::Unknown).is_none());
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

            let weight = migrate_rate_limiting::<Runtime>();
            assert!(!weight.is_zero());
            assert!(pallet_subtensor::HasMigrationRun::<Runtime>::get(
                MIGRATION_NAME
            ));

            let tx_target = RateLimitTarget::Transaction(subtensor_identifier(70));
            let delegate_group = RateLimitTarget::Group(DELEGATE_TAKE_GROUP_ID);

            assert_eq!(
                pallet_rate_limiting::Limits::<Runtime, RateLimitingInstance>::get(tx_target),
                Some(RateLimit::Global(RateLimitKind::Exact(
                    10u64.saturated_into()
                )))
            );
            assert_eq!(
                pallet_rate_limiting::Limits::<Runtime, RateLimitingInstance>::get(delegate_group),
                Some(RateLimit::Global(RateLimitKind::Exact(
                    3u64.saturated_into()
                )))
            );

            let usage_key = RateLimitUsageKey::Account(account.clone());
            assert_eq!(
                pallet_rate_limiting::LastSeen::<Runtime, RateLimitingInstance>::get(
                    tx_target,
                    Some(usage_key.clone())
                ),
                Some(5u64.saturated_into())
            );

            let group = pallet_rate_limiting::Groups::<Runtime, RateLimitingInstance>::get(
                DELEGATE_TAKE_GROUP_ID,
            )
            .expect("group stored");
            assert_eq!(group.id, DELEGATE_TAKE_GROUP_ID);
            assert_eq!(group.name.as_slice(), b"delegate-take");
            assert_eq!(
                pallet_rate_limiting::CallGroups::<Runtime, RateLimitingInstance>::get(
                    subtensor_identifier(66)
                ),
                Some(DELEGATE_TAKE_GROUP_ID)
            );
            assert_eq!(
                pallet_rate_limiting::NextGroupId::<Runtime, RateLimitingInstance>::get(),
                6
            );
        });
    }

    #[test]
    fn migration_skips_when_already_run() {
        new_test_ext().execute_with(|| {
            pallet_subtensor::HasMigrationRun::<Runtime>::insert(MIGRATION_NAME, true);
            pallet_subtensor::TxRateLimit::<Runtime>::put(99);

            let base_weight = <Runtime as frame_system::Config>::DbWeight::get().reads(1);
            let weight = migrate_rate_limiting::<Runtime>();

            assert_eq!(weight, base_weight);
            assert!(
                pallet_rate_limiting::Limits::<Runtime, RateLimitingInstance>::iter()
                    .next()
                    .is_none()
            );
            assert!(
                pallet_rate_limiting::LastSeen::<Runtime, RateLimitingInstance>::iter()
                    .next()
                    .is_none()
            );
        });
    }
}
