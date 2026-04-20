#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::arithmetic_side_effects)]

use core::iter::IntoIterator;
use std::collections::BTreeSet;

use fp_evm::{Context, ExitError, PrecompileFailure, PrecompileResult};
use node_subtensor_runtime::{BuildStorage, Runtime, RuntimeGenesisConfig, System};
use pallet_evm::{AddressMapping, BalanceConverter, PrecompileSet};
use precompile_utils::solidity::{codec::Address, encode_return_value, encode_with_selector};
use precompile_utils::testing::{MockHandle, PrecompileTesterExt};
use sp_core::{H160, H256, Pair, U256, ed25519};
use sp_runtime::traits::Hash;
use substrate_fixed::types::{I96F32, U96F32};
use subtensor_precompiles::{
    AddressMappingPrecompile, AlphaPrecompile, BalanceTransferPrecompile, Ed25519Verify,
    MetagraphPrecompile, NeuronPrecompile, PrecompileExt, Precompiles, UidLookupPrecompile,
};
use subtensor_runtime_common::{AlphaBalance, NetUid, NetUidStorageIndex, TaoBalance, Token};
use subtensor_swap_interface::{Order, SwapHandler};

type AccountId = <Runtime as frame_system::Config>::AccountId;

const TEST_NETUID_U16: u16 = 1;

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

/// Encodes one 32-byte ABI output word for exact raw return checks.
fn abi_word(value: U256) -> Vec<u8> {
    value.to_big_endian().to_vec()
}

