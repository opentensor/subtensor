use fp_consensus::{FindLogError, ensure_log};
use fp_rpc::EthereumRuntimeRPCApi;
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, ImportResult,
};
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_consensus::Error as ConsensusError;
use sp_runtime::traits::{Block as BlockT, Header};
use std::{marker::PhantomData, sync::Arc};


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Multiple runtime Ethereum blocks, rejecting!")]
    MultipleRuntimeLogs,
    #[error("Runtime Ethereum block not found, rejecting!")]
    NoRuntimeLog,
    #[error("Cannot access the runtime at genesis, rejecting!")]
    RuntimeApiCallFailed,
}

pub struct ConditionalEVMBlockImport<B: BlockT, I, F, C> {
    inner: I,
    frontier_block_import: F,
    client: Arc<C>,
    _marker: PhantomData<B>,
}

impl<B, I, F, C> Clone for ConditionalEVMBlockImport<B, I, F, C>
where
    B: BlockT,
    I: Clone + BlockImport<B>,
    F: Clone + BlockImport<B>,
{
    fn clone(&self) -> Self {
        ConditionalEVMBlockImport {
            inner: self.inner.clone(),
            frontier_block_import: self.frontier_block_import.clone(),
            client: self.client.clone(),
            _marker: PhantomData,
        }
    }
}

impl<B, I, F, C> ConditionalEVMBlockImport<B, I, F, C>
where
    B: BlockT,
    I: BlockImport<B>,
    I::Error: Into<ConsensusError>,
    F: BlockImport<B>,
    F::Error: Into<ConsensusError>,
    C: ProvideRuntimeApi<B>,
    C::Api: BlockBuilderApi<B> + EthereumRuntimeRPCApi<B>,
{
    pub fn new(inner: I, frontier_block_import: F, client: Arc<C>) -> Self {
        Self {
            inner,
            frontier_block_import,
            client,
            _marker: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<B, I, F, C> BlockImport<B> for ConditionalEVMBlockImport<B, I, F, C>
where
    B: BlockT,
    I: BlockImport<B> + Send + Sync,
    I::Error: Into<ConsensusError>,
    F: BlockImport<B> + Send + Sync,
    F::Error: Into<ConsensusError>,
    C: ProvideRuntimeApi<B> + Send + Sync,
    C::Api: BlockBuilderApi<B> + EthereumRuntimeRPCApi<B>,
{
    type Error = ConsensusError;

    async fn check_block(&self, block: BlockCheckParams<B>) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await.map_err(Into::into)
    }

    async fn import_block(&self, block: BlockImportParams<B>) -> Result<ImportResult, Self::Error> {
        // Import like Frontier, but fallback to grandpa import for errors
        match ensure_log(block.header.digest()).map_err(Error::from) {
            Ok(()) => self.inner.import_block(block).await.map_err(Into::into),
            _ => self.inner.import_block(block).await.map_err(Into::into),
        }
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.to_string()
    }
}

impl From<FindLogError> for Error {
    fn from(error: FindLogError) -> Error {
        match error {
            FindLogError::NotFound => Error::NoRuntimeLog,
            FindLogError::MultipleLogs => Error::MultipleRuntimeLogs,
        }
    }
}

impl From<Error> for ConsensusError {
    fn from(error: Error) -> ConsensusError {
        ConsensusError::ClientImport(error.to_string())
    }
}
