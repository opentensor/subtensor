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
pub use transaction_error::*;

mod currency;
mod evm_context;
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

impl Default for ProxyType {
    // allow all Calls; required to be most permissive
    fn default() -> Self {
        Self::Any
    }
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

#[macro_export]
macro_rules! WeightMeterWrapper {
    ( $meter:expr, $weight:expr ) => {{
        if !$meter.can_consume($weight.clone()) {
            return false;
        }
        $meter.consume($weight.clone());
    }};
}

#[macro_export]
macro_rules! LoopRemovePrefixWithWeightMeter {
    ( $meter:expr, $weight:expr, $storage:ty, $netuid:expr ) => {{
        let limit = $meter
            .remaining()
            .checked_div_per_component(&$weight.clone());
        match limit {
            Some(limit) => {
                let limit = u32::try_from(limit).unwrap_or(u32::MAX);
                let result: $crate::MultiRemovalResults =
                    <$storage>::clear_prefix($netuid, limit, None);
                $meter.consume($weight.saturating_mul(result.backend.into()));

                let remove_all = result.maybe_cursor.is_none();
                if !remove_all {
                    return false;
                }
            }
            None => return false,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;
    use frame_support::weights::WeightMeter;
    const REF_TIME_WEIGHT: u64 = 100;
    const PROOF_SIZE_WEIGHT: u64 = 100;

    #[test]
    fn netuid_has_u16_bin_repr() {
        assert_eq!(NetUid(5).encode(), 5u16.encode());
    }

    fn test_weight(weight_meter: &mut WeightMeter, weight: Weight) -> bool {
        WeightMeterWrapper!(weight_meter, weight);
        true
    }

    #[test]
    fn test_weight_meter_wrapper() {
        // Enough budget for one (ref, proof) unit of `weight`.
        let remaining_weight = Weight::from_parts(REF_TIME_WEIGHT * 2, PROOF_SIZE_WEIGHT * 2);
        let weight = Weight::from_parts(REF_TIME_WEIGHT, PROOF_SIZE_WEIGHT);
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        assert!(test_weight(&mut weight_meter, weight));

        // Not enough to consume 3x ref and 3x proof in one step.
        let mut weight_meter = WeightMeter::with_limit(remaining_weight);
        let consumed = test_weight(
            &mut weight_meter,
            Weight::from_parts(REF_TIME_WEIGHT * 3, PROOF_SIZE_WEIGHT * 3),
        );
        assert!(!consumed);
    }

    // --- LoopRemovePrefixWithWeightMeter integration (stub storage) ---

    thread_local! {
        static LAST_CLEAR_LIMIT: Cell<u32> = const { Cell::new(0) };
    }

    /// Stub: all keys removed in one batch; obeys `limit` for debugging assertions.
    struct LoopRemoveStubFull;
    impl LoopRemoveStubFull {
        fn clear_prefix<K>(_prefix: K, limit: u32, _maybe: Option<&[u8]>) -> MultiRemovalResults {
            LAST_CLEAR_LIMIT.with(|c| c.set(limit));
            MultiRemovalResults {
                maybe_cursor: None,
                backend: limit,
                unique: limit,
                loops: if limit == 0 { 0 } else { 1 },
            }
        }
    }

    /// Stub: always reports partial removal (cursor set).
    struct LoopRemoveStubPartial;
    impl LoopRemoveStubPartial {
        fn clear_prefix<K>(_prefix: K, _limit: u32, _maybe: Option<&[u8]>) -> MultiRemovalResults {
            MultiRemovalResults {
                maybe_cursor: Some(vec![0xAB]),
                backend: 1,
                unique: 1,
                loops: 1,
            }
        }
    }

    fn last_limit() -> u32 {
        LAST_CLEAR_LIMIT.with(|c| c.get())
    }

    fn run_loop_remove_full(meter: &mut WeightMeter, per_item: Weight, netuid: u16) -> bool {
        LoopRemovePrefixWithWeightMeter!(meter, per_item, LoopRemoveStubFull, netuid);
        true
    }

    fn run_loop_remove_partial(meter: &mut WeightMeter, per_item: Weight) -> bool {
        LoopRemovePrefixWithWeightMeter!(meter, per_item, LoopRemoveStubPartial, 0u16);
        true
    }

    #[test]
    fn loop_remove_clear_limit_is_budget_over_per_write_ref_time() {
        LAST_CLEAR_LIMIT.with(|c| c.set(0));
        let mut meter = WeightMeter::with_limit(Weight::from_parts(5_000, 0));
        let per = Weight::from_parts(200, 0);
        let done = run_loop_remove_full(&mut meter, per, 7);
        assert!(done);
        assert_eq!(last_limit(), 25, "5000 / 200 = 25 deletions per batch");
        assert_eq!(
            meter.consumed().ref_time(),
            5_000,
            "charges write_ref_time * limit_64"
        );
        assert_eq!(meter.consumed().proof_size(), 0);
    }

    #[test]
    fn loop_remove_zero_remaining_ref_time_yields_zero_limit() {
        LAST_CLEAR_LIMIT.with(|c| c.set(u32::MAX));
        let mut meter = WeightMeter::with_limit(Weight::zero());
        let done = run_loop_remove_full(&mut meter, Weight::from_parts(100, 0), 1);
        assert!(done);
        assert_eq!(last_limit(), 0);
    }

    #[test]
    fn loop_remove_zero_write_ref_time_uses_zero_limit_via_checked_div() {
        LAST_CLEAR_LIMIT.with(|c| c.set(u32::MAX));
        let mut meter = WeightMeter::with_limit(Weight::from_parts(9_999, 9_999));
        // Non-zero proof_size does not affect the macro (ref_time-only math).
        let per = Weight::from_parts(0, 500);
        let done = run_loop_remove_full(&mut meter, per, 2);
        assert!(done);
        assert_eq!(last_limit(), 9_999 / 500);
    }

    #[test]
    fn loop_remove_limit_truncates_to_u32_max_when_budget_huge() {
        LAST_CLEAR_LIMIT.with(|c| c.set(0));
        let mut meter = WeightMeter::with_limit(Weight::from_parts(u64::MAX, 0));
        let per = Weight::from_parts(1, 0);
        let done = run_loop_remove_full(&mut meter, per, 3);
        assert!(done);
        assert_eq!(last_limit(), u32::MAX);
    }

    #[test]
    fn loop_remove_partial_cursor_returns_false_from_enclosing_fn() {
        let mut meter = WeightMeter::with_limit(Weight::from_parts(100_001, 0));
        let before = meter.consumed();
        let done = run_loop_remove_partial(&mut meter, Weight::from_parts(400, 0));
        assert!(!done);
        assert!(
            meter.consumed().ref_time() > before.ref_time(),
            "batch cost applied before early return"
        );
        assert!(
            meter.consumed().all_lte(meter.limit()),
            "consumption must stay within the meter limit for this call"
        );
    }

    #[test]
    fn loop_remove_second_full_pass_uses_remaining_budget() {
        let mut meter = WeightMeter::with_limit(Weight::from_parts(5_000, 0));
        let per = Weight::from_parts(100, 0);
        assert!(run_loop_remove_full(&mut meter, per, 0));
        LAST_CLEAR_LIMIT.with(|c| c.set(u32::MAX));
        let _ = run_loop_remove_full(&mut meter, per, 0);
        assert_eq!(last_limit(), 0);
    }

    #[test]
    fn loop_remove_exact_multiple_consumes_full_budget_ref_component() {
        LAST_CLEAR_LIMIT.with(|c| c.set(0));
        let mut meter = WeightMeter::with_limit(Weight::from_parts(800, 0));
        let per = Weight::from_parts(100, 0);
        let done = run_loop_remove_full(&mut meter, per, 99);
        assert!(done);
        assert_eq!(last_limit(), 8);
        assert_eq!(meter.consumed().ref_time(), 800);
    }
}
