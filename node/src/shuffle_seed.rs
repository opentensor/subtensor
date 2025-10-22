use rand::{RngCore, rngs::OsRng};
use sp_core::hashing::blake2_256;
use zeroize::Zeroizing;

pub fn ephemeral_block_salt<H: AsRef<[u8]>>(parent_hash: H) -> [u8; 32] {
    // Fresh per-block seed; automatically zeroized when it goes out of scope.
    let mut seed = Zeroizing::new([0u8; 32]);
    OsRng.fill_bytes(seed.as_mut());

    let mut buf = Zeroizing::new([0u8; 64]);
    buf[..32].copy_from_slice(&*seed);

    let h = parent_hash.as_ref();
    let n = core::cmp::min(h.len(), 32);
    buf[32..32 + n].copy_from_slice(&h[..n]);

    blake2_256(&buf[..32 + n])
}