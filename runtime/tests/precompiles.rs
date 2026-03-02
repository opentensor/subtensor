#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use core::iter::IntoIterator;
use std::collections::BTreeSet;

use fp_evm::{Context, ExitError, PrecompileFailure, PrecompileResult};
use frame_support::BoundedVec;
use node_subtensor_runtime::{BuildStorage, Runtime, RuntimeGenesisConfig, System};
use pallet_drand::{LastStoredRound, Pulses, types::Pulse};
use pallet_evm::{AddressMapping, BalanceConverter, PrecompileSet};
use precompile_utils::testing::{MockHandle, PrecompileTesterExt};
use sp_core::{H160, H256, U256};
use sp_runtime::traits::Hash;
use subtensor_precompiles::{
    AddressMappingPrecompile, BalanceTransferPrecompile, DrandPrecompile, PrecompileExt,
    Precompiles,
};

type AccountId = <Runtime as frame_system::Config>::AccountId;

fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext: sp_io::TestExternalities = RuntimeGenesisConfig::default()
        .build_storage()
        .unwrap()
        .into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn execute_precompile(
    precompiles: &Precompiles<Runtime>,
    precompile_address: H160,
    caller: H160,
    input: Vec<u8>,
    apparent_value: U256,
) -> Option<PrecompileResult> {
    let mut handle = MockHandle::new(
        precompile_address,
        Context {
            address: precompile_address,
            caller,
            apparent_value,
        },
    );
    handle.input = input;
    precompiles.execute(&mut handle)
}

fn evm_apparent_value_from_substrate(amount: u64) -> U256 {
    <Runtime as pallet_evm::Config>::BalanceConverter::into_evm_balance(amount.into())
        .expect("runtime balance conversion should work for test amount")
        .into()
}

fn addr_from_index(index: u64) -> H160 {
    H160::from_low_u64_be(index)
}

#[test]
fn precompile_registry_addresses_are_unique() {
    new_test_ext().execute_with(|| {
        let addresses = Precompiles::<Runtime>::used_addresses();
        let unique: BTreeSet<_> = IntoIterator::into_iter(addresses).collect();
        assert_eq!(unique.len(), addresses.len());
    });
}

mod address_mapping {
    use super::*;

    fn address_mapping_call_data(target: H160) -> Vec<u8> {
        // Solidity selector for addressMapping(address).
        let selector = sp_io::hashing::keccak_256(b"addressMapping(address)");
        let mut input = Vec::with_capacity(4 + 32);
        // First 4 bytes of keccak256(function_signature): ABI function selector.
        input.extend_from_slice(&selector[..4]);
        // Left-pad the 20-byte address argument to a 32-byte ABI word.
        input.extend_from_slice(&[0u8; 12]);
        // The 20-byte address payload (right-aligned in the 32-byte ABI word).
        input.extend_from_slice(target.as_bytes());
        input
    }

    #[test]
    fn address_mapping_precompile_returns_runtime_address_mapping() {
        new_test_ext().execute_with(|| {
            let precompiles = Precompiles::<Runtime>::new();

            let caller = addr_from_index(1);
            let target_address = addr_from_index(0x1234);
            let input = address_mapping_call_data(target_address);

            let mapped_account =
                <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(target_address);
            let expected_output: [u8; 32] = mapped_account.into();

            let precompile_addr = addr_from_index(AddressMappingPrecompile::<Runtime>::INDEX);
            precompiles
                .prepare_test(caller, precompile_addr, input)
                .with_static_call(true)
                .execute_returns_raw(expected_output.to_vec());
        });
    }
}

mod balance_transfer {
    use super::*;

    fn balance_transfer_call_data(target: H256) -> Vec<u8> {
        // Solidity selector for transfer(bytes32).
        let selector = sp_io::hashing::keccak_256(b"transfer(bytes32)");
        let mut input = Vec::with_capacity(4 + 32);
        input.extend_from_slice(&selector[..4]);
        input.extend_from_slice(target.as_bytes());
        input
    }

    #[test]
    fn balance_transfer_precompile_transfers_balance() {
        new_test_ext().execute_with(|| {
            let precompiles = Precompiles::<Runtime>::new();
            let precompile_addr = addr_from_index(BalanceTransferPrecompile::<Runtime>::INDEX);
            let dispatch_account: AccountId = BalanceTransferPrecompile::<Runtime>::account_id();
            let destination_raw = H256::repeat_byte(7);
            let destination_account: AccountId = destination_raw.0.into();

            let amount = 123_456;
            pallet_subtensor::Pallet::<Runtime>::add_balance_to_coldkey_account(
                &dispatch_account,
                (amount * 2).into(),
            );

            let source_balance_before =
                pallet_balances::Pallet::<Runtime>::free_balance(&dispatch_account);
            let destination_balance_before =
                pallet_balances::Pallet::<Runtime>::free_balance(&destination_account);

            let result = execute_precompile(
                &precompiles,
                precompile_addr,
                addr_from_index(1),
                balance_transfer_call_data(destination_raw),
                evm_apparent_value_from_substrate(amount),
            );
            let precompile_result =
                result.expect("expected precompile transfer call to be routed to a precompile");
            precompile_result.expect("expected successful precompile transfer dispatch");

            let source_balance_after =
                pallet_balances::Pallet::<Runtime>::free_balance(&dispatch_account);
            let destination_balance_after =
                pallet_balances::Pallet::<Runtime>::free_balance(&destination_account);

            assert_eq!(source_balance_after, source_balance_before - amount);
            assert_eq!(
                destination_balance_after,
                destination_balance_before + amount
            );
        });
    }

