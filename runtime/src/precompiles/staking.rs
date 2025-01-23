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
use pallet_evm::{
    AddressMapping, BalanceConverter, ExitError, ExitSucceed, HashedAddressMapping,
    PrecompileFailure, PrecompileHandle, PrecompileOutput, PrecompileResult,
};
use precompile_utils::prelude::RuntimeHelper;
use sp_core::crypto::Ss58Codec;
use sp_core::U256;
use sp_runtime::traits::{BlakeTwo256, Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_runtime::AccountId32;
use sp_std::vec;

use crate::{
    precompiles::{get_method_id, get_slice},
    ProxyType, Runtime, RuntimeCall,
};

pub const STAKING_PRECOMPILE_INDEX: u64 = 2049;

pub struct StakingPrecompile;

impl StakingPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
        let txdata = handle.input();
        let method_id = get_slice(txdata, 0, 4)?;
        let method_input = txdata
            .get(4..)
            .map_or_else(vec::Vec::new, |slice| slice.to_vec()); // Avoiding borrowing conflicts

        if method_id == get_method_id("addStake(bytes32,uint256)") {
            Self::add_stake(handle, &method_input)
        } else if method_id == get_method_id("removeStake(bytes32,uint256,uint256)") {
            Self::remove_stake(handle, &method_input)
        } else if method_id == get_method_id("getStake(bytes32,bytes32,uint256)") {
            Self::get_stake(&method_input)
        } else if method_id == get_method_id("addProxy(bytes32)") {
            Self::add_proxy(handle, &method_input)
        } else if method_id == get_method_id("removeProxy(bytes32)") {
            Self::remove_proxy(handle, &method_input)
        } else {
            Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            })
        }
    }

    fn add_stake(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let hotkey = Self::parse_pub_key(data)?.into();
        let amount: U256 = handle.context().apparent_value;
        let netuid = Self::parse_netuid(data, 0x3E)?;

        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        // Create the add_stake call
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::add_stake {
            hotkey,
            netuid,
            amount: amount_sub.unique_saturated_into(),
        });
        // Dispatch the add_stake call
        Self::dispatch(handle, call)
    }

    fn remove_stake(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let hotkey = Self::parse_pub_key(data)?.into();
        let netuid = Self::parse_netuid(data, 0x5E)?;

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
            netuid,
            amount_unstaked: amount_sub.unique_saturated_into(),
        });
        Self::dispatch(handle, call)
    }

    fn add_proxy(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let delegate = AccountId32::from(Self::parse_pub_key(data)?);
        let delegate = <Runtime as frame_system::Config>::Lookup::unlookup(delegate);
        let call = RuntimeCall::Proxy(pallet_proxy::Call::<Runtime>::add_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0,
        });

        Self::dispatch(handle, call)
    }

    fn remove_proxy(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let delegate = AccountId32::from(Self::parse_pub_key(data)?);
        let delegate = <Runtime as frame_system::Config>::Lookup::unlookup(delegate);
        let call = RuntimeCall::Proxy(pallet_proxy::Call::<Runtime>::remove_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0,
        });

        Self::dispatch(handle, call)
    }

    fn get_stake(data: &[u8]) -> PrecompileResult {
        let (hotkey, coldkey) = Self::parse_hotkey_coldkey(data)?;
        let netuid = Self::parse_netuid(data, 0x5E)?;

        let stake = pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey.into(),
            &coldkey.into(),
            netuid,
        );

        // Convert to EVM decimals
        let stake_u256 = U256::from(stake);
        let stake_eth =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake_u256)
                .ok_or(ExitError::InvalidRange)?;

        // Format output
        let mut result = [0_u8; 32];
        U256::to_big_endian(&stake_eth, &mut result);

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            output: result.into(),
        })
    }

    fn parse_hotkey_coldkey(data: &[u8]) -> Result<([u8; 32], [u8; 32]), PrecompileFailure> {
        if data.len() < 64 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut hotkey = [0u8; 32];
        hotkey.copy_from_slice(get_slice(data, 0, 32)?);
        let mut coldkey = [0u8; 32];
        coldkey.copy_from_slice(get_slice(data, 32, 64)?);
        Ok((hotkey, coldkey))
    }

    fn parse_pub_key(data: &[u8]) -> Result<[u8; 32], PrecompileFailure> {
        if data.len() < 32 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }
        let mut pubkey = [0u8; 32];
        pubkey.copy_from_slice(get_slice(data, 0, 32)?);
        Ok(pubkey)
    }

    fn parse_netuid(data: &[u8], offset: usize) -> Result<u16, PrecompileFailure> {
        if data.len() < offset + 2 {
            return Err(PrecompileFailure::Error {
                exit_status: ExitError::InvalidRange,
            });
        }

        let mut netuid_bytes = [0u8; 2];
        netuid_bytes.copy_from_slice(get_slice(data, offset, offset + 2)?);
        let netuid: u16 = netuid_bytes[1] as u16 | ((netuid_bytes[0] as u16) << 8u16);

        Ok(netuid)
    }

    fn dispatch(handle: &mut impl PrecompileHandle, call: RuntimeCall) -> PrecompileResult {
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        // Transfer the amount back to the caller before executing the staking operation
        let amount = handle.context().apparent_value;

        if !amount.is_zero() {
            Self::transfer_back_to_caller(&account_id, amount)?;
        }

        match RuntimeHelper::<Runtime>::try_dispatch(
            handle,
            RawOrigin::Signed(account_id.clone()).into(),
            call,
        ) {
            Ok(post_info) => {
                log::info!("Dispatch succeeded. Post info: {:?}", post_info);

                Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output: vec![],
                })
            }

            Err(dispatch_error) => {
                log::error!("Dispatch failed. Error: {:?}", dispatch_error);
                log::warn!("Returning error PrecompileFailure::Error");
                Err(PrecompileFailure::Error {
                    exit_status: ExitError::Other("Subtensor call failed".into()),
                })
            }
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
