#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{bounded_vec::BoundedVec, H256};
use sp_runtime::traits::ConstU32;

pub const MEV_SHIELD_IBE_VERSION: u16 = 1;
pub const MEV_SHIELD_IBE_MAGIC: [u8; 4] = *b"MSI2";

pub const KEY_ID_LEN: usize = 16;
pub const IBE_DOMAIN: &[u8] = b"bittensor.mev-shield.v2.block-identity";

/// BLS12-381 compressed G1 signature / BF-IBE extracted identity secret.
pub const COMPRESSED_IDENTITY_KEY_LEN: usize = 48;

/// BLS12-381 compressed G2 master public key.
pub const COMPRESSED_MASTER_PUBLIC_KEY_LEN: usize = 96;

pub const COMPRESSED_IDENTITY_KEY_LEN_U32: u32 = 48;
pub const COMPRESSED_MASTER_PUBLIC_KEY_LEN_U32: u32 = 96;

/// v2 envelope stored in PR-2533 PendingExtrinsics.
///
/// Important: this ciphertext encrypts the signed inner extrinsic, not a RuntimeCall.
#[subtensor_macros::freeze_struct("d1904a1f68dc401a")]
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct IbeEncryptedExtrinsicV1 {
    /// Must be `MSI2`, so existing v1 ML-KEM ciphertexts cannot be misdecoded as v2.
    pub magic: [u8; 4],

    pub version: u16,

    /// Epoch whose IBE master public key was used.
    pub epoch: u64,

    /// The block-number IBE identity. Clients target current_block + 2.
    pub target_block: u64,

    /// Identifies the epoch DKG output / master public key.
    pub key_id: [u8; KEY_ID_LEN],

    /// Hash of the plaintext signed inner extrinsic. Used for dedup/accountability.
    pub commitment: H256,

    /// Canonically serialized `tle::tlock::TLECiphertext<TinyBLS381>`.
    pub ciphertext: Vec<u8>,
}

impl IbeEncryptedExtrinsicV1 {
    pub fn is_v2_prefixed(bytes: &[u8]) -> bool {
        bytes.starts_with(&MEV_SHIELD_IBE_MAGIC)
    }

    pub fn decode_v2(bytes: &[u8]) -> Result<Self, ()> {
        let envelope = Self::decode(&mut &bytes[..]).map_err(|_| ())?;

        if envelope.magic != MEV_SHIELD_IBE_MAGIC || envelope.version != MEV_SHIELD_IBE_VERSION {
            return Err(());
        }

        Ok(envelope)
    }
}

#[subtensor_macros::freeze_struct("10aeaaf81afa6e70")]
#[derive(
    Clone, Eq, PartialEq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct IbeEpochPublicKey {
    pub epoch: u64,
    pub key_id: [u8; KEY_ID_LEN],

    /// Compressed BLS12-381 G2 master public key: g2^msk.
    pub master_public_key: BoundedMasterPublicKey,

    /// Total active stake weight represented by this DKG.
    pub total_weight: u128,

    /// Usually ceil(2/3 * total_weight).
    pub threshold_weight: u128,

    pub first_block: u64,
    pub last_block: u64,
}

pub type BoundedMasterPublicKey = BoundedVec<u8, ConstU32<COMPRESSED_MASTER_PUBLIC_KEY_LEN_U32>>;

pub type BoundedIdentityKey = BoundedVec<u8, ConstU32<COMPRESSED_IDENTITY_KEY_LEN_U32>>;

#[subtensor_macros::freeze_struct("8b8dbad2d6b33368")]
#[derive(
    Clone, Eq, PartialEq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct IbeBlockDecryptionKeyV1 {
    pub version: u16,
    pub epoch: u64,
    pub target_block: u64,
    pub key_id: [u8; KEY_ID_LEN],

    /// d_id = H(identity)^msk.
    pub identity_decryption_key: BoundedIdentityKey,

    /// Finalized block proving the encrypted ordering was locked before release.
    pub finalized_ordering_block_number: u64,
    pub finalized_ordering_block_hash: H256,
}

#[subtensor_macros::freeze_struct("637a2b7834d1086")]
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct IbePendingIdentity {
    pub epoch: u64,
    pub target_block: u64,
    pub key_id: [u8; KEY_ID_LEN],
    pub first_queue_index: u32,
    pub last_queue_index: u32,
}

#[subtensor_macros::freeze_struct("ab897b1f761e7743")]
#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct IbePartialDecryptionKeyShareV1 {
    pub version: u16,
    pub epoch: u64,
    pub target_block: u64,
    pub key_id: [u8; KEY_ID_LEN],

    pub finalized_ordering_block_number: u64,
    pub finalized_ordering_block_hash: H256,

    /// Weighted Shamir atom id, not validator index.
    pub share_id: u32,
    pub weight: u128,

    /// Compressed g2^s_i.
    pub public_share: Vec<u8>,

    /// Compressed H(identity)^s_i.
    pub partial_identity_key: Vec<u8>,
}

pub fn plaintext_commitment(inner_signed_extrinsic: &[u8]) -> H256 {
    H256::from(sp_core::blake2_256(inner_signed_extrinsic))
}

/// The semantic IBE identity is `target_block`.
/// The other fields are domain-separation context so the same block number
/// cannot be replayed across chains, epochs, or DKG outputs.
pub fn block_identity_bytes(
    genesis_hash: H256,
    epoch: u64,
    target_block: u64,
    key_id: [u8; KEY_ID_LEN],
) -> Vec<u8> {
    (IBE_DOMAIN, genesis_hash, epoch, target_block, key_id).encode()
}

pub fn block_key_storage_key(
    epoch: u64,
    target_block: u64,
    key_id: [u8; KEY_ID_LEN],
) -> (u64, u64, [u8; KEY_ID_LEN]) {
    (epoch, target_block, key_id)
}
