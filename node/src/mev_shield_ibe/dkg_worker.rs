use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use codec::Encode;
use futures::{FutureExt, StreamExt, channel::mpsc};
use rand_core::OsRng;
use sc_client_api::HeaderBackend;
use sc_service::SpawnTaskHandle;
use sp_api::ProvideRuntimeApi;
use sp_core::H256;
use sp_runtime::traits::Block as BlockT;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

use mev_shield_ibe_runtime_api::{
    DkgAuthorityInfo, DkgConsensusKeyKind, EpochDkgPublication, MevShieldDkgApi,
};

use super::{
    dkg::{DkgKeySource, DkgOutputSink, ProductionDkgKeySource},
    dkg_protocol::{
        DkgAcceptanceVoteV1, DkgDealerCommitmentV1, DkgOutputAttestationV1, DkgRoundAccumulator,
        DkgRoundId, LocalDkgKeys, VerifiedDealerShare, acceptance_vote_payload_hash,
        build_dealer_commitment, dealer_commitment_payload_hash, decrypt_share,
        epoch_publication_payload_hash, finalize_local_output, output_attestation_payload_hash,
        plan_from_runtime_authorities, verify_plain_share, verify_sr25519_authority_signature,
    },
    network::WireMessage,
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct LocalConsensusAuthority {
    pub consensus_key_kind: DkgConsensusKeyKind,
    pub authority_id: Vec<u8>,
    pub signature_key_hint: Vec<u8>,
}

#[derive(Clone)]
pub struct DkgWorkerConfig {
    pub poll_interval: Duration,
    pub max_atoms: u32,
    /// All local authoring keys discovered from the keystore.  During POA this
    /// normally contains Aura.  During/after the POS transition it should also
    /// contain BABE.  The runtime plan selects which key kind is live for an
    /// epoch, so the worker has no manual switch.
    pub local_authorities: Vec<LocalConsensusAuthority>,
    pub x25519_static_secret_bytes: [u8; 32],
}

pub trait AuthoritySigner: Send + Sync + 'static {
    fn sign(
        &self,
        key_kind: DkgConsensusKeyKind,
        key_hint: &[u8],
        payload_hash: H256,
    ) -> Result<Vec<u8>, String>;
}

struct WorkerState {
    rounds: BTreeMap<DkgRoundId, DkgRoundAccumulator>,
    committed: BTreeSet<DkgRoundId>,
    finalized: BTreeSet<DkgRoundId>,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            rounds: BTreeMap::new(),
            committed: BTreeSet::new(),
            finalized: BTreeSet::new(),
        }
    }
}

pub fn spawn_epoch_ahead_dkg_worker<Block, Client>(
    spawn_handle: &SpawnTaskHandle,
    client: Arc<Client>,
    dkg_source: Arc<ProductionDkgKeySource>,
    cfg: DkgWorkerConfig,
    signer: Arc<dyn AuthoritySigner>,
    mut inbound: mpsc::UnboundedReceiver<WireMessage>,
    outbound: mpsc::UnboundedSender<WireMessage>,
    epoch_publication_tx: Option<mpsc::UnboundedSender<EpochDkgPublication>>,
) where
    Block: BlockT,
    Client: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    Client::Api: MevShieldDkgApi<Block>,
{
    spawn_handle.spawn(
        "mev-shield-ibe-epoch-ahead-dkg",
        None,
        Box::pin(async move {
            let x25519_secret = StaticSecret::from(cfg.x25519_static_secret_bytes);
            let x25519_public: [u8; 32] = X25519PublicKey::from(&x25519_secret).to_bytes();
            let first_local_authority = cfg.local_authorities.first().map(|a| a.authority_id.clone()).unwrap_or_default();
            let local_keys = LocalDkgKeys {
                authority_id: first_local_authority,
                authority_ids: cfg.local_authorities.iter().map(|a| a.authority_id.clone()).collect(),
                x25519_secret,
                x25519_public,
            };
            let mut state = WorkerState::default();
            let mut interval = futures_timer::Delay::new(cfg.poll_interval).fuse();

            loop {
                futures::select! {
                    msg = inbound.next().fuse() => {
                        let Some(msg) = msg else { break; };
                        import_dkg_wire_message(&mut state, &local_keys, msg);
                    }
                    _ = interval => {
                        interval = futures_timer::Delay::new(cfg.poll_interval).fuse();
                        if let Err(err) = tick::<Block, Client>(&client, &dkg_source, &cfg, &local_keys, signer.as_ref(), &outbound, epoch_publication_tx.as_ref(), &mut state).await {
                            log::warn!(target: "mev-shield-ibe", "epoch-ahead DKG tick failed: {err}");
                        }
                    }
                }
            }
        }),
    );
}

