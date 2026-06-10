use codec::Encode;
use futures::FutureExt;
use mev_shield_ibe_runtime_api::MevShieldIbeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::{Environment, InherentData, Proposer};
use sp_runtime::{
    Digest, DigestItem,
    traits::{Block as BlockT, Header as HeaderT, SaturatedConversion},
};
use std::{
    collections::BTreeSet, future::Future, marker::PhantomData, pin::Pin, sync::Arc, time::Duration,
};
use stp_mev_shield_ibe::{
    IBE_BLOCK_DECRYPTION_KEYS_ENGINE_ID, IbeBlockDecryptionKeyInherentData,
    IbeBlockDecryptionKeyShareBundleV1,
};

use super::MevShieldIbeSharePool;

const LOG_TARGET: &str = "mev-shield-ibe";
const MAX_IBE_BLOCK_KEY_BUNDLES_PER_PRERUNTIME_DIGEST: usize = 32;
const MAX_IBE_BLOCK_KEY_PRERUNTIME_DIGEST_BYTES: usize = 128 * 1024;

/// Proposer-factory wrapper that injects threshold-IBE block-key release
/// bundles into the header before the runtime executes `on_initialize`.
pub struct IbePreRuntimeDigestProposerFactory<Inner, Block, Client> {
    inner: Inner,
    share_pool: Option<MevShieldIbeSharePool>,
    client: Arc<Client>,
    _marker: PhantomData<fn() -> Block>,
}

impl<Inner, Block, Client> IbePreRuntimeDigestProposerFactory<Inner, Block, Client> {
    pub fn new(
        inner: Inner,
        share_pool: Option<MevShieldIbeSharePool>,
        client: Arc<Client>,
    ) -> Self {
        Self {
            inner,
            share_pool,
            client,
            _marker: PhantomData,
        }
    }
}

impl<Inner, Block, Client> Environment<Block>
    for IbePreRuntimeDigestProposerFactory<Inner, Block, Client>
where
    Inner: Environment<Block> + Send + 'static,
    Inner::CreateProposer: Send + 'static,
    Inner::Proposer: Send + 'static,
    Inner::Error: Send + 'static,
    Block: BlockT + Send + Sync + 'static,
    <Block as BlockT>::Hash: Copy + Send + Sync + 'static,
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
    Client::Api: MevShieldIbeApi<Block>,
{
    type Proposer = IbePreRuntimeDigestProposer<Inner::Proposer, Block, Client>;
    type CreateProposer = Pin<Box<dyn Future<Output = Result<Self::Proposer, Self::Error>> + Send>>;
    type Error = Inner::Error;

    fn init(&mut self, parent_header: &<Block as BlockT>::Header) -> Self::CreateProposer {
        let parent_hash = parent_header.hash();
        let parent_number = (*parent_header.number()).saturated_into::<u64>();
        let share_pool = self.share_pool.clone();
        let client = self.client.clone();
        let create = self.inner.init(parent_header);

        async move {
            let inner = create.await?;
            Ok(IbePreRuntimeDigestProposer {
                inner,
                share_pool,
                client,
                parent_hash,
                parent_number,
                _marker: PhantomData,
            })
        }
        .boxed()
    }
}

pub struct IbePreRuntimeDigestProposer<Inner, Block: BlockT, Client> {
    inner: Inner,
    share_pool: Option<MevShieldIbeSharePool>,
    client: Arc<Client>,
    parent_hash: <Block as BlockT>::Hash,
    parent_number: u64,
    _marker: PhantomData<fn() -> Block>,
}