    #[test]
    fn balance_transfer_precompile_respects_dispatch_guard_policy() {
        new_test_ext().execute_with(|| {
            let precompiles = Precompiles::<Runtime>::new();
            let precompile_addr = addr_from_index(BalanceTransferPrecompile::<Runtime>::INDEX);
            let dispatch_account: AccountId = BalanceTransferPrecompile::<Runtime>::account_id();
            let destination_raw = H256::repeat_byte(8);
            let destination_account: AccountId = destination_raw.0.into();

            let amount = 100;
            pallet_subtensor::Pallet::<Runtime>::add_balance_to_coldkey_account(
                &dispatch_account,
                1_000_000_u64.into(),
            );

            // Activate coldkey-swap guard for precompile dispatch account.
            let replacement_coldkey = AccountId::from([9u8; 32]);
            let replacement_hash =
                <Runtime as frame_system::Config>::Hashing::hash_of(&replacement_coldkey);
            pallet_subtensor::ColdkeySwapAnnouncements::<Runtime>::insert(
                &dispatch_account,
                (System::block_number(), replacement_hash),
            );

            let source_balance_before =
                pallet_balances::Pallet::<Runtime>::free_balance(&dispatch_account);
            let destination_balance_before =
                pallet_balances::Pallet::<Runtime>::free_balance(&destination_account);

            let result = execute_precompile(
                &precompiles,
                precompile_addr,
                addr_from_index(1),
                balance_transfer_call_data(destination_raw),
                evm_apparent_value_from_substrate(amount),
            );
            let precompile_result =
                result.expect("expected precompile transfer call to be routed to a precompile");
            let failure = precompile_result
                .expect_err("expected transaction extension rejection on precompile dispatch");
            let message = match failure {
                PrecompileFailure::Error {
                    exit_status: ExitError::Other(message),
                } => message,
                other => panic!("unexpected precompile failure: {other:?}"),
            };
            assert!(
                message.contains("dispatch execution failed: ColdkeySwapAnnounced"),
                "unexpected precompile failure: {message}"
            );

            let source_balance_after =
                pallet_balances::Pallet::<Runtime>::free_balance(&dispatch_account);
            let destination_balance_after =
                pallet_balances::Pallet::<Runtime>::free_balance(&destination_account);
            assert_eq!(source_balance_after, source_balance_before);
            assert_eq!(destination_balance_after, destination_balance_before);
        });
    }
}

mod drand {
    use super::*;

    fn get_last_stored_round_call_data() -> Vec<u8> {
        let selector = sp_io::hashing::keccak_256(b"getLastStoredRound()");
        selector[..4].to_vec()
    }

    fn get_randomness_call_data(round: u64) -> Vec<u8> {
        let selector = sp_io::hashing::keccak_256(b"getRandomness(uint64)");
        let mut input = Vec::with_capacity(4 + 32);
        input.extend_from_slice(&selector[..4]);
        input.extend_from_slice(&[0u8; 24]);
        input.extend_from_slice(&round.to_be_bytes());
        input
    }

    #[test]
    fn drand_precompile_get_last_stored_round_returns_value() {
        new_test_ext().execute_with(|| {
            let round = 1000;
            LastStoredRound::<Runtime>::put(round);
            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(DrandPrecompile::<Runtime>::INDEX);
            let input = get_last_stored_round_call_data();

            let result =
                execute_precompile(&precompiles, precompile_addr, caller, input, U256::zero());
            let precompile_result =
                result.expect("expected precompile call to be routed to drand precompile");
            let output = precompile_result
                .expect("expected successful getLastStoredRound call")
                .output;

            assert_eq!(
                output.len(),
                32,
                "getLastStoredRound should return 32 bytes (uint64 ABI)"
            );
            #[allow(clippy::indexing_slicing)]
            let output_u64 = u64::from_be_bytes(output[24..32].try_into().unwrap());
            assert_eq!(output_u64, round);
        });
    }

    #[test]
    fn drand_precompile_get_randomness_returns_bytes32() {
        new_test_ext().execute_with(|| {
            let round = 1000;
            let value = 1u8;
            Pulses::<Runtime>::insert(
                round,
                Pulse {
                    round: 1000,
                    randomness: BoundedVec::truncate_from(vec![value; 32]),
                    signature: BoundedVec::truncate_from(vec![0u8; 96]),
                },
            );
            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(DrandPrecompile::<Runtime>::INDEX);
            let input = get_randomness_call_data(round);

            let result =
                execute_precompile(&precompiles, precompile_addr, caller, input, U256::zero());
            let precompile_result =
                result.expect("expected precompile call to be routed to drand precompile");
            let output = precompile_result
                .expect("expected successful getRandomness call")
                .output;

            assert_eq!(
                output.len(),
                32,
                "getRandomness should return 32 bytes (bytes32)"
            );
            assert!(
                output.iter().all(|&b| b == value),
                "getRandomness for round 1000 should return the inserted randomness (32 bytes of 1s)"
            );

            let non_existent_round = 999_999_999u64;
            let input = get_randomness_call_data(non_existent_round);
            let result =
                execute_precompile(&precompiles, precompile_addr, caller, input, U256::zero());
            let precompile_result =
                result.expect("expected precompile call to be routed to drand precompile");
            let output = precompile_result
                .expect("expected successful getRandomness call")
                .output;
            assert!(output.iter().all(|&b| b == 0), "getRandomness for non-existent round should return 32 bytes of 0s");
        });
    }
}
