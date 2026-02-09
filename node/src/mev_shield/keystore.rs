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
        Self(Mutex::new(ShieldKeys::new()))
    }

    pub fn roll_for_next_slot(&self) -> Result<(), anyhow::Error> {
        let mut keys = self
            .0
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock shield keystore: {}", e))?;

        keys.current_sk.zeroize();
        keys.current_sk = core::mem::take(&mut keys.next_sk);
        keys.current_pk = core::mem::take(&mut keys.next_pk);

        let (next_sk, next_pk) = MlKem768::generate(&mut OsRng);
        keys.next_sk = next_sk.into();
        keys.next_pk = next_pk.into();

        Ok(())
    }

    pub fn next_public_key(&self) -> Result<PublicKey, anyhow::Error> {
        let keys = self
            .0
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock shield keystore: {}", e))?;
        Ok(keys.next_pk.clone())
    }
}

#[derive(Default, Zeroize, ZeroizeOnDrop)]
struct PrivateKey(Vec<u8>);

impl From<DecapsulationKey<MlKem768Params>> for PrivateKey {
    fn from(key: DecapsulationKey<MlKem768Params>) -> Self {
        PrivateKey(key.as_bytes().to_vec())
    }
}

#[derive(Default, Clone, Encode)]
pub struct PublicKey(Vec<u8>);

impl From<EncapsulationKey<MlKem768Params>> for PublicKey {
    fn from(key: EncapsulationKey<MlKem768Params>) -> Self {
        PublicKey(key.as_bytes().to_vec())
    }
}

/// Holds the current/next ML‑KEM keypairs.
pub struct ShieldKeys {
    current_sk: PrivateKey,
    current_pk: PublicKey,
    next_sk: PrivateKey,
    next_pk: PublicKey,
}

impl ShieldKeys {
    pub fn new() -> Self {
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

impl Default for ShieldKeys {
    fn default() -> Self {
        Self::new()
    }
}

impl Zeroize for ShieldKeys {
    fn zeroize(&mut self) {
        self.current_sk.zeroize();
        self.next_sk.zeroize();
    }
}

impl ZeroizeOnDrop for ShieldKeys {}
