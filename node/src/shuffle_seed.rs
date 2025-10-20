use rand::{RngCore, rngs::OsRng};
use sp_core::hashing::blake2_256;
use std::sync::OnceLock;

static VALIDATOR_SEED: OnceLock<[u8; 32]> = OnceLock::new();

pub fn init_if_needed() -> [u8; 32] {
    *VALIDATOR_SEED.get_or_init(|| {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        seed
    })
}

pub fn seed() -> [u8; 32] {
    *VALIDATOR_SEED
        .get()
        .expect("seed initialized at startup; qed")
}

/// Derive per-block salt from (secret) validator seed and the parent hash.
pub fn block_shuffle_seed<H: AsRef<[u8]>>(prev_hash: H) -> [u8; 32] {
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&seed());
    let h = prev_hash.as_ref();
    let n = core::cmp::min(h.len(), 32);
    buf[32..32 + n].copy_from_slice(&h[..n]);
    blake2_256(&buf[..32 + n])
}
