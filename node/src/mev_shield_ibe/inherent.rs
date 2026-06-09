use crate::mev_shield_ibe::MevShieldIbeSharePool;
use async_trait::async_trait;
use mev_shield_ibe_runtime_api::MevShieldIbeApi;
use sc_client_api::HeaderBackend;
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, SaturatedConversion};
use std::{collections::BTreeSet, sync::Arc};
use stp_mev_shield_ibe::{
    IBE_BLOCK_DECRYPTION_KEYS_INHERENT_IDENTIFIER, IbeBlockDecryptionKeyInherentData,
};

const LOG_TARGET: &str = "mev-shield-ibe";
const MAX_IBE_BLOCK_KEYS_PER_INHERENT: usize = 64;

/// Author-side inherent-data provider for threshold-IBE block decryption keys.
///
/// The finality gate unlocks identities once target-1 is finalized. The share
/// pool combines ready validator shares, and this provider places reconstructed
/// keys for the child block currently being authored into inherent data.
pub struct IbeBlockDecryptionKeyInherentDataProvider<Block: BlockT, Client> {
    share_pool: Option<MevShieldIbeSharePool>,
    client: Arc<Client>,
    parent_hash: <Block as BlockT>::Hash,
}

impl<Block: BlockT, Client> IbeBlockDecryptionKeyInherentDataProvider<Block, Client> {
    pub fn new(
        share_pool: Option<MevShieldIbeSharePool>,
        client: Arc<Client>,
        parent_hash: <Block as BlockT>::Hash,
    ) -> Self {
        Self {
            share_pool,
            client,
            parent_hash,
        }
    }
}

/// Combines the consensus-specific inherent data provider with the threshold-IBE
/// block-key provider without nesting the consensus provider tuple.
///
/// The consensus provider returned by `CM::create_inherent_data_providers`
/// already knows how to expose its slot through `sc_consensus_slots`. Returning
/// `(base, ibe)` nests that provider tuple and loses the slot extension. This
/// wrapper delegates inherent data to both providers and delegates `slot()`
/// directly to the consensus provider.
pub struct IbeCompositeInherentDataProvider<Base, Ibe> {
    base: Base,
    ibe: Ibe,
}

impl<Base, Ibe> IbeCompositeInherentDataProvider<Base, Ibe> {
    pub fn new(base: Base, ibe: Ibe) -> Self {
        Self { base, ibe }
    }
}

#[async_trait]
impl<Base, Ibe> sp_inherents::InherentDataProvider for IbeCompositeInherentDataProvider<Base, Ibe>
where
    Base: sp_inherents::InherentDataProvider + Send + Sync,
    Ibe: sp_inherents::InherentDataProvider + Send + Sync,
{
    async fn provide_inherent_data(
        &self,
        inherent_data: &mut sp_inherents::InherentData,
    ) -> Result<(), sp_inherents::Error> {
        self.base.provide_inherent_data(inherent_data).await?;
        self.ibe.provide_inherent_data(inherent_data).await
    }

    async fn try_handle_error(
        &self,
        identifier: &sp_inherents::InherentIdentifier,
        error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        if let Some(result) = self.base.try_handle_error(identifier, error).await {
            return Some(result);
        }
        self.ibe.try_handle_error(identifier, error).await
    }
}

impl<Base, Ibe> sc_consensus_slots::InherentDataProviderExt
    for IbeCompositeInherentDataProvider<Base, Ibe>
where
    Base: sc_consensus_slots::InherentDataProviderExt,
{
    fn slot(&self) -> sp_consensus_slots::Slot {
        sc_consensus_slots::InherentDataProviderExt::slot(&self.base)
    }
}

#[async_trait]
impl<Block, Client> sp_inherents::InherentDataProvider
    for IbeBlockDecryptionKeyInherentDataProvider<Block, Client>
where
    Block: BlockT,
    Client: HeaderBackend<Block> + ProvideRuntimeApi<Block> + Send + Sync,
    Client::Api: MevShieldIbeApi<Block>,
{
    async fn provide_inherent_data(
        &self,
        inherent_data: &mut sp_inherents::InherentData,
    ) -> Result<(), sp_inherents::Error> {
        let Some(share_pool) = &self.share_pool else {
            return Ok(());
        };

        let mut keys = share_pool.try_combine_ready_keys();
        if keys.is_empty() {
            return Ok(());
        }

        let parent_number: u64 = self
            .client
            .header(self.parent_hash)
            .ok()
            .flatten()
            .map(|header| (*header.number()).saturated_into::<u64>())
            .unwrap_or_else(|| self.client.info().best_number.saturated_into::<u64>());
        let target_block = parent_number.saturating_add(1);

        keys.retain(|key| key.target_block == target_block);
        if keys.len() > MAX_IBE_BLOCK_KEYS_PER_INHERENT {
            keys.truncate(MAX_IBE_BLOCK_KEYS_PER_INHERENT);
        }

        let mut seen = BTreeSet::new();
        keys.retain(|key| seen.insert((key.epoch, key.target_block, key.key_id)));

        let api = self.client.runtime_api();
        keys.retain(|key| {
            match api.has_ibe_block_key(
                self.parent_hash,
                key.epoch,
                key.target_block,
                key.key_id,
            ) {
                Ok(already_on_chain) => !already_on_chain,
                Err(error) => {
                    log::debug!(
                        target: LOG_TARGET,
                        "skipping IBE block-key inherent candidate after runtime API error: {error:?}"
                    );
                    false
                }
            }
        });

        if keys.is_empty() {
            return Ok(());
        }

        log::debug!(
            target: LOG_TARGET,
            "including {} threshold-IBE block decryption key(s) for target block {}",
            keys.len(),
            target_block,
        );

        inherent_data.put_data(
            IBE_BLOCK_DECRYPTION_KEYS_INHERENT_IDENTIFIER,
            &IbeBlockDecryptionKeyInherentData { keys },
        )
    }

    async fn try_handle_error(
        &self,
        _identifier: &sp_inherents::InherentIdentifier,
        _error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        None
    }
}
