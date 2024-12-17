//! RPC interface for the custom Subtensor rpc methods

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{error::ErrorObject, ErrorObjectOwned},
};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

use sp_api::ProvideRuntimeApi;

pub use subtensor_custom_rpc_runtime_api::{
    DelegateInfoRuntimeApi, NeuronInfoRuntimeApi, RateLimitInfoRuntimeApi, SubnetInfoRuntimeApi,
    SubnetRegistrationRuntimeApi,
};

#[rpc(client, server)]
pub trait SubtensorCustomApi<BlockHash> {
    #[method(name = "delegateInfo_getDelegates")]
    fn get_delegates(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "delegateInfo_getDelegate")]
    fn get_delegate(
        &self,
        delegate_account_vec: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "delegateInfo_getDelegated")]
    fn get_delegated(
        &self,
        delegatee_account_vec: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;

    #[method(name = "neuronInfo_getNeuronsLite")]
    fn get_neurons_lite(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "neuronInfo_getNeuronLite")]
    fn get_neuron_lite(&self, netuid: u16, uid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "neuronInfo_getNeurons")]
    fn get_neurons(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "neuronInfo_getNeuron")]
    fn get_neuron(&self, netuid: u16, uid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetInfo")]
    fn get_subnet_info(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetsInfo")]
    fn get_subnets_info(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetInfo_v2")]
    fn get_subnet_info_v2(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetsInf_v2")]
    fn get_subnets_info_v2(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetHyperparams")]
    fn get_subnet_hyperparams(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getLockCost")]
    fn get_network_lock_cost(&self, at: Option<BlockHash>) -> RpcResult<u64>;
    #[method(name = "subnetInfo_getDynamicInfo")]
    fn get_dynamic_info(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getAllDynamicInfo")]
    fn get_all_dynamic_info(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetState")]
    fn get_subnet_state(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "rateLimitInfo_getRateLimits")]
    fn get_rate_limits(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "rateLimitInfo_getLimitedTxInfoForHotkey")]
    fn get_limited_tx_info_for_hotkey(
        &self,
        hotkey: Vec<u8>,
        netuid: u16,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "rateLimitInfo_getStakesThisInterval")]
    fn get_stakes_this_interval(
        &self,
        coldkey: Vec<u8>,
        hotkey: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<u64>;
}

pub struct SubtensorCustom<C, P> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> SubtensorCustom<C, P> {
    /// Creates a new instance of the TransactionPayment Rpc helper.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The call to runtime failed.
    RuntimeError(String),
}

impl From<Error> for ErrorObjectOwned {
    fn from(e: Error) -> Self {
        match e {
            Error::RuntimeError(e) => ErrorObject::owned(1, e, None::<()>),
        }
    }
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError(_) => 1,
        }
    }
}

macro_rules! call_api {
    ($self:ident, $at:ident, $err_msg:expr, $name:ident, $($param:ident),* ) => {{
        let api = $self.client.runtime_api();
        let at = $at.unwrap_or_else(|| $self.client.info().best_hash);

        api.$name(at $(,$param)*)
            .map_err(|e| Error::RuntimeError(format!("{}: {:?}", $err_msg, e)).into())
	}}
}

impl<C, Block> SubtensorCustomApiServer<<Block as BlockT>::Hash> for SubtensorCustom<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: DelegateInfoRuntimeApi<Block>,
    C::Api: NeuronInfoRuntimeApi<Block>,
    C::Api: SubnetInfoRuntimeApi<Block>,
    C::Api: SubnetRegistrationRuntimeApi<Block>,
    C::Api: RateLimitInfoRuntimeApi<Block>,
{
    fn get_delegates(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        call_api!(self, at, "Unable to get delegates info", get_delegates,)
    }

    fn get_delegate(
        &self,
        delegate_account_vec: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get delegate info",
            get_delegate,
            delegate_account_vec
        )
    }

    fn get_delegated(
        &self,
        delegatee_account_vec: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get delegates info",
            get_delegated,
            delegatee_account_vec
        )
    }

    fn get_neurons_lite(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get neurons lite info",
            get_neurons_lite,
            netuid
        )
    }

    fn get_neuron_lite(
        &self,
        netuid: u16,
        uid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get neuron lite info",
            get_neuron_lite,
            netuid,
            uid
        )
    }

    fn get_neurons(&self, netuid: u16, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        call_api!(self, at, "Unable to get neurons info", get_neurons, netuid)
    }

    fn get_neuron(
        &self,
        netuid: u16,
        uid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get neuron info",
            get_neuron,
            netuid,
            uid
        )
    }

    fn get_subnet_state(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get subnet state",
            get_subnet_state,
            netuid
        )
    }

    fn get_subnet_info(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get subnet info",
            get_subnet_info,
            netuid
        )
    }

    fn get_subnet_hyperparams(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get subnet hyperparams",
            get_subnet_hyperparams,
            netuid
        )
    }

    fn get_subnets_info(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        call_api!(self, at, "Unable to get subnets info", get_subnets_info,)
    }

    fn get_all_dynamic_info(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        call_api!(self, at, "Unable to get subnets info", get_all_dynamic_info,)
    }

    fn get_dynamic_info(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get subnets info",
            get_dynamic_info,
            netuid
        )
    }

    fn get_subnet_info_v2(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get subnet info",
            get_subnet_info_v2,
            netuid
        )
    }

    fn get_subnets_info_v2(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        call_api!(self, at, "Unable to get subnets info", get_subnets_info_v2,)
    }

    fn get_network_lock_cost(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
        call_api!(
            self,
            at,
            "Unable to get subnet lock cost",
            get_network_registration_cost,
        )
    }

    fn get_rate_limits(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        call_api!(self, at, "Unable to get rate limits", get_rate_limits,)
    }

    fn get_limited_tx_info_for_hotkey(
        &self,
        hotkey: Vec<u8>,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        call_api!(
            self,
            at,
            "Unable to get rate limits info for hotkey",
            get_limited_tx_info_for_hotkey,
            hotkey,
            netuid
        )
    }

    fn get_stakes_this_interval(
        &self,
        coldkey: Vec<u8>,
        hotkey: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<u64> {
        call_api!(
            self,
            at,
            "Unable to get number of stakes for the interval",
            get_stakes_this_interval,
            coldkey,
            hotkey
        )
    }
}
