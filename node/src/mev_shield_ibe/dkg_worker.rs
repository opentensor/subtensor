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
    DkgAuthorityInfo, DkgConsensusKeyKind, DkgTransportKeyRegistration, EpochDkgPublication,
    MevShieldDkgApi,
};

use super::{
    dkg::{DkgKeySource, DkgOutputSink, ProductionDkgKeySource},
    dkg_protocol::{
        DkgAcceptanceVoteV1, DkgDealerCommitmentV1, DkgOutputAttestationV1, DkgRoundAccumulator,
        DkgRoundId, LocalDkgKeys, VerifiedDealerShare, acceptance_vote_payload_hash,
        build_dealer_commitment, dealer_commitment_payload_hash, decrypt_share,
        dkg_transport_key_payload_hash, epoch_publication_payload_hash, finalize_local_output,
        output_attestation_payload_hash, plan_from_runtime_authorities, verify_plain_share,
        verify_sr25519_authority_signature,
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
    transport_keys: BTreeMap<(DkgRoundId, Vec<u8>), [u8; 32]>,
    pending_transport_keys: BTreeMap<Vec<u8>, DkgTransportKeyRegistration>,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            rounds: BTreeMap::new(),
            committed: BTreeSet::new(),
            finalized: BTreeSet::new(),
            transport_keys: BTreeMap::new(),
            pending_transport_keys: BTreeMap::new(),
        }
    }
}

