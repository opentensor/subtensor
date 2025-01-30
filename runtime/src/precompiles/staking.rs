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

use crate::precompiles::{
    contract_to_origin, get_method_id, get_pubkey, get_slice, parse_netuid,
    try_dispatch_runtime_call,
};
use crate::{ProxyType, Runtime, RuntimeCall};
use pallet_evm::{
    BalanceConverter, ExitError, ExitSucceed, PrecompileFailure, PrecompileHandle,
    PrecompileOutput, PrecompileResult,
};
use sp_core::U256;
use sp_runtime::traits::{StaticLookup, UniqueSaturatedInto};
use sp_std::vec;

pub const STAKING_PRECOMPILE_INDEX: u64 = 2049;

// ss58 public key i.e., the contract sends funds it received to the destination address from the
// method parameter.
const CONTRACT_ADDRESS_SS58: [u8; 32] = [
    0x26, 0xf4, 0x10, 0x1e, 0x52, 0xb7, 0x57, 0x34, 0x33, 0x24, 0x5b, 0xc3, 0x0a, 0xe1, 0x8b, 0x63,
    0x99, 0x53, 0xd8, 0x41, 0x79, 0x33, 0x03, 0x61, 0x4d, 0xfa, 0xcf, 0xf0, 0x37, 0xf7, 0x12, 0x94,
];

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
        let (hotkey, _) = get_pubkey(data)?;
        let amount: U256 = handle.context().apparent_value;
        let netuid = Self::parse_netuid(data, 0x3E)?;

        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        // let (account_id_src, _) = get_pubkey(&CONTRACT_ADDRESS_SS58)?;
        // Create the add_stake call
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::add_stake {
            hotkey,
            netuid,
            amount_staked: amount_sub.unique_saturated_into(),
        });
        // let origin = RawOrigin::Signed(account_id_src);
        // Dispatch the add_stake call
        try_dispatch_runtime_call(handle, call, contract_to_origin(&CONTRACT_ADDRESS_SS58)?)
    }

    fn remove_stake(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (hotkey, _) = get_pubkey(data)?;
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
        try_dispatch_runtime_call(handle, call, contract_to_origin(&CONTRACT_ADDRESS_SS58)?)
    }

    fn add_proxy(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (delegate, _) = get_pubkey(data)?;
        let delegate = <Runtime as frame_system::Config>::Lookup::unlookup(delegate);
        let call = RuntimeCall::Proxy(pallet_proxy::Call::<Runtime>::add_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0,
        });

        try_dispatch_runtime_call(handle, call, contract_to_origin(&CONTRACT_ADDRESS_SS58)?)
    }

    fn remove_proxy(handle: &mut impl PrecompileHandle, data: &[u8]) -> PrecompileResult {
        let (delegate, _) = get_pubkey(data)?;
        let delegate = <Runtime as frame_system::Config>::Lookup::unlookup(delegate);
        let call = RuntimeCall::Proxy(pallet_proxy::Call::<Runtime>::remove_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0,
        });

        try_dispatch_runtime_call(handle, call, contract_to_origin(&CONTRACT_ADDRESS_SS58)?)
    }

    fn get_stake(data: &[u8]) -> PrecompileResult {
        let (hotkey, left_data) = get_pubkey(data)?;
        let (coldkey, _) = get_pubkey(&left_data)?;
        let netuid = parse_netuid(data, 0x5E)?;

        let stake = pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey, &coldkey, netuid,
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
}
