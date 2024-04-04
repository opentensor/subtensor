use std::sync::Arc;

use sp_api::{ApiRef, ProvideRuntimeApi};
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use sp_runtime::{
    generic::{self},
    traits::{BlakeTwo256, Block as BlockT, Extrinsic, NumberFor, Verify, Zero},
};

use sp_blockchain::HeaderBackend;
use subtensor_custom_rpc::{
    DelegateInfoRuntimeApi, StakeInfoRuntimeApi, SubnetInfoRuntimeApi,
    SubnetRegistrationRuntimeApi, SubtensorCustom,
};
pub type BlockNumber = u32;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub struct TestApi {}
pub struct TestRuntimeApi {}

sp_api::mock_impl_runtime_apis! {
    impl DelegateInfoRuntimeApi<Block> for TestRuntimeApi {
        fn get_delegates() -> Vec<u8>{
            unimplemented!()
        }
        fn get_delegate(delegate_account_vec: Vec<u8>) -> Vec<u8> {
            unimplemented!()
        }

        fn get_delegated(delegatee_account_vec: Vec<u8>) -> Vec<u8> {
            unimplemented!()
        }
    }

    impl StakeInfoRuntimeApi<Block> for TestRuntimeApi {
        fn get_stake_info_for_coldkey( coldkey_account_vec: Vec<u8> ) -> Vec<u8> {
            unimplemented!()
        }
        fn get_stake_info_for_coldkeys( coldkey_account_vecs: Vec<Vec<u8>> ) -> Vec<u8> {
            unimplemented!()
        }
        fn get_subnet_stake_info_for_coldkeys( coldkey_account_vecs: Vec<Vec<u8>>, netuid: u16 ) -> Vec<u8> {
            unimplemented!()
        }
        fn get_total_subnet_stake( netuid: u16 ) -> Vec<u8> {
            unimplemented!()
        }
        fn get_all_stake_info_for_coldkey( coldkey_account_vec: Vec<u8> ) -> Vec<u8> {
            unimplemented!()
        }
    }

    impl SubnetRegistrationRuntimeApi<Block> for TestRuntimeApi {
        fn get_network_registration_cost() -> u64 {
            unimplemented!()
        }
    }

    impl SubnetInfoRuntimeApi<Block> for TestRuntimeApi {
        fn get_subnet_info(netuid: u16) -> Vec<u8> {
            unimplemented!()
        }
        fn get_subnets_info() -> Vec<u8> {
            unimplemented!()
        }
        fn get_subnet_hyperparams(netuid: u16) -> Vec<u8> {
            unimplemented!()
        }
}
}

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

#[tokio::test]
async fn get_delegates_should_work() {
    let client = Arc::new(TestApi {});
    let api = SubtensorCustom::new(client.clone());
    let request = api.get_delegates();
    let response = request.await.unwrap();
    println!("response: {:?}", response);
}
