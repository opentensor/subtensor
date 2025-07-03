use babe_primitives::BabeApi;
use babe_primitives::BabeConfiguration;
use futures::{channel::mpsc::channel, prelude::*};
use log;
use sc_client_api::AuxStore;
use sc_consensus::BlockImport;
use sc_consensus::BlockImportParams;
use sc_consensus::BoxJustificationImport;
use sc_consensus::Verifier;
use sc_consensus::{BasicQueue, DefaultImportQueue};
use sc_consensus_aura::AuraVerifier;
use sc_consensus_aura::AuthorityId;
use sc_consensus_aura::CheckForEquivocation;
use sc_consensus_babe::BabeLink;
use sc_consensus_babe::BabeVerifier;
use sc_consensus_babe::BabeWorkerHandle;
use sc_consensus_epochs::SharedEpochChanges;
use sc_consensus_slots::InherentDataProviderExt;
use sc_telemetry::TelemetryHandle;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ApiExt;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::{HeaderBackend, HeaderMetadata, Result as ClientResult};
use sp_consensus::error::Error as ConsensusError;
use sp_consensus_aura::AURA_ENGINE_ID;
use sp_consensus_aura::AuraApi;
use sp_consensus_aura::sr25519::AuthorityPair;
use sp_core::traits::SpawnEssentialNamed;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::NumberFor;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use std::sync::Arc;
use substrate_prometheus_endpoint::Registry;

/// Parameters passed to [`import_queue`].
pub struct ImportQueueParams<'a, Block: BlockT, BI, Client, CIDP, SelectChain, Spawn> {
    /// The BABE link that is created by [`block_import`].
    pub link: BabeLink<Block>,
    /// The block import that should be wrapped.
    pub block_import: BI,
    /// Optional justification import.
    pub justification_import: Option<BoxJustificationImport<Block>>,
    /// The client to interact with the internals of the node.
    pub client: Arc<Client>,
    /// A [`SelectChain`] implementation.
    ///
    /// Used to determine the best block that should be used as basis when sending an equivocation
    /// report.
    pub select_chain: SelectChain,
    /// Used to crate the inherent data providers.
    ///
    /// These inherent data providers are then used to create the inherent data that is
    /// passed to the `check_inherents` runtime call.
    pub create_inherent_data_providers: CIDP,
    /// Spawner for spawning futures.
    pub spawner: &'a Spawn,
    /// Registry for prometheus metrics.
    pub registry: Option<&'a Registry>,
    /// Optional telemetry handle to report telemetry events.
    pub telemetry: Option<TelemetryHandle>,
    /// The offchain transaction pool factory.
    ///
    /// Will be used when sending equivocation reports.
    pub offchain_tx_pool_factory: OffchainTransactionPoolFactory<Block>,
    /// Should we check for equivocation? (Aura)
    pub check_for_equivocation: CheckForEquivocation,
}

/// A dynamic verifier that can verify both Aura and Babe blocks.
struct AuraBabeVerifier<Block: BlockT, Client, SelectChain, CIDP> {
    babe_verifier: BabeVerifier<Block, Client, SelectChain, CIDP>,
    aura_verifier: AuraVerifier<Client, AuthorityPair, CIDP, NumberFor<Block>>,
}

impl<Block, Client, SelectChain, CIDP> AuraBabeVerifier<Block, Client, SelectChain, CIDP>
where
    Block: BlockT,
    Client: HeaderMetadata<Block, Error = sp_blockchain::Error>
        + HeaderBackend<Block>
        + ProvideRuntimeApi<Block>
        + Send
        + Sync
        + AuxStore,
    Client::Api:
        BlockBuilderApi<Block> + BabeApi<Block> + AuraApi<Block, AuthorityId<AuthorityPair>>,
    SelectChain: sp_consensus::SelectChain<Block>,
    CIDP: CreateInherentDataProviders<Block, ()> + Send + Sync + Clone,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
    pub fn new(
        client: Arc<Client>,
        select_chain: SelectChain,
        create_inherent_data_providers: CIDP,
        babe_config: BabeConfiguration,
        epoch_changes: SharedEpochChanges<Block, sc_consensus_babe::Epoch>,
        telemetry: Option<TelemetryHandle>,
        offchain_tx_pool_factory: OffchainTransactionPoolFactory<Block>,
    ) -> Self {
        let babe_verifier = BabeVerifier::new(
            client.clone(),
            select_chain,
            create_inherent_data_providers.clone(),
            babe_config,
            epoch_changes,
            telemetry.clone(),
            offchain_tx_pool_factory,
        );

        let aura_verifier_params = sc_consensus_aura::BuildVerifierParams::<Client, CIDP, _> {
            client,
            create_inherent_data_providers,
            telemetry,
            check_for_equivocation: CheckForEquivocation::Yes,
            compatibility_mode: sc_consensus_aura::CompatibilityMode::<NumberFor<Block>>::None,
        };
        let aura_verifier =
            sc_consensus_aura::build_verifier::<AuthorityPair, _, _, _>(aura_verifier_params);

        AuraBabeVerifier {
            babe_verifier,
            aura_verifier,
        }
    }
}

