#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::arithmetic_side_effects)]

use core::iter::IntoIterator;
use std::collections::BTreeSet;

use fp_evm::{Context, ExitError, PrecompileFailure, PrecompileResult};
use node_subtensor_runtime::{BuildStorage, Runtime, RuntimeGenesisConfig, System};
use pallet_evm::{AddressMapping, BalanceConverter, PrecompileSet};
use precompile_utils::testing::{MockHandle, PrecompileTesterExt};
use sp_core::{H160, H256, Pair, U256, ed25519};
use sp_runtime::traits::Hash;
use substrate_fixed::types::{I96F32, U96F32};
use subtensor_precompiles::{
    AddressMappingPrecompile, AlphaPrecompile, BalanceTransferPrecompile, Ed25519Verify,
    PrecompileExt, Precompiles, UidLookupPrecompile,
};
use subtensor_runtime_common::{AlphaBalance, NetUid, TaoBalance, Token};
use subtensor_swap_interface::{Order, SwapHandler};

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

/// Appends one 32-byte ABI word to manually encoded precompile input.
fn push_abi_word(input: &mut Vec<u8>, value: U256) {
    input.extend_from_slice(&value.to_big_endian());
}

/// Encodes one 32-byte ABI output word for exact raw return checks.
fn abi_word(value: U256) -> Vec<u8> {
    value.to_big_endian().to_vec()
}

