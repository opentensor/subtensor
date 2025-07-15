//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use futures::channel::mpsc;

use crate::ethereum::DefaultEthConfig;
use crate::ethereum::EthDeps;
use crate::{client::FullClient, ethereum::create_eth};
use jsonrpsee::RpcModule;
use node_subtensor_runtime::opaque::Block;
use polkadot_rpc::BabeDeps;
use sc_consensus_babe_rpc::{Babe, BabeApiServer};
use sc_consensus_manual_seal::EngineCommand;
use sc_rpc::SubscriptionTaskExecutor;
use sc_transaction_pool_api::TransactionPool;
use sp_consensus::SelectChain;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::{OpaqueExtrinsic, traits::BlakeTwo256, traits::Block as BlockT};
use subtensor_runtime_common::Hash;

/// Full client dependencies.
pub struct FullDeps<P, CT, CIDP, SC> {
    /// The client instance to use.
    pub client: Arc<FullClient>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Manual seal command sink
    pub command_sink: Option<mpsc::Sender<EngineCommand<Hash>>>,
    /// Ethereum-compatibility specific dependencies.
    pub eth: EthDeps<P, CT, CIDP>,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// The [`SelectChain`] Strategy
    pub select_chain: SC,
}

/// Instantiate all full RPC extensions.
pub fn create_full<P, CT, CIDP, SC>(
    deps: FullDeps<P, CT, CIDP, SC>,
    subscription_task_executor: SubscriptionTaskExecutor,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<Block>,
        >,
    >,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    P: TransactionPool<
            Block = Block,
            Hash = <sp_runtime::generic::Block<
                sp_runtime::generic::Header<u32, BlakeTwo256>,
                OpaqueExtrinsic,
            > as BlockT>::Hash,
        > + 'static,
    CIDP: CreateInherentDataProviders<Block, ()> + Send + Clone + 'static,
    CT: fp_rpc::ConvertTransaction<<Block as BlockT>::Extrinsic> + Send + Sync + Clone + 'static,
    SC: SelectChain<Block> + 'static,
{
    use pallet_subtensor_swap_rpc::{Swap, SwapRpcApiServer};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};
    use subtensor_custom_rpc::{SubtensorCustom, SubtensorCustomApiServer};

    let mut module = RpcModule::new(());
    let FullDeps {
        client,
        pool,
        command_sink,
        eth,
        babe,
        select_chain,
    } = deps;

    sc_consensus_babe::configuration(&*client)?;

    let BabeDeps {
        babe_worker_handle,
        keystore,
    } = babe;

    // Custom RPC methods for Paratensor
    module.merge(SubtensorCustom::new(client.clone()).into_rpc())?;

    // Swap RPC
    module.merge(Swap::new(client.clone()).into_rpc())?;

    module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

    if let Some(command_sink) = command_sink {
        module.merge(
            // We provide the rpc handler with the sending end of the channel to allow the rpc
            // send EngineCommands to the background block authorship task.
            ManualSeal::new(command_sink).into_rpc(),
        )?;
    }

    module.merge(
        Babe::new(
            client.clone(),
            babe_worker_handle.clone(),
            keystore,
            select_chain,
        )
        .into_rpc(),
    )?;

    // Ethereum compatibility RPCs
    let module = create_eth::<_, _, _, DefaultEthConfig>(
        module,
        eth,
        subscription_task_executor,
        pubsub_notification_sinks,
        Some(Box::new(fc_babe::BabeConsensusDataProvider::new(client)?)),
    )?;

    Ok(module)
}
