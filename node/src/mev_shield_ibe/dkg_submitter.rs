//! Local unsigned submission of finalized epoch DKG public outputs.

use std::{collections::VecDeque, sync::Arc, time::Duration};

use codec::Encode;
use node_subtensor_runtime::{
    RuntimeCall, UncheckedExtrinsic,
    opaque::{Block, UncheckedExtrinsic as OpaqueExtrinsic},
};
use sc_client_api::HeaderBackend;
use sc_service::SpawnTaskHandle;
use sc_transaction_pool::TransactionPoolHandle;
use sc_transaction_pool_api::{TransactionPool, TransactionSource};
use sp_runtime::OpaqueExtrinsic as SpOpaqueExtrinsic;

use super::dkg_worker::{AuthoritySigner, LocalConsensusAuthority};
use crate::client::FullClient;
use mev_shield_ibe_runtime_api::{DkgConsensusKeyKind, EpochDkgPublication};

fn build_unsigned_publish_epoch_key_extrinsic(
    publication: EpochDkgPublication,
) -> Option<OpaqueExtrinsic> {
    let call =
        RuntimeCall::MevShield(pallet_shield::Call::publish_ibe_epoch_public_key { publication });
    let unchecked = UncheckedExtrinsic::new_bare(call);
    SpOpaqueExtrinsic::from_bytes(&unchecked.encode()).ok()
}

fn publication_attestation_weight(publication: &EpochDkgPublication) -> u128 {
    publication
        .attestations
        .iter()
        .fold(0u128, |acc, att| acc.saturating_add(att.stake))
}

fn queue_epoch_publication(
    pending_publications: &mut VecDeque<EpochDkgPublication>,
    publication: EpochDkgPublication,
) {
    let new_weight = publication_attestation_weight(&publication);
    let mut already_have_at_least_as_good = false;
    pending_publications.retain(|queued| {
        let same_key = queued.epoch == publication.epoch && queued.key_id == publication.key_id;
        if !same_key {
            return true;
        }
        let queued_weight = publication_attestation_weight(queued);
        if queued_weight >= new_weight {
            already_have_at_least_as_good = true;
            true
        } else {
            false
        }
    });
    if !already_have_at_least_as_good {
        pending_publications.push_back(publication);
    }
}

fn dkg_authority_registration_payload_hash(
    hotkey: &subtensor_runtime_common::AccountId,
    consensus_key_kind: DkgConsensusKeyKind,
    authority_id: &[u8],
    dkg_x25519_public_key: &[u8; 32],
) -> sp_core::H256 {
    sp_core::H256::from(sp_core::hashing::blake2_256(
        &(
            b"bittensor.mev-shield.v2.dkg.authority-registration",
            hotkey,
            consensus_key_kind,
            authority_id,
            dkg_x25519_public_key,
        )
            .encode(),
    ))
}

fn build_unsigned_register_ibe_dkg_authority_key_extrinsic(
    hotkey: subtensor_runtime_common::AccountId,
    consensus_key_kind: DkgConsensusKeyKind,
    authority_id: Vec<u8>,
    dkg_x25519_public_key: [u8; 32],
    proof_signature: Vec<u8>,
) -> Option<OpaqueExtrinsic> {
    let call = RuntimeCall::MevShield(
        pallet_shield::Call::register_ibe_dkg_authority_key_unsigned {
            hotkey,
            consensus_key_kind,
            authority_id,
            dkg_x25519_public_key,
            proof_signature,
        },
    );
    let unchecked = UncheckedExtrinsic::new_bare(call);
    SpOpaqueExtrinsic::from_bytes(&unchecked.encode()).ok()
}