/// Builds a 4-byte Solidity selector from a function signature.
fn selector(signature: &str) -> [u8; 4] {
    let hash = sp_io::hashing::keccak_256(signature.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

/// Encodes a selector-only call with no arguments.
fn call_data_no_args(signature: &str) -> Vec<u8> {
    selector(signature).to_vec()
}

/// Encodes a selector plus one uint16 ABI argument.
fn call_data_u16(signature: &str, value: u16) -> Vec<u8> {
    // 4-byte selector + 1 ABI word for the uint16 argument.
    let mut input = Vec::with_capacity(4 + 32);
    input.extend_from_slice(&selector(signature));
    push_abi_word(&mut input, U256::from(value));
    input
}

/// Encodes a selector plus `(uint16,uint64)` ABI arguments.
fn call_data_u16_u64(signature: &str, first: u16, second: u64) -> Vec<u8> {
    // 4-byte selector + 2 ABI words for `(uint16,uint64)`.
    let mut input = Vec::with_capacity(4 + 64);
    input.extend_from_slice(&selector(signature));
    push_abi_word(&mut input, U256::from(first));
    push_abi_word(&mut input, U256::from(second));
    input
}

/// Matches the alpha precompile conversion from fixed-point price to EVM `uint256`.
fn alpha_price_to_evm(price: U96F32) -> U256 {
    let scaled_price = (price * U96F32::from_num(1_000_000_000)).to_num::<u64>();
    <Runtime as pallet_evm::Config>::BalanceConverter::into_evm_balance(scaled_price.into())
        .expect("runtime balance conversion should work for alpha price")
        .into_u256()
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
        // 4-byte selector + 1 ABI word for the address argument.
        let mut input = Vec::with_capacity(4 + 32);
        input.extend_from_slice(&selector("addressMapping(address)"));
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

    #[test]
    fn address_mapping_precompile_maps_distinct_addresses_to_distinct_accounts() {
        new_test_ext().execute_with(|| {
            let caller = addr_from_index(1);
            let first_address = addr_from_index(0x1234);
            let second_address = addr_from_index(0x5678);
            let precompile_addr = addr_from_index(AddressMappingPrecompile::<Runtime>::INDEX);

            let first_output = execute_precompile(
                &Precompiles::<Runtime>::new(),
                precompile_addr,
                caller,
                address_mapping_call_data(first_address),
                U256::zero(),
            )
            .expect("expected precompile mapping call to be routed to a precompile")
            .expect("address mapping call should succeed")
            .output;
            let second_output = execute_precompile(
                &Precompiles::<Runtime>::new(),
                precompile_addr,
                caller,
                address_mapping_call_data(second_address),
                U256::zero(),
            )
            .expect("expected precompile mapping call to be routed to a precompile")
            .expect("address mapping call should succeed")
            .output;

            assert_ne!(first_output, second_output);
        });
    }

    #[test]
    fn address_mapping_precompile_is_deterministic() {
        new_test_ext().execute_with(|| {
            let caller = addr_from_index(1);
            let target_address = addr_from_index(0x1234);
            let precompile_addr = addr_from_index(AddressMappingPrecompile::<Runtime>::INDEX);
            let input = address_mapping_call_data(target_address);

            let first_output = execute_precompile(
                &Precompiles::<Runtime>::new(),
                precompile_addr,
                caller,
                input.clone(),
                U256::zero(),
            )
            .expect("expected precompile mapping call to be routed to a precompile")
            .expect("address mapping call should succeed")
            .output;
            let second_output = execute_precompile(
                &Precompiles::<Runtime>::new(),
                precompile_addr,
                caller,
                input,
                U256::zero(),
            )
            .expect("expected precompile mapping call to be routed to a precompile")
            .expect("address mapping call should succeed")
            .output;

            assert_eq!(first_output, second_output);
        });
    }
}

mod alpha {
    use super::*;

    const DYNAMIC_NETUID_U16: u16 = 1;
    const SUM_PRICE_NETUID_U16: u16 = 2;
    const TAO_WEIGHT: u64 = 444;
    const CK_BURN: u64 = 555;
    const EMA_HALVING_BLOCKS: u64 = 777;
    const SUBNET_VOLUME: u128 = 888;
    const TAO_IN_EMISSION: u64 = 111;
    const ALPHA_IN_EMISSION: u64 = 222;
    const ALPHA_OUT_EMISSION: u64 = 333;

    fn dynamic_netuid() -> NetUid {
        NetUid::from(DYNAMIC_NETUID_U16)
    }

    fn sum_price_netuid() -> NetUid {
        NetUid::from(SUM_PRICE_NETUID_U16)
    }

    fn seed_alpha_test_state() {
        let dynamic_netuid = dynamic_netuid();
        let sum_price_netuid = sum_price_netuid();

        pallet_subtensor::TaoWeight::<Runtime>::put(TAO_WEIGHT);
        pallet_subtensor::CKBurn::<Runtime>::put(CK_BURN);

        pallet_subtensor::NetworksAdded::<Runtime>::insert(dynamic_netuid, true);
        pallet_subtensor::SubnetMechanism::<Runtime>::insert(dynamic_netuid, 1);
        pallet_subtensor::SubnetTAO::<Runtime>::insert(
            dynamic_netuid,
            TaoBalance::from(20_000_000_000_u64),
        );
        pallet_subtensor::SubnetAlphaIn::<Runtime>::insert(
            dynamic_netuid,
            AlphaBalance::from(10_000_000_000_u64),
        );
        pallet_subtensor::SubnetAlphaOut::<Runtime>::insert(
            dynamic_netuid,
            AlphaBalance::from(3_000_000_000_u64),
        );
        pallet_subtensor::SubnetTaoInEmission::<Runtime>::insert(
            dynamic_netuid,
            TaoBalance::from(TAO_IN_EMISSION),
        );
        pallet_subtensor::SubnetAlphaInEmission::<Runtime>::insert(
            dynamic_netuid,
            AlphaBalance::from(ALPHA_IN_EMISSION),
        );
        pallet_subtensor::SubnetAlphaOutEmission::<Runtime>::insert(
            dynamic_netuid,
            AlphaBalance::from(ALPHA_OUT_EMISSION),
        );
        pallet_subtensor::SubnetVolume::<Runtime>::insert(dynamic_netuid, SUBNET_VOLUME);
        pallet_subtensor::EMAPriceHalvingBlocks::<Runtime>::insert(
            dynamic_netuid,
            EMA_HALVING_BLOCKS,
        );
        pallet_subtensor::SubnetMovingPrice::<Runtime>::insert(
            dynamic_netuid,
            I96F32::from_num(3.0 / 2.0),
        );

        pallet_subtensor::NetworksAdded::<Runtime>::insert(sum_price_netuid, true);
        pallet_subtensor::SubnetMechanism::<Runtime>::insert(sum_price_netuid, 1);
        pallet_subtensor::SubnetTAO::<Runtime>::insert(
            sum_price_netuid,
            TaoBalance::from(5_000_000_000_u64),
        );
        pallet_subtensor::SubnetAlphaIn::<Runtime>::insert(
            sum_price_netuid,
            AlphaBalance::from(10_000_000_000_u64),
        );
    }

    fn assert_static_call(
        precompiles: &Precompiles<Runtime>,
        caller: H160,
        precompile_addr: H160,
        input: Vec<u8>,
        expected: U256,
    ) {
        precompiles
            .prepare_test(caller, precompile_addr, input)
            .with_static_call(true)
            .execute_returns_raw(abi_word(expected));
    }

    #[test]
    fn alpha_precompile_matches_runtime_values_for_dynamic_subnet() {
        new_test_ext().execute_with(|| {
            seed_alpha_test_state();

            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(AlphaPrecompile::<Runtime>::INDEX);

            let dynamic_netuid = dynamic_netuid();
            let alpha_price =
                <pallet_subtensor_swap::Pallet<Runtime> as SwapHandler>::current_alpha_price(
                    dynamic_netuid,
                );
            let moving_alpha_price =
                pallet_subtensor::Pallet::<Runtime>::get_moving_alpha_price(dynamic_netuid);

            assert!(alpha_price > U96F32::from_num(1));
            assert!(moving_alpha_price > U96F32::from_num(1));

            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getAlphaPrice(uint16)", DYNAMIC_NETUID_U16),
                alpha_price_to_evm(alpha_price),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getMovingAlphaPrice(uint16)", DYNAMIC_NETUID_U16),
                alpha_price_to_evm(moving_alpha_price),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getTaoInPool(uint16)", DYNAMIC_NETUID_U16),
                U256::from(pallet_subtensor::SubnetTAO::<Runtime>::get(dynamic_netuid).to_u64()),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getAlphaInPool(uint16)", DYNAMIC_NETUID_U16),
                U256::from(u64::from(pallet_subtensor::SubnetAlphaIn::<Runtime>::get(
                    dynamic_netuid,
                ))),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getAlphaOutPool(uint16)", DYNAMIC_NETUID_U16),
                U256::from(u64::from(pallet_subtensor::SubnetAlphaOut::<Runtime>::get(
                    dynamic_netuid,
                ))),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getAlphaIssuance(uint16)", DYNAMIC_NETUID_U16),
                U256::from(u64::from(
                    pallet_subtensor::Pallet::<Runtime>::get_alpha_issuance(dynamic_netuid),
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getSubnetMechanism(uint16)", DYNAMIC_NETUID_U16),
                U256::from(pallet_subtensor::SubnetMechanism::<Runtime>::get(
                    dynamic_netuid,
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getEMAPriceHalvingBlocks(uint16)", DYNAMIC_NETUID_U16),
                U256::from(pallet_subtensor::EMAPriceHalvingBlocks::<Runtime>::get(
                    dynamic_netuid,
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getSubnetVolume(uint16)", DYNAMIC_NETUID_U16),
                U256::from(pallet_subtensor::SubnetVolume::<Runtime>::get(
                    dynamic_netuid,
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getTaoInEmission(uint16)", DYNAMIC_NETUID_U16),
                U256::from(
                    pallet_subtensor::SubnetTaoInEmission::<Runtime>::get(dynamic_netuid).to_u64(),
                ),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getAlphaInEmission(uint16)", DYNAMIC_NETUID_U16),
                U256::from(
                    pallet_subtensor::SubnetAlphaInEmission::<Runtime>::get(dynamic_netuid)
                        .to_u64(),
                ),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16("getAlphaOutEmission(uint16)", DYNAMIC_NETUID_U16),
                U256::from(
                    pallet_subtensor::SubnetAlphaOutEmission::<Runtime>::get(dynamic_netuid)
                        .to_u64(),
                ),
            );
        });
    }

    #[test]
    fn alpha_precompile_matches_runtime_global_values() {
        new_test_ext().execute_with(|| {
            seed_alpha_test_state();

            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(AlphaPrecompile::<Runtime>::INDEX);

            let mut sum_alpha_price = U96F32::from_num(0);
            for (netuid, _) in pallet_subtensor::NetworksAdded::<Runtime>::iter() {
                if netuid.is_root() {
                    continue;
                }
                let price =
                    <pallet_subtensor_swap::Pallet<Runtime> as SwapHandler>::current_alpha_price(
                        netuid,
                    );
                if price < U96F32::from_num(1) {
                    sum_alpha_price += price;
                }
            }

            assert!(sum_alpha_price > U96F32::from_num(0));

            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_no_args("getCKBurn()"),
                U256::from(pallet_subtensor::CKBurn::<Runtime>::get()),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_no_args("getTaoWeight()"),
                U256::from(pallet_subtensor::TaoWeight::<Runtime>::get()),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_no_args("getRootNetuid()"),
                U256::from(u16::from(NetUid::ROOT)),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_no_args("getSumAlphaPrice()"),
                alpha_price_to_evm(sum_alpha_price),
            );
        });
    }

    #[test]
    fn alpha_precompile_matches_runtime_swap_simulations() {
        new_test_ext().execute_with(|| {
            seed_alpha_test_state();

            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(AlphaPrecompile::<Runtime>::INDEX);

            let tao_amount = 1_000_000_000_u64;
            let alpha_amount = 1_000_000_000_u64;
            let expected_alpha = <pallet_subtensor_swap::Pallet<Runtime> as SwapHandler>::sim_swap(
                dynamic_netuid(),
                pallet_subtensor::GetAlphaForTao::<Runtime>::with_amount(tao_amount),
            )
            .expect("tao-for-alpha simulation should succeed")
            .amount_paid_out
            .to_u64();
            let expected_tao = <pallet_subtensor_swap::Pallet<Runtime> as SwapHandler>::sim_swap(
                dynamic_netuid(),
                pallet_subtensor::GetTaoForAlpha::<Runtime>::with_amount(alpha_amount),
            )
            .expect("alpha-for-tao simulation should succeed")
            .amount_paid_out
            .to_u64();

            assert!(expected_alpha > 0);
            assert!(expected_tao > 0);

            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16_u64(
                    "simSwapTaoForAlpha(uint16,uint64)",
                    DYNAMIC_NETUID_U16,
                    tao_amount,
                ),
                U256::from(expected_alpha),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16_u64(
                    "simSwapAlphaForTao(uint16,uint64)",
                    DYNAMIC_NETUID_U16,
                    alpha_amount,
                ),
                U256::from(expected_tao),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16_u64("simSwapTaoForAlpha(uint16,uint64)", DYNAMIC_NETUID_U16, 0),
                U256::zero(),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                call_data_u16_u64("simSwapAlphaForTao(uint16,uint64)", DYNAMIC_NETUID_U16, 0),
                U256::zero(),
            );
        });
    }
}

mod ed25519_verify {
    use super::*;

    fn ed25519_verify_call_data(
        message: [u8; 32],
        public_key: [u8; 32],
        signature: [u8; 64],
    ) -> Vec<u8> {
        // 4-byte selector + 4 ABI words: message, public key, signature R, signature S.
        let mut input = Vec::with_capacity(4 + 32 * 4);
        input.extend_from_slice(&selector("verify(bytes32,bytes32,bytes32,bytes32)"));
        input.extend_from_slice(&message);
        input.extend_from_slice(&public_key);
        input.extend_from_slice(&signature[..32]);
        input.extend_from_slice(&signature[32..]);
        input
    }

    #[test]
    fn ed25519_precompile_verifies_valid_and_invalid_signatures() {
        new_test_ext().execute_with(|| {
            let pair = ed25519::Pair::from_string("//Alice", None)
                .expect("Alice ed25519 key should be available");
            let message = sp_io::hashing::keccak_256(b"Sign this message");
            let signature = pair.sign(&message).0;
            let public_key = pair.public().0;

            let mut broken_message = message;
            broken_message[0] ^= 0x01;

            let mut broken_signature = signature;
            broken_signature[0] ^= 0x01;

            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(Ed25519Verify::<AccountId>::INDEX);

            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    ed25519_verify_call_data(message, public_key, signature),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::one()));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    ed25519_verify_call_data(broken_message, public_key, signature),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::zero()));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    ed25519_verify_call_data(message, public_key, broken_signature),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::zero()));
        });
    }
}

mod uid_lookup {
    use super::*;

    fn uid_lookup_call_data(netuid: u16, evm_address: H160, limit: u16) -> Vec<u8> {
        // 4-byte selector + 3 ABI words: netuid, address, limit.
        let mut input = Vec::with_capacity(4 + 32 * 3);
        input.extend_from_slice(&selector("uidLookup(uint16,address,uint16)"));
        push_abi_word(&mut input, U256::from(netuid));
        input.extend_from_slice(&[0u8; 12]);
        input.extend_from_slice(evm_address.as_bytes());
        push_abi_word(&mut input, U256::from(limit));
        input
    }

    fn abi_uid_lookup_output(entries: &[(u16, u64)]) -> Vec<u8> {
        // ABI dynamic array encoding:
        // head offset word + array length word + 2 words per tuple entry `(uid, block_associated)`.
        let mut output = Vec::with_capacity(64 + entries.len() * 64);
        push_abi_word(&mut output, U256::from(32u64));
        push_abi_word(&mut output, U256::from(entries.len()));
        for (uid, block_associated) in entries {
            push_abi_word(&mut output, U256::from(*uid));
            push_abi_word(&mut output, U256::from(*block_associated));
        }
        output
    }

    #[test]
    fn uid_lookup_precompile_returns_associated_uid_and_block() {
        new_test_ext().execute_with(|| {
            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(UidLookupPrecompile::<Runtime>::INDEX);

            let netuid = NetUid::from(1);
            let netuid_u16: u16 = netuid.into();
            let uid = 0u16;
            let evm_address = H160::from_low_u64_be(0xdead_beef);
            let block_associated = 42u64;
            let limit = 1024u16;

            pallet_subtensor::AssociatedEvmAddress::<Runtime>::insert(
                netuid,
                uid,
                (evm_address, block_associated),
            );

            let expected =
                pallet_subtensor::Pallet::<Runtime>::uid_lookup(netuid, evm_address, limit);
            assert_eq!(expected, vec![(uid, block_associated)]);

            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    uid_lookup_call_data(netuid_u16, evm_address, limit),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_uid_lookup_output(&expected));
        });
    }
}

mod balance_transfer {
    use super::*;

    fn balance_transfer_call_data(target: H256) -> Vec<u8> {
        // 4-byte selector + 1 ABI word for the bytes32 destination.
        let mut input = Vec::with_capacity(4 + 32);
        input.extend_from_slice(&selector("transfer(bytes32)"));
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

            assert_eq!(source_balance_after, source_balance_before - amount.into());
            assert_eq!(
                destination_balance_after,
                destination_balance_before + amount.into()
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
