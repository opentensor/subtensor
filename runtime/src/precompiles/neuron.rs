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
            id if id == get_method_id("setWeights(uint16,uint16[],uint16[],uint64)") => {
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

        log::error!("++++++ set_weights call parse_netuid_dests_weights");

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
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1] first item
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128] first variable item
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]  second variable item
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45]  second item
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3] first length
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9]
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8]
    2025-01-14 17:08:40 index: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7]
         */

    fn parse_netuid_dests_weights(
        data: &[u8],
    ) -> Result<(u16, Vec<u16>, Vec<u16>, u64), PrecompileFailure> {
        if data.len() < 12 * 32 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut netuid_vec = [0u8; 2];
        netuid_vec.copy_from_slice(get_slice(data, 30, 32)?);
        let netuid = u16::from_be_bytes(netuid_vec);

        let mut first_position_vec = [0u8; 2];
        first_position_vec.copy_from_slice(get_slice(data, 62, 64)?);
        let first_position = u16::from_be_bytes(first_position_vec) as usize;

        let mut second_position_vec = [0u8; 2];
        second_position_vec.copy_from_slice(get_slice(data, 94, 96)?);
        let second_position = u16::from_be_bytes(second_position_vec) as usize;

        let mut version_key_vec = [0u8; 8];
        version_key_vec.copy_from_slice(get_slice(data, 120, 128)?);
        let version_key = u64::from_be_bytes(version_key_vec);

        let mut dests = vec![];
        let mut weights = vec![];

        let mut dests_len_vec = [0u8; 2];
        dests_len_vec.copy_from_slice(get_slice(data, first_position + 30, first_position + 32)?);
        let dests_len = u16::from_be_bytes(dests_len_vec) as usize;

        for i in 0..dests_len {
            let mut tmp_vec = [0u8; 2];
            tmp_vec.copy_from_slice(get_slice(
                data,
                first_position + 62 + i * 32,
                first_position + 64 + i * 32,
            )?);
            let dest = u16::from_be_bytes(tmp_vec);
            dests.push(dest);
        }

        let mut weights_len_vec = [0u8; 2];
        weights_len_vec.copy_from_slice(get_slice(
            data,
            second_position + 30,
            second_position + 32,
        )?);
        let weights_len = u16::from_be_bytes(weights_len_vec) as usize;

        log::error!("++++++ 4 {}", weights_len);

        for i in 0..weights_len {
            let mut tmp_vec = [0u8; 2];
            tmp_vec.copy_from_slice(get_slice(
                data,
                second_position + 62 + i * 32,
                second_position + 64 + i * 32,
            )?);
            let weight = u16::from_be_bytes(tmp_vec);
            weights.push(weight);
        }

        log::error!("{:?}", netuid);
        log::error!("{:?}", dests);
        log::error!("{:?}", weights);
        log::error!("{:?}", version_key);

        Ok((netuid, dests, weights, version_key))
    }
}