impl<Inner, Block, Client> Proposer<Block> for IbePreRuntimeDigestProposer<Inner, Block, Client>
where
    Inner: Proposer<Block>,
    Block: BlockT + Send + Sync + 'static,
    <Block as BlockT>::Hash: Copy,
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync + 'static,
    Client::Api: MevShieldIbeApi<Block>,
{
    type Error = Inner::Error;
    type Proposal = Inner::Proposal;
    type ProofRecording = Inner::ProofRecording;
    type Proof = Inner::Proof;

    fn propose(
        self,
        inherent_data: InherentData,
        mut inherent_digests: Digest,
        max_duration: Duration,
        block_size_limit: Option<usize>,
    ) -> Self::Proposal {
        if let Some(digest) = build_ibe_block_key_preruntime_digest::<Block, Client>(
            self.share_pool,
            self.client,
            self.parent_hash,
            self.parent_number,
        ) {
            inherent_digests.push(digest);
        }

        self.inner.propose(
            inherent_data,
            inherent_digests,
            max_duration,
            block_size_limit,
        )
    }
}

pub fn build_ibe_block_key_preruntime_digest<Block, Client>(
    share_pool: Option<MevShieldIbeSharePool>,
    client: Arc<Client>,
    parent_hash: <Block as BlockT>::Hash,
    parent_number: u64,
) -> Option<DigestItem>
where
    Block: BlockT,
    <Block as BlockT>::Hash: Copy,
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: MevShieldIbeApi<Block>,
{
    let bundles = collect_ready_ibe_block_key_bundles_for_child::<Block, Client>(
        share_pool,
        client,
        parent_hash,
        parent_number,
    );

    if bundles.is_empty() {
        return None;
    }

    let payload = IbeBlockDecryptionKeyInherentData {
        keys: Vec::new(),
        share_bundles: bundles,
    }
    .encode();

    if payload.len() > MAX_IBE_BLOCK_KEY_PRERUNTIME_DIGEST_BYTES {
        log::warn!(
            target: LOG_TARGET,
            "dropping oversized IBE block-key pre-runtime digest: {} bytes",
            payload.len(),
        );
        return None;
    }

    Some(DigestItem::PreRuntime(
        IBE_BLOCK_DECRYPTION_KEYS_ENGINE_ID,
        payload,
    ))
}

pub fn collect_ready_ibe_block_key_bundles_for_child<Block, Client>(
    share_pool: Option<MevShieldIbeSharePool>,
    client: Arc<Client>,
    parent_hash: <Block as BlockT>::Hash,
    parent_number: u64,
) -> Vec<IbeBlockDecryptionKeyShareBundleV1>
where
    Block: BlockT,
    <Block as BlockT>::Hash: Copy,
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block>,
    Client::Api: MevShieldIbeApi<Block>,
{
    let Some(share_pool) = share_pool else {
        return Vec::new();
    };

    let target_block = parent_number.saturating_add(1);
    let mut seen = BTreeSet::new();
    let api = client.runtime_api();

    let mut bundles: Vec<IbeBlockDecryptionKeyShareBundleV1> = share_pool
        .try_combine_ready_key_bundles()
        .into_iter()
        .filter(|bundle| bundle.key.target_block == target_block)
        .filter(|bundle| {
            seen.insert((bundle.key.epoch, bundle.key.target_block, bundle.key.key_id))
        })
        .filter(|bundle| {
            match api.has_ibe_block_key(
                parent_hash,
                bundle.key.epoch,
                bundle.key.target_block,
                bundle.key.key_id,
            ) {
                Ok(already_present) => !already_present,
                Err(error) => {
                    log::warn!(
                        target: LOG_TARGET,
                        "failed to query parent IBE block-key state: {:?}",
                        error,
                    );
                    false
                }
            }
        })
        .collect();

    if bundles.len() > MAX_IBE_BLOCK_KEY_BUNDLES_PER_PRERUNTIME_DIGEST {
        bundles.truncate(MAX_IBE_BLOCK_KEY_BUNDLES_PER_PRERUNTIME_DIGEST);
    }

    if !bundles.is_empty() {
        log::debug!(
            target: LOG_TARGET,
            "prepared {} IBE block-key release bundle(s) for pre-runtime digest at target block {}",
            bundles.len(),
            target_block,
        );
    }

    bundles
}
