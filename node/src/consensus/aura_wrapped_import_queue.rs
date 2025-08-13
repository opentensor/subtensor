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
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use sp_api::ApiExt;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::error::Error as ConsensusError;
use sp_consensus_aura::AuraApi;
use sp_consensus_aura::sr25519::AuthorityId;
use sp_consensus_aura::sr25519::AuthorityPair;
use sp_consensus_babe::BABE_ENGINE_ID;
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
    inner: AuraVerifier<C, AuthorityPair, CIDP, N>,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: BlockT, C, CIDP, N> AuraWrappedVerifier<B, C, CIDP, N> {
    pub fn new(
        client: Arc<C>,
        create_inherent_data_providers: CIDP,
        telemetry: Option<TelemetryHandle>,
        check_for_equivocation: CheckForEquivocation,
        compatibility_mode: sc_consensus_aura::CompatibilityMode<N>,
    ) -> Self {
        let verifier_params = sc_consensus_aura::BuildVerifierParams::<C, CIDP, _> {
            client,
            create_inherent_data_providers,
            telemetry,
            check_for_equivocation,
            compatibility_mode,
        };
        let verifier =
            sc_consensus_aura::build_verifier::<AuthorityPair, C, CIDP, N>(verifier_params);

        AuraWrappedVerifier {
            inner: verifier,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<B: BlockT, C, CIDP> Verifier<B> for AuraWrappedVerifier<B, C, CIDP, NumberFor<B>>
where
    C: ProvideRuntimeApi<B> + Send + Sync + sc_client_api::backend::AuxStore,
    C::Api: BlockBuilderApi<B> + AuraApi<B, AuthorityId> + ApiExt<B>,
    CIDP: CreateInherentDataProviders<B, ()> + Send + Sync,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
    async fn verify(&self, block: BlockImportParams<B>) -> Result<BlockImportParams<B>, String> {
        let number: NumberFor<B> = *block.post_header().number();
        log::debug!("Verifying block: {:?}", number);
        if is_babe_digest(block.header.digest()) {
            // TODO: Use a BabeVerifier to verify Babe blocks. This will
            // prevent rapid validation failure and subsequent re-fetching
            // of the same block from peers, which triggers the peers to
            // blacklist the offending node and refuse to connect with them until they
            // are restarted.
            //
            // Unfortunately, BabeVerifier construction logic is NOT public outside of
            // its crate in vanilla Polkadot SDK, so we are unable to use it until we
            // migrate to our Polkadot SDK fork.
            self.inner.verify(block).await
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
    C::Api: BlockBuilderApi<B> + AuraApi<B, AuthorityId> + ApiExt<B>,
    C: 'static
        + ProvideRuntimeApi<B>
        + BlockOf
        + Send
        + Sync
        + AuxStore
        + UsageProvider<B>
        + HeaderBackend<B>,
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
