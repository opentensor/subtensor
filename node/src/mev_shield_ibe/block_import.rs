use async_trait::async_trait;
use codec::{Decode, Encode};
use mev_shield_ibe_runtime_api::{MevShieldExtrinsicClass, MevShieldIbeApi};
use sc_client_api::BlockBackend;
use sc_consensus::{BlockCheckParams, BlockImport, BlockImportParams, ImportResult};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::Error as ConsensusError;
use sp_core::H256;
use sp_runtime::{
    DigestItem,
    traits::{Block as BlockT, Header as HeaderT},
};
use std::{collections::BTreeSet, error::Error as StdError, marker::PhantomData, sync::Arc};
use stp_mev_shield_ibe::{
    IBE_BLOCK_DECRYPTION_KEYS_ENGINE_ID, IbeBlockDecryptionKeyPreRuntimeDigestData, KEY_ID_LEN,
};

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
    B::Hash: From<H256> + Copy,
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

    fn accept_block_key_payload_class(
        &self,
        block_number: u64,
        payload: &[u8],
        class: MevShieldExtrinsicClass,
        preruntime_key_identities: &mut BTreeSet<(u64, u64, [u8; KEY_ID_LEN])>,
    ) -> Result<(), String> {
        let data = IbeBlockDecryptionKeyPreRuntimeDigestData::decode(&mut &payload[..])
            .map_err(|_| "IBE pre-runtime digest payload failed to decode".to_string())?;

        let MevShieldExtrinsicClass::SubmitBlockDecryptionKeyInherent {
            finality_proofs,
            invalid_key_count,
        } = class
        else {
            return Err("IBE pre-runtime digest did not classify as block-key material".into());
        };

        if invalid_key_count > 0 {
            return Err(format!(
                "IBE pre-runtime digest contains {invalid_key_count} invalid key bundle(s)",
            ));
        }

        if finality_proofs.len() != data.share_bundles.len() {
            return Err(format!(
                "IBE pre-runtime digest validation mismatch: {} proof(s) for {} bundle(s)",
                finality_proofs.len(),
                data.share_bundles.len(),
            ));
        }

        for (
            bundle,
            (target_block, finalized_ordering_block_number, finalized_ordering_block_hash),
        ) in data.share_bundles.iter().zip(finality_proofs.iter())
        {
            let key = &bundle.key;
            if *target_block != key.target_block
                || *finalized_ordering_block_number != key.finalized_ordering_block_number
                || *finalized_ordering_block_hash != key.finalized_ordering_block_hash
            {
                return Err("IBE pre-runtime digest proof does not match bundle key".into());
            }

            if key.target_block != block_number {
                return Err(format!(
                    "IBE pre-runtime digest target {} does not match block {block_number}",
                    key.target_block,
                ));
            }

            self.verify_decryption_key_finality(
                key.target_block,
                key.finalized_ordering_block_number,
                key.finalized_ordering_block_hash.into(),
            )?;

            preruntime_key_identities.insert((key.epoch, key.target_block, key.key_id));
        }

        Ok(())
    }

    fn has_due_ibe_queue_head_without_available_key(
        &self,
        parent_hash: B::Hash,
        block_number: u64,
        preruntime_key_identities: &BTreeSet<(u64, u64, [u8; KEY_ID_LEN])>,
    ) -> Result<bool, String> {
        let api = self.client.runtime_api();
        let Some(identity) = api
            .due_ibe_queue_head(parent_hash, block_number)
            .map_err(|e| format!("due_ibe_queue_head runtime API failed: {e:?}"))?
        else {
            return Ok(false);
        };

        let key_available_at_parent = api
            .has_ibe_block_key(
                parent_hash,
                identity.epoch,
                identity.target_block,
                identity.key_id,
            )
            .map_err(|e| format!("has_ibe_block_key runtime API failed: {e:?}"))?;

        Ok(!key_available_at_parent
            && !preruntime_key_identities.contains(&(
                identity.epoch,
                identity.target_block,
                identity.key_id,
            )))
    }

    fn verify_mev_shield_block(&self, parent_hash: B::Hash, block: &B) -> Result<(), String> {
        let api = self.client.runtime_api();
        let block_number = Self::block_number_u64(block.header())?;
        let mut preruntime_key_identities = BTreeSet::new();

        for log in block.header().digest().logs().iter() {
            let DigestItem::PreRuntime(engine_id, payload) = log else {
                continue;
            };
            if engine_id != &IBE_BLOCK_DECRYPTION_KEYS_ENGINE_ID {
                continue;
            }
            let class = api
                .classify_ibe_block_key_preruntime_digest(parent_hash, payload.clone())
                .map_err(|e| {
                    format!("classify_ibe_block_key_preruntime_digest runtime API failed: {e:?}")
                })?;
            self.accept_block_key_payload_class(
                block_number,
                payload.as_slice(),
                class,
                &mut preruntime_key_identities,
            )?;
        }

        if self.has_due_ibe_queue_head_without_available_key(
            parent_hash,
            block_number,
            &preruntime_key_identities,
        )? {
            return Err(
                "due encrypted queue head requires a valid threshold-IBE pre-runtime key bundle"
                    .into(),
            );
        }

        let encoded = block
            .extrinsics()
            .iter()
            .map(|xt| xt.encode())
            .collect::<Vec<_>>();

        for xt in encoded {
            let class = api
                .classify_extrinsic(parent_hash, xt)
                .map_err(|e| format!("classify_extrinsic runtime API failed: {e:?}"))?;
            match class {
                MevShieldExtrinsicClass::SubmitEncryptedV2 { target_block, .. } => {
                    let expected = block_number.saturating_add(IBE_TARGET_LOOKAHEAD_BLOCKS);
                    if target_block != expected {
                        return Err(format!(
                            "encrypted v2 target {target_block} must equal block {block_number} + {IBE_TARGET_LOOKAHEAD_BLOCKS} ({expected})",
                        ));
                    }
                }
                MevShieldExtrinsicClass::SubmitBlockDecryptionKeyInherent { .. } => {
                    return Err(
                        "threshold-IBE block-key inherent extrinsic is obsolete; use the pre-runtime digest"
                            .into(),
                    );
                }
                MevShieldExtrinsicClass::Operational => {}
                MevShieldExtrinsicClass::UnencryptedNonOperational => {
                    if self.has_due_ibe_queue_head_without_available_key(
                        parent_hash,
                        block_number,
                        &preruntime_key_identities,
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
    B::Hash: From<H256> + Copy,
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
