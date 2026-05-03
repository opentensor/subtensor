//! Local unsigned submission of stake-hotkey ↔ consensus-key ↔ DKG-transport-key bindings.
//!
//! This is the POS-critical registration path.  The runtime derives DKG stake
//! from Subtensor root-validator hotkeys, but DKG messages are signed by the
//! live consensus key for the epoch (Aura before the transition, BABE after the
//! transition).  A registration is accepted only when both the hotkey and the
//! consensus key sign the same DKG transport key.

use std::{sync::Arc, time::Duration};

use codec::Encode;
use futures::FutureExt;
use mev_shield_ibe_runtime_api::DkgAuthorityRegistration;
use node_subtensor_runtime::{
    RuntimeCall, UncheckedExtrinsic,
    opaque::{Block, UncheckedExtrinsic as OpaqueExtrinsic},
};
use sc_client_api::HeaderBackend;
use sc_service::SpawnTaskHandle;
use sc_transaction_pool::TransactionPoolHandle;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_runtime::OpaqueExtrinsic as SpOpaqueExtrinsic;

use crate::client::FullClient;

use super::{
    dkg_runtime_keys::{
        SubtensorAuthoritySigner, authority_registration_payload_hash,
        build_dkg_authority_registration,
    },
    dkg_worker::{AuthoritySigner, LocalConsensusAuthority},
};

fn build_unsigned_authority_registration_extrinsic(
    registration: DkgAuthorityRegistration,
) -> Option<OpaqueExtrinsic> {
    let call = RuntimeCall::MevShield(pallet_shield::Call::submit_ibe_dkg_authority_registration {
        registration,
    });
    let unchecked = UncheckedExtrinsic::new_bare(call);
    SpOpaqueExtrinsic::from_bytes(&unchecked.encode()).ok()
}

pub fn spawn_dkg_authority_registration_submitter(
    spawn: &SpawnTaskHandle,
    client: Arc<FullClient>,
    transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>,
    signer: Arc<SubtensorAuthoritySigner>,
    local_hotkey_public_keys: Vec<Vec<u8>>,
    local_consensus_authorities: Vec<LocalConsensusAuthority>,
    dkg_x25519_public_key: [u8; 32],
) {
    spawn.spawn(
        "mev-shield-ibe-dkg-authority-registration-submitter",
        None,
        Box::pin(async move {
            if local_hotkey_public_keys.is_empty() {
                log::warn!(
                    target: "mev-shield-ibe",
                    "no local Subtensor hotkey key found under key type bthk; POS stake-weighted MeV Shield DKG registration will not be submitted",
                );
                return;
            }
            if local_consensus_authorities.is_empty() {
                log::warn!(
                    target: "mev-shield-ibe",
                    "no local Aura/BABE authority keys found; MeV Shield DKG registration will not be submitted",
                );
                return;
            }

            let mut extrinsics = Vec::new();
            for hotkey in &local_hotkey_public_keys {
                for consensus in &local_consensus_authorities {
                    let payload = authority_registration_payload_hash(
                        hotkey,
                        consensus.consensus_key_kind,
                        &consensus.authority_id,
                        &dkg_x25519_public_key,
                    );

                    let Ok(hotkey_signature) = signer.sign_hotkey(hotkey, payload) else {
                        log::warn!(target: "mev-shield-ibe", "unable to sign DKG authority registration with local hotkey for {:?}", consensus.consensus_key_kind);
                        continue;
                    };
                    let Ok(consensus_signature) = signer.sign(
                        consensus.consensus_key_kind,
                        &consensus.signature_key_hint,
                        payload,
                    ) else {
                        log::warn!(target: "mev-shield-ibe", "unable to sign DKG authority registration with local consensus key for {:?}", consensus.consensus_key_kind);
                        continue;
                    };

                    let registration = build_dkg_authority_registration(
                        hotkey.clone(),
                        consensus,
                        dkg_x25519_public_key,
                        hotkey_signature,
                        consensus_signature,
                    );
                    if let Some(xt) = build_unsigned_authority_registration_extrinsic(registration) {
                        extrinsics.push(xt);
                    }
                }
            }

            if extrinsics.is_empty() {
                log::warn!(
                    target: "mev-shield-ibe",
                    "no valid DKG authority-registration extrinsics could be constructed",
                );
                return;
            }

            // Resubmit at a low cadence until the chain includes the registration.
            // Unsigned validation tags make duplicate submissions harmless.
            let mut ticker = futures_timer::Delay::new(Duration::from_secs(3)).fuse();
            loop {
                let at_hash = client.info().best_hash;
                for xt in &extrinsics {
                    let _ = transaction_pool
                        .submit_one(at_hash, TransactionSource::Local, xt.clone())
                        .await;
                }
                ticker.await;
                ticker = futures_timer::Delay::new(Duration::from_secs(30)).fuse();
            }
        }),
    );
}
