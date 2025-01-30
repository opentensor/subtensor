use pallet_evm::{ExitError, PrecompileFailure, PrecompileHandle, PrecompileResult};

use crate::precompiles::{
    contract_to_origin, get_method_id, get_pubkey, get_slice, parse_netuid,
    try_dispatch_runtime_call,
};
use sp_runtime::AccountId32;
use sp_std::vec;

use crate::{Runtime, RuntimeCall};
pub const NEURON_PRECOMPILE_INDEX: u64 = 2052;

// ss58 public key i.e., the contract sends funds it received to the destination address from the
// method parameter.
const CONTRACT_ADDRESS_SS58: [u8; 32] = [
    0xbc, 0x46, 0x35, 0x79, 0xbc, 0x99, 0xf9, 0xee, 0x7c, 0x59, 0xed, 0xee, 0x20, 0x61, 0xa3, 0x09,
    0xd2, 0x1e, 0x68, 0xd5, 0x39, 0xb6, 0x40, 0xec, 0x66, 0x46, 0x90, 0x30, 0xab, 0x74, 0xc1, 0xdb,
];
pub struct NeuronPrecompile;

impl NeuronPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata
            .get(4..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

        match method_id {
            id if id == get_method_id("burnedRegister(uint16,bytes32)") => {
                Self::burned_register(handle, &method_input)
            }

            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            }),
        }
    }

    pub fn burned_register(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (netuid, hotkey) = Self::parse_netuid_hotkey_parameter(data)?;
        let call =
            RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::burned_register {
                netuid,
                hotkey,
            });
        try_dispatch_runtime_call(handle, call, contract_to_origin(&CONTRACT_ADDRESS_SS58)?)
    }

    fn parse_netuid_hotkey_parameter(data: &[u8]) -> Result<(u16, AccountId32), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let netuid = parse_netuid(data, 30)?;

        let (hotkey, _) = get_pubkey(get_slice(data, 32, 64)?)?;

        Ok((netuid, hotkey))
    }
}
