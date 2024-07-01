// #![no_std]
// use std::sync::Arc;

// use sp_api::{ApiRef, ProvideRuntimeApi};
// pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
// use sp_runtime::{
//     generic::{self},
//     traits::{BlakeTwo256, Block as BlockT, Extrinsic, NumberFor, Verify, Zero},
// };

// use codec::{Compact, Encode};
// use pallet_subtensor::stake_info::SubnetStakeInfo;
// use sp_blockchain::HeaderBackend;
// // use substrate_test_runtime_client::runtime::{Block, Hash};
// use subtensor_custom_rpc::SubtensorCustomApiServer;
// use subtensor_custom_rpc::{
//     DelegateInfoRuntimeApi, NeuronInfoRuntimeApi, StakeInfoRuntimeApi, SubnetInfoRuntimeApi,
//     SubnetRegistrationRuntimeApi, SubtensorCustom,
// };

// /// An identifier for an account on this system.
// pub type AccountId = <Signature as Verify>::Signer;
// /// A simple hash type for all our hashing.
// pub type Hash = H256;
// /// The hashing algorithm used.
// pub type Hashing = BlakeTwo256;
// /// The block number type used in this runtime.
// pub type BlockNumber = u64;
// /// Index of a transaction.
// pub type Nonce = u64;
// /// The item of a block digest.
// pub type DigestItem = sp_runtime::generic::DigestItem;
// /// The digest of a block.
// pub type Digest = sp_runtime::generic::Digest;
// /// A test block.
// pub type Block = sp_runtime::generic::Block<Header, Extrinsic>;
// /// A test block's header.
// pub type Header = sp_runtime::generic::Header<BlockNumber, Hashing>;
// /// Balance of an account.
// pub type Balance = u64;

// pub struct TestApi {}
// pub struct TestRuntimeApi {}

// sp_api::mock_impl_runtime_apis! {
//     impl DelegateInfoRuntimeApi<Block> for TestRuntimeApi {
//         #[advanced]
//         fn get_delegates(&self, at: Hash) -> Result<Vec<u8>, sp_api::ApiError> {
//             // let result = SubtensorModule::get_delegates();
//             // result.encode()
//             Ok(Vec::new())
//         }
//         fn get_delegate(&self, delegate_account_vec: Vec<u8>) -> Vec<u8> {
//             unimplemented!()
//         }

//         fn get_delegated(&self, delegatee_account_vec: Vec<u8>) -> Vec<u8> {
//             unimplemented!()
//         }
//     }

//     impl NeuronInfoRuntimeApi<Block> for TestRuntimeApi {
//         fn get_neurons(netuid: u16) -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_neuron(netuid: u16, uid: u16) -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_neurons_lite(netuid: u16) -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_neuron_lite(netuid: u16, uid: u16) -> Vec<u8> {
//             unimplemented!()
//         }
//     }

//     impl StakeInfoRuntimeApi<Block> for TestRuntimeApi {
//         fn get_stake_info_for_coldkey( coldkey_account_vec: Vec<u8> ) -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_stake_info_for_coldkeys( coldkey_account_vecs: Vec<Vec<u8>> ) -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_subnet_stake_info_for_coldkeys( coldkey_account_vecs: Vec<Vec<u8>>, netuid: u16 ) -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_total_subnet_stake( netuid: u16 ) -> Vec<u8> {
//             unimplemented!()
//         }
//         #[advanced]
//         fn get_all_stake_info_for_coldkey(&self, _at: Hash, _coldkey_account_vec: Vec<u8>) -> Result<Vec<u8>, sp_api::ApiError> {

//             // Mock result from pallet as a SubnetStakeInfo with production AccountId
//             // let coldkey: T::AccountId = T::AccountId::decode(&mut &coldkey_account_vec[..])
//             // .expect("Failed to decode AccountId");