/// Builds a 4-byte Solidity selector from a function signature.
fn selector_u32(signature: &str) -> u32 {
    let hash = sp_io::hashing::keccak_256(signature.as_bytes());
    u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]])
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

    #[test]
    fn address_mapping_precompile_returns_runtime_address_mapping() {
        new_test_ext().execute_with(|| {
            let precompiles = Precompiles::<Runtime>::new();

            let caller = addr_from_index(1);
            let target_address = addr_from_index(0x1234);
            let input = encode_with_selector(
                selector_u32("addressMapping(address)"),
                (Address(target_address),),
            );

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
                encode_with_selector(
                    selector_u32("addressMapping(address)"),
                    (Address(first_address),),
                ),
                U256::zero(),
            )
            .expect("expected precompile mapping call to be routed to a precompile")
            .expect("address mapping call should succeed")
            .output;
            let second_output = execute_precompile(
                &Precompiles::<Runtime>::new(),
                precompile_addr,
                caller,
                encode_with_selector(
                    selector_u32("addressMapping(address)"),
                    (Address(second_address),),
                ),
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
            let input = encode_with_selector(
                selector_u32("addressMapping(address)"),
                (Address(target_address),),
            );

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
                encode_with_selector(selector_u32("getAlphaPrice(uint16)"), (DYNAMIC_NETUID_U16,)),
                alpha_price_to_evm(alpha_price),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getMovingAlphaPrice(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                alpha_price_to_evm(moving_alpha_price),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(selector_u32("getTaoInPool(uint16)"), (DYNAMIC_NETUID_U16,)),
                U256::from(pallet_subtensor::SubnetTAO::<Runtime>::get(dynamic_netuid).to_u64()),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getAlphaInPool(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(u64::from(pallet_subtensor::SubnetAlphaIn::<Runtime>::get(
                    dynamic_netuid,
                ))),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getAlphaOutPool(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(u64::from(pallet_subtensor::SubnetAlphaOut::<Runtime>::get(
                    dynamic_netuid,
                ))),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getAlphaIssuance(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(u64::from(
                    pallet_subtensor::Pallet::<Runtime>::get_alpha_issuance(dynamic_netuid),
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getSubnetMechanism(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(pallet_subtensor::SubnetMechanism::<Runtime>::get(
                    dynamic_netuid,
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getEMAPriceHalvingBlocks(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(pallet_subtensor::EMAPriceHalvingBlocks::<Runtime>::get(
                    dynamic_netuid,
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getSubnetVolume(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(pallet_subtensor::SubnetVolume::<Runtime>::get(
                    dynamic_netuid,
                )),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getTaoInEmission(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(
                    pallet_subtensor::SubnetTaoInEmission::<Runtime>::get(dynamic_netuid).to_u64(),
                ),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getAlphaInEmission(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
                U256::from(
                    pallet_subtensor::SubnetAlphaInEmission::<Runtime>::get(dynamic_netuid)
                        .to_u64(),
                ),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("getAlphaOutEmission(uint16)"),
                    (DYNAMIC_NETUID_U16,),
                ),
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
                selector_u32("getCKBurn()").to_be_bytes().to_vec(),
                U256::from(pallet_subtensor::CKBurn::<Runtime>::get()),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                selector_u32("getTaoWeight()").to_be_bytes().to_vec(),
                U256::from(pallet_subtensor::TaoWeight::<Runtime>::get()),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                selector_u32("getRootNetuid()").to_be_bytes().to_vec(),
                U256::from(u16::from(NetUid::ROOT)),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                selector_u32("getSumAlphaPrice()").to_be_bytes().to_vec(),
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
                encode_with_selector(
                    selector_u32("simSwapTaoForAlpha(uint16,uint64)"),
                    (DYNAMIC_NETUID_U16, tao_amount),
                ),
                U256::from(expected_alpha),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("simSwapAlphaForTao(uint16,uint64)"),
                    (DYNAMIC_NETUID_U16, alpha_amount),
                ),
                U256::from(expected_tao),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("simSwapTaoForAlpha(uint16,uint64)"),
                    (DYNAMIC_NETUID_U16, 0_u64),
                ),
                U256::zero(),
            );
            assert_static_call(
                &precompiles,
                caller,
                precompile_addr,
                encode_with_selector(
                    selector_u32("simSwapAlphaForTao(uint16,uint64)"),
                    (DYNAMIC_NETUID_U16, 0_u64),
                ),
                U256::zero(),
            );
        });
    }
}

mod ed25519_verify {
    use super::*;

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
            let message = H256::from(message);
            let broken_message = H256::from(broken_message);
            let public_key = H256::from(public_key);
            let signature_r = H256::from_slice(&signature[..32]);
            let signature_s = H256::from_slice(&signature[32..]);
            let broken_signature_r = H256::from_slice(&broken_signature[..32]);
            let broken_signature_s = H256::from_slice(&broken_signature[32..]);

            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("verify(bytes32,bytes32,bytes32,bytes32)"),
                        (message, public_key, signature_r, signature_s),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::one()));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("verify(bytes32,bytes32,bytes32,bytes32)"),
                        (broken_message, public_key, signature_r, signature_s),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::zero()));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("verify(bytes32,bytes32,bytes32,bytes32)"),
                        (message, public_key, broken_signature_r, broken_signature_s),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::zero()));
        });
    }
}

mod uid_lookup {
    use super::*;

    #[test]
    fn uid_lookup_precompile_returns_associated_uid_and_block() {
        new_test_ext().execute_with(|| {
            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(UidLookupPrecompile::<Runtime>::INDEX);

            let netuid = NetUid::from(TEST_NETUID_U16);
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
                    encode_with_selector(
                        selector_u32("uidLookup(uint16,address,uint16)"),
                        (netuid_u16, Address(evm_address), limit),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(encode_return_value(expected));
        });
    }
}

mod metagraph {
    use super::*;

    const UID: u16 = 0;
    const EMISSION: u64 = 111;
    const VTRUST: u16 = 222;
    const LAST_UPDATE: u64 = 333;
    const AXON_BLOCK: u64 = 444;
    const AXON_VERSION: u32 = 555;
    const AXON_IP: u128 = 666;
    const AXON_PORT: u16 = 777;
    const AXON_IP_TYPE: u8 = 4;
    const AXON_PROTOCOL: u8 = 1;

    fn seed_metagraph_test_state() -> (NetUid, AccountId, AccountId, pallet_subtensor::AxonInfo) {
        let netuid = NetUid::from(TEST_NETUID_U16);
        let hotkey = pallet_subtensor::Keys::<Runtime>::get(netuid, UID);
        let coldkey = pallet_subtensor::Owner::<Runtime>::get(&hotkey);

        let axon = pallet_subtensor::AxonInfo {
            block: AXON_BLOCK,
            version: AXON_VERSION,
            ip: AXON_IP,
            port: AXON_PORT,
            ip_type: AXON_IP_TYPE,
            protocol: AXON_PROTOCOL,
            placeholder1: 0,
            placeholder2: 0,
        };

        pallet_subtensor::SubnetworkN::<Runtime>::insert(netuid, 1);
        pallet_subtensor::Emission::<Runtime>::insert(netuid, vec![AlphaBalance::from(EMISSION)]);
        pallet_subtensor::ValidatorTrust::<Runtime>::insert(netuid, vec![VTRUST]);
        pallet_subtensor::ValidatorPermit::<Runtime>::insert(netuid, vec![true]);
        pallet_subtensor::LastUpdate::<Runtime>::insert(
            NetUidStorageIndex::from(netuid),
            vec![LAST_UPDATE],
        );
        pallet_subtensor::Active::<Runtime>::insert(netuid, vec![true]);
        pallet_subtensor::Axons::<Runtime>::insert(netuid, &hotkey, axon.clone());

        (netuid, hotkey, coldkey, axon)
    }

    #[test]
    fn metagraph_precompile_matches_runtime_values() {
        new_test_ext().execute_with(|| {
            let (netuid, hotkey, coldkey, axon) = seed_metagraph_test_state();

            let precompiles = Precompiles::<Runtime>::new();
            let caller = addr_from_index(1);
            let precompile_addr = addr_from_index(MetagraphPrecompile::<Runtime>::INDEX);

            let uid_count = pallet_subtensor::SubnetworkN::<Runtime>::get(netuid);
            let emission =
                pallet_subtensor::Pallet::<Runtime>::get_emission_for_uid(netuid, UID).to_u64();
            let vtrust =
                pallet_subtensor::Pallet::<Runtime>::get_validator_trust_for_uid(netuid, UID);
            let validator_status =
                pallet_subtensor::Pallet::<Runtime>::get_validator_permit_for_uid(netuid, UID);
            let last_update = pallet_subtensor::Pallet::<Runtime>::get_last_update_for_uid(
                NetUidStorageIndex::from(netuid),
                UID,
            );
            let is_active = pallet_subtensor::Pallet::<Runtime>::get_active_for_uid(netuid, UID);
            let runtime_axon = pallet_subtensor::Pallet::<Runtime>::get_axon_info(netuid, &hotkey);

            assert_eq!(uid_count, 1);
            assert_eq!(emission, EMISSION);
            assert_eq!(vtrust, VTRUST);
            assert!(validator_status);
            assert_eq!(last_update, LAST_UPDATE);
            assert!(is_active);
            assert_eq!(runtime_axon, axon);

            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(selector_u32("getUidCount(uint16)"), (TEST_NETUID_U16,)),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::from(uid_count)));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getAxon(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(encode_return_value((
                    runtime_axon.block,
                    runtime_axon.version,
                    runtime_axon.ip,
                    runtime_axon.port,
                    runtime_axon.ip_type,
                    runtime_axon.protocol,
                )));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getEmission(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::from(emission)));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getVtrust(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::from(vtrust)));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getValidatorStatus(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::from(validator_status as u8)));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getLastUpdate(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::from(last_update)));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getIsActive(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(abi_word(U256::from(is_active as u8)));
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getHotkey(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(H256::from_slice(hotkey.as_ref()).as_bytes().to_vec());
            precompiles
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("getColdkey(uint16,uint16)"),
                        (TEST_NETUID_U16, UID),
                    ),
                )
                .with_static_call(true)
                .execute_returns_raw(H256::from_slice(coldkey.as_ref()).as_bytes().to_vec());
        });
    }
}