pub fn spawn_epoch_ahead_dkg_worker<Block, Client>(
    spawn_handle: &SpawnTaskHandle,
    shutdown_registration: futures::future::AbortRegistration,
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
    let mev_shield_dkg_worker = async move {
        let x25519_secret = StaticSecret::from(cfg.x25519_static_secret_bytes);
        let x25519_public: [u8; 32] = X25519PublicKey::from(&x25519_secret).to_bytes();
        let first_local_authority = cfg
            .local_authorities
            .first()
            .map(|a| a.authority_id.clone())
            .unwrap_or_default();
        let local_keys = LocalDkgKeys {
            authority_id: first_local_authority,
            authority_ids: cfg
                .local_authorities
                .iter()
                .map(|a| a.authority_id.clone())
                .collect(),
            x25519_secret,
            x25519_public,
        };
        let mut state = WorkerState::default();
        let mut interval = futures_timer::Delay::new(cfg.poll_interval).fuse();

        let shutdown = tokio::signal::ctrl_c().fuse();
        futures::pin_mut!(shutdown);
        loop {
            futures::select! {
                _ = shutdown => {
                break;
            }
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
    };

    spawn_handle.spawn(
        "mev-shield-ibe-epoch-ahead-dkg",
        None,
        Box::pin(async move {
            let _ =
                futures::future::Abortable::new(mev_shield_dkg_worker, shutdown_registration).await;
        }),
    );
}

fn import_dkg_wire_message(state: &mut WorkerState, local_keys: &LocalDkgKeys, msg: WireMessage) {
    match msg {
        WireMessage::DkgTransportKeyV1(registration) => {
            state
                .pending_transport_keys
                .insert(registration.authority_id.clone(), registration);
        }
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

    state.transport_keys.insert(
        (round.clone(), local_authority.authority_id.clone()),
        local_keys.x25519_public,
    );

    let transport_hash = dkg_transport_key_payload_hash(
        &round,
        &local_authority.authority_id,
        &local_keys.x25519_public,
    );
    let transport_signature = signer.sign(
        local_authority.consensus_key_kind,
        &local_authority.signature_key_hint,
        transport_hash,
    )?;
    let _ = outbound.unbounded_send(WireMessage::DkgTransportKeyV1(
        DkgTransportKeyRegistration {
            authority_id: local_authority.authority_id.clone(),
            dkg_x25519_public_key: local_keys.x25519_public,
            signature: transport_signature,
        },
    ));

    for registration in state
        .pending_transport_keys
        .values()
        .cloned()
        .collect::<Vec<_>>()
    {
        let payload = dkg_transport_key_payload_hash(
            &round,
            &registration.authority_id,
            &registration.dkg_x25519_public_key,
        );
        if verify_sr25519_authority_signature(
            &registration.authority_id,
            payload,
            &registration.signature,
        ) {
            state.transport_keys.insert(
                (round.clone(), registration.authority_id),
                registration.dkg_x25519_public_key,
            );
        }
    }

    let mut effective_authorities = plan.authorities.clone();
    for authority in effective_authorities.iter_mut() {
        if let Some(public_key) = state
            .transport_keys
            .get(&(round.clone(), authority.authority_id.clone()))
        {
            authority.dkg_x25519_public_key = *public_key;
        }
    }

    if effective_authorities
        .iter()
        .any(|authority| authority.dkg_x25519_public_key == [0u8; 32])
    {
        return Ok(());
    }

    let authorities = effective_authorities
        .iter()
        .map(|a| (a.authority_id.clone(), a.stake, a.dkg_x25519_public_key))
        .collect::<Vec<_>>();
    let atom_plan = plan_from_runtime_authorities(authorities, plan.max_atoms)?;
    let local_share_ids = local_share_ids_for_authority(&atom_plan, &local_authority.authority_id);
    if local_share_ids.is_empty() {
        return Err("local DKG authority has no share atoms in runtime plan".into());
    }

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
        let dealer_hash = dealer_commitment_payload_hash(&commitment);
        if !verify_sr25519_authority_signature(
            &dealer_id,
            dealer_hash,
            &commitment.authority_signature,
        ) {
            continue;
        }

        if !all_local_atom_shares_verified_for_dealer(&local_share_ids, &dealer_id, &*acc) {
            continue;
        }
        let mut vote = DkgAcceptanceVoteV1 {
            version: stp_mev_shield_ibe::MEV_SHIELD_IBE_VERSION,
            round: round.clone(),
            voter_authority_id: local_authority.authority_id.clone(),
            accepted_dealer_authority_id: dealer_id.clone(),
            vote_hash: dealer_hash,
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

fn local_share_ids_for_authority(
    atom_plan: &super::dkg_weighting::DkgAtomPlan<Vec<u8>>,
    authority_id: &[u8],
) -> BTreeSet<u32> {
    atom_plan
        .atoms
        .iter()
        .filter(|atom| atom.authority_id.as_slice() == authority_id)
        .map(|atom| atom.share_id)
        .collect()
}

fn all_local_atom_shares_verified_for_dealer(
    local_share_ids: &BTreeSet<u32>,
    dealer_id: &[u8],
    acc: &DkgRoundAccumulator,
) -> bool {
    !local_share_ids.is_empty()
        && local_share_ids.iter().all(|share_id| {
            acc.local_verified_shares
                .contains_key(&(dealer_id.to_vec(), *share_id))
        })
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

#[cfg(test)]
mod mev_shield_dkg_worker_unit_tests {
    use super::*;
    use crate::mev_shield_ibe::crypto::Scalar;
    use crate::mev_shield_ibe::dkg_protocol::VerifiedDealerShare;
    use crate::mev_shield_ibe::dkg_weighting::{DkgAtomPlan, DkgShareAtom};

    fn atom(authority_id: Vec<u8>, share_id: u32) -> DkgShareAtom<Vec<u8>> {
        DkgShareAtom {
            authority_id,
            dkg_x25519_public_key: [share_id as u8; 32],
            share_id,
            weight: 1,
        }
    }

    fn verified(dealer: &[u8], share_id: u32) -> VerifiedDealerShare {
        VerifiedDealerShare {
            dealer_authority_id: dealer.to_vec(),
            share_id,
            scalar: Scalar::from(share_id as u64),
        }
    }

    #[test]
    fn local_share_ids_select_only_this_authority_atoms() {
        let local = b"local".to_vec();
        let other = b"other".to_vec();
        let plan = DkgAtomPlan {
            atoms: vec![
                atom(local.clone(), 1),
                atom(other, 2),
                atom(local.clone(), 3),
            ],
            total_weight: 3,
            threshold_weight: 3,
        };

        let ids = local_share_ids_for_authority(&plan, &local);
        assert_eq!(ids.iter().copied().collect::<Vec<_>>(), vec![1, 3]);
    }

    #[test]
    fn dealer_vote_requires_every_local_atom_share() {
        let dealer = b"dealer".to_vec();
        let local_ids = [1_u32, 3_u32].into_iter().collect::<BTreeSet<_>>();
        let mut acc = DkgRoundAccumulator::default();

        assert!(!all_local_atom_shares_verified_for_dealer(
            &local_ids, &dealer, &acc
        ));

        acc.local_verified_shares
            .insert((dealer.clone(), 1), verified(&dealer, 1));
        assert!(!all_local_atom_shares_verified_for_dealer(
            &local_ids, &dealer, &acc
        ));

        acc.local_verified_shares
            .insert((dealer.clone(), 3), verified(&dealer, 3));
        assert!(all_local_atom_shares_verified_for_dealer(
            &local_ids, &dealer, &acc
        ));
    }

    #[test]
    fn dealer_vote_rejects_empty_local_atom_assignment() {
        let acc = DkgRoundAccumulator::default();
        assert!(!all_local_atom_shares_verified_for_dealer(
            &BTreeSet::new(),
            b"dealer",
            &acc
        ));
    }

    #[test]
    fn dealer_vote_does_not_count_other_dealers_shares() {
        let dealer = b"dealer-a".to_vec();
        let other = b"dealer-b".to_vec();
        let local_ids = [1_u32].into_iter().collect::<BTreeSet<_>>();
        let mut acc = DkgRoundAccumulator::default();
        acc.local_verified_shares
            .insert((other.clone(), 1), verified(&other, 1));

        assert!(!all_local_atom_shares_verified_for_dealer(
            &local_ids, &dealer, &acc
        ));
    }
}

#[cfg(test)]
mod comprehensive_green_path_tests {
    use super::*;
    use crate::mev_shield_ibe::crypto::{
        combine_identity_key, derive_partial_identity_key, verify_partial_identity_key,
    };
    use crate::mev_shield_ibe::dkg_weighting::two_thirds_plus_one;
    use mev_shield_ibe_runtime_api::{DkgConsensusSource, DkgOutputAttestation};
    use sp_core::{Pair, crypto::ByteArray, sr25519};
    use stp_mev_shield_ibe::{KEY_ID_LEN, MEV_SHIELD_IBE_VERSION};
    use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

    struct TestAuthority {
        pair: sr25519::Pair,
        authority_id: Vec<u8>,
        stake: u128,
        x25519_seed: [u8; 32],
        x25519_public: [u8; 32],
    }

    impl TestAuthority {
        fn new(seed: u8, stake: u128) -> Self {
            let pair = sr25519::Pair::from_seed(&[seed; 32]);
            let authority_id = pair.public().to_raw_vec();
            let x25519_seed = [seed.wrapping_add(100); 32];
            let x25519_public = X25519PublicKey::from(&StaticSecret::from(x25519_seed)).to_bytes();
            Self {
                pair,
                authority_id,
                stake,
                x25519_seed,
                x25519_public,
            }
        }

        fn local_keys(&self) -> LocalDkgKeys {
            LocalDkgKeys {
                authority_id: self.authority_id.clone(),
                authority_ids: vec![self.authority_id.clone()],
                x25519_secret: StaticSecret::from(self.x25519_seed),
                x25519_public: self.x25519_public,
            }
        }

        fn runtime_info(&self) -> DkgAuthorityInfo {
            DkgAuthorityInfo {
                hotkey_account_id: self.authority_id.clone(),
                consensus_key_kind: DkgConsensusKeyKind::BabeSr25519,
                authority_id: self.authority_id.clone(),
                stake: self.stake,
                dkg_x25519_public_key: self.x25519_public,
            }
        }

        fn sign_hash(&self, hash: H256) -> Vec<u8> {
            self.pair.sign(hash.as_fixed_bytes()).to_raw_vec()
        }
    }

    #[test]
    fn stake_weighted_dkg_green_path_reconstructs_finality_bound_block_key() {
        let authorities = vec![
            TestAuthority::new(11, 60),
            TestAuthority::new(22, 25),
            TestAuthority::new(33, 14),
            TestAuthority::new(44, 7),
            TestAuthority::new(55, 1),
        ];
        let runtime_authorities = authorities
            .iter()
            .map(TestAuthority::runtime_info)
            .collect::<Vec<_>>();
        let plan = mev_shield_ibe_runtime_api::EpochDkgPlan {
            epoch: 41,
            first_block: 4_000,
            last_block: 4_999,
            consensus_source: DkgConsensusSource::PosBabeRootValidators,
            authorities: runtime_authorities.clone(),
            max_atoms: 31,
        };
        let atom_plan = plan_from_runtime_authorities(
            runtime_authorities
                .iter()
                .map(|a| (a.authority_id.clone(), a.stake, a.dkg_x25519_public_key))
                .collect(),
            plan.max_atoms,
        )
        .expect("stake weighted atom plan builds");
        assert_eq!(atom_plan.total_weight, plan.max_atoms as u128);
        assert_eq!(
            atom_plan.threshold_weight,
            two_thirds_plus_one(atom_plan.total_weight).expect("threshold computes")
        );
        assert!(
            authorities.iter().all(|authority| atom_plan
                .atoms
                .iter()
                .any(|atom| atom.authority_id == authority.authority_id)),
            "every active validator receives at least one atom"
        );

        let local = &authorities[0];
        let local_share_ids = local_share_ids_for_authority(&atom_plan, &local.authority_id);
        assert!(local_share_ids.len() > 1);

        let round = DkgRoundId {
            epoch: plan.epoch,
            key_id: [77u8; KEY_ID_LEN],
            first_block: plan.first_block,
            last_block: plan.last_block,
            genesis_hash: H256::repeat_byte(9),
        };

        let mut commitments = BTreeMap::<Vec<u8>, DkgDealerCommitmentV1>::new();
        for authority in &authorities {
            let keys = authority.local_keys();
            let mut commitment = build_dealer_commitment(
                &mut OsRng,
                round.clone(),
                &keys,
                authority.stake,
                &atom_plan,
                Vec::new(),
            )
            .expect("dealer commitment builds");
            let hash = dealer_commitment_payload_hash(&commitment);
            commitment.authority_signature = authority.sign_hash(hash);
            assert!(verify_sr25519_authority_signature(
                &authority.authority_id,
                hash,
                &commitment.authority_signature
            ));
            assert_eq!(commitment.encrypted_shares.len(), atom_plan.atoms.len());
            commitments.insert(authority.authority_id.clone(), commitment);
        }

        let mut verified_by_recipient = BTreeMap::<Vec<u8>, Vec<VerifiedDealerShare>>::new();
        let mut worker_acc = DkgRoundAccumulator::default();
        worker_acc.commitments = commitments.clone();

        for commitment in commitments.values() {
            for authority in &authorities {
                let keys = authority.local_keys();
                let mut verified_share_ids = BTreeSet::new();
                for encrypted in commitment
                    .encrypted_shares
                    .iter()
                    .filter(|share| share.recipient_authority_id == authority.authority_id)
                {
                    let plain = decrypt_share(&keys.x25519_secret, &round, encrypted)
                        .expect("recipient decrypts its addressed DKG atom share");
                    verify_plain_share(commitment, &plain)
                        .expect("plain DKG atom share verifies against dealer commitments");
                    let scalar = crate::mev_shield_ibe::dkg_protocol::scalar_from_bytes_for_worker(
                        &plain.secret_scalar,
                    )
                    .expect("plain share scalar decodes");
                    let verified = VerifiedDealerShare {
                        dealer_authority_id: plain.dealer_authority_id.clone(),
                        share_id: plain.share_id,
                        scalar,
                    };
                    verified_share_ids.insert(plain.share_id);
                    verified_by_recipient
                        .entry(authority.authority_id.clone())
                        .or_default()
                        .push(verified.clone());
                    if authority.authority_id == local.authority_id {
                        worker_acc.local_verified_shares.insert(
                            (plain.dealer_authority_id.clone(), plain.share_id),
                            verified,
                        );
                    }
                }
                assert_eq!(
                    verified_share_ids,
                    local_share_ids_for_authority(&atom_plan, &authority.authority_id)
                );
            }
        }

        let first_dealer_id = authorities[0].authority_id.clone();
        let first_local_share = worker_acc
            .local_verified_shares
            .iter()
            .find(|((dealer_id, _), _)| *dealer_id == first_dealer_id)
            .map(|(key, share)| (key.clone(), share.clone()))
            .expect("first dealer has local shares");
        let mut partial_acc = DkgRoundAccumulator::default();
        partial_acc
            .local_verified_shares
            .insert(first_local_share.0, first_local_share.1);
        assert!(!all_local_atom_shares_verified_for_dealer(
            &local_share_ids,
            &first_dealer_id,
            &partial_acc
        ));
        assert!(all_local_atom_shares_verified_for_dealer(
            &local_share_ids,
            &first_dealer_id,
            &worker_acc
        ));

        for commitment in commitments.values() {
            let vote_hash = dealer_commitment_payload_hash(commitment);
            for voter in &authorities {
                let mut vote = DkgAcceptanceVoteV1 {
                    version: MEV_SHIELD_IBE_VERSION,
                    round: round.clone(),
                    voter_authority_id: voter.authority_id.clone(),
                    accepted_dealer_authority_id: commitment.dealer_authority_id.clone(),
                    vote_hash,
                    authority_signature: Vec::new(),
                };
                vote.authority_signature = voter.sign_hash(acceptance_vote_payload_hash(&vote));
                assert!(verify_sr25519_authority_signature(
                    &voter.authority_id,
                    acceptance_vote_payload_hash(&vote),
                    &vote.authority_signature
                ));
                worker_acc.accepted_votes.insert(
                    (
                        voter.authority_id.clone(),
                        commitment.dealer_authority_id.clone(),
                    ),
                    vote,
                );
            }
        }

        let accepted_dealers = select_threshold_accepted_dealers(&runtime_authorities, &worker_acc)
            .expect("threshold accepted dealers are selected");
        let by_stake = runtime_authorities
            .iter()
            .map(|a| (a.authority_id.clone(), a.stake))
            .collect::<BTreeMap<_, _>>();
        let total_stake = runtime_authorities
            .iter()
            .fold(0u128, |sum, authority| sum.saturating_add(authority.stake));
        let stake_threshold = total_stake.saturating_mul(2) / 3 + 1;
        let accepted_stake = accepted_dealers.iter().fold(0u128, |sum, dealer| {
            sum.saturating_add(*by_stake.get(&dealer.dealer_authority_id).unwrap())
        });
        assert!(accepted_stake >= stake_threshold);

        let mut bundles = Vec::new();
        for authority in &authorities {
            let verified = verified_by_recipient
                .get(&authority.authority_id)
                .expect("recipient has verified shares")
                .clone();
            let bundle = finalize_local_output(
                &round,
                &atom_plan,
                &authority.authority_id,
                &accepted_dealers,
                &verified,
            )
            .expect("local DKG output finalizes");
            let expected_local_ids =
                local_share_ids_for_authority(&atom_plan, &authority.authority_id);
            assert_eq!(bundle.validator_authority, authority.authority_id);
            assert_eq!(bundle.local_atoms.len(), expected_local_ids.len());
            assert!(
                bundle
                    .local_atoms
                    .iter()
                    .all(|atom| expected_local_ids.contains(&atom.public.share_id))
            );
            assert_eq!(bundle.public.epoch_key.epoch, round.epoch);
            assert_eq!(bundle.public.epoch_key.key_id, round.key_id);
            assert_eq!(bundle.public.epoch_key.first_block, round.first_block);
            assert_eq!(bundle.public.epoch_key.last_block, round.last_block);
            assert_eq!(bundle.public.epoch_key.total_weight, atom_plan.total_weight);
            assert_eq!(
                bundle.public.epoch_key.threshold_weight,
                atom_plan.threshold_weight
            );
            assert_eq!(bundle.public.public_atoms.len(), atom_plan.atoms.len());
            assert_eq!(bundle.public.total_public_weight(), atom_plan.total_weight);
            bundles.push(bundle);
        }
        for bundle in bundles.iter().skip(1) {
            assert_eq!(bundle.public.encode(), bundles[0].public.encode());
        }

        let target_block = round.first_block + 2;
        let finalized_number = target_block + 12;
        let finalized_hash = H256::repeat_byte(88);
        let mut partial_identity_shares = Vec::new();
        for bundle in &bundles {
            for atom in &bundle.local_atoms {
                let share = derive_partial_identity_key(
                    round.genesis_hash,
                    bundle,
                    target_block,
                    atom,
                    finalized_number,
                    finalized_hash,
                )
                .expect("partial identity key derives");
                let public_atom = bundle
                    .public
                    .public_atom(share.share_id)
                    .expect("public atom exists for partial identity share");
                assert!(verify_partial_identity_key(
                    round.genesis_hash,
                    public_atom,
                    &share
                ));
                partial_identity_shares.push(share);
            }
        }
        let block_key =
            combine_identity_key(&bundles[0].public, target_block, &partial_identity_shares)
                .expect("threshold partial identity shares combine into a block decryption key");
        assert_eq!(block_key.version, MEV_SHIELD_IBE_VERSION);
        assert_eq!(block_key.epoch, round.epoch);
        assert_eq!(block_key.target_block, target_block);
        assert_eq!(block_key.key_id, round.key_id);
        assert_eq!(block_key.finalized_ordering_block_number, finalized_number);
        assert_eq!(block_key.finalized_ordering_block_hash, finalized_hash);
        assert!(!block_key.identity_decryption_key.is_empty());

        let master_public_key = bundles[0]
            .public
            .epoch_key
            .master_public_key
            .as_slice()
            .to_vec();
        let publication_hash = epoch_publication_payload_hash(
            plan.epoch,
            round.key_id,
            round.first_block,
            round.last_block,
            plan.consensus_source,
            &master_public_key,
            bundles[0].public.epoch_key.total_weight,
            bundles[0].public.epoch_key.threshold_weight,
        );
        let mut publication_attestations = Vec::new();
        for authority in &authorities {
            let mut attestation = DkgOutputAttestationV1 {
                version: MEV_SHIELD_IBE_VERSION,
                round: round.clone(),
                authority_id: authority.authority_id.clone(),
                stake: authority.stake,
                public_output_hash: publication_hash,
                authority_signature: Vec::new(),
            };
            attestation.authority_signature =
                authority.sign_hash(output_attestation_payload_hash(&attestation));
            assert!(verify_sr25519_authority_signature(
                &authority.authority_id,
                output_attestation_payload_hash(&attestation),
                &attestation.authority_signature
            ));
            publication_attestations.push(DkgOutputAttestation {
                authority_id: attestation.authority_id,
                stake: attestation.stake,
                public_output_hash: attestation.public_output_hash,
                signature: attestation.authority_signature,
            });
        }
        let publication = EpochDkgPublication {
            epoch: plan.epoch,
            key_id: round.key_id,
            first_block: round.first_block,
            last_block: round.last_block,
            consensus_source: plan.consensus_source,
            master_public_key,
            total_weight: bundles[0].public.epoch_key.total_weight,
            threshold_weight: bundles[0].public.epoch_key.threshold_weight,
            public_output_hash: publication_hash,
            attestations: publication_attestations,
        };
        assert_eq!(publication.attestations.len(), authorities.len());
        assert_eq!(publication.public_output_hash, publication_hash);
        assert!(
            publication
                .attestations
                .iter()
                .fold(0u128, |sum, att| sum.saturating_add(att.stake))
                >= stake_threshold
        );
    }
}
