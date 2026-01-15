#![allow(dead_code)]

use codec::{Decode, Encode};
use frame_support::{Identity, migration::storage_key_iter};
use runtime_common::prod_or_fast;
use scale_info::TypeInfo;
use sp_io::{
    hashing::twox_128,
    storage::{self as io_storage, next_key},
};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use subtensor_runtime_common::{NetUid, NetUidStorageIndex};

use super::AccountId;
use crate::{
    SubtensorInitialNetworkRateLimit, SubtensorInitialTxChildKeyTakeRateLimit,
    SubtensorInitialTxDelegateTakeRateLimit, SubtensorInitialTxRateLimit,
};

pub use types::{Hyperparameter, RateLimitKey, TransactionType};

const PALLET_PREFIX: &[u8] = b"SubtensorModule";
const BLAKE2_128_PREFIX_LEN: usize = 16;

pub mod storage {
    use super::*;

    pub fn serving_rate_limits() -> (BTreeMap<NetUid, u64>, u64) {
        let items: Vec<_> =
            storage_key_iter::<NetUid, u64, Identity>(PALLET_PREFIX, b"ServingRateLimit").collect();
        let reads = items.len() as u64;
        (items.into_iter().collect(), reads)
    }

    pub fn weights_set_rate_limits() -> (BTreeMap<NetUid, u64>, u64) {
        let items: Vec<_> =
            storage_key_iter::<NetUid, u64, Identity>(PALLET_PREFIX, b"WeightsSetRateLimit")
                .collect();
        let reads = items.len() as u64;
        (items.into_iter().collect(), reads)
    }

    pub fn get_weights_set_rate_limit(netuid: NetUid) -> u64 {
        let mut key = storage_prefix(PALLET_PREFIX, b"WeightsSetRateLimit");
        key.extend(netuid.encode());
        io_storage::get(&key)
            .and_then(|bytes| Decode::decode(&mut &bytes[..]).ok())
            .unwrap_or_else(defaults::weights_set_rate_limit)
    }

    pub fn set_weights_set_rate_limit(netuid: NetUid, span: u64) {
        let mut key = storage_prefix(PALLET_PREFIX, b"WeightsSetRateLimit");
        key.extend(netuid.encode());
        io_storage::set(&key, &span.encode());
    }

    pub fn last_updates() -> (Vec<(NetUidStorageIndex, Vec<u64>)>, u64) {
        let items: Vec<_> = storage_key_iter::<NetUidStorageIndex, Vec<u64>, Identity>(
            PALLET_PREFIX,
            b"LastUpdate",
        )
        .collect();
        let reads = items.len() as u64;
        (items, reads)
    }

    pub fn set_last_update(netuid_index: NetUidStorageIndex, blocks: Vec<u64>) {
        let mut key = storage_prefix(PALLET_PREFIX, b"LastUpdate");
        key.extend(netuid_index.encode());
        io_storage::set(&key, &blocks.encode());
    }

    pub fn get_last_update(netuid_index: NetUidStorageIndex) -> Vec<u64> {
        let mut key = storage_prefix(PALLET_PREFIX, b"LastUpdate");
        key.extend(netuid_index.encode());
        io_storage::get(&key)
            .and_then(|bytes| Decode::decode(&mut &bytes[..]).ok())
            .unwrap_or_default()
    }

    pub fn set_serving_rate_limit(netuid: NetUid, span: u64) {
        let mut key = storage_prefix(PALLET_PREFIX, b"ServingRateLimit");
        key.extend(netuid.encode());
        io_storage::set(&key, &span.encode());
    }

    pub fn tx_rate_limit() -> (u64, u64) {
        value_with_default(b"TxRateLimit", defaults::tx_rate_limit())
    }

    pub fn set_tx_rate_limit(span: u64) {
        let key = storage_prefix(PALLET_PREFIX, b"TxRateLimit");
        io_storage::set(&key, &span.encode());
    }

    pub fn tx_delegate_take_rate_limit() -> (u64, u64) {
        value_with_default(
            b"TxDelegateTakeRateLimit",
            defaults::tx_delegate_take_rate_limit(),
        )
    }

    pub fn set_tx_delegate_take_rate_limit(span: u64) {
        let key = storage_prefix(PALLET_PREFIX, b"TxDelegateTakeRateLimit");
        io_storage::set(&key, &span.encode());
    }

