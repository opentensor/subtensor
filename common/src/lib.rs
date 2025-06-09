#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use runtime_common::prod_or_fast;
use scale_info::TypeInfo;
use sp_runtime::{
    MultiSignature,
    traits::{IdentifyAccount, Verify},
};

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

pub mod time {
    use super::*;

    /// This determines the average expected block time that we are targeting. Blocks will be
    /// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
    /// `pallet_timestamp` which is in turn picked up by `pallet_babe` to implement `fn
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
