#![cfg_attr(not(feature = "std"), no_std)]
use core::fmt::{self, Display, Formatter};

use codec::{
    Compact, CompactAs, Decode, DecodeWithMemTracking, Encode, Error as CodecError, MaxEncodedLen,
};
use frame_support::pallet_prelude::*;
use runtime_common::prod_or_fast;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::{
    MultiSignature, Vec,
    traits::{IdentifyAccount, Verify},
};

pub use sp_io::MultiRemovalResults;
use subtensor_macros::freeze_struct;

pub use currency::*;
pub use evm_context::*;
pub use traits::*;
pub use transaction_error::*;

use frame_support::weights::WeightMeter;

mod currency;
mod evm_context;
mod traits;
mod transaction_error;

/// Balance of an account.
pub type Balance = TaoBalance;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent to the
/// public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

pub type Nonce = u32;

/// Transfers below SMALL_TRANSFER_LIMIT are considered small transfers
pub const SMALL_TRANSFER_LIMIT: Balance = TaoBalance::new(500_000_000); // 0.5 TAO
pub const SMALL_ALPHA_TRANSFER_LIMIT: AlphaBalance = AlphaBalance::new(500_000_000); // 0.5 Alpha

#[freeze_struct("c972489bff40ae48")]
#[repr(transparent)]
#[derive(
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Decode,
    DecodeWithMemTracking,
    Default,
    Encode,
    Eq,
    Hash,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    RuntimeDebug,
)]
#[serde(transparent)]
pub struct NetUid(u16);

impl NetUid {
    pub const ROOT: NetUid = Self(0);

    pub fn is_root(&self) -> bool {
        *self == Self::ROOT
    }

    pub fn next(&self) -> NetUid {
        Self(self.0.saturating_add(1))
    }

    pub fn prev(&self) -> NetUid {
        Self(self.0.saturating_sub(1))
    }

    pub fn inner(&self) -> u16 {
        self.0
    }
}

impl Display for NetUid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl CompactAs for NetUid {
    type As = u16;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<NetUid>> for NetUid {
    fn from(c: Compact<NetUid>) -> Self {
        c.0
    }
}

impl From<NetUid> for u16 {
    fn from(val: NetUid) -> Self {
        val.0
    }
}

impl From<u16> for NetUid {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl TypeInfo for NetUid {
    type Identity = <u16 as TypeInfo>::Identity;
    fn type_info() -> scale_info::Type {
        <u16 as TypeInfo>::type_info()
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Debug,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum ProxyType {
    Any,
    Owner, // Subnet owner Calls
    NonCritical,
    NonTransfer,
    Senate,
    NonFungible, // Nothing involving moving TAO
    Triumvirate,
    Governance, // Both above governance
    Staking,
    Registration,
    Transfer,
    SmallTransfer,
    RootWeights, // deprecated
    ChildKeys,
    SudoUncheckedSetCode,
    SwapHotkey,
    SubnetLeaseBeneficiary, // Used to operate the leased subnet
    RootClaim,
}

impl TryFrom<u8> for ProxyType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Any),
            1 => Ok(Self::Owner),
            2 => Ok(Self::NonCritical),
            3 => Ok(Self::NonTransfer),
            4 => Ok(Self::Senate),
            5 => Ok(Self::NonFungible),
            6 => Ok(Self::Triumvirate),
            7 => Ok(Self::Governance),
            8 => Ok(Self::Staking),
            9 => Ok(Self::Registration),
            10 => Ok(Self::Transfer),
            11 => Ok(Self::SmallTransfer),
            12 => Ok(Self::RootWeights),
            13 => Ok(Self::ChildKeys),
            14 => Ok(Self::SudoUncheckedSetCode),
            15 => Ok(Self::SwapHotkey),
            16 => Ok(Self::SubnetLeaseBeneficiary),
            17 => Ok(Self::RootClaim),
            _ => Err(()),
        }
    }
}

