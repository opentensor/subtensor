#![allow(non_snake_case)]

use std::sync::Arc;

use sp_api::{ApiRef, ProvideRuntimeApi};
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
use sp_runtime::{
    generic::{self},
    traits::{BlakeTwo256, Block as BlockT, Extrinsic, NumberFor, Verify, Zero},
};
use codec::{ Compact, Encode };

use sp_blockchain::HeaderBackend;
use subtensor_custom_rpc::{
    DelegateInfoRuntimeApi, NeuronInfoRuntimeApi, StakeInfoRuntimeApi, SubnetInfoRuntimeApi,
    SubnetRegistrationRuntimeApi, SubtensorCustom,
};
use substrate_test_runtime_client::runtime::{Block, Hash};
use subtensor_custom_rpc::SubtensorCustomApiServer;
use pallet_subtensor::{
    stake_info::SubnetStakeInfo,
    delegate_info::DelegateInfo,
};
use node_subtensor_runtime::Runtime;
use sp_core::Bytes;

pub struct TestApi {}
pub struct TestRuntimeApi {}

const ONE_KEY: [u8; 32] = [1; 32];

sp_api::mock_impl_runtime_apis! {
    impl DelegateInfoRuntimeApi<Block> for TestRuntimeApi {
        fn get_delegates(&self) -> Vec<u8> {
            let mut result = Vec::new();
            result.push(DelegateInfo::<Runtime> {
                delegate_ss58: sp_runtime::AccountId32::from(ONE_KEY),
                take: Compact(1),
                nominators: Vec::new(),
                owner_ss58: sp_runtime::AccountId32::from(ONE_KEY),
                registrations: Vec::new(),
                validator_permits: Vec::new(),
                return_per_1000: Compact(123),
                total_daily_return: Compact(456),
            });

            result.encode()
        }
        fn get_delegate(&self, _delegate_account_vec: Vec<u8>) -> Vec<u8> {
            (Some(DelegateInfo::<Runtime> {
                delegate_ss58: sp_runtime::AccountId32::from(ONE_KEY),
                take: Compact(1),
                nominators: Vec::new(),
                owner_ss58: sp_runtime::AccountId32::from(ONE_KEY),
                registrations: Vec::new(),
                validator_permits: Vec::new(),
                return_per_1000: Compact(123),
                total_daily_return: Compact(456),
            })).encode()
        }

        fn get_delegated(&self, _delegatee_account_vec: Vec<u8>) -> Vec<u8> {
            let mut result = Vec::new();
            result.push(DelegateInfo::<Runtime> {
                delegate_ss58: sp_runtime::AccountId32::from(ONE_KEY),
                take: Compact(1),
                nominators: Vec::new(),
                owner_ss58: sp_runtime::AccountId32::from(ONE_KEY),
                registrations: Vec::new(),
                validator_permits: Vec::new(),
                return_per_1000: Compact(123),
                total_daily_return: Compact(456),
            });

            result.encode()
        }
    }

    impl NeuronInfoRuntimeApi<Block> for TestRuntimeApi {
        fn get_neurons(netuid: u16) -> Vec<u8> {
            unimplemented!()
        }
        fn get_neuron(netuid: u16, uid: u16) -> Vec<u8> {
            unimplemented!()
        }
        fn get_neurons_lite(netuid: u16) -> Vec<u8> {
            unimplemented!()
        }
        fn get_neuron_lite(netuid: u16, uid: u16) -> Vec<u8> {
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
        #[advanced]
        fn get_all_stake_info_for_coldkey(&self, _at: Hash, coldkey_account_vec: Bytes) -> Result<Vec<u8>, sp_api::ApiError> {
           // Mock result from pallet as a SubnetStakeInfo with production AccountId
            let mut pubkey_array: [u8; 32] = [0; 32];
            pubkey_array.copy_from_slice(&coldkey_account_vec[..32]);
            let coldkey: node_subtensor_runtime::AccountId = sp_runtime::AccountId32::from(pubkey_array);

            let mut result = Vec::<SubnetStakeInfo<Runtime>>::new();
            result.push(SubnetStakeInfo{
                hotkey: coldkey,
                netuid: 1,
                stake: Compact(1),
            });

             Ok(result.encode())
        }
    }

    impl SubnetRegistrationRuntimeApi<Block> for TestRuntimeApi {
        fn get_network_registration_cost() -> u64 {
            0xCCCC
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
    let api = SubtensorCustom::new(client);
    let request = api.get_delegates(None);
    let response = request.unwrap();
    println!("response: {:?}", response);
}

#[tokio::test]
async fn get_all_stake_info_for_coldkey_should_work() {
    let client = Arc::new(TestApi {});
    let api = SubtensorCustom::new(client);

    let magic_address = Bytes(Vec::from([0xd2, 0xb7, 0x73, 0x64, 0xd1,
        0xc3, 0xb4, 0x45, 0xcd, 0x69, 0xbd, 0x59, 0xf1, 0xa8, 0x7d, 0xcb,
        0x26, 0xc9, 0xce, 0x3f, 0x46, 0x43, 0x7d, 0x55, 0xb8, 0x8b, 0x43,
        0xf1, 0xc1, 0x77, 0xe7, 0x76]));

    let request = api.get_all_stake_info_for_coldkey(magic_address, None);
    let response = request.unwrap();
    println!("response: {:?}", response);
}
