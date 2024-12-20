use pallet_evm::{
    ExitError, ExitSucceed, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};

use crate::precompiles::{get_method_id, get_slice};
use sp_std::vec;

use crate::{Runtime, RuntimeCall};
pub const SUBNETS_PRECOMPILE_INDEX: u64 = 2052;

// this is subnets smart contract's(0x0000000000000000000000000000000000000804) sr25519 address
pub const SUBNETS_CONTRACT_ADDRESS: &str = "5GKZiUUgTnWSz3BgiVBMehEKkLszsG4ZXnvgWpWFUFKqrqyn";

pub struct SubnetsPrecompile;

impl SubnetsPrecompile {
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
                hotkey: hotkey.into(),
            });
        Self::dispatch(handle, call)
    }

    fn parse_netuid_hotkey_parameter(data: &[u8]) -> Result<(u16, [u8; 32]), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut parameter = [0u8; 32];
        parameter.copy_from_slice(get_slice(data, 32, 64)?);

        Ok((netuid, parameter))
    }

    // will remove it after merge with other PR
    fn dispatch(_handle: &mut impl PrecompileHandle, _call: RuntimeCall) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: vec![],
        })
    }
}