mod neuron {
    use super::*;

    const REGISTRATION_BURN: u64 = 1_000;
    const RESERVE: u64 = 1_000_000_000;
    const COLDKEY_BALANCE: u64 = 50_000;
    const TEMPO: u16 = 100;
    const REVEAL_PERIOD: u64 = 1;
    const VERSION_KEY: u64 = 0;
    const REGISTERED_UID: u16 = 1;
    const REVEAL_UIDS: [u16; 1] = [REGISTERED_UID];
    const REVEAL_VALUES: [u16; 1] = [5];
    const REVEAL_SALT: [u16; 1] = [9];

    fn setup_registered_caller(caller: H160) -> (NetUid, AccountId) {
        let netuid = NetUid::from(TEST_NETUID_U16);
        let caller_account =
            <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(caller);
        let caller_hotkey = H256::from_slice(caller_account.as_ref());

        pallet_subtensor::Pallet::<Runtime>::set_network_registration_allowed(netuid, true);
        pallet_subtensor::Pallet::<Runtime>::set_burn(netuid, REGISTRATION_BURN.into());
        pallet_subtensor::Pallet::<Runtime>::set_max_allowed_uids(netuid, 4096);
        pallet_subtensor::Pallet::<Runtime>::set_weights_set_rate_limit(netuid, 0);
        pallet_subtensor::Pallet::<Runtime>::set_tempo(netuid, TEMPO);
        pallet_subtensor::Pallet::<Runtime>::set_commit_reveal_weights_enabled(netuid, true);
        pallet_subtensor::Pallet::<Runtime>::set_reveal_period(netuid, REVEAL_PERIOD)
            .expect("reveal period setup should succeed");
        pallet_subtensor::SubnetTAO::<Runtime>::insert(netuid, TaoBalance::from(RESERVE));
        pallet_subtensor::SubnetAlphaIn::<Runtime>::insert(netuid, AlphaBalance::from(RESERVE));
        pallet_subtensor::Pallet::<Runtime>::add_balance_to_coldkey_account(
            &caller_account,
            COLDKEY_BALANCE.into(),
        );

        Precompiles::<Runtime>::new()
            .prepare_test(
                caller,
                addr_from_index(NeuronPrecompile::<Runtime>::INDEX),
                encode_with_selector(
                    selector_u32("burnedRegister(uint16,bytes32)"),
                    (TEST_NETUID_U16, caller_hotkey),
                ),
            )
            .execute_returns(());

        let registered_uid = pallet_subtensor::Pallet::<Runtime>::get_uid_for_net_and_hotkey(
            netuid,
            &caller_account,
        )
        .expect("caller should be registered on subnet");
        assert_eq!(registered_uid, REGISTERED_UID);

        (netuid, caller_account)
    }

