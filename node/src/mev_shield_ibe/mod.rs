pub mod block_import;
pub mod crypto;
pub mod dkg;
pub mod finality;
pub mod network;
pub mod submitter;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use codec::{Decode, Encode};
use futures::{StreamExt, channel::mpsc};
use parking_lot::Mutex;
use sp_core::H256;
use stp_mev_shield_ibe::{
    IbeBlockDecryptionKeyV1, IbePartialDecryptionKeyShareV1, IbePendingIdentity,
};

use self::{
    crypto::{combine_identity_key, derive_partial_identity_key, verify_partial_identity_key},
    dkg::{DkgKeySource, IdentityRoundKey},
    network::WireMessage,
};

#[derive(Clone)]
pub struct SharePoolConfig {
    pub max_pending_identities: u32,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode)]
pub struct IdentityRound {
    pub epoch: u64,
    pub target_block: u64,
    pub key_id: [u8; 16],
}

impl From<&IbePendingIdentity> for IdentityRound {
    fn from(identity: &IbePendingIdentity) -> Self {
        Self {
            epoch: identity.epoch,
            target_block: identity.target_block,
            key_id: identity.key_id,
        }
    }
}

impl From<&IdentityRound> for IdentityRoundKey {
    fn from(round: &IdentityRound) -> Self {
        Self {
            epoch: round.epoch,
            target_block: round.target_block,
            key_id: round.key_id,
        }
    }
}

#[derive(Default)]
struct RoundState {
    shares: BTreeMap<u32, IbePartialDecryptionKeyShareV1>,
    combined_submitted: bool,
}

#[derive(Clone)]
pub struct MevShieldIbeSharePool {
    inner: Arc<Inner>,
}

struct Inner {
    cfg: SharePoolConfig,
    dkg: Arc<dyn DkgKeySource>,
    genesis_hash: H256,

    rounds: Mutex<BTreeMap<IdentityRound, RoundState>>,

    /// Identities whose encrypted ordering has finalized and whose target block
    /// has arrived. Shares are accepted only for these identities.
    finalized_unlocked: Mutex<BTreeSet<IdentityRound>>,

    outbound: mpsc::UnboundedSender<WireMessage>,
}

impl MevShieldIbeSharePool {
    pub fn new(
        cfg: SharePoolConfig,
        dkg: Arc<dyn DkgKeySource>,
    ) -> Result<
        (
            Self,
            mpsc::UnboundedReceiver<WireMessage>,
            mpsc::UnboundedSender<WireMessage>,
        ),
        String,
    > {
        dkg.refresh()?;

        let genesis_hash = dkg.genesis_hash();

        let (out_tx, out_rx) = mpsc::unbounded();
        let (in_tx, in_rx) = mpsc::unbounded();

        let pool = Self {
            inner: Arc::new(Inner {
                cfg,
                dkg,
                genesis_hash,
                rounds: Mutex::new(BTreeMap::new()),
                finalized_unlocked: Mutex::new(BTreeSet::new()),
                outbound: out_tx,
            }),
        };

        pool.spawn_inbound(in_rx);

        Ok((pool, out_rx, in_tx))
    }

    fn spawn_inbound(&self, mut inbound: mpsc::UnboundedReceiver<WireMessage>) {
        let pool = self.clone();

        std::thread::Builder::new()
            .name("mev-shield-ibe-inbound".into())
            .spawn(move || {
                futures::executor::block_on(async move {
                    while let Some(msg) = inbound.next().await {
                        match msg {
                            WireMessage::PartialDecryptionKeyShareV1(share) => {
                                pool.import_share(share);
                            }
                        }
                    }
                });
            })
            .expect("spawn inbound share pool");
    }

    pub fn refresh_dkg(&self) -> Result<(), String> {
        self.inner.dkg.refresh()
    }

    pub fn mark_finalized_identity_unlocked(
        &self,
        identity: IbePendingIdentity,
        finalized_ordering_block_number: u64,
        finalized_ordering_block_hash: H256,
        current_best_block: u64,
    ) {
        if current_best_block < identity.target_block {
            return;
        }

        let round = IdentityRound::from(&identity);
        let dkg_round = IdentityRoundKey::from(&round);

        if self
            .inner
            .dkg
            .secret_bundle_for_identity(&dkg_round)
            .is_none()
        {
            return;
        }

        self.inner.finalized_unlocked.lock().insert(round.clone());

        self.publish_local_shares(
            round,
            finalized_ordering_block_number,
            finalized_ordering_block_hash,
        );
    }

    fn publish_local_shares(
        &self,
        round: IdentityRound,
        finalized_ordering_block_number: u64,
        finalized_ordering_block_hash: H256,
    ) {
        let dkg_round = IdentityRoundKey::from(&round);

        let Some(bundle) = self.inner.dkg.secret_bundle_for_identity(&dkg_round) else {
            return;
        };

        let epoch_key = bundle.epoch_key();

        if epoch_key.epoch != round.epoch || epoch_key.key_id != round.key_id {
            return;
        }

        for atom in &bundle.local_atoms {
            let Ok(share) = derive_partial_identity_key(
                self.inner.genesis_hash,
                &bundle,
                round.target_block,
                atom,
                finalized_ordering_block_number,
                finalized_ordering_block_hash,
            ) else {
                continue;
            };

            self.import_share(share.clone());

            let _ = self
                .inner
                .outbound
                .unbounded_send(WireMessage::PartialDecryptionKeyShareV1(share));
        }
    }

    pub fn import_share(&self, share: IbePartialDecryptionKeyShareV1) -> bool {
        let round = IdentityRound {
            epoch: share.epoch,
            target_block: share.target_block,
            key_id: share.key_id,
        };

        if !self.inner.finalized_unlocked.lock().contains(&round) {
            return false;
        }

        let dkg_round = IdentityRoundKey::from(&round);

        let Some(public_atom) = self
            .inner
            .dkg
            .public_atom_for_identity(&dkg_round, share.share_id)
        else {
            return false;
        };

        if !verify_partial_identity_key(self.inner.genesis_hash, &public_atom, &share) {
            return false;
        }

        let mut rounds = self.inner.rounds.lock();
        let state = rounds.entry(round).or_default();

        state.shares.insert(share.share_id, share);
        true
    }

    pub fn try_combine_ready_keys(&self) -> Vec<IbeBlockDecryptionKeyV1> {
        let mut out = Vec::new();
        let mut rounds = self.inner.rounds.lock();

        for (round, state) in rounds.iter_mut() {
            if state.combined_submitted {
                continue;
            }

            let dkg_round = IdentityRoundKey::from(round);

            let Some(public_output) = self.inner.dkg.public_output_for_identity(&dkg_round) else {
                continue;
            };

            let shares: Vec<_> = state.shares.values().cloned().collect();

            let Ok(combined) = combine_identity_key(&public_output, round.target_block, &shares)
            else {
                continue;
            };

            state.combined_submitted = true;
            out.push(combined);
        }

        out
    }

    pub fn max_pending_identities(&self) -> u32 {
        self.inner.cfg.max_pending_identities
    }
}