#[async_trait::async_trait]
impl<Block, Client, SelectChain, CIDP> Verifier<Block>
    for AuraBabeVerifier<Block, Client, SelectChain, CIDP>
where
    Block: BlockT,
    Client: HeaderMetadata<Block, Error = sp_blockchain::Error>
        + HeaderBackend<Block>
        + ProvideRuntimeApi<Block>
        + Send
        + Sync
        + AuxStore,
    Client::Api:
        BlockBuilderApi<Block> + BabeApi<Block> + AuraApi<Block, AuthorityId<AuthorityPair>>,
    SelectChain: sp_consensus::SelectChain<Block>,
    CIDP: CreateInherentDataProviders<Block, ()> + Send + Sync + Clone,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
{
    async fn verify(
        &self,
        block: BlockImportParams<Block>,
    ) -> Result<BlockImportParams<Block>, String> {
        log::info!("Block: {:?}", &block);
        let number: NumberFor<Block> = block.post_header().number().clone();
        log::info!("Verifying block: {:?}", number);
        let consensus_engine_id = crate::common::block_consensus_engine_id(&block);
        if consensus_engine_id == AURA_ENGINE_ID {
            log::info!("Using Aura Verifier.");
            self.aura_verifier.verify(block).await
            // panic!("succeeded aura verify!");
        } else {
            log::info!("Using Babe Verifier.");
            self.babe_verifier.verify(block).await
            // panic!("succeeded babe verify!");
        }
    }
}

/// Start an import queue which verifies both Aura and Babe blocks.
pub fn import_queue<Block: BlockT, Client, SelectChain, BI, CIDP, Spawn>(
    ImportQueueParams {
        block_import,
        justification_import,
        client,
        create_inherent_data_providers,
        spawner,
        registry,
        check_for_equivocation,
        telemetry,
        link: babe_link,
        select_chain,
        offchain_tx_pool_factory,
    }: ImportQueueParams<'_, Block, BI, Client, CIDP, SelectChain, Spawn>,
) -> ClientResult<(DefaultImportQueue<Block>, BabeWorkerHandle<Block>)>
where
    BI: BlockImport<Block, Error = ConsensusError> + Send + Sync + 'static,
    Client: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + HeaderMetadata<Block, Error = sp_blockchain::Error>
        + AuxStore
        + Send
        + Sync
        + 'static,
    Client::Api: BlockBuilderApi<Block>
        + BabeApi<Block>
        + AuraApi<Block, AuthorityId<AuthorityPair>>
        + ApiExt<Block>,
    SelectChain: sp_consensus::SelectChain<Block> + 'static,
    CIDP: CreateInherentDataProviders<Block, ()> + Send + Sync + 'static + Clone,
    CIDP::InherentDataProviders: InherentDataProviderExt + Send + Sync,
    Spawn: SpawnEssentialNamed,
{
    const HANDLE_BUFFER_SIZE: usize = 1024;

    let verifier = AuraBabeVerifier::new(
        client.clone(),
        select_chain,
        create_inherent_data_providers,
        babe_link.config().clone(),
        babe_link.epoch_changes().clone(),
        telemetry,
        offchain_tx_pool_factory,
    );

    let (worker_tx, worker_rx) = channel(HANDLE_BUFFER_SIZE);

    let answer_requests = sc_consensus_babe::answer_requests(
        worker_rx,
        babe_link.config().clone(),
        client,
        babe_link.epoch_changes().clone(),
    );

    spawner.spawn_essential("babe-worker", Some("babe"), answer_requests.boxed());

    Ok((
        BasicQueue::new(
            verifier,
            Box::new(block_import),
            justification_import,
            spawner,
            registry,
        ),
        BabeWorkerHandle::new(worker_tx),
    ))
}