    pub fn tx_childkey_take_rate_limit() -> (u64, u64) {
        value_with_default(
            b"TxChildkeyTakeRateLimit",
            defaults::tx_childkey_take_rate_limit(),
        )
    }

    pub fn network_rate_limit() -> (u64, u64) {
        value_with_default(b"NetworkRateLimit", defaults::network_rate_limit())
    }

    pub fn set_network_rate_limit(span: u64) {
        let key = storage_prefix(PALLET_PREFIX, b"NetworkRateLimit");
        io_storage::set(&key, &span.encode());
    }

    pub fn owner_hyperparam_rate_limit() -> (u64, u64) {
        let (value, reads) = value_with_default::<u16>(
            b"OwnerHyperparamRateLimit",
            defaults::owner_hyperparam_rate_limit(),
        );
        (u64::from(value), reads)
    }

    pub fn weights_version_key_rate_limit() -> (u64, u64) {
        value_with_default(
            b"WeightsVersionKeyRateLimit",
            defaults::weights_version_key_rate_limit(),
        )
    }

    pub fn last_rate_limited_blocks() -> (Vec<(RateLimitKey<AccountId>, u64)>, u64) {
        let entries: Vec<_> = storage_key_iter::<RateLimitKey<AccountId>, u64, Identity>(
            PALLET_PREFIX,
            b"LastRateLimitedBlock",
        )
        .collect();
        let reads = entries.len() as u64;
        (entries, reads)
    }

    pub fn set_last_rate_limited_block(key: RateLimitKey<AccountId>, block: u64) {
        let mut storage_key = storage_prefix(PALLET_PREFIX, b"LastRateLimitedBlock");
        storage_key.extend(key.encode());
        io_storage::set(&storage_key, &block.encode());
    }

    pub fn transaction_key_last_block() -> (Vec<((AccountId, NetUid, u16), u64)>, u64) {
        let prefix = storage_prefix(PALLET_PREFIX, b"TransactionKeyLastBlock");
        let mut cursor = prefix.clone();
        let mut entries = Vec::new();

        while let Some(next) = next_key(&cursor) {
            if !next.starts_with(&prefix) {
                break;
            }
            if let Some(value) = io_storage::get(&next) {
                let key_bytes = &next[prefix.len()..];
                if let (Some(key), Some(decoded_value)) = (
                    decode_transaction_key(key_bytes),
                    decode_value::<u64>(&value),
                ) {
                    entries.push((key, decoded_value));
                }
            }
            cursor = next;
        }

        let reads = entries.len() as u64;
        (entries, reads)
    }

    fn storage_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
        [twox_128(pallet), twox_128(storage)].concat()
    }

    fn value_with_default<V: Decode + Copy>(storage_name: &[u8], default: V) -> (V, u64) {
        let key = storage_prefix(PALLET_PREFIX, storage_name);
        let value = io_storage::get(&key)
            .and_then(|bytes| Decode::decode(&mut &bytes[..]).ok())
            .unwrap_or(default);
        (value, 1)
    }

    fn decode_value<V: Decode>(bytes: &[u8]) -> Option<V> {
        Decode::decode(&mut &bytes[..]).ok()
    }

    fn decode_transaction_key<AccountId: Decode>(
        encoded: &[u8],
    ) -> Option<(AccountId, NetUid, u16)> {
        if encoded.len() < BLAKE2_128_PREFIX_LEN {
            return None;
        }
        let mut slice = &encoded[BLAKE2_128_PREFIX_LEN..];
        let account = AccountId::decode(&mut slice).ok()?;
        let netuid = NetUid::decode(&mut slice).ok()?;
        let tx_kind = u16::decode(&mut slice).ok()?;

        Some((account, netuid, tx_kind))
    }
}

pub mod defaults {
    use super::*;

    pub fn serving_rate_limit() -> u64 {
        // SubtensorInitialServingRateLimit::get()
        50
    }

    pub fn weights_set_rate_limit() -> u64 {
        100
    }

    pub fn tx_rate_limit() -> u64 {
        SubtensorInitialTxRateLimit::get()
    }

    pub fn tx_delegate_take_rate_limit() -> u64 {
        SubtensorInitialTxDelegateTakeRateLimit::get()
    }

    pub fn tx_childkey_take_rate_limit() -> u64 {
        SubtensorInitialTxChildKeyTakeRateLimit::get()
    }