impl From<ProxyType> for u8 {
    fn from(proxy_type: ProxyType) -> Self {
        match proxy_type {
            ProxyType::Any => 0,
            ProxyType::Owner => 1,
            ProxyType::NonCritical => 2,
            ProxyType::NonTransfer => 3,
            ProxyType::Senate => 4,
            ProxyType::NonFungible => 5,
            ProxyType::Triumvirate => 6,
            ProxyType::Governance => 7,
            ProxyType::Staking => 8,
            ProxyType::Registration => 9,
            ProxyType::Transfer => 10,
            ProxyType::SmallTransfer => 11,
            ProxyType::RootWeights => 12,
            ProxyType::ChildKeys => 13,
            ProxyType::SudoUncheckedSetCode => 14,
            ProxyType::SwapHotkey => 15,
            ProxyType::SubnetLeaseBeneficiary => 16,
            ProxyType::RootClaim => 17,
        }
    }
}

impl ProxyType {
    pub fn is_deprecated(&self) -> bool {
        matches!(
            self,
            Self::Triumvirate | Self::Senate | Self::Governance | Self::RootWeights
        )
    }
}

impl Default for ProxyType {
    // allow all Calls; required to be most permissive
    fn default() -> Self {
        Self::Any
    }
}

/// Conditions that must be met beyond matching the call itself.
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub enum CallCondition {
    /// A numeric parameter must be less than this limit
    ParamLessThan { param_name: Vec<u8>, limit: u128 },
    /// The nested call inside must match this pallet/call
    NestedCallMustBe {
        pallet_name: Vec<u8>,
        call_name: Vec<u8>,
    },
}

/// Describes which call(s) a proxy filter rule applies to.
///
/// When `call_name` and `call_index` are `None`, the rule applies to ALL calls in the pallet.
/// When they are `Some`, the rule applies to that specific call only.
#[freeze_struct("57f984617f6084cc")]
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub struct CallInfo {
    /// Pallet name (always present)
    pub pallet_name: Vec<u8>,
    /// Pallet index in runtime (always present)
    pub pallet_index: u8,
    /// Call name within pallet. None means ALL calls in this pallet.
    pub call_name: Option<Vec<u8>>,
    /// Call index within pallet. None means ALL calls in this pallet.
    pub call_index: Option<u8>,
    /// Additional condition that must be met (value limits, nested call requirements)
    pub condition: Option<CallCondition>,
}

/// Describes how a ProxyType filters incoming calls.
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub enum FilterMode {
    /// All calls are permitted regardless of the calls list (e.g. ProxyType::Any)
    AllowAll,
    /// No calls are permitted (e.g. deprecated proxy types)
    DenyAll,
    /// Only calls listed in the `calls` field are permitted
    Allow,
    /// All calls are permitted EXCEPT those listed in the `calls` field
    Deny,
}

/// Complete filter description for a ProxyType, returned by the Runtime API.
///
/// Interpretation:
/// - `filter_mode: AllowAll` — everything permitted, `calls` is empty
/// - `filter_mode: DenyAll` — nothing permitted, `calls` is empty
/// - `filter_mode: Allow` — only `calls` are permitted (minus `exceptions`)
/// - `filter_mode: Deny` — everything EXCEPT `calls` is permitted
/// - `call_name: None` in a CallInfo — rule applies to ALL calls in the pallet
#[freeze_struct("4453d44869f8a188")]
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub struct ProxyFilterInfo {
    pub proxy_type: u8,
    pub name: Vec<u8>,
    pub deprecated: bool,
    pub filter_mode: FilterMode,
    pub calls: Vec<CallInfo>,
    pub exceptions: Vec<CallInfo>,
}

#[freeze_struct("b0cce66ed9b2451b")]
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub struct ProxyTypeInfo {
    pub name: Vec<u8>,
    pub index: u8,
    pub deprecated: bool,
}

pub trait SubnetInfo<AccountId> {
    fn exists(netuid: NetUid) -> bool;
    fn mechanism(netuid: NetUid) -> u16;
    fn is_owner(account_id: &AccountId, netuid: NetUid) -> bool;
    fn is_subtoken_enabled(netuid: NetUid) -> bool;
    fn get_validator_trust(netuid: NetUid) -> Vec<u16>;
    fn get_validator_permit(netuid: NetUid) -> Vec<bool>;
    fn hotkey_of_uid(netuid: NetUid, uid: u16) -> Option<AccountId>;
}

