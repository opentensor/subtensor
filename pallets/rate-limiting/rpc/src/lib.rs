//! RPC interface for the rate limiting pallet.

use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObjectOwned, error::ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

pub use pallet_rate_limiting_runtime_api::{RateLimitRpcResponse, RateLimitingRuntimeApi};

#[rpc(client, server)]
pub trait RateLimitingRpcApi<BlockHash> {
    #[method(name = "rateLimiting_getRateLimit")]
    fn get_rate_limit(
        &self,
        pallet: Vec<u8>,
        extrinsic: Vec<u8>,
        at: Option<BlockHash>,
    ) -> RpcResult<Option<RateLimitRpcResponse>>;
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

/// RPC implementation for the rate limiting pallet.
pub struct RateLimiting<C, Block> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> RateLimiting<C, Block> {
    /// Creates a new instance of the rate limiting RPC helper.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> RateLimitingRpcApiServer<<Block as BlockT>::Hash> for RateLimiting<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: RateLimitingRuntimeApi<Block>,
{
    fn get_rate_limit(
        &self,
        pallet: Vec<u8>,
        extrinsic: Vec<u8>,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<Option<RateLimitRpcResponse>> {
        let api = self.client.runtime_api();
        let at = at.unwrap_or_else(|| self.client.info().best_hash);

        api.get_rate_limit(at, pallet, extrinsic)
            .map_err(|e| Error::RuntimeError(format!("Unable to fetch rate limit: {e:?}")).into())
    }
}
