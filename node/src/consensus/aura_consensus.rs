use crate::consensus::{ConsensusMechanism, StartAuthoringParams};
use crate::{
    client::{FullBackend, FullClient},
    conditional_evm_block_import::ConditionalEVMBlockImport,
    ethereum::EthConfiguration,
    service::{BIQ, FullSelectChain, GrandpaBlockImport},
};
use fc_consensus::FrontierBlockImport;
use jsonrpsee::tokio;
use node_subtensor_runtime::opaque::Block;
use sc_client_api::{AuxStore, BlockOf};
use sc_consensus::{BlockImport, BoxBlockImport};
use sc_consensus_grandpa::BlockNumberOps;
use sc_consensus_slots::{BackoffAuthoringBlocksStrategy, InherentDataProviderExt};
use sc_service::{Configuration, TaskManager};
use sc_telemetry::TelemetryHandle;
use sc_transaction_pool::TransactionPoolHandle;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{HeaderBackend, HeaderMetadata};
use sp_consensus::{Environment, Proposer, SelectChain, SyncOracle};
use sp_consensus_aura::sr25519::AuthorityId;
use sp_consensus_aura::{AuraApi, sr25519::AuthorityPair as AuraPair};
use sp_consensus_slots::SlotDuration;
use sp_inherents::CreateInherentDataProviders;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::NumberFor;
use std::{error::Error, sync::Arc};

pub struct AuraConsensus;

impl ConsensusMechanism for AuraConsensus {
    type InherentDataProviders = (
        sp_consensus_aura::inherents::InherentDataProvider,
        sp_timestamp::InherentDataProvider,
    );

    fn start_authoring<C, SC, I, PF, SO, L, CIDP, BS, Error>(
        self,
        task_manager: &mut TaskManager,
        params: StartAuthoringParams<C, SC, I, PF, SO, L, CIDP, BS>,
    ) -> Result<(), sp_consensus::Error>
    where
        C: ProvideRuntimeApi<Block>
            + BlockOf
            + AuxStore
            + HeaderBackend<Block>
            + HeaderMetadata<Block, Error = sp_blockchain::Error>
            + Send
            + Sync
            + 'static,
        C::Api: AuraApi<Block, AuthorityId>,
        SC: SelectChain<Block> + 'static,
        I: BlockImport<Block, Error = sp_consensus::Error> + Send + Sync + 'static,
        PF: Environment<Block, Error = Error> + Send + Sync + 'static,
        PF::Proposer: Proposer<Block, Error = Error>,
        SO: SyncOracle + Send + Sync + Clone + 'static,
        L: sc_consensus::JustificationSyncLink<Block> + 'static,
        CIDP: CreateInherentDataProviders<Block, ()> + Send + Sync + 'static,
        CIDP::InherentDataProviders: InherentDataProviderExt + Send,
        BS: BackoffAuthoringBlocksStrategy<NumberFor<Block>> + Send + Sync + 'static,
        Error: std::error::Error + Send + From<sp_consensus::Error> + From<I::Error> + 'static,
    {
        let aura = sc_consensus_aura::start_aura::<AuraPair, Block, _, _, _, _, _, _, _, _, _>(
            sc_consensus_aura::StartAuraParams {
                slot_duration: params.slot_duration,
                client: params.client,
                select_chain: params.select_chain,
                block_import: params.block_import,
                proposer_factory: params.proposer_factory,
                sync_oracle: params.sync_oracle,
                justification_sync_link: params.justification_sync_link,
                create_inherent_data_providers: params.create_inherent_data_providers,
                force_authoring: params.force_authoring,
                backoff_authoring_blocks: params.backoff_authoring_blocks,
                keystore: params.keystore,
                block_proposal_slot_portion: params.block_proposal_slot_portion,
                max_block_proposal_slot_portion: params.max_block_proposal_slot_portion,
                telemetry: params.telemetry,
                compatibility_mode: Default::default(),
            },
        )?;

        // the AURA authoring task is considered essential, i.e. if it
        // fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("aura", Some("block-authoring"), aura);

        Ok(())
    }

