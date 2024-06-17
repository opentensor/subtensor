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

use pallet_subtensor::types::TensorBytes;
pub use subtensor_custom_rpc_runtime_api::{
    DelegateInfoRuntimeApi, DynamicPoolInfoRuntimeApi, NeuronInfoRuntimeApi, StakeInfoRuntimeApi,
    SubnetInfoRuntimeApi, SubnetRegistrationRuntimeApi,
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

    #[method(name = "delegateInfo_getSubStakeForHotkey")]
    fn get_substake_for_hotkey(
        &self,
        hotkey_bytes: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "delegateInfo_getSubStakeForColdkey")]
    fn get_substake_for_coldkey(
        &self,
        coldkey_bytes: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "delegateInfo_getSubStakeForNetuid")]
    fn get_substake_for_netuid(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "delegateInfo_getTotalStakeForHotkey")]
    fn get_total_stake_for_hotkey(
        &self,
        hotkey_bytes: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<u64>;
    #[method(name = "delegateInfo_getTotalStakeForColdkey")]
    fn get_total_stake_for_coldkey(
        &self,
        hotkey_bytes: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<u64>;

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

    #[method(name = "subnetInfo_getSubnetInfoV2")]
    fn get_subnet_info_v2(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetsInfoV2")]
    fn get_subnets_info_v2(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getLockCost")]
    fn get_network_lock_cost(&self, at: Option<BlockHash>) -> RpcResult<u64>;

    #[method(name = "subnetInfo_getSubnetStakeInfoForColdKey")]
    fn get_subnet_stake_info_for_cold_key(
        &self,
        coldkey_account_vec: TensorBytes,
        netuid: u16,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getSubnetStakeInfoForColdKeys")]
    fn get_subnet_stake_info_for_coldkeys(
        &self,
        coldkey_account_vecs: Vec<TensorBytes>,
        netuid: u16,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getTotalSubnetStake")]
    fn get_total_subnet_stake(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getAllStakeInfoForColdKey")]
    fn get_all_stake_info_for_coldkey(
        &self,
        coldkey_account_vec: TensorBytes,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getAllSubnetStakeInfoForColdKey")]
    fn get_all_subnet_stake_info_for_coldkey(
        &self,
        coldkey_account_vec: TensorBytes,
        at: Option<BlockHash>,
    ) -> RpcResult<Vec<u8>>;
    #[method(name = "subnetInfo_getTotalStakeForEachSubnet")]
    fn get_total_stake_for_each_subnet(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;

    #[method(name = "dynamicPoolInfo_getDynamicPoolInfo")]
    fn get_dynamic_pool_info(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "dynamicPoolInfo_getAllDynamicPoolInfos")]
    fn get_all_dynamic_pool_infos(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    
    #[method(name = "dynamicPoolInfo_getDynamicPoolInfoV2")]
    fn get_dynamic_pool_info_v2(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
    #[method(name = "dynamicPoolInfo_getAllDynamicPoolInfosV2")]
    fn get_all_dynamic_pool_infos_v2(&self, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
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

impl<C, Block> SubtensorCustomApiServer<<Block as BlockT>::Hash> for SubtensorCustom<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: DelegateInfoRuntimeApi<Block>,
    C::Api: NeuronInfoRuntimeApi<Block>,
    C::Api: SubnetInfoRuntimeApi<Block>,
    C::Api: SubnetRegistrationRuntimeApi<Block>,
    C::Api: StakeInfoRuntimeApi<Block>,
    C::Api: DynamicPoolInfoRuntimeApi<Block>,
{
    fn get_substake_for_hotkey(
        &self,
        hotkey_bytes: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_substake_for_hotkey(at, hotkey_bytes).map_err(|e| {
            Error::RuntimeError(format!("Unable to get delegates info: {:?}", e)).into()
        })
    }

    fn get_substake_for_coldkey(
        &self,
        coldkey_bytes: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_substake_for_coldkey(at, coldkey_bytes)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get delegates info: {:?}", e)).into()
            })
    }

    fn get_substake_for_netuid(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_substake_for_netuid(at, netuid).map_err(|e| {
            Error::RuntimeError(format!("Unable to get delegates info: {:?}", e)).into()
        })
    }

    fn get_total_stake_for_hotkey(
        &self,
        hotkey_bytes: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_total_stake_for_hotkey(at, hotkey_bytes)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get total stake for hotkey: {:?}", e)).into()
            })
    }

    fn get_total_stake_for_coldkey(
        &self,
        hotkey_bytes: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);
        api.get_total_stake_for_coldkey(at, hotkey_bytes)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get total stake for coldkey: {:?}", e))
                    .into()
            })
    }

    fn get_delegates(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_delegates(at).map_err(|e| {
            Error::RuntimeError(format!("Unable to get delegates info: {:?}", e)).into()
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
            Error::RuntimeError(format!("Unable to get delegates info: {:?}", e)).into()
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
            Error::RuntimeError(format!("Unable to get delegates info: {:?}", e)).into()
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
            Error::RuntimeError(format!("Unable to get neurons lite info: {:?}", e)).into()
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
            Error::RuntimeError(format!("Unable to get neurons lite info: {:?}", e)).into()
        })
    }

    fn get_neurons(&self, netuid: u16, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_neurons(at, netuid)
            .map_err(|e| Error::RuntimeError(format!("Unable to get neurons info: {:?}", e)).into())
    }

    fn get_neuron(
        &self,
        netuid: u16,
        uid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_neuron(at, netuid, uid)
            .map_err(|e| Error::RuntimeError(format!("Unable to get neuron info: {:?}", e)).into())
    }

    fn get_subnet_info(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_info(at, netuid)
            .map_err(|e| Error::RuntimeError(format!("Unable to get subnet info: {:?}", e)).into())
    }

    fn get_subnet_info_v2(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_info_v2(at, netuid)
            .map_err(|e| Error::RuntimeError(format!("Unable to get subnet info: {:?}", e)).into())
    }

    fn get_subnet_hyperparams(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_hyperparams(at, netuid)
            .map_err(|e| Error::RuntimeError(format!("Unable to get subnet info: {:?}", e)).into())
    }

    fn get_subnets_info(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnets_info(at)
            .map_err(|e| Error::RuntimeError(format!("Unable to get subnets info: {:?}", e)).into())
    }

    fn get_subnets_info_v2(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnets_info_v2(at)
            .map_err(|e| Error::RuntimeError(format!("Unable to get subnets info: {:?}", e)).into())
    }

    fn get_network_lock_cost(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_network_registration_cost(at).map_err(|e| {
            Error::RuntimeError(format!("Unable to get subnet lock cost: {}", e)).into()
        })
    }

    fn get_subnet_stake_info_for_cold_key(
        &self,
        coldkey_account_vec: TensorBytes,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_stake_info_for_coldkey(at, coldkey_account_vec, netuid)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get subnet stake info: {}", e)).into()
            })
    }

    fn get_subnet_stake_info_for_coldkeys(
        &self,
        coldkey_account_vecs: Vec<TensorBytes>,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_subnet_stake_info_for_coldkeys(at, coldkey_account_vecs, netuid)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get subnet stake info: {}", e)).into()
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
            Error::RuntimeError(format!("Unable to get total subnet stake: {}", e)).into()
        })
    }

    fn get_all_stake_info_for_coldkey(
        &self,
        coldkey_account_vec: TensorBytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_all_stake_info_for_coldkey(at, coldkey_account_vec)
            .map_err(|e| {
                Error::RuntimeError(format!("Unable to get all stake info for coldkey: {}", e))
                    .into()
            })
    }

    fn get_all_subnet_stake_info_for_coldkey(
        &self,
        coldkey_account_vec: TensorBytes,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_all_subnet_stake_info_for_coldkey(at, coldkey_account_vec)
            .map_err(|e| {
                Error::RuntimeError(format!(
                    "Unable to get all subnet stake info for coldkey: {}",
                    e
                ))
                .into()
            })
    }

    fn get_dynamic_pool_info(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_dynamic_pool_info(at, netuid).map_err(|e| {
            Error::RuntimeError(format!("Unable to get dynamic pool info: {}", e)).into()
        })
    }

    fn get_dynamic_pool_info_v2(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_dynamic_pool_info_v2(at, netuid).map_err(|e| {
            Error::RuntimeError(format!("Unable to get dynamic pool info: {}", e)).into()
        })
    }

    fn get_all_dynamic_pool_infos(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_all_dynamic_pool_infos(at).map_err(|e| {
            Error::RuntimeError(format!("Unable to get all dynamic pool infos: {}", e)).into()
        })
    }

    fn get_all_dynamic_pool_infos_v2(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_all_dynamic_pool_infos_v2(at).map_err(|e| {
            Error::RuntimeError(format!("Unable to get all dynamic pool infos: {}", e)).into()
        })
    }

    fn get_total_stake_for_each_subnet(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_total_stake_for_each_subnet(at).map_err(|e| {
            Error::RuntimeError(format!("Unable to get total stake for each subnet: {}", e)).into()
        })
    }
}
