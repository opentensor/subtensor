// The goal of staking precompile is to allow interaction between EVM users and smart contracts and
// subtensor staking functionality, namely add_stake, and remove_stake extrinsicsk, as well as the
// staking state.
//
// Additional requirement is to preserve compatibility with Ethereum indexers, which requires
// no balance transfers from EVM accounts without a corresponding transaction that can be
// parsed by an indexer.
//
// Implementation of add_stake:
//   - User transfers balance that will be staked to the precompile address with a payable
//     method addStake. This method also takes hotkey public key (bytes32) of the hotkey
//     that the stake should be assigned to.
//   - Precompile transfers the balance back to the signing address, and then invokes
//     do_add_stake from subtensor pallet with signing origin that mmatches to HashedAddressMapping
//     of the message sender, which will effectively withdraw and stake balance from the message
//     sender.
//   - Precompile checks the result of do_add_stake and, in case of a failure, reverts the transaction,
//     and leaves the balance on the message sender account.
//
// Implementation of remove_stake:
//   - User involkes removeStake method and specifies hotkey public key (bytes32) of the hotkey
//     to remove stake from, and the amount to unstake.
//   - Precompile calls do_remove_stake method of the subtensor pallet with the signing origin of message
//     sender, which effectively unstakes the specified amount and credits it to the message sender
//   - Precompile checks the result of do_remove_stake and, in case of a failure, reverts the transaction.
//

use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, BalanceConverter, HashedAddressMapping};
use pallet_evm::{
    ExitError, ExitSucceed, PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use sp_core::crypto::Ss58Codec;
use sp_core::U256;
use sp_runtime::traits::Dispatchable;
use sp_runtime::traits::{BlakeTwo256, UniqueSaturatedInto};
use sp_runtime::AccountId32;

use crate::precompiles::{get_method_id, get_slice};
use sp_std::vec;

use crate::{Runtime, RuntimeCall};
pub const STAKING_PRECOMPILE_INDEX: u64 = 2049;

pub struct StakingPrecompile;

impl StakingPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata
            .get(4..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

        match method_id {
            id if id == get_method_id("addStake(bytes32,uint16)") => {
                Self::add_stake(handle, &method_input)
            }
            id if id == get_method_id("removeStake(bytes32,uint256,uint16)") => {
                Self::remove_stake(handle, &method_input)
            }
            _ => Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            }),
        }
    }

    fn add_stake(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let hotkey = Self::parse_hotkey(data)?.into();
        let amount: U256 = handle.context().apparent_value;
        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        // Create the add_stake call
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::add_stake {
            hotkey,
            // TODO update contract to add netuid
            netuid: 1,
            amount_staked: amount_sub.unique_saturated_into(),
        });
        // Dispatch the add_stake call
        Self::dispatch(handle, call)
    }
    fn remove_stake(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let hotkey = Self::parse_hotkey(data)?.into();

        // We have to treat this as uint256 (because of Solidity ABI encoding rules, it pads uint64),
        // but this will never exceed 8 bytes, se we will ignore higher bytes and will only use lower
        // 8 bytes.
        let amount = data
            .get(56..64)
            .map(U256::from_big_endian)
            .ok_or(ExitError::OutOfFund)?;
        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::remove_stake {
            hotkey,
            // TODO update contract to add netuid
            netuid: 1,
            amount_unstaked: amount_sub.unique_saturated_into(),
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
        hotkey.copy_from_slice(get_slice(data, 0, 32)?);
        Ok(hotkey)
    }

    fn dispatch(handle: &mut impl PrecompileHandle, call: RuntimeCall) -> PrecompileResult {
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        // Transfer the amount back to the caller before executing the staking operation
        // let caller = handle.context().caller;
        let amount = handle.context().apparent_value;

        if !amount.is_zero() {
            Self::transfer_back_to_caller(&account_id, amount)?;
        }

        let result = call.dispatch(RawOrigin::Signed(account_id.clone()).into());
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

    fn transfer_back_to_caller(
        account_id: &AccountId32,
        amount: U256,
    ) -> Result<(), PrecompileFailure> {
        // this is staking smart contract's(0x0000000000000000000000000000000000000801) sr25519 address
        let smart_contract_account_id =
            match AccountId32::from_ss58check("5CwnBK9Ack1mhznmCnwiibCNQc174pYQVktYW3ayRpLm4K2X") {
                Ok(addr) => addr,
                Err(_) => {
                    return Err(PrecompileFailure::Error {
                        exit_status: ExitError::Other("Invalid SS58 address".into()),
                    });
                }
            };
        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        // Create a transfer call from the smart contract to the caller
        let transfer_call =
            RuntimeCall::Balances(pallet_balances::Call::<Runtime>::transfer_allow_death {
                dest: account_id.clone().into(),
                value: amount_sub.unique_saturated_into(),
            });

        // Execute the transfer
        let transfer_result =
            transfer_call.dispatch(RawOrigin::Signed(smart_contract_account_id).into());

        if let Err(dispatch_error) = transfer_result {
            log::error!(
                "Transfer back to caller failed. Error: {:?}",
                dispatch_error
            );
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::Other("Transfer back to caller failed".into()),
            });
        }

        Ok(())
    }
}