    fn frontier_consensus_data_provider(
        client: Arc<FullClient>,
    ) -> Result<Box<dyn fc_rpc::pending::ConsensusDataProvider<Block>>, sp_blockchain::Error> {
        Ok(Box::new(fc_aura::AuraConsensusDataProvider::new(client)))
    }

    fn create_inherent_data_providers(
        slot_duration: SlotDuration,
    ) -> Result<Self::InherentDataProviders, Box<dyn Error + Send + Sync>> {
        let current = sp_timestamp::InherentDataProvider::from_system_time();
        let next_slot = current
            .timestamp()
            .as_millis()
            .saturating_add(slot_duration.as_millis());
        let timestamp = sp_timestamp::InherentDataProvider::new(next_slot.into());
        let slot =
            sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
        Ok((slot, timestamp))
    }

    fn new() -> Self {
        Self {}
    }

    fn build_biq(&mut self) -> Result<BIQ, sc_service::Error>
    where
        NumberFor<Block>: BlockNumberOps,
    {
        let build_import_queue = Box::new(
            move |client: Arc<FullClient>,
                  _backend: Arc<FullBackend>,
                  config: &Configuration,
                  _eth_config: &EthConfiguration,
                  task_manager: &TaskManager,
                  telemetry: Option<TelemetryHandle>,
                  grandpa_block_import: GrandpaBlockImport,
                  _transaction_pool: Arc<TransactionPoolHandle<Block, FullClient>>| {
                let conditional_block_import = ConditionalEVMBlockImport::new(
                    grandpa_block_import.clone(),
                    FrontierBlockImport::new(grandpa_block_import.clone(), client.clone()),
                );

                let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
                let create_inherent_data_providers = move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
                    let slot =
						sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);
                    Ok((slot, timestamp))
                };

                let import_queue = super::aura_wrapped_import_queue::import_queue(
                    sc_consensus_aura::ImportQueueParams {
                        block_import: conditional_block_import.clone(),
                        justification_import: Some(Box::new(grandpa_block_import.clone())),
                        client,
                        create_inherent_data_providers,
                        spawner: &task_manager.spawn_essential_handle(),
                        registry: config.prometheus_registry(),
                        check_for_equivocation: Default::default(),
                        telemetry,
                        compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
                    },
                )
                .map_err::<sc_service::Error, _>(Into::into)?;

                Ok((
                    import_queue,
                    Box::new(conditional_block_import) as BoxBlockImport<Block>,
                ))
            },
        );

        Ok(build_import_queue)
    }

    fn slot_duration(&self, client: &FullClient) -> Result<SlotDuration, sc_service::Error> {
        sc_consensus_aura::slot_duration(client).map_err(Into::into)
    }

    fn spawn_essential_handles(
        &self,
        task_manager: &mut TaskManager,
        client: Arc<FullClient>,
        triggered: Option<Arc<std::sync::atomic::AtomicBool>>,
    ) -> Result<(), sc_service::Error> {
        let client_clone = client.clone();
        let triggered_clone = triggered.clone();
        let slot_duration = self.slot_duration(&client)?;
        task_manager.spawn_essential_handle().spawn(
        "babe-switch",
        None,
        Box::pin(async move {
            let client = client_clone;
            let triggered = triggered_clone;
            loop {
                // Check if the runtime is Babe once per block.
                if let Ok(c) = sc_consensus_babe::configuration(&*client) {
                    if !c.authorities.is_empty() {
                        log::info!("Babe runtime detected! Intentionally failing the essential handle `babe-switch` to trigger switch to Babe service.");
						if let Some(triggered) = triggered {
                        	triggered.store(true, std::sync::atomic::Ordering::SeqCst);
						};
                        break;
                    }
                };
                tokio::time::sleep(slot_duration.as_duration()).await;
            }
        }));
        Ok(())
    }

    fn rpc_methods(
        &self,
        _client: Arc<FullClient>,
        _keystore: KeystorePtr,
        _select_chain: FullSelectChain,
    ) -> Result<Vec<jsonrpsee::Methods>, sc_service::Error> {
        // Aura requires no special RPC methods.
        Ok(Default::default())
    }
}