    pub fn network_rate_limit() -> u64 {
        if cfg!(feature = "pow-faucet") {
            0
        } else {
            SubtensorInitialNetworkRateLimit::get()
        }
    }

    pub fn owner_hyperparam_rate_limit() -> u16 {
        2
    }

    pub fn weights_version_key_rate_limit() -> u64 {
        5
    }

    pub fn sn_owner_hotkey_rate_limit() -> u64 {
        50_400
    }

    pub fn mechanism_count_rate_limit() -> u64 {
        prod_or_fast!(7_200, 1)
    }

    pub fn mechanism_emission_rate_limit() -> u64 {
        prod_or_fast!(7_200, 1)
    }

    pub fn max_uids_trimming_rate_limit() -> u64 {
        prod_or_fast!(30 * 7_200, 1)
    }
}

pub mod types {
    use super::*;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub enum RateLimitKey<AccountId> {
        #[codec(index = 0)]
        SetSNOwnerHotkey(NetUid),
        #[codec(index = 1)]
        OwnerHyperparamUpdate(NetUid, Hyperparameter),
        #[codec(index = 2)]
        NetworkLastRegistered,
        #[codec(index = 3)]
        LastTxBlock(AccountId),
        #[codec(index = 4)]
        LastTxBlockChildKeyTake(AccountId),
        #[codec(index = 5)]
        LastTxBlockDelegateTake(AccountId),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum TransactionType {
        SetChildren,
        SetChildkeyTake,
        Unknown,
        RegisterNetwork,
        SetWeightsVersionKey,
        SetSNOwnerHotkey,
        OwnerHyperparamUpdate(Hyperparameter),
        MechanismCountUpdate,
        MechanismEmission,
        MaxUidsTrimming,
    }

    impl From<u16> for TransactionType {
        fn from(value: u16) -> Self {
            match value {
                0 => TransactionType::SetChildren,
                1 => TransactionType::SetChildkeyTake,
                3 => TransactionType::RegisterNetwork,
                4 => TransactionType::SetWeightsVersionKey,
                5 => TransactionType::SetSNOwnerHotkey,
                6 => TransactionType::OwnerHyperparamUpdate(Hyperparameter::Unknown),
                7 => TransactionType::MechanismCountUpdate,
                8 => TransactionType::MechanismEmission,
                9 => TransactionType::MaxUidsTrimming,
                _ => TransactionType::Unknown,
            }
        }
    }

    impl From<TransactionType> for u16 {
        fn from(tx_type: TransactionType) -> Self {
            match tx_type {
                TransactionType::SetChildren => 0,
                TransactionType::SetChildkeyTake => 1,
                TransactionType::Unknown => 2,
                TransactionType::RegisterNetwork => 3,
                TransactionType::SetWeightsVersionKey => 4,
                TransactionType::SetSNOwnerHotkey => 5,
                TransactionType::OwnerHyperparamUpdate(_) => 6,
                TransactionType::MechanismCountUpdate => 7,
                TransactionType::MechanismEmission => 8,
                TransactionType::MaxUidsTrimming => 9,
            }
        }
    }

    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, TypeInfo)]
    #[non_exhaustive]
    pub enum Hyperparameter {
        Unknown = 0,
        ServingRateLimit = 1,
        MaxDifficulty = 2,
        AdjustmentAlpha = 3,
        MaxWeightLimit = 4,
        ImmunityPeriod = 5,
        MinAllowedWeights = 6,
        Kappa = 7,
        Rho = 8,
        ActivityCutoff = 9,
        PowRegistrationAllowed = 10,
        MinBurn = 11,
        MaxBurn = 12,
        BondsMovingAverage = 13,
        BondsPenalty = 14,
        CommitRevealEnabled = 15,
        LiquidAlphaEnabled = 16,
        AlphaValues = 17,
        WeightCommitInterval = 18,
        TransferEnabled = 19,
        AlphaSigmoidSteepness = 20,
        Yuma3Enabled = 21,
        BondsResetEnabled = 22,
        ImmuneNeuronLimit = 23,
        RecycleOrBurn = 24,
        MaxAllowedUids = 25,
    }

    impl From<Hyperparameter> for TransactionType {
        fn from(param: Hyperparameter) -> Self {
            Self::OwnerHyperparamUpdate(param)
        }
    }
}
