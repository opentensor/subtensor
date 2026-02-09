use codec::Encode;
use ml_kem::{
    EncodedSizeUser, KemCore, MlKem768, MlKem768Params,
    kem::{DecapsulationKey, EncapsulationKey},
};
use rand::rngs::OsRng;
use std::sync::Mutex;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// The keystore for the MEV-Shield.
pub struct ShieldKeystore(Mutex<ShieldKeys>);

impl ShieldKeystore {
    pub fn new() -> Self {
        Self(Mutex::new(ShieldKeys::generate()))
    }

    pub fn roll_for_next_slot(&self) -> Result<(), anyhow::Error> {
        let mut keys = self
            .0
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock shield keystore: {}", e))?;

        keys.current_sk.zeroize();

        // SAFETY: We are swapping the private keys and public keys in a safe way
        // without intermediate variables or implementing "Default" for the key types.
        unsafe {
            std::ptr::swap(&raw mut keys.current_sk, &raw mut keys.next_sk);
            std::ptr::swap(&raw mut keys.current_pk, &raw mut keys.next_pk);
        }

        let (next_sk, next_pk) = MlKem768::generate(&mut OsRng);
        keys.next_sk = next_sk.into();
        keys.next_pk = next_pk.into();

        Ok(())
    }

    pub fn next_public_key(&self) -> Result<Vec<u8>, anyhow::Error> {
        let keys = self
            .0
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock shield keystore: {}", e))?;
        Ok(keys.next_pk.0.clone())
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
struct PrivateKey(Vec<u8>);

impl From<DecapsulationKey<MlKem768Params>> for PrivateKey {
    fn from(key: DecapsulationKey<MlKem768Params>) -> Self {
        PrivateKey(key.as_bytes().to_vec())
    }
}

#[derive(Clone, Encode)]
pub struct PublicKey(Vec<u8>);

impl From<EncapsulationKey<MlKem768Params>> for PublicKey {
    fn from(key: EncapsulationKey<MlKem768Params>) -> Self {
        PublicKey(key.as_bytes().to_vec())
    }
}

/// Holds the current/next ML‑KEM keypairs in-memory for a single author.
struct ShieldKeys {
    current_sk: PrivateKey,
    current_pk: PublicKey,
    next_sk: PrivateKey,
    next_pk: PublicKey,
}

impl ShieldKeys {
    fn generate() -> Self {
        let (current_sk, current_pk) = MlKem768::generate(&mut OsRng);
        let (next_sk, next_pk) = MlKem768::generate(&mut OsRng);
        Self {
            current_sk: PrivateKey::from(current_sk),
            current_pk: PublicKey::from(current_pk),
            next_sk: PrivateKey::from(next_sk),
            next_pk: PublicKey::from(next_pk),
        }
    }
}

impl Zeroize for ShieldKeys {
    fn zeroize(&mut self) {
        self.current_sk.zeroize();
        self.next_sk.zeroize();
    }
}

impl ZeroizeOnDrop for ShieldKeys {}
