// use frame_support::sp_runtime::DispatchError;
// use frame_support::{
//     assert_err, assert_noop, assert_ok,
//     dispatch::{DispatchClass, GetDispatchInfo, Pays},
//     traits::Hooks,
// };
// use frame_system::Config;
// use pallet_subtensor::{Error as SubtensorError, SubnetOwner, Tempo, WeightsVersionKeyRateLimit};
// use pallet_subtensor::Event;
// use sp_consensus_grandpa::AuthorityId as GrandpaId;
// use sp_core::{Get, Pair, U256, ed25519};
// use substrate_fixed::types::I96F32;
// use subtensor_runtime_common::NetUid;

use mock::*;

mod mock;

#[test]
fn test_remove_stake_fees_tao() {
    new_test_ext().execute_with(|| {});
}

#[test]
fn test_remove_stake_fees_alpha() {
    new_test_ext().execute_with(|| {});
}