    fn reveal_commit_hash(caller_account: &AccountId, netuid: NetUid) -> H256 {
        <Runtime as frame_system::Config>::Hashing::hash_of(&(
            caller_account.clone(),
            NetUidStorageIndex::from(netuid),
            REVEAL_UIDS.as_slice(),
            REVEAL_VALUES.as_slice(),
            REVEAL_SALT.as_slice(),
            VERSION_KEY,
        ))
    }

    #[test]
    fn neuron_precompile_burned_register_adds_a_new_uid_and_key() {
        new_test_ext().execute_with(|| {
            let netuid = NetUid::from(TEST_NETUID_U16);
            let caller = addr_from_index(0x1234);
            let caller_account =
                <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(caller);
            let hotkey_account = AccountId::from([0x42; 32]);
            let hotkey = H256::from_slice(hotkey_account.as_ref());

            pallet_subtensor::Pallet::<Runtime>::set_network_registration_allowed(netuid, true);
            pallet_subtensor::Pallet::<Runtime>::set_burn(netuid, REGISTRATION_BURN.into());
            pallet_subtensor::Pallet::<Runtime>::set_max_allowed_uids(netuid, 4096);
            pallet_subtensor::SubnetTAO::<Runtime>::insert(netuid, TaoBalance::from(RESERVE));
            pallet_subtensor::SubnetAlphaIn::<Runtime>::insert(netuid, AlphaBalance::from(RESERVE));
            pallet_subtensor::Pallet::<Runtime>::add_balance_to_coldkey_account(
                &caller_account,
                COLDKEY_BALANCE.into(),
            );

            let uid_before = pallet_subtensor::SubnetworkN::<Runtime>::get(netuid);
            let balance_before =
                pallet_subtensor::Pallet::<Runtime>::get_coldkey_balance(&caller_account).to_u64();

            Precompiles::<Runtime>::new()
                .prepare_test(
                    caller,
                    addr_from_index(NeuronPrecompile::<Runtime>::INDEX),
                    encode_with_selector(
                        selector_u32("burnedRegister(uint16,bytes32)"),
                        (TEST_NETUID_U16, hotkey),
                    ),
                )
                .execute_returns(());

            let uid_after = pallet_subtensor::SubnetworkN::<Runtime>::get(netuid);
            let registered_hotkey = pallet_subtensor::Keys::<Runtime>::get(netuid, uid_before);
            let owner = pallet_subtensor::Owner::<Runtime>::get(&hotkey_account);
            let balance_after =
                pallet_subtensor::Pallet::<Runtime>::get_coldkey_balance(&caller_account).to_u64();

            assert_eq!(uid_after, uid_before + 1);
            assert_eq!(registered_hotkey, hotkey_account);
            assert_eq!(owner, caller_account);
            assert!(balance_after < balance_before);
        });
    }

