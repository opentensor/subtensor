extern crate alloc;

use alloc::vec::Vec;

use crate::precompiles::{get_method_id, get_slice};
use crate::{Runtime, RuntimeCall};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use fp_evm::{
    ExitError, ExitSucceed, LinearCostPrecompile, PrecompileFailure, PrecompileHandle,
    PrecompileOutput, PrecompileResult,
};
use sp_core::U256;
use sp_std::vec;
pub const METAGRAPH_PRECOMPILE_INDEX: u64 = 2050;
pub struct MetagraphPrecompile;

/*
get_uid_count	SubnetworkN
get_stake	Total stake of the neuron in Tao
get_rank	Rank score of the neuron
get_trust	Trust score assigned to the neuron by other neurons
get_consensus	Consensus score of the neuron
get_incentive	Incentive score representing the neuron's incentive alignment
get_dividends	Dividends earned by the neuron
get_emission	Emission received by the neuron (with 18 decimals)
get_vtrust	Validator trust score indicating the network's trust in the neuron as a validator
get_validator_status	Validator status of the neuron
get_last_updated	Number of blocks since the neuron's last update
get_is_active	Activity status of the neuron
get_axon	Network endpoint information of the neuron
get_hotkey	Hotkey (public key as bytes32) of the neuron
get_coldkey	Coldkey (public key as bytes32) of the neuron
 */

impl MetagraphPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        log::error!("++++++ execute metagraph");
        let txdata = handle.input();
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata
            .get(4..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

        match method_id {
            id if id == get_method_id("getUidCount(uint16)") => Self::get_uid_count(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_stake(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_rank(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_trust(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_consensus(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_emission(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_vtrust(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => {
                Self::get_validator_status(&method_input)
            }
            id if id == get_method_id("getUidCount(uint16)") => {
                Self::get_last_updated(&method_input)
            }
            id if id == get_method_id("getUidCount(uint16)") => Self::get_is_active(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_axon(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_hotkey(&method_input),
            id if id == get_method_id("getUidCount(uint16)") => Self::get_coldkey(&method_input),

            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            }),
        }
    }

    fn get_uid_count(data: &[u8]) -> PrecompileResult {
        if data.len() < 2 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut netuid = [0u8; 2];
        netuid.copy_from_slice(get_slice(data, 0, 2)?);
        let netuid = u16::from_be_bytes(netuid);

        log::error!("++++++ netuid is {:?}", netuid);

        let uid_count = pallet_subtensor::SubnetworkN::<Runtime>::get(netuid);

        let uid_count_u256 = U256::from(uid_count);
        let mut result = [0_u8; 32];
        U256::to_big_endian(&uid_count_u256, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn get_stake(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_rank(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_trust(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_consensus(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_incentive(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_dividends(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_emission(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_vtrust(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_validator_status(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_last_updated(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_is_active(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_axon(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_hotkey(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }

    fn get_coldkey(data: &[u8]) -> PrecompileResult {
        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: [].into(),
        })
    }
}
