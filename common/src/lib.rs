#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
    MultiSignature,
    traits::{IdentifyAccount, Verify},
};

use subtensor_macros::freeze_struct;

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

#[freeze_struct("2a62496e31bbcddc")]
#[derive(
    Clone, Copy, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
)]
pub struct NetUid(u16);

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

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, MaxEncodedLen, TypeInfo,
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
}

impl Default for ProxyType {
    // allow all Calls; required to be most permissive
    fn default() -> Self {
        Self::Any
    }
}

pub trait SubnetInfo<AccountId> {
    fn tao_reserve(netuid: NetUid) -> u64;
    fn alpha_reserve(netuid: NetUid) -> u64;
    fn exists(netuid: NetUid) -> bool;
    fn mechanism(netuid: NetUid) -> u16;
    fn is_owner(account_id: &AccountId, netuid: NetUid) -> bool;
}

pub trait BalanceOps<AccountId> {
    fn tao_balance(account_id: &AccountId) -> u64;
    fn alpha_balance(netuid: NetUid, coldkey: &AccountId, hotkey: &AccountId) -> u64;
    fn increase_balance(coldkey: &AccountId, tao: u64);
    fn decrease_balance(coldkey: &AccountId, tao: u64) -> Result<u64, DispatchError>;
    fn increase_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: u64,
    ) -> Result<(), DispatchError>;
    fn decrease_stake(
        coldkey: &AccountId,
        hotkey: &AccountId,
        netuid: NetUid,
        alpha: u64,
    ) -> Result<u64, DispatchError>;
}

pub mod time {
    use super::*;

    /// This determines the average expected block time that we are targeting. Blocks will be
    /// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
    /// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
    /// slot_duration()`.
    ///
    /// Change this to adjust the block time.
    #[cfg(not(feature = "fast-blocks"))]
    pub const MILLISECS_PER_BLOCK: u64 = 12000;

    /// Fast blocks for development
    #[cfg(feature = "fast-blocks")]
    pub const MILLISECS_PER_BLOCK: u64 = 250;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // Time is measured by number of blocks.
    pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}