//             // let mut result = Vec::<(SubnetStakeInfo<pallet_subtensor::Config>, u16, Compact<u64>)>::new();
//             // result.push(SubnetStakeInfo{
//             //     hotkey: Default::default(),
//             //     netuid: 1,
//             //     stake: Compact(1),
//             // });

//             // Mock result from pallet as a tuple with u64 AccountId
//             let mut result = Vec::<(u64, u16, Compact<u64>)>::new();
//             for i in 0..10 {
//                 result.push((
//                     i,
//                     i as u16,
//                     Compact(1),
//                 ));
//             }

//             Ok(result.encode())
//         }
//     }

//     impl SubnetRegistrationRuntimeApi<Block> for TestRuntimeApi {
//         fn get_network_registration_cost() -> u64 {
//             unimplemented!()
//         }
//     }

//     impl SubnetInfoRuntimeApi<Block> for TestRuntimeApi {
//         fn get_subnet_info(netuid: u16) -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_subnets_info() -> Vec<u8> {
//             unimplemented!()
//         }
//         fn get_subnet_hyperparams(netuid: u16) -> Vec<u8> {
//             unimplemented!()
//         }
//     }
// }

// impl ProvideRuntimeApi<Block> for TestApi {
//     type Api = TestRuntimeApi;

//     fn runtime_api<'a>(&'a self) -> ApiRef<'a, Self::Api> {
//         TestRuntimeApi {}.into()
//     }
// }
// /// Blockchain database header backend. Does not perform any validation.
// impl<Block: BlockT> HeaderBackend<Block> for TestApi {
//     fn header(
//         &self,
//         _id: <Block as sp_api::BlockT>::Hash,
//     ) -> std::result::Result<Option<Block::Header>, sp_blockchain::Error> {
//         Ok(None)
//     }

//     fn info(&self) -> sc_client_api::blockchain::Info<Block> {
//         sc_client_api::blockchain::Info {
//             best_hash: Default::default(),
//             best_number: Zero::zero(),
//             finalized_hash: Default::default(),
//             finalized_number: Zero::zero(),
//             genesis_hash: Default::default(),
//             number_leaves: Default::default(),
//             finalized_state: None,
//             block_gap: None,
//         }
//     }

//     fn status(
//         &self,
//         _id: <Block as sp_api::BlockT>::Hash,
//     ) -> std::result::Result<sc_client_api::blockchain::BlockStatus, sp_blockchain::Error> {
//         Ok(sc_client_api::blockchain::BlockStatus::Unknown)
//     }

//     fn number(
//         &self,
//         _hash: Block::Hash,
//     ) -> std::result::Result<Option<NumberFor<Block>>, sp_blockchain::Error> {
//         Ok(None)
//     }

//     fn hash(
//         &self,
//         _number: NumberFor<Block>,
//     ) -> std::result::Result<Option<Block::Hash>, sp_blockchain::Error> {
//         Ok(None)
//     }
// }

// #[tokio::test]
// async fn get_delegates_should_work() {
//     let client = Arc::new(TestApi {});
//     let api = SubtensorCustom::new(client);
//     let request = api.get_delegates(None);
//     let response = request.unwrap();
//     println!("response: {:?}", response);
// }

// #[tokio::test]
// async fn get_all_stake_info_for_coldkey_should_work() {
//     let client = Arc::new(TestApi {});
//     let api = SubtensorCustom::new(client);

//     let magic_address = Vec::from([
//         0xd2, 0xb7, 0x73, 0x64, 0xd1, 0xc3, 0xb4, 0x45, 0xcd, 0x69, 0xbd, 0x59, 0xf1, 0xa8, 0x7d,
//         0xcb, 0x26, 0xc9, 0xce, 0x3f, 0x46, 0x43, 0x7d, 0x55, 0xb8, 0x8b, 0x43, 0xf1, 0xc1, 0x77,
//         0xe7, 0x76,
//     ]);

//     let request = api.get_all_stake_info_for_coldkey(magic_address, None);
//     let response = request.unwrap();
//     println!("response: {:?}", response);
// }
