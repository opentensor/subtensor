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
    MultiSignature,
    traits::{IdentifyAccount, Verify},
};
use subtensor_macros::freeze_struct;

pub use currency::*;

mod currency;

/// Balance of an account.
pub type Balance = u64;

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
pub const SMALL_TRANSFER_LIMIT: Balance = 500_000_000; // 0.5 TAO

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
    NonFungibile, // Nothing involving moving TAO
    Triumvirate,
    Governance, // Both above governance
    Staking,
    Registration,
    Transfer,
    SmallTransfer,
    RootWeights,
    ChildKeys,
    SudoUncheckedSetCode,
    SwapHotkey,
    SubnetLeaseBeneficiary, // Used to operate the leased subnet
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
}

pub trait CurrencyReserve<C: Currency> {
    fn reserve(netuid: NetUid) -> C;
    fn increase_provided(netuid: NetUid, amount: C);
    fn decrease_provided(netuid: NetUid, amount: C);
}

pub trait BalanceOps<AccountId> {
    fn tao_balance(account_id: &AccountId) -> TaoCurrency;
    fn alpha_balance(netuid: NetUid, coldkey: &AccountId, hotkey: &AccountId) -> AlphaCurrency;
    fn increase_balance(coldkey: &AccountId, tao: TaoCurrency);
    fn decrease_balance(
        coldkey: &AccountId,
        tao: TaoCurrency,
    ) -> Result<TaoCurrency, DispatchError>;
    fn increase_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
    ) -> Result<(), DispatchError>;
    fn decrease_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: AlphaCurrency,
    ) -> Result<AlphaCurrency, DispatchError>;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn netuid_has_u16_bin_repr() {
        assert_eq!(NetUid(5).encode(), 5u16.encode());
    }
}