pub fn spawn_dkg_authority_registration_submitter(
    spawn: &SpawnTaskHandle,
    client: Arc<FullClient>,
    transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>,
    local_authorities: Vec<LocalConsensusAuthority>,
    x25519_static_secret_bytes: [u8; 32],
    signer: Arc<dyn AuthoritySigner>,
) {
    if local_authorities.is_empty() {
        return;
    }

    let x25519_public = x25519_dalek::PublicKey::from(&x25519_dalek::StaticSecret::from(
        x25519_static_secret_bytes,
    ))
    .to_bytes();

    spawn.spawn(
        "mev-shield-ibe-dkg-authority-registration-submitter",
        None,
        Box::pin(async move {
            loop {
                let at_hash = client.info().best_hash;
                for authority in &local_authorities {
                    if authority.authority_id.len() != 32 {
                        log::debug!(
                            target: "mev-shield-ibe",
                            "skipping DKG authority registration for non-32-byte authority id"
                        );
                        continue;
                    }

                    // Devnet and the PoA->PoS handoff use the same validator
                    // account bytes as the initial consensus authority id.  If a
                    // production deployment uses distinct hotkeys, the existing
                    // signed pallet call remains available for explicit/manual
                    // registration of that mapping.
                    let mut hotkey_bytes = [0u8; 32];
                    hotkey_bytes.copy_from_slice(&authority.authority_id[..32]);
                    let hotkey: subtensor_runtime_common::AccountId =
                        sp_runtime::AccountId32::new(hotkey_bytes);

                    let payload_hash = dkg_authority_registration_payload_hash(
                        &hotkey,
                        authority.consensus_key_kind,
                        &authority.authority_id,
                        &x25519_public,
                    );
                    let proof_signature = match signer.sign(
                        authority.consensus_key_kind,
                        &authority.signature_key_hint,
                        payload_hash,
                    ) {
                        Ok(signature) => signature,
                        Err(err) => {
                            log::debug!(
                                target: "mev-shield-ibe",
                                "could not sign DKG authority registration proof: {err}"
                            );
                            continue;
                        }
                    };

                    let Some(xt) = build_unsigned_register_ibe_dkg_authority_key_extrinsic(
                        hotkey,
                        authority.consensus_key_kind,
                        authority.authority_id.clone(),
                        x25519_public,
                        proof_signature,
                    ) else {
                        continue;
                    };

                    if let Err(err) = transaction_pool
                        .submit_one(at_hash, TransactionSource::Local, xt)
                        .await
                    {
                        log::debug!(
                            target: "mev-shield-ibe",
                            "failed to submit local DKG authority registration; retrying: {err:?}",
                        );
                    }
                }

                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        }),
    );
}

pub fn spawn_dkg_publication_submitter(
    spawn: &SpawnTaskHandle,
    client: Arc<FullClient>,
    transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>,
    mut rx: futures::channel::mpsc::UnboundedReceiver<EpochDkgPublication>,
) {
    use futures::StreamExt;

    spawn.spawn(
        "mev-shield-ibe-dkg-publication-submitter",
        None,
        Box::pin(async move {
            let mut pending_publications: VecDeque<EpochDkgPublication> = VecDeque::new();
		loop {
			tokio::select! {
				publication = rx.next() => {
					let Some(publication) = publication else { break; };
					queue_epoch_publication(&mut pending_publications, publication);
				},
				_ = tokio::time::sleep(Duration::from_millis(250)) => {},
			}

			let attempts = pending_publications.len();
			for _ in 0..attempts {
				let Some(publication) = pending_publications.pop_front() else { break; };
				let Some(xt) = build_unsigned_publish_epoch_key_extrinsic(publication.clone()) else {
					pending_publications.push_back(publication);
					continue;
				};
				let at_hash = client.info().best_hash;
				if let Err(err) = transaction_pool
					.submit_one(at_hash, TransactionSource::Local, xt)
					.await
				{
					log::debug!(
						target: "mev-shield-ibe",
						"failed to submit local epoch DKG public-key publication; retrying: {err:?}",
					);
					pending_publications.push_back(publication);
				}
			}
		}
        }),
    );
}
