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
use std::{collections::BTreeSet, error::Error as StdError, marker::PhantomData, sync::Arc};
const IBE_TARGET_LOOKAHEAD_BLOCKS: u64 = 2;

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

    fn verify_decryption_key_finality(
        &self,
        target_block: u64,
        finalized_ordering_block_number: u64,
        finalized_ordering_block_hash: B::Hash,
    ) -> Result<(), String> {
        let expected_finalized = target_block.saturating_sub(1);
        if finalized_ordering_block_number != expected_finalized {
            return Err(format!(
                "decryption key finality point {finalized_ordering_block_number} does not match target-1 {expected_finalized} for target {target_block}",
            ));
        }
        if !self.canonical_finalized_contains(
            finalized_ordering_block_number,
            finalized_ordering_block_hash,
        )? {
            return Err(format!(
                "decryption key for target {target_block} accepted before local finality of {finalized_ordering_block_number}",
            ));
        }
        Ok(())
    }

    fn has_due_ibe_queue_head_without_available_key(
        &self,
        parent_hash: B::Hash,
        block_number: u64,
        inherent_key_targets: &BTreeSet<u64>,
    ) -> Result<bool, String> {
        let api = self.client.runtime_api();
        let pending_len = api
            .pending_encrypted_queue_len(parent_hash)
            .map_err(|e| format!("pending_encrypted_queue_len runtime API failed: {e:?}"))?;
        if pending_len == 0 {
            return Ok(false);
        }

        let identities = api
            .pending_ibe_identities(parent_hash, pending_len)
            .map_err(|e| format!("pending_ibe_identities runtime API failed: {e:?}"))?;

        for identity in identities {
            if identity.target_block > block_number {
                return Ok(false);
            }

            let key_available_at_parent = api
                .has_ibe_block_key(
                    parent_hash,
                    identity.epoch,
                    identity.target_block,
                    identity.key_id,
                )
                .map_err(|e| format!("has_ibe_block_key runtime API failed: {e:?}"))?;

            if key_available_at_parent || inherent_key_targets.contains(&identity.target_block) {
                continue;
            }

            return Ok(true);
        }

        Ok(false)
    }

    fn verify_mev_shield_block(&self, parent_hash: B::Hash, block: &B) -> Result<(), String> {
        let api = self.client.runtime_api();
        let encoded = block
            .extrinsics()
            .iter()
            .map(|xt| xt.encode())
            .collect::<Vec<_>>();

        let block_number = Self::block_number_u64(block.header())?;
        let mut inherent_key_targets = BTreeSet::new();

        for xt in encoded {
            let class = api
                .classify_extrinsic(parent_hash, xt)
                .map_err(|e| format!("classify_extrinsic runtime API failed: {e:?}"))?;

            match class {
                MevShieldExtrinsicClass::SubmitEncryptedV2 { target_block, .. } => {
                    let min_target = block_number.saturating_add(1);
                    let max_target = block_number.saturating_add(IBE_TARGET_LOOKAHEAD_BLOCKS);
                    if target_block < min_target || target_block > max_target {
                        return Err(format!(
                            "encrypted v2 target {target_block} must be in ({block_number}, {max_target}]",
                        ));
                    }
                }
                MevShieldExtrinsicClass::SubmitBlockDecryptionKey {
                    target_block,
                    finalized_ordering_block_number,
                    finalized_ordering_block_hash,
                    ..
                } => {
                    self.verify_decryption_key_finality(
                        target_block,
                        finalized_ordering_block_number,
                        finalized_ordering_block_hash.into(),
                    )?;
                }
                MevShieldExtrinsicClass::SubmitBlockDecryptionKeyInherent {
                    finality_proofs,
                    invalid_key_count,
                } => {
                    if invalid_key_count > 0 {
                        return Err(format!(
                            "IBE block-key inherent contains {invalid_key_count} invalid key(s)",
                        ));
                    }
                    for (
                        target_block,
                        finalized_ordering_block_number,
                        finalized_ordering_block_hash,
                    ) in finality_proofs
                    {
                        self.verify_decryption_key_finality(
                            target_block,
                            finalized_ordering_block_number,
                            finalized_ordering_block_hash.into(),
                        )?;
                        inherent_key_targets.insert(target_block);
                    }
                }
                MevShieldExtrinsicClass::Operational => {}
                MevShieldExtrinsicClass::UnencryptedNonOperational => {
                    if self.has_due_ibe_queue_head_without_available_key(
                        parent_hash,
                        block_number,
                        &inherent_key_targets,
                    )? {
                        return Err(
                        "plaintext non-operational extrinsic while due encrypted queue head lacks a block key"
                            .into(),
                    );
                    }
                }
            }
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
