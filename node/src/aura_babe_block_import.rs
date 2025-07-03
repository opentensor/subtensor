use babe_primitives::BABE_ENGINE_ID;
use babe_primitives::BabeApi;
use babe_primitives::BabeConfiguration;
use sc_client_api::AuxStore;
use sc_client_api::PreCommitActions;
use sc_consensus::BlockCheckParams;
use sc_consensus::BlockImport;
use sc_consensus::BlockImportParams;
use sc_consensus::ImportResult;
use sc_consensus_babe::BabeBlockImport;
use sc_consensus_babe::BabeLink;
use sp_api::ApiExt;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_blockchain::HeaderMetadata;
use sp_blockchain::Result as ClientResult;
use sp_consensus::Error as ConsensusError;
use sp_runtime::{
    DigestItem,
    generic::OpaqueDigestItemId,
    traits::{Block as BlockT, Header, NumberFor, SaturatedConversion, Zero},
};
use std::sync::Arc;

/// Wrapper for BabeBlockImport that will only perform the Babe block import
/// logic when it is a Babe block. If it is Aura, it will skip the Babe block
/// import and call the inner import.
pub struct AuraOrBabeBlockImport<Block: BlockT, Client, Inner> {
    babe_block_import: BabeBlockImport<Block, Client, Inner>,
    inner: Inner,
}

impl<Block: BlockT, I: Clone, Client> Clone for AuraOrBabeBlockImport<Block, Client, I> {
    fn clone(&self) -> Self {
        AuraOrBabeBlockImport {
            inner: self.inner.clone(),
            babe_block_import: self.babe_block_import.clone(),
        }
    }
}

impl<Block: BlockT, Client, Inner> AuraOrBabeBlockImport<Block, Client, Inner>
where
    Block: BlockT,
    Inner: BlockImport<Block> + Send + Sync + Clone,
    Inner::Error: Into<ConsensusError>,
    Client: HeaderBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + AuxStore
        + ProvideRuntimeApi<Block>
        + Send
        + Sync
        + PreCommitActions<Block>
        + 'static,
    Client::Api: BabeApi<Block> + ApiExt<Block>,
{
    pub fn block_import(
        babe_configuration: BabeConfiguration,
        inner: Inner,
        client: Arc<Client>,
    ) -> ClientResult<(AuraOrBabeBlockImport<Block, Client, Inner>, BabeLink<Block>)> {
        let (babe_import, babe_link) =
            sc_consensus_babe::block_import(babe_configuration, inner.clone(), client)?;
        Ok((
            AuraOrBabeBlockImport {
                babe_block_import: babe_import,
                inner,
            },
            babe_link,
        ))
    }
}

#[async_trait::async_trait]
impl<Block, Client, Inner> BlockImport<Block> for AuraOrBabeBlockImport<Block, Client, Inner>
where
    Block: BlockT,
    Inner: BlockImport<Block> + Send + Sync + Clone,
    Inner::Error: Into<ConsensusError>,
    Client: HeaderBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + AuxStore
        + ProvideRuntimeApi<Block>
        + Send
        + Sync
        + PreCommitActions<Block>
        + 'static,
    Client::Api: BabeApi<Block> + ApiExt<Block>,
{
    type Error = ConsensusError;

    async fn import_block(
        &self,
        mut block: BlockImportParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        // Block zero has no seal.
        let number: NumberFor<Block> = block.post_header().number().clone();
        log::info!("import_block number: {:?}", number);
        if number.is_zero() {
            return self.inner.import_block(block).await.map_err(Into::into);
        }

        let consensus_engine_id = crate::common::block_consensus_engine_id(&block);
        if consensus_engine_id == BABE_ENGINE_ID {
            self.babe_block_import.import_block(block).await
        } else {
            self.inner.import_block(block).await.map_err(Into::into)
        }
    }

    async fn check_block(
        &self,
        block_check_params: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        let block_check_params = BlockCheckParams {
            hash: block_check_params.hash,
            number: block_check_params.number,
            parent_hash: block_check_params.parent_hash,
            import_existing: block_check_params.import_existing,
            allow_missing_state: true,
            allow_missing_parent: true,
        };
        self.babe_block_import.check_block(block_check_params).await
    }
}
