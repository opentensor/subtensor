#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::H256;
use stp_mev_shield_ibe::IbePendingIdentity;

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub enum MevShieldExtrinsicClass {
    /// Operational or mandatory extrinsic that may appear even while the
    /// encrypted queue is pending.
    Operational,

    /// Normal unencrypted extrinsic. If the v2 encrypted queue is nonempty,
    /// block import rejects this because it preempts the encrypted queue.
    UnencryptedNonOperational,

    /// Existing v1 MEVShield wrapper. This remains valid and must not be
    /// interpreted as v2 threshold IBE.
    SubmitEncryptedV1,

    /// v2 threshold-IBE wrapper.
    SubmitEncryptedV2 {
        epoch: u64,
        target_block: u64,
        key_id: [u8; 16],
        commitment: H256,
    },

    /// v2 reconstructed block identity decryption key.
    SubmitBlockDecryptionKey {
        epoch: u64,
        target_block: u64,
        key_id: [u8; 16],
        finalized_ordering_block_number: u64,
        finalized_ordering_block_hash: H256,
    },
}

sp_api::decl_runtime_apis! {
    pub trait MevShieldIbeApi {
        fn pending_ibe_identities(limit: u32) -> Vec<IbePendingIdentity>;

        fn has_ibe_block_key(
            epoch: u64,
            target_block: u64,
            key_id: [u8; 16],
        ) -> bool;

        fn pending_encrypted_queue_len() -> u32;

        fn classify_extrinsic(encoded_xt: Vec<u8>) -> MevShieldExtrinsicClass;
    }
}