pub trait TokenReserve<C: Token> {
    fn reserve(netuid: NetUid) -> C;
    fn increase_provided(netuid: NetUid, amount: C);
    fn decrease_provided(netuid: NetUid, amount: C);
}

pub trait BalanceOps<AccountId> {
    fn tao_balance(account_id: &AccountId) -> TaoBalance;
    fn alpha_balance(netuid: NetUid, coldkey: &AccountId, hotkey: &AccountId) -> AlphaBalance;
    fn increase_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: AlphaBalance,
    ) -> Result<(), DispatchError>;
    fn decrease_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: AlphaBalance,
    ) -> Result<(), DispatchError>;
}

/// Allows to query the current block author
pub trait AuthorshipInfo<AccountId> {
    /// Return the current block author
    fn author() -> Option<AccountId>;
}

pub mod time {
    use super::*;

    /// This determines the average expected block time that we are targeting. Blocks will be
    /// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
    /// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
    /// slot_duration()`.
    ///
    /// Change this to adjust the block time.
    pub const MILLISECS_PER_BLOCK: u64 = prod_or_fast!(12000, 250);

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}

#[freeze_struct("7e5202d7f18b39d4")]
#[repr(transparent)]
#[derive(
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Decode,
    DecodeWithMemTracking,
    Default,
    Encode,
    Eq,
    Hash,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    RuntimeDebug,
)]
#[serde(transparent)]
pub struct MechId(u8);

impl MechId {
    pub const MAIN: MechId = Self(0);
}

impl From<u8> for MechId {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<MechId> for u16 {
    fn from(val: MechId) -> Self {
        Into::<u16>::into(val.0)
    }
}

impl From<MechId> for u64 {
    fn from(val: MechId) -> Self {
        Into::<u64>::into(val.0)
    }
}

impl From<MechId> for u8 {
    fn from(val: MechId) -> Self {
        Into::<u8>::into(val.0)
    }
}

impl Display for MechId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl CompactAs for MechId {
    type As = u8;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<MechId>> for MechId {
    fn from(c: Compact<MechId>) -> Self {
        c.0
    }
}

impl TypeInfo for MechId {
    type Identity = <u8 as TypeInfo>::Identity;
    fn type_info() -> scale_info::Type {
        <u8 as TypeInfo>::type_info()
    }
}

#[freeze_struct("2d995c5478e16d4d")]
#[repr(transparent)]
#[derive(
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Decode,
    DecodeWithMemTracking,
    Default,
    Encode,
    Eq,
    Hash,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    RuntimeDebug,
)]
#[serde(transparent)]
pub struct NetUidStorageIndex(u16);

impl NetUidStorageIndex {
    pub const ROOT: NetUidStorageIndex = Self(0);
}

impl Display for NetUidStorageIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl CompactAs for NetUidStorageIndex {
    type As = u16;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<NetUidStorageIndex>> for NetUidStorageIndex {
    fn from(c: Compact<NetUidStorageIndex>) -> Self {
        c.0
    }
}

impl From<NetUid> for NetUidStorageIndex {
    fn from(val: NetUid) -> Self {
        val.0.into()
    }
}

impl From<NetUidStorageIndex> for u16 {
    fn from(val: NetUidStorageIndex) -> Self {
        val.0
    }
}

impl From<u16> for NetUidStorageIndex {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl TypeInfo for NetUidStorageIndex {
    type Identity = <u16 as TypeInfo>::Identity;
    fn type_info() -> scale_info::Type {
        <u16 as TypeInfo>::type_info()
    }
}

/// Clears as many entries as the weight budget allows via `clear_prefix`, charging
/// `per_item` for each removed entry. Returns `true` once the prefix is fully cleared.
pub fn clear_prefix_with_meter(
    meter: &mut WeightMeter,
    per_item: Weight,
    clear_prefix: impl FnOnce(u32) -> MultiRemovalResults,
) -> bool {
    let Some(limit) = meter.remaining().checked_div_per_component(&per_item) else {
        return false;
    };
    // Saturate: a budget allowing more than u32::MAX removals is capped, not rejected.
    let limit = u32::try_from(limit).unwrap_or(u32::MAX);

    if limit == 0 {
        return false;
    }

    let result = clear_prefix(limit);
    meter.consume(per_item.saturating_mul(result.unique.max(result.loops).into()));

    result.maybe_cursor.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        Blake2_128Concat, storage::types::StorageDoubleMap, traits::StorageInstance,
        weights::WeightMeter,
    };
    const REF_TIME_WEIGHT: u64 = 100;
    const PROOF_SIZE_WEIGHT: u64 = 100;

