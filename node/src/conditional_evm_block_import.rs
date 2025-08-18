use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::{Block as BlockT, Header};
use std::marker::PhantomData;

pub struct ConditionalEVMBlockImport<B: BlockT, I, F> {
    inner: I,
    frontier_block_import: F,
    _marker: PhantomData<B>,
}

impl<B, I, F> Clone for ConditionalEVMBlockImport<B, I, F>
where
    B: BlockT,
    I: Clone + BlockImport<B>,
    F: Clone + BlockImport<B>,
{
    fn clone(&self) -> Self {
        ConditionalEVMBlockImport {
            inner: self.inner.clone(),
            frontier_block_import: self.frontier_block_import.clone(),
            _marker: PhantomData,
        }
    }
}

impl<B, I, F> ConditionalEVMBlockImport<B, I, F>
where
    B: BlockT,
    I: BlockImport<B>,
    I::Error: Into<ConsensusError>,
    F: BlockImport<B>,
    F::Error: Into<ConsensusError>,
{
    pub fn new(inner: I, frontier_block_import: F) -> Self {
        Self {
            inner,
            frontier_block_import,
            _marker: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<B, I, F> BlockImport<B> for ConditionalEVMBlockImport<B, I, F>
where
    B: BlockT,
    I: BlockImport<B> + Send + Sync,
    I::Error: Into<ConsensusError>,
    F: BlockImport<B> + Send + Sync,
    F::Error: Into<ConsensusError>,
{
    type Error = ConsensusError;

    async fn check_block(&self, block: BlockCheckParams<B>) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await.map_err(Into::into)
    }

    async fn import_block(&self, block: BlockImportParams<B>) -> Result<ImportResult, Self::Error> {
        // 4345556 - mainnet runtime upgrade block with Frontier
        if *block.header.number() < 4345557u32.into() {
            self.inner.import_block(block).await.map_err(Into::into)
        } else {
            self.frontier_block_import
                .import_block(block)
                .await
                .map_err(Into::into)
        }
    }
}
