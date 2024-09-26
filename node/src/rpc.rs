//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::{collections::BTreeMap, sync::Arc};

use fp_rpc::{ConvertTransactionRuntimeApi, EthereumRuntimeRPCApi};
use futures::channel::mpsc;

use crate::ethereum::create_eth;
pub use fc_rpc::EthBlockDataCacheTask;
pub use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
use fc_storage::StorageOverride;
use jsonrpsee::RpcModule;
use node_subtensor_runtime::{AccountId, Balance, Hash, Nonce};
use sc_client_api::{
    backend::{Backend, StorageProvider},
    client::BlockchainEvents,
    AuxStore, UsageProvider,
};
use sc_consensus_manual_seal::EngineCommand;
use sc_network::service::traits::NetworkService;
use sc_network_sync::SyncingService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_aura::AuraApi;
use sp_core::H256;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::Block as BlockT;

/// Extra dependencies for Ethereum compatibility.
pub struct EthDeps<B: BlockT, C, P, A: ChainApi, CT, CIDP> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Graph pool instance.
    pub graph: Arc<Pool<A>>,
    /// Ethereum transaction converter.
    pub converter: Option<CT>,
    /// The Node authority flag
    pub is_authority: bool,
    /// Whether to enable dev signer
    pub enable_dev_signer: bool,
    /// Network service
    pub network: Arc<dyn NetworkService>,
    /// Chain syncing service
    pub sync: Arc<SyncingService<B>>,
    /// Frontier Backend.
    pub frontier_backend: Arc<dyn fc_api::Backend<B>>,
    /// Ethereum data access overrides.
    pub storage_override: Arc<dyn StorageOverride<B>>,
    /// Cache for Ethereum block data.
    pub block_data_cache: Arc<EthBlockDataCacheTask<B>>,
    /// EthFilterApi pool.
    pub filter_pool: Option<FilterPool>,
    /// Maximum number of logs in a query.
    pub max_past_logs: u32,
    /// Fee history cache.
    pub fee_history_cache: FeeHistoryCache,
    /// Maximum fee history cache size.
    pub fee_history_cache_limit: FeeHistoryCacheLimit,
    /// Maximum allowed gas limit will be ` block.gas_limit * execute_gas_limit_multiplier` when
    /// using eth_call/eth_estimateGas.
    pub execute_gas_limit_multiplier: u64,
    /// Mandated parent hashes for a given block hash.
    pub forced_parent_hashes: Option<BTreeMap<H256, H256>>,
    /// Something that can create the inherent data providers for pending state
    pub pending_create_inherent_data_providers: CIDP,
}

/// Default Eth RPC configuration
pub struct DefaultEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);

impl<B, C, BE> fc_rpc::EthConfig<B, C> for DefaultEthConfig<C, BE>
where
	B: BlockT,
	C: StorageProvider<B, BE> + Sync + Send + 'static,
	BE: Backend<B> + 'static,
{
	type EstimateGasAdapter = ();
	type RuntimeStorageOverride =
		fc_rpc::frontier_backend_client::SystemAccountId20StorageOverride<B, C, BE>;
}

/// Full client dependencies.
pub struct FullDeps<B: BlockT, C, P, A: ChainApi, CT, CIDP> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Manual seal command sink
	pub command_sink: Option<mpsc::Sender<EngineCommand<Hash>>>,
	/// Ethereum-compatibility specific dependencies.
	pub eth: EthDeps<B, C, P, A, CT, CIDP>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<B, C, P, BE, A, CT, CIDP>(
    deps: FullDeps<B, C, P, A, CT, CIDP>,
    subscription_task_executor: SubscriptionTaskExecutor,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<B>,
        >,
    >,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = sp_core::H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: BlockBuilder<B>,
    C::Api: AuraApi<B, AuraId>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<B, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<B, Balance>,
    C::Api: ConvertTransactionRuntimeApi<B>,
    C::Api: EthereumRuntimeRPCApi<B>,
    C::Api: subtensor_custom_rpc_runtime_api::DelegateInfoRuntimeApi<B>,
    C::Api: subtensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi<B>,
    C::Api: subtensor_custom_rpc_runtime_api::SubnetInfoRuntimeApi<B>,
    C::Api: subtensor_custom_rpc_runtime_api::SubnetRegistrationRuntimeApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE>,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + Clone + 'static,
    CT: fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + Clone + 'static,
{
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
    } = deps;

    // Custom RPC methods for Paratensor
    module.merge(SubtensorCustom::new(client.clone()).into_rpc())?;

    module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;
    module.merge(TransactionPayment::new(client).into_rpc())?;

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

    // Ethereum compatibility RPCs
    let module = create_eth::<_, _, _, _, _, _, _, DefaultEthConfig<C, BE>>(
        module,
        eth,
        subscription_task_executor,
        pubsub_notification_sinks,
    )?;

    Ok(module)
}
