use core::convert::TryFrom;

use codec::Encode;
use frame_support::{pallet_prelude::Parameter, traits::Get, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use log::info;
use pallet_rate_limiting::{RateLimit, RateLimitKind, TransactionIdentifier};
use sp_io::{
    hashing::{blake2_128, twox_128},
    storage,
};
use sp_runtime::traits::SaturatedConversion;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use subtensor_runtime_common::{MechId, NetUid, RateLimitScope, RateLimitUsageKey};

use pallet_subtensor::{
    self,
    utils::rate_limiting::{Hyperparameter, TransactionType},
    AssociatedEvmAddress, Axons, Config as SubtensorConfig, HasMigrationRun, LastRateLimitedBlock,
    LastUpdate, MaxUidsTrimmingRateLimit, MechanismCountCurrent, MechanismCountSetRateLimit,
    MechanismEmissionRateLimit, NetworkRateLimit, OwnerHyperparamRateLimit, Pallet, Prometheus,
    RateLimitKey, TransactionKeyLastBlock, TxChildkeyTakeRateLimit, TxDelegateTakeRateLimit,
    TxRateLimit, WeightsVersionKeyRateLimit,
};

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

/// Subtensor call indices that reuse the serving rate-limit configuration.
/// TODO(grouped-rate-limits): `serve_axon` (4), `serve_axon_tls` (40), and
/// `serve_prometheus` (5) share one cooldown today. The new pallet still misses
/// grouped identifiers, so we simply port the timers as-is.
const SERVE_CALLS: [u8; 3] = [4, 40, 5];
/// Subtensor call indices that reuse the per-subnet weight limit.
/// TODO(grouped-rate-limits): Weight commits via call 100 still touch the same
/// `LastUpdate` entries but cannot be expressed here until grouping exists.
const WEIGHT_CALLS_SUBNET: [u8; 3] = [0, 96, 113];
/// Subtensor call indices that reuse the per-mechanism weight limit.
const WEIGHT_CALLS_MECHANISM: [u8; 4] = [119, 115, 117, 118];
/// Subtensor call indices for register-network extrinsics.
/// TODO(grouped-rate-limits): `register_network` (59) and
/// `register_network_with_identity` (79) still share the same helper and should
/// remain grouped once pallet-rate-limiting supports aliases.
const REGISTER_NETWORK_CALLS: [u8; 2] = [59, 79];

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

type RateLimitConfigOf<T> = RateLimit<RateLimitScope, BlockNumberFor<T>>;
type LimitEntries<T> = Vec<(TransactionIdentifier, RateLimitConfigOf<T>)>;
type LastSeenKey<T> = (
    TransactionIdentifier,
    Option<RateLimitUsageKey<<T as frame_system::Config>::AccountId>>,
);
type LastSeenEntries<T> = Vec<(LastSeenKey<T>, BlockNumberFor<T>)>;

pub fn migrate_rate_limiting<T: SubtensorConfig>() -> Weight {
    let mut weight = T::DbWeight::get().reads(1);
    if HasMigrationRun::<T>::get(MIGRATION_NAME) {
        info!("Rate-limiting migration already executed. Skipping.");
        return weight;
    }

    let (limits, limit_reads) = build_limits::<T>();
    let (last_seen, seen_reads) = build_last_seen::<T>();

    let limit_writes = write_limits::<T>(&limits);
    let seen_writes = write_last_seen::<T>(&last_seen);

    HasMigrationRun::<T>::insert(MIGRATION_NAME, true);

    weight = weight
        .saturating_add(T::DbWeight::get().reads(limit_reads.saturating_add(seen_reads)))
        .saturating_add(
            T::DbWeight::get().writes(limit_writes.saturating_add(seen_writes).saturating_add(1)),
        );

    info!(
        "Migrated {} rate-limit configs and {} last-seen entries into pallet-rate-limiting",
        limits.len(),
        last_seen.len()
    );

    weight
}

fn build_limits<T: SubtensorConfig>() -> (LimitEntries<T>, u64) {
    let mut limits = LimitEntries::<T>::new();
    let mut reads: u64 = 0;

    reads += gather_simple_limits::<T>(&mut limits);
    reads += gather_owner_hparam_limits::<T>(&mut limits);
    reads += gather_serving_limits::<T>(&mut limits);
    reads += gather_weight_limits::<T>(&mut limits);

    (limits, reads)
}

fn gather_simple_limits<T: SubtensorConfig>(limits: &mut LimitEntries<T>) -> u64 {
    let mut reads: u64 = 0;

    reads += 1;
    if let Some(span) = block_number::<T>(TxRateLimit::<T>::get()) {
        set_global_limit::<T>(limits, subtensor_identifier(70), span);
    }

        reads += 1;
        if let Some(span) = block_number::<T>(TxDelegateTakeRateLimit::<T>::get()) {
            // TODO(grouped-rate-limits): `decrease_take` shares the same timestamp but
            // does not have its own ID here yet.
            set_global_limit::<T>(limits, subtensor_identifier(66), span);
        }

    reads += 1;
    if let Some(span) = block_number::<T>(TxChildkeyTakeRateLimit::<T>::get()) {
        set_global_limit::<T>(limits, subtensor_identifier(75), span);
    }

    reads += 1;
    if let Some(span) = block_number::<T>(NetworkRateLimit::<T>::get()) {
        for call in REGISTER_NETWORK_CALLS {
            set_global_limit::<T>(limits, subtensor_identifier(call), span);
        }
    }

    reads += 1;
    if let Some(span) = block_number::<T>(WeightsVersionKeyRateLimit::<T>::get()) {
        set_global_limit::<T>(limits, admin_utils_identifier(6), span);
    }

    if let Some(span) = block_number::<T>(DEFAULT_SET_SN_OWNER_HOTKEY_LIMIT) {
        set_global_limit::<T>(limits, admin_utils_identifier(67), span);
    }

    if let Some(span) = block_number::<T>(<T as SubtensorConfig>::EvmKeyAssociateRateLimit::get()) {
        set_global_limit::<T>(limits, subtensor_identifier(93), span);
    }

    if let Some(span) = block_number::<T>(MechanismCountSetRateLimit::<T>::get()) {
        set_global_limit::<T>(limits, admin_utils_identifier(76), span);
    }

    if let Some(span) = block_number::<T>(MechanismEmissionRateLimit::<T>::get()) {
        set_global_limit::<T>(limits, admin_utils_identifier(77), span);
    }

    if let Some(span) = block_number::<T>(MaxUidsTrimmingRateLimit::<T>::get()) {
        set_global_limit::<T>(limits, admin_utils_identifier(78), span);
    }

    if let Some(span) = block_number::<T>(SET_CHILDREN_RATE_LIMIT) {
        set_global_limit::<T>(limits, subtensor_identifier(67), span);
    }

    reads
}

fn gather_owner_hparam_limits<T: SubtensorConfig>(limits: &mut LimitEntries<T>) -> u64 {
    let mut reads: u64 = 0;

    reads += 1;
    if let Some(span) = block_number::<T>(u64::from(OwnerHyperparamRateLimit::<T>::get())) {
        for hparam in HYPERPARAMETERS {
            if let Some(identifier) = identifier_for_hyperparameter(*hparam) {
                set_global_limit::<T>(limits, identifier, span);
            }
        }
    }

    reads
}

fn gather_serving_limits<T: SubtensorConfig>(limits: &mut LimitEntries<T>) -> u64 {
    let mut reads: u64 = 0;
    let netuids = Pallet::<T>::get_all_subnet_netuids();

    for netuid in netuids {
        reads += 1;
        if let Some(span) = block_number::<T>(Pallet::<T>::get_serving_rate_limit(netuid)) {
            for call in SERVE_CALLS {
                set_scoped_limit::<T>(
                    limits,
                    subtensor_identifier(call),
                    RateLimitScope::Subnet(netuid),
                    span,
                );
            }
        }
    }

    reads
}

fn gather_weight_limits<T: SubtensorConfig>(limits: &mut LimitEntries<T>) -> u64 {
    let mut reads: u64 = 0;
    let netuids = Pallet::<T>::get_all_subnet_netuids();

    let mut subnet_limits = BTreeMap::<NetUid, BlockNumberFor<T>>::new();
    for netuid in &netuids {
        reads += 1;
        if let Some(span) = block_number::<T>(Pallet::<T>::get_weights_set_rate_limit(*netuid)) {
            subnet_limits.insert(*netuid, span);
            for call in WEIGHT_CALLS_SUBNET {
                set_scoped_limit::<T>(
                    limits,
                    subtensor_identifier(call),
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
            for call in WEIGHT_CALLS_MECHANISM {
                set_scoped_limit::<T>(limits, subtensor_identifier(call), scope.clone(), span);
            }
        }
    }

    reads
}

fn build_last_seen<T: SubtensorConfig>() -> (LastSeenEntries<T>, u64) {
    let mut last_seen = LastSeenEntries::<T>::new();
    let mut reads: u64 = 0;

    reads += import_last_rate_limited_blocks::<T>(&mut last_seen);
    reads += import_transaction_key_last_blocks::<T>(&mut last_seen);
    reads += import_last_update_entries::<T>(&mut last_seen);
    reads += import_serving_entries::<T>(&mut last_seen);
    reads += import_evm_entries::<T>(&mut last_seen);

    (last_seen, reads)
}

fn import_last_rate_limited_blocks<T: SubtensorConfig>(entries: &mut LastSeenEntries<T>) -> u64 {
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
                        identifier,
                        Some(RateLimitUsageKey::Subnet(netuid)),
                        block,
                    );
                }
            }
            RateLimitKey::OwnerHyperparamUpdate(netuid, hyper) => {
                if let Some(identifier) = identifier_for_hyperparameter(hyper) {
                    record_last_seen_entry::<T>(
                        entries,
                        identifier,
                        Some(RateLimitUsageKey::Subnet(netuid)),
                        block,
                    );
                }
            }
            RateLimitKey::LastTxBlock(account) => {
                record_last_seen_entry::<T>(
                    entries,
                    subtensor_identifier(70),
                    Some(RateLimitUsageKey::Account(account.clone())),
                    block,
                );
            }
            RateLimitKey::LastTxBlockDelegateTake(account) => {
                record_last_seen_entry::<T>(
                    entries,
                    subtensor_identifier(66),
                    Some(RateLimitUsageKey::Account(account.clone())),
                    block,
                );
            }
            RateLimitKey::NetworkLastRegistered | RateLimitKey::LastTxBlockChildKeyTake(_) => {
                // TODO(grouped-rate-limits): Global network registration lock is still outside
                // pallet-rate-limiting. We will migrate it once grouped identifiers land.
            }
        }
    }
    reads
}