fn import_dkg_wire_message(state: &mut WorkerState, local_keys: &LocalDkgKeys, msg: WireMessage) {
    match msg {
        WireMessage::DkgDealerCommitmentV1(commitment) => {
            let round = commitment.round.clone();
            let acc = state.rounds.entry(round).or_default();
            acc.commitments
                .insert(commitment.dealer_authority_id.clone(), commitment.clone());
            for encrypted in &commitment.encrypted_shares {
                if !local_keys
                    .authority_ids
                    .iter()
                    .any(|id| *id == encrypted.recipient_authority_id)
                {
                    continue;
                }
                // The dealer's X25519 public key is included in the runtime plan; the service checks
                // that `recipient_x25519_public_key` matches the plan before accepting the message.
                // Here we use the recipient field only to reject messages for other validators.
                if encrypted.sender_authority_id != commitment.dealer_authority_id {
                    continue;
                }
                if let Ok(plain) =
                    decrypt_share(&local_keys.x25519_secret, &commitment.round, encrypted)
                {
                    if verify_plain_share(&commitment, &plain).is_ok() {
                        if let Ok(scalar) =
                            super::dkg_protocol::scalar_from_bytes_for_worker(&plain.secret_scalar)
                        {
                            acc.local_verified_shares.insert(
                                (plain.dealer_authority_id.clone(), plain.share_id),
                                VerifiedDealerShare {
                                    dealer_authority_id: plain.dealer_authority_id,
                                    share_id: plain.share_id,
                                    scalar,
                                },
                            );
                        }
                    }
                }
            }
        }
        WireMessage::DkgAcceptanceVoteV1(vote) => {
            let acc = state.rounds.entry(vote.round.clone()).or_default();
            acc.accepted_votes.insert(
                (
                    vote.voter_authority_id.clone(),
                    vote.accepted_dealer_authority_id.clone(),
                ),
                vote,
            );
        }
        WireMessage::DkgOutputAttestationV1(att) => {
            let acc = state.rounds.entry(att.round.clone()).or_default();
            acc.output_attestations
                .insert(att.authority_id.clone(), att);
        }
        WireMessage::PartialDecryptionKeyShareV1(_) => {}
    }
}

