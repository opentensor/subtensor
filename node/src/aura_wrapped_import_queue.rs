use babe_primitives::BABE_ENGINE_ID;
use futures::future::pending;
use jsonrpsee::tokio::sync::Mutex;
use jsonrpsee::tokio::sync::oneshot;
use log;
use sc_client_api::AuxStore;
use sc_client_api::BlockOf;
use sc_client_api::UsageProvider;
use sc_consensus::BlockImport;
use sc_consensus::BlockImportParams;
use sc_consensus::Verifier;
use sc_consensus::{BasicQueue, DefaultImportQueue};
use sc_consensus_aura::AuraVerifier;
use sc_consensus_aura::AuthorityId;
use sc_consensus_aura::CheckForEquivocation;
use sc_consensus_aura::ImportQueueParams;
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use scale_codec::Codec;
use sp_api::ApiExt;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::error::Error as ConsensusError;
use sp_consensus_aura::AuraApi;
use sp_core::Pair;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::Digest;
use sp_runtime::DigestItem;
use sp_runtime::traits::NumberFor;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use std::fmt::Debug;
use std::sync::Arc;

/// A wrapped Aura verifier which will SIGINT when it detects a Babe block, to
/// allow seemless syncing of a chain that has undergone an Aura to Babe upgrade.
///
/// TODO: More graceful signal than SIGINT
///
/// TODO: Handle also signalling when a Babe runtime upgrade is detected (before first Babe block is
/// imported)
struct AuraWrappedVerifier<B, C, P, CIDP, N> {
    inner: AuraVerifier<C, P, CIDP, N>,
    babe_switch_tx: Mutex<Option<oneshot::Sender<()>>>,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: BlockT, C, P, CIDP, N> AuraWrappedVerifier<B, C, P, CIDP, N> {
    pub fn new(
        client: Arc<C>,
        create_inherent_data_providers: CIDP,
        telemetry: Option<TelemetryHandle>,
        check_for_equivocation: CheckForEquivocation,
        compatibility_mode: sc_consensus_aura::CompatibilityMode<N>,
        babe_switch_tx: oneshot::Sender<()>,
    ) -> Self {
        let verifier_params = sc_consensus_aura::BuildVerifierParams::<C, CIDP, _> {
            client,
            create_inherent_data_providers,
            telemetry,
            check_for_equivocation,
            compatibility_mode,
        };
        let verifier = sc_consensus_aura::build_verifier::<P, C, CIDP, N>(verifier_params);

        AuraWrappedVerifier {
            inner: verifier,
            babe_switch_tx: Mutex::new(Some(babe_switch_tx)),
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<B: BlockT, C, P, CIDP> Verifier<B> for AuraWrappedVerifier<B, C, P, CIDP, NumberFor<B>>
where
    C: ProvideRuntimeApi<B> + Send + Sync + sc_client_api::backend::AuxStore,
    C::Api: BlockBuilderApi<B> + AuraApi<B, AuthorityId<P>> + ApiExt<B>,
    P: Pair,
    P::Public: Codec + Debug,
    P::Signature: Codec,
    CIDP: CreateInherentDataProviders<B, ()> + Send + Sync,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
    async fn verify(&self, block: BlockImportParams<B>) -> Result<BlockImportParams<B>, String> {
        let number: NumberFor<B> = block.post_header().number().clone();
        log::debug!("Verifying block: {:?}", number);
        // Here is the trick: check if the block bring verified is .
        if is_babe_digest(block.header.digest()) {
            log::info!("Detected Babe block, sending signal to switch to Babe-based processing.");
            self.babe_switch_tx
                .lock()
                .await
                .take()
                .unwrap()
                .send(())
                .expect("Failed to send Babe switch signal; receiver should be ready.");
            pending::<()>().await;
            unreachable!("Should not reach here, pending forever.");
        } else {
            self.inner.verify(block).await
        }
    }
}

/// Start an import queue for the Aura consensus algorithm.
pub fn import_queue<P, B, I, C, S, CIDP>(
    (
        ImportQueueParams {
            block_import,
            justification_import,
            client,
            create_inherent_data_providers,
            spawner,
            registry,
            check_for_equivocation,
            telemetry,
            compatibility_mode,
        },
        babe_switch_tx,
    ): (ImportQueueParams<B, I, C, S, CIDP>, oneshot::Sender<()>),
) -> Result<DefaultImportQueue<B>, sp_consensus::Error>
where
    B: BlockT,
    C::Api: BlockBuilderApi<B> + AuraApi<B, AuthorityId<P>> + ApiExt<B>,
    C: 'static
        + ProvideRuntimeApi<B>
        + BlockOf
        + Send
        + Sync
        + AuxStore
        + UsageProvider<B>
        + HeaderBackend<B>,
    I: BlockImport<B, Error = ConsensusError> + Send + Sync + 'static,
    P: Pair + 'static,
    P::Public: Codec + Debug,
    P::Signature: Codec,
    S: sp_core::traits::SpawnEssentialNamed,
    CIDP: CreateInherentDataProviders<B, ()> + Sync + Send + 'static,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
    let verifier = AuraWrappedVerifier::<B, C, P, CIDP, NumberFor<B>>::new(
        client,
        create_inherent_data_providers,
        telemetry,
        check_for_equivocation,
        compatibility_mode,
        babe_switch_tx,
    );

    Ok(BasicQueue::new(
        verifier,
        Box::new(block_import),
        justification_import,
        spawner,
        registry,
    ))
}

fn is_babe_digest(digest: &Digest) -> bool {
    digest
        .log(|d| match d {
            DigestItem::PreRuntime(engine_id, _) => {
                if engine_id.clone() == BABE_ENGINE_ID {
                    Some(d)
                } else {
                    None
                }
            }
            _ => None,
        })
        .is_some()
}
