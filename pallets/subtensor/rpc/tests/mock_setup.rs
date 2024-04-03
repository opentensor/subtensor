use sp_api::{ApiExt, ApiRef, ProvideRuntimeApi, StorageProof};
// use sp_core::storage::StateBackend as CoreStateBackend;
use sp_runtime::traits::{Block as BlockT, NumberFor, Zero};
use sp_runtime::{generic::Block as GenericBlock, traits::BlakeTwo256};
use sp_runtime::{generic::Header, traits::BlakeTwo256};
// use sp_version::RuntimeVersion;
use substrate_test_runtime_client::runtime::{Block, Extrinsic, RuntimeApiImpl};
// use substrate_test_runtime_client::substrate_test_runtime::Extrinsic;

use sp_blockchain::HeaderBackend;
// use sp_runtime::traits::{Block as BlockT, NumberFor, Zero};

pub struct TestApi {}

pub struct TestRuntimeApi {}

impl ProvideRuntimeApi<Block> for TestApi {
    type Api = TestRuntimeApi;

    fn runtime_api<'a>(&'a self) -> ApiRef<'a, Self::Api> {
        TestRuntimeApi {}.into()
    }
}
