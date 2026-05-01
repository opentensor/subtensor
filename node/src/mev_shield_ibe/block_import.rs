use std::sync::Arc;

use async_trait::async_trait;
use codec::Encode;
use mev_shield_ibe_runtime_api::{MevShieldExtrinsicClass, MevShieldIbeApi};
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT, NumberFor, SaturatedConversion};

pub struct MevShieldBlockImport<I, C> {
    inner: I,
    client: Arc<C>,
}

impl<I, C> MevShieldBlockImport<I, C> {
    pub fn new(inner: I, client: Arc<C>) -> Self {
        Self { inner, client }
    }
}

impl<I, C> Clone for MevShieldBlockImport<I, C>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            client: self.client.clone(),
        }
    }
}

#[async_trait]
impl<Block, I, C> BlockImport<Block> for MevShieldBlockImport<I, C>
where
    Block: BlockT,
    Block::Hash: Clone + Into<sp_core::H256>,
    I: BlockImport<Block> + Send + Sync,
    I::Error: Into<sp_consensus::Error>,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: MevShieldIbeApi<Block>,
{
    type Error = sp_consensus::Error;

    async fn check_block(
        &self,
        block: BlockCheckParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await.map_err(Into::into)
    }

    async fn import_block(
        &self,
        block: BlockImportParams<Block>,
    ) -> Result<ImportResult, Self::Error> {
        self.verify_mev_shield_validity::<Block>(&block)?;

        self.inner.import_block(block).await.map_err(Into::into)
    }
}

impl<I, C> MevShieldBlockImport<I, C> {
    fn invalid(reason: &'static str) -> sp_consensus::Error {
        sp_consensus::Error::ClientImport(reason.to_string())
    }

    fn local_finality_contains<Block>(&self, number: u64, hash: sp_core::H256) -> bool
    where
        Block: BlockT,
        Block::Hash: Into<sp_core::H256>,
        C: HeaderBackend<Block>,
    {
        let finalized_number: u64 = self.client.info().finalized_number.saturated_into();

        if number > finalized_number {
            return false;
        }

        let number_for_block: NumberFor<Block> = number.saturated_into();

        let Ok(Some(local_hash)) = self.client.hash(number_for_block) else {
            return false;
        };

        local_hash.into() == hash
    }

    fn verify_mev_shield_validity<Block>(
        &self,
        block: &BlockImportParams<Block>,
    ) -> Result<(), sp_consensus::Error>
    where
        Block: BlockT,
        Block::Hash: Clone + Into<sp_core::H256>,
        C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
        C::Api: MevShieldIbeApi<Block>,
    {
        let parent_hash = block.header.parent_hash().clone();

        let pending = self
            .client
            .runtime_api()
            .pending_encrypted_queue_len(parent_hash.clone())
            .map_err(|_| Self::invalid("MEVShield v2 runtime API pending queue lookup failed"))?;

        if let Some(body) = block.body.as_ref() {
            for xt in body.iter() {
                let class = self
                    .client
                    .runtime_api()
                    .classify_extrinsic(parent_hash.clone(), xt.encode())
                    .map_err(|_| {
                        Self::invalid("MEVShield v2 runtime API extrinsic classification failed")
                    })?;

                match class {
                    MevShieldExtrinsicClass::Operational => {}

                    MevShieldExtrinsicClass::SubmitEncryptedV1 => {}

                    MevShieldExtrinsicClass::SubmitEncryptedV2 { target_block, .. } => {
                        let block_number: u64 = (*block.header.number()).saturated_into();

                        if target_block <= block_number {
                            return Err(Self::invalid(
                                "MEVShield v2 encrypted transaction targets a stale block",
                            ));
                        }
                    }

                    MevShieldExtrinsicClass::SubmitBlockDecryptionKey {
                        finalized_ordering_block_number,
                        finalized_ordering_block_hash,
                        ..
                    } => {
                        if !self.local_finality_contains::<Block>(
                            finalized_ordering_block_number,
                            finalized_ordering_block_hash,
                        ) {
                            return Err(Self::invalid(
                                "MEVShield v2 block decryption key submitted before ordering finality",
                            ));
                        }
                    }

                    MevShieldExtrinsicClass::UnencryptedNonOperational => {
                        if pending > 0 {
                            return Err(Self::invalid(
                                "MEVShield v2 pending encrypted queue preempted by unencrypted non-operational transaction",
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
