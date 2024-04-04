mod mock_setup;

use super::*;
use mock_setup::*;

// use common_primitives::node::BlockNumber;
// use pallet_messages_runtime_api::MessagesRuntimeApi;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;
use subtensor_custom_rpc::{DelegateInfoRuntimeApi, SubtensorCustom};

sp_api::mock_impl_runtime_apis! {
    impl DelegateInfoRuntimeApi<Block> for TestRuntimeApi {
        fn get_delegates() -> Vec<u8>{
            let result = SubtensorModule::get_delegates();
            result.encode()
        }
        fn get_delegate(delegate_account_vec: Vec<u8>) -> Vec<u8> {
            let _result = SubtensorModule::get_delegate(delegate_account_vec);
            if _result.is_some() {
                let result = _result.expect("Could not get DelegateInfo");
                result.encode()
            } else {
                vec![]
            }
        }

        fn get_delegated(delegatee_account_vec: Vec<u8>) -> Vec<u8> {
            let result = SubtensorModule::get_delegated(delegatee_account_vec);
            result.encode()
        }
}

}

#[tokio::test]
async fn get_messages_by_schema_with_invalid_request_should_panic() {
    let client = Arc::new(TestApi {});
    let api = SubtensorCustom::new(client.clone());
}
