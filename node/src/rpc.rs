//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use node_subtensor_runtime::{opaque::Block, AccountId, Balance, BlockNumber, Hash, Index};
use polkadot_rpc::BabeDeps;
use sc_consensus_grandpa::FinalityProofProvider;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;

/// Dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: sc_consensus_grandpa::SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: sc_consensus_grandpa::SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: sc_consensus_grandpa::GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, B, SC> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// Babe import setup.
    pub babe: BabeDeps,
    /// Grandpa block import setup.
    pub grandpa: GrandpaDeps<B>,
    /// Backend used by the node.
    pub _backend: Arc<B>,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, B, SC>(
    deps: FullDeps<C, P, B, SC>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: sc_consensus_babe::BabeApi<Block>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    C::Api: subtensor_custom_rpc_runtime_api::DelegateInfoRuntimeApi<Block>,
    C::Api: subtensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi<Block>,
    C::Api: subtensor_custom_rpc_runtime_api::SubnetInfoRuntimeApi<Block>,
    C::Api: subtensor_custom_rpc_runtime_api::SubnetRegistrationRuntimeApi<Block>,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    P: TransactionPool + 'static,
    SC: SelectChain<Block> + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use sc_consensus_babe_rpc::{Babe, BabeApiServer};
    use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};
    use subtensor_custom_rpc::{SubtensorCustom, SubtensorCustomApiServer};

    let mut module = RpcModule::new(());

    let FullDeps {
        client,
        pool,
        select_chain,
        grandpa,
        babe,
        _backend: _,
    } = deps;

    // Custom RPC methods for Paratensor
    module.merge(SubtensorCustom::new(client.clone()).into_rpc())?;

    module.merge(System::new(client.clone(), pool.clone()).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    let BabeDeps {
        babe_worker_handle,
        keystore,
    } = babe;

    module.merge(
        Babe::new(
            client.clone(),
            babe_worker_handle.clone(),
            keystore,
            select_chain,
        )
        .into_rpc(),
    )?;

    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    module.merge(
        Grandpa::new(
            subscription_executor,
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            finality_provider,
        )
        .into_rpc(),
    )?;

    // Extend this RPC with a custom API by using the following syntax.
    // `YourRpcStruct` should have a reference to a client, which is needed
    // to call into the runtime.
    // `module.merge(YourRpcTrait::into_rpc(YourRpcStruct::new(ReferenceToClient, ...)))?;`

    Ok(module)
}
