//! Runtime/session key helpers for MeV Shield v2 epoch-ahead DKG.
//!
//! The DKG authority set is stake-bearing Subtensor hotkeys joined to the active
//! consensus key registered by that hotkey.  This helper supports the POA->POS
//! transition automatically by discovering both local Aura and BABE keys and by
//! signing DKG messages with the key kind selected by the runtime plan.

use std::{fs, path::Path};

use mev_shield_ibe_runtime_api::DkgConsensusKeyKind;
use rand_core::{OsRng, RngCore};
use sp_core::{H256, crypto::KeyTypeId, sr25519};
use sp_keystore::{Keystore, KeystorePtr};

use super::dkg_worker::{AuthoritySigner, LocalConsensusAuthority};

pub const AURA_KEY_TYPE: KeyTypeId = KeyTypeId(*b"aura");
pub const BABE_KEY_TYPE: KeyTypeId = KeyTypeId(*b"babe");
/// Loads a persistent X25519 DKG-transport secret, or creates it on first boot.
///
/// This key is not the threshold IBE secret.  It only encrypts per-round DKG
/// shares between validators.  It must be durable so an authority can recover
/// during a still-active DKG round.
pub fn load_or_generate_x25519_secret(path: impl AsRef<Path>) -> Result<[u8; 32], String> {
    let path = path.as_ref();
    if let Ok(bytes) = fs::read(path) {
        if bytes.len() == 32 {
            let mut out = [0u8; 32];
            out.copy_from_slice(&bytes);
            return Ok(out);
        }
        return Err(format!(
            "invalid X25519 secret length in {}",
            path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create DKG key directory: {e}"))?;
    }
    let mut out = [0u8; 32];
    OsRng.fill_bytes(&mut out);
    fs::write(path, out).map_err(|e| format!("write X25519 DKG secret: {e}"))?;
    Ok(out)
}

/// Discover local consensus authority keys that can participate in DKG.
///
/// Before POS activation this returns the local Aura key.  Once BABE session keys
/// are present it also returns the local BABE key.  The runtime plan chooses
/// which one is valid for a given epoch; the worker simply matches the plan.
pub fn local_consensus_authorities(keystore: &KeystorePtr) -> Vec<LocalConsensusAuthority> {
    let mut out = Vec::new();
    for pk in keystore.sr25519_public_keys(AURA_KEY_TYPE) {
        out.push(LocalConsensusAuthority {
            consensus_key_kind: DkgConsensusKeyKind::AuraSr25519,
            authority_id: pk.0.to_vec(),
            signature_key_hint: pk.0.to_vec(),
        });
    }
    for pk in keystore.sr25519_public_keys(BABE_KEY_TYPE) {
        out.push(LocalConsensusAuthority {
            consensus_key_kind: DkgConsensusKeyKind::BabeSr25519,
            authority_id: pk.0.to_vec(),
            signature_key_hint: pk.0.to_vec(),
        });
    }
    out.sort_by(|a, b| {
        (a.consensus_key_kind, &a.authority_id).cmp(&(b.consensus_key_kind, &b.authority_id))
    });
    out.dedup_by(|a, b| {
        a.consensus_key_kind == b.consensus_key_kind && a.authority_id == b.authority_id
    });
    out
}

#[derive(Clone)]
pub struct SubtensorAuthoritySigner {
    keystore: KeystorePtr,
}

impl SubtensorAuthoritySigner {
    pub fn new(keystore: KeystorePtr) -> Self {
        Self { keystore }
    }

    fn try_sr25519(
        &self,
        key_type: KeyTypeId,
        key_hint: &[u8],
        payload_hash: H256,
    ) -> Result<Option<Vec<u8>>, String> {
        if key_hint.len() != 32 {
            return Ok(None);
        }
        let mut public = [0u8; 32];
        public.copy_from_slice(key_hint);
        let public = sr25519::Public::from_raw(public);
        match self
            .keystore
            .sr25519_sign(key_type, &public, payload_hash.as_bytes())
        {
            Ok(Some(sig)) => Ok(Some(sig.0.to_vec())),
            Ok(None) => Ok(None),
            Err(e) => Err(format!("sr25519 authority signing failed: {e}")),
        }
    }
}

impl AuthoritySigner for SubtensorAuthoritySigner {
    fn sign(
        &self,
        key_kind: DkgConsensusKeyKind,
        key_hint: &[u8],
        payload_hash: H256,
    ) -> Result<Vec<u8>, String> {
        match key_kind {
            DkgConsensusKeyKind::AuraSr25519 => self
                .try_sr25519(AURA_KEY_TYPE, key_hint, payload_hash)?
                .ok_or_else(|| "no matching local Aura authority key for DKG signing".into()),
            DkgConsensusKeyKind::BabeSr25519 => self
                .try_sr25519(BABE_KEY_TYPE, key_hint, payload_hash)?
                .ok_or_else(|| "no matching local BABE authority key for DKG signing".into()),
            DkgConsensusKeyKind::BabeEd25519 => {
                Err("BABE ed25519 DKG signing is not configured for this runtime".into())
            }
        }
    }
}
