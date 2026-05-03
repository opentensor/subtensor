use async_trait::async_trait;
use codec::Encode;
use mev_shield_ibe_runtime_api::{MevShieldExtrinsicClass, MevShieldIbeApi};
use sc_client_api::BlockBackend;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::Error as ConsensusError;
use sp_core::H256;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use std::{error::Error as StdError, marker::PhantomData, sync::Arc};

pub struct MevShieldBlockImport<I, C, B> {
    inner: I,
    client: Arc<C>,
    _marker: PhantomData<B>,
}

impl<I, C, B> Clone for MevShieldBlockImport<I, C, B>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            client: self.client.clone(),
            _marker: PhantomData,
        }
    }
}

impl<I, C, B> MevShieldBlockImport<I, C, B> {
    pub fn new(inner: I, client: Arc<C>) -> Self {
        Self {
            inner,
            client,
            _marker: PhantomData,
        }
    }
}

impl<I, C, B> MevShieldBlockImport<I, C, B>
where
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + BlockBackend<B> + Send + Sync + 'static,
    C::Api: MevShieldIbeApi<B>,
    B: BlockT,
    B::Hash: From<H256>,
{
    fn block_number_u64(header: &B::Header) -> Result<u64, String> {
        (*header.number())
            .try_into()
            .map_err(|_| "block number does not fit u64".to_string())
    }

    fn canonical_finalized_contains(&self, number: u64, hash: B::Hash) -> Result<bool, String> {
        let finalized_hash = self.client.info().finalized_hash;
        let finalized_header = self
            .client
            .header(finalized_hash)
            .map_err(|e| format!("read finalized header failed: {e:?}"))?
            .ok_or_else(|| "missing finalized header".to_string())?;

        let finalized_number: u64 = (*finalized_header.number())
            .try_into()
            .map_err(|_| "finalized number does not fit u64".to_string())?;
        if number > finalized_number {
            return Ok(false);
        }

        let header_number = number
            .try_into()
            .map_err(|_| "finality number does not fit header number type".to_string())?;
        let canonical_hash = self
            .client
            .hash(header_number)
            .map_err(|e| format!("read canonical hash failed: {e:?}"))?;
        Ok(canonical_hash == Some(hash))
    }

    fn verify_mev_shield_block(&self, parent_hash: B::Hash, block: &B) -> Result<(), String> {
        let api = self.client.runtime_api();
        let encoded = block
            .extrinsics()
            .iter()
            .map(|xt| xt.encode())
            .collect::<Vec<_>>();
        let composition = api
            .block_composition(parent_hash, encoded.clone())
            .map_err(|e| format!("block_composition runtime API failed: {e:?}"))?;

        let block_number = Self::block_number_u64(block.header())?;
        let mut pending_queue_len = composition.pending_queue_len_at_parent;

        for xt in encoded {
            let class = api
                .classify_extrinsic(parent_hash, xt)
                .map_err(|e| format!("classify_extrinsic runtime API failed: {e:?}"))?;

            match class {
                MevShieldExtrinsicClass::SubmitEncryptedV2 { target_block, .. } => {
                    if target_block <= block_number {
                        return Err(format!(
                            "encrypted v2 target {target_block} is not future of block {block_number}",
                        ));
                    }
                    if target_block > block_number.saturating_add(2) {
                        return Err(format!(
                            "encrypted v2 target {target_block} exceeds +2 lookahead from block {block_number}",
                        ));
                    }
                    pending_queue_len = pending_queue_len.saturating_add(1);
                }
                MevShieldExtrinsicClass::SubmitBlockDecryptionKey {
                    target_block,
                    finalized_ordering_block_number,
                    finalized_ordering_block_hash,
                    ..
                } => {
                    if finalized_ordering_block_number < target_block {
                        return Err(format!(
                            "decryption key finality point {finalized_ordering_block_number} precedes target {target_block}",
                        ));
                    }
                    if !self.canonical_finalized_contains(
                        finalized_ordering_block_number,
                        finalized_ordering_block_hash.into(),
                    )? {
                        return Err(format!(
                            "decryption key for target {target_block} accepted before local finality of {finalized_ordering_block_number}",
                        ));
                    }
                }
                MevShieldExtrinsicClass::Operational => {}
                MevShieldExtrinsicClass::UnencryptedNonOperational => {
                    if pending_queue_len > 0 {
                        return Err(
                            "plaintext non-operational extrinsic while encrypted queue is pending"
                                .into(),
                        );
                    }
                }
            }
        }

        if composition.is_full()
            && composition.pending_queue_len_at_parent > 0
            && composition.contains_plaintext_non_operational
            && !composition.contains_encrypted_v2
        {
            return Err(
                "full block uses plaintext non-operational space while encrypted queue is pending"
                    .into(),
            );
        }

        Ok(())
    }
}

#[async_trait]
impl<I, C, B> BlockImport<B> for MevShieldBlockImport<I, C, B>
where
    I: BlockImport<B> + Send + Sync,
    I::Error: StdError + Send + From<ConsensusError> + 'static,
    C: ProvideRuntimeApi<B> + HeaderBackend<B> + BlockBackend<B> + Send + Sync + 'static,
    C::Api: MevShieldIbeApi<B>,
    B: BlockT,
    B::Hash: From<H256>,
{
    type Error = I::Error;

    async fn check_block(&self, block: BlockCheckParams<B>) -> Result<ImportResult, Self::Error> {
        self.inner.check_block(block).await
    }

    async fn import_block(
        &self,
        params: BlockImportParams<B>,
    ) -> Result<ImportResult, Self::Error> {
        if let Some(body) = params.body.as_ref() {
            let block = B::new(params.header.clone(), body.clone());
            let parent_hash = *block.header().parent_hash();
            self.verify_mev_shield_block(parent_hash, &block)
                .map_err(|reason| ConsensusError::ClientImport(reason).into())?;
        }
        self.inner.import_block(params).await
    }
}
