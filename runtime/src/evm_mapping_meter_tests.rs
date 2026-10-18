//! Exercise pre-read charging through the actual pinned EVM executor.
#![allow(clippy::unwrap_used, clippy::arithmetic_side_effects)]
use evm::{
    ExitError, Opcode,
    executor::stack::{StackExecutor, StackSubstateMetadata},
};
use fp_evm::{Vicinity, WeightInfo};
use frame_support::{derive_impl, parameter_types, weights::Weight};
use pallet_evm::{AddressMapping, runner::stack::SubstrateStackState};
use sp_core::{H160, H256, U256};

frame_support::construct_runtime! {
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        EVM: pallet_evm,
    }
}
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Test {
    type Block = frame_system::mocking::MockBlock<Self>;
    type AccountData = pallet_balances::AccountData<u64>;
}
#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}
parameter_types! {
    pub static MappingReadWeight: Weight = Weight::zero();
    pub static MappingReadCount: u64 = 0;
}
pub struct CountingAddressMapping;
pub struct TestTime;
impl frame_support::traits::Time for TestTime {
    type Moment = u64;
    fn now() -> Self::Moment {
        0
    }
}
impl AddressMapping<sp_runtime::AccountId32> for CountingAddressMapping {
    fn into_account_id(address: H160) -> sp_runtime::AccountId32 {
        MappingReadCount::set(MappingReadCount::get().saturating_add(1));
        pallet_evm::HashedAddressMapping::<sp_runtime::traits::BlakeTwo256>::into_account_id(
            address,
        )
    }
    fn extra_read_weight() -> Weight {
        MappingReadWeight::get()
    }
}
#[derive_impl(pallet_evm::config_preludes::TestDefaultConfig)]
impl pallet_evm::Config for Test {
    type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
    type AddressMapping = CountingAddressMapping;
    type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
    type Currency = Balances;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type Timestamp = TestTime;
    type BalanceConverter = crate::SubtensorEvmBalanceConverter;
}

#[cfg(test)]
mod mapping_precharge_tests {

    use super::*;
    use evm::{Context, Handler, Stack};

    fn check(
        opcode: Opcode,
        gas: u64,
        limit: Weight,
    ) -> (Result<(), ExitError>, u64, Option<WeightInfo>) {
        let vicinity = Vicinity::default();
        let config = <Test as pallet_evm::Config>::config();
        let metadata = StackSubstateMetadata::new(gas, config);
        let weight_info = WeightInfo::new_from_weight_limit(Some(limit), Some(0)).unwrap();
        let state = SubstrateStackState::<Test>::new(&vicinity, metadata, weight_info, None);
        let precompiles = ();
        let mut executor = StackExecutor::new_with_precompiles(state, config, &precompiles);
        let mut stack = Stack::new(1024);
        // Seven zero operands suffice for every CALL variant; SELFDESTRUCT uses the top.
        for _ in 0..7 {
            stack.push(H256::zero()).unwrap();
        }
        MappingReadCount::set(0);
        let result = executor.pre_validate(
            &Context {
                address: H160::repeat_byte(9),
                caller: H160::repeat_byte(8),
                apparent_value: U256::zero(),
            },
            opcode,
            &stack,
        );
        let info = executor.state().weight_info();
        (result, MappingReadCount::get(), info)
    }