fn select_local_authority<'a>(
    cfg: &'a DkgWorkerConfig,
    plan: &'a mev_shield_ibe_runtime_api::EpochDkgPlan,
) -> Option<(&'a LocalConsensusAuthority, &'a DkgAuthorityInfo)> {
    for local in &cfg.local_authorities {
        if let Some(info) = plan.authorities.iter().find(|a| {
            a.consensus_key_kind == local.consensus_key_kind && a.authority_id == local.authority_id
        }) {
            return Some((local, info));
        }
    }
    None
}
async fn tick<Block, Client>(
    client: &Arc<Client>,
    dkg_source: &Arc<ProductionDkgKeySource>,
    cfg: &DkgWorkerConfig,
    local_keys: &LocalDkgKeys,
    signer: &dyn AuthoritySigner,
    outbound: &mpsc::UnboundedSender<WireMessage>,
    epoch_publication_tx: Option<&mpsc::UnboundedSender<EpochDkgPublication>>,
    state: &mut WorkerState,
) -> Result<(), String>
where
    Block: BlockT,
    Client: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    Client::Api: MevShieldDkgApi<Block>,
{
    let best_hash = client.info().best_hash;
    let api = client.runtime_api();
    let Some(plan) = api
        .next_epoch_dkg_plan(best_hash)
        .map_err(|e| format!("runtime DKG plan call failed: {e:?}"))?
    else {
        return Ok(());
    };

    if cfg.max_atoms > 0 && plan.max_atoms > cfg.max_atoms {
        return Err(format!(
            "runtime DKG plan atom budget {} exceeds configured node max_atoms {}",
            plan.max_atoms, cfg.max_atoms
        ));
    }

    let Some((local_authority, local_info)) = select_local_authority(cfg, &plan) else {
        return Ok(());
    };
    let local_stake = local_info.stake;

    let authorities = plan
        .authorities
        .iter()
        .map(|a| (a.authority_id.clone(), a.stake, a.dkg_x25519_public_key))
        .collect::<Vec<_>>();
    let atom_plan = plan_from_runtime_authorities(authorities, plan.max_atoms)?;
    let plan_hash = H256::from(sp_core::blake2_256(&plan.encode()));
    let key_id = super::dkg_protocol::derive_key_id(
        dkg_source.genesis_hash(),
        plan.epoch,
        plan.first_block,
        plan.last_block,
        plan_hash,
    );
    let round = DkgRoundId {
        epoch: plan.epoch,
        key_id,
        first_block: plan.first_block,
        last_block: plan.last_block,
        genesis_hash: dkg_source.genesis_hash(),
    };

    let active_local_keys = LocalDkgKeys {
        authority_id: local_authority.authority_id.clone(),
        authority_ids: local_keys.authority_ids.clone(),
        x25519_secret: local_keys.x25519_secret.clone(),
        x25519_public: local_keys.x25519_public,
    };

    if !state.committed.contains(&round) {
        let mut commitment = build_dealer_commitment(
            &mut OsRng,
            round.clone(),
            &active_local_keys,
            local_stake,
            &atom_plan,
            Vec::new(),
        )?;
        let commitment_hash = dealer_commitment_payload_hash(&commitment);
        commitment.authority_signature = signer.sign(
            local_authority.consensus_key_kind,
            &local_authority.signature_key_hint,
            commitment_hash,
        )?;
        outbound
            .unbounded_send(WireMessage::DkgDealerCommitmentV1(commitment.clone()))
            .map_err(|e| format!("broadcast DKG commitment: {e}"))?;
        {
            let acc = state.rounds.entry(round.clone()).or_default();
            acc.commitments
                .insert(local_authority.authority_id.clone(), commitment.clone());
            // The node may not receive its own gossip notification, so decrypt
            // and verify its locally addressed shares immediately.
            for encrypted in &commitment.encrypted_shares {
                if !local_keys
                    .authority_ids
                    .iter()
                    .any(|id| *id == encrypted.recipient_authority_id)
                {
                    continue;
                }
                if let Ok(plain) =
                    decrypt_share(&local_keys.x25519_secret, &commitment.round, encrypted)
                {
                    if verify_plain_share(&commitment, &plain).is_ok() {
                        if let Ok(scalar) =
                            super::dkg_protocol::scalar_from_bytes_for_worker(&plain.secret_scalar)
                        {
                            acc.local_verified_shares.insert(
                                (plain.dealer_authority_id.clone(), plain.share_id),
                                VerifiedDealerShare {
                                    dealer_authority_id: plain.dealer_authority_id,
                                    share_id: plain.share_id,
                                    scalar,
                                },
                            );
                        }
                    }
                }
            }
        }
        state.committed.insert(round.clone());
    }

    let acc = state.rounds.entry(round.clone()).or_default();
    for (dealer_id, commitment) in acc.commitments.clone() {
        let vote_key = (local_authority.authority_id.clone(), dealer_id.clone());
        if acc.accepted_votes.contains_key(&vote_key) {
            continue;
        }
        // A dealer is accepted by this validator only after all local shares addressed to it verify.
        let has_local_share = acc
            .local_verified_shares
            .keys()
            .any(|(d, _)| d == &dealer_id);
        if !has_local_share && dealer_id != local_authority.authority_id {
            continue;
        }
        let mut vote = DkgAcceptanceVoteV1 {
            version: stp_mev_shield_ibe::MEV_SHIELD_IBE_VERSION,
            round: round.clone(),
            voter_authority_id: local_authority.authority_id.clone(),
            accepted_dealer_authority_id: dealer_id.clone(),
            vote_hash: dealer_commitment_payload_hash(&commitment),
            authority_signature: Vec::new(),
        };
        let sig = signer.sign(
            local_authority.consensus_key_kind,
            &local_authority.signature_key_hint,
            acceptance_vote_payload_hash(&vote),
        )?;
        vote.authority_signature = sig;
        outbound
            .unbounded_send(WireMessage::DkgAcceptanceVoteV1(vote.clone()))
            .map_err(|e| format!("broadcast DKG vote: {e}"))?;
        acc.accepted_votes.insert(vote_key, vote);
    }

    if state.finalized.contains(&round) {
        return Ok(());
    }

    let accepted_dealers = select_threshold_accepted_dealers(&plan.authorities, acc)?;
    if accepted_dealers.is_empty() {
        return Ok(());
    }

    let local_verified = acc
        .local_verified_shares
        .values()
        .cloned()
        .collect::<Vec<_>>();
    let bundle = finalize_local_output(
        &round,
        &atom_plan,
        &local_authority.authority_id,
        &accepted_dealers,
        &local_verified,
    )?;
    let master_public_key = bundle
        .public
        .epoch_key
        .master_public_key
        .clone()
        .into_inner();
    let publication_hash = epoch_publication_payload_hash(
        plan.epoch,
        key_id,
        plan.first_block,
        plan.last_block,
        plan.consensus_source,
        &master_public_key,
        bundle.public.epoch_key.total_weight,
        bundle.public.epoch_key.threshold_weight,
    );
    dkg_source.upsert_public_output(bundle.public.clone())?;
    dkg_source.upsert_secret_bundle(bundle.clone())?;

    let mut att = DkgOutputAttestationV1 {
        version: stp_mev_shield_ibe::MEV_SHIELD_IBE_VERSION,
        round: round.clone(),
        authority_id: local_authority.authority_id.clone(),
        stake: local_stake,
        public_output_hash: publication_hash,
        authority_signature: Vec::new(),
    };
    att.authority_signature = signer.sign(
        local_authority.consensus_key_kind,
        &local_authority.signature_key_hint,
        output_attestation_payload_hash(&att),
    )?;
    outbound
        .unbounded_send(WireMessage::DkgOutputAttestationV1(att.clone()))
        .map_err(|e| format!("broadcast DKG attestation: {e}"))?;
    acc.output_attestations
        .insert(local_authority.authority_id.clone(), att.clone());

    if let Some(tx) = epoch_publication_tx {
        let publication = EpochDkgPublication {
            epoch: plan.epoch,
            key_id,
            first_block: plan.first_block,
            last_block: plan.last_block,
            consensus_source: plan.consensus_source,
            master_public_key,
            total_weight: bundle.public.epoch_key.total_weight,
            threshold_weight: bundle.public.epoch_key.threshold_weight,
            public_output_hash: publication_hash,
            attestations: acc
                .output_attestations
                .values()
                .map(|a| mev_shield_ibe_runtime_api::DkgOutputAttestation {
                    authority_id: a.authority_id.clone(),
                    stake: a.stake,
                    public_output_hash: a.public_output_hash,
                    signature: a.authority_signature.clone(),
                })
                .collect(),
        };
        let _ = tx.unbounded_send(publication);
    }

    state.finalized.insert(round);
    Ok(())
}

