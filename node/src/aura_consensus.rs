use crate::{
    aura_service::{ConsensusBuilder, GrandpaBlockImport},
    client::FullClient,
    conditional_evm_block_import::ConditionalEVMBlockImport,
    ethereum::EthConfiguration,
};
use fc_consensus::FrontierBlockImport;
use jsonrpsee::tokio;
use node_subtensor_runtime::opaque::Block;
use sc_consensus::{BasicQueue, BoxBlockImport};
use sc_consensus_grandpa::BlockNumberOps;
use sc_service::{Configuration, TaskManager};
use sc_telemetry::TelemetryHandle;
use sp_consensus_slots::SlotDuration;
use sp_runtime::traits::NumberFor;
use std::{error::Error, sync::Arc};

pub struct AuraConsensus;

impl ConsensusBuilder for AuraConsensus {
    type InherentDataProviders = (
        sp_consensus_aura::inherents::InherentDataProvider,
        sp_timestamp::InherentDataProvider,
    );

    fn pending_create_inherent_data_providers(
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

    fn build_import_queue(
        client: Arc<FullClient>,
        config: &Configuration,
        _eth_config: &EthConfiguration,
        task_manager: &TaskManager,
        telemetry: Option<TelemetryHandle>,
        grandpa_block_import: GrandpaBlockImport,
    ) -> Result<(BasicQueue<Block>, BoxBlockImport<Block>), sc_service::Error>
    where
        NumberFor<Block>: BlockNumberOps,
    {
        let conditional_block_import = ConditionalEVMBlockImport::new(
            grandpa_block_import.clone(),
            FrontierBlockImport::new(grandpa_block_import.clone(), client.clone()),
            client.clone(),
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

        let import_queue =
            crate::aura_wrapped_import_queue::import_queue(sc_consensus_aura::ImportQueueParams {
                block_import: conditional_block_import.clone(),
                justification_import: Some(Box::new(grandpa_block_import.clone())),
                client,
                create_inherent_data_providers,
                spawner: &task_manager.spawn_essential_handle(),
                registry: config.prometheus_registry(),
                check_for_equivocation: Default::default(),
                telemetry,
                compatibility_mode: sc_consensus_aura::CompatibilityMode::None,
            })
            .map_err::<sc_service::Error, _>(Into::into)?;

        Ok((import_queue, Box::new(conditional_block_import)))
    }

    fn slot_duration(client: &FullClient) -> Result<SlotDuration, sc_service::Error> {
        sc_consensus_aura::slot_duration(&*client).map_err(Into::into)
    }

    fn spawn_essential_handles(
        task_manager: &mut TaskManager,
        client: Arc<FullClient>,
        triggered: Option<Arc<std::sync::atomic::AtomicBool>>,
    ) -> Result<(), sc_service::Error> {
        let client_clone = client.clone();
        let triggered_clone = triggered.clone();
        let slot_duration = AuraConsensus::slot_duration(&client)?;
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
}
