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
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{BlakeTwo256, Dispatchable, StaticLookup, UniqueSaturatedInto};
use sp_runtime::AccountId32;
use sp_std::vec;

use crate::precompiles::{
    get_method_id, get_pubkey, get_slice, parse_netuid, try_dispatch_runtime_call,
    try_u16_from_u256,
};
use crate::{ProxyType, Runtime, RuntimeCall};

pub const STAKING_PRECOMPILE_INDEX: u64 = 2049;

// ss58 public key i.e., the contract sends funds it received to the destination address from the
// method parameter.
const CONTRACT_ADDRESS_SS58: [u8; 32] = [
    0x26, 0xf4, 0x10, 0x1e, 0x52, 0xb7, 0x57, 0x34, 0x33, 0x24, 0x5b, 0xc3, 0x0a, 0xe1, 0x8b, 0x63,
    0x99, 0x53, 0xd8, 0x41, 0x79, 0x33, 0x03, 0x61, 0x4d, 0xfa, 0xcf, 0xf0, 0x37, 0xf7, 0x12, 0x94,
];

pub struct StakingPrecompile;

#[precompile_utils::precompile]
impl StakingPrecompile {
    #[precompile::public("addStake(bytes32,uint256)")]
    #[precompile::payable]
    fn add_stake(handle: &mut impl PrecompileHandle, address: H256, netuid: U256) -> EvmResult<()> {
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        let amount = handle.context().apparent_value;

        if !amount.is_zero() {
            Self::transfer_back_to_caller(&account_id, amount)?;
        }

        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        let (hotkey, _) = get_pubkey(address.as_bytes())?;
        let netuid = try_u16_from_u256(netuid)?;
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::add_stake {
            hotkey,
            netuid,
            amount_staked: amount_sub.unique_saturated_into(),
        });

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStake(bytes32,uint256,uint256)")]
    fn remove_stake(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );

        let amount_sub =
            <Runtime as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        let (hotkey, _) = get_pubkey(address.as_bytes())?;
        let netuid = try_u16_from_u256(netuid)?;
        let call = RuntimeCall::SubtensorModule(pallet_subtensor::Call::<Runtime>::remove_stake {
            hotkey,
            netuid,
            amount_unstaked: amount_sub.unique_saturated_into(),
        });

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("addProxy(bytes32)")]
    fn add_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );
        let (delegate, _) = get_pubkey(delegate.as_bytes())?;
        let delegate = <Runtime as frame_system::Config>::Lookup::unlookup(delegate);
        let call = RuntimeCall::Proxy(pallet_proxy::Call::<Runtime>::add_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0,
        });

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeProxy(bytes32)")]
    fn remove_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id =
            <HashedAddressMapping<BlakeTwo256> as AddressMapping<AccountId32>>::into_account_id(
                handle.context().caller,
            );
        let (delegate, _) = get_pubkey(delegate.as_bytes())?;
        let delegate = <Runtime as frame_system::Config>::Lookup::unlookup(delegate);
        let call = RuntimeCall::Proxy(pallet_proxy::Call::<Runtime>::remove_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0,
        });

        try_dispatch_runtime_call(handle, call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("getStake(bytes32,bytes32,uint256)")]
    #[precompile::view]
    fn get_stake(
        _: &mut impl PrecompileHandle,
        hotkey: H256,
        coldkey: H256,
        netuid: U256,
    ) -> EvmResult<U256> {
        let (hotkey, _) = get_pubkey(hotkey.as_bytes())?;
        let (coldkey, _) = get_pubkey(coldkey.as_bytes())?;
        let netuid = try_u16_from_u256(netuid)?;

        let stake = pallet_subtensor::Pallet::<Runtime>::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey, &coldkey, netuid,
        );

        // Convert to EVM decimals
        <Runtime as pallet_evm::Config>::BalanceConverter::into_evm_balance(stake.into())
            .ok_or(ExitError::InvalidRange.into())
    }

    fn transfer_back_to_caller(
        account_id: &AccountId32,
        amount: U256,
    ) -> Result<(), PrecompileFailure> {
        let smart_contract_account_id: AccountId32 = CONTRACT_ADDRESS_SS58.into();

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