fn import_transaction_key_last_blocks<T: SubtensorConfig>(entries: &mut LastSeenEntries<T>) -> u64 {
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
        record_last_seen_entry::<T>(entries, identifier, Some(usage), block);
    }
    reads
}

fn import_last_update_entries<T: SubtensorConfig>(entries: &mut LastSeenEntries<T>) -> u64 {
    let mut reads: u64 = 0;
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

            let call_set: &[u8] = if is_mechanism {
                &WEIGHT_CALLS_MECHANISM
            } else {
                &WEIGHT_CALLS_SUBNET
            };

            for call in call_set {
                record_last_seen_entry::<T>(
                    entries,
                    subtensor_identifier(*call),
                    Some(usage.clone()),
                    last_block,
                );
            }
        }
    }
    reads
}

fn import_serving_entries<T: SubtensorConfig>(entries: &mut LastSeenEntries<T>) -> u64 {
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
        for call in [4u8, 40u8] {
            record_last_seen_entry::<T>(
                entries,
                subtensor_identifier(call),
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
        record_last_seen_entry::<T>(entries, subtensor_identifier(5), Some(usage), prom.block);
    }

    reads
}

fn import_evm_entries<T: SubtensorConfig>(entries: &mut LastSeenEntries<T>) -> u64 {
    let mut reads: u64 = 0;
    for (netuid, uid, (_, block)) in AssociatedEvmAddress::<T>::iter() {
        reads += 1;
        if block == 0 {
            continue;
        }
        record_last_seen_entry::<T>(
            entries,
            subtensor_identifier(93),
            Some(RateLimitUsageKey::SubnetNeuron { netuid, uid }),
            block,
        );
    }
    reads
}

/// TODO(rate-limiting-storage): Swap these manual writes for
/// `pallet_rate_limiting::Pallet` APIs once the runtime wires the pallet in.
fn write_limits<T: SubtensorConfig>(limits: &LimitEntries<T>) -> u64 {
    if limits.is_empty() {
        return 0;
    }
    let prefix = storage_prefix("RateLimiting", "Limits");
    let mut writes = 0;
    for (identifier, limit) in limits.iter() {
        let key = map_storage_key(&prefix, identifier);
        storage::set(&key, &limit.encode());
        writes += 1;
    }
    writes
}

fn write_last_seen<T: SubtensorConfig>(entries: &LastSeenEntries<T>) -> u64 {
    if entries.is_empty() {
        return 0;
    }
    let prefix = storage_prefix("RateLimiting", "LastSeen");
    let mut writes = 0;
    for ((identifier, usage), block) in entries.iter() {
        let key = double_map_storage_key(&prefix, identifier, usage);
        storage::set(&key, &block.encode());
        writes += 1;
    }
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
    identifier: TransactionIdentifier,
    span: BlockNumberFor<T>,
) {
    if let Some((_, config)) = limits.iter_mut().find(|(id, _)| *id == identifier) {
        *config = RateLimit::global(RateLimitKind::Exact(span));
    } else {
        limits.push((identifier, RateLimit::global(RateLimitKind::Exact(span))));
    }
}

fn set_scoped_limit<T: SubtensorConfig>(
    limits: &mut LimitEntries<T>,
    identifier: TransactionIdentifier,
    scope: RateLimitScope,
    span: BlockNumberFor<T>,
) {
    if let Some((_, config)) = limits.iter_mut().find(|(id, _)| *id == identifier) {
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
            identifier,
            RateLimit::scoped_single(scope, RateLimitKind::Exact(span)),
        ));
    }
}

