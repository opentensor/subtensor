//! RPC interface for the Swap pallet

use std::sync::Arc;

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObjectOwned, error::ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

pub use pallet_subtensor_swap_runtime_api::SwapRuntimeApi;

#[rpc(client, server)]
pub trait SwapRpcApi<BlockHash> {
    #[method(name = "swap_currentAlphaPrice")]
    fn current_alpha_price(&self, netuid: u16, at: Option<BlockHash>) -> RpcResult<u64>;
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

/// Swap RPC implementation.
pub struct Swap<C, Block> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> Swap<C, Block> {
    /// Create new `Swap` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> SwapRpcApiServer<<Block as BlockT>::Hash> for Swap<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: SwapRuntimeApi<Block>,
{
    fn current_alpha_price(
        &self,
        netuid: u16,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.current_alpha_price(at, netuid).map_err(|e| {
            Error::RuntimeError(format!("Unable to get current alpha price: {e:?}")).into()
        })
    }
}
