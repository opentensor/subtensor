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

use core::marker::PhantomData;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_system::RawOrigin;
use pallet_evm::{
    AddressMapping, BalanceConverter, ExitError, PrecompileFailure, PrecompileHandle,
};
use precompile_utils::EvmResult;
use sp_core::{H256, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup, UniqueSaturatedInto};
use subtensor_runtime_common::ProxyType;

use crate::parser::{parse_pubkey, try_u16_from_u256};
use crate::{PrecompileExt, PrecompileHandleExt};

pub(crate) struct StakingPrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt for StakingPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_balances::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    const INDEX: u64 = 2049;
    const ADDRESS_SS58: [u8; 32] = [
        0x26, 0xf4, 0x10, 0x1e, 0x52, 0xb7, 0x57, 0x34, 0x33, 0x24, 0x5b, 0xc3, 0x0a, 0xe1, 0x8b,
        0x63, 0x99, 0x53, 0xd8, 0x41, 0x79, 0x33, 0x03, 0x61, 0x4d, 0xfa, 0xcf, 0xf0, 0x37, 0xf7,
        0x12, 0x94,
    ];
}

#[precompile_utils::precompile]
impl<R> StakingPrecompile<R>
where
    R: frame_system::Config
        + pallet_evm::Config
        + pallet_subtensor::Config
        + pallet_proxy::Config<ProxyType = ProxyType>
        + pallet_balances::Config,
    R::AccountId: From<[u8; 32]>,
    <R as frame_system::Config>::RuntimeCall: From<pallet_subtensor::Call<R>>
        + From<pallet_proxy::Call<R>>
        + From<pallet_balances::Call<R>>
        + GetDispatchInfo
        + Dispatchable<PostInfo = PostDispatchInfo>,
    <R as pallet_evm::Config>::AddressMapping: AddressMapping<R::AccountId>,
    <R as pallet_balances::Config>::Balance: TryFrom<U256>,
    <<R as frame_system::Config>::Lookup as StaticLookup>::Source: From<R::AccountId>,
{
    #[precompile::public("addStake(bytes32,uint256)")]
    #[precompile::payable]
    fn add_stake(handle: &mut impl PrecompileHandle, address: H256, netuid: U256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let amount = handle.context().apparent_value;

        if !amount.is_zero() {
            Self::transfer_back_to_caller(&account_id, amount)?;
        }

        let amount_sub = handle.try_convert_apparent_value::<R>()?;
        let (hotkey, _) = parse_pubkey(address.as_bytes())?;
        let netuid = try_u16_from_u256(netuid)?;
        let call = pallet_subtensor::Call::<R>::add_stake {
            hotkey,
            netuid,
            amount_staked: amount_sub.unique_saturated_into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeStake(bytes32,uint256,uint256)")]
    fn remove_stake(
        handle: &mut impl PrecompileHandle,
        address: H256,
        amount: U256,
        netuid: U256,
    ) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let (hotkey, _) = parse_pubkey(address.as_bytes())?;
        let netuid = try_u16_from_u256(netuid)?;
        let amount_unstaked = amount.unique_saturated_into();
        let call = pallet_subtensor::Call::<R>::remove_stake {
            hotkey,
            netuid,
            amount_unstaked,
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("addProxy(bytes32)")]
    fn add_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let (delegate, _) = parse_pubkey(delegate.as_bytes())?;
        let delegate = <R as frame_system::Config>::Lookup::unlookup(delegate);
        let call = pallet_proxy::Call::<R>::add_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0u32.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("removeProxy(bytes32)")]
    fn remove_proxy(handle: &mut impl PrecompileHandle, delegate: H256) -> EvmResult<()> {
        let account_id = handle.caller_account_id::<R>();
        let (delegate, _) = parse_pubkey(delegate.as_bytes())?;
        let delegate = <R as frame_system::Config>::Lookup::unlookup(delegate);
        let call = pallet_proxy::Call::<R>::remove_proxy {
            delegate,
            proxy_type: ProxyType::Staking,
            delay: 0u32.into(),
        };

        handle.try_dispatch_runtime_call::<R, _>(call, RawOrigin::Signed(account_id))
    }

    #[precompile::public("getStake(bytes32,bytes32,uint256)")]
    #[precompile::view]
    fn get_stake(
        _: &mut impl PrecompileHandle,
        hotkey: H256,
        coldkey: H256,
        netuid: U256,
    ) -> EvmResult<U256> {
        let (hotkey, _) = parse_pubkey(hotkey.as_bytes())?;
        let (coldkey, _) = parse_pubkey(coldkey.as_bytes())?;
        let netuid = try_u16_from_u256(netuid)?;
        let stake = pallet_subtensor::Pallet::<R>::get_stake_for_hotkey_and_coldkey_on_subnet(
            &hotkey, &coldkey, netuid,
        );

        Ok(stake.into())
    }

    fn transfer_back_to_caller(
        account_id: &<R as frame_system::Config>::AccountId,
        amount: U256,
    ) -> Result<(), PrecompileFailure> {
        let smart_contract_account_id = R::AccountId::from(Self::ADDRESS_SS58);
        let amount_sub =
            <R as pallet_evm::Config>::BalanceConverter::into_substrate_balance(amount)
                .ok_or(ExitError::OutOfFund)?;

        // Create a transfer call from the smart contract to the caller
        let transfer_call = <R as frame_system::Config>::RuntimeCall::from(
            pallet_balances::Call::<R>::transfer_allow_death {
                dest: account_id.clone().into(),
                value: amount_sub.unique_saturated_into(),
            },
        );

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
