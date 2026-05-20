use ml_kem::{EncodedSizeUser, KemCore, MlKem768};
use rand_core::OsRng;
use stp_shield::{Result as TraitResult, ShieldKeystore};

/// A fixed (non-rotating) shield keystore for single-validator dev/manual-seal nodes.
///
/// Uses the same ML-KEM-768 keypair for both `next_enc_key()` and `current_dec_key()`,
/// bypassing the multi-validator key-rotation timing assumption. In a real multi-validator
/// AURA chain, each validator builds every Kth block, so the keystore rolls at the same
/// cadence as the on-chain PendingKey pipeline (2 blocks). In single-validator manual-seal
/// mode the keystore would roll on every block, drifting 2 pairs ahead of PendingKey.
/// This keystore avoids that by keeping both keys from the same generated pair.
pub struct DevShieldKeystore {
    enc_key_bytes: Vec<u8>,
    dec_key_bytes: Vec<u8>,
}

impl DevShieldKeystore {
    pub fn new() -> Self {
        let (dec_key, enc_key) = MlKem768::generate(&mut OsRng);
        Self {
            enc_key_bytes: enc_key.as_bytes().to_vec(),
            dec_key_bytes: dec_key.as_bytes().to_vec(),
        }
    }
}

impl ShieldKeystore for DevShieldKeystore {
    fn roll_for_next_slot(&self) -> TraitResult<()> {
        Ok(())
    }

    fn next_enc_key(&self) -> TraitResult<Vec<u8>> {
        Ok(self.enc_key_bytes.clone())
    }

    fn current_dec_key(&self) -> TraitResult<Vec<u8>> {
        Ok(self.dec_key_bytes.clone())
    }
}
