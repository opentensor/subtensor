use sp_api::{ApiExt, ApiRef, ProvideRuntimeApi, StorageProof};
// use sp_core::storage::StateBackend as CoreStateBackend;
use sp_runtime::generic::Header;
use sp_runtime::traits::{Block as BlockT, NumberFor, Zero};
// use sp_runtime::{generic::Block as GenericBlock, traits::BlakeTwo256};
// use sp_version::RuntimeVersion;
use substrate_test_runtime_client::runtime::Block;

use sp_blockchain::HeaderBackend;

pub struct TestApi {}

pub struct TestRuntimeApi {}

impl ProvideRuntimeApi<Block> for TestApi {
    type Api = TestRuntimeApi;

    fn runtime_api<'a>(&'a self) -> ApiRef<'a, Self::Api> {
        TestRuntimeApi {}.into()
    }
}
/// Blockchain database header backend. Does not perform any validation.
impl<Block: BlockT> HeaderBackend<Block> for TestApi {
    fn header(
        &self,
        _id: <Block as sp_api::BlockT>::Hash,
    ) -> std::result::Result<Option<Block::Header>, sp_blockchain::Error> {
        Ok(None)
    }

    fn info(&self) -> sc_client_api::blockchain::Info<Block> {
        sc_client_api::blockchain::Info {
            best_hash: Default::default(),
            best_number: Zero::zero(),
            finalized_hash: Default::default(),
            finalized_number: Zero::zero(),
            genesis_hash: Default::default(),
            number_leaves: Default::default(),
            finalized_state: None,
            block_gap: None,
        }
    }

    fn status(
        &self,
        _id: <Block as sp_api::BlockT>::Hash,
    ) -> std::result::Result<sc_client_api::blockchain::BlockStatus, sp_blockchain::Error> {
        Ok(sc_client_api::blockchain::BlockStatus::Unknown)
    }

    fn number(
        &self,
        _hash: Block::Hash,
    ) -> std::result::Result<Option<NumberFor<Block>>, sp_blockchain::Error> {
        Ok(None)
    }

    fn hash(
        &self,
        _number: NumberFor<Block>,
    ) -> std::result::Result<Option<Block::Hash>, sp_blockchain::Error> {
        Ok(None)
    }
}