fn record_last_seen_entry<T: SubtensorConfig>(
    entries: &mut LastSeenEntries<T>,
    identifier: TransactionIdentifier,
    usage: Option<RateLimitUsageKey<T::AccountId>>,
    block: u64,
) {
    let Some(block_number) = block_number::<T>(block) else {
        return;
    };

    let key = (identifier, usage);
    if let Some((_, existing)) = entries.iter_mut().find(|(entry_key, _)| *entry_key == key) {
        if block_number > *existing {
            *existing = block_number;
        }
    } else {
        entries.push((key, block_number));
    }
}

fn storage_prefix(pallet: &str, storage: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(32);
    out.extend_from_slice(&twox_128(pallet.as_bytes()));
    out.extend_from_slice(&twox_128(storage.as_bytes()));
    out
}

fn map_storage_key(prefix: &[u8], key: impl Encode) -> Vec<u8> {
    let mut final_key = Vec::with_capacity(prefix.len() + 32);
    final_key.extend_from_slice(prefix);
    let encoded = key.encode();
    let hash = blake2_128(&encoded);
    final_key.extend_from_slice(&hash);
    final_key.extend_from_slice(&encoded);
    final_key
}

fn double_map_storage_key(prefix: &[u8], key1: impl Encode, key2: impl Encode) -> Vec<u8> {
    let mut final_key = Vec::with_capacity(prefix.len() + 64);
    final_key.extend_from_slice(prefix);
    let first = map_storage_key(&[], key1);
    final_key.extend_from_slice(&first);
    let second = map_storage_key(&[], key2);
    final_key.extend_from_slice(&second);
    final_key
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

/// Maps legacy `RateLimitKey` entries to the new usage-key representation.
pub fn usage_key_from_legacy_key<AccountId>(
    key: &RateLimitKey<AccountId>,
) -> Option<RateLimitUsageKey<AccountId>>
where
    AccountId: Parameter + Clone,
{
    match key {
        RateLimitKey::SetSNOwnerHotkey(netuid) => Some(RateLimitUsageKey::Subnet(*netuid)),
        RateLimitKey::OwnerHyperparamUpdate(netuid, _) => Some(RateLimitUsageKey::Subnet(*netuid)),
        RateLimitKey::NetworkLastRegistered => None,
        RateLimitKey::LastTxBlock(account)
        | RateLimitKey::LastTxBlockChildKeyTake(account)
        | RateLimitKey::LastTxBlockDelegateTake(account) => {
            Some(RateLimitUsageKey::Account(account.clone()))
        }
    }
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
        TransactionType::RegisterNetwork => Some(RateLimitUsageKey::Account(account.clone())),
        TransactionType::SetSNOwnerHotkey => Some(RateLimitUsageKey::Subnet(netuid)),
        TransactionType::Unknown => None,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn maps_usage_keys() {
        let acct = 42u64;
        assert!(matches!(
            usage_key_from_legacy_key(&RateLimitKey::LastTxBlock(acct)),
            Some(RateLimitUsageKey::Account(42))
        ));
    }
}