fn select_threshold_accepted_dealers(
    authorities: &[DkgAuthorityInfo],
    acc: &DkgRoundAccumulator,
) -> Result<Vec<DkgDealerCommitmentV1>, String> {
    let by_stake: BTreeMap<Vec<u8>, u128> = authorities
        .iter()
        .map(|a| (a.authority_id.clone(), a.stake))
        .collect();
    let total_stake = authorities
        .iter()
        .fold(0u128, |s, a| s.saturating_add(a.stake));
    let threshold = total_stake.saturating_mul(2) / 3 + 1;

    let mut valid_commitments = BTreeMap::<Vec<u8>, DkgDealerCommitmentV1>::new();
    let mut commitment_hashes = BTreeMap::<Vec<u8>, H256>::new();
    for (dealer_id, commitment) in acc.commitments.iter() {
        let Some(expected_stake) = by_stake.get(dealer_id).copied() else {
            continue;
        };
        if commitment.dealer_stake != expected_stake
            || commitment.version != stp_mev_shield_ibe::MEV_SHIELD_IBE_VERSION
        {
            continue;
        }
        let hash = dealer_commitment_payload_hash(commitment);
        if verify_sr25519_authority_signature(dealer_id, hash, &commitment.authority_signature) {
            valid_commitments.insert(dealer_id.clone(), commitment.clone());
            commitment_hashes.insert(dealer_id.clone(), hash);
        }
    }

    let mut valid_votes_for_dealer = BTreeMap::<Vec<u8>, BTreeSet<Vec<u8>>>::new();
    for ((voter_id, dealer_id), vote) in acc.accepted_votes.iter() {
        if !by_stake.contains_key(voter_id) || !valid_commitments.contains_key(dealer_id) {
            continue;
        }
        let Some(expected_hash) = commitment_hashes.get(dealer_id) else {
            continue;
        };
        if vote.vote_hash != *expected_hash
            || vote.version != stp_mev_shield_ibe::MEV_SHIELD_IBE_VERSION
        {
            continue;
        }
        if verify_sr25519_authority_signature(
            voter_id,
            acceptance_vote_payload_hash(vote),
            &vote.authority_signature,
        ) {
            valid_votes_for_dealer
                .entry(dealer_id.clone())
                .or_default()
                .insert(voter_id.clone());
        }
    }

    let mut accepted = Vec::new();
    let mut accepted_dealer_weight = 0u128;
    for (dealer_id, commitment) in valid_commitments.iter() {
        let vote_weight = valid_votes_for_dealer
            .get(dealer_id)
            .into_iter()
            .flat_map(|voters| voters.iter())
            .filter_map(|voter| by_stake.get(voter))
            .fold(0u128, |s, w| s.saturating_add(*w));
        if vote_weight >= threshold {
            accepted_dealer_weight =
                accepted_dealer_weight.saturating_add(*by_stake.get(dealer_id).unwrap_or(&0));
            accepted.push(commitment.clone());
        }
        if accepted_dealer_weight >= threshold {
            return Ok(accepted);
        }
    }
    Ok(Vec::new())
}