    struct ClearPrefixTestStorage;

    impl StorageInstance for ClearPrefixTestStorage {
        fn pallet_prefix() -> &'static str {
            "CommonTests"
        }

        const STORAGE_PREFIX: &'static str = "ClearPrefixTestStorage";
    }

    type ClearPrefixTestMap =
        StorageDoubleMap<ClearPrefixTestStorage, Identity, NetUid, Blake2_128Concat, u16, u32>;

    #[test]
    fn netuid_has_u16_bin_repr() {
        assert_eq!(NetUid(5).encode(), 5u16.encode());
    }

    #[test]
    fn test_clear_prefix_with_meter_respects_budget() {
        let netuid = NetUid::from(42);
        let entry_weight = Weight::from_parts(REF_TIME_WEIGHT, PROOF_SIZE_WEIGHT);
        let mut ext = sp_io::TestExternalities::default();

        ext.execute_with(|| {
            for key in 0..3 {
                ClearPrefixTestMap::insert(netuid, key, key as u32);
            }
        });

        let _ = ext.commit_all();

        ext.execute_with(|| {
            assert_eq!(ClearPrefixTestMap::iter_prefix(netuid).count(), 3);

            // Budget for exactly one entry: one entry is removed, not done yet.
            let mut weight_meter = WeightMeter::with_limit(entry_weight);
            assert!(!clear_prefix_with_meter(
                &mut weight_meter,
                entry_weight,
                |limit| ClearPrefixTestMap::clear_prefix(netuid, limit, None),
            ));

            assert_eq!(ClearPrefixTestMap::iter_prefix(netuid).count(), 2);
            assert_eq!(weight_meter.consumed(), entry_weight);
        });
    }

    #[test]
    fn test_clear_prefix_with_meter_zero_budget_is_noop() {
        let netuid = NetUid::from(43);
        let entry_weight = Weight::from_parts(REF_TIME_WEIGHT, PROOF_SIZE_WEIGHT);
        let mut ext = sp_io::TestExternalities::default();

        ext.execute_with(|| {
            ClearPrefixTestMap::insert(netuid, 0, 0u32);

            let mut weight_meter = WeightMeter::with_limit(Weight::zero());
            assert!(!clear_prefix_with_meter(
                &mut weight_meter,
                entry_weight,
                |limit| ClearPrefixTestMap::clear_prefix(netuid, limit, None),
            ));

            assert_eq!(ClearPrefixTestMap::iter_prefix(netuid).count(), 1);
            assert!(weight_meter.consumed().is_zero());
        });
    }

    #[test]
    fn test_clear_prefix_with_meter_completes_with_enough_budget() {
        let netuid = NetUid::from(44);
        let entry_weight = Weight::from_parts(REF_TIME_WEIGHT, PROOF_SIZE_WEIGHT);
        let mut ext = sp_io::TestExternalities::default();

        ext.execute_with(|| {
            for key in 0..3 {
                ClearPrefixTestMap::insert(netuid, key, key as u32);
            }
        });

        let _ = ext.commit_all();

        ext.execute_with(|| {
            // Budget for more entries than exist: everything is cleared in one call.
            let mut weight_meter = WeightMeter::with_limit(entry_weight.saturating_mul(10));
            assert!(clear_prefix_with_meter(
                &mut weight_meter,
                entry_weight,
                |limit| ClearPrefixTestMap::clear_prefix(netuid, limit, None),
            ));

            assert_eq!(ClearPrefixTestMap::iter_prefix(netuid).count(), 0);
            assert_eq!(weight_meter.consumed(), entry_weight.saturating_mul(3));
        });
    }
}
