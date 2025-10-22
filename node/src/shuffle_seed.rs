use rand::{RngCore, rngs::OsRng};
use sp_core::hashing::blake2_256;
use zeroize::Zeroizing;

pub fn ephemeral_block_salt<H: AsRef<[u8]>>(parent_hash: H) -> [u8; 32] {
    // Fresh per-block seed that gets wiped on scope exit.
    let mut seed = Zeroizing::new([0u8; 32]);
    OsRng.fill_bytes(seed.as_mut());

    let mut data = Zeroizing::new(Vec::with_capacity(64));
    data.extend(seed.iter().copied());
    data.extend(parent_hash.as_ref().iter().copied().take(32));

    blake2_256(&data)
}
