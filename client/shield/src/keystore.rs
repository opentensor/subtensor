use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload},
};
use ml_kem::{
    Ciphertext, EncodedSizeUser, KemCore, MlKem768, MlKem768Params,
    kem::{Decapsulate, DecapsulationKey, EncapsulationKey},
};
use rand::rngs::OsRng;
use std::sync::RwLock;
use stp_shield::{Error as TraitError, Result as TraitResult, ShieldKeystore};

/// An memory-based keystore for the MEV-Shield.
pub struct MemoryShieldKeystore(RwLock<ShieldKeystoreInner>);

impl MemoryShieldKeystore {
    pub fn new() -> Self {
        Self(RwLock::new(ShieldKeystoreInner::new()))
    }
}

impl ShieldKeystore for MemoryShieldKeystore {
    fn roll_for_next_slot(&self) -> TraitResult<()> {
        self.0
            .write()
            .map_err(|_| TraitError::Unavailable)?
            .roll_for_next_slot()
    }

    fn next_public_key(&self) -> TraitResult<Vec<u8>> {
        self.0
            .read()
            .map_err(|_| TraitError::Unavailable)?
            .next_public_key()
    }

    fn mlkem768_decapsulate(&self, ciphertext: &[u8]) -> TraitResult<[u8; 32]> {
        self.0
            .read()
            .map_err(|_| TraitError::Unavailable)?
            .mlkem768_decapsulate(ciphertext)
    }

    fn aead_decrypt(
        &self,
        key: [u8; 32],
        nonce: [u8; 24],
        msg: &[u8],
        aad: &[u8],
    ) -> TraitResult<Vec<u8>> {
        self.0
            .read()
            .map_err(|_| TraitError::Unavailable)?
            .aead_decrypt(key, nonce, msg, aad)
    }
}

/// Holds the current/next ML‑KEM keypairs in-memory for a single author.
pub struct ShieldKeystoreInner {
    current_pair: ShieldKeyPair,
    next_pair: ShieldKeyPair,
}

impl ShieldKeystoreInner {
    fn new() -> Self {
        Self {
            current_pair: ShieldKeyPair::generate(),
            next_pair: ShieldKeyPair::generate(),
        }
    }

    fn roll_for_next_slot(&mut self) -> TraitResult<()> {
        std::mem::swap(&mut self.current_pair, &mut self.next_pair);
        self.next_pair = ShieldKeyPair::generate();
        Ok(())
    }

    fn next_public_key(&self) -> TraitResult<Vec<u8>> {
        Ok(self.next_pair.enc_key.as_bytes().to_vec())
    }

    fn mlkem768_decapsulate(&self, ciphertext: &[u8]) -> TraitResult<[u8; 32]> {
        let ciphertext = Ciphertext::<MlKem768>::try_from(ciphertext)
            .map_err(|e| TraitError::ValidationError(e.to_string()))?;
        let shared_secret = self
            .current_pair
            .dec_key
            .decapsulate(&ciphertext)
            .map_err(|_| {
                TraitError::Other("Failed to decapsulate ciphertext using ML-KEM 768".into())
            })?;

        Ok(shared_secret.into())
    }

    fn aead_decrypt(
        &self,
        key: [u8; 32],
        nonce: [u8; 24],
        msg: &[u8],
        aad: &[u8],
    ) -> TraitResult<Vec<u8>> {
        let aead = XChaCha20Poly1305::new((&key).into());
        let nonce = XNonce::from_slice(&nonce);
        let payload = Payload { msg, aad };
        let decrypted = aead
            .decrypt(nonce, payload)
            .map_err(|_| TraitError::Other("Failed to decrypt message using AEAD".into()))?;

        Ok(decrypted)
    }
}

/// A pair of ML‑KEM‑768 decapsulation and encapsulation keys.
#[derive(Debug, Clone)]
struct ShieldKeyPair {
    dec_key: DecapsulationKey<MlKem768Params>,
    enc_key: EncapsulationKey<MlKem768Params>,
}

impl ShieldKeyPair {
    fn generate() -> Self {
        let (dec_key, enc_key) = MlKem768::generate(&mut OsRng);
        Self { dec_key, enc_key }
    }
}
