//! RPC interface for the custom Subtensor rpc methods

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

use sp_api::ProvideRuntimeApi;
use sp_core::Bytes;

pub use subtensor_custom_rpc_runtime_api::{
    DelegateInfoRuntimeApi, NeuronInfoRuntimeApi, StakeInfoRuntimeApi, SubnetInfoRuntimeApi,
    SubnetRegistrationRuntimeApi,
};

#[rpc(client, server)]
pub trait SubtensorCustomApi<BlockHash> {

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

    #[method(name = "delegateInfo_getDelegates")]
    fn get_delegates(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
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
    #[method(name = "subnetInfo_getSubnetHyperparams")]
    fn get_subnet_hyperparams(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;

    #[method(name = "subnetInfo_getLockCost")]
    fn get_network_lock_cost(&self, at: Option<BlockHash>) -> RpcResult<u64>;

    #[method(name = "subnetInfo_getSubnetStakeInfoForColdKey")]
    fn get_subnet_stake_info_for_cold_key(
        &self,
        coldkey_account_vec: Vec<u8>,
        netuid: u16,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetStakeInfoForColdKeys")]
    fn get_subnet_stake_info_for_coldkeys(
        &self,
        coldkey_account_vecs: Vec<Vec<u8>>,
        netuid: u16,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getTotalSubnetStake")]
    fn get_total_subnet_stake(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getAllStakeInfoForColdKey")]
    fn get_all_stake_info_for_coldkey(
        &self,
        coldkey_account_vec: Bytes,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
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
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
        }
    }
}

impl<C, Block> SubtensorCustomApiServer<<Block as BlockT>::Hash> for SubtensorCustom<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: DelegateInfoRuntimeApi<Block>,
    C::Api: NeuronInfoRuntimeApi<Block>,
    C::Api: SubnetInfoRuntimeApi<Block>,
    C::Api: SubnetRegistrationRuntimeApi<Block>,
    C::Api: StakeInfoRuntimeApi<Block>,
{
    fn get_delegates(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_delegates(at).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get delegates info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_delegate(
        &self,
        delegate_account_vec: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_delegate(at, delegate_account_vec).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get delegate info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_delegated(
        &self,
        delegatee_account_vec: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_delegated(at, delegatee_account_vec).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get delegated info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_neurons_lite(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_neurons_lite(at, netuid).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get neurons lite info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_neuron_lite(
        &self,
        netuid: u16,
        uid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_neuron_lite(at, netuid, uid).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get neuron lite info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_neurons(&self, netuid: u16, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_neurons(at, netuid).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get neurons info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_neuron(
        &self,
        netuid: u16,
        uid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_neuron(at, netuid, uid).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get neuron info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_subnet_info(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_info(at, netuid).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get subnet info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_subnet_hyperparams(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_hyperparams(at, netuid).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get subnet info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_subnets_info(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnets_info(at).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get subnets info.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_network_lock_cost(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_network_registration_cost(at).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get subnet lock cost.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_subnet_stake_info_for_cold_key(
        &self,
        coldkey_account_vec: Vec<u8>,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_stake_info_for_coldkey(at, coldkey_account_vec, netuid)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to get subnet stake info.",
                    Some(e.to_string()),
                ))
                .into()
            })
    }

    fn get_subnet_stake_info_for_coldkeys(
        &self,
        coldkey_account_vecs: Vec<Vec<u8>>,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_stake_info_for_coldkeys(at, coldkey_account_vecs, netuid)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to get subnet stake info.",
                    Some(e.to_string()),
                ))
                .into()
            })
    }

    fn get_total_subnet_stake(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_total_subnet_stake(at, netuid).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to get total subnet stake.",
                Some(e.to_string()),
            ))
            .into()
        })
    }

    fn get_all_stake_info_for_coldkey(
        &self,
        coldkey_account_vec: Bytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_all_stake_info_for_coldkey(at, coldkey_account_vec)
            .map_err(|e| {
                CallError::Custom(ErrorObject::owned(
                    Error::RuntimeError.into(),
                    "Unable to get all stake info for coldkey.",
                    Some(e.to_string()),
                ))
                .into()
            })
    }
}
