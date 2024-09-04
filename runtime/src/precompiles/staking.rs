use frame_system::RawOrigin;
use pallet_evm::{
    ExitError, ExitSucceed, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use pallet_evm::{HashedAddressMapping,AddressMapping};
use sp_core::U256;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Dispatchable;
use sp_runtime::AccountId32;
use sp_std::vec;
use crate::precompiles::{bytes_to_account_id, get_method_id, get_slice};

use crate::{Runtime, RuntimeCall};
pub const STAKING_PRECOMPILE_INDEX: u64 = 2049;

pub struct StakingPrecompile;

impl StakingPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata[4..].to_vec(); // Avoiding borrowing conflicts

        match method_id {
            id if id == get_method_id("addStake(bytes32)") => Self::add_stake(handle, &method_input),
            id if id == get_method_id("removeStake(bytes32,uint64)") => Self::remove_stake(handle, &method_input),
            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            })
        }
    }

    fn add_stake(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let hotkey = Self::parse_hotkey(data)?.into();
        let amount: U256 = handle.context().apparent_value;
        // Create the add_stake call
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::add_stake {
            hotkey,
            amount_staked: amount.as_u64(),
        });
        // Dispatch the add_stake call
        Self::dispatch(handle, call)
    }
    fn remove_stake(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let hotkey = Self::parse_hotkey(data)?.into();
        let amount = U256::from_big_endian(&data[32..40]).as_u64(); // Assuming the next 8 bytes represent the amount
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::remove_stake {
            hotkey,
            amount_unstaked: amount,
        });
        Self::dispatch(handle, call)
    }

    fn parse_hotkey(data: &[u8]) -> Result<[u8; 32], PrecompileFailure> {
        if data.len() < 32 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut hotkey = [0u8; 32];
        hotkey.copy_from_slice(&data[0..32]);
        Ok(hotkey)
    }

    fn dispatch(handle: &mut impl PrecompileHandle, call: RuntimeCall) -> PrecompileResult {
        let account_id = <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(handle.context().caller);
        let result = call.dispatch(RawOrigin::Signed(account_id).into());
        match &result {
            Ok(post_info) => log::info!("Dispatch succeeded. Post info: {:?}", post_info),
            Err(dispatch_error) => log::error!("Dispatch failed. Error: {:?}", dispatch_error),
        }
        match result {
            Ok(_) => Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output: vec![],
            }),
            Err(_) => Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Subtensor call failed".into()),
            }),
        }
    }
}
