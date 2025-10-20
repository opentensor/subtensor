use rand::{RngCore, rngs::OsRng};
use sp_core::hashing::blake2_256;

pub fn ephemeral_block_salt<H: AsRef<[u8]>>(parent_hash: H) -> [u8; 32] {
    let mut seed = [0u8; 32];
    OsRng.fill_bytes(&mut seed);

    let h = parent_hash.as_ref();
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&seed);
    let n = core::cmp::min(h.len(), 32);
    buf[32..32 + n].copy_from_slice(&h[..n]);

    blake2_256(&buf[..32 + n])
}