    #[test]
    fn failing_dynamic_opcodes_charge_mapping_before_reading_storage() {
        sp_io::TestExternalities::default().execute_with(|| {
            let read = Weight::from_parts(1_000_000, 32);
            MappingReadWeight::set(read);
            let cost = pallet_evm::account_cost::gas::<Test>(read);
            assert!(cost > 0);
            for opcode in [
                Opcode::CALL,
                Opcode::CALLCODE,
                Opcode::DELEGATECALL,
                Opcode::STATICCALL,
                Opcode::SUICIDE,
            ] {
                // Neither gas, ref-time nor proof-size exhaustion may expose the read.
                for (gas, limit) in [
                    (cost - 1, Weight::MAX),
                    (u64::MAX, Weight::from_parts(read.ref_time() - 1, u64::MAX)),
                    (
                        u64::MAX,
                        Weight::from_parts(u64::MAX, read.proof_size() - 1),
                    ),
                    (
                        u64::MAX,
                        Weight::from_parts(
                            u64::MAX,
                            read.proof_size() + fp_evm::ACCOUNT_BASIC_PROOF_SIZE - 1,
                        ),
                    ),
                ] {
                    let (result, reads, _) = check(opcode, gas, limit);
                    assert_eq!(result, Err(ExitError::OutOfGas), "{opcode:?}");
                    assert_eq!(reads, 0, "{opcode:?}");
                }
            }
            // Source balance may be read only after its own charge, even if the
            // second (target existence) lookup cannot be afforded.
            let (result, reads, info) = check(Opcode::SUICIDE, cost, Weight::MAX);
            assert_eq!(result, Err(ExitError::OutOfGas));
            assert_eq!(reads, 1);
            assert_eq!(info.unwrap().ref_time_usage, Some(read.ref_time()));
            // Both early reads remain paid when ordinary opcode gas then fails.
            let (result, reads, info) = check(Opcode::SUICIDE, cost * 2, Weight::MAX);
            assert_eq!(result, Err(ExitError::OutOfGas));
            assert_eq!(reads, 2);
            assert_eq!(info.unwrap().ref_time_usage, Some(read.ref_time() * 2));
            MappingReadWeight::set(Weight::zero());
        });
    }
    fn execute_parent(
        code: Vec<u8>,
        child_code: Vec<u8>,
        funded: bool,
    ) -> (evm::ExitReason, Vec<u8>, u64) {
        use frame_support::traits::Currency;
        use std::rc::Rc;
        let parent = H160::repeat_byte(9);
        let child = H160::repeat_byte(7);
        pallet_evm::AccountCodes::<Test>::insert(child, child_code);
        if funded {
            let who = CountingAddressMapping::into_account_id(parent);
            let _ = Balances::make_free_balance_be(&who, 1_000_000);
        }
        let vicinity = Vicinity::default();
        let config = <Test as pallet_evm::Config>::config();
        let metadata = StackSubstateMetadata::new(1_000_000, config);
        let state = SubstrateStackState::<Test>::new(&vicinity, metadata, None, None);
        let precompiles = ();
        let mut executor = StackExecutor::new_with_precompiles(state, config, &precompiles);
        let mut runtime = evm::Runtime::new(
            Rc::new(code),
            Rc::new(Vec::new()),
            Context {
                address: parent,
                caller: H160::repeat_byte(8),
                apparent_value: U256::zero(),
            },
            config.stack_limit,
            config.memory_limit,
        );
        MappingReadCount::set(0);
        let result = executor.execute(&mut runtime);
        (
            result,
            runtime.machine().return_value(),
            MappingReadCount::get(),
        )
    }

    fn push_call(code: &mut Vec<u8>, gas: u16, value: u32) {
        // out size/offset, input size/offset, value, target, gas.
        code.extend_from_slice(&[0x60, 0, 0x60, 0, 0x60, 0, 0x60, 0, 0x63]);
        code.extend_from_slice(&value.to_be_bytes());
        code.push(0x73);
        code.extend_from_slice(H160::repeat_byte(7).as_bytes());
        code.push(0x61);
        code.extend_from_slice(&gas.to_be_bytes());
        code.push(0xf1);
    }

    #[test]
    fn caught_low_gas_children_cannot_repeat_unpaid_selfdestruct_reads() {
        for attempts in [1, 8, 64] {
            sp_io::TestExternalities::default().execute_with(|| {
                let read = Weight::from_parts(1_000_000, 32);
                MappingReadWeight::set(read);
                let cost = u16::try_from(pallet_evm::account_cost::gas::<Test>(read)).unwrap();
                let mut code = Vec::new();
                for _ in 0..attempts {
                    push_call(&mut code, cost - 1, 0);
                    code.push(0x50); // caller catches child failure
                }
                code.push(0);
                let (result, _, reads) = execute_parent(code, vec![0x60, 0, 0xff], false);
                assert!(result.is_succeed(), "{result:?}");
                // Each caller pays/reads target existence and the two transfer
                // mappings. Exhausted children must perform zero alias reads.
                assert_eq!(reads, attempts * 3);
                MappingReadWeight::set(Weight::zero());
            });
        }
    }

    #[test]
    fn value_transfer_with_only_the_2300_gas_stipend_still_succeeds() {
        sp_io::TestExternalities::default().execute_with(|| {
            MappingReadWeight::set(Weight::from_parts(1_000_000, 32));
            let mut code = Vec::new();
            push_call(&mut code, 0, 1_000_000_000); // one rao; child gets only stipend
            code.extend_from_slice(&[0x60, 0, 0x52, 0x60, 32, 0x60, 0, 0xf3]);
            let (result, output, _) = execute_parent(code, vec![0], true);
            assert!(result.is_succeed(), "{result:?}");
            assert_eq!(U256::from_big_endian(&output), U256::one());
            MappingReadWeight::set(Weight::zero());
        });
    }
}
