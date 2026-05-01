use std::sync::Arc;

use async_trait::async_trait;
use codec::Encode;
use mev_shield_ibe_runtime_api::{MevShieldExtrinsicClass, MevShieldIbeApi};
use sc_consensus::{
    BlockCheckParams, BlockImport, BlockImportParams, ForkChoiceStrategy, ImportResult,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, SaturatedConversion};

pub struct MevShieldBlockImport<I, C> {
    inner: I,
    client: Arc<C>,
}

impl<I, C> MevShieldBlockImport<I, C> {
    pub fn new(inner: I, client: Arc<C>) -> Self {
        Self { inner, client }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MevShieldBlockImportError<E> {
    #[error("inner block import error: {0:?}")]
    Inner(E),

    #[error("runtime api error")]
    RuntimeApi,

    #[error("v2 key submitted before finalized ordering proof is locally valid")]
    KeyBeforeFinality,

    #[error("stale v2 encrypted target block")]
    StaleEncryptedTarget,

    #[error("unencrypted non-operational transaction preempts encrypted queue")]
    UnencryptedPreemptsEncryptedQueue,

    #[error("full block censors pending encrypted queue")]
    FullBlockCensorsEncryptedQueue,
}

#[async_trait]
impl<Block, I, C> BlockImport<Block> for MevShieldBlockImport<I, C>
where
    Block: BlockT,
    I: BlockImport<Block> + Send,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: MevShieldIbeApi<Block>,
{
    type Error = MevShieldBlockImportError<I::Error>;

    async fn check_block(
        &mut self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner
            .check_block(block)
            .await
            .map_err(MevShieldBlockImportError::Inner)
    }

    async fn import_block(
        &mut self,
        block: BlockImportParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.verify_mev_shield_validity(&block)?;

        self.inner
            .import_block(block)
            .await
            .map_err(MevShieldBlockImportError::Inner)
    }
}

impl<I, C> MevShieldBlockImport<I, C> {
    fn local_finality_contains<Block>(&self, number: u64, hash: Block::Hash) -> bool
    where
        Block: BlockT,
        C: HeaderBackend<Block>,
    {
        let finalized_number: u64 = self.client.info().finalized_number.saturated_into();

        if number > finalized_number {
            return false;
        }

        let Ok(Some(local_hash)) = self.client.hash(number.into()) else {
            return false;
        };

        local_hash == hash
    }

    fn verify_mev_shield_validity<Block>(
        &self,
        block: &BlockImportParams<Block>,
    ) -> Result<(), MevShieldBlockImportError<I::Error>>
    where
        Block: BlockT,
        Block::Hash: From<sp_core::H256>,
        C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
        C::Api: MevShieldIbeApi<Block>,
    {
        let parent_hash = block.header.parent_hash();

        let pending = self
            .client
            .runtime_api()
            .pending_encrypted_queue_len(*parent_hash)
            .map_err(|_| MevShieldBlockImportError::RuntimeApi)?;

        let mut has_unencrypted_non_operational = false;
        let mut has_only_operational_or_encrypted = true;

        for xt in block.body.as_ref().unwrap_or(&Vec::new()).iter() {
            let class = self
                .client
                .runtime_api()
                .classify_extrinsic(*parent_hash, xt.encode())
                .map_err(|_| MevShieldBlockImportError::RuntimeApi)?;

            match class {
                MevShieldExtrinsicClass::Operational => {}

                MevShieldExtrinsicClass::SubmitEncryptedV1 => {
                    // v1 remains valid.
                }

                MevShieldExtrinsicClass::SubmitEncryptedV2 { target_block, .. } => {
                    let block_number: u64 = block.header.number().saturated_into();

                    if target_block <= block_number {
                        return Err(MevShieldBlockImportError::StaleEncryptedTarget);
                    }
                }

                MevShieldExtrinsicClass::SubmitBlockDecryptionKey {
                    finalized_ordering_block_number,
                    finalized_ordering_block_hash,
                    ..
                } => {
                    if !self.local_finality_contains::<Block>(
                        finalized_ordering_block_number,
                        finalized_ordering_block_hash.into(),
                    ) {
                        return Err(MevShieldBlockImportError::KeyBeforeFinality);
                    }
                }

                MevShieldExtrinsicClass::UnencryptedNonOperational => {
                    has_unencrypted_non_operational = true;
                    has_only_operational_or_encrypted = false;
                }
            }
        }

        // Invariant 1.
        if pending > 0 && has_unencrypted_non_operational {
            return Err(MevShieldBlockImportError::UnencryptedPreemptsEncryptedQueue);
        }

        // Invariant 3.
        // Replace this placeholder with Subtensor's actual block-full predicate.
        let block_is_full = crate::block_weight::block_is_full(block);

        if block_is_full && pending > 0 && !has_only_operational_or_encrypted {
            return Err(MevShieldBlockImportError::FullBlockCensorsEncryptedQueue);
        }

        // Invariant 2 is enforced in runtime by processing only the queue head:
        // NotReady breaks, Ready/Invalid consumes exactly the current head.
        Ok(())
    }
}
