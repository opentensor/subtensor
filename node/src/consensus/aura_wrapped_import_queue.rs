use sc_client_api::AuxStore;
use sc_client_api::BlockOf;
use sc_client_api::UsageProvider;
use sc_consensus::BlockImport;
use sc_consensus::BlockImportParams;
use sc_consensus::Verifier;
use sc_consensus::{BasicQueue, DefaultImportQueue};
use sc_consensus_aura::AuraVerifier;
use sc_consensus_aura::CheckForEquivocation;
use sc_consensus_aura::ImportQueueParams;
use sc_consensus_babe::CompatibleDigestItem as _;
use futures::future::pending;
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use sp_api::ApiExt;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::HeaderBackend;
use sp_blockchain::HeaderMetadata;
use sp_consensus::error::Error as ConsensusError;
use sp_consensus_aura::AuraApi;
use sp_consensus_aura::sr25519::AuthorityId as AuraAuthorityId;
use sp_consensus_aura::sr25519::AuthorityPair as AuraAuthorityPair;
use sp_consensus_babe::AuthorityId as BabeAuthorityId;
use sp_consensus_babe::AuthorityPair as BabeAuthorityPair;
use sp_consensus_babe::BABE_ENGINE_ID;
use sp_core::Pair;
use sp_core::crypto::Ss58Codec;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::Digest;
use sp_runtime::DigestItem;
use sp_runtime::traits::NumberFor;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use std::sync::Arc;

/// A wrapped Aura verifier which will stall verification if it encounters a
/// Babe block, rather than error out.
///
/// This is required to prevent rapid validation failure and subsequent
/// re-fetching of the same block from peers, which triggers the peers to
/// blacklist the offending node and refuse to connect with them until they
/// are restarted
struct AuraWrappedVerifier<B, C, CIDP, N> {
    inner: AuraVerifier<C, AuraAuthorityPair, CIDP, N>,
    client: Arc<C>,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: BlockT, C, CIDP, N> AuraWrappedVerifier<B, C, CIDP, N>
where
    CIDP: CreateInherentDataProviders<B, ()> + Send + Sync,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
    C: ProvideRuntimeApi<B> + Send + Sync + sc_client_api::backend::AuxStore,
    C::Api: BlockBuilderApi<B> + AuraApi<B, AuraAuthorityId> + ApiExt<B>,
    C: HeaderBackend<B> + HeaderMetadata<B>,
{
    pub fn new(
        client: Arc<C>,
        create_inherent_data_providers: CIDP,
        telemetry: Option<TelemetryHandle>,
        check_for_equivocation: CheckForEquivocation,
        compatibility_mode: sc_consensus_aura::CompatibilityMode<N>,
    ) -> Self {
        let verifier_params = sc_consensus_aura::BuildVerifierParams::<C, CIDP, _> {
            client: client.clone(),
            create_inherent_data_providers,
            telemetry,
            check_for_equivocation,
            compatibility_mode,
        };
        let verifier =
            sc_consensus_aura::build_verifier::<AuraAuthorityPair, C, CIDP, N>(verifier_params);

        AuraWrappedVerifier {
            inner: verifier,
            client,
            _phantom: std::marker::PhantomData,
        }
    }

    /// When a Babe block is encountered in Aura mode, we need to check it is legitimate
    /// before switching to the Babe service.
    ///
    /// We can't use a full [`BabeVerifier`] because we don't have a Babe link running, however we
    /// can check that the block author is one of the authorities from the last verified Aura block.
    ///
    /// The Babe block will be verified in full after the node spins back up as a Babe service.
    async fn check_babe_block(&self, block: BlockImportParams<B>) -> Result<(), String> {
        log::info!(
            "Checking Babe block {:?} is legitimate",
            block.post_header().hash()
        );
        let mut header = block.header.clone();
        let seal = header
            .digest_mut()
            .pop()
            .ok_or_else(|| "Header Unsealed".to_string())?;
        let sig = seal
            .as_babe_seal()
            .ok_or_else(|| "Header bad seal".to_string())?;

        let authorities = self.get_last_aura_authorities(block.header)?;
        if let Some(a) = authorities.into_iter().find(|a| {
            let babe_key = BabeAuthorityId::from(a.clone().into_inner());
            BabeAuthorityPair::verify(&sig, header.hash(), &babe_key)
        }) {
            log::info!(
                "Babe block has a valid signature by author: {}",
                a.to_ss58check()
            );
            Ok(())
        } else {
            Err("Babe block has a bad signature. Rejecting.".to_string())
        }
    }

    /// Given the hash of the first Babe block mined, get the Aura authorities that existed prior to
    /// the runtime upgrade.
    ///
    /// Note: We need get the Aura authorities from grandparent rather than the parent,
    /// because the runtime upgrade clearing the Aura authorities occurs in the parent.
    fn get_last_aura_authorities(
        &self,
        first_babe_block_header: B::Header,
    ) -> Result<Vec<AuraAuthorityId>, String> {
        let parent_header = self
            .client
            .header(*first_babe_block_header.parent_hash())
            .map_err(|e| format!("Failed to get parent header: {}", e))?
            .ok_or("Parent header not found".to_string())?;
        let grandparent_hash = parent_header.parent_hash();

        let runtime_api = self.client.runtime_api();
        let authorities = runtime_api
            .authorities(*grandparent_hash)
            .map_err(|e| format!("Failed to get Aura authorities: {}", e))?;

        Ok(authorities)
    }
}

#[async_trait::async_trait]
impl<B: BlockT, C, CIDP> Verifier<B> for AuraWrappedVerifier<B, C, CIDP, NumberFor<B>>
where
    C: ProvideRuntimeApi<B> + Send + Sync + sc_client_api::backend::AuxStore,
    C::Api: BlockBuilderApi<B> + AuraApi<B, AuraAuthorityId> + ApiExt<B>,
    C: HeaderBackend<B> + HeaderMetadata<B>,
    CIDP: CreateInherentDataProviders<B, ()> + Send + Sync,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
    async fn verify(&self, block: BlockImportParams<B>) -> Result<BlockImportParams<B>, String> {
        let number: NumberFor<B> = *block.post_header().number();
        log::debug!("Verifying block: {:?}", number);
        if is_babe_digest(block.header.digest()) {
            self.check_babe_block(block).await?;
            log::debug!(
                "Detected Babe block! Verifier cannot continue, upgrade must be triggered elsewhere..."
            );
            pending::<()>().await;
            unreachable!("Should not reach here, pending forever.");
        } else {
            self.inner.verify(block).await
        }
    }
}