    #[test]
    fn neuron_precompile_commit_weights_respects_stake_threshold_and_stores_commit() {
        new_test_ext().execute_with(|| {
            let caller = addr_from_index(0x2234);
            let (netuid, caller_account) = setup_registered_caller(caller);
            let commit_hash = reveal_commit_hash(&caller_account, netuid);
            let precompile_addr = addr_from_index(NeuronPrecompile::<Runtime>::INDEX);

            pallet_subtensor::Pallet::<Runtime>::set_stake_threshold(1);
            let rejected = execute_precompile(
                &Precompiles::<Runtime>::new(),
                precompile_addr,
                caller,
                encode_with_selector(
                    selector_u32("commitWeights(uint16,bytes32)"),
                    (TEST_NETUID_U16, commit_hash),
                ),
                U256::zero(),
            )
            .expect("commit weights should route to neuron precompile");
            assert!(rejected.is_err());

            pallet_subtensor::Pallet::<Runtime>::set_stake_threshold(0);
            Precompiles::<Runtime>::new()
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("commitWeights(uint16,bytes32)"),
                        (TEST_NETUID_U16, commit_hash),
                    ),
                )
                .execute_returns(());

            let commits = pallet_subtensor::WeightCommits::<Runtime>::get(
                NetUidStorageIndex::from(netuid),
                &caller_account,
            )
            .expect("weight commits should be stored after successful commit");
            assert_eq!(commits.len(), 1);
        });
    }

    #[test]
    fn neuron_precompile_reveal_weights_respects_stake_threshold_and_sets_weights() {
        new_test_ext().execute_with(|| {
            let caller = addr_from_index(0x3234);
            let (netuid, caller_account) = setup_registered_caller(caller);
            let commit_hash = reveal_commit_hash(&caller_account, netuid);
            let precompile_addr = addr_from_index(NeuronPrecompile::<Runtime>::INDEX);

            Precompiles::<Runtime>::new()
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("commitWeights(uint16,bytes32)"),
                        (TEST_NETUID_U16, commit_hash),
                    ),
                )
                .execute_returns(());

            let commits = pallet_subtensor::WeightCommits::<Runtime>::get(
                NetUidStorageIndex::from(netuid),
                &caller_account,
            )
            .expect("weight commit should exist before reveal");
            let (_, _, first_reveal_block, _) = commits
                .front()
                .copied()
                .expect("weight commit queue should contain the committed hash");

            System::set_block_number(
                u32::try_from(first_reveal_block)
                    .expect("first reveal block should fit in runtime block number"),
            );

            pallet_subtensor::Pallet::<Runtime>::set_stake_threshold(1);
            let rejected = execute_precompile(
                &Precompiles::<Runtime>::new(),
                precompile_addr,
                caller,
                encode_with_selector(
                    selector_u32("revealWeights(uint16,uint16[],uint16[],uint16[],uint64)"),
                    (
                        TEST_NETUID_U16,
                        REVEAL_UIDS.to_vec(),
                        REVEAL_VALUES.to_vec(),
                        REVEAL_SALT.to_vec(),
                        VERSION_KEY,
                    ),
                ),
                U256::zero(),
            )
            .expect("reveal weights should route to neuron precompile");
            assert!(rejected.is_err());

            pallet_subtensor::Pallet::<Runtime>::set_stake_threshold(0);
            Precompiles::<Runtime>::new()
                .prepare_test(
                    caller,
                    precompile_addr,
                    encode_with_selector(
                        selector_u32("revealWeights(uint16,uint16[],uint16[],uint16[],uint64)"),
                        (
                            TEST_NETUID_U16,
                            REVEAL_UIDS.to_vec(),
                            REVEAL_VALUES.to_vec(),
                            REVEAL_SALT.to_vec(),
                            VERSION_KEY,
                        ),
                    ),
                )
                .execute_returns(());

            assert!(
                pallet_subtensor::WeightCommits::<Runtime>::get(
                    NetUidStorageIndex::from(netuid),
                    &caller_account,
                )
                .is_none()
            );

            let neuron_uid = pallet_subtensor::Pallet::<Runtime>::get_uid_for_net_and_hotkey(
                netuid,
                &caller_account,
            )
            .expect("caller should remain registered after reveal");
            let weights = pallet_subtensor::Weights::<Runtime>::get(
                NetUidStorageIndex::from(netuid),
                neuron_uid,
            );

            assert_eq!(weights.len(), 1);
            assert_eq!(weights[0].0, neuron_uid);
            assert!(weights[0].1 > 0);
        });
    }
}

mod balance_transfer {
    use super::*;

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
                encode_with_selector(selector_u32("transfer(bytes32)"), (destination_raw,)),
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
                encode_with_selector(selector_u32("transfer(bytes32)"), (destination_raw,)),
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
