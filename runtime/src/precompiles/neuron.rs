use frame_support::weights;
use pallet_evm::{
    ExitError, ExitSucceed, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};

use crate::precompiles::{get_method_id, get_slice};
use sp_std::{vec, vec::Vec};

use crate::{Runtime, RuntimeCall};
pub const NEURON_PRECOMPILE_INDEX: u64 = 2053;

// this is subnets smart contract's(0x0000000000000000000000000000000000000805) sr25519 address
pub const NEURON_CONTRACT_ADDRESS: &str = "5GKZiUUgTnWSz3BgiVBMehEKkLszsG4ZXnvgWpWFUFKqrqyn";

pub struct NeuronPrecompile;

impl NeuronPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata
            .get(4..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

        match method_id {
            id if id == get_method_id("setWeights(uint16,bytes,bytes,uint64)") => {
                Self::set_weights(handle, &method_input)
            }

            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            }),
        }
    }

    pub fn set_weights(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        log::error!("++++++ set_weights {:?}", data);
        let len = data.len();
        let mut index = 0;
        while index < len / 32 {
            let tmp = get_slice(data, index * 32, index * 32 + 32).unwrap();
            log::error!("index: {:?}", tmp);
            index += 1;
        }

        let (netuid, dests, weights, version_key) = Self::parse_netuid_dests_weights(data)?;
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::set_weights {
            netuid,
            dests,
            weights,
            version_key,
        });

        // dispatch(handle, call, NEURON_CONTRACT_ADDRESS)
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: vec![],
        })
    }

    // pub fn commit_weights(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
    //     let (netuid, hotkey) = Self::parse_netuid_hotkey_parameter(data)?;
    //     let call =
    //         RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::burned_register {
    //             netuid,
    //             hotkey: hotkey.into(),
    //         });
    //     dispatch(handle, call, SUBNETS_CONTRACT_ADDRESS)
    // }

    // pub fn reveal_weights(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
    //     let (netuid, hotkey) = Self::parse_netuid_hotkey_parameter(data)?;
    //     let call =
    //         RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::burned_register {
    //             netuid,
    //             hotkey: hotkey.into(),
    //         });
    //     dispatch(handle, call, SUBNETS_CONTRACT_ADDRESS)
    // }

    /*
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128]  pos of first var
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 64]   pos of second var
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160] length of whole data
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32]  length of each part
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]   length of data
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8]
    2025-01-13 22:27:16 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7]
     */

    fn parse_netuid_dests_weights(
        data: &[u8],
    ) -> Result<(u16, Vec<u16>, Vec<u16>, u64), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        // let mut netuid_vec = [0u8; 2];
        // netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        // let netuid = u16::from_be_bytes(netuid_vec);

        // let mut parameter = [0u8; 32];
        // parameter.copy_from_slice(get_slice(data, 32, 64)?);

        let netuid = 1;
        let dests = vec![22, 33, 44];
        let weights = vec![55, 66, 77];
        let version_key = 8_u64;

        Ok((netuid, dests, weights, version_key))
    }
}