/// Start an import queue for the Aura consensus algorithm.
pub fn import_queue<B, I, C, S, CIDP>(
    params: ImportQueueParams<B, I, C, S, CIDP>,
) -> Result<DefaultImportQueue<B>, sp_consensus::Error>
where
    B: BlockT,
    C::Api: BlockBuilderApi<B> + AuraApi<B, AuraAuthorityId> + ApiExt<B>,
    C: 'static
        + ProvideRuntimeApi<B>
        + BlockOf
        + Send
        + Sync
        + AuxStore
        + UsageProvider<B>
        + HeaderBackend<B>
        + HeaderMetadata<B>,
    I: BlockImport<B, Error = ConsensusError> + Send + Sync + 'static,
    S: sp_core::traits::SpawnEssentialNamed,
    CIDP: CreateInherentDataProviders<B, ()> + Sync + Send + 'static,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
    let verifier = AuraWrappedVerifier::<B, C, CIDP, NumberFor<B>>::new(
        params.client,
        params.create_inherent_data_providers,
        params.telemetry,
        params.check_for_equivocation,
        params.compatibility_mode,
    );

    Ok(BasicQueue::new(
        verifier,
        Box::new(params.block_import),
        params.justification_import,
        params.spawner,
        params.registry,
    ))
}

fn is_babe_digest(digest: &Digest) -> bool {
    digest
        .logs()
        .iter()
        .any(|d| matches!(d, DigestItem::PreRuntime(engine_id, _) if engine_id == &BABE_ENGINE_ID))
}
