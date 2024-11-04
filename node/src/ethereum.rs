use crate::rpc::EthDeps;
use fc_rpc::{
    pending::AuraConsensusDataProvider, Debug, DebugApiServer, Eth, EthApiServer, EthConfig,
    EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub, EthPubSubApiServer, EthSigner, EthTask,
    Net, NetApiServer, Web3, Web3ApiServer,
};
use fp_rpc::{ConvertTransaction, ConvertTransactionRuntimeApi, EthereumRuntimeRPCApi};
use futures::future;
use futures::StreamExt;
use jsonrpsee::RpcModule;
use sc_client_api::{
    backend::{Backend, StorageProvider},
    client::BlockchainEvents,
    AuxStore, UsageProvider,
};
use sc_executor::HostFunctions;
use sc_network_sync::SyncingService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_transaction_pool::ChainApi;
use sc_transaction_pool_api::TransactionPool;
use sp_api::{CallApiAt, ConstructRuntimeApi, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder as BlockBuilderApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_aura::AuraApi;
use sp_core::H256;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::Block as BlockT;
use std::path::PathBuf;
use std::time::Duration;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub use fc_consensus::FrontierBlockImport;
pub use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
/// Frontier DB backend type.
pub use fc_storage::{StorageOverride, StorageOverrideHandler};

use crate::client::{FullBackend, FullClient};

pub type FrontierBackend<B, C> = fc_db::Backend<B, C>;

/// Avalailable frontier backend types.
#[derive(Debug, Copy, Clone, Default, clap::ValueEnum)]
pub enum BackendType {
    /// Either RocksDb or ParityDb as per inherited from the global backend settings.
    #[default]
    KeyValue,
    /// Sql database with custom log indexing.
    Sql,
}

/// The ethereum-compatibility configuration used to run a node.
#[derive(Clone, Debug, clap::Parser)]
pub struct EthConfiguration {
    /// Maximum number of logs in a query.
    #[arg(long, default_value = "10000")]
    pub max_past_logs: u32,

    /// Maximum fee history cache size.
    #[arg(long, default_value = "2048")]
    pub fee_history_limit: u64,

    #[arg(long)]
    pub enable_dev_signer: bool,

    /// The dynamic-fee pallet target gas price set by block author
    #[arg(long, default_value = "1")]
    pub target_gas_price: u64,

    /// Maximum allowed gas limit will be `block.gas_limit * execute_gas_limit_multiplier`
    /// when using eth_call/eth_estimateGas.
    #[arg(long, default_value = "10")]
    pub execute_gas_limit_multiplier: u64,

    /// Size in bytes of the LRU cache for block data.
    #[arg(long, default_value = "50")]
    pub eth_log_block_cache: usize,

    /// Size in bytes of the LRU cache for transactions statuses data.
    #[arg(long, default_value = "50")]
    pub eth_statuses_cache: usize,

    /// Sets the frontier backend type (KeyValue or Sql)
    #[arg(long, value_enum, ignore_case = true, default_value_t = BackendType::default())]
    pub frontier_backend_type: BackendType,

    // Sets the SQL backend's pool size.
    #[arg(long, default_value = "100")]
    pub frontier_sql_backend_pool_size: u32,

    /// Sets the SQL backend's query timeout in number of VM ops.
    #[arg(long, default_value = "10000000")]
    pub frontier_sql_backend_num_ops_timeout: u32,

    /// Sets the SQL backend's auxiliary thread limit.
    #[arg(long, default_value = "4")]
    pub frontier_sql_backend_thread_count: u32,

    /// Sets the SQL backend's query timeout in number of VM ops.
    /// Default value is 200MB.
    #[arg(long, default_value = "209715200")]
    pub frontier_sql_backend_cache_size: u64,
}

pub fn db_config_dir(config: &Configuration) -> PathBuf {
    config.base_path.config_dir(config.chain_spec.id())
}

pub struct FrontierPartialComponents {
    pub filter_pool: Option<FilterPool>,
    pub fee_history_cache: FeeHistoryCache,
    pub fee_history_cache_limit: FeeHistoryCacheLimit,
}

pub fn new_frontier_partial(
    config: &EthConfiguration,
) -> Result<FrontierPartialComponents, ServiceError> {
    Ok(FrontierPartialComponents {
        filter_pool: Some(Arc::new(Mutex::new(BTreeMap::new()))),
        fee_history_cache: Arc::new(Mutex::new(BTreeMap::new())),
        fee_history_cache_limit: config.fee_history_limit,
    })
}

/// A set of APIs that ethereum-compatible runtimes must implement.
pub trait EthCompatRuntimeApiCollection<Block: BlockT>:
    sp_api::ApiExt<Block>
    + fp_rpc::ConvertTransactionRuntimeApi<Block>
    + fp_rpc::EthereumRuntimeRPCApi<Block>
{
}

impl<Block, Api> EthCompatRuntimeApiCollection<Block> for Api
where
    Block: BlockT,
    Api: sp_api::ApiExt<Block>
        + fp_rpc::ConvertTransactionRuntimeApi<Block>
        + fp_rpc::EthereumRuntimeRPCApi<Block>,
{
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_frontier_tasks<B, RA, HF>(
    task_manager: &TaskManager,
    client: Arc<FullClient<B, RA, HF>>,
    backend: Arc<FullBackend<B>>,
    frontier_backend: Arc<FrontierBackend<B, FullClient<B, RA, HF>>>,
    filter_pool: Option<FilterPool>,
    storage_override: Arc<dyn StorageOverride<B>>,
    fee_history_cache: FeeHistoryCache,
    fee_history_cache_limit: FeeHistoryCacheLimit,
    sync: Arc<SyncingService<B>>,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<B>,
        >,
    >,
) where
    B: BlockT<Hash = H256>,
    RA: ConstructRuntimeApi<B, FullClient<B, RA, HF>>,
    RA: Send + Sync + 'static,
    RA::RuntimeApi: EthCompatRuntimeApiCollection<B>,
    HF: HostFunctions + 'static,
{
    // Spawn main mapping sync worker background task.
    match &*frontier_backend {
        fc_db::Backend::KeyValue(b) => {
            task_manager.spawn_essential_handle().spawn(
                "frontier-mapping-sync-worker",
                Some("frontier"),
                fc_mapping_sync::kv::MappingSyncWorker::new(
                    client.import_notification_stream(),
                    Duration::new(6, 0),
                    client.clone(),
                    backend,
                    storage_override.clone(),
                    b.clone(),
                    3,
                    0u32.into(),
                    fc_mapping_sync::SyncStrategy::Normal,
                    sync,
                    pubsub_notification_sinks,
                )
                .for_each(|()| future::ready(())),
            );
        }
        fc_db::Backend::Sql(b) => {
            task_manager.spawn_essential_handle().spawn_blocking(
                "frontier-mapping-sync-worker",
                Some("frontier"),
                fc_mapping_sync::sql::SyncWorker::run(
                    client.clone(),
                    backend,
                    b.clone(),
                    client.import_notification_stream(),
                    fc_mapping_sync::sql::SyncWorkerConfig {
                        read_notification_timeout: Duration::from_secs(30),
                        check_indexed_blocks_interval: Duration::from_secs(60),
                    },
                    fc_mapping_sync::SyncStrategy::Parachain,
                    sync,
                    pubsub_notification_sinks,
                ),
            );
        }
    }

    // Spawn Frontier EthFilterApi maintenance task.
    if let Some(filter_pool) = filter_pool {
        // Each filter is allowed to stay in the pool for 100 blocks.
        const FILTER_RETAIN_THRESHOLD: u64 = 100;
        task_manager.spawn_essential_handle().spawn(
            "frontier-filter-pool",
            Some("frontier"),
            EthTask::filter_pool_task(client.clone(), filter_pool, FILTER_RETAIN_THRESHOLD),
        );
    }

    // Spawn Frontier FeeHistory cache maintenance task.
    task_manager.spawn_essential_handle().spawn(
        "frontier-fee-history",
        Some("frontier"),
        EthTask::fee_history_task(
            client,
            storage_override,
            fee_history_cache,
            fee_history_cache_limit,
        ),
    );
}

fn extend_rpc_aet_api<B, C, BE, P, A, CT, CIDP, EC>(
    io: &mut RpcModule<()>,
    deps: &EthDeps<B, C, P, A, CT, CIDP>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: AuraApi<B, AuraId>
        + BlockBuilderApi<B>
        + ConvertTransactionRuntimeApi<B>
        + EthereumRuntimeRPCApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CT: ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + Clone + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + Clone + 'static,
    EC: EthConfig<B, C>,
{
    let mut signers = Vec::new();
    if deps.enable_dev_signer {
        signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
    }

    io.merge(
        Eth::<B, C, P, CT, BE, A, CIDP, EC>::new(
            deps.client.clone(),
            deps.pool.clone(),
            deps.graph.clone(),
            deps.converter.clone(),
            deps.sync.clone(),
            signers,
            deps.storage_override.clone(),
            deps.frontier_backend.clone(),
            deps.is_authority,
            deps.block_data_cache.clone(),
            deps.fee_history_cache.clone(),
            deps.fee_history_cache_limit,
            deps.execute_gas_limit_multiplier,
            deps.forced_parent_hashes.clone(),
            deps.pending_create_inherent_data_providers.clone(),
            Some(Box::new(AuraConsensusDataProvider::new(
                deps.client.clone(),
            ))),
        )
        .replace_config::<EC>()
        .into_rpc(),
    )?;
    Ok(())
}

fn extend_rpc_eth_filter<B, C, BE, P, A, CT, CIDP>(
    io: &mut RpcModule<()>,
    deps: &EthDeps<B, C, P, A, CT, CIDP>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: AuraApi<B, AuraId>
        + BlockBuilderApi<B>
        + ConvertTransactionRuntimeApi<B>
        + EthereumRuntimeRPCApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CT: ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + Clone + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + Clone + 'static,
{
    if let Some(filter_pool) = deps.filter_pool.clone() {
        io.merge(
            EthFilter::new(
                deps.client.clone(),
                deps.frontier_backend.clone(),
                deps.graph.clone(),
                filter_pool,
                500_usize, // max stored filters
                deps.max_past_logs,
                deps.block_data_cache.clone(),
            )
            .into_rpc(),
        )?;
    }
    Ok(())
}

// Function for EthPubSub merge
fn extend_rpc_eth_pubsub<B, C, BE, P, A, CT, CIDP>(
    io: &mut RpcModule<()>,
    deps: &EthDeps<B, C, P, A, CT, CIDP>,
    subscription_task_executor: SubscriptionTaskExecutor,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<B>,
        >,
    >,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: AuraApi<B, AuraId>
        + BlockBuilderApi<B>
        + ConvertTransactionRuntimeApi<B>
        + EthereumRuntimeRPCApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CT: ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + 'static,
{
    io.merge(
        EthPubSub::new(
            deps.pool.clone(),
            deps.client.clone(),
            deps.sync.clone(),
            subscription_task_executor,
            deps.storage_override.clone(),
            pubsub_notification_sinks,
        )
        .into_rpc(),
    )?;
    Ok(())
}

fn extend_rpc_net<B, C, BE, P, A, CT, CIDP>(
    io: &mut RpcModule<()>,
    deps: &EthDeps<B, C, P, A, CT, CIDP>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: AuraApi<B, AuraId>
        + BlockBuilderApi<B>
        + ConvertTransactionRuntimeApi<B>
        + EthereumRuntimeRPCApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CT: ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + 'static,
{
    io.merge(
        Net::new(
            deps.client.clone(),
            deps.network.clone(),
            true, // Whether to format the `peer_count` response as Hex (default) or not.
        )
        .into_rpc(),
    )?;
    Ok(())
}

fn extend_rpc_web3<B, C, BE, P, A, CT, CIDP>(
    io: &mut RpcModule<()>,
    deps: &EthDeps<B, C, P, A, CT, CIDP>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: AuraApi<B, AuraId>
        + BlockBuilderApi<B>
        + ConvertTransactionRuntimeApi<B>
        + EthereumRuntimeRPCApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CT: ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + 'static,
{
    io.merge(Web3::new(deps.client.clone()).into_rpc())?;
    Ok(())
}

fn extend_rpc_debug<B, C, BE, P, A, CT, CIDP>(
    io: &mut RpcModule<()>,
    deps: &EthDeps<B, C, P, A, CT, CIDP>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: AuraApi<B, AuraId>
        + BlockBuilderApi<B>
        + ConvertTransactionRuntimeApi<B>
        + EthereumRuntimeRPCApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CT: ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + 'static,
{
    io.merge(
        Debug::new(
            deps.client.clone(),
            deps.frontier_backend.clone(),
            deps.storage_override.clone(),
            deps.block_data_cache.clone(),
        )
        .into_rpc(),
    )?;
    Ok(())
}

/// Extend RpcModule with Eth RPCs
pub fn create_eth<B, C, BE, P, A, CT, CIDP, EC>(
    mut io: RpcModule<()>,
    deps: EthDeps<B, C, P, A, CT, CIDP>,
    subscription_task_executor: SubscriptionTaskExecutor,
    pubsub_notification_sinks: Arc<
        fc_mapping_sync::EthereumBlockNotificationSinks<
            fc_mapping_sync::EthereumBlockNotification<B>,
        >,
    >,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    B: BlockT<Hash = H256>,
    C: CallApiAt<B> + ProvideRuntimeApi<B>,
    C::Api: AuraApi<B, AuraId>
        + BlockBuilderApi<B>
        + ConvertTransactionRuntimeApi<B>
        + EthereumRuntimeRPCApi<B>,
    C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError>,
    C: BlockchainEvents<B> + AuxStore + UsageProvider<B> + StorageProvider<B, BE> + 'static,
    BE: Backend<B> + 'static,
    P: TransactionPool<Block = B> + 'static,
    A: ChainApi<Block = B> + 'static,
    CT: ConvertTransaction<<B as BlockT>::Extrinsic> + Send + Sync + Clone + 'static,
    CIDP: CreateInherentDataProviders<B, ()> + Send + Clone + 'static,
    EC: EthConfig<B, C>,
{
    extend_rpc_aet_api::<B, C, BE, P, A, CT, CIDP, EC>(&mut io, &deps)?;
    extend_rpc_eth_filter::<B, C, BE, P, A, CT, CIDP>(&mut io, &deps)?;
    extend_rpc_eth_pubsub::<B, C, BE, P, A, CT, CIDP>(
        &mut io,
        &deps,
        subscription_task_executor,
        pubsub_notification_sinks,
    )?;
    extend_rpc_net::<B, C, BE, P, A, CT, CIDP>(&mut io, &deps)?;
    extend_rpc_web3::<B, C, BE, P, A, CT, CIDP>(&mut io, &deps)?;
    extend_rpc_debug::<B, C, BE, P, A, CT, CIDP>(&mut io, &deps)?;

    Ok(io)
}
