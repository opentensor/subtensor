//! Local unsigned publication of an authority's durable X25519 DKG transport key.
//!
//! This is kept for backward compatibility with the current POA branch.  POS
//! correctness comes from the full authority registration path in the pallet: the
//! consensus key kind is included here and the runtime provider joins active
//! root-validator stake to registered consensus keys.

use std::{sync::Arc, time::Duration};

use codec::Encode;
use futures::FutureExt;
use mev_shield_ibe_runtime_api::{DkgConsensusKeyKind, DkgTransportKeyRegistration};
use node_subtensor_runtime::{
    RuntimeCall, UncheckedExtrinsic,
    opaque::{Block, UncheckedExtrinsic as OpaqueExtrinsic},
};
use sc_client_api::HeaderBackend;
use sc_service::SpawnTaskHandle;
use sc_transaction_pool::TransactionPoolHandle;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_core::H256;
use sp_runtime::OpaqueExtrinsic as SpOpaqueExtrinsic;

use crate::client::FullClient;

use super::dkg_worker::AuthoritySigner;

pub fn dkg_transport_key_payload_hash(
    authority_id: &[u8],
    dkg_x25519_public_key: &[u8; 32],
) -> H256 {
    H256::from(sp_core::blake2_256(
        &(
            b"bittensor.mev-shield.v2.dkg.transport-key",
            authority_id,
            dkg_x25519_public_key,
        )
            .encode(),
    ))
}

fn build_unsigned_transport_key_extrinsic(
    registration: DkgTransportKeyRegistration,
) -> Option<OpaqueExtrinsic> {
    let call =
        RuntimeCall::MevShield(pallet_shield::Call::submit_ibe_dkg_transport_key { registration });
    let unchecked = UncheckedExtrinsic::new_bare(call);
    SpOpaqueExtrinsic::from_bytes(&unchecked.encode()).ok()
}

pub fn spawn_dkg_transport_key_submitter(
    spawn: &SpawnTaskHandle,
    client: Arc<FullClient>,
    transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>,
    signer: Arc<dyn AuthoritySigner>,
    authority_id: Vec<u8>,
    consensus_key_kind: DkgConsensusKeyKind,
    authority_signature_key_hint: Vec<u8>,
    dkg_x25519_public_key: [u8; 32],
) {
    spawn.spawn(
        "mev-shield-ibe-dkg-transport-key-submitter",
        None,
        Box::pin(async move {
            // Submit repeatedly at a low cadence until included.  The unsigned
            // validation tag makes repeats harmless.
            let payload = dkg_transport_key_payload_hash(&authority_id, &dkg_x25519_public_key);
            let Ok(signature) = signer.sign(consensus_key_kind, &authority_signature_key_hint, payload) else {
                log::warn!(target: "mev-shield-ibe", "unable to sign DKG transport-key registration");
                return;
            };
            let registration = DkgTransportKeyRegistration {
                authority_id,
                dkg_x25519_public_key,
                signature,
            };
            let Some(xt) = build_unsigned_transport_key_extrinsic(registration) else {
                return;
            };
            let mut ticker = futures_timer::Delay::new(Duration::from_secs(3)).fuse();
            loop {
                let at_hash = client.info().best_hash;
                let _ = transaction_pool
                    .submit_one(at_hash, TransactionSource::Local, xt.clone())
                    .await;
                ticker.await;
                ticker = futures_timer::Delay::new(Duration::from_secs(30)).fuse();
            }
        }),
    );
}
