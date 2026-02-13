//! MEV Shielded Transaction

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, Encode};
use scale_info::TypeInfo;

const KEY_HASH_LEN: usize = 16;

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub struct ShieldedTransaction {
    pub key_hash: [u8; KEY_HASH_LEN],
    pub kem_ct: Vec<u8>,
    pub aead_ct: Vec<u8>,
    pub nonce: [u8; 24],
}

impl ShieldedTransaction {
    pub fn parse(ciphertext: &[u8]) -> Option<Self> {
        let mut cursor: usize = 0;

        let key_hash_end = cursor.checked_add(KEY_HASH_LEN)?;
        let key_hash: [u8; KEY_HASH_LEN] = ciphertext.get(cursor..key_hash_end)?.try_into().ok()?;
        cursor = key_hash_end;

        let kem_ct_len_end = cursor.checked_add(2)?;
        let kem_ct_len = ciphertext
            .get(cursor..kem_ct_len_end)?
            .try_into()
            .map(u16::from_le_bytes)
            .ok()?
            .into();
        cursor = kem_ct_len_end;

        let kem_ct_end = cursor.checked_add(kem_ct_len)?;
        let kem_ct = ciphertext.get(cursor..kem_ct_end)?.to_vec();
        cursor = kem_ct_end;

        const NONCE_LEN: usize = 24;
        let nonce_end = cursor.checked_add(NONCE_LEN)?;
        let nonce = ciphertext.get(cursor..nonce_end)?.try_into().ok()?;
        cursor = nonce_end;

        let aead_ct = ciphertext.get(cursor..)?.to_vec();

        Some(Self {
            key_hash,
            kem_ct,
            aead_ct,
            nonce,
        })
    }
}